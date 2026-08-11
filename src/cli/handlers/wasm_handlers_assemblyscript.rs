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
    _memory_analysis: bool,
    security: bool,
    output: Option<PathBuf>,
    _timeout: u64,
    perf: bool,
) -> Result<()> {
    // Found alongside GH-663/GH-666: a nonexistent path printed
    // "📁 Found 0 AssemblyScript files" and a complete report with
    // "**Files analyzed**: 0", exit 0.
    crate::cli::ensure_analysis_path_exists(&project_path)?;

    eprintln!("🔍 Analyzing AssemblyScript code...");
    let start = std::time::Instant::now();

    let results = process_assemblyscript_files(&project_path, security).await?;
    let elapsed = start.elapsed();

    eprintln!("📊 Analysis complete in {:.2}s", elapsed.as_secs_f64());

    let output_text = format_assemblyscript_results(&results, &format, perf, elapsed)?;
    write_analysis_output(output_text, output).await?;

    Ok(())
}

async fn process_assemblyscript_files(
    project_path: &Path,
    security: bool,
) -> Result<Vec<(PathBuf, WasmComplexity)>> {
    let detector = WasmLanguageDetector::new();
    let mut parser = AssemblyScriptParser::new()?;
    let mut results = Vec::new();

    let as_files = collect_assemblyscript_files(project_path)?;
    eprintln!("📁 Found {} AssemblyScript files", as_files.len());

    for file_path in as_files {
        if let Some(analysis_result) =
            analyze_single_file(&file_path, &detector, &mut parser, security).await?
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
    security: bool,
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
            eprintln!("❌ Failed to parse {}: {}", file_path.display(), e);
            return Ok(None);
        }
    };

    eprintln!("✅ Parsed: {}", file_path.display());

    let result = process_parsed_ast(&ast, file_path, security)?;
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
    file_path: &Path,
    security: bool,
) -> Result<Option<(PathBuf, WasmComplexity)>> {
    let complexity_analyzer = WasmComplexityAnalyzer::new();
    let complexity = complexity_analyzer.analyze_ast(ast)?;

    if security {
        validate_ast_security(ast, file_path);
    }

    Ok(Some((file_path.to_path_buf(), complexity)))
}

fn validate_ast_security(ast: &AstDag, file_path: &Path) {
    let security_validator = WasmSecurityValidator::new();
    if let Err(e) = security_validator.validate_ast(ast) {
        eprintln!("⚠️  Security issue in {}: {}", file_path.display(), e);
    }
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
        assert_eq!(found.len(), 2, "fixture should present 2 AssemblyScript files");

        // security=false, and no --wasm-complexity anywhere in the call.
        let results = process_assemblyscript_files(dir.path(), false)
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
        eprintln!("📝 Results written to: {}", output_path.display());
    } else {
        println!("{output_text}");
    }
    Ok(())
}
