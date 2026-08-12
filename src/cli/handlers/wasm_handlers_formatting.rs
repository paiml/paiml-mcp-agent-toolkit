// Report renderers for `analyze web-assembly` / `analyze assembly-script`.
// (This file is `include!`d into wasm_handlers.rs, so it cannot carry `//!`.)
//
// The human report is built here and nowhere else, and every styled span goes
// through `crate::cli::colors`. Both reports used to be assembled from bare
// `format!` with no style calls at all, so `--color always` and `--color never`
// produced byte-identical output (md5 4ff4f1f54cca either way) — the flag
// parsed and changed nothing. `--format json` stays plain: a JSON document is
// not a terminal surface, and escapes inside it would not parse.
//
// FOUR DECLARED FORMATS, ONE RENDERER: both functions used to branch on a
// single equality, `if format == &ComplexityOutputFormat::Json`, dropping
// `summary`, `full` AND `sarif` into one `else`. So `--format sarif` handed an
// IDE a markdown document, and `--format full` ("Full report with violations")
// was byte-for-byte `--format summary` ("Summary statistics only") — md5
// 52abdd686c5f for all three on the same fixture. Every declared value now
// renders as itself:
//
//   summary — the aggregate plus one ranked row per file
//   full    — the aggregate plus every measured field per file
//   json    — the machine document
//   sarif   — SARIF 2.1.0. `kind: informational` / `level: none`, because these
//             two analyses measure modules and define no violation threshold;
//             inventing `warning`s to fill the document would report a
//             judgement nothing made.
//
// `--top-files N` is a row limit on the ranked list, applied in exactly one
// place per report through `crate::cli::top_files_slice`; `0` means all. It
// used to be dropped at the route (`top_files: _top_files`), so `--top-files 1`
// and `--top-files 50` listed all twelve files either way.

// FINDINGS BELONG IN THE REPORT, NOT ONLY ON STDERR: `--security`,
// `--memory-analysis` and `--complexity` were wired to real rules, but every
// finding they produced was written with a bare `eprintln!` from the handler.
// The report on stdout — the document a caller redirects, pipes, parses or
// sends to `--output` — was byte-identical with and without each flag, and
// `--format json` never carried a single finding, so no machine consumer could
// observe them at all. They are report sections now, rendered in all four
// formats, and the handlers no longer print them separately.

/// One row of evidence one of the analysis flags contributed to the report.
#[derive(Debug, Clone)]
struct WasmFinding {
    /// File the row is about.
    file: PathBuf,
    /// `Critical`/`High`/`Medium`/`Low` for a security issue, `info` for a
    /// measured fact or a clean result.
    severity: String,
    /// Rule family the row came from.
    category: String,
    /// What was found, in the words of the rule that found it.
    message: String,
}

impl WasmFinding {
    /// A measured fact or a clean result — reported, but not a judgement.
    fn info(file: &Path, category: &str, message: String) -> Self {
        Self {
            file: file.to_path_buf(),
            severity: "info".to_string(),
            category: category.to_string(),
            message,
        }
    }

    /// SARIF `level` for this row's severity.
    fn sarif_level(&self) -> &'static str {
        match self.severity.as_str() {
            "Critical" | "High" => "error",
            "Medium" => "warning",
            "Low" => "note",
            _ => "none",
        }
    }
}

/// The optional report sections the analysis flags produce.
///
/// `None` means the flag was not passed and the section is absent. `Some(vec)`
/// means the flag ran: an empty vec is itself a result ("the rules ran and
/// matched nothing"), which is why a clean file still contributes an `info`
/// row rather than silence.
#[derive(Debug, Clone, Default)]
struct WasmSections {
    /// `--security`
    security: Option<Vec<WasmFinding>>,
    /// `--memory-analysis`
    memory: Option<Vec<WasmFinding>>,
    /// `--complexity` (web-assembly text modules)
    complexity: Option<Vec<WasmFinding>>,
}

impl WasmSections {
    /// Every populated section, in report order, with its heading.
    fn named(&self) -> Vec<(&'static str, &Vec<WasmFinding>)> {
        [
            ("Security (--security)", self.security.as_ref()),
            ("Memory (--memory-analysis)", self.memory.as_ref()),
            ("Complexity (--complexity)", self.complexity.as_ref()),
        ]
        .into_iter()
        .filter_map(|(title, rows)| rows.map(|rows| (title, rows)))
        .collect()
    }

    /// Flattened rows for the machine formats, tagged with their section.
    fn flat(&self) -> Vec<(&'static str, &WasmFinding)> {
        self.named()
            .into_iter()
            .flat_map(|(title, rows)| rows.iter().map(move |row| (title, row)))
            .collect()
    }
}

/// Append the populated finding sections to a human report.
fn push_findings_sections(output: &mut String, sections: &WasmSections) {
    use crate::cli::colors as c;

    for (title, rows) in sections.named() {
        output.push_str(&format!("\n{}\n\n", c::subheader(&format!("## {title}"))));
        if rows.is_empty() {
            output.push_str(&format!(
                "{}\n",
                c::dim("No rows: nothing these rules apply to was analysed.")
            ));
            continue;
        }
        for row in rows {
            output.push_str(&format!(
                "- {} {} {}\n",
                c::label(&format!("[{}/{}]", row.severity, row.category)),
                c::path(&row.file.display().to_string()),
                row.message
            ));
        }
    }
}

/// JSON value for one finding section, or `null` when the flag was not passed.
fn findings_json(rows: Option<&Vec<WasmFinding>>) -> serde_json::Value {
    match rows {
        None => serde_json::Value::Null,
        Some(rows) => serde_json::Value::Array(
            rows.iter()
                .map(|row| {
                    serde_json::json!({
                        "file": row.file.display().to_string(),
                        "severity": row.severity,
                        "category": row.category,
                        "message": row.message,
                    })
                })
                .collect(),
        ),
    }
}

/// Rank `AssemblyScript` results worst-first so `--top-files` selects the top.
fn rank_assemblyscript(results: &[(PathBuf, WasmComplexity)]) -> Vec<(PathBuf, WasmComplexity)> {
    let mut ranked = results.to_vec();
    // Path last, so the order is total: the walk order is filesystem order and
    // was reproducing rows as mod4, mod8, mod6 … between runs.
    ranked.sort_by(|a, b| {
        b.1.cyclomatic
            .cmp(&a.1.cyclomatic)
            .then_with(|| b.1.cognitive.cmp(&a.1.cognitive))
            .then_with(|| a.0.cmp(&b.0))
    });
    ranked
}

/// Rank WebAssembly results largest-first so `--top-files` selects the top.
fn rank_webassembly(results: &[(PathBuf, WasmMetrics)]) -> Vec<(PathBuf, WasmMetrics)> {
    let mut ranked = results.to_vec();
    ranked.sort_by(|a, b| {
        b.1.function_count
            .cmp(&a.1.function_count)
            .then_with(|| b.1.linear_memory_pages.cmp(&a.1.linear_memory_pages))
            .then_with(|| a.0.cmp(&b.0))
    });
    ranked
}

/// Header shared by the `summary` and `full` renderers of both commands.
fn wasm_report_header(
    title: &str,
    analyzed: usize,
    listed: usize,
    elapsed: std::time::Duration,
) -> String {
    use crate::cli::colors as c;

    let mut output = format!("{}\n\n", c::header(title));
    output.push_str(&format!(
        "📁 {} {}\n",
        c::label("**Files analyzed**:"),
        c::number(&analyzed.to_string())
    ));
    output.push_str(&format!(
        "⏱️  {} {}\n\n",
        c::label("**Analysis time**:"),
        c::number(&format!("{:.2}s", elapsed.as_secs_f64()))
    ));
    // A capped list that looks complete is how "10 of 63" gets read as "63".
    if listed < analyzed {
        output.push_str(&format!(
            "{}\n\n",
            c::dim(&format!(
                "Showing the top {listed} of {analyzed} files (--top-files, 0 = all)."
            ))
        ));
    }
    output
}

/// Performance block shared by both JSON documents.
fn wasm_perf_json(
    perf: bool,
    count: usize,
    elapsed: std::time::Duration,
) -> Option<serde_json::Value> {
    if perf {
        Some(serde_json::json!({
            "total_time_ms": elapsed.as_millis(),
            "avg_time_per_file_ms": elapsed.as_millis() / (count as u128).max(1)
        }))
    } else {
        None
    }
}

/// SARIF 2.1.0 envelope shared by both commands.
///
/// `artifacts` names every file the run reported and `results` carries one
/// informational entry per file, so an empty-result document still proves what
/// was scanned rather than reading as "nothing found".
/// Findings carry their own rule and a real `level`, because a security issue
/// is a judgement and must not be filed as `informational` beside the metrics.
fn wasm_sarif_document(
    driver: &str,
    rule_id: &str,
    rule_text: &str,
    entries: &[(String, String, serde_json::Value)],
    sections: &WasmSections,
) -> Result<String> {
    const FINDING_RULE: &str = "wasm-analysis-finding";

    let findings = sections.flat();
    let mut artifacts: Vec<_> = entries
        .iter()
        .map(|(uri, _, _)| serde_json::json!({ "location": { "uri": uri } }))
        .collect();
    for (_, row) in &findings {
        let uri = row.file.display().to_string();
        let artifact = serde_json::json!({ "location": { "uri": uri } });
        if !artifacts.contains(&artifact) {
            artifacts.push(artifact);
        }
    }

    let mut results: Vec<_> = entries
        .iter()
        .map(|(uri, message, properties)| {
            serde_json::json!({
                "ruleId": rule_id,
                "kind": "informational",
                "level": "none",
                "message": { "text": message },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": { "uri": uri }
                    }
                }],
                "properties": properties
            })
        })
        .collect();
    for (section, row) in &findings {
        results.push(serde_json::json!({
            "ruleId": FINDING_RULE,
            "kind": if row.severity == "info" { "informational" } else { "fail" },
            "level": row.sarif_level(),
            "message": { "text": format!("{}: {}", row.category, row.message) },
            "locations": [{
                "physicalLocation": {
                    "artifactLocation": { "uri": row.file.display().to_string() }
                }
            }],
            "properties": {
                "section": section,
                "severity": row.severity,
                "category": row.category
            }
        }));
    }

    let mut rules = vec![serde_json::json!({
        "id": rule_id,
        "name": rule_id,
        "shortDescription": { "text": rule_text },
        "defaultConfiguration": { "level": "none" }
    })];
    if !findings.is_empty() {
        rules.push(serde_json::json!({
            "id": FINDING_RULE,
            "name": FINDING_RULE,
            "shortDescription": {
                "text": "Finding reported by --security / --memory-analysis / --complexity"
            },
            "defaultConfiguration": { "level": "note" }
        }));
    }

    let doc = serde_json::json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": driver,
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                    "rules": rules
                }
            },
            "artifacts": artifacts,
            "results": results
        }]
    });
    Ok(serde_json::to_string_pretty(&doc)?)
}

/// Format `AssemblyScript` analysis results
fn format_assemblyscript_results(
    results: &[(PathBuf, WasmComplexity)],
    format: &ComplexityOutputFormat,
    perf: bool,
    elapsed: std::time::Duration,
    top_files: usize,
    sections: &WasmSections,
) -> Result<String> {
    use crate::cli::colors as c;

    let ranked = rank_assemblyscript(results);
    let shown = crate::cli::top_files_slice(&ranked, top_files);

    match format {
        ComplexityOutputFormat::Json => {
            let output = serde_json::json!({
                "analysis_type": "assemblyscript",
                "files_analyzed": results.len(),
                "files_listed": shown.len(),
                "files_truncated": shown.len() < results.len(),
                "results": shown.iter().map(|(path, complexity)| {
                    serde_json::json!({
                        "file": path.display().to_string(),
                        "complexity": complexity
                    })
                }).collect::<Vec<_>>(),
                "security": findings_json(sections.security.as_ref()),
                "memory": findings_json(sections.memory.as_ref()),
                "performance": wasm_perf_json(perf, results.len(), elapsed)
            });
            Ok(serde_json::to_string_pretty(&output)?)
        }
        ComplexityOutputFormat::Sarif => {
            let entries: Vec<_> = shown
                .iter()
                .map(|(path, cx)| {
                    (
                        path.display().to_string(),
                        format!(
                            "cyclomatic {}, cognitive {}, memory pressure {:.2}",
                            cx.cyclomatic, cx.cognitive, cx.memory_pressure
                        ),
                        serde_json::json!({ "complexity": cx }),
                    )
                })
                .collect();
            wasm_sarif_document(
                "pmat-assemblyscript-analyzer",
                "assemblyscript-module-complexity",
                "Measured AssemblyScript module complexity",
                &entries,
                sections,
            )
        }
        ComplexityOutputFormat::Summary => {
            let mut output = wasm_report_header(
                "# AssemblyScript Analysis Report",
                results.len(),
                shown.len(),
                elapsed,
            );
            if !shown.is_empty() {
                output.push_str(&format!("{}\n\n", c::subheader("## Results")));
                for (i, (path, cx)) in shown.iter().enumerate() {
                    output.push_str(&format!(
                        "{}. {} — {} {} / {} {}\n",
                        c::number(&(i + 1).to_string()),
                        c::path(&path.display().to_string()),
                        c::label("cyclomatic"),
                        c::number(&cx.cyclomatic.to_string()),
                        c::label("cognitive"),
                        c::number(&cx.cognitive.to_string()),
                    ));
                }
            }
            push_findings_sections(&mut output, sections);
            Ok(output)
        }
        ComplexityOutputFormat::Full => {
            let mut output = wasm_report_header(
                "# AssemblyScript Analysis Report",
                results.len(),
                shown.len(),
                elapsed,
            );
            if !shown.is_empty() {
                output.push_str(&format!("{}\n\n", c::subheader("## Results")));
                for (path, cx) in shown {
                    output.push_str(&format!("### {}\n", c::path(&path.display().to_string())));
                    for (label, value) in [
                        ("**Cyclomatic complexity**:", cx.cyclomatic.to_string()),
                        ("**Cognitive complexity**:", cx.cognitive.to_string()),
                        ("**Max loop depth**:", cx.max_loop_depth.to_string()),
                        ("**Memory pressure**:", format!("{:.2}", cx.memory_pressure)),
                        ("**Hot path score**:", format!("{:.2}", cx.hot_path_score)),
                        (
                            "**Indirect call overhead**:",
                            format!("{:.2}", cx.indirect_call_overhead),
                        ),
                        ("**Estimated gas**:", format!("{:.0}", cx.estimated_gas)),
                    ] {
                        output.push_str(&format!("- {} {}\n", c::label(label), c::number(&value)));
                    }
                    output.push('\n');
                }
            }
            push_findings_sections(&mut output, sections);
            Ok(output)
        }
    }
}

/// Format WebAssembly analysis results
fn format_webassembly_results(
    results: &[(PathBuf, WasmMetrics)],
    format: &ComplexityOutputFormat,
    perf: bool,
    elapsed: std::time::Duration,
    top_files: usize,
    sections: &WasmSections,
) -> Result<String> {
    use crate::cli::colors as c;

    let ranked = rank_webassembly(results);
    let shown = crate::cli::top_files_slice(&ranked, top_files);

    match format {
        ComplexityOutputFormat::Json => {
            let output = serde_json::json!({
                "analysis_type": "webassembly",
                "files_analyzed": results.len(),
                "files_listed": shown.len(),
                "files_truncated": shown.len() < results.len(),
                "results": shown.iter().map(|(path, metrics)| {
                    serde_json::json!({
                        "file": path.display().to_string(),
                        "metrics": metrics
                    })
                }).collect::<Vec<_>>(),
                "security": findings_json(sections.security.as_ref()),
                "memory": findings_json(sections.memory.as_ref()),
                "complexity": findings_json(sections.complexity.as_ref()),
                "performance": wasm_perf_json(perf, results.len(), elapsed)
            });
            Ok(serde_json::to_string_pretty(&output)?)
        }
        ComplexityOutputFormat::Sarif => {
            let entries: Vec<_> = shown
                .iter()
                .map(|(path, m)| {
                    (
                        path.display().to_string(),
                        format!(
                            "{} functions, {} imports, {} exports, {} memory pages",
                            m.function_count, m.import_count, m.export_count, m.linear_memory_pages
                        ),
                        serde_json::json!({ "metrics": m }),
                    )
                })
                .collect();
            wasm_sarif_document(
                "pmat-wasm-analyzer",
                "wasm-module-metrics",
                "Measured WebAssembly module metrics",
                &entries,
                sections,
            )
        }
        ComplexityOutputFormat::Summary => {
            let mut output = wasm_report_header(
                "# WebAssembly Analysis Report",
                results.len(),
                shown.len(),
                elapsed,
            );
            if !shown.is_empty() {
                output.push_str(&format!("{}\n\n", c::subheader("## Results")));
                for (i, (path, m)) in shown.iter().enumerate() {
                    output.push_str(&format!(
                        "{}. {} — {} {} / {} {}\n",
                        c::number(&(i + 1).to_string()),
                        c::path(&path.display().to_string()),
                        c::label("functions"),
                        c::number(&m.function_count.to_string()),
                        c::label("memory pages"),
                        c::number(&m.linear_memory_pages.to_string()),
                    ));
                }
            }
            push_findings_sections(&mut output, sections);
            Ok(output)
        }
        ComplexityOutputFormat::Full => {
            let mut output = wasm_report_header(
                "# WebAssembly Analysis Report",
                results.len(),
                shown.len(),
                elapsed,
            );
            if !shown.is_empty() {
                output.push_str(&format!("{}\n\n", c::subheader("## Results")));
                for (path, m) in shown {
                    output.push_str(&format!("### {}\n", c::path(&path.display().to_string())));
                    for (label, value) in [
                        ("**Functions**:", m.function_count),
                        ("**Imports**:", m.import_count),
                        ("**Exports**:", m.export_count),
                        ("**Globals**:", m.global_count),
                        ("**Memory pages**:", m.linear_memory_pages),
                        ("**Memory sections**:", m.memory_sections),
                        ("**Table sections**:", m.table_sections),
                        ("**Indirect calls**:", m.indirect_calls),
                        ("**Custom sections**:", m.custom_sections),
                        ("**Element segments**:", m.element_segments),
                        ("**Data segments**:", m.data_segments),
                        ("**Memory loads**:", m.memory_operations.loads),
                        ("**Memory stores**:", m.memory_operations.stores),
                        ("**Memory grows**:", m.memory_operations.grows),
                        ("**Atomic ops**:", m.memory_operations.atomic_ops),
                        ("**SIMD ops**:", m.memory_operations.simd_ops),
                        ("**Bulk ops**:", m.memory_operations.bulk_ops),
                    ] {
                        output.push_str(&format!(
                            "- {} {}\n",
                            c::label(label),
                            c::number(&value.to_string())
                        ));
                    }
                    output.push('\n');
                }
            }
            push_findings_sections(&mut output, sections);
            Ok(output)
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod wasm_format_and_limit_tests {
    //! `--format` must select a format and `--top-files` must be a row limit.
    use super::*;

    fn as_results(n: usize) -> Vec<(PathBuf, WasmComplexity)> {
        (0..n)
            .map(|i| {
                (
                    PathBuf::from(format!("assembly/mod{i:02}.ts")),
                    WasmComplexity {
                        cyclomatic: u32::try_from(i).unwrap_or(0),
                        cognitive: u32::try_from(i).unwrap_or(0),
                        memory_pressure: 1.0,
                        hot_path_score: 10.0,
                        estimated_gas: 5000.0,
                        indirect_call_overhead: 1.0,
                        max_loop_depth: 1,
                    },
                )
            })
            .collect()
    }

    fn wasm_results(n: usize) -> Vec<(PathBuf, WasmMetrics)> {
        (0..n)
            .map(|i| {
                (
                    PathBuf::from(format!("wasm/m{i:02}.wasm")),
                    WasmMetrics {
                        function_count: u32::try_from(i).unwrap_or(0),
                        linear_memory_pages: u32::try_from(i).unwrap_or(0),
                        ..WasmMetrics::default()
                    },
                )
            })
            .collect()
    }

    fn rows(report: &str) -> usize {
        report
            .lines()
            .filter(|l| l.starts_with("### ") || l.contains(" — "))
            .count()
    }

    const D: std::time::Duration = std::time::Duration::from_millis(1);

    /// No analysis flag passed: every optional section absent.
    const NO_SECTIONS: WasmSections = WasmSections {
        security: None,
        memory: None,
        complexity: None,
    };

    /// Twelve files, and the row count is whatever `--top-files` says. Both
    /// commands used to print all twelve for every value of the flag.
    #[test]
    fn top_files_limits_rows_for_both_commands() {
        let as_r = as_results(12);
        let wasm_r = wasm_results(12);
        for fmt in [
            ComplexityOutputFormat::Summary,
            ComplexityOutputFormat::Full,
        ] {
            for (limit, expected) in [(1usize, 1usize), (3, 3), (50, 12), (0, 12)] {
                let a = format_assemblyscript_results(&as_r, &fmt, false, D, limit, &NO_SECTIONS)
                    .unwrap();
                assert_eq!(
                    rows(&a),
                    expected,
                    "assembly-script {fmt:?} --top-files {limit}:\n{a}"
                );
                let w = format_webassembly_results(&wasm_r, &fmt, false, D, limit, &NO_SECTIONS)
                    .unwrap();
                assert_eq!(
                    rows(&w),
                    expected,
                    "web-assembly {fmt:?} --top-files {limit}:\n{w}"
                );
            }
        }
    }

    /// The limit takes the WORST rows, not the first ones the walk happened to
    /// yield: `mod11` (cyclomatic 11) outranks `mod00` (cyclomatic 0).
    #[test]
    fn top_files_takes_the_top_of_the_ranking() {
        let report = format_assemblyscript_results(
            &as_results(12),
            &ComplexityOutputFormat::Summary,
            false,
            D,
            1,
            &NO_SECTIONS,
        )
        .unwrap();
        assert!(
            report.contains("mod11.ts"),
            "expected the worst file:\n{report}"
        );
        assert!(
            !report.contains("mod00.ts"),
            "expected only the worst file:\n{report}"
        );
    }

    /// Four declared formats, four distinct renderings. `summary`, `full` and
    /// `sarif` were one `else` branch: byte-for-byte identical output.
    #[test]
    fn every_declared_format_renders_as_itself() {
        let as_r = as_results(3);
        let wasm_r = wasm_results(3);
        for results_kind in 0..2 {
            let render = |fmt: &ComplexityOutputFormat| {
                if results_kind == 0 {
                    format_assemblyscript_results(&as_r, fmt, false, D, 0, &NO_SECTIONS).unwrap()
                } else {
                    format_webassembly_results(&wasm_r, fmt, false, D, 0, &NO_SECTIONS).unwrap()
                }
            };
            let summary = render(&ComplexityOutputFormat::Summary);
            let full = render(&ComplexityOutputFormat::Full);
            let json = render(&ComplexityOutputFormat::Json);
            let sarif = render(&ComplexityOutputFormat::Sarif);

            assert_ne!(summary, full, "summary and full must differ");
            assert_ne!(summary, sarif, "summary and sarif must differ");
            assert_ne!(full, sarif, "full and sarif must differ");
            assert_ne!(json, sarif, "json and sarif must differ");

            // And each is actually the format it claims to be.
            let sarif_doc: serde_json::Value =
                serde_json::from_str(&sarif).expect("--format sarif must emit JSON");
            assert_eq!(sarif_doc["version"], "2.1.0", "sarif must be SARIF 2.1.0");
            assert_eq!(
                sarif_doc["runs"][0]["results"]
                    .as_array()
                    .expect("sarif results array")
                    .len(),
                3
            );
            serde_json::from_str::<serde_json::Value>(&json).expect("--format json must emit JSON");
            assert!(summary.contains("Analysis Report"));
            // `full` names fields `summary` does not.
            assert!(full.contains("Estimated gas") || full.contains("Data segments"));
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod wasm_color_census_tests {
    //! `--color` must move something on the human wasm reports.
    //!
    //! Both reports used to be pure `format!` with no style calls, so
    //! `analyze web-assembly --color always` and `--color never` had the same
    //! md5 (4ff4f1f54cca) — the flag parsed and changed nothing. A test that
    //! only asserts "output is plain when colour is off" is satisfied by that
    //! defect, so both halves are asserted here.
    use super::*;
    use crate::cli::colors::{assert_honours_color, ForcedColor};

    const D: std::time::Duration = std::time::Duration::from_millis(1);

    /// No analysis flag passed: every optional section absent.
    const NO_SECTIONS: WasmSections = WasmSections {
        security: None,
        memory: None,
        complexity: None,
    };

    fn as_one() -> Vec<(PathBuf, WasmComplexity)> {
        vec![(
            PathBuf::from("assembly/index.ts"),
            WasmComplexity {
                cyclomatic: 5,
                cognitive: 5,
                memory_pressure: 1.0,
                hot_path_score: 10.0,
                estimated_gas: 5000.0,
                indirect_call_overhead: 1.0,
                max_loop_depth: 1,
            },
        )]
    }

    fn wasm_one() -> Vec<(PathBuf, WasmMetrics)> {
        vec![(
            PathBuf::from("build/index.wasm"),
            WasmMetrics {
                function_count: 3,
                linear_memory_pages: 2,
                ..WasmMetrics::default()
            },
        )]
    }

    #[test]
    fn human_wasm_reports_honour_color() {
        for fmt in [
            ComplexityOutputFormat::Summary,
            ComplexityOutputFormat::Full,
        ] {
            assert_honours_color(&format!("assembly-script --format {fmt}"), || {
                format_assemblyscript_results(&as_one(), &fmt, false, D, 0, &NO_SECTIONS).unwrap()
            });
            assert_honours_color(&format!("web-assembly --format {fmt}"), || {
                format_webassembly_results(&wasm_one(), &fmt, false, D, 0, &NO_SECTIONS).unwrap()
            });
        }
    }

    /// An empty result set still renders a header and the counts, so it must
    /// honour the flag too — "nothing found" is the case a reader is most
    /// likely to be staring at.
    #[test]
    fn empty_human_reports_honour_color() {
        for fmt in [
            ComplexityOutputFormat::Summary,
            ComplexityOutputFormat::Full,
        ] {
            assert_honours_color(&format!("web-assembly (empty) --format {fmt}"), || {
                format_webassembly_results(&[], &fmt, false, D, 0, &NO_SECTIONS).unwrap()
            });
        }
    }

    /// The machine documents are not terminal surfaces: escapes inside them
    /// would not parse. They stay plain even with colour forced on.
    #[test]
    fn machine_wasm_formats_stay_plain_even_with_color_on() {
        let _guard = ForcedColor::on();
        for fmt in [ComplexityOutputFormat::Json, ComplexityOutputFormat::Sarif] {
            for doc in [
                format_assemblyscript_results(&as_one(), &fmt, true, D, 0, &NO_SECTIONS).unwrap(),
                format_webassembly_results(&wasm_one(), &fmt, true, D, 0, &NO_SECTIONS).unwrap(),
            ] {
                assert!(
                    !doc.contains('\u{1b}'),
                    "--format {fmt} must never carry ANSI, got: {doc:?}"
                );
                serde_json::from_str::<serde_json::Value>(&doc)
                    .expect("machine format must still parse");
            }
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod wasm_findings_reach_the_report_tests {
    //! `--security`, `--memory-analysis` and `--complexity` must change the
    //! REPORT, not only stderr.
    //!
    //! Every one of these flags was wired to a real rule set, and every finding
    //! went out with a bare `eprintln!`. The report on stdout — the document a
    //! caller redirects, pipes, parses, or sends to `--output` — was
    //! byte-identical with and without each flag, and `--format json` carried
    //! no findings at all, so no machine consumer could observe them.
    use super::*;

    const D: std::time::Duration = std::time::Duration::from_millis(1);

    fn wasm_one() -> Vec<(PathBuf, WasmMetrics)> {
        vec![(
            PathBuf::from("wasm/mod.wasm"),
            WasmMetrics {
                function_count: 1,
                linear_memory_pages: 2,
                memory_sections: 1,
                ..WasmMetrics::default()
            },
        )]
    }

    fn finding() -> WasmFinding {
        WasmFinding {
            file: PathBuf::from("wasm/broken.wasm"),
            severity: "Critical".to_string(),
            category: "InvalidFormat".to_string(),
            message: "Invalid WASM magic number".to_string(),
        }
    }

    fn all_formats() -> [ComplexityOutputFormat; 4] {
        [
            ComplexityOutputFormat::Summary,
            ComplexityOutputFormat::Full,
            ComplexityOutputFormat::Json,
            ComplexityOutputFormat::Sarif,
        ]
    }

    /// Every declared format must move when the flag is on. `summary` and
    /// `full` used to be the only place a caller could look, and neither of
    /// them moved either.
    #[test]
    fn security_findings_change_every_format() {
        let sections = WasmSections {
            security: Some(vec![finding()]),
            memory: None,
            complexity: None,
        };
        let none = WasmSections::default();

        for fmt in all_formats() {
            let plain = format_webassembly_results(&wasm_one(), &fmt, false, D, 0, &none).unwrap();
            let with =
                format_webassembly_results(&wasm_one(), &fmt, false, D, 0, &sections).unwrap();

            assert_ne!(plain, with, "--security changed nothing in {fmt:?}");
            assert!(
                with.contains("Invalid WASM magic number"),
                "{fmt:?} must carry the finding:\n{with}"
            );
            assert!(
                !plain.contains("Invalid WASM magic number"),
                "control: the finding must be absent without the flag:\n{plain}"
            );
        }
    }

    /// A machine consumer must be able to read the findings, and the absent
    /// section must be distinguishable from an empty one.
    #[test]
    fn json_carries_findings_and_distinguishes_absent_from_empty() {
        let none = WasmSections::default();
        let ran_clean = WasmSections {
            security: Some(vec![]),
            memory: None,
            complexity: None,
        };
        let found = WasmSections {
            security: Some(vec![finding()]),
            memory: Some(vec![WasmFinding::info(
                Path::new("wasm/mod.wasm"),
                "linear-memory",
                "1 memory section(s), 2 initial page(s) = 128 KiB reserved".to_string(),
            )]),
            complexity: None,
        };

        let parse = |sections: &WasmSections| -> serde_json::Value {
            serde_json::from_str(
                &format_webassembly_results(
                    &wasm_one(),
                    &ComplexityOutputFormat::Json,
                    false,
                    D,
                    0,
                    sections,
                )
                .unwrap(),
            )
            .expect("--format json must emit JSON")
        };

        assert!(
            parse(&none)["security"].is_null(),
            "flag not passed => section absent"
        );
        assert_eq!(
            parse(&ran_clean)["security"],
            serde_json::json!([]),
            "flag passed with nothing found => empty section, not null"
        );

        let doc = parse(&found);
        assert_eq!(doc["security"][0]["severity"], "Critical");
        assert_eq!(doc["security"][0]["message"], "Invalid WASM magic number");
        assert_eq!(doc["memory"][0]["category"], "linear-memory");
    }

    /// A `Critical` security issue must not be filed as `informational` beside
    /// the metrics: SARIF is read by tools that key off `level`.
    #[test]
    fn sarif_gives_a_security_finding_a_real_level() {
        let sections = WasmSections {
            security: Some(vec![finding()]),
            memory: None,
            complexity: None,
        };
        let doc: serde_json::Value = serde_json::from_str(
            &format_webassembly_results(
                &wasm_one(),
                &ComplexityOutputFormat::Sarif,
                false,
                D,
                0,
                &sections,
            )
            .unwrap(),
        )
        .expect("sarif is JSON");

        let results = doc["runs"][0]["results"].as_array().expect("results");
        let critical = results
            .iter()
            .find(|r| r["ruleId"] == "wasm-analysis-finding")
            .expect("the finding must be a SARIF result");
        assert_eq!(critical["level"], "error");
        assert_eq!(critical["kind"], "fail");
        assert!(
            doc["runs"][0]["tool"]["driver"]["rules"]
                .as_array()
                .expect("rules")
                .iter()
                .any(|r| r["id"] == "wasm-analysis-finding"),
            "a SARIF result must reference a declared rule: {doc}"
        );
    }

    /// The AssemblyScript report has the same two sections and the same bug.
    #[test]
    fn assemblyscript_findings_change_every_format() {
        let results = vec![(
            PathBuf::from("assembly/mem.ts"),
            WasmComplexity {
                cyclomatic: 5,
                cognitive: 5,
                memory_pressure: 1.0,
                hot_path_score: 10.0,
                estimated_gas: 5000.0,
                indirect_call_overhead: 1.0,
                max_loop_depth: 1,
            },
        )];
        let none = WasmSections::default();
        let sections = WasmSections {
            security: Some(vec![WasmFinding {
                file: PathBuf::from("assembly/mem.ts"),
                severity: "Medium".to_string(),
                category: "MemorySafety".to_string(),
                message: "line 3: load<T>() reads linear memory directly".to_string(),
            }]),
            memory: Some(vec![WasmFinding::info(
                Path::new("assembly/mem.ts"),
                "memory-sites",
                "memory.grow: 1".to_string(),
            )]),
            complexity: None,
        };

        for fmt in all_formats() {
            let plain = format_assemblyscript_results(&results, &fmt, false, D, 0, &none).unwrap();
            let with =
                format_assemblyscript_results(&results, &fmt, false, D, 0, &sections).unwrap();
            assert_ne!(plain, with, "{fmt:?} did not move");
            assert!(with.contains("load<T>()"), "{fmt:?}:\n{with}");
        }
    }
}
