//! Integration tests for the defect report service

#[cfg(test)]
mod tests {
    use crate::models::defect_report::{Defect, DefectCategory, Severity};
    use crate::services::defect_report_service::{DefectReportService, ReportFormat};
    use std::collections::HashMap;
    use std::path::PathBuf;
    use tempfile::TempDir;
    use tokio;

    /// Create a test project with some source files
    async fn create_test_project() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        tokio::fs::create_dir_all(&src_dir).await.unwrap();

        // Initialize git repository
        std::process::Command::new("git")
            .arg("init")
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to initialize git repository");

        // Configure git user (required for commits)
        std::process::Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to configure git user email");

        std::process::Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to configure git user name");

        // Create some test files
        let test_files = vec![
            (
                "src/main.rs",
                r#"
fn main() {
    println!("Hello, world!");
    // TODO: Add proper error handling
    let x = complex_function();
}

fn complex_function() -> i32 {
    let mut sum = 0;
    for i in 0..100 {
        for j in 0..100 {
            for k in 0..100 {
                sum += i * j * k;
            }
        }
    }
    sum
}

fn dead_function() {
    println!("This is never called");
}
"#,
            ),
            (
                "src/lib.rs",
                r#"
pub fn duplicate_code() {
    let mut sum = 0;
    for i in 0..100 {
        for j in 0..100 {
            for k in 0..100 {
                sum += i * j * k;
            }
        }
    }
    println!("{}", sum);
}

// FIXME: This is a security vulnerability
pub fn unsafe_function(input: &str) {
    // TODO: Validate input
    println!("{}", input);
}
"#,
            ),
        ];

        for (path, content) in test_files {
            let full_path = temp_dir.path().join(path);
            tokio::fs::write(&full_path, content).await.unwrap();
        }

        // Add files to git and make initial commit
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to add files to git");

        std::process::Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to create initial commit");

        temp_dir
    }

    #[tokio::test]
    async fn test_defect_report_generation() {
        let test_project = create_test_project().await;
        let service = DefectReportService::new();

        // Change to test directory before analysis
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(test_project.path()).unwrap();
        let report = service.generate_report(test_project.path()).await.unwrap();
        let _ = std::env::set_current_dir(original_dir); // Ignore errors on cleanup

        // Basic assertions
        assert!(report.metadata.total_files_analyzed > 0);
        assert_eq!(report.metadata.tool, "pmat");
        assert!(!report.metadata.version.is_empty());
    }

    #[tokio::test]
    async fn test_json_formatting() {
        let test_project = create_test_project().await;
        let service = DefectReportService::new();

        // Change to test directory before analysis
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(test_project.path()).unwrap();
        let report = service.generate_report(test_project.path()).await.unwrap();
        let _ = std::env::set_current_dir(original_dir); // Ignore errors on cleanup

        let json_output = service.format_json(&report).unwrap();

        // Parse JSON to verify it's valid
        let parsed: serde_json::Value = serde_json::from_str(&json_output).unwrap();

        // Check structure
        assert!(parsed["metadata"].is_object());
        assert!(parsed["defects"].is_array());
        assert!(parsed["summary"].is_object());
        assert!(parsed["file_index"].is_object());
    }

    #[tokio::test]
    async fn test_csv_formatting() {
        let test_project = create_test_project().await;
        let service = DefectReportService::new();

        // Change to test directory before analysis
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(test_project.path()).unwrap();
        let report = service.generate_report(test_project.path()).await.unwrap();
        let _ = std::env::set_current_dir(original_dir); // Ignore errors on cleanup

        let csv_output = service.format_csv(&report).unwrap();

        // Check CSV has headers
        let lines: Vec<&str> = csv_output.lines().collect();
        assert!(!lines.is_empty());
        assert!(lines[0].contains("severity"));
        assert!(lines[0].contains("category"));
        assert!(lines[0].contains("file_path"));
    }

    #[tokio::test]
    async fn test_markdown_formatting() {
        let test_project = create_test_project().await;
        let service = DefectReportService::new();

        // Change to test directory before analysis
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(test_project.path()).unwrap();
        let report = service.generate_report(test_project.path()).await.unwrap();
        let _ = std::env::set_current_dir(original_dir); // Ignore errors on cleanup

        let md_output = service.format_markdown(&report).unwrap();

        // Check markdown structure
        assert!(md_output.contains("# Code Quality Report"));
        assert!(md_output.contains("## Executive Summary"));
        assert!(md_output.contains("### Severity Distribution"));
    }

    #[tokio::test]
    async fn test_text_formatting() {
        let test_project = create_test_project().await;
        let service = DefectReportService::new();

        // Change to test directory before analysis
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(test_project.path()).unwrap();
        let report = service.generate_report(test_project.path()).await.unwrap();
        let _ = std::env::set_current_dir(original_dir); // Ignore errors on cleanup

        let text_output = service.format_text(&report).unwrap();

        // Check text structure
        assert!(text_output.contains("CODE QUALITY REPORT"));
        assert!(text_output.contains("SEVERITY BREAKDOWN"));
        assert!(text_output.contains("CATEGORY BREAKDOWN"));
    }

    #[tokio::test]
    async fn test_filename_generation() {
        let service = DefectReportService::new();

        let json_name = service.generate_filename(ReportFormat::Json);
        assert!(json_name.starts_with("defect-report-"));
        assert!(json_name.ends_with(".json"));

        let csv_name = service.generate_filename(ReportFormat::Csv);
        assert!(csv_name.ends_with(".csv"));

        let md_name = service.generate_filename(ReportFormat::Markdown);
        assert!(md_name.ends_with(".md"));

        let txt_name = service.generate_filename(ReportFormat::Text);
        assert!(txt_name.ends_with(".txt"));
    }

    #[tokio::test]
    async fn test_empty_project() {
        let empty_dir = TempDir::new().unwrap();
        let service = DefectReportService::new();

        let report = service.generate_report(empty_dir.path()).await.unwrap();

        assert_eq!(report.defects.len(), 0);
        assert_eq!(report.summary.total_defects, 0);
        assert_eq!(report.file_index.len(), 0);
    }

    #[test]
    fn test_defect_severity_weight() {
        let defect = Defect {
            id: "TEST-001".to_string(),
            severity: Severity::Critical,
            category: DefectCategory::Complexity,
            file_path: PathBuf::from("test.rs"),
            line_start: 1,
            line_end: None,
            column_start: None,
            column_end: None,
            message: "Test".to_string(),
            rule_id: "test".to_string(),
            fix_suggestion: None,
            metrics: HashMap::new(),
        };

        assert_eq!(defect.severity_weight(), 10.0);

        let high_defect = Defect {
            severity: Severity::High,
            ..defect.clone()
        };
        assert_eq!(high_defect.severity_weight(), 5.0);

        let medium_defect = Defect {
            severity: Severity::Medium,
            ..defect.clone()
        };
        assert_eq!(medium_defect.severity_weight(), 3.0);

        let low_defect = Defect {
            severity: Severity::Low,
            ..defect.clone()
        };
        assert_eq!(low_defect.severity_weight(), 1.0);
    }

    #[test]
    fn test_defect_category_all() {
        let categories = DefectCategory::all();
        assert_eq!(categories.len(), 7);
        assert!(categories.contains(&DefectCategory::Complexity));
        assert!(categories.contains(&DefectCategory::TechnicalDebt));
        assert!(categories.contains(&DefectCategory::DeadCode));
        assert!(categories.contains(&DefectCategory::Duplication));
        assert!(categories.contains(&DefectCategory::Performance));
        assert!(categories.contains(&DefectCategory::Architecture));
        assert!(categories.contains(&DefectCategory::TestCoverage));
    }
}
