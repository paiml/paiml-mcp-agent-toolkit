/// Format output based on format type
///
/// `evolution`/`days` are gone from this dispatcher: nothing measured debt over
/// time. Summary and SARIF ignored the flag outright, JSON emitted the literal
/// string "Evolution tracking would show SATD trends over time" and Markdown a
/// "## Evolution (Last N Days)" heading over the same sentence, so `--days`
/// only ever changed a heading. `handle_analyze_satd` now refuses `--evolution`
/// instead of rendering a placeholder that looks like a measurement.
fn format_output(
    result: &SatdAnalysisResult,
    format: SatdOutputFormat,
    metrics: bool,
    top_files: usize,
) -> String {
    match format {
        SatdOutputFormat::Summary => format_summary(result, top_files),
        SatdOutputFormat::Json => format_json(result, metrics, top_files),
        // SARIF alone lists every violation regardless of `--top-files`: it is
        // the format an IDE/code-scanner ingests, and a row limit there hides
        // real findings from the tool that is supposed to surface them.
        SatdOutputFormat::Sarif => format_sarif(result),
        SatdOutputFormat::Markdown => format_markdown(result, top_files),
    }
}

/// Format as summary
fn format_summary(result: &SatdAnalysisResult, top_files: usize) -> String {
    use crate::cli::colors as c;

    let mut output = String::new();
    output.push_str(&format!("{}\n\n", c::header("SATD Analysis Summary")));
    output.push_str(&result.summary);
    output.push_str(&format!(
        "\n\n{}  {}\n",
        c::label("Total violations:"),
        c::number(&result.violations.len().to_string())
    ));
    // The population that count was measured over. A violation count with no
    // denominator is the whole of #1035: "0" reads identically whether the tree
    // was read and clean or never read at all.
    if let Some(note) = result.census.note() {
        output.push_str(&format!("{}  {}\n", c::label("Scope:"), c::dim(&note)));
    }

    // Group by severity
    let critical_count = result
        .violations
        .iter()
        .filter(|v| {
            matches!(
                v.severity,
                crate::services::facades::satd_facade::SatdSeverity::Critical
            )
        })
        .count();
    let high_count = result
        .violations
        .iter()
        .filter(|v| {
            matches!(
                v.severity,
                crate::services::facades::satd_facade::SatdSeverity::High
            )
        })
        .count();
    let medium_count = result
        .violations
        .iter()
        .filter(|v| {
            matches!(
                v.severity,
                crate::services::facades::satd_facade::SatdSeverity::Medium
            )
        })
        .count();
    let low_count = result
        .violations
        .iter()
        .filter(|v| {
            matches!(
                v.severity,
                crate::services::facades::satd_facade::SatdSeverity::Low
            )
        })
        .count();

    // `--color never` / NO_COLOR reached these lines and changed nothing: the
    // raw `c::BOLD`/`c::RED`/`c::RESET` consts are unconditional, so
    // `analyze satd ... --color never > out.txt` still wrote five escape-bearing
    // lines. `c::seq` is the same sequence gated on `colors_enabled()`.
    output.push_str(&format!("\n{}\n", c::subheader("Severity Distribution")));
    output.push_str(&format!(
        "  {}{}Critical:{} {}\n",
        c::seq(c::BOLD),
        c::seq(c::RED),
        c::seq(c::RESET),
        c::number(&critical_count.to_string())
    ));
    output.push_str(&format!(
        "  {}{}High:{} {}\n",
        c::seq(c::BOLD),
        c::seq(c::RED),
        c::seq(c::RESET),
        c::number(&high_count.to_string())
    ));
    output.push_str(&format!(
        "  {}{}Medium:{} {}\n",
        c::seq(c::BOLD),
        c::seq(c::YELLOW),
        c::seq(c::RESET),
        c::number(&medium_count.to_string())
    ));
    output.push_str(&format!(
        "  {}{}Low:{} {}\n",
        c::seq(c::BOLD),
        c::seq(c::GREEN),
        c::seq(c::RESET),
        c::number(&low_count.to_string())
    ));

    if !result.violations.is_empty() {
        // `.take(10)` was hardcoded here: over a corpus of 63 violations,
        // `--top-files 1` and `--top-files 50` both printed ten rows.
        let shown = crate::cli::top_files_slice(&result.violations, top_files);
        output.push_str(&format!("\n{}\n", c::subheader("Top Violations")));
        for (i, violation) in shown.iter().enumerate() {
            let sev_color = match violation.severity {
                crate::services::facades::satd_facade::SatdSeverity::Critical
                | crate::services::facades::satd_facade::SatdSeverity::High => c::RED,
                crate::services::facades::satd_facade::SatdSeverity::Medium => c::YELLOW,
                crate::services::facades::satd_facade::SatdSeverity::Low => c::GREEN,
            };
            output.push_str(&format!(
                "  {}. {}:{} - {} {}{:?}{}\n",
                c::number(&(i + 1).to_string()),
                c::path(&violation.file_path),
                c::dim(&violation.line_number.to_string()),
                violation.violation_type,
                c::seq(sev_color),
                violation.severity,
                c::seq(c::RESET)
            ));
        }
        // A list that is secretly a cap tells the reader nothing about what it
        // hid; name both numbers, the way the big-O surfaces already do.
        if shown.len() < result.violations.len() {
            output.push_str(&format!(
                "  {}\n",
                c::dim(&format!(
                    "… {} more not shown (--top-files {}, 0 = all)",
                    result.violations.len() - shown.len(),
                    top_files
                ))
            ));
        }
    }

    output
}

/// Format as JSON
fn format_json(result: &SatdAnalysisResult, metrics: bool, top_files: usize) -> String {
    let listed = crate::cli::top_files_slice(&result.violations, top_files);
    let census = &result.census;
    let mut json_data = serde_json::json!({
        // Files that HELD a violation. Kept under its historical name, and
        // named here so it is never mistaken for the denominator: on a clean
        // tree it is 0 whether one file was read or a thousand.
        "total_files": result.total_files,
        "total_violations": result.violations.len(),
        "violations_listed": listed.len(),
        "violations_truncated": listed.len() < result.violations.len(),
        // The denominator, in the vocabulary `analyze complexity --format json`
        // already uses. Without it, `total_violations: 0` reads the same
        // whether the tree is clean or whether the walk skipped nearly all of it.
        "files_discovered": census.discovered,
        "files_analyzed": census.analyzed,
        // The two buckets PARTITION `files_discovered`. `census_balances` says
        // so as a fact a consumer can assert on rather than an invariant it has
        // to trust, and `files_unaccounted` publishes the gap if there is one —
        // a census that does not add up is the defect this block exists to
        // remove, wearing new clothes.
        "census_balances": census.partitions(),
        "files_unaccounted": census.unaccounted(),
        "files_not_read": {
            "total": census.not_read.total(),
            "tests": census.not_read.tests,
            // fuzz harnesses, vendored and generated code, build manifests, and
            // pmat's own SATD analyser. `examples/` and `demo/` LEFT this
            // bucket in #1035: they are shipped, compiled code and are analysed
            // now, which is why the key is no longer named after them.
            "out_of_scope": census.not_read.out_of_scope,
            "minified_or_vendor": census.not_read.minified_or_vendor,
            "too_large": census.not_read.too_large,
            // A file the walk selected and then could not decode. Counted as
            // an analysed-and-clean file until #1035.
            "unreadable": census.not_read.unreadable,
            // Named, not merely counted: the size skip used to reach stderr
            // only, which `--format json` and `--output FILE` both discard, so
            // a consumer could not tell "clean" from "not looked at".
            "oversized": census.oversized.iter().map(|f| serde_json::json!({
                "path": f.path,
                "bytes": f.bytes,
                "limit_bytes": f.limit_bytes,
            })).collect::<Vec<_>>()
        },
        "summary": result.summary,
        "violations": listed.iter().map(|v| {
            serde_json::json!({
                "file": v.file_path,
                "line": v.line_number,
                "type": v.violation_type,
                "message": v.message,
                "severity": format!("{:?}", v.severity)
            })
        }).collect::<Vec<_>>()
    });

    if metrics {
        json_data["metrics"] = serde_json::json!({
            "critical_count": result.violations.iter()
                .filter(|v| matches!(v.severity, crate::services::facades::satd_facade::SatdSeverity::Critical))
                .count(),
            "high_count": result.violations.iter()
                .filter(|v| matches!(v.severity, crate::services::facades::satd_facade::SatdSeverity::High))
                .count(),
            "medium_count": result.violations.iter()
                .filter(|v| matches!(v.severity, crate::services::facades::satd_facade::SatdSeverity::Medium))
                .count(),
            "low_count": result.violations.iter()
                .filter(|v| matches!(v.severity, crate::services::facades::satd_facade::SatdSeverity::Low))
                .count(),
        });
    }

    serde_json::to_string_pretty(&json_data).unwrap_or_else(|_| "{}".to_string())
}

/// Format as SARIF
fn format_sarif(result: &SatdAnalysisResult) -> String {
    let census = &result.census;
    let rules = vec![serde_json::json!({
        "id": "satd-violation",
        "shortDescription": {
            "text": "Self-Admitted Technical Debt"
        },
        "fullDescription": {
            "text": "Code contains self-admitted technical debt that should be addressed"
        }
    })];

    let results: Vec<_> = result
        .violations
        .iter()
        .map(|violation| {
            let level = match violation.severity {
                crate::services::facades::satd_facade::SatdSeverity::Critical => "error",
                crate::services::facades::satd_facade::SatdSeverity::High => "error",
                crate::services::facades::satd_facade::SatdSeverity::Medium => "warning",
                crate::services::facades::satd_facade::SatdSeverity::Low => "note",
            };

            serde_json::json!({
                "ruleId": "satd-violation",
                "level": level,
                "message": {
                    "text": format!("{}: {}", violation.violation_type, violation.message)
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": violation.file_path.clone()
                        },
                        "region": {
                            "startLine": violation.line_number
                        }
                    }
                }]
            })
        })
        .collect();

    serde_json::json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "pmat-satd-detector",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                    "rules": rules
                }
            },
            "results": results,
            // SARIF is the format a code-scanner ingests, and it had NO
            // denominator at all: a run that read nothing produced the same
            // empty `results` array as a run that read everything and found
            // nothing. `invocations` is where SARIF puts facts about the run
            // itself, and the property bag carries the same census the other
            // three formats print.
            "invocations": [{
                "executionSuccessful": true,
                "properties": {
                    "filesDiscovered": census.discovered,
                    "filesAnalyzed": census.analyzed,
                    "filesNotRead": census.not_read.total(),
                    "censusBalances": census.partitions(),
                    "notReadByReason": {
                        "tests": census.not_read.tests,
                        "outOfScope": census.not_read.out_of_scope,
                        "minifiedOrVendor": census.not_read.minified_or_vendor,
                        "tooLarge": census.not_read.too_large,
                        "unreadable": census.not_read.unreadable,
                    },
                    "oversized": census.oversized.iter().map(|f| serde_json::json!({
                        "path": f.path,
                        "bytes": f.bytes,
                        "limitBytes": f.limit_bytes,
                    })).collect::<Vec<_>>(),
                }
            }]
        }]
    })
    .to_string()
}

/// Format as Markdown
fn format_markdown(result: &SatdAnalysisResult, top_files: usize) -> String {
    let mut output = String::new();
    output.push_str("# SATD Analysis Report\n\n");
    output.push_str(&format!("**Summary:** {}\n\n", result.summary));

    output.push_str("## Metrics\n\n");
    output.push_str("| Metric | Value |\n");
    output.push_str("|--------|-------|\n");
    // `Total Files` was the count of files that HELD a violation, presented in
    // a "Metrics" table directly above the violation count as if it were the
    // population — so a report reading "Total Files 0 / Total Violations 0"
    // could not be told apart from one where nothing was read. Both numbers are
    // now named for what they are, beside the census that divides them.
    output.push_str(&format!("| Files Walked | {} |\n", result.census.discovered));
    output.push_str(&format!("| Files Analysed | {} |\n", result.census.analyzed));
    output.push_str(&format!(
        "| Files Not Read | {} |\n",
        result.census.not_read.total()
    ));
    output.push_str(&format!(
        "| Files With Violations | {} |\n",
        result.total_files
    ));
    output.push_str(&format!(
        "| Total Violations | {} |\n",
        result.violations.len()
    ));

    let critical_count = result
        .violations
        .iter()
        .filter(|v| {
            matches!(
                v.severity,
                crate::services::facades::satd_facade::SatdSeverity::Critical
            )
        })
        .count();
    let high_count = result
        .violations
        .iter()
        .filter(|v| {
            matches!(
                v.severity,
                crate::services::facades::satd_facade::SatdSeverity::High
            )
        })
        .count();

    output.push_str(&format!("| Critical Violations | {critical_count} |\n"));
    output.push_str(&format!("| High Violations | {high_count} |\n\n"));

    if !result.violations.is_empty() {
        output.push_str("## Violations\n\n");
        output.push_str("| File | Line | Type | Severity | Message |\n");
        output.push_str("|------|------|------|----------|----------|\n");

        let shown = crate::cli::top_files_slice(&result.violations, top_files);
        for violation in shown {
            output.push_str(&format!(
                "| {} | {} | {} | {:?} | {} |\n",
                violation.file_path,
                violation.line_number,
                violation.violation_type,
                violation.severity,
                violation.message
            ));
        }
        if shown.len() < result.violations.len() {
            output.push_str(&format!(
                "\n_{} of {} violations shown (`--top-files {}`, 0 = all)._\n",
                shown.len(),
                result.violations.len(),
                top_files
            ));
        }
    }

    output
}

/// Print metrics to stderr
fn print_metrics(result: &SatdAnalysisResult) {
    use crate::cli::colors as c;

    eprintln!("\n{} SATD Metrics:", c::subheader("📊"));
    eprintln!(
        "  {} {}",
        c::label("Total files analyzed:"),
        c::number(&result.total_files.to_string())
    );
    eprintln!(
        "  {} {}",
        c::label("Total violations:"),
        c::number(&result.violations.len().to_string())
    );

    let critical_count = result
        .violations
        .iter()
        .filter(|v| {
            matches!(
                v.severity,
                crate::services::facades::satd_facade::SatdSeverity::Critical
            )
        })
        .count();
    let high_count = result
        .violations
        .iter()
        .filter(|v| {
            matches!(
                v.severity,
                crate::services::facades::satd_facade::SatdSeverity::High
            )
        })
        .count();

    // Same unconditional-const defect as the severity block above.
    eprintln!(
        "  {}Critical violations:{} {}",
        c::seq(c::BOLD_RED),
        c::seq(c::RESET),
        c::number(&critical_count.to_string())
    );
    eprintln!(
        "  {}High violations:{} {}",
        c::seq(c::BOLD_RED),
        c::seq(c::RESET),
        c::number(&high_count.to_string())
    );

    if !result.violations.is_empty() {
        eprintln!("\n  {}:", c::label("Top violation types"));
        use std::collections::HashMap;
        let mut type_counts: HashMap<&str, usize> = HashMap::new();
        for violation in &result.violations {
            *type_counts.entry(&violation.violation_type).or_insert(0) += 1;
        }

        let mut sorted_types: Vec<_> = type_counts.iter().collect();
        sorted_types.sort_by(|a, b| b.1.cmp(a.1));

        for (violation_type, count) in sorted_types.iter().take(5) {
            eprintln!(
                "    - {}: {}",
                violation_type,
                c::number(&count.to_string())
            );
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod every_format_carries_the_denominator_tests {
    //! Issue #1035. A finding count with no denominator is the whole defect:
    //! `total_violations: 0` reads identically whether the tree was read in full
    //! and found clean or whether nothing in it was read at all.
    //!
    //! RED, on the build before this module existed — all four renderers of the
    //! same clean result:
    //!
    //! ```text
    //! json      {"total_files": 0, "total_violations": 0, "files_not_read": {...}}
    //! summary   Total violations:  0
    //! markdown  | Total Files | 0 |   ← files that HELD a violation, not a population
    //! sarif     {"results": []}       ← no denominator at all
    //! ```
    //!
    //! Only `files_not_read` existed, and it could not be checked against
    //! anything: there was no count of files walked and no count of files read.
    use super::*;
    use crate::services::facades::satd_facade::SatdAnalysisResult;
    use crate::services::satd_detector::{FileCensus, OversizedFile, SkipCounts, MAX_FILE_BYTES};

    /// Six files walked, two read: one declined for size (named), two out of
    /// scope, one a test file — the shape of the issue's own fixture.
    fn census() -> FileCensus {
        FileCensus {
            discovered: 6,
            analyzed: 2,
            not_read: SkipCounts {
                tests: 1,
                out_of_scope: 2,
                too_large: 1,
                ..Default::default()
            },
            oversized: vec![OversizedFile {
                path: "src/huge.rs".to_string(),
                bytes: MAX_FILE_BYTES + 1,
                limit_bytes: MAX_FILE_BYTES,
            }],
        }
    }

    fn clean_result() -> SatdAnalysisResult {
        SatdAnalysisResult {
            total_files: 0,
            violations: vec![],
            summary: "Found 0 SATD violations in 0 files".to_string(),
            census: census(),
        }
    }

    #[test]
    fn json_states_the_population_and_that_it_partitions() {
        let doc: serde_json::Value =
            serde_json::from_str(&format_json(&clean_result(), false, 0)).expect("valid json");

        assert_eq!(doc["total_violations"], 0);
        assert_eq!(doc["files_discovered"], 6, "{doc}");
        assert_eq!(doc["files_analyzed"], 2, "{doc}");
        assert_eq!(doc["files_not_read"]["total"], 4, "{doc}");
        assert_eq!(
            doc["census_balances"],
            serde_json::Value::Bool(true),
            "2 analysed + 4 not read == 6 walked: {doc}"
        );
        assert_eq!(doc["files_unaccounted"], 0, "{doc}");
        // The size skip once reached stderr only, which --format json discards.
        assert_eq!(doc["files_not_read"]["oversized"][0]["path"], "src/huge.rs");
        assert_eq!(
            doc["files_not_read"]["oversized"][0]["limit_bytes"],
            MAX_FILE_BYTES
        );
        // `examples/` left this bucket: it is analysed now, so the key must not
        // still be named after it.
        assert!(
            doc["files_not_read"]["examples_demo_fuzz_generated"].is_null(),
            "a stale key is a false claim about scope: {doc}"
        );
        assert_eq!(doc["files_not_read"]["out_of_scope"], 2, "{doc}");
    }

    #[test]
    fn the_summary_report_states_the_population() {
        let out = format_summary(&clean_result(), 10);
        assert!(out.contains("analysed 2 of 6"), "{out}");
        assert!(out.contains("not read"), "{out}");
        assert!(out.contains("too large"), "{out}");
    }

    #[test]
    fn markdown_separates_the_population_from_the_files_that_held_a_finding() {
        let out = format_markdown(&clean_result(), 10);
        assert!(out.contains("| Files Walked | 6 |"), "{out}");
        assert!(out.contains("| Files Analysed | 2 |"), "{out}");
        assert!(out.contains("| Files Not Read | 4 |"), "{out}");
        assert!(
            out.contains("| Files With Violations | 0 |"),
            "the old `Total Files` row was this number under a name that read \
             like a denominator: {out}"
        );
    }

    #[test]
    fn sarif_carries_the_census_in_its_invocation() {
        let doc: serde_json::Value =
            serde_json::from_str(&format_sarif(&clean_result())).expect("valid sarif");
        let props = &doc["runs"][0]["invocations"][0]["properties"];
        assert_eq!(props["filesDiscovered"], 6, "{doc}");
        assert_eq!(props["filesAnalyzed"], 2, "{doc}");
        assert_eq!(props["filesNotRead"], 4, "{doc}");
        assert_eq!(props["censusBalances"], serde_json::Value::Bool(true));
        assert_eq!(props["oversized"][0]["path"], "src/huge.rs");
    }

    /// COUNTER-TEST. Disclosure must not become a standing complaint: a tree
    /// read in full still reports zero findings, over a denominator that is
    /// stated and non-zero, and claims nothing was skipped. Otherwise the new
    /// lines are noise and readers learn to skip them — which is how the
    /// original defect survives a fix.
    #[test]
    fn a_fully_read_tree_states_its_denominator_and_claims_no_skips() {
        let result = SatdAnalysisResult {
            total_files: 0,
            violations: vec![],
            summary: "Found 0 SATD violations in 0 files".to_string(),
            census: FileCensus {
                discovered: 12,
                analyzed: 12,
                ..Default::default()
            },
        };

        let doc: serde_json::Value =
            serde_json::from_str(&format_json(&result, false, 0)).expect("valid json");
        assert_eq!(doc["total_violations"], 0);
        assert_eq!(doc["files_analyzed"], 12, "{doc}");
        assert_eq!(doc["files_not_read"]["total"], 0, "{doc}");
        assert_eq!(
            doc["census_balances"],
            serde_json::Value::Bool(true),
            "{doc}"
        );

        let out = format_summary(&result, 10);
        assert!(out.contains("analysed 12 of 12"), "{out}");
        assert!(
            !out.contains("not read"),
            "nothing was skipped, so nothing may be claimed as skipped: {out}"
        );
    }
}
