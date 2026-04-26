/// Print validation result as text
fn print_validation_text(result: &QaValidationResult) {
    use crate::cli::colors as c;
    println!("Validating {}...\n", c::label(&result.task_id));

    for category in result.categories.values() {
        let status = if category.passed == category.total {
            c::pass("")
        } else if category.passed > 0 {
            c::warn("")
        } else {
            c::fail("")
        };

        println!(
            "{} {} ({}/{})",
            status, c::subheader(&category.name), category.passed, category.total
        );

        for item in &category.items {
            let item_status = match item.status {
                ValidationStatus::Passed => format!("  {}✓{}", c::GREEN, c::RESET),
                ValidationStatus::Failed => format!("  {}✗{}", c::RED, c::RESET),
                ValidationStatus::Warning => format!("  {}⚠{}", c::YELLOW, c::RESET),
                ValidationStatus::Skipped => format!("  {}-{}", c::DIM, c::RESET),
                ValidationStatus::Manual => format!("  {}?{}", c::BLUE, c::RESET),
            };
            println!("{}  {}: {}", item_status, item.id, item.description);
        }
        println!();
    }

    println!("{}: {:.1}%", c::label("Overall Score"), result.overall_score);
    println!();

    if !result.manual_checks_required.is_empty() {
        println!("{}:", c::warn("Manual Checks Required"));
        for check in &result.manual_checks_required {
            println!("  - {}", check);
        }
        println!();
    }

    if result.passed {
        println!("{}", c::pass("QA Validation PASSED"));
    } else {
        println!("{}", c::fail("QA Validation FAILED"));
    }
}

/// Print validation result as markdown
fn print_validation_markdown(result: &QaValidationResult) {
    println!("# QA Validation Report: {}\n", result.task_id);
    println!(
        "**Date**: {}",
        result.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!("**Score**: {:.1}%\n", result.overall_score);

    for category in result.categories.values() {
        println!(
            "## {} ({}/{})\n",
            category.name, category.passed, category.total
        );

        for item in &category.items {
            let checkbox = match item.status {
                ValidationStatus::Passed => "[x]",
                ValidationStatus::Failed => "[ ] **FAILED**",
                ValidationStatus::Warning => "[ ] *warning*",
                ValidationStatus::Skipped => "[ ] *(skipped)*",
                ValidationStatus::Manual => "[ ] *(manual)*",
            };
            println!("- {} {}: {}", checkbox, item.id, item.description);
        }
        println!();
    }

    if !result.manual_checks_required.is_empty() {
        println!("## Manual Checks Required\n");
        for check in &result.manual_checks_required {
            println!("- [ ] {}", check);
        }
    }
}

/// Generate QA report for audit trail
async fn handle_report(
    task_id: &str,
    project_path: &Path,
    with_evidence: bool,
    output: Option<&Path>,
    format: QaOutputFormat,
) -> Result<()> {
    use crate::cli::colors as c;
    println!("{} {}", c::label("Generating QA report for task:"), task_id);

    // First run validation
    let mut result = QaValidationResult {
        task_id: task_id.to_string(),
        timestamp: Utc::now(),
        categories: HashMap::new(),
        overall_score: 0.0,
        passed: true,
        manual_checks_required: vec![],
    };

    result.categories.insert(
        "code_quality".into(),
        run_code_quality_checks(project_path).await,
    );
    result
        .categories
        .insert("testing".into(), run_testing_checks(project_path).await);
    result.categories.insert(
        "documentation".into(),
        run_documentation_checks(project_path, task_id).await,
    );
    result.categories.insert(
        "process".into(),
        run_process_checks(project_path, task_id).await,
    );

    // Calculate score
    let (total_passed, total_items) = result
        .categories
        .values()
        .fold((0, 0), |(p, t), cat| (p + cat.passed, t + cat.total));
    result.overall_score = if total_items > 0 {
        (total_passed as f64 / total_items as f64) * 100.0
    } else {
        0.0
    };

    // Generate report content
    let report = match format {
        QaOutputFormat::Json => serde_json::to_string_pretty(&result)?,
        QaOutputFormat::Yaml => serde_yaml_ng::to_string(&result)?,
        QaOutputFormat::Markdown | QaOutputFormat::Text => {
            let mut md = String::new();
            md.push_str(&format!("# QA Report: {}\n\n", task_id));
            md.push_str("## Summary\n\n");
            md.push_str(&format!("- **Task**: {}\n", task_id));
            md.push_str(&format!(
                "- **Status**: {}\n",
                if result.passed {
                    "PASSED"
                } else {
                    "NEEDS ATTENTION"
                }
            ));
            md.push_str(&format!("- **Score**: {:.1}%\n", result.overall_score));
            md.push_str(&format!(
                "- **Date**: {}\n\n",
                result.timestamp.format("%Y-%m-%d")
            ));

            md.push_str("## Checklist Results\n\n");
            for category in result.categories.values() {
                md.push_str(&format!(
                    "### {} ({}/{})\n\n",
                    category.name, category.passed, category.total
                ));
                for item in &category.items {
                    let status_icon = match item.status {
                        ValidationStatus::Passed => "✅",
                        ValidationStatus::Failed => "❌",
                        ValidationStatus::Warning => "⚠️",
                        ValidationStatus::Skipped => "⏭️",
                        ValidationStatus::Manual => "📝",
                    };
                    md.push_str(&format!(
                        "- {} **{}**: {}\n",
                        status_icon, item.id, item.description
                    ));
                }
                md.push('\n');
            }

            if with_evidence {
                md.push_str("## Evidence\n\n");
                md.push_str("- Coverage Report: `target/llvm-cov/html/index.html`\n");
                md.push_str("- Test Results: See CI/CD logs\n");
                md.push_str("- Complexity: Run `pmat analyze complexity`\n");
            }

            md
        }
    };

    // Output
    if let Some(output_path) = output {
        fs::write(output_path, &report)?;
        println!("{} Report saved to: {}", c::pass(""), c::path(&output_path.display().to_string()));
    } else {
        println!("{}", report);
    }

    Ok(())
}

/// Show QA status summary
async fn handle_summary(
    task_id: Option<&str>,
    project_path: &Path,
    epic_id: Option<&str>,
) -> anyhow::Result<()> {
    use crate::cli::colors as c;
    let qa_dir = project_path.join(".pmat-qa");

    if !qa_dir.exists() {
        println!("{}", c::dim("No QA data found. Run 'pmat qa-work generate-checklist <TASK-ID>' first."));
        return Ok(());
    }

    // Handle epic summary
    if let Some(epic) = epic_id {
        return handle_epic_summary(epic, &qa_dir);
    }

    println!("{}\n", c::header("QA Status Summary"));
    println!(
        "{:<15} {:<12} {:<10}",
        c::dim("Task ID"),
        c::dim("Status"),
        c::dim("Score")
    );
    println!("{}", c::separator());

    if let Some(id) = task_id {
        // Show specific task
        let task_dir = qa_dir.join(id);
        if task_dir.exists() {
            print_task_status(id, &task_dir)?;
        } else {
            println!("{} No QA data found for task: {}", c::warn(""), id);
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

#[cfg(test)]
mod print_tests {
    //! Covers print_validation_text + print_validation_markdown in
    //! qa_work_handler/impl_print.rs (178 uncov on broad, 0% cov).
    //! Skips async handle_report / handle_summary (require run_*_checks
    //! infrastructure + project fixtures).
    use super::*;

    fn item(id: &str, desc: &str, status: ValidationStatus) -> ValidationItem {
        ValidationItem {
            id: id.to_string(),
            description: desc.to_string(),
            status,
            value: None,
            threshold: None,
            evidence: None,
        }
    }

    fn category(name: &str, items: Vec<ValidationItem>) -> CategoryResult {
        let passed = items
            .iter()
            .filter(|i| i.status == ValidationStatus::Passed)
            .count() as u32;
        let total = items.len() as u32;
        CategoryResult {
            name: name.to_string(),
            passed,
            total,
            items,
        }
    }

    fn make_result(categories: HashMap<String, CategoryResult>) -> QaValidationResult {
        QaValidationResult {
            task_id: "PMAT-100".to_string(),
            timestamp: Utc::now(),
            categories,
            overall_score: 80.0,
            passed: true,
            manual_checks_required: vec![],
        }
    }

    // ── print_validation_text ──

    #[test]
    fn test_print_validation_text_empty_categories() {
        let r = make_result(HashMap::new());
        // No panic on empty.
        print_validation_text(&r);
    }

    #[test]
    fn test_print_validation_text_all_passed_category() {
        let mut cats = HashMap::new();
        cats.insert(
            "tests".to_string(),
            category(
                "Tests",
                vec![
                    item("T1", "passes", ValidationStatus::Passed),
                    item("T2", "passes", ValidationStatus::Passed),
                ],
            ),
        );
        print_validation_text(&make_result(cats));
    }

    #[test]
    fn test_print_validation_text_partial_passed_category() {
        let mut cats = HashMap::new();
        cats.insert(
            "tests".to_string(),
            category(
                "Tests",
                vec![
                    item("T1", "passes", ValidationStatus::Passed),
                    item("T2", "fails", ValidationStatus::Failed),
                ],
            ),
        );
        print_validation_text(&make_result(cats));
    }

    #[test]
    fn test_print_validation_text_all_failed_category() {
        let mut cats = HashMap::new();
        cats.insert(
            "tests".to_string(),
            category(
                "Tests",
                vec![
                    item("T1", "fails", ValidationStatus::Failed),
                    item("T2", "fails", ValidationStatus::Failed),
                ],
            ),
        );
        print_validation_text(&make_result(cats));
    }

    #[test]
    fn test_print_validation_text_all_5_validation_statuses() {
        let mut cats = HashMap::new();
        cats.insert(
            "all".to_string(),
            category(
                "All",
                vec![
                    item("P", "passed", ValidationStatus::Passed),
                    item("F", "failed", ValidationStatus::Failed),
                    item("W", "warning", ValidationStatus::Warning),
                    item("S", "skipped", ValidationStatus::Skipped),
                    item("M", "manual", ValidationStatus::Manual),
                ],
            ),
        );
        print_validation_text(&make_result(cats));
    }

    // ── print_validation_markdown ──

    #[test]
    fn test_print_validation_markdown_empty_no_panic() {
        let r = make_result(HashMap::new());
        print_validation_markdown(&r);
    }

    #[test]
    fn test_print_validation_markdown_with_all_5_status_arms() {
        let mut cats = HashMap::new();
        cats.insert(
            "tests".to_string(),
            category(
                "Tests",
                vec![
                    item("P", "passed", ValidationStatus::Passed),
                    item("F", "failed", ValidationStatus::Failed),
                    item("W", "warning", ValidationStatus::Warning),
                    item("S", "skipped", ValidationStatus::Skipped),
                    item("M", "manual", ValidationStatus::Manual),
                ],
            ),
        );
        print_validation_markdown(&make_result(cats));
    }

    #[test]
    fn test_print_validation_markdown_with_manual_checks() {
        let r = QaValidationResult {
            task_id: "X".to_string(),
            timestamp: Utc::now(),
            categories: HashMap::new(),
            overall_score: 0.0,
            passed: false,
            manual_checks_required: vec![
                "Review feature flag rollout".to_string(),
                "Verify production logs".to_string(),
            ],
        };
        print_validation_markdown(&r);
    }

    #[test]
    fn test_print_validation_markdown_no_manual_checks_section_when_empty() {
        let r = QaValidationResult {
            task_id: "X".to_string(),
            timestamp: Utc::now(),
            categories: HashMap::new(),
            overall_score: 100.0,
            passed: true,
            manual_checks_required: vec![],
        };
        // Empty manual_checks_required → "Manual Checks Required" section skipped.
        // Just verify no panic.
        print_validation_markdown(&r);
    }
}
