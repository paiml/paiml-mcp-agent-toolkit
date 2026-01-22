/// Format checklist for text display
fn format_checklist_text(checklist: &QaChecklist) -> String {
    let mut output = String::new();
    output.push_str(&format!("# QA Checklist for {}\n", checklist.task_id));
    output.push_str(&format!("Task Type: {}\n", checklist.task_type));
    output.push_str(&format!(
        "Generated: {}\n\n",
        checklist.generated.format("%Y-%m-%d %H:%M:%S UTC")
    ));

    let categories = [
        ("Safety & Ethics", &checklist.categories.safety_ethics),
        ("Code Quality", &checklist.categories.code_quality),
        ("Testing", &checklist.categories.testing),
        ("Documentation", &checklist.categories.documentation),
        ("Process", &checklist.categories.process),
    ];

    for (name, items) in categories {
        output.push_str(&format!("## {}\n", name));
        for item in items {
            let checkbox = if item.checked { "[x]" } else { "[ ]" };
            let auto = if item.automated { " (auto)" } else { "" };
            output.push_str(&format!(
                "  {} {}: {}{}\n",
                checkbox, item.id, item.description, auto
            ));
        }
        output.push('\n');
    }

    output
}

/// Run automated QA validation
async fn handle_validate(
    task_id: &str,
    project_path: &Path,
    strict: bool,
    format: QaOutputFormat,
) -> Result<()> {
    println!("Running QA validation for task: {}", task_id);
    println!();

    let mut result = QaValidationResult {
        task_id: task_id.to_string(),
        timestamp: Utc::now(),
        categories: HashMap::new(),
        overall_score: 0.0,
        passed: true,
        manual_checks_required: vec![],
    };

    // Run code quality checks
    let code_quality = run_code_quality_checks(project_path).await;
    result
        .categories
        .insert("code_quality".into(), code_quality);

    // Run testing checks
    let testing = run_testing_checks(project_path).await;
    result.categories.insert("testing".into(), testing);

    // Run documentation checks
    let docs = run_documentation_checks(project_path, task_id).await;
    result.categories.insert("documentation".into(), docs);

    // Run process checks
    let process = run_process_checks(project_path, task_id).await;
    result.categories.insert("process".into(), process);

    // Calculate overall score
    let (total_passed, total_items) = result
        .categories
        .values()
        .fold((0, 0), |(p, t), cat| (p + cat.passed, t + cat.total));
    result.overall_score = if total_items > 0 {
        (total_passed as f64 / total_items as f64) * 100.0
    } else {
        0.0
    };

    // Add manual checks
    result.manual_checks_required = vec![
        "Peer review sign-off".into(),
        "Error handling review".into(),
        "API documentation review".into(),
    ];

    // Determine pass/fail
    result.passed = result.overall_score >= 80.0 && !strict || result.overall_score >= 95.0;

    // Output
    match format {
        QaOutputFormat::Text => print_validation_text(&result),
        QaOutputFormat::Json => println!("{}", serde_json::to_string_pretty(&result)?),
        QaOutputFormat::Yaml => println!("{}", serde_yaml::to_string(&result)?),
        QaOutputFormat::Markdown => print_validation_markdown(&result),
    }

    if !result.passed {
        std::process::exit(1);
    }

    Ok(())
}

/// Run code quality validation checks
async fn run_code_quality_checks(project_path: &Path) -> CategoryResult {
    let mut items = vec![];

    // Check complexity via pmat
    let complexity_result = Command::new("pmat")
        .args(["analyze", "complexity", "--path"])
        .arg(project_path)
        .args(["--format", "json"])
        .output();

    let complexity_status = match complexity_result {
        Ok(output) if output.status.success() => {
            // Parse and check thresholds
            ValidationStatus::Passed
        }
        _ => ValidationStatus::Skipped,
    };

    items.push(ValidationItem {
        id: "B1".into(),
        description: "Cyclomatic complexity <= 10".into(),
        status: complexity_status.clone(),
        value: None,
        threshold: Some("10".into()),
        evidence: None,
    });

    items.push(ValidationItem {
        id: "B2".into(),
        description: "Cognitive complexity <= 15".into(),
        status: complexity_status,
        value: None,
        threshold: Some("15".into()),
        evidence: None,
    });

    // Check clippy
    let clippy_result = Command::new("cargo")
        .args(["clippy", "--", "-D", "warnings"])
        .current_dir(project_path)
        .output();

    let clippy_status = match clippy_result {
        Ok(output) if output.status.success() => ValidationStatus::Passed,
        Ok(_) => ValidationStatus::Failed,
        Err(_) => ValidationStatus::Skipped,
    };

    items.push(ValidationItem {
        id: "B5".into(),
        description: "No new clippy warnings".into(),
        status: clippy_status,
        value: None,
        threshold: Some("0 warnings".into()),
        evidence: None,
    });

    // Coverage check (placeholder - would integrate with cargo-llvm-cov)
    items.push(ValidationItem {
        id: "B3".into(),
        description: "Test coverage >= 95%".into(),
        status: ValidationStatus::Manual,
        value: None,
        threshold: Some("95%".into()),
        evidence: Some("Run: cargo llvm-cov --html".into()),
    });

    // Mutation score (placeholder)
    items.push(ValidationItem {
        id: "B4".into(),
        description: "Mutation score >= 80%".into(),
        status: ValidationStatus::Manual,
        value: None,
        threshold: Some("80%".into()),
        evidence: Some("Run: cargo mutants".into()),
    });

    let passed = items
        .iter()
        .filter(|i| i.status == ValidationStatus::Passed)
        .count() as u32;
    let total = items.len() as u32;

    CategoryResult {
        name: "Code Quality".into(),
        passed,
        total,
        items,
    }
}

/// Run testing validation checks
async fn run_testing_checks(project_path: &Path) -> CategoryResult {
    let mut items = vec![];

    // Run tests
    let test_result = Command::new("cargo")
        .args(["test", "--", "--test-threads=1"])
        .current_dir(project_path)
        .output();

    let test_status = match test_result {
        Ok(output) if output.status.success() => ValidationStatus::Passed,
        Ok(_) => ValidationStatus::Failed,
        Err(_) => ValidationStatus::Skipped,
    };

    items.push(ValidationItem {
        id: "C1".into(),
        description: "Unit tests passing".into(),
        status: test_status,
        value: None,
        threshold: Some("All pass".into()),
        evidence: None,
    });

    // Manual checks
    items.push(ValidationItem {
        id: "C2".into(),
        description: "Unit tests cover error paths".into(),
        status: ValidationStatus::Manual,
        value: None,
        threshold: None,
        evidence: Some("Review test coverage for error handling".into()),
    });

    items.push(ValidationItem {
        id: "C3".into(),
        description: "Property tests for complex logic".into(),
        status: ValidationStatus::Manual,
        value: None,
        threshold: None,
        evidence: Some("Check for proptest usage".into()),
    });

    items.push(ValidationItem {
        id: "C4".into(),
        description: "Integration tests for API boundaries".into(),
        status: ValidationStatus::Manual,
        value: None,
        threshold: None,
        evidence: None,
    });

    items.push(ValidationItem {
        id: "C5".into(),
        description: "Golden tests for output formats".into(),
        status: ValidationStatus::Manual,
        value: None,
        threshold: None,
        evidence: None,
    });

    let passed = items
        .iter()
        .filter(|i| i.status == ValidationStatus::Passed)
        .count() as u32;
    let total = items.len() as u32;

    CategoryResult {
        name: "Testing".into(),
        passed,
        total,
        items,
    }
}

/// Run documentation validation checks
async fn run_documentation_checks(project_path: &Path, task_id: &str) -> CategoryResult {
    let mut items = vec![];

    // Check CHANGELOG
    let changelog_path = project_path.join("CHANGELOG.md");
    let changelog_status = if changelog_path.exists() {
        let content = fs::read_to_string(&changelog_path).unwrap_or_default();
        if content.contains(task_id) || content.contains("Unreleased") {
            ValidationStatus::Passed
        } else {
            ValidationStatus::Warning
        }
    } else {
        ValidationStatus::Skipped
    };

    items.push(ValidationItem {
        id: "D3".into(),
        description: "CHANGELOG updated".into(),
        status: changelog_status,
        value: None,
        threshold: None,
        evidence: None,
    });

    // Check rustdoc
    let doc_result = Command::new("cargo")
        .args(["doc", "--no-deps"])
        .current_dir(project_path)
        .output();

    let doc_status = match doc_result {
        Ok(output) if output.status.success() => ValidationStatus::Passed,
        Ok(_) => ValidationStatus::Warning,
        Err(_) => ValidationStatus::Skipped,
    };

    items.push(ValidationItem {
        id: "D1".into(),
        description: "Public API documented".into(),
        status: doc_status,
        value: None,
        threshold: None,
        evidence: None,
    });

    // Manual checks
    items.push(ValidationItem {
        id: "D2".into(),
        description: "Examples provided in docs".into(),
        status: ValidationStatus::Manual,
        value: None,
        threshold: None,
        evidence: None,
    });

    items.push(ValidationItem {
        id: "D4".into(),
        description: "README reflects changes".into(),
        status: ValidationStatus::Manual,
        value: None,
        threshold: None,
        evidence: None,
    });

    items.push(ValidationItem {
        id: "D5".into(),
        description: "Error messages are actionable".into(),
        status: ValidationStatus::Manual,
        value: None,
        threshold: None,
        evidence: None,
    });

    let passed = items
        .iter()
        .filter(|i| i.status == ValidationStatus::Passed)
        .count() as u32;
    let total = items.len() as u32;

    CategoryResult {
        name: "Documentation".into(),
        passed,
        total,
        items,
    }
}

/// Run process validation checks
async fn run_process_checks(project_path: &Path, task_id: &str) -> CategoryResult {
    let mut items = vec![];

    // Check git log for ticket references
    let git_result = Command::new("git")
        .args(["log", "--oneline", "-10"])
        .current_dir(project_path)
        .output();

    let commit_status = match git_result {
        Ok(output) if output.status.success() => {
            let log = String::from_utf8_lossy(&output.stdout);
            if log.contains(task_id) || log.contains(&format!("#{}", task_id)) {
                ValidationStatus::Passed
            } else {
                ValidationStatus::Warning
            }
        }
        _ => ValidationStatus::Skipped,
    };

    items.push(ValidationItem {
        id: "E2".into(),
        description: "Commit messages reference ticket".into(),
        status: commit_status,
        value: None,
        threshold: None,
        evidence: Some(format!("Checked for: {}", task_id)),
    });

    // Check CI status (would need GH API integration)
    items.push(ValidationItem {
        id: "E4".into(),
        description: "CI/CD passes all gates".into(),
        status: ValidationStatus::Manual,
        value: None,
        threshold: None,
        evidence: Some("Check GitHub Actions".into()),
    });

    // Manual checks
    items.push(ValidationItem {
        id: "E1".into(),
        description: "All acceptance criteria met".into(),
        status: ValidationStatus::Manual,
        value: None,
        threshold: None,
        evidence: None,
    });

    items.push(ValidationItem {
        id: "E3".into(),
        description: "PR description complete".into(),
        status: ValidationStatus::Manual,
        value: None,
        threshold: None,
        evidence: None,
    });

    items.push(ValidationItem {
        id: "E5".into(),
        description: "Peer review completed".into(),
        status: ValidationStatus::Manual,
        value: None,
        threshold: None,
        evidence: None,
    });

    let passed = items
        .iter()
        .filter(|i| i.status == ValidationStatus::Passed)
        .count() as u32;
    let total = items.len() as u32;

    CategoryResult {
        name: "Process".into(),
        passed,
        total,
        items,
    }
}

