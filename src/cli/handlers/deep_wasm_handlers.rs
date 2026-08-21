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
use std::path::{Path, PathBuf};

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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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

    // Say what input was needed, before the service says "os error 21".
    guard_source_file(&source_path, language.clone())?;

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

/// `--source-path` names one source FILE, and this says so when it does not.
///
/// A directory reached the service and surfaced as
/// `Error: IO error: Is a directory (os error 21)` — an errno where a sentence
/// belonged, naming neither the flag nor what it wanted.
///
/// The unknown-extension arm is the same defect one level down:
/// `auto_detect_language` falls back to `SourceLanguage::Rust` for ANY
/// unrecognised extension, so `--source-path app.py` was analysed as Rust and
/// reported as a finding about Rust. A guess is only honest when it is
/// declared, so an extension pmat does not recognise is refused unless
/// `--language` says what to treat it as.
#[cfg(feature = "deep-wasm")]
fn guard_source_file(path: &Path, language: Option<DeepWasmLanguage>) -> Result<()> {
    if !path.exists() {
        anyhow::bail!("--source-path {} does not exist", path.display());
    }
    if path.is_dir() {
        anyhow::bail!(
            "--source-path {} is a directory. deep-wasm inspects ONE source file through its \
             compiled WASM, so point it at the .rs or .ruchy file itself.",
            path.display()
        );
    }
    if language.is_some() {
        return Ok(());
    }
    match path.extension().and_then(|e| e.to_str()) {
        Some("rs" | "rch" | "ruchy") => Ok(()),
        Some(other) => anyhow::bail!(
            "--source-path {} has extension .{other}, which deep-wasm has no source reader for \
             (it reads .rs, .rch and .ruchy). Pass --language rust|ruchy to analyse it as one of \
             those anyway.",
            path.display()
        ),
        None => anyhow::bail!(
            "--source-path {} has no file extension, so deep-wasm cannot tell which language to \
             read it as. Pass --language rust|ruchy.",
            path.display()
        ),
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
        max_module_size: 5_242_880,    // Stricter 5MB limit
        max_wasm_complexity: 15,       // Stricter complexity limit
        min_source_map_coverage: 0.99, // Stricter coverage
        ..Default::default()
    }
}

/// Creates relaxed quality gates
#[cfg(feature = "deep-wasm")]
fn create_relaxed_quality_gates() -> crate::services::deep_wasm::WasmQualityGates {
    use crate::services::deep_wasm::WasmQualityGates;
    WasmQualityGates {
        max_module_size: 20_971_520,  // Relaxed 20MB limit
        max_wasm_complexity: 30,      // Relaxed complexity limit
        min_source_map_coverage: 0.0, // Don't require source maps
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
        DeepWasmOutputFormat::Json => Ok(serde_json::to_string_pretty(report)?),
        DeepWasmOutputFormat::Html => Err(anyhow::anyhow!("HTML output not yet implemented")),
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
#[cfg(feature = "deep-wasm")]
mod source_path_tests {
    use super::*;

    /// A directory must be named as such, not surfaced as an errno.
    ///
    /// `analyze deep-wasm --source-path <dir>` reached the service and failed
    /// with `Error: IO error: Is a directory (os error 21)` — which names
    /// neither the flag nor what it wanted.
    #[test]
    fn a_directory_is_refused_by_name() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let err =
            guard_source_file(dir.path(), None).expect_err("a directory is not a source file");
        let msg = err.to_string();
        assert!(msg.contains("--source-path"), "must name the flag: {msg}");
        assert!(msg.contains("directory"), "must say what was wrong: {msg}");
        assert!(
            !msg.contains("os error"),
            "an errno is not an explanation: {msg}"
        );
    }

    /// An extension pmat has no reader for must not be silently read as Rust.
    ///
    /// `auto_detect_language` falls back to `SourceLanguage::Rust` for any
    /// unrecognised extension, so `--source-path app.py` produced a report
    /// about Rust.
    #[test]
    fn an_unreadable_extension_is_not_silently_treated_as_rust() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let file = dir.path().join("app.py");
        std::fs::write(&file, "print('hi')\n").expect("write py");

        let err = guard_source_file(&file, None).expect_err("a .py is not a deep-wasm source");
        let msg = err.to_string();
        assert!(msg.contains(".py"), "must name the extension it got: {msg}");
        assert!(
            msg.contains("--language"),
            "must say how to override: {msg}"
        );
    }

    /// ...but an explicit --language is the user declaring the guess, so it is
    /// honoured. Without this, the guard would block a deliberate override.
    #[test]
    fn an_explicit_language_overrides_the_extension() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let file = dir.path().join("app.py");
        std::fs::write(&file, "print('hi')\n").expect("write py");

        guard_source_file(&file, Some(DeepWasmLanguage::Rust))
            .expect("an explicit --language must be honoured");
    }

    /// The counter-test: a real .rs file still passes.
    #[test]
    fn a_rust_source_file_still_passes() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let file = dir.path().join("lib.rs");
        std::fs::write(&file, "pub fn f() {}\n").expect("write rs");

        guard_source_file(&file, None).expect("a .rs file is exactly what this reads");
    }

    #[test]
    fn a_missing_path_says_it_is_missing() {
        let err = guard_source_file(Path::new("/definitely/not/here.rs"), None)
            .expect_err("a nonexistent path must be refused");
        assert!(err.to_string().contains("does not exist"), "{err}");
    }
}
