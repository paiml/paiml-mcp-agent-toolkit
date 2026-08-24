// Dead code output formatting - included from dead_code_handlers.rs
// NO `use` imports or `#!` inner attributes allowed here.

/// Format dead code result based on output format
fn format_dead_code_result(
    result: &crate::models::dead_code::DeadCodeResult,
    format: &DeadCodeOutputFormat,
    scope: DeadCodeReportScope,
) -> Result<String> {
    match format {
        DeadCodeOutputFormat::Json => format_dead_code_as_json_scoped(result, scope),
        DeadCodeOutputFormat::Sarif => format_dead_code_as_sarif(result),
        DeadCodeOutputFormat::Summary => format_dead_code_as_summary_scoped(result, scope),
        DeadCodeOutputFormat::Markdown => format_dead_code_as_markdown(result),
    }
}

/// Format result as JSON, with no analyzer scope information.
fn format_dead_code_as_json(result: &crate::models::dead_code::DeadCodeResult) -> Result<String> {
    format_dead_code_as_json_scoped(result, DeadCodeReportScope::default())
}

/// Format result as JSON, including what the report's own filters removed.
///
/// The bare `to_string_pretty(result)` this replaces published
/// `"files_with_dead_code": 0` and `"files": []` beside
/// `"files_with_dead_code_found": 1` — one object stating both that a file with
/// dead code was found and that none exists — while the text renderer beside it
/// spelled the omission out. JSON is the surface agents and CI read, so it is
/// the one that must not require a reader to already know about
/// `--min-dead-lines`. `omitted` is always present: "nothing was dropped" is a
/// claim worth being able to read, and an absent field is not one.
fn format_dead_code_as_json_scoped(
    result: &crate::models::dead_code::DeadCodeResult,
    scope: DeadCodeReportScope,
) -> Result<String> {
    let mut value = serde_json::to_value(result)?;
    let omitted = scope.omitted;
    if let Some(object) = value.as_object_mut() {
        // Issue #1058. This document names its counts `total_files` (what the
        // walk discovered) and `analyzed_files` (what the engine read). The MCP
        // payload for the SAME analysis named the second `files_analyzed` and
        // had no counterpart for the first, so asking both transports for
        // "dead-code files" got 38 here and 29 there — on copia, where the two
        // agree exactly at 29. `analyze complexity` already publishes the pair
        // as `files_analyzed` / `files_discovered`; both spellings now exist on
        // both surfaces, so one reader works on either. The original keys stay:
        // clients read them.
        object.insert(
            "files_analyzed".to_string(),
            serde_json::json!(result.analyzed_files),
        );
        object.insert(
            "files_discovered".to_string(),
            serde_json::json!(result.total_files),
        );
        object.insert(
            "omitted".to_string(),
            serde_json::json!({
                "files": omitted.files,
                "dead_lines": omitted.dead_lines,
                "dead_functions": omitted.dead_functions,
                "dead_classes": omitted.dead_classes,
                "dead_modules": omitted.dead_modules,
                "unreachable_blocks": omitted.unreachable_blocks,
                "reasons": omission_reasons(result, scope),
            }),
        );
    }
    Ok(serde_json::to_string_pretty(&value)?)
}

/// Which of the report's filters could account for what is missing from the
/// list.
///
/// Empty when nothing was omitted — a reason for an omission that did not happen
/// is itself a false claim. A file dropped by `--exclude 'src/**'` used to be
/// reported as "below --min-dead-lines", blaming a threshold that had nothing to
/// do with it.
fn omission_reasons(
    result: &crate::models::dead_code::DeadCodeResult,
    scope: DeadCodeReportScope,
) -> Vec<&'static str> {
    if scope.omitted.is_empty() && result.files_omitted() == 0 {
        return Vec::new();
    }
    let mut reasons = vec!["below --min-dead-lines"];
    if result.files_truncated {
        reasons.push("beyond --top-files");
    }
    if scope.list_filtered {
        reasons.push("removed by --include/--exclude");
    }
    reasons
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
            // The library verdict, at RUN level: it is a property of the
            // analysis, not of any one result, and a SARIF consumer that cannot
            // read it cannot tell a library's exported API from dead code. This
            // is the format a CI pipeline ingests, so the caveat has to survive
            // the conversion. `null` when the analyzer did not record one.
            "properties": {
                "libraryTarget": result.library_target,
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
                                    DeadCodeType::Module => "Dead module",
                                    DeadCodeType::UnreachableCode => "Unreachable code",
                                    DeadCodeType::Other => "Dead item",
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

/// Format result as summary, with no analyzer scope information.
///
/// Every figure the renderer cannot verify is then reported as unknown rather
/// than guessed: no project-wide percentage, no name for the skipped files.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_dead_code_as_summary(
    result: &crate::models::dead_code::DeadCodeResult,
) -> Result<String> {
    format_dead_code_as_summary_scoped(result, DeadCodeReportScope::default())
}

/// Format result as summary, told what the analyzer could and could not measure.
fn format_dead_code_as_summary_scoped(
    result: &crate::models::dead_code::DeadCodeResult,
    scope: DeadCodeReportScope,
) -> Result<String> {
    let mut output = String::new();

    write_dead_code_header(&mut output, result, scope)?;

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
    scope: DeadCodeReportScope,
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
    //
    // The parenthetical comes from the analyzer that did the skipping. It used
    // to be one hardcoded phrase naming the test tree AND the example and
    // benchmark trees, neither of which the cargo scan skips -- and the Top
    // Files list directly beneath it was made of `examples/`.
    if result.total_files > result.analyzed_files {
        writeln!(
            output,
            "  {} {} ({})",
            c::label("Files skipped (out of scope):"),
            c::number(&(result.total_files - result.analyzed_files).to_string()),
            scope
                .skipped_kind
                .unwrap_or("the analyzer did not say which files")
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
    // `--include`/`--exclude` filter the REPORT, not the walk. Saying so is the
    // difference between "Files analyzed: 2" being a fact about the scan and it
    // reading as a claim that the filter narrowed the scan -- which it does not:
    // `--include 'examples/**'` on a two-file crate still analyzes both files
    // and still divides by the whole project's lines.
    if scope.list_filtered {
        writeln!(
            output,
            "  {} --include/--exclude filter this report, not the scan; \
             the counts above and the percentage below cover every scanned file",
            c::label("Note:")
        )?;
    }
    let omitted = result.files_omitted();
    if omitted > 0 {
        // Name every cut that could have removed them, and what went with them.
        // The file count alone left the categories below reading as measurements
        // of the project: "Dead functions: 0" over three dead functions the
        // threshold had just cut is the same contradiction #928 fixed inside the
        // list, one level up.
        writeln!(
            output,
            "  {} {} ({} not listed{}: {})",
            c::label("Files found with dead code:"),
            c::number(&result.files_with_dead_code_found.to_string()),
            c::number(&omitted.to_string()),
            omitted_items_note(scope.omitted),
            omission_reasons(result, scope).join(" or ")
        )?;
    }
    writeln!(
        output,
        "  {} {}",
        c::label("Total dead lines:"),
        c::number(&result.summary.total_dead_lines.to_string())
    )?;
    // Named for its scope. Both this and the figure `--fail-on-violation`
    // compares are real, but they measure different sets — this one covers the
    // files actually LISTED (which `--min-dead-lines` and `--top-files` shrink),
    // the gate's covers every line walked. Printing this one as plain "Dead code
    // percentage" made them look like one number disagreeing with itself: a run
    // could report 0.0% here while the gate failed the same run at 100%.
    //
    // When there is NO project-wide figure at all — the multi-language analyzer
    // never counts total project lines — the note says so. It used to print a
    // bare "Dead code percentage: 100.0%" and then, in the same run,
    // `--fail-on-violation` bailed with "no project-wide dead-code percentage
    // was measured for this project": the report both stated a measurement and
    // denied making one.
    let scope_note = match scope.project_dead_percentage {
        Some(project) if result.files_omitted() > 0 || scope.list_filtered => {
            format!(" (listed files only; project-wide: {project:.1}%)")
        }
        Some(_) => String::new(),
        None => {
            " (listed files only; no project-wide figure was measured for this project)".to_string()
        }
    };
    writeln!(
        output,
        "  {}{} {}\n",
        c::label("Dead code percentage:"),
        c::dim(&scope_note),
        c::pct(f64::from(result.summary.dead_percentage), 5.0, 15.0)
    )?;

    write_library_target_line(output, result)?;

    Ok(())
}

/// State whether the analyzer decided this target was a library — the decision
/// that says whether an un-called export is above or below the line.
///
/// The `undetermined` verdict is the one that has to be readable: it means the
/// list DOES contain exported items, listed as dead because nothing calls them
/// rather than because they are known to be unreachable. The percentage printed
/// directly above it was "100.0%" over a Python package's entire public API.
fn write_library_target_line(
    output: &mut String,
    result: &crate::models::dead_code::DeadCodeResult,
) -> Result<()> {
    use crate::cli::colors as c;
    use std::fmt::Write;

    let Some(library) = &result.library_target else {
        return Ok(());
    };
    writeln!(
        output,
        "  {} {} — {}",
        c::label("Library target:"),
        library.verdict,
        c::dim(&library.detail)
    )?;
    if let Some(roots) = library.exported_roots {
        if roots > 0 {
            writeln!(
                output,
                "  {} {} (exported items kept as entry points, not listed below)",
                c::label("Exported roots:"),
                c::number(&roots.to_string())
            )?;
        }
    }
    writeln!(output)?;

    Ok(())
}

/// What the omitted files took with them, in the categories the breakdown
/// section prints, so the two can be read against each other.
///
/// Empty when the caller has no item-level figures to offer (the renderer is
/// public and can be handed a default scope) rather than printing a zero it did
/// not measure.
fn omitted_items_note(omitted: DeadCodeFindingTotals) -> String {
    // Both spellings are written out: "class" does not pluralise by appending
    // an "s", and a report that prints "2 dead classs" undermines the figure
    // beside it.
    let counted = [
        ("dead function", "dead functions", omitted.dead_functions),
        ("dead class", "dead classes", omitted.dead_classes),
        ("dead module", "dead modules", omitted.dead_modules),
        (
            "unreachable block",
            "unreachable blocks",
            omitted.unreachable_blocks,
        ),
    ];
    let parts: Vec<String> = counted
        .iter()
        .filter(|(_, _, n)| *n > 0)
        .map(|(one, many, n)| {
            let label = if *n == 1 { one } else { many };
            format!("{n} {label}")
        })
        .collect();
    if parts.is_empty() {
        String::new()
    } else {
        format!(", holding {}", parts.join(", "))
    }
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
    use std::fmt::Write;

    let summary = &result.summary;
    let other_items = count_other_items(result);

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

/// The items that belong in the "Other" row: bindings (fields, constants,
/// statics, variants) and anything whose kind the producer could not name.
///
/// ONE implementation, called by both renderers. #928: this predicate used to
/// be written out twice as `matches!(item.item_type, DeadCodeType::Variable)`,
/// and because a dead MODULE was also typed `Variable` back then, every module
/// was counted in BOTH the "Dead modules" row and this one — the two rows
/// summed to more items than the report listed.
fn count_other_items(result: &crate::models::dead_code::DeadCodeResult) -> usize {
    use crate::models::dead_code::DeadCodeType;

    result
        .files
        .iter()
        .flat_map(|f| f.items.iter())
        .filter(|item| matches!(item.item_type, DeadCodeType::Variable | DeadCodeType::Other))
        .count()
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
    // Every row below the first two is scoped to the LISTED files. When the
    // report's filters removed files, saying so is the difference between
    // "Files with Dead Code | 0" being a measurement and it being a filter
    // artefact that reads as a clean bill of health.
    let omitted = result.files_omitted();
    let omission_row = if omitted > 0 {
        format!(
            "| Files Found with Dead Code | {} |\n\
             | Files Not Listed (filtered) | {} |\n",
            result.files_with_dead_code_found, omitted
        )
    } else {
        String::new()
    };

    let library_row = markdown_library_row(result);

    format!(
        "# Dead Code Analysis Report\n\n\
         ## Summary\n\n\
         | Metric | Value |\n\
         |--------|-------|\n\
         | Files Analyzed | {} |\n\
         | Files Skipped (out of scope) | {} |\n\
         | Files with Dead Code | {} |\n\
         {omission_row}\
         | Total Dead Lines | {} |\n\
         | Dead Code Percentage | {:.2}% |\n\
         {library_row}",
        result.analyzed_files,
        result.total_files.saturating_sub(result.analyzed_files),
        result.summary.files_with_dead_code,
        result.summary.total_dead_lines,
        result.summary.dead_percentage
    )
}

/// The library verdict as a markdown table row, empty when the analyzer
/// recorded none.
///
/// It sits directly under the percentage because that is the figure it
/// qualifies: "100.00%" over a library's whole public API is a different
/// statement from "100.00%" over three abandoned helpers, and the table gave a
/// reader no way to tell which they were looking at.
fn markdown_library_row(result: &crate::models::dead_code::DeadCodeResult) -> String {
    match &result.library_target {
        Some(library) => format!(
            "| Library Target | {} — {} |\n",
            library.verdict,
            // Pipes would break the row; the detail is prose and contains none
            // today, but a future reason must not be able to corrupt the table.
            library.detail.replace('|', "\\|")
        ),
        None => String::new(),
    }
}

/// Write the markdown breakdown table.
///
/// The `Modules` row prints `summary.dead_modules`, which is a MODULE count on
/// the cargo path. It was labelled `Variables` here -- the same mislabel the
/// text renderer carried (#721) -- so a cargo run reported its dead modules
/// under a row heading no producer fills. Fields, constants and statics are
/// counted from the items themselves, exactly as the text renderer does, so
/// every reported dead item lands in one row.
fn format_dead_code_breakdown_section(result: &crate::models::dead_code::DeadCodeResult) -> String {
    let summary = &result.summary;
    let other_items = count_other_items(result);

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
            crate::status_eprintln!("📝 Results written to: {}", path.display());
        }
        None => {
            println!("{content}");
        }
    }
    Ok(())
}
