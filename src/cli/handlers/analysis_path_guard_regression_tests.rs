//! Regression tests: an analysis handler must never report a result for a path
//! it could not read.
//!
//! Every test here pins a defect observed in the 3.29.0 pre-release dogfood
//! sweep, where a handler walked a nonexistent tree to zero files and printed a
//! *successful* empty (or, worse, perfect) report with exit 0:
//!
//! * GH-662 `cuda-tdg` — "Score: 55.5/100 (Grade: D) / Gateway: PASSED"
//! * GH-663 `analyze comprehensive` — "Quality Score: 100.0% / Code quality looks good!"
//! * GH-664 `analyze defects --file <missing>` — "Total Files Scanned: 1"
//! * GH-666 `analyze dag|provability|defect-prediction|defects`
//! * GH-681 `analyze entropy` — "Files Analyzed: 0 / Total Violations: 1"
//! * GH-682 `analyze complexity` on a `chmod 000` directory — exit 0
//!
//! Found in the same sweep and fixed with the same guard: `analyze deep-context`,
//! `analyze proof-annotations`, `analyze assembly-script`, `analyze web-assembly`.

use std::path::{Path, PathBuf};

/// A path that cannot exist, unique per test so a stray directory cannot mask a
/// regression.
fn missing_path(tag: &str) -> PathBuf {
    PathBuf::from(format!("/nonexistent-pmat-{tag}-9f3a/does/not/exist"))
}

/// Assert the error is the path guard talking, and that it names the path — a
/// typo in a CI script has to be obvious from the message alone.
fn assert_names_missing_path(err: &anyhow::Error, path: &Path) {
    let msg = format!("{err:#}");
    assert!(
        msg.contains("Path not found"),
        "expected the missing-path guard, got: {msg}"
    );
    assert!(
        msg.contains(&path.display().to_string()),
        "error must name the offending path, got: {msg}"
    );
}

#[tokio::test]
async fn gh666_dag_rejects_missing_path() {
    let path = missing_path("dag");
    let err = super::complexity_handlers::handle_analyze_dag(
        crate::cli::DagType::FullDependency,
        path.clone(),
        None,
        None,
        None,
        false,
        false,
        false,
        false,
        false,
    )
    .await
    .expect_err("a missing path must not produce a graph");
    assert_names_missing_path(&err, &path);
}

#[tokio::test]
async fn gh666_defects_rejects_missing_path() {
    use super::analyze_defects_handler::{handle_analyze_defects, OutputFormat};

    let path = missing_path("defects-path");
    let err = handle_analyze_defects(Some(&path), None, None, OutputFormat::Json)
        .await
        .expect_err("a missing path must not produce a defect report");
    assert_names_missing_path(&err, &path);
}

/// GH-664: the missing file was pushed into the scan list unread, so the report
/// claimed `total_files_scanned: 1` for a file that does not exist.
#[tokio::test]
async fn gh664_defects_rejects_missing_file() {
    use super::analyze_defects_handler::{handle_analyze_defects, OutputFormat};

    let file = missing_path("defects-file").join("nope.rs");
    let err = handle_analyze_defects(None, Some(&file), None, OutputFormat::Json)
        .await
        .expect_err("a missing --file must not be counted as scanned");
    assert_names_missing_path(&err, &file);
}

#[tokio::test]
async fn gh666_provability_rejects_missing_path() {
    use super::provability_handler::ProvabilityConfig;

    let path = missing_path("provability");
    let err = super::provability_handler::handle_analyze_provability(ProvabilityConfig {
        project_path: path.clone(),
        functions: vec![],
        analysis_depth: 1,
        format: crate::cli::ProvabilityOutputFormat::Json,
        high_confidence_only: false,
        include_evidence: false,
        output: None,
        top_files: 0,
    })
    .await
    .expect_err("a missing path must not produce a provability report");
    assert_names_missing_path(&err, &path);
}

#[tokio::test]
async fn gh666_defect_prediction_rejects_missing_path() {
    use super::defect_prediction_handler::{
        handle_analyze_defect_prediction, DefectPredictionConfig,
    };

    let path = missing_path("defect-prediction");
    let err = handle_analyze_defect_prediction(DefectPredictionConfig {
        project_path: path.clone(),
        confidence_threshold: 0.5,
        min_lines: 10,
        include_low_confidence: false,
        format: crate::cli::DefectPredictionOutputFormat::Json,
        high_risk_only: false,
        include_recommendations: false,
        include: None,
        exclude: None,
        output: None,
        perf: false,
        top_files: 0,
    })
    .await
    .expect_err("a missing path must not produce a defect prediction");
    assert_names_missing_path(&err, &path);
}

/// GH-663: this reported "Quality Score: 100.0%" and "Code quality looks good!"
/// for a path that does not exist — a green CI run from a typo.
#[tokio::test]
async fn gh663_comprehensive_rejects_missing_path() {
    use super::comprehensive_analysis_handler::{
        handle_analyze_comprehensive, ComprehensiveAnalysisConfig,
    };

    let path = missing_path("comprehensive");
    let err = handle_analyze_comprehensive(ComprehensiveAnalysisConfig {
        project_path: path.clone(),
        file: None,
        files: vec![],
        format: crate::cli::ComprehensiveOutputFormat::Json,
        include_duplicates: false,
        include_dead_code: false,
        include_defects: false,
        include_complexity: true,
        include_tdg: false,
        confidence_threshold: 0.5,
        min_lines: 10,
        include: None,
        exclude: None,
        output: None,
        perf: false,
        executive_summary: false,
        top_files: 10,
    })
    .await
    .expect_err("a missing path must not score 100%");
    assert_names_missing_path(&err, &path);
}

#[tokio::test]
async fn deep_context_rejects_missing_path() {
    let path = missing_path("deep-context");
    let err = super::advanced_analysis_handlers::handle_analyze_deep_context(
        path.clone(),
        None,
        crate::cli::DeepContextOutputFormat::Json,
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
        10,
    )
    .await
    .expect_err("a missing path must not produce a deep context report");
    assert_names_missing_path(&err, &path);
}

#[tokio::test]
async fn proof_annotations_rejects_missing_path() {
    let path = missing_path("proof-annotations");
    let err = super::proof_annotations_handler::handle_analyze_proof_annotations(
        path.clone(),
        crate::cli::ProofAnnotationOutputFormat::Json,
        false,
        false,
        None,
        None,
        None,
        false,
        false,
        10,
    )
    .await
    .expect_err("a missing path must not produce proof annotations");
    assert_names_missing_path(&err, &path);
}

#[tokio::test]
async fn assemblyscript_rejects_missing_path() {
    let path = missing_path("assemblyscript");
    let err = super::wasm_handlers::handle_analyze_assemblyscript(
        path.clone(),
        crate::cli::ComplexityOutputFormat::Json,
        false,
        false,
        false,
        None,
        30,
        false,
        10,
    )
    .await
    .expect_err("a missing path must not produce an AssemblyScript report");
    assert_names_missing_path(&err, &path);
}

#[tokio::test]
async fn webassembly_rejects_missing_path() {
    let path = missing_path("webassembly");
    let err = super::wasm_handlers::handle_analyze_webassembly(
        path.clone(),
        crate::cli::ComplexityOutputFormat::Json,
        true,
        true,
        false,
        false,
        false,
        None,
        false,
        10,
    )
    .await
    .expect_err("a missing path must not produce a WebAssembly report");
    assert_names_missing_path(&err, &path);
}

/// GH-681, routed through the real `analyze entropy` dispatch so the `--file`
/// and `-p` argument handling is covered too.
#[tokio::test]
async fn gh681_entropy_rejects_missing_path() {
    use crate::cli::handlers::analysis_handlers::route_analyze_command;
    use crate::cli::AnalyzeCommands;

    let path = missing_path("entropy");
    let err = route_analyze_command(AnalyzeCommands::Entropy {
        path: path.clone(),
        project_path: None,
        format: crate::cli::EntropyOutputFormat::Json,
        output: None,
        min_severity: crate::cli::EntropySeverity::Medium,
        top_violations: 20,
        file: None,
        include_tests: false,
    })
    .await
    .expect_err("a missing path must not produce entropy violations");
    assert_names_missing_path(&err, &path);
}

#[tokio::test]
async fn gh681_entropy_rejects_missing_file() {
    use crate::cli::handlers::analysis_handlers::route_analyze_command;
    use crate::cli::AnalyzeCommands;

    let file = missing_path("entropy-file").join("nope.rs");
    let err = route_analyze_command(AnalyzeCommands::Entropy {
        path: PathBuf::from("."),
        project_path: None,
        format: crate::cli::EntropyOutputFormat::Json,
        output: None,
        min_severity: crate::cli::EntropySeverity::Medium,
        top_violations: 20,
        file: Some(file.clone()),
        include_tests: false,
    })
    .await
    .expect_err("a missing --file must not produce entropy violations");
    assert_names_missing_path(&err, &file);
}

/// GH-662: every nonexistent path scored an identical 55.5/100, Grade D,
/// "Gateway: PASSED".
#[test]
fn gh662_cuda_tdg_rejects_missing_path() {
    let path = missing_path("cuda-tdg");
    let err = crate::tdg::CudaSimdAnalyzer::new()
        .analyze(&path)
        .expect_err("a missing path must not be scored");
    assert_names_missing_path(&err, &path);
}

/// GH-662, second half: an empty real directory measured nothing, yet the same
/// 55.5/100 "Gateway: PASSED" was printed. Nothing read ⇒ nothing scored.
#[tokio::test]
async fn gh662_cuda_tdg_reports_unmeasured_for_empty_directory() {
    use crate::cli::commands::CudaTdgOutputFormat;
    use crate::cli::handlers::cuda_tdg_handlers::{handle_cuda_tdg_command, CudaTdgCommandConfig};

    let dir = tempfile::TempDir::new().expect("tempdir");
    let report = dir.path().join("report.txt");

    handle_cuda_tdg_command(CudaTdgCommandConfig {
        path: dir.path().to_path_buf(),
        command: None,
        format: CudaTdgOutputFormat::Terminal,
        min_score: 85.0,
        fail_on_p0: false,
        simd: true,
        wgpu: true,
        output: Some(report.clone()),
        quiet: false,
    })
    .await
    .expect("an empty directory is not an error, it is simply unmeasured");

    let written = std::fs::read_to_string(&report).expect("report written");
    assert!(
        written.contains("not measured"),
        "an unread tree must be reported as unmeasured, got: {written}"
    );
    assert!(
        !written.contains("Gateway: PASSED"),
        "a gateway cannot pass on zero measured files, got: {written}"
    );
    assert!(
        !written.contains("/100"),
        "no score may be synthesised from zero files, got: {written}"
    );
}

/// GH-682: the directory exists, so the old `path.exists()` check passed; the
/// walk was then denied and returned zero files, and complexity exited 0 with
/// "⚠️  Warning: No files were found or analyzed" over content it never read.
#[cfg(unix)]
#[tokio::test]
async fn gh682_complexity_rejects_unreadable_directory() {
    use std::os::unix::fs::PermissionsExt;

    let parent = tempfile::TempDir::new().expect("tempdir");
    let locked = parent.path().join("noread");
    std::fs::create_dir(&locked).expect("create dir");
    std::fs::write(locked.join("main.rs"), "fn main() { let _ = 1; }").expect("write");
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000)).expect("chmod 000");

    // Root ignores permission bits; detect that instead of asserting a falsehood.
    let permissions_bite = std::fs::read_dir(&locked).is_err();

    let result = super::complexity_handlers::handle_analyze_complexity(
        locked.clone(),
        None,
        vec![],
        Some("rust".to_string()),
        crate::cli::ComplexityOutputFormat::Json,
        None,
        None,
        None,
        vec![],
        false,
        0,
        false,
        30,
    )
    .await;

    let _ = std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755));

    if !permissions_bite {
        return;
    }

    let err = result.expect_err("an unreadable directory must not be reported as clean");
    let msg = format!("{err:#}");
    assert!(
        msg.contains("Path not readable"),
        "expected the unreadable-path guard, got: {msg}"
    );
    assert!(
        msg.contains("noread"),
        "error must name the offending path, got: {msg}"
    );
}
