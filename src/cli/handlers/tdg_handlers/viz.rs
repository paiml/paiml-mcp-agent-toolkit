#![cfg_attr(coverage_nightly, coverage(off))]
//! TDG visualization: terminal graph rendering via trueno-viz
//!
//! Uses force-directed layout with PageRank-based criticality scoring.
//! Supports multiple themes including colorblind-safe (Okabe-Ito palette).

use super::TdgCommandConfig;
use crate::tdg::TdgAnalyzer;
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Collect all Rust files from path, excluding target directory
fn collect_rust_files(path: &Path) -> Vec<PathBuf> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    use walkdir::WalkDir;
    if path.is_file() {
        return vec![path.to_path_buf()];
    }
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().is_some_and(|ext| ext == "rs")
                && !e.path().to_string_lossy().contains("/target/")
        })
        .map(|e| e.path().to_path_buf())
        .collect()
}

/// Parse visualization theme from string
fn parse_viz_theme(theme_str: &str) -> crate::viz::terminal::TerminalTheme {
    use crate::viz::terminal::TerminalTheme;
    match theme_str.to_lowercase().as_str() {
        "high-contrast" | "highcontrast" => TerminalTheme::HighContrast,
        "light" => TerminalTheme::Light,
        "colorblind-safe" | "colorblind" | "cb" => TerminalTheme::ColorblindSafe,
        _ => TerminalTheme::Default,
    }
}

/// Print visualization legend and critical functions
fn print_viz_legend(
    tdg_graph: &crate::tdg::tdg_graph::TdgGraph,
    theme: crate::viz::terminal::TerminalTheme,
) {
    println!("\n--- TDG Dependency Graph ---");
    println!("Theme: {:?}", theme);
    println!("Nodes: {} functions", tdg_graph.num_nodes());
    println!("Edges: {} dependencies", tdg_graph.num_edges());

    let critical = tdg_graph.critical_functions();
    if !critical.is_empty() {
        println!("\nTop Critical Functions (by PageRank):");
        for (i, (name, score)) in critical.iter().take(10).enumerate() {
            println!("  {}. {} (score: {:.4})", i + 1, name, score);
        }
    }
}

/// Handle --viz mode: render TDG dependency graph in terminal
///
/// Uses trueno-viz force-directed layout with PageRank-based criticality scoring.
/// Supports multiple themes including colorblind-safe (Okabe-Ito palette).
pub(super) async fn handle_viz_mode(
    _analyzer: &TdgAnalyzer,
    config: &TdgCommandConfig,
) -> Result<()> {
    use crate::tdg::function_analyzer::FunctionAnalyzer;
    use crate::tdg::tdg_graph::TdgGraph;
    use crate::viz::terminal::{RenderConfig, Visualizable};

    let mut tdg_graph = TdgGraph::new();
    let mut func_analyzer = FunctionAnalyzer::new()?;
    let rust_files = collect_rust_files(&config.path);

    let mut all_functions = Vec::new();
    for file_path in &rust_files {
        if let Ok(functions) = func_analyzer.analyze_file(file_path) {
            for func in functions {
                let func_name = format!("{}::{}", file_path.display(), func.name);
                let _ = tdg_graph.add_function(func_name.clone());
                all_functions.push((file_path.clone(), func_name));
            }
        }
    }

    // Add edges: functions in same file are likely connected
    for (i, (file1, name1)) in all_functions.iter().enumerate() {
        for (file2, name2) in all_functions.iter().skip(i + 1) {
            if file1 == file2 {
                let _ = tdg_graph.add_edge(name1, name2);
            }
        }
    }

    tdg_graph.update_criticality()?;

    let theme = parse_viz_theme(&config.viz_theme);
    let render_config = RenderConfig {
        width: 120,
        height: 40,
        theme,
        mode: trueno_viz::output::TerminalMode::AnsiTrueColor,
        iterations: 100,
        critical_threshold: 0.5,
        max_nodes: 50,
        show_labels: true,
    };

    println!("{}", tdg_graph.render_terminal(&render_config)?);
    print_viz_legend(&tdg_graph, theme);

    Ok(())
}
