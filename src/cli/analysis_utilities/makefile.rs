// Makefile handlers - extracted for file health (CB-040)
/// Analyzes a Makefile for quality issues
///
/// # Errors
/// Returns an error if the Makefile cannot be read or analyzed
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_analyze_makefile(
    path: PathBuf,
    rules: Vec<String>,
    format: MakefileOutputFormat,
    fix: bool,
    gnu_version: Option<String>,
    top_files: usize,
) -> Result<()> {
    use crate::services::makefile_linter;

    crate::status_eprintln!("🔧 Analyzing Makefile...");

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
    let mut filtered_violations = filter_makefile_violations(&lint_result.violations, &rules);

    // --gnu-version used to reach nothing but the report header: a Makefile
    // built from `.ONESHELL:`, `::=` and `!=` produced byte-identical reports
    // for --gnu-version 3.0 and 4.4, while --help promised "GNU Make version to
    // check compatibility against". The linter itself takes no version, so the
    // compatibility check lives here, against the source it was pointed at.
    if let Some(ref requested) = gnu_version {
        let source = std::fs::read_to_string(&path)?;
        let mut incompat = check_gnu_version_compatibility(&source, requested)?;
        if rules.is_empty() || rules == vec!["all"] || rules.contains(&GNU_VERSION_RULE.to_string())
        {
            crate::status_eprintln!(
                "📌 {} construct(s) newer than GNU Make {requested}",
                incompat.len()
            );
            filtered_violations.append(&mut incompat);
        }
    }

    // Format output based on requested format
    let content = format_makefile_output(
        &path,
        &filtered_violations,
        &lint_result,
        gnu_version.as_ref(),
        format,
        top_files,
    )?;

    // Print output
    println!("{content}");

    // Handle fix mode if requested
    handle_makefile_fix_mode(fix, &filtered_violations);

    Ok(())
}

/// Rule name carried by every `--gnu-version` incompatibility.
const GNU_VERSION_RULE: &str = "gnuversion";

/// GNU Make constructs and the release that introduced them.
///
/// Deliberately short: each entry is a construct whose introducing release is
/// documented in the GNU Make NEWS file, recognised by a shape that cannot be
/// confused with a recipe's shell syntax. Guessing more would put fabricated
/// violations in the report, which is the failure mode this whole check exists
/// to correct.
const GNU_MAKE_FEATURES: &[(&str, &str)] = &[
    (".ONESHELL:", "3.82"),
    (".NOTINTERMEDIATE:", "4.4"),
    ("::=", "4.0"),
    ("!=", "4.0"),
    ("$(file ", "4.0"),
    ("$(intcmp ", "4.4"),
    ("$(let ", "4.4"),
];

/// Parse a `major.minor` GNU Make version.
fn parse_gnu_version(version: &str) -> Result<(u32, u32)> {
    let mut parts = version.trim().split('.');
    let major = parts
        .next()
        .and_then(|p| p.parse::<u32>().ok())
        .ok_or_else(|| anyhow::anyhow!("invalid --gnu-version '{version}': expected e.g. 4.4"))?;
    let minor = parts.next().unwrap_or("0").parse::<u32>().unwrap_or(0);
    Ok((major, minor))
}

/// Does `line` use `token` as Make syntax rather than as shell text?
///
/// Recipe lines are handed to the shell verbatim, where `!=` is a string
/// comparison and `::=` never appears; reporting those would be noise.
fn uses_make_construct(line: &str, token: &str) -> bool {
    if line.starts_with('\t') {
        return false;
    }
    let trimmed = line.trim_start();
    match token {
        "!=" | "::=" => match trimmed.find(token) {
            // Only a variable assignment: the whole left-hand side must be a
            // Make variable name.
            Some(pos) => {
                let lhs = trimmed.get(..pos).unwrap_or_default().trim_end();
                !lhs.is_empty()
                    && lhs
                        .chars()
                        .all(|c| c.is_alphanumeric() || c == '_' || c == '.')
            }
            None => false,
        },
        _ => trimmed.contains(token),
    }
}

/// Violations for constructs newer than the requested GNU Make version.
///
/// # Errors
/// Returns an error when `requested` is not a `major.minor` version.
fn check_gnu_version_compatibility(
    source: &str,
    requested: &str,
) -> Result<Vec<makefile_linter::Violation>> {
    use crate::services::makefile_linter::ast::SourceSpan;

    let target = parse_gnu_version(requested)?;
    let mut violations = Vec::new();

    for (idx, line) in source.lines().enumerate() {
        for (token, introduced_in) in GNU_MAKE_FEATURES {
            let needs = parse_gnu_version(introduced_in)?;
            if needs <= target || !uses_make_construct(line, token) {
                continue;
            }
            let column = line.find(token).unwrap_or(0) + 1;
            violations.push(makefile_linter::Violation {
                rule: GNU_VERSION_RULE.to_string(),
                severity: makefile_linter::Severity::Warning,
                span: SourceSpan {
                    start: 0,
                    end: 0,
                    line: idx + 1,
                    column,
                },
                message: format!(
                    "`{token}` requires GNU Make {introduced_in}, but compatibility was requested against {requested}"
                ),
                fix_hint: None,
            });
        }
    }

    Ok(violations)
}

// Helper: Print analysis summary
fn print_makefile_analysis_summary(lint_result: &makefile_linter::LintResult) {
    crate::status_eprintln!("📊 Found {} violations", lint_result.violations.len());
    crate::status_eprintln!(
        "✨ Quality score: {:.1}%",
        lint_result.quality_score * 100.0
    );
}

// Helper: Filter violations by rules
fn filter_makefile_violations(
    violations: &[makefile_linter::Violation],
    rules: &[String],
) -> Vec<makefile_linter::Violation> {
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
//
// This function does no I/O — it never has. It announced that it was applying
// automatic fixes and then reported five violations as having been fixed, over
// a Makefile whose md5 was identical before and after the run. Nothing here
// rewrites a Makefile yet, so nothing here may say it did; the wording now
// matches what the code actually does.
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

    eprintln!("\n🔧 Fixable violations (suggestions only — no file was modified):");
    let fix_count = fixable_violations.len();
    for violation in fixable_violations {
        if let Some(fix_hint) = &violation.fix_hint {
            eprintln!("  • {}: {}", violation.rule, fix_hint);
        }
    }
    eprintln!("💡 {fix_count} fixable violation(s); applying them automatically is not implemented, so the Makefile is unchanged.");
}

// Helper: Format makefile output based on format
//
// `--top-files` reached this function as `_top_files` and was dropped: every
// renderer below printed the whole violation list, so `--top-files 1` and
// `--top-files 50` produced byte-identical reports over a Makefile with 62
// violations. The limit is applied once, here, with the same authority the
// other listing surfaces use (`crate::cli::top_files_slice`, 0 = all), so no
// renderer can disagree with another about how many rows a limit permits.
// Aggregates (quality score, the "Found N violations" summary) stay whole:
// the flag bounds what is listed, never what is measured.
fn format_makefile_output(
    path: &Path,
    filtered_violations: &[makefile_linter::Violation],
    lint_result: &makefile_linter::LintResult,
    gnu_version: Option<&String>,
    format: MakefileOutputFormat,
    top_files: usize,
) -> Result<String> {
    let listed = crate::cli::top_files_slice(filtered_violations, top_files);
    let total = filtered_violations.len();
    match format {
        MakefileOutputFormat::Json => {
            format_makefile_as_json(path, listed, total, lint_result, gnu_version)
        }
        MakefileOutputFormat::Human => {
            format_makefile_as_human(path, listed, total, top_files, lint_result, gnu_version)
        }
        MakefileOutputFormat::Sarif => format_makefile_as_sarif(path, listed),
        MakefileOutputFormat::Gcc => format_makefile_as_gcc(path, listed),
    }
}

// Helper: Format as JSON
fn format_makefile_as_json(
    path: &Path,
    listed: &[makefile_linter::Violation],
    total: usize,
    lint_result: &makefile_linter::LintResult,
    gnu_version: Option<&String>,
) -> Result<String> {
    Ok(serde_json::to_string_pretty(&serde_json::json!({
        "path": path.display().to_string(),
        "violations": listed,
        "violations_total": total,
        "violations_listed": listed.len(),
        "violations_truncated": listed.len() < total,
        "quality_score": lint_result.quality_score,
        "gnu_version": gnu_version,
    }))?)
}

// Helper: Format as human-readable
fn format_makefile_as_human(
    path: &Path,
    listed: &[makefile_linter::Violation],
    total: usize,
    top_files: usize,
    lint_result: &makefile_linter::LintResult,
    gnu_version: Option<&String>,
) -> Result<String> {
    let mut output = String::new();

    write_makefile_human_header(&mut output, path, lint_result, gnu_version)?;
    write_makefile_violations_table(&mut output, listed, total, top_files)?;
    write_makefile_fix_suggestions(&mut output, listed)?;

    Ok(output)
}

// Helper: Write human format header
fn write_makefile_human_header(
    output: &mut String,
    path: &Path,
    lint_result: &makefile_linter::LintResult,
    gnu_version: Option<&String>,
) -> Result<()> {
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
    listed: &[makefile_linter::Violation],
    total: usize,
    top_files: usize,
) -> Result<()> {
    use std::fmt::Write;

    if listed.is_empty() {
        writeln!(output, "✅ No violations found!")?;
    } else {
        writeln!(output, "## Violations\n")?;
        writeln!(output, "| Line | Rule | Severity | Message |")?;
        writeln!(output, "|------|------|----------|---------|")?;

        for violation in listed {
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

        // A truncated table that does not say so reads as a clean bill of
        // health for the rows it hid; name both numbers, as the SATD and
        // proof-annotation surfaces do.
        if listed.len() < total {
            writeln!(
                output,
                "\n… {} more not shown (--top-files {top_files}, 0 = all)",
                total - listed.len()
            )?;
        }
    }
    Ok(())
}

// Helper: Get severity display string
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
/// Get severity display.
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
    // BTreeSet, not HashSet: with a HashSet the `rules` array came out in a
    // different order on every run, so two identical invocations produced
    // different SARIF bytes — enough to make a diff-based check read a change
    // where there was none.
    filtered_violations
        .iter()
        .map(|v| &v.rule)
        .collect::<std::collections::BTreeSet<_>>()
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
/// Get sarif level.
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
/// Get gcc level.
pub fn get_gcc_level(severity: &makefile_linter::Severity) -> &'static str {
    match severity {
        makefile_linter::Severity::Error => "error",
        makefile_linter::Severity::Warning => "warning",
        makefile_linter::Severity::Performance => "note",
        makefile_linter::Severity::Info => "note",
    }
}

#[cfg(test)]
mod makefile_gnu_version_tests {
    use super::*;

    /// The Makefile from the report: `.ONESHELL:` (3.82+), `::=` and `!=`
    /// (4.0+). Checking it against 3.0 and against 4.4 used to produce
    /// byte-identical reports because --gnu-version reached only the header.
    const MODERN: &str =
        ".ONESHELL:\nX ::= hello\nY != echo dynamic\n.PHONY: all\nall:\n\t@echo hi\n";

    #[test]
    fn test_old_gnu_version_reports_incompatible_constructs() {
        let v = check_gnu_version_compatibility(MODERN, "3.0").unwrap();
        let messages: Vec<&str> = v.iter().map(|x| x.message.as_str()).collect();
        assert_eq!(
            v.len(),
            3,
            "expected .ONESHELL:, ::= and != — got {messages:?}"
        );
        assert!(messages.iter().any(|m| m.contains(".ONESHELL:")));
        assert!(messages.iter().any(|m| m.contains("::=")));
        assert!(messages.iter().any(|m| m.contains("!=")));
        assert!(v.iter().all(|x| x.rule == GNU_VERSION_RULE));
    }

    #[test]
    fn test_new_gnu_version_reports_nothing() {
        assert!(check_gnu_version_compatibility(MODERN, "4.4")
            .unwrap()
            .is_empty());
    }

    #[test]
    fn test_gnu_version_boundaries() {
        // 3.81 predates .ONESHELL: (3.82); 4.0 covers ::= and != but not $(let).
        assert_eq!(
            check_gnu_version_compatibility(MODERN, "3.81")
                .unwrap()
                .len(),
            3
        );
        assert!(check_gnu_version_compatibility(MODERN, "4.0")
            .unwrap()
            .is_empty());
        let let_fn = "X = $(let a,1,$(a))\n";
        assert_eq!(
            check_gnu_version_compatibility(let_fn, "4.0")
                .unwrap()
                .len(),
            1
        );
        assert!(check_gnu_version_compatibility(let_fn, "4.4")
            .unwrap()
            .is_empty());
    }

    /// `!=` inside a recipe is shell string comparison, not Make's shell
    /// assignment; reporting it would be a fabricated violation.
    #[test]
    fn test_shell_inequality_in_a_recipe_is_not_a_make_construct() {
        let src = "all:\n\t@if [ \"$$a\" != \"$$b\" ]; then echo differ; fi\n";
        assert!(check_gnu_version_compatibility(src, "3.0")
            .unwrap()
            .is_empty());
    }

    #[test]
    fn test_bad_gnu_version_is_rejected() {
        assert!(check_gnu_version_compatibility(MODERN, "banana").is_err());
        assert_eq!(parse_gnu_version("4").unwrap(), (4, 0));
        assert_eq!(parse_gnu_version("3.81").unwrap(), (3, 81));
    }

    /// `--fix` must not claim to have modified a file it never opened.
    #[test]
    fn test_fix_mode_does_not_claim_to_have_written() {
        // The wording is the fix; assert the old claim is gone from the source
        // of truth the user sees.
        let violations = vec![makefile_linter::Violation {
            rule: "phonydeclared".to_string(),
            severity: makefile_linter::Severity::Warning,
            span: crate::services::makefile_linter::ast::SourceSpan {
                start: 0,
                end: 0,
                line: 1,
                column: 1,
            },
            message: "target 'a' is not declared .PHONY".to_string(),
            fix_hint: Some("Add 'a' to .PHONY declaration".to_string()),
        }];
        handle_makefile_fix_mode(true, &violations);
        handle_makefile_fix_mode(false, &violations);

        // Split so this probe cannot match its own source text.
        let claim = concat!("violations automatically", " fixed.");
        let source = include_str!("makefile.rs");
        assert!(
            !source.contains(claim),
            "--fix must not report violations as fixed while it writes no file"
        );
        assert!(source.contains("is not implemented, so the Makefile is unchanged"));
    }
}

/// `analyze makefile --top-files N` reached `_top_files` and was dropped: over a
/// Makefile with 62 violations, `--top-files 1` and `--top-files 50` produced
/// byte-identical reports in every format. These tests fail on that code.
#[cfg(test)]
mod makefile_top_files_tests {
    use super::*;

    fn violations(n: usize) -> Vec<makefile_linter::Violation> {
        (0..n)
            .map(|i| makefile_linter::Violation {
                rule: format!("rule{i}"),
                severity: makefile_linter::Severity::Warning,
                span: crate::services::makefile_linter::ast::SourceSpan {
                    start: 0,
                    end: 0,
                    line: i + 1,
                    column: 1,
                },
                message: format!("violation number {i}"),
                fix_hint: Some(format!("fix number {i}")),
            })
            .collect()
    }

    fn lint_result() -> makefile_linter::LintResult {
        makefile_linter::LintResult {
            path: PathBuf::from("Makefile"),
            violations: Vec::new(),
            quality_score: 0.5,
        }
    }

    fn render(
        v: &[makefile_linter::Violation],
        format: MakefileOutputFormat,
        top: usize,
    ) -> String {
        format_makefile_output(Path::new("Makefile"), v, &lint_result(), None, format, top)
            .expect("render")
    }

    #[test]
    fn the_limit_bites_in_every_format() {
        let v = violations(12);
        for format in [
            MakefileOutputFormat::Human,
            MakefileOutputFormat::Json,
            MakefileOutputFormat::Gcc,
            MakefileOutputFormat::Sarif,
        ] {
            let one = render(&v, format.clone(), 1);
            let fifty = render(&v, format.clone(), 50);
            assert_ne!(
                one, fifty,
                "--top-files 1 and 50 rendered identically for {format:?}"
            );
            assert!(
                one.contains("violation number 0") || one.contains("fix number 0"),
                "the one row kept must be the first: {one}"
            );
            assert!(
                !one.contains("violation number 11") && !one.contains("fix number 11"),
                "--top-files 1 still printed row 12 for {format:?}"
            );
        }
    }

    #[test]
    fn zero_means_every_row_not_none() {
        let v = violations(12);
        // `.take(0)` would render an empty list here; 0 is documented as "all".
        assert_eq!(render(&v, MakefileOutputFormat::Gcc, 0).lines().count(), 12);
        assert_eq!(
            render(&v, MakefileOutputFormat::Gcc, 0),
            render(&v, MakefileOutputFormat::Gcc, 50)
        );
    }

    #[test]
    fn a_truncated_table_says_what_it_hid() {
        let v = violations(12);
        let one = render(&v, MakefileOutputFormat::Human, 1);
        assert!(
            one.contains("11 more not shown (--top-files 1, 0 = all)"),
            "a capped table must name the rows it hid: {one}"
        );
        assert!(!render(&v, MakefileOutputFormat::Human, 50).contains("more not shown"));
    }

    #[test]
    fn json_reports_the_total_it_measured_not_only_the_rows_it_listed() {
        let v = violations(12);
        let parsed: serde_json::Value =
            serde_json::from_str(&render(&v, MakefileOutputFormat::Json, 3)).expect("json");
        assert_eq!(parsed["violations_total"], 12);
        assert_eq!(parsed["violations_listed"], 3);
        assert_eq!(parsed["violations_truncated"], true);
        assert_eq!(parsed["violations"].as_array().expect("array").len(), 3);
    }

    /// A SARIF document that reorders itself between two identical runs makes
    /// any diff-based check unreliable.
    #[test]
    fn sarif_rules_are_ordered_deterministically() {
        let v = violations(12);
        let first = render(&v, MakefileOutputFormat::Sarif, 0);
        for _ in 0..8 {
            assert_eq!(first, render(&v, MakefileOutputFormat::Sarif, 0));
        }
    }
}
