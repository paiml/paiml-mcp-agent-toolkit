//! Proof Annotations Analysis Handler
//!
//! Refactored handler for formal proof annotation analysis.

use crate::cli::proof_annotation_helpers::{
    collect_and_filter_annotations, format_as_full, format_as_json, format_as_markdown,
    format_as_sarif, format_as_summary, setup_proof_annotator, ProofAnnotationFilter,
};
use crate::cli::{ProofAnnotationOutputFormat, PropertyTypeFilter, VerificationMethodFilter};
use crate::models::unified_ast::{Location, ProofAnnotation};
use crate::services::proof_annotator::ProofAnnotator;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Refactored handler for proof annotations analysis.
#[allow(clippy::too_many_arguments)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_analyze_proof_annotations(
    project_path: PathBuf,
    format: ProofAnnotationOutputFormat,
    high_confidence_only: bool,
    include_evidence: bool,
    property_type: Option<PropertyTypeFilter>,
    verification_method: Option<VerificationMethodFilter>,
    output: Option<PathBuf>,
    _perf: bool,
    clear_cache: bool,
) -> Result<()> {
    // A path that does not exist must fail, not produce a proof report. This
    // command used to exit 0 and emit ten annotations for `/no/such/dir`,
    // because the mock source never looked at the path. Matches the guard
    // `analyze complexity`/`satd`/`duplicates` already apply.
    // contracts/pmat-no-fabrication-v1.yaml, equation `missing_path_fails`.
    // Found alongside GH-663/GH-666: a nonexistent path exited 0 with a ranked
    // list of annotated files ("borrow_checker_0.rs - 1 annotations", ...) that
    // do not exist anywhere on disk.
    crate::cli::ensure_analysis_path_exists(&project_path)?;

    eprintln!("🔍 Collecting proof annotations from project...");
    let start = Instant::now();

    // Setup annotator
    let annotator = setup_proof_annotator(clear_cache);

    // Create filter
    let filter = ProofAnnotationFilter {
        high_confidence_only,
        property_type,
        verification_method,
    };

    // Collect and filter annotations
    let annotations = collect_and_filter_annotations(&annotator, &project_path, &filter).await;
    let elapsed = start.elapsed();

    // The run's wall clock lives here, on stderr, and not inside the JSON
    // document: `dateVerified` used to be stamped onto every one of 1298
    // annotations and `analysis_time_ms` into the summary, which made two
    // byte-identical analyses of the same tree diff on every invocation.
    eprintln!(
        "✅ Found {} matching proof annotations in {} ms (verified at {})",
        annotations.len(),
        elapsed.as_millis(),
        chrono::Utc::now().to_rfc3339()
    );

    // Format output using helpers
    let content = format_proof_annotations(
        format,
        &annotations,
        elapsed,
        &annotator,
        &project_path,
        include_evidence,
    )?;

    // Write output
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        eprintln!("✅ Proof annotations written to: {}", output_path.display());
    } else {
        println!("{content}");
    }

    Ok(())
}

/// Format proof annotations based on output format (complexity: 6)
fn format_proof_annotations(
    format: ProofAnnotationOutputFormat,
    annotations: &[(Location, ProofAnnotation)],
    elapsed: std::time::Duration,
    annotator: &ProofAnnotator,
    project_path: &Path,
    include_evidence: bool,
) -> Result<String> {
    match format {
        ProofAnnotationOutputFormat::Json => format_as_json(annotations, elapsed, annotator),
        ProofAnnotationOutputFormat::Summary => format_as_summary(annotations, elapsed),
        ProofAnnotationOutputFormat::Full => {
            format_as_full(annotations, project_path, include_evidence)
        }
        ProofAnnotationOutputFormat::Markdown => {
            format_as_markdown(annotations, project_path, include_evidence)
        }
        ProofAnnotationOutputFormat::Sarif => format_as_sarif(annotations, project_path),
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod active_tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_handle_analyze_proof_annotations_empty_dir() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let result = handle_analyze_proof_annotations(
            temp_dir.path().to_path_buf(),
            ProofAnnotationOutputFormat::Summary,
            false,
            false,
            None,
            None,
            None,
            false,
            false,
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_proof_annotations_json_format() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        std::fs::write(temp_dir.path().join("lib.rs"), "fn test() {}").expect("write");
        let result = handle_analyze_proof_annotations(
            temp_dir.path().to_path_buf(),
            ProofAnnotationOutputFormat::Json,
            false,
            true,
            None,
            None,
            None,
            false,
            false,
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_proof_annotations_with_filters() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        std::fs::write(temp_dir.path().join("lib.rs"), "fn test() {}").expect("write");
        let result = handle_analyze_proof_annotations(
            temp_dir.path().to_path_buf(),
            ProofAnnotationOutputFormat::Summary,
            true, // high_confidence_only
            false,
            Some(PropertyTypeFilter::MemorySafety),
            Some(VerificationMethodFilter::BorrowChecker),
            None,
            false,
            true, // clear_cache
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_proof_annotations_with_output_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let output_path = temp_dir.path().join("output.json");
        std::fs::write(temp_dir.path().join("lib.rs"), "fn test() {}").expect("write");
        let result = handle_analyze_proof_annotations(
            temp_dir.path().to_path_buf(),
            ProofAnnotationOutputFormat::Json,
            false,
            false,
            None,
            None,
            Some(output_path.clone()),
            false,
            false,
        )
        .await;
        assert!(result.is_ok());
        assert!(output_path.exists());
    }

    #[tokio::test]
    async fn test_format_proof_annotations_summary() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        std::fs::write(temp_dir.path().join("lib.rs"), "pub fn exported() {}").expect("write");
        let result = handle_analyze_proof_annotations(
            temp_dir.path().to_path_buf(),
            ProofAnnotationOutputFormat::Summary,
            false,
            false,
            None,
            None,
            None,
            false,
            false,
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_format_proof_annotations_full() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        std::fs::write(temp_dir.path().join("main.rs"), "fn main() {}").expect("write");
        let result = handle_analyze_proof_annotations(
            temp_dir.path().to_path_buf(),
            ProofAnnotationOutputFormat::Full,
            false,
            true,
            None,
            None,
            None,
            false,
            false,
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_format_proof_annotations_markdown() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        std::fs::write(temp_dir.path().join("lib.rs"), "fn test() {}").expect("write");
        let result = handle_analyze_proof_annotations(
            temp_dir.path().to_path_buf(),
            ProofAnnotationOutputFormat::Markdown,
            false,
            false,
            None,
            None,
            None,
            false,
            false,
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_format_proof_annotations_sarif() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        std::fs::write(temp_dir.path().join("lib.rs"), "unsafe fn danger() {}").expect("write");
        let result = handle_analyze_proof_annotations(
            temp_dir.path().to_path_buf(),
            ProofAnnotationOutputFormat::Sarif,
            false,
            true,
            None,
            None,
            None,
            false,
            false,
        )
        .await;
        assert!(result.is_ok());
    }

    /// DETERMINISM (round-3 sweep): `analyze proof-annotations --format json`
    /// produced five different md5 sums over five runs on an unchanged tree.
    /// Three separate causes, all fixed:
    ///
    /// * entry order came out of a `HashMap<Location, _>` (run 1 started at
    ///   `name_similarity_help…`, run 2 at `satd_formatting.rs`, run 3 at
    ///   `tdg_handler_analysis…`);
    /// * `annotationId` was a fresh `Uuid::new_v4()` per annotation per run, so
    ///   the field could not identify anything;
    /// * `dateVerified` / `analysis_time_ms` were wall clocks.
    ///
    /// This drives the real handler over a real fixture, five times, and
    /// requires the rendered document to be byte-identical.
    #[tokio::test]
    async fn json_output_is_byte_identical_across_five_runs() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        // Several files and several items per file, so both the map ordering
        // and the per-annotation id have something to disagree about.
        for name in ["alpha.rs", "beta.rs", "gamma.rs", "delta.rs"] {
            std::fs::write(
                temp_dir.path().join(name),
                "pub fn one(a: u32) -> u32 { a }\n                 pub const fn two(b: u32) -> u32 { b }\n                 pub fn three(c: String) -> String { c }\n",
            )
            .expect("write");
        }

        let render = || async {
            let out = temp_dir.path().join("out.json");
            handle_analyze_proof_annotations(
                temp_dir.path().to_path_buf(),
                ProofAnnotationOutputFormat::Json,
                false,
                true,
                None,
                None,
                Some(out.clone()),
                false,
                true,
            )
            .await
            .expect("json render succeeds");
            let content = std::fs::read_to_string(&out).expect("output written");
            std::fs::remove_file(&out).ok();
            content
        };

        let first = render().await;
        assert!(
            first.contains("annotationId"),
            "fixture must actually produce annotations: {first}"
        );
        assert!(
            !first.contains("dateVerified"),
            "a per-run wall clock makes the document undiffable: {first}"
        );
        for i in 1..5 {
            assert_eq!(
                render().await,
                first,
                "run {i}: identical input must produce byte-identical JSON"
            );
        }
    }
}

/// NOTE: Temporarily disabled due to struct definition mismatches
#[cfg(all(test, feature = "broken-tests"))]
mod coverage_tests {
    include!("proof_annotations_coverage_tests.rs");
    include!("proof_annotations_coverage_tests_part2.rs");
    include!("proof_annotations_coverage_tests_part3.rs");
}
