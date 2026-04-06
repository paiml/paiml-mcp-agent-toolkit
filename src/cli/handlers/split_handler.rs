#![cfg_attr(coverage_nightly, coverage(off))]
//! Handler for `pmat split` command — suggests and executes semantic file splits.

use crate::cli::colors as c;
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
                "{} {} is {} lines (under 500-line threshold). Showing plan anyway.",
                c::warn(""),
                c::path(&file_path),
                c::number(&line_count.to_string())
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
                println!("\n{}", c::label("Executing split..."));
                let created = execute_split(&plan, &project_path)?;
                println!(
                    "{} Created {} files:",
                    c::pass(""),
                    c::number(&created.len().to_string())
                );
                for f in &created {
                    println!("  {}", c::path(&f.display().to_string()));
                }
                println!(
                    "\n{}",
                    c::dim("Note: Review generated files and update the source file manually.")
                );
            }
        }
        None => {
            eprintln!(
                "{} No functions found in {} {}",
                c::fail(""),
                c::path(&file_path),
                c::dim("(file may not be indexed)")
            );
            std::process::exit(1);
        }
    }

    Ok(())
}

fn normalize_file_path(file: &Path, project_root: &Path) -> Result<String> {
    debug_assert!(file.exists(), "file must exist: {}", file.display());
    debug_assert!(
        project_root.exists(),
        "project_root must exist: {}",
        project_root.display()
    );
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
        eprintln!(
            "{} Written to {}",
            c::pass(""),
            c::path(&path.display().to_string())
        );
    } else {
        println!("{}", json);
    }
    Ok(())
}

fn output_text(plan: &SplitPlan) {
    println!(
        "{} {}",
        c::label("Split Plan for:"),
        c::path(&plan.source_file)
    );
    println!(
        "{}: ~{}",
        c::dim("Total lines"),
        c::number(&plan.total_lines.to_string())
    );
    println!(
        "{}: {}",
        c::dim("Modularity"),
        c::number(&format!("{:.3}", plan.modularity))
    );
    println!(
        "{}: {}",
        c::dim("Clusters"),
        c::number(&plan.clusters.len().to_string())
    );
    println!(
        "{}: {}",
        c::dim("Unclustered items"),
        c::number(&plan.unclustered.len().to_string())
    );
    println!();

    for (i, cluster) in plan.clusters.iter().enumerate() {
        println!(
            "{} {} ({}: {}, {}: {:.0}%)",
            c::label(&format!("Cluster {}", i + 1)),
            c::BOLD,
            c::dim("signal"),
            cluster.naming_signal,
            c::dim("confidence"),
            cluster.confidence * 100.0
        );
        println!(
            "  {} ~{} lines, {}: {:.2}",
            c::dim(""),
            cluster.estimated_lines,
            c::dim("cohesion"),
            cluster.cohesion
        );
        println!("  {}", c::subheader(&cluster.suggested_name));
        for item in &cluster.items {
            println!(
                "    {} {} {} (L{}-L{})",
                c::dim(""),
                c::dim(&item.definition_type),
                c::label(&item.name),
                item.line_range.0,
                item.line_range.1
            );
        }
        println!();
    }

    if !plan.unclustered.is_empty() {
        println!("{}", c::subheader("Unclustered:"));
        for item in &plan.unclustered {
            println!(
                "  {} {} (L{}-L{})",
                c::dim(&item.definition_type),
                c::label(&item.name),
                item.line_range.0,
                item.line_range.1
            );
        }
        println!();
    }

    if !plan.impact.importing_files.is_empty() {
        println!(
            "{} {} files import this module:",
            c::subheader("Impact"),
            c::number(&plan.impact.importing_files.len().to_string())
        );
        for f in &plan.impact.importing_files {
            println!("  {}", c::path(f));
        }
    }
}
