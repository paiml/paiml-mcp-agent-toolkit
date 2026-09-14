//! #1350 — `pmat --version` must identify the binary, not shrug at it.
//!
//! `3.40.0` names two different CLIs (#1349): the published crate has no
//! `work add --github-issue`, master does. The banner is the only thing that
//! could tell them apart, and it printed `commit: unknown / worktree: unknown`
//! — a shape that reads like two fields which failed to populate rather than a
//! statement that a `.crate` tarball carries no git metadata.
//!
//! This is the same blindness forjar's `scripts/ratchets/cb21xx-baseline.json`
//! already records against this tool: the check roster went 172 -> 166 with six
//! checks vanishing and a retired one live again, while `pmat --version` read
//! 3.40.0 throughout and the banner said `unknown`.
//!
//! These cases fail on the pre-#1350 binary: it emits no `source:` line at all.

#[path = "support/pmat_cmd.rs"]
mod pmat_cmd;

fn version_output() -> String {
    // pmat_cmd::pmat(), not Command::new: src/services/test_env_hygiene.rs refuses
    // an unledgered raw spawn, because several ambient variables change what the
    // binary DOES and a test that inherits them is measuring the shell it ran in.
    // It caught this file on its first CI run.
    let out = pmat_cmd::pmat()
        .arg("--version")
        .output()
        .expect("the built binary runs");
    assert!(out.status.success(), "--version exited {:?}", out.status);
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// The banner names the source of the build, always.
///
/// Without this line a reader cannot tell a registry build from a git build, and
/// that is exactly the pair `3.40.0` currently spans.
#[test]
fn the_banner_names_where_the_build_came_from() {
    let v = version_output();
    let source = v
        .lines()
        .find_map(|l| l.strip_prefix("source: "))
        .unwrap_or_else(|| panic!("no `source:` line in --version output:\n{v}"));
    assert!(
        source == "git checkout"
            || source == "source archive"
            || source == "source archive; commit supplied by the builder",
        "unrecognised build source {source:?}; add the case here rather than widening the check"
    );
}

/// No field is the bare word `unknown`.
///
/// An absence that is genuinely unknowable — a tarball has no git — is a FACT
/// and says so. `unknown` is what a field prints when nobody decided what it
/// should say, and the reader cannot tell the two apart.
#[test]
fn no_provenance_field_is_the_bare_word_unknown() {
    let v = version_output();
    for line in v.lines() {
        for key in ["commit: ", "worktree: ", "source: "] {
            if let Some(value) = line.strip_prefix(key) {
                assert_ne!(
                    value.trim(),
                    "unknown",
                    "`{key}` is the bare word `unknown`. Say which case this is \
                     (git checkout, source archive, or baked) — the whole point of \
                     #1350 is that a reader can tell them apart.\n{v}"
                );
            }
        }
    }
}

/// The version stays the FIRST token, so `-V` parsers and every assertion of the
/// form "output contains <version>" keep working.
#[test]
fn the_version_is_still_the_first_token() {
    let v = version_output();
    let first = v.lines().next().expect("non-empty --version output");
    assert_eq!(
        first.trim(),
        format!("pmat {}", env!("CARGO_PKG_VERSION")),
        "the first line must stay `pmat <version>`"
    );
}

/// A git build names its commit and its worktree state.
///
/// Skipped rather than failed off a git checkout: the assertion is about what a
/// git build promises, and a tarball build cannot keep that promise by
/// construction. The skip is printed so a green run cannot hide it.
#[test]
fn a_git_build_names_its_commit() {
    let v = version_output();
    let source = v
        .lines()
        .find_map(|l| l.strip_prefix("source: "))
        .unwrap_or_default()
        .to_string();
    if source != "git checkout" {
        eprintln!("SKIP a_git_build_names_its_commit: source is {source:?}, not a git checkout");
        return;
    }
    let commit = v
        .lines()
        .find_map(|l| l.strip_prefix("commit: "))
        .expect("a git build has a commit line");
    assert_eq!(commit.len(), 40, "expected a full sha, got {commit:?}");
    assert!(
        commit.chars().all(|c| c.is_ascii_hexdigit()),
        "expected hex, got {commit:?}"
    );
    let worktree = v
        .lines()
        .find_map(|l| l.strip_prefix("worktree: "))
        .expect("a git build has a worktree line");
    assert!(
        worktree == "clean" || worktree == "dirty",
        "a git build's worktree is clean or dirty, got {worktree:?}"
    );
}
