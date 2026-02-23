#![cfg_attr(coverage_nightly, coverage(off))]
//! Agent scaffolding handlers and context builders

use anyhow::Result;
use std::path::Path;
use std::path::PathBuf;

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
pub(super) fn build_agent_context(
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
pub(super) fn add_features_to_builder(
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
pub(super) fn add_quality_level_to_builder(
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
pub(super) fn add_hybrid_specs_to_builder(
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
pub(super) fn add_deterministic_core_spec(
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
pub(super) fn add_probabilistic_wrapper_spec(
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
pub(super) fn validate_output_path(output_path: &Path, force: bool) -> Result<()> {
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
pub(super) fn print_dry_run_info(context: &crate::scaffold::agent::AgentContext, output_path: &Path) {
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
