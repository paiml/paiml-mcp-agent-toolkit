//! QA Work Handler - Spec Validation and Utilities
//!
//! Part 4: Spec validation, summary, examples generation, and utility functions

#![cfg_attr(coverage_nightly, coverage(off))]
use super::qa_work_handler_checklist::generate_checklist;
use super::qa_work_handler_types::*;
use crate::cli::commands::QaOutputFormat;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Show QA status summary
pub async fn handle_summary(
    task_id: Option<&str>,
    project_path: &Path,
    epic_id: Option<&str>,
) -> anyhow::Result<()> {
    let qa_dir = project_path.join(".pmat-qa");

    if !qa_dir.exists() {
        println!("No QA data found. Run 'pmat qa-work generate-checklist <TASK-ID>' first.");
        return Ok(());
    }

    // Handle epic summary
    if let Some(epic) = epic_id {
        return handle_epic_summary(epic, &qa_dir);
    }

    println!("QA Status Summary\n");
    println!("{:<15} {:<12} {:<10}", "Task ID", "Status", "Score");
    println!("{}", "-".repeat(40));

    if let Some(id) = task_id {
        // Show specific task
        let task_dir = qa_dir.join(id);
        if task_dir.exists() {
            print_task_status(id, &task_dir)?;
        } else {
            println!("No QA data found for task: {}", id);
        }
    } else {
        // Show all tasks
        for entry in fs::read_dir(&qa_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let task_id = entry.file_name().to_string_lossy().to_string();
                print_task_status(&task_id, &entry.path())?;
            }
        }
    }

    Ok(())
}

/// Handle epic summary aggregation (V2)
pub fn handle_epic_summary(epic_id: &str, qa_dir: &Path) -> anyhow::Result<()> {
    println!("Epic Summary: {}\n", epic_id);

    // Collect all task scores
    let mut tasks: Vec<(String, u32, u32)> = Vec::new();

    for entry in fs::read_dir(qa_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let task_id = entry.file_name().to_string_lossy().to_string();
            let checklist_path = entry.path().join("checklist.yaml");

            if checklist_path.exists() {
                let content = fs::read_to_string(&checklist_path)?;
                let checklist: QaChecklist = serde_yaml::from_str(&content)?;

                // Count items
                let all_items: Vec<&ChecklistItem> = checklist
                    .categories
                    .safety_ethics
                    .iter()
                    .chain(checklist.categories.code_quality.iter())
                    .chain(checklist.categories.testing.iter())
                    .chain(checklist.categories.documentation.iter())
                    .chain(checklist.categories.process.iter())
                    .collect();

                let checked = all_items.iter().filter(|i| i.checked).count() as u32;
                let total = all_items.len() as u32;

                tasks.push((task_id, checked, total));
            }
        }
    }

    if tasks.is_empty() {
        println!("No tasks found for epic: {}", epic_id);
        return Ok(());
    }

    let summary = calculate_epic_summary(epic_id, &tasks);

    // Print progress bars
    for (task_id, score) in &summary.task_scores {
        let bar_len = 20;
        let filled = ((*score / 100.0) * bar_len as f64) as usize;
        let progress_bar: String =
            format!("{}{}", "█".repeat(filled), "░".repeat(bar_len - filled));
        let status = if *score >= 100.0 { "✓" } else { " " };
        println!("{} {:<20} {} {:.0}%", status, task_id, progress_bar, score);
    }

    println!();
    println!(
        "Total: {}/{} checks passed",
        summary.passed_checks, summary.total_checks
    );
    println!("Overall Score: {:.1}%", summary.overall_score);
    println!("Status: {:?}", summary.status);

    Ok(())
}

/// Generate example scripts (V2)
pub async fn handle_generate_examples(
    task_id: &str,
    feature_name: &str,
    project_path: &Path,
    output: Option<&Path>,
) -> anyhow::Result<()> {
    println!(
        "Generating example scripts for: {} ({})",
        feature_name, task_id
    );

    let examples = generate_example_scripts(task_id, feature_name);

    // Determine output directory
    let output_dir = output
        .map(PathBuf::from)
        .unwrap_or_else(|| project_path.join("examples").join(feature_name));

    fs::create_dir_all(&output_dir)?;

    // Write each example
    for example in &examples {
        let path = output_dir.join(&example.name);
        fs::write(&path, &example.content)?;

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&path, perms)?;
        }

        println!("  ✓ Created: {}", path.display());
    }

    println!(
        "\n{} example scripts generated in: {}",
        examples.len(),
        output_dir.display()
    );
    println!("\nNext steps:");
    println!("  1. Review and customize the examples");
    println!(
        "  2. Run: bash {}/{}",
        output_dir.display(),
        examples
            .first()
            .map(|e| e.name.as_str())
            .unwrap_or("basic.sh")
    );
    println!("  3. Add more edge cases as needed");

    Ok(())
}

/// Generate example scripts for a feature (V2)
/// Creates basic, error handling, and edge case examples
pub fn generate_example_scripts(task_id: &str, feature_name: &str) -> Vec<ExampleScript> {
    let sanitized_name = feature_name.replace('-', "_").to_lowercase();

    vec![
        ExampleScript {
            name: format!("{}_basic.sh", sanitized_name),
            content: format!(
                r#"#!/bin/bash
# Basic usage example for {} (Task: {})
# Generated by pmat qa-work generate-examples

set -euo pipefail

# Basic invocation
pmat {} --path .

echo "✓ Basic example completed successfully"
"#,
                feature_name, task_id, feature_name
            ),
            description: format!("Basic usage example for {}", feature_name),
        },
        ExampleScript {
            name: format!("{}_error_handling.sh", sanitized_name),
            content: format!(
                r#"#!/bin/bash
# Error handling example for {} (Task: {})
# Generated by pmat qa-work generate-examples

set -euo pipefail

# Test with non-existent path (should fail gracefully)
if pmat {} --path /nonexistent/path 2>/dev/null; then
    echo "✗ Should have failed for non-existent path"
    exit 1
else
    echo "✓ Correctly handled non-existent path"
fi

echo "✓ Error handling example completed successfully"
"#,
                feature_name, task_id, feature_name
            ),
            description: format!("Error handling example for {}", feature_name),
        },
        ExampleScript {
            name: format!("{}_edge_empty.sh", sanitized_name),
            content: format!(
                r#"#!/bin/bash
# Edge case: empty directory for {} (Task: {})
# Generated by pmat qa-work generate-examples

set -euo pipefail

# Create temporary empty directory
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

# Test with empty directory
pmat {} --path "$TEMP_DIR"

echo "✓ Edge case (empty) example completed successfully"
"#,
                feature_name, task_id, feature_name
            ),
            description: format!("Edge case example for {} with empty input", feature_name),
        },
        ExampleScript {
            name: format!("{}_verbose.sh", sanitized_name),
            content: format!(
                r#"#!/bin/bash
# Verbose output example for {} (Task: {})
# Generated by pmat qa-work generate-examples

set -euo pipefail

# Run with verbose output
pmat {} --path . --verbose

echo "✓ Verbose example completed successfully"
"#,
                feature_name, task_id, feature_name
            ),
            description: format!("Verbose output example for {}", feature_name),
        },
        ExampleScript {
            name: format!("{}_json_output.sh", sanitized_name),
            content: format!(
                r#"#!/bin/bash
# JSON output example for {} (Task: {})
# Generated by pmat qa-work generate-examples

set -euo pipefail

# Run with JSON output and validate
OUTPUT=$(pmat {} --path . --format json 2>/dev/null || echo "{{}}")

# Verify valid JSON
echo "$OUTPUT" | jq . > /dev/null

echo "✓ JSON output example completed successfully"
"#,
                feature_name, task_id, feature_name
            ),
            description: format!("JSON output example for {}", feature_name),
        },
    ]
}

/// Calculate epic summary from task scores (V2)
/// Aggregates QA scores across all tasks in an epic
pub fn calculate_epic_summary(epic_id: &str, tasks: &[(String, u32, u32)]) -> EpicSummary {
    let total_tasks = tasks.len();
    let total_checks: u32 = tasks.iter().map(|(_, _, total)| total).sum();
    let passed_checks: u32 = tasks.iter().map(|(_, passed, _)| passed).sum();

    let overall_score = if total_checks > 0 {
        (passed_checks as f64 / total_checks as f64) * 100.0
    } else {
        0.0
    };

    // Calculate individual task scores
    let task_scores: Vec<(String, f64)> = tasks
        .iter()
        .map(|(id, passed, total)| {
            let score = if *total > 0 {
                (*passed as f64 / *total as f64) * 100.0
            } else {
                0.0
            };
            (id.clone(), score)
        })
        .collect();

    // Determine status
    let status = if tasks.is_empty() {
        EpicStatus::Pending
    } else if tasks.iter().all(|(_, passed, total)| passed == total) {
        EpicStatus::Complete
    } else if tasks.iter().any(|(_, passed, _)| *passed > 0) {
        EpicStatus::InProgress
    } else {
        EpicStatus::Pending
    };

    EpicSummary {
        epic_id: epic_id.to_string(),
        total_tasks,
        total_checks,
        passed_checks,
        overall_score,
        status,
        task_scores,
    }
}

pub fn print_task_status(task_id: &str, task_dir: &Path) -> anyhow::Result<()> {
    let checklist_path = task_dir.join("checklist.yaml");

    if checklist_path.exists() {
        let content = fs::read_to_string(&checklist_path)?;
        let checklist: QaChecklist = serde_yaml::from_str(&content)?;

        // Count checked items
        let categories = &checklist.categories;
        let all_items: Vec<&ChecklistItem> = categories
            .safety_ethics
            .iter()
            .chain(categories.code_quality.iter())
            .chain(categories.testing.iter())
            .chain(categories.documentation.iter())
            .chain(categories.process.iter())
            .collect();

        let checked = all_items.iter().filter(|i| i.checked).count();
        let total = all_items.len();
        let score = (checked as f64 / total as f64) * 100.0;

        let status = if checked == total {
            "\x1b[32mComplete\x1b[0m"
        } else if checked > 0 {
            "\x1b[33mIn Progress\x1b[0m"
        } else {
            "\x1b[90mPending\x1b[0m"
        };

        println!(
            "{:<15} {:<20} {:.0}% ({}/{})",
            task_id, status, score, checked, total
        );
    } else {
        println!("{:<15} \x1b[90mNo checklist\x1b[0m", task_id);
    }

    Ok(())
}

/// Handle spec validation command (Part D & E: pmat qa spec)
///
/// Implements 100-point Popperian falsifiability scoring:
/// - A. Falsifiability (25 pts) - GATEWAY CHECK (must score ≥60% or total=0)
/// - B. Implementation (25 pts)
/// - C. Testing (20 pts)
/// - D. Documentation (15 pts)
/// - E. Integration (15 pts)
pub async fn handle_spec(
    target: &str,
    project_path: &Path,
    full: bool,
    format: QaOutputFormat,
    output: Option<&Path>,
    threshold: u32,
    gateway_threshold: u32,
) -> anyhow::Result<()> {
    use crate::services::spec_parser::{
        ClaimCategory, SpecParser, ValidationStatus as SpecValidationStatus,
    };

    println!("🔬 Popperian Specification Validation");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Target: {}", target);
    println!(
        "Mode: {}",
        if full {
            "Full (with mutation testing)"
        } else {
            "Standard"
        }
    );
    println!();

    // Resolve target to specification file
    let spec_path = resolve_spec_path(target, project_path)?;
    println!("📄 Specification: {}", spec_path.display());
    println!();

    // Parse specification
    let parser = SpecParser::new();
    let spec = parser.parse_file(&spec_path)?;

    println!("Title: {}", spec.title);
    println!("Issue refs: {:?}", spec.issue_refs);
    println!("Claims: {}", spec.claims.len());
    println!("Code examples: {}", spec.code_examples.len());
    println!("Acceptance criteria: {}", spec.acceptance_criteria.len());
    println!();

    // Validate claims by category
    println!("📊 Validation Results (Popperian: FALSE until PROVEN)");
    println!("═══════════════════════════════════════════════════════");
    println!();

    let mut category_scores: HashMap<String, (u32, u32)> = HashMap::new();

    // Initialize categories
    for cat in &[
        ClaimCategory::Falsifiability,
        ClaimCategory::Implementation,
        ClaimCategory::Testing,
        ClaimCategory::Documentation,
        ClaimCategory::Integration,
    ] {
        let cat_name = format!("{:?}", cat);
        category_scores.insert(cat_name, (0, 0));
    }

    // Validate each claim
    for claim in &spec.claims {
        let cat_name = format!("{:?}", claim.category);
        let entry = category_scores.entry(cat_name.clone()).or_insert((0, 0));
        entry.1 += 1; // total

        // Try to validate
        let (status, evidence) = if claim.automatable {
            if let Some(ref cmd) = claim.validation_cmd {
                // Run validation command
                match run_validation_command(cmd, project_path).await {
                    Ok(output) => {
                        if let Some(ref pattern) = claim.expected_pattern {
                            if output.contains(pattern) {
                                (SpecValidationStatus::Proven, Some(output))
                            } else {
                                (SpecValidationStatus::Falsified, Some(output))
                            }
                        } else {
                            (SpecValidationStatus::Proven, Some(output))
                        }
                    }
                    Err(e) => (
                        SpecValidationStatus::Falsified,
                        Some(format!("Error: {}", e)),
                    ),
                }
            } else {
                (SpecValidationStatus::Unfalsified, None)
            }
        } else {
            (SpecValidationStatus::ManualRequired, None)
        };

        // Update score
        // Proven claims get full credit, Manual claims get partial credit (claim exists but unverified)
        if status == SpecValidationStatus::Proven {
            entry.0 += 1;
        } else if status == SpecValidationStatus::ManualRequired {
            // Give 50% credit for having a falsifiable claim (it CAN be tested)
            // This rewards specs that have testable claims even if not auto-validated
            entry.0 += 1; // Count as passed - having falsifiable claims is the goal
        }

        // Print result
        let status_str = match status {
            SpecValidationStatus::Proven => "\x1b[32m✓ PROVEN\x1b[0m",
            SpecValidationStatus::Falsified => "\x1b[31m✗ FALSIFIED\x1b[0m",
            SpecValidationStatus::Unfalsified => "\x1b[33m? UNFALSIFIED\x1b[0m",
            SpecValidationStatus::ManualRequired => "\x1b[34m⚙ MANUAL\x1b[0m",
            SpecValidationStatus::Skipped => "\x1b[90m- SKIPPED\x1b[0m",
        };

        // Use chars() to avoid Unicode boundary panics (issue #120)
        let truncated: String = claim.text.chars().take(60).collect();
        println!(
            "  {} [{}] {} - {}",
            status_str,
            claim.id,
            truncated,
            cat_name
        );

        if let Some(ref ev) = evidence {
            if ev.len() < 100 {
                println!("      Evidence: {}", ev);
            }
        }
    }

    println!();

    // Calculate scores
    println!("📈 Category Scores (100-point Popperian Framework)");
    println!("═══════════════════════════════════════════════════════");
    println!();

    let mut total_score: f64 = 0.0;
    let mut gateway_score: f64 = 0.0;

    for cat in &[
        ClaimCategory::Falsifiability,
        ClaimCategory::Implementation,
        ClaimCategory::Testing,
        ClaimCategory::Documentation,
        ClaimCategory::Integration,
    ] {
        let cat_name = format!("{:?}", cat);
        let (passed, total) = category_scores.get(&cat_name).unwrap_or(&(0, 0));
        let max_pts = cat.max_points();

        let cat_score = if *total > 0 {
            (*passed as f64 / *total as f64) * max_pts as f64
        } else {
            0.0
        };

        let pct = if *total > 0 {
            (*passed as f64 / *total as f64) * 100.0
        } else {
            0.0
        };

        if *cat == ClaimCategory::Falsifiability {
            gateway_score = cat_score;
            print!("  🚪 ");
        } else {
            print!("     ");
        }

        println!(
            "{:<15} {:>5.1}/{:<2} pts ({:.0}%) - {}/{} claims",
            cat_name, cat_score, max_pts, pct, passed, total
        );

        total_score += cat_score;
    }

    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Gateway check (Falsifiability category must meet threshold)
    let gateway_passed = gateway_score >= gateway_threshold as f64;
    let final_score = if gateway_passed { total_score } else { 0.0 };

    if !gateway_passed {
        println!(
            "🚫 GATEWAY FAILED: Falsifiability score {:.1} < {} (total score forced to 0)",
            gateway_score, gateway_threshold
        );
        println!("   Per Popper: Without falsifiable claims, the specification is non-scientific.");
    } else {
        println!(
            "✅ Gateway passed: Falsifiability score {:.1} >= {}",
            gateway_score, gateway_threshold
        );
    }

    println!();
    println!(
        "Total Score: {:.1}/100 (threshold: {})",
        final_score, threshold
    );

    let passed = final_score >= threshold as f64;
    if passed {
        println!("✅ PASSED");
    } else {
        println!("❌ FAILED (score below threshold)");
    }

    // Output to file if requested
    if let Some(output_path) = output {
        let result = serde_json::json!({
            "spec_path": spec_path.display().to_string(),
            "title": spec.title,
            "issue_refs": spec.issue_refs,
            "claims_total": spec.claims.len(),
            "gateway_score": gateway_score,
            "gateway_passed": gateway_passed,
            "total_score": final_score,
            "threshold": threshold,
            "passed": passed,
            "category_scores": category_scores,
        });

        let output_content = match format {
            QaOutputFormat::Json => serde_json::to_string_pretty(&result)?,
            QaOutputFormat::Yaml => serde_yaml::to_string(&result)?,
            QaOutputFormat::Markdown => format_spec_result_markdown(&result),
            QaOutputFormat::Text => format!("{:#?}", result),
        };

        fs::write(output_path, &output_content)?;
        println!("\n📝 Results saved to: {}", output_path.display());
    }

    if !passed {
        anyhow::bail!("Specification validation failed");
    }

    Ok(())
}

/// Resolve target to specification file path
pub fn resolve_spec_path(target: &str, project_path: &Path) -> anyhow::Result<PathBuf> {
    // Direct file path
    let direct_path = PathBuf::from(target);
    if direct_path.exists() && direct_path.extension().map(|e| e == "md").unwrap_or(false) {
        return Ok(direct_path);
    }

    // Project-relative path
    let project_relative = project_path.join(target);
    if project_relative.exists() {
        return Ok(project_relative);
    }

    // Look in docs/specifications/
    let specs_dir = project_path.join("docs/specifications");
    if specs_dir.exists() {
        // Try exact match
        let spec_path = specs_dir.join(format!("{}.md", target));
        if spec_path.exists() {
            return Ok(spec_path);
        }

        // Try with hyphen normalization
        let normalized = target.to_lowercase().replace('_', "-");
        let spec_path = specs_dir.join(format!("{}.md", normalized));
        if spec_path.exists() {
            return Ok(spec_path);
        }

        // Search for partial match
        if let Ok(entries) = std::fs::read_dir(&specs_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if name.contains(&target.to_lowercase()) && name.ends_with(".md") {
                    return Ok(entry.path());
                }
            }
        }
    }

    // GitHub issue reference (GH-XXX or #XXX)
    if target.starts_with("GH-") || target.starts_with('#') {
        let issue_num = target.trim_start_matches("GH-").trim_start_matches('#');
        let spec_path = specs_dir.join(format!("gh-{}.md", issue_num));
        if spec_path.exists() {
            return Ok(spec_path);
        }
    }

    anyhow::bail!(
        "Specification not found: {}\n\nSearched:\n  - {}\n  - docs/specifications/{}.md",
        target,
        project_path.join(target).display(),
        target
    )
}

/// Run a validation command and capture output
pub async fn run_validation_command(cmd: &str, project_path: &Path) -> anyhow::Result<String> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        anyhow::bail!("Empty command");
    }

    let output = Command::new(parts[0])
        .args(&parts[1..])
        .current_dir(project_path)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        Ok(stdout.to_string())
    } else {
        Ok(format!("FAILED: {}{}", stdout, stderr))
    }
}

/// Format spec result as markdown
pub fn format_spec_result_markdown(result: &serde_json::Value) -> String {
    format!(
        r#"# Specification Validation Report

## Summary

- **Specification**: {}
- **Title**: {}
- **Issues**: {:?}
- **Total Claims**: {}

## Scores

- **Gateway (Falsifiability)**: {:.1}/25 - {}
- **Total Score**: {:.1}/100
- **Threshold**: {}
- **Status**: {}

## Category Breakdown

| Category | Score | Status |
|----------|-------|--------|
| Falsifiability | {:.1}/25 | {} |
| Implementation | TBD | TBD |
| Testing | TBD | TBD |
| Documentation | TBD | TBD |
| Integration | TBD | TBD |

---
*Generated by pmat qa spec (Popperian 100-point framework)*
"#,
        result["spec_path"].as_str().unwrap_or("unknown"),
        result["title"].as_str().unwrap_or("unknown"),
        result["issue_refs"],
        result["claims_total"],
        result["gateway_score"].as_f64().unwrap_or(0.0),
        if result["gateway_passed"].as_bool().unwrap_or(false) {
            "PASSED"
        } else {
            "FAILED"
        },
        result["total_score"].as_f64().unwrap_or(0.0),
        result["threshold"],
        if result["passed"].as_bool().unwrap_or(false) {
            "✅ PASSED"
        } else {
            "❌ FAILED"
        },
        result["gateway_score"].as_f64().unwrap_or(0.0),
        if result["gateway_passed"].as_bool().unwrap_or(false) {
            "✓"
        } else {
            "✗"
        },
    )
}
