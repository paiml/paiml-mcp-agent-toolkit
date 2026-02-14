//! Example demonstrating PMAT Perfection Score and Spec commands
//!
//! This example shows how to:
//! - Calculate the 200-point perfection score
//! - Validate specifications with Popperian scoring
//! - Create and manage specification files
//!
//! Based on: master-plan-pmat-work-system.md specification
//! Run with: cargo run --example perfection_score_demo

use anyhow::Result;
use pmat::services::perfection_score::PerfectionScoreCalculator;
use pmat::services::spec_parser::SpecParser;
use std::path::Path;
use tempfile::TempDir;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🏆 PMAT Perfection Score Demo\n");
    println!("Based on: master-plan-pmat-work-system.md");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // === Example 1: Calculate Perfection Score ===
    println!("=== Example 1: Perfection Score (200-point scale) ===\n");

    let calculator = PerfectionScoreCalculator::new().fast_mode(true);
    let result = calculator.calculate(Path::new(".")).await?;

    println!(
        "Total Score: {:.1}/{} pts",
        result.total_score, result.max_score
    );
    println!("Grade: {}\n", result.grade);

    println!("Category Breakdown:");
    println!("────────────────────────────────────────────────────");
    for cat in &result.categories {
        let bar = progress_bar(cat.earned_points / f64::from(cat.max_points));
        println!(
            "  {:30} {} {:.1}/{} ({:.0}%)",
            cat.name,
            bar,
            cat.earned_points,
            cat.max_points,
            (cat.earned_points / f64::from(cat.max_points)) * 100.0
        );
    }

    println!("\nRecommendations:");
    for rec in &result.recommendations {
        println!("  {}", rec);
    }

    // === Example 2: Grade Thresholds ===
    println!("\n=== Example 2: Grade Thresholds (Maslow Hierarchy) ===\n");

    let thresholds = [
        (190, "S+", "Perfection - Publishing ready"),
        (180, "S", "Excellent - Production ready"),
        (170, "A+", "Very Good - Release candidate"),
        (160, "A", "Good - Feature complete"),
        (150, "B+", "Above Average - Beta quality"),
        (140, "B", "Average - Alpha quality"),
        (120, "C", "Below Average - Early development"),
        (100, "D", "Poor - Needs work"),
        (0, "F", "Failing - Critical issues"),
    ];

    println!("  {:>5}  {:>3}  DESCRIPTION", "SCORE", "GRADE");
    println!("  {:->5}  {:->3}  {:->30}", "", "", "");
    for (score, grade, desc) in thresholds {
        let marker =
            if result.total_score as u16 >= score && result.total_score < ((score + 10) as f64) {
                " ◀ You are here"
            } else {
                ""
            };
        println!("  {:>5}+ {:>3}   {}{}", score, grade, desc, marker);
    }

    // === Example 3: Spec Parser ===
    println!("\n=== Example 3: Specification Parser (Popperian Validation) ===\n");

    let temp_dir = TempDir::new()?;
    let spec_path = temp_dir.path().join("test-spec.md");

    // Create a sample spec
    let spec_content = r##"---
title: "Example Feature Specification"
version: "1.0.0"
status: "Draft"
created: "2025-12-13"
issue_refs: ["#123", "#124"]
epic: "PMAT-001"
---

# Example Feature Specification

## Executive Summary

This specification MUST be implemented with 100% test coverage.
The feature SHALL complete within 50ms response time.

## Requirements

### Functional Requirements

- [ ] FR-001: The system MUST support JSON input
- [ ] FR-002: The system SHOULD support YAML input
- [ ] FR-003: The system SHALL NOT accept invalid UTF-8

### Non-Functional Requirements

- [ ] NFR-001: Response time MUST be under 50ms (p99)
- [ ] NFR-002: Memory usage SHALL NOT exceed 100MB

## Acceptance Criteria

- [ ] AC-001: Parser handles valid JSON correctly
- [ ] AC-002: Parser handles valid YAML correctly
- [ ] AC-003: Invalid input returns descriptive errors
- [ ] AC-004: Performance meets p99 < 50ms target
- [ ] AC-005: Memory stays under 100MB limit

## Code Examples

### Example 1: Basic Usage

```rust
let parser = Parser::new();
let result = parser.parse(input)?;
```

### Example 2: Error Handling

```rust
match parser.parse(input) {
    Ok(data) => process(data),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Testing Strategy

- Unit tests: 100% coverage target
- Integration tests: All I/O paths
- Property tests: Fuzzing for edge cases
"##;

    std::fs::write(&spec_path, spec_content)?;
    println!("Created sample spec at: {:?}\n", spec_path);

    let parser = SpecParser::new();
    let spec = parser.parse_file(&spec_path)?;

    println!("Parsed Specification:");
    println!("  Title: {}", spec.title);
    println!("  Issue refs: {:?}", spec.issue_refs);
    println!("  Claims: {}", spec.claims.len());
    println!("  Acceptance criteria: {}", spec.acceptance_criteria.len());
    println!("  Code examples: {}", spec.code_examples.len());
    println!("  Test requirements: {}", spec.test_requirements.len());

    // Calculate a simple score
    let mut score = 0.0;
    if !spec.issue_refs.is_empty() {
        score += 10.0;
    }
    score += (spec.code_examples.len().min(5) * 4) as f64;
    score += (spec.acceptance_criteria.len().min(10) * 3) as f64;
    score += (spec.claims.len().min(20)) as f64;
    if !spec.title.is_empty() {
        score += 5.0;
    }
    score += (spec.test_requirements.len().min(5) * 3) as f64;

    println!("\n  Calculated Score: {:.1}/100", score.min(100.0));
    println!(
        "  Status: {}",
        if score >= 95.0 {
            "✅ PASS"
        } else {
            "❌ FAIL (needs ≥95)"
        }
    );

    // === Example 4: Category Weight Distribution ===
    println!("\n=== Example 4: 200-Point Weight Distribution ===\n");

    let categories = [
        (
            "TDG (Technical Debt Grade)",
            40,
            "Code quality and debt metrics",
        ),
        ("Repo Score", 30, "Repository health and hygiene"),
        ("Rust Project Score", 30, "Rust-specific quality"),
        ("Popper Score", 25, "Popperian falsifiability"),
        ("Test Coverage", 25, "Line and branch coverage"),
        ("Mutation Testing", 20, "Test effectiveness"),
        ("Documentation", 15, "API docs and README"),
        ("Performance", 15, "Benchmarks and profiling"),
    ];

    println!(
        "  {:35} {:>6} {:>7}  DESCRIPTION",
        "CATEGORY", "MAX", "%"
    );
    println!("  {:->35} {:->6} {:->7}  {:->25}", "", "", "", "");
    for (name, max, desc) in categories {
        let pct = (max as f64 / 200.0) * 100.0;
        println!("  {:35} {:>6} {:>6.1}%  {}", name, max, pct, desc);
    }
    println!("  {:->35} {:->6}", "", "");
    println!("  {:35} {:>6}", "TOTAL", 200);

    // === Summary ===
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Demo completed successfully!\n");
    println!("CLI Commands:");
    println!("  pmat perfection-score --fast --breakdown");
    println!("  pmat spec score <path>");
    println!("  pmat spec create \"Feature Name\" --issue \"#123\"");
    println!("  pmat spec list docs/specifications/");

    Ok(())
}

/// Create a simple ASCII progress bar
fn progress_bar(ratio: f64) -> String {
    let width: usize = 15;
    let filled = (ratio * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
}
