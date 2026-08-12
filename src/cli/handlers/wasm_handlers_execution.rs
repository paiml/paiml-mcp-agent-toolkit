/// Handle WebAssembly analysis
#[allow(clippy::too_many_arguments)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_analyze_webassembly(
    project_path: PathBuf,
    format: ComplexityOutputFormat,
    include_binary: bool,
    include_text: bool,
    memory_analysis: bool,
    security: bool,
    complexity: bool,
    output: Option<PathBuf>,
    perf: bool,
    top_files: usize,
) -> Result<()> {
    // Found alongside GH-663/GH-666: a nonexistent path printed
    // "📁 Found 0 WebAssembly files" and a complete report with
    // "**Files analyzed**: 0", exit 0.
    crate::cli::ensure_analysis_path_exists(&project_path)?;

    use crate::cli::colors as c;
    crate::status_eprintln!("🔍 {}", c::label("Analyzing WebAssembly files..."));
    let start = std::time::Instant::now();

    let wasm_files = collect_wasm_files(&project_path, include_binary, include_text)?;
    crate::status_eprintln!(
        "📁 Found {} WebAssembly files",
        c::number(&wasm_files.len().to_string())
    );

    // `security`/`complexity` findings are collected, not printed: they used to
    // go to stderr with a bare `eprintln!` while the report on stdout stayed
    // byte-identical with and without the flag, so nothing a caller redirected,
    // piped or parsed could see them.
    let mut sections = WasmSections {
        security: security.then(Vec::new),
        complexity: complexity.then(Vec::new),
        memory: None,
    };

    let results = analyze_wasm_files(wasm_files, include_binary, include_text, &mut sections).await;

    // `--memory-analysis` was `_memory_analysis: bool` — the router parsed it
    // and forwarded it into a parameter nothing read, so the flag was
    // byte-identical to no flag on a fixture whose module declares 2 pages of
    // linear memory. The facts below come from the decoded Memory section, the
    // same measurement `Memory pages` in the report is built from.
    if memory_analysis {
        sections.memory = Some(memory_analysis_findings(&results));
    }

    let elapsed = start.elapsed();
    crate::status_eprintln!(
        "📊 Analysis complete in {}",
        c::number(&format!("{:.2}s", elapsed.as_secs_f64()))
    );

    write_wasm_analysis_output(
        &results, &format, perf, elapsed, output, top_files, &sections,
    )
    .await?;

    Ok(())
}

/// Analyze WASM files based on type and flags (cognitive complexity ≤8)
async fn analyze_wasm_files(
    wasm_files: Vec<PathBuf>,
    include_binary: bool,
    include_text: bool,
    sections: &mut WasmSections,
) -> Vec<(PathBuf, WasmMetrics)> {
    let mut results = Vec::new();

    for file_path in wasm_files {
        if let Some(result) =
            analyze_single_wasm_file(&file_path, include_binary, include_text, sections).await
        {
            results.push(result);
        }
    }

    results
}

/// Analyze a single WASM file (cognitive complexity ≤7)
async fn analyze_single_wasm_file(
    file_path: &Path,
    include_binary: bool,
    include_text: bool,
    sections: &mut WasmSections,
) -> Option<(PathBuf, WasmMetrics)> {
    match file_path.extension().and_then(|s| s.to_str()) {
        Some("wasm") if include_binary => {
            // `--security` never reached a .wasm file at all: this arm did not
            // even take `security`, and the only thing the flag did reach was
            // `WasmSecurityValidator::validate_ast`, a constant `Ok(())`. The
            // real detector — `validate(&[u8])`, which flags a bad magic number
            // and oversized modules and has had tests the whole time — had zero
            // production callers.
            if let Some(rows) = sections.security.as_mut() {
                rows.extend(wasm_binary_security_findings(file_path).await);
            }
            analyze_wasm_binary(file_path).await
        }
        Some("wat") if include_text => {
            analyze_wat_text(file_path, sections).await;
            // A parsed .wat contributes no row, so a .wat-only directory
            // reported "Found 1 WebAssembly files" on stderr and then
            // "**Files analyzed**: 0" with no Results section and exit 0 — the
            // file was silently dropped. It still is (the WAT front end
            // produces no `WasmMetrics`; emitting a zero-filled row would
            // report measurements nothing took), but say so instead of
            // leaving the gap to be read as "nothing found".
            eprintln!(
                "{}",
                crate::cli::colors::colored(
                    crate::cli::colors::YELLOW,
                    &format!(
                        "⚠️  Not reported: {} — .wat text format is parsed for \
                         security/complexity checks only; metrics come from binary .wasm files",
                        file_path.display()
                    )
                )
            );
            None
        }
        _ => None,
    }
}

/// Analyze WASM binary file (cognitive complexity ≤3)
async fn analyze_wasm_binary(file_path: &Path) -> Option<(PathBuf, WasmMetrics)> {
    let analyzer = WasmBinaryAnalyzer::new();
    match analyzer.analyze_file(file_path).await {
        Ok(analysis) => {
            crate::status_eprintln!(
                "✅ Analyzed binary: {}",
                crate::cli::colors::path(&file_path.display().to_string())
            );
            Some((file_path.to_path_buf(), analysis))
        }
        Err(e) => {
            eprintln!(
                "{}",
                crate::cli::colors::colored(
                    crate::cli::colors::RED,
                    &format!("❌ Failed to analyze {}: {}", file_path.display(), e)
                )
            );
            None
        }
    }
}

/// Analyze WAT text file (cognitive complexity ≤6)
async fn analyze_wat_text(file_path: &Path, sections: &mut WasmSections) {
    if let Ok(content) = tokio::fs::read_to_string(file_path).await {
        let mut parser = WatParser::new();
        match parser.parse(&content) {
            Ok(_ast) => {
                crate::status_eprintln!(
                    "✅ Parsed WAT: {}",
                    crate::cli::colors::path(&file_path.display().to_string())
                );
                collect_wat_findings(&content, file_path, sections);
            }
            Err(e) => {
                eprintln!(
                    "{}",
                    crate::cli::colors::colored(
                        crate::cli::colors::RED,
                        &format!("❌ Failed to parse {}: {}", file_path.display(), e)
                    )
                );
            }
        }
    }
}

/// Collect the `--security` and `--complexity` rows for one text-format file.
///
/// The security half used to run `WasmSecurityValidator::validate_ast(ast)`, a
/// constant `Ok(())`, over an `AstDag` that `WatParser::parse` returns empty —
/// two independent reasons `--security` could never report anything. It runs
/// over the WAT source text, which is what was actually read from disk.
///
/// The complexity half used to be `let _ = analyzer.analyze_ast(ast);` — the
/// value was bound to `_` and dropped, so `--complexity` was byte-identical to
/// no flag. It now uses [`WasmComplexityAnalyzer::analyze_text`], which derives
/// its numbers from that same source; `analyze_ast` is NOT used, because it
/// returns the same constant (`cyclomatic: 5, cognitive: 5`) for every input
/// and reporting it would report a measurement nothing took.
fn collect_wat_findings(content: &str, file_path: &Path, sections: &mut WasmSections) {
    if let Some(rows) = sections.complexity.as_mut() {
        let analyzer = WasmComplexityAnalyzer::new();
        if let Ok(cx) = analyzer.analyze_text(content) {
            rows.push(WasmFinding::info(
                file_path,
                "text-module-complexity",
                format!(
                    "cyclomatic {}, cognitive {}, max loop depth {}, estimated gas {:.0}",
                    cx.cyclomatic, cx.cognitive, cx.max_loop_depth, cx.estimated_gas
                ),
            ));
        }
    }

    if let Some(rows) = sections.security.as_mut() {
        rows.extend(text_security_findings(content, file_path));
    }
}

/// Security rows for one text-format file, or one row saying nothing matched.
///
/// Silence is what made `--security` indistinguishable from no flag; "no issue
/// found" is a result and is reported as one. It is scoped to the rules
/// [`WasmSecurityValidator::validate_text`] documents, never to "this module is
/// safe".
fn text_security_findings(content: &str, file_path: &Path) -> Vec<WasmFinding> {
    let validator = WasmSecurityValidator::new();
    let Ok(validation) = validator.validate_text(content) else {
        return Vec::new();
    };
    security_findings(
        file_path,
        &validation,
        "no issue found by the memory/resource rules",
    )
}

/// Run the real binary security rules over a `.wasm` module.
///
/// `WasmSecurityValidator::validate(&[u8])` — bad magic number, file too small,
/// oversized module — existed and was tested but had no production caller, so
/// `--security` never inspected a single binary. Findings are reported here
/// rather than suppressed when the module also fails to decode: an unparseable
/// module is precisely the one worth reporting.
async fn wasm_binary_security_findings(file_path: &Path) -> Vec<WasmFinding> {
    let Ok(bytes) = tokio::fs::read(file_path).await else {
        return Vec::new();
    };

    let validator = WasmSecurityValidator::new();
    let Ok(validation) = validator.validate(&bytes) else {
        return Vec::new();
    };
    security_findings(
        file_path,
        &validation,
        "no issue found by the binary format/size rules",
    )
}

/// Turn one validation into report rows, keeping the clean case visible.
fn security_findings(
    file_path: &Path,
    validation: &crate::services::wasm::security::SecurityValidation,
    clean_message: &str,
) -> Vec<WasmFinding> {
    if validation.passed {
        return vec![WasmFinding::info(
            file_path,
            "security",
            clean_message.to_string(),
        )];
    }

    validation
        .issues
        .iter()
        .map(|issue| WasmFinding {
            file: file_path.to_path_buf(),
            severity: format!("{:?}", issue.severity),
            category: format!("{:?}", issue.category),
            message: issue.description.clone(),
        })
        .collect()
}

/// Bytes of linear memory one WebAssembly page reserves.
const WASM_PAGE_BYTES: u64 = 64 * 1024;

/// Linear-memory facts for every analysed module, as report rows.
///
/// Only decoded quantities are reported: the number of Memory sections and the
/// initial page count the Memory section declares. `WasmMetrics::memory_operations`
/// and `instruction_histogram` are still `Default::default()` (the binary
/// front end does not walk code bodies), so they are NOT reported — printing
/// zeros for them would be reporting a measurement nothing took.
fn memory_analysis_findings(results: &[(PathBuf, WasmMetrics)]) -> Vec<WasmFinding> {
    results
        .iter()
        .map(|(path, metrics)| {
            if metrics.memory_sections == 0 {
                return WasmFinding::info(
                    path,
                    "linear-memory",
                    "declares no linear memory".to_string(),
                );
            }

            let bytes = u64::from(metrics.linear_memory_pages) * WASM_PAGE_BYTES;
            WasmFinding::info(
                path,
                "linear-memory",
                format!(
                    "{} memory section(s), {} initial page(s) = {} KiB reserved",
                    metrics.memory_sections,
                    metrics.linear_memory_pages,
                    bytes / 1024
                ),
            )
        })
        .collect()
}

/// Write WASM analysis output (cognitive complexity ≤4)
async fn write_wasm_analysis_output(
    results: &[(PathBuf, WasmMetrics)],
    format: &ComplexityOutputFormat,
    perf: bool,
    elapsed: std::time::Duration,
    output: Option<PathBuf>,
    top_files: usize,
    sections: &WasmSections,
) -> Result<()> {
    let output_text =
        format_webassembly_results(results, format, perf, elapsed, top_files, sections)?;

    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &output_text).await?;
        crate::status_eprintln!(
            "📝 Results written to: {}",
            crate::cli::colors::path(&output_path.display().to_string())
        );
    } else {
        println!("{output_text}");
    }

    Ok(())
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod wasm_flag_wiring_tests {
    //! The flags must reach the rules AND land in `WasmSections`.
    use super::*;

    const UNBOUNDED_WAT: &str = "(module\n  (memory 2)\n  (func $add (param i32 i32) (result i32)\n    local.get 0\n    local.get 1\n    i32.add)\n  (export \"add\" (func $add)))\n";

    fn fixture() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("mod.wat"), UNBOUNDED_WAT).expect("write wat");
        std::fs::write(dir.path().join("broken.wasm"), b"NOTWASM!________").expect("write wasm");
        dir
    }

    /// `--complexity` was `let _ = complexity_analyzer.analyze_ast(ast);` — the
    /// value was bound to `_` and dropped, so the flag could not change any
    /// output for any input. It now yields a row, measured from the WAT source
    /// (`analyze_ast` returns the same constant for every input and is not
    /// used).
    #[tokio::test]
    async fn complexity_flag_yields_a_row_for_a_text_module() {
        let dir = fixture();
        let files = vec![dir.path().join("mod.wat")];

        let mut off = WasmSections::default();
        analyze_wasm_files(files.clone(), true, true, &mut off).await;
        assert!(off.complexity.is_none(), "no flag => no section");

        let mut on = WasmSections {
            complexity: Some(Vec::new()),
            ..WasmSections::default()
        };
        analyze_wasm_files(files, true, true, &mut on).await;

        let rows = on.complexity.expect("section present");
        assert_eq!(
            rows.len(),
            1,
            "one text module, one complexity row: {rows:?}"
        );
        assert!(
            rows[0].message.contains("cyclomatic"),
            "the row must carry the measurement: {rows:?}"
        );
    }

    /// `--security` must produce rows for BOTH front ends: the text rules over
    /// the .wat source and the binary rules over the .wasm bytes. The binary
    /// half had no production caller at all, so a module with a bad magic
    /// number went unreported.
    #[tokio::test]
    async fn security_flag_collects_text_and_binary_findings() {
        let dir = fixture();
        let files = vec![dir.path().join("mod.wat"), dir.path().join("broken.wasm")];

        let mut sections = WasmSections {
            security: Some(Vec::new()),
            ..WasmSections::default()
        };
        analyze_wasm_files(files, true, true, &mut sections).await;

        let rows = sections.security.expect("section present");
        assert!(
            rows.iter().any(|r| r.message.contains("without bound")),
            "the .wat memory rule must fire: {rows:?}"
        );
        assert!(
            rows.iter()
                .any(|r| r.severity == "Critical" && r.message.contains("magic number")),
            "the .wasm binary rule must fire: {rows:?}"
        );
    }

    /// `--memory-analysis` reports only decoded quantities, and a module with
    /// no Memory section says so rather than reporting a zero it never read.
    #[test]
    fn memory_findings_are_decoded_facts() {
        let rows = memory_analysis_findings(&[
            (
                PathBuf::from("with.wasm"),
                WasmMetrics {
                    memory_sections: 1,
                    linear_memory_pages: 2,
                    ..WasmMetrics::default()
                },
            ),
            (PathBuf::from("without.wasm"), WasmMetrics::default()),
        ]);

        assert_eq!(rows.len(), 2);
        assert!(rows[0].message.contains("128 KiB reserved"), "{rows:?}");
        assert!(
            rows[1].message.contains("declares no linear memory"),
            "{rows:?}"
        );
    }
}
