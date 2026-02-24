// Async builder methods for UnifiedContextBuilder
// Included via include!() in unified_context_builder.rs

impl UnifiedContextBuilder {
    // Add Big-O complexity analysis (async version)
    pub async fn add_big_o_analysis_async(&mut self) -> &mut Self {
        self.output.push_str("## Big-O Complexity Analysis\n\n");

        // Run big-o analysis
        let big_o_result = run_big_o_analysis(&self.project_path).await;

        if let Ok(analysis) = big_o_result {
            for (function, complexity) in analysis.complexities {
                self.output
                    .push_str(&format!("- `{}`: {}\n", function, complexity));
            }
        } else {
            self.output.push_str("*Big-O analysis not available*\n");
        }
        self.output.push('\n');
        self
    }

    // Add entropy analysis (async version)
    pub async fn add_entropy_analysis_async(&mut self) -> &mut Self {
        self.output.push_str("## Entropy Analysis\n\n");

        // Run entropy analysis
        let entropy_result = run_entropy_analysis(&self.project_path).await;

        if let Ok(analysis) = entropy_result {
            self.output.push_str(&format!(
                "- Pattern Entropy: {:.3}\n",
                analysis.pattern_entropy
            ));
            self.output.push_str(&format!(
                "- Code Duplication: {:.1}%\n",
                analysis.duplication_percentage
            ));
            self.output.push_str(&format!(
                "- Structural Entropy: {:.3}\n",
                analysis.structural_entropy
            ));

            if !analysis.actionable_improvements.is_empty() {
                self.output.push_str("- Actionable Improvements:\n");
                for improvement in &analysis.actionable_improvements {
                    self.output.push_str(&format!("  - {}\n", improvement));
                }
            }
        } else {
            self.output.push_str("*Entropy analysis not available*\n");
        }
        self.output.push('\n');
        self
    }

    // Add provability analysis
    pub async fn add_provability_analysis(&mut self) -> &mut Self {
        self.output.push_str("## Provability Analysis\n\n");

        // Run provability analysis
        let provability_result = run_provability_analysis(&self.project_path).await;

        if let Ok(analysis) = provability_result {
            self.output.push_str("### Invariants\n");
            for invariant in &analysis.invariants {
                self.output.push_str(&format!("- {}\n", invariant));
            }

            self.output.push_str("\n### Pre-conditions\n");
            for precondition in &analysis.preconditions {
                self.output.push_str(&format!("- {}\n", precondition));
            }

            self.output.push_str("\n### Post-conditions\n");
            for postcondition in &analysis.postconditions {
                self.output.push_str(&format!("- {}\n", postcondition));
            }

            self.output
                .push_str("\n### Abstract Interpretation Results\n");
            self.output
                .push_str(&format!("- Sound: {}\n", analysis.is_sound));
            self.output
                .push_str(&format!("- Complete: {}\n", analysis.is_complete));
        } else {
            self.output
                .push_str("*Provability analysis not available*\n");
        }
        self.output.push('\n');
        self
    }

    // Add graph metrics
    pub async fn add_graph_metrics(&mut self) -> &mut Self {
        self.output.push_str("## Graph Metrics\n\n");

        // Run graph metrics analysis
        let graph_result = run_graph_metrics_analysis(&self.project_path).await;

        if let Ok(analysis) = graph_result {
            self.output.push_str("### Centrality Measures\n");
            self.output.push_str(&format!(
                "- Betweenness Centrality: {:.3}\n",
                analysis.betweenness
            ));
            self.output.push_str(&format!(
                "- Closeness Centrality: {:.3}\n",
                analysis.closeness
            ));
            self.output
                .push_str(&format!("- Degree Centrality: {:.3}\n", analysis.degree));

            self.output.push_str("\n### Dependency Graph\n");
            self.output
                .push_str(&format!("- Nodes: {}\n", analysis.node_count));
            self.output
                .push_str(&format!("- Edges: {}\n", analysis.edge_count));
            self.output.push_str(&format!(
                "- Cyclomatic Complexity: {}\n",
                analysis.cyclomatic
            ));

            if !analysis.critical_paths.is_empty() {
                self.output.push_str("\n### Critical Paths\n");
                for path in &analysis.critical_paths {
                    self.output.push_str(&format!("- {}\n", path));
                }
            }
        } else {
            self.output.push_str("*Graph metrics not available*\n");
        }
        self.output.push('\n');
        self
    }

    // Add TDG analysis (async version)
    pub async fn add_tdg_analysis_async(&mut self) -> &mut Self {
        self.output.push_str("## Technical Debt Gradient (TDG)\n\n");

        // Run TDG analysis
        let tdg_result = run_tdg_analysis(&self.project_path).await;

        if let Ok(analysis) = tdg_result {
            self.output.push_str(&format!(
                "### Overall TDG Score: {:.2}\n\n",
                analysis.overall_score
            ));

            self.output.push_str("### File-level TDG:\n");
            for (file, score) in &analysis.file_scores {
                self.output
                    .push_str(&format!("- `{}`: {:.2}\n", file, score));
            }

            self.output.push_str("\n### Debt Hotspots:\n");
            for hotspot in &analysis.hotspots {
                self.output.push_str(&format!(
                    "- {} (Score: {:.2})\n",
                    hotspot.location, hotspot.score
                ));
            }

            self.output.push_str("\n### Refactoring Priority:\n");
            for (i, priority) in analysis.priorities.iter().enumerate().take(5) {
                self.output.push_str(&format!("{}. {}\n", i + 1, priority));
            }
        } else {
            self.output.push_str("*TDG analysis not available*\n");
        }
        self.output.push('\n');
        self
    }

    // Add dead code analysis
    pub async fn add_dead_code_analysis(&mut self) -> &mut Self {
        self.output.push_str("## Dead Code Analysis\n\n");

        // Run dead code analysis
        let dead_code_result = run_dead_code_analysis(&self.project_path).await;

        if let Ok(analysis) = dead_code_result {
            if !analysis.unreachable_functions.is_empty() {
                self.output.push_str("### Unreachable Functions:\n");
                for func in &analysis.unreachable_functions {
                    self.output.push_str(&format!("- `{}`\n", func));
                }
            }

            if !analysis.unused_variables.is_empty() {
                self.output.push_str("\n### Unused Variables:\n");
                for var in &analysis.unused_variables {
                    self.output.push_str(&format!("- `{}`\n", var));
                }
            }

            if !analysis.unused_imports.is_empty() {
                self.output.push_str("\n### Unused Imports:\n");
                for import in &analysis.unused_imports {
                    self.output.push_str(&format!("- `{}`\n", import));
                }
            }

            if !analysis.dead_branches.is_empty() {
                self.output.push_str("\n### Dead Branches:\n");
                for branch in &analysis.dead_branches {
                    self.output.push_str(&format!("- {}\n", branch));
                }
            }

            if analysis.is_empty() {
                self.output.push_str("*No dead code detected*\n");
            }
        } else {
            self.output.push_str("*Dead code analysis not available*\n");
        }
        self.output.push('\n');
        self
    }

    // Add SATD analysis
    pub async fn add_satd_analysis(&mut self) -> &mut Self {
        self.output
            .push_str("## Self-Admitted Technical Debt (SATD)\n\n");

        // Run SATD analysis
        let satd_result = run_satd_analysis(&self.project_path).await;

        if let Ok(analysis) = satd_result {
            let total_satd = analysis.todos.len()
                + analysis.fixmes.len()
                + analysis.hacks.len()
                + analysis.tech_debt.len();

            self.output
                .push_str(&format!("### Total SATD Comments: {}\n\n", total_satd));

            if !analysis.todos.is_empty() {
                self.output
                    .push_str(&format!("### TODO Comments: {}\n", analysis.todos.len()));
                for (_i, todo) in analysis.todos.iter().enumerate().take(5) {
                    self.output
                        .push_str(&format!("- {}: {}\n", todo.location, todo.comment));
                }
                if analysis.todos.len() > 5 {
                    self.output
                        .push_str(&format!("- ... and {} more\n", analysis.todos.len() - 5));
                }
            }

            if !analysis.fixmes.is_empty() {
                self.output.push_str(&format!(
                    "\n### FIXME Comments: {}\n",
                    analysis.fixmes.len()
                ));
                for (_i, fixme) in analysis.fixmes.iter().enumerate().take(5) {
                    self.output
                        .push_str(&format!("- {}: {}\n", fixme.location, fixme.comment));
                }
            }

            if !analysis.hacks.is_empty() {
                self.output
                    .push_str(&format!("\n### HACK Comments: {}\n", analysis.hacks.len()));
                for hack in analysis.hacks.iter().take(3) {
                    self.output
                        .push_str(&format!("- {}: {}\n", hack.location, hack.comment));
                }
            }

            self.output.push_str("\n### Debt Categories:\n");
            self.output
                .push_str(&format!("- Design Debt: {}\n", analysis.design_debt_count));
            self.output
                .push_str(&format!("- Code Debt: {}\n", analysis.code_debt_count));
            self.output
                .push_str(&format!("- Test Debt: {}\n", analysis.test_debt_count));
            self.output.push_str(&format!(
                "- Documentation Debt: {}\n",
                analysis.doc_debt_count
            ));
        } else {
            self.output.push_str("*SATD analysis not available*\n");
        }
        self.output.push('\n');
        self
    }
}
