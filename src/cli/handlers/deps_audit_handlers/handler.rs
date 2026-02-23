#![cfg_attr(coverage_nightly, coverage(off))]

use std::path::Path;

use anyhow::Result;

use super::classify::analyze_dep;
use super::graph::{analyze_dependency_graph, apply_graph_analysis};
use super::output::{print_pareto_report, print_text_report};
use super::pareto::run_pareto_analysis;
use super::parser::{parse_cargo_lock, parse_cargo_toml};
use super::types::{DepCategory, DepsAuditReport, SortMode};

/// Handle the deps-audit command
pub fn handle_deps_audit(
    path: &Path,
    format: &str,
    show_all: bool,
    pareto: bool,
    sort_by: &str,
) -> Result<()> {
    let cargo_toml = path.join("Cargo.toml");
    if !cargo_toml.exists() {
        anyhow::bail!("No Cargo.toml found at {}", path.display());
    }

    let (deps, dev_deps) = parse_cargo_toml(&cargo_toml)?;

    // Parse Cargo.lock for graph analysis
    let (all_packages, edges) = parse_cargo_lock(path)?;
    let direct_dep_names: Vec<String> = deps.iter().map(|(n, _, _)| n.clone()).collect();

    // Build and analyze dependency graph
    let graph_analysis = analyze_dependency_graph(&direct_dep_names, &all_packages, &edges);

    let mut all_deps: Vec<_> = deps
        .iter()
        .map(|(name, version, is_dev)| analyze_dep(name, version, *is_dev))
        .collect();

    let dev_analyses: Vec<_> = dev_deps
        .iter()
        .map(|(name, version, _)| analyze_dep(name, version, true))
        .collect();

    all_deps.extend(dev_analyses);

    // Apply graph analysis to populate graph metrics
    apply_graph_analysis(&mut all_deps, &graph_analysis);

    // Sort based on user preference
    let sort_mode = SortMode::from_str(sort_by);
    all_deps.sort_by(|a, b| match sort_mode {
        SortMode::Transitive => b.transitive_count.cmp(&a.transitive_count),
        SortMode::Size => b.estimated_size_kb.cmp(&a.estimated_size_kb),
        SortMode::PageRank => b
            .pagerank_score
            .partial_cmp(&a.pagerank_score)
            .unwrap_or(std::cmp::Ordering::Equal),
        SortMode::Name => a.name.cmp(&b.name),
        SortMode::Category => {
            let priority = |cat: DepCategory| match cat {
                DepCategory::Removable => 0,
                DepCategory::Heavy => 1,
                DepCategory::Replaceable => 2,
                DepCategory::DevOnly => 3,
                DepCategory::Core => 4,
                DepCategory::Sovereign => 5,
            };
            priority(a.category).cmp(&priority(b.category))
        }
    });

    // Calculate stats
    let sovereign_count = all_deps
        .iter()
        .filter(|d| d.category == DepCategory::Sovereign)
        .count();
    let replaceable_count = all_deps
        .iter()
        .filter(|d| d.category == DepCategory::Replaceable)
        .count();
    let removable_count = all_deps
        .iter()
        .filter(|d| d.category == DepCategory::Removable)
        .count();
    let heavy_count = all_deps
        .iter()
        .filter(|d| d.category == DepCategory::Heavy)
        .count();
    let orphan_count = all_deps.iter().filter(|d| d.is_orphan).count();
    let bridge_count = all_deps.iter().filter(|d| d.is_bridge).count();
    let estimated_savings: usize = all_deps
        .iter()
        .filter(|d| {
            matches!(
                d.category,
                DepCategory::Removable | DepCategory::Heavy | DepCategory::Replaceable
            )
        })
        .map(|d| d.estimated_size_kb)
        .sum();

    // Top critical deps by PageRank
    let mut top_critical: Vec<(String, f32)> = all_deps
        .iter()
        .filter(|d| d.pagerank_score > 0.0)
        .map(|d| (d.name.clone(), d.pagerank_score))
        .collect();
    top_critical.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    top_critical.truncate(10);

    // Removal candidates: orphan deps that are removable or heavy
    let removal_candidates: Vec<String> = all_deps
        .iter()
        .filter(|d| {
            d.is_orphan && matches!(d.category, DepCategory::Removable | DepCategory::Heavy)
        })
        .map(|d| d.name.clone())
        .collect();

    // Generate recommendations
    let mut recommendations = Vec::new();

    if heavy_count > 0 {
        recommendations.push(format!(
            "Consider removing/replacing {} heavy dependencies to reduce binary size by ~{}KB",
            heavy_count, estimated_savings
        ));
    }

    // SWC recommendation
    let swc_deps: Vec<_> = all_deps
        .iter()
        .filter(|d| d.name.starts_with("swc_"))
        .collect();
    if !swc_deps.is_empty() {
        recommendations.push(
            "SWC dependencies add ~15MB. Consider using tree-sitter-typescript only.".to_string(),
        );
    }

    // Git2 recommendation
    if all_deps.iter().any(|d| d.name == "git2") {
        recommendations.push(
            "git2 (libgit2) adds ~6MB. Consider shelling out to `git` CLI instead.".to_string(),
        );
    }

    // Octocrab recommendation
    if all_deps.iter().any(|d| d.name == "octocrab") {
        recommendations.push(
            "octocrab adds ~5MB. Consider using ureq + serde_json for GitHub API.".to_string(),
        );
    }

    // Graph-based recommendations
    if !removal_candidates.is_empty() {
        recommendations.push(format!(
            "Graph analysis: {} orphan deps are safe to remove: {}",
            removal_candidates.len(),
            removal_candidates
                .iter()
                .take(5)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    // High transitive count warning
    let high_transitive: Vec<_> = all_deps
        .iter()
        .filter(|d| d.transitive_count > 50)
        .collect();
    if !high_transitive.is_empty() {
        let names: Vec<_> = high_transitive
            .iter()
            .map(|d| format!("{}({})", d.name, d.transitive_count))
            .collect();
        recommendations.push(format!(
            "High transitive deps (each brings 50+ deps): {}",
            names.join(", ")
        ));
    }

    // Run Pareto analysis if requested (before consuming all_deps)
    if pareto {
        let pareto_entries = run_pareto_analysis(&all_deps, path);
        print_pareto_report(&pareto_entries);
        return Ok(());
    }

    let report = DepsAuditReport {
        total_deps: all_deps.len(),
        direct_deps: deps.len() + dev_deps.len(),
        transitive_deps: all_packages
            .len()
            .saturating_sub(deps.len() + dev_deps.len()),
        sovereign_deps: sovereign_count,
        replaceable_deps: replaceable_count,
        removable_deps: removable_count,
        heavy_deps: heavy_count,
        orphan_deps: orphan_count,
        bridge_deps: bridge_count,
        estimated_savings_kb: estimated_savings,
        dependencies: if show_all {
            all_deps
        } else {
            all_deps
                .into_iter()
                .filter(|d| !matches!(d.category, DepCategory::Core | DepCategory::Sovereign))
                .collect()
        },
        recommendations,
        top_critical,
        removal_candidates,
    };

    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        "yaml" => {
            println!("{}", serde_yaml_ng::to_string(&report)?);
        }
        _ => {
            print_text_report(&report);
        }
    }

    Ok(())
}
