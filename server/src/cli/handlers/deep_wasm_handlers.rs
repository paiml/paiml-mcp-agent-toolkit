//! Deep WASM CLI handlers
//!
//! Handles the `pmat analyze deep-wasm` command for deep WASM pipeline inspection.

#[cfg(feature = "deep-wasm")]
use crate::cli::enums::{DeepWasmFocus, DeepWasmLanguage, DeepWasmOutputFormat};
#[cfg(feature = "deep-wasm")]
use crate::services::deep_wasm::{
    AnalysisFocus, DeepWasmAnalysisRequest, DeepWasmService, SourceLanguage,
};
use anyhow::Result;
use std::path::PathBuf;

/// Options for deep WASM analysis
#[cfg(feature = "deep-wasm")]
pub struct DeepWasmOptions {
    pub source_path: PathBuf,
    pub wasm_file: Option<PathBuf>,
    pub dwarf_file: Option<PathBuf>,
    pub source_map: Option<PathBuf>,
    pub language: Option<DeepWasmLanguage>,
    pub focus: DeepWasmFocus,
    pub format: DeepWasmOutputFormat,
    pub output: Option<PathBuf>,
    pub strict: bool,
    pub _include_mir: bool,
    pub _include_llvm_ir: bool,
    pub _track_memory: bool,
    pub _detect_deadlocks: bool,
}

/// Handles the deep-wasm analysis command
#[cfg(feature = "deep-wasm")]
pub async fn handle_deep_wasm(options: DeepWasmOptions) -> Result<()> {
    let DeepWasmOptions {
        source_path,
        wasm_file,
        dwarf_file,
        source_map,
        language,
        focus,
        format,
        output,
        strict,
        _include_mir,
        _include_llvm_ir,
        _track_memory,
        _detect_deadlocks,
    } = options;

    // Convert CLI options to service request
    let request = create_analysis_request(
        source_path,
        wasm_file,
        dwarf_file,
        source_map,
        language,
        focus,
    );

    // Create and configure service
    let service = create_configured_service(strict);

    // Run analysis
    let report = service.analyze(request).await?;

    // Generate and write output
    write_analysis_output(&report, format, output)?;

    // Validate quality gates
    validate_quality_gates(&report, strict)?;

    Ok(())
}

/// Creates the analysis request from CLI parameters
#[cfg(feature = "deep-wasm")]
fn create_analysis_request(
    source_path: PathBuf,
    wasm_file: Option<PathBuf>,
    dwarf_file: Option<PathBuf>,
    source_map: Option<PathBuf>,
    language: Option<DeepWasmLanguage>,
    focus: DeepWasmFocus,
) -> DeepWasmAnalysisRequest {
    let source_language = detect_source_language(&source_path, language);
    let analysis_focus = convert_analysis_focus(focus);

    DeepWasmAnalysisRequest {
        source_path,
        wasm_path: wasm_file,
        dwarf_path: dwarf_file,
        source_map_path: source_map,
        language: source_language,
        analysis_focus,
    }
}

/// Detects or converts source language
#[cfg(feature = "deep-wasm")]
fn detect_source_language(
    source_path: &PathBuf,
    language: Option<DeepWasmLanguage>,
) -> SourceLanguage {
    match language {
        Some(DeepWasmLanguage::Rust) => SourceLanguage::Rust,
        Some(DeepWasmLanguage::Ruchy) => SourceLanguage::Ruchy,
        None => auto_detect_language(source_path),
    }
}

/// Auto-detects language from file extension
#[cfg(feature = "deep-wasm")]
fn auto_detect_language(source_path: &PathBuf) -> SourceLanguage {
    source_path
        .extension()
        .and_then(|ext| ext.to_str())
        .and_then(|ext_str| match ext_str {
            "rs" => Some(SourceLanguage::Rust),
            "rch" | "ruchy" => Some(SourceLanguage::Ruchy),
            _ => None,
        })
        .unwrap_or(SourceLanguage::Rust)
}

/// Converts CLI focus enum to service focus enum
#[cfg(feature = "deep-wasm")]
fn convert_analysis_focus(focus: DeepWasmFocus) -> AnalysisFocus {
    match focus {
        DeepWasmFocus::Full => AnalysisFocus::Full,
        DeepWasmFocus::Source => AnalysisFocus::Source,
        DeepWasmFocus::Compilation => AnalysisFocus::Compilation,
        DeepWasmFocus::Runtime => AnalysisFocus::Runtime,
        DeepWasmFocus::Interop => AnalysisFocus::Interop,
    }
}

/// Creates service with quality gates based on strict mode
#[cfg(feature = "deep-wasm")]
fn create_configured_service(strict: bool) -> DeepWasmService {
    use crate::services::deep_wasm::WasmQualityGates;

    let gates = if strict {
        create_strict_quality_gates()
    } else {
        create_relaxed_quality_gates()
    };

    DeepWasmService::new().with_quality_gates(gates)
}

/// Creates strict quality gates
#[cfg(feature = "deep-wasm")]
fn create_strict_quality_gates() -> crate::services::deep_wasm::WasmQualityGates {
    use crate::services::deep_wasm::WasmQualityGates;
    WasmQualityGates {
        max_module_size: 5_242_880,      // Stricter 5MB limit
        max_wasm_complexity: 15,         // Stricter complexity limit
        min_source_map_coverage: 0.99,   // Stricter coverage
        ..Default::default()
    }
}

/// Creates relaxed quality gates
#[cfg(feature = "deep-wasm")]
fn create_relaxed_quality_gates() -> crate::services::deep_wasm::WasmQualityGates {
    use crate::services::deep_wasm::WasmQualityGates;
    WasmQualityGates {
        max_module_size: 20_971_520,     // Relaxed 20MB limit
        max_wasm_complexity: 30,         // Relaxed complexity limit
        min_source_map_coverage: 0.0,    // Don't require source maps
        ..Default::default()
    }
}

/// Writes analysis output in the specified format
#[cfg(feature = "deep-wasm")]
fn write_analysis_output(
    report: &crate::services::deep_wasm::DeepWasmReport,
    format: DeepWasmOutputFormat,
    output: Option<PathBuf>,
) -> Result<()> {
    let output_content = generate_output_content(report, format)?;

    if let Some(output_path) = output {
        std::fs::write(output_path, output_content)?;
    } else {
        println!("{}", output_content);
    }

    Ok(())
}

/// Generates output content in the specified format
#[cfg(feature = "deep-wasm")]
fn generate_output_content(
    report: &crate::services::deep_wasm::DeepWasmReport,
    format: DeepWasmOutputFormat,
) -> Result<String> {
    match format {
        DeepWasmOutputFormat::Markdown => {
            use crate::services::deep_wasm::ReportGenerator;
            let generator = ReportGenerator::new();
            Ok(generator.generate_markdown(report)?)
        }
        DeepWasmOutputFormat::Json => {
            Ok(serde_json::to_string_pretty(report)?)
        }
        DeepWasmOutputFormat::Html => {
            Err(anyhow::anyhow!("HTML output not yet implemented"))
        }
    }
}

/// Validates quality gates and fails in strict mode if violations found
#[cfg(feature = "deep-wasm")]
fn validate_quality_gates(
    report: &crate::services::deep_wasm::DeepWasmReport,
    strict: bool,
) -> Result<()> {
    if !report.quality_gate_results.passed {
        print_quality_violations(&report.quality_gate_results.violations);

        if strict {
            return Err(anyhow::anyhow!(
                "Quality gate violations detected in strict mode. {} violation(s) found.",
                report.quality_gate_results.violations.len()
            ));
        }
    }

    Ok(())
}

/// Prints quality gate violations to stderr
#[cfg(feature = "deep-wasm")]
fn print_quality_violations(violations: &[crate::services::deep_wasm::QualityViolation]) {
    eprintln!("\n❌ Quality gate violations detected:");
    for violation in violations {
        eprintln!("  - {}: {}", violation.rule, violation.message);
    }
}

/// Stub handler when feature is disabled
#[cfg(not(feature = "deep-wasm"))]
pub async fn handle_deep_wasm(
    _source_path: PathBuf,
    _wasm_file: Option<PathBuf>,
    _dwarf_file: Option<PathBuf>,
    _source_map: Option<PathBuf>,
    _language: Option<()>,
    _focus: (),
    _format: (),
    _output: Option<PathBuf>,
    _strict: bool,
    _include_mir: bool,
    _include_llvm_ir: bool,
    _track_memory: bool,
    _detect_deadlocks: bool,
) -> Result<()> {
    Err(anyhow::anyhow!(
        "Deep WASM feature not enabled. Recompile with --features deep-wasm"
    ))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_handler_compiles() {
        // Basic compilation test
        assert!(true);
    }
}
