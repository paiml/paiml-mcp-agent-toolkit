// Tests for export module.
// Included from export.rs — no `use` imports or inner attributes allowed.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_export() {
        let mut mermaid_graphs = HashMap::new();
        mermaid_graphs.insert("main".to_string(), "graph TD\n    A --> B".to_string());

        let report = ExportReport {
            repository: "test-repo".to_string(),
            timestamp: Utc::now(),
            metadata: ContextMetadata {
                generated_at: Utc::now(),
                tool_version: "1.0.0".to_string(),
                project_root: std::path::PathBuf::from("test-repo"),
                cache_stats: crate::services::deep_context::CacheStats {
                    hit_rate: 0.0,
                    memory_efficiency: 0.0,
                    time_saved_ms: 0,
                },
                analysis_duration: std::time::Duration::from_millis(1500),
            },
            ast_contexts: vec![],
            dependency_graph: DependencyGraph::new(),
            complexity_analysis: ComplexityAnalysis {
                hotspots: vec![Hotspot {
                    file: "main.rs::main".to_string(),
                    complexity: 15,
                    churn_score: 20,
                }],
                total_files: 10,
                average_complexity: 5.5,
                technical_debt_hours: 8,
            },
            churn_analysis: None,
            satd_analysis: None,
            dead_code_results: None,
            cross_references: vec![],
            quality_scorecard: None,
            defect_summary: None,
            mermaid_graphs,
            summary: ProjectSummary {
                total_nodes: 5,
                total_edges: 4,
                analyzed_files: 10,
                analysis_time_ms: 1500,
            },
        };

        let exporter = MarkdownExporter;
        let result = exporter.export(&report).unwrap();

        assert!(result.contains("# Analysis: test-repo"));
        assert!(result.contains("```mermaid"));
        assert!(result.contains("graph TD"));
        assert!(result.contains("main.rs::main"));
        assert!(result.contains("| 15 |"));
    }

    #[test]
    fn test_json_export() {
        let mut mermaid_graphs = HashMap::new();
        mermaid_graphs.insert("main".to_string(), "graph TD".to_string());

        let report = ExportReport {
            repository: "test-repo".to_string(),
            timestamp: Utc::now(),
            metadata: ContextMetadata {
                generated_at: Utc::now(),
                tool_version: "1.0.0".to_string(),
                project_root: std::path::PathBuf::from("test-repo"),
                cache_stats: crate::services::deep_context::CacheStats {
                    hit_rate: 0.0,
                    memory_efficiency: 0.0,
                    time_saved_ms: 0,
                },
                analysis_duration: std::time::Duration::from_millis(100),
            },
            ast_contexts: vec![],
            dependency_graph: DependencyGraph::new(),
            complexity_analysis: ComplexityAnalysis {
                hotspots: vec![],
                total_files: 0,
                average_complexity: 0.0,
                technical_debt_hours: 0,
            },
            churn_analysis: None,
            satd_analysis: None,
            dead_code_results: None,
            cross_references: vec![],
            quality_scorecard: None,
            defect_summary: None,
            mermaid_graphs,
            summary: ProjectSummary {
                total_nodes: 0,
                total_edges: 0,
                analyzed_files: 0,
                analysis_time_ms: 100,
            },
        };

        let exporter = JsonExporter::new(false);
        let result = exporter.export(&report).unwrap();

        assert!(result.contains("\"repository\":\"test-repo\""));
        assert!(!result.contains('\n')); // Not pretty printed
    }

    #[test]
    fn test_export_service() {
        let service = ExportService::new();
        let formats = service.supported_formats();

        assert!(formats.contains(&"markdown"));
        assert!(formats.contains(&"json"));
        assert!(formats.contains(&"sarif"));
    }
}

