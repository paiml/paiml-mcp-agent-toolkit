#[cfg(test)]
mod tests {
    use crate::models::churn::{
        ChurnOutputFormat, ChurnSummary, CodeChurnAnalysis, FileChurnMetrics,
    };
    use crate::services::git_analysis::GitAnalysisService;
    use chrono::Utc;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_analysis() -> CodeChurnAnalysis {
        let files = vec![
            FileChurnMetrics {
                path: PathBuf::from("/test/repo/src/main.rs"),
                relative_path: "src/main.rs".to_string(),
                commit_count: 25,
                unique_authors: vec!["alice".to_string(), "bob".to_string()],
                additions: 300,
                deletions: 150,
                churn_score: 0.85,
                last_modified: Utc::now(),
                first_seen: Utc::now() - chrono::Duration::days(30),
            },
            FileChurnMetrics {
                path: PathBuf::from("/test/repo/src/lib.rs"),
                relative_path: "src/lib.rs".to_string(),
                commit_count: 5,
                unique_authors: vec!["alice".to_string()],
                additions: 50,
                deletions: 10,
                churn_score: 0.15,
                last_modified: Utc::now() - chrono::Duration::days(20),
                first_seen: Utc::now() - chrono::Duration::days(30),
            },
        ];

        let mut author_contributions = HashMap::new();
        author_contributions.insert("alice".to_string(), 2);
        author_contributions.insert("bob".to_string(), 1);

        let summary = ChurnSummary {
            total_commits: 30,
            total_files_changed: 2,
            hotspot_files: vec![files[0].path.clone()],
            stable_files: vec![files[1].path.clone()],
            author_contributions,
        };

        CodeChurnAnalysis {
            generated_at: Utc::now(),
            period_days: 30,
            repository_root: PathBuf::from("/test/repo"),
            files,
            summary,
        }
    }

    #[test]
    fn test_churn_score_calculation() {
        let mut metric = FileChurnMetrics {
            path: PathBuf::from("test.rs"),
            relative_path: "test.rs".to_string(),
            commit_count: 10,
            unique_authors: vec![],
            additions: 100,
            deletions: 50,
            churn_score: 0.0,
            last_modified: Utc::now(),
            first_seen: Utc::now(),
        };

        metric.calculate_churn_score(20, 300);
        assert!(metric.churn_score > 0.0 && metric.churn_score <= 1.0);
        assert!((metric.churn_score - 0.5).abs() < 0.01); // Expected: (10/20)*0.6 + (150/300)*0.4 = 0.5
    }

    #[test]
    fn test_output_format_parsing() {
        assert!(matches!(
            "json".parse::<ChurnOutputFormat>().unwrap(),
            ChurnOutputFormat::Json
        ));
        assert!(matches!(
            "markdown".parse::<ChurnOutputFormat>().unwrap(),
            ChurnOutputFormat::Markdown
        ));
        assert!(matches!(
            "csv".parse::<ChurnOutputFormat>().unwrap(),
            ChurnOutputFormat::Csv
        ));
        assert!(matches!(
            "summary".parse::<ChurnOutputFormat>().unwrap(),
            ChurnOutputFormat::Summary
        ));
        assert!("invalid".parse::<ChurnOutputFormat>().is_err());
    }

    #[test]
    fn test_format_churn_summary() {
        use crate::handlers::tools::format_churn_summary;

        let analysis = create_test_analysis();
        let summary = format_churn_summary(&analysis);

        assert!(summary.contains("Code Churn Analysis"));
        assert!(summary.contains("Period: 30 days"));
        assert!(summary.contains("Total files changed: 2"));
        assert!(summary.contains("Total commits: 30"));
        assert!(summary.contains("Hotspot Files"));
        assert!(summary.contains("src/main.rs"));
        assert!(summary.contains("Stable Files"));
        assert!(summary.contains("src/lib.rs"));
    }

    #[test]
    fn test_format_churn_markdown() {
        use crate::handlers::tools::format_churn_as_markdown;

        let analysis = create_test_analysis();
        let markdown = format_churn_as_markdown(&analysis);

        assert!(markdown.contains("# Code Churn Analysis Report"));
        assert!(markdown.contains("**Repository:**"));
        assert!(markdown.contains("## Summary"));
        assert!(markdown.contains("## Top 10 Files by Churn Score"));
        assert!(markdown.contains("| File | Commits | Changes | Churn Score | Authors |"));
        assert!(markdown.contains("src/main.rs"));
        assert!(markdown.contains("| 25 |"));
        assert!(markdown.contains("| 0.85 |"));
    }

    #[test]
    fn test_format_churn_csv() {
        use crate::handlers::tools::format_churn_as_csv;

        let analysis = create_test_analysis();
        let csv = format_churn_as_csv(&analysis);

        assert!(csv.contains(
            "file_path,commits,additions,deletions,churn_score,unique_authors,last_modified"
        ));
        assert!(csv.contains("src/main.rs,25,300,150,0.850,2,"));
        assert!(csv.contains("src/lib.rs,5,50,10,0.150,1,"));
    }

    #[test]
    fn test_no_git_repository_error() {
        let temp_dir = TempDir::new().unwrap();
        let result = GitAnalysisService::analyze_code_churn(temp_dir.path(), 30);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No git repository found"));
    }

    #[test]
    fn test_parse_commit_line() {
        // Access private method through test module
        use crate::services::git_analysis::GitAnalysisService;

        // We need to test the parsing logic by creating a mock git repo
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // Initialize git repo
        std::process::Command::new("git")
            .arg("init")
            .current_dir(repo_path)
            .output()
            .expect("Failed to init git repo");

        // Configure git user
        std::process::Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to configure git email");

        std::process::Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to configure git name");

        // Create a test file
        std::fs::write(repo_path.join("test.txt"), "Hello").unwrap();

        // Add and commit
        std::process::Command::new("git")
            .args(["add", "test.txt"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to add file");

        std::process::Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to commit");

        // Now test analyze_code_churn on a real git repo
        let result = GitAnalysisService::analyze_code_churn(repo_path, 30);
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert_eq!(analysis.period_days, 30);
        assert_eq!(analysis.files.len(), 1);
        assert_eq!(analysis.files[0].relative_path, "test.txt");
        assert_eq!(analysis.files[0].commit_count, 1);
        assert!(analysis.files[0]
            .unique_authors
            .contains(&"Test User".to_string()));
    }

    #[test]
    fn test_multiple_commits_and_files() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // Initialize git repo
        std::process::Command::new("git")
            .arg("init")
            .current_dir(repo_path)
            .output()
            .expect("Failed to init git repo");

        // Configure git user
        std::process::Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to configure git email");

        std::process::Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to configure git name");

        // Create multiple files with multiple commits
        std::fs::write(repo_path.join("file1.txt"), "Content1").unwrap();
        std::process::Command::new("git")
            .args(["add", "file1.txt"])
            .current_dir(repo_path)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "Add file1"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        // Modify file1 and add file2
        std::fs::write(repo_path.join("file1.txt"), "Content1 Modified").unwrap();
        std::fs::write(repo_path.join("file2.txt"), "Content2").unwrap();
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(repo_path)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "Modify file1 and add file2"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        // Test analysis
        let result = GitAnalysisService::analyze_code_churn(repo_path, 30);
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert_eq!(analysis.summary.total_files_changed, 2);
        assert_eq!(analysis.summary.total_commits, 3); // file1 has 2 commits, file2 has 1

        // Find file1 - should have 2 commits
        let file1 = analysis
            .files
            .iter()
            .find(|f| f.relative_path == "file1.txt");
        assert!(file1.is_some());
        assert_eq!(file1.unwrap().commit_count, 2);

        // Find file2 - should have 1 commit
        let file2 = analysis
            .files
            .iter()
            .find(|f| f.relative_path == "file2.txt");
        assert!(file2.is_some());
        assert_eq!(file2.unwrap().commit_count, 1);
    }

    #[test]
    fn test_churn_score_edge_cases() {
        // Test with zero max values
        let mut metric = FileChurnMetrics {
            path: PathBuf::from("test.rs"),
            relative_path: "test.rs".to_string(),
            commit_count: 0,
            unique_authors: vec![],
            additions: 0,
            deletions: 0,
            churn_score: 0.0,
            last_modified: Utc::now(),
            first_seen: Utc::now(),
        };

        metric.calculate_churn_score(0, 0);
        assert_eq!(metric.churn_score, 0.0);

        // Test with non-zero values but zero max
        metric.commit_count = 5;
        metric.additions = 100;
        metric.deletions = 50;
        metric.calculate_churn_score(0, 0);
        assert_eq!(metric.churn_score, 0.0); // Both factors are 0 when max is 0
    }

    #[test]
    fn test_empty_repository() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // Initialize empty git repo
        std::process::Command::new("git")
            .arg("init")
            .current_dir(repo_path)
            .output()
            .expect("Failed to init git repo");

        // Test analysis on empty repo
        let result = GitAnalysisService::analyze_code_churn(repo_path, 30);
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert_eq!(analysis.files.len(), 0);
        assert_eq!(analysis.summary.total_files_changed, 0);
        assert_eq!(analysis.summary.total_commits, 0);
        assert!(analysis.summary.hotspot_files.is_empty());
        assert!(analysis.summary.stable_files.is_empty());
    }
}
