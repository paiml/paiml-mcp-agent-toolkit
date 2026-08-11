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

    // `c::seq`, not the bare `pub const`s: the raw sequences are `const` and so
    // cannot consult `colors_enabled()`, which is why `--color never` and a
    // redirected stdout still received `^[[1mFunction:^[[0m` here long after
    // the summary renderer was migrated (GH #684).
    writeln!(output, "  {}Function:{} {}", c::seq(c::BOLD), c::seq(c::RESET), c::label(&func_id.function_name))?;
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
        writeln!(output, "    {}Verified:{} {}{}{}", c::seq(c::BOLD), c::seq(c::RESET), c::seq(c::GREEN), verified.join(", "), c::seq(c::RESET))?;
    }
    if !missing.is_empty() {
        writeln!(output, "    {}Missing:{} {}{}{}", c::seq(c::BOLD), c::seq(c::RESET), c::seq(c::RED), missing.join(", "), c::seq(c::RESET))?;
    }

    Ok(())
}

/// Format provability results as GitHub-flavoured Markdown.
///
/// `-f markdown` was an alias for `-f full`: the two produced byte-identical
/// output, so the "markdown" format emitted the ANSI-decorated terminal
/// rendering (`^[[1mFunction:^[[0m add`) with not one markdown construct in it.
/// A format that names a syntax has to produce that syntax.
pub fn format_provability_markdown(
    function_ids: &[FunctionId],
    summaries: &[ProofSummary],
    include_evidence: bool,
) -> Result<String> {
    use crate::services::lightweight_provability_analyzer::PropertyType;

    const ALL_PROPERTIES: [PropertyType; 6] = [
        PropertyType::NullSafety,
        PropertyType::BoundsCheck,
        PropertyType::NoAliasing,
        PropertyType::PureFunction,
        PropertyType::MemorySafety,
        PropertyType::ThreadSafety,
    ];

    let mut output = String::new();
    writeln!(&mut output, "# Provability Analysis\n")?;
    writeln!(
        &mut output,
        "Functions analyzed: {}\n",
        function_ids.len()
    )?;

    for (file_path, functions) in group_functions_by_file(function_ids, summaries) {
        writeln!(&mut output, "## `{file_path}`\n")?;
        writeln!(
            &mut output,
            "| Function | Line | Provability | Verified | Missing |"
        )?;
        writeln!(&mut output, "| --- | ---: | ---: | --- | --- |")?;

        for (func_id, summary) in &functions {
            let verified_types: Vec<&PropertyType> = summary
                .verified_properties
                .iter()
                .map(|p| &p.property_type)
                .collect();
            let (verified, missing): (Vec<String>, Vec<String>) = ALL_PROPERTIES
                .iter()
                .map(|pt| (format!("{pt:?}"), verified_types.contains(&pt)))
                .fold((Vec::new(), Vec::new()), |(mut v, mut m), (name, ok)| {
                    if ok {
                        v.push(name);
                    } else {
                        m.push(name);
                    }
                    (v, m)
                });
            let or_dash = |items: Vec<String>| {
                if items.is_empty() {
                    "—".to_string()
                } else {
                    items.join(", ")
                }
            };

            writeln!(
                &mut output,
                "| `{}` | {} | {:.1}% | {} | {} |",
                func_id.function_name,
                func_id.line_number,
                summary.provability_score * 100.0,
                or_dash(verified),
                or_dash(missing),
            )?;
        }
        writeln!(&mut output)?;

        if include_evidence {
            for (func_id, summary) in &functions {
                if summary.verified_properties.is_empty() {
                    continue;
                }
                writeln!(&mut output, "### `{}` — evidence\n", func_id.function_name)?;
                for prop in &summary.verified_properties {
                    writeln!(
                        &mut output,
                        "- **{:?}** ({:.1}% confidence): {}",
                        prop.property_type,
                        prop.confidence * 100.0,
                        prop.evidence,
                    )?;
                }
                writeln!(&mut output)?;
            }
        }
    }

    Ok(output)
}

fn write_verified_properties(
    output: &mut String,
    properties: &[crate::services::lightweight_provability_analyzer::VerifiedProperty],
) -> Result<()> {
    use crate::cli::colors as c;
    writeln!(output, "\n    {}Verified Properties:{}", c::seq(c::BOLD), c::seq(c::RESET))?;

    for prop in properties {
        writeln!(
            output,
            "      {}{:?}{} (confidence: {})",
            c::seq(c::GREEN), prop.property_type, c::seq(c::RESET),
            c::pct(prop.confidence * 100.0, 80.0, 50.0),
        )?;
        writeln!(output, "        Evidence: {}{}{}", c::seq(c::DIM), prop.evidence, c::seq(c::RESET))?;
    }

    Ok(())
}

#[cfg(test)]
mod markdown_and_plain_output_tests {
    //! `-f markdown` was an alias for `-f full` (identical md5), and the
    //! detailed renderer interpolated the raw `pub const` ANSI sequences, so a
    //! redirected `-f markdown` file was ANSI-decorated terminal text.
    use super::*;
    use crate::services::lightweight_provability_analyzer::{
        FunctionId, ProofSummary, PropertyType, VerifiedProperty,
    };

    fn fixture() -> (Vec<FunctionId>, Vec<ProofSummary>) {
        let ids = vec![
            FunctionId {
                file_path: "src/lib.rs".to_string(),
                function_name: "add".to_string(),
                line_number: 2,
            },
            FunctionId {
                file_path: "src/lib.rs".to_string(),
                function_name: "complex".to_string(),
                line_number: 4,
            },
        ];
        let summaries = vec![
            ProofSummary {
                provability_score: 0.2,
                analysis_time_us: 12,
                verified_properties: vec![],
                version: 1,
            },
            ProofSummary {
                provability_score: 0.9,
                analysis_time_us: 3,
                verified_properties: vec![VerifiedProperty {
                    property_type: PropertyType::PureFunction,
                    confidence: 0.75,
                    evidence: "no writes to globals".to_string(),
                }],
                version: 1,
            },
        ];
        (ids, summaries)
    }

    #[test]
    fn detailed_output_is_plain_text_when_colour_is_disabled() {
        assert!(
            !crate::cli::colors::colors_enabled(),
            "cargo test captures stdout, so colour must resolve to off here"
        );

        let (ids, summaries) = fixture();
        let rendered = format_provability_detailed(&ids, &summaries, true).expect("render");

        assert!(
            !rendered.contains('\u{1b}'),
            "no ANSI escape may reach a redirected stdout: {:?}",
            rendered
                .lines()
                .filter(|l| l.contains('\u{1b}'))
                .collect::<Vec<_>>()
        );
        // The payload must survive the de-colouring.
        assert!(rendered.contains("Detailed Provability Analysis"));
        assert!(rendered.contains("Function:"));
        assert!(rendered.contains("Missing:"));
        assert!(rendered.contains("no writes to globals"));
    }

    #[test]
    fn markdown_is_markdown_and_not_the_terminal_rendering() {
        let (ids, summaries) = fixture();
        let markdown = format_provability_markdown(&ids, &summaries, true).expect("render");
        let detailed = format_provability_detailed(&ids, &summaries, true).expect("render");

        assert_ne!(
            markdown, detailed,
            "`-f markdown` must not be an alias for `-f full`"
        );
        assert!(!markdown.contains('\u{1b}'), "markdown must be plain text");

        assert!(markdown.starts_with("# Provability Analysis"));
        assert!(markdown.contains("## `src/lib.rs`"));
        assert!(markdown.contains("| Function | Line | Provability | Verified | Missing |"));
        assert!(markdown.contains("| `add` | 2 | 20.0% |"));
        assert!(markdown.contains("| `complex` | 4 | 90.0% | PureFunction |"));
        // Evidence is a section, not a colour-coded block.
        assert!(markdown.contains("### `complex` — evidence"));
        assert!(markdown.contains("- **PureFunction** (75.0% confidence): no writes to globals"));
    }

    #[test]
    fn markdown_omits_the_evidence_sections_when_not_requested() {
        let (ids, summaries) = fixture();
        let markdown = format_provability_markdown(&ids, &summaries, false).expect("render");
        assert!(!markdown.contains("evidence"));
        // The table itself is unaffected.
        assert!(markdown.contains("| `complex` | 4 | 90.0% | PureFunction |"));
    }
}
