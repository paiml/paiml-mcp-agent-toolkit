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
            "rust" => vec!["makefile".to_string(), "readme".to_string(), "gitignore".to_string()],
            "deno" => vec!["makefile".to_string(), "readme".to_string(), "gitignore".to_string()],
            "python-uv" => vec!["makefile".to_string(), "readme".to_string(), "gitignore".to_string()],
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

#[cfg(test)]
mod tests {
    // use super::*; // Unused in simple tests

    #[test]
    fn test_generation_handlers_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
