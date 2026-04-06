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

/// Result of a successful mtime-based file reuse check.
struct MtimeReuseResult {
    functions: Vec<FunctionEntry>,
    checksum: String,
    coverage_off: bool,
}

/// Check if a file can be reused based on mtime (no read or SHA256 needed).
///
/// Returns Some if the file's mtime is older than the index built_at timestamp
/// and its checksum exists in the existing manifest. Returns None otherwise,
/// signaling the caller must fall back to content-based SHA256 comparison.
fn check_mtime_reuse(
    path: &Path,
    relative_path: &str,
    index_built_at: &Option<std::time::SystemTime>,
    existing: &AgentContextIndex,
) -> Option<MtimeReuseResult> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let built_at = index_built_at.as_ref()?;
    let mtime = fs::metadata(path).ok()?.modified().ok()?;
    if mtime >= *built_at {
        return None;
    }
    let checksum = existing.manifest.file_checksums.get(relative_path)?.clone();
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
    let secs = dt.timestamp();
    if secs < 0 {
        return None;
    }
    Some(std::time::UNIX_EPOCH + std::time::Duration::from_secs(secs as u64))
}

/// Extract the project prefix from a file path (everything before the first `/`).
///
/// For workspace-merged paths like `aprender/src/lib.rs`, returns `aprender`.
/// For local paths like `src/lib.rs`, returns `src`.
fn project_prefix(path: &str) -> &str {
    path.split('/').next().unwrap_or(path)
}
