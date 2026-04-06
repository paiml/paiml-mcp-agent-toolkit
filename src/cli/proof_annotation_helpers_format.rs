// Formatting functions: JSON output, table output, summary output, annotator setup, and annotation collection.

/// Format proof annotations as JSON
pub fn format_as_json(
    annotations: &[(Location, ProofAnnotation)],
    elapsed: std::time::Duration,
    annotator: &ProofAnnotator,
) -> Result<String> {
    debug_assert!(!annotations.is_empty(), "annotations must not be empty");
    let cache_stats = annotator.cache_stats();
    let annotations_json: Vec<serde_json::Value> = annotations
        .iter()
        .map(|(location, annotation)| {
            serde_json::json!({
                "location": {
                    "file_path": location.file_path.to_string_lossy(),
                    "start_pos": location.span.start.0,
                    "end_pos": location.span.end.0
                },
                "annotation": annotation
            })
        })
        .collect();

    let json_data = serde_json::json!({
        "proof_annotations": annotations_json,
        "summary": {
            "total_annotations": annotations.len(),
            "analysis_time_ms": elapsed.as_millis(),
            "cache_stats": {
                "size": cache_stats.size,
                "files_tracked": cache_stats.files_tracked
            }
        }
    });

    serde_json::to_string_pretty(&json_data).map_err(Into::into)
}

/// Setup proof annotator with mock sources
#[must_use]
pub fn setup_proof_annotator(clear_cache: bool) -> ProofAnnotator {
    use crate::services::{proof_annotator::MockProofSource, symbol_table::SymbolTable};

    let symbol_table = std::sync::Arc::new(SymbolTable::new());
    let mut annotator = ProofAnnotator::new(symbol_table);

    if clear_cache {
        annotator.clear_cache();
    }

    // Add mock proof sources
    annotator.add_source(MockProofSource::new("borrow_checker".to_string(), 10, 5));
    annotator.add_source(MockProofSource::new("static_analyzer".to_string(), 20, 3));
    annotator.add_source(MockProofSource::new("formal_verifier".to_string(), 50, 2));

    annotator
}

/// Filter and collect proof annotations
pub async fn collect_and_filter_annotations(
    annotator: &ProofAnnotator,
    project_path: &Path,
    filter: &ProofAnnotationFilter,
) -> Vec<(Location, ProofAnnotation)> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let proof_map = annotator.collect_proofs(project_path).await;

    proof_map
        .into_iter()
        .flat_map(|(location, annotations)| {
            annotations
                .into_iter()
                .filter(|annotation| filter_annotation(annotation, filter))
                .map(|annotation| (location.clone(), annotation))
                .collect::<Vec<_>>()
        })
        .collect()
}

/// Format annotations as table output
pub fn format_as_table(
    annotations: &[(Location, ProofAnnotation)],
    _elapsed: std::time::Duration,
) -> Result<String> {
    debug_assert!(!annotations.is_empty(), "annotations must not be empty");
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
pub fn format_as_summary(
    annotations: &[(Location, ProofAnnotation)],
    elapsed: std::time::Duration,
) -> Result<String> {
    debug_assert!(!annotations.is_empty(), "annotations must not be empty");
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
    debug_assert!(!annotations.is_empty(), "annotations must not be empty");
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
    debug_assert!(!annotations.is_empty(), "annotations must not be empty");
    use std::fmt::Write;

    let mut property_counts = std::collections::HashMap::new();
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
    debug_assert!(!annotations.is_empty(), "annotations must not be empty");
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

    let mut sorted_files: Vec<_> = file_counts.into_iter().collect();
    sorted_files.sort_by(|a, b| b.1.cmp(&a.1));

    for (i, (file_path, count)) in sorted_files.iter().take(10).enumerate() {
        let filename = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(file_path.to_str().unwrap_or("unknown"));
        writeln!(output, "{}. `{}` - {} annotations", i + 1, filename, count)?;
    }

    Ok(())
}
