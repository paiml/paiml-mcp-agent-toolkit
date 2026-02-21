#![cfg_attr(coverage_nightly, coverage(off))]
//! Handler for `pmat split` command — suggests and executes semantic file splits.

use crate::services::agent_context::AgentContextIndex;
use crate::services::file_split::{execute_split, suggest_split, SplitPlan};
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Output format for split command
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SplitOutputFormat {
    Text,
    Json,
}

/// Configuration for the split command
pub struct SplitConfig {
    pub file: PathBuf,
    pub project_path: PathBuf,
    pub execute: bool,
    pub format: SplitOutputFormat,
    pub output: Option<PathBuf>,
    pub min_cluster_lines: usize,
    pub resolution: f64,
}

/// Handle the `pmat split` command.
pub async fn handle_split(config: SplitConfig) -> Result<()> {
    let project_path = config
        .project_path
        .canonicalize()
        .unwrap_or(config.project_path.clone());

    // Build or load index
    let index = AgentContextIndex::build(&project_path).map_err(|e| anyhow::anyhow!(e))?;

    // Normalize file path to be relative to project root
    let file_path = normalize_file_path(&config.file, &project_path)?;

    // Check file size warning
    let abs_file = project_path.join(&file_path);
    if abs_file.exists() {
        let content = std::fs::read_to_string(&abs_file)?;
        let line_count = content.lines().count();
        if line_count < 500 {
            eprintln!(
                "Warning: {} is {} lines (under 500-line threshold). Showing plan anyway.",
                file_path, line_count
            );
        }
    }

    // Run split analysis
    let plan = suggest_split(
        &index,
        &file_path,
        config.resolution,
        config.min_cluster_lines,
    );

    match plan {
        Some(plan) => {
            // Output the plan
            match config.format {
                SplitOutputFormat::Json => output_json(&plan, config.output.as_deref())?,
                SplitOutputFormat::Text => output_text(&plan),
            }

            // Execute if requested
            if config.execute {
                println!("\nExecuting split...");
                let created = execute_split(&plan, &project_path)?;
                println!("Created {} files:", created.len());
                for f in &created {
                    println!("  {}", f.display());
                }
                println!("\nNote: Review generated files and update the source file manually.");
            }
        }
        None => {
            eprintln!(
                "No functions found in {} (file may not be indexed)",
                file_path
            );
            std::process::exit(1);
        }
    }

    Ok(())
}

fn normalize_file_path(file: &Path, project_root: &Path) -> Result<String> {
    let abs_file = if file.is_absolute() {
        file.to_path_buf()
    } else {
        std::env::current_dir()?.join(file)
    };

    let rel = abs_file
        .strip_prefix(project_root)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| file.to_string_lossy().to_string());

    Ok(rel)
}

fn output_json(plan: &SplitPlan, output: Option<&Path>) -> Result<()> {
    let json = serde_json::to_string_pretty(plan)?;
    if let Some(path) = output {
        std::fs::write(path, &json)?;
        eprintln!("Written to {}", path.display());
    } else {
        println!("{}", json);
    }
    Ok(())
}

fn output_text(plan: &SplitPlan) {
    println!("Split Plan for: {}", plan.source_file);
    println!("Total lines: ~{}", plan.total_lines);
    println!("Modularity: {:.3}", plan.modularity);
    println!("Clusters: {}", plan.clusters.len());
    println!("Unclustered items: {}", plan.unclustered.len());
    println!();

    for (i, cluster) in plan.clusters.iter().enumerate() {
        println!(
            "Cluster {} — {} (signal: {}, confidence: {:.0}%)",
            i + 1,
            cluster.suggested_name,
            cluster.naming_signal,
            cluster.confidence * 100.0
        );
        println!(
            "  ~{} lines, cohesion: {:.2}",
            cluster.estimated_lines, cluster.cohesion
        );
        for item in &cluster.items {
            println!(
                "    {} {} (L{}-L{})",
                item.definition_type, item.name, item.line_range.0, item.line_range.1
            );
        }
        println!();
    }

    if !plan.unclustered.is_empty() {
        println!("Unclustered:");
        for item in &plan.unclustered {
            println!(
                "  {} {} (L{}-L{})",
                item.definition_type, item.name, item.line_range.0, item.line_range.1
            );
        }
        println!();
    }

    if !plan.impact.importing_files.is_empty() {
        println!(
            "Impact — {} files import this module:",
            plan.impact.importing_files.len()
        );
        for f in &plan.impact.importing_files {
            println!("  {}", f);
        }
    }
}
