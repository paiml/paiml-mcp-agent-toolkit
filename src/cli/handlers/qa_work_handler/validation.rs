//! QA Work Handler - Validation and Reporting
//!
//! Orchestrates validation checks, report generation, and output formatting.
//! Implementation split across:
//! - validation_checks.rs: Code quality, testing, documentation, and process checks
//! - validation_reporting.rs: Text/markdown output and report generation

#![cfg_attr(coverage_nightly, coverage(off))]
use super::qa_work_handler_types::*;
use crate::cli::commands::QaOutputFormat;
use chrono::Utc;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Run automated QA validation
pub async fn handle_validate(
    task_id: &str,
    project_path: &Path,
    strict: bool,
    format: QaOutputFormat,
) -> anyhow::Result<()> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    debug_assert!(!task_id.is_empty(), "task_id must not be empty");
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
        QaOutputFormat::Yaml => println!("{}", serde_yaml_ng::to_string(&result)?),
        QaOutputFormat::Markdown => print_validation_markdown(&result),
    }

    if !result.passed {
        std::process::exit(1);
    }

    Ok(())
}

// Validation check implementations (code quality, testing, docs, process)
include!("validation_checks.rs");

// Reporting and output formatting (text, markdown, report generation)
include!("validation_reporting.rs");
