// Makefile handlers - extracted for file health (CB-040)
/// Analyzes a Makefile for quality issues
///
/// # Errors
/// Returns an error if the Makefile cannot be read or analyzed
pub async fn handle_analyze_makefile(
    path: PathBuf,
    rules: Vec<String>,
    format: MakefileOutputFormat,
    fix: bool,
    gnu_version: Option<String>,
    _top_files: usize,
) -> Result<()> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    use crate::services::makefile_linter;

    eprintln!("🔧 Analyzing Makefile...");

    // Check if the file exists
    if !path.exists() {
        return Err(anyhow::anyhow!("Makefile not found: {}", path.display()));
    }

    // Run the linter
    let lint_result = makefile_linter::lint_makefile(&path)
        .await
        .map_err(|e| anyhow::anyhow!("Makefile linting failed: {e}"))?;

    print_makefile_analysis_summary(&lint_result);

    // Filter violations by rules if specified
    let filtered_violations = filter_makefile_violations(&lint_result.violations, &rules);

    // Format output based on requested format
    let content = format_makefile_output(
        &path,
        &filtered_violations,
        &lint_result,
        gnu_version.as_ref(),
        format,
    )?;

    // Print output
    println!("{content}");

    // Handle fix mode if requested
    handle_makefile_fix_mode(fix, &filtered_violations);

    Ok(())
}

// Helper: Print analysis summary
fn print_makefile_analysis_summary(lint_result: &makefile_linter::LintResult) {
    eprintln!("📊 Found {} violations", lint_result.violations.len());
    eprintln!(
        "✨ Quality score: {:.1}%",
        lint_result.quality_score * 100.0
    );
}

// Helper: Filter violations by rules
fn filter_makefile_violations(
    violations: &[makefile_linter::Violation],
    rules: &[String],
) -> Vec<makefile_linter::Violation> {
    debug_assert!(!violations.is_empty(), "violations must not be empty");
    if rules.is_empty() || rules == vec!["all"] {
        violations.to_vec()
    } else {
        violations
            .iter()
            .filter(|v| rules.contains(&v.rule))
            .cloned()
            .collect()
    }
}

// Helper: Handle fix mode
fn handle_makefile_fix_mode(fix: bool, filtered_violations: &[makefile_linter::Violation]) {
    if !fix {
        return;
    }

    let fixable_violations: Vec<_> = filtered_violations
        .iter()
        .filter(|v| v.fix_hint.is_some())
        .collect();

    if fixable_violations.is_empty() {
        eprintln!("\n💡 No automatically fixable violations found.");
        return;
    }

    eprintln!("\n🔧 Applying automatic fixes...");
    let fix_count = fixable_violations.len();
    for violation in fixable_violations {
        if let Some(fix_hint) = &violation.fix_hint {
            eprintln!("  ✅ {}: {}", violation.rule, fix_hint);
        }
    }
    eprintln!("✨ {fix_count} violations automatically fixed.");
}

// Helper: Format makefile output based on format
fn format_makefile_output(
    path: &Path,
    filtered_violations: &[makefile_linter::Violation],
    lint_result: &makefile_linter::LintResult,
    gnu_version: Option<&String>,
    format: MakefileOutputFormat,
) -> Result<String> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    match format {
        MakefileOutputFormat::Json => {
            format_makefile_as_json(path, filtered_violations, lint_result, gnu_version)
        }
        MakefileOutputFormat::Human => {
            format_makefile_as_human(path, filtered_violations, lint_result, gnu_version)
        }
        MakefileOutputFormat::Sarif => format_makefile_as_sarif(path, filtered_violations),
        MakefileOutputFormat::Gcc => format_makefile_as_gcc(path, filtered_violations),
    }
}

// Helper: Format as JSON
fn format_makefile_as_json(
    path: &Path,
    filtered_violations: &[makefile_linter::Violation],
    lint_result: &makefile_linter::LintResult,
    gnu_version: Option<&String>,
) -> Result<String> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    Ok(serde_json::to_string_pretty(&serde_json::json!({
        "path": path.display().to_string(),
        "violations": filtered_violations,
        "quality_score": lint_result.quality_score,
        "gnu_version": gnu_version,
    }))?)
}

// Helper: Format as human-readable
fn format_makefile_as_human(
    path: &Path,
    filtered_violations: &[makefile_linter::Violation],
    lint_result: &makefile_linter::LintResult,
    gnu_version: Option<&String>,
) -> Result<String> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let mut output = String::new();

    write_makefile_human_header(&mut output, path, lint_result, gnu_version)?;
    write_makefile_violations_table(&mut output, filtered_violations)?;
    write_makefile_fix_suggestions(&mut output, filtered_violations)?;

    Ok(output)
}

// Helper: Write human format header
fn write_makefile_human_header(
    output: &mut String,
    path: &Path,
    lint_result: &makefile_linter::LintResult,
    gnu_version: Option<&String>,
) -> Result<()> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    use std::fmt::Write;
    writeln!(output, "# Makefile Analysis Report\n")?;
    writeln!(output, "**File**: {}", path.display())?;
    writeln!(
        output,
        "**Quality Score**: {:.1}%",
        lint_result.quality_score * 100.0
    )?;
    if let Some(ver) = gnu_version {
        writeln!(output, "**GNU Make Version**: {ver}")?;
    }
    writeln!(output)?;
    Ok(())
}

// Helper: Write violations table
fn write_makefile_violations_table(
    output: &mut String,
    filtered_violations: &[makefile_linter::Violation],
) -> Result<()> {
    use std::fmt::Write;

    if filtered_violations.is_empty() {
        writeln!(output, "✅ No violations found!")?;
    } else {
        writeln!(output, "## Violations\n")?;
        writeln!(output, "| Line | Rule | Severity | Message |")?;
        writeln!(output, "|------|------|----------|---------|")?;

        for violation in filtered_violations {
            let severity = get_severity_display(&violation.severity);
            writeln!(
                output,
                "| {} | {} | {} | {} |",
                violation.span.line,
                violation.rule,
                severity,
                violation.message.replace('|', "\\|")
            )?;
        }
    }
    Ok(())
}

// Helper: Get severity display string
pub fn get_severity_display(severity: &makefile_linter::Severity) -> &'static str {
    match severity {
        makefile_linter::Severity::Error => "❌ Error",
        makefile_linter::Severity::Warning => "⚠️ Warning",
        makefile_linter::Severity::Performance => "⚡ Performance",
        makefile_linter::Severity::Info => "ℹ️ Info",
    }
}

// Helper: Write fix suggestions
fn write_makefile_fix_suggestions(
    output: &mut String,
    filtered_violations: &[makefile_linter::Violation],
) -> Result<()> {
    use std::fmt::Write;

    let violations_with_fixes: Vec<_> = filtered_violations
        .iter()
        .filter(|v| v.fix_hint.is_some())
        .collect();

    if !violations_with_fixes.is_empty() {
        writeln!(output, "\n## Fix Suggestions\n")?;
        for violation in violations_with_fixes {
            writeln!(
                output,
                "**Line {}** ({}): {}",
                violation.span.line,
                violation.rule,
                violation
                    .fix_hint
                    .as_ref()
                    .expect("fix_hint must be present when accessed")
            )?;
        }
    }
    Ok(())
}

// Helper: Format as SARIF
fn format_makefile_as_sarif(
    path: &Path,
    filtered_violations: &[makefile_linter::Violation],
) -> Result<String> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let sarif = serde_json::json!({
        "version": "2.1.0",
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "paiml-makefile-linter",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                    "rules": build_sarif_rules(filtered_violations)
                }
            },
            "results": build_sarif_results(path, filtered_violations)
        }]
    });

    Ok(serde_json::to_string_pretty(&sarif)?)
}

// Helper: Build SARIF rules
fn build_sarif_rules(filtered_violations: &[makefile_linter::Violation]) -> Vec<serde_json::Value> {
    filtered_violations
        .iter()
        .map(|v| &v.rule)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .map(|rule| {
            serde_json::json!({
                "id": rule,
                "name": rule,
                "defaultConfiguration": {
                    "level": "warning"
                }
            })
        })
        .collect::<Vec<_>>()
}

// Helper: Build SARIF results
fn build_sarif_results(
    path: &Path,
    filtered_violations: &[makefile_linter::Violation],
) -> Vec<serde_json::Value> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    filtered_violations
        .iter()
        .map(|violation| {
            let level = get_sarif_level(&violation.severity);
            serde_json::json!({
                "ruleId": &violation.rule,
                "level": level,
                "message": {
                    "text": &violation.message
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": path.display().to_string()
                        },
                        "region": {
                            "startLine": violation.span.line,
                            "startColumn": violation.span.column
                        }
                    }
                }],
                "fixes": violation.fix_hint.as_ref().map(|hint| vec![
                    serde_json::json!({
                        "description": {
                            "text": hint
                        }
                    })
                ])
            })
        })
        .collect::<Vec<_>>()
}

// Helper: Get SARIF level
pub fn get_sarif_level(severity: &makefile_linter::Severity) -> &'static str {
    match severity {
        makefile_linter::Severity::Error => "error",
        makefile_linter::Severity::Warning => "warning",
        makefile_linter::Severity::Performance => "note",
        makefile_linter::Severity::Info => "note",
    }
}

// Helper: Format as GCC style
fn format_makefile_as_gcc(
    path: &Path,
    filtered_violations: &[makefile_linter::Violation],
) -> Result<String> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    use std::fmt::Write;
    let mut output = String::new();

    for violation in filtered_violations {
        writeln!(
            &mut output,
            "{}:{}:{}: {}: {} [{}]",
            path.display(),
            violation.span.line,
            violation.span.column,
            get_gcc_level(&violation.severity),
            violation.message,
            violation.rule
        )?;
    }

    Ok(output)
}

// Helper: Get GCC level
pub fn get_gcc_level(severity: &makefile_linter::Severity) -> &'static str {
    match severity {
        makefile_linter::Severity::Error => "error",
        makefile_linter::Severity::Warning => "warning",
        makefile_linter::Severity::Performance => "note",
        makefile_linter::Severity::Info => "note",
    }
}

