//! One skip-or-grade rule, every surface — R14.
//!
//! The critical-defect gate was unified across both `analyze_source`
//! implementations, but "is this file gradable at all" was not, so the
//! contradiction simply moved: for a committed `.rs` file holding three
//! `Option::unwrap()` calls, `pmat tdg <repo>/tests/bad.rs` answered
//!
//! ```text
//! {"analyzed":false,"skipped":true,
//!  "skip_reason":"test-or-bench file: TDG does not grade test sources",
//!  "score":null,"not_measured":["score","grade"]}
//! ```
//!
//! while the MCP `quality_gate` tool answered
//! `{"passed":true,"score":90.0,"grade":"A","not_measured":[],
//! "files_analyzed":1,"blocking_violations":0}` for the same bytes — the
//! untouched component caps, which is what the heuristic analyzer returns when
//! its regex heuristics recognise nothing. Unmeasured came back as an A.
//!
//! Every test here fixes a path a verdict can travel and asserts the SAME
//! answer arrives.

use std::path::{Path, PathBuf};

use crate::tdg::file_discovery::{self, Policy, TEST_SOURCE_SKIP_REASON};

/// Three `Option::unwrap()` calls: 3 critical defects wherever the gate runs.
const THREE_UNWRAPS: &str = "pub fn a(x: Option<i32>) -> i32 { x.unwrap() }\n\
                             pub fn b(x: Option<i32>) -> i32 { x.unwrap() }\n\
                             pub fn c(x: Option<i32>) -> i32 { x.unwrap() }\n";

fn write(dir: &Path, relative: &str, body: &str) -> PathBuf {
    let path = dir.join(relative);
    std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    std::fs::write(&path, body).expect("write");
    path
}

/// The paths `pmat tdg` declines. MCP graded every one of them 90.0/A.
fn test_source_paths(root: &Path) -> Vec<PathBuf> {
    [
        "tests/bad.rs",
        "benches/bad.rs",
        "examples/bad.rs",
        "fuzz/bad.rs",
    ]
    .iter()
    .map(|relative| write(root, relative, THREE_UNWRAPS))
    .chain(
        ["src/bad_test.rs", "src/bad_tests.rs", "src/test_bad.rs"]
            .iter()
            .map(|relative| write(root, relative, THREE_UNWRAPS)),
    )
    .collect()
}

/// The refusal is a property of the path, not of the analyzer that asked.
///
/// RED before the fix: `Policy::heuristic()` had `skip_tests: false`, so the
/// two policies disagreed by construction and the heuristic analyzer — the one
/// behind MCP `quality_gate` — had no skip rule at all.
#[test]
fn both_analyzer_policies_refuse_the_same_test_sources() {
    let dir = tempfile::tempdir().expect("tempdir");
    for path in test_source_paths(dir.path()) {
        let ast = file_discovery::refusal(&path, Policy::ast());
        let heuristic = file_discovery::refusal(&path, Policy::heuristic());
        assert_eq!(
            ast,
            heuristic,
            "{} must get ONE skip-or-grade answer, not one per analyzer",
            path.display()
        );
        assert_eq!(
            ast.as_deref(),
            Some(TEST_SOURCE_SKIP_REASON),
            "{} is test or bench source",
            path.display()
        );
    }
}

/// The gradability predicate MCP `quality_gate` consults before it grades.
///
/// RED before the fix: `grades_source` tested the extension alone, answered
/// `true` for `tests/bad.rs`, and the 90.0/A followed.
#[test]
fn grades_source_refuses_test_sources_and_says_why() {
    let dir = tempfile::tempdir().expect("tempdir");
    for path in test_source_paths(dir.path()) {
        assert!(
            !crate::tdg::grades_source(&path),
            "{} is test source; grading it is how MCP produced 90.0/A for a file \
             `pmat tdg` reports as skipped",
            path.display()
        );
        assert_eq!(
            crate::tdg::analyzer_simple::not_gradable_reason(&path).as_deref(),
            Some(TEST_SOURCE_SKIP_REASON),
            "the reason must be the one `pmat tdg` publishes in `skip_reason`"
        );
    }
}

/// A refusal must not turn into a score. Byte-identical production source is
/// still graded, so this is a skip rule and not a blanket refusal.
#[test]
fn the_heuristic_analyzer_grades_production_source_and_refuses_test_source() {
    let dir = tempfile::tempdir().expect("tempdir");
    let production = write(dir.path(), "src/widget.rs", THREE_UNWRAPS);
    let analyzer = crate::tdg::TdgAnalyzerSimple::new().expect("analyzer");

    let graded = analyzer
        .analyze_file(&production)
        .expect("production source is graded");
    assert!(
        graded.total < 70.0,
        "three unwraps are three critical defects: {} is not a verdict",
        graded.total
    );

    for path in test_source_paths(dir.path()) {
        let refused = analyzer
            .analyze_file(&path)
            .expect_err("test source must come back refused, never as a score");
        assert!(
            refused.to_string().contains(TEST_SOURCE_SKIP_REASON),
            "refusal must name the rule, got: {refused}"
        );
    }
}

/// The AST analyzer behind `pmat tdg` applies the same rule to the same paths.
///
/// RED before the fix: `src/bad_tests.rs` was graded 100.0/A+ — the defect
/// detector excluded it from detection while the grader graded it anyway, so an
/// exclusion became a perfect score for bytes that score 25.164/F next door.
#[tokio::test]
async fn the_ast_analyzer_refuses_what_the_heuristic_analyzer_refuses() {
    let dir = tempfile::tempdir().expect("tempdir");
    let analyzer = crate::tdg::TdgAnalyzer::new().expect("analyzer");

    for path in test_source_paths(dir.path()) {
        let refused = analyzer
            .analyze_file(&path)
            .await
            .expect_err("test source must come back refused, never as a score");
        assert!(
            refused.to_string().contains(TEST_SOURCE_SKIP_REASON),
            "refusal must name the rule, got: {refused}"
        );
    }
}

/// The reported symptom, end to end through the MCP entry point.
///
/// RED before the fix: `passed: true`, `score: 90.0`, `grade: "A"`,
/// `not_measured: []`, `files_analyzed: 1`.
#[tokio::test]
async fn mcp_quality_gate_reports_a_skipped_file_as_unmeasured_not_as_an_a() {
    let dir = tempfile::tempdir().expect("tempdir");

    for path in test_source_paths(dir.path()) {
        let verdict = crate::mcp_pmcp::tool_functions::check_quality_gates(
            std::slice::from_ref(&path),
            false,
        )
        .await
        .expect("gate reports");

        assert_eq!(
            verdict["passed"],
            false,
            "{}: a file this build refuses to grade must not pass a gate — {verdict}",
            path.display()
        );
        assert!(
            verdict["score"].is_null() && verdict["grade"].is_null(),
            "{}: unmeasured is null, not 90.0/A — {verdict}",
            path.display()
        );
        assert_eq!(
            verdict["files_analyzed"],
            0,
            "{}: nothing was graded — {verdict}",
            path.display()
        );
        let not_measured = verdict["not_measured"]
            .as_array()
            .expect("not_measured array");
        assert!(
            not_measured.iter().any(|v| v == "score"),
            "{}: `not_measured: []` is a positive claim of full coverage — {verdict}",
            path.display()
        );
    }
}

/// A project walk must not put test source back in through the other door.
#[test]
fn a_project_walk_excludes_test_source_for_both_policies() {
    let dir = tempfile::tempdir().expect("tempdir");
    write(dir.path(), "src/widget.rs", THREE_UNWRAPS);
    let hidden = test_source_paths(dir.path());

    for policy in [Policy::ast(), Policy::heuristic()] {
        let found = file_discovery::discover(dir.path(), policy).expect("walk");
        assert_eq!(
            found.gradable.len(),
            1,
            "only src/widget.rs is production source, got {:?}",
            found.gradable
        );
        for path in &hidden {
            assert!(
                !found.ungraded.iter().any(|(file, _)| file == path),
                "{} is excluded by policy, not a hole in the verdict",
                path.display()
            );
        }
    }
}

/// The critical-defect count is a property of the bytes, which is what
/// `critical_defect_gate`'s own doc comment claims and what its detector
/// contradicted.
///
/// RED before the fix: the gate handed the analysed path to
/// `RustDefectDetector::detect`, which opens with its own private copy of the
/// test-source rule and returns an empty vector for anything under `examples/`
/// — so identical bytes counted 3 defects at one path and 0 at another, and an
/// exclusion from DETECTION silently became a perfect GRADE.
#[test]
fn the_defect_count_does_not_depend_on_where_the_bytes_sit() {
    use crate::tdg::Language;

    let analyzer = crate::tdg::TdgAnalyzerSimple::new().expect("analyzer");
    let production = analyzer
        .analyze_source(
            THREE_UNWRAPS,
            Language::Rust,
            Some(PathBuf::from("/w/widget.rs")),
        )
        .expect("scored");
    assert_eq!(production.critical_defects_count, 3, "three unwraps");

    for label in [
        "/w/examples/bad.rs",
        "/w/benches/bad.rs",
        "/w/src/bad_tests.rs",
    ] {
        let elsewhere = analyzer
            .analyze_source(THREE_UNWRAPS, Language::Rust, Some(PathBuf::from(label)))
            .expect("scored");
        assert_eq!(
            elsewhere.critical_defects_count, production.critical_defects_count,
            "{label}: the same bytes carry the same defects; a detector that \
             excludes by PATH turns 'not detected here' into a perfect grade"
        );
        assert_eq!(
            elsewhere.total, production.total,
            "{label}: and therefore the same score"
        );
    }
}

/// R13 residual #1: an extensionless shebang script was dropped in silence
/// while byte-identical `.sh` was disclosed as a hole.
///
/// RED before the fix: `classify` returned "not in the population" for every
/// extensionless path, so `deploy`, `configure` and `run` vanished from
/// `ungraded` — and `not_measured` is the field a reader consults to learn what
/// a verdict does NOT cover.
#[test]
fn a_shebang_script_is_disclosed_like_its_dotted_twin() {
    let dir = tempfile::tempdir().expect("tempdir");
    let script = "#!/usr/bin/env bash\nset -euo pipefail\ndeploy() { :; }\n";
    write(dir.path(), "deploy", script);
    write(dir.path(), "deploy.sh", script);
    // Not source, and must stay out of the population: a shebang is the only
    // thing that puts an extensionless file in.
    write(dir.path(), "LICENSE", "All rights reserved.\n");

    let found = file_discovery::discover(dir.path(), Policy::heuristic()).expect("walk");
    let disclosed: Vec<String> = found
        .ungraded
        .iter()
        .filter_map(|(path, _)| path.file_name()?.to_str().map(str::to_string))
        .collect();

    assert!(
        disclosed.contains(&"deploy".to_string()),
        "an extensionless shell script is source this build cannot grade, and a \
         silent drop is exactly what `not_measured: []` misreports: {disclosed:?}"
    );
    assert!(
        disclosed.contains(&"deploy.sh".to_string()),
        "the dotted twin was already disclosed: {disclosed:?}"
    );
    assert!(
        !disclosed.contains(&"LICENSE".to_string()),
        "a licence is not source and is not a hole in a source average: {disclosed:?}"
    );
}

/// Every language the walk refuses is NAMED, whether the canonical registry
/// knows it or the local gap list does. Deleting the gap list outright would
/// have turned these back into silent drops.
#[test]
fn ungradable_source_is_named_from_one_authority_plus_its_declared_gaps() {
    let dir = tempfile::tempdir().expect("tempdir");
    // First three come from `services::language_registry`, last three from the
    // declared gap list; the caller cannot tell, which is the point.
    for name in ["a.sh", "a.zig", "a.php", "a.f90", "a.vue", "a.awk"] {
        write(dir.path(), name, "x\n");
    }
    write(dir.path(), "a.json", "{}\n");

    let found = file_discovery::discover(dir.path(), Policy::heuristic()).expect("walk");
    let disclosed: Vec<String> = found
        .ungraded
        .iter()
        .filter_map(|(path, _)| path.file_name()?.to_str().map(str::to_string))
        .collect();

    for name in ["a.sh", "a.zig", "a.php", "a.f90", "a.vue", "a.awk"] {
        assert!(
            disclosed.contains(&name.to_string()),
            "{name} is source this build cannot grade and must be disclosed: {disclosed:?}"
        );
    }
    assert!(
        !disclosed.contains(&"a.json".to_string()),
        "data is not a hole in a source average: {disclosed:?}"
    );
}
