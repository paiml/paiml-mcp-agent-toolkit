//! PMAT #1370: `pmat roadmap sync` is the ONE writer of `docs/roadmaps/roadmap.yaml`.
//!
//! A repository opts in to the per-ticket store by having `docs/roadmaps/entries/`,
//! the predicate paiml/.github's `roadmap-fragment-parity` gate and every
//! `RoadmapService` writer already use. In such a repository `pmat work
//! add|edit|start|complete` write only `entries/<id>.yaml`, and `sync` regenerates
//! the aggregate from them — on the default branch, never in a pull request:
//!
//! | invocation                    | fragment mode (`entries/` exists)        | otherwise (MACS-013)                 |
//! |-------------------------------|------------------------------------------|--------------------------------------|
//! | `pmat roadmap sync`           | write `roadmap.yaml` = aggregate, locked | write `ROADMAP.yaml`                 |
//! | `pmat roadmap sync --dry-run` | print the aggregate                      | print the render                     |
//! | `pmat roadmap sync --check`   | exit 1 on drift, naming the first row    | exit 1 when `ROADMAP.yaml` is stale  |
//!
//! `--check` exit contract, both modes: 0 in parity, 1 drift, 2 an input that
//! cannot be read — never a pass. `pmat roadmap aggregate` stays as the same engine
//! under its PMAT-1363 name.

use std::path::{Path, PathBuf};

use super::aggregate::{run_aggregate, AggregateMode, AggregateReport};
use super::sync::{handle_roadmap_sync, read_work_store_rows, render_roadmap, RoadmapSources};

/// The aggregate `sync` writes in fragment mode, relative to the project root.
pub const FRAGMENT_ROADMAP: &str = "docs/roadmaps/roadmap.yaml";

/// Which store `sync` renders from, decided by what the project has on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncTarget {
    /// `docs/roadmaps/entries/` exists: aggregate the fragments into this roadmap.
    Fragments(PathBuf),
    /// No `entries/`: the MACS-013 render of the roadmap into `ROADMAP.yaml`.
    WorkStore,
}

/// The target for `project_path`.
#[must_use]
pub fn sync_target(project_path: &Path) -> SyncTarget {
    let roadmap = project_path.join(FRAGMENT_ROADMAP);
    if crate::services::roadmap_fragments::entries_dir_for(&roadmap).is_some() {
        SyncTarget::Fragments(roadmap)
    } else {
        SyncTarget::WorkStore
    }
}

/// What `sync` does with the render.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncMode {
    /// Write it (the default).
    Write,
    /// Print it to stdout.
    DryRun,
    /// Exit 1 unless what is on disk already is the render.
    Check,
}

impl SyncMode {
    /// The mode `--dry-run` / `--check` select. clap refuses both at once.
    #[must_use]
    pub fn from_flags(dry_run: bool, check: bool) -> Self {
        match (dry_run, check) {
            (true, _) => Self::DryRun,
            (false, true) => Self::Check,
            (false, false) => Self::Write,
        }
    }
}

fn report(code: i32, stderr: String) -> AggregateReport {
    AggregateReport {
        code,
        stdout: String::new(),
        stderr,
    }
}

/// `sync` in fragment mode: the aggregate engine, with `sync`'s flags.
#[must_use]
pub fn run_fragment_sync(
    roadmap: &Path,
    gh_snapshot: Option<&str>,
    mode: SyncMode,
) -> AggregateReport {
    if gh_snapshot.is_some() {
        return report(
            2,
            "FAIL --gh-snapshot applies to the ROADMAP.yaml render only; \
             this project aggregates docs/roadmaps/entries/\n"
                .to_string(),
        );
    }
    let mode = match mode {
        SyncMode::Write => AggregateMode::Write,
        SyncMode::DryRun => AggregateMode::Print,
        SyncMode::Check => AggregateMode::Check,
    };
    run_aggregate(roadmap, None, mode)
}

/// `sync --check` without `entries/`: `ROADMAP.yaml` is current iff re-rendering
/// it with its own `generated_at` reproduces it byte for byte. The timestamp is the
/// one field the render does not derive from its sources, so it is taken from the
/// file rather than guessed; an absent `--gh-snapshot` likewise falls back to the
/// pin the file records.
#[must_use]
pub fn check_work_store(project_path: &Path, gh_snapshot: Option<String>) -> AggregateReport {
    let path = project_path.join("ROADMAP.yaml");
    let committed = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) => {
            return report(
                2,
                format!(
                    "FAIL read {}: {e} — an input that cannot be read is never a pass\n",
                    path.display()
                ),
            )
        }
    };
    let rows = match read_work_store_rows(project_path) {
        Ok(rows) => rows,
        Err(e) => return report(2, format!("FAIL {e:#}\n")),
    };
    let snapshot = gh_snapshot.or_else(|| recorded_field(&committed, "gh_snapshot:"));
    let generated_at = recorded_field(&committed, "generated_at:").unwrap_or_default();
    let rendered = render_roadmap(&RoadmapSources::new(rows, snapshot), &generated_at);
    match first_differing_line(&committed, &rendered) {
        None => report(0, format!("ok  {} == render\n", path.display())),
        Some(line) => report(
            1,
            format!(
                "FAIL {} is not what `pmat roadmap sync` renders — first differing line {line}. \
                 Run `pmat roadmap sync` on the default branch.\n",
                path.display()
            ),
        ),
    }
}

/// The double-quoted value of the first top-level `key` line, if any.
fn recorded_field(text: &str, key: &str) -> Option<String> {
    let rest = text.lines().find_map(|l| l.strip_prefix(key))?.trim_start();
    let inner = rest.strip_prefix('"')?;
    Some(inner[..inner.find('"')?].to_string())
}

/// 1-based line number of the first difference, or `None` when identical.
fn first_differing_line(a: &str, b: &str) -> Option<usize> {
    if a == b {
        return None;
    }
    let mut left = a.split_inclusive('\n');
    let mut right = b.split_inclusive('\n');
    let mut n = 1;
    loop {
        match (left.next(), right.next()) {
            (Some(x), Some(y)) if x == y => n += 1,
            _ => return Some(n),
        }
    }
}

/// `pmat roadmap sync` as the CLI runs it: print both streams, and exit with the
/// report's code when it is not 0.
///
/// # Errors
///
/// The MACS-013 render's read or write error; every other failure is the process
/// exit code, which is the command's contract.
pub fn execute(
    project_path: &Path,
    gh_snapshot: Option<String>,
    dry_run: bool,
    check: bool,
    generated_at: &str,
) -> anyhow::Result<()> {
    let mode = SyncMode::from_flags(dry_run, check);
    let report = match (sync_target(project_path), mode) {
        (SyncTarget::Fragments(roadmap), _) => {
            run_fragment_sync(&roadmap, gh_snapshot.as_deref(), mode)
        }
        (SyncTarget::WorkStore, SyncMode::Check) => check_work_store(project_path, gh_snapshot),
        (SyncTarget::WorkStore, _) => {
            return handle_roadmap_sync(project_path, gh_snapshot, dry_run, generated_at)
        }
    };
    print!("{}", report.stdout);
    eprint!("{}", report.stderr);
    if report.code != 0 {
        std::process::exit(report.code);
    }
    Ok(())
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
#[path = "sync_route_tests.rs"]
mod tests;
