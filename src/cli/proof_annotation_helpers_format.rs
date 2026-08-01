// Formatting functions: JSON output, table output, summary output, annotator setup, and annotation collection.

/// Format proof annotations as JSON
///
/// DETERMINISM (round-3 sweep): this document is a pure function of the
/// analysed tree. Two wall-clock fields used to make that false, so five runs
/// over an unchanged tree produced five different md5 sums even after the entry
/// order and the `annotationId`s were made stable:
///
/// * per-annotation `dateVerified` — 1298 copies of one `Utc::now()`. That is
///   one measurement of the RUN, not 1298 measurements of the code, and it is
///   now reported once on stderr by the handler instead of 1298 times in the
///   document.
/// * `summary.analysis_time_ms` — how long this machine took, under whatever
///   load it happened to be under. Also reported on stderr.
///
/// Both are still shown to the operator; neither is a property of the input, so
/// neither belongs in the artifact a baseline is diffed against.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_as_json(
    annotations: &[(Location, ProofAnnotation)],
    _elapsed: std::time::Duration,
    annotator: &ProofAnnotator,
) -> Result<String> {
    let cache_stats = annotator.cache_stats();
    let annotations_json: Vec<serde_json::Value> = annotations
        .iter()
        .map(|(location, annotation)| {
            let mut rendered = serde_json::to_value(annotation).unwrap_or_else(|_| {
                serde_json::json!({ "error": "annotation could not be serialized" })
            });
            if let Some(obj) = rendered.as_object_mut() {
                obj.remove("dateVerified");
            }
            serde_json::json!({
                "location": {
                    "file_path": location.file_path.to_string_lossy(),
                    "start_pos": location.span.start.0,
                    "end_pos": location.span.end.0
                },
                "annotation": rendered
            })
        })
        .collect();

    let json_data = serde_json::json!({
        "proof_annotations": annotations_json,
        "summary": {
            "total_annotations": annotations.len(),
            "cache_stats": {
                "size": cache_stats.size,
                "files_tracked": cache_stats.files_tracked
            }
        }
    });

    serde_json::to_string_pretty(&json_data).map_err(Into::into)
}

/// Setup the proof annotator with the real proof source.
///
/// This used to register three `MockProofSource`s -- a TEST DOUBLE -- in the
/// production path. Its own doc comment said "with mock sources". The mock
/// ignores the project path entirely and synthesises `count` annotations
/// against invented filenames, so `analyze proof-annotations` emitted exactly
/// 5 + 3 + 2 = 10 annotations naming borrow_checker_0.rs, static_analyzer_1.rs
/// and friends -- files that exist nowhere on disk -- for ANY path, including
/// a nonexistent one, each stamped with a fresh UUID and a current-time
/// `dateVerified`. That is machine-readable fabricated evidence of formal
/// verification, which is the most damaging thing this tool could emit.
///
/// `RustBorrowChecker` is a real, default-feature-enabled `ProofSource` that
/// walks the project and derives annotations from parsed source. It existed
/// the whole time with zero callers.
///
/// See contracts/pmat-no-fabrication-v1.yaml, `output_derived_from_input`.
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn setup_proof_annotator(clear_cache: bool) -> ProofAnnotator {
    use crate::services::{rust_borrow_checker::RustBorrowChecker, symbol_table::SymbolTable};

    let symbol_table = std::sync::Arc::new(SymbolTable::new());
    let mut annotator = ProofAnnotator::new(symbol_table);

    if clear_cache {
        annotator.clear_cache();
    }

    annotator.add_source(RustBorrowChecker::default());

    annotator
}

/// Filter and collect proof annotations
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn collect_and_filter_annotations(
    annotator: &ProofAnnotator,
    project_path: &Path,
    filter: &ProofAnnotationFilter,
) -> Vec<(Location, ProofAnnotation)> {
    let proof_map = annotator.collect_proofs(project_path).await;

    let mut collected: Vec<(Location, ProofAnnotation)> = proof_map
        .into_iter()
        .flat_map(|(location, annotations)| {
            annotations
                .into_iter()
                .filter(|annotation| filter_annotation(annotation, filter))
                .map(|annotation| (location.clone(), annotation))
                .collect::<Vec<_>>()
        })
        .collect();

    // DETERMINISM (round-3 sweep): `ProofMap` is a `HashMap<Location, ...>`, so
    // this vector came out in a per-process random order and every renderer
    // inherited it — 5 runs of `analyze proof-annotations --format json` over
    // an unchanged tree produced 5 different orderings of the same 1298
    // annotations (run 1 started at `name_similarity_help…`, run 2 at
    // `satd_formatting.rs`, run 3 at `tdg_handler_analysis…`). The content was
    // identical as a SET every time, which is precisely why the ordering was
    // the only thing making the output undiffable.
    //
    // Sorted by source position first, then by what is asserted there, so the
    // key is total: no two annotations at one site assert the same property by
    // the same method (that is also what `annotation_id` is derived from).
    collected.sort_by(|(la, aa), (lb, ab)| {
        la.file_path
            .cmp(&lb.file_path)
            .then_with(|| la.span.start.0.cmp(&lb.span.start.0))
            .then_with(|| la.span.end.0.cmp(&lb.span.end.0))
            .then_with(|| format!("{:?}", aa.property_proven).cmp(&format!("{:?}", ab.property_proven)))
            .then_with(|| format!("{:?}", aa.method).cmp(&format!("{:?}", ab.method)))
            .then_with(|| aa.specification_id.cmp(&ab.specification_id))
            .then_with(|| aa.annotation_id.cmp(&ab.annotation_id))
    });

    collected
}

/// Format annotations as table output
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_as_table(
    annotations: &[(Location, ProofAnnotation)],
    _elapsed: std::time::Duration,
) -> Result<String> {
    use std::fmt::Write;
    let mut output = String::new();

    writeln!(
        &mut output,
        "| File | Position | Property | Method | Confidence |"
    )?;
    writeln!(
        &mut output,
        "|------|----------|----------|---------|------------|"
    )?;

    for (location, annotation) in annotations {
        writeln!(
            &mut output,
            "| {} | {}-{} | {:?} | {:?} | {:?} |",
            location
                .file_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy(),
            location.span.start.0,
            location.span.end.0,
            annotation.property_proven,
            annotation.method,
            annotation.confidence_level
        )?;
    }

    Ok(output)
}

/// Format annotations as summary output
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_as_summary(
    annotations: &[(Location, ProofAnnotation)],
    elapsed: std::time::Duration,
) -> Result<String> {
    let mut output = String::new();

    format_summary_header(&mut output, annotations, elapsed)?;
    format_summary_property_counts(&mut output, annotations)?;
    format_summary_top_files(&mut output, annotations)?;

    Ok(output)
}

fn format_summary_header(
    output: &mut String,
    annotations: &[(Location, ProofAnnotation)],
    elapsed: std::time::Duration,
) -> Result<()> {
    use std::fmt::Write;

    let total_proofs = annotations.len();
    let high_confidence = annotations
        .iter()
        .filter(|(_, ann)| matches!(ann.confidence_level, ConfidenceLevel::High))
        .count();

    writeln!(output, "Proof Annotations Summary:")?;
    writeln!(output, "Total proofs: {total_proofs}\n")?;
    writeln!(
        output,
        "High confidence: {} ({:.1}%)",
        high_confidence,
        if total_proofs > 0 {
            (high_confidence as f64 / total_proofs as f64) * 100.0
        } else {
            0.0
        }
    )?;
    writeln!(output, "Analysis time: {:.2}s\n", elapsed.as_secs_f64())?;

    Ok(())
}

fn format_summary_property_counts(
    output: &mut String,
    annotations: &[(Location, ProofAnnotation)],
) -> Result<()> {
    use std::fmt::Write;

    // DETERMINISM: a `BTreeMap`, not a `HashMap`. `--format summary` listed the
    // same property counts in a different order on every run, for the same
    // reason the JSON entry order moved: nothing sorted them.
    let mut property_counts = std::collections::BTreeMap::new();
    for (_, ann) in annotations {
        let key = format!("{:?}", ann.property_proven);
        *property_counts.entry(key).or_insert(0) += 1;
    }

    if !property_counts.is_empty() {
        writeln!(output, "\nProofs by property type:")?;
        for (prop_type, count) in property_counts {
            writeln!(output, "  {prop_type}: {count}")?;
        }
    }

    Ok(())
}

fn format_summary_top_files(
    output: &mut String,
    annotations: &[(Location, ProofAnnotation)],
) -> Result<()> {
    use std::fmt::Write;

    if annotations.is_empty() {
        return Ok(());
    }

    writeln!(output, "\n## Top Files with Proof Annotations\n")?;

    let mut file_counts: std::collections::HashMap<&std::path::Path, usize> =
        std::collections::HashMap::new();
    for (location, _) in annotations {
        *file_counts.entry(&location.file_path).or_insert(0) += 1;
    }

    // DETERMINISM: the count alone is not a total order — most files tie — and
    // the input is a `HashMap`, so `.take(10)` picked whichever tied files the
    // hash seed visited first. Path breaks the tie.
    let mut sorted_files: Vec<_> = file_counts.into_iter().collect();
    sorted_files.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));

    for (i, (file_path, count)) in sorted_files.iter().take(10).enumerate() {
        let filename = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(file_path.to_str().unwrap_or("unknown"));
        writeln!(output, "{}. `{}` - {} annotations", i + 1, filename, count)?;
    }

    Ok(())
}
