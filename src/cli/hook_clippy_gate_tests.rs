//! End-to-end replay of the defect the pre-commit clippy gate exists to catch.
//!
//! PMAT-630 (#1034, CASE 1). On `049a925a1` an agent committed two clippy
//! errors and the hook answered "✅ All quality gates passed!":
//!
//! ```text
//! src/services/gate_effect/roster.rs:108          clippy::unnecessary_sort_by
//! .../check_handlers/check_evidence_gates.rs:291  clippy::question_mark
//! ```
//!
//! (The brief for this work said both errors were in `roster.rs`. They were
//! not — the second is in `check_evidence_gates.rs`, per `6285aaec6`'s own
//! message and diffstat. Both are replayed below.)
//!
//! These tests are not synthetic. `historical_roster_line` and
//! `historical_slug_parser` are the code as it stood at `049a925a1`, lifted
//! from `git show 049a925a1:src/services/gate_effect/roster.rs` and from
//! `6285aaec6`'s reversed diff hunk.
//!
//! Registered in `lib.rs` on purpose: `autotests = false` (`Cargo.toml:30`)
//! means a test file nobody declares is silently never compiled, and a gate
//! whose own tests do not run is the failure it exists to prevent.
//! `cargo test --lib -- hook_clippy_gate` lists them.

use std::path::Path;
use std::process::Command;

use crate::cli::verify::{CLIPPY_LINTS, CLIPPY_TARGETS};

/// `src/services/gate_effect/roster.rs:108` as committed by `049a925a1`.
const HISTORICAL_ROSTER_LINE: &str = "rules.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));";

/// The fix `6285aaec6` applied to it.
const FIXED_ROSTER_LINE: &str = "rules.sort_by_key(Rule::sort_key);";

fn roster_fixture(sort_line: &str) -> String {
    format!(
        "pub struct Rule {{ pub key: String }}\n\
         impl Rule {{ pub fn sort_key(&self) -> String {{ self.key.clone() }} }}\n\
         pub fn collect(mut rules: Vec<Rule>) -> Vec<Rule> {{\n    {sort_line}\n    rules\n}}\n"
    )
}

/// `parse_github_slug` as committed by `049a925a1` (the `clippy::question_mark`
/// error), and as `6285aaec6` left it.
const HISTORICAL_SLUG_PARSER: &str = r#"
pub fn parse_github_slug(url: &str) -> Option<String> {
    let rest = if let Some(r) = url.strip_prefix("https://github.com/") {
        r
    } else if let Some(r) = url.strip_prefix("ssh://git@github.com/") {
        r
    } else {
        return None;
    };
    Some(rest.trim_end_matches('/').to_string())
}
"#;

const FIXED_SLUG_PARSER: &str = r#"
pub fn parse_github_slug(url: &str) -> Option<String> {
    let rest = if let Some(r) = url.strip_prefix("https://github.com/") {
        r
    } else {
        url.strip_prefix("ssh://git@github.com/")?
    };
    Some(rest.trim_end_matches('/').to_string())
}
"#;

/// Lay down a dependency-free crate and lint it with **the gate's own flag
/// constants**, so the test cannot drift into asking clippy a friendlier
/// question than `pmat verify --stage clippy` asks.
fn clippy_verdict(dir: &Path, lib_rs: &str) -> (bool, String) {
    std::fs::create_dir_all(dir.join("src")).expect("mkdir");
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"replay\"\nversion = \"0.0.0\"\nedition = \"2021\"\n",
    )
    .expect("write manifest");
    std::fs::write(dir.join("src/lib.rs"), lib_rs).expect("write lib");
    let out = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .args(["clippy", CLIPPY_TARGETS, "--offline", "--"])
        .args(CLIPPY_LINTS)
        .current_dir(dir)
        // Its own target dir: the fixture must not read or write the real one.
        .env("CARGO_TARGET_DIR", dir.join("t"))
        .env("RUSTC_WORKSPACE_WRAPPER", "")
        .output()
        .expect("spawn cargo clippy");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), text)
}

/// The acceptance test: the exact line `049a925a1` shipped is rejected by the
/// exact lints the gate runs.
#[test]
fn the_historical_roster_line_is_rejected_by_the_gates_own_lints() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (ok, out) = clippy_verdict(dir.path(), &roster_fixture(HISTORICAL_ROSTER_LINE));
    assert!(
        !ok,
        "049a925a1's roster.rs line must fail the gate's lints, got a pass:\n{out}"
    );
    assert!(
        out.contains("unnecessary_sort_by") || out.contains("unnecessary-sort-by"),
        "expected clippy::unnecessary_sort_by, got:\n{out}"
    );
}

/// Counter-test: the gate must not fire on the fixed line. A gate that rejects
/// everything catches the defect for the wrong reason and would be turned off.
#[test]
fn the_fix_that_6285aaec6_applied_passes_the_same_lints() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (ok, out) = clippy_verdict(dir.path(), &roster_fixture(FIXED_ROSTER_LINE));
    assert!(ok, "the shipped fix must pass the gate, got:\n{out}");
}

/// The second error the same commit shipped, in `check_evidence_gates.rs`.
#[test]
fn the_historical_slug_parser_is_rejected_and_its_fix_is_not() {
    let red = tempfile::tempdir().expect("tempdir");
    let (ok, out) = clippy_verdict(red.path(), HISTORICAL_SLUG_PARSER);
    assert!(!ok, "049a925a1's parse_github_slug must fail:\n{out}");
    assert!(
        out.contains("question_mark") || out.contains("question-mark"),
        "expected clippy::question_mark, got:\n{out}"
    );

    let green = tempfile::tempdir().expect("tempdir");
    let (ok, out) = clippy_verdict(green.path(), FIXED_SLUG_PARSER);
    assert!(ok, "the shipped fix must pass:\n{out}");
}

// ---------------------------------------------------------------------------
// The hook's own control flow, with the clippy stage stubbed.
//
// The tests above prove clippy rejects the historical code. These prove the
// generated hook turns that rejection into a refused commit — the two halves
// the defect fell between.
// ---------------------------------------------------------------------------

/// Absolute path to bash. Needed where the test empties `PATH`: Rust resolves
/// the program name against the *child's* environment, so a bare "bash" is
/// unfindable exactly in the scenario under test.
fn bash() -> &'static str {
    if Path::new("/bin/bash").exists() {
        "/bin/bash"
    } else {
        "/usr/bin/bash"
    }
}

fn generated_hook() -> String {
    use crate::cli::handlers::hooks_command_handlers::HooksCommand;
    use std::path::PathBuf;
    let cmd = HooksCommand::new(PathBuf::from("/tmp"), PathBuf::from("/tmp"));
    format!(
        "{}\n{}",
        cmd.generate_hook_header(),
        cmd.generate_quality_checks()
    )
}

/// Run the generated hook in a throwaway git repo whose PATH offers a `pmat`
/// stub with the given exit code, and no `cargo`, so only the clippy gate's
/// branch is under test.
fn run_hook_with_stub_pmat(exit_code: i32) -> (i32, String) {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    let bin = root.join("bin");
    std::fs::create_dir_all(&bin).expect("mkdir");

    // `pmat` stub: prints a recognisable line, exits with the code under test.
    // Every other pmat subcommand (analyze complexity / satd) also lands here,
    // so exit 0 must keep those green.
    let stub = if exit_code == 0 {
        "#!/bin/sh\necho 'Total violations: 0'\nexit 0\n".to_string()
    } else {
        format!("#!/bin/sh\necho 'STUB-CLIPPY-RED'\nexit {exit_code}\n")
    };
    std::fs::write(bin.join("pmat"), stub).expect("write stub");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(bin.join("pmat"), std::fs::Permissions::from_mode(0o755))
            .expect("chmod");
    }

    let git = |args: &[&str]| {
        let ok = Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .expect("git");
        assert!(ok.status.success(), "git {args:?} failed");
    };
    git(&["init", "-q"]);
    git(&["config", "user.email", "t@t"]);
    git(&["config", "user.name", "t"]);
    std::fs::write(root.join("Cargo.toml"), "[package]\nname='x'\n").expect("write");
    std::fs::write(root.join("src.rs"), "pub fn a() {}\n").expect("write");
    git(&["add", "Cargo.toml", "src.rs"]);

    let hook = root.join("hook.sh");
    std::fs::write(&hook, generated_hook()).expect("write hook");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).expect("chmod");
    }

    let out = Command::new(bash())
        .arg(&hook)
        .current_dir(root)
        // PATH holds only the stub: no cargo, so the fmt gate is skipped and
        // the clippy branch is the one being measured.
        .env("PATH", format!("{}:/usr/bin:/bin", bin.display()))
        .output()
        .expect("run hook");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.code().unwrap_or(-1), text)
}

/// THE acceptance test for the hook half: a red clippy stage refuses the commit.
#[test]
fn the_hook_refuses_when_the_clippy_stage_comes_back_red() {
    let (code, out) = run_hook_with_stub_pmat(1);
    assert_ne!(
        code, 0,
        "hook must refuse a clippy-red tree; output:\n{out}"
    );
    assert!(
        out.contains("Clippy check... ❌"),
        "the refusal must name the gate that refused; got:\n{out}"
    );
    assert!(
        !out.contains("All quality gates passed"),
        "049a925a1 was told exactly this on the way out; got:\n{out}"
    );
}

/// Counter-test: a green clippy stage does not block a clean commit.
#[test]
fn the_hook_allows_a_commit_when_the_clippy_stage_is_green() {
    let (code, out) = run_hook_with_stub_pmat(0);
    assert_eq!(
        code, 0,
        "hook must not fire on a clean tree; output:\n{out}"
    );
    assert!(
        out.contains("Clippy check... ✅"),
        "the gate must report that it ran; got:\n{out}"
    );
}

/// A hook that cannot reach its gate must not report the gate as passed.
#[test]
fn the_hook_fails_closed_when_pmat_is_missing_from_a_cargo_project() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    std::fs::write(root.join("Cargo.toml"), "[package]\nname='x'\n").expect("write");
    std::fs::write(root.join("src.rs"), "pub fn a() {}\n").expect("write");
    let hook = root.join("hook.sh");
    std::fs::write(&hook, generated_hook()).expect("write hook");

    // Absolute interpreter, because PATH is about to be emptied.
    let out = Command::new(bash())
        .arg(&hook)
        .current_dir(root)
        // An empty PATH: neither pmat nor cargo is reachable. The hook reaches
        // its refusal using shell builtins only, so this is a fair test.
        .env("PATH", "")
        .output()
        .expect("run hook");
    assert_ne!(
        out.status.code().unwrap_or(-1),
        0,
        "a Cargo project whose hook cannot find pmat must fail, not pass"
    );
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.contains("pmat is not in PATH"),
        "the refusal has to say what is missing; got:\n{text}"
    );
}

/// Both installers must ship the *same* gate.
///
/// `scripts/install-git-hooks.sh` and `pmat hooks install` write different
/// pre-commit hooks into the same path, so whichever ran last decides what is
/// enforced. Adding the clippy gate to only one of them would mean the gate's
/// presence depends on install order — which is how CLAUDE.md's documented
/// divergence ("Two installers, two different hook sets") became load-bearing
/// in the first place. The sentinel-delimited block is compared byte for byte.
#[test]
fn both_hook_installers_embed_the_identical_clippy_gate() {
    const OPEN: &str = "# >>> pmat clippy gate (PMAT-630) >>>";
    const CLOSE: &str = "# <<< pmat clippy gate (PMAT-630) <<<";
    fn block(src: &str, what: &str) -> String {
        let start = src
            .find(OPEN)
            .unwrap_or_else(|| panic!("{what} has no clippy gate sentinel {OPEN}"));
        let end = src
            .find(CLOSE)
            .unwrap_or_else(|| panic!("{what} has no closing sentinel {CLOSE}"));
        assert!(start < end, "{what} sentinels are out of order");
        src[start..end + CLOSE.len()].to_string()
    }
    let from_rust = block(&generated_hook(), "the generated hook");
    let from_shell = block(
        include_str!("../../scripts/install-git-hooks.sh"),
        "scripts/install-git-hooks.sh",
    );
    assert_eq!(
        from_rust, from_shell,
        "the two installers' clippy gates have diverged; whichever installer \
         ran last would decide whether clippy is enforced"
    );
}
