/// Handle `AssemblyScript` analysis
#[allow(clippy::too_many_arguments)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_analyze_assemblyscript(
    project_path: PathBuf,
    format: ComplexityOutputFormat,
    // --wasm-complexity used to decide whether a parsed file appeared in the
    // report at all, which is why the default run reported files_analyzed 0.
    // Complexity is now measured for every parsed file and the flag has nothing
    // left to gate; it is kept so existing invocations keep working.
    _wasm_complexity: bool,
    memory_analysis: bool,
    security: bool,
    output: Option<PathBuf>,
    _timeout: u64,
    perf: bool,
    top_files: usize,
) -> Result<()> {
    // Found alongside GH-663/GH-666: a nonexistent path printed
    // "📁 Found 0 AssemblyScript files" and a complete report with
    // "**Files analyzed**: 0", exit 0.
    crate::cli::ensure_analysis_path_exists(&project_path)?;

    use crate::cli::colors as c;
    crate::status_eprintln!("🔍 {}", c::label("Analyzing AssemblyScript code..."));
    let start = std::time::Instant::now();

    // Both flags used to write their findings to stderr with a bare
    // `eprintln!`, leaving the report on stdout byte-identical with and without
    // them. They are report sections now, so `--output`, a pipe and
    // `--format json` all carry what the rules found.
    let mut sections = WasmSections {
        security: security.then(Vec::new),
        memory: memory_analysis.then(Vec::new),
        complexity: None,
    };

    let results = process_assemblyscript_files(&project_path, &mut sections).await?;
    let elapsed = start.elapsed();

    crate::status_eprintln!(
        "📊 Analysis complete in {}",
        c::number(&format!("{:.2}s", elapsed.as_secs_f64()))
    );

    let output_text =
        format_assemblyscript_results(&results, &format, perf, elapsed, top_files, &sections)?;
    write_analysis_output(output_text, output).await?;

    Ok(())
}

async fn process_assemblyscript_files(
    project_path: &Path,
    sections: &mut WasmSections,
) -> Result<Vec<(PathBuf, WasmComplexity)>> {
    let detector = WasmLanguageDetector::new();
    let mut parser = AssemblyScriptParser::new()?;
    let mut results = Vec::new();

    let as_files = collect_assemblyscript_files(project_path)?;
    crate::status_eprintln!(
        "📁 Found {} AssemblyScript files",
        crate::cli::colors::number(&as_files.len().to_string())
    );

    for file_path in as_files {
        if let Some(analysis_result) =
            analyze_single_file(&file_path, &detector, &mut parser, sections).await?
        {
            results.push(analysis_result);
        }
    }

    Ok(results)
}

async fn analyze_single_file(
    file_path: &Path,
    detector: &WasmLanguageDetector,
    parser: &mut AssemblyScriptParser,
    sections: &mut WasmSections,
) -> Result<Option<(PathBuf, WasmComplexity)>> {
    let content = match tokio::fs::read_to_string(file_path).await {
        Ok(content) => content,
        Err(_) => return Ok(None),
    };

    if !detector.is_assemblyscript(&content) {
        return Ok(None);
    }

    let ast = match parser.parse_file(file_path, &content).await {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!(
                "{}",
                crate::cli::colors::colored(
                    crate::cli::colors::RED,
                    &format!("❌ Failed to parse {}: {}", file_path.display(), e)
                )
            );
            return Ok(None);
        }
    };

    crate::status_eprintln!(
        "✅ Parsed: {}",
        crate::cli::colors::path(&file_path.display().to_string())
    );

    let result = process_parsed_ast(&ast, &content, file_path, sections)?;
    Ok(result)
}

/// Build the report row for one parsed file.
///
/// This used to return `None` unless `--wasm-complexity` was passed, and
/// `files_analyzed` is `results.len()`, so the DEFAULT run printed
/// "📁 Found 2 AssemblyScript files" and two "✅ Parsed:" lines and then
/// reported `files_analyzed: 0, results: []` — a report contradicting its own
/// progress output. Every file we successfully parsed gets a row; the
/// complexity is measured from that same AST rather than left out or defaulted.
fn process_parsed_ast(
    ast: &AstDag,
    content: &str,
    file_path: &Path,
    sections: &mut WasmSections,
) -> Result<Option<(PathBuf, WasmComplexity)>> {
    let complexity_analyzer = WasmComplexityAnalyzer::new();
    let complexity = complexity_analyzer.analyze_ast(ast)?;

    if let Some(rows) = sections.security.as_mut() {
        rows.extend(source_security_findings(content, file_path));
    }

    if let Some(rows) = sections.memory.as_mut() {
        rows.push(source_memory_finding(content, file_path));
    }

    Ok(Some((file_path.to_path_buf(), complexity)))
}

/// Security findings for one `AssemblyScript` source file.
///
/// This used to be `validate_ast_security`, which called
/// `WasmSecurityValidator::validate_ast` — `Ok(())` for every input — over an
/// `AstDag` that `AssemblyScriptParser::parse_file` returns EMPTY. The
/// `if let Err(e)` branch that would have printed a finding was unreachable for
/// every possible file, so `--security` was byte-identical to no flag on
/// sources using `memory.grow` and raw `load<T>()`. The rules now run over the
/// source that was read from disk, and "nothing found" is reported as a result
/// instead of as silence.
fn source_security_findings(content: &str, file_path: &Path) -> Vec<WasmFinding> {
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

/// Memory constructs counted in one `AssemblyScript` source file.
///
/// `--memory-analysis` was `_memory_analysis: bool` in the handler signature —
/// parsed, forwarded, never read. The counts below are occurrences in the file
/// that was read, nothing else: in particular the report's "Memory pressure"
/// line is NOT used here, because `WasmComplexityAnalyzer::analyze_ast` returns
/// the same constant (`memory_pressure: 1.0`) for every input.
fn source_memory_finding(content: &str, file_path: &Path) -> WasmFinding {
    const SITES: &[(&str, &str)] = &[
        ("memory.grow", "memory.grow"),
        ("load<", "raw load<T>()"),
        ("store<", "raw store<T>()"),
        ("changetype<", "changetype<>"),
        ("new ", "`new` allocation"),
    ];

    let counts: Vec<String> = SITES
        .iter()
        .map(|(needle, label)| {
            let count = content.matches(needle).count();
            format!("{label}: {count}")
        })
        .collect();

    WasmFinding::info(file_path, "memory-sites", counts.join(", "))
}

#[cfg(test)]
mod assemblyscript_default_run_tests {
    //! The default run must report the files it says it parsed.
    use super::*;

    const FIXTURE: &str = "export function add(a: i32, b: i32): i32 {\n  return a + b;\n}\n";

    /// Without `--wasm-complexity`, every parsed file was dropped, so
    /// `files_analyzed` (= results.len()) was 0 while stderr had just printed
    /// "Found 2 AssemblyScript files" and two "✅ Parsed:" lines.
    #[tokio::test]
    async fn test_default_run_yields_one_result_per_parsed_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join("assembly")).expect("mkdir assembly");
        std::fs::write(dir.path().join("assembly/index.ts"), FIXTURE).expect("write index.ts");
        std::fs::write(dir.path().join("top.ts"), FIXTURE).expect("write top.ts");

        let found = collect_assemblyscript_files(dir.path()).expect("collect");
        assert_eq!(
            found.len(),
            2,
            "fixture should present 2 AssemblyScript files"
        );

        // No flags at all, and no --wasm-complexity anywhere in the call.
        let mut sections = WasmSections::default();
        let results = process_assemblyscript_files(dir.path(), &mut sections)
            .await
            .expect("analysis");

        assert_eq!(
            results.len(),
            found.len(),
            "reported {} results for {} parsed files",
            results.len(),
            found.len()
        );
    }
}

async fn write_analysis_output(output_text: String, output_path: Option<PathBuf>) -> Result<()> {
    if let Some(output_path) = output_path {
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
