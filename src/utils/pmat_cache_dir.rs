//! The state directories pmat writes into the project it is analysing, and the
//! ignore rule that keeps them out of that project's git status.
//!
//! Issue #1050 P8. An analysis tool dirtied the working tree it was analysing:
//!
//! ```text
//! $ pmat analyze dead-code --path $R >/dev/null 2>&1
//! $ pmat tdg $R              >/dev/null 2>&1
//! $ git status --porcelain
//! ?? .pmat/
//! ```
//!
//! …holding `tdg-cold.db`, `tdg-warm.db` and `dead-code-cache-<hash>.json`.
//! Projects were told to add `.pmat/dead-code-cache.json` to `.gitignore`, and
//! that rule is what depyler's `.gitignore` still carries — but the cache key
//! grew a scope-and-depth suffix (`dead-code-cache-eb-d8.json`), so the rule
//! stopped matching the file it was written for, silently, with no error on
//! either side.
//!
//! A rule kept in the analysed project's `.gitignore` is a rule that has to
//! track pmat's cache filenames forever, and it lost that race once already.
//! So the rule moves to where the files are: pmat drops a `.gitignore` INTO
//! `.pmat/` the moment it creates it, the way cargo does for `target/`. It
//! covers whatever pmat writes there next, it needs no cooperation from the
//! project, and a project that never runs pmat again is left with one
//! self-describing file instead of a stale pattern in its own `.gitignore`.
//!
//! # `.pmat/` was not the only directory
//!
//! The first cut of this module wired three call sites and left the rest of the
//! tree writing `create_dir_all(project.join(".pmat"))` by hand, so the class of
//! defect survived the fix for its instance: `pmat score` persists its composite
//! to `.pmat-metrics/commit-<sha>-meta.json`, creates `.pmat-metrics/` to do it,
//! and left `?? .pmat-metrics/` in a clean checkout of the fixture in this
//! module's own reproducer. Same defect, one directory over.
//!
//! So the rule is stated once, [`PMAT_DIR_GITIGNORE`], and applied by
//! [`ensure_self_ignoring_dir`] to every pmat-owned *derived state* directory:
//!
//! | directory        | holds                                              |
//! |------------------|----------------------------------------------------|
//! | `.pmat/`         | caches, indexes, tiered stores, baselines, backups |
//! | `.pmat-metrics/` | score/test/coverage/lint measurements per commit   |
//!
//! Deliberately NOT covered: `.pmat-tickets/` (this repo tracks its own — see
//! `git ls-files .pmat-tickets`), `.pmat-work/` and `.pmat-qa/`, which hold
//! work items, receipts and checklists a project may legitimately want in
//! version control; and `~/.pmat/`, which is not inside anybody's checkout to
//! begin with. Auto-ignoring those would be a decision about someone else's
//! content, not a fix for pmat's own litter.

use std::path::{Path, PathBuf};

/// What pmat writes into the `.gitignore` of every state directory it creates.
///
/// A bare `*`, which is exactly what cargo writes into `target/.gitignore`,
/// and it is the rule this repo already enforces on everyone else: CB-529
/// ("`.pmat/` Tracked in Git") reports ANY tracked path with a `.pmat/`
/// segment as an `Error` — *".pmat/ artifact tracked in git — will ship to
/// crates.io. Fix: git rm --cached … && add '**/.pmat/' to .gitignore"* — and
/// `baseline.json` is classified as generated (`bottleneck_handler.rs`) and as
/// a cache (`check_commit_enforcement.rs`) elsewhere in this tree. A rule with
/// exceptions would have made pmat's cache directory disagree with pmat's own
/// published rule about that directory.
///
/// This cannot untrack anything: gitignore has no effect on files already in
/// the index, so a project that tracks `.pmat/baseline.json` today keeps it,
/// keeps seeing its diffs, and is told about it by CB-529 as before.
pub const PMAT_DIR_GITIGNORE: &str = "\
# Written by pmat. Everything in this directory is derived from the tree and
# rebuilt on demand: caches, indexes, tiered stores, baselines, backups,
# per-commit measurements.
#
# Kept here rather than in the project's own .gitignore because these filenames
# are pmat's to change — the rule projects were once told to add,
# `.pmat/dead-code-cache.json`, stopped matching the day the cache key gained a
# scope-and-depth suffix, in silence, and nothing on either side could notice.
# A rule that matches by directory cannot lose that race.
*
";

/// The `.pmat/` directory of `project_root`.
#[must_use]
pub fn pmat_dir(project_root: &Path) -> PathBuf {
    project_root.join(".pmat")
}

/// The `.pmat-metrics/` directory of `project_root`.
#[must_use]
pub fn pmat_metrics_dir(project_root: &Path) -> PathBuf {
    project_root.join(".pmat-metrics")
}

/// Create a pmat-owned state directory if absent and make sure it ignores
/// itself.
///
/// Best-effort by design: a read-only checkout must not turn "I could not write
/// a `.gitignore`" into a failed analysis, and the caller's own write will
/// report the real problem a moment later, with the path it actually wanted.
///
/// Never overwrites an existing `.gitignore` — a project that has edited one is
/// making a decision, and silently replacing it would be the tool dirtying the
/// tree in a second way. `exists()` rather than an unconditional write for the
/// same reason: rewriting identical bytes on every invocation would move the
/// file's mtime and wake every watcher pointed at the tree.
pub fn ensure_self_ignoring_dir(dir: &Path) -> PathBuf {
    let _ = std::fs::create_dir_all(dir);
    write_ignore_rule(dir);
    dir.to_path_buf()
}

fn write_ignore_rule(dir: &Path) {
    let ignore = dir.join(".gitignore");
    if !ignore.exists() {
        let _ = std::fs::write(&ignore, PMAT_DIR_GITIGNORE);
    }
}

/// The directory names this module claims. A directory named anything else is
/// somebody's content and is left alone — see the module header.
const SELF_IGNORING_DIR_NAMES: [&str; 2] = [".pmat", ".pmat-metrics"];

/// Create the parent directory of a file pmat is about to write, and give every
/// pmat state directory on the way there its ignore rule.
///
/// For the call sites that name the FILE — `<project>/.pmat/context.idx`,
/// `<project>/.pmat-metrics/deny-status.json` — and reach the directory through
/// `path.parent()`. Those are the sites the first sweep missed, because the
/// `.pmat` literal is in the path expression and never in the `create_dir_all`
/// call, so grepping for one did not find the other.
///
/// Unlike [`ensure_self_ignoring_dir`] this propagates the `create_dir_all`
/// error, because the callers it replaces did: they are writing the file next
/// and want to fail with the reason.
pub fn ensure_parent_dir(path: &Path) -> std::io::Result<()> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    std::fs::create_dir_all(parent)?;
    // Ancestors, not just `parent`: a write to `.pmat/backup/x.json` creates
    // `.pmat/` on the way, and that is the directory the rule belongs in.
    for dir in parent.ancestors() {
        if dir
            .file_name()
            .is_some_and(|n| SELF_IGNORING_DIR_NAMES.iter().any(|d| n == *d))
        {
            write_ignore_rule(dir);
        }
    }
    Ok(())
}

/// Create `<project_root>/.pmat/` if absent and make sure it ignores itself.
///
/// Call this instead of `create_dir_all(project.join(".pmat"))` at every site
/// that writes a cache into an analysed project.
pub fn ensure_cache_dir(project_root: &Path) -> PathBuf {
    ensure_self_ignoring_dir(&pmat_dir(project_root))
}

/// Create `<project_root>/.pmat-metrics/` if absent and make sure it ignores
/// itself.
///
/// Call this instead of `create_dir_all(project.join(".pmat-metrics"))` at every
/// site that records a measurement into an analysed project.
pub fn ensure_metrics_dir(project_root: &Path) -> PathBuf {
    ensure_self_ignoring_dir(&pmat_metrics_dir(project_root))
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn git(dir: &Path, args: &[&str]) -> std::process::Output {
        Command::new("git")
            .current_dir(dir)
            .args(args)
            .output()
            .expect("git must be runnable")
    }

    /// `--template=` keeps the developer's global hook template out of the
    /// fixture; without it a machine with `pmat hooks install` in its template
    /// dir runs pmat's own gates inside this test.
    fn repo() -> tempfile::TempDir {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let out = git(dir.path(), &["init", "-q", "--template=", "."]);
        assert!(out.status.success(), "git init failed: {out:?}");
        dir
    }

    /// Stage and commit `rel` even though the ignore rule covers it, the way a
    /// project that deliberately versions a pmat artifact would have to.
    ///
    /// `--no-verify`: a global `core.hooksPath` would otherwise run the
    /// developer's own pre-commit gates against this two-file fixture.
    fn commit_forced(dir: &Path, rel: &str) {
        git(dir, &["add", "-f", rel]);
        let out = git(
            dir,
            &[
                "-c",
                "user.email=t@t",
                "-c",
                "user.name=t",
                "commit",
                "--no-verify",
                "-qm",
                "tracked",
            ],
        );
        assert!(out.status.success(), "commit failed: {out:?}");
    }

    fn porcelain(dir: &Path) -> String {
        String::from_utf8_lossy(&git(dir, &["status", "--porcelain"]).stdout).into_owned()
    }

    /// A tracked file, edited on disk, must still be tracked and must report as
    /// MODIFIED.
    ///
    /// `porcelain().contains(rel)` is not enough and was not enough: an
    /// implementation that reached for `git rm --cached` to make the directory
    /// stop showing up passes a `contains` check, because the path is still in
    /// the output — as ` D`, the project's file dropped out of the index. The
    /// two halves below are what separates "git no longer mentions our cache"
    /// from "git no longer knows about your file".
    fn assert_still_tracked_and_modified(dir: &Path, rel: &str) {
        let tracked = git(dir, &["ls-files", "--error-unmatch", rel]);
        assert!(
            tracked.status.success(),
            "{rel} was dropped from the index: {}",
            String::from_utf8_lossy(&tracked.stderr)
        );
        let status = porcelain(dir);
        assert!(
            status.lines().any(|l| l == format!(" M {rel}")),
            "an edit to a TRACKED file must report as modified, got: {status:?}"
        );
    }

    /// The reproducer, asked of git rather than of a filename pattern: after
    /// pmat has written a cache, the analysed tree must be clean.
    #[test]
    fn a_cache_written_into_a_repo_leaves_its_status_clean() {
        let dir = repo();
        let cache = ensure_cache_dir(dir.path());
        // The exact filenames the sweep found, plus one that does not exist
        // yet — the whole point of the rule living here is that it does not
        // have to be updated for the next one.
        for name in [
            "tdg-cold.db",
            "tdg-warm.db",
            "dead-code-cache-eb-d8.json",
            "some-cache-invented-next-year.bin",
        ] {
            std::fs::write(cache.join(name), b"x").expect("write cache");
        }

        let status = git(dir.path(), &["status", "--porcelain"]);
        assert!(
            status.stdout.is_empty(),
            "pmat dirtied the tree it analysed: {}",
            String::from_utf8_lossy(&status.stdout)
        );
    }

    /// The counter-test bounding the correction in the direction that would
    /// hurt: ignoring the directory must not make a project LOSE a file it
    /// already tracks. gitignore has no effect on the index, and depyler tracks
    /// `.pmat/baseline.json` — so the rule has to be provably harmless to it.
    #[test]
    fn a_file_the_project_already_tracks_is_untouched() {
        let dir = repo();
        let pmat = pmat_dir(dir.path());
        std::fs::create_dir_all(&pmat).expect("mkdir");
        std::fs::write(pmat.join("baseline.json"), b"{}").expect("write");
        commit_forced(dir.path(), ".pmat/baseline.json");

        ensure_cache_dir(dir.path());
        std::fs::write(pmat.join("baseline.json"), b"{\"changed\":1}").expect("write");

        assert_still_tracked_and_modified(dir.path(), ".pmat/baseline.json");
    }

    /// `.pmat/` was fixed and `.pmat-metrics/` was not, so `pmat score` still
    /// left `?? .pmat-metrics/` in a clean checkout. Same reproducer, one
    /// directory over, with the filenames `persist_score` and `pmat test-record`
    /// actually write.
    #[test]
    fn a_metric_recorded_into_a_repo_leaves_its_status_clean() {
        let dir = repo();
        let metrics = ensure_metrics_dir(dir.path());
        for name in [
            "commit-deadbee-meta.json",
            "commit-deadbee-tests.json",
            "coverage.json",
            "ratchet-overrides.jsonl",
        ] {
            std::fs::write(metrics.join(name), b"{}").expect("write metric");
        }
        std::fs::create_dir_all(metrics.join("trends")).expect("mkdir trends");
        std::fs::write(metrics.join("trends/lint.json"), b"[]").expect("write trend");

        assert!(
            porcelain(dir.path()).is_empty(),
            "pmat dirtied the tree it measured: {:?}",
            porcelain(dir.path())
        );
    }

    /// The counter-test for the second directory: a project that versions its
    /// score history keeps seeing it.
    #[test]
    fn a_tracked_metric_file_is_untouched() {
        let dir = repo();
        let metrics = pmat_metrics_dir(dir.path());
        std::fs::create_dir_all(&metrics).expect("mkdir");
        std::fs::write(metrics.join("commit-deadbee-meta.json"), b"{}").expect("write");
        commit_forced(dir.path(), ".pmat-metrics/commit-deadbee-meta.json");

        ensure_metrics_dir(dir.path());
        std::fs::write(metrics.join("commit-deadbee-meta.json"), b"{\"c\":1}").expect("write");

        assert_still_tracked_and_modified(dir.path(), ".pmat-metrics/commit-deadbee-meta.json");
    }

    /// The two directories are the same promise, so they carry the same rule —
    /// and neither of them is the project's own `.gitignore`, which stays as
    /// the project left it.
    #[test]
    fn both_state_directories_ignore_themselves_and_leave_the_project_alone() {
        let dir = repo();
        std::fs::write(dir.path().join(".gitignore"), b"target/\n").expect("write");
        commit_forced(dir.path(), ".gitignore");

        for d in [ensure_cache_dir(dir.path()), ensure_metrics_dir(dir.path())] {
            assert_eq!(
                std::fs::read_to_string(d.join(".gitignore")).expect("read"),
                PMAT_DIR_GITIGNORE
            );
        }
        assert_eq!(
            std::fs::read_to_string(dir.path().join(".gitignore")).expect("read"),
            "target/\n",
            "the project's own .gitignore is not pmat's to edit"
        );
        assert!(porcelain(dir.path()).is_empty(), "tree must stay clean");
    }

    /// The sites that name a file rather than a directory — `.pmat/context.idx`,
    /// `.pmat-metrics/deny-status.json` — and the nested case, where `.pmat/` is
    /// created only as a side effect of making `.pmat/backup/`.
    #[test]
    fn creating_a_parent_directory_also_leaves_the_tree_clean() {
        let dir = repo();
        for rel in [
            ".pmat/context.idx",
            ".pmat-metrics/deny-status.json",
            ".pmat/backup/3.31.0/project.toml",
        ] {
            let path = dir.path().join(rel);
            ensure_parent_dir(&path).expect("mkdir parent");
            std::fs::write(&path, b"x").expect("write");
        }

        assert_eq!(
            std::fs::read_to_string(pmat_dir(dir.path()).join(".gitignore")).expect("read"),
            PMAT_DIR_GITIGNORE,
            "`.pmat/` created only as the parent of `.pmat/backup/` still gets the rule"
        );
        assert!(
            porcelain(dir.path()).is_empty(),
            "pmat dirtied the tree it analysed: {:?}",
            porcelain(dir.path())
        );
    }

    /// The rule is for pmat's own state directories and no others: a directory
    /// that merely happens to sit under the project keeps its normal git
    /// behaviour, or this would be pmat quietly ignoring somebody's source.
    #[test]
    fn a_directory_pmat_does_not_own_gets_no_rule() {
        let dir = repo();
        let path = dir.path().join("src/generated/table.rs");
        ensure_parent_dir(&path).expect("mkdir parent");
        std::fs::write(&path, b"// generated\n").expect("write");

        assert!(
            !dir.path().join("src/generated/.gitignore").exists(),
            "pmat must not drop an ignore rule into a directory it does not own"
        );
        assert!(
            porcelain(dir.path()).contains("src/"),
            "a normal file must still show as untracked: {:?}",
            porcelain(dir.path())
        );
    }

    /// A `.gitignore` a project has edited is a decision. Re-running pmat must
    /// not silently revert it.
    #[test]
    fn an_existing_ignore_file_is_left_alone() {
        let dir = repo();
        let cache = ensure_cache_dir(dir.path());
        std::fs::write(cache.join(".gitignore"), b"# mine\n").expect("write");

        ensure_cache_dir(dir.path());

        assert_eq!(
            std::fs::read_to_string(cache.join(".gitignore")).expect("read"),
            "# mine\n"
        );
    }

    /// Idempotent, and creates what it promises.
    #[test]
    fn the_directory_and_its_rule_are_created_once() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        for _ in 0..3 {
            let cache = ensure_cache_dir(dir.path());
            assert!(cache.is_dir());
            assert_eq!(
                std::fs::read_to_string(cache.join(".gitignore")).expect("read"),
                PMAT_DIR_GITIGNORE
            );
        }
    }

    /// "Created once" has to mean the bytes are not re-written, not merely that
    /// they end up the same: an unconditional `fs::write` of identical content
    /// still moves the mtime, and pmat runs on every commit. Asked of the
    /// filesystem, which records modification times in nanoseconds.
    #[test]
    fn a_second_run_does_not_rewrite_the_rule() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let stamp = |p: &Path| {
            std::fs::metadata(p)
                .and_then(|m| m.modified())
                .expect("mtime")
        };

        let cache = ensure_cache_dir(dir.path()).join(".gitignore");
        let metrics = ensure_metrics_dir(dir.path()).join(".gitignore");
        let (before_cache, before_metrics) = (stamp(&cache), stamp(&metrics));

        for _ in 0..4 {
            ensure_cache_dir(dir.path());
            ensure_metrics_dir(dir.path());
        }

        assert_eq!(
            stamp(&cache),
            before_cache,
            "{} was rewritten",
            cache.display()
        );
        assert_eq!(
            stamp(&metrics),
            before_metrics,
            "{} was rewritten",
            metrics.display()
        );
    }
}
