/// Format provability results as detailed markdown
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_provability_detailed(
    function_ids: &[FunctionId],
    summaries: &[ProofSummary],
    include_evidence: bool,
) -> Result<String> {
    let mut output = String::new();

    use crate::cli::colors as c;
    writeln!(&mut output, "{}\n", c::header("Detailed Provability Analysis"))?;
    let by_file = group_functions_by_file(function_ids, summaries);
    write_detailed_analysis_by_file(&mut output, by_file, include_evidence)?;

    Ok(output)
}

/// DETERMINISM (round-3 sweep): a `BTreeMap`, not a `HashMap` — the caller
/// iterates this map to emit one section per file, so the file sections came
/// out in a per-process order and `--format markdown` / `--format full` on an
/// unchanged tree diffed against itself.
fn group_functions_by_file<'a>(
    function_ids: &'a [FunctionId],
    summaries: &'a [ProofSummary],
) -> std::collections::BTreeMap<&'a str, Vec<(&'a FunctionId, &'a ProofSummary)>> {
    let mut by_file = std::collections::BTreeMap::new();

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
    by_file: std::collections::BTreeMap<&str, Vec<(&FunctionId, &ProofSummary)>>,
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
    use crate::cli::colors as c;
    writeln!(output, "{}\n", c::subheader(&c::path(file_path)))?;

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
    use crate::cli::colors as c;
    use crate::services::lightweight_provability_analyzer::PropertyType;

    writeln!(output, "  {}Function:{} {}", c::BOLD, c::RESET, c::label(&func_id.function_name))?;
    writeln!(output, "    Line: {}", c::number(&func_id.line_number.to_string()))?;
    writeln!(
        output,
        "    Provability Score: {}",
        c::pct(summary.provability_score * 100.0, 80.0, 50.0)
    )?;
    // DETERMINISM (round-3 sweep): `analysis_time_us` used to be printed here
    // and in the JSON. It is how long THIS machine took under whatever load it
    // was under — the same four functions came back 28/1/1/1 µs on one run and
    // 12/2/2/2 µs on the next — so it made `--format markdown` diff against
    // itself on unchanged input while telling the reader nothing about the
    // code. The command's total elapsed time is still reported by the handler.

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
        writeln!(output, "    {}Verified:{} {}{}{}", c::BOLD, c::RESET, c::GREEN, verified.join(", "), c::RESET)?;
    }
    if !missing.is_empty() {
        writeln!(output, "    {}Missing:{} {}{}{}", c::BOLD, c::RESET, c::RED, missing.join(", "), c::RESET)?;
    }

    Ok(())
}

fn write_verified_properties(
    output: &mut String,
    properties: &[crate::services::lightweight_provability_analyzer::VerifiedProperty],
) -> Result<()> {
    use crate::cli::colors as c;
    writeln!(output, "\n    {}Verified Properties:{}", c::BOLD, c::RESET)?;

    for prop in properties {
        writeln!(
            output,
            "      {}{:?}{} (confidence: {})",
            c::GREEN, prop.property_type, c::RESET,
            c::pct(prop.confidence * 100.0, 80.0, 50.0),
        )?;
        writeln!(output, "        Evidence: {}{}{}", c::DIM, prop.evidence, c::RESET)?;
    }

    Ok(())
}
