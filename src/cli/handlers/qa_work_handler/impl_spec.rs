fn print_task_status(task_id: &str, task_dir: &Path) -> Result<()> {
    use crate::cli::colors as c;
    let checklist_path = task_dir.join("checklist.yaml");

    if checklist_path.exists() {
        let content = fs::read_to_string(&checklist_path)?;
        let checklist: QaChecklist = serde_yaml_ng::from_str(&content)?;

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
            format!("{}Complete{}", c::GREEN, c::RESET)
        } else if checked > 0 {
            format!("{}In Progress{}", c::YELLOW, c::RESET)
        } else {
            format!("{}Pending{}", c::DIM, c::RESET)
        };

        println!(
            "{:<15} {:<20} {:.0}% ({}/{})",
            task_id, status, score, checked, total
        );
    } else {
        println!("{:<15} {}", task_id, c::dim("No checklist"));
    }

    Ok(())
}

/// Credit a claim earns for its validation status: `(proven, manual_unverified)`.
///
/// A MANUAL claim used to be added to the same counter as a PROVEN one, so a spec
/// whose claims were all unverified scored 100% in every category and the
/// Falsifiability gateway could never fail. Unverified is not verified: MANUAL is
/// tallied and reported, but scores nothing.
fn claim_credit(status: &crate::services::spec_parser::ValidationStatus) -> (u32, u32) {
    use crate::services::spec_parser::ValidationStatus as S;
    match status {
        S::Proven => (1, 0),
        S::ManualRequired => (0, 1),
        _ => (0, 0),
    }
}

/// The Falsifiability gateway: the category must clear the threshold *and* contain
/// at least one claim we actually proved. A gateway satisfied by unverified claims
/// is not a gateway.
fn falsifiability_gateway_passed(
    gateway_score: f64,
    gateway_threshold: u32,
    gateway_proven: u32,
) -> bool {
    gateway_score >= gateway_threshold as f64 && gateway_proven > 0
}

/// Handle spec validation command (Part D & E: pmat qa spec)
///
/// Implements 100-point Popperian falsifiability scoring:
/// - A. Falsifiability (25 pts) - GATEWAY CHECK (must score >=60% or total=0)
/// - B. Implementation (25 pts)
/// - C. Testing (20 pts)
/// - D. Documentation (15 pts)
/// - E. Integration (15 pts)
async fn handle_spec(
    target: &str,
    project_path: &Path,
    full: bool,
    format: QaOutputFormat,
    output: Option<&Path>,
    threshold: u32,
    gateway_threshold: u32,
) -> Result<()> {
    use crate::cli::colors as c;
    use crate::services::spec_parser::{
        ClaimCategory, SpecParser, ValidationStatus as SpecValidationStatus,
    };

    println!("{}", c::header("Popperian Specification Validation"));
    println!("{}", c::rule());
    println!("{}: {}", c::dim("Target"), target);
    println!(
        "{}: {}",
        c::dim("Mode"),
        if full {
            "Full (with mutation testing)"
        } else {
            "Standard"
        }
    );
    println!();

    // Resolve target to specification file
    let spec_path = resolve_spec_path(target, project_path)?;
    println!("{}: {}", c::dim("Specification"), c::path(&spec_path.display().to_string()));
    println!();

    // Parse specification
    let parser = SpecParser::new();
    let spec = parser.parse_file(&spec_path)?;

    println!("{}: {}", c::dim("Title"), spec.title);
    println!("{}: {:?}", c::dim("Issue refs"), spec.issue_refs);
    println!("{}: {}", c::dim("Claims"), c::number(&spec.claims.len().to_string()));
    println!("{}: {}", c::dim("Code examples"), c::number(&spec.code_examples.len().to_string()));
    println!("{}: {}", c::dim("Acceptance criteria"), c::number(&spec.acceptance_criteria.len().to_string()));
    println!();

    // Validate claims by category
    println!("{}", c::header("Validation Results (Popperian: FALSE until PROVEN)"));
    println!();

    // (proven, manual_unverified, total) per category. A MANUAL claim used to be
    // added to the same counter as a PROVEN one ("Count as passed"), so every
    // category with only unverified claims scored 100% and the Falsifiability
    // gateway could never fail. Unverified is not verified: only PROVEN earns points.
    let mut category_scores: HashMap<String, (u32, u32, u32)> = HashMap::new();

    // Initialize categories
    for cat in &[
        ClaimCategory::Falsifiability,
        ClaimCategory::Implementation,
        ClaimCategory::Testing,
        ClaimCategory::Documentation,
        ClaimCategory::Integration,
    ] {
        let cat_name = format!("{:?}", cat);
        category_scores.insert(cat_name, (0, 0, 0));
    }

    // Validate each claim
    for claim in &spec.claims {
        let cat_name = format!("{:?}", claim.category);
        let entry = category_scores.entry(cat_name.clone()).or_insert((0, 0, 0));
        entry.2 += 1; // total

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

        // Update score. Only a claim we actually verified scores; MANUAL claims are
        // tallied separately and reported, but they earn nothing — counting them as
        // passed is what let an all-unverified spec show 100% in every category.
        let (proven_credit, manual_credit) = claim_credit(&status);
        entry.0 += proven_credit;
        entry.1 += manual_credit;

        // Print result
        let status_str = match status {
            SpecValidationStatus::Proven => format!("{}✓ PROVEN{}", c::GREEN, c::RESET),
            SpecValidationStatus::Falsified => format!("{}✗ FALSIFIED{}", c::RED, c::RESET),
            SpecValidationStatus::Unfalsified => format!("{}? UNFALSIFIED{}", c::YELLOW, c::RESET),
            SpecValidationStatus::ManualRequired => format!("{}⚙ MANUAL{}", c::BLUE, c::RESET),
            SpecValidationStatus::Skipped => format!("{}- SKIPPED{}", c::DIM, c::RESET),
        };

        // Use chars() to avoid Unicode boundary panics (issue #120)
        let truncated: String = claim.text.chars().take(60).collect();
        println!(
            "  {} [{}] {} - {}",
            status_str,
            claim.id,
            truncated,
            c::dim(&cat_name)
        );

        if let Some(ref ev) = evidence {
            if ev.len() < 100 {
                println!("      {}: {}", c::dim("Evidence"), ev);
            }
        }
    }

    println!();

    // Calculate scores
    println!("{}", c::header("Category Scores (100-point Popperian Framework)"));
    println!();

    let mut total_score: f64 = 0.0;
    let mut gateway_score: f64 = 0.0;
    let mut gateway_proven: u32 = 0;

    for cat in &[
        ClaimCategory::Falsifiability,
        ClaimCategory::Implementation,
        ClaimCategory::Testing,
        ClaimCategory::Documentation,
        ClaimCategory::Integration,
    ] {
        let cat_name = format!("{:?}", cat);
        let (proven, manual, total) = category_scores.get(&cat_name).unwrap_or(&(0, 0, 0));
        let max_pts = cat.max_points();

        let cat_score = if *total > 0 {
            (*proven as f64 / *total as f64) * max_pts as f64
        } else {
            0.0
        };

        let pct = if *total > 0 {
            (*proven as f64 / *total as f64) * 100.0
        } else {
            0.0
        };

        if *cat == ClaimCategory::Falsifiability {
            gateway_score = cat_score;
            gateway_proven = *proven;
            print!("  {} ", c::label("GATE"));
        } else {
            print!("     ");
        }

        println!(
            "{:<15} {}/{} pts ({:.0}%) - {}/{} claims proven{}",
            cat_name,
            c::number(&format!("{:.1}", cat_score)),
            max_pts,
            pct,
            proven,
            total,
            if *manual > 0 {
                format!(" ({} manual, unverified)", manual)
            } else {
                String::new()
            }
        );

        total_score += cat_score;
    }

    println!();
    println!("{}", c::rule());

    // Gateway check (Falsifiability category must meet threshold AND have at least
    // one claim we actually proved — a gateway satisfied by unverified claims is no
    // gateway at all).
    let gateway_passed =
        falsifiability_gateway_passed(gateway_score, gateway_threshold, gateway_proven);
    let final_score = if gateway_passed { total_score } else { 0.0 };

    if !gateway_passed {
        println!(
            "{} GATEWAY FAILED: Falsifiability score {:.1} < {} or no claim proven (total score forced to 0)",
            c::fail(""),
            gateway_score,
            gateway_threshold
        );
        println!(
            "   {}",
            c::dim("Per Popper: Without falsifiable claims, the specification is non-scientific.")
        );
    } else {
        println!(
            "{} Gateway passed: Falsifiability score {:.1} >= {}",
            c::pass(""),
            gateway_score,
            gateway_threshold
        );
    }

    println!();
    // Named "claim falsifiability score", not "total score": `pmat spec score`
    // reports a different 100-point number for the same file (artefact completeness,
    // ≥95 bar) and the two were indistinguishable in the output.
    println!(
        "{}: {}/100 (threshold: {})",
        c::label("Claim falsifiability score"),
        c::number(&format!("{:.1}", final_score)),
        threshold
    );

    let passed = final_score >= threshold as f64;
    if passed {
        println!("{}", c::pass("PASSED"));
    } else {
        println!("{}", c::fail("FAILED (score below threshold)"));
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
            QaOutputFormat::Yaml => serde_yaml_ng::to_string(&result)?,
            QaOutputFormat::Markdown => format_spec_result_markdown(&result),
            QaOutputFormat::Text => format!("{:#?}", result),
        };

        fs::write(output_path, &output_content)?;
        println!("\n{} Results saved to: {}", c::pass(""), c::path(&output_path.display().to_string()));
    }

    if !passed {
        anyhow::bail!("Specification validation failed");
    }

    Ok(())
}

/// Resolve target to specification file path
fn resolve_spec_path(target: &str, project_path: &Path) -> Result<PathBuf> {
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
async fn run_validation_command(cmd: &str, project_path: &Path) -> Result<String> {
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
fn format_spec_result_markdown(result: &serde_json::Value) -> String {
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
            "PASSED"
        } else {
            "FAILED"
        },
        result["gateway_score"].as_f64().unwrap_or(0.0),
        if result["gateway_passed"].as_bool().unwrap_or(false) {
            "✓"
        } else {
            "✗"
        },
    )
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod impl_spec_scoring_tests {
    use super::{claim_credit, falsifiability_gateway_passed};
    use crate::services::spec_parser::ValidationStatus as S;

    /// A MANUAL (unverified) claim must earn no points. It used to be counted as
    /// passed, which is why every category read 100% on a spec with 9 MANUAL claims.
    #[test]
    fn test_manual_claims_earn_no_score() {
        assert_eq!(claim_credit(&S::Proven), (1, 0));
        assert_eq!(claim_credit(&S::ManualRequired), (0, 1));
        assert_eq!(claim_credit(&S::Falsified), (0, 0));
        assert_eq!(claim_credit(&S::Unfalsified), (0, 0));
        assert_eq!(claim_credit(&S::Skipped), (0, 0));
    }

    /// 9 MANUAL + 1 PROVEN in Implementation scores 1/10 of the category, not 100%.
    #[test]
    fn test_category_score_uses_proven_only() {
        let statuses = [
            S::Proven,
            S::ManualRequired,
            S::ManualRequired,
            S::ManualRequired,
        ];
        let (proven, manual) = statuses.iter().fold((0u32, 0u32), |(p, m), s| {
            let (pc, mc) = claim_credit(s);
            (p + pc, m + mc)
        });
        assert_eq!((proven, manual), (1, 3));
        let score = proven as f64 / statuses.len() as f64 * 25.0;
        assert!((score - 6.25).abs() < 1e-9, "got {score}");
    }

    /// The gateway must fail when its only falsifiability claim is unverified,
    /// however many claims exist.
    #[test]
    fn test_gateway_fails_without_a_proven_claim() {
        assert!(
            !falsifiability_gateway_passed(0.0, 15, 0),
            "no proven claim must not satisfy the gateway"
        );
        assert!(
            !falsifiability_gateway_passed(25.0, 15, 0),
            "a score without a proven claim must not satisfy the gateway"
        );
        assert!(falsifiability_gateway_passed(25.0, 15, 1));
        assert!(!falsifiability_gateway_passed(10.0, 15, 1));
    }
}
