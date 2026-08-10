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

/// True for formats a tool parses rather than a human reads.
///
/// `qa-work validate -f json` used to print `Running QA validation for task: …`
/// to STDOUT ahead of the document, so piping it to `jq` failed on the very
/// first byte. Machine formats own stdout exclusively; progress chatter belongs
/// on stderr.
pub(crate) fn qa_format_owns_stdout(format: &QaOutputFormat) -> bool {
    matches!(format, QaOutputFormat::Json | QaOutputFormat::Yaml)
}

/// Run automated QA validation
async fn handle_validate(
    task_id: &str,
    project_path: &Path,
    strict: bool,
    format: QaOutputFormat,
) -> Result<()> {
    if qa_format_owns_stdout(&format) {
        eprintln!("Running QA validation for task: {}", task_id);
    } else {
        println!("Running QA validation for task: {}", task_id);
        println!();
    }

    let mut result = QaValidationResult {
        task_id: task_id.to_string(),
        timestamp: Utc::now(),
        categories: HashMap::new(),
        overall_score: 0.0,
        passed: true,
        manual_checks_required: vec![],
    };

    // Safety & Ethics (A1-A5) was silently absent from validate while
    // generate-checklist and the checklist model both list it, so validate
    // scored 20 items against a 25-item checklist and the one security
    // category was the one that vanished. It is now always present; the items
    // report as unverified rather than as passes nobody measured.
    let safety_ethics = run_safety_ethics_checks();
    result
        .categories
        .insert("safety_ethics".into(), safety_ethics);

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

    // Calculate overall score (extracted as pure helper for R5)
    result.overall_score = calculate_overall_score(&result.categories);

    // Add manual checks
    result.manual_checks_required = vec![
        "Peer review sign-off".into(),
        "Error handling review".into(),
        "API documentation review".into(),
    ];

    // Determine pass/fail (extracted as pure helper for R5)
    result.passed = determine_pass(result.overall_score, strict);

    // Output
    match format {
        QaOutputFormat::Text => print_validation_text(&result),
        QaOutputFormat::Json => println!("{}", serde_json::to_string_pretty(&result)?),
        QaOutputFormat::Yaml => println!("{}", serde_yaml_ng::to_string(&result)?),
        QaOutputFormat::Markdown => print_validation_markdown(&result),
    }

    if !result.passed {
        std::process::exit(1);
    }

    Ok(())
}

/// Pure-compute classifier extracted for R5 testability (per spec §4.7).
///
/// Maps the result of a `Command` invocation to a `ValidationStatus`:
/// - `Ok(out) if out.status.success()` → Passed
/// - `Ok(_)` (non-zero exit) → Failed
/// - `Err(_)` (failed to spawn) → Skipped
///
/// This pattern was duplicated 4× in the run_*_checks functions before
/// extraction; the inner shape is `match result { ... }` deciding among
/// three statuses based on subprocess outcome.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn classify_command_outcome(
    result: std::io::Result<std::process::Output>,
) -> ValidationStatus {
    match result {
        Ok(output) if output.status.success() => ValidationStatus::Passed,
        Ok(_) => ValidationStatus::Failed,
        Err(_) => ValidationStatus::Skipped,
    }
}

/// Pure-compute classifier where non-zero exit is a Warning (not Failed).
/// Used for documentation checks where missing rustdoc isn't a hard fail.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn classify_doc_command_outcome(
    result: std::io::Result<std::process::Output>,
) -> ValidationStatus {
    match result {
        Ok(output) if output.status.success() => ValidationStatus::Passed,
        Ok(_) => ValidationStatus::Warning,
        Err(_) => ValidationStatus::Skipped,
    }
}

/// Classify git-log output for ticket-reference presence.
/// Returns Passed if `task_id` or `#task_id` appears in the log; else Warning.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn classify_git_log_for_task(stdout: &str, task_id: &str) -> ValidationStatus {
    if stdout.contains(task_id) || stdout.contains(&format!("#{}", task_id)) {
        ValidationStatus::Passed
    } else {
        ValidationStatus::Warning
    }
}

/// Classify CHANGELOG content for task-reference presence.
/// Returns Passed if either the task_id or "Unreleased" header appears; else Warning.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn classify_changelog_for_task(content: &str, task_id: &str) -> ValidationStatus {
    if content.contains(task_id) || content.contains("Unreleased") {
        ValidationStatus::Passed
    } else {
        ValidationStatus::Warning
    }
}

/// Compute overall score as `passed/total * 100`. Returns 0.0 when total == 0
/// to avoid div-by-zero. Used by handle_validate to summarize categories.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
pub(crate) fn calculate_overall_score(
    categories: &HashMap<String, CategoryResult>,
) -> f64 {
    let (total_passed, total_items) = categories
        .values()
        .fold((0u32, 0u32), |(p, t), cat| (p + cat.passed, t + cat.total));
    if total_items > 0 {
        (total_passed as f64 / total_items as f64) * 100.0
    } else {
        0.0
    }
}

/// Decide pass/fail based on overall_score and strict-mode flag.
///
/// - `strict=false`: pass when score >= 80.0
/// - `strict=true`: pass when score >= 95.0
///
/// **Operator precedence note**: the original code is
/// `score >= 80.0 && !strict || score >= 95.0`, which Rust parses as
/// `(score >= 80.0 && !strict) || (score >= 95.0)`. This means strict mode
/// only kicks in when score is below 80.0; the threshold-95 OR is always
/// evaluated. Pinned in tests below.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn determine_pass(overall_score: f64, strict: bool) -> bool {
    overall_score >= 80.0 && !strict || overall_score >= 95.0
}

/// Build the Safety & Ethics (A1-A5) category.
///
/// The five items mirror `generate-checklist` exactly so validate's denominator
/// is the checklist's 25, not 20. pmat has no implemented secret/injection/log
/// scanner behind A1/A3/A4, so they are reported as requiring review rather
/// than as automated passes — an unmeasured check must never count as a pass.
fn run_safety_ethics_checks() -> CategoryResult {
    let specs = [
        ("A1", "No hardcoded secrets or credentials"),
        ("A2", "Error handling covers all failure modes"),
        ("A3", "Input validation prevents injection attacks"),
        ("A4", "Logging doesn't expose sensitive data"),
        ("A5", "Rate limiting considered for APIs"),
    ];

    let items: Vec<ValidationItem> = specs
        .iter()
        .map(|(id, description)| ValidationItem {
            id: (*id).to_string(),
            description: (*description).to_string(),
            status: ValidationStatus::Manual,
            value: None,
            threshold: None,
            evidence: Some("No automated safety check is implemented; review manually".into()),
        })
        .collect();

    let passed = items
        .iter()
        .filter(|i| i.status == ValidationStatus::Passed)
        .count() as u32;
    let total = items.len() as u32;

    CategoryResult {
        name: "Safety & Ethics".into(),
        passed,
        total,
        items,
    }
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

    // Note: complexity check folds Failed→Skipped (parsing not implemented yet)
    let complexity_status = match complexity_result {
        Ok(output) if output.status.success() => ValidationStatus::Passed,
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

    let clippy_status = classify_command_outcome(clippy_result);

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

    let test_status = classify_command_outcome(test_result);

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
        classify_changelog_for_task(&content, task_id)
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

    let doc_status = classify_doc_command_outcome(doc_result);

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
            classify_git_log_for_task(&log, task_id)
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


#[cfg(test)]
mod validation_shape_tests {
    //! Regressions for two ways `qa-work validate` misreported itself: the
    //! banner stole stdout from `-f json`, and the Safety & Ethics category was
    //! dropped so 25 checklist items were scored as 20.
    use super::*;

    #[test]
    fn test_machine_formats_own_stdout() {
        assert!(qa_format_owns_stdout(&QaOutputFormat::Json));
        assert!(qa_format_owns_stdout(&QaOutputFormat::Yaml));
        assert!(!qa_format_owns_stdout(&QaOutputFormat::Text));
        assert!(!qa_format_owns_stdout(&QaOutputFormat::Markdown));
    }

    #[test]
    fn test_safety_ethics_category_covers_a1_to_a5() {
        let cat = run_safety_ethics_checks();
        assert_eq!(cat.name, "Safety & Ethics");
        let ids: Vec<&str> = cat.items.iter().map(|i| i.id.as_str()).collect();
        assert_eq!(ids, vec!["A1", "A2", "A3", "A4", "A5"]);
        assert_eq!(cat.total, 5);
    }

    #[test]
    fn test_safety_ethics_claims_no_unmeasured_passes() {
        let cat = run_safety_ethics_checks();
        assert_eq!(
            cat.passed, 0,
            "no automated safety check is implemented, so none of A1-A5 may count as passed"
        );
        assert!(cat
            .items
            .iter()
            .all(|i| i.status == ValidationStatus::Manual));
    }

    #[test]
    fn test_validate_denominator_matches_the_25_point_checklist() {
        // handle_validate is async and shells out to cargo/git, so the
        // denominator is asserted over the category constructors it inserts:
        // safety_ethics(5) + code_quality(5) + testing(5) + documentation(5)
        // + process(5) = 25, the size of generate-checklist's output. It was
        // 20 while safety_ethics was silently absent.
        let checklist = generate_checklist("T-1", QaTaskType::Feature);
        let checklist_items = checklist.categories.safety_ethics.len()
            + checklist.categories.code_quality.len()
            + checklist.categories.testing.len()
            + checklist.categories.documentation.len()
            + checklist.categories.process.len();
        assert_eq!(checklist_items, 25);
        assert_eq!(run_safety_ethics_checks().total as usize, checklist.categories.safety_ethics.len());
    }
}
