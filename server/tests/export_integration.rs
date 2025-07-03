#[cfg(test)]
mod export_integration_tests {
    use anyhow::Result;
    use chrono::Utc;
    use pmat::demo::export::*;
    use pmat::demo::Hotspot;
    use pmat::models::churn::{ChurnSummary, CodeChurnAnalysis, FileChurnMetrics};
    use pmat::models::dag::{DependencyGraph, Edge, EdgeType, NodeInfo, NodeType};
    use pmat::services::complexity::{
        ComplexityMetrics, ComplexityReport, ComplexitySummary, FileComplexityMetrics,
        FunctionComplexity,
    };
    use pmat::services::deep_context::{CacheStats, ContextMetadata};
    use rustc_hash::FxHashMap;
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn create_test_export_report() -> ExportReport {
        let mut mermaid_graphs = HashMap::new();
        mermaid_graphs.insert(
            "main".to_string(),
            "graph TD\n    A --> B\n    B --> C".to_string(),
        );

        // Use a fixed timestamp for reproducible tests
        let fixed_timestamp =
            chrono::TimeZone::with_ymd_and_hms(&Utc, 2025, 1, 1, 12, 0, 0).unwrap();

        ExportReport {
            repository: "test-repo".to_string(),
            timestamp: fixed_timestamp,
            metadata: ContextMetadata {
                generated_at: Utc::now(),
                tool_version: "1.0.0".to_string(),
                project_root: std::path::PathBuf::from("test-repo"),
                cache_stats: CacheStats {
                    hit_rate: 0.0,
                    memory_efficiency: 0.0,
                    time_saved_ms: 0,
                },
                analysis_duration: std::time::Duration::from_millis(1500),
            },
            ast_contexts: vec![],
            dependency_graph: DependencyGraph::new(),
            complexity_analysis: ComplexityAnalysis {
                hotspots: vec![
                    Hotspot {
                        file: "main.rs::main".to_string(),
                        complexity: 15,
                        churn_score: 20,
                    },
                    Hotspot {
                        file: "lib.rs::process".to_string(),
                        complexity: 12,
                        churn_score: 18,
                    },
                ],
                total_files: 10,
                average_complexity: 5.5,
                technical_debt_hours: 8,
            },
            churn_analysis: Some(create_test_churn_analysis()),
            satd_analysis: None,
            dead_code_results: None,
            cross_references: vec![],
            quality_scorecard: None,
            defect_summary: None,
            mermaid_graphs,
            summary: ProjectSummary {
                total_nodes: 15,
                total_edges: 22,
                analyzed_files: 10,
                analysis_time_ms: 1500,
            },
        }
    }

    fn create_test_churn_analysis() -> CodeChurnAnalysis {
        CodeChurnAnalysis {
            generated_at: Utc::now(),
            period_days: 30,
            repository_root: std::path::PathBuf::from("test-repo"),
            files: vec![
                FileChurnMetrics {
                    path: std::path::PathBuf::from("src/main.rs"),
                    relative_path: "src/main.rs".to_string(),
                    commit_count: 45,
                    unique_authors: vec![],
                    additions: 100,
                    deletions: 50,
                    churn_score: 0.85,
                    last_modified: Utc::now(),
                    first_seen: Utc::now(),
                },
                FileChurnMetrics {
                    path: std::path::PathBuf::from("src/lib.rs"),
                    relative_path: "src/lib.rs".to_string(),
                    commit_count: 38,
                    unique_authors: vec![],
                    additions: 80,
                    deletions: 30,
                    churn_score: 0.72,
                    last_modified: Utc::now(),
                    first_seen: Utc::now(),
                },
            ],
            summary: ChurnSummary {
                total_commits: 83,
                total_files_changed: 2,
                hotspot_files: vec![],
                stable_files: vec![],
                author_contributions: HashMap::new(),
            },
        }
    }

    #[test]
    fn test_markdown_export() {
        let report = create_test_export_report();
        let exporter = MarkdownExporter;

        let result = exporter.export(&report).unwrap();

        // Verify structure
        assert!(result.contains("# Analysis: test-repo"));
        assert!(result.contains("Generated: 2025-01-01 12:00 UTC"));
        assert!(result.contains("## Summary"));
        assert!(result.contains("**Analyzed Files**: 10"));
        assert!(result.contains("**Average Complexity**: 5.50"));
        assert!(result.contains("**Technical Debt**: 8 hours"));
        assert!(result.contains("**Analysis Time**: 1500ms"));

        // Verify dependency graph
        assert!(result.contains("## Dependency Graph"));
        assert!(result.contains("```mermaid"));
        assert!(result.contains("graph TD"));
        assert!(result.contains("A --> B"));
        assert!(result.contains("B --> C"));

        // Verify complexity hotspots
        assert!(result.contains("## Complexity Hotspots"));
        assert!(result.contains("| main.rs::main | 15 | 20 |"));
        assert!(result.contains("| lib.rs::process | 12 | 18 |"));

        // Verify churn analysis
        assert!(result.contains("## Code Churn Analysis (Last 30 Days)"));
        assert!(result.contains("| src/main.rs | 0.85 | 45 |"));
        assert!(result.contains("| src/lib.rs | 0.72 | 38 |"));

        // Verify raw data section
        assert!(result.contains("<details>"));
        assert!(result.contains("<summary>Raw Data</summary>"));
        assert!(result.contains("```json"));
    }

    #[test]
    fn test_json_export_pretty() {
        let report = create_test_export_report();
        let exporter = JsonExporter::new(true);

        let result = exporter.export(&report).unwrap();

        // Verify pretty printing
        assert!(result.contains('\n'));
        assert!(result.contains("  ")); // Indentation

        // Parse JSON to verify structure
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["repository"], "test-repo");
        assert_eq!(parsed["complexity_analysis"]["total_files"], 10);
        assert_eq!(parsed["complexity_analysis"]["average_complexity"], 5.5);
        assert_eq!(parsed["summary"]["total_nodes"], 15);
    }

    #[test]
    fn test_json_export_compact() {
        let report = create_test_export_report();
        let exporter = JsonExporter::new(false);

        let result = exporter.export(&report).unwrap();

        // Verify compact format (no newlines except in strings)
        let newline_count = result.matches('\n').count();
        assert_eq!(newline_count, 0);

        // Still valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["repository"], "test-repo");
    }

    #[test]
    fn test_sarif_export() {
        let report = create_test_export_report();
        let exporter = SarifExporter;

        let result = exporter.export(&report).unwrap();

        // Parse SARIF
        let sarif: serde_json::Value = serde_json::from_str(&result).unwrap();

        // Verify SARIF structure
        assert_eq!(sarif["version"], "2.1.0");
        assert!(sarif["$schema"]
            .as_str()
            .unwrap()
            .contains("sarif-schema-2.1.0.json"));

        // Verify tool information
        let driver = &sarif["runs"][0]["tool"]["driver"];
        assert_eq!(driver["name"], "pmat");
        assert!(driver["informationUri"]
            .as_str()
            .unwrap()
            .contains("github.com"));

        // Verify rules
        let rules = driver["rules"].as_array().unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0]["id"], "COMPLEXITY001");
        assert_eq!(rules[0]["name"], "HighComplexity");

        // Verify results (only functions with complexity > 10 should be included)
        let results = sarif["runs"][0]["results"].as_array().unwrap();
        assert_eq!(results.len(), 2); // Both hotspots have complexity > 10

        // Check first result
        assert_eq!(results[0]["ruleId"], "COMPLEXITY001");
        assert_eq!(results[0]["level"], "warning"); // Complexity 15 is warning
        assert!(results[0]["message"]["text"]
            .as_str()
            .unwrap()
            .contains("main.rs::main"));
        assert!(results[0]["message"]["text"]
            .as_str()
            .unwrap()
            .contains("complexity of 15"));
    }

    #[test]
    fn test_export_service() {
        let service = ExportService::new();
        let report = create_test_export_report();

        // Test supported formats
        let formats = service.supported_formats();
        assert!(formats.contains(&"markdown"));
        assert!(formats.contains(&"json"));
        assert!(formats.contains(&"sarif"));

        // Test each format
        let markdown_result = service.export("markdown", &report).unwrap();
        assert!(markdown_result.contains("# Analysis: test-repo"));

        let json_result = service.export("json", &report).unwrap();
        let _: serde_json::Value = serde_json::from_str(&json_result).unwrap();

        let sarif_result = service.export("sarif", &report).unwrap();
        let sarif: serde_json::Value = serde_json::from_str(&sarif_result).unwrap();
        assert_eq!(sarif["version"], "2.1.0");

        // Test unsupported format
        let unsupported_result = service.export("xml", &report);
        assert!(unsupported_result.is_err());
    }

    #[test]
    fn test_export_service_save_to_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let service = ExportService::new();
        let report = create_test_export_report();

        // Save as markdown
        let md_path = temp_dir.path().join("report.md");
        service.save_to_file("markdown", &report, &md_path)?;
        assert!(md_path.exists());
        let md_content = std::fs::read_to_string(&md_path)?;
        assert!(md_content.contains("# Analysis: test-repo"));

        // Save as JSON
        let json_path = temp_dir.path().join("report.json");
        service.save_to_file("json", &report, &json_path)?;
        assert!(json_path.exists());
        let json_content = std::fs::read_to_string(&json_path)?;
        let _: serde_json::Value = serde_json::from_str(&json_content)?;

        // Save as SARIF
        let sarif_path = temp_dir.path().join("report.sarif");
        service.save_to_file("sarif", &report, &sarif_path)?;
        assert!(sarif_path.exists());

        Ok(())
    }

    #[test]
    fn test_create_export_report() {
        // Create test data
        let mut nodes = FxHashMap::default();
        nodes.insert(
            "node1".to_string(),
            NodeInfo {
                id: "node1".to_string(),
                label: "Module A".to_string(),
                node_type: NodeType::Module,
                file_path: "src/module_a.rs".to_string(),
                line_number: 1,
                complexity: 5,
                metadata: FxHashMap::default(),
            },
        );
        nodes.insert(
            "node2".to_string(),
            NodeInfo {
                id: "node2".to_string(),
                label: "Module B".to_string(),
                node_type: NodeType::Module,
                file_path: "src/module_b.rs".to_string(),
                line_number: 1,
                complexity: 8,
                metadata: FxHashMap::default(),
            },
        );

        let edges = vec![Edge {
            from: "node1".to_string(),
            to: "node2".to_string(),
            edge_type: EdgeType::Imports,
            weight: 1,
        }];

        let dag = DependencyGraph { nodes, edges };

        let complexity_report = ComplexityReport {
            summary: ComplexitySummary {
                total_files: 2,
                total_functions: 4,
                median_cyclomatic: 5.0,
                median_cognitive: 7.0,
                max_cyclomatic: 15,
                max_cognitive: 20,
                p90_cyclomatic: 12,
                p90_cognitive: 18,
                technical_debt_hours: 4.5,
            },
            violations: vec![],
            hotspots: vec![],
            files: vec![FileComplexityMetrics {
                path: "src/main.rs".to_string(),
                total_complexity: ComplexityMetrics {
                    cyclomatic: 15,
                    cognitive: 20,
                    nesting_max: 3,
                    lines: 50,
                },
                functions: vec![FunctionComplexity {
                    name: "main".to_string(),
                    line_start: 1,
                    line_end: 50,
                    metrics: ComplexityMetrics {
                        cyclomatic: 15,
                        cognitive: 20,
                        nesting_max: 3,
                        lines: 50,
                    },
                }],
                classes: vec![],
            }],
        };

        let churn_analysis = CodeChurnAnalysis {
            generated_at: Utc::now(),
            period_days: 30,
            repository_root: std::path::PathBuf::from("/test/repo"),
            files: vec![FileChurnMetrics {
                path: std::path::PathBuf::from("/test/repo/src/main.rs"),
                relative_path: "src/main.rs".to_string(),
                commit_count: 25,
                unique_authors: vec![
                    "author1".to_string(),
                    "author2".to_string(),
                    "author3".to_string(),
                ],
                additions: 500,
                deletions: 200,
                churn_score: 0.75,
                last_modified: Utc::now(),
                first_seen: Utc::now(),
            }],
            summary: ChurnSummary {
                total_commits: 100,
                total_files_changed: 20,
                hotspot_files: vec![std::path::PathBuf::from("src/main.rs")],
                stable_files: vec![std::path::PathBuf::from("src/config.rs")],
                author_contributions: HashMap::new(),
            },
        };

        let report = create_export_report(
            "test-repo",
            &dag,
            Some(&complexity_report),
            Some(&churn_analysis),
            "graph TD\n    A --> B",
            1234,
        );

        assert_eq!(report.repository, "test-repo");
        assert_eq!(
            report.mermaid_graphs.get("main").unwrap(),
            "graph TD\n    A --> B"
        );
        assert_eq!(report.complexity_analysis.total_files, 1);
        assert_eq!(report.complexity_analysis.average_complexity, 5.0);
        assert_eq!(report.complexity_analysis.technical_debt_hours, 4);
        assert_eq!(report.summary.total_nodes, 2);
        assert_eq!(report.summary.total_edges, 1);
        assert_eq!(report.summary.analysis_time_ms, 1234);

        // Check hotspots
        assert_eq!(report.complexity_analysis.hotspots.len(), 1);
        assert_eq!(
            report.complexity_analysis.hotspots[0].file,
            "src/main.rs::main"
        );
        assert_eq!(report.complexity_analysis.hotspots[0].complexity, 15);

        // Check churn
        assert!(report.churn_analysis.is_some());
        let churn = report.churn_analysis.unwrap();
        assert_eq!(churn.period_days, 30);
        assert_eq!(churn.files.len(), 1);
        assert_eq!(churn.files[0].relative_path, "src/main.rs");
        assert_eq!(churn.files[0].churn_score, 0.75);
    }

    #[test]
    fn test_export_without_optional_data() {
        let dag = DependencyGraph {
            nodes: FxHashMap::default(),
            edges: vec![],
        };

        let report = create_export_report("minimal-repo", &dag, None, None, "graph TD", 100);

        assert_eq!(report.repository, "minimal-repo");
        assert_eq!(report.complexity_analysis.hotspots.len(), 0);
        assert_eq!(report.complexity_analysis.total_files, 0);
        assert_eq!(report.complexity_analysis.average_complexity, 0.0);
        assert!(report.churn_analysis.is_none());
        assert_eq!(report.summary.analyzed_files, 0);
    }
}
