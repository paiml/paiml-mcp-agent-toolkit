//! Example demonstrating advanced code similarity detection
//!
//! Run with: cargo run --example similarity_demo

use pmat::services::similarity::{SimilarityConfig, SimilarityDetector};
use std::path::PathBuf;

fn main() {
    println!("🔍 Code Similarity Detection Demo\n");

    // Create test files with duplicates
    let files = vec![
        (
            PathBuf::from("file1.rs"),
            r#"
fn calculate_sum(a: i32, b: i32) -> i32 {
    let result = a + b;
    println!("Sum: {}", result);
    result
}

fn process_data(input: &str) -> String {
    let parsed = input.trim();
    let uppercase = parsed.to_uppercase();
    uppercase
}
"#
            .to_string(),
        ),
        (
            PathBuf::from("file2.rs"),
            r#"
// Exact duplicate
fn calculate_sum(a: i32, b: i32) -> i32 {
    let result = a + b;
    println!("Sum: {}", result);
    result
}

// Structurally similar (renamed variables)
fn handle_info(data: &str) -> String {
    let cleaned = data.trim();
    let upper = cleaned.to_uppercase();
    upper
}
"#
            .to_string(),
        ),
        (
            PathBuf::from("file3.rs"),
            r#"
// Semantically similar (different implementation)
fn add_numbers(x: i32, y: i32) -> i32 {
    x + y
}

// Another pattern
fn process_string(s: &str) -> String {
    s.trim().to_uppercase()
}
"#
            .to_string(),
        ),
    ];

    // Configure detector
    let config = SimilarityConfig {
        min_lines: 3,
        min_tokens: 10,
        similarity_threshold: 0.7,
        enable_entropy: true,
        enable_ast: true,
        enable_semantic: true,
        window_size: 40,
        k_gram_size: 15,
    };

    let detector = SimilarityDetector::new(config);

    // Run comprehensive analysis
    println!("Analyzing {} files...\n", files.len());
    let report = detector.comprehensive_analysis(&files);

    // Display results
    println!("📊 Analysis Results:");
    println!("═══════════════════");
    println!("Duplication: {:.1}%", report.metrics.duplication_percentage);
    println!("Average Entropy: {:.2}", report.metrics.average_entropy);
    println!("Total Clones: {}", report.metrics.total_clones);
    println!();

    // Show exact duplicates
    if !report.exact_duplicates.is_empty() {
        println!("🔴 Exact Duplicates Found:");
        for (i, block) in report.exact_duplicates.iter().enumerate() {
            println!(
                "  {}. {} locations, {} lines",
                i + 1,
                block.locations.len(),
                block.lines
            );
            for loc in &block.locations {
                println!(
                    "     - {}:{}-{}",
                    loc.file.display(),
                    loc.start_line,
                    loc.end_line
                );
            }
        }
        println!();
    }

    // Show structural similarities
    if !report.structural_similarities.is_empty() {
        println!("🟡 Structural Similarities:");
        for (i, block) in report.structural_similarities.iter().enumerate() {
            println!(
                "  {}. Similarity: {:.1}%, Type: {:?}",
                i + 1,
                block.similarity * 100.0,
                block.clone_type
            );
            for loc in &block.locations {
                println!(
                    "     - {}:{}-{}",
                    loc.file.display(),
                    loc.start_line,
                    loc.end_line
                );
            }
        }
        println!();
    }

    // Show semantic similarities
    if !report.semantic_similarities.is_empty() {
        println!("🟢 Semantic Similarities:");
        for (i, block) in report.semantic_similarities.iter().enumerate() {
            println!(
                "  {}. Similarity: {:.1}%, Type: {:?}",
                i + 1,
                block.similarity * 100.0,
                block.clone_type
            );
            for loc in &block.locations {
                println!(
                    "     - {}:{}-{}",
                    loc.file.display(),
                    loc.start_line,
                    loc.end_line
                );
            }
        }
        println!();
    }

    // Show entropy analysis
    if let Some(entropy) = &report.entropy_analysis {
        println!("📈 Entropy Analysis:");
        println!("  Average: {:.2}", entropy.average_entropy);
        if !entropy.high_entropy_blocks.is_empty() {
            println!(
                "  High complexity blocks: {}",
                entropy.high_entropy_blocks.len()
            );
        }
        if !entropy.low_entropy_patterns.is_empty() {
            println!(
                "  Repetitive patterns: {}",
                entropy.low_entropy_patterns.len()
            );
        }
        println!();
    }

    // Show refactoring opportunities
    if !report.refactoring_opportunities.is_empty() {
        println!("💡 Refactoring Opportunities:");
        for (i, hint) in report.refactoring_opportunities.iter().enumerate() {
            println!(
                "  {}. {} (Priority: {:?})",
                i + 1,
                hint.pattern,
                hint.priority
            );
            println!("     Suggestion: {}", hint.suggestion);
        }
    }

    println!("\n✅ Demo complete!");
}
