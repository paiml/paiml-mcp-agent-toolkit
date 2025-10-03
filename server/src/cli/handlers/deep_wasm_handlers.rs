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

/// Handles the deep-wasm analysis command
#[cfg(feature = "deep-wasm")]
pub async fn handle_deep_wasm(
    source_path: PathBuf,
    wasm_file: Option<PathBuf>,
    dwarf_file: Option<PathBuf>,
    source_map: Option<PathBuf>,
    language: Option<DeepWasmLanguage>,
    focus: DeepWasmFocus,
    format: DeepWasmOutputFormat,
    output: Option<PathBuf>,
    strict: bool,
    _include_mir: bool,
    _include_llvm_ir: bool,
    _track_memory: bool,
    _detect_deadlocks: bool,
) -> Result<()> {
    // Convert CLI enums to service types
    let source_language = match language {
        Some(DeepWasmLanguage::Rust) => SourceLanguage::Rust,
        Some(DeepWasmLanguage::Ruchy) => SourceLanguage::Ruchy,
        None => {
            // Auto-detect based on file extension
            if let Some(ext) = source_path.extension() {
                match ext.to_str() {
                    Some("rs") => SourceLanguage::Rust,
                    Some("rch") | Some("ruchy") => SourceLanguage::Ruchy,
                    _ => SourceLanguage::Rust, // Default
                }
            } else {
                SourceLanguage::Rust
            }
        }
    };

    let analysis_focus = match focus {
        DeepWasmFocus::Full => AnalysisFocus::Full,
        DeepWasmFocus::Source => AnalysisFocus::Source,
        DeepWasmFocus::Compilation => AnalysisFocus::Compilation,
        DeepWasmFocus::Runtime => AnalysisFocus::Runtime,
        DeepWasmFocus::Interop => AnalysisFocus::Interop,
    };

    // Create request
    let request = DeepWasmAnalysisRequest {
        source_path,
        wasm_path: wasm_file,
        dwarf_path: dwarf_file,
        source_map_path: source_map,
        language: source_language,
        analysis_focus,
    };

    // Create service
    let mut service = DeepWasmService::new();

    // Apply strict mode if requested
    if strict {
        use crate::services::deep_wasm::WasmQualityGates;
        let gates = WasmQualityGates {
            max_module_size: 5_242_880, // Stricter 5MB limit
            max_wasm_complexity: 15, // Stricter complexity limit
            min_source_map_coverage: 0.99, // Stricter coverage
            ..Default::default()
        };
        service = service.with_quality_gates(gates);
    }

    // Run analysis
    let report = service.analyze(request).await?;

    // Generate output
    let output_content = match format {
        DeepWasmOutputFormat::Markdown => {
            use crate::services::deep_wasm::ReportGenerator;
            let generator = ReportGenerator::new();
            generator.generate_markdown(&report)?
        }
        DeepWasmOutputFormat::Json => {
            serde_json::to_string_pretty(&report)?
        }
        DeepWasmOutputFormat::Html => {
            // TODO: Implement HTML generator
            return Err(anyhow::anyhow!("HTML output not yet implemented"));
        }
    };

    // Write output
    if let Some(output_path) = output {
        std::fs::write(output_path, output_content)?;
    } else {
        println!("{}", output_content);
    }

    // Check quality gates
    if !report.quality_gate_results.passed {
        eprintln!("\n❌ Quality gate violations detected:");
        for violation in &report.quality_gate_results.violations {
            eprintln!("  - {}: {}", violation.rule, violation.message);
        }
        std::process::exit(1);
    }

    Ok(())
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
