//! The `.pmat/` directory pmat writes into the project it is analysing, and the
//! ignore rule that keeps it out of that project's git status.
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

use std::path::{Path, PathBuf};

/// What pmat writes into `.pmat/.gitignore`.
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
# Written by pmat. Everything under .pmat/ is derived from the tree and rebuilt
# on demand: caches, indexes, tiered stores, baselines, backups.
#
# Kept here rather than in the project's own .gitignore because these filenames
# are pmat's to change — the rule projects were once told to add,
# `.pmat/dead-code-cache.json`, stopped matching the day the cache key gained a
# scope-and-depth suffix, in silence, and nothing on either side could notice.
*
";

/// The `.pmat/` directory of `project_root`.
#[must_use]
pub fn pmat_dir(project_root: &Path) -> PathBuf {
    project_root.join(".pmat")
}

/// Create `<project_root>/.pmat/` if absent and make sure it ignores itself.
///
/// Call this instead of `create_dir_all(project.join(".pmat"))` at every site
/// that writes a cache into an analysed project. Best-effort by design: a
/// read-only checkout must not turn "I could not write a `.gitignore`" into a
/// failed analysis, and the caller's own write will report the real problem.
///
/// Never overwrites an existing `.gitignore` — a project that has edited one is
/// making a decision, and silently replacing it would be the tool dirtying the
/// tree in a second way.
pub fn ensure_cache_dir(project_root: &Path) -> PathBuf {
    let dir = pmat_dir(project_root);
    let _ = std::fs::create_dir_all(&dir);
    let ignore = dir.join(".gitignore");
    if !ignore.exists() {
        let _ = std::fs::write(&ignore, PMAT_DIR_GITIGNORE);
    }
    dir
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
        git(dir.path(), &["add", "-f", ".pmat/baseline.json"]);
        // `--no-verify`: a global `core.hooksPath` would otherwise run the
        // developer's own pre-commit gates against this two-file fixture.
        let out = git(
            dir.path(),
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

        ensure_cache_dir(dir.path());
        std::fs::write(pmat.join("baseline.json"), b"{\"changed\":1}").expect("write");

        let status = String::from_utf8_lossy(&git(dir.path(), &["status", "--porcelain"]).stdout)
            .into_owned();
        assert!(
            status.contains(".pmat/baseline.json"),
            "an edit to a TRACKED file must still show up: {status:?}"
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
}
