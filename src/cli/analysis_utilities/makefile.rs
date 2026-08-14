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

    validate_rule_names(&rules)?;

    // Run the linter
    let lint_result = makefile_linter::lint_makefile(&path)
        .await
        .map_err(|e| anyhow::anyhow!("Makefile linting failed: {e}"))?;

    let mut measured = lint_result.violations.clone();

    // --gnu-version used to reach nothing but the report header: a Makefile
    // built from `.ONESHELL:`, `::=` and `!=` produced byte-identical reports
    // for --gnu-version 3.0 and 4.4, while --help promised "GNU Make version to
    // check compatibility against". The linter itself takes no version, so the
    // compatibility check lives here, against the source it was pointed at.
    if let Some(ref requested) = gnu_version {
        let source = std::fs::read_to_string(&path)?;
        let mut incompat = check_gnu_version_compatibility(&source, requested)?;
        crate::status_eprintln!(
            "📌 {} construct(s) newer than GNU Make {requested}",
            incompat.len()
        );
        measured.append(&mut incompat);
    }

    // One filter, applied once, and its arithmetic carried to every surface —
    // stdout said "✅ No violations found!" while stderr of the same process
    // said "Found 3 violations" and the header kept the 50.0% score derived
    // from them, because the summary was printed from the unfiltered list and
    // the report from the filtered one.
    let outcome = apply_rule_filter(&measured, &rules);
    print_makefile_analysis_summary(&lint_result, &outcome);

    // Format output based on requested format
    let content = format_makefile_output(
        &path,
        &outcome,
        &lint_result,
        gnu_version.as_ref(),
        format,
        top_files,
    )?;

    // Print output
    println!("{content}");

    // Handle fix mode if requested
    handle_makefile_fix_mode(fix, &outcome.kept);

    Ok(())
}

/// Reject `--rules` values that cannot name a rule.
///
/// `--rules ''` was accepted, matched nothing, and produced a report that said
/// "✅ No violations found!" over a Makefile with a shell-injection Error.
///
/// #961 residual: a non-blank but UNREGISTERED name (`--rules
/// nonexistent-rule-xyz`) was accepted just as silently — it filtered every
/// measured violation away and exited 0. The valid set is now read from the
/// registry that actually runs the rules (`RuleRegistry::rule_ids`), so
/// "no such rule" is an error while "a real rule that found nothing" stays a
/// clean, zero-violation success. Deriving the set from the registry rather
/// than from a second hand-written list is deliberate: a list maintained here
/// would drift the moment a rule is registered.
///
/// # Errors
/// Returns an error when any requested rule name is blank or unregistered.
fn validate_rule_names(rules: &[String]) -> Result<()> {
    if rules.iter().any(|r| r.trim().is_empty()) {
        return Err(anyhow::anyhow!(
            "--rules contains an empty rule name; pass rule ids (e.g. \
             security/shell-injection, undefinedvariable) or omit --rules for all rules"
        ));
    }

    // Only names that will actually filter are checked. With `all` present
    // nothing is filtered, so no name can remove anything and there is no
    // silent loss to protect against; the residual is that `--rules all,typo`
    // does not flag the typo.
    if rules.iter().any(|r| r.trim() == "all") {
        return Ok(());
    }

    let valid = valid_rule_names();
    let unknown: Vec<&str> = rules
        .iter()
        .map(|r| r.trim())
        .filter(|r| !valid.iter().any(|v| v == r))
        .collect();
    if !unknown.is_empty() {
        return Err(anyhow::anyhow!(
            "--rules names no such rule: {}. Valid rule ids are: {}. \
             Omit --rules (or pass `all`) to run every rule.",
            unknown.join(", "),
            valid.join(", ")
        ));
    }
    Ok(())
}

/// Every value `--rules` accepts: the registered rule ids, plus the two names
/// that are real but do not come from the registry.
fn valid_rule_names() -> Vec<String> {
    let registry = makefile_linter::RuleRegistry::new();
    let mut names: Vec<String> = registry
        .rule_ids()
        .into_iter()
        .map(ToString::to_string)
        .collect();
    // `gnuversion` is emitted here, by --gnu-version, not by a registered rule.
    names.push(GNU_VERSION_RULE.to_string());
    // `all` is the documented "no filter" spelling handled by apply_rule_filter.
    names.push("all".to_string());
    names.sort();
    names.dedup();
    names
}

/// The result of applying `--rules`, kept whole so that no surface can report a
/// count another surface contradicts.
struct RuleFilterOutcome {
    /// Violations that survived the filter — what every renderer lists.
    kept: Vec<makefile_linter::Violation>,
    /// How many violations were measured before filtering.
    measured: usize,
    /// The requested rule names, empty when no filter was applied.
    filter: Vec<String>,
    /// Rule ids that actually produced a violation, sorted and deduplicated.
    rules_present: Vec<String>,
}

impl RuleFilterOutcome {
    /// A filter is in force and it removed every measured violation.
    fn filtered_everything_away(&self) -> bool {
        !self.filter.is_empty() && self.kept.is_empty() && self.measured > 0
    }

    fn filter_display(&self) -> String {
        self.filter.join(",")
    }
}

/// Apply `--rules`, recording what was measured as well as what survived.
fn apply_rule_filter(
    violations: &[makefile_linter::Violation],
    rules: &[String],
) -> RuleFilterOutcome {
    // `all` means "no filter" wherever it appears, not only when it is the only
    // entry: `--rules all,minphony` used to be read as "only minphony", which
    // silently narrowed a request that literally asks for everything.
    let no_filter = rules.is_empty() || rules.iter().any(|r| r.trim() == "all");
    let kept = if no_filter {
        violations.to_vec()
    } else {
        violations
            .iter()
            .filter(|v| rules.contains(&v.rule))
            .cloned()
            .collect()
    };
    let rules_present: Vec<String> = violations
        .iter()
        .map(|v| v.rule.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();

    RuleFilterOutcome {
        kept,
        measured: violations.len(),
        filter: if no_filter { Vec::new() } else { rules.to_vec() },
        rules_present,
    }
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
//
// The count here is the one the report lists; the count that was measured is
// named alongside it whenever a filter changed them, so stderr and stdout state
// the same two facts.
fn print_makefile_analysis_summary(
    lint_result: &makefile_linter::LintResult,
    outcome: &RuleFilterOutcome,
) {
    if outcome.filter.is_empty() {
        crate::status_eprintln!("📊 Found {} violations", outcome.measured);
    } else {
        crate::status_eprintln!(
            "📊 Found {} violations; {} match --rules {}",
            outcome.measured,
            outcome.kept.len(),
            outcome.filter_display()
        );
    }
    crate::status_eprintln!(
        "✨ Quality score: {:.1}% ({})",
        lint_result.quality_score * 100.0,
        score_scope(lint_result, outcome)
    );
}

// What the quality score was computed over, in words.
//
// #961 residual, same shape as the one that issue is about: the score comes
// from `LintResult::quality_score`, which the linter derives from ITS OWN
// violations, but the caption said "over all {measured} violation(s) found" —
// and `measured` also counts the incompatibilities `--gnu-version` appends
// afterwards. `--gnu-version 3.0` over a Makefile using `.ONESHELL:` and `::=`
// therefore printed "Quality score: 100.0% (over all 2 violations found)": a
// perfect score attributed to a set of two findings it was not computed over.
// A number is only as honest as the set it names, so the caption now names the
// set the score actually covers and reports the appended findings separately.
fn score_scope(
    lint_result: &makefile_linter::LintResult,
    outcome: &RuleFilterOutcome,
) -> String {
    let scored = lint_result.violations.len();
    let appended = outcome.measured.saturating_sub(scored);
    if appended == 0 {
        // The common case keeps its original wording: nothing was appended, so
        // "all N" is exactly what the score covers.
        return format!("over all {scored} violation(s) found");
    }
    format!(
        "over the {scored} violation(s) the linter measured; {appended} further \
         --gnu-version finding(s) are not in the score"
    )
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
    outcome: &RuleFilterOutcome,
    lint_result: &makefile_linter::LintResult,
    gnu_version: Option<&String>,
    format: MakefileOutputFormat,
    top_files: usize,
) -> Result<String> {
    let listed = crate::cli::top_files_slice(&outcome.kept, top_files);
    let total = outcome.kept.len();
    match format {
        MakefileOutputFormat::Json => {
            format_makefile_as_json(path, listed, total, outcome, lint_result, gnu_version)
        }
        MakefileOutputFormat::Human => format_makefile_as_human(
            path,
            listed,
            total,
            top_files,
            outcome,
            lint_result,
            gnu_version,
        ),
        MakefileOutputFormat::Sarif => format_makefile_as_sarif(path, listed),
        MakefileOutputFormat::Gcc => format_makefile_as_gcc(path, listed),
    }
}

// Helper: Format as JSON
fn format_makefile_as_json(
    path: &Path,
    listed: &[makefile_linter::Violation],
    total: usize,
    outcome: &RuleFilterOutcome,
    lint_result: &makefile_linter::LintResult,
    gnu_version: Option<&String>,
) -> Result<String> {
    Ok(serde_json::to_string_pretty(&serde_json::json!({
        "path": path.display().to_string(),
        "violations": listed,
        "violations_total": total,
        "violations_listed": listed.len(),
        "violations_truncated": listed.len() < total,
        // What was measured, before --rules removed anything: a 200-byte
        // document with an empty violations array used to be the only trace of
        // a run that found a shell-injection Error.
        "violations_measured": outcome.measured,
        "rules_filter": outcome.filter,
        "rules_present": outcome.rules_present,
        "quality_score": lint_result.quality_score,
        "gnu_version": gnu_version,
    }))?)
}

// Helper: Format as human-readable
#[allow(clippy::too_many_arguments)]
fn format_makefile_as_human(
    path: &Path,
    listed: &[makefile_linter::Violation],
    total: usize,
    top_files: usize,
    outcome: &RuleFilterOutcome,
    lint_result: &makefile_linter::LintResult,
    gnu_version: Option<&String>,
) -> Result<String> {
    let mut output = String::new();

    write_makefile_human_header(&mut output, path, lint_result, gnu_version, outcome)?;
    write_makefile_violations_table(&mut output, listed, total, top_files, outcome)?;
    write_makefile_fix_suggestions(&mut output, listed)?;

    Ok(output)
}

// Helper: Write human format header
fn write_makefile_human_header(
    output: &mut String,
    path: &Path,
    lint_result: &makefile_linter::LintResult,
    gnu_version: Option<&String>,
    outcome: &RuleFilterOutcome,
) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "# Makefile Analysis Report\n")?;
    writeln!(output, "**File**: {}", path.display())?;
    // The score is derived from every violation the linter found, so it must
    // say so when the table below shows a subset — a 50.0% score printed above
    // "✅ No violations found!" is a report contradicting itself. See
    // `score_scope` for why the caption is not simply `outcome.measured`.
    writeln!(
        output,
        "**Quality Score**: {:.1}% ({})",
        lint_result.quality_score * 100.0,
        score_scope(lint_result, outcome)
    )?;
    if !outcome.filter.is_empty() {
        writeln!(
            output,
            "**Rules Filter**: {} — {} of {} violation(s) match",
            outcome.filter_display(),
            outcome.kept.len(),
            outcome.measured
        )?;
    }
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
    outcome: &RuleFilterOutcome,
) -> Result<()> {
    use std::fmt::Write;

    if listed.is_empty() {
        // "No violations found" is a statement about the Makefile; when a
        // --rules value simply matched nothing, the Makefile is not clean and
        // the report must not say it is.
        if outcome.filtered_everything_away() {
            writeln!(
                output,
                "⚠️ {} violation(s) found; 0 match --rules {}.",
                outcome.measured,
                outcome.filter_display()
            )?;
            writeln!(
                output,
                "\nRule(s) that fired on this Makefile: {}",
                outcome.rules_present.join(", ")
            )?;
            writeln!(
                output,
                "(no requested name matched — check the spelling of --rules)"
            )?;
        } else {
            writeln!(output, "✅ No violations found!")?;
        }
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
        let outcome = apply_rule_filter(v, &[]);
        format_makefile_output(
            Path::new("Makefile"),
            &outcome,
            &lint_result(),
            None,
            format,
            top,
        )
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

/// `analyze makefile --rules <typo>` silently filtered every violation away:
/// stdout printed "✅ No violations found!" under a 50.0% Quality Score while
/// stderr of the same process printed "Found 3 violations". These tests fail on
/// that code.
#[cfg(test)]
mod makefile_rules_filter_tests {
    use super::*;

    fn three() -> Vec<makefile_linter::Violation> {
        [
            ("security/shell-injection", makefile_linter::Severity::Error),
            ("undefinedvariable", makefile_linter::Severity::Warning),
            ("undefinedvariable", makefile_linter::Severity::Warning),
        ]
        .into_iter()
        .enumerate()
        .map(|(i, (rule, severity))| makefile_linter::Violation {
            rule: rule.to_string(),
            severity,
            span: crate::services::makefile_linter::ast::SourceSpan {
                start: 0,
                end: 0,
                line: i + 1,
                column: 1,
            },
            message: format!("violation {i}"),
            fix_hint: None,
        })
        .collect()
    }

    fn lint_result() -> makefile_linter::LintResult {
        makefile_linter::LintResult {
            path: PathBuf::from("Makefile"),
            violations: three(),
            quality_score: 0.5,
        }
    }

    fn render(rules: &[&str], format: MakefileOutputFormat) -> String {
        let owned: Vec<String> = rules.iter().map(|r| (*r).to_string()).collect();
        let outcome = apply_rule_filter(&three(), &owned);
        format_makefile_output(
            Path::new("Makefile"),
            &outcome,
            &lint_result(),
            None,
            format,
            0,
        )
        .expect("render")
    }

    #[test]
    fn an_unmatched_rules_value_does_not_report_a_clean_makefile() {
        let human = render(&["nonexistent-rule-xyz"], MakefileOutputFormat::Human);
        assert!(
            !human.contains("No violations found"),
            "a filter that matched nothing reported the Makefile clean:\n{human}"
        );
        assert!(
            human.contains("3 violation(s) found; 0 match --rules nonexistent-rule-xyz"),
            "the report must name both counts:\n{human}"
        );
        assert!(
            human.contains("security/shell-injection"),
            "the report must name the rules that did fire:\n{human}"
        );
    }

    /// The printed score is derived from the unfiltered violations, so it may
    /// not sit unqualified above a table showing a subset of them.
    #[test]
    fn the_quality_score_says_what_it_was_measured_over() {
        let human = render(&["nonexistent-rule-xyz"], MakefileOutputFormat::Human);
        assert!(
            human.contains("**Quality Score**: 50.0% (over all 3 violation(s) found)"),
            "{human}"
        );
        assert!(
            human.contains("**Rules Filter**: nonexistent-rule-xyz — 0 of 3 violation(s) match"),
            "{human}"
        );
    }

    /// #961 residual: `--gnu-version` appends its incompatibilities to the
    /// measured list AFTER the linter computed `quality_score`, so the caption
    /// "over all {measured} violation(s) found" attributed the score to a set
    /// it was never computed over. Measured on a real Makefile using
    /// `.ONESHELL:` and `::=`, `--gnu-version 3.0` printed
    /// "Quality score: 100.0% (over all 2 violations found)" — a perfect score
    /// over two findings.
    ///
    /// RED on the old code, which printed the appended total in both cases.
    #[test]
    fn the_score_caption_names_only_the_violations_it_covers() {
        // No appended findings: the original wording, unchanged.
        let mut outcome = apply_rule_filter(&three(), &[]);
        assert_eq!(
            score_scope(&lint_result(), &outcome),
            "over all 3 violation(s) found"
        );

        // Two `--gnu-version` findings appended to the three the linter scored.
        outcome.measured = 5;
        assert_eq!(
            score_scope(&lint_result(), &outcome),
            "over the 3 violation(s) the linter measured; 2 further --gnu-version \
             finding(s) are not in the score"
        );

        // A clean Makefile whose only findings come from --gnu-version must not
        // present its 100% as covering them.
        let clean = makefile_linter::LintResult {
            path: PathBuf::from("Makefile"),
            violations: vec![],
            quality_score: 1.0,
        };
        let mut empty = apply_rule_filter(&[], &[]);
        empty.measured = 2;
        assert_eq!(
            score_scope(&clean, &empty),
            "over the 0 violation(s) the linter measured; 2 further --gnu-version \
             finding(s) are not in the score"
        );
    }

    #[test]
    fn json_carries_the_measured_total_and_the_filter() {
        let parsed: serde_json::Value =
            serde_json::from_str(&render(&["nonexistent-rule-xyz"], MakefileOutputFormat::Json))
                .expect("json");
        assert_eq!(parsed["violations_measured"], 3);
        assert_eq!(parsed["violations_total"], 0);
        assert_eq!(parsed["rules_filter"][0], "nonexistent-rule-xyz");
        assert_eq!(parsed["rules_present"][0], "security/shell-injection");
        assert_eq!(parsed["rules_present"][1], "undefinedvariable");
    }

    #[test]
    fn a_matching_rule_still_filters_and_says_so() {
        let human = render(&["undefinedvariable"], MakefileOutputFormat::Human);
        assert!(human.contains("2 of 3 violation(s) match"), "{human}");
        assert!(!human.contains("shell-injection"), "{human}");
        assert!(!human.contains("0 match --rules"), "{human}");
    }

    /// A genuinely clean Makefile keeps its clean report.
    #[test]
    fn no_violations_at_all_still_reads_as_clean() {
        let outcome = apply_rule_filter(&[], &["undefinedvariable".to_string()]);
        assert!(!outcome.filtered_everything_away());
        let mut out = String::new();
        write_makefile_violations_table(&mut out, &[], 0, 0, &outcome).expect("write");
        assert!(out.contains("✅ No violations found!"), "{out}");
    }

    /// `--rules ''` matched nothing and was accepted; a blank name can never be
    /// a rule id, so it is rejected the way a nonexistent path already is.
    #[test]
    fn a_blank_rule_name_is_rejected() {
        assert!(validate_rule_names(&["".to_string()]).is_err());
        assert!(validate_rule_names(&["   ".to_string()]).is_err());
        assert!(validate_rule_names(&["undefinedvariable".to_string()]).is_ok());
        assert!(validate_rule_names(&[]).is_ok());
    }

    #[test]
    fn all_and_empty_mean_no_filter() {
        for rules in [
            vec![],
            vec!["all".to_string()],
            vec!["all".to_string(), "undefinedvariable".to_string()],
        ] {
            let outcome = apply_rule_filter(&three(), &rules);
            assert_eq!(outcome.kept.len(), 3, "{rules:?}");
            assert!(outcome.filter.is_empty(), "{rules:?}");
            assert!(!outcome.filtered_everything_away(), "{rules:?}");
        }
    }

    /// #961 residual: a non-blank but UNREGISTERED `--rules` value was accepted,
    /// removed every measured violation and exited 0. RED on the old code, whose
    /// `validate_rule_names` only rejected blanks.
    #[test]
    fn an_unregistered_rule_name_is_rejected() {
        let err = validate_rule_names(&["nonexistent-rule-xyz".to_string()])
            .expect_err("a rule id no rule can emit must be rejected, not silently applied");
        let message = err.to_string();
        assert!(
            message.contains("nonexistent-rule-xyz"),
            "the error must name the offending value:\n{message}"
        );
        assert!(
            message.contains("security/shell-injection") && message.contains("undefinedvariable"),
            "the error must list the valid rule ids:\n{message}"
        );
    }

    /// The counterpart the fix must not break: a REGISTERED rule that simply
    /// found nothing is a clean run, not an error. Rejecting it would trade one
    /// silent failure for a loud false one.
    #[test]
    fn every_registered_rule_id_is_an_accepted_rules_value() {
        let registry = makefile_linter::RuleRegistry::new();
        let ids = registry.rule_ids();
        assert!(
            ids.len() >= 11,
            "the registry should expose every registered rule id, got {ids:?}"
        );
        for id in ids {
            validate_rule_names(&[id.to_string()])
                .unwrap_or_else(|e| panic!("registered rule '{id}' must be accepted: {e}"));
        }
        // Not registry rules, but real `--rules` values.
        validate_rule_names(&["all".to_string()]).expect("all");
        validate_rule_names(&[GNU_VERSION_RULE.to_string()]).expect("gnuversion");
    }

    /// Every id the registry advertises must be an id violations actually carry,
    /// or the validation would accept names that can never match.
    #[test]
    fn advertised_rule_ids_are_the_ids_violations_carry() {
        let registry = makefile_linter::RuleRegistry::new();
        let ids = registry.rule_ids();
        for rule in ["security/shell-injection", "undefinedvariable", "minphony"] {
            assert!(ids.contains(&rule), "{rule} missing from {ids:?}");
        }
    }
}
