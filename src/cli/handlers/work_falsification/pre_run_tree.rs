#![cfg_attr(coverage_nightly, coverage(off))]
//! GH #630: judge the github-sync claim on the tree pmat *found*, not the one it made.
//!
//! `pmat work complete` writes while it runs — caches under `.pmat/`, the
//! ledger and receipts under `.pmat-work/`, commit metadata under
//! `.pmat-metrics/`, and, once the claims pass, `docs/roadmaps/roadmap.yaml`
//! and `CHANGELOG.md`. PMAT-154 filtered the three pmat-owned prefixes out of
//! the dirty count, but filtering by path can only ever cover the writes
//! someone remembered to enumerate, and it cannot cover writes into user-owned
//! files. The claim stayed unsatisfiable often enough that every completion
//! carried `--override-claims github-sync`, which is the same as not having the
//! claim at all.
//!
//! So instead of enumerating what pmat writes, record the working tree once,
//! before pmat has written anything, and judge the claim against that. Whatever
//! the run goes on to touch, the verdict describes the user's work — which is
//! what "All changes pushed" was ever meant to assert. The snapshot is taken
//! per process and the first call wins, so a later write cannot move the
//! baseline it is being judged against.

use std::path::Path;
use std::process::Command;
use std::sync::Mutex;

/// The working tree as pmat found it, captured before any pmat-managed write.
///
/// Scoped to one command invocation rather than to the process: pmat also runs
/// as a long-lived MCP server, where a single process serves many completions
/// across different projects. A snapshot that outlived its command would judge
/// the next project against the previous one's tree.
static PRE_RUN_STATUS: Mutex<Option<String>> = Mutex::new(None);

/// Keeps a snapshot alive for one command, and clears it on drop.
///
/// Hold it for as long as the claim may be evaluated — binding it to `_guard`
/// in the handler is enough; dropping it early re-exposes the live-read path.
#[must_use = "the snapshot is cleared as soon as this guard is dropped"]
pub struct TreeSnapshot {
    /// Only the call that recorded the snapshot may clear it, so a nested
    /// snapshot cannot pull the baseline out from under its caller.
    owns: bool,
}

impl Drop for TreeSnapshot {
    fn drop(&mut self) {
        if self.owns {
            if let Ok(mut slot) = PRE_RUN_STATUS.lock() {
                *slot = None;
            }
        }
    }
}

/// Record the working tree before pmat writes anything.
///
/// Call as early as possible — ideally the first statement of the command
/// handler, before any cache, ledger or roadmap write — and keep the returned
/// guard alive for the rest of the command.
pub fn snapshot_working_tree(project_path: &Path) -> TreeSnapshot {
    let Ok(mut slot) = PRE_RUN_STATUS.lock() else {
        return TreeSnapshot { owns: false };
    };
    if slot.is_some() {
        // An enclosing command already established the baseline.
        return TreeSnapshot { owns: false };
    }
    *slot = read_porcelain_status(project_path);
    TreeSnapshot {
        owns: slot.is_some(),
    }
}

/// The recorded snapshot, or `None` if no command established one.
///
/// A `None` here means the caller is not a command that mutates the tree, so
/// reading git status live is still correct for it.
pub(crate) fn pre_run_status() -> Option<String> {
    PRE_RUN_STATUS.lock().ok().and_then(|slot| slot.clone())
}

/// Read `git status --porcelain -b`, or `None` outside a working repo.
pub(crate) fn read_porcelain_status(project_path: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["status", "--porcelain", "-b"])
        .current_dir(project_path)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Count files with uncommitted changes in porcelain output.
///
/// Two exclusions: untracked `??` entries are not uncommitted *changes*
/// (GH #224), and pmat-owned state is not the user's work (PMAT-154). The
/// leading `## branch...upstream` header line is skipped.
pub(crate) fn count_dirty_files(status: &str) -> usize {
    status
        .lines()
        .skip(1)
        .filter(|l| !l.is_empty() && !l.starts_with("??"))
        .filter(|l| !super::pmat_owned_state::is_pmat_owned_state(l))
        .count()
}

/// The paths behind [`count_dirty_files`], for reporting.
///
/// The claim used to say only "1 uncommitted file(s)". With no path attached, a
/// true positive is indistinguishable from a pmat self-dirty bug — which is how
/// #630 came to be filed against a filter that was working. Naming the files
/// makes the verdict checkable by the person reading it.
pub(crate) fn dirty_file_paths(status: &str) -> Vec<String> {
    status
        .lines()
        .skip(1)
        .filter(|l| !l.is_empty() && !l.starts_with("??"))
        .filter(|l| !super::pmat_owned_state::is_pmat_owned_state(l))
        .filter_map(|l| l.get(3..).map(display_path))
        .collect()
}

/// Render one porcelain path field as the path a reader can act on.
///
/// Two shapes need handling, or the message shows raw porcelain rather than a
/// filename: renames arrive as `OLD -> NEW` (the working tree carries NEW), and
/// paths with spaces or specials arrive quoted.
fn display_path(field: &str) -> String {
    let path = field.trim();
    let path = path.rsplit(" -> ").next().unwrap_or(path).trim();
    super::pmat_owned_state::unquote_porcelain_path(path).to_string()
}

/// True if the current branch tracks an upstream.
///
/// The porcelain header is `## branch...upstream [ahead N]`; without an
/// upstream git emits just `## branch`. That header contains no "ahead", so the
/// old parse returned 0 and a branch that had never been pushed *at all*
/// satisfied "All changes pushed" — a false pass on the claim's whole point.
pub(crate) fn has_upstream(status: &str) -> bool {
    status
        .lines()
        .next()
        .is_some_and(|header| header.contains("..."))
}

/// Parse the unpushed-commit count from the porcelain branch header.
///
/// The header reads `## branch...upstream [ahead 3]`, and on a diverged branch
/// `[ahead 3, behind 2]` — so the count can be followed by `]` *or* by `,`.
///
/// The divergence must be read out of the bracketed group, never by searching
/// the whole header: `"ahead"` also occurs in ordinary branch names, and
/// `## read-ahead...origin/read-ahead [ahead 1]` then matched inside the branch
/// name and parsed to 0 — a false pass claiming everything was pushed while a
/// commit was not. Git rejects `[` in ref names (`git check-ref-format`), so
/// the bracket is an unambiguous anchor.
pub(crate) fn parse_ahead_count(status: &str) -> usize {
    let Some(header) = status.lines().next() else {
        return 0;
    };
    let Some(divergence) = header
        .rsplit_once('[')
        .map(|(_, tail)| tail)
        .filter(|tail| tail.starts_with("ahead"))
    else {
        return 0;
    };
    divergence
        .split_whitespace()
        .nth(1)
        .map(|n| n.trim_matches(|c: char| !c.is_ascii_digit()))
        .and_then(|n| n.parse::<usize>().ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ahead_count_on_a_simple_lead() {
        assert_eq!(
            parse_ahead_count("## master...origin/master [ahead 3]\n"),
            3
        );
    }

    #[test]
    fn ahead_count_on_a_diverged_branch() {
        // GH #630 review: `[ahead 1, behind 2]` leaves a comma on the number,
        // which `trim_end_matches(']')` did not remove -- the parse failed and
        // the claim reported 0 unpushed commits on a diverged branch.
        assert_eq!(
            parse_ahead_count("## main...origin/main [ahead 1, behind 2]\n"),
            1
        );
        assert_eq!(
            parse_ahead_count("## main...origin/main [ahead 12, behind 40]\n"),
            12
        );
    }

    #[test]
    fn ahead_count_is_zero_when_only_behind() {
        assert_eq!(parse_ahead_count("## main...origin/main [behind 3]\n"), 0);
    }

    #[test]
    fn ahead_count_is_zero_when_in_sync_or_headerless() {
        assert_eq!(parse_ahead_count("## main...origin/main\n"), 0);
        assert_eq!(parse_ahead_count(""), 0);
    }

    #[test]
    fn ahead_count_is_read_from_the_bracket_not_the_branch_name() {
        // Regression: searching the whole header matched "ahead" inside the
        // branch name, so this real header parsed to 0 and the claim passed
        // while a commit was unpushed. The earlier version of this test used
        // `## ahead...origin/ahead`, where 0 happens to be the right answer —
        // so it exercised the bug and pinned it as correct.
        assert_eq!(
            parse_ahead_count("## read-ahead...origin/read-ahead [ahead 1]\n"),
            1
        );
        assert_eq!(
            parse_ahead_count("## feat/lookahead...origin/feat/lookahead [ahead 4, behind 2]\n"),
            4
        );
        // A branch merely named "ahead", with nothing unpushed, is still 0.
        assert_eq!(parse_ahead_count("## ahead...origin/ahead\n"), 0);
    }

    #[test]
    fn dirty_paths_are_rendered_readably() {
        // Renames arrive as `OLD -> NEW` and specials arrive quoted; showing
        // either raw makes the verdict harder to act on, not easier.
        let status = concat!(
            "## main...origin/main\n",
            "R  src/old.rs -> src/new.rs\n",
            " M \"src/we ird.rs\"\n",
        );
        assert_eq!(
            dirty_file_paths(status),
            vec!["src/new.rs", "src/we ird.rs"]
        );
    }

    #[test]
    fn dirty_count_excludes_untracked_and_pmat_state() {
        let status = concat!(
            "## master...origin/master\n",
            "M  .pmat-work/ledger.jsonl\n",
            " M .pmat/context.db\n",
            "A  .pmat-metrics/commit-abc123-meta.json\n",
            " M src/lib.rs\n",
            "?? untracked.txt\n",
        );
        assert_eq!(count_dirty_files(status), 1);
    }

    #[test]
    fn upstream_detected_from_the_header() {
        assert!(has_upstream("## master...origin/master\n"));
        assert!(has_upstream("## main...origin/main [ahead 1, behind 2]\n"));
    }

    #[test]
    fn missing_upstream_detected() {
        // A branch that was never pushed has no `...upstream` in the header.
        // It contains no "ahead" either, so the old parse called it 0 unpushed
        // commits and the claim passed on a branch with nothing pushed at all.
        assert!(!has_upstream("## solo\n M src/lib.rs\n"));
        assert!(!has_upstream("## No commits yet on master\n"));
        assert!(!has_upstream(""));
    }

    #[test]
    fn dirty_paths_are_reported_for_the_verdict() {
        let status = concat!(
            "## master...origin/master\n",
            " M src/lib.rs\n",
            "A  src/new.rs\n",
            " M .pmat/context.db\n",
            "?? untracked.txt\n",
        );
        assert_eq!(dirty_file_paths(status), vec!["src/lib.rs", "src/new.rs"]);
    }

    #[test]
    fn dirty_paths_agree_with_the_count() {
        let status = concat!(
            "## main...origin/main\n",
            " M docs/roadmaps/roadmap.yaml\n",
            " M CHANGELOG.md\n",
            "M  .pmat-work/ledger.jsonl\n",
        );
        assert_eq!(dirty_file_paths(status).len(), count_dirty_files(status));
    }

    #[test]
    fn dirty_count_is_zero_on_a_clean_tree() {
        assert_eq!(count_dirty_files("## master...origin/master\n"), 0);
        assert_eq!(count_dirty_files(""), 0);
    }

    #[test]
    fn dirty_count_sees_user_owned_roadmap_writes() {
        // The roadmap is user-owned, so a genuine edit must still count -- the
        // snapshot, not a path filter, is what keeps pmat's own write out.
        let status = concat!(
            "## main...origin/main\n",
            " M docs/roadmaps/roadmap.yaml\n",
            " M CHANGELOG.md\n",
        );
        assert_eq!(count_dirty_files(status), 2);
    }

    /// Build a fixture repo with one committed file.
    fn repo_with_one_commit() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        run_git(dir.path(), &["init", "-q"]);
        run_git(dir.path(), &["config", "user.email", "t@example.com"]);
        run_git(dir.path(), &["config", "user.name", "t"]);
        std::fs::write(dir.path().join("tracked.txt"), "v1").unwrap();
        run_git(dir.path(), &["add", "."]);
        run_git(dir.path(), &["commit", "-qm", "init"]);
        dir
    }

    // These drive the real process-wide slot, so they must not interleave.
    #[test]
    #[serial_test::serial(pmat_pre_run_tree)]
    fn snapshot_is_taken_once_and_never_moves() {
        // GH #630 core property: pmat writing after the snapshot cannot change
        // the baseline the claim is judged against.
        let dir = repo_with_one_commit();

        let _guard = snapshot_working_tree(dir.path());
        let first = pre_run_status().expect("a git repo must record a baseline");
        assert_eq!(
            count_dirty_files(&first),
            0,
            "tree was clean when snapshotted"
        );

        // Simulate pmat dirtying the tree mid-run, then re-snapshotting.
        std::fs::write(dir.path().join("tracked.txt"), "v2").unwrap();
        let _nested = snapshot_working_tree(dir.path());

        assert_eq!(
            pre_run_status().unwrap(),
            first,
            "a write after the snapshot must not move the baseline"
        );
        assert_eq!(count_dirty_files(&pre_run_status().unwrap()), 0);
    }

    #[test]
    #[serial_test::serial(pmat_pre_run_tree)]
    fn snapshot_records_genuinely_dirty_user_work() {
        // The claim must still catch the thing it exists for.
        let dir = repo_with_one_commit();
        std::fs::write(dir.path().join("tracked.txt"), "uncommitted edit").unwrap();

        let _guard = snapshot_working_tree(dir.path());

        assert_eq!(count_dirty_files(&pre_run_status().unwrap()), 1);
    }

    #[test]
    #[serial_test::serial(pmat_pre_run_tree)]
    fn snapshot_is_cleared_when_the_command_ends() {
        // pmat also runs as a long-lived MCP server. A baseline that outlived
        // its command would judge the next project against the previous one's
        // tree, which is a worse failure than the bug this fixes.
        let dir = repo_with_one_commit();
        {
            let _guard = snapshot_working_tree(dir.path());
            assert!(pre_run_status().is_some());
        }
        assert!(
            pre_run_status().is_none(),
            "the snapshot must not outlive the command that took it"
        );

        // A second command against a different project sees its own tree.
        let other = repo_with_one_commit();
        std::fs::write(other.path().join("tracked.txt"), "dirty").unwrap();
        let _guard = snapshot_working_tree(other.path());
        assert_eq!(count_dirty_files(&pre_run_status().unwrap()), 1);
    }

    #[test]
    #[serial_test::serial(pmat_pre_run_tree)]
    fn no_snapshot_outside_a_git_repo() {
        let dir = tempfile::tempdir().unwrap();
        let _guard = snapshot_working_tree(dir.path());
        assert!(
            pre_run_status().is_none(),
            "a non-repo must not record an empty baseline"
        );
    }

    /// Run git in the fixture repo, isolated from the developer's global
    /// config. Without `core.hooksPath` and `commit.gpgsign` overrides these
    /// tests inherit whatever hooks the machine has installed -- pmat's own
    /// pre-commit gate among them -- and fail for reasons unrelated to #630.
    fn run_git(dir: &Path, args: &[&str]) {
        let ok = Command::new("git")
            .args([
                "-c",
                "core.hooksPath=/dev/null",
                "-c",
                "commit.gpgsign=false",
            ])
            .args(args)
            .current_dir(dir)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        assert!(ok, "git {:?} failed", args);
    }
}
