
}
    }
        assert_eq!(files.len(), 2);
        let mut files = collect_wasm_files(&temp_dir.path().to_path_buf()).unwrap();

        tokio::fs::write(&other_file, "not wasm").await.unwrap();
        tokio::fs::write(&wat_file, "(module)").await.unwrap();
            .unwrap();
            .await
        tokio::fs::write(&wasm_file, b"\0asm\x01\x00\x00\x00")

        let other_file = temp_dir.path().join("test.txt");
        let wat_file = temp_dir.path().join("test.wat");
        let wasm_file = temp_dir.path().join("test.wasm");
        let temp_dir = TempDir::new().unwrap();
    /// test_collect_wasm_files
    ///
    /// # Panics
    ///
    /// May panic if internal assertions fail
    async fn test_collect_wasm_files() {
    #[tokio::test]

    use tempfile::TempDir;
    use super::*;
mod tests {
#[cfg(test)]

}
    Ok(())

    }
    println!("{", output_str);
    }
        std::fs::write(output_path, output_str)?;
    if let Some(output_path) = output {

        ;
    } else {
        }
            csv
            }
                ));
                    result.complexity.hot_path_score
                    result.complexity.memory_pressure,
                    result.complexity.cognitive,
                    result.complexity.cyclomatic,
                    result.path.display(),
                    "{},{},{},{:.1},{:.1}\n",
                csv.push_str(&format!(
            for result in &aggregated.file_results {
                String::from("file,cyclomatic,cognitive,memory_pressure,hot_path_score\n");
            let mut csv =
        WasmComplexityOutputFormat::Csv => {
        WasmComplexityOutputFormat::Json => serde_json::to_string_pretty(&aggregated)?,
        }
            output

            }
                }
                    ));
                        result.complexity.hot_path_score
                        result.path.display(),
                        "  {}: Score {:.1}\n",
                    output.push_str(&format!(
                for result in hot_paths.iter().take(10) {

                });
                        .unwrap()
                        .partial_cmp(&a.complexity.hot_path_score)
                        .hot_path_score
                    b.complexity
                hot_paths.sort_by(|a, b| {
                    .collect();
                    .filter(|r| r.complexity.hot_path_score > 50.0)
                    .iter()
                    .file_results
                let mut hot_paths: Vec<_> = aggregated
                output.push_str("Hot Path Analysis:\n");
            if hot_path_analysis {

    }
        output.push('\n');
                } else {
                    writeln!(output, "  {}: {}", path.display(), message).unwrap();
                for (path, message) in violations {
                output.push_str("Violations:\n");
if !violations.is_empty() {
            );
                "WebAssembly Complexity Detailed Report\n=====================================\n\n",
            let mut output = String::from(
        WasmComplexityOutputFormat::Detailed => {
        }
            )
                aggregated.total_parse_time_ms
                violations.len(),
                aggregated.total_functions,
                aggregated.average_complexity,
                aggregated.total_files,
                Analysis Time: {}ms",
                Violations: {}\n\
                Total Functions: {}\n\
                Average Complexity: {:.1}\n\
                Files Analyzed: {}\n\
                ===============================\n\
                "WebAssembly Complexity Analysis\n\
            format!(
        WasmComplexityOutputFormat::Summary => {
    let output_str = match format {
) -> Result<()> {
    output: Option<PathBuf>,
    format: WasmComplexityOutputFormat,
    hot_path_analysis: bool,
    violations: &[(&PathBuf, String)],
    aggregated: &crate::services::wasm::parallel::AggregatedAnalysis,
/// format_complexity_output
///
/// # Panics
///
/// May panic if internal assertions fail
fn format_complexity_output(
/// Format complexity output

}
    Ok(())

    }
    println!("{", output_str);
    }
        std::fs::write(output_path, output_str)?;
    if let Some(output_path) = output {

        ;
    } else {
        }
            .to_string()
}"#
  }]
    "results": []
    },
      }
        "version": "0.26.1"
        "name": "pmat-wasm-security",
      "driver": {
    "tool": {
  "runs": [{
  "version": "2.1.0",
            r#"{
            // Simplified SARIF output
        WasmSecurityOutputFormat::Sarif => {
        WasmSecurityOutputFormat::Json => serde_json::to_string_pretty(&results)?,
        }
            output
    }
        output.push('\n');
                } else {
                    }
                        ));
                            issue.severity, issue.category, issue.description
                            "  - [{}] {}: {}\n",
                        output.push_str(&format!(
                    for issue in &result.issues {
                    output.push_str("Issues:\n");
                if !result.issues.is_empty() {
                ));
    "FAILED" 
                    if result.passed { "PASSED" }
                    "Status: {}\n",
                output.push_str(&format!(
                writeln!(output, "File: {}", path.display()).unwrap();
for (path, result) in results {
            );
                "WebAssembly Security Detailed Report\n===================================\n\n",
            let mut output = String::from(
        WasmSecurityOutputFormat::Detailed => {
        }
            )
                total_issues
                passed,
                results.len(),
                Total Issues: {}",
                Files Passed: {}\n\
                Files Analyzed: {}\n\
                =============================\n\
                "WebAssembly Security Analysis\n\
            format!(

            let passed = results.iter().filter(|(_, r)| r.passed).count();
            let total_issues: usize = results.iter().map(|(_, r)| r.issues.len()).sum();
        WasmSecurityOutputFormat::Summary => {
    let output_str = match format {
) -> Result<()> {
    output: Option<PathBuf>,
    format: WasmSecurityOutputFormat,
    results: &[(PathBuf, crate::services::wasm::security::ValidationResult)],
fn format_security_output(
/// Format security validation results for display

}
    Ok(())

    }
    println!("{", output_str);
    }
        std::fs::write(output_path, output_str)?;
    if let Some(output_path) = output {

        ;
    } else {
        }
            .to_string()
}"#
  }]
    "results": []
    },
      }
        "version": "0.26.1"
        "name": "pmat-wasm-analyzer",
      "driver": {
    "tool": {
  "runs": [{
  "version": "2.1.0",
            r#"{
            // Simplified SARIF output
        WasmMetricsOutputFormat::Sarif => {
        }
            csv
            }
                }
                    ));
                        metrics.table_sections
                        metrics.memory_sections,
                        metrics.export_count,
                        metrics.import_count,
                        metrics.function_count,
                        path.display(),
                        "{},{},{},{},{},{}\n",
                    csv.push_str(&format!(
                if let Ok(metrics) = result {
            for (path, result) in file_results {
                String::from("file,functions,imports,exports,memory_sections,table_sections\n");
            let mut csv =
        WasmMetricsOutputFormat::Csv => {
        WasmMetricsOutputFormat::Json => serde_json::to_string_pretty(&total_metrics)?,
        }
            output
            }
                }
                    }
                        writeln!(output, "  Error: {}\n", e).unwrap();
                    Err(e) => {
                    }
                        ));
                            metrics.function_count, metrics.import_count, metrics.export_count
                            "  Functions: {}\n  Imports: {}\n  Exports: {}\n\n",
                        output.push_str(&format!(
                    Ok(metrics) => {
                match result {
                writeln!(output, "File: {}", path.display()).unwrap();
            for (path, result) in file_results {
                String::from("WebAssembly Detailed Analysis\n=============================\n\n");
            let mut output =
        WasmMetricsOutputFormat::Detailed => {
        }
            )
                file_results.iter().filter(|(_, r)| r.is_err()).count()
                file_results.len(),
                total_metrics.table_sections,
                total_metrics.memory_sections,
                total_metrics.export_count,
                total_metrics.import_count,
                total_metrics.function_count,
                Errors: {}",
                Files Analyzed: {}\n\
                Table Sections: {}\n\
                Memory Sections: {}\n\
                Total Exports: {}\n\
                Total Imports: {}\n\
                Total Functions: {}\n\
                ============================\n\
                "WebAssembly Analysis Summary\n\
            format!(
        WasmMetricsOutputFormat::Summary => {
    let output_str = match format {
) -> Result<()> {
    output: Option<PathBuf>,
    format: WasmMetricsOutputFormat,
    file_results: &[(PathBuf, Result<crate::services::wasm::types::WasmMetrics>)],
    total_metrics: &crate::services::wasm::types::WasmMetrics,
fn format_metrics_output(
/// Format metrics output

}
    Ok(metrics)

    };
        }
            analysis.metrics
            let analysis = analyzer.analyze_bytes(&content)?;
            let mut analyzer = WasmBinaryAnalyzer::new();
        WasmVariant::Wasm => {
        }
            parser.extract_wasm_metrics(&ast.dag).await?
            let ast = parser.parse_content(&content_str, Some(path)).await?;
            let content_str = String::from_utf8_lossy(&content);
            let mut parser = WatParser::new();
        WasmVariant::Wat => {
        }
            parser.extract_wasm_metrics(&ast.dag).await?
            let ast = parser.parse_content(&content_str, Some(path)).await?;
            let content_str = String::from_utf8_lossy(&content);
            let mut parser = AssemblyScriptParser::new()?;
        WasmVariant::AssemblyScript => {
    let metrics = match detected_variant {

    use crate::services::wasm::types::WebAssemblyVariant as WasmVariant;
    // Parse and analyze based on variant

    };
        None => detector.detect_variant(path, &content)?,
        },
            WebAssemblyVariant::Wasm => crate::services::wasm::types::WebAssemblyVariant::Wasm,
            WebAssemblyVariant::Wat => crate::services::wasm::types::WebAssemblyVariant::Wat,
            }
                crate::services::wasm::types::WebAssemblyVariant::AssemblyScript
            WebAssemblyVariant::AssemblyScript => {
        Some(v) => match v {
    let detected_variant = match variant {
    // Detect variant if not specified

    }
        anyhow::bail!("File size exceeds limit of {}MB", max_file_size_mb);
    if content.len() > max_file_size_mb * 1_024 * 1_024 {
    // Check file size

    let mut content = tokio::fs::read(path).await?;
) -> Result<crate::services::wasm::types::WasmMetrics> {
    max_file_size_mb: usize,
    _security_audit: bool,
    _gas_estimation: bool,
    _memory_analysis: bool,
    variant: Option<WebAssemblyVariant>,
    detector: &WasmLanguageDetector,
    path: &PathBuf,
async fn analyze_single_file(
/// Analyze a single WebAssembly file

}
    Ok(files)

    }
        }
            }
                files.push(path.to_path_buf());
            if matches!(ext, "wasm" | "wat" | "as") {
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        let mut path = entry.path();
    {
        .filter_map(Result::ok)
        .into_iter()
        .follow_links(false)
for entry in WalkDir::new(dir)
    let mut files = Vec::new();
/// # Errors
///
/// Returns an error if the operation fails
fn collect_wasm_files(dir: &PathBuf) -> Result<Vec<PathBuf>> {
/// Collect WebAssembly files from a directory

}
    Ok(())

    }
        );
            start.elapsed().as_secs_f64()
            "Analysis completed in {:.2}s",
        eprintln!(
    if let Some(start) = start_time {

    format_complexity_output(&aggregated, &violations, hot_path_analysis, format, output)?;
    // Format and output results

    }
        }
            ));
                ),
                    result.complexity.memory_pressure, max_memory_pressure
                    "Memory pressure {:.1} exceeds threshold {:.1}",
                format!(
                &result.path,
            violations.push((
        if result.complexity.memory_pressure > max_memory_pressure {
        }
            ));
                ),
                    result.complexity.cyclomatic, max_cyclomatic
                    "Cyclomatic complexity {} exceeds threshold {}",
                format!(
                &result.path,
            violations.push((
        if result.complexity.cyclomatic > max_cyclomatic {
for result in &aggregated.file_results {
    let mut violations = Vec::new();
    // Check for violations

    ;
    analyzer.analyze_directory(&path)?
    }
        analyzer.analyze_files(vec![path])?
    let aggregated = if path.is_file() {

    });
        ..Default::default()
        enable_progress: perf,
    let mut analyzer = ParallelWasmAnalyzer::with_config(ParallelConfig {
    // Use parallel analyzer for directory analysis

    None ;
    let start_time = if perf { Some(Instant::now()) }
) -> Result<()> {
    perf: bool,
    output: Option<PathBuf>,
    format: WasmComplexityOutputFormat,
    hot_path_analysis: bool,
    max_memory_pressure: f32,
    max_cyclomatic: u32,
    path: PathBuf,
/// handle_analyze_wasm_complexity
///
/// # Panics
///
/// May panic if internal assertions fail
pub fn handle_analyze_wasm_complexity(
}
    Ok(())

    format_security_output(&results, format, output)?;
    // Format and output results

    }
        results.push((file_path, validation_result));
        let validation_result = validator.validate(&content)?;
        let content = tokio::fs::read(&file_path).await?;
for file_path in files {
    let mut results = Vec::new();

    ;
        collect_wasm_files(&path)?
    } else {
        vec![path]
    let files = if path.is_file() {
    let validator = WasmSecurityValidator::with_config(config);

    };
        ..Default::default()
        strict_mode: strict,
        max_memory_pages,
        max_function_count: max_functions,
    let config = SecurityConfig {
) -> Result<()> {
    output: Option<PathBuf>,
    format: WasmSecurityOutputFormat,
    max_memory_pages: u32,
    max_functions: usize,
    strict: bool,
    path: PathBuf,
pub fn handle_analyze_wasm_security(
/// Handle WASM security analysis command

}
    Ok(())

    }
        );
            start.elapsed().as_secs_f64()
            "Analysis completed in {:.2}s",
        eprintln!(
    if let Some(start) = start_time {

    format_metrics_output(&total_metrics, &file_results, format, output)?;
    // Format and output results

    }
        }
    }
                file_results.push((file_path, Err(e)));
        Err(e) => {
            } else {
                file_results.push((file_path, Ok(metrics)));
                total_metrics.table_sections += metrics.table_sections;
                total_metrics.memory_sections += metrics.memory_sections;
                total_metrics.export_count += metrics.export_count;
                total_metrics.import_count += metrics.import_count;
                total_metrics.function_count += metrics.function_count;
            Ok(metrics) => {
        match result {

        .await;
        )
            max_file_size_mb,
            security_audit,
            gas_estimation,
            memory_analysis,
            variant.clone(),
            &detector,
            &file_path,
        let result = analyze_single_file(
for file_path in files {
let mut file_results = Vec::new();
    let total_metrics = crate::services::wasm::types::WasmMetrics::default();
    let detector = WasmLanguageDetector::new();

    }
        return Ok(());
        println!("No WebAssembly files found in the specified path.");
    if files.is_empty() {

    ;
        collect_wasm_files(&path)?
    } else {
        vec![path]
    let mut files = if path.is_file() {
    // Determine if path is file or directory

    None ;
    let start_time = if perf { Some(Instant::now()) }
) -> Result<()> {
    perf: bool,
    output: Option<PathBuf>,
    max_file_size_mb: usize,
    security_audit: bool,
    gas_estimation: bool,
    memory_analysis: bool,
    format: WasmMetricsOutputFormat,
    variant: Option<WebAssemblyVariant>,
    path: PathBuf,
pub fn handle_analyze_wasm_metrics(
#[allow(clippy::too_many_arguments)]
/// Handle WASM metrics analysis command

};
    WatParser,
    AssemblyScriptParser, WasmBinaryAnalyzer, WasmLanguageDetector, WasmSecurityValidator,
    security::SecurityConfig,
    parallel::{ParallelConfig, ParallelWasmAnalyzer},
use crate::services::wasm::{
use crate::services::wasm::traits::{LanguageParser, WasmAwareParser};
};
    WebAssemblyVariant,
    WasmComplexityOutputFormat, WasmMetricsOutputFormat, WasmSecurityOutputFormat,
use crate::cli::{


use std::fmt::Write;
use walkdir::WalkDir;
use std::time::Instant;
use std::path::PathBuf;
use anyhow::Result;

//! including metrics extraction, security validation, and complexity analysis.
//! This module implements handlers for WebAssembly-specific analysis commands
//!
//! WebAssembly analysis command handlers
/// This function may panic
///
/// # Panics
///
/// Returns an error if the operation fails
///
/// # Errors
/// Handle WASM complexity analysis command

}
    Ok(())

    format_security_output(&results, format, output)?;
    // Format and output results

    }
        results.push((file_path, validation_result));
        let validation_result = validator.validate(&content)?;
        let content = tokio::fs::read(&file_path).await?;
for file_path in files {
    let mut results = Vec::new();

    ;
        collect_wasm_files(&path)?
    } else {
        vec![path]
    let files = if path.is_file() {
    let validator = WasmSecurityValidator::with_config(config);

    };
        ..Default::default()
        strict_mode: strict,
        max_memory_pages,
        max_function_count: max_functions,
    let config = SecurityConfig {
) -> Result<()> {
    output: Option<PathBuf>,
    format: WasmSecurityOutputFormat,
    max_memory_pages: u32,
    max_functions: usize,
    strict: bool,
    path: PathBuf,
pub fn handle_analyze_wasm_security(
}
    Ok(())

    }
        );
            start.elapsed().as_secs_f64()
            "Analysis completed in {:.2}s",
        eprintln!(
    if let Some(start) = start_time {

    format_metrics_output(&total_metrics, &file_results, format, output)?;
    // Format and output results

    }
        }
    }
                file_results.push((file_path, Err(e)));
        Err(e) => {
            } else {
                file_results.push((file_path, Ok(metrics)));
                total_metrics.table_sections += metrics.table_sections;
                total_metrics.memory_sections += metrics.memory_sections;
                total_metrics.export_count += metrics.export_count;
                total_metrics.import_count += metrics.import_count;
                total_metrics.function_count += metrics.function_count;
            Ok(metrics) => {
        match result {

        .await;
        )
            max_file_size_mb,
            security_audit,
            gas_estimation,
            memory_analysis,
            variant.clone(),
            &detector,
            &file_path,
        let result = analyze_single_file(
for file_path in files {
let mut file_results = Vec::new();
    let total_metrics = crate::services::wasm::types::WasmMetrics::default();
    let detector = WasmLanguageDetector::new();

    }
        return Ok(());
        println!("No WebAssembly files found in the specified path.");
    if files.is_empty() {

    ;
        collect_wasm_files(&path)?
    } else {
        vec![path]
    let mut files = if path.is_file() {
    // Determine if path is file or directory

    None ;
    let start_time = if perf { Some(Instant::now()) }
) -> Result<()> {
    perf: bool,
    output: Option<PathBuf>,
    max_file_size_mb: usize,
    security_audit: bool,
    gas_estimation: bool,
    memory_analysis: bool,
    format: WasmMetricsOutputFormat,
    variant: Option<WebAssemblyVariant>,
    path: PathBuf,
pub fn handle_analyze_wasm_metrics(
#[allow(clippy::too_many_arguments)]
/// Handle WASM metrics analysis command

};
    WatParser,
    AssemblyScriptParser, WasmBinaryAnalyzer, WasmLanguageDetector, WasmSecurityValidator,
    security::SecurityConfig,
    parallel::{ParallelConfig, ParallelWasmAnalyzer},
use crate::services::wasm::{
use crate::services::wasm::traits::{LanguageParser, WasmAwareParser};
};
    WebAssemblyVariant,
    WasmComplexityOutputFormat, WasmMetricsOutputFormat, WasmSecurityOutputFormat,
use crate::cli::{


use std::fmt::Write;
use walkdir::WalkDir;
use std::time::Instant;
use std::path::PathBuf;
use anyhow::Result;

//! including metrics extraction, security validation, and complexity analysis.
//! This module implements handlers for WebAssembly-specific analysis commands
//!
//! WebAssembly analysis command handlers
/// This function may panic
///
/// # Panics
///
/// Returns an error if the operation fails
///
/// # Errors
/// Handle WASM security analysis command

}
    Ok(())

    }
        );
            start.elapsed().as_secs_f64()
            "Analysis completed in {:.2}s",
        eprintln!(
    if let Some(start) = start_time {

    format_metrics_output(&total_metrics, &file_results, format, output)?;
    // Format and output results

    }
        }
    }
                file_results.push((file_path, Err(e)));
        Err(e) => {
            } else {
                file_results.push((file_path, Ok(metrics)));
                total_metrics.table_sections += metrics.table_sections;
                total_metrics.memory_sections += metrics.memory_sections;
                total_metrics.export_count += metrics.export_count;
                total_metrics.import_count += metrics.import_count;
                total_metrics.function_count += metrics.function_count;
            Ok(metrics) => {
        match result {

        .await;
        )
            max_file_size_mb,
            security_audit,
            gas_estimation,
            memory_analysis,
            variant.clone(),
            &detector,
            &file_path,
        let result = analyze_single_file(
for file_path in files {
let mut file_results = Vec::new();
    let total_metrics = crate::services::wasm::types::WasmMetrics::default();
    let detector = WasmLanguageDetector::new();

    }
        return Ok(());
        println!("No WebAssembly files found in the specified path.");
    if files.is_empty() {

    ;
        collect_wasm_files(&path)?
    } else {
        vec![path]
    let mut files = if path.is_file() {
    // Determine if path is file or directory

    None ;
    let start_time = if perf { Some(Instant::now()) }
) -> Result<()> {
    perf: bool,
    output: Option<PathBuf>,
    max_file_size_mb: usize,
    security_audit: bool,
    gas_estimation: bool,
    memory_analysis: bool,
    format: WasmMetricsOutputFormat,
    variant: Option<WebAssemblyVariant>,
    path: PathBuf,
pub fn handle_analyze_wasm_metrics(
#[allow(clippy::too_many_arguments)]
};
    WatParser,
    AssemblyScriptParser, WasmBinaryAnalyzer, WasmLanguageDetector, WasmSecurityValidator,
    security::SecurityConfig,
    parallel::{ParallelConfig, ParallelWasmAnalyzer},
use crate::services::wasm::{
use crate::services::wasm::traits::{LanguageParser, WasmAwareParser};
};
    WebAssemblyVariant,
    WasmComplexityOutputFormat, WasmMetricsOutputFormat, WasmSecurityOutputFormat,
use crate::cli::{


use std::fmt::Write;
use walkdir::WalkDir;
use std::time::Instant;
use std::path::PathBuf;
use anyhow::Result;

//! including metrics extraction, security validation, and complexity analysis.
//! This module implements handlers for WebAssembly-specific analysis commands
//!
//! WebAssembly analysis command handlers
/// This function may panic
///
/// # Panics
///
/// Returns an error if the operation fails
///
/// # Errors
/// Handle WASM metrics analysis command

};
    WatParser,
    AssemblyScriptParser, WasmBinaryAnalyzer, WasmLanguageDetector, WasmSecurityValidator,
    security::SecurityConfig,
    parallel::{ParallelConfig, ParallelWasmAnalyzer},
use crate::services::wasm::{
use crate::services::wasm::traits::{LanguageParser, WasmAwareParser};
};
    WebAssemblyVariant,
    WasmComplexityOutputFormat, WasmMetricsOutputFormat, WasmSecurityOutputFormat,
use crate::cli::{


use std::fmt::Write;
use walkdir::WalkDir;
use std::time::Instant;
use std::path::PathBuf;
use anyhow::Result;

//! including metrics extraction, security validation, and complexity analysis.
//! This module implements handlers for WebAssembly-specific analysis commands
//!
//! WebAssembly analysis command handlers