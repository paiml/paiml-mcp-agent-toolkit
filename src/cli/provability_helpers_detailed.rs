/// Format provability results as detailed markdown
pub fn format_provability_detailed(
    function_ids: &[FunctionId],
    summaries: &[ProofSummary],
    include_evidence: bool,
) -> Result<String> {
    let mut output = String::new();

    writeln!(&mut output, "# Detailed Provability Analysis\n")?;
    let by_file = group_functions_by_file(function_ids, summaries);
    write_detailed_analysis_by_file(&mut output, by_file, include_evidence)?;

    Ok(output)
}

fn group_functions_by_file<'a>(
    function_ids: &'a [FunctionId],
    summaries: &'a [ProofSummary],
) -> HashMap<&'a str, Vec<(&'a FunctionId, &'a ProofSummary)>> {
    let mut by_file = HashMap::new();

    for (func_id, summary) in function_ids.iter().zip(summaries.iter()) {
        by_file
            .entry(func_id.file_path.as_str())
            .or_insert_with(Vec::new)
            .push((func_id, summary));
    }

    by_file
}

fn write_detailed_analysis_by_file(
    output: &mut String,
    by_file: HashMap<&str, Vec<(&FunctionId, &ProofSummary)>>,
    include_evidence: bool,
) -> Result<()> {
    for (file_path, functions) in by_file {
        write_file_section(output, file_path, &functions, include_evidence)?;
    }
    Ok(())
}

fn write_file_section(
    output: &mut String,
    file_path: &str,
    functions: &[(&FunctionId, &ProofSummary)],
    include_evidence: bool,
) -> Result<()> {
    writeln!(output, "## {file_path}\n")?;

    for (func_id, summary) in functions {
        write_function_details(output, func_id, summary)?;

        if include_evidence && !summary.verified_properties.is_empty() {
            write_verified_properties(output, &summary.verified_properties)?;
        }

        writeln!(output)?;
    }

    Ok(())
}

fn write_function_details(
    output: &mut String,
    func_id: &FunctionId,
    summary: &ProofSummary,
) -> Result<()> {
    use crate::services::lightweight_provability_analyzer::PropertyType;

    writeln!(output, "### Function: `{}`", func_id.function_name)?;
    writeln!(output, "- **Line**: {}", func_id.line_number)?;
    writeln!(
        output,
        "- **Provability Score**: {:.1}%",
        summary.provability_score * 100.0
    )?;
    writeln!(
        output,
        "- **Analysis Time**: {}\u{00b5}s",
        summary.analysis_time_us
    )?;

    // Show which properties are verified vs missing (#229)
    let all_types = [
        PropertyType::NullSafety,
        PropertyType::BoundsCheck,
        PropertyType::NoAliasing,
        PropertyType::PureFunction,
        PropertyType::MemorySafety,
        PropertyType::ThreadSafety,
    ];
    let verified_types: Vec<&PropertyType> = summary.verified_properties.iter().map(|p| &p.property_type).collect();
    let mut verified = Vec::new();
    let mut missing = Vec::new();
    for pt in &all_types {
        if verified_types.contains(&pt) {
            verified.push(format!("{pt:?}"));
        } else {
            missing.push(format!("{pt:?}"));
        }
    }
    if !verified.is_empty() {
        writeln!(output, "- **Verified**: {}", verified.join(", "))?;
    }
    if !missing.is_empty() {
        writeln!(output, "- **Missing**: {}", missing.join(", "))?;
    }

    Ok(())
}

fn write_verified_properties(
    output: &mut String,
    properties: &[crate::services::lightweight_provability_analyzer::VerifiedProperty],
) -> Result<()> {
    writeln!(output, "\n#### Verified Properties:")?;

    for prop in properties {
        writeln!(
            output,
            "- **{:?}** (confidence: {:.0}%)",
            prop.property_type,
            prop.confidence * 100.0
        )?;
        writeln!(output, "  - Evidence: {}", prop.evidence)?;
    }

    Ok(())
}
