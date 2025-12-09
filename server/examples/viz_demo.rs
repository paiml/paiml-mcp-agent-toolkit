//! Terminal Graph Visualization Demo
//!
//! Demonstrates the trueno-viz integration for visualizing TDG dependency graphs
//! in the terminal with ANSI TrueColor rendering and PageRank criticality scoring.
//!
//! # Features
//!
//! - Force-directed graph layout (Fruchterman-Reingold algorithm)
//! - PageRank-based criticality scoring
//! - ANSI TrueColor rendering for 16.7M color support
//! - Accessibility-focused dual encoding (shape + color)
//! - Multiple themes (default, high-contrast, light, colorblind-safe)
//!
//! # Usage
//!
//! ```bash
//! cargo run --example viz_demo --features viz
//! cargo run --example viz_demo --features viz -- --theme colorblind-safe
//! ```

use anyhow::Result;

#[cfg(feature = "viz")]
fn main() -> Result<()> {
    use pmat::tdg::tdg_graph::TdgGraph;
    use pmat::viz::terminal::{RenderConfig, TerminalTheme, Visualizable};
    use trueno_viz::output::TerminalMode;

    println!("TDG Terminal Graph Visualization Demo");
    println!("======================================\n");

    // Parse CLI args for theme selection
    let args: Vec<String> = std::env::args().collect();
    let theme_name = args
        .iter()
        .position(|a| a == "--theme")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("default");

    // Demo 1: Simple dependency graph
    demo_simple_graph(theme_name)?;

    // Demo 2: Hub-and-spoke pattern (identifies critical functions)
    demo_hub_spoke_graph(theme_name)?;

    // Demo 3: Larger realistic graph
    demo_realistic_graph(theme_name)?;

    println!("\n✓ All visualization demos completed!");
    println!("\nAvailable themes:");
    println!("  --theme default         : Standard colors (green/yellow/red gradient)");
    println!("  --theme high-contrast   : High contrast for visibility");
    println!("  --theme light           : Light terminal backgrounds");
    println!("  --theme colorblind-safe : Okabe-Ito palette (WCAG 2.1 compliant)");
    println!("\nCLI usage:");
    println!("  pmat tdg --viz                     : Default visualization");
    println!("  pmat tdg --viz --viz-theme light   : Light theme");

    Ok(())
}

#[cfg(feature = "viz")]
fn demo_simple_graph(theme_name: &str) -> Result<()> {
    use pmat::tdg::tdg_graph::TdgGraph;
    use pmat::viz::terminal::{RenderConfig, TerminalTheme, Visualizable};
    use trueno_viz::output::TerminalMode;

    println!("1. Simple Dependency Graph");
    println!("--------------------------");

    let mut graph = TdgGraph::new();

    // main → helper → utils
    graph.add_function("main".to_string())?;
    graph.add_function("helper".to_string())?;
    graph.add_function("utils".to_string())?;

    graph.add_edge("main", "helper")?;
    graph.add_edge("helper", "utils")?;
    graph.add_edge("main", "utils")?;

    // Compute PageRank criticality
    graph.update_criticality()?;

    // Render with selected theme
    let theme = parse_theme(theme_name);
    let config = RenderConfig {
        width: 60,
        height: 20,
        theme,
        mode: TerminalMode::UnicodeHalfBlock,
        iterations: 100,
        critical_threshold: 0.1,
        max_nodes: 50,
        show_labels: true,
    };

    let output = graph.render_terminal(&config)?;
    println!("{}", output);
    println!("Nodes: {}, Edges: {}\n", graph.num_nodes(), graph.num_edges());

    Ok(())
}

#[cfg(feature = "viz")]
fn demo_hub_spoke_graph(theme_name: &str) -> Result<()> {
    use pmat::tdg::tdg_graph::TdgGraph;
    use pmat::viz::terminal::{RenderConfig, Visualizable};
    use trueno_viz::output::TerminalMode;

    println!("2. Hub-and-Spoke Pattern (Critical Function Detection)");
    println!("------------------------------------------------------");

    let mut graph = TdgGraph::new();

    // Central hub function called by many others
    graph.add_function("validate_input".to_string())?;

    // Spoke functions that all call the hub
    let spokes = ["parse_json", "parse_xml", "parse_yaml", "parse_toml", "parse_csv"];
    for spoke in spokes {
        graph.add_function(spoke.to_string())?;
        graph.add_edge(spoke, "validate_input")?;
    }

    // Compute PageRank - validate_input should have highest score
    graph.update_criticality()?;

    let theme = parse_theme(theme_name);
    let config = RenderConfig {
        width: 60,
        height: 25,
        theme,
        mode: TerminalMode::UnicodeHalfBlock,
        iterations: 100,
        critical_threshold: 0.1,
        max_nodes: 50,
        show_labels: true,
    };

    let output = graph.render_terminal(&config)?;
    println!("{}", output);

    // Show criticality ranking
    println!("Critical Functions (by PageRank):");
    for (i, (name, score)) in graph.critical_functions().iter().take(5).enumerate() {
        println!("  {}. {} (score: {:.4})", i + 1, name, score);
    }
    println!();

    Ok(())
}

#[cfg(feature = "viz")]
fn demo_realistic_graph(theme_name: &str) -> Result<()> {
    use pmat::tdg::tdg_graph::TdgGraph;
    use pmat::viz::terminal::{RenderConfig, Visualizable};
    use trueno_viz::output::TerminalMode;

    println!("3. Realistic Dependency Graph");
    println!("-----------------------------");

    let mut graph = TdgGraph::new();

    // Layered architecture: API → Service → Repository → Database
    let functions = [
        // API layer
        "handle_request",
        "parse_params",
        "validate_token",
        // Service layer
        "user_service",
        "order_service",
        "payment_service",
        // Repository layer
        "user_repo",
        "order_repo",
        "payment_repo",
        // Database layer
        "db_connect",
        "db_query",
        "db_transaction",
    ];

    for func in functions {
        graph.add_function(func.to_string())?;
    }

    // API → Service edges
    graph.add_edge("handle_request", "parse_params")?;
    graph.add_edge("handle_request", "validate_token")?;
    graph.add_edge("handle_request", "user_service")?;
    graph.add_edge("handle_request", "order_service")?;
    graph.add_edge("handle_request", "payment_service")?;

    // Service → Repository edges
    graph.add_edge("user_service", "user_repo")?;
    graph.add_edge("order_service", "order_repo")?;
    graph.add_edge("payment_service", "payment_repo")?;
    graph.add_edge("order_service", "user_repo")?; // Cross-reference

    // Repository → Database edges (all repos use db)
    for repo in ["user_repo", "order_repo", "payment_repo"] {
        graph.add_edge(repo, "db_connect")?;
        graph.add_edge(repo, "db_query")?;
    }
    graph.add_edge("payment_repo", "db_transaction")?;

    // Compute criticality
    graph.update_criticality()?;

    let theme = parse_theme(theme_name);
    let config = RenderConfig {
        width: 80,
        height: 30,
        theme,
        mode: TerminalMode::UnicodeHalfBlock,
        iterations: 100,
        critical_threshold: 0.1,
        max_nodes: 50,
        show_labels: true,
    };

    let output = graph.render_terminal(&config)?;
    println!("{}", output);

    println!("Architecture Analysis:");
    println!("  Nodes: {} functions", graph.num_nodes());
    println!("  Edges: {} dependencies", graph.num_edges());
    println!("\nTop Critical Functions (highest impact if changed):");
    for (i, (name, score)) in graph.critical_functions().iter().take(5).enumerate() {
        println!("  {}. {} (PageRank: {:.4})", i + 1, name, score);
    }
    println!();

    Ok(())
}

#[cfg(feature = "viz")]
fn parse_theme(name: &str) -> pmat::viz::terminal::TerminalTheme {
    use pmat::viz::terminal::TerminalTheme;

    match name {
        "high-contrast" => TerminalTheme::HighContrast,
        "light" => TerminalTheme::Light,
        "colorblind-safe" => TerminalTheme::ColorblindSafe,
        _ => TerminalTheme::Default,
    }
}

#[cfg(not(feature = "viz"))]
fn main() {
    eprintln!("This example requires the 'viz' feature.");
    eprintln!("Run with: cargo run --example viz_demo --features viz");
    std::process::exit(1);
}
