// Data structure construction and validation tests for unified protocol service
// Included via include!() from service_tests.rs

    // === Template Data Structures ===

    #[test]
    fn test_list_templates_query_default() {
        let query = ListTemplatesQuery {
            format: None,
            category: None,
        };
        assert!(query.format.is_none());
        assert!(query.category.is_none());
    }

    #[test]
    fn test_list_templates_query_with_values() {
        let query = ListTemplatesQuery {
            format: Some("json".to_string()),
            category: Some("cli".to_string()),
        };
        assert_eq!(query.format, Some("json".to_string()));
        assert_eq!(query.category, Some("cli".to_string()));
    }

    #[test]
    fn test_template_list_creation() {
        let list = TemplateList {
            templates: vec![TemplateInfo {
                id: "test/template".to_string(),
                name: "Test Template".to_string(),
                description: "A test template".to_string(),
                version: "1.0.0".to_string(),
                parameters: vec![],
            }],
            total: 1,
        };
        assert_eq!(list.total, 1);
        assert_eq!(list.templates.len(), 1);
        assert_eq!(list.templates[0].id, "test/template");
    }

    #[test]
    fn test_template_parameter_creation() {
        let param = TemplateParameter {
            name: "project_name".to_string(),
            description: "The name of the project".to_string(),
            required: true,
            default_value: Some(Value::String("my-project".to_string())),
        };
        assert!(param.required);
        assert_eq!(param.name, "project_name");
    }

    #[test]
    fn test_generate_params_creation() {
        let mut params = HashMap::new();
        params.insert("key".to_string(), Value::String("value".to_string()));

        let generate_params = GenerateParams {
            template_uri: "template://rust/cli".to_string(),
            parameters: params,
        };
        assert_eq!(generate_params.template_uri, "template://rust/cli");
        assert!(generate_params.parameters.contains_key("key"));
    }

    #[test]
    fn test_generated_template_creation() {
        let template = GeneratedTemplate {
            template_id: "rust/cli".to_string(),
            content: "# Makefile\nall:\n\techo 'build'".to_string(),
            metadata: TemplateMetadata {
                name: "Generated".to_string(),
                version: "1.0.0".to_string(),
                generated_at: "2025-01-09T00:00:00Z".to_string(),
            },
        };
        assert_eq!(template.template_id, "rust/cli");
        assert!(template.content.contains("Makefile"));
    }

    // === Complexity Analysis Data Structures ===

    #[test]
    fn test_complexity_params_creation() {
        let params = ComplexityParams {
            project_path: "/test/path".to_string(),
            toolchain: "rust".to_string(),
            format: "json".to_string(),
            max_cyclomatic: Some(20),
            max_cognitive: Some(15),
            top_files: Some(10),
        };
        assert_eq!(params.project_path, "/test/path");
        assert_eq!(params.max_cyclomatic, Some(20));
    }

    #[test]
    fn test_complexity_query_params_defaults() {
        let params = ComplexityQueryParams {
            project_path: None,
            toolchain: None,
            format: None,
            max_cyclomatic: None,
            max_cognitive: None,
            top_files: None,
        };
        assert!(params.project_path.is_none());
        assert!(params.toolchain.is_none());
    }

    #[test]
    fn test_complexity_analysis_creation() {
        let analysis = ComplexityAnalysis {
            summary: ComplexitySummary {
                total_functions: 100,
                average_complexity: 5.5,
                max_complexity: 25,
                files_analyzed: 10,
            },
            files: vec![FileComplexity {
                path: "src/main.rs".to_string(),
                functions: vec![FunctionComplexity {
                    name: "main".to_string(),
                    cyclomatic: 5,
                    cognitive: 3,
                    line_count: 50,
                }],
            }],
        };
        assert_eq!(analysis.summary.total_functions, 100);
        assert_eq!(analysis.files[0].functions[0].name, "main");
    }

    // === Churn Analysis Data Structures ===

    #[test]
    fn test_churn_params_creation() {
        let params = ChurnParams {
            project_path: "/test".to_string(),
            period_days: 30,
            format: "json".to_string(),
        };
        assert_eq!(params.period_days, 30);
    }

    #[test]
    fn test_churn_analysis_creation() {
        let analysis = ChurnAnalysis {
            summary: ChurnSummary {
                total_commits: 150,
                files_changed: 45,
                period_days: 30,
            },
            hotspots: vec![ChurnHotspot {
                file: "src/lib.rs".to_string(),
                changes: 25,
                authors: vec!["alice".to_string(), "bob".to_string()],
            }],
        };
        assert_eq!(analysis.summary.total_commits, 150);
        assert_eq!(analysis.hotspots[0].changes, 25);
    }

    // === DAG Analysis Data Structures ===

    #[test]
    fn test_dag_params_creation() {
        let params = DagParams {
            project_path: "/project".to_string(),
            dag_type: "call-graph".to_string(),
            show_complexity: true,
            format: "mermaid".to_string(),
        };
        assert_eq!(params.dag_type, "call-graph");
        assert!(params.show_complexity);
    }

    #[test]
    fn test_dag_analysis_creation() {
        let analysis = DagAnalysis {
            graph: "graph TD; A-->B; B-->C;".to_string(),
            nodes: 3,
            edges: 2,
            cycles: vec!["A->B->A".to_string()],
        };
        assert_eq!(analysis.nodes, 3);
        assert_eq!(analysis.edges, 2);
        assert_eq!(analysis.cycles.len(), 1);
    }

    // === Context Analysis Data Structures ===

    #[test]
    fn test_context_params_creation() {
        let params = ContextParams {
            toolchain: "rust".to_string(),
            project_path: "/project".to_string(),
            format: "markdown".to_string(),
        };
        assert_eq!(params.toolchain, "rust");
    }

    #[test]
    fn test_project_context_creation() {
        let context = ProjectContext {
            project_name: "test-project".to_string(),
            toolchain: "rust".to_string(),
            structure: ProjectStructure {
                directories: vec!["src".to_string(), "tests".to_string()],
                files: vec!["Cargo.toml".to_string()],
            },
            metrics: ContextMetrics {
                total_files: 50,
                total_lines: 5000,
                complexity_score: 7.5,
            },
        };
        assert_eq!(context.project_name, "test-project");
        assert_eq!(context.metrics.total_files, 50);
    }

    // === Dead Code Analysis Data Structures ===

    #[test]
    fn test_dead_code_params_creation() {
        let params = DeadCodeParams {
            project_path: "/test".to_string(),
            format: "json".to_string(),
            top_files: Some(10),
            include_unreachable: true,
            min_dead_lines: 5,
            include_tests: false,
        };
        assert!(params.include_unreachable);
        assert_eq!(params.min_dead_lines, 5);
    }

    #[test]
    fn test_dead_code_analysis_creation() {
        let analysis = DeadCodeAnalysis {
            summary: DeadCodeSummary {
                total_files_analyzed: 100,
                files_with_dead_code: 10,
                total_dead_lines: 500,
                dead_percentage: 2.5,
            },
            files: vec![FileDeadCode {
                path: "src/unused.rs".to_string(),
                dead_lines: 50,
                dead_percentage: 25.0,
                dead_functions: 3,
                dead_classes: 1,
                confidence: "high".to_string(),
            }],
        };
        assert_eq!(analysis.summary.dead_percentage, 2.5);
        assert_eq!(analysis.files[0].dead_functions, 3);
    }

    // === Makefile Lint Data Structures ===

    #[test]
    fn test_makefile_lint_params_creation() {
        let params = MakefileLintParams {
            path: "Makefile".to_string(),
            rules: vec!["all".to_string()],
            fix: false,
            gnu_version: "4.3".to_string(),
        };
        assert!(!params.fix);
        assert_eq!(params.gnu_version, "4.3");
    }

    #[test]
    fn test_makefile_lint_analysis_creation() {
        let analysis = MakefileLintAnalysis {
            path: "Makefile".to_string(),
            violations: vec![MakefileLintViolation {
                rule: "no-shell-expansion".to_string(),
                severity: "warning".to_string(),
                line: 10,
                column: 5,
                message: "Unquoted variable".to_string(),
                fix_hint: Some("Quote the variable".to_string()),
            }],
            quality_score: 85.0,
            rules_applied: vec!["all".to_string()],
        };
        assert_eq!(analysis.quality_score, 85.0);
        assert_eq!(analysis.violations[0].line, 10);
    }

    // === Provability Data Structures ===

    #[test]
    fn test_provability_params_creation() {
        let params = ProvabilityParams {
            project_path: "/test".to_string(),
            functions: Some(vec!["main".to_string(), "parse_config".to_string()]),
            analysis_depth: Some(10),
        };
        assert!(params.functions.is_some());
        assert_eq!(params.functions.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_provability_analysis_creation() {
        let analysis = ProvabilityAnalysis {
            project_path: "/test".to_string(),
            analysis_depth: 10,
            functions_analyzed: 5,
            average_provability_score: 0.85,
            summaries: vec![],
        };
        assert_eq!(analysis.average_provability_score, 0.85);
    }

    // === SATD Data Structures ===

    #[test]
    fn test_satd_params_creation() {
        let params = SatdParams {
            project_path: "/test".to_string(),
            strict: Some(true),
            exclude_tests: Some(false),
            critical_only: Some(true),
        };
        assert!(params.strict.unwrap());
        assert!(params.critical_only.unwrap());
    }

    #[test]
    fn test_satd_file_creation() {
        let file = SatdFile {
            path: "src/lib.rs".to_string(),
            debt_count: 3,
            items: vec![SatdItem {
                line: 25,
                category: "FIXME".to_string(),
                severity: "High".to_string(),
                text: "Fix memory leak".to_string(),
                context: Some("fn process_data".to_string()),
            }],
        };
        assert_eq!(file.debt_count, 3);
        assert!(file.items[0].context.is_some());
    }

    // === Lint Hotspot Data Structures ===

    #[test]
    fn test_lint_hotspot_params_creation() {
        let params = LintHotspotParams {
            project_path: "/test".to_string(),
            top_files: Some(15),
            min_violations: Some(5),
            include: Some("*.rs".to_string()),
            exclude: Some("test_*".to_string()),
        };
        assert_eq!(params.top_files, Some(15));
        assert_eq!(params.min_violations, Some(5));
    }

    #[test]
    fn test_lint_hotspot_analysis_creation() {
        let mut severity_dist = std::collections::HashMap::new();
        severity_dist.insert("error".to_string(), 5);
        severity_dist.insert("warning".to_string(), 10);

        let analysis = LintHotspotAnalysis {
            project_path: "/test".to_string(),
            total_files_analyzed: 50,
            total_violations: 100,
            average_violations_per_file: 2.0,
            hotspots: vec![LintHotspot {
                file_path: "src/main.rs".to_string(),
                violations: 15,
                lines_of_code: 200,
                defect_density: 0.075,
                severity_distribution: severity_dist,
            }],
        };
        assert_eq!(analysis.total_violations, 100);
        assert_eq!(analysis.hotspots[0].defect_density, 0.075);
    }
