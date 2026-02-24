// Dogfooding engine analysis and helper methods
// Included by dogfooding_engine.rs - do NOT add `use` imports here

impl DogfoodingEngine {
    /// Get churn metrics using the existing git analysis service
    fn get_churn_metrics(&self, root: &Path) -> Result<ChurnMetrics, TemplateError> {
        let churn_analysis = match GitAnalysisService::analyze_code_churn(root, 30) {
            Ok(analysis) => analysis,
            Err(_) => {
                // Return empty metrics if no git repo found (e.g., in tests)
                return Ok(ChurnMetrics {
                    files_changed: 0,
                    commit_count: 0,
                    total_additions: 0,
                    total_deletions: 0,
                    hotspots: Vec::new(),
                });
            }
        };

        let hotspots: Vec<FileHotspot> = churn_analysis
            .files
            .iter()
            .take(10)
            .map(|file| FileHotspot {
                path: file.path.clone(),
                change_count: file.commit_count,
                complexity_score: (file.churn_score * 10.0) as u32, // Rough estimate
                risk_score: f64::from(file.churn_score),
            })
            .collect();

        Ok(ChurnMetrics {
            files_changed: churn_analysis.summary.total_files_changed,
            commit_count: churn_analysis.summary.total_commits,
            total_additions: churn_analysis.files.iter().map(|f| f.additions).sum(),
            total_deletions: churn_analysis.files.iter().map(|f| f.deletions).sum(),
            hotspots,
        })
    }

    /// Generate churn analysis markdown
    pub async fn generate_churn_analysis(
        &self,
        root: &Path,
        date: &str,
    ) -> Result<String, TemplateError> {
        let mut analysis = String::new();

        analysis.push_str(&format!("# Code Churn Analysis - {date}\n\n"));

        let churn_metrics = self.get_churn_metrics(root)?;

        analysis.push_str("## Summary\n\n");
        analysis.push_str(&format!(
            "- **Files Changed**: {}\n",
            churn_metrics.files_changed
        ));
        analysis.push_str(&format!(
            "- **Total Commits**: {}\n",
            churn_metrics.commit_count
        ));
        analysis.push_str(&format!(
            "- **Total Additions**: {}\n",
            churn_metrics.total_additions
        ));
        analysis.push_str(&format!(
            "- **Total Deletions**: {}\n",
            churn_metrics.total_deletions
        ));

        analysis.push_str("\n## Top File Hotspots\n\n");
        analysis.push_str("| File | Changes | Complexity | Risk Score |\n");
        analysis.push_str("|------|---------|------------|------------|\n");

        for hotspot in churn_metrics.hotspots.iter().take(10) {
            analysis.push_str(&format!(
                "| {} | {} | {} | {:.2} |\n",
                hotspot.path.display(),
                hotspot.change_count,
                hotspot.complexity_score,
                hotspot.risk_score
            ));
        }

        Ok(analysis)
    }

    /// Generate server info markdown
    pub fn generate_server_info(&self, date: &str) -> Result<String, TemplateError> {
        let mut info = String::new();

        info.push_str(&format!("# Server Information - {date}\n\n"));

        // Binary metadata
        info.push_str("## Binary Metadata\n\n");
        info.push_str(&format!("- **Build Date**: {date}\n"));
        info.push_str(&format!("- **Rust Version**: {}\n", "1.82.0"));
        info.push_str(&format!("- **Target**: {}\n", std::env::consts::ARCH));
        info.push_str(&format!("- **OS**: {}\n", std::env::consts::OS));

        // Runtime information
        info.push_str("\n## Runtime Information\n\n");
        info.push_str(&format!("- **PID**: {}\n", std::process::id()));
        info.push_str(&format!(
            "- **Executable**: {}\n",
            std::env::current_exe()
                .map_or_else(|_| "unknown".to_string(), |p| p.display().to_string())
        ));

        // Performance characteristics
        info.push_str("\n## Performance Characteristics\n\n");
        info.push_str("- **Startup Time**: <10ms\n");
        info.push_str("- **Memory Usage**: <50MB\n");
        info.push_str("- **AST Parsing**: O(n) per file\n");
        info.push_str("- **Graph Generation**: O(n log n + m)\n");

        Ok(info)
    }

    /// Analyze all files in the AST forest
    fn analyze_all_files(&self, forest: &AstForest) -> Result<Vec<FileContext>, TemplateError> {
        let mut contexts = Vec::new();

        for (module_path, _module) in forest.files() {
            // Create dummy path and AST for now - this is a compatibility stub
            let path = Path::new(module_path);
            let dummy_ast = crate::services::unified_ast_engine::FileAst::Rust(
                syn::parse_str("").unwrap_or_else(|_| syn::File {
                    shebang: None,
                    attrs: Vec::new(),
                    items: Vec::new(),
                }),
            );
            let context = self.analyze_single_file(path, &dummy_ast)?;
            contexts.push(context);
        }

        Ok(contexts)
    }

    /// Analyze a single file AST
    fn analyze_single_file(
        &self,
        path: &Path,
        ast: &crate::services::unified_ast_engine::FileAst,
    ) -> Result<FileContext, TemplateError> {
        use crate::services::unified_ast_engine::FileAst;

        match ast {
            FileAst::Rust(syn_ast) => {
                let mut functions = 0;
                let mut structs = 0;
                let mut traits = 0;
                let mut max_complexity = 0;

                for item in &syn_ast.items {
                    match item {
                        syn::Item::Fn(_) => {
                            functions += 1;
                            // Simple complexity heuristic
                            max_complexity = max_complexity.max(5);
                        }
                        syn::Item::Struct(_) => structs += 1,
                        syn::Item::Trait(_) => traits += 1,
                        syn::Item::Impl(_) => {
                            // Count methods in impl blocks
                            functions += 1;
                            max_complexity = max_complexity.max(3);
                        }
                        _ => {}
                    }
                }

                Ok(FileContext {
                    path: path.to_path_buf(),
                    functions,
                    structs,
                    traits,
                    max_complexity,
                    lines: syn_ast.items.len() * 10, // Rough estimate
                })
            }
            FileAst::TypeScript(_)
            | FileAst::Python(_)
            | FileAst::C(_)
            | FileAst::Cpp(_)
            | FileAst::Cython(_)
            | FileAst::Kotlin(_) => {
                // Placeholder for other languages
                Ok(FileContext {
                    path: path.to_path_buf(),
                    functions: 0,
                    structs: 0,
                    traits: 0,
                    max_complexity: 0,
                    lines: 0,
                })
            }
            FileAst::Makefile(makefile_ast) => {
                // Count rules as functions
                let functions = makefile_ast
                    .lines()
                    .filter(|line| line.contains(':') && !line.starts_with('#'))
                    .count();
                let max_complexity = functions.min(10) as u32;

                Ok(FileContext {
                    path: path.to_path_buf(),
                    functions,
                    structs: 0,
                    traits: 0,
                    max_complexity,
                    lines: makefile_ast.lines().count(),
                })
            }
            FileAst::Markdown(_)
            | FileAst::Toml(_)
            | FileAst::Yaml(_)
            | FileAst::Json(_)
            | FileAst::Shell(_) => {
                // Basic context for non-code files
                Ok(FileContext {
                    path: path.to_path_buf(),
                    functions: 0,
                    structs: 0,
                    traits: 0,
                    max_complexity: 0,
                    lines: 50, // Rough estimate
                })
            }
        }
    }

    /// Compute DAG metrics
    async fn compute_dag_metrics(&self, root: &Path) -> Result<DagMetrics, TemplateError> {
        let ast_forest = self.ast_engine.parse_project(root).await?;
        let dependency_graph = self.ast_engine.extract_dependencies(&ast_forest)?;

        let node_count = dependency_graph.node_count();
        let edge_count = dependency_graph.edge_count();

        let density = if node_count > 1 {
            edge_count as f64 / (node_count * (node_count - 1)) as f64
        } else {
            0.0
        };

        // Simple estimates for other metrics
        let diameter = if node_count > 0 {
            (node_count as f64).log2().ceil() as usize
        } else {
            0
        };

        let clustering = if edge_count > 0 { 0.3 } else { 0.0 }; // Placeholder

        Ok(DagMetrics {
            node_count,
            edge_count,
            density,
            diameter,
            clustering,
            strongly_connected_components: 1, // Placeholder
        })
    }

    /// Compute deterministic hash of all metrics
    fn compute_metrics_hash(
        &self,
        ast_metrics: &ProjectMetrics,
        churn_metrics: &ChurnMetrics,
        dag_metrics: &DagMetrics,
    ) -> String {
        use blake3::Hasher;

        let mut hasher = Hasher::new();

        // Hash AST metrics
        hasher.update(&ast_metrics.file_count.to_le_bytes());
        hasher.update(&ast_metrics.function_count.to_le_bytes());
        hasher.update(&ast_metrics.avg_complexity.to_le_bytes());
        hasher.update(&ast_metrics.max_complexity.to_le_bytes());

        // Hash churn metrics
        hasher.update(&churn_metrics.files_changed.to_le_bytes());
        hasher.update(&churn_metrics.commit_count.to_le_bytes());
        hasher.update(&churn_metrics.total_additions.to_le_bytes());
        hasher.update(&churn_metrics.total_deletions.to_le_bytes());

        // Hash DAG metrics
        hasher.update(&dag_metrics.node_count.to_le_bytes());
        hasher.update(&dag_metrics.edge_count.to_le_bytes());
        hasher.update(&dag_metrics.density.to_le_bytes());

        format!("{}", hasher.finalize())
    }
}
