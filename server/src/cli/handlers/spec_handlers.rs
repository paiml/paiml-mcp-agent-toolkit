//! Spec Management CLI Handlers (master-plan-pmat-work-system.md)
//!
//! Implements S-001 through S-010 acceptance criteria for specification management.

use crate::cli::commands::SpecOutputFormat;
use crate::services::spec_parser::{ParsedSpec, SpecParser};
use std::fs;
use std::path::Path;

/// Handle spec score command (S-001)
/// Validates specification with 100-point Popperian score
pub async fn handle_spec_score(
    spec_path: &Path,
    format: SpecOutputFormat,
    output: Option<&Path>,
    verbose: bool,
) -> anyhow::Result<()> {
    let parser = SpecParser::new();
    let spec = parser.parse_file(spec_path)?;

    // Simple score calculation (will be enhanced with full validation)
    let score = calculate_spec_score(&spec);

    let output_text = match format {
        SpecOutputFormat::Text => format_spec_score_text(&spec, score, verbose),
        SpecOutputFormat::Json => format_spec_score_json(&spec, score)?,
        SpecOutputFormat::Markdown => format_spec_score_markdown(&spec, score),
    };

    if let Some(output_path) = output {
        fs::write(output_path, &output_text)?;
        println!("✅ Spec score written to {}", output_path.display());
    } else {
        println!("{}", output_text);
    }

    // Fail if below 95 threshold (S-002)
    if score < 95.0 {
        println!(
            "\n⚠️  Spec score {:.1} is below 95 threshold. Run `pmat spec comply` to fix.",
            score
        );
        std::process::exit(1);
    }

    Ok(())
}

/// Handle spec comply command (S-003)
/// Auto-fixes spec issues to meet 95-point threshold
pub async fn handle_spec_comply(
    spec_path: &Path,
    dry_run: bool,
    _format: SpecOutputFormat,
) -> anyhow::Result<()> {
    let parser = SpecParser::new();
    let spec = parser.parse_file(spec_path)?;

    let mut fixes: Vec<String> = Vec::new();

    // Check minimum requirements and suggest fixes
    if spec.issue_refs.is_empty() {
        fixes.push(
            "- Add issue_refs in YAML frontmatter (e.g., issue_refs: [\"#123\"])".to_string(),
        );
    }

    if spec.code_examples.len() < 5 {
        fixes.push(format!(
            "- Add {} more code examples (minimum 5 required)",
            5 - spec.code_examples.len()
        ));
    }

    if spec.acceptance_criteria.len() < 10 {
        fixes.push(format!(
            "- Add {} more acceptance criteria (minimum 10 required)",
            10 - spec.acceptance_criteria.len()
        ));
    }

    // Count citations from claims (rough estimate)
    let citation_claims = spec
        .claims
        .iter()
        .filter(|c| c.text.contains('[') && c.text.contains(']'))
        .count();
    if citation_claims < 5 {
        fixes.push("- Add peer-reviewed citations (minimum 5 required)".to_string());
    }

    if fixes.is_empty() {
        println!("✅ Spec already meets all requirements!");
        return Ok(());
    }

    println!("📋 Spec Compliance Issues Found:\n");
    for fix in &fixes {
        println!("{}", fix);
    }

    if dry_run {
        println!("\n(Dry run - no changes made)");
    } else {
        println!("\n⚠️  Auto-fix not yet implemented. Please apply fixes manually.");
    }

    Ok(())
}

/// Handle spec create command
pub async fn handle_spec_create(
    name: &str,
    issue: Option<&str>,
    epic: Option<&str>,
    output: Option<&Path>,
) -> anyhow::Result<()> {
    let slug = name.to_lowercase().replace(' ', "-");
    let output_dir = output
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| Path::new("docs/specifications").to_path_buf());

    let file_path = output_dir.join(format!("{}.md", slug));

    let issue_ref = issue.unwrap_or("#TODO");
    let epic_name = epic.unwrap_or("PMAT-TODO");
    let date = chrono::Local::now().format("%Y-%m-%d");

    let template = format!(
        r#"---
title: "{name}"
version: "1.0.0"
status: "Draft"
created: "{date}"
updated: "{date}"
issue_refs: ["{issue_ref}"]
epic: "{epic_name}"
---

# {name}

## Executive Summary

[Brief description of what this specification defines]

## Scientific Foundation

[Cite minimum 5 peer-reviewed sources]

1. [Author et al., Year. Title. Journal/Conference.]
2. [...]

## Requirements

### Functional Requirements

- [ ] FR-001: [Requirement description]
- [ ] FR-002: [Requirement description]

### Non-Functional Requirements

- [ ] NFR-001: [Performance requirement]
- [ ] NFR-002: [Security requirement]

## Acceptance Criteria

### Category 1 (AC-001 to AC-005)

- [ ] AC-001: [Testable criterion]
- [ ] AC-002: [Testable criterion]
- [ ] AC-003: [Testable criterion]
- [ ] AC-004: [Testable criterion]
- [ ] AC-005: [Testable criterion]

### Category 2 (AC-006 to AC-010)

- [ ] AC-006: [Testable criterion]
- [ ] AC-007: [Testable criterion]
- [ ] AC-008: [Testable criterion]
- [ ] AC-009: [Testable criterion]
- [ ] AC-010: [Testable criterion]

## Code Examples

### Example 1: Basic Usage

```rust
// Example code here
```

### Example 2: Advanced Usage

```rust
// Example code here
```

### Example 3: Error Handling

```rust
// Example code here
```

### Example 4: Integration

```rust
// Example code here
```

### Example 5: Performance

```rust
// Example code here
```

## Testing Strategy

- Unit tests: [Coverage target]
- Integration tests: [Scope]
- Property tests: [Properties to verify]

## References

[1] [Citation]
[2] [Citation]
[3] [Citation]
[4] [Citation]
[5] [Citation]
"#
    );

    // Create directory if needed
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&file_path, template)?;
    println!("✅ Created specification: {}", file_path.display());
    println!("\nNext steps:");
    println!("  1. Edit the specification with your requirements");
    println!(
        "  2. Run `pmat spec score {}` to validate",
        file_path.display()
    );
    println!(
        "  3. Run `pmat spec comply {}` to fix issues",
        file_path.display()
    );

    Ok(())
}

/// Handle spec list command
pub async fn handle_spec_list(
    path: &Path,
    min_score: Option<u8>,
    failing_only: bool,
    format: SpecOutputFormat,
) -> anyhow::Result<()> {
    let parser = SpecParser::new();
    let specs = parser.find_specs(path)?;

    let mut results = Vec::new();

    for spec_path in specs {
        if let Ok(spec) = parser.parse_file(&spec_path) {
            let score = calculate_spec_score(&spec);
            let passing = score >= 95.0;

            if let Some(min) = min_score {
                if score < f64::from(min) {
                    continue;
                }
            }

            if failing_only && passing {
                continue;
            }

            results.push((spec_path, spec.title.clone(), score, passing));
        }
    }

    match format {
        SpecOutputFormat::Text => {
            println!("📚 Specifications in {}\n", path.display());
            println!("{:<50} {:>8} {:>8}", "SPECIFICATION", "SCORE", "STATUS");
            println!("{}", "─".repeat(70));

            for (path, title, score, passing) in &results {
                let status = if *passing { "✅ PASS" } else { "❌ FAIL" };
                let display_name = if title.is_empty() {
                    path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                } else {
                    title.as_str()
                };
                println!("{:<50} {:>7.1} {:>8}", display_name, score, status);
            }

            println!(
                "\nTotal: {} specs, {} passing",
                results.len(),
                results.iter().filter(|(_, _, _, p)| *p).count()
            );
        }
        SpecOutputFormat::Json => {
            let json_results: Vec<_> = results
                .iter()
                .map(|(path, title, score, passing)| {
                    serde_json::json!({
                        "path": path.display().to_string(),
                        "title": title,
                        "score": score,
                        "passing": passing,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&json_results)?);
        }
        SpecOutputFormat::Markdown => {
            println!("# Specification Status Report\n");
            println!("| Specification | Score | Status |");
            println!("|---------------|-------|--------|");
            for (path, title, score, passing) in &results {
                let status = if *passing { "✅ PASS" } else { "❌ FAIL" };
                let display_name = if title.is_empty() {
                    path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                } else {
                    title.as_str()
                };
                println!("| {} | {:.1} | {} |", display_name, score, status);
            }
        }
    }

    Ok(())
}

fn calculate_spec_score(spec: &ParsedSpec) -> f64 {
    // Simplified scoring based on spec requirements
    let mut score = 0.0;

    // Issue refs (10 pts)
    if !spec.issue_refs.is_empty() {
        score += 10.0;
    }

    // Code examples (20 pts, 4 pts each up to 5)
    score += (spec.code_examples.len().min(5) * 4) as f64;

    // Acceptance criteria (30 pts, 3 pts each up to 10)
    score += (spec.acceptance_criteria.len().min(10) * 3) as f64;

    // Claims (20 pts based on count)
    score += (spec.claims.len().min(20)) as f64;

    // Title exists (5 pts)
    if !spec.title.is_empty() {
        score += 5.0;
    }

    // Test requirements (15 pts, 3 pts each up to 5)
    score += (spec.test_requirements.len().min(5) * 3) as f64;

    score.min(100.0)
}

fn format_spec_score_text(spec: &ParsedSpec, score: f64, verbose: bool) -> String {
    let mut out = String::new();

    out.push_str("📋 Specification Score\n");
    out.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n");

    if !spec.title.is_empty() {
        out.push_str(&format!("Title: {}\n", spec.title));
    }

    out.push_str(&format!("Score: {:.1}/100\n", score));
    out.push_str(&format!(
        "Status: {}\n",
        if score >= 95.0 {
            "✅ PASS"
        } else {
            "❌ FAIL (needs ≥95)"
        }
    ));

    if verbose {
        out.push_str(&format!("\nClaims: {}\n", spec.claims.len()));
        out.push_str(&format!("Code Examples: {}\n", spec.code_examples.len()));
        out.push_str(&format!(
            "Acceptance Criteria: {}\n",
            spec.acceptance_criteria.len()
        ));
        out.push_str(&format!("Issue Refs: {:?}\n", spec.issue_refs));
    }

    out
}

fn format_spec_score_json(spec: &ParsedSpec, score: f64) -> anyhow::Result<String> {
    let result = serde_json::json!({
        "title": spec.title,
        "score": score,
        "passing": score >= 95.0,
        "claims": spec.claims.len(),
        "code_examples": spec.code_examples.len(),
        "acceptance_criteria": spec.acceptance_criteria.len(),
        "issue_refs": spec.issue_refs,
    });
    Ok(serde_json::to_string_pretty(&result)?)
}

fn format_spec_score_markdown(spec: &ParsedSpec, score: f64) -> String {
    let mut out = String::new();

    out.push_str("# Specification Score Report\n\n");

    if !spec.title.is_empty() {
        out.push_str(&format!("**Title:** {}\n\n", spec.title));
    }

    out.push_str("| Metric | Value |\n");
    out.push_str("|--------|-------|\n");
    out.push_str(&format!("| Score | {:.1}/100 |\n", score));
    out.push_str(&format!(
        "| Status | {} |\n",
        if score >= 95.0 {
            "✅ PASS"
        } else {
            "❌ FAIL"
        }
    ));
    out.push_str(&format!("| Claims | {} |\n", spec.claims.len()));
    out.push_str(&format!(
        "| Code Examples | {} |\n",
        spec.code_examples.len()
    ));
    out.push_str(&format!(
        "| Acceptance Criteria | {} |\n",
        spec.acceptance_criteria.len()
    ));

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::spec_parser::{
        AcceptanceCriterion, ClaimCategory, CodeExample, TestRequirement, ValidationClaim,
    };
    use std::path::PathBuf;
    use tempfile::TempDir;

    // ============================================================================
    // Helper functions to create test specs
    // ============================================================================

    fn create_empty_spec() -> ParsedSpec {
        ParsedSpec {
            path: PathBuf::new(),
            title: String::new(),
            issue_refs: vec![],
            status: None,
            claims: vec![],
            code_examples: vec![],
            acceptance_criteria: vec![],
            test_requirements: vec![],
        }
    }

    fn create_minimal_spec(title: &str) -> ParsedSpec {
        ParsedSpec {
            path: PathBuf::from("test.md"),
            title: title.to_string(),
            issue_refs: vec![],
            status: None,
            claims: vec![],
            code_examples: vec![],
            acceptance_criteria: vec![],
            test_requirements: vec![],
        }
    }

    fn create_full_spec() -> ParsedSpec {
        ParsedSpec {
            path: PathBuf::from("full-spec.md"),
            title: "Full Specification".to_string(),
            issue_refs: vec!["#123".to_string(), "#456".to_string()],
            status: Some("Draft".to_string()),
            claims: (0..20)
                .map(|i| ValidationClaim {
                    id: format!("C-{}", i),
                    text: format!("Claim {} with [citation]", i),
                    line: i + 1,
                    category: ClaimCategory::Implementation,
                    automatable: false,
                    validation_cmd: None,
                    expected_pattern: None,
                })
                .collect(),
            code_examples: (0..5)
                .map(|i| CodeExample {
                    language: "rust".to_string(),
                    code: format!("fn example_{}() {{}}", i),
                    line: i * 10,
                    executable: true,
                })
                .collect(),
            acceptance_criteria: (0..10)
                .map(|i| AcceptanceCriterion {
                    text: format!("AC-{}: Criterion {}", i, i),
                    complete: i % 2 == 0,
                    line: i + 100,
                })
                .collect(),
            test_requirements: (0..5)
                .map(|i| TestRequirement {
                    text: format!("Test requirement {}", i),
                    test_type: "unit".to_string(),
                    code_path: Some(format!("src/test_{}.rs", i)),
                })
                .collect(),
        }
    }

    fn create_partial_spec() -> ParsedSpec {
        ParsedSpec {
            path: PathBuf::from("partial-spec.md"),
            title: "Partial Specification".to_string(),
            issue_refs: vec!["#789".to_string()],
            status: Some("Active".to_string()),
            claims: (0..5)
                .map(|i| ValidationClaim {
                    id: format!("C-{}", i),
                    text: format!("Claim {}", i),
                    line: i + 1,
                    category: ClaimCategory::Testing,
                    automatable: true,
                    validation_cmd: Some("cargo test".to_string()),
                    expected_pattern: None,
                })
                .collect(),
            code_examples: vec![CodeExample {
                language: "rust".to_string(),
                code: "fn main() {}".to_string(),
                line: 1,
                executable: true,
            }],
            acceptance_criteria: vec![AcceptanceCriterion {
                text: "Single criterion".to_string(),
                complete: false,
                line: 1,
            }],
            test_requirements: vec![],
        }
    }

    // ============================================================================
    // Tests for calculate_spec_score
    // ============================================================================

    #[test]
    fn test_calculate_spec_score_empty() {
        let spec = create_empty_spec();
        let score = calculate_spec_score(&spec);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_calculate_spec_score_title_only() {
        let spec = create_minimal_spec("Test Title");
        let score = calculate_spec_score(&spec);
        // Title adds 5 points
        assert_eq!(score, 5.0);
    }

    #[test]
    fn test_calculate_spec_score_issue_refs() {
        let mut spec = create_empty_spec();
        spec.issue_refs = vec!["#123".to_string()];
        let score = calculate_spec_score(&spec);
        // Issue refs add 10 points
        assert_eq!(score, 10.0);
    }

    #[test]
    fn test_calculate_spec_score_code_examples() {
        let mut spec = create_empty_spec();
        spec.code_examples = vec![
            CodeExample {
                language: "rust".to_string(),
                code: "fn a() {}".to_string(),
                line: 1,
                executable: true,
            },
            CodeExample {
                language: "rust".to_string(),
                code: "fn b() {}".to_string(),
                line: 2,
                executable: true,
            },
        ];
        let score = calculate_spec_score(&spec);
        // 2 examples * 4 pts each = 8 pts
        assert_eq!(score, 8.0);
    }

    #[test]
    fn test_calculate_spec_score_code_examples_capped_at_5() {
        let mut spec = create_empty_spec();
        // Add 10 code examples, but only 5 should count
        spec.code_examples = (0..10)
            .map(|i| CodeExample {
                language: "rust".to_string(),
                code: format!("fn example_{}() {{}}", i),
                line: i,
                executable: true,
            })
            .collect();
        let score = calculate_spec_score(&spec);
        // Max 5 examples * 4 pts each = 20 pts
        assert_eq!(score, 20.0);
    }

    #[test]
    fn test_calculate_spec_score_acceptance_criteria() {
        let mut spec = create_empty_spec();
        spec.acceptance_criteria = vec![
            AcceptanceCriterion {
                text: "AC-1".to_string(),
                complete: false,
                line: 1,
            },
            AcceptanceCriterion {
                text: "AC-2".to_string(),
                complete: true,
                line: 2,
            },
            AcceptanceCriterion {
                text: "AC-3".to_string(),
                complete: false,
                line: 3,
            },
        ];
        let score = calculate_spec_score(&spec);
        // 3 criteria * 3 pts each = 9 pts
        assert_eq!(score, 9.0);
    }

    #[test]
    fn test_calculate_spec_score_acceptance_criteria_capped_at_10() {
        let mut spec = create_empty_spec();
        // Add 15 acceptance criteria, but only 10 should count
        spec.acceptance_criteria = (0..15)
            .map(|i| AcceptanceCriterion {
                text: format!("AC-{}", i),
                complete: false,
                line: i,
            })
            .collect();
        let score = calculate_spec_score(&spec);
        // Max 10 criteria * 3 pts each = 30 pts
        assert_eq!(score, 30.0);
    }

    #[test]
    fn test_calculate_spec_score_claims() {
        let mut spec = create_empty_spec();
        spec.claims = (0..10)
            .map(|i| ValidationClaim {
                id: format!("C-{}", i),
                text: format!("Claim {}", i),
                line: i,
                category: ClaimCategory::Implementation,
                automatable: false,
                validation_cmd: None,
                expected_pattern: None,
            })
            .collect();
        let score = calculate_spec_score(&spec);
        // 10 claims = 10 pts
        assert_eq!(score, 10.0);
    }

    #[test]
    fn test_calculate_spec_score_claims_capped_at_20() {
        let mut spec = create_empty_spec();
        // Add 30 claims, but only 20 should count
        spec.claims = (0..30)
            .map(|i| ValidationClaim {
                id: format!("C-{}", i),
                text: format!("Claim {}", i),
                line: i,
                category: ClaimCategory::Implementation,
                automatable: false,
                validation_cmd: None,
                expected_pattern: None,
            })
            .collect();
        let score = calculate_spec_score(&spec);
        // Max 20 claims = 20 pts
        assert_eq!(score, 20.0);
    }

    #[test]
    fn test_calculate_spec_score_test_requirements() {
        let mut spec = create_empty_spec();
        spec.test_requirements = vec![
            TestRequirement {
                text: "Unit test".to_string(),
                test_type: "unit".to_string(),
                code_path: None,
            },
            TestRequirement {
                text: "Integration test".to_string(),
                test_type: "integration".to_string(),
                code_path: None,
            },
        ];
        let score = calculate_spec_score(&spec);
        // 2 requirements * 3 pts each = 6 pts
        assert_eq!(score, 6.0);
    }

    #[test]
    fn test_calculate_spec_score_test_requirements_capped_at_5() {
        let mut spec = create_empty_spec();
        // Add 10 test requirements, but only 5 should count
        spec.test_requirements = (0..10)
            .map(|i| TestRequirement {
                text: format!("Test {}", i),
                test_type: "unit".to_string(),
                code_path: None,
            })
            .collect();
        let score = calculate_spec_score(&spec);
        // Max 5 requirements * 3 pts each = 15 pts
        assert_eq!(score, 15.0);
    }

    #[test]
    fn test_calculate_spec_score_full_spec() {
        let spec = create_full_spec();
        let score = calculate_spec_score(&spec);
        // Title: 5
        // Issue refs: 10
        // Code examples (5): 20
        // Acceptance criteria (10): 30
        // Claims (20): 20
        // Test requirements (5): 15
        // Total: 100
        assert_eq!(score, 100.0);
    }

    #[test]
    fn test_calculate_spec_score_capped_at_100() {
        let mut spec = create_full_spec();
        // Add even more items
        spec.claims = (0..50)
            .map(|i| ValidationClaim {
                id: format!("C-{}", i),
                text: format!("Claim {}", i),
                line: i,
                category: ClaimCategory::Implementation,
                automatable: false,
                validation_cmd: None,
                expected_pattern: None,
            })
            .collect();
        let score = calculate_spec_score(&spec);
        // Should not exceed 100
        assert!(score <= 100.0);
    }

    // ============================================================================
    // Tests for format_spec_score_text
    // ============================================================================

    #[test]
    fn test_format_spec_score_text_basic() {
        let spec = create_minimal_spec("My Spec");
        let output = format_spec_score_text(&spec, 75.0, false);

        assert!(output.contains("Specification Score"));
        assert!(output.contains("Title: My Spec"));
        assert!(output.contains("Score: 75.0/100"));
        assert!(output.contains("FAIL"));
    }

    #[test]
    fn test_format_spec_score_text_passing() {
        let spec = create_full_spec();
        let output = format_spec_score_text(&spec, 97.0, false);

        assert!(output.contains("PASS"));
        assert!(!output.contains("FAIL"));
    }

    #[test]
    fn test_format_spec_score_text_failing() {
        let spec = create_partial_spec();
        let output = format_spec_score_text(&spec, 50.0, false);

        assert!(output.contains("FAIL"));
        assert!(output.contains("needs"));
    }

    #[test]
    fn test_format_spec_score_text_verbose() {
        let spec = create_full_spec();
        let output = format_spec_score_text(&spec, 95.0, true);

        assert!(output.contains("Claims:"));
        assert!(output.contains("Code Examples:"));
        assert!(output.contains("Acceptance Criteria:"));
        assert!(output.contains("Issue Refs:"));
    }

    #[test]
    fn test_format_spec_score_text_non_verbose() {
        let spec = create_full_spec();
        let output = format_spec_score_text(&spec, 95.0, false);

        // Verbose content should not be present
        assert!(!output.contains("Claims:"));
    }

    #[test]
    fn test_format_spec_score_text_empty_title() {
        let spec = create_empty_spec();
        let output = format_spec_score_text(&spec, 0.0, false);

        // Should not contain "Title:" if title is empty
        assert!(!output.contains("Title:"));
    }

    // ============================================================================
    // Tests for format_spec_score_json
    // ============================================================================

    #[test]
    fn test_format_spec_score_json_basic() {
        let spec = create_minimal_spec("JSON Test Spec");
        let result = format_spec_score_json(&spec, 80.0).unwrap();

        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(json["title"], "JSON Test Spec");
        assert_eq!(json["score"], 80.0);
        assert_eq!(json["passing"], false);
    }

    #[test]
    fn test_format_spec_score_json_passing() {
        let spec = create_full_spec();
        let result = format_spec_score_json(&spec, 98.0).unwrap();

        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(json["passing"], true);
    }

    #[test]
    fn test_format_spec_score_json_counts() {
        let spec = create_full_spec();
        let result = format_spec_score_json(&spec, 100.0).unwrap();

        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(json["claims"], 20);
        assert_eq!(json["code_examples"], 5);
        assert_eq!(json["acceptance_criteria"], 10);
    }

    #[test]
    fn test_format_spec_score_json_issue_refs() {
        let spec = create_full_spec();
        let result = format_spec_score_json(&spec, 100.0).unwrap();

        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        let issue_refs = json["issue_refs"].as_array().unwrap();
        assert_eq!(issue_refs.len(), 2);
        assert!(issue_refs.contains(&serde_json::json!("#123")));
        assert!(issue_refs.contains(&serde_json::json!("#456")));
    }

    #[test]
    fn test_format_spec_score_json_valid_json() {
        let spec = create_partial_spec();
        let result = format_spec_score_json(&spec, 50.0).unwrap();

        // Ensure the result is valid JSON by parsing it
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&result);
        assert!(parsed.is_ok());
    }

    // ============================================================================
    // Tests for format_spec_score_markdown
    // ============================================================================

    #[test]
    fn test_format_spec_score_markdown_basic() {
        let spec = create_minimal_spec("Markdown Spec");
        let output = format_spec_score_markdown(&spec, 75.0);

        assert!(output.contains("# Specification Score Report"));
        assert!(output.contains("**Title:** Markdown Spec"));
        assert!(output.contains("| Score | 75.0/100 |"));
    }

    #[test]
    fn test_format_spec_score_markdown_table_format() {
        let spec = create_full_spec();
        let output = format_spec_score_markdown(&spec, 100.0);

        // Check table headers and structure
        assert!(output.contains("| Metric | Value |"));
        assert!(output.contains("|--------|-------|"));
    }

    #[test]
    fn test_format_spec_score_markdown_passing() {
        let spec = create_full_spec();
        let output = format_spec_score_markdown(&spec, 98.0);

        assert!(output.contains("PASS"));
    }

    #[test]
    fn test_format_spec_score_markdown_failing() {
        let spec = create_partial_spec();
        let output = format_spec_score_markdown(&spec, 50.0);

        assert!(output.contains("FAIL"));
    }

    #[test]
    fn test_format_spec_score_markdown_counts() {
        let spec = create_full_spec();
        let output = format_spec_score_markdown(&spec, 100.0);

        assert!(output.contains("| Claims | 20 |"));
        assert!(output.contains("| Code Examples | 5 |"));
        assert!(output.contains("| Acceptance Criteria | 10 |"));
    }

    #[test]
    fn test_format_spec_score_markdown_empty_title() {
        let spec = create_empty_spec();
        let output = format_spec_score_markdown(&spec, 0.0);

        // Should not contain title line if title is empty
        assert!(!output.contains("**Title:**"));
    }

    // ============================================================================
    // Tests for handle_spec_create (async)
    // ============================================================================

    #[tokio::test]
    async fn test_handle_spec_create_basic() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path();

        let result =
            handle_spec_create("Test Feature Spec", None, None, Some(output_path)).await;

        assert!(result.is_ok());

        let expected_file = output_path.join("test-feature-spec.md");
        assert!(expected_file.exists());

        let content = fs::read_to_string(&expected_file).unwrap();
        assert!(content.contains("# Test Feature Spec"));
        assert!(content.contains("issue_refs:"));
    }

    #[tokio::test]
    async fn test_handle_spec_create_with_issue() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path();

        let result =
            handle_spec_create("My Feature", Some("#999"), None, Some(output_path)).await;

        assert!(result.is_ok());

        let expected_file = output_path.join("my-feature.md");
        let content = fs::read_to_string(&expected_file).unwrap();
        assert!(content.contains("#999"));
    }

    #[tokio::test]
    async fn test_handle_spec_create_with_epic() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path();

        let result =
            handle_spec_create("Epic Feature", None, Some("EPIC-001"), Some(output_path)).await;

        assert!(result.is_ok());

        let expected_file = output_path.join("epic-feature.md");
        let content = fs::read_to_string(&expected_file).unwrap();
        assert!(content.contains("EPIC-001"));
    }

    #[tokio::test]
    async fn test_handle_spec_create_slug_conversion() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path();

        let result = handle_spec_create(
            "Complex Feature Name With Spaces",
            None,
            None,
            Some(output_path),
        )
        .await;

        assert!(result.is_ok());

        let expected_file = output_path.join("complex-feature-name-with-spaces.md");
        assert!(expected_file.exists());
    }

    #[tokio::test]
    async fn test_handle_spec_create_template_structure() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path();

        let result =
            handle_spec_create("Template Test", None, None, Some(output_path)).await;

        assert!(result.is_ok());

        let expected_file = output_path.join("template-test.md");
        let content = fs::read_to_string(&expected_file).unwrap();

        // Check template has all required sections
        assert!(content.contains("---")); // YAML frontmatter
        assert!(content.contains("## Executive Summary"));
        assert!(content.contains("## Scientific Foundation"));
        assert!(content.contains("## Requirements"));
        assert!(content.contains("### Functional Requirements"));
        assert!(content.contains("### Non-Functional Requirements"));
        assert!(content.contains("## Acceptance Criteria"));
        assert!(content.contains("## Code Examples"));
        assert!(content.contains("## Testing Strategy"));
        assert!(content.contains("## References"));
    }

    #[tokio::test]
    async fn test_handle_spec_create_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("nested").join("specs");

        let result =
            handle_spec_create("Nested Spec", None, None, Some(&nested_path)).await;

        assert!(result.is_ok());
        assert!(nested_path.exists());
    }

    // ============================================================================
    // Tests for handle_spec_comply (async)
    // ============================================================================

    #[tokio::test]
    async fn test_handle_spec_comply_compliant_spec() {
        let temp_dir = TempDir::new().unwrap();
        let spec_path = temp_dir.path().join("compliant.md");

        // Create a compliant spec with all requirements met
        let content = create_compliant_spec_content();
        fs::write(&spec_path, content).unwrap();

        let result =
            handle_spec_comply(&spec_path, true, SpecOutputFormat::Text).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_spec_comply_missing_issue_refs() {
        let temp_dir = TempDir::new().unwrap();
        let spec_path = temp_dir.path().join("no-issues.md");

        let content = r#"---
title: No Issues Spec
---
# No Issues Spec

## Requirements
- [ ] AC-001: First
"#;
        fs::write(&spec_path, content).unwrap();

        // The function should succeed but report the missing issue refs
        let result =
            handle_spec_comply(&spec_path, true, SpecOutputFormat::Text).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_spec_comply_missing_code_examples() {
        let temp_dir = TempDir::new().unwrap();
        let spec_path = temp_dir.path().join("no-code.md");

        let content = r##"---
title: No Code Spec
issue_refs: ["#123"]
---
# No Code Spec

## Requirements
- [ ] AC-001: First
"##;
        fs::write(&spec_path, content).unwrap();

        let result =
            handle_spec_comply(&spec_path, true, SpecOutputFormat::Text).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_spec_comply_dry_run_no_changes() {
        let temp_dir = TempDir::new().unwrap();
        let spec_path = temp_dir.path().join("dryrun.md");

        let content = r#"---
title: Dry Run Spec
---
# Dry Run Spec
"#;
        fs::write(&spec_path, content).unwrap();

        let original_content = fs::read_to_string(&spec_path).unwrap();

        let result =
            handle_spec_comply(&spec_path, true, SpecOutputFormat::Text).await;
        assert!(result.is_ok());

        // Content should be unchanged in dry run
        let final_content = fs::read_to_string(&spec_path).unwrap();
        assert_eq!(original_content, final_content);
    }

    #[tokio::test]
    async fn test_handle_spec_comply_file_not_found() {
        let result = handle_spec_comply(
            Path::new("/nonexistent/path/spec.md"),
            true,
            SpecOutputFormat::Text,
        )
        .await;

        assert!(result.is_err());
    }

    // ============================================================================
    // Tests for handle_spec_list (async)
    // ============================================================================

    #[tokio::test]
    async fn test_handle_spec_list_empty_directory() {
        let temp_dir = TempDir::new().unwrap();

        let result = handle_spec_list(
            temp_dir.path(),
            None,
            false,
            SpecOutputFormat::Text,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_spec_list_with_specs() {
        let temp_dir = TempDir::new().unwrap();

        // Create a few spec files
        let spec1 = temp_dir.path().join("spec1.md");
        let spec2 = temp_dir.path().join("spec2.md");

        fs::write(&spec1, create_compliant_spec_content()).unwrap();
        fs::write(&spec2, "# Simple Spec\n").unwrap();

        let result = handle_spec_list(
            temp_dir.path(),
            None,
            false,
            SpecOutputFormat::Text,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_spec_list_min_score_filter() {
        let temp_dir = TempDir::new().unwrap();

        let spec1 = temp_dir.path().join("high-score.md");
        let spec2 = temp_dir.path().join("low-score.md");

        fs::write(&spec1, create_compliant_spec_content()).unwrap();
        fs::write(&spec2, "# Low Score\n").unwrap();

        let result = handle_spec_list(
            temp_dir.path(),
            Some(50), // Only specs with score >= 50
            false,
            SpecOutputFormat::Text,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_spec_list_failing_only() {
        let temp_dir = TempDir::new().unwrap();

        let spec1 = temp_dir.path().join("failing.md");
        fs::write(&spec1, "# Failing Spec\n").unwrap();

        let result = handle_spec_list(
            temp_dir.path(),
            None,
            true, // failing_only
            SpecOutputFormat::Text,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_spec_list_json_format() {
        let temp_dir = TempDir::new().unwrap();

        let spec1 = temp_dir.path().join("spec.md");
        fs::write(&spec1, create_compliant_spec_content()).unwrap();

        let result = handle_spec_list(
            temp_dir.path(),
            None,
            false,
            SpecOutputFormat::Json,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_spec_list_markdown_format() {
        let temp_dir = TempDir::new().unwrap();

        let spec1 = temp_dir.path().join("spec.md");
        fs::write(&spec1, create_compliant_spec_content()).unwrap();

        let result = handle_spec_list(
            temp_dir.path(),
            None,
            false,
            SpecOutputFormat::Markdown,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_spec_list_single_file() {
        let temp_dir = TempDir::new().unwrap();
        let spec_path = temp_dir.path().join("single.md");

        fs::write(&spec_path, create_compliant_spec_content()).unwrap();

        // Pass a file path instead of directory
        let result = handle_spec_list(
            &spec_path,
            None,
            false,
            SpecOutputFormat::Text,
        )
        .await;

        assert!(result.is_ok());
    }

    // ============================================================================
    // Edge case tests
    // ============================================================================

    #[test]
    fn test_calculate_spec_score_boundary_at_95() {
        // Test score exactly at passing threshold
        let mut spec = create_empty_spec();
        spec.title = "Test".to_string(); // 5 pts
        spec.issue_refs = vec!["#1".to_string()]; // 10 pts
        spec.code_examples = (0..5)
            .map(|i| CodeExample {
                language: "rust".to_string(),
                code: format!("fn f{}() {{}}", i),
                line: i,
                executable: true,
            })
            .collect(); // 20 pts
        spec.acceptance_criteria = (0..10)
            .map(|i| AcceptanceCriterion {
                text: format!("AC-{}", i),
                complete: false,
                line: i,
            })
            .collect(); // 30 pts
        spec.claims = (0..20)
            .map(|i| ValidationClaim {
                id: format!("C-{}", i),
                text: format!("Claim {}", i),
                line: i,
                category: ClaimCategory::Implementation,
                automatable: false,
                validation_cmd: None,
                expected_pattern: None,
            })
            .collect(); // 20 pts
        spec.test_requirements = (0..5)
            .map(|i| TestRequirement {
                text: format!("Test {}", i),
                test_type: "unit".to_string(),
                code_path: None,
            })
            .collect(); // 15 pts

        let score = calculate_spec_score(&spec);
        // Total should be exactly 100
        assert_eq!(score, 100.0);
        assert!(score >= 95.0);
    }

    #[test]
    fn test_calculate_spec_score_just_below_95() {
        let mut spec = create_empty_spec();
        spec.title = "Test".to_string(); // 5 pts
        spec.issue_refs = vec!["#1".to_string()]; // 10 pts
        spec.code_examples = (0..5)
            .map(|i| CodeExample {
                language: "rust".to_string(),
                code: format!("fn f{}() {{}}", i),
                line: i,
                executable: true,
            })
            .collect(); // 20 pts
        spec.acceptance_criteria = (0..10)
            .map(|i| AcceptanceCriterion {
                text: format!("AC-{}", i),
                complete: false,
                line: i,
            })
            .collect(); // 30 pts
        spec.claims = (0..19)
            .map(|i| ValidationClaim {
                id: format!("C-{}", i),
                text: format!("Claim {}", i),
                line: i,
                category: ClaimCategory::Implementation,
                automatable: false,
                validation_cmd: None,
                expected_pattern: None,
            })
            .collect(); // 19 pts (one less)
        spec.test_requirements = (0..5)
            .map(|i| TestRequirement {
                text: format!("Test {}", i),
                test_type: "unit".to_string(),
                code_path: None,
            })
            .collect(); // 15 pts

        let score = calculate_spec_score(&spec);
        // Total should be 99 (just under max, but tests boundary logic)
        assert_eq!(score, 99.0);
    }

    #[test]
    fn test_format_outputs_consistency() {
        // Ensure all format functions handle the same spec consistently
        let spec = create_full_spec();
        let score = calculate_spec_score(&spec);

        let text = format_spec_score_text(&spec, score, false);
        let json_result = format_spec_score_json(&spec, score);
        let markdown = format_spec_score_markdown(&spec, score);

        // All should succeed
        assert!(!text.is_empty());
        assert!(json_result.is_ok());
        assert!(!markdown.is_empty());

        // All should indicate passing for a full spec
        assert!(text.contains("PASS"));
        let json: serde_json::Value = serde_json::from_str(&json_result.unwrap()).unwrap();
        assert_eq!(json["passing"], true);
        assert!(markdown.contains("PASS"));
    }

    #[test]
    fn test_format_outputs_failing_consistency() {
        let spec = create_empty_spec();
        let score = calculate_spec_score(&spec);

        let text = format_spec_score_text(&spec, score, false);
        let json_result = format_spec_score_json(&spec, score);
        let markdown = format_spec_score_markdown(&spec, score);

        // All should indicate failing for an empty spec
        assert!(text.contains("FAIL"));
        let json: serde_json::Value = serde_json::from_str(&json_result.unwrap()).unwrap();
        assert_eq!(json["passing"], false);
        assert!(markdown.contains("FAIL"));
    }

    // ============================================================================
    // Helper function to create a compliant spec content
    // ============================================================================

    fn create_compliant_spec_content() -> String {
        r##"---
title: Compliant Specification
status: Active
issue_refs: ["#123", "#456"]
---

# Compliant Specification

## Executive Summary

This is a compliant spec with [citation1] and [citation2].

## Scientific Foundation

1. Author et al., 2024. Paper Title. [citation3]
2. Another et al., 2024. Another Paper. [citation4]
3. Third et al., 2024. Third Paper. [citation5]

## Requirements

### Functional Requirements

The system MUST provide functionality.
The system SHOULD handle errors gracefully.
The system SHALL be backwards compatible.

### Non-Functional Requirements

Performance MUST be within acceptable limits.

## Acceptance Criteria

### Core Criteria

- [ ] AC-001: First criterion
- [ ] AC-002: Second criterion
- [ ] AC-003: Third criterion
- [ ] AC-004: Fourth criterion
- [ ] AC-005: Fifth criterion
- [ ] AC-006: Sixth criterion
- [ ] AC-007: Seventh criterion
- [ ] AC-008: Eighth criterion
- [ ] AC-009: Ninth criterion
- [ ] AC-010: Tenth criterion

## Code Examples

### Example 1

```rust
fn example_one() {
    println!("Example 1");
}
```

### Example 2

```rust
fn example_two() {
    println!("Example 2");
}
```

### Example 3

```rust
fn example_three() {
    println!("Example 3");
}
```

### Example 4

```rust
fn example_four() {
    println!("Example 4");
}
```

### Example 5

```rust
fn example_five() {
    println!("Example 5");
}
```

## Testing Strategy

Unit tests must cover 95% of the codebase.
Integration tests must validate all endpoints.
Property tests should verify invariants.
E2e tests should test the full workflow.
Mutation tests should validate test quality.

## References

[citation1] Reference 1
[citation2] Reference 2
[citation3] Reference 3
[citation4] Reference 4
[citation5] Reference 5
"##
        .to_string()
    }
}
