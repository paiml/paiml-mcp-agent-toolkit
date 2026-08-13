//! Library entry point for defect prediction analysis.
//!
//! This module is a forwarder. It used to be a SECOND implementation of
//! `analyze defect-prediction`, and it was the worse one.

use crate::cli::DefectPredictionOutputFormat;
use anyhow::Result;
use std::path::PathBuf;

/// Handle defect prediction analysis.
///
/// # One command name, one implementation (#948, same shape as #954)
///
/// `pmat analyze defect-prediction` is routed by
/// `analysis_handlers/advanced_routes.rs` to
/// [`crate::cli::handlers::defect_prediction_handler`], which measures churn
/// from git history over a 90-day window, reports its yardstick in
/// `churn_source`, and states in the summary that duplication is *not
/// measured* because the command runs no clone detection.
///
/// This entry point — public, re-exported as
/// `pmat::cli::analysis::handle_analyze_defect_prediction`, and reachable
/// through `handlers::advanced_analysis_handlers` — used to run a private
/// pipeline instead, and three of the model's inputs were not what their names
/// said:
///
/// * `churn_score` was `(1.0 - comment_ratio) * 0.5 + todo_factor * 0.5` — a
///   measure of how heavily a file is COMMENTED, computed from the file's own
///   text, with no git history involved at all;
/// * `duplicate_ratio` was the literal `0.0` — "not measured" rendered as
///   "measured, and clean";
/// * `efferent_coupling` was the literal `0.0`.
///
/// A library consumer therefore got a different risk ranking from the same
/// project than the CLI did, computed partly from constants, with nothing in
/// any of its five output formats saying so. Two implementations of one
/// command name is how the CLI/MCP contradictions in this project keep
/// recurring, so the divergent one is deleted rather than synced: the summary,
/// detailed, JSON, CSV and SARIF renderers that consumed those numbers are
/// gone with it, and this signature now forwards to the wired handler
/// unchanged.
#[allow(clippy::too_many_arguments)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_analyze_defect_prediction(
    project_path: PathBuf,
    confidence_threshold: f32,
    min_lines: usize,
    include_low_confidence: bool,
    format: DefectPredictionOutputFormat,
    high_risk_only: bool,
    include_recommendations: bool,
    include: Option<String>,
    exclude: Option<String>,
    output: Option<PathBuf>,
    perf: bool,
    top_files: usize,
) -> Result<()> {
    use crate::cli::handlers::defect_prediction_handler::{
        handle_analyze_defect_prediction as wired, DefectPredictionConfig,
    };

    wired(DefectPredictionConfig {
        project_path,
        confidence_threshold,
        min_lines,
        include_low_confidence,
        format,
        high_risk_only,
        include_recommendations,
        // An empty `--include`/`--exclude` is no filter. The caller in
        // `advanced_analysis_handlers` joins its `Vec<String>` with commas and
        // always passes `Some`, so an unset flag arrives here as `Some("")`;
        // forwarding that verbatim would install a pattern that matches
        // nothing and silently empty the report.
        include: non_empty(include),
        exclude: non_empty(exclude),
        output,
        perf,
        top_files,
    })
    .await
}

/// `Some("")` and `Some("  ")` mean "no filter given", not "filter on nothing".
fn non_empty(value: Option<String>) -> Option<String> {
    value.filter(|v| !v.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_filter_string_is_not_a_filter() {
        assert_eq!(non_empty(Some(String::new())), None);
        assert_eq!(non_empty(Some("   ".to_string())), None);
        assert_eq!(non_empty(None), None);
        assert_eq!(
            non_empty(Some("*.rs".to_string())),
            Some("*.rs".to_string())
        );
    }

    /// #948: the library entry point must produce the document the CLI
    /// produces, not a private one computed from constants.
    ///
    /// RED on the old code: the private pipeline emitted a summary headed
    /// "🔮 Defect Prediction Summary" with a risk distribution and no
    /// provenance at all. The wired handler states where churn came from and
    /// that duplication was not measured — the two facts that distinguish a
    /// measured report from a fabricated one.
    #[tokio::test]
    async fn the_library_entry_point_renders_the_wired_report() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        std::fs::create_dir_all(dir.path().join("src")).expect("mkdir");
        std::fs::write(
            dir.path().join("src/lib.rs"),
            (0..40)
                .map(|i| {
                    format!("pub fn f{i}(a: i32) -> i32 {{ if a > {i} {{ a }} else {{ -a }} }}")
                })
                .collect::<Vec<_>>()
                .join("\n"),
        )
        .expect("write");

        let out = dir.path().join("report.json");
        handle_analyze_defect_prediction(
            dir.path().to_path_buf(),
            0.0,
            1,
            true,
            DefectPredictionOutputFormat::Json,
            false,
            false,
            Some(String::new()),
            Some(String::new()),
            Some(out.clone()),
            false,
            0,
        )
        .await
        .expect("library entry point runs");

        let doc: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&out).expect("read"))
                .expect("valid json");
        assert!(
            doc.get("churn_source").is_some(),
            "the wired document names its churn yardstick: {doc}"
        );
        assert!(
            doc.get("duplication_source").is_some(),
            "the wired document says duplication was not measured: {doc}"
        );
    }
}
