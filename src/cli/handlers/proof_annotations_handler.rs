//! Proof Annotations Analysis Handler
//!
//! Refactored handler for formal proof annotation analysis.

use crate::cli::proof_annotation_helpers::{
    collect_and_filter_annotations, format_as_full, format_as_json, format_as_markdown,
    format_as_sarif, format_as_summary, incomplete_analysis_note, setup_proof_annotator,
    ProofAnnotationFilter,
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
    top_files: usize,
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

    crate::status_eprintln!("🔍 Collecting proof annotations from project...");
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
    crate::status_eprintln!(
        "✅ Found {} matching proof annotations in {} ms (verified at {})",
        annotations.len(),
        elapsed.as_millis(),
        chrono::Utc::now().to_rfc3339()
    );
    if let Some(note) = incomplete_analysis_note(annotator.collection_errors()) {
        eprint!("⚠️{note}");
    }

    // Format output using helpers
    let content = format_proof_annotations(
        format,
        &annotations,
        elapsed,
        &annotator,
        &project_path,
        include_evidence,
        top_files,
    )?;

    // Write output
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        crate::status_eprintln!("✅ Proof annotations written to: {}", output_path.display());
    } else {
        println!("{content}");
    }

    Ok(())
}

/// Format proof annotations based on output format (complexity: 6)
#[allow(clippy::too_many_arguments)]
fn format_proof_annotations(
    format: ProofAnnotationOutputFormat,
    annotations: &[(Location, ProofAnnotation)],
    elapsed: std::time::Duration,
    annotator: &ProofAnnotator,
    project_path: &Path,
    include_evidence: bool,
    top_files: usize,
) -> Result<String> {
    let mut content = match format {
        ProofAnnotationOutputFormat::Json => format_as_json(annotations, elapsed, annotator)?,
        ProofAnnotationOutputFormat::Summary => format_as_summary(annotations, elapsed, top_files)?,
        ProofAnnotationOutputFormat::Full => {
            format_as_full(annotations, project_path, include_evidence)?
        }
        ProofAnnotationOutputFormat::Markdown => {
            format_as_markdown(annotations, project_path, include_evidence)?
        }
        ProofAnnotationOutputFormat::Sarif => format_as_sarif(annotations, project_path)?,
    };

    // Files the collector could not parse contributed no annotations, and that
    // fact stopped at one `warn!` line on stderr — the report said "Total
    // proofs: 38050" as though it had seen the whole tree, while 31 files had
    // been skipped. The human-readable renderers get the disclosure appended
    // here; the JSON document already carries `summary.files_not_analyzed`, and
    // SARIF has no free-text slot for it.
    if !matches!(
        format,
        ProofAnnotationOutputFormat::Json | ProofAnnotationOutputFormat::Sarif
    ) {
        if let Some(note) = incomplete_analysis_note(annotator.collection_errors()) {
            content.push_str(&note);
        }
    }

    Ok(content)
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
            10,
        )
        .await;
        assert!(result.is_ok());
    }

    /// A tree with one file the collector cannot parse. On the pmat repo 31
    /// such files were skipped, their failure logged once at `warn!` and
    /// dropped: the report still said "Total proofs: N" with nothing to say it
    /// had been computed over a subset. Both renderings must disclose the gap.
    #[tokio::test]
    async fn test_unparseable_files_are_disclosed_in_the_report() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        std::fs::write(
            temp_dir.path().join("good.rs"),
            "pub fn good(x: &str) -> usize { x.len() }\n",
        )
        .expect("write");
        std::fs::write(temp_dir.path().join("broken.rs"), "fn ((( <<< not rust\n").expect("write");

        // Summary (and every other human-readable format) gets a note.
        let summary_out = temp_dir.path().join("summary.txt");
        handle_analyze_proof_annotations(
            temp_dir.path().to_path_buf(),
            ProofAnnotationOutputFormat::Summary,
            false,
            false,
            None,
            None,
            Some(summary_out.clone()),
            false,
            true,
            10,
        )
        .await
        .expect("summary run");

        let summary = std::fs::read_to_string(&summary_out).expect("read summary");
        assert!(
            summary.contains("INCOMPLETE"),
            "the summary must disclose the file it could not parse, got:\n{summary}"
        );

        // The JSON document carries the same fact as a field, not prose.
        let json_out = temp_dir.path().join("out.json");
        handle_analyze_proof_annotations(
            temp_dir.path().to_path_buf(),
            ProofAnnotationOutputFormat::Json,
            false,
            false,
            None,
            None,
            Some(json_out.clone()),
            false,
            true,
            10,
        )
        .await
        .expect("json run");

        let doc: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&json_out).expect("read json"))
                .expect("valid json");
        assert_eq!(
            doc["summary"]["files_not_analyzed"].as_u64(),
            Some(1),
            "the JSON summary must report the skipped file, got: {doc}"
        );
    }

    /// A tree the collector reads completely must read exactly as before — the
    /// disclosure is only for a partial analysis.
    #[tokio::test]
    async fn test_complete_analysis_carries_no_incomplete_note() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        std::fs::write(
            temp_dir.path().join("good.rs"),
            "pub fn good(x: &str) -> usize { x.len() }\n",
        )
        .expect("write");

        let out = temp_dir.path().join("summary.txt");
        handle_analyze_proof_annotations(
            temp_dir.path().to_path_buf(),
            ProofAnnotationOutputFormat::Summary,
            false,
            false,
            None,
            None,
            Some(out.clone()),
            false,
            true,
            10,
        )
        .await
        .expect("summary run");

        let summary = std::fs::read_to_string(&out).expect("read summary");
        assert!(!summary.contains("INCOMPLETE"), "got:\n{summary}");
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
            10,
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
            10,
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
            10,
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
            10,
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
            10,
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
            10,
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
            10,
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
                10,
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
