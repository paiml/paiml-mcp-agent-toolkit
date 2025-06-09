//! Helper functions for mermaid readme test to reduce complexity

use std::fs;
use std::path::Path;
use std::fmt::Write;

use super::{ArtifactCategory, MermaidArtifactSpec};

/// Generate all mermaid artifacts to disk
pub fn generate_artifacts(base_path: &Path, artifact_specs: &[MermaidArtifactSpec]) {
    for spec in artifact_specs.iter() {
        let category_path = base_path.join(spec.category.path());
        fs::create_dir_all(&category_path).unwrap();
        let file_path = category_path.join(spec.name);
        let content = (spec.generator)();
        fs::write(&file_path, &content).unwrap();
    }
}

/// Generate the header section of the README
pub fn generate_header(content: &mut String) {
    writeln!(content, "# Mermaid Diagram Artifacts\n").unwrap();
    writeln!(content, "This directory contains test-maintained Mermaid diagram examples demonstrating the capabilities of the PAIML MCP Agent Toolkit.\n").unwrap();
}

/// Generate the directory structure section
pub fn generate_directory_structure(content: &mut String) {
    writeln!(content, "## Directory Structure\n").unwrap();
    writeln!(content, "```").unwrap();
    writeln!(content, "mermaid/").unwrap();
    writeln!(content, "├── non-code/          # Hand-crafted architectural diagrams").unwrap();
    writeln!(content, "│   ├── simple/       # Without styling").unwrap();
    writeln!(content, "│   └── styled/       # With complexity indicators").unwrap();
    writeln!(content, "└── ast-generated/     # Generated from codebase analysis").unwrap();
    writeln!(content, "    ├── simple/       # Basic structure").unwrap();
    writeln!(content, "    └── styled/       # With metrics").unwrap();
    writeln!(content, "```\n").unwrap();
}

/// Process a single category and add its content to the README
pub fn process_category(
    content: &mut String,
    category: &ArtifactCategory,
    base_path: &Path,
    artifact_specs: &[MermaidArtifactSpec],
) {
    writeln!(content, "## {}\n", format_category_title(category)).unwrap();

    let category_path = base_path.join(category.path());
    if let Ok(entries) = fs::read_dir(&category_path) {
        for entry in entries.filter_map(Result::ok) {
            if entry.path().extension() == Some(std::ffi::OsStr::new("mmd")) {
                process_mermaid_file(content, &entry, artifact_specs);
            }
        }
    }
}

/// Process a single mermaid file
fn process_mermaid_file(
    content: &mut String,
    entry: &fs::DirEntry,
    artifact_specs: &[MermaidArtifactSpec],
) {
    let file_name = entry.file_name();
    let file_content = fs::read_to_string(entry.path()).unwrap();

    // Find matching spec
    if let Some(spec) = artifact_specs
        .iter()
        .find(|s| s.name == file_name.to_str().unwrap())
    {
        writeln!(content, "### {}\n", spec.name).unwrap();
        writeln!(content, "{}\n", spec.description).unwrap();
        writeln!(content, "```mermaid").unwrap();
        writeln!(content, "{}", file_content.trim()).unwrap();
        writeln!(content, "```\n").unwrap();

        // Add metrics
        add_diagram_metrics(content, &file_content);
    }
}

/// Add diagram metrics to the content
fn add_diagram_metrics(content: &mut String, file_content: &str) {
    let metrics = analyze_diagram_metrics(file_content);
    writeln!(content, "**Metrics:**").unwrap();
    writeln!(content, "- Nodes: {}", metrics.node_count).unwrap();
    writeln!(content, "- Edges: {}", metrics.edge_count).unwrap();
    writeln!(content, "- Max depth: {}", metrics.max_depth).unwrap();
    if metrics.has_styling {
        writeln!(content, "- Styling: ✓ Complexity indicators").unwrap();
    }
    writeln!(content).unwrap();
}

/// Generate validation status section
pub fn generate_validation_status(content: &mut String) {
    writeln!(content, "## Validation Status\n").unwrap();
    writeln!(content, "All diagrams are automatically validated for:").unwrap();
    writeln!(content, "- ✓ Correct Mermaid syntax").unwrap();
    writeln!(content, "- ✓ Node count ≤ 15").unwrap();
    writeln!(content, "- ✓ Proper labeling (no empty nodes)").unwrap();
    writeln!(content, "- ✓ Category-appropriate styling").unwrap();
    writeln!(
        content,
        "\nLast validated: {}",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    ).unwrap();
}

pub fn format_category_title(category: &ArtifactCategory) -> &'static str {
    match category {
        ArtifactCategory::NonCodeSimple => "Non-Code Simple Diagrams",
        ArtifactCategory::NonCodeStyled => "Non-Code Styled Diagrams",
        ArtifactCategory::AstSimple => "AST-Generated Simple Diagrams",
        ArtifactCategory::AstStyled => "AST-Generated Styled Diagrams",
    }
}

#[derive(Default)]
pub struct DiagramMetrics {
    pub node_count: usize,
    pub edge_count: usize,
    pub max_depth: usize,
    pub has_styling: bool,
}

pub fn analyze_diagram_metrics(content: &str) -> DiagramMetrics {
    DiagramMetrics {
        node_count: count_nodes(content),
        edge_count: count_edges(content),
        max_depth: calculate_graph_depth(content),
        has_styling: check_has_styling(content),
    }
}

fn count_nodes(content: &str) -> usize {
    content
        .lines()
        .filter(|l| {
            let line = l.trim();
            (line.contains('[') && line.contains(']'))
                || (line.contains('(') && line.contains(')'))
        })
        .count()
}

fn count_edges(content: &str) -> usize {
    content
        .lines()
        .filter(|l| l.contains("-->") || l.contains("-.->") || l.contains("---"))
        .count()
}

fn check_has_styling(content: &str) -> bool {
    content.contains("style") || content.contains("fill:")
}

pub fn calculate_graph_depth(content: &str) -> usize {
    // Simple heuristic - count indentation levels
    content
        .lines()
        .map(|line| line.chars().take_while(|&c| c == ' ').count() / 4)
        .max()
        .unwrap_or(0)
}