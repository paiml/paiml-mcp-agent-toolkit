/// Load function source from SQLite by file path and start line.
fn load_source_from_sqlite(db_path: &Path, file_path: &str, start_line: usize) -> Option<String> {
    let conn = super::sqlite_backend::open_db(db_path).ok()?;
    let src = super::sqlite_backend::load_source_by_location(&conn, file_path, start_line).ok()?;
    if src.is_empty() {
        None
    } else {
        Some(src)
    }
}

/// Load function source from filesystem using line range.
fn load_source_from_file(file_path: &str, start_line: usize, end_line: usize) -> Option<String> {
    if end_line == 0 || start_line == 0 {
        return None;
    }
    let content = std::fs::read_to_string(file_path).ok()?;
    let lines: Vec<&str> = content.lines().collect();
    let start = start_line.saturating_sub(1);
    let end = end_line.min(lines.len());
    if start < end {
        Some(lines[start..end].join("\n"))
    } else {
        None
    }
}

/// How many slices the rotating re-verification sweep is spread over: each
/// incremental run re-hashes one 64th of the tree the fast path would have
/// skipped, so every file is re-verified at least once per 64 incremental runs.
pub(super) const VERIFY_SLICE_DIVISOR: u64 = 64;

/// FNV-1a (64-bit) over the path's bytes.
///
/// Written out rather than reached for in `std`: `DefaultHasher` is explicitly
/// not guaranteed stable across Rust releases, and a hash that changes under
/// the compiler would reshuffle which files each slice covers on every
/// toolchain bump — the one property the sweep needs is that a given path
/// always lands in the same slice.
pub(super) fn path_slice_hash(relative_path: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in relative_path.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// Is this file in the slice this run re-verifies?
///
/// The stat fast path believes the index's own recorded checksum. Nothing in
/// `len` or `ctime` can contradict a digest that was laundered — written into
/// the manifest for content that is not on disk — so a file whose stats agree
/// would be served from that digest forever. Re-reading a rotating 1/64 of the
/// skippable files bounds "forever" at 64 incremental runs, for 1/64 of the
/// read cost per run.
pub(super) fn in_verify_slice(relative_path: &str, run_counter: u64) -> bool {
    path_slice_hash(relative_path) % VERIFY_SLICE_DIVISOR == run_counter % VERIFY_SLICE_DIVISOR
}

/// Result of a successful mtime-based file reuse check.
struct MtimeReuseResult {
    functions: Vec<FunctionEntry>,
    checksum: FileRecord,
    coverage_off: bool,
}

/// The stat fields the fast path rests on: length, and (unix) ctime in
/// NANOSECONDS since the epoch.
///
/// Nanoseconds, not seconds: `built_at` and a file touched moments before it
/// routinely land in the same second, and a whole-second ctime cannot be
/// ordered against a build in that second — the fast path would refuse
/// forever, because the refusal itself is what would have to be recorded.
///
/// On non-unix targets ctime is reported as 0 ("unknown"), which
/// [`stats_agree`] treats as no evidence either way — the length check still
/// applies there.
pub(super) fn file_stat_fields(path: &Path) -> Option<(u64, i64)> {
    let md = fs::metadata(path).ok()?;
    #[cfg(unix)]
    let ctime = {
        use std::os::unix::fs::MetadataExt;
        md.ctime()
            .checked_mul(1_000_000_000)?
            .checked_add(md.ctime_nsec())?
    };
    #[cfg(not(unix))]
    let ctime = 0i64;
    Some((md.len(), ctime))
}

/// Build the record persisted for a file: checksum plus today's stat evidence.
pub(super) fn file_record(path: &Path, checksum: String) -> FileRecord {
    let (len, ctime) = file_stat_fields(path).unwrap_or((0, 0));
    FileRecord {
        checksum,
        len,
        ctime,
    }
}

/// Does what we see on disk match what the index recorded, well enough to skip
/// the read entirely?
///
/// A recorded length must match exactly, and on unix the recorded ctime must
/// predate the build: a rewrite advances ctime even when the writer backdates
/// mtime, so `ctime < built_at` is the evidence that no write happened since
/// the index was built. A record without stats (`has_stats() == false`) is a
/// pre-CRUX-07 entry and authorises nothing.
pub(super) fn stats_agree(record: &FileRecord, observed: (u64, i64), built_at_nanos: i64) -> bool {
    if !record.has_stats() {
        return false;
    }
    let (len, ctime) = observed;
    if len != record.len {
        return false;
    }
    // ctime == 0 means the platform gave us none; fall back to len + mtime.
    ctime == 0 || ctime < built_at_nanos
}

/// Check if a file can be reused based on mtime (no read or SHA256 needed).
///
/// Returns Some only when every cheap signal agrees the file is the one the
/// index read: mtime older than `built_at`, recorded length equal to the
/// length on disk, and (unix) an inode change time that also predates
/// `built_at`. mtime alone is not sufficient — it is writable by any process,
/// so a rewritten file whose mtime is backdated behind `built_at` used to be
/// served from the previous build's checksum forever (CRUX-07 leg a).
///
/// Returns None otherwise, signaling the caller must fall back to
/// content-based SHA256 comparison — including for every file in this run's
/// verification slice (see [`in_verify_slice`]), whose stat evidence is
/// deliberately not consulted.
fn check_mtime_reuse(
    path: &Path,
    relative_path: &str,
    index_built_at: &Option<std::time::SystemTime>,
    existing: &AgentContextIndex,
    run_counter: u64,
) -> Option<MtimeReuseResult> {
    // The rotating slice is checked FIRST: a file the sweep has selected must
    // be re-read whatever its stats say, because its stats are exactly what a
    // laundered checksum agrees with.
    if in_verify_slice(relative_path, run_counter) {
        return None;
    }
    let built_at = index_built_at.as_ref()?;
    let mtime = fs::metadata(path).ok()?.modified().ok()?;
    if mtime >= *built_at {
        return None;
    }
    let record = existing.manifest.file_checksums.get(relative_path)?;
    let built_at_nanos =
        i64::try_from(built_at.duration_since(std::time::UNIX_EPOCH).ok()?.as_nanos()).ok()?;
    let observed = file_stat_fields(path)?;
    if !stats_agree(record, observed, built_at_nanos) {
        return None;
    }
    let checksum = record.clone();
    let funcs = existing
        .file_index
        .get(relative_path)
        .map(|indices| {
            indices
                .iter()
                .map(|&idx| existing.functions[idx].clone())
                .collect()
        })
        .unwrap_or_default();
    let coverage_off = existing.coverage_off_files.contains(relative_path);
    Some(MtimeReuseResult {
        functions: funcs,
        checksum,
        coverage_off,
    })
}

/// Parse an RFC 3339 timestamp string into a SystemTime.
///
/// Returns None if the string can't be parsed (graceful fallback to SHA256-only path).
fn parse_built_at(built_at: &str) -> Option<std::time::SystemTime> {
    let dt = chrono::DateTime::parse_from_rfc3339(built_at).ok()?;
    let nanos = u64::try_from(dt.timestamp_nanos_opt()?).ok()?;
    // Sub-second precision is kept: truncating to whole seconds made a build
    // and a write in the same second indistinguishable, which is exactly the
    // window the fast path has to decide in.
    Some(std::time::UNIX_EPOCH + std::time::Duration::from_nanos(nanos))
}

/// Extract the project prefix from a file path (everything before the first `/`).
///
/// For workspace-merged paths like `aprender/src/lib.rs`, returns `aprender`.
/// For local paths like `src/lib.rs`, returns `src`.
fn project_prefix(path: &str) -> &str {
    path.split('/').next().unwrap_or(path)
}
