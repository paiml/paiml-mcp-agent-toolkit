//! PMAT-1363: `pmat roadmap aggregate` — `docs/roadmaps/roadmap.yaml` regenerated
//! from its base and `docs/roadmaps/entries/<id>.yaml`.
//!
//! The command surface of aprender's `scripts/lib/roadmap_fragments.py aggregate
//! [--write|--check]`, with the same exit contract: 0 ok, 1 a violation, 2 an input
//! this box cannot read. A check that cannot read its input never passes.
//!
//! `--check` is what a CI gate calls. It is two claims, and both are checked: the
//! committed roadmap IS the aggregate, and aggregating the aggregate changes nothing.
//! The second is not decoration — the aggregation runs post-merge on the default
//! branch, so a generator that is not idempotent churns a commit on every merge.

use std::path::{Path, PathBuf};

use crate::services::roadmap_fragments::{self as fragments, FragmentError};
use crate::services::roadmap_service::RoadmapService;
use crate::services::roadmap_write_lock::RoadmapWriteLock;

/// What `pmat roadmap aggregate` does with the aggregate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggregateMode {
    /// Print it to stdout (the default).
    Print,
    /// Write it over the roadmap, under the repository's roadmap lock.
    Write,
    /// Exit 1 unless the roadmap already is the aggregate and re-aggregating is a no-op.
    Check,
}

impl AggregateMode {
    /// The mode `--write` / `--check` select. clap refuses both at once.
    #[must_use]
    pub fn from_flags(write: bool, check: bool) -> Self {
        match (write, check) {
            (true, _) => Self::Write,
            (false, true) => Self::Check,
            (false, false) => Self::Print,
        }
    }
}

/// Everything one run produced: the exit code and both streams, returned rather
/// than printed so the whole contract is testable without a subprocess.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AggregateReport {
    /// 0 ok, 1 a violation, 2 an input that cannot be read.
    pub code: i32,
    /// The aggregate, in `Print` mode; empty otherwise.
    pub stdout: String,
    /// One line saying what happened.
    pub stderr: String,
}

impl AggregateReport {
    fn new(code: i32, stdout: String, stderr: String) -> Self {
        Self {
            code,
            stdout,
            stderr,
        }
    }
}

/// Run `pmat roadmap aggregate` over `roadmap` and `entries` (default: `entries/`
/// beside the roadmap). `--write` holds the exclusive repository lock; the other
/// modes hold the shared one, so neither reads a fragment mid-write.
#[must_use]
pub fn run_aggregate(
    roadmap: &Path,
    entries: Option<&Path>,
    mode: AggregateMode,
) -> AggregateReport {
    let entries_dir = entries.map_or_else(|| default_entries_dir(roadmap), Path::to_path_buf);
    let service = RoadmapService::new(roadmap);
    // PMAT-1385: the write arm is handed the lock token, so writing without the
    // exclusive lock does not type-check.
    let locked = match mode {
        AggregateMode::Write => service
            .with_write_lock(|lock| aggregate_locked(roadmap, &entries_dir, Access::Write(lock))),
        AggregateMode::Print => {
            service.with_read_lock(|| aggregate_locked(roadmap, &entries_dir, Access::Print))
        }
        AggregateMode::Check => {
            service.with_read_lock(|| aggregate_locked(roadmap, &entries_dir, Access::Check))
        }
    };
    locked.unwrap_or_else(|e| {
        AggregateReport::new(
            2,
            String::new(),
            format!(
                "FAIL cannot take the roadmap lock for {} ({e}) — this box cannot judge\n",
                roadmap.display()
            ),
        )
    })
}

/// `entries/` beside the roadmap — a caller that moves `--roadmap` moves both, so a
/// check over another tree never reads THIS checkout's fragments by accident.
fn default_entries_dir(roadmap: &Path) -> PathBuf {
    match roadmap.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.join("entries"),
        _ => PathBuf::from("entries"),
    }
}

/// What the locked section may do: only the write arm holds the write-lock token.
enum Access<'l> {
    Print,
    Write(&'l RoadmapWriteLock),
    Check,
}

fn aggregate_locked(roadmap: &Path, entries_dir: &Path, access: Access<'_>) -> AggregateReport {
    fragments::aggregate_paths(roadmap, entries_dir).map_or_else(refusal, |(base, out)| {
        emit(roadmap, entries_dir, access, &base, out)
    })
}

/// Exit 2 for an input that cannot be read — never a pass — and 1 for a violation.
fn refusal(e: FragmentError) -> AggregateReport {
    match e {
        FragmentError::Io { .. } => AggregateReport::new(
            2,
            String::new(),
            format!("FAIL {e} — an input that cannot be read is never a pass\n"),
        ),
        _ => AggregateReport::new(1, String::new(), format!("FAIL {e}\n")),
    }
}

/// The three terminal arms: print, write, or check.
fn emit(
    roadmap: &Path,
    entries_dir: &Path,
    access: Access<'_>,
    base: &str,
    out: String,
) -> AggregateReport {
    // Re-read rather than threading the list through: aggregate_paths already
    // proved it readable, and the count is only for the message.
    let fragments = fragments::read_fragments(entries_dir).unwrap_or_default();
    match access {
        Access::Print => AggregateReport::new(0, out, String::new()),
        Access::Write(lock) => write_aggregate(lock, roadmap, base, &out, fragments.len()),
        Access::Check => check_aggregate(roadmap, entries_dir, base, &out, &fragments),
    }
}

fn write_aggregate(
    lock: &RoadmapWriteLock,
    roadmap: &Path,
    base: &str,
    out: &str,
    fragment_count: usize,
) -> AggregateReport {
    let rows = fragments::split_entries(base).1.len();
    if out == base {
        return AggregateReport::new(
            0,
            String::new(),
            format!(
                "aggregate: {rows} base row(s) + {fragment_count} fragment(s) -> {} (unchanged)\n",
                roadmap.display()
            ),
        );
    }
    if let Err(e) = replace_atomically(lock, roadmap, out) {
        return AggregateReport::new(
            2,
            String::new(),
            format!("FAIL write {}: {e}\n", roadmap.display()),
        );
    }
    AggregateReport::new(
        0,
        String::new(),
        format!(
            "aggregate: {rows} base row(s) + {fragment_count} fragment(s) -> {}\n",
            roadmap.display()
        ),
    )
}

fn check_aggregate(
    roadmap: &Path,
    entries_dir: &Path,
    base: &str,
    out: &str,
    fragments: &[(String, String)],
) -> AggregateReport {
    match fragments::aggregate(out, fragments) {
        Ok(again) if again == out => {}
        _ => {
            return AggregateReport::new(
                1,
                String::new(),
                "FAIL aggregate is not idempotent on this input\n".to_string(),
            )
        }
    }
    if let Some(id) = fragments::first_difference(out, base) {
        return AggregateReport::new(
            1,
            String::new(),
            format!(
                "FAIL {} is not what the aggregator produces from {}/ — first differing row: {id}. \
                 Run `pmat roadmap aggregate --write` on the default branch; never edit it in a pull request.\n",
                roadmap.display(),
                entries_dir.display()
            ),
        );
    }
    AggregateReport::new(
        0,
        String::new(),
        format!(
            "ok  {} == aggregate({} fragment(s)), idempotent\n",
            roadmap.display(),
            fragments.len()
        ),
    )
}

/// Write `contents` beside `path` and rename it over `path`.
///
/// The repository lock keeps every pmat reader out while this runs, but git, an
/// editor, or the CI parity gate take no such lock: an in-place write would let
/// them read a truncated aggregate, and a crash mid-write would leave one on disk.
/// A rename within one directory is atomic, the same guarantee `write_fragment`
/// gives each fragment. PMAT-1385: [`RoadmapWriteLock::replace`] does both steps.
fn replace_atomically(lock: &RoadmapWriteLock, path: &Path, contents: &str) -> std::io::Result<()> {
    let name = path.file_name().map_or_else(
        || "roadmap.yaml".into(),
        |n| n.to_string_lossy().into_owned(),
    );
    lock.replace(
        &path.with_file_name(format!(".{name}.aggregate.tmp")),
        path,
        contents,
    )
}

/// `pmat roadmap aggregate` as the CLI runs it: print both streams, and exit with
/// the report's code when it is not 0.
///
/// # Errors
///
/// Never; a failure is the process exit code, which is the command's contract.
pub fn execute(
    write: bool,
    check: bool,
    roadmap: &Path,
    entries: Option<&Path>,
) -> anyhow::Result<()> {
    let report = run_aggregate(roadmap, entries, AggregateMode::from_flags(write, check));
    print!("{}", report.stdout);
    eprint!("{}", report.stderr);
    if report.code != 0 {
        std::process::exit(report.code);
    }
    Ok(())
}
