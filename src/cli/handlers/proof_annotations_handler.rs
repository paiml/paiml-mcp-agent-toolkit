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

    // #1015: "Total proofs: 0 / High confidence: 0 (0.0%)" was printed, exit 0,
    // for a directory holding no Rust source at all — the same document a tree
    // that was fully scanned and carries no annotations produces. The disclosure
    // machinery below already exists for the *filter* matching nothing; this is
    // the case one step earlier, where there was nothing to filter.
    crate::cli::ensure_source_files_were_analyzed(
        "proof-annotation",
        &project_path,
        annotator.files_processed(),
    )?;

    // #953 (residual): `--verification-method formal-proof` (and
    // `model-checking`, and `abstract-interpretation`) printed an empty,
    // exit-0 report over a tree containing 42 kani harnesses. An empty result
    // is the same document whether the method was checked and found absent or
    // whether NO COLLECTOR IN THIS BUILD can produce it — absence rendered as
    // success. What the method filter removed is now measured and disclosed.
    let method_note = measure_method_filter(&annotator, &project_path, &filter, &annotations).await;

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
    if let Some(ref outcome) = method_note {
        eprint!("⚠️{}", outcome.note());
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
        method_note.as_ref(),
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

/// What `--verification-method` removed, measured rather than assumed.
///
/// Constructed only when the filter matched NOTHING while the same project
/// does yield annotations by some other method — the one case in which an
/// empty report would otherwise be indistinguishable from "checked, none
/// found". The methods listed are read off the annotations this run actually
/// collected, so nothing here can drift out of step with the registered proof
/// sources the way a hand-maintained list of supported methods would.
struct MethodFilterOutcome {
    /// The `--verification-method` value the user asked for.
    requested: String,
    /// Annotations that pass every other filter, whatever their method.
    collected_ignoring_method: usize,
    /// The verification methods those annotations actually carry.
    methods_present: Vec<String>,
}

impl MethodFilterOutcome {
    /// The disclosure, in the same shape as [`incomplete_analysis_note`].
    fn note(&self) -> String {
        format!(
            "  UNMEASURED: --verification-method {} matched 0 of the {} annotation(s) this \
             project yields. The methods actually present are: {}. pmat collects proof \
             annotations from its Rust static analyser only, so an empty result under \
             --verification-method {} means no collector in this build produces that method — \
             NOT that the property was verified and found absent.\n",
            self.requested,
            self.collected_ignoring_method,
            self.methods_present.join(", "),
            self.requested,
        )
    }
}

/// Measure what `--verification-method` removed.
///
/// Returns `None` — no disclosure needed — when no method filter was applied,
/// when the filter matched something, or when the project yields no
/// annotations at all (an empty report is then honest on its own terms). The
/// second collection runs only in the one case that needs it, and
/// `collect_proofs` *stores* rather than accumulates its error count, so
/// re-running it cannot inflate `files_not_analyzed`.
async fn measure_method_filter(
    annotator: &ProofAnnotator,
    project_path: &Path,
    filter: &ProofAnnotationFilter,
    matched: &[(Location, ProofAnnotation)],
) -> Option<MethodFilterOutcome> {
    let requested = match filter.verification_method.as_ref() {
        None | Some(VerificationMethodFilter::All) => return None,
        Some(method) => method.to_string(),
    };
    if !matched.is_empty() {
        return None;
    }

    let without_method = ProofAnnotationFilter {
        high_confidence_only: filter.high_confidence_only,
        property_type: filter.property_type.clone(),
        verification_method: None,
    };
    let all = collect_and_filter_annotations(annotator, project_path, &without_method).await;
    if all.is_empty() {
        return None;
    }

    // Sorted and deduplicated by the BTreeSet, so the disclosure is identical
    // run to run on unchanged input.
    let methods_present: Vec<String> = all
        .iter()
        .map(|(_, annotation)| format!("{:?}", annotation.method))
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();

    Some(MethodFilterOutcome {
        requested,
        collected_ignoring_method: all.len(),
        methods_present,
    })
}

/// Add the method-filter disclosure to a rendered JSON document.
///
/// The fact belongs in the machine-readable document as a field, not only as
/// prose on stderr, for the same reason `summary.files_not_analyzed` does: a
/// consumer that reads `total_annotations: 0` has no other way to tell a
/// measured zero from an uncollectable one.
fn attach_method_filter_to_json(content: &str, outcome: &MethodFilterOutcome) -> Result<String> {
    let mut doc: serde_json::Value = serde_json::from_str(content)?;
    if let Some(summary) = doc.get_mut("summary").and_then(|s| s.as_object_mut()) {
        summary.insert(
            "verification_method_filter".to_string(),
            // No `matched: 0` leaf: it would be constant by construction (the
            // field exists only when the filter matched nothing), and a
            // numeric leaf that reads the same for an empty directory and for
            // a 4,000-file tree is what the differential falsification gate
            // exists to catch. `summary.total_annotations` already carries the
            // zero, measured.
            serde_json::json!({
                "requested": outcome.requested,
                "annotations_ignoring_method_filter": outcome.collected_ignoring_method,
                "methods_present": outcome.methods_present,
                "reason": "no registered proof source produces this verification method",
            }),
        );
    }
    Ok(serde_json::to_string_pretty(&doc)?)
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
    method_note: Option<&MethodFilterOutcome>,
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

    // Same split for the method-filter disclosure: prose for the readers that
    // have room for it, a field for the JSON document. SARIF has neither.
    if let Some(outcome) = method_note {
        match format {
            ProofAnnotationOutputFormat::Json => {
                content = attach_method_filter_to_json(&content, outcome)?;
            }
            ProofAnnotationOutputFormat::Sarif => {}
            _ => content.push_str(&outcome.note()),
        }
    }

    Ok(content)
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod active_tests {
    use super::*;
    use tempfile::TempDir;

    /// An empty directory is refused, not reported as "Total proofs: 0".
    ///
    /// This asserted `result.is_ok()` — it pinned the defect (#1015). "Total
    /// proofs: 0 / High confidence: 0 (0.0%)" is what a fully scanned tree
    /// carrying no annotations prints too, so the two were the same document
    /// and the same exit code.
    #[tokio::test]
    async fn test_handle_analyze_proof_annotations_refuses_empty_dir() {
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
        let message = result
            .expect_err("nothing was scanned, so there is no proof measurement to report")
            .to_string();
        assert!(
            message.contains("no source files were found")
                && message.contains("This is not a clean result"),
            "the refusal must name what was missing and say it is not clean, got: {message}"
        );
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

    /// #953 (residual): `--verification-method formal-proof` produced an
    /// exit-0, zero-annotation report indistinguishable from "checked and none
    /// found" — over a tree that yields annotations by another method and a
    /// repo that carries 42 kani harnesses. pmat registers exactly one proof
    /// source (`RustBorrowChecker`), so no formal-proof / model-checking /
    /// abstract-interpretation collector exists in this build.
    ///
    /// RED on the old code: `format_proof_annotations` took no method outcome
    /// and the summary/JSON carried nothing about the filter, so both
    /// assertions below failed.
    #[tokio::test]
    async fn an_uncollectable_verification_method_is_disclosed_not_rendered_as_zero() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        std::fs::write(
            temp_dir.path().join("lib.rs"),
            "pub fn safe_add(a: i32, b: i32) -> i32 { a + b }\n",
        )
        .expect("write");

        let summary_out = temp_dir.path().join("summary.txt");
        handle_analyze_proof_annotations(
            temp_dir.path().to_path_buf(),
            ProofAnnotationOutputFormat::Summary,
            false,
            false,
            None,
            Some(VerificationMethodFilter::FormalProof),
            Some(summary_out.clone()),
            false,
            true,
            10,
        )
        .await
        .expect("summary run");

        let summary = std::fs::read_to_string(&summary_out).expect("read summary");
        assert!(
            summary.contains("UNMEASURED") && summary.contains("formal-proof"),
            "an empty formal-proof report must say no collector produces that \
             method, got:\n{summary}"
        );

        let json_out = temp_dir.path().join("out.json");
        handle_analyze_proof_annotations(
            temp_dir.path().to_path_buf(),
            ProofAnnotationOutputFormat::Json,
            false,
            false,
            None,
            Some(VerificationMethodFilter::FormalProof),
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
        let filter = &doc["summary"]["verification_method_filter"];
        assert_eq!(filter["requested"].as_str(), Some("formal-proof"), "{doc}");
        assert!(
            filter["annotations_ignoring_method_filter"]
                .as_u64()
                .unwrap_or(0)
                > 0,
            "the disclosure must carry the count the filter removed: {doc}"
        );
        assert!(
            filter["methods_present"]
                .as_array()
                .is_some_and(|m| !m.is_empty()),
            "the methods actually collected must be named: {doc}"
        );
    }

    /// The disclosure must not fire when the filter genuinely matched, nor on
    /// a project that yields nothing at all — an over-broad note would make
    /// every honest empty report look like a tool failure.
    ///
    /// The "yields nothing" half used to be an EMPTY DIRECTORY, which is a
    /// different event: nothing was scanned, so there was no report to judge.
    /// It is a scanned file that declares no annotatable item now (#1015), so
    /// the test still exercises a real measured zero — the only kind of empty
    /// report this command is allowed to print.
    #[tokio::test]
    async fn a_matching_method_and_an_unannotated_project_carry_no_disclosure() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        std::fs::write(
            temp_dir.path().join("lib.rs"),
            "pub fn safe_add(a: i32, b: i32) -> i32 { a + b }\n",
        )
        .expect("write");

        let matched = temp_dir.path().join("matched.txt");
        handle_analyze_proof_annotations(
            temp_dir.path().to_path_buf(),
            ProofAnnotationOutputFormat::Summary,
            false,
            false,
            None,
            Some(VerificationMethodFilter::BorrowChecker),
            Some(matched.clone()),
            false,
            true,
            10,
        )
        .await
        .expect("matched run");
        assert!(
            !std::fs::read_to_string(&matched)
                .expect("read")
                .contains("UNMEASURED"),
            "borrow-checker matches, so there is nothing to disclose"
        );

        let empty_dir = TempDir::new().expect("Failed to create temp dir");
        // Scanned, but declares nothing the borrow checker annotates: a
        // measured zero, as opposed to the unmeasured zero an empty directory
        // would give.
        std::fs::write(
            empty_dir.path().join("consts.rs"),
            "pub const X: i32 = 1;\npub struct S;\n",
        )
        .expect("write");
        let empty_out = empty_dir.path().join("empty.txt");
        handle_analyze_proof_annotations(
            empty_dir.path().to_path_buf(),
            ProofAnnotationOutputFormat::Summary,
            false,
            false,
            None,
            Some(VerificationMethodFilter::FormalProof),
            Some(empty_out.clone()),
            false,
            true,
            10,
        )
        .await
        .expect("empty run");
        assert!(
            !std::fs::read_to_string(&empty_out)
                .expect("read")
                .contains("UNMEASURED"),
            "a project with no annotations at all yields an honest empty report"
        );
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
