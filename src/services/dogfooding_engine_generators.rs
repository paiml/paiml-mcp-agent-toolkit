// Dogfooding engine generator methods
// Included by dogfooding_engine.rs - do NOT add `use` imports here

impl DogfoodingEngine {
    /// Generate AST context analysis markdown
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn generate_ast_context(
        &self,
        root: &Path,
        date: &str,
    ) -> Result<String, TemplateError> {
        debug_assert!(root.exists(), "root must exist: {}", root.display());
        debug_assert!(!date.is_empty(), "date must not be empty");
        let mut context = String::new();

        context.push_str(&format!("# AST Context Analysis - {date}\n\n"));
        context.push_str("## Project Structure\n\n");

        // Parse project to get AST forest
        let ast_forest = self.ast_engine.parse_project(root).await?;

        // Extract file contexts in deterministic order
        let file_contexts: BTreeMap<PathBuf, FileContext> = self
            .analyze_all_files(&ast_forest)?
            .into_iter()
            .map(|ctx| (ctx.path.clone(), ctx))
            .collect();

        for (path, ctx) in &file_contexts {
            context.push_str(&format!("### {}\n\n", path.display()));
            context.push_str(&format!("- **Functions**: {}\n", ctx.functions));
            context.push_str(&format!("- **Structs**: {}\n", ctx.structs));
            context.push_str(&format!("- **Traits**: {}\n", ctx.traits));
            context.push_str(&format!("- **Max Complexity**: {}\n", ctx.max_complexity));
            context.push_str(&format!("- **Lines**: {}\n\n", ctx.lines));
        }

        // Add summary statistics
        let total_functions: usize = file_contexts.values().map(|ctx| ctx.functions).sum();
        let total_structs: usize = file_contexts.values().map(|ctx| ctx.structs).sum();
        let total_traits: usize = file_contexts.values().map(|ctx| ctx.traits).sum();
        let max_complexity = file_contexts
            .values()
            .map(|ctx| ctx.max_complexity)
            .max()
            .unwrap_or(0);
        let total_lines: usize = file_contexts.values().map(|ctx| ctx.lines).sum();

        context.push_str("## Summary Statistics\n\n");
        context.push_str(&format!("- **Total Files**: {}\n", file_contexts.len()));
        context.push_str(&format!("- **Total Functions**: {total_functions}\n"));
        context.push_str(&format!("- **Total Structs**: {total_structs}\n"));
        context.push_str(&format!("- **Total Traits**: {total_traits}\n"));
        context.push_str(&format!("- **Maximum Complexity**: {max_complexity}\n"));
        context.push_str(&format!("- **Total Lines**: {total_lines}\n"));

        Ok(context)
    }

    /// Generate combined metrics JSON
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn generate_combined_metrics(
        &self,
        root: &Path,
        date: &str,
    ) -> Result<Value, TemplateError> {
        debug_assert!(root.exists(), "root must exist: {}", root.display());
        debug_assert!(!date.is_empty(), "date must not be empty");
        let ast_forest = self.ast_engine.parse_project(root).await?;
        let ast_metrics = self.ast_engine.compute_metrics(&ast_forest)?;
        let churn_metrics = self.get_churn_metrics(root)?;
        let dag_metrics = self.compute_dag_metrics(root).await?;

        Ok(json!({
            "timestamp": date,
            "generation_time": Utc::now().to_rfc3339(),
            "ast": {
                "total_files": ast_metrics.file_count,
                "total_functions": ast_metrics.function_count,
                "avg_complexity": ast_metrics.avg_complexity,
                "max_complexity": ast_metrics.max_complexity,
            },
            "churn": {
                "files_changed": churn_metrics.files_changed,
                "total_commits": churn_metrics.commit_count,
                "total_additions": churn_metrics.total_additions,
                "total_deletions": churn_metrics.total_deletions,
                "hotspots": churn_metrics.hotspots.iter().take(5).map(|h| json!({
                    "path": h.path.display().to_string(),
                    "change_count": h.change_count,
                    "complexity_score": h.complexity_score,
                    "risk_score": h.risk_score,
                })).collect::<Vec<_>>(),
            },
            "dag": {
                "node_count": dag_metrics.node_count,
                "edge_count": dag_metrics.edge_count,
                "density": dag_metrics.density,
                "diameter": dag_metrics.diameter,
                "clustering_coefficient": dag_metrics.clustering,
                "strongly_connected_components": dag_metrics.strongly_connected_components,
            },
            "hash": self.compute_metrics_hash(&ast_metrics, &churn_metrics, &dag_metrics),
        }))
    }

    /// Generate complexity analysis markdown
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn generate_complexity_analysis(
        &self,
        root: &Path,
        date: &str,
    ) -> Result<String, TemplateError> {
        debug_assert!(root.exists(), "root must exist: {}", root.display());
        debug_assert!(!date.is_empty(), "date must not be empty");
        let mut analysis = String::new();

        analysis.push_str(&format!("# Complexity Analysis - {date}\n\n"));

        let ast_forest = self.ast_engine.parse_project(root).await?;
        let file_contexts = self.analyze_all_files(&ast_forest)?;

        // Sort by complexity descending
        let mut sorted_contexts = file_contexts;
        sorted_contexts.sort_by(|a, b| b.max_complexity.cmp(&a.max_complexity));

        analysis.push_str("## High Complexity Files\n\n");
        analysis.push_str("| File | Max Complexity | Functions | Structs | Traits |\n");
        analysis.push_str("|------|----------------|-----------|---------|--------|\n");

        for ctx in sorted_contexts.iter().take(10) {
            analysis.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                ctx.path.display(),
                ctx.max_complexity,
                ctx.functions,
                ctx.structs,
                ctx.traits
            ));
        }

        // Calculate distribution
        let complexities: Vec<u32> = sorted_contexts
            .iter()
            .map(|ctx| ctx.max_complexity)
            .collect();
        let total_files = complexities.len();
        let avg_complexity: f64 = f64::from(complexities.iter().sum::<u32>()) / total_files as f64;
        let median_complexity = if total_files > 0 {
            complexities[total_files / 2]
        } else {
            0
        };

        analysis.push_str("\n## Complexity Distribution\n\n");
        analysis.push_str(&format!("- **Total Files**: {total_files}\n"));
        analysis.push_str(&format!("- **Average Complexity**: {avg_complexity:.2}\n"));
        analysis.push_str(&format!("- **Median Complexity**: {median_complexity}\n"));
        analysis.push_str(&format!(
            "- **Maximum Complexity**: {}\n",
            complexities.first().unwrap_or(&0)
        ));

        // Complexity buckets
        let low_complexity = complexities.iter().filter(|&&c| c <= 5).count();
        let medium_complexity = complexities.iter().filter(|&&c| c > 5 && c <= 15).count();
        let high_complexity = complexities.iter().filter(|&&c| c > 15).count();

        analysis.push_str("\n## Complexity Buckets\n\n");
        analysis.push_str(&format!(
            "- **Low (≤5)**: {} files ({:.1}%)\n",
            low_complexity,
            low_complexity as f64 / total_files as f64 * 100.0
        ));
        analysis.push_str(&format!(
            "- **Medium (6-15)**: {} files ({:.1}%)\n",
            medium_complexity,
            medium_complexity as f64 / total_files as f64 * 100.0
        ));
        analysis.push_str(&format!(
            "- **High (>15)**: {} files ({:.1}%)\n",
            high_complexity,
            high_complexity as f64 / total_files as f64 * 100.0
        ));

        Ok(analysis)
    }
}
