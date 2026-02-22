//! QA Work Handler - Checklist Generation
//!
//! Part 2: Checklist generation and formatting functions

#![cfg_attr(coverage_nightly, coverage(off))]
use super::qa_work_handler_types::*;
use crate::cli::commands::QaTaskType;
use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};

/// Generate a QA checklist for a task
pub async fn handle_generate_checklist(
    task_id: &str,
    task_type: QaTaskType,
    project_path: &Path,
    output: Option<&Path>,
) -> anyhow::Result<()> {
    println!("Generating QA checklist for task: {}", task_id);

    let checklist = generate_checklist(task_id, task_type);

    // Ensure .pmat-qa directory exists
    let qa_dir = project_path.join(".pmat-qa").join(task_id);
    fs::create_dir_all(&qa_dir)?;

    // Output path
    let output_path = output
        .map(PathBuf::from)
        .unwrap_or_else(|| qa_dir.join("checklist.yaml"));

    // Write checklist
    let yaml = serde_yaml_ng::to_string(&checklist)?;
    fs::write(&output_path, &yaml)?;

    println!("\n{}", format_checklist_text(&checklist));
    println!("\nChecklist saved to: {}", output_path.display());

    Ok(())
}

/// Generate checklist based on task type
pub fn generate_checklist(task_id: &str, task_type: QaTaskType) -> QaChecklist {
    let type_str = match task_type {
        QaTaskType::Feature => "feature",
        QaTaskType::Bugfix => "bugfix",
        QaTaskType::Refactor => "refactor",
        QaTaskType::Docs => "docs",
        QaTaskType::Performance => "performance",
        QaTaskType::Security => "security",
    };

    QaChecklist {
        task_id: task_id.to_string(),
        task_type: type_str.to_string(),
        generated: Utc::now(),
        categories: ChecklistCategories {
            safety_ethics: vec![
                ChecklistItem {
                    id: "A1".into(),
                    description: "No hardcoded secrets or credentials".into(),
                    checked: false,
                    automated: true,
                    evidence: None,
                },
                ChecklistItem {
                    id: "A2".into(),
                    description: "Error handling covers all failure modes".into(),
                    checked: false,
                    automated: false,
                    evidence: None,
                },
                ChecklistItem {
                    id: "A3".into(),
                    description: "Input validation prevents injection attacks".into(),
                    checked: false,
                    automated: true,
                    evidence: None,
                },
                ChecklistItem {
                    id: "A4".into(),
                    description: "Logging doesn't expose sensitive data".into(),
                    checked: false,
                    automated: true,
                    evidence: None,
                },
                ChecklistItem {
                    id: "A5".into(),
                    description: "Rate limiting considered for APIs".into(),
                    checked: false,
                    automated: false,
                    evidence: None,
                },
            ],
            code_quality: vec![
                ChecklistItem {
                    id: "B1".into(),
                    description: "Cyclomatic complexity <= 10".into(),
                    checked: false,
                    automated: true,
                    evidence: None,
                },
                ChecklistItem {
                    id: "B2".into(),
                    description: "Cognitive complexity <= 15".into(),
                    checked: false,
                    automated: true,
                    evidence: None,
                },
                ChecklistItem {
                    id: "B3".into(),
                    description: "Test coverage >= 95%".into(),
                    checked: false,
                    automated: true,
                    evidence: None,
                },
                ChecklistItem {
                    id: "B4".into(),
                    description: "Mutation score >= 80%".into(),
                    checked: false,
                    automated: true,
                    evidence: None,
                },
                ChecklistItem {
                    id: "B5".into(),
                    description: "No new clippy warnings".into(),
                    checked: false,
                    automated: true,
                    evidence: None,
                },
            ],
            testing: vec![
                ChecklistItem {
                    id: "C1".into(),
                    description: "Unit tests cover happy path".into(),
                    checked: false,
                    automated: false,
                    evidence: None,
                },
                ChecklistItem {
                    id: "C2".into(),
                    description: "Unit tests cover error paths".into(),
                    checked: false,
                    automated: false,
                    evidence: None,
                },
                ChecklistItem {
                    id: "C3".into(),
                    description: "Property tests for complex logic".into(),
                    checked: false,
                    automated: false,
                    evidence: None,
                },
                ChecklistItem {
                    id: "C4".into(),
                    description: "Integration tests for API boundaries".into(),
                    checked: false,
                    automated: false,
                    evidence: None,
                },
                ChecklistItem {
                    id: "C5".into(),
                    description: "Golden tests for output formats".into(),
                    checked: false,
                    automated: false,
                    evidence: None,
                },
            ],
            documentation: vec![
                ChecklistItem {
                    id: "D1".into(),
                    description: "Public API documented".into(),
                    checked: false,
                    automated: true,
                    evidence: None,
                },
                ChecklistItem {
                    id: "D2".into(),
                    description: "Examples provided in docs".into(),
                    checked: false,
                    automated: false,
                    evidence: None,
                },
                ChecklistItem {
                    id: "D3".into(),
                    description: "CHANGELOG updated".into(),
                    checked: false,
                    automated: true,
                    evidence: None,
                },
                ChecklistItem {
                    id: "D4".into(),
                    description: "README reflects changes".into(),
                    checked: false,
                    automated: false,
                    evidence: None,
                },
                ChecklistItem {
                    id: "D5".into(),
                    description: "Error messages are actionable".into(),
                    checked: false,
                    automated: false,
                    evidence: None,
                },
            ],
            process: vec![
                ChecklistItem {
                    id: "E1".into(),
                    description: "All acceptance criteria met".into(),
                    checked: false,
                    automated: false,
                    evidence: None,
                },
                ChecklistItem {
                    id: "E2".into(),
                    description: "Commit messages reference ticket".into(),
                    checked: false,
                    automated: true,
                    evidence: None,
                },
                ChecklistItem {
                    id: "E3".into(),
                    description: "PR description complete".into(),
                    checked: false,
                    automated: false,
                    evidence: None,
                },
                ChecklistItem {
                    id: "E4".into(),
                    description: "CI/CD passes all gates".into(),
                    checked: false,
                    automated: true,
                    evidence: None,
                },
                ChecklistItem {
                    id: "E5".into(),
                    description: "Peer review completed".into(),
                    checked: false,
                    automated: false,
                    evidence: None,
                },
            ],
        },
    }
}

/// Format checklist for text display
pub fn format_checklist_text(checklist: &QaChecklist) -> String {
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
