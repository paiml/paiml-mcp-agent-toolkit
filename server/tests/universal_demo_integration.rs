//! Integration tests for Universal Demo "Just Works" functionality
//!
//! These tests verify that pmat can analyze any GitHub repository
//! regardless of language and provide meaningful results.

#[cfg(test)]
mod universal_demo_tests {
    use anyhow::Result;
    use pmat::demo::runner::resolve_repository_async;
    use pmat::services::deep_context::{DeepContextAnalyzer, DeepContextConfig};
    use pmat::services::quality_gates::VerificationStatus;
    use std::path::PathBuf;

    /// Test repository info
    struct TestRepo {
        url: &'static str,
        language: &'static str,
        min_files: usize,
        has_functions: bool,
        expected_quality: VerificationStatus,
    }

    /// Get a list of diverse test repositories
    fn get_test_repos() -> Vec<TestRepo> {
        vec![
            TestRepo {
                url: "https://github.com/serde-rs/json",
                language: "Rust",
                min_files: 10,
                has_functions: true,
                expected_quality: VerificationStatus::Pass,
            },
            TestRepo {
                url: "https://github.com/pallets/flask",
                language: "Python",
                min_files: 50,
                has_functions: true,
                expected_quality: VerificationStatus::Pass,
            },
            TestRepo {
                url: "https://github.com/expressjs/express",
                language: "JavaScript",
                min_files: 30,
                has_functions: true,
                expected_quality: VerificationStatus::Pass,
            },
            TestRepo {
                url: "https://github.com/microsoft/TypeScript",
                language: "TypeScript",
                min_files: 100,
                has_functions: true,
                expected_quality: VerificationStatus::Pass,
            },
            TestRepo {
                url: "https://github.com/golang/example",
                language: "Go",
                min_files: 5,
                has_functions: true,
                expected_quality: VerificationStatus::Pass,
            },
        ]
    }

    #[tokio::test]
    #[ignore] // Ignore by default as it requires network access
    async fn test_rust_repository_analysis() -> Result<()> {
        let repo = &get_test_repos()[0]; // Rust repo
        test_repository_analysis(repo).await
    }

    #[tokio::test]
    #[ignore] // Ignore by default as it requires network access
    async fn test_python_repository_analysis() -> Result<()> {
        let repo = &get_test_repos()[1]; // Python repo
        test_repository_analysis(repo).await
    }

    #[tokio::test]
    #[ignore] // Ignore by default as it requires network access
    async fn test_javascript_repository_analysis() -> Result<()> {
        let repo = &get_test_repos()[2]; // JavaScript repo
        test_repository_analysis(repo).await
    }

    #[tokio::test]
    #[ignore] // Ignore by default as it requires network access
    async fn test_typescript_repository_analysis() -> Result<()> {
        let repo = &get_test_repos()[3]; // TypeScript repo
        test_repository_analysis(repo).await
    }

    #[tokio::test]
    #[ignore] // Ignore by default as it requires network access
    async fn test_go_repository_analysis() -> Result<()> {
        let repo = &get_test_repos()[4]; // Go repo
        test_repository_analysis(repo).await
    }

    /// Common test logic for any repository
    async fn test_repository_analysis(repo: &TestRepo) -> Result<()> {
        println!("Testing {} repository: {}", repo.language, repo.url);

        // Clone the repository
        let repo_path = resolve_repository_async(None, Some(repo.url.to_string()), None).await?;

        assert!(repo_path.exists(), "Repository should be cloned");

        // Analyze the repository
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        let result = analyzer.analyze_project(&repo_path).await?;

        // Verify basic metadata
        assert!(result.metadata.project_root.exists());
        assert!(
            result.metadata.analysis_duration.as_secs() < 300,
            "Analysis should complete within 5 minutes"
        );

        // Verify file discovery
        let file_count = count_files_in_tree(&result.file_tree.root);
        assert!(
            file_count >= repo.min_files,
            "Should discover at least {} files, found {}",
            repo.min_files,
            file_count
        );

        // Verify quality gates
        assert!(
            result.qa_verification.is_some(),
            "Should have QA verification"
        );
        let qa = result.qa_verification.unwrap();

        // Check overall status matches expectation
        assert_eq!(
            qa.overall, repo.expected_quality,
            "Quality gate status should be {:?} for {} repo",
            repo.expected_quality, repo.language
        );

        // Verify complexity analysis (if applicable)
        if repo.has_functions && repo.language == "Rust" {
            assert!(
                result.analyses.complexity_report.is_some(),
                "Should have complexity report for {} repo",
                repo.language
            );
        }

        // Verify SATD analysis works
        assert!(
            result.analyses.satd_results.is_some(),
            "Should have SATD analysis results"
        );

        // Clean up
        if repo_path.starts_with("/tmp") {
            let _ = std::fs::remove_dir_all(&repo_path);
        }

        println!("✅ {} repository test passed", repo.language);
        Ok(())
    }

    fn count_files_in_tree(node: &pmat::services::deep_context::AnnotatedNode) -> usize {
        use pmat::services::deep_context::NodeType;

        match node.node_type {
            NodeType::File => 1,
            NodeType::Directory => node
                .children
                .iter()
                .map(|child| count_files_in_tree(child))
                .sum(),
        }
    }

    #[tokio::test]
    async fn test_quality_gate_edge_cases() -> Result<()> {
        use chrono::Utc;
        use pmat::services::deep_context::{
            AnalysisResults, CacheStats, ContextMetadata, DeepContextResult, DefectSummary,
            QualityScorecard,
        };
        use pmat::services::quality_gates::QAVerification;
        use std::time::Duration;

        // Create a minimal result with no analysis
        let minimal_result = DeepContextResult {
            metadata: ContextMetadata {
                generated_at: Utc::now(),
                tool_version: env!("CARGO_PKG_VERSION").to_string(),
                project_root: PathBuf::from("/tmp/test"),
                cache_stats: CacheStats {
                    hit_rate: 0.0,
                    memory_efficiency: 0.0,
                    time_saved_ms: 0,
                },
                analysis_duration: Duration::from_secs(1),
            },
            file_tree: vec!["test.py".to_string()],
            analyses: AnalysisResults {
                complexity_report: None,
                churn_analysis: None,
                dependency_graph: None,
                dead_code_results: None,
                satd_results: None,
                duplicate_code_results: None,
                provability_results: None,
                cross_language_refs: vec![],
                big_o_analysis: None,
                ast_contexts: vec![],
            },
            quality_scorecard: QualityScorecard {
                overall_health: 75.0,
                complexity_score: 80.0,
                maintainability_index: 70.0,
                modularity_score: 85.0,
                test_coverage: Some(0.0),
                technical_debt_hours: 0.0,
            },
            template_provenance: None,
            defect_summary: DefectSummary {
                total_defects: 0,
                by_severity: Default::default(),
                by_type: Default::default(),
                defect_density: 0.0,
            },
            hotspots: vec![],
            recommendations: vec![],
            qa_verification: None,
            complexity_metrics: None,
            dead_code_analysis: None,
            ast_summaries: None,
            churn_analysis: None,
            language_stats: None,
            build_info: None,
            project_overview: None,
        };

        // Verify quality gates handle minimal results gracefully
        let qa = QAVerification::new();
        let verification = qa.verify(&minimal_result);

        // Should not panic and should provide some result
        assert!(verification.contains_key("dead_code_sanity"));

        Ok(())
    }

    #[tokio::test]
    async fn test_repository_cloning_errors() -> Result<()> {
        // Test invalid URL
        let result = resolve_repository_async(
            None,
            Some("https://github.com/nonexistent/repo-that-does-not-exist".to_string()),
            None,
        )
        .await;

        // Should handle gracefully (might succeed with empty repo or fail with clear error)
        match result {
            Ok(path) => {
                // If it somehow succeeds, path should exist
                assert!(path.exists() || path.to_string_lossy().contains("nonexistent"));
            }
            Err(e) => {
                // Error message should be informative
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("clone")
                        || error_msg.contains("repository")
                        || error_msg.contains("not found"),
                    "Error should be descriptive: {}",
                    error_msg
                );
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_mixed_language_repository() -> Result<()> {
        // Test a repository with multiple languages
        // Using a small repo that likely has mixed content
        let repo_url = "https://github.com/github/gitignore";

        let repo_path = resolve_repository_async(None, Some(repo_url.to_string()), None).await?;

        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        let result = analyzer.analyze_project(&repo_path).await?;

        // Should handle mixed content without errors
        assert!(result.qa_verification.is_some());

        // Clean up
        if repo_path.starts_with("/tmp") {
            let _ = std::fs::remove_dir_all(&repo_path);
        }

        Ok(())
    }

    #[test]
    fn test_verification_status_handling() {
        // Ensure all verification statuses are handled properly
        assert_eq!(VerificationStatus::Pass, VerificationStatus::Pass);
        assert_ne!(VerificationStatus::Pass, VerificationStatus::Fail);
        assert_ne!(VerificationStatus::Pass, VerificationStatus::Partial);
    }
}
