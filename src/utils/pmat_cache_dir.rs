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
//!
//! # And one directory that is not in the project at all
//!
//! The second half of this module ([`user_cache_root`] downward) is the case
//! where "keep it out of git status" is not enough and the state must not be in
//! the tree in the first place: an AUDIT, which has to run on a checkout it may
//! not write to. That state is keyed by project under the user's cache
//! directory — see [`comply_state_dir`] for why, and #1008 for what it cost.

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

// ─────────────────────────────────────────────────────────────────────────────
// The other half of the rule: state that must live OUTSIDE the audited tree
// ─────────────────────────────────────────────────────────────────────────────
//
// Everything above keeps pmat's writes out of a project's `git status`. That is
// the right answer for a cache a project asked for. It is the WRONG answer for
// a cache an AUDIT needs, because an audit must be able to run on a tree it may
// not write to at all — a fresh CI checkout, a read-only mount, someone else's
// repository — and `comply check` answered that by declining to measure:
//
// ```text
// - CB-200: TDG Grade Gate: Not measured: no .pmat/context.db. `comply check`
//   will not build one - building writes .pmat/context.db and .pmat/context.idx
//   into the project being audited.
// ```
//
// Honest, and unenforceable (#1008). A fresh checkout never has an index, so
// CB-200 could only ever fire on a developer's machine — the one place where
// failing it decides nothing. Measured A/B on one tree at one commit, the index
// the only variable: 2 failing checks without it, 3 with.
//
// A gate that cannot run where merges are decided is not a gate. So the audit's
// state goes where the audit is allowed to write: the user's cache directory,
// keyed by project. CB-081 already did exactly this for the same reason (#939)
// and the helpers below are that scheme, stated once instead of twice.

/// Environment override for [`user_cache_root`].
///
/// Set it to place pmat's per-user cache somewhere a CI job can restore between
/// runs — an index that survives is the difference between a gate that costs a
/// rebuild every run and one that costs a stat.
pub const CACHE_DIR_ENV: &str = "PMAT_CACHE_DIR";

/// The one directory name pmat owns under the platform cache root.
const CACHE_NAMESPACE: &str = "paiml-mcp-agent-toolkit";

/// The root of pmat's per-user cache: derived state that belongs to the MACHINE
/// and not to any project on it.
///
/// `$PMAT_CACHE_DIR`, else the platform cache dir (`$XDG_CACHE_HOME`,
/// `~/Library/Caches`, `%LOCALAPPDATA%`), else `~/.cache`, else the temp dir.
/// The last two fallbacks matter: a container with neither `HOME` nor
/// `XDG_CACHE_HOME` set must still get a usable path, because the alternative
/// is a check that reports "unmeasured" for a reason that has nothing to do
/// with the code it was asked about.
#[must_use]
pub fn user_cache_root() -> PathBuf {
    if let Some(raw) = std::env::var_os(CACHE_DIR_ENV) {
        if !raw.is_empty() {
            return PathBuf::from(raw);
        }
    }
    dirs::cache_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join(".cache")))
        .unwrap_or_else(std::env::temp_dir)
        .join(CACHE_NAMESPACE)
}

/// A directory name that identifies one project, readably and without
/// collisions: `<basename>-<16 hex digits of its canonical path>`.
///
/// The hash is FNV-1a, written out, and that is deliberate. The scheme this
/// generalises used `DefaultHasher`, whose docs say in terms that its output
/// "is not guaranteed to be equal across Rust releases" — so a toolchain bump
/// silently renames every project's cache directory. For a dependency count
/// that costs a re-read of `Cargo.lock`. For an INDEX it costs a multi-minute
/// rebuild that looks, from the outside, exactly like a hang, on a machine
/// where nothing about the project changed. A cache key has to be a function
/// of the input alone.
///
/// The basename is kept in front so a human can read `~/.cache/…/comply/index/`
/// and see whose index is whose; it is sanitised because it becomes a path
/// component, and the hash — over the canonical path, not the basename — is
/// what actually distinguishes two projects that share a name.
#[must_use]
pub fn project_cache_key(project_path: &Path) -> String {
    let canonical =
        std::fs::canonicalize(project_path).unwrap_or_else(|_| project_path.to_path_buf());
    let name: String = canonical
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project")
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .take(40)
        .collect();
    let name = if name.is_empty() {
        "project".to_string()
    } else {
        name
    };
    format!(
        "{name}-{:016x}",
        fnv1a64(canonical.as_os_str().as_encoded_bytes())
    )
}

/// FNV-1a, 64-bit. Stable across Rust releases, machines and pmat versions,
/// which is the only property this use needs of it.
fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// Where a comply check keeps `component`'s state for `project_path`, outside
/// that project.
///
/// `<user cache>/comply/<component>/<project key>` — the layout CB-081 has
/// used since #939, now with one implementation instead of one per check.
#[must_use]
pub fn comply_state_dir(project_path: &Path, component: &str) -> PathBuf {
    user_cache_root()
        .join("comply")
        .join(component)
        .join(project_cache_key(project_path))
}

/// The agent-context index `comply check` reads when the audited project has
/// none of its own, and builds there when it has none at all.
///
/// Same filenames as the in-project index — `context.idx/` beside `context.db`,
/// because `AgentContextIndex::save` derives the second from the first — so
/// this is the same artifact in a different place, not a second format.
#[must_use]
pub fn comply_index_path(project_path: &Path) -> PathBuf {
    comply_state_dir(project_path, "index").join("context.idx")
}

/// How long an unused per-project entry is kept before the next write reclaims
/// it.
///
/// A number is needed because these entries are not small: pmat's own
/// agent-context index measures 79 MB, and pmat is one repository of a fleet.
/// The key is derived from a project's canonical path, so an entry is orphaned
/// the moment that path goes away — a deleted checkout, a worktree, and above
/// all a temporary directory: two runs of this repository's own test suite left
/// 15 entries behind, each for a `TempDir` that no longer existed by the time
/// the run finished.
///
/// 30 days rather than something clever, because the cost of being wrong is
/// exactly one rebuild and the cost of never sweeping is a directory that only
/// grows.
pub const STATE_MAX_IDLE: std::time::Duration = std::time::Duration::from_secs(30 * 24 * 60 * 60);

/// Delete per-project entries of `component` that nothing has written in
/// `max_idle`, returning how many were removed.
///
/// Best-effort throughout: a cache that cannot be tidied must never fail the
/// analysis that tried. Called only from the paths that WRITE, so the sweep
/// costs one `read_dir` on the rare run that rebuilds and nothing at all on the
/// runs that hit the cache.
///
/// The clock is on the last WRITE, not the last read — a directory's mtime does
/// not move when a file inside it is opened. So a project that has not changed
/// in a month has its index reclaimed and pays one rebuild for it, which is the
/// side to be wrong on.
pub fn sweep_idle_state(component: &str, max_idle: std::time::Duration) -> usize {
    let dir = user_cache_root().join("comply").join(component);
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return 0;
    };
    let now = std::time::SystemTime::now();
    let mut removed = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let idle = entry
            .metadata()
            .and_then(|m| m.modified())
            .ok()
            .and_then(|m| now.duration_since(m).ok());
        if idle.is_some_and(|idle| idle > max_idle) && std::fs::remove_dir_all(&path).is_ok() {
            removed += 1;
        }
    }
    removed
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_user_cache {
    use super::*;

    /// `$PMAT_CACHE_DIR`, restored on drop so a failing assertion cannot leak
    /// it into the rest of the suite.
    struct CacheDirGuard(Option<std::ffi::OsString>);

    impl CacheDirGuard {
        fn set(value: Option<&Path>) -> Self {
            let previous = std::env::var_os(CACHE_DIR_ENV);
            match value {
                Some(v) => std::env::set_var(CACHE_DIR_ENV, v),
                None => std::env::remove_var(CACHE_DIR_ENV),
            }
            Self(previous)
        }
    }

    impl Drop for CacheDirGuard {
        fn drop(&mut self) {
            match self.0.take() {
                Some(v) => std::env::set_var(CACHE_DIR_ENV, v),
                None => std::env::remove_var(CACHE_DIR_ENV),
            }
        }
    }

    /// The published FNV-1a/64 vectors. This is a cache KEY: if it ever moves,
    /// every project on the machine silently loses its index and pays for a
    /// rebuild that looks like a hang. Pinned to the standard so that "the hash
    /// changed" can never be something a reader has to deduce from a slow run.
    #[test]
    fn the_key_hash_is_fnv1a_and_stays_fnv1a() {
        assert_eq!(fnv1a64(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv1a64(b"a"), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(fnv1a64(b"foobar"), 0x8594_4171_f739_67e8);
    }

    /// The reason the hash is there at all: `~/src/foo` and `~/work/foo` are
    /// two projects, and an index built from one must never be reported as a
    /// measurement of the other.
    #[test]
    fn two_projects_sharing_a_basename_get_different_directories() {
        let root = tempfile::TempDir::new().expect("tempdir");
        let (a, b) = (root.path().join("src/api"), root.path().join("work/api"));
        std::fs::create_dir_all(&a).expect("mkdir");
        std::fs::create_dir_all(&b).expect("mkdir");

        let (ka, kb) = (project_cache_key(&a), project_cache_key(&b));
        assert_ne!(ka, kb, "two projects collided on one cache directory");
        // Counter-test: it is still a key, not a nonce. The same project asked
        // twice — and asked by a path that needs canonicalising — is one entry.
        assert_eq!(ka, project_cache_key(&a));
        assert_eq!(ka, project_cache_key(&root.path().join("src/./api")));
        assert!(ka.starts_with("api-"), "the name stays readable: {ka}");
    }

    /// A basename is a path component here, so it may not smuggle in a
    /// separator or a `..`; the hash, not the name, is what distinguishes.
    #[test]
    fn a_hostile_basename_cannot_escape_the_cache_directory() {
        let root = tempfile::TempDir::new().expect("tempdir");
        let odd = root.path().join("a b/../c#d");
        std::fs::create_dir_all(root.path().join("c#d")).expect("mkdir");
        let key = project_cache_key(&odd);
        assert!(
            !key.contains('/') && !key.contains("..") && !key.contains('#'),
            "cache key is a single sanitised path component, got {key}"
        );
    }

    #[test]
    #[serial_test::serial]
    fn the_environment_moves_the_whole_cache_and_nothing_else() {
        let elsewhere = tempfile::TempDir::new().expect("tempdir");
        let _guard = CacheDirGuard::set(Some(elsewhere.path()));
        assert_eq!(user_cache_root(), elsewhere.path());

        let project = tempfile::TempDir::new().expect("tempdir");
        let index = comply_index_path(project.path());
        assert!(
            index.starts_with(elsewhere.path()),
            "{} is not under {}",
            index.display(),
            elsewhere.path().display()
        );
        // The audited project is never part of the answer: that is the whole
        // point of asking this module rather than joining ".pmat" by hand.
        assert!(
            !index.starts_with(project.path()),
            "the audited tree got the index anyway: {}",
            index.display()
        );
    }

    /// An empty value is not a location. Left as an override it would send
    /// every cache to the filesystem root.
    ///
    /// `#[serial]`, like every test here that touches `$PMAT_CACHE_DIR`: an
    /// environment variable is process-global, and two of these running at once
    /// read each other's value — which is exactly how this pair first failed,
    /// one of them asserting the default and getting the other's `/cache-root`.
    #[test]
    #[serial_test::serial]
    fn an_empty_override_falls_back_instead_of_rooting_the_cache() {
        let _guard = CacheDirGuard::set(Some(Path::new("")));
        let root = user_cache_root();
        assert_ne!(root, Path::new(""));
        assert!(
            root.ends_with("paiml-mcp-agent-toolkit"),
            "expected the namespaced default, got {}",
            root.display()
        );
    }

    /// One scheme, not one per check: CB-081's dependency cache and CB-200's
    /// index are two components of the same per-project directory. The layout
    /// is asserted because CB-081 has been living at it since #939 and moving
    /// it silently would orphan every existing cache.
    #[test]
    #[serial_test::serial]
    fn every_comply_component_shares_one_per_project_layout() {
        let _guard = CacheDirGuard::set(Some(Path::new("/cache-root")));
        let project = tempfile::TempDir::new().expect("tempdir");
        let key = project_cache_key(project.path());

        assert_eq!(
            comply_state_dir(project.path(), "cb081"),
            PathBuf::from("/cache-root/comply/cb081").join(&key)
        );
        assert_eq!(
            comply_index_path(project.path()),
            PathBuf::from("/cache-root/comply/index")
                .join(&key)
                .join("context.idx")
        );
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_sweep {
    use super::*;

    fn aged(dir: &Path, name: &str, age: std::time::Duration) -> PathBuf {
        let entry = dir.join(name);
        std::fs::create_dir_all(&entry).expect("mkdir");
        std::fs::write(entry.join("context.db"), b"x").expect("write");
        let when = std::time::SystemTime::now() - age;
        std::fs::File::options()
            .write(true)
            .open(&entry)
            .or_else(|_| std::fs::File::open(&entry))
            .and_then(|f| f.set_modified(when))
            .expect("age the entry");
        entry
    }

    /// An entry nobody has written in a month is reclaimed — 79 MB of it, in
    /// pmat's own case — and one written yesterday is NOT.
    ///
    /// The second half is the counter-test, and it is the one that matters: a
    /// sweep that removes everything would turn every audit into a rebuild, so
    /// "the cache is tidy" and "the cache is empty" must not be the same
    /// outcome.
    #[test]
    #[serial_test::serial]
    fn only_the_idle_entries_are_reclaimed() {
        let root = tempfile::TempDir::new().expect("tempdir");
        let previous = std::env::var_os(CACHE_DIR_ENV);
        std::env::set_var(CACHE_DIR_ENV, root.path());

        let component = root.path().join("comply").join("index");
        std::fs::create_dir_all(&component).expect("mkdir");
        let day = std::time::Duration::from_secs(24 * 60 * 60);
        let stale = aged(&component, "gone-1111111111111111", 40 * day);
        let live = aged(&component, "here-2222222222222222", day);

        let removed = sweep_idle_state("index", STATE_MAX_IDLE);

        match previous {
            Some(v) => std::env::set_var(CACHE_DIR_ENV, v),
            None => std::env::remove_var(CACHE_DIR_ENV),
        }
        assert_eq!(removed, 1, "exactly the idle entry");
        assert!(!stale.exists(), "an idle entry must be reclaimed");
        assert!(
            live.join("context.db").exists(),
            "a live entry must survive, or every audit becomes a rebuild"
        );
    }

    /// A cache directory that does not exist yet is not an error — the sweep
    /// runs on the way to CREATING it.
    #[test]
    #[serial_test::serial]
    fn sweeping_an_absent_cache_is_a_no_op() {
        let root = tempfile::TempDir::new().expect("tempdir");
        let previous = std::env::var_os(CACHE_DIR_ENV);
        std::env::set_var(CACHE_DIR_ENV, root.path().join("not-created-yet"));
        let removed = sweep_idle_state("index", STATE_MAX_IDLE);
        match previous {
            Some(v) => std::env::set_var(CACHE_DIR_ENV, v),
            None => std::env::remove_var(CACHE_DIR_ENV),
        }
        assert_eq!(removed, 0);
    }
}
