// Report generation: full detailed output, markdown output, and SARIF output formatting.

/// Format annotations as full detailed output
pub fn format_as_full(
    annotations: &[(Location, ProofAnnotation)],
    project_path: &Path,
    include_evidence: bool,
) -> Result<String> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let mut output = String::new();

    write_report_header(&mut output, project_path, annotations.len())?;

    let proofs_by_file = group_proofs_by_file(annotations);

    for (file, proofs) in proofs_by_file {
        write_file_section(&mut output, &file, proofs, include_evidence)?;
    }

    Ok(output)
}

/// Write the report header with project information
fn write_report_header(
    output: &mut String,
    project_path: &Path,
    total_proofs: usize,
) -> Result<()> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    use std::fmt::Write;

    writeln!(output, "# Full Proof Annotations Report\n")?;
    writeln!(
        output,
        "**Generated**: {}",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    )?;
    writeln!(output, "**Project**: {}", project_path.display())?;
    writeln!(output, "**Total proofs**: {total_proofs}\n")?;

    Ok(())
}

/// Group proof annotations by file
fn group_proofs_by_file(
    annotations: &[(Location, ProofAnnotation)],
) -> std::collections::HashMap<std::path::PathBuf, Vec<(Location, ProofAnnotation)>> {
    debug_assert!(!annotations.is_empty(), "annotations must not be empty");
    let mut proofs_by_file: std::collections::HashMap<
        std::path::PathBuf,
        Vec<(Location, ProofAnnotation)>,
    > = std::collections::HashMap::new();

    for (loc, ann) in annotations {
        proofs_by_file
            .entry(loc.file_path.clone())
            .or_default()
            .push((loc.clone(), ann.clone()));
    }

    proofs_by_file
}

/// Write a file section with its proofs
fn write_file_section(
    output: &mut String,
    file: &Path,
    mut proofs: Vec<(Location, ProofAnnotation)>,
    include_evidence: bool,
) -> Result<()> {
    debug_assert!(file.exists(), "file must exist: {}", file.display());
    use std::fmt::Write;

    writeln!(output, "## File: {}\n", file.display())?;

    // Sort by line number
    proofs.sort_by_key(|(loc, _)| loc.span.start.0);

    for (loc, ann) in proofs {
        write_proof_annotation(output, &loc, &ann, include_evidence)?;
    }

    Ok(())
}

/// Write a single proof annotation
fn write_proof_annotation(
    output: &mut String,
    loc: &Location,
    ann: &ProofAnnotation,
    include_evidence: bool,
) -> Result<()> {
    use std::fmt::Write;

    write_annotation_header(output, loc)?;
    write_annotation_basic_info(output, ann)?;
    write_annotation_assumptions(output, ann)?;

    if include_evidence {
        write_annotation_evidence(output, ann)?;
    }

    writeln!(output)?;
    Ok(())
}

/// Write annotation position header
fn write_annotation_header(output: &mut String, loc: &Location) -> Result<()> {
    use std::fmt::Write;
    writeln!(
        output,
        "### Position {}-{}\n",
        loc.span.start.0, loc.span.end.0
    )?;
    Ok(())
}

/// Write basic annotation information
fn write_annotation_basic_info(output: &mut String, ann: &ProofAnnotation) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "**Property**: {:?}", ann.property_proven)?;
    writeln!(output, "**Method**: {:?}", ann.method)?;
    writeln!(output, "**Tool**: {} v{}", ann.tool_name, ann.tool_version)?;
    writeln!(output, "**Confidence**: {:?}", ann.confidence_level)?;
    writeln!(
        output,
        "**Verified**: {}",
        ann.date_verified.format("%Y-%m-%d %H:%M:%S UTC")
    )?;

    Ok(())
}

/// Write annotation assumptions
fn write_annotation_assumptions(output: &mut String, ann: &ProofAnnotation) -> Result<()> {
    use std::fmt::Write;

    if !ann.assumptions.is_empty() {
        writeln!(output, "\n**Assumptions**:")?;
        for assumption in &ann.assumptions {
            writeln!(output, "- {assumption}")?;
        }
    }

    Ok(())
}

/// Write annotation evidence information
fn write_annotation_evidence(output: &mut String, ann: &ProofAnnotation) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "\n**Evidence**: {:?}", ann.evidence_type)?;
    if let Some(ref spec_id) = ann.specification_id {
        writeln!(output, "**Specification ID**: {spec_id}")?;
    }

    Ok(())
}

/// Format annotations as markdown output
pub fn format_as_markdown(
    annotations: &[(Location, ProofAnnotation)],
    project_path: &Path,
    include_evidence: bool,
) -> Result<String> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let mut output = String::new();

    write_markdown_header(&mut output, project_path, annotations.len())?;
    write_summary_statistics(&mut output, annotations)?;

    if include_evidence {
        write_detailed_proofs(&mut output, annotations, include_evidence)?;
    }

    Ok(output)
}

/// Write markdown report header
fn write_markdown_header(
    output: &mut String,
    project_path: &Path,
    total_proofs: usize,
) -> Result<()> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    use std::fmt::Write;

    writeln!(output, "# Proof Annotations Analysis\n")?;
    writeln!(output, "This report shows formal verification proofs collected from various tools and analyzers.\n")?;

    writeln!(output, "**Project Path**: `{}`", project_path.display())?;
    writeln!(
        output,
        "**Analysis Date**: {}",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    )?;
    writeln!(output, "**Total Proofs**: {total_proofs}\n")?;

    Ok(())
}

/// Write summary statistics table
fn write_summary_statistics(
    output: &mut String,
    annotations: &[(Location, ProofAnnotation)],
) -> Result<()> {
    debug_assert!(!annotations.is_empty(), "annotations must not be empty");
    use std::fmt::Write;

    writeln!(output, "## Summary Statistics\n")?;
    writeln!(output, "| Metric | Count |")?;
    writeln!(output, "|--------|-------|")?;

    let confidence_counts = count_by_confidence(annotations);

    for (level, count) in &confidence_counts {
        writeln!(output, "| {level} Confidence | {count} |")?;
    }

    Ok(())
}

/// Count annotations by confidence level
fn count_by_confidence(
    annotations: &[(Location, ProofAnnotation)],
) -> std::collections::HashMap<String, usize> {
    debug_assert!(!annotations.is_empty(), "annotations must not be empty");
    let mut confidence_counts = std::collections::HashMap::new();

    for (_, ann) in annotations {
        let key = format!("{:?}", ann.confidence_level);
        *confidence_counts.entry(key).or_insert(0) += 1;
    }

    confidence_counts
}

/// Write detailed proofs section
fn write_detailed_proofs(
    output: &mut String,
    annotations: &[(Location, ProofAnnotation)],
    include_evidence: bool,
) -> Result<()> {
    debug_assert!(!annotations.is_empty(), "annotations must not be empty");
    use std::fmt::Write;

    writeln!(output, "\n## Detailed Proofs\n")?;

    let proofs_by_file = group_proofs_by_file(annotations);

    for (file, proofs) in proofs_by_file {
        write_file_proofs_section(output, &file, &proofs, include_evidence)?;
    }

    Ok(())
}

/// Write proofs section for a specific file
fn write_file_proofs_section(
    output: &mut String,
    file: &Path,
    proofs: &[(Location, ProofAnnotation)],
    include_evidence: bool,
) -> Result<()> {
    debug_assert!(file.exists(), "file must exist: {}", file.display());
    use std::fmt::Write;

    writeln!(output, "### {}\n", file.display())?;

    for (loc, ann) in proofs {
        write_proof_summary_item(output, loc, ann, include_evidence)?;
    }

    writeln!(output)?;
    Ok(())
}

/// Write a single proof summary item
fn write_proof_summary_item(
    output: &mut String,
    loc: &Location,
    ann: &ProofAnnotation,
    include_evidence: bool,
) -> Result<()> {
    use std::fmt::Write;

    writeln!(
        output,
        "- **{:?}** at lines {}-{}",
        ann.property_proven, loc.span.start.0, loc.span.end.0
    )?;
    writeln!(output, "  - Method: {:?}", ann.method)?;
    writeln!(output, "  - Confidence: {:?}", ann.confidence_level)?;

    if include_evidence {
        writeln!(output, "  - Evidence: {:?}", ann.evidence_type)?;
    }

    Ok(())
}

/// Format annotations as SARIF output
pub fn format_as_sarif(
    annotations: &[(Location, ProofAnnotation)],
    _project_path: &Path,
) -> Result<String> {
    debug_assert!(_project_path.exists(), "_project_path must exist: {}", _project_path.display());
    let mut results = Vec::new();

    for (location, annotation) in annotations {
        let rule_id = match annotation.confidence_level {
            ConfidenceLevel::Low => "low-confidence-proof",
            ConfidenceLevel::Medium => "medium-confidence-proof",
            ConfidenceLevel::High => "high-confidence-proof",
        };

        let level = match annotation.confidence_level {
            ConfidenceLevel::Low => "warning",
            ConfidenceLevel::Medium => "note",
            ConfidenceLevel::High => "none",
        };

        results.push(serde_json::json!({
            "ruleId": rule_id,
            "level": level,
            "message": {
                "text": format!(
                    "{:?} verified by {} using {:?}",
                    annotation.property_proven,
                    annotation.tool_name,
                    annotation.method
                )
            },
            "locations": [{
                "physicalLocation": {
                    "artifactLocation": {
                        "uri": location.file_path.to_string_lossy()
                    },
                    "region": {
                        "startLine": location.span.start.0,
                        "endLine": location.span.end.0
                    }
                }
            }]
        }));
    }

    let sarif = serde_json::json!({
        "version": "2.1.0",
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "paiml-proof-annotator",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                    "rules": [
                        {
                            "id": "low-confidence-proof",
                            "name": "Low Confidence Proof",
                            "shortDescription": {
                                "text": "Property verification has low confidence"
                            },
                            "defaultConfiguration": {
                                "level": "warning"
                            }
                        },
                        {
                            "id": "medium-confidence-proof",
                            "name": "Medium Confidence Proof",
                            "shortDescription": {
                                "text": "Property verification has medium confidence"
                            },
                            "defaultConfiguration": {
                                "level": "note"
                            }
                        },
                        {
                            "id": "high-confidence-proof",
                            "name": "High Confidence Proof",
                            "shortDescription": {
                                "text": "Property verification has high confidence"
                            },
                            "defaultConfiguration": {
                                "level": "none"
                            }
                        }
                    ]
                }
            },
            "results": results
        }]
    });

    serde_json::to_string_pretty(&sarif).map_err(Into::into)
}
