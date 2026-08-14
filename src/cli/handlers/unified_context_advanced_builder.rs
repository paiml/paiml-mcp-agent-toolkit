// Core builder methods for AdvancedUnifiedContextBuilder.
// Contains: new(), build_complete_context(), get_basic_context(),
// add_project_header(), add_project_structure(), add_key_components().

impl AdvancedUnifiedContextBuilder {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    /// Create a new instance.
    pub fn new(project_path: &Path) -> Self {
        Self {
            project_path: project_path.to_path_buf(),
            output: String::new(),
            enable_big_o: true,
            enable_entropy: true,
            enable_provability: true,
            enable_graph_metrics: true,
            enable_tdg: true,
            enable_dead_code: true,
            enable_satd: true,
        }
    }

    /// Build the complete unified context with all annotations
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn build_complete_context(&mut self) -> Result<String> {
        info!(
            "Building unified context with advanced annotations for {:?}",
            self.project_path
        );

        // Step 1: Get basic context using SimpleDeepContext
        let basic_context = self.get_basic_context().await?;
        self.add_project_header(&basic_context);
        self.add_project_structure(&basic_context);
        self.add_key_components(&basic_context);

        // Step 2: Add all advanced annotations
        if self.enable_big_o {
            self.add_big_o_analysis().await?;
        }

        if self.enable_entropy {
            self.add_entropy_analysis().await?;
        }

        if self.enable_provability {
            self.add_provability_analysis().await?;
        }

        if self.enable_graph_metrics {
            self.add_graph_metrics().await?;
        }

        if self.enable_tdg {
            self.add_tdg_analysis().await?;
        }

        if self.enable_dead_code {
            self.add_dead_code_analysis().await?;
        }

        if self.enable_satd {
            self.add_satd_analysis().await?;
        }

        // Step 3: Add quality insights and recommendations
        self.add_quality_insights(&basic_context);
        self.add_recommendations(&basic_context);

        Ok(self.output.clone())
    }

    async fn get_basic_context(&self) -> Result<ProjectContext> {
        use crate::services::simple_deep_context::{SimpleAnalysisConfig, SimpleDeepContext};

        let analyzer = SimpleDeepContext::new();
        let config = SimpleAnalysisConfig {
            project_path: self.project_path.clone(),
            include_features: vec![],
            include_patterns: vec![],
            exclude_patterns: vec!["**/node_modules/**".to_string(), "**/target/**".to_string()],
            enable_verbose: false,
        };

        let analysis_report = analyzer.analyze(config).await?;

        // Convert SimpleDeepContext report to ProjectContext
        Ok(ProjectContext {
            project_type: "rust".to_string(), // Default to rust
            files: vec![],                    // Would need to be populated from analysis_report
            graph: None,
            summary: ProjectSummary {
                total_files: analysis_report.file_count,
                total_functions: analysis_report.complexity_metrics.total_functions,
                total_structs: 0,
                total_enums: 0,
                total_traits: 0,
                total_impls: 0,
                dependencies: vec![],
            },
        })
    }

    fn add_project_header(&mut self, context: &ProjectContext) {
        self.output.push_str("# Project Context\n\n");
        self.output
            .push_str(&format!("Project: {}\n", self.project_path.display()));
        self.output
            .push_str(&format!("Language: {}\n\n", context.project_type));
    }

    fn add_project_structure(&mut self, context: &ProjectContext) {
        self.output.push_str("## Project Structure\n\n");
        self.output.push_str(&format!(
            "- **Total Files**: {}\n",
            context.summary.total_files
        ));
        self.output.push_str(&format!(
            "- **Total Functions**: {}\n",
            context.summary.total_functions
        ));
        self.output.push_str(&format!(
            "- **Total Structs**: {}\n",
            context.summary.total_structs
        ));
        self.output.push_str(&format!(
            "- **Total Enums**: {}\n",
            context.summary.total_enums
        ));
        self.output.push_str(&format!(
            "- **Total Traits**: {}\n",
            context.summary.total_traits
        ));
        self.output.push('\n');
    }

    fn add_key_components(&mut self, context: &ProjectContext) {
        self.output.push_str("## Key Components\n\n");

        for file in &context.files {
            if let Some(complexity) = &file.complexity_metrics {
                if !complexity.functions.is_empty() {
                    self.output
                        .push_str(&format!("### File: `{}`\n", file.path));
                    self.output.push_str(&format!(
                        "- Cyclomatic: {}\n",
                        complexity.total_complexity.cyclomatic
                    ));
                    self.output.push_str(&format!(
                        "- Cognitive: {}\n",
                        complexity.total_complexity.cognitive
                    ));
                    self.output.push_str(&format!(
                        "- Function Count: {}\n",
                        complexity.functions.len()
                    ));

                    // Function names would be extracted from AST items
                    let function_count = file
                        .items
                        .iter()
                        .filter(|i| matches!(i, AstItem::Function { .. }))
                        .count();
                    if function_count > 0 {
                        self.output
                            .push_str(&format!("- Functions: {}\n", function_count));
                    }
                    self.output.push('\n');
                }
            }
        }
    }
}
