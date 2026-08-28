//! Tests for the unrun-test gate.
//!
//! The gate's own regression risk is that it becomes vacuous — a gate that
//! flags everything and a gate that flags nothing are equally useless — so the
//! suite pins BOTH directions: the four historical defects must be named, and
//! the ~20,000 tests that do run must not be.

use super::*;
use std::path::PathBuf;
use std::sync::OnceLock;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The real-tree analysis is shared: walking 2,000+ files with `syn` is the
/// expensive part and every assertion below reads the same answer.
fn this_repo() -> &'static Report {
    static R: OnceLock<Report> = OnceLock::new();
    R.get_or_init(|| analyze(&repo(), &[String::new()]).expect("the repository must analyze"))
}

fn parse_cfg(src: &str) -> cfg::CfgExpr {
    let f: syn::ItemFn = syn::parse_str(&format!("{src} fn t() {{}}")).expect("parses");
    cfg::of_attrs(&f.attrs)
}

fn env_of(feats: &[&str]) -> cfg::Env {
    cfg::Env {
        features: feats.iter().map(|s| (*s).to_string()).collect(),
    }
}

// ── cfg: three-valued, because "we could not tell" is an answer ──────────

#[test]
fn a_feature_predicate_is_decided_by_the_legs_feature_set() {
    let e = parse_cfg(r#"#[cfg(feature = "mcp-integration")]"#);
    assert_eq!(e.eval(&env_of(&["mcp-integration"])), cfg::Tri::True);
    assert_eq!(e.eval(&env_of(&["full"])), cfg::Tri::False);
}

#[test]
fn conjunction_and_disjunction_and_negation_compose() {
    let e = parse_cfg(r#"#[cfg(all(test, feature = "a", not(feature = "b")))]"#);
    assert_eq!(e.eval(&env_of(&["a"])), cfg::Tri::True);
    assert_eq!(e.eval(&env_of(&["a", "b"])), cfg::Tri::False);
    let e = parse_cfg(r#"#[cfg(any(feature = "a", feature = "b"))]"#);
    assert_eq!(e.eval(&env_of(&["b"])), cfg::Tri::True);
}

#[test]
fn stacked_cfg_attributes_conjoin() {
    let e = parse_cfg("#[cfg(test)]\n#[cfg(feature = \"a\")]");
    assert_eq!(e.eval(&env_of(&["a"])), cfg::Tri::True);
    assert_eq!(e.eval(&env_of(&[])), cfg::Tri::False);
}

/// FAIL CLOSED. A predicate outside the decidable allowlist must not be folded
/// into either polarity: `false` would silently report the test unrun and
/// `true` would silently report it run, and neither answer is distinguishable
/// from a correct one.
#[test]
fn an_undecidable_predicate_is_unknown_not_false() {
    assert_eq!(
        parse_cfg("#[cfg(some_bespoke_flag)]").eval(&env_of(&[])),
        cfg::Tri::Unknown
    );
    // Unknown propagates through the connectives rather than collapsing.
    assert_eq!(
        parse_cfg("#[cfg(all(test, some_bespoke_flag))]").eval(&env_of(&[])),
        cfg::Tri::Unknown
    );
    assert_eq!(
        parse_cfg("#[cfg(not(some_bespoke_flag))]").eval(&env_of(&[])),
        cfg::Tri::Unknown
    );
    // …but a definite answer still wins: `all(false, unknown)` is false.
    assert_eq!(
        parse_cfg(r#"#[cfg(all(feature = "absent", some_bespoke_flag))]"#).eval(&env_of(&[])),
        cfg::Tri::False
    );
}

#[test]
fn cfg_attr_does_not_gate_compilation() {
    let e = parse_cfg("#[cfg_attr(coverage_nightly, coverage(off))]");
    assert_eq!(e, cfg::CfgExpr::True);
}

#[test]
fn polarity_decides_the_bucket_label() {
    let mut out = std::collections::BTreeSet::new();
    parse_cfg(r#"#[cfg(all(feature = "a", not(feature = "b")))]"#)
        .positive_features(false, &mut out);
    assert_eq!(out.into_iter().collect::<Vec<_>>(), vec!["a".to_string()]);
}

// ── features: the closure, not the flag ─────────────────────────────────

#[test]
fn a_feature_flag_enables_its_whole_closure() {
    let g = features::parse(
        "[features]\ndefault = [\"a\"]\na = [\"b\", \"dep:x\", \"crate/y\"]\nb = []\n[other]\n",
    );
    let c = features::closure(&g, ["default"]);
    assert!(c.contains("a") && c.contains("b"), "{c:?}");
    assert!(
        !c.contains("x") && !c.contains("y"),
        "dep activations are not features"
    );
}

#[test]
fn a_multi_line_feature_list_is_parsed_whole() {
    let g = features::parse("[features]\nbig = [\n  \"a\",\n  \"b\",\n]\n[x]\n");
    assert_eq!(g.get("big").map(Vec::len), Some(2));
}

#[test]
fn this_repos_real_feature_table_parses() {
    let manifest = std::fs::read_to_string(repo().join("Cargo.toml")).expect("manifest");
    let g = features::parse(&manifest);
    assert!(g.len() >= 40, "only {} features parsed", g.len());
    let full = features::closure(&g, ["default", "full"]);
    assert!(
        full.contains("mutation-testing"),
        "full must reach advanced-analysis"
    );
    assert!(
        !full.contains("mcp-integration"),
        "mcp-integration is an orphan feature"
    );
}

// ── legs: derived from the workflows, and `cargo check` is not one ───────

const WF: &str = r#"
name: x
jobs:
  compile-only:
    steps:
      - run: cargo check --lib --features mcp-integration
  feature-tests:
    strategy:
      matrix:
        include:
          - features: unified-protocol
          - features: full
    steps:
      - name: go
        run: |
          # cargo test --lib --features never-this-one
          cargo test --lib --locked --features '${{ matrix.features }}'
"#;

#[test]
fn legs_come_from_cargo_test_and_resolve_the_matrix() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("a.yml"), WF).expect("write");
    let legs = legs::from_workflows(dir.path());
    let feats: Vec<Vec<String>> = legs.iter().map(|l| l.features.clone()).collect();
    assert_eq!(
        feats,
        vec![
            vec!["full".to_string()],
            vec!["unified-protocol".to_string()]
        ],
        "one leg per matrix entry, and nothing else: {legs:?}"
    );
}

/// `cargo check` compiles no `#[cfg(test)]` item and runs no body. Counting it
/// as coverage is precisely how `mcp-integration` stayed green for months with
/// 910 tests that had never executed.
#[test]
fn cargo_check_is_not_a_leg() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("a.yml"),
        "jobs:\n  j:\n    steps:\n      - run: cargo check --lib --features mcp-integration\n",
    )
    .expect("write");
    assert!(legs::from_workflows(dir.path()).is_empty());
}

/// A commented-out invocation is not an invocation. `feature-matrix.yml`
/// documents past measurements as `#   cargo test --lib --features full`, and
/// reading those as legs would mark their tests run on the strength of a
/// comment.
#[test]
fn a_commented_out_invocation_is_not_a_leg() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("a.yml"),
        "jobs:\n  j:\n    steps:\n      - run: |\n          #   cargo test --lib --features full\n",
    )
    .expect("write");
    assert!(legs::from_workflows(dir.path()).is_empty());
}

#[test]
fn this_repos_workflows_yield_the_three_in_repo_test_legs() {
    let legs: Vec<String> = legs::from_workflows(&repo().join(".github/workflows"))
        .into_iter()
        .filter(|l| l.runs_lib)
        .map(|l| l.origin)
        .collect();
    assert_eq!(
        legs,
        vec![
            "feature-matrix.yml:feature-tests[full]",
            "feature-matrix.yml:feature-tests[mcp-integration]",
            "feature-matrix.yml:feature-tests[unified-protocol]",
        ],
        "the in-repo legs are exactly the feature-tests matrix; \
         `ci / test` lives in paiml/.github and is supplied with --executed"
    );
}

// ── walk: the full module path is the key ───────────────────────────────

fn fixture(files: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    for (rel, body) in files {
        let p = dir.path().join(rel);
        std::fs::create_dir_all(p.parent().expect("parent")).expect("mkdir");
        std::fs::write(p, body).expect("write");
    }
    dir
}

#[test]
fn include_splices_into_the_current_module_without_growing_the_path() {
    let d = fixture(&[
        ("src/lib.rs", "pub mod a;\n"),
        (
            "src/a.rs",
            "#[cfg(test)]\nmod tests { include!(\"a_tests.rs\"); }\n",
        ),
        ("src/a_tests.rs", "#[test]\nfn t() {}\n"),
    ]);
    let c = walk::collect(d.path(), &d.path().join("src/lib.rs"));
    assert_eq!(
        c.tests.iter().map(|t| t.path.as_str()).collect::<Vec<_>>(),
        vec!["crate::a::tests::t"]
    );
    assert_eq!(
        c.tests[0].file, "src/a_tests.rs",
        "the body's file, not the includer's"
    );
}

/// THE SUBTLETY THAT DEFEATED A NAME SEARCH. Both copies exist, one runs, one
/// does not, and `cargo test -- --list | grep <name>` shows the name either
/// way. Keying on the path is what separates them.
#[test]
fn identically_named_tests_in_different_modules_are_distinct_records() {
    let d = fixture(&[
        (
            "src/lib.rs",
            "pub mod runs;\n#[cfg(feature = \"hidden\")]\npub mod hidden;\n",
        ),
        (
            "src/runs.rs",
            "#[cfg(test)]\nmod tests {\n#[test]\nfn twin() {}\n}\n",
        ),
        (
            "src/hidden.rs",
            "#[cfg(test)]\nmod tests {\n#[test]\nfn twin() {}\n}\n",
        ),
    ]);
    let c = walk::collect(d.path(), &d.path().join("src/lib.rs"));
    let paths: Vec<&str> = c.tests.iter().map(|t| t.path.as_str()).collect();
    assert_eq!(
        paths,
        vec!["crate::hidden::tests::twin", "crate::runs::tests::twin"]
    );

    let e = env_of(&[]);
    let by = |p: &str| c.tests.iter().find(|t| t.path == p).expect("present");
    assert_eq!(by("crate::runs::tests::twin").cfg.eval(&e), cfg::Tri::True);
    assert_eq!(
        by("crate::hidden::tests::twin").cfg.eval(&e),
        cfg::Tri::False
    );
}

#[test]
fn a_cfg_on_an_ancestor_module_reaches_the_test() {
    let d = fixture(&[
        (
            "src/lib.rs",
            "#[cfg(all(feature = \"a\", feature = \"b\"))]\npub mod m;\n",
        ),
        (
            "src/m.rs",
            "#[cfg(test)]\nmod tests {\n#[test]\nfn t() {}\n}\n",
        ),
    ]);
    let c = walk::collect(d.path(), &d.path().join("src/lib.rs"));
    assert_eq!(c.tests[0].cfg.eval(&env_of(&["a"])), cfg::Tri::False);
    assert_eq!(c.tests[0].cfg.eval(&env_of(&["a", "b"])), cfg::Tri::True);
}

#[test]
fn tokio_and_actix_test_attributes_count_as_tests() {
    let d = fixture(&[(
        "src/lib.rs",
        "#[tokio::test]\nasync fn a() {}\n#[actix_rt::test]\nasync fn b() {}\n#[test]\nfn c() {}\n\
         #[ignore]\n#[test]\nfn d() {}\nfn not_a_test() {}\n",
    )]);
    let c = walk::collect(d.path(), &d.path().join("src/lib.rs"));
    assert_eq!(c.tests.len(), 4);
    assert!(
        c.tests
            .iter()
            .find(|t| t.path.ends_with("::d"))
            .expect("d")
            .ignored
    );
}

#[test]
fn an_unresolvable_mod_is_reported_rather_than_swallowed() {
    let d = fixture(&[("src/lib.rs", "pub mod nowhere;\n")]);
    let c = walk::collect(d.path(), &d.path().join("src/lib.rs"));
    assert_eq!(c.unresolved.len(), 1, "{:?}", c.unresolved);
}

// ── the historical defect, replayed hermetically ────────────────────────

/// The four tests of the 3.32.0 cycle, reconstructed: the `mcp_integration`
/// module chain behind `all(standard-deps, mcp-integration)`, an `include!`d
/// test file, the `analytics-gpu` gate applied to the function itself, and —
/// the part that defeated a name search — twins with IDENTICAL names under
/// `mcp_pmcp`, which the only leg does run.
///
/// Verified against the real tree at f47d75170 (the commit before
/// `mcp-integration` became a leg), where this analysis names all four and
/// neither twin.
fn historical_fixture() -> tempfile::TempDir {
    fixture(&[
        (
            "Cargo.toml",
            &{
                let mut m = String::from("[features]\ndefault = [\"standard-deps\"]\n\
                    standard-deps = []\nfull = [\"analytics-simd\"]\n\
                    unified-protocol = []\nmcp-integration = []\n\
                    analytics-gpu = [\"analytics-simd\"]\nanalytics-simd = []\n");
                for i in 0..40 {
                    m.push_str(&format!("pad{i} = []\n"));
                }
                m.push_str("[package]\nname = \"x\"\n");
                m
            },
        ),
        (
            ".github/workflows/feature-matrix.yml",
            concat!(
                "jobs:\n",
                "  individual:\n    steps:\n",
                "      - run: cargo check --lib --features mcp-integration\n",
                "  feature-tests:\n    strategy:\n      matrix:\n        include:\n",
                "          - features: unified-protocol\n    steps:\n",
                "      - run: cargo test --lib --features '${{ matrix.features }}'\n",
            ),
        ),
        (
            "src/lib.rs",
            "#[cfg(all(feature = \"standard-deps\", feature = \"mcp-integration\"))]\n\
             pub mod mcp_integration;\n\
             #[cfg(feature = \"standard-deps\")]\npub mod mcp_pmcp;\n\
             pub mod services;\n",
        ),
        ("src/mcp_integration/mod.rs", "pub mod tdg_tools;\npub mod tools;\n"),
        (
            "src/mcp_integration/tdg_tools.rs",
            "include!(\"tdg_tools_tests.rs\");\n",
        ),
        (
            "src/mcp_integration/tdg_tools_tests.rs",
            "#[cfg(test)]\nmod tests {\n#[tokio::test]\n\
             async fn test_analyze_technical_debt_refuses_ungradable_file() {}\n}\n",
        ),
        ("src/mcp_integration/tools/mod.rs", "pub mod context_adapters;\n"),
        (
            "src/mcp_integration/tools/context_adapters.rs",
            "#[cfg(test)]\nmod tests {\n\
             #[tokio::test]\nasync fn documented_bounds_are_invalid_params_not_internal_errors() {}\n\
             #[test]\nfn our_own_failures_stay_internal() {}\n}\n",
        ),
        (
            "src/mcp_pmcp/mod.rs",
            "pub mod agent_context_handlers;\n",
        ),
        (
            "src/mcp_pmcp/agent_context_handlers.rs",
            "#[cfg(test)]\nmod tests {\n\
             #[tokio::test]\nasync fn documented_bounds_are_invalid_params_not_internal_errors() {}\n\
             #[test]\nfn our_own_failures_stay_internal() {}\n}\n",
        ),
        ("src/services/mod.rs", "pub mod analytics_backend;\n"),
        (
            "src/services/analytics_backend.rs",
            "#[cfg(test)]\nmod tests { include!(\"analytics_backend_tests.rs\"); }\n",
        ),
        (
            "src/services/analytics_backend_tests.rs",
            "#[test]\nfn test_backend_auto_select() {}\n\
             #[test]\n#[cfg(feature = \"analytics-gpu\")]\n\
             fn test_gpu_is_reported_unavailable_and_never_selected() {}\n",
        ),
    ])
}

#[test]
fn the_four_tests_of_the_3_32_0_cycle_are_all_named() {
    let d = historical_fixture();
    let r = analyze(d.path(), &[String::new()]).expect("analyzes");
    let named: Vec<&str> = r.unrun.iter().map(|f| f.path.as_str()).collect();
    for want in [
        "crate::mcp_integration::tdg_tools::tests::\
         test_analyze_technical_debt_refuses_ungradable_file",
        "crate::mcp_integration::tools::context_adapters::tests::\
         documented_bounds_are_invalid_params_not_internal_errors",
        "crate::mcp_integration::tools::context_adapters::tests::\
         our_own_failures_stay_internal",
        "crate::services::analytics_backend::tests::\
         test_gpu_is_reported_unavailable_and_never_selected",
    ] {
        assert!(
            named.contains(&want),
            "not named: {want}\nnamed: {named:#?}"
        );
    }
    assert_eq!(r.unrun.len(), 4, "and nothing else: {named:#?}");
}

/// The counter-half of the same fixture, and the whole reason the key is the
/// path. A bare-name search for either of these two names succeeds against the
/// running `mcp_pmcp` copy, which is how the hidden `mcp_integration` copies
/// stayed hidden.
#[test]
fn the_identically_named_twins_that_do_run_are_not_flagged() {
    let d = historical_fixture();
    let r = analyze(d.path(), &[String::new()]).expect("analyzes");
    let named: Vec<&str> = r.unrun.iter().map(|f| f.path.as_str()).collect();
    for twin in [
        "crate::mcp_pmcp::agent_context_handlers::tests::our_own_failures_stay_internal",
        "crate::mcp_pmcp::agent_context_handlers::tests::\
         documented_bounds_are_invalid_params_not_internal_errors",
        "crate::services::analytics_backend::tests::test_backend_auto_select",
    ] {
        assert!(
            !named.contains(&twin),
            "{twin} runs by default and must not be flagged"
        );
    }
    assert_eq!(
        r.executed, 3,
        "the three default-feature tests must be counted as run"
    );
}

/// `cargo check --features mcp-integration` appears in the fixture workflow and
/// must buy nothing: it is precisely the invocation that kept the real feature
/// looking covered while 910 tests had never executed.
#[test]
fn a_compile_check_leg_does_not_rescue_the_four() {
    let d = historical_fixture();
    let r = analyze(d.path(), &[String::new()]).expect("analyzes");
    assert!(
        r.legs.iter().all(|l| !l.contains("individual")),
        "a `cargo check` job must not appear as a leg: {:?}",
        r.legs
    );
}

/// FAIL CLOSED at the top level: with no resolvable leg, the honest answer is
/// an error, not "every test in the tree is a finding".
#[test]
fn zero_legs_is_an_error_not_a_catastrophic_report() {
    let d = historical_fixture();
    std::fs::remove_dir_all(d.path().join(".github")).expect("rm");
    assert!(analyze(d.path(), &[]).is_err());
}

// ── the gate on the real tree ───────────────────────────────────────────

#[test]
fn the_analysis_resolves_legs_and_walks_the_whole_lib() {
    let r = this_repo();
    // Three, and every one of them resolved from a workflow file.
    //
    // This was `>= 4`, and the fourth was not a CI leg: `this_repo()` calls
    // `analyze` with `&[String::new()]`, and an empty `--executed` spec used to
    // be pushed as a leg named `--executed ''`. The count was met by an
    // artifact of how the test called the analyser, and a fabricated leg makes
    // tests read as executed when nothing executes them.
    //
    // The origin assertion is the part that matters: it pins that every leg
    // came from `.github/workflows`, so restoring the old behaviour cannot
    // quietly satisfy the count again.
    assert!(r.legs.len() >= 3, "legs: {:?}", r.legs);
    assert!(
        r.legs.iter().all(|l| l.contains(".yml:")),
        "every leg must resolve from a workflow file, not from a CLI argument: {:?}",
        r.legs
    );
    assert!(
        r.total_tests > 20_000,
        "only {} tests walked",
        r.total_tests
    );
    assert!(r.files > 1_000, "only {} files walked", r.files);
}

/// The counter-test. A gate that flags everything is useless, so the tests
/// every leg already runs must come back clean.
#[test]
fn the_tests_that_do_run_are_not_flagged() {
    let r = this_repo();
    assert!(
        r.executed > 20_000,
        "{} executed — the analysis has stopped seeing the tests that run",
        r.executed
    );
    let flagged: Vec<&str> = r.unrun.iter().map(|f| f.path.as_str()).collect();
    for path in [
        "crate::mcp_pmcp::agent_context_handlers::tests::our_own_failures_stay_internal",
        "crate::services::unrun_tests::tests::the_tests_that_do_run_are_not_flagged",
        "crate::services::vacuous_tests::tests::",
    ] {
        assert!(
            !flagged.iter().any(|p| p.starts_with(path)),
            "{path} runs by default and must not be flagged"
        );
    }
    let rate = r.unrun.len() as f64 / r.total_tests as f64;
    assert!(
        rate < 0.25,
        "flagged {:.1}% of the tree — too blunt to be a finding",
        rate * 100.0
    );
}

/// One of the four. The other three were fixed between the defect and this
/// commit by promoting `mcp-integration` to a real leg; this one was not, and
/// the gate must keep saying so.
#[test]
fn the_analytics_gpu_test_is_still_unrun_at_head() {
    let r = this_repo();
    assert!(
        r.unrun.iter().any(|f| f.path
            == "crate::services::analytics_backend::tests::\
                test_gpu_is_reported_unavailable_and_never_selected"),
        "the analytics-gpu regression test is compiled by no leg and must be named"
    );
}

/// FAIL CLOSED, reported as a number rather than assumed to be zero.
#[test]
fn nothing_in_this_tree_is_undeterminable() {
    let r = this_repo();
    assert!(
        r.undeterminable.is_empty(),
        "{} test(s) have a cfg predicate this analysis cannot decide: {:?}",
        r.undeterminable.len(),
        r.undeterminable.iter().take(5).collect::<Vec<_>>()
    );
    assert!(r.unparsed.is_empty(), "unparsable files: {:?}", r.unparsed);
}

/// THE GATE. Regenerating the ledger from the tree must reproduce the
/// committed bytes; anything else names the tests that moved.
#[test]
fn the_committed_ledger_matches_the_tree() {
    let r = this_repo();
    let d = ledger::check(&repo(), r);
    assert!(
        d.is_clean(),
        "the unrun-tests ledger has drifted.\n\
         NEWLY UNRUN (no CI leg compiles these): {:#?}\n\
         NO LONGER UNRUN: {:#?}\n\
         BUCKETS WITH NO RECORDED REASON: {:?}\n\
         STALE REASONS: {:?}\n\
         RENDERED TEXT DIFFERS: {}\n\
         Run `pmat analyze unrun-tests --write-ledger` and justify every addition.",
        d.added,
        d.removed,
        d.unexplained,
        d.stale_reasons,
        d.text_differs
    );
}

// ── write refuses a dirty tree (PMAT-630 / #1034, B1) ────────────────────
//
// docs/status/unrun-tests-ledger.md was committed at 0781352ea reading
// "22882 of 26003"; a clean checkout of that SAME commit, in an isolated
// worktree untouched by anything else on disk, walks to "22878 of 25999" —
// four fewer. The delta is exactly the four hop-suppression tests that sat
// uncommitted in gate_effect/{invocation,tests}.rs while 0781352ea (and
// every commit after it, up to the one that finally committed them) was
// made: `--write-ledger` reads the WORKING TREE, not the commit, so it
// silently recorded a future the ledger's own commit had not reached yet.
// A ledger that describes a tree no checkout of its commit reproduces is
// exactly the "artifact does not match what it claims to describe" defect
// this whole ledger exists to catch in test CODE — it must catch itself.

/// Init a git repo in `dir` and leave it however `dirty` files say to.
///
/// Every path is created and `git add`ed (so `--untracked-files=no` still
/// sees it) and then, only for the ones named in `dirty`, appended to after
/// the commit — a tracked file with an uncommitted edit, the exact shape of
/// the historical defect, not an untracked new one.
fn git_repo(dir: &std::path::Path, files: &[&str], dirty: &[&str]) {
    let run = |args: &[&str]| {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .output()
            .expect("git runs");
        assert!(out.status.success(), "git {args:?}: {out:?}");
    };
    run(&["init", "--quiet", "--initial-branch=main"]);
    run(&["config", "user.email", "t@t"]);
    run(&["config", "user.name", "t"]);
    for f in files {
        std::fs::write(dir.join(f), "committed\n").expect("write");
    }
    run(&["add", "-A"]);
    // `--no-verify`: this is a synthetic fixture repo, not a real commit, and
    // must not depend on whatever hooks this machine's global git template
    // (`init.templateDir`) happens to install into every fresh `git init`.
    run(&["commit", "--quiet", "--no-verify", "-m", "init"]);
    for f in dirty {
        std::fs::write(dir.join(f), "committed\nedited after commit\n").expect("edit");
    }
}

/// A minimal analyzable tree, layered onto whatever `git_repo` already wrote.
fn write_analyzable_tree(dir: &std::path::Path) {
    // `analyze()` refuses fewer than 40 parsed features ("the parser is
    // broken, not the crate") as a guard against a fixture too small to be
    // a realistic manifest, so this one pads out to the floor the same way
    // `historical_fixture` does.
    let mut manifest = String::from(
        "[features]\ndefault = []\nfull = []\nmcp-integration = []\n\
                       unified-protocol = []\n",
    );
    for i in 0..40 {
        manifest.push_str(&format!("pad{i} = []\n"));
    }
    fixture_into(
        dir,
        &[
            ("Cargo.toml", manifest.as_str()),
            (
                ".github/workflows/ci.yml",
                "jobs:\n  test:\n    steps:\n      - run: cargo test --lib\n",
            ),
            (
                "src/lib.rs",
                "#[cfg(test)]\nmod tests { #[test]\nfn t() {} }\n",
            ),
        ],
    );
}

fn fixture_into(dir: &std::path::Path, files: &[(&str, &str)]) {
    for (rel, body) in files {
        let p = dir.join(rel);
        std::fs::create_dir_all(p.parent().expect("parent")).expect("mkdir");
        std::fs::write(p, body).expect("write");
    }
}

#[test]
fn write_refuses_a_tree_with_uncommitted_tracked_changes() {
    let d = tempfile::tempdir().expect("tempdir");
    write_analyzable_tree(d.path());
    // `unrelated.txt` is the stand-in for gate_effect/{invocation,tests}.rs:
    // a file this analysis never reads, edited for a wholly different reason,
    // sitting uncommitted while the ledger gets written.
    git_repo(d.path(), &["unrelated.txt"], &["unrelated.txt"]);

    let report = analyze(d.path(), &[String::new()]).expect("analyzes");
    let err = ledger::write(d.path(), &report, false).expect_err("a dirty tree must refuse");
    assert!(
        err.contains("dirty") && err.contains("unrelated.txt"),
        "the refusal must name the dirty path so it can be acted on: {err}"
    );
    assert!(
        !d.path().join(ledger::LEDGER_PATH).exists(),
        "a refused write must not leave a ledger behind for --check-ledger to trust"
    );
}

#[test]
fn counter_write_succeeds_on_a_clean_tree() {
    let d = tempfile::tempdir().expect("tempdir");
    write_analyzable_tree(d.path());
    git_repo(d.path(), &["unrelated.txt"], &[]);

    let report = analyze(d.path(), &[String::new()]).expect("analyzes");
    ledger::write(d.path(), &report, false).expect("a clean tree must be allowed to write");
    assert!(d.path().join(ledger::LEDGER_PATH).is_file());
}

#[test]
fn counter_allow_dirty_overrides_the_refusal() {
    let d = tempfile::tempdir().expect("tempdir");
    write_analyzable_tree(d.path());
    git_repo(d.path(), &["unrelated.txt"], &["unrelated.txt"]);

    let report = analyze(d.path(), &[String::new()]).expect("analyzes");
    ledger::write(d.path(), &report, true).expect("--allow-dirty is an explicit, loud opt-out");
    assert!(d.path().join(ledger::LEDGER_PATH).is_file());
}

#[test]
fn counter_write_proceeds_outside_a_git_repository() {
    // Not every build has git metadata (a crates.io tarball, for instance);
    // there is no commit to be inconsistent with, so nothing to refuse.
    let d = tempfile::tempdir().expect("tempdir");
    write_analyzable_tree(d.path());

    let report = analyze(d.path(), &[String::new()]).expect("analyzes");
    ledger::write(d.path(), &report, false).expect("no git metadata must not block a write");
    assert!(d.path().join(ledger::LEDGER_PATH).is_file());
}

#[cfg(test)]
mod parser_health_tests {
    use super::super::{declared_feature_count, parser_is_intact};

    const FOUR: &str = "\
[package]
name = \"corpus\"

[features]
default = [\"std\"]
std = []
simd = []
tracing = []

[[bench]]
name = \"throughput\"
";

    /// A small crate is not a broken parser.
    ///
    /// The guard was `if graph.len() < 40`, a threshold calibrated to pmat's own
    /// manifest and then stated as a fact about the parser. A fixture declaring
    /// exactly four features was told "only 4 features parsed from [features] —
    /// the parser is broken, not the crate", and a crate with no [features]
    /// table was told the same about zero. Both were false, both were exit 1,
    /// and between them they made `analyze unrun-tests` unusable on any crate
    /// but this one.
    #[test]
    fn a_small_feature_table_is_not_a_broken_parser() {
        assert_eq!(declared_feature_count(FOUR), 4);
        parser_is_intact(FOUR, 4).expect("four declared and four parsed is intact");
    }

    #[test]
    fn a_crate_with_no_features_table_is_not_a_broken_parser() {
        let manifest = "[package]\nname = \"tiny\"\n";
        assert_eq!(declared_feature_count(manifest), 0);
        parser_is_intact(manifest, 0).expect("no table to lose entries from");
    }

    /// The guard must still catch what it was written for: entries dropped.
    ///
    /// Without this the fix would be "delete the check", which passes both
    /// tests above and detects nothing.
    #[test]
    fn dropping_declared_features_is_still_refused() {
        let err = parser_is_intact(FOUR, 1).expect_err("1 parsed of 4 declared is a broken parser");
        assert!(err.contains("declares 4"), "{err}");
        assert!(err.contains("only 1 parsed"), "{err}");
    }

    /// Continuation lines of a multi-line value are not extra features.
    #[test]
    fn a_multi_line_feature_value_counts_once() {
        let manifest = "\
[features]
default = [
    \"std\",
    \"simd\",
]
std = []
";
        assert_eq!(declared_feature_count(manifest), 2);
    }

    /// End-to-end: `analyze` on a small crate must not blame its own parser.
    ///
    /// The tests above call the guard directly, so they do not cover the call
    /// site where the threshold actually lived. This one does: a four-feature
    /// crate reaches `analyze` and must get past the manifest check. It still
    /// fails afterwards — no CI workflow means no test leg, which is a
    /// different and legitimate refusal — so the assertion is on WHICH error.
    #[test]
    fn analyze_does_not_blame_the_parser_on_a_small_crate() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        std::fs::write(dir.path().join("Cargo.toml"), FOUR).expect("write manifest");

        let err = super::super::analyze(dir.path(), &[])
            .expect_err("no workflows means no leg, so this still fails");
        assert!(
            !err.contains("the parser is broken"),
            "a four-feature crate is not a broken parser: {err}"
        );
        assert!(
            err.contains("cargo test") || err.contains("leg"),
            "expected the no-leg refusal, got: {err}"
        );
    }

    /// The counter must agree with the parser on pmat's own manifest — the one
    /// input where the old threshold happened to be right.
    #[test]
    fn the_two_counts_agree_on_this_crate() {
        let manifest = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
            .expect("read our own Cargo.toml");
        let parsed = super::super::features::parse(&manifest).len();
        let declared = declared_feature_count(&manifest);
        assert_eq!(
            parsed, declared,
            "the independent count and the parser disagree on our own manifest"
        );
        parser_is_intact(&manifest, parsed).expect("our own manifest must be intact");
    }
}
