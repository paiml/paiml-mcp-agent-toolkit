//! Template generation and scaffolding handlers
//!
//! This module contains the extracted implementations for template generation,
//! project scaffolding, and template validation operations.

// use crate::cli::*; // Currently unused
use crate::services::template_service::*;
use crate::stateless_server::StatelessTemplateServer;
use anyhow::Result;
use serde_json::Value;
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
    let params_json = super::super::stubs::params_to_json(params);

    let result = generate_template(server.as_ref(), &uri, params_json).await?;

    if let Some(path) = output {
        if create_dirs {
            tokio::fs::create_dir_all(path.parent().unwrap()).await?;
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

    let params_json = super::super::stubs::params_to_json(params);

    // If no templates specified, use default templates for the toolchain
    let templates_to_use = if templates.is_empty() {
        match toolchain.as_str() {
            "rust" => vec![
                "makefile".to_string(),
                "readme".to_string(),
                "gitignore".to_string(),
            ],
            "deno" => vec![
                "makefile".to_string(),
                "readme".to_string(),
                "gitignore".to_string(),
            ],
            "python-uv" => vec![
                "makefile".to_string(),
                "readme".to_string(),
                "gitignore".to_string(),
            ],
            _ => vec!["readme".to_string()],
        }
    } else {
        templates
    };

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
            eprintln!("❌ Failed to write file: {}", e);
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

/// Handle template validation command
pub async fn handle_validate(
    server: Arc<StatelessTemplateServer>,
    uri: String,
    params: Vec<(String, Value)>,
) -> Result<()> {
    let params_json = super::super::stubs::params_to_json(params);
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
    use crate::scaffold::agent::{
        AgentContextBuilder, AgentFeature, InteractiveScaffolder, QualityLevel, scaffold_agent,
    };

    // Use interactive mode if requested
    if interactive {
        let mut scaffolder = InteractiveScaffolder::new();
        let context = scaffolder.run()?;
        
        let output_path = output.unwrap_or_else(|| PathBuf::from(&context.name));
        
        if !dry_run {
            if output_path.exists() && !force {
                anyhow::bail!("Directory {} already exists. Use --force to overwrite.", output_path.display());
            }
            
            scaffold_agent(&context, &output_path).await?;
            eprintln!("✅ Agent '{}' scaffolded successfully at {}", context.name, output_path.display());
        } else {
            eprintln!("🔍 Dry run - would generate agent '{}' at {}", context.name, output_path.display());
        }
        
        return Ok(());
    }

    // Build context from CLI arguments
    let mut builder = AgentContextBuilder::new(&name, &template);
    
    // Parse and add features
    for feature_str in &features {
        if let Ok(feature) = feature_str.parse::<AgentFeature>() {
            builder = builder.with_feature(feature);
        } else {
            eprintln!("⚠️ Warning: Unknown feature '{}', skipping", feature_str);
        }
    }
    
    // Parse quality level
    let quality_level = match quality.to_lowercase().as_str() {
        "standard" => QualityLevel::Standard,
        "strict" => QualityLevel::Strict,
        "extreme" => QualityLevel::Extreme,
        _ => {
            eprintln!("⚠️ Unknown quality level '{}', using 'strict'", quality);
            QualityLevel::Strict
        }
    };
    builder = builder.with_quality_level(quality_level);
    
    // Handle hybrid agent specifications
    if let Some(_core_spec) = deterministic_core {
        // Parse deterministic core spec (simplified for now)
        use crate::scaffold::agent::hybrid::{CoreSpec, VerificationMethod};
        let core = CoreSpec {
            verification_method: VerificationMethod::PropertyTests,
            max_complexity: 10,
            invariants: Vec::new(),
        };
        builder = builder.with_deterministic_core(core);
    }
    
    if let Some(_wrapper_spec) = probabilistic_wrapper {
        // Parse probabilistic wrapper spec (simplified for now)
        use crate::scaffold::agent::hybrid::{WrapperSpec, ModelType, FallbackStrategy};
        let wrapper = WrapperSpec {
            model_type: ModelType::GPT4,
            fallback_strategy: FallbackStrategy::Deterministic,
            confidence_threshold: 0.95,
        };
        builder = builder.with_probabilistic_wrapper(wrapper);
    }
    
    let context = builder.build()?;
    let output_path = output.unwrap_or_else(|| PathBuf::from(&name));
    
    if dry_run {
        eprintln!("🔍 Dry run mode - would generate the following:");
        eprintln!("  Agent: {}", context.name);
        eprintln!("  Template: {:?}", context.template_type);
        eprintln!("  Quality: {:?}", context.quality_level);
        eprintln!("  Features: {} enabled", context.features.len());
        eprintln!("  Output: {}", output_path.display());
    } else {
        if output_path.exists() && !force {
            anyhow::bail!("Directory {} already exists. Use --force to overwrite.", output_path.display());
        }
        
        scaffold_agent(&context, &output_path).await?;
        eprintln!("✅ Agent '{}' scaffolded successfully at {}", name, output_path.display());
    }
    
    Ok(())
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
        Ok(_) => {
            eprintln!("✅ Template is valid!");
        }
        Err(e) => {
            eprintln!("❌ Template validation failed:");
            eprintln!("   {}", e);
            
            // Print detailed errors
            let mut source = e.source();
            while let Some(err) = source {
                eprintln!("   Caused by: {}", err);
                source = err.source();
            }
            
            std::process::exit(1);
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    // use super::*; // Unused in simple tests

    #[test]
    fn test_generation_handlers_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
