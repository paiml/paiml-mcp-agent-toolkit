// Dead code output formatting - included from dead_code_handlers.rs
// NO `use` imports or `#!` inner attributes allowed here.

/// Format dead code result based on output format
fn format_dead_code_result(
    result: &crate::models::dead_code::DeadCodeResult,
    format: &DeadCodeOutputFormat,
) -> Result<String> {
    match format {
        DeadCodeOutputFormat::Json => format_dead_code_as_json(result),
        DeadCodeOutputFormat::Sarif => format_dead_code_as_sarif(result),
        DeadCodeOutputFormat::Summary => format_dead_code_as_summary(result),
        DeadCodeOutputFormat::Markdown => format_dead_code_as_markdown(result),
    }
}

/// Format result as JSON
fn format_dead_code_as_json(result: &crate::models::dead_code::DeadCodeResult) -> Result<String> {
    Ok(serde_json::to_string_pretty(result)?)
}

/// Format result as SARIF
fn format_dead_code_as_sarif(result: &crate::models::dead_code::DeadCodeResult) -> Result<String> {
    use crate::models::dead_code::{ConfidenceLevel, DeadCodeType};
    use serde_json::json;

    let sarif = json!({
        "version": "2.1.0",
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "pmat",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                    "rules": [{
                        "id": "dead-code",
                        "name": "Dead Code Detection",
                        "shortDescription": {
                            "text": "Code that is never executed or referenced"
                        },
                        "fullDescription": {
                            "text": "Detects functions, classes, and code blocks that are not reachable from any entry point"
                        },
                        "defaultConfiguration": {
                            "level": "warning"
                        }
                    }]
                }
            },
            "results": result.files.iter().flat_map(|file| {
                file.items.iter().map(|item| {
                    let level = match file.confidence {
                        ConfidenceLevel::High => "error",
                        ConfidenceLevel::Medium => "warning",
                        ConfidenceLevel::Low => "note",
                    };
                    json!({
                        "ruleId": "dead-code",
                        "level": level,
                        "message": {
                            "text": format!("{}: {}",
                                match item.item_type {
                                    DeadCodeType::Function => "Dead function",
                                    DeadCodeType::Class => "Dead class",
                                    DeadCodeType::Variable => "Dead variable",
                                    DeadCodeType::UnreachableCode => "Unreachable code",
                                },
                                item.reason
                            )
                        },
                        "locations": [{
                            "physicalLocation": {
                                "artifactLocation": {
                                    "uri": &file.path
                                },
                                "region": {
                                    "startLine": item.line
                                }
                            }
                        }]
                    })
                }).collect::<Vec<_>>()
            }).collect::<Vec<_>>()
        }]
    });
    Ok(serde_json::to_string_pretty(&sarif)?)
}

/// Format result as summary
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_dead_code_as_summary(
    result: &crate::models::dead_code::DeadCodeResult,
) -> Result<String> {
    let mut output = String::new();

    write_dead_code_header(&mut output, result)?;

    // Print the breakdown whenever there is anything to break down. Gating on
    // `dead_functions > 0` hid it exactly when it was needed: a report of 26
    // dead lines made entirely of dead fields showed no types at all.
    if result.summary.total_dead_lines > 0 || !result.files.is_empty() {
        write_dead_code_by_type_section(&mut output, result)?;
    }

    if !result.files.is_empty() {
        write_top_files_section(&mut output, &result.files)?;
    }

    Ok(output)
}

/// Write dead code analysis header section
fn write_dead_code_header(
    output: &mut String,
    result: &crate::models::dead_code::DeadCodeResult,
) -> Result<()> {
    use crate::cli::colors as c;
    use std::fmt::Write;

    writeln!(output, "{}\n", c::header("Dead Code Analysis Summary"))?;
    writeln!(
        output,
        "  {} {}",
        c::label("Files analyzed:"),
        c::number(&result.analyzed_files.to_string())
    )?;
    // Name the narrowing. Without it, a repo whose only dead code lives in test
    // code got an all-zero report headed by the project's whole file count -- a
    // clean bill of health for files the scan never opened.
    if result.total_files > result.analyzed_files {
        writeln!(
            output,
            "  {} {} (tests, examples and benches; --include-tests scans test code)",
            c::label("Files skipped (out of scope):"),
            c::number(&(result.total_files - result.analyzed_files).to_string())
        )?;
    }
    writeln!(
        output,
        "  {} {}",
        c::label("Files with dead code:"),
        c::number(&result.summary.files_with_dead_code.to_string())
    )?;
    // Name the cap instead of letting the reported count stand for the total:
    // `files_with_dead_code: 26` used to head a list of 4.
    let omitted = result.files_omitted();
    if omitted > 0 {
        writeln!(
            output,
            "  {} {} ({} not listed: below --min-dead-lines{})",
            c::label("Files found with dead code:"),
            c::number(&result.files_with_dead_code_found.to_string()),
            c::number(&omitted.to_string()),
            if result.files_truncated {
                " or beyond --top-files"
            } else {
                ""
            }
        )?;
    }
    writeln!(
        output,
        "  {} {}",
        c::label("Total dead lines:"),
        c::number(&result.summary.total_dead_lines.to_string())
    )?;
    writeln!(
        output,
        "  {} {}\n",
        c::label("Dead code percentage:"),
        c::pct(
            f64::from(result.summary.dead_percentage),
            5.0,
            15.0,
        )
    )?;

    Ok(())
}

/// Write dead code by type breakdown section.
///
/// Every reported dead item lands in exactly one row. The "Dead variables" row
/// used to print `summary.dead_modules` — a module count under a variable label
/// — and fields, constants and statics were counted in no row at all, so a
/// report of 26 dead lines could show four zeros beneath it.
fn write_dead_code_by_type_section(
    output: &mut String,
    result: &crate::models::dead_code::DeadCodeResult,
) -> Result<()> {
    use crate::cli::colors as c;
    use crate::models::dead_code::DeadCodeType;
    use std::fmt::Write;

    let summary = &result.summary;
    let other_items = result
        .files
        .iter()
        .flat_map(|f| f.items.iter())
        .filter(|item| matches!(item.item_type, DeadCodeType::Variable))
        .count();

    writeln!(output, "{}\n", c::subheader("Dead Code by Type"))?;
    for (label, value) in [
        ("Dead functions:", summary.dead_functions),
        ("Dead classes:", summary.dead_classes),
        ("Dead modules:", summary.dead_modules),
        ("Other (fields, constants, statics):", other_items),
        ("Unreachable blocks:", summary.unreachable_blocks),
    ] {
        writeln!(
            output,
            "  {} {}",
            c::label(label),
            c::number(&value.to_string())
        )?;
    }

    Ok(())
}

/// Write top files with dead code section
fn write_top_files_section(
    output: &mut String,
    files: &[crate::models::dead_code::FileDeadCodeMetrics],
) -> Result<()> {
    use crate::cli::colors as c;
    use std::fmt::Write;

    writeln!(output, "\n{}\n", c::subheader("Top Files with Dead Code"))?;
    for (i, file) in files.iter().take(10).enumerate() {
        writeln!(
            output,
            "  {}. {} - {} dead ({} lines)",
            c::number(&(i + 1).to_string()),
            c::path(&file.path),
            c::pct(f64::from(file.dead_percentage), 5.0, 15.0),
            c::number(&file.dead_lines.to_string())
        )?;
    }

    Ok(())
}

/// Format result as markdown
fn format_dead_code_as_markdown(
    result: &crate::models::dead_code::DeadCodeResult,
) -> Result<String> {
    let mut sections = Vec::new();

    // Build summary section
    sections.push(format_dead_code_summary_section(result));

    // Build breakdown section if needed
    if result.summary.dead_functions > 0 {
        sections.push(format_dead_code_breakdown_section(result));
    }

    // Build file details section if needed
    if !result.files.is_empty() {
        sections.push(format_dead_code_file_details_section(&result.files));
    }

    // Build recommendations section
    sections.push(format_dead_code_recommendations_section());

    Ok(sections.join("\n"))
}

fn format_dead_code_summary_section(result: &crate::models::dead_code::DeadCodeResult) -> String {
    format!(
        "# Dead Code Analysis Report\n\n\
         ## Summary\n\n\
         | Metric | Value |\n\
         |--------|-------|\n\
         | Files Analyzed | {} |\n\
         | Files Skipped (out of scope) | {} |\n\
         | Files with Dead Code | {} |\n\
         | Total Dead Lines | {} |\n\
         | Dead Code Percentage | {:.2}% |\n",
        result.analyzed_files,
        result.total_files.saturating_sub(result.analyzed_files),
        result.summary.files_with_dead_code,
        result.summary.total_dead_lines,
        result.summary.dead_percentage
    )
}

/// Write the markdown breakdown table.
///
/// The `Modules` row prints `summary.dead_modules`, which is a MODULE count on
/// the cargo path. It was labelled `Variables` here -- the same mislabel the
/// text renderer carried (#721) -- so a cargo run reported its dead modules
/// under a row heading no producer fills. Fields, constants and statics are
/// counted from the items themselves, exactly as the text renderer does, so
/// every reported dead item lands in one row.
fn format_dead_code_breakdown_section(
    result: &crate::models::dead_code::DeadCodeResult,
) -> String {
    use crate::models::dead_code::DeadCodeType;

    let summary = &result.summary;
    let other_items = result
        .files
        .iter()
        .flat_map(|f| f.items.iter())
        .filter(|item| matches!(item.item_type, DeadCodeType::Variable))
        .count();

    format!(
        "## Dead Code Breakdown\n\n\
         | Type | Count |\n\
         |------|-------|\n\
         | Functions | {} |\n\
         | Classes | {} |\n\
         | Modules | {} |\n\
         | Other (fields, constants, statics) | {} |\n\
         | Unreachable Blocks | {} |\n",
        summary.dead_functions,
        summary.dead_classes,
        summary.dead_modules,
        other_items,
        summary.unreachable_blocks
    )
}

fn format_dead_code_file_details_section(
    files: &[crate::models::dead_code::FileDeadCodeMetrics],
) -> String {
    let mut output = String::from(
        "## File Details\n\n\
         | File | Dead % | Dead Lines | Confidence | Items |\n\
         |------|--------|------------|------------|-------|\n",
    );

    for file in files.iter().take(20) {
        output.push_str(&format!(
            "| {} | {:.1}% | {} | {:?} | {} |\n",
            file.path,
            file.dead_percentage,
            file.dead_lines,
            file.confidence,
            file.items.len()
        ));
    }

    output
}

fn format_dead_code_recommendations_section() -> String {
    "## Recommendations\n\n\
     1. **Review High Confidence Dead Code**: Start with files marked as high confidence.\n\
     2. **Check Test Coverage**: Dead code often indicates missing tests.\n\
     3. **Consider Refactoring**: Large amounts of dead code may indicate design issues.\n\
     4. **Remove Carefully**: Ensure code is truly dead before removal.\n"
        .to_string()
}

/// Write dead code output to file or stdout
async fn write_dead_code_output(content: String, output: Option<PathBuf>) -> Result<()> {
    match output {
        Some(path) => {
            tokio::fs::write(&path, content).await?;
            eprintln!("📝 Results written to: {}", path.display());
        }
        None => {
            println!("{content}");
        }
    }
    Ok(())
}
