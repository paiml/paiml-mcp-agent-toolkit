// DeepContextAnalyzer formatting methods - extracted for file health (CB-040)
pub struct DeepContextAnalyzer {
    config: DeepContextConfig,
}

impl DeepContextAnalyzer {
    /// Creates a new `DeepContextAnalyzer` with the given configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::services::deep_context::{DeepContextAnalyzer, DeepContextConfig};
    ///
    /// let config = DeepContextConfig::default();
    /// let analyzer = DeepContextAnalyzer::new(config);
    /// // Analyzer is ready to perform deep context analysis
    /// ```
    #[must_use]
    pub fn new(config: DeepContextConfig) -> Self {
        Self { config }
    }

    /// Format as comprehensive markdown output using simple formatting
    pub async fn format_as_comprehensive_markdown(
        &self,
        context: &DeepContext,
    ) -> anyhow::Result<String> {
        let mut output = String::with_capacity(1024);
        output.push_str("# Deep Context Analysis Report\n\n");

        self.append_project_overview(&mut output, &context.project_overview)?;
        self.append_build_info(&mut output, &context.build_info)?;
        self.append_quality_scorecard(&mut output, &context.quality_scorecard)?;
        self.append_project_structure(&mut output, &context.file_tree)?;
        self.append_analysis_results(&mut output, &context.analyses)?;
        self.append_recommendations(&mut output, &context.recommendations)?;

        Ok(output)
    }

    fn append_project_overview(
        &self,
        output: &mut String,
        overview: &Option<crate::models::project_meta::ProjectOverview>,
    ) -> anyhow::Result<()> {
        if let Some(ref overview) = overview {
            output.push_str("## Project Overview\n\n");
            if !overview.compressed_description.is_empty() {
                output.push_str(&overview.compressed_description);
                output.push_str("\n\n");
            }
            if !overview.key_features.is_empty() {
                output.push_str("**Key Features:**\n");
                for feature in &overview.key_features {
                    output.push_str(&format!("- {feature}\n"));
                }
                output.push('\n');
            }
            if let Some(ref arch) = overview.architecture_summary {
                output.push_str("**Architecture:**\n");
                output.push_str(arch);
                output.push_str("\n\n");
            }
        }
        Ok(())
    }

    fn append_build_info(
        &self,
        output: &mut String,
        build_info: &Option<crate::models::project_meta::BuildInfo>,
    ) -> anyhow::Result<()> {
        if let Some(ref build_info) = build_info {
            output.push_str("## Build System\n\n");
            output.push_str(&format!(
                "**Detected Toolchain:** {}\n",
                build_info.toolchain
            ));
            if !build_info.targets.is_empty() {
                output.push_str(&format!(
                    "**Primary Targets:** {}\n",
                    build_info.targets.join(", ")
                ));
            }
            if !build_info.dependencies.is_empty() {
                output.push_str(&format!(
                    "**Key Dependencies:** {}\n",
                    build_info.dependencies.join(", ")
                ));
            }
            if let Some(ref cmd) = build_info.primary_command {
                output.push_str(&format!("**Build Command:** `{cmd}`\n"));
            }
            output.push('\n');
        }
        Ok(())
    }

    fn append_quality_scorecard(
        &self,
        output: &mut String,
        scorecard: &QualityScorecard,
    ) -> anyhow::Result<()> {
        output.push_str("## Quality Scorecard\n\n");
        output.push_str(&format!(
            "- Overall Health: {:.1}%\n",
            scorecard.overall_health
        ));
        output.push_str(&format!(
            "- Maintainability Index: {:.1}%\n",
            scorecard.maintainability_index
        ));
        output.push_str(&format!(
            "- Refactoring Time: {:.1} hours\n",
            scorecard.technical_debt_hours
        ));
        output.push_str(&format!(
            "- Complexity Score: {:.1}%\n",
            scorecard.complexity_score
        ));
        output.push('\n');
        Ok(())
    }

    fn append_project_structure(
        &self,
        output: &mut String,
        file_tree: &AnnotatedFileTree,
    ) -> anyhow::Result<()> {
        output.push_str("## Project Structure\n\n");
        output.push_str("```\n");
        output.push_str(&format!(
            "Total Files: {}\nTotal Size: {} bytes\n",
            file_tree.total_files, file_tree.total_size_bytes
        ));
        output.push_str("\n```\n\n");
        Ok(())
    }

    fn append_analysis_results(
        &self,
        output: &mut String,
        analyses: &AnalysisResults,
    ) -> anyhow::Result<()> {
        output.push_str("## Analysis Results\n\n");

        if !analyses.ast_contexts.is_empty() {
            output.push_str(&format!(
                "### AST Analysis\n- Files analyzed: {}\n\n",
                analyses.ast_contexts.len()
            ));
        }

        if let Some(ref complexity) = analyses.complexity_report {
            output.push_str(&format!("### Complexity Analysis\n- Total files: {}\n- Total functions: {}\n- Median cyclomatic complexity: {:.1}\n\n",
                complexity.summary.total_files, complexity.summary.total_functions, complexity.summary.median_cyclomatic));
        }

        if let Some(ref churn) = analyses.churn_analysis {
            output.push_str(&format!(
                "### Code Churn\n- Files analyzed: {}\n- Total commits: {}\n\n",
                churn.files.len(),
                churn.summary.total_commits
            ));
        }
        Ok(())
    }

    fn append_recommendations(
        &self,
        output: &mut String,
        recommendations: &[PrioritizedRecommendation],
    ) -> anyhow::Result<()> {
        if !recommendations.is_empty() {
            output.push_str("## Recommendations\n\n");
            for (i, rec) in recommendations.iter().enumerate() {
                output.push_str(&format!(
                    "{}. **{}** (Priority: {:?})\n   {}\n   Effort: {:?}\n\n",
                    i + 1,
                    rec.title,
                    rec.priority,
                    rec.description,
                    rec.estimated_effort
                ));
            }
        }
        Ok(())
    }

    /// Legacy format method (kept for backward compatibility)
    pub fn format_as_comprehensive_markdown_legacy(
        &self,
        context: &DeepContext,
    ) -> anyhow::Result<String> {
        let mut output = String::with_capacity(1024);

        // Step 1: Format header and metadata
        self.format_legacy_header(&mut output, context)?;

        // Step 2: Format main content sections
        self.format_legacy_main_sections(&mut output, context)?;

        // Step 3: Format analysis sections
        self.format_legacy_analysis_sections(&mut output, context)?;

        Ok(output)
    }

    /// Format header and metadata for legacy markdown
    fn format_legacy_header(
        &self,
        output: &mut String,
        context: &DeepContext,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;

        let project_name = context
            .metadata
            .project_root
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();

        writeln!(output, "# Deep Context: {project_name}")?;
        writeln!(output, "Generated: {}", context.metadata.generated_at)?;
        writeln!(output, "Version: {}", context.metadata.tool_version)?;
        writeln!(
            output,
            "Analysis Time: {:.2}s",
            context.metadata.analysis_duration.as_secs_f64()
        )?;
        writeln!(
            output,
            "Cache Hit Rate: {:.1}%",
            context.metadata.cache_stats.hit_rate * 100.0
        )?;

        Ok(())
    }

    /// Format main content sections for legacy markdown
    fn format_legacy_main_sections(
        &self,
        output: &mut String,
        context: &DeepContext,
    ) -> anyhow::Result<()> {
        self.write_quality_scorecard_section(output, &context.quality_scorecard)?;
        self.write_project_structure_section(output, &context.file_tree)?;
        self.write_ast_section_if_present(output, &context.analyses.ast_contexts)?;
        Ok(())
    }

    fn write_quality_scorecard_section(
        &self,
        output: &mut String,
        scorecard: &QualityScorecard,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        writeln!(output, "\n## Quality Scorecard\n")?;
        writeln!(
            output,
            "- **Overall Health**: {} ({:.1}/100)",
            self.overall_health_emoji(scorecard.overall_health),
            scorecard.overall_health
        )?;
        writeln!(
            output,
            "- **Maintainability Index**: {:.1}",
            scorecard.maintainability_index
        )?;
        writeln!(
            output,
            "- **Refactoring Time**: {:.1} hours estimated",
            scorecard.technical_debt_hours
        )?;
        Ok(())
    }

    fn write_project_structure_section(
        &self,
        output: &mut String,
        file_tree: &AnnotatedFileTree,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        writeln!(output, "\n## Project Structure\n")?;
        writeln!(output, "```")?;
        self.format_annotated_tree(output, file_tree)?;
        writeln!(output, "```\n")?;
        Ok(())
    }

    fn write_ast_section_if_present(
        &self,
        output: &mut String,
        ast_contexts: &[EnhancedFileContext],
    ) -> anyhow::Result<()> {
        if !ast_contexts.is_empty() {
            self.format_enhanced_ast_section(output, ast_contexts)?;
        }
        Ok(())
    }

    /// Format analysis sections for legacy markdown
    fn format_legacy_analysis_sections(
        &self,
        output: &mut String,
        context: &DeepContext,
    ) -> anyhow::Result<()> {
        // Code quality metrics
        self.format_complexity_hotspots(output, context)?;
        self.format_churn_analysis(output, context)?;
        self.format_technical_debt(output, context)?;
        self.format_dead_code_analysis(output, context)?;

        // Cross-language references
        self.format_cross_references(output, &context.analyses.cross_language_refs)?;

        // Defect probability analysis
        self.format_defect_predictions(output, context)?;

        // Actionable recommendations
        self.format_prioritized_recommendations(output, &context.recommendations)?;

        Ok(())
    }

    /// Format as JSON output for machine consumption and API responses
    pub fn format_as_json(&self, context: &DeepContext) -> anyhow::Result<String> {
        serde_json::to_string_pretty(context)
            .map_err(|e| anyhow::anyhow!("Failed to serialize to JSON: {e}"))
    }

    /// Format as SARIF (Static Analysis Results Interchange Format) for tool integration
    pub fn format_as_sarif(&self, context: &DeepContext) -> anyhow::Result<String> {
        use serde_json::json;

        let mut results = Vec::new();
        let mut rules = Vec::new();

        // Process each analysis type through dedicated handlers
        self.add_complexity_sarif_items_from_analyses(&context.analyses, &mut rules, &mut results);
        self.add_satd_sarif_items_from_analyses(&context.analyses, &mut rules, &mut results);
        self.add_dead_code_sarif_items_from_analyses(&context.analyses, &mut rules, &mut results);

        let sarif = json!({
            "version": "2.1.0",
            "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
            "runs": [{
                "tool": {
                    "driver": {
                        "name": "pmat",
                        "version": context.metadata.tool_version,
                        "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                        "shortDescription": {"text": "Professional project scaffolding and analysis toolkit"},
                        "rules": rules
                    }
                },
                "results": results,
                "properties": {
                    "analysis_duration_seconds": context.metadata.analysis_duration.as_secs_f64(),
                    "cache_hit_rate": context.metadata.cache_stats.hit_rate,
                    "overall_health_score": context.quality_scorecard.overall_health,
                    "technical_debt_hours": context.quality_scorecard.technical_debt_hours
                }
            }]
        });

        serde_json::to_string_pretty(&sarif)
            .map_err(|e| anyhow::anyhow!("Failed to serialize to SARIF: {e}"))
    }

    /// Add complexity violations to SARIF results from `AnalysisResults`
    fn add_complexity_sarif_items_from_analyses(
        &self,
        analyses: &AnalysisResults,
        rules: &mut Vec<serde_json::Value>,
        results: &mut Vec<serde_json::Value>,
    ) {
        use serde_json::json;

        if let Some(ref complexity) = analyses.complexity_report {
            // Add complexity rules once
            rules.extend_from_slice(&[
                json!({
                    "id": "complexity/high-cyclomatic",
                    "shortDescription": {"text": "High cyclomatic complexity"},
                    "fullDescription": {"text": "Function has cyclomatic complexity above recommended threshold"},
                    "defaultConfiguration": {"level": "warning"},
                    "properties": {"tags": ["complexity", "maintainability"]}
                }),
                json!({
                    "id": "complexity/high-cognitive",
                    "shortDescription": {"text": "High cognitive complexity"},
                    "fullDescription": {"text": "Function has cognitive complexity above recommended threshold"},
                    "defaultConfiguration": {"level": "warning"},
                    "properties": {"tags": ["complexity", "maintainability"]}
                })
            ]);

            // Process complexity violations
            for file in &complexity.files {
                for func in &file.functions {
                    self.add_complexity_violation(file, func, results);
                }
            }
        }
    }

    /// Add a single complexity violation
    fn add_complexity_violation(
        &self,
        file: &crate::services::complexity::FileComplexityMetrics,
        func: &crate::services::complexity::FunctionComplexity,
        results: &mut Vec<serde_json::Value>,
    ) {
        use serde_json::json;

        if func.metrics.cyclomatic > 10 {
            results.push(json!({
                "ruleId": "complexity/high-cyclomatic",
                "level": if func.metrics.cyclomatic > 20 { "error" } else { "warning" },
                "message": {"text": format!("Function '{}' has cyclomatic complexity of {}", func.name, func.metrics.cyclomatic)},
                "locations": [self.create_location(&file.path, func.line_start as usize, func.line_end as usize)],
                "properties": {
                    "cyclomatic_complexity": func.metrics.cyclomatic,
                    "cognitive_complexity": func.metrics.cognitive
                }
            }));
        }

        if func.metrics.cognitive > 15 {
            results.push(json!({
                "ruleId": "complexity/high-cognitive",
                "level": if func.metrics.cognitive > 25 { "error" } else { "warning" },
                "message": {"text": format!("Function '{}' has cognitive complexity of {}", func.name, func.metrics.cognitive)},
                "locations": [self.create_location(&file.path, func.line_start as usize, func.line_end as usize)],
                "properties": {
                    "cyclomatic_complexity": func.metrics.cyclomatic,
                    "cognitive_complexity": func.metrics.cognitive
                }
            }));
        }
    }

    /// Add SATD items to SARIF results from `AnalysisResults`
    fn add_satd_sarif_items_from_analyses(
        &self,
        analyses: &AnalysisResults,
        rules: &mut Vec<serde_json::Value>,
        results: &mut Vec<serde_json::Value>,
    ) {
        use serde_json::json;

        if let Some(ref satd) = analyses.satd_results {
            rules.push(json!({
                "id": "debt/technical-debt",
                "shortDescription": {"text": "Code quality issue"},
                "fullDescription": {"text": "Self-admitted code issue requiring attention"},
                "defaultConfiguration": {"level": "note"},
                "properties": {"tags": ["debt", "maintainability"]}
            }));

            for item in &satd.items {
                let level = self.satd_severity_to_level(&item.severity);
                results.push(json!({
                    "ruleId": "debt/technical-debt",
                    "level": level,
                    "message": {"text": format!("{}: {}", item.category, item.text.trim())},
                    "locations": [self.create_location(&item.file.to_string_lossy(), item.line as usize, item.line as usize)],
                    "properties": {
                        "category": format!("{:?}", item.category),
                        "severity": format!("{:?}", item.severity),
                        "debt_type": "self_admitted"
                    }
                }));
            }
        }
    }

    /// Add dead code items to SARIF results from `AnalysisResults`
    fn add_dead_code_sarif_items_from_analyses(
        &self,
        analyses: &AnalysisResults,
        rules: &mut Vec<serde_json::Value>,
        results: &mut Vec<serde_json::Value>,
    ) {
        use serde_json::json;

        if let Some(ref dead_code) = analyses.dead_code_results {
            rules.push(json!({
                "id": "dead-code/unused-code",
                "shortDescription": {"text": "Dead code detected"},
                "fullDescription": {"text": "Code that appears to be unused and can potentially be removed"},
                "defaultConfiguration": {"level": "warning"},
                "properties": {"tags": ["dead-code", "maintainability"]}
            }));

            results.extend(
                dead_code.ranked_files
                    .iter()
                    .filter(|file| file.dead_functions > 0)
                    .map(|file| json!({
                        "ruleId": "dead-code/unused-code",
                        "level": "warning",
                        "message": {"text": format!("File contains {} dead functions and {} dead lines", 
                            file.dead_functions, file.dead_lines)},
                        "locations": [self.create_location(&file.path, 1, 1)],
                        "properties": {
                            "dead_functions": file.dead_functions,
                            "dead_lines": file.dead_lines,
                            "dead_code_percentage": file.dead_lines as f64 / file.total_lines.max(1) as f64 * 100.0
                        }
                    }))
            );
        }
    }

    /// Helper to create location objects
    fn create_location(&self, uri: &str, start_line: usize, end_line: usize) -> serde_json::Value {
        serde_json::json!({
            "physicalLocation": {
                "artifactLocation": {"uri": uri},
                "region": {
                    "startLine": start_line,
                    "startColumn": 1,
                    "endLine": end_line
                }
            }
        })
    }

    /// Convert SATD severity to SARIF level
    fn satd_severity_to_level(
        &self,
        severity: &crate::services::satd_detector::Severity,
    ) -> &'static str {
        match severity {
            crate::services::satd_detector::Severity::Critical => "error",
            crate::services::satd_detector::Severity::High => "warning",
            crate::services::satd_detector::Severity::Medium => "note",
            crate::services::satd_detector::Severity::Low => "note",
        }
    }

    fn overall_health_emoji(&self, health: f64) -> &'static str {
        if health >= 80.0 {
            "✅"
        } else if health >= 60.0 {
            "⚠️"
        } else {
            "❌"
        }
    }

    fn format_annotated_tree(
        &self,
        output: &mut String,
        tree: &AnnotatedFileTree,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        self.format_tree_node(output, &tree.root, "", true)?;
        writeln!(
            output,
            "\n📊 Total Files: {}, Total Size: {} bytes",
            tree.total_files, tree.total_size_bytes
        )?;
        Ok(())
    }

    #[allow(clippy::only_used_in_recursion)]
    fn format_tree_node(
        &self,
        output: &mut String,
        node: &AnnotatedNode,
        prefix: &str,
        is_last: bool,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;

        let connector = if is_last { "└── " } else { "├── " };
        let extension = if is_last { "    " } else { "│   " };

        let node_display = self.format_node_display(node)?;
        writeln!(output, "{prefix}{connector}{node_display}")?;

        // Process children
        let child_prefix = format!("{prefix}{extension}");
        for (i, child) in node.children.iter().enumerate() {
            let is_last_child = i == node.children.len() - 1;
            self.format_tree_node(output, child, &child_prefix, is_last_child)?;
        }

        Ok(())
    }

    fn format_node_display(&self, node: &AnnotatedNode) -> anyhow::Result<String> {
        let mut display = node.name.clone();

        if matches!(node.node_type, NodeType::Directory) {
            display.push('/');
        }

        let annotations = self.collect_node_annotations(&node.annotations);
        if !annotations.is_empty() {
            display.push_str(&format!(" [{}]", annotations.join(" ")));
        }

        Ok(display)
    }

    fn collect_node_annotations(&self, annotations: &NodeAnnotations) -> Vec<String> {
        let mut result = Vec::new();

        // Defect score
        if let Some(score) = annotations.defect_score {
            self.add_defect_indicator(&mut result, score);
        }

        // Cognitive complexity
        if let Some(complexity) = annotations.cognitive_complexity {
            self.add_cognitive_complexity_indicator(&mut result, complexity);
        }

        // SATD items
        if annotations.satd_items > 0 {
            result.push(format!("📝{}", annotations.satd_items));
        }

        // Dead code items
        if annotations.dead_code_items > 0 {
            result.push(format!("💀{}", annotations.dead_code_items));
        }

        // Test coverage
        if let Some(coverage) = annotations.test_coverage {
            self.add_coverage_indicator(&mut result, coverage);
        }

        // Big-O complexity
        if let Some(ref big_o) = annotations.big_o_complexity {
            let emoji = self.get_big_o_emoji(big_o);
            result.push(format!("{emoji}{big_o}"));
        }

        // Churn score
        if let Some(churn) = annotations.churn_score {
            self.add_churn_indicator(&mut result, churn);
        }

        // Memory complexity
        if let Some(ref mem_complexity) = annotations.memory_complexity {
            self.add_memory_complexity_indicator(&mut result, mem_complexity);
        }

        // Duplication score
        if let Some(duplication) = annotations.duplication_score {
            self.add_duplication_indicator(&mut result, duplication);
        }

        result
    }

    /// Add defect score indicator
    fn add_defect_indicator(&self, result: &mut Vec<String>, score: f32) {
        if score > 0.7 {
            result.push(format!("🔴{score:.1}"));
        } else if score > 0.4 {
            result.push(format!("🟡{score:.1}"));
        }
    }

    /// Add cognitive complexity indicator
    fn add_cognitive_complexity_indicator(&self, result: &mut Vec<String>, complexity: u16) {
        if complexity > 30 {
            result.push(format!("🧠{complexity}"));
        } else if complexity > 15 {
            result.push(format!("🧪{complexity}"));
        }
    }

    /// Add test coverage indicator
    fn add_coverage_indicator(&self, result: &mut Vec<String>, coverage: f32) {
        if coverage < 0.5 {
            result.push(format!("🚨{:.0}%", coverage * 100.0));
        } else if coverage < 0.8 {
            result.push(format!("⚠️{:.0}%", coverage * 100.0));
        } else {
            result.push(format!("✅{:.0}%", coverage * 100.0));
        }
    }

    /// Add churn indicator
    fn add_churn_indicator(&self, result: &mut Vec<String>, churn: f32) {
        if churn > 0.8 {
            result.push(format!("🔥{churn:.1}")); // High churn - hot file
        } else if churn > 0.5 {
            result.push(format!("🌡️{churn:.1}")); // Medium churn
        } else if churn > 0.2 {
            result.push(format!("🌊{churn:.1}")); // Low churn
        }
    }

    /// Add memory complexity indicator
    fn add_memory_complexity_indicator(&self, result: &mut Vec<String>, mem_complexity: &str) {
        let emoji = match mem_complexity {
            "O(1)" => "💎",       // Constant memory - excellent
            "O(log n)" => "💚",   // Logarithmic memory - very good
            "O(n)" => "💙",       // Linear memory - good
            "O(n log n)" => "💛", // Linearithmic memory - okay
            "O(n²)" => "🟠",      // Quadratic memory - warning
            _ => "💔",            // High memory usage - critical
        };
        result.push(format!("{emoji}{mem_complexity}"));
    }

    /// Add duplication indicator
    fn add_duplication_indicator(&self, result: &mut Vec<String>, duplication: f32) {
        if duplication > 0.3 {
            result.push(format!("📑{:.0}%", duplication * 100.0)); // High duplication
        } else if duplication > 0.1 {
            result.push(format!("📄{:.0}%", duplication * 100.0)); // Medium duplication
        }
    }

    /// Get emoji for Big-O complexity notation
    fn get_big_o_emoji(&self, big_o: &str) -> &'static str {
        match big_o {
            "O(1)" => "🎯",            // Constant - excellent
            "O(log n)" => "⚡",        // Logarithmic - very good
            "O(n)" => "📊",            // Linear - good
            "O(n log n)" => "📈",      // Linearithmic - acceptable
            "O(n²)" => "⚠️",           // Quadratic - warning
            "O(n³)" => "🚨",           // Cubic - danger
            "O(2ⁿ)" | "O(n!)" => "💥", // Exponential/Factorial - critical
            _ => "❓",                 // Unknown
        }
    }

    pub fn format_enhanced_ast_section(
        &self,
        output: &mut String,
        ast_contexts: &[EnhancedFileContext],
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        writeln!(output, "## Enhanced AST Analysis\n")?;

        for context in ast_contexts {
            self.format_single_file_ast(output, context)?;
        }

        Ok(())
    }

    fn format_single_file_ast(
        &self,
        output: &mut String,
        context: &EnhancedFileContext,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;

        writeln!(output, "### {}\n", context.base.path)?;
        writeln!(output, "**Language:** {}", context.base.language)?;
        writeln!(output, "**Total Symbols:** {}", context.base.items.len())?;

        // Categorize AST items
        let categorized_items = self.categorize_ast_items(&context.base.items);

        // Write summary counts
        self.write_ast_summary(output, &categorized_items)?;

        // Write detailed breakdowns
        self.write_ast_details(output, &categorized_items)?;

        // Write metrics
        self.write_file_metrics(output, context)?;

        Ok(())
    }

    fn categorize_ast_items(
        &self,
        items: &[crate::services::context::AstItem],
    ) -> CategorizedAstItems {
        let mut categorized = CategorizedAstItems::new();

        for item in items {
            self.categorize_single_ast_item(item, &mut categorized);
        }

        categorized
    }

    fn categorize_single_ast_item(
        &self,
        item: &crate::services::context::AstItem,
        categorized: &mut CategorizedAstItems,
    ) {
        match item {
            crate::services::context::AstItem::Function {
                name,
                visibility,
                is_async,
                line,
            } => {
                categorized.functions.push(AstFunction {
                    name: name.clone(),
                    visibility: visibility.clone(),
                    is_async: *is_async,
                    line: *line,
                });
            }
            crate::services::context::AstItem::Struct {
                name,
                visibility,
                fields_count,
                derives,
                line,
            } => {
                categorized.structs.push(AstStruct {
                    name: name.clone(),
                    visibility: visibility.clone(),
                    fields_count: *fields_count,
                    derives: derives.clone(),
                    line: *line,
                });
            }
            crate::services::context::AstItem::Enum {
                name,
                visibility,
                variants_count,
                line,
            } => {
                categorized.enums.push(AstEnum {
                    name: name.clone(),
                    visibility: visibility.clone(),
                    variants_count: *variants_count,
                    line: *line,
                });
            }
            crate::services::context::AstItem::Trait {
                name,
                visibility,
                line,
            } => {
                categorized.traits.push(AstTrait {
                    name: name.clone(),
                    visibility: visibility.clone(),
                    line: *line,
                });
            }
            crate::services::context::AstItem::Impl {
                type_name,
                trait_name,
                line,
            } => {
                categorized.impls.push(AstImpl {
                    type_name: type_name.clone(),
                    trait_name: trait_name.clone(),
                    line: *line,
                });
            }
            crate::services::context::AstItem::Module {
                name,
                visibility,
                line,
            } => {
                categorized.modules.push(AstModule {
                    name: name.clone(),
                    visibility: visibility.clone(),
                    line: *line,
                });
            }
            crate::services::context::AstItem::Use { path, line } => {
                categorized.uses.push(AstUse {
                    path: path.clone(),
                    line: *line,
                });
            }
            crate::services::context::AstItem::Import {
                module,
                items,
                alias,
                line,
            } => {
                let path = self.format_import_path(module, items, alias);
                categorized.uses.push(AstUse { path, line: *line });
            }
        }
    }

    fn format_import_path(&self, module: &str, items: &[String], alias: &Option<String>) -> String {
        if let Some(alias) = alias {
            format!("{module} as {alias}")
        } else if !items.is_empty() {
            format!("{} ({})", module, items.join(", "))
        } else {
            module.to_string()
        }
    }

    fn write_ast_summary(
        &self,
        output: &mut String,
        items: &CategorizedAstItems,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        writeln!(output, "**Functions:** {} | **Structs:** {} | **Enums:** {} | **Traits:** {} | **Impls:** {} | **Modules:** {} | **Imports:** {}",
            items.functions.len(), items.structs.len(), items.enums.len(),
            items.traits.len(), items.impls.len(), items.modules.len(), items.uses.len())?;
        Ok(())
    }

    fn write_ast_details(
        &self,
        output: &mut String,
        items: &CategorizedAstItems,
    ) -> anyhow::Result<()> {
        self.write_functions_section(output, &items.functions)?;
        self.write_structs_section(output, &items.structs)?;
        self.write_enums_section(output, &items.enums)?;
        self.write_traits_section(output, &items.traits)?;
        self.write_impls_section(output, &items.impls)?;
        self.write_modules_section(output, &items.modules)?;
        self.write_imports_section(output, &items.uses)?;
        Ok(())
    }

    fn write_functions_section(
        &self,
        output: &mut String,
        functions: &[AstFunction],
    ) -> anyhow::Result<()> {
        if functions.is_empty() {
            return Ok(());
        }

        use std::fmt::Write;
        writeln!(output, "\n**Functions:**")?;

        for func in functions.iter().take(10) {
            let async_marker = if func.is_async { " (async)" } else { "" };
            writeln!(
                output,
                "  - `{}{}` ({}) at line {}",
                func.name, async_marker, func.visibility, func.line
            )?;
        }

        if functions.len() > 10 {
            writeln!(
                output,
                "  - ... and {} more functions",
                functions.len() - 10
            )?;
        }

        Ok(())
    }

    fn write_structs_section(
        &self,
        output: &mut String,
        structs: &[AstStruct],
    ) -> anyhow::Result<()> {
        if structs.is_empty() {
            return Ok(());
        }

        use std::fmt::Write;
        writeln!(output, "\n**Structs:**")?;

        for struct_item in structs.iter().take(5) {
            let derives_str = if struct_item.derives.is_empty() {
                String::with_capacity(1024)
            } else {
                format!(" (derives: {})", struct_item.derives.join(", "))
            };
            let field_plural = if struct_item.fields_count == 1 {
                ""
            } else {
                "s"
            };
            writeln!(
                output,
                "  - `{}` ({}) with {} field{}{} at line {}",
                struct_item.name,
                struct_item.visibility,
                struct_item.fields_count,
                field_plural,
                derives_str,
                struct_item.line
            )?;
        }

        if structs.len() > 5 {
            writeln!(output, "  - ... and {} more structs", structs.len() - 5)?;
        }

        Ok(())
    }

    fn write_enums_section(&self, output: &mut String, enums: &[AstEnum]) -> anyhow::Result<()> {
        if enums.is_empty() {
            return Ok(());
        }

        use std::fmt::Write;
        writeln!(output, "\n**Enums:**")?;

        for enum_item in enums.iter().take(5) {
            let variant_plural = if enum_item.variants_count == 1 {
                ""
            } else {
                "s"
            };
            writeln!(
                output,
                "  - `{}` ({}) with {} variant{} at line {}",
                enum_item.name,
                enum_item.visibility,
                enum_item.variants_count,
                variant_plural,
                enum_item.line
            )?;
        }

        if enums.len() > 5 {
            writeln!(output, "  - ... and {} more enums", enums.len() - 5)?;
        }

        Ok(())
    }

    fn write_traits_section(&self, output: &mut String, traits: &[AstTrait]) -> anyhow::Result<()> {
        if traits.is_empty() {
            return Ok(());
        }

        use std::fmt::Write;
        writeln!(output, "\n**Traits:**")?;

        for trait_item in traits.iter().take(5) {
            writeln!(
                output,
                "  - `{}` ({}) at line {}",
                trait_item.name, trait_item.visibility, trait_item.line
            )?;
        }

        if traits.len() > 5 {
            writeln!(output, "  - ... and {} more traits", traits.len() - 5)?;
        }

        Ok(())
    }

    fn write_impls_section(&self, output: &mut String, impls: &[AstImpl]) -> anyhow::Result<()> {
        if impls.is_empty() {
            return Ok(());
        }

        use std::fmt::Write;
        writeln!(output, "\n**Implementations:**")?;

        for impl_item in impls.iter().take(5) {
            if let Some(trait_name) = &impl_item.trait_name {
                writeln!(
                    output,
                    "  - `{} for {}` at line {}",
                    trait_name, impl_item.type_name, impl_item.line
                )?;
            } else {
                writeln!(
                    output,
                    "  - `impl {}` at line {}",
                    impl_item.type_name, impl_item.line
                )?;
            }
        }

        if impls.len() > 5 {
            writeln!(
                output,
                "  - ... and {} more implementations",
                impls.len() - 5
            )?;
        }

        Ok(())
    }

    fn write_modules_section(
        &self,
        output: &mut String,
        modules: &[AstModule],
    ) -> anyhow::Result<()> {
        if modules.is_empty() {
            return Ok(());
        }

        use std::fmt::Write;
        writeln!(output, "\n**Modules:**")?;

        for module_item in modules.iter().take(5) {
            writeln!(
                output,
                "  - `{}` ({}) at line {}",
                module_item.name, module_item.visibility, module_item.line
            )?;
        }

        if modules.len() > 5 {
            writeln!(output, "  - ... and {} more modules", modules.len() - 5)?;
        }

        Ok(())
    }

    fn write_imports_section(&self, output: &mut String, uses: &[AstUse]) -> anyhow::Result<()> {
        if uses.is_empty() {
            return Ok(());
        }

        use std::fmt::Write;

        if uses.len() <= 8 {
            writeln!(output, "\n**Key Imports:**")?;
            for use_item in uses.iter().take(8) {
                writeln!(output, "  - `{}` at line {}", use_item.path, use_item.line)?;
            }
        } else {
            writeln!(output, "\n**Imports:** {} import statements", uses.len())?;
        }

        Ok(())
    }

    fn write_file_metrics(
        &self,
        output: &mut String,
        context: &EnhancedFileContext,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;

        // Complexity metrics if available
        if let Some(ref complexity) = context.complexity_metrics {
            writeln!(output, "\n**Complexity Metrics:**")?;
            writeln!(
                output,
                "  - Cyclomatic: {:.1} | Cognitive: {:.1} | Lines: {}",
                complexity.total_complexity.cyclomatic,
                complexity.total_complexity.cognitive,
                complexity.total_complexity.lines
            )?;
        }

        // Churn metrics if available
        if let Some(ref churn) = context.churn_metrics {
            writeln!(output, "\n**Code Churn:**")?;
            writeln!(
                output,
                "  - {} commits by {} authors",
                churn.commits, churn.authors
            )?;
        }

        // TDG Score
        if let Some(ref tdg) = context.defects.tdg_score {
            writeln!(output, "\n**Code Quality Gradient:** {:.2}\n", tdg.value)?;
            writeln!(
                output,
                "**TDG Severity:** {:?}\n",
                TDGSeverity::from(tdg.value)
            )?;
        }

        Ok(())
    }

    fn format_complexity_hotspots(
        &self,
        output: &mut String,
        context: &DeepContext,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        if let Some(ref complexity) = context.analyses.complexity_report {
            writeln!(output, "## Complexity Hotspots\n")?;

            // Find top 10 most complex functions
            let mut all_functions: Vec<_> = complexity
                .files
                .par_iter()
                .flat_map(|f| f.functions.par_iter().map(move |func| (f, func)))
                .collect();
            all_functions.sort_by_key(|(_, func)| std::cmp::Reverse(func.metrics.cyclomatic));

            writeln!(output, "| Function | File | Cyclomatic | Cognitive |")?;
            writeln!(output, "|----------|------|------------|-----------|")?;

            for (file, func) in all_functions.iter().take(10) {
                writeln!(
                    output,
                    "| `{}` | `{}` | {} | {} |",
                    func.name, file.path, func.metrics.cyclomatic, func.metrics.cognitive
                )?;
            }
            writeln!(output)?;
        }

        Ok(())
    }

    fn format_churn_analysis(
        &self,
        output: &mut String,
        context: &DeepContext,
    ) -> anyhow::Result<()> {
        if let Some(ref churn) = context.analyses.churn_analysis {
            self.write_churn_header(output)?;
            self.write_churn_summary(output, churn)?;
            self.write_churn_files_table(output, &churn.files)?;
        }
        Ok(())
    }

    fn write_churn_header(&self, output: &mut String) -> anyhow::Result<()> {
        use std::fmt::Write;
        writeln!(output, "## Code Churn Analysis\n")?;
        Ok(())
    }

    fn write_churn_summary(
        &self,
        output: &mut String,
        churn: &CodeChurnAnalysis,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        writeln!(output, "**Summary:**")?;
        writeln!(output, "- Total Commits: {}", churn.summary.total_commits)?;
        writeln!(output, "- Files Changed: {}", churn.files.len())?;
        Ok(())
    }

    fn write_churn_files_table(
        &self,
        output: &mut String,
        files: &[crate::models::churn::FileChurnMetrics],
    ) -> anyhow::Result<()> {
        use std::fmt::Write;

        // Sort files by commit count
        let mut sorted_files = files.to_vec();
        sorted_files.sort_by_key(|f| std::cmp::Reverse(f.commit_count));

        writeln!(output, "\n**Top Changed Files:**")?;
        writeln!(output, "| File | Commits | Authors |")?;
        writeln!(output, "|------|---------|---------|")?;

        for file in sorted_files.iter().take(10) {
            writeln!(
                output,
                "| `{}` | {} | {} |",
                file.relative_path,
                file.commit_count,
                file.unique_authors.len()
            )?;
        }
        writeln!(output)?;
        Ok(())
    }

    fn format_technical_debt(
        &self,
        output: &mut String,
        context: &DeepContext,
    ) -> anyhow::Result<()> {
        if let Some(ref satd) = context.analyses.satd_results {
            use std::fmt::Write;
            writeln!(output, "## Code Quality Analysis\n")?;
            self.write_satd_severity_summary(output, satd)?;
            self.write_critical_items(output, satd)?;
            writeln!(output)?;
        }
        Ok(())
    }

    fn write_satd_severity_summary(
        &self,
        output: &mut String,
        satd: &crate::services::satd_detector::SATDAnalysisResult,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        let by_severity = self.group_satd_by_severity(satd);
        writeln!(output, "**SATD Summary:**")?;
        for (severity, count) in by_severity {
            writeln!(output, "- {severity:?}: {count}")?;
        }
        Ok(())
    }

    fn group_satd_by_severity<'a>(
        &self,
        satd: &'a crate::services::satd_detector::SATDAnalysisResult,
    ) -> FxHashMap<&'a crate::services::satd_detector::Severity, i32> {
        let mut by_severity = FxHashMap::default();
        for item in &satd.items {
            *by_severity.entry(&item.severity).or_insert(0) += 1;
        }
        by_severity
    }

    fn write_critical_items(
        &self,
        output: &mut String,
        satd: &crate::services::satd_detector::SATDAnalysisResult,
    ) -> anyhow::Result<()> {
        let critical_items = self.get_critical_satd_items(satd);
        if critical_items.is_empty() {
            return Ok(());
        }

        use std::fmt::Write;
        writeln!(output, "\n**Critical Items:**")?;
        for item in critical_items {
            writeln!(
                output,
                "- `{}:{} {}`: {}",
                item.file.display(),
                item.line,
                item.category,
                item.text.trim()
            )?;
        }
        Ok(())
    }

    fn get_critical_satd_items<'a>(
        &self,
        satd: &'a crate::services::satd_detector::SATDAnalysisResult,
    ) -> Vec<&'a crate::services::satd_detector::TechnicalDebt> {
        satd.items
            .iter()
            .filter(|item| {
                matches!(
                    item.severity,
                    crate::services::satd_detector::Severity::Critical
                )
            })
            .take(5)
            .collect()
    }

    fn format_dead_code_analysis(
        &self,
        output: &mut String,
        context: &DeepContext,
    ) -> anyhow::Result<()> {
        if let Some(ref dead_code) = context.analyses.dead_code_results {
            self.write_dead_code_header(output)?;
            self.write_dead_code_summary(output, &dead_code.summary)?;
            self.write_dead_code_files_table(output, &dead_code.ranked_files)?;
        }
        Ok(())
    }

    fn write_dead_code_header(&self, output: &mut String) -> anyhow::Result<()> {
        use std::fmt::Write;
        writeln!(output, "## Dead Code Analysis\n")?;
        Ok(())
    }

    fn write_dead_code_summary(
        &self,
        output: &mut String,
        summary: &crate::models::dead_code::DeadCodeSummary,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        writeln!(output, "**Summary:**")?;
        writeln!(output, "- Dead Functions: {}", summary.dead_functions)?;
        writeln!(output, "- Total Dead Lines: {}", summary.total_dead_lines)?;
        Ok(())
    }

    fn write_dead_code_files_table(
        &self,
        output: &mut String,
        files: &[crate::models::dead_code::FileDeadCodeMetrics],
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        if !files.is_empty() {
            writeln!(output, "\n**Top Files with Dead Code:**")?;
            writeln!(output, "| File | Dead Lines | Dead Functions |")?;
            writeln!(output, "|------|------------|----------------|")?;

            for file in files.iter().take(10) {
                writeln!(
                    output,
                    "| `{}` | {} | {} |",
                    file.path, file.dead_lines, file.dead_functions
                )?;
            }
            writeln!(output)?;
        }
        Ok(())
    }

    fn format_cross_references(
        &self,
        output: &mut String,
        cross_refs: &[CrossLangReference],
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        if !cross_refs.is_empty() {
            writeln!(output, "## Cross-Language References\n")?;

            writeln!(output, "| Source | Target | Type | Confidence |")?;
            writeln!(output, "|--------|--------|------|------------|")?;

            for cross_ref in cross_refs {
                writeln!(
                    output,
                    "| `{}` | `{}` | {:?} | {:.1}% |",
                    cross_ref.source_file.display(),
                    cross_ref.target_file.display(),
                    cross_ref.reference_type,
                    cross_ref.confidence * 100.0
                )?;
            }
            writeln!(output)?;
        }

        Ok(())
    }

    fn format_defect_predictions(
        &self,
        output: &mut String,
        context: &DeepContext,
    ) -> anyhow::Result<()> {
        self.write_defect_header(output)?;
        self.write_defect_summary(output, &context.defect_summary)?;
        self.write_defect_hotspots_table(output, &context.hotspots)?;
        Ok(())
    }

    fn write_defect_header(&self, output: &mut String) -> anyhow::Result<()> {
        use std::fmt::Write;
        writeln!(output, "## Defect Probability Analysis\n")?;
        Ok(())
    }

    fn write_defect_summary(
        &self,
        output: &mut String,
        summary: &DefectSummary,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        writeln!(output, "**Risk Assessment:**")?;
        writeln!(
            output,
            "- Total Defects Predicted: {}",
            summary.total_defects
        )?;
        writeln!(
            output,
            "- Defect Density: {:.2} defects per 1000 lines",
            summary.defect_density
        )?;
        Ok(())
    }

    fn write_defect_hotspots_table(
        &self,
        output: &mut String,
        hotspots: &[DefectHotspot],
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        if !hotspots.is_empty() {
            writeln!(output, "\n**High-Risk Hotspots:**")?;
            writeln!(output, "| File:Line | Risk Score | Effort (hours) |")?;
            writeln!(output, "|-----------|------------|----------------|")?;

            for hotspot in hotspots.iter().take(10) {
                writeln!(
                    output,
                    "| `{}:{}` | {:.1} | {:.1} |",
                    hotspot.location.file.display(),
                    hotspot.location.line,
                    hotspot.composite_score,
                    hotspot.refactoring_effort.estimated_hours
                )?;
            }
        }
        writeln!(output)?;
        Ok(())
    }

    fn format_prioritized_recommendations(
        &self,
        output: &mut String,
        recommendations: &[PrioritizedRecommendation],
    ) -> anyhow::Result<()> {
        if recommendations.is_empty() {
            return Ok(());
        }

        self.write_recommendations_header(output)?;

        for (i, rec) in recommendations.iter().enumerate() {
            self.write_single_recommendation(output, i, rec)?;
        }

        Ok(())
    }

    fn write_recommendations_header(&self, output: &mut String) -> anyhow::Result<()> {
        use std::fmt::Write;
        writeln!(output, "## Prioritized Recommendations\n")?;
        Ok(())
    }

    fn write_single_recommendation(
        &self,
        output: &mut String,
        index: usize,
        rec: &PrioritizedRecommendation,
    ) -> anyhow::Result<()> {
        let priority_emoji = self.get_priority_emoji(&rec.priority);
        self.write_recommendation_title(output, priority_emoji, index + 1, &rec.title)?;
        self.write_recommendation_details(output, rec)?;
        self.write_recommendation_prerequisites(output, &rec.prerequisites)?;
        Ok(())
    }

    fn get_priority_emoji(&self, priority: &Priority) -> &'static str {
        match priority {
            Priority::Critical => "🔴",
            Priority::High => "🟡",
            Priority::Medium => "🔵",
            Priority::Low => "⚪",
        }
    }

    fn write_recommendation_title(
        &self,
        output: &mut String,
        emoji: &str,
        number: usize,
        title: &str,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        writeln!(output, "### {emoji} {number} {title}")?;
        Ok(())
    }

    fn write_recommendation_details(
        &self,
        output: &mut String,
        rec: &PrioritizedRecommendation,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        writeln!(output, "**Description:** {}", rec.description)?;
        writeln!(output, "**Effort:** {:?}", rec.estimated_effort)?;
        writeln!(output, "**Impact:** {:?}", rec.impact)?;
        Ok(())
    }

    fn write_recommendation_prerequisites(
        &self,
        output: &mut String,
        prerequisites: &[String],
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        if !prerequisites.is_empty() {
            writeln!(output, "**Prerequisites:**")?;
            for prereq in prerequisites {
                writeln!(output, "- {prereq}")?;
            }
        }
        writeln!(output)?;
        Ok(())
    }
}
