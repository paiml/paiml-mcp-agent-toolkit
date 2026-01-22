/// Print validation result as text
fn print_validation_text(result: &QaValidationResult) {
    println!("Validating {}...\n", result.task_id);

    for category in result.categories.values() {
        let status = if category.passed == category.total {
            "\x1b[32m✓\x1b[0m"
        } else if category.passed > 0 {
            "\x1b[33m⚠\x1b[0m"
        } else {
            "\x1b[31m✗\x1b[0m"
        };

        println!(
            "{} {} ({}/{})",
            status, category.name, category.passed, category.total
        );

        for item in &category.items {
            let item_status = match item.status {
                ValidationStatus::Passed => "  \x1b[32m✓\x1b[0m",
                ValidationStatus::Failed => "  \x1b[31m✗\x1b[0m",
                ValidationStatus::Warning => "  \x1b[33m⚠\x1b[0m",
                ValidationStatus::Skipped => "  \x1b[90m-\x1b[0m",
                ValidationStatus::Manual => "  \x1b[34m?\x1b[0m",
            };
            println!("{}  {}: {}", item_status, item.id, item.description);
        }
        println!();
    }

    println!("Overall Score: {:.1}%", result.overall_score);
    println!();

    if !result.manual_checks_required.is_empty() {
        println!("\x1b[33mManual Checks Required:\x1b[0m");
        for check in &result.manual_checks_required {
            println!("  - {}", check);
        }
        println!();
    }

    if result.passed {
        println!("\x1b[32m✓ QA Validation PASSED\x1b[0m");
    } else {
        println!("\x1b[31m✗ QA Validation FAILED\x1b[0m");
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
    println!("Generating QA report for task: {}", task_id);

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
        QaOutputFormat::Yaml => serde_yaml::to_string(&result)?,
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
        println!("Report saved to: {}", output_path.display());
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
) -> Result<()> {
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
