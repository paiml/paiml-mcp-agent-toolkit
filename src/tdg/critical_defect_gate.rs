//! The ONE place TDG decides whether a piece of source carries critical defects.
//!
//! pmat has two analyzers that both answer "grade this source": the AST one
//! (`TdgAnalyzerAst`, behind `pmat tdg`) and the heuristic one
//! (`analyzer_simple::TdgAnalyzer`, behind the MCP `quality_gate` tool and
//! `mcp_integration::tdg_tools`). Only the AST one ran the Known-Defects gate,
//! so one build gave two verdicts on the same bytes: a committed `.rs` file with
//! three `Option::unwrap()` calls came back
//!
//! ```text
//! pmat tdg a.rs --format json  => {"total": 25.16, "grade": "F"}
//! MCP quality_gate {paths:[a.rs]} => {"passed": true, "score": 90.0, "grade": "A"}
//! ```
//!
//! The gate is not a property of an analyzer, it is a property of the source, so
//! it lives here and both `analyze_source` implementations call it. There is no
//! second copy to fall out of step: if you are looking for the defect rule, this
//! file is it.

use std::path::{Path, PathBuf};

use crate::services::defect_detector::{LuaDefectDetector, RustDefectDetector, Severity};
use crate::tdg::language_simple::Language;
use crate::tdg::score::TdgScore;

/// Why the #279 waiver fired, recorded verbatim on the score.
///
/// #919: the waiver used to be expressed by clearing `has_critical_defects`
/// while leaving `critical_defects_count` set, so one record asserted "1
/// critical defect" and "no critical defects" at once. The waiver names itself
/// instead.
pub(crate) const NEW_FILE_WAIVER: &str =
    "file is not tracked by git; critical-defect auto-fail is not applied to code \
     with no history (#279)";

/// Detect critical defects in `source` and stamp the verdict onto `score`.
///
/// Call this from every `analyze_source`, right before `calculate_total()`.
/// `score.file_path` is consulted for the #279 waiver only — detection itself
/// depends on the source, not on where it sits, which is the invariant #919 was
/// filed for.
pub(crate) fn apply(score: &mut TdgScore, source: &str, language: Language) {
    let count = count_critical_defects(source, language, score.file_path.as_deref());

    score.critical_defects_count = count;
    score.has_critical_defects = count > 0;

    if score.has_critical_defects
        && score
            .file_path
            .as_deref()
            .is_some_and(is_exempt_as_new_file)
    {
        score.critical_defects_suppressed = Some(NEW_FILE_WAIVER.to_string());
    }
}

/// How many critical defects `source` contains, as a number.
///
/// `path` is only a label for the detector's instance records; a `None` path is
/// not an absence of defects. An analyzer handed a source string with no file
/// behind it still sees the `unwrap()`s, and reporting zero for it would be the
/// "empty collection means clean" answer this gate exists to refuse.
pub(crate) fn count_critical_defects(
    source: &str,
    language: Language,
    path: Option<&Path>,
) -> usize {
    let label: PathBuf = path.map_or_else(|| PathBuf::from("<source>"), Path::to_path_buf);

    let detected: usize = match language {
        Language::Rust => critical_instances(&RustDefectDetector::new().detect(source, &label)),
        Language::Lua => critical_instances(&LuaDefectDetector::new().detect(source, &label)),
        _ => 0,
    };

    // Lean-specific: `sorry` is an admitted proof obligation, i.e. a critical
    // defect. The AST analyzer counted it only when a path was present and the
    // heuristic analyzer counted it only through its own byte-identical copy of
    // the counter; there is now one counter and one rule.
    let lean_sorry = if language == Language::Lean {
        count_lean_sorry(source)
    } else {
        0
    };

    detected + lean_sorry
}

fn critical_instances(defects: &[crate::services::defect_detector::DefectPattern]) -> usize {
    defects
        .iter()
        .filter(|d| d.severity == Severity::Critical)
        .map(|d| d.instances.len())
        .sum()
}

/// Whether the #279 auto-fail waiver applies to this file.
///
/// #279 waives a file that has no git history yet, because a gate that blocks
/// the commit is a gate the file cannot pass. That reasoning presupposes a
/// repository — the file is *about to* gain history. It says nothing about code
/// that is simply not under version control at all (an unpacked tarball, a
/// vendored tree, a scratch directory), where there is no commit to be blocked
/// and so nothing to waive.
///
/// The old predicate collapsed those two cases into one `false`, so analysing
/// any code outside a repository silently waived the Known-Defects gate — the
/// same bytes scoring 0.0/F inside a repo and 100.0/A+ outside it (#919).
///
/// The git query MUST be rooted at the ANALYSED FILE, never at the process
/// working directory. Before this was fixed the command was plain
/// `git log --oneline -1 -- <path>`, which git resolves against the CWD: run
/// from anywhere outside the analysed repository git exits 128 ("not a git
/// repository"), the committed file was read as untracked, and
/// `has_critical_defects` was silently cleared. Observed on one unchanged
/// fixture (round 3): `cd <repo> && pmat analyze tdg -p .` scored 0.0 / grade F
/// while `cd /tmp && pmat analyze tdg -p <repo>` scored 100.0 / grade A+ — a
/// 5-band grade swing produced by nothing but the caller's CWD.
pub(crate) fn is_exempt_as_new_file(path: &Path) -> bool {
    matches!(git_tracking_status(path), GitTracking::UntrackedInRepo)
}

/// Where a file stands with git, keeping "no repository" distinct from
/// "in a repository but not yet committed" — see [`is_exempt_as_new_file`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GitTracking {
    /// Has at least one commit.
    Tracked,
    /// Inside a work tree, but with no history yet — the #279 case.
    UntrackedInRepo,
    /// Not under version control, or git is unavailable. Not a #279 case: the
    /// gate applies exactly as it would to committed code.
    NotVersioned,
}

pub(crate) fn git_tracking_status(path: &Path) -> GitTracking {
    let Some(repo_anchor) = git_anchor_for(path) else {
        return GitTracking::NotVersioned;
    };
    let absolute = absolute_path(path);

    let run = |args: &[&str], file: Option<&Path>| {
        let mut cmd = std::process::Command::new("git");
        cmd.arg("-C").arg(&repo_anchor).args(args);
        if let Some(f) = file {
            cmd.arg("--").arg(f);
        }
        cmd.stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .output()
    };

    // Is there a work tree at all? If git is missing or errors, treat the code
    // as unversioned rather than exempt — an exemption must be established, not
    // assumed, or a broken git install silently disables the gate.
    let inside_work_tree = run(&["rev-parse", "--is-inside-work-tree"], None)
        .map(|o| o.status.success() && String::from_utf8_lossy(&o.stdout).trim() == "true")
        .unwrap_or(false);
    if !inside_work_tree {
        return GitTracking::NotVersioned;
    }

    let has_history = run(&["log", "--oneline", "-1"], Some(&absolute))
        .map(|o| o.status.success() && !o.stdout.is_empty())
        .unwrap_or(false);

    if has_history {
        GitTracking::Tracked
    } else {
        GitTracking::UntrackedInRepo
    }
}

fn absolute_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| {
        std::env::current_dir().map_or_else(|_| path.to_path_buf(), |cwd| cwd.join(path))
    })
}

/// The directory to run git from: the file's own directory, so the query finds
/// the file's repository rather than the process working directory's.
fn git_anchor_for(path: &Path) -> Option<PathBuf> {
    let absolute = absolute_path(path);
    if absolute.is_dir() {
        return Some(absolute);
    }
    match absolute.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => Some(parent.to_path_buf()),
        _ => Some(absolute),
    }
}

/// Retained for the original call shape; see [`git_tracking_status`].
#[cfg(test)]
pub(crate) fn is_file_git_tracked(path: &Path) -> bool {
    matches!(git_tracking_status(path), GitTracking::Tracked)
}

/// Count `sorry` occurrences in Lean source (an admitted proof obligation).
///
/// Skips line comments (`--`) and nested block comments (`/- ... -/`), and
/// requires a word boundary so `sorryAx` does not count. This used to exist
/// twice, byte for byte, once per analyzer.
pub(crate) fn count_lean_sorry(source: &str) -> usize {
    let mut count = 0;
    let mut in_block_comment: i32 = 0;

    for line in source.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("--") {
            continue;
        }

        let cleaned = strip_lean_block_comments(trimmed, &mut in_block_comment);

        if in_block_comment > 0 {
            continue;
        }

        if contains_lean_sorry_word(&cleaned) {
            count += 1;
        }
    }

    count
}

/// Strips Lean block comment content (`/- ... -/`) from a line.
fn strip_lean_block_comments(line: &str, depth: &mut i32) -> String {
    let bytes = line.as_bytes();
    let mut result = String::with_capacity(line.len());
    let mut i = 0;

    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'-' {
            *depth += 1;
            i += 2;
            continue;
        }
        if i + 1 < bytes.len() && bytes[i] == b'-' && bytes[i + 1] == b'/' && *depth > 0 {
            *depth -= 1;
            i += 2;
            continue;
        }
        if *depth == 0 {
            result.push(bytes[i] as char);
        }
        i += 1;
    }

    result
}

/// Checks if a line contains "sorry" as a standalone word.
fn contains_lean_sorry_word(line: &str) -> bool {
    let bytes = line.as_bytes();
    let sorry = b"sorry";

    let mut pos = 0;
    while pos + sorry.len() <= bytes.len() {
        if let Some(idx) = line[pos..].find("sorry") {
            let abs_idx = pos + idx;
            let before_ok = abs_idx == 0
                || (!bytes[abs_idx - 1].is_ascii_alphanumeric() && bytes[abs_idx - 1] != b'_');
            let after_ok = abs_idx + sorry.len() >= bytes.len()
                || (!bytes[abs_idx + sorry.len()].is_ascii_alphanumeric()
                    && bytes[abs_idx + sorry.len()] != b'_');
            if before_ok && after_ok {
                return true;
            }
            pos = abs_idx + 1;
        } else {
            break;
        }
    }
    false
}
