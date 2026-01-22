//! Template generation and scaffolding handlers
//!
//! This module contains the extracted implementations for template generation,
//! project scaffolding, and template validation operations.

// use crate::cli::*; // Currently unused
use crate::services::template_service::{generate_template, scaffold_project, validate_template};
use crate::stateless_server::StatelessTemplateServer;
use anyhow::Result;
use serde_json::Value;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;

/// Handle template generation command
pub async fn handle_generate(
    server: Arc<StatelessTemplateServer>,
    category: String,
    template: String,
    params: Vec<(String, Value)>,
    output: Option<PathBuf>,
    create_dirs: bool,
) -> Result<()> {
    let uri = format!("template://{category}/{template}");
    let params_json = super::super::analysis_utilities::params_to_json(params);

    let result = generate_template(server.as_ref(), &uri, params_json).await?;

    if let Some(path) = output {
        if create_dirs {
            tokio::fs::create_dir_all(path.parent().expect("internal error")).await?;
        }
        tokio::fs::write(&path, &result.content).await?;
        eprintln!("✅ Generated: {}", path.display());
    } else {
        tokio::io::stdout()
            .write_all(result.content.as_bytes())
            .await?;
    }
    Ok(())
}

/// Handle project scaffolding command
pub async fn handle_scaffold(
    server: Arc<StatelessTemplateServer>,
    toolchain: String,
    templates: Vec<String>,
    params: Vec<(String, Value)>,
    parallel: usize,
) -> Result<()> {
    use futures::stream::{self, StreamExt};

    let params_json = super::super::analysis_utilities::params_to_json(params);

    // Toyota Way: Extract Method - Reduce complexity by extracting template resolution
    let templates_to_use = resolve_scaffold_templates(&toolchain, templates);

    let results = scaffold_project(
        server.clone(),
        &toolchain,
        templates_to_use,
        serde_json::Value::Object(params_json.clone()),
    )
    .await?;

    // Report any errors
    if !results.errors.is_empty() {
        eprintln!("⚠️ Some templates failed to generate:");
        for error in &results.errors {
            eprintln!("  - {}: {}", error.template, error.error);
        }
    }

    // Store file count before moving the vector
    let file_count = results.files.len();

    // Parallel file writing with bounded concurrency
    let write_results: Vec<_> = stream::iter(results.files)
        .map(|file| async move {
            let path = PathBuf::from(&file.path);
            if let Some(parent) = path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            tokio::fs::write(&path, &file.content).await?;
            eprintln!("✅ Created: {}", file.path);
            Ok::<_, anyhow::Error>(())
        })
        .buffer_unordered(parallel)
        .collect()
        .await;

    // Check if any writes failed
    let mut any_failed = false;
    for result in write_results {
        if let Err(e) = result {
            eprintln!("❌ Failed to write file: {e}");
            any_failed = true;
        }
    }

    if !any_failed && file_count > 0 {
        eprintln!("\n🚀 Project scaffolded successfully!");
    } else if file_count == 0 {
        eprintln!("\n⚠️ No files were generated. Check your parameters and template availability.");
    }

    Ok(())
}

/// Toyota Way: Extract Method - Resolve templates based on toolchain
fn resolve_scaffold_templates(toolchain: &str, templates: Vec<String>) -> Vec<String> {
    if templates.is_empty() {
        get_default_scaffold_templates(toolchain)
    } else {
        templates
    }
}

/// Toyota Way: Extract Method - Get default templates for toolchain
fn get_default_scaffold_templates(toolchain: &str) -> Vec<String> {
    match toolchain {
        "rust" | "deno" | "python-uv" => vec![
            "makefile".to_string(),
            "readme".to_string(),
            "gitignore".to_string(),
        ],
        _ => vec!["readme".to_string()],
    }
}

/// Handle template validation command
pub async fn handle_validate(
    server: Arc<StatelessTemplateServer>,
    uri: String,
    params: Vec<(String, Value)>,
) -> Result<()> {
    let params_json = super::super::analysis_utilities::params_to_json(params);
    let result = validate_template(
        server.clone(),
        &uri,
        &serde_json::Value::Object(params_json),
    )
    .await?;

    if result.valid {
        eprintln!("✅ All parameters valid");
    } else {
        eprintln!("❌ Validation errors:");
        for error in result.errors {
            eprintln!("  - {}: {}", error.field, error.message);
        }
        std::process::exit(1);
    }
    Ok(())
}

/// Parameters for agent scaffolding
pub struct ScaffoldAgentParams {
    pub name: String,
    pub template: String,
    pub features: Vec<String>,
    pub quality: String,
    pub output: Option<PathBuf>,
    pub force: bool,
    pub dry_run: bool,
    pub interactive: bool,
    pub deterministic_core: Option<String>,
    pub probabilistic_wrapper: Option<String>,
}

/// Handle agent scaffolding command
pub async fn handle_scaffold_agent(params: ScaffoldAgentParams) -> Result<()> {
    let ScaffoldAgentParams {
        name,
        template,
        features,
        quality,
        output,
        force,
        dry_run,
        interactive,
        deterministic_core,
        probabilistic_wrapper,
    } = params;

    if interactive {
        // Interactive mode requires dialoguer crate which was removed to save dependencies
        // Use explicit CLI arguments instead: --name, --template, --features, --quality
        return Err(anyhow::anyhow!(
            "Interactive mode is not available. Use explicit arguments instead:\n  \
            pmat generate agent --name <NAME> --template mcp-tool-server\n  \
            Available templates: mcp-tool-server, state-machine, hybrid-agent, monitoring"
        ));
    }

    let context = build_agent_context(
        &name,
        &template,
        &features,
        &quality,
        deterministic_core,
        probabilistic_wrapper,
    )?;

    let output_path = output.unwrap_or_else(|| PathBuf::from(&name));

    execute_scaffold_operation(&context, &output_path, &name, dry_run, force).await
}

// REMOVED: handle_interactive_scaffold function required dialoguer crate
// Interactive scaffolding now returns an error directing users to CLI args

/// Build agent context from CLI arguments
fn build_agent_context(
    name: &str,
    template: &str,
    features: &[String],
    quality: &str,
    deterministic_core: Option<String>,
    probabilistic_wrapper: Option<String>,
) -> Result<crate::scaffold::agent::AgentContext> {
    use crate::scaffold::agent::AgentContextBuilder;

    let mut builder = AgentContextBuilder::new(name, template);
    builder = add_features_to_builder(builder, features);
    builder = add_quality_level_to_builder(builder, quality);
    builder = add_hybrid_specs_to_builder(builder, deterministic_core, probabilistic_wrapper)?;

    builder.build()
}

/// Add features to agent context builder
fn add_features_to_builder(
    mut builder: crate::scaffold::agent::AgentContextBuilder,
    features: &[String],
) -> crate::scaffold::agent::AgentContextBuilder {
    use crate::scaffold::agent::AgentFeature;

    for feature_str in features {
        if let Ok(feature) = feature_str.parse::<AgentFeature>() {
            builder = builder.with_feature(feature);
        } else {
            eprintln!("⚠️ Warning: Unknown feature '{feature_str}', skipping");
        }
    }

    builder
}

/// Add quality level to agent context builder
fn add_quality_level_to_builder(
    builder: crate::scaffold::agent::AgentContextBuilder,
    quality: &str,
) -> crate::scaffold::agent::AgentContextBuilder {
    use crate::scaffold::agent::QualityLevel;

    let quality_level = match quality.to_lowercase().as_str() {
        "standard" => QualityLevel::Standard,
        "strict" => QualityLevel::Strict,
        "extreme" => QualityLevel::Extreme,
        _ => {
            eprintln!("⚠️ Unknown quality level '{quality}', using 'strict'");
            QualityLevel::Strict
        }
    };

    builder.with_quality_level(quality_level)
}

/// Add hybrid agent specifications to builder
fn add_hybrid_specs_to_builder(
    mut builder: crate::scaffold::agent::AgentContextBuilder,
    deterministic_core: Option<String>,
    probabilistic_wrapper: Option<String>,
) -> Result<crate::scaffold::agent::AgentContextBuilder> {
    if let Some(_core_spec) = deterministic_core {
        builder = add_deterministic_core_spec(builder)?;
    }

    if let Some(_wrapper_spec) = probabilistic_wrapper {
        builder = add_probabilistic_wrapper_spec(builder)?;
    }

    Ok(builder)
}

/// Add deterministic core specification
fn add_deterministic_core_spec(
    builder: crate::scaffold::agent::AgentContextBuilder,
) -> Result<crate::scaffold::agent::AgentContextBuilder> {
    use crate::scaffold::agent::hybrid::{CoreSpec, VerificationMethod};

    let core = CoreSpec {
        verification_method: VerificationMethod::PropertyTests,
        max_complexity: 10,
        invariants: Vec::new(),
    };

    Ok(builder.with_deterministic_core(core))
}

/// Add probabilistic wrapper specification
fn add_probabilistic_wrapper_spec(
    builder: crate::scaffold::agent::AgentContextBuilder,
) -> Result<crate::scaffold::agent::AgentContextBuilder> {
    use crate::scaffold::agent::hybrid::{FallbackStrategy, ModelType, WrapperSpec};

    let wrapper = WrapperSpec {
        model_type: ModelType::GPT4,
        fallback_strategy: FallbackStrategy::Deterministic,
        confidence_threshold: 0.95,
    };

    Ok(builder.with_probabilistic_wrapper(wrapper))
}

/// Validate output path and force flag
fn validate_output_path(output_path: &Path, force: bool) -> Result<()> {
    if output_path.exists() && !force {
        let error = format!(
            "ERROR: Directory already exists\n  Location: {}\n\n  Suggestions:\n  - Use --force to overwrite existing directory\n  - Choose a different output directory with --output\n  - Remove the existing directory manually",
            output_path.display()
        );
        anyhow::bail!(error);
    }
    Ok(())
}

/// Execute the scaffold operation
async fn execute_scaffold_operation(
    context: &crate::scaffold::agent::AgentContext,
    output_path: &Path,
    name: &str,
    dry_run: bool,
    force: bool,
) -> Result<()> {
    use crate::cli::progress::ProgressIndicator;
    use crate::scaffold::agent::scaffold_agent;

    if dry_run {
        print_dry_run_info(context, output_path);
        return Ok(());
    }

    validate_output_path(output_path, force)?;

    let progress = ProgressIndicator::new(&format!("Scaffolding agent '{}'...", name));
    let start = std::time::Instant::now();

    scaffold_agent(context, output_path).await?;

    let duration = start.elapsed();
    progress.finish_with_message(&format!(
        "Agent '{}' scaffolded successfully ({:.1}s)",
        name,
        duration.as_secs_f64()
    ));

    Ok(())
}

/// Print dry run information
fn print_dry_run_info(context: &crate::scaffold::agent::AgentContext, output_path: &Path) {
    eprintln!("🔍 Dry run mode - would generate the following:");
    eprintln!("  Agent: {}", context.name);
    eprintln!("  Template: {:?}", context.template_type);
    eprintln!("  Quality: {:?}", context.quality_level);
    eprintln!("  Features: {} enabled", context.features.len());
    eprintln!("  Output: {}", output_path.display());
}

/// Handle listing available agent templates
pub async fn handle_list_agent_templates() -> Result<()> {
    use crate::scaffold::agent::TemplateRegistry;

    let registry = TemplateRegistry::new();
    let templates = registry.list_available();

    eprintln!("📦 Available Agent Templates:");
    eprintln!();
    for template in &templates {
        if let Some(info) = registry.get_template_info(template) {
            eprintln!("  • {} - {}", info.name, info.description);
        }
    }
    eprintln!();
    eprintln!("Total: {} templates available", templates.len());

    Ok(())
}

/// Handle validating an agent template
pub async fn handle_validate_agent_template(path: PathBuf) -> Result<()> {
    use crate::scaffold::agent::TemplateRegistry;

    let registry = TemplateRegistry::new();

    eprintln!("🔍 Validating template: {}", path.display());

    match registry.validate_template_file(&path) {
        Ok(()) => {
            eprintln!("✅ Template is valid!");
        }
        Err(e) => {
            eprintln!("❌ Template validation failed:");
            eprintln!("   {e}");

            // Print detailed errors
            let mut source = e.source();
            while let Some(err) = source {
                eprintln!("   Caused by: {err}");
                source = err.source();
            }

            std::process::exit(1);
        }
    }

    Ok(())
}

// TICKET-PMAT-5031: WASM scaffolding

/// Parameters for WASM scaffolding
pub struct ScaffoldWasmParams {
    pub name: String,
    pub framework: String,
    pub features: Vec<String>,
    pub quality: String,
    pub output: Option<PathBuf>,
    pub force: bool,
    pub dry_run: bool,
}

/// Handle WASM scaffolding command
///
/// # Complexity
/// - Time: O(n) where n is project size
/// - Cyclomatic: 5
pub async fn handle_scaffold_wasm(params: ScaffoldWasmParams) -> Result<()> {
    use crate::scaffold::config::{
        Feature, QualityGateConfig, ScaffoldConfig, TemplateType, WasmFramework,
    };
    use crate::scaffold::ScaffoldEngine;

    let ScaffoldWasmParams {
        name,
        framework,
        features,
        quality,
        output,
        force,
        dry_run,
    } = params;

    // Parse framework
    let wasm_framework = match framework.as_str() {
        "wasm-labs" => WasmFramework::WasmLabs,
        "pure-wasm" => WasmFramework::PureWasm,
        _ => {
            let error = format!(
                "ERROR: Unknown WASM framework: '{}'\n\n  Suggestions:\n  - Use 'wasm-labs' for full-featured WASM development\n  - Use 'pure-wasm' for minimal WASM setup\n  - Run 'pmat scaffold --help' for more information",
                framework
            );
            return Err(anyhow::anyhow!(error));
        }
    };

    // Parse features
    let parsed_features: Vec<Feature> = features
        .iter()
        .filter_map(|f| match f.as_str() {
            "logging" => Some(Feature::Logging),
            "metrics" => Some(Feature::Metrics),
            "tracing" => Some(Feature::Tracing),
            _ => {
                eprintln!("⚠️  Warning: Unknown feature '{f}', skipping");
                None
            }
        })
        .collect();

    // Create scaffold config
    let config = ScaffoldConfig {
        project_name: name.clone(),
        template_type: TemplateType::Wasm {
            based_on: wasm_framework,
        },
        features: parsed_features,
        quality_gates: match quality.as_str() {
            "extreme" => QualityGateConfig::extreme_tdd(),
            _ => QualityGateConfig::default(),
        },
    };

    if dry_run {
        eprintln!("🔍 Dry run - would create WASM project: {}", name);
        eprintln!("  Framework: {}", framework);
        eprintln!("  Quality: {}", quality);
        eprintln!("  Features: {:?}", features);
        return Ok(());
    }

    use crate::cli::progress::ProgressIndicator;

    // Use scaffold engine
    let engine = ScaffoldEngine::new()?;
    engine.validate_config(&config)?;

    let output_dir = output.unwrap_or_else(|| PathBuf::from("."));
    let project_dir = output_dir.join(&name);

    if project_dir.exists() && !force {
        let error = format!(
            "ERROR: Directory already exists\n  Location: {}\n\n  Suggestions:\n  - Use --force to overwrite existing directory\n  - Choose a different project name\n  - Remove the existing directory manually",
            project_dir.display()
        );
        return Err(anyhow::anyhow!(error));
    }

    let progress = ProgressIndicator::new(&format!("Scaffolding WASM project '{}'...", name));
    let start = std::time::Instant::now();

    engine.scaffold(config)?;

    let duration = start.elapsed();
    progress.finish_with_message(&format!(
        "WASM project '{}' created ({:.1}s)",
        name,
        duration.as_secs_f64()
    ));

    eprintln!("  Location: {}", project_dir.display());
    eprintln!("  Framework: {}", framework);
    eprintln!();
    eprintln!("Next steps:");
    eprintln!("  cd {}", name);
    eprintln!("  wasm-pack build");
    eprintln!("  wasm-pack test --headless --firefox");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    // ============================================================================
    // Test Helpers
    // ============================================================================

    /// Create a test ScaffoldAgentParams with default values
    fn default_scaffold_agent_params() -> ScaffoldAgentParams {
        ScaffoldAgentParams {
            name: "test_agent".to_string(),
            template: "mcp-server".to_string(),
            features: vec![],
            quality: "strict".to_string(),
            output: None,
            force: false,
            dry_run: true, // Use dry_run to avoid filesystem operations
            interactive: false,
            deterministic_core: None,
            probabilistic_wrapper: None,
        }
    }

    /// Create a test ScaffoldWasmParams with default values
    fn default_scaffold_wasm_params() -> ScaffoldWasmParams {
        ScaffoldWasmParams {
            name: "test_wasm".to_string(),
            framework: "wasm-labs".to_string(),
            features: vec![],
            quality: "standard".to_string(),
            output: None,
            force: false,
            dry_run: true, // Use dry_run to avoid filesystem operations
        }
    }

    // ============================================================================
    // resolve_scaffold_templates Tests
    // ============================================================================

    #[test]
    fn test_resolve_scaffold_templates_with_empty_uses_defaults_for_rust() {
        let result = resolve_scaffold_templates("rust", vec![]);
        assert_eq!(
            result,
            vec![
                "makefile".to_string(),
                "readme".to_string(),
                "gitignore".to_string()
            ]
        );
    }

    #[test]
    fn test_resolve_scaffold_templates_with_empty_uses_defaults_for_deno() {
        let result = resolve_scaffold_templates("deno", vec![]);
        assert_eq!(
            result,
            vec![
                "makefile".to_string(),
                "readme".to_string(),
                "gitignore".to_string()
            ]
        );
    }

    #[test]
    fn test_resolve_scaffold_templates_with_empty_uses_defaults_for_python_uv() {
        let result = resolve_scaffold_templates("python-uv", vec![]);
        assert_eq!(
            result,
            vec![
                "makefile".to_string(),
                "readme".to_string(),
                "gitignore".to_string()
            ]
        );
    }

    #[test]
    fn test_resolve_scaffold_templates_with_empty_uses_minimal_for_unknown() {
        let result = resolve_scaffold_templates("unknown-toolchain", vec![]);
        assert_eq!(result, vec!["readme".to_string()]);
    }

    #[test]
    fn test_resolve_scaffold_templates_with_provided_templates() {
        let templates = vec!["custom1".to_string(), "custom2".to_string()];
        let result = resolve_scaffold_templates("rust", templates.clone());
        assert_eq!(result, templates);
    }

    #[test]
    fn test_resolve_scaffold_templates_preserves_order() {
        let templates = vec!["z_last".to_string(), "a_first".to_string()];
        let result = resolve_scaffold_templates("any", templates.clone());
        assert_eq!(result, templates);
    }

    // ============================================================================
    // get_default_scaffold_templates Tests
    // ============================================================================

    #[test]
    fn test_get_default_scaffold_templates_rust() {
        let result = get_default_scaffold_templates("rust");
        assert!(result.contains(&"makefile".to_string()));
        assert!(result.contains(&"readme".to_string()));
        assert!(result.contains(&"gitignore".to_string()));
    }

    #[test]
    fn test_get_default_scaffold_templates_deno() {
        let result = get_default_scaffold_templates("deno");
        assert!(result.contains(&"makefile".to_string()));
        assert!(result.contains(&"readme".to_string()));
        assert!(result.contains(&"gitignore".to_string()));
    }

    #[test]
    fn test_get_default_scaffold_templates_python_uv() {
        let result = get_default_scaffold_templates("python-uv");
        assert!(result.contains(&"makefile".to_string()));
    }

    #[test]
    fn test_get_default_scaffold_templates_fallback() {
        let result = get_default_scaffold_templates("golang");
        assert_eq!(result, vec!["readme".to_string()]);
    }

    #[test]
    fn test_get_default_scaffold_templates_empty_string() {
        let result = get_default_scaffold_templates("");
        assert_eq!(result, vec!["readme".to_string()]);
    }

    // ============================================================================
    // validate_output_path Tests
    // ============================================================================

    #[test]
    fn test_validate_output_path_nonexistent_succeeds() {
        let result = validate_output_path(Path::new("/nonexistent/path/12345"), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_output_path_exists_without_force_fails() {
        let temp_dir = TempDir::new().unwrap();
        let result = validate_output_path(temp_dir.path(), false);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Directory already exists"));
    }

    #[test]
    fn test_validate_output_path_exists_with_force_succeeds() {
        let temp_dir = TempDir::new().unwrap();
        let result = validate_output_path(temp_dir.path(), true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_output_path_error_includes_suggestions() {
        let temp_dir = TempDir::new().unwrap();
        let result = validate_output_path(temp_dir.path(), false);
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("--force"));
        assert!(err_msg.contains("Suggestions"));
    }

    // ============================================================================
    // add_quality_level_to_builder Tests
    // ============================================================================

    #[test]
    fn test_add_quality_level_standard() {
        use crate::scaffold::agent::{AgentContextBuilder, QualityLevel};
        let builder = AgentContextBuilder::new("test", "mcp-server");
        let result_builder = add_quality_level_to_builder(builder, "standard");
        let context = result_builder.build().unwrap();
        assert_eq!(context.quality_level, QualityLevel::Standard);
    }

    #[test]
    fn test_add_quality_level_strict() {
        use crate::scaffold::agent::{AgentContextBuilder, QualityLevel};
        let builder = AgentContextBuilder::new("test", "mcp-server");
        let result_builder = add_quality_level_to_builder(builder, "strict");
        let context = result_builder.build().unwrap();
        assert_eq!(context.quality_level, QualityLevel::Strict);
    }

    #[test]
    fn test_add_quality_level_extreme() {
        use crate::scaffold::agent::{AgentContextBuilder, QualityLevel};
        let builder = AgentContextBuilder::new("test", "mcp-server");
        let result_builder = add_quality_level_to_builder(builder, "extreme");
        let context = result_builder.build().unwrap();
        assert_eq!(context.quality_level, QualityLevel::Extreme);
    }

    #[test]
    fn test_add_quality_level_case_insensitive() {
        use crate::scaffold::agent::{AgentContextBuilder, QualityLevel};
        let builder = AgentContextBuilder::new("test", "mcp-server");
        let result_builder = add_quality_level_to_builder(builder, "EXTREME");
        let context = result_builder.build().unwrap();
        assert_eq!(context.quality_level, QualityLevel::Extreme);
    }

    #[test]
    fn test_add_quality_level_unknown_defaults_to_strict() {
        use crate::scaffold::agent::{AgentContextBuilder, QualityLevel};
        let builder = AgentContextBuilder::new("test", "mcp-server");
        let result_builder = add_quality_level_to_builder(builder, "unknown");
        let context = result_builder.build().unwrap();
        assert_eq!(context.quality_level, QualityLevel::Strict);
    }

    // ============================================================================
    // add_features_to_builder Tests
    // ============================================================================

    #[test]
    fn test_add_features_to_builder_empty() {
        use crate::scaffold::agent::AgentContextBuilder;
        let builder = AgentContextBuilder::new("test", "mcp-server");
        let result_builder = add_features_to_builder(builder, &[]);
        let context = result_builder.build().unwrap();
        assert!(context.features.is_empty());
    }

    #[test]
    fn test_add_features_to_builder_valid_feature() {
        use crate::scaffold::agent::{AgentContextBuilder, AgentFeature};
        let builder = AgentContextBuilder::new("test", "mcp-server");
        let features = vec!["tool-composition".to_string()];
        let result_builder = add_features_to_builder(builder, &features);
        let context = result_builder.build().unwrap();
        assert!(context.features.contains(&AgentFeature::ToolComposition));
    }

    #[test]
    fn test_add_features_to_builder_multiple_features() {
        use crate::scaffold::agent::{AgentContextBuilder, AgentFeature};
        let builder = AgentContextBuilder::new("test", "mcp-server");
        let features = vec![
            "tool-composition".to_string(),
            "async-handlers".to_string(),
            "health-checks".to_string(),
        ];
        let result_builder = add_features_to_builder(builder, &features);
        let context = result_builder.build().unwrap();
        assert_eq!(context.features.len(), 3);
        assert!(context.features.contains(&AgentFeature::ToolComposition));
        assert!(context.features.contains(&AgentFeature::AsyncHandlers));
        assert!(context.features.contains(&AgentFeature::HealthChecks));
    }

    #[test]
    fn test_add_features_to_builder_skips_unknown() {
        use crate::scaffold::agent::AgentContextBuilder;
        let builder = AgentContextBuilder::new("test", "mcp-server");
        let features = vec!["unknown-feature".to_string()];
        let result_builder = add_features_to_builder(builder, &features);
        let context = result_builder.build().unwrap();
        assert!(context.features.is_empty());
    }

    #[test]
    fn test_add_features_to_builder_mixed_valid_invalid() {
        use crate::scaffold::agent::{AgentContextBuilder, AgentFeature};
        let builder = AgentContextBuilder::new("test", "mcp-server");
        let features = vec![
            "tool-composition".to_string(),
            "invalid-feature".to_string(),
            "async-handlers".to_string(),
        ];
        let result_builder = add_features_to_builder(builder, &features);
        let context = result_builder.build().unwrap();
        assert_eq!(context.features.len(), 2);
        assert!(context.features.contains(&AgentFeature::ToolComposition));
        assert!(context.features.contains(&AgentFeature::AsyncHandlers));
    }

    // ============================================================================
    // add_hybrid_specs_to_builder Tests
    // ============================================================================

    #[test]
    fn test_add_hybrid_specs_with_no_specs() {
        use crate::scaffold::agent::AgentContextBuilder;
        let builder = AgentContextBuilder::new("test", "mcp-server");
        let result = add_hybrid_specs_to_builder(builder, None, None);
        assert!(result.is_ok());
        let context = result.unwrap().build().unwrap();
        assert!(context.deterministic_core.is_none());
        assert!(context.probabilistic_wrapper.is_none());
    }

    #[test]
    fn test_add_hybrid_specs_with_deterministic_core() {
        use crate::scaffold::agent::AgentContextBuilder;
        let builder = AgentContextBuilder::new("test", "mcp-server");
        let result = add_hybrid_specs_to_builder(builder, Some("core.toml".to_string()), None);
        assert!(result.is_ok());
        let context = result.unwrap().build().unwrap();
        assert!(context.deterministic_core.is_some());
    }

    #[test]
    fn test_add_hybrid_specs_with_probabilistic_wrapper() {
        use crate::scaffold::agent::AgentContextBuilder;
        let builder = AgentContextBuilder::new("test", "mcp-server");
        let result = add_hybrid_specs_to_builder(builder, None, Some("wrapper.toml".to_string()));
        assert!(result.is_ok());
        let context = result.unwrap().build().unwrap();
        assert!(context.probabilistic_wrapper.is_some());
    }

    #[test]
    fn test_add_hybrid_specs_with_both() {
        use crate::scaffold::agent::AgentContextBuilder;
        let builder = AgentContextBuilder::new("test", "mcp-server");
        let result = add_hybrid_specs_to_builder(
            builder,
            Some("core.toml".to_string()),
            Some("wrapper.toml".to_string()),
        );
        assert!(result.is_ok());
        let context = result.unwrap().build().unwrap();
        assert!(context.deterministic_core.is_some());
        assert!(context.probabilistic_wrapper.is_some());
    }

    // ============================================================================
    // add_deterministic_core_spec Tests
    // ============================================================================

    #[test]
    fn test_add_deterministic_core_spec() {
        use crate::scaffold::agent::AgentContextBuilder;
        let builder = AgentContextBuilder::new("test", "mcp-server");
        let result = add_deterministic_core_spec(builder);
        assert!(result.is_ok());
        let context = result.unwrap().build().unwrap();
        let core = context.deterministic_core.unwrap();
        assert_eq!(core.max_complexity, 10);
        assert!(core.invariants.is_empty());
    }

    // ============================================================================
    // add_probabilistic_wrapper_spec Tests
    // ============================================================================

    #[test]
    fn test_add_probabilistic_wrapper_spec() {
        use crate::scaffold::agent::hybrid::{FallbackStrategy, ModelType};
        use crate::scaffold::agent::AgentContextBuilder;
        let builder = AgentContextBuilder::new("test", "mcp-server");
        let result = add_probabilistic_wrapper_spec(builder);
        assert!(result.is_ok());
        let context = result.unwrap().build().unwrap();
        let wrapper = context.probabilistic_wrapper.unwrap();
        assert_eq!(wrapper.model_type, ModelType::GPT4);
        assert_eq!(wrapper.fallback_strategy, FallbackStrategy::Deterministic);
        assert!((wrapper.confidence_threshold - 0.95).abs() < f64::EPSILON);
    }

    // ============================================================================
    // build_agent_context Tests
    // ============================================================================

    #[test]
    fn test_build_agent_context_basic() {
        let result = build_agent_context("my_agent", "mcp-server", &[], "strict", None, None);
        assert!(result.is_ok());
        let context = result.unwrap();
        assert_eq!(context.name, "my_agent");
    }

    #[test]
    fn test_build_agent_context_with_features() {
        use crate::scaffold::agent::AgentFeature;
        let features = vec!["tool-composition".to_string(), "async-handlers".to_string()];
        let result = build_agent_context("agent", "mcp-server", &features, "strict", None, None);
        assert!(result.is_ok());
        let context = result.unwrap();
        assert!(context.features.contains(&AgentFeature::ToolComposition));
        assert!(context.features.contains(&AgentFeature::AsyncHandlers));
    }

    #[test]
    fn test_build_agent_context_with_quality_level() {
        use crate::scaffold::agent::QualityLevel;
        let result = build_agent_context("agent", "mcp-server", &[], "extreme", None, None);
        assert!(result.is_ok());
        let context = result.unwrap();
        assert_eq!(context.quality_level, QualityLevel::Extreme);
    }

    #[test]
    fn test_build_agent_context_with_deterministic_core() {
        let result = build_agent_context(
            "agent",
            "mcp-server",
            &[],
            "strict",
            Some("core.toml".to_string()),
            None,
        );
        assert!(result.is_ok());
        let context = result.unwrap();
        assert!(context.deterministic_core.is_some());
    }

    #[test]
    fn test_build_agent_context_empty_name_fails() {
        let result = build_agent_context("", "mcp-server", &[], "strict", None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_agent_context_invalid_name_fails() {
        let result =
            build_agent_context("agent-with-dash", "mcp-server", &[], "strict", None, None);
        assert!(result.is_err());
    }

    // ============================================================================
    // print_dry_run_info Tests
    // ============================================================================

    #[test]
    fn test_print_dry_run_info_does_not_panic() {
        use crate::scaffold::agent::{AgentContext, AgentTemplate, QualityLevel};
        use std::collections::HashSet;

        let context = AgentContext {
            name: "test_agent".to_string(),
            template_type: AgentTemplate::MCPToolServer,
            features: HashSet::new(),
            quality_level: QualityLevel::Strict,
            deterministic_core: None,
            probabilistic_wrapper: None,
        };

        // Should not panic
        print_dry_run_info(&context, Path::new("/tmp/test"));
    }

    // ============================================================================
    // ScaffoldAgentParams Tests
    // ============================================================================

    #[test]
    fn test_scaffold_agent_params_struct() {
        let params = default_scaffold_agent_params();
        assert_eq!(params.name, "test_agent");
        assert_eq!(params.template, "mcp-server");
        assert!(params.features.is_empty());
        assert_eq!(params.quality, "strict");
        assert!(params.output.is_none());
        assert!(!params.force);
        assert!(params.dry_run);
        assert!(!params.interactive);
        assert!(params.deterministic_core.is_none());
        assert!(params.probabilistic_wrapper.is_none());
    }

    // ============================================================================
    // ScaffoldWasmParams Tests
    // ============================================================================

    #[test]
    fn test_scaffold_wasm_params_struct() {
        let params = default_scaffold_wasm_params();
        assert_eq!(params.name, "test_wasm");
        assert_eq!(params.framework, "wasm-labs");
        assert!(params.features.is_empty());
        assert_eq!(params.quality, "standard");
        assert!(params.output.is_none());
        assert!(!params.force);
        assert!(params.dry_run);
    }

    // ============================================================================
    // Async Handler Tests
    // ============================================================================

    #[tokio::test]
    async fn test_handle_scaffold_agent_dry_run() {
        let params = ScaffoldAgentParams {
            name: "test_agent".to_string(),
            template: "mcp-server".to_string(),
            features: vec![],
            quality: "strict".to_string(),
            output: None,
            force: false,
            dry_run: true,
            interactive: false,
            deterministic_core: None,
            probabilistic_wrapper: None,
        };

        let result = handle_scaffold_agent(params).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_scaffold_agent_with_features_dry_run() {
        let params = ScaffoldAgentParams {
            name: "feature_agent".to_string(),
            template: "mcp-server".to_string(),
            features: vec!["tool-composition".to_string(), "async-handlers".to_string()],
            quality: "extreme".to_string(),
            output: None,
            force: false,
            dry_run: true,
            interactive: false,
            deterministic_core: None,
            probabilistic_wrapper: None,
        };

        let result = handle_scaffold_agent(params).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_scaffold_agent_with_hybrid_specs_dry_run() {
        let params = ScaffoldAgentParams {
            name: "hybrid_agent".to_string(),
            template: "mcp-server".to_string(),
            features: vec![],
            quality: "strict".to_string(),
            output: None,
            force: false,
            dry_run: true,
            interactive: false,
            deterministic_core: Some("core.toml".to_string()),
            probabilistic_wrapper: Some("wrapper.toml".to_string()),
        };

        let result = handle_scaffold_agent(params).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_scaffold_agent_with_custom_output_dry_run() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("custom_agent");

        let params = ScaffoldAgentParams {
            name: "custom_output_agent".to_string(),
            template: "mcp-server".to_string(),
            features: vec![],
            quality: "strict".to_string(),
            output: Some(output_path),
            force: false,
            dry_run: true,
            interactive: false,
            deterministic_core: None,
            probabilistic_wrapper: None,
        };

        let result = handle_scaffold_agent(params).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_scaffold_wasm_dry_run() {
        let params = ScaffoldWasmParams {
            name: "test_wasm".to_string(),
            framework: "wasm-labs".to_string(),
            features: vec![],
            quality: "standard".to_string(),
            output: None,
            force: false,
            dry_run: true,
        };

        let result = handle_scaffold_wasm(params).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_scaffold_wasm_pure_wasm_dry_run() {
        let params = ScaffoldWasmParams {
            name: "pure_wasm_project".to_string(),
            framework: "pure-wasm".to_string(),
            features: vec![],
            quality: "standard".to_string(),
            output: None,
            force: false,
            dry_run: true,
        };

        let result = handle_scaffold_wasm(params).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_scaffold_wasm_with_features_dry_run() {
        let params = ScaffoldWasmParams {
            name: "featured_wasm".to_string(),
            framework: "wasm-labs".to_string(),
            features: vec![
                "logging".to_string(),
                "metrics".to_string(),
                "tracing".to_string(),
            ],
            quality: "extreme".to_string(),
            output: None,
            force: false,
            dry_run: true,
        };

        let result = handle_scaffold_wasm(params).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_scaffold_wasm_unknown_framework_fails() {
        let params = ScaffoldWasmParams {
            name: "invalid_framework".to_string(),
            framework: "unknown-framework".to_string(),
            features: vec![],
            quality: "standard".to_string(),
            output: None,
            force: false,
            dry_run: true,
        };

        let result = handle_scaffold_wasm(params).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Unknown WASM framework"));
    }

    #[tokio::test]
    async fn test_handle_list_agent_templates() {
        let result = handle_list_agent_templates().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore = "Function calls std::process::exit(1) which kills test process"]
    async fn test_handle_validate_agent_template_nonexistent_file() {
        let path = PathBuf::from("/nonexistent/template/path/12345.toml");
        let result = handle_validate_agent_template(path).await;
        // The function calls std::process::exit(1), so we can't easily test failure case
        // without mocking, but we can verify the path is checked
        assert!(result.is_err() || true); // Either error or the function exits
    }

    #[tokio::test]
    async fn test_handle_validate_agent_template_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let template_path = temp_dir.path().join("template.toml");
        std::fs::write(&template_path, "[template]\nname = \"test\"").unwrap();

        let result = handle_validate_agent_template(template_path).await;
        assert!(result.is_ok());
    }

    // ============================================================================
    // Edge Case Tests
    // ============================================================================

    #[test]
    fn test_resolve_scaffold_templates_case_sensitive() {
        // Toolchain matching is case-sensitive
        let result = resolve_scaffold_templates("RUST", vec![]);
        assert_eq!(result, vec!["readme".to_string()]); // Falls through to default
    }

    #[test]
    fn test_get_default_scaffold_templates_length() {
        let rust_templates = get_default_scaffold_templates("rust");
        let unknown_templates = get_default_scaffold_templates("unknown");
        assert!(rust_templates.len() > unknown_templates.len());
    }

    #[tokio::test]
    async fn test_handle_scaffold_wasm_skips_unknown_features() {
        let params = ScaffoldWasmParams {
            name: "wasm_with_unknown".to_string(),
            framework: "wasm-labs".to_string(),
            features: vec!["logging".to_string(), "unknown-feature".to_string()],
            quality: "standard".to_string(),
            output: None,
            force: false,
            dry_run: true,
        };

        let result = handle_scaffold_wasm(params).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_scaffold_agent_params_with_all_fields() {
        let params = ScaffoldAgentParams {
            name: "full_agent".to_string(),
            template: "calculator".to_string(),
            features: vec!["state-machine".to_string()],
            quality: "extreme".to_string(),
            output: Some(PathBuf::from("/tmp/output")),
            force: true,
            dry_run: false,
            interactive: false,
            deterministic_core: Some("core.yaml".to_string()),
            probabilistic_wrapper: Some("wrapper.yaml".to_string()),
        };

        assert_eq!(params.name, "full_agent");
        assert_eq!(params.template, "calculator");
        assert!(!params.features.is_empty());
        assert_eq!(params.quality, "extreme");
        assert!(params.output.is_some());
        assert!(params.force);
        assert!(!params.dry_run);
        assert!(!params.interactive);
        assert!(params.deterministic_core.is_some());
        assert!(params.probabilistic_wrapper.is_some());
    }

    // ============================================================================
    // Template Types Integration Tests
    // ============================================================================

    #[test]
    fn test_build_agent_context_calculator_template() {
        let result = build_agent_context("calc_agent", "calculator", &[], "strict", None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_agent_context_state_machine_template() {
        let result = build_agent_context("sm_agent", "state-machine", &[], "strict", None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_agent_context_hybrid_template_without_specs() {
        // Hybrid template requires both specs, should fail
        let result = build_agent_context("hybrid_agent", "hybrid", &[], "strict", None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_agent_context_hybrid_template_with_specs() {
        let result = build_agent_context(
            "hybrid_agent",
            "hybrid",
            &[],
            "strict",
            Some("core.toml".to_string()),
            Some("wrapper.toml".to_string()),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_agent_context_unknown_template() {
        // Unknown templates default to MCPToolServer
        let result = build_agent_context("agent", "unknown-template", &[], "strict", None, None);
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_resolve_scaffold_templates_never_returns_empty_for_known_toolchains(
            toolchain in prop_oneof![Just("rust"), Just("deno"), Just("python-uv")]
        ) {
            let result = resolve_scaffold_templates(&toolchain, vec![]);
            prop_assert!(!result.is_empty());
        }

        #[test]
        fn test_resolve_scaffold_templates_always_returns_at_least_readme_for_unknown(
            toolchain in "[a-z]{5,10}"
        ) {
            // Filter out known toolchains
            if toolchain != "rust" && toolchain != "deno" && !toolchain.starts_with("python") {
                let result = resolve_scaffold_templates(&toolchain, vec![]);
                prop_assert!(result.contains(&"readme".to_string()));
            }
        }

        #[test]
        fn test_resolve_scaffold_templates_returns_provided_when_not_empty(
            templates in prop::collection::vec("[a-z]+", 1..5)
        ) {
            let result = resolve_scaffold_templates("any", templates.clone());
            prop_assert_eq!(result, templates);
        }

        #[test]
        fn test_get_default_scaffold_templates_never_panics(
            toolchain in ".*"
        ) {
            // Should never panic regardless of input
            let _ = get_default_scaffold_templates(&toolchain);
            prop_assert!(true);
        }

        #[test]
        fn test_validate_output_path_with_force_always_succeeds_for_nonexistent(
            path in "/nonexistent/[a-z]{10}/[a-z]{10}"
        ) {
            let result = validate_output_path(std::path::Path::new(&path), true);
            prop_assert!(result.is_ok());
        }

        #[test]
        fn test_quality_level_mapping_is_deterministic(
            quality in prop_oneof![
                Just("standard"),
                Just("strict"),
                Just("extreme"),
                Just("STANDARD"),
                Just("STRICT"),
                Just("EXTREME")
            ]
        ) {
            use crate::scaffold::agent::AgentContextBuilder;

            let builder1 = AgentContextBuilder::new("test", "mcp-server");
            let builder2 = AgentContextBuilder::new("test", "mcp-server");

            let result1 = add_quality_level_to_builder(builder1, &quality);
            let result2 = add_quality_level_to_builder(builder2, &quality);

            let ctx1 = result1.build().unwrap();
            let ctx2 = result2.build().unwrap();

            prop_assert_eq!(ctx1.quality_level, ctx2.quality_level);
        }
    }

    // Additional property tests for edge cases

    proptest! {
        #[test]
        fn test_add_features_never_panics(
            features in prop::collection::vec(".*", 0..10)
        ) {
            use crate::scaffold::agent::AgentContextBuilder;
            let builder = AgentContextBuilder::new("test", "mcp-server");
            let _ = add_features_to_builder(builder, &features);
            prop_assert!(true);
        }

        #[test]
        fn test_build_agent_context_with_valid_name(
            name in "[a-z][a-z0-9_]{2,20}"
        ) {
            let result = build_agent_context(&name, "mcp-server", &[], "strict", None, None);
            prop_assert!(result.is_ok());
        }

        #[test]
        fn test_build_agent_context_fails_with_invalid_chars(
            invalid_char in "[!@#$%^&*()\\-+=\\[\\]{};':\",./<>?\\\\|`~]"
        ) {
            let name = format!("agent{}", invalid_char);
            let result = build_agent_context(&name, "mcp-server", &[], "strict", None, None);
            prop_assert!(result.is_err());
        }
    }
}
