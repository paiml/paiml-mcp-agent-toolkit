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
pub fn format_dead_code_as_summary(
    result: &crate::models::dead_code::DeadCodeResult,
) -> Result<String> {
    let mut output = String::new();

    write_dead_code_header(&mut output, result)?;

    if result.summary.dead_functions > 0 {
        write_dead_code_by_type_section(&mut output, &result.summary)?;
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
        c::number(&result.total_files.to_string())
    )?;
    writeln!(
        output,
        "  {} {}",
        c::label("Files with dead code:"),
        c::number(&result.summary.files_with_dead_code.to_string())
    )?;
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

/// Write dead code by type breakdown section
fn write_dead_code_by_type_section(
    output: &mut String,
    summary: &crate::models::dead_code::DeadCodeSummary,
) -> Result<()> {
    use crate::cli::colors as c;
    use std::fmt::Write;

    writeln!(output, "{}\n", c::subheader("Dead Code by Type"))?;
    writeln!(
        output,
        "  {} {}",
        c::label("Dead functions:"),
        c::number(&summary.dead_functions.to_string())
    )?;
    writeln!(
        output,
        "  {} {}",
        c::label("Dead classes:"),
        c::number(&summary.dead_classes.to_string())
    )?;
    writeln!(
        output,
        "  {} {}",
        c::label("Dead variables:"),
        c::number(&summary.dead_modules.to_string())
    )?;
    writeln!(
        output,
        "  {} {}",
        c::label("Unreachable blocks:"),
        c::number(&summary.unreachable_blocks.to_string())
    )?;

    Ok(())
}

/// Write top files with dead code section
fn write_top_files_section(
    output: &mut String,
    files: &[crate::models::dead_code::FileDeadCodeMetrics],
) -> Result<()> {
    debug_assert!(!files.is_empty(), "files must not be empty");
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
        sections.push(format_dead_code_breakdown_section(&result.summary));
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
         | Files with Dead Code | {} |\n\
         | Total Dead Lines | {} |\n\
         | Dead Code Percentage | {:.2}% |\n",
        result.total_files,
        result.summary.files_with_dead_code,
        result.summary.total_dead_lines,
        result.summary.dead_percentage
    )
}

fn format_dead_code_breakdown_section(
    summary: &crate::models::dead_code::DeadCodeSummary,
) -> String {
    format!(
        "## Dead Code Breakdown\n\n\
         | Type | Count |\n\
         |------|-------|\n\
         | Functions | {} |\n\
         | Classes | {} |\n\
         | Variables | {} |\n\
         | Unreachable Blocks | {} |\n",
        summary.dead_functions,
        summary.dead_classes,
        summary.dead_modules,
        summary.unreachable_blocks
    )
}

fn format_dead_code_file_details_section(
    files: &[crate::models::dead_code::FileDeadCodeMetrics],
) -> String {
    debug_assert!(!files.is_empty(), "files must not be empty");
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
