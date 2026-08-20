//! Content-addressed proof that `cargo clippy` was green over an exact tree.
//!
//! # Why this exists
//!
//! `.git/hooks/pre-commit` ran format, complexity and SATD — and not clippy. So
//! commit `049a925a1` committed two clippy errors (`clippy::unnecessary_sort_by`
//! in `src/services/gate_effect/roster.rs`, `clippy::question_mark` in
//! `check_evidence_gates.rs`) through a hook that then printed
//! "✅ All quality gates passed!". Only a later human reviewer caught it
//! (`6285aaec6`).
//!
//! The obvious fix — run clippy in the hook — is unaffordable. Measured on this
//! crate, with a warm cache and **nothing changed at all**:
//!
//! | command                                          | wall clock |
//! |--------------------------------------------------|-----------|
//! | `cargo clippy --all-targets` (after --all-targets)| 1m 06s    |
//! | `cargo clippy --lib` (after --all-targets)        | 4m 09s    |
//! | `cargo clippy --lib` (after --lib, i.e. warm)     | 3m 11s    |
//!
//! Two things follow. First, a per-commit minute is exactly the cost that
//! trains `--no-verify`, which is worse than no hook. Second, the tempting
//! resolution — "scope clippy to the files in the commit" — is **falsified**:
//! clippy's unit of work is the crate, not the file, and narrowing the target
//! selection made it *slower*, because `--lib` and `--all-targets` resolve
//! features differently and thrash each other's cache.
//!
//! What is actually cheap is not re-deciding a question already decided. Clippy
//! is a pure function of (source tree, toolchain, lint flags). Hash those, and
//! a green verdict can be reused for exactly the tree it was taken on and no
//! other. Hashing every non-ignored file in this 60MB / 5,598-file repo costs
//! **0.2s**, and probing the toolchain costs **0.04s**.
//!
//! So the receipt is not a heuristic cache with a staleness window. It is a
//! content address: a different byte anywhere in the tree is a different key, and
//! a different key means clippy runs for real. There is no time-to-live to get
//! wrong, and no way to be handed a green answer for a tree nobody linted.
//!
//! # Why the key covers every file, not just the Rust ones
//!
//! The obvious economy — hash `*.rs` and the manifests, skip the other 1,239
//! files — is unsound in *this* crate, and demonstrably so. `include_str!`
//! makes non-Rust files compilation inputs, and pmat does that with markdown,
//! YAML, HTML, JS, shell and `.gitignore`:
//!
//! ```text
//! include_str!("../../../../../CHANGELOG.md")
//! include_str!("../../.gitignore")
//! include_str!("../../assets/dashboard.html")
//! include_str!("../../../prompts/debug.yaml")
//! include_str!("../../../.agents/hooks/pmat-quality-feedback.sh")
//! ```
//!
//! A "*.md cannot affect clippy" rule would therefore hand back a green verdict
//! for a tree whose `CHANGELOG.md` had changed under it. `build.rs` widens the
//! same hole further: it generates Rust into `OUT_DIR` from `templates/`, and
//! that generated code is linted too. Enumerating the real input set is a
//! research project with a wrong answer at the end of it, so the walk takes
//! everything git does not ignore.
//!
//! The price is paid in the safe direction: a docs-only edit invalidates the
//! receipt and buys a full clippy run it did not strictly need. Over-approximate
//! and it costs 90 seconds; under-approximate and the gate lies.

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// Where the receipt lives. `.pmat/` is gitignored (`.gitignore:268`), so a
/// receipt can never be committed, shared, or inherited from another machine.
const RECEIPT_RELPATH: &str = ".pmat/clippy-receipt";

/// Hidden files that change clippy's verdict and might still be missed by the
/// tree walk — `.cargo/config.toml` is commonly gitignored, and the walk honours
/// gitignore. Kept as belt-and-braces; entries are deduplicated below, so a path
/// the walk already reached costs nothing.
///
/// This list USED to be the only hidden input hashed at all, because the walk ran
/// with `.hidden(true)` and skipped every dot-path. That silently excluded 85 of
/// this repository's 88 tracked dot-paths, two of which are `include_str!`
/// COMPILATION inputs — `.agents/hooks/pmat-quality-feedback.sh`
/// (src/services/workspace_init/templates.rs) and `.gitignore`
/// (this file's own tests). Deleting either breaks the build while leaving the
/// fingerprint byte-identical, so `pmat verify --stage clippy` returned a cached
/// green for a tree that does not compile.
const HIDDEN_INPUTS: &[&str] = &[".cargo/config.toml", ".cargo/config", ".clippy.toml"];

/// Set to any non-empty value to force a real clippy run even when a receipt
/// matches. Deliberately one-directional: it can only make the gate do *more*
/// work. There is no variable that makes it do less — that is what `--no-verify`
/// is for, and that at least leaves a trace in the operator's shell history.
const FORCE_ENV: &str = "PMAT_VERIFY_NO_CACHE";

/// The inputs clippy's verdict depends on, reduced to one hex digest.
///
/// `flags` is the exact lint invocation (targets + `-D`/`-A` list) so that
/// changing what we ask clippy invalidates every receipt taken under the old
/// question.
pub fn fingerprint(project: &Path, flags: &str) -> Result<String> {
    let mut hasher = Sha256::new();
    hasher.update(b"pmat-clippy-receipt-v1\n");
    hasher.update(flags.as_bytes());
    hasher.update(b"\n");
    hasher.update(toolchain_id().as_bytes());
    hasher.update(b"\n");

    let mut entries: Vec<(String, [u8; 32])> = Vec::new();
    // `.hidden(false)`: dotfiles are compilation inputs here, not noise. The
    // module's contract is "a different byte anywhere in the tree is a different
    // key", and `.hidden(true)` made that false for every dot-path.
    //
    // `.git/` is excluded explicitly because it is enormous and changes on every
    // git operation, which would make the receipt never match. `.pmat/` needs no
    // rule: it is gitignored (`**/.pmat/`), so `git_ignore(true)` already drops
    // it — which also means the receipt this function writes cannot become an
    // input to its own fingerprint.
    for entry in ignore::WalkBuilder::new(project)
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .filter_entry(|e| e.file_name() != std::ffi::OsStr::new(".git"))
        .build()
        .filter_map(std::result::Result::ok)
    {
        if !entry.file_type().is_some_and(|t| t.is_file()) {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(project)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .into_owned();
        entries.push((rel, file_digest(entry.path())));
    }
    for hidden in HIDDEN_INPUTS {
        let path = project.join(hidden);
        if path.is_file() {
            entries.push(((*hidden).to_string(), file_digest(&path)));
        }
    }

    // Sort: the walk order is filesystem-dependent, and a fingerprint that
    // changes with directory iteration order would invalidate itself at random.
    entries.sort_unstable();
    // HIDDEN_INPUTS may name a path the walk already reached; hashing it twice
    // would still be deterministic, but deduplicating keeps the key equal to
    // "the set of input files", which is what the doc above claims it is.
    entries.dedup();
    for (rel, digest) in entries {
        hasher.update(rel.as_bytes());
        hasher.update(b"\0");
        hasher.update(digest);
    }
    Ok(hex(&hasher.finalize()))
}

/// SHA-256 of a file's bytes; an unreadable file hashes to a distinct sentinel
/// rather than being skipped, so "I could not read this" is itself part of the
/// key instead of silently matching a tree where the file was readable.
fn file_digest(path: &Path) -> [u8; 32] {
    match std::fs::read(path) {
        Ok(bytes) => Sha256::digest(&bytes).into(),
        Err(e) => Sha256::digest(format!("<unreadable: {e}>").as_bytes()).into(),
    }
}

/// Toolchain identity. A clippy upgrade adds and removes lints, so a receipt
/// taken on 0.1.97 says nothing about 0.1.98.
fn toolchain_id() -> String {
    let probe = |bin: &str, args: &[&str]| -> Option<String> {
        let out = std::process::Command::new(bin).args(args).output().ok()?;
        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
    };
    let clippy = probe("cargo", &["clippy", "--version"])
        .filter(|s| !s.is_empty())
        // No clippy is not "no toolchain constraint" — it must never collide
        // with a key taken when clippy was present.
        .unwrap_or_else(|| "<clippy-unavailable>".to_string());
    let rustc = probe("rustc", &["--version"]).unwrap_or_else(|| "<rustc-unavailable>".to_string());
    format!("{clippy}|{rustc}")
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    bytes.iter().fold(String::new(), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    })
}

/// Path of the receipt for `project`.
pub fn receipt_path(project: &Path) -> PathBuf {
    project.join(RECEIPT_RELPATH)
}

/// Is there a receipt proving this exact tree already linted green?
///
/// Any doubt answers `false`: no receipt, unreadable receipt, different key, or
/// `PMAT_VERIFY_NO_CACHE` set. There is no path through this function that
/// returns `true` without having read a stored key equal to the one just
/// computed from the tree on disk.
pub fn is_proven(project: &Path, fingerprint: &str) -> bool {
    is_proven_when(project, fingerprint, force_requested())
}

fn force_requested() -> bool {
    std::env::var_os(FORCE_ENV).is_some_and(|v| !v.is_empty())
}

/// The decision itself, with the environment lifted into an argument so it can
/// be exercised without `set_var` (process-global, and this suite is threaded).
fn is_proven_when(project: &Path, fingerprint: &str, force: bool) -> bool {
    if force || fingerprint.is_empty() {
        return false;
    }
    match std::fs::read_to_string(receipt_path(project)) {
        Ok(stored) => stored.trim() == fingerprint,
        Err(_) => false,
    }
}

/// Record that clippy was green over the tree with this fingerprint.
pub fn record(project: &Path, fingerprint: &str) -> Result<()> {
    let path = receipt_path(project);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)
            .with_context(|| format!("creating {} for the clippy receipt", dir.display()))?;
    }
    std::fs::write(&path, format!("{fingerprint}\n"))
        .with_context(|| format!("writing clippy receipt to {}", path.display()))
}

/// Drop any receipt. Called when clippy comes back red, so a tree that was
/// green, went red, and came back to a previously-green state cannot be waved
/// through by a stale key.
pub fn revoke(project: &Path) {
    let _ = std::fs::remove_file(receipt_path(project));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn scratch() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(dir.path().join("Cargo.toml"), "[package]\nname='x'\n").expect("write");
        fs::write(dir.path().join("lib.rs"), "pub fn a() {}\n").expect("write");
        dir
    }

    /// The whole safety argument: a changed byte is a changed key.
    #[test]
    fn a_changed_source_byte_changes_the_fingerprint() {
        let dir = scratch();
        let before = fingerprint(dir.path(), "flags").expect("fingerprint");
        fs::write(dir.path().join("lib.rs"), "pub fn a() {}\npub fn b() {}\n").expect("write");
        let after = fingerprint(dir.path(), "flags").expect("fingerprint");
        assert_ne!(
            before, after,
            "a source edit must produce a different key, or clippy's verdict \
             would be reused for a tree it never saw"
        );
    }

    /// A *new* file must invalidate too. Hashing only the files a previous run
    /// knew about would let an added module ride in on an old receipt.
    #[test]
    fn an_added_file_changes_the_fingerprint() {
        let dir = scratch();
        let before = fingerprint(dir.path(), "flags").expect("fingerprint");
        fs::write(dir.path().join("extra.rs"), "pub fn c() {}\n").expect("write");
        assert_ne!(
            before,
            fingerprint(dir.path(), "flags").expect("fingerprint"),
            "an added file must change the key"
        );
    }

    /// A hidden file is a compilation input, not noise.
    ///
    /// The walk ran with `.hidden(true)`, so every dot-path was excluded from
    /// the key except the three in `HIDDEN_INPUTS`. In this repository that
    /// silently dropped 85 of 88 tracked dot-paths, two of them `include_str!`
    /// inputs whose deletion stops the crate compiling — and the receipt would
    /// still have matched, so `pmat verify --stage clippy` returned green for a
    /// tree that does not build.
    ///
    /// Three cases, because the class is "any dot-path", not "the file I
    /// happened to name": a nested hidden file, a top-level dotfile, and a file
    /// inside a hidden directory.
    #[test]
    fn a_changed_hidden_file_changes_the_fingerprint() {
        for rel in [
            ".agents/hooks/hook.sh",
            ".gitattributes",
            ".config/thing.toml",
        ] {
            let dir = scratch();
            let path = dir.path().join(rel);
            fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
            fs::write(&path, "before\n").expect("write");
            let before = fingerprint(dir.path(), "flags").expect("fingerprint");

            fs::write(&path, "after\n").expect("rewrite");
            assert_ne!(
                before,
                fingerprint(dir.path(), "flags").expect("fingerprint"),
                "editing {rel} must change the key"
            );

            fs::remove_file(&path).expect("rm");
            assert_ne!(
                before,
                fingerprint(dir.path(), "flags").expect("fingerprint"),
                "deleting {rel} must change the key"
            );
        }
    }

    /// The counter-test: `.git/` must NOT be in the key. It changes on every git
    /// operation, so hashing it would mean no receipt ever matched — a cache
    /// that never hits is as useless as one that always does, and would push
    /// people to disable the gate.
    #[test]
    fn git_internals_are_not_part_of_the_fingerprint() {
        let dir = scratch();
        let git = dir.path().join(".git");
        fs::create_dir_all(&git).expect("mkdir .git");
        fs::write(git.join("HEAD"), "ref: refs/heads/a\n").expect("write");
        let before = fingerprint(dir.path(), "flags").expect("fingerprint");
        fs::write(git.join("HEAD"), "ref: refs/heads/b\n").expect("rewrite");
        assert_eq!(
            before,
            fingerprint(dir.path(), "flags").expect("fingerprint"),
            "a git operation must not invalidate the receipt"
        );
    }

    /// Two files swapping contents must not hash the same. This is why the path
    /// is fed to the hasher next to its digest rather than the digests alone.
    #[test]
    fn swapping_two_files_contents_changes_the_fingerprint() {
        let dir = scratch();
        fs::write(dir.path().join("a.rs"), "AAA").expect("write");
        fs::write(dir.path().join("b.rs"), "BBB").expect("write");
        let before = fingerprint(dir.path(), "flags").expect("fingerprint");
        fs::write(dir.path().join("a.rs"), "BBB").expect("write");
        fs::write(dir.path().join("b.rs"), "AAA").expect("write");
        assert_ne!(
            before,
            fingerprint(dir.path(), "flags").expect("fingerprint"),
            "path-blind hashing would call these two trees identical"
        );
    }

    /// Asking clippy a different question invalidates answers to the old one.
    #[test]
    fn changing_the_lint_flags_changes_the_fingerprint() {
        let dir = scratch();
        assert_ne!(
            fingerprint(dir.path(), "-D warnings").expect("fingerprint"),
            fingerprint(dir.path(), "-D warnings -D clippy::pedantic").expect("fingerprint"),
        );
    }

    /// Same tree, same question, same key — otherwise the receipt never hits and
    /// the gate costs a minute every commit, which is the failure mode that
    /// makes people reach for `--no-verify`.
    #[test]
    fn an_unchanged_tree_reproduces_its_fingerprint() {
        let dir = scratch();
        assert_eq!(
            fingerprint(dir.path(), "flags").expect("fingerprint"),
            fingerprint(dir.path(), "flags").expect("fingerprint"),
        );
    }

    #[test]
    fn a_recorded_fingerprint_is_proven_and_others_are_not() {
        let dir = scratch();
        assert!(!is_proven(dir.path(), "abc"), "no receipt must not prove");
        record(dir.path(), "abc").expect("record");
        assert!(is_proven(dir.path(), "abc"));
        assert!(
            !is_proven(dir.path(), "def"),
            "a receipt for another tree must not prove this one"
        );
    }

    /// Red must erase the proof. Without this, a tree that was green, was
    /// edited red, and was edited back could be waved through on the old key —
    /// which is fine — but a *revoke-less* implementation also leaves the
    /// receipt in place across the red state, and any bug that mis-computes the
    /// key then reads as green.
    #[test]
    fn revoke_removes_the_receipt() {
        let dir = scratch();
        record(dir.path(), "abc").expect("record");
        revoke(dir.path());
        assert!(!is_proven(dir.path(), "abc"));
        assert!(!receipt_path(dir.path()).exists());
    }

    /// An empty fingerprint is a computation that failed, not a tree that
    /// matched. It must never satisfy an empty receipt file.
    #[test]
    fn an_empty_fingerprint_never_proves_anything() {
        let dir = scratch();
        fs::create_dir_all(dir.path().join(".pmat")).expect("mkdir");
        fs::write(receipt_path(dir.path()), "\n").expect("write");
        assert!(!is_proven(dir.path(), ""));
    }

    /// The force switch may only add work: it can turn a hit into a miss, and
    /// there is no input that turns a miss into a hit.
    #[test]
    fn forcing_turns_a_hit_into_a_miss_and_never_the_reverse() {
        let dir = scratch();
        record(dir.path(), "abc").expect("record");
        assert!(is_proven_when(dir.path(), "abc", false), "matching receipt");
        assert!(
            !is_proven_when(dir.path(), "abc", true),
            "forcing must re-run clippy even when the receipt matches"
        );
        // No receipt at all: neither setting can conjure a pass.
        revoke(dir.path());
        assert!(!is_proven_when(dir.path(), "abc", false));
        assert!(!is_proven_when(dir.path(), "abc", true));
        assert_eq!(
            FORCE_ENV, "PMAT_VERIFY_NO_CACHE",
            "the documented variable is the one that is read"
        );
    }

    /// The receipt must be unshareable between machines and un-committable.
    #[test]
    fn the_receipt_lives_under_a_gitignored_directory() {
        assert!(RECEIPT_RELPATH.starts_with(".pmat/"));
        let ignore_rules = include_str!("../../.gitignore");
        assert!(
            ignore_rules.lines().any(|l| l.trim() == "**/.pmat/"),
            ".pmat/ must stay gitignored or receipts become shareable artifacts"
        );
    }
}
