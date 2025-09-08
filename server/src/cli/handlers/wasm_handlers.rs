//! WebAssembly and AssemblyScript analysis handlers
//!
//! This module contains handlers for WebAssembly binary/text format and
//! AssemblyScript source code analysis.

use crate::cli::ComplexityOutputFormat;
use crate::models::unified_ast::AstDag;
use crate::services::wasm::{
    AssemblyScriptParser, WasmBinaryAnalyzer, WasmComplexity, WasmComplexityAnalyzer,
    WasmLanguageDetector, WasmMetrics, WasmSecurityValidator, WatParser,
};
use anyhow::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Handle AssemblyScript analysis
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
        println!("{}", output_text);
    }
    Ok(())
}

/// Handle WebAssembly analysis
#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_webassembly(
    project_path: PathBuf,
    format: ComplexityOutputFormat,
    include_binary: bool,
    include_text: bool,
    _memory_analysis: bool,
    security: bool,
    complexity: bool,
    output: Option<PathBuf>,
    perf: bool,
) -> Result<()> {
    eprintln!("🔍 Analyzing WebAssembly files...");
    let start = std::time::Instant::now();

    let wasm_files = collect_wasm_files(&project_path, include_binary, include_text)?;
    eprintln!("📁 Found {} WebAssembly files", wasm_files.len());

    let results = analyze_wasm_files(
        wasm_files,
        include_binary,
        include_text,
        security,
        complexity,
    )
    .await;

    let elapsed = start.elapsed();
    eprintln!("📊 Analysis complete in {:.2}s", elapsed.as_secs_f64());

    write_wasm_analysis_output(&results, &format, perf, elapsed, output).await?;

    Ok(())
}

/// Analyze WASM files based on type and flags (cognitive complexity ≤8)
async fn analyze_wasm_files(
    wasm_files: Vec<PathBuf>,
    include_binary: bool,
    include_text: bool,
    security: bool,
    complexity: bool,
) -> Vec<(PathBuf, WasmMetrics)> {
    let mut results = Vec::new();

    for file_path in wasm_files {
        if let Some(result) = analyze_single_wasm_file(
            &file_path,
            include_binary,
            include_text,
            security,
            complexity,
        )
        .await
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
    security: bool,
    complexity: bool,
) -> Option<(PathBuf, WasmMetrics)> {
    match file_path.extension().and_then(|s| s.to_str()) {
        Some("wasm") if include_binary => analyze_wasm_binary(file_path).await,
        Some("wat") if include_text => {
            analyze_wat_text(file_path, security, complexity).await;
            None // WAT analysis doesn't return WasmMetrics currently
        }
        _ => None,
    }
}

/// Analyze WASM binary file (cognitive complexity ≤3)
async fn analyze_wasm_binary(file_path: &Path) -> Option<(PathBuf, WasmMetrics)> {
    let analyzer = WasmBinaryAnalyzer::new();
    match analyzer.analyze_file(file_path).await {
        Ok(analysis) => {
            eprintln!("✅ Analyzed binary: {}", file_path.display());
            Some((file_path.to_path_buf(), analysis))
        }
        Err(e) => {
            eprintln!("❌ Failed to analyze {}: {}", file_path.display(), e);
            None
        }
    }
}

/// Analyze WAT text file (cognitive complexity ≤6)
async fn analyze_wat_text(file_path: &Path, security: bool, complexity: bool) {
    if let Ok(content) = tokio::fs::read_to_string(file_path).await {
        let mut parser = WatParser::new();
        match parser.parse(&content) {
            Ok(ast) => {
                eprintln!("✅ Parsed WAT: {}", file_path.display());
                process_wat_ast(&ast, file_path, security, complexity);
            }
            Err(e) => {
                eprintln!("❌ Failed to parse {}: {}", file_path.display(), e);
            }
        }
    }
}

/// Process WAT AST for security and complexity (cognitive complexity ≤5)
fn process_wat_ast(ast: &AstDag, file_path: &Path, security: bool, complexity: bool) {
    if complexity {
        let complexity_analyzer = WasmComplexityAnalyzer::new();
        let _ = complexity_analyzer.analyze_ast(ast);
    }

    if security {
        let security_validator = WasmSecurityValidator::new();
        if let Err(e) = security_validator.validate_ast(ast) {
            eprintln!("⚠️  Security issue in {}: {}", file_path.display(), e);
        }
    }
}

/// Write WASM analysis output (cognitive complexity ≤4)
async fn write_wasm_analysis_output(
    results: &[(PathBuf, WasmMetrics)],
    format: &ComplexityOutputFormat,
    perf: bool,
    elapsed: std::time::Duration,
    output: Option<PathBuf>,
) -> Result<()> {
    let output_text = format_webassembly_results(results, format, perf, elapsed)?;

    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &output_text).await?;
        eprintln!("📝 Results written to: {}", output_path.display());
    } else {
        println!("{}", output_text);
    }

    Ok(())
}

/// Collect AssemblyScript files (.as, .ts with AS context)
fn collect_assemblyscript_files(project_path: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(project_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            match ext {
                "as" => files.push(path.to_path_buf()),
                "ts" => {
                    // Check if TypeScript file is actually AssemblyScript
                    if let Ok(content) = std::fs::read_to_string(path) {
                        if content.contains("@global")
                            || content.contains("@inline")
                            || content.contains("i32")
                            || content.contains("f64")
                            || content.contains("memory.")
                        {
                            files.push(path.to_path_buf());
                        }
                    }
                }
                _ => {}
            }
        }
    }

    Ok(files)
}

/// Collect WebAssembly files (.wasm, .wat)
fn collect_wasm_files(
    project_path: &Path,
    include_binary: bool,
    include_text: bool,
) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(project_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            match ext {
                "wasm" if include_binary => files.push(path.to_path_buf()),
                "wat" if include_text => files.push(path.to_path_buf()),
                _ => {}
            }
        }
    }

    Ok(files)
}

/// Format AssemblyScript analysis results
fn format_assemblyscript_results(
    results: &[(PathBuf, WasmComplexity)],
    format: &ComplexityOutputFormat,
    perf: bool,
    elapsed: std::time::Duration,
) -> Result<String> {
    match format {
        ComplexityOutputFormat::Json => {
            let output = serde_json::json!({
                "analysis_type": "assemblyscript",
                "files_analyzed": results.len(),
                "results": results.iter().map(|(path, complexity)| {
                    serde_json::json!({
                        "file": path.display().to_string(),
                        "complexity": complexity
                    })
                }).collect::<Vec<_>>(),
                "performance": if perf {
                    Some(serde_json::json!({
                        "total_time_ms": elapsed.as_millis(),
                        "avg_time_per_file_ms": elapsed.as_millis() / (results.len() as u128).max(1)
                    }))
                } else { None }
            });
            Ok(serde_json::to_string_pretty(&output)?)
        }
        _ => {
            let mut output = String::from("# AssemblyScript Analysis Report\n\n");
            output.push_str(&format!("📁 **Files analyzed**: {}\n", results.len()));
            output.push_str(&format!(
                "⏱️  **Analysis time**: {:.2}s\n\n",
                elapsed.as_secs_f64()
            ));

            if !results.is_empty() {
                output.push_str("## Results\n\n");
                for (path, complexity) in results {
                    output.push_str(&format!("### {}\n", path.display()));
                    output.push_str(&format!(
                        "- **Cyclomatic complexity**: {}\n",
                        complexity.cyclomatic
                    ));
                    output.push_str(&format!(
                        "- **Cognitive complexity**: {}\n",
                        complexity.cognitive
                    ));
                    output.push_str(&format!(
                        "- **Memory pressure**: {:.2}\n\n",
                        complexity.memory_pressure
                    ));
                }
            }

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
) -> Result<String> {
    match format {
        ComplexityOutputFormat::Json => {
            let output = serde_json::json!({
                "analysis_type": "webassembly",
                "files_analyzed": results.len(),
                "results": results.iter().map(|(path, metrics)| {
                    serde_json::json!({
                        "file": path.display().to_string(),
                        "metrics": metrics
                    })
                }).collect::<Vec<_>>(),
                "performance": if perf {
                    Some(serde_json::json!({
                        "total_time_ms": elapsed.as_millis(),
                        "avg_time_per_file_ms": elapsed.as_millis() / (results.len() as u128).max(1)
                    }))
                } else { None }
            });
            Ok(serde_json::to_string_pretty(&output)?)
        }
        _ => {
            let mut output = String::from("# WebAssembly Analysis Report\n\n");
            output.push_str(&format!("📁 **Files analyzed**: {}\n", results.len()));
            output.push_str(&format!(
                "⏱️  **Analysis time**: {:.2}s\n\n",
                elapsed.as_secs_f64()
            ));

            if !results.is_empty() {
                output.push_str("## Results\n\n");
                for (path, metrics) in results {
                    output.push_str(&format!("### {}\n", path.display()));
                    output.push_str(&format!("- **Functions**: {}\n", metrics.function_count));
                    output.push_str(&format!("- **Imports**: {}\n", metrics.import_count));
                    output.push_str(&format!("- **Exports**: {}\n", metrics.export_count));
                    output.push_str(&format!(
                        "- **Memory pages**: {}\n\n",
                        metrics.linear_memory_pages
                    ));
                }
            }

            Ok(output)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_collect_assemblyscript_files() {
        let temp_dir = TempDir::new().unwrap();
        let as_file = temp_dir.path().join("test.as");
        let ts_file = temp_dir.path().join("assembly.ts");
        let other_file = temp_dir.path().join("test.txt");

        tokio::fs::write(&as_file, "function test(): i32 { return 42; }")
            .await
            .unwrap();
        tokio::fs::write(&ts_file, "const value: i32 = 42; @global let ptr: usize;")
            .await
            .unwrap();
        tokio::fs::write(&other_file, "not assemblyscript")
            .await
            .unwrap();

        let files = collect_assemblyscript_files(&temp_dir.path().to_path_buf()).unwrap();
        assert_eq!(files.len(), 2);
    }

    #[tokio::test]
    async fn test_collect_wasm_files() {
        let temp_dir = TempDir::new().unwrap();
        let wasm_file = temp_dir.path().join("test.wasm");
        let wat_file = temp_dir.path().join("test.wat");
        let other_file = temp_dir.path().join("test.txt");

        tokio::fs::write(&wasm_file, b"\0asm\x01\x00\x00\x00")
            .await
            .unwrap();
        tokio::fs::write(&wat_file, "(module)").await.unwrap();
        tokio::fs::write(&other_file, "not wasm").await.unwrap();

        let files = collect_wasm_files(&temp_dir.path().to_path_buf(), true, true).unwrap();
        assert_eq!(files.len(), 2);
    }
}
