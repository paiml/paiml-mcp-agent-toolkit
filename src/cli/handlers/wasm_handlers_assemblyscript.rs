/// Handle `AssemblyScript` analysis
#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_assemblyscript(
    project_path: PathBuf,
    format: ComplexityOutputFormat,
    wasm_complexity: bool,
    _memory_analysis: bool,
    security: bool,
    output: Option<PathBuf>,
    _timeout: u64,
    perf: bool,
) -> Result<()> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    eprintln!("🔍 Analyzing AssemblyScript code...");
    let start = std::time::Instant::now();

    let results = process_assemblyscript_files(&project_path, wasm_complexity, security).await?;
    let elapsed = start.elapsed();

    eprintln!("📊 Analysis complete in {:.2}s", elapsed.as_secs_f64());

    let output_text = format_assemblyscript_results(&results, &format, perf, elapsed)?;
    write_analysis_output(output_text, output).await?;

    Ok(())
}

async fn process_assemblyscript_files(
    project_path: &Path,
    wasm_complexity: bool,
    security: bool,
) -> Result<Vec<(PathBuf, WasmComplexity)>> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let detector = WasmLanguageDetector::new();
    let mut parser = AssemblyScriptParser::new()?;
    let mut results = Vec::new();

    let as_files = collect_assemblyscript_files(project_path)?;
    eprintln!("📁 Found {} AssemblyScript files", as_files.len());

    for file_path in as_files {
        if let Some(analysis_result) = analyze_single_file(
            &file_path,
            &detector,
            &mut parser,
            wasm_complexity,
            security,
        )
        .await?
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
    wasm_complexity: bool,
    security: bool,
) -> Result<Option<(PathBuf, WasmComplexity)>> {
    debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
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

    let result = process_parsed_ast(&ast, file_path, wasm_complexity, security)?;
    Ok(result)
}

fn process_parsed_ast(
    ast: &AstDag,
    file_path: &Path,
    wasm_complexity: bool,
    security: bool,
) -> Result<Option<(PathBuf, WasmComplexity)>> {
    debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
    let mut result = None;

    if wasm_complexity {
        let complexity_analyzer = WasmComplexityAnalyzer::new();
        let complexity = complexity_analyzer.analyze_ast(ast)?;
        result = Some((file_path.to_path_buf(), complexity));
    }

    if security {
        validate_ast_security(ast, file_path);
    }

    Ok(result)
}

fn validate_ast_security(ast: &AstDag, file_path: &Path) {
    debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
    let security_validator = WasmSecurityValidator::new();
    if let Err(e) = security_validator.validate_ast(ast) {
        eprintln!("⚠️  Security issue in {}: {}", file_path.display(), e);
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
