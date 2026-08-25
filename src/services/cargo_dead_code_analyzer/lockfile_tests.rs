//! The analyser must not WRITE into the repository it was asked to READ.
//!
//! `cargo check` generates a `Cargo.lock` when none exists, so
//! `pmat analyze dead-code` left an untracked, source-controlled artifact
//! behind in every crate that had not got one yet (#1076). That is not
//! cosmetic: a library deliberately omits its lockfile and a binary
//! deliberately commits one, pmat cannot tell which it is looking at, and a
//! `git add -A` after the analysis then changes a dependency-resolution policy
//! nobody chose. It also dirties the tree for any "is the working copy clean"
//! gate downstream — including pmat's own dogfood check.
//!
//! The fix is `--locked`: cargo REFUSES rather than writes. The refusal costs
//! the compiler-lint layer, and the whole point of these tests is that the loss
//! is stated rather than absorbed into an unchanged report shape.

use super::{lockfile_refusal_line, CargoDeadCodeAnalyzer};
use crate::models::dead_code::{
    COMPILER_SCAN_REASON_ENV_SKIP, COMPILER_SCAN_REASON_LOCKFILE, COMPILER_SCAN_REASON_OK,
};

/// A crate with one live export and one genuinely dead private function.
///
/// `dead_one` is what rustc's dead-code lint finds and the suppression scan
/// cannot: it carries no `allow(dead_code)`, so it is the marker that says
/// whether the compiler layer really ran.
fn crate_fixture() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        tmp.path().join("Cargo.toml"),
        "[package]\nname = \"lockfix\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[workspace]\n",
    )
    .expect("write manifest");
    std::fs::create_dir_all(tmp.path().join("src")).expect("mkdir src");
    std::fs::write(
        tmp.path().join("src/lib.rs"),
        "pub fn entry(n: u64) -> u64 {\n    n + 1\n}\n\nfn dead_one() -> u64 {\n    1\n}\n",
    )
    .expect("write lib.rs");
    tmp
}

/// THE ISSUE. Analysing a crate that has no lockfile must not create one.
///
/// Pre-fix this assertion fails: `cargo check` writes `Cargo.lock` into the
/// analysed tree and `git status --porcelain` reports `?? Cargo.lock`.
#[ignore = "#1076 is OPEN: --locked was reverted because it silently disabled the compiler scan (80 dead functions -> 0) on any repo with an absent or stale lockfile. This test is the SPEC for the real fix — analyse a copy, or snapshot/restore the lockfile — and must go green when that lands, not be deleted."]
#[tokio::test]
async fn analysing_a_lockfile_less_crate_creates_no_lockfile() {
    let tmp = crate_fixture();
    let lockfile = tmp.path().join("Cargo.lock");
    assert!(!lockfile.exists(), "fixture starts with no lockfile");

    let _report = CargoDeadCodeAnalyzer::new(tmp.path())
        .analyze()
        .await
        .expect("analysis runs");

    assert!(
        !lockfile.exists(),
        "analyse-only wrote {} into the analysed repository: a lockfile is a \
         source-controlled artifact whose presence is the project's decision, and \
         pmat cannot tell a library (which omits it) from a binary (which commits \
         it)",
        lockfile.display()
    );
}

/// …and the cost of not writing it is DECLARED, with a machine-readable reason.
///
/// Without this the report is the same shape over a much smaller search: only
/// explicit `allow(dead_code)` admissions were looked for, so `0 dead items`
/// would read as "nothing is dead" when it means "nothing was admitted".
#[ignore = "#1076 is OPEN: --locked was reverted because it silently disabled the compiler scan (80 dead functions -> 0) on any repo with an absent or stale lockfile. This test is the SPEC for the real fix — analyse a copy, or snapshot/restore the lockfile — and must go green when that lands, not be deleted."]
#[tokio::test]
async fn the_refused_compiler_scan_is_declared_on_the_report() {
    let tmp = crate_fixture();

    let report = CargoDeadCodeAnalyzer::new(tmp.path())
        .analyze()
        .await
        .expect("analysis runs");

    let scan = report
        .compiler_scan
        .as_ref()
        .expect("a cargo run always records whether its compiler layer ran");
    assert!(
        !scan.is_full(),
        "the compiler layer cannot have run: compiling this crate needs a lockfile \
         that does not exist, yet the report claims a full scan: {scan:?}"
    );
    assert_eq!(
        scan.reason, COMPILER_SCAN_REASON_LOCKFILE,
        "the cause must be a stable token a consumer can branch on, not prose: {scan:?}"
    );
    assert!(
        scan.detail.contains("Cargo.lock"),
        "the reason must NAME the artifact that was not written: {scan:?}"
    );
    assert!(
        scan.detail.contains("allow(dead_code)"),
        "the reason must say what WAS searched for, or a reader cannot weigh the \
         count beside it: {scan:?}"
    );
}

/// COUNTER-TEST. A crate that already has a lockfile is analysed at FULL
/// fidelity, and its lockfile is byte-identical afterwards.
///
/// This is what stops the fix from over-correcting. A "refuse everything"
/// implementation — always report `reduced`, never run cargo — passes the two
/// tests above and fails here, because `dead_one()` carries no suppression
/// attribute and is therefore findable ONLY by rustc's dead-code lint. The
/// byte comparison is the other half: a fix that ran cargo freely and deleted
/// or rewrote the lockfile afterwards would fail it.
#[tokio::test]
async fn a_crate_with_a_lockfile_is_analysed_fully_and_its_lockfile_is_untouched() {
    let tmp = crate_fixture();
    let lockfile = tmp.path().join("Cargo.lock");
    // Written by hand rather than by `cargo generate-lockfile`, so the test
    // needs no network and no second cargo invocation. The fixture has no
    // dependencies, so this IS the resolution cargo would compute.
    std::fs::write(
        &lockfile,
        "# This file is automatically @generated by Cargo.\n\
         # It is not intended for manual editing.\n\
         version = 4\n\n\
         [[package]]\n\
         name = \"lockfix\"\n\
         version = \"0.1.0\"\n",
    )
    .expect("write lockfile");
    let before = std::fs::read(&lockfile).expect("read lockfile");

    let report = CargoDeadCodeAnalyzer::new(tmp.path())
        .analyze()
        .await
        .expect("analysis runs");

    let scan = report
        .compiler_scan
        .as_ref()
        .expect("a cargo run always records whether its compiler layer ran");
    assert!(
        scan.is_full(),
        "a crate WITH a lockfile must still be compiled: refusing here would trade \
         the bug for a tool that measures nothing anywhere: {scan:?}"
    );
    assert_eq!(scan.reason, COMPILER_SCAN_REASON_OK, "{scan:?}");

    // Full fidelity is a claim about findings, not a label. `dead_one` has no
    // `allow(dead_code)`, so only the compiler layer can produce it.
    let found: Vec<&str> = report
        .files_with_dead_code
        .iter()
        .flat_map(|f| f.dead_items.iter())
        .map(|i| i.name.as_str())
        .collect();
    assert!(
        found.contains(&"dead_one"),
        "the report says it scanned fully but did not find the one item only a \
         compile can find; findings were {found:?}"
    );

    assert!(lockfile.exists(), "the lockfile must still be there");
    assert_eq!(
        before,
        std::fs::read(&lockfile).expect("read lockfile"),
        "the analysed project's lockfile was modified by a read-only analysis"
    );
}

/// The mechanism is cargo's refusal, not a cleanup of ours.
///
/// A "delete it afterwards" fix leaves the reproducer clean too, so the
/// observable end-state cannot tell the two apart — but they are not equally
/// safe: a killed run never reaches a cleanup, and an invisible cleanup is a
/// second thing the user is not told about. Pinning `--locked` in the argv pins
/// which of the two this is.
#[ignore = "#1076 is OPEN: --locked was reverted because it silently disabled the compiler scan (80 dead functions -> 0) on any repo with an absent or stale lockfile. This test is the SPEC for the real fix — analyse a copy, or snapshot/restore the lockfile — and must go green when that lands, not be deleted."]
#[test]
fn the_cargo_invocation_forbids_cargo_from_writing_the_lockfile() {
    let tmp = crate_fixture();
    let analyzer = CargoDeadCodeAnalyzer::new(tmp.path());
    let cmd = analyzer
        .build_cargo_check_command()
        .expect("PMAT_DEAD_CODE_SKIP is unset in this test");

    let args: Vec<String> = cmd
        .get_args()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    assert!(
        args.iter().any(|a| a == "--locked"),
        "without --locked cargo generates the analysed repo's Cargo.lock, and any \
         clean-up afterwards is skipped by a killed run: {args:?}"
    );
}

/// `PMAT_DEAD_CODE_SKIP` is the OTHER way the compiler layer does not run, and
/// it was equally silent: the analyzer returned a synthetic "build finished,
/// success" record and the report was indistinguishable from a real scan.
#[test]
fn a_suppressed_scan_is_a_reduced_scan_and_says_so() {
    let outcome = super::CargoCheckOutcome::suppressed_by_env();
    assert!(!outcome.scan.is_full(), "{:?}", outcome.scan);
    assert_eq!(outcome.scan.reason, COMPILER_SCAN_REASON_ENV_SKIP);
    assert!(
        outcome.scan.detail.contains("PMAT_DEAD_CODE_SKIP"),
        "the reason must name the switch that caused it: {:?}",
        outcome.scan
    );
}

/// A genuine compile failure is still a failure. Only cargo's own lockfile
/// refusal is downgraded to a disclosed reduction — otherwise every broken
/// crate would quietly report "reduced fidelity" instead of erroring.
#[test]
fn only_cargos_lockfile_refusal_is_treated_as_a_refusal() {
    assert_eq!(
        lockfile_refusal_line(
            "error: cannot create the lock file /x/Cargo.lock because --locked was \
             passed to prevent this\nhelp: to generate the lock file ...\n"
        )
        .as_deref(),
        Some(
            "error: cannot create the lock file /x/Cargo.lock because --locked was \
             passed to prevent this"
        )
    );
    assert!(
        lockfile_refusal_line(
            "error: the lock file /x/Cargo.lock needs to be updated but --locked was \
             passed to prevent this\n"
        )
        .is_some(),
        "cargo's older wording for the same refusal must be recognised too"
    );
    assert!(
        lockfile_refusal_line("error[E0425]: cannot find value `x` in this scope\n").is_none(),
        "a compile error must stay an error, not become a fidelity caveat"
    );
    assert!(
        lockfile_refusal_line("").is_none(),
        "an empty stderr says nothing about the lockfile"
    );
}
