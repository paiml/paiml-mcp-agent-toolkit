// JSON output structs for --format json (#216)
#[derive(serde::Serialize)]
struct ContextJsonOutput {
    version: &'static str,
    project: ContextJsonProject,
    files: Vec<ContextJsonFile>,
}

#[derive(serde::Serialize)]
struct ContextJsonProject {
    language: String,
    path: String,
    total_files: usize,
    total_functions: usize,
    overall_health: f64,
    maintainability_index: f64,
}

#[derive(serde::Serialize)]
struct ContextJsonFile {
    path: String,
    items: Vec<ContextJsonItem>,
}

#[derive(serde::Serialize)]
struct ContextJsonItem {
    name: String,
    #[serde(rename = "type")]
    item_type: String,
    line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    complexity: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cognitive_complexity: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fields_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    variants_count: Option<usize>,
}

/// Generate enhanced AST context with rich annotations
async fn generate_enhanced_ast_context(
    toolchain: &str,
    project_path: &Path,
    context: &crate::services::deep_context::DeepContext,
    format: ContextFormat,
    include_large_files: bool,
) -> Result<String> {
    // GH-46: --include-large-files not yet implemented
    if include_large_files {
        eprintln!("Warning: --include-large-files is not yet implemented. Flag ignored.");
    }
    match format {
        ContextFormat::Json => generate_json_context(toolchain, project_path, context),
        ContextFormat::Sarif => generate_sarif_stub(toolchain, project_path),
        ContextFormat::Markdown | ContextFormat::LlmOptimized => {
            generate_markdown_context(toolchain, project_path, context)
        }
    }
}

/// Find file-level complexity metrics for a given FileContext (shared by markdown + json)
fn find_file_metrics<'a>(
    file: &crate::services::context::FileContext,
    analyses: &'a crate::services::deep_context::AnalysisResults,
) -> Option<&'a crate::services::complexity::FileComplexityMetrics> {
    analyses.complexity_report.as_ref().and_then(|report| {
        report.files.iter().find(|f| {
            use std::path::Path;
            let file_path = Path::new(&file.path);
            let metric_path = Path::new(&f.path);
            if file.path == f.path {
                return true;
            }
            if file.path.ends_with(&f.path) || f.path.ends_with(&file.path) {
                return true;
            }
            if let (Some(a), Some(b)) = (file_path.file_name(), metric_path.file_name()) {
                if a == b {
                    return true;
                }
            }
            false
        })
    })
}

/// Generate minimal SARIF v2.1 stub
fn generate_sarif_stub(toolchain: &str, project_path: &Path) -> Result<String> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let sarif = serde_json::json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "pmat",
                    "informationUri": "https://github.com/paiml/pmat",
                    "semanticVersion": env!("CARGO_PKG_VERSION"),
                }
            },
            "invocations": [{
                "executionSuccessful": true,
                "toolConfigurationNotifications": [],
            }],
            "properties": {
                "language": toolchain,
                "projectPath": project_path.display().to_string(),
            },
            "results": []
        }]
    });
    serde_json::to_string_pretty(&sarif).map_err(|e| anyhow::anyhow!("SARIF serialize: {e}"))
}

/// Detect toolchain or use provided one
fn detect_or_use_toolchain(toolchain: Option<String>, project_path: &Path) -> Result<String> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    use std::io::{self, Write};

    if let Some(t) = toolchain {
        Ok(t)
    } else {
        // Print without newline for in-place update
        eprint!("🔍 Auto-detecting project language...");
        io::stderr().flush().ok();

        // First try with confidence
        if let Some((lang, confidence)) =
            super::super::detect_primary_language_with_confidence(project_path)
        {
            // Clear line and print result (\r = carriage return, \x1b[K = clear to end of line)
            eprintln!("\r\x1b[K✅ Detected: {lang} (confidence: {confidence:.1}%)");
            return Ok(lang);
        }

        // Fall back to simple detection
        if let Some(lang) = super::super::detect_primary_language(project_path) {
            // Clear line and print result
            eprintln!("\r\x1b[K✅ Detected: {lang}");
            return Ok(lang);
        }

        // Default to rust if no language detected
        // Clear line and print warning
        eprintln!("\r\x1b[K⚠️  Could not detect language, defaulting to Rust");
        Ok("rust".to_string())
    }
}

// Include split sub-files (share parent module scope)
include!("context_generation_json.rs");
include!("context_generation_markdown.rs");
include!("context_generation_annotations.rs");
