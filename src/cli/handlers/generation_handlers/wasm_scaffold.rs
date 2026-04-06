#![cfg_attr(coverage_nightly, coverage(off))]
//! WASM project scaffolding handlers
//!
//! TICKET-PMAT-5031: WASM scaffolding

use anyhow::Result;
use std::path::PathBuf;

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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
