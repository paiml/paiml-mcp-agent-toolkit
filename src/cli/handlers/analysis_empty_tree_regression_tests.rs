//! Regression tests: an analyzer must never report a clean zero over a tree it
//! never opened.
//!
//! #1015. [`crate::cli::ensure_analysis_path_exists`] closed the "the tree is
//! not there" hole; this file pins the one behind it. The path exists, is
//! readable, and holds no source file — and eight analyzers answered that with
//! a full report of zeros and exit 0, byte-for-byte the document a genuinely
//! clean tree produces:
//!
//! | command | what an empty directory printed |
//! |---|---|
//! | `analyze dag` | `graph TD` |
//! | `analyze duplicates` | `Duplication: 0.0% (0 / 0 lines)` |
//! | `analyze big-o` | `Total Functions Analyzed: 0` + eight zero buckets |
//! | `analyze provability` | `Average provability score: 0.0%` |
//! | `analyze deep-context` | `Files Analyzed: 0 / Average Complexity: 0.0` |
//! | `analyze symbol-table` | `Total symbols: 0` |
//! | `analyze graph-metrics` | `Total nodes: 0 / Density: 0.000` |
//! | `analyze proof-annotations` | `Total proofs: 0 / High confidence: 0 (0.0%)` |
//!
//! Each is now refused with the sentence `analyze satd` already used, so a CI
//! gate pointed at the wrong directory goes red instead of green.
//!
//! Every analyzer is exercised against THREE trees, because a refusal that also
//! fires on real input is a worse bug than the one it fixes:
//!
//! 1. an empty directory — must refuse;
//! 2. a NON-GIT directory holding Rust sources — must measure normally (none of
//!    these eight enumerate files through `git ls-files`, and that must stay
//!    true: `analyze hardcoded-paths` does, and refuses outside a repo);
//! 3. the same sources inside a git repository — must measure normally.
//!
//! Fixtures 2 and 3 are what make this file able to fail in the other
//! direction. A version of these tests that only checked the empty directory
//! would pass just as well against an analyzer that refused everything.
//!
//! A second pass over EVERY analyzer — not just the eight above — found five
//! more, `analyze comprehensive` worst among them. See the block partway down
//! this file.

use std::path::Path;
use tempfile::TempDir;

/// Two Rust files with functions, imports and a nested loop: enough for all
/// eight analyzers to have something to say (nodes, symbols, functions,
/// annotations, blocks).
fn write_sources(root: &Path) {
    std::fs::create_dir_all(root.join("src")).expect("mkdir src");
    std::fs::write(
        root.join("src/lib.rs"),
        "pub mod helper;\n\
         pub fn add(a: i32, b: i32) -> i32 { a + b }\n\
         pub fn pairs(items: &[i32]) -> i32 {\n\
        \x20   let mut total = 0;\n\
        \x20   for i in items { for j in items { total += i * j; } }\n\
        \x20   total\n\
         }\n",
    )
    .expect("write lib.rs");
    std::fs::write(
        root.join("src/helper.rs"),
        "pub fn double(x: i32) -> i32 { x * 2 }\n\
         pub fn halve(x: i32) -> i32 { x / 2 }\n",
    )
    .expect("write helper.rs");
    std::fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .expect("write Cargo.toml");
}

/// A readable directory with nothing in it: the case every analyzer answered
/// with a clean zero.
fn empty_tree() -> TempDir {
    TempDir::new().expect("tempdir")
}

/// Rust sources with no `.git` anywhere. Nothing here may reach for git.
fn non_git_tree() -> TempDir {
    let dir = TempDir::new().expect("tempdir");
    write_sources(dir.path());
    assert!(
        !dir.path().join(".git").exists(),
        "the non-git fixture must not be a repository"
    );
    dir
}

/// The same sources, committed to a repository.
fn git_tree() -> TempDir {
    let dir = TempDir::new().expect("tempdir");
    write_sources(dir.path());
    let git = |args: &[&str]| {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(dir.path())
            .env("GIT_AUTHOR_NAME", "t")
            .env("GIT_AUTHOR_EMAIL", "t@example.invalid")
            .env("GIT_COMMITTER_NAME", "t")
            .env("GIT_COMMITTER_EMAIL", "t@example.invalid")
            .output()
            .expect("git");
        assert!(
            out.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    };
    git(&["init", "-q", "--template=", "--initial-branch=main"]);
    git(&["add", "-A"]);
    git(&["commit", "-q", "--no-verify", "-m", "fixture"]);
    dir
}

/// The refusal every one of the eight must produce, in the wording
/// `analyze satd` established.
fn assert_refused_as_unmeasured(result: anyhow::Result<()>, path: &Path, what: &str) {
    let message = format!(
        "{:#}",
        result.expect_err(&format!(
            "{what}: an empty tree must be refused, not reported as a clean zero"
        ))
    );
    assert!(
        message.contains("no source files were found"),
        "{what}: the refusal must name what was missing, got: {message}"
    );
    assert!(
        message.contains("This is not a clean result"),
        "{what}: the refusal must say the zero is not clean, got: {message}"
    );
    assert!(
        message.contains(&path.display().to_string()),
        "{what}: the refusal must name the path it refused, got: {message}"
    );
}

fn assert_measured(result: anyhow::Result<()>, what: &str, tree: &str) {
    if let Err(e) = result {
        panic!("{what}: must still measure a {tree} tree normally, got: {e:#}");
    }
}

// ---------------------------------------------------------------- dag

async fn run_dag(root: &Path, out: Option<std::path::PathBuf>) -> anyhow::Result<()> {
    super::complexity_handlers::handle_analyze_dag(
        crate::cli::DagType::FullDependency,
        root.to_path_buf(),
        out,
        None,
        None,
        false,
        false,
        false,
        false,
        false,
    )
    .await
}

#[tokio::test]
async fn dag_refuses_an_empty_tree_and_still_graphs_a_real_one() {
    let empty = empty_tree();
    assert_refused_as_unmeasured(run_dag(empty.path(), None).await, empty.path(), "dag");

    for (tree, label) in [(non_git_tree(), "non-git"), (git_tree(), "git")] {
        let out = tree.path().join("dag.mmd");
        assert_measured(run_dag(tree.path(), Some(out.clone())).await, "dag", label);
        let rendered = std::fs::read_to_string(&out).expect("dag output");
        assert!(
            rendered.lines().count() > 1,
            "dag: a {label} tree must render more than the bare `graph TD` header, got: {rendered}"
        );
    }
}

/// The refusal must NOT swallow the "measured, and it is empty" case that
/// `DagBuildStats::explain_empty` exists to report: files parsed, no edge of
/// the requested type. That is a real measurement and keeps exit 0.
#[tokio::test]
async fn dag_still_succeeds_when_files_parsed_but_the_requested_graph_is_empty() {
    let dir = TempDir::new().expect("tempdir");
    std::fs::write(dir.path().join("consts.py"), "x = 1\ny = 2\n").expect("write");

    let out = dir.path().join("dag.mmd");
    super::complexity_handlers::handle_analyze_dag(
        crate::cli::DagType::CallGraph,
        dir.path().to_path_buf(),
        Some(out),
        None,
        None,
        false,
        false,
        false,
        false,
        false,
    )
    .await
    .expect("a parsed tree with no call edges is a measurement, not a refusal");
}

// ---------------------------------------------------------------- duplicates

async fn run_duplicates(root: &Path, out: Option<std::path::PathBuf>) -> anyhow::Result<()> {
    use super::duplication_analysis::{handle_analyze_duplicates, DuplicateAnalysisConfig};
    handle_analyze_duplicates(DuplicateAnalysisConfig {
        project_path: root.to_path_buf(),
        detection_type: crate::cli::DuplicateType::Exact,
        threshold: 0.8,
        min_lines: 3,
        max_tokens: 100,
        format: crate::cli::DuplicateOutputFormat::Json,
        perf: false,
        include: None,
        exclude: None,
        output: out,
        top_files: 0,
    })
    .await
}

#[tokio::test]
async fn duplicates_refuses_an_empty_tree_and_still_measures_a_real_one() {
    let empty = empty_tree();
    assert_refused_as_unmeasured(
        run_duplicates(empty.path(), None).await,
        empty.path(),
        "duplicates",
    );

    for (tree, label) in [(non_git_tree(), "non-git"), (git_tree(), "git")] {
        let out = tree.path().join("dup.json");
        assert_measured(
            run_duplicates(tree.path(), Some(out.clone())).await,
            "duplicates",
            label,
        );
        let report: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&out).expect("dup report"))
                .expect("dup json");
        assert!(
            report["total_lines"].as_u64().unwrap_or(0) > 0,
            "duplicates: a {label} tree must have a non-zero denominator, got: {report}"
        );
    }
}

// ---------------------------------------------------------------- big-o

async fn run_big_o(root: &Path, out: Option<std::path::PathBuf>) -> anyhow::Result<()> {
    super::big_o_handlers::handle_analyze_big_o(
        root.to_path_buf(),
        crate::cli::BigOOutputFormat::Json,
        70,
        false,
        vec![],
        vec![],
        false,
        out,
        false,
        0,
    )
    .await
}

#[tokio::test]
async fn big_o_refuses_an_empty_tree_and_still_measures_a_real_one() {
    let empty = empty_tree();
    assert_refused_as_unmeasured(run_big_o(empty.path(), None).await, empty.path(), "big-o");

    for (tree, label) in [(non_git_tree(), "non-git"), (git_tree(), "git")] {
        let out = tree.path().join("bigo.json");
        assert_measured(
            run_big_o(tree.path(), Some(out.clone())).await,
            "big-o",
            label,
        );
        let text = std::fs::read_to_string(&out).expect("big-o report");
        assert!(
            !text.contains("\"analyzed_functions\": 0"),
            "big-o: a {label} tree declares functions, got: {text}"
        );
    }
}

// ---------------------------------------------------------------- provability

async fn run_provability(root: &Path, out: Option<std::path::PathBuf>) -> anyhow::Result<()> {
    use super::provability_handler::{handle_analyze_provability, ProvabilityConfig};
    handle_analyze_provability(ProvabilityConfig {
        project_path: root.to_path_buf(),
        functions: vec![],
        analysis_depth: crate::services::lightweight_provability_analyzer::ANALYSIS_DEPTH,
        format: crate::cli::ProvabilityOutputFormat::Json,
        high_confidence_only: false,
        include_evidence: false,
        output: out,
        top_files: 0,
    })
    .await
}

#[tokio::test]
async fn provability_refuses_an_empty_tree_and_still_scores_a_real_one() {
    let empty = empty_tree();
    assert_refused_as_unmeasured(
        run_provability(empty.path(), None).await,
        empty.path(),
        "provability",
    );

    for (tree, label) in [(non_git_tree(), "non-git"), (git_tree(), "git")] {
        let out = tree.path().join("prov.json");
        assert_measured(
            run_provability(tree.path(), Some(out.clone())).await,
            "provability",
            label,
        );
        let report: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&out).expect("prov report"))
                .expect("prov json");
        assert!(
            report["provability_analysis"]["results"]
                .as_array()
                .is_some_and(|r| !r.is_empty()),
            "provability: a {label} tree must yield scored functions, got: {report}"
        );
    }
}

// ---------------------------------------------------------------- deep-context

async fn run_deep_context(
    root: &Path,
    format: crate::cli::DeepContextOutputFormat,
    out: Option<std::path::PathBuf>,
) -> anyhow::Result<()> {
    super::advanced_analysis_handlers::handle_analyze_deep_context(
        root.to_path_buf(),
        out,
        format,
        false,
        vec![],
        vec![],
        30,
        None,
        None,
        vec![],
        vec![],
        None,
        false,
        false,
        0,
    )
    .await
}

#[tokio::test]
async fn deep_context_refuses_an_empty_tree_and_still_reports_a_real_one() {
    // Both analyzers behind this command: `--format sarif` runs
    // `DeepContextAnalyzer`, everything else runs `SimpleDeepContext`, and each
    // needed the refusal stated over its own denominator.
    for format in [
        crate::cli::DeepContextOutputFormat::Json,
        crate::cli::DeepContextOutputFormat::Sarif,
    ] {
        let empty = empty_tree();
        let label = format!("deep-context --format {format}");
        assert_refused_as_unmeasured(
            run_deep_context(empty.path(), format, None).await,
            empty.path(),
            &label,
        );
    }

    for (tree, label) in [(non_git_tree(), "non-git"), (git_tree(), "git")] {
        let out = tree.path().join("ctx.json");
        assert_measured(
            run_deep_context(
                tree.path(),
                crate::cli::DeepContextOutputFormat::Json,
                Some(out.clone()),
            )
            .await,
            "deep-context",
            label,
        );
        let text = std::fs::read_to_string(&out).expect("deep-context report");
        assert!(
            !text.is_empty(),
            "deep-context: a {label} tree must produce a report"
        );
    }
}

// ---------------------------------------------------------------- symbol-table

async fn run_symbol_table(root: &Path, out: Option<std::path::PathBuf>) -> anyhow::Result<()> {
    super::advanced_analysis_handlers::handle_analyze_symbol_table(
        root.to_path_buf(),
        crate::cli::SymbolTableOutputFormat::Json,
        None,
        None,
        vec![],
        vec![],
        false,
        false,
        out,
        false,
        0,
    )
    .await
}

#[tokio::test]
async fn symbol_table_refuses_an_empty_tree_and_still_lists_a_real_one() {
    let empty = empty_tree();
    assert_refused_as_unmeasured(
        run_symbol_table(empty.path(), None).await,
        empty.path(),
        "symbol-table",
    );

    for (tree, label) in [(non_git_tree(), "non-git"), (git_tree(), "git")] {
        let out = tree.path().join("symbols.json");
        assert_measured(
            run_symbol_table(tree.path(), Some(out.clone())).await,
            "symbol-table",
            label,
        );
        let report: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&out).expect("symbol report"))
                .expect("symbol json");
        assert!(
            report["total_symbols"].as_u64().unwrap_or(0) > 0,
            "symbol-table: a {label} tree declares symbols, got: {report}"
        );
    }
}

// ---------------------------------------------------------------- graph-metrics

async fn run_graph_metrics(root: &Path, out: Option<std::path::PathBuf>) -> anyhow::Result<()> {
    super::advanced_analysis_handlers::handle_analyze_graph_metrics(
        root.to_path_buf(),
        vec![crate::cli::GraphMetricType::All],
        vec![],
        0.85,
        20,
        1e-6,
        false,
        crate::cli::GraphMetricsOutputFormat::Json,
        None,
        None,
        out,
        false,
        0,
        0.0,
    )
    .await
}

#[tokio::test]
async fn graph_metrics_refuses_an_empty_tree_and_still_measures_a_real_one() {
    let empty = empty_tree();
    assert_refused_as_unmeasured(
        run_graph_metrics(empty.path(), None).await,
        empty.path(),
        "graph-metrics",
    );

    for (tree, label) in [(non_git_tree(), "non-git"), (git_tree(), "git")] {
        let out = tree.path().join("graph.json");
        assert_measured(
            run_graph_metrics(tree.path(), Some(out.clone())).await,
            "graph-metrics",
            label,
        );
        let text = std::fs::read_to_string(&out).expect("graph report");
        assert!(
            !text.is_empty(),
            "graph-metrics: a {label} tree must produce metrics"
        );
    }
}

// ------------------------------------------------------------ proof-annotations

async fn run_proof_annotations(root: &Path, out: Option<std::path::PathBuf>) -> anyhow::Result<()> {
    super::proof_annotations_handler::handle_analyze_proof_annotations(
        root.to_path_buf(),
        crate::cli::ProofAnnotationOutputFormat::Json,
        false,
        false,
        None,
        None,
        out,
        false,
        false,
        0,
    )
    .await
}

#[tokio::test]
async fn proof_annotations_refuses_an_empty_tree_and_still_collects_from_a_real_one() {
    let empty = empty_tree();
    assert_refused_as_unmeasured(
        run_proof_annotations(empty.path(), None).await,
        empty.path(),
        "proof-annotations",
    );

    for (tree, label) in [(non_git_tree(), "non-git"), (git_tree(), "git")] {
        let out = tree.path().join("proofs.json");
        assert_measured(
            run_proof_annotations(tree.path(), Some(out.clone())).await,
            "proof-annotations",
            label,
        );
        let text = std::fs::read_to_string(&out).expect("proof report");
        assert!(
            !text.is_empty(),
            "proof-annotations: a {label} tree must produce a report"
        );
    }
}

/// A scanned file that declares nothing annotatable is a MEASURED zero and must
/// keep succeeding — the refusal is for "nothing was scanned", never for
/// "scanned and found none". Without this, the fix could be satisfied by
/// refusing every empty report, which would be a new lie in the other
/// direction.
#[tokio::test]
async fn proof_annotations_still_reports_a_measured_zero() {
    let dir = TempDir::new().expect("tempdir");
    std::fs::write(
        dir.path().join("consts.rs"),
        "pub const X: i32 = 1;\npub struct S;\n",
    )
    .expect("write");

    let out = dir.path().join("proofs.json");
    run_proof_annotations(dir.path(), Some(out.clone()))
        .await
        .expect("a scanned file with no annotations is a measurement");
    let text = std::fs::read_to_string(&out).expect("proof report");
    assert!(
        !text.is_empty(),
        "a measured zero is still a report, got nothing"
    );
}

// ===========================================================================
// Second pass. The eight above were fixed and re-verified; running EVERY
// analyzer against an empty directory afterwards turned up five more that
// still exited 0 over a tree they read nothing from:
//
// | command | what an empty directory printed |
// |---|---|
// | `analyze comprehensive` | `Quality Score: 100.0%` + "Code quality looks good!" |
// | `analyze complexity` | `Files analyzed: 0` + `Median Cyclomatic: 0.0` on STDOUT |
// | `analyze assembly-script` | `**Files analyzed**: 0` |
// | `analyze web-assembly` | `**Files analyzed**: 0` |
// | `analyze name-similarity` | `Found: 0 matches` |
//
// `comprehensive` is the worst of them and the reason this pass exists: it
// runs `analyze satd`, CAUGHT satd's refusal ("no source files were found …
// This is not a clean result"), printed it as `Warning: satd analysis failed`,
// and then awarded the tree a perfect score — a passing command wrapped around
// a refusing one.
//
// `name-similarity` is the subtlest: its stdout over an empty directory was
// BYTE-IDENTICAL to its stdout over a real codebase for a query that genuinely
// matches nothing, because the report printed the numerator and never the
// denominator.
// ===========================================================================

/// The AssemblyScript source the `.as`/`.ts` fixtures use. `i32` alone is
/// enough for `WasmLanguageDetector::is_assemblyscript`, but this is a real
/// function so the complexity analyzer has something to measure.
const ASSEMBLYSCRIPT_FIXTURE: &str =
    "export function add(a: i32, b: i32): i32 {\n  return a + b;\n}\n";

/// The eight bytes every WebAssembly module starts with: `\0asm` + version 1.
/// `WasmBinaryAnalyzer::analyze` accepts it; anything shorter is rejected as
/// "File too small to be valid WASM", which would make the fixture prove the
/// wrong thing.
const WASM_HEADER: &[u8] = &[0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];

/// A text-format module: parsed for the `--security`/`--complexity` rules, but
/// contributing no `WasmMetrics` row by design.
const WAT_FIXTURE: &str = "(module\n  (memory 2)\n  (func $add (param i32 i32) (result i32)\n    local.get 0\n    local.get 1\n    i32.add)\n  (export \"add\" (func $add)))\n";

/// Like [`assert_refused_as_unmeasured`] but for the analyzers whose population
/// is not "source files" — the sentence is the same, the noun is the one that
/// is true (`no AssemblyScript files were found under …`).
fn assert_refused_naming(result: anyhow::Result<()>, path: &Path, population: &str, what: &str) {
    let message = format!(
        "{:#}",
        result.expect_err(&format!(
            "{what}: an empty tree must be refused, not reported as a clean zero"
        ))
    );
    assert!(
        message.contains(&format!("no {population} were found")),
        "{what}: the refusal must name the population that was missing, got: {message}"
    );
    assert!(
        message.contains("This is not a clean result"),
        "{what}: the refusal must say the zero is not clean, got: {message}"
    );
    assert!(
        message.contains(&path.display().to_string()),
        "{what}: the refusal must name the path it refused, got: {message}"
    );
}

// ---------------------------------------------------------------- comprehensive

fn comprehensive_config(
    root: &Path,
    out: Option<std::path::PathBuf>,
) -> super::comprehensive_analysis_handler::ComprehensiveAnalysisConfig {
    super::comprehensive_analysis_handler::ComprehensiveAnalysisConfig {
        project_path: root.to_path_buf(),
        file: None,
        files: vec![],
        format: crate::cli::ComprehensiveOutputFormat::Json,
        // Defect prediction and duplicate detection are the two sub-analyses
        // that shell out / walk again; off here so the fixture runs fast. The
        // orchestrated three (complexity, dead code, SATD) are what
        // `total_files` comes from.
        include_duplicates: false,
        include_dead_code: true,
        include_defects: false,
        include_complexity: true,
        include_tdg: true,
        confidence_threshold: 0.5,
        min_lines: 1,
        include: None,
        exclude: None,
        output: out,
        perf: false,
        executive_summary: true,
        top_files: 10,
    }
}

/// The blocker: `Quality Score: 100.0%`, `Total Files: 0`, `Total Issues: 0`
/// and "Code quality looks good! Continue following best practices." over a
/// directory nothing was read from, exit 0.
#[tokio::test]
async fn comprehensive_refuses_an_empty_tree_and_still_scores_a_real_one() {
    let empty = empty_tree();
    assert_refused_as_unmeasured(
        super::comprehensive_analysis_handler::handle_analyze_comprehensive(comprehensive_config(
            empty.path(),
            None,
        ))
        .await,
        empty.path(),
        "comprehensive",
    );

    for (tree, label) in [(non_git_tree(), "non-git"), (git_tree(), "git")] {
        let out = tree.path().join("comprehensive.json");
        assert_measured(
            super::comprehensive_analysis_handler::handle_analyze_comprehensive(
                comprehensive_config(tree.path(), Some(out.clone())),
            )
            .await,
            "comprehensive",
            label,
        );
        let report: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&out).expect("comprehensive report"))
                .expect("comprehensive json");
        assert!(
            report["summary"]["total_files"].as_u64().unwrap_or(0) > 0,
            "comprehensive: a {label} tree must have a non-zero denominator, got: {report}"
        );
        assert!(
            report["summary"]["quality_score"].as_f64().is_some(),
            "comprehensive: a {label} tree HAS a score and must still print it, got: {report}"
        );
    }
}

/// The refusal must name the score, so a CI log says which measurement went
/// missing rather than only that something did.
#[tokio::test]
async fn comprehensive_refusal_names_the_quality_score() {
    let empty = empty_tree();
    let message = format!(
        "{:#}",
        super::comprehensive_analysis_handler::handle_analyze_comprehensive(comprehensive_config(
            empty.path(),
            None
        ))
        .await
        .expect_err("an empty tree must not be scored")
    );
    assert!(
        message.contains("comprehensive quality-score"),
        "the refusal must name the measurement that was not taken, got: {message}"
    );
    assert!(
        !message.contains("100"),
        "no score may appear anywhere in the refusal, got: {message}"
    );
}

// ---------------------------------------------------------------- complexity

async fn run_complexity(
    root: &Path,
    thresholds: (Option<u16>, Option<u16>),
    out: Option<std::path::PathBuf>,
) -> anyhow::Result<()> {
    super::complexity_handlers::handle_analyze_complexity(
        root.to_path_buf(),
        None,
        vec![],
        None,
        crate::cli::ComplexityOutputFormat::Json,
        out,
        thresholds.0,
        thresholds.1,
        vec![],
        false,
        0,
        false,
        60,
    )
    .await
}

#[tokio::test]
async fn complexity_refuses_an_empty_tree_and_still_measures_a_real_one() {
    let empty = empty_tree();
    assert_refused_as_unmeasured(
        run_complexity(empty.path(), (None, None), None).await,
        empty.path(),
        "complexity",
    );

    for (tree, label) in [(non_git_tree(), "non-git"), (git_tree(), "git")] {
        let out = tree.path().join("complexity.json");
        assert_measured(
            run_complexity(tree.path(), (None, None), Some(out.clone())).await,
            "complexity",
            label,
        );
        let report: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&out).expect("complexity report"))
                .expect("complexity json");
        let files = report["files"].as_array().map_or(0, Vec::len);
        assert!(
            files > 0,
            "complexity: a {label} tree must report the files it read, got: {report}"
        );
    }
}

/// Files that WERE read and then dropped by `--max-cyclomatic` are a measured
/// zero and must keep exit 0 — the refusal is for "nothing was read", never for
/// "read them and none crossed the threshold". Without this the fix could be
/// satisfied by refusing every empty report, which is a new lie in the other
/// direction.
#[tokio::test]
async fn complexity_still_succeeds_when_thresholds_filter_every_file_out() {
    let tree = non_git_tree();
    let out = tree.path().join("filtered.json");
    run_complexity(tree.path(), (Some(9999), Some(9999)), Some(out))
        .await
        .expect("a threshold that excludes every file is a measurement, not a refusal");
}

// ------------------------------------------------------------ assembly-script

fn assemblyscript_tree() -> TempDir {
    let dir = TempDir::new().expect("tempdir");
    std::fs::create_dir_all(dir.path().join("assembly")).expect("mkdir assembly");
    std::fs::write(dir.path().join("assembly/index.ts"), ASSEMBLYSCRIPT_FIXTURE)
        .expect("write index.ts");
    dir
}

async fn run_assemblyscript(root: &Path, out: Option<std::path::PathBuf>) -> anyhow::Result<()> {
    super::wasm_handlers::handle_analyze_assemblyscript(
        root.to_path_buf(),
        crate::cli::ComplexityOutputFormat::Json,
        false,
        false,
        false,
        out,
        60,
        false,
        10,
    )
    .await
}

#[tokio::test]
async fn assembly_script_refuses_an_empty_tree_and_still_measures_a_real_one() {
    let empty = empty_tree();
    assert_refused_naming(
        run_assemblyscript(empty.path(), None).await,
        empty.path(),
        "AssemblyScript files",
        "assembly-script",
    );

    // A tree full of Rust holds no AssemblyScript either, and that is the same
    // event: a report of zeros there would read as "your AssemblyScript is
    // clean" about a project that has none.
    let rust_only = non_git_tree();
    assert_refused_naming(
        run_assemblyscript(rust_only.path(), None).await,
        rust_only.path(),
        "AssemblyScript files",
        "assembly-script over a Rust tree",
    );

    let real = assemblyscript_tree();
    let out = real.path().join("as.json");
    assert_measured(
        run_assemblyscript(real.path(), Some(out.clone())).await,
        "assembly-script",
        "AssemblyScript",
    );
    let report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&out).expect("as report")).expect("as json");
    assert!(
        report["files_analyzed"].as_u64().unwrap_or(0) > 0,
        "assembly-script: a real AssemblyScript tree must report its files, got: {report}"
    );
}

/// Candidates opened and none of them AssemblyScript is a different event from
/// "the tree holds none", and gets its own sentence — a `.as` file full of
/// something else must not be reported as a clean zero either.
#[tokio::test]
async fn assembly_script_refuses_when_no_candidate_parsed() {
    let dir = TempDir::new().expect("tempdir");
    // Collected because of the `.as` extension, rejected by the detector: no
    // `i32`/`f64`/`@inline`/`memory.` and no `export … function`.
    std::fs::write(dir.path().join("notes.as"), "plain prose, not code\n").expect("write notes.as");

    let message = format!(
        "{:#}",
        run_assemblyscript(dir.path(), None)
            .await
            .expect_err("a candidate that parsed as nothing is not a clean zero")
    );
    assert!(
        message.contains("1 candidate file(s)"),
        "the refusal must say how many files were opened, got: {message}"
    );
    assert!(
        message.contains("none parsed as AssemblyScript"),
        "the refusal must say why they yielded nothing, got: {message}"
    );
    assert!(
        message.contains("This is not a clean result"),
        "the refusal must say the zero is not clean, got: {message}"
    );
}

// ------------------------------------------------------------- web-assembly

async fn run_webassembly(root: &Path, out: Option<std::path::PathBuf>) -> anyhow::Result<()> {
    super::wasm_handlers::handle_analyze_webassembly(
        root.to_path_buf(),
        crate::cli::ComplexityOutputFormat::Json,
        true,
        true,
        false,
        false,
        false,
        out,
        false,
        10,
    )
    .await
}

#[tokio::test]
async fn web_assembly_refuses_an_empty_tree_and_still_measures_a_real_one() {
    let empty = empty_tree();
    assert_refused_naming(
        run_webassembly(empty.path(), None).await,
        empty.path(),
        "WebAssembly (.wasm/.wat) files",
        "web-assembly",
    );

    let rust_only = non_git_tree();
    assert_refused_naming(
        run_webassembly(rust_only.path(), None).await,
        rust_only.path(),
        "WebAssembly (.wasm/.wat) files",
        "web-assembly over a Rust tree",
    );

    let real = TempDir::new().expect("tempdir");
    std::fs::write(real.path().join("mod.wasm"), WASM_HEADER).expect("write wasm");
    let out = real.path().join("wasm.json");
    assert_measured(
        run_webassembly(real.path(), Some(out.clone())).await,
        "web-assembly",
        "wasm",
    );
    let report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&out).expect("wasm report"))
            .expect("wasm json");
    assert!(
        report["files_analyzed"].as_u64().unwrap_or(0) > 0,
        "web-assembly: a real .wasm module must be reported, got: {report}"
    );
}

/// A `.wat`-only tree yields no `WasmMetrics` row by design (the text front end
/// produces none) and already says so on stderr. Those files WERE opened, so
/// that run is measured and keeps exit 0 — the refusal is keyed on discovery,
/// not on the row count.
#[tokio::test]
async fn web_assembly_still_succeeds_for_a_text_only_module() {
    let dir = TempDir::new().expect("tempdir");
    std::fs::write(dir.path().join("mod.wat"), WAT_FIXTURE).expect("write wat");

    run_webassembly(dir.path(), Some(dir.path().join("wat.json")))
        .await
        .expect("a parsed .wat is a measurement, not a refusal");
}

// ----------------------------------------------------------- name-similarity

async fn run_name_similarity(
    root: &Path,
    query: &str,
    out: Option<std::path::PathBuf>,
) -> anyhow::Result<()> {
    super::name_similarity_analysis::handle_analyze_name_similarity(
        root.to_path_buf(),
        query.to_string(),
        10,
        false,
        crate::cli::SearchScope::All,
        0.7,
        crate::cli::NameSimilarityOutputFormat::Json,
        None,
        None,
        out,
        false,
        false,
        false,
    )
    .await
}

#[tokio::test]
async fn name_similarity_refuses_an_empty_tree_and_still_searches_a_real_one() {
    let empty = empty_tree();
    assert_refused_naming(
        run_name_similarity(empty.path(), "add", None).await,
        empty.path(),
        "names",
        "name-similarity",
    );

    for (tree, label) in [(non_git_tree(), "non-git"), (git_tree(), "git")] {
        let out = tree.path().join("similar.json");
        assert_measured(
            run_name_similarity(tree.path(), "add", Some(out.clone())).await,
            "name-similarity",
            label,
        );
        let report: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&out).expect("similarity report"))
                .expect("similarity json");
        assert!(
            !report["matches"]
                .as_array()
                .expect("matches array")
                .is_empty(),
            "name-similarity: `add` is declared in the fixture, got: {report}"
        );
    }
}

/// A query that matches nothing over a real corpus is a RESULT, and must stay
/// exit 0 — but the report has to carry the denominator, because "0 of 0" and
/// "0 of 8" were the same document. `total_candidates` used to be set to
/// `top_matches.len()`, i.e. the number of matches, so it printed the numerator
/// twice.
#[tokio::test]
async fn name_similarity_reports_the_corpus_it_searched_when_nothing_matches() {
    let tree = non_git_tree();
    let out = tree.path().join("nomatch.json");
    run_name_similarity(tree.path(), "zzzzzzzzzz", Some(out.clone()))
        .await
        .expect("a query that matches nothing over a real corpus is a result");

    let report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&out).expect("similarity report"))
            .expect("similarity json");
    assert!(
        report["matches"]
            .as_array()
            .expect("matches array")
            .is_empty(),
        "the fixture declares no name like `zzzzzzzzzz`, got: {report}"
    );
    assert!(
        report["total_candidates"].as_u64().unwrap_or(0) > 0,
        "0 matches out of nothing and 0 matches out of a real corpus must not be \
         the same document, got: {report}"
    );
}
