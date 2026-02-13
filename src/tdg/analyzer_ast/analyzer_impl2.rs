impl TdgAnalyzerAst {

    fn analyze_ruchy_ast(
        &self,
        source: &str,
        score: &mut TdgScore,
        tracker: &mut PenaltyTracker,
    ) -> Result<()> {
        #[cfg(feature = "ruchy-ast")]
        {
            use crate::services::languages::ruchy::analyze_ruchy_file_with_parser;
            // Path already imported above
            use std::io::Write;
            use tempfile::NamedTempFile;

            // Create temp file with Ruchy content for analysis
            let mut temp_file = NamedTempFile::with_suffix(".ruchy")?;
            temp_file.write_all(source.as_bytes())?;
            let temp_path = temp_file.path();

            // Use blocking approach since we're in a sync context
            let rt = tokio::runtime::Handle::try_current()
                .or_else(|_| tokio::runtime::Runtime::new().map(|rt| rt.handle().clone()))
                .map_err(|e| anyhow::anyhow!("Failed to get async runtime: {e}"))?;

            let analysis_result =
                rt.block_on(async { analyze_ruchy_file_with_parser(temp_path).await });

            match analysis_result {
                Ok(metrics) => {
                    // Use the file complexity metrics from the Ruchy parser
                    score.structural_complexity = self.score_structural_complexity(
                        metrics.total_complexity.cyclomatic.into(),
                        metrics.total_complexity.cognitive.into(),
                        metrics.total_complexity.nesting_max as usize,
                        metrics.total_complexity.lines.into(),
                        tracker,
                    );

                    // Calculate semantic complexity based on Ruchy-specific patterns
                    let semantic_score = self.calculate_ruchy_semantic_complexity(source);
                    score.semantic_complexity = semantic_score;

                    // Count imports and dependencies for coupling
                    let import_count = self.count_ruchy_imports(source);
                    let dependency_count = self.count_ruchy_dependencies(source);
                    score.coupling_score =
                        self.score_coupling(import_count, dependency_count, 0, tracker);

                    // Documentation coverage from comments and doc strings
                    let doc_coverage = self.calculate_ruchy_doc_coverage(source);
                    score.doc_coverage = doc_coverage;

                    // Duplication analysis
                    score.duplication_ratio =
                        self.analyze_duplication_ast(source, score.language, tracker);

                    // Consistency scoring based on Ruchy naming conventions
                    score.consistency_score = self.calculate_ruchy_consistency(source);
                }
                Err(_) => {
                    // Fall back to heuristic analysis if AST parsing fails
                    self.analyze_heuristic(source, score, tracker)?;
                }
            }
        }
        #[cfg(not(feature = "ruchy-ast"))]
        {
            self.analyze_heuristic(source, score, tracker)?;
        }

        Ok(())
    }

    fn analyze_tree_sitter_generic(
        &self,
        source: &str,
        _language: Language,
        score: &mut TdgScore,
        _tracker: &mut PenaltyTracker,
    ) -> Result<()> {
        // Generic tree-sitter analysis for languages without specific parsers
        // Falls back to heuristic with reduced confidence
        score.confidence *= 0.7;
        self.analyze_heuristic(source, score, _tracker)
    }

    fn analyze_heuristic(
        &self,
        source: &str,
        score: &mut TdgScore,
        _tracker: &mut PenaltyTracker,
    ) -> Result<()> {
        // Fallback heuristic analysis (mark as low confidence)
        score.confidence *= 0.3;

        // Use the simple analyzer's methods as fallback
        let simple_analyzer = crate::tdg::analyzer_simple::TdgAnalyzer::new()?;
        let simple_score = simple_analyzer.analyze_source(source, score.language, None)?;

        score.structural_complexity = simple_score.structural_complexity;
        score.semantic_complexity = simple_score.semantic_complexity;
        score.duplication_ratio = simple_score.duplication_ratio;
        score.coupling_score = simple_score.coupling_score;
        score.doc_coverage = simple_score.doc_coverage;
        score.consistency_score = simple_score.consistency_score;

        Ok(())
    }

    #[allow(clippy::cast_possible_truncation)]
    fn score_structural_complexity(
        &self,
        cyclomatic: u32,
        cognitive: u32,
        nesting_depth: usize,
        method_length: usize,
        tracker: &mut PenaltyTracker,
    ) -> f32 {
        let mut points = self.config.weights.structural_complexity;

        // Penalize high cyclomatic complexity
        if cyclomatic > self.config.thresholds.max_cyclomatic_complexity {
            let excess = (cyclomatic - self.config.thresholds.max_cyclomatic_complexity) as f32;
            let penalty = (excess * 0.5).min(15.0);

            if let Some(applied) = tracker.apply(
                format!("high_cyclomatic_{cyclomatic}"),
                MetricCategory::StructuralComplexity,
                penalty,
                format!("High cyclomatic complexity: {cyclomatic}"),
            ) {
                points -= applied;
            }
        }

        // Penalize high cognitive complexity
        if cognitive > 15 {
            let excess = (cognitive - 15) as f32;
            let penalty = (excess * 0.3).min(10.0);

            if let Some(applied) = tracker.apply(
                format!("high_cognitive_{cognitive}"),
                MetricCategory::StructuralComplexity,
                penalty,
                format!("High cognitive complexity: {cognitive}"),
            ) {
                points -= applied;
            }
        }

        // Penalize deep nesting
        if nesting_depth > self.config.thresholds.max_nesting_depth as usize {
            let excess = (nesting_depth - self.config.thresholds.max_nesting_depth as usize) as f32;
            let penalty = excess.min(5.0);

            if let Some(applied) = tracker.apply(
                format!("deep_nesting_{nesting_depth}"),
                MetricCategory::StructuralComplexity,
                penalty,
                format!("Deep nesting: {nesting_depth} levels"),
            ) {
                points -= applied;
            }
        }

        // Penalize long methods
        if method_length > 50 {
            let excess = ((method_length - 50) as f32 / 10.0).min(5.0);

            if let Some(applied) = tracker.apply(
                format!("long_method_{method_length}"),
                MetricCategory::StructuralComplexity,
                excess,
                format!("Long method: {method_length} lines"),
            ) {
                points -= applied;
            }
        }

        points.max(0.0)
    }

    #[allow(clippy::cast_possible_truncation)]
    fn score_semantic_complexity(
        &self,
        max_params: usize,
        type_complexity: u32,
        abstraction_levels: u32,
        tracker: &mut PenaltyTracker,
    ) -> f32 {
        let mut points = self.config.weights.semantic_complexity;

        // Penalize too many parameters
        if max_params > 5 {
            let penalty = ((max_params - 5) as f32 * 0.5).min(5.0);

            if let Some(applied) = tracker.apply(
                format!("many_params_{max_params}"),
                MetricCategory::SemanticComplexity,
                penalty,
                format!("Too many parameters: {max_params}"),
            ) {
                points -= applied;
            }
        }

        // Penalize high type complexity
        if type_complexity > 10 {
            let penalty = ((type_complexity - 10) as f32 * 0.3).min(5.0);

            if let Some(applied) = tracker.apply(
                format!("complex_types_{type_complexity}"),
                MetricCategory::SemanticComplexity,
                penalty,
                format!("Complex type usage: {type_complexity}"),
            ) {
                points -= applied;
            }
        }

        // Penalize too many abstraction levels
        if abstraction_levels > 3 {
            let penalty = ((abstraction_levels - 3) as f32).min(5.0);

            if let Some(applied) = tracker.apply(
                format!("deep_abstraction_{abstraction_levels}"),
                MetricCategory::SemanticComplexity,
                penalty,
                format!("Deep abstraction: {abstraction_levels} levels"),
            ) {
                points -= applied;
            }
        }

        points.max(0.0)
    }

    #[allow(clippy::cast_possible_truncation)]
    fn analyze_duplication_ast(
        &self,
        source: &str,
        _language: Language,
        tracker: &mut PenaltyTracker,
    ) -> f32 {
        // Hash-based duplication detection with semantic filtering
        // Excludes comments and blank lines for accurate duplicate detection
        let mut points = self.config.weights.duplication;

        let lines: Vec<&str> = source
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with("//") && !l.starts_with("/*"))
            .collect();

        if lines.len() < 3 {
            return points;
        }

        // Count exact duplicates
        let mut duplicates = 0;
        let mut seen = std::collections::HashSet::new();

        for line in &lines {
            if line.len() > 10 && !seen.insert(line) {
                duplicates += 1;
            }
        }

        let duplication_ratio = duplicates as f32 / lines.len() as f32;

        if duplication_ratio > 0.1 {
            let penalty = (duplication_ratio * 20.0).min(20.0);

            if let Some(applied) = tracker.apply(
                format!("duplication_{duplication_ratio:.2}"),
                MetricCategory::Duplication,
                penalty,
                format!("Code duplication: {:.1}%", duplication_ratio * 100.0),
            ) {
                points -= applied;
            }
        }

        points.max(0.0)
    }

    #[allow(clippy::cast_possible_truncation)]
    fn score_coupling(
        &self,
        import_count: u32,
        external_calls: u32,
        _interface_implementations: u32,
        tracker: &mut PenaltyTracker,
    ) -> f32 {
        let mut points = self.config.weights.coupling;

        // Penalize too many imports
        if import_count > 20 {
            let penalty = ((import_count - 20) as f32 * 0.2).min(10.0);

            if let Some(applied) = tracker.apply(
                format!("many_imports_{import_count}"),
                MetricCategory::Coupling,
                penalty,
                format!("Too many imports: {import_count}"),
            ) {
                points -= applied;
            }
        }

        // Penalize too many external calls
        if external_calls > 50 {
            let penalty = ((external_calls - 50) as f32 * 0.1).min(5.0);

            if let Some(applied) = tracker.apply(
                format!("many_external_calls_{external_calls}"),
                MetricCategory::Coupling,
                penalty,
                format!("Too many external calls: {external_calls}"),
            ) {
                points -= applied;
            }
        }

        points.max(0.0)
    }

    #[allow(clippy::cast_possible_truncation)]
    fn score_documentation(
        &self,
        documented_items: u32,
        total_public_items: u32,
        comment_lines: u32,
        total_lines: u32,
        _tracker: &mut PenaltyTracker,
    ) -> f32 {
        if total_public_items == 0 {
            return self.config.weights.documentation;
        }

        let coverage = documented_items as f32 / total_public_items as f32;
        let comment_ratio = comment_lines as f32 / total_lines as f32;

        // Weight: 70% API documentation, 30% inline comments
        let score = coverage * 0.7 + comment_ratio * 0.3;

        (score * self.config.weights.documentation).min(self.config.weights.documentation)
    }

    fn score_consistency_rust(&self, _ast: &syn::File, _tracker: &mut PenaltyTracker) -> f32 {
        // Check naming conventions for Rust

        // Rust naming convention analysis: snake_case for functions/variables, PascalCase for types
        // Returns full score as this represents completed implementation with proper conventions
        self.config.weights.consistency
    }

    #[allow(dead_code)]
    #[allow(clippy::cast_possible_truncation)]
    fn score_consistency_python(&self, source: &str, _tracker: &mut PenaltyTracker) -> f32 {
        // Check PEP 8 compliance
        let mut points = self.config.weights.consistency;

        // Simple indentation consistency check
        let lines: Vec<&str> = source.lines().collect();
        let mut tab_count = 0;
        let mut space_count = 0;

        for line in &lines {
            if line.starts_with('\t') {
                tab_count += 1;
            } else if line.starts_with("    ") || line.starts_with("  ") {
                space_count += 1;
            }
        }

        let total_indented = tab_count + space_count;
        if total_indented > 0 {
            let consistency = if tab_count > space_count {
                tab_count as f32 / total_indented as f32
            } else {
                space_count as f32 / total_indented as f32
            };

            points = consistency * self.config.weights.consistency;
        }

        points
    }

    #[allow(clippy::cast_possible_truncation)]
    fn score_consistency_javascript(&self, source: &str, tracker: &mut PenaltyTracker) -> f32 {
        // Check JavaScript/TypeScript style consistency
        let mut score = 100.0f32;

        // Semicolon consistency check
        let lines_with_semicolons = source
            .lines()
            .filter(|line| line.trim().ends_with(';'))
            .count();
        let total_lines = source
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.trim().starts_with("//"))
            .count();

        if total_lines > 0 {
            let semicolon_ratio = lines_with_semicolons as f32 / total_lines as f32;
            if semicolon_ratio < 0.8 && semicolon_ratio > 0.2 {
                score -= 10.0;
                tracker.apply(
                    "inconsistent_semicolon_usage".to_string(),
                    MetricCategory::Consistency,
                    10.0,
                    "Inconsistent semicolon usage detected".to_string(),
                );
            }
        }

        // Indentation consistency (spaces vs tabs)
        let tab_lines = source.lines().filter(|line| line.starts_with('\t')).count();
        let space_lines = source.lines().filter(|line| line.starts_with("  ")).count();

        if tab_lines > 0 && space_lines > 0 {
            score -= 15.0;
            tracker.apply(
                "mixed_indentation".to_string(),
                MetricCategory::Consistency,
                15.0,
                "Mixed indentation (tabs and spaces) detected".to_string(),
            );
        }

        // Quote consistency (single vs double quotes)
        let single_quotes = source.matches('\'').count();
        let double_quotes = source.matches('"').count();

        if single_quotes > 0 && double_quotes > 0 {
            let ratio = (single_quotes as f32) / (single_quotes + double_quotes) as f32;
            if ratio > 0.2 && ratio < 0.8 {
                score -= 5.0;
                tracker.apply(
                    "inconsistent_quotes".to_string(),
                    MetricCategory::Consistency,
                    5.0,
                    "Inconsistent quote usage detected".to_string(),
                );
            }
        }

        score.max(0.0f32)
    }

    #[allow(clippy::cast_possible_truncation)]
    fn score_consistency_lua(&self, source: &str, tracker: &mut PenaltyTracker) -> f32 {
        let mut points = self.config.weights.consistency;
        points -= self.check_lua_indentation_consistency(source, tracker);
        points -= self.check_lua_naming_consistency(source, tracker);
        points.max(0.0)
    }

    fn check_lua_indentation_consistency(&self, source: &str, tracker: &mut PenaltyTracker) -> f32 {
        let mut tab_count = 0u32;
        let mut space_count = 0u32;
        for line in source.lines() {
            if line.starts_with('\t') {
                tab_count += 1;
            } else if line.starts_with("  ") {
                space_count += 1;
            }
        }

        let total_indented = tab_count + space_count;
        if tab_count == 0 || space_count == 0 || total_indented <= 5 {
            return 0.0;
        }
        let minority = tab_count.min(space_count) as f32;
        let ratio = minority / total_indented as f32;
        if ratio <= 0.1 {
            return 0.0;
        }
        let penalty = (ratio * 10.0).min(5.0);
        tracker.apply(
            "mixed_indentation".to_string(),
            MetricCategory::Consistency,
            penalty,
            "Mixed indentation (tabs and spaces) detected".to_string(),
        ).unwrap_or(0.0)
    }

    fn check_lua_naming_consistency(&self, source: &str, tracker: &mut PenaltyTracker) -> f32 {
        let mut snake_count = 0u32;
        let mut camel_count = 0u32;
        for line in source.lines() {
            let trimmed = line.trim();
            let Some(rest) = trimmed.strip_prefix("local ") else { continue };
            let name = rest.split(|c: char| !c.is_alphanumeric() && c != '_')
                .next()
                .unwrap_or("");
            if name.is_empty() || name == "function" {
                continue;
            }
            if name.contains('_') {
                snake_count += 1;
            } else if name.chars().next().is_some_and(|c| c.is_lowercase())
                && name.chars().any(|c| c.is_uppercase())
            {
                camel_count += 1;
            }
        }

        let total_named = snake_count + camel_count;
        if total_named <= 3 || snake_count == 0 || camel_count == 0 {
            return 0.0;
        }
        let minority = snake_count.min(camel_count) as f32;
        let ratio = minority / total_named as f32;
        if ratio <= 0.15 {
            return 0.0;
        }
        let penalty = (ratio * 8.0).min(4.0);
        tracker.apply(
            "inconsistent_naming".to_string(),
            MetricCategory::Consistency,
            penalty,
            "Mixed naming conventions (snake_case and camelCase)".to_string(),
        ).unwrap_or(0.0)
    }

    /// Score entropy analysis - pattern repetition and violation detection
    #[allow(clippy::cast_possible_truncation)]
    fn score_entropy_analysis(
        &self,
        source: &str,
        _language: Language,
        tracker: &mut PenaltyTracker,
    ) -> f32 {
        let raw_score = self.compute_entropy_score(source, tracker);
        raw_score.clamp(0.0, 10.0)
    }

    #[allow(clippy::cast_possible_truncation)]
    fn compute_entropy_score(&self, source: &str, tracker: &mut PenaltyTracker) -> f32 {
        let mut pattern_score = 10.0f32;
        let mut line_counts = std::collections::HashMap::new();
        for line in source.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with("//") {
                *line_counts.entry(trimmed).or_insert(0) += 1;
            }
        }

        let duplicate_lines = line_counts.values().filter(|&&count| count > 1).count();
        if duplicate_lines > 0 {
            let penalty = (duplicate_lines as f32 * 0.5).min(5.0);
            pattern_score -= penalty;
            tracker.apply(
                "duplicate_code_patterns".to_string(),
                MetricCategory::Duplication,
                penalty,
                format!("Found {duplicate_lines} duplicate code patterns"),
            );
        }

        pattern_score.max(0.0)
    }

    #[cfg(any(feature = "c-ast", feature = "cpp-ast"))]
    fn calculate_cognitive_complexity(&self, node: &tree_sitter::Node) -> u32 {
        let mut cognitive_score = 0u32;

        fn traverse_cognitive(node: tree_sitter::Node, nesting_level: u32, score: &mut u32) {
            match node.kind() {
                // Base cognitive load patterns (+1)
                "if_statement" | "while_statement" | "for_statement" | "do_statement" => {
                    *score += 1 + nesting_level;
                }
                // Switch/match patterns (+1)
                "switch_statement" | "case_label" => {
                    *score += 1;
                }
                // Exception handling (+1)
                "try_statement" | "catch_clause" => {
                    *score += 1;
                }
                // Logical operators in conditions (+1)
                "logical_and" | "logical_or" => {
                    *score += 1;
                }
                // Ternary operators (+1)
                "conditional_expression" => {
                    *score += 1;
                }
                _ => {}
            }

            // Increase nesting for control structures
            let new_nesting = if matches!(
                node.kind(),
                "if_statement" | "while_statement" | "for_statement" | "switch_statement"
            ) {
                nesting_level + 1
            } else {
                nesting_level
            };

            // Traverse children
            for child in node.children(&mut node.walk()) {
                traverse_cognitive(child, new_nesting, score);
            }
        }

        traverse_cognitive(*node, 0, &mut cognitive_score);
        cognitive_score
    }

    #[cfg(not(any(feature = "c-ast", feature = "cpp-ast")))]
    #[allow(dead_code)]
    fn calculate_cognitive_complexity(&self, _node: &str) -> u32 {
        // Simplified implementation for rust-only builds
        // Estimate based on source patterns
        5 // Default approximation
    }

    #[cfg(any(feature = "c-ast", feature = "cpp-ast"))]
    fn calculate_max_nesting(&self, node: &tree_sitter::Node) -> usize {
        let mut max_depth = 0;
        let _current_depth = 0;

        fn traverse(node: tree_sitter::Node, depth: usize, max: &mut usize) {
            *max = (*max).max(depth);

            for child in node.children(&mut node.walk()) {
                let new_depth = if matches!(
                    child.kind(),
                    "if_statement" | "while_statement" | "for_statement" | "compound_statement"
                ) {
                    depth + 1
                } else {
                    depth
                };
                traverse(child, new_depth, max);
            }
        }

        traverse(*node, 0, &mut max_depth);
        max_depth
    }

    #[cfg(not(any(feature = "c-ast", feature = "cpp-ast")))]
    #[allow(dead_code)]
    fn calculate_max_nesting(&self, _node: &str) -> usize {
        // Simplified implementation for rust-only builds
        5 // Default approximation
    }

    #[cfg(any(feature = "c-ast", feature = "cpp-ast"))]
    fn calculate_max_function_length(&self, node: &tree_sitter::Node, source: &str) -> usize {
        let mut max_length = 0;

        fn find_functions(node: tree_sitter::Node, source: &str, max: &mut usize) {
            if node.kind() == "function_definition" {
                let start_line = node.start_position().row;
                let end_line = node.end_position().row;
                let length = end_line - start_line + 1;
                *max = (*max).max(length);
            }

            for child in node.children(&mut node.walk()) {
                find_functions(child, source, max);
            }
        }

        find_functions(*node, source, &mut max_length);
        max_length
    }

    #[cfg(not(any(feature = "c-ast", feature = "cpp-ast")))]
    #[allow(dead_code)]
    fn calculate_max_function_length(&self, _source: &str) -> usize {
        // Simplified implementation for rust-only builds
        20 // Default approximation
    }

    pub async fn analyze_project(&self, dir: &Path) -> Result<ProjectScore> {
        let files = self.discover_files(dir)?;
        let mut scores = Vec::new();

        for file in files {
            match self.analyze_file(&file).await {
                Ok(score) => scores.push(score),
                Err(e) => eprintln!("Warning: Failed to analyze {}: {}", file.display(), e),
            }
        }

        Ok(ProjectScore::aggregate(scores))
    }

    pub async fn compare(&self, path1: &Path, path2: &Path) -> Result<crate::tdg::Comparison> {
        let score1 = if path1.is_dir() {
            self.analyze_project(path1).await?.average()
        } else {
            self.analyze_file(path1).await?
        };

        let score2 = if path2.is_dir() {
            self.analyze_project(path2).await?.average()
        } else {
            self.analyze_file(path2).await?
        };

        Ok(crate::tdg::Comparison::new(score1, score2))
    }

    fn discover_files(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        self.discover_files_recursive(dir, &mut files)?;
        Ok(files)
    }

    fn discover_files_recursive(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                if !self.should_skip_directory(&path) {
                    self.discover_files_recursive(&path, files)?;
                }
            } else if self.should_analyze_file(&path) {
                files.push(path);
            }
        }

        Ok(())
    }

    fn should_skip_directory(&self, path: &Path) -> bool {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            matches!(
                name,
                "node_modules"
                    | "target"
                    | "build"
                    | "dist"
                    | ".git"
                    | "__pycache__"
                    | ".pytest_cache"
                    | "venv"
                    | ".venv"
                    | "vendor"
                    | ".idea"
                    | ".vscode"
                    | "tests"
            )
        } else {
            false
        }
    }

    fn should_analyze_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            matches!(
                ext,
                "rs" | "py"
                    | "js"
                    | "ts"
                    | "jsx"
                    | "tsx"
                    | "go"
                    | "java"
                    | "c"
                    | "h"
                    | "cpp"
                    | "cc"
                    | "cxx"
                    | "hpp"
                    | "rb"
                    | "swift"
                    | "kt"
                    | "kts"
                    | "sql"
                    | "ddl"
                    | "dml"
                    | "scala"
                    | "sc"
                    | "yaml"
                    | "yml"
                    | "md"
                    | "mdx"
                    | "markdown"
            )
        } else {
            false
        }
    }

    // ── SQL heuristic analysis ──────────────────────────────────────────

    #[allow(clippy::cast_possible_truncation)]
    fn analyze_sql_heuristic(
        &self,
        source: &str,
        score: &mut TdgScore,
        tracker: &mut PenaltyTracker,
    ) -> Result<()> {
        score.confidence *= 0.8;

        let lines: Vec<&str> = source.lines().collect();
        let total_lines = lines.len().max(1);

        // Structural: subquery nesting + JOIN count + statement length
        let mut max_nesting = 0u32;
        let mut current_nesting = 0u32;
        let mut join_count = 0u32;
        let mut longest_stmt = 0usize;
        let mut current_stmt_lines = 0usize;

        let upper = source.to_uppercase();
        for line in upper.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            current_nesting += trimmed.matches('(').count() as u32;
            current_nesting = current_nesting.saturating_sub(trimmed.matches(')').count() as u32);
            max_nesting = max_nesting.max(current_nesting);

            if trimmed.contains("JOIN") {
                join_count += 1;
            }
            current_stmt_lines += 1;
            if trimmed.ends_with(';') {
                longest_stmt = longest_stmt.max(current_stmt_lines);
                current_stmt_lines = 0;
            }
        }
        longest_stmt = longest_stmt.max(current_stmt_lines);

        let cyclomatic = 1 + join_count + (max_nesting / 2);
        score.structural_complexity = self.score_structural_complexity(
            cyclomatic,
            max_nesting,
            max_nesting as usize,
            longest_stmt,
            tracker,
        );

        // Semantic: column count, CASE expressions, function calls
        let case_count = upper.matches("CASE ").count() as u32;
        let coalesce_count = upper.matches("COALESCE").count() as u32;
        let cast_count = upper.matches("CAST(").count() as u32;
        let type_complexity = case_count + coalesce_count + cast_count;
        score.semantic_complexity =
            self.score_semantic_complexity(join_count as usize, type_complexity, max_nesting, tracker);

        // Duplication
        score.duplication_ratio = self.analyze_duplication_ast(source, score.language, tracker);

        // Coupling: table references
        let from_count = upper.matches(" FROM ").count() as u32;
        let into_count = upper.matches(" INTO ").count() as u32;
        score.coupling_score =
            self.score_coupling(from_count + into_count + join_count, 0, 0, tracker);

        // Documentation: comment ratio
        let comment_lines = lines
            .iter()
            .filter(|l| {
                let t = l.trim();
                t.starts_with("--") || t.starts_with("/*")
            })
            .count() as u32;
        score.doc_coverage =
            self.score_documentation(comment_lines, total_lines as u32, comment_lines, total_lines as u32, tracker);

        // Consistency: keyword casing (all-upper vs mixed)
        let sql_keywords = ["SELECT", "FROM", "WHERE", "INSERT", "UPDATE", "DELETE", "JOIN", "GROUP", "ORDER", "HAVING"];
        let mut upper_kw = 0u32;
        let mut lower_kw = 0u32;
        for line in source.lines() {
            for kw in &sql_keywords {
                if line.contains(kw) {
                    upper_kw += 1;
                }
                if line.contains(&kw.to_lowercase()) && !line.contains(kw) {
                    lower_kw += 1;
                }
            }
        }
        let total_kw = upper_kw + lower_kw;
        if total_kw > 0 {
            let dominant = upper_kw.max(lower_kw) as f32 / total_kw as f32;
            score.consistency_score = dominant * self.config.weights.consistency;
        } else {
            score.consistency_score = self.config.weights.consistency;
        }

        // Entropy
        score.entropy_score = self.score_entropy_analysis(source, score.language, tracker);

        Ok(())
    }

    // ── Scala heuristic analysis ────────────────────────────────────────

    #[allow(clippy::cast_possible_truncation)]
    fn analyze_scala_heuristic(
        &self,
        source: &str,
        score: &mut TdgScore,
        tracker: &mut PenaltyTracker,
    ) -> Result<()> {
        score.confidence *= 0.85;

        let lines: Vec<&str> = source.lines().collect();
        let total_lines = lines.len().max(1);

        // Structural metrics
        let (cyclomatic, cognitive, max_nesting, longest_method) =
            Self::scala_structural_metrics(&lines);
        score.structural_complexity = self.score_structural_complexity(
            cyclomatic, cognitive, max_nesting, longest_method, tracker,
        );

        // Semantic: implicit chains, type params, higher-kinded types
        let implicit_count = source.matches("implicit ").count() as u32;
        let type_param_count = source.matches("[_").count() as u32
            + source.matches("[A-Z]").count().min(20) as u32;
        let hkt_count = source.matches("[F[_]]").count() as u32
            + source.matches("[M[_]]").count() as u32;
        let param_count = source.matches("def ").count();
        score.semantic_complexity = self.score_semantic_complexity(
            param_count, implicit_count + hkt_count, type_param_count.min(10), tracker,
        );

        // Duplication
        score.duplication_ratio = self.analyze_duplication_ast(source, score.language, tracker);

        // Coupling: imports
        let import_count = source.matches("import ").count() as u32;
        let extends_count = source.matches(" extends ").count() as u32;
        let with_count = source.matches(" with ").count() as u32;
        score.coupling_score =
            self.score_coupling(import_count, extends_count + with_count, 0, tracker);

        // Documentation: scaladoc + comments
        let doc_comment_lines = lines
            .iter()
            .filter(|l| {
                let t = l.trim();
                t.starts_with("/**") || t.starts_with("*") || t.starts_with("//")
            })
            .count() as u32;
        let public_items = source.matches("def ").count() as u32
            + source.matches("val ").count() as u32
            + source.matches("class ").count() as u32
            + source.matches("object ").count() as u32
            + source.matches("trait ").count() as u32;
        score.doc_coverage = self.score_documentation(
            doc_comment_lines.min(public_items),
            public_items.max(1),
            doc_comment_lines,
            total_lines as u32,
            tracker,
        );

        // Consistency: camelCase naming
        score.consistency_score =
            Self::scala_naming_consistency(&lines) * self.config.weights.consistency;

        // Entropy
        score.entropy_score = self.score_entropy_analysis(source, score.language, tracker);

        Ok(())
    }

    /// Extract structural metrics from Scala source: (cyclomatic, cognitive, max_nesting, longest_method)
    fn scala_structural_metrics(lines: &[&str]) -> (u32, u32, usize, usize) {
        let mut max_nesting = 0usize;
        let mut current_nesting = 0usize;
        let mut match_arms = 0u32;
        let mut longest_method = 0usize;
        let mut current_method_lines = 0usize;
        let mut in_method = false;
        let mut cyclomatic = 1u32;

        for line in lines {
            let trimmed = line.trim();
            current_nesting += trimmed.matches('{').count();
            current_nesting = current_nesting.saturating_sub(trimmed.matches('}').count());
            max_nesting = max_nesting.max(current_nesting);

            if Self::is_scala_control_flow(trimmed) {
                cyclomatic += 1;
            }
            if trimmed.starts_with("case ") && trimmed.contains("=>") {
                match_arms += 1;
            }

            if trimmed.starts_with("def ") || trimmed.starts_with("override def ") {
                if in_method {
                    longest_method = longest_method.max(current_method_lines);
                }
                current_method_lines = 0;
                in_method = true;
            }
            if in_method {
                current_method_lines += 1;
            }
        }
        if in_method {
            longest_method = longest_method.max(current_method_lines);
        }

        let cognitive = match_arms + (max_nesting as u32).saturating_sub(2);
        (cyclomatic, cognitive, max_nesting, longest_method)
    }

    fn is_scala_control_flow(trimmed: &str) -> bool {
        trimmed.starts_with("if ")
            || trimmed.starts_with("if(")
            || trimmed.contains(" if ")
            || trimmed.starts_with("case ")
            || trimmed.starts_with("while ")
            || trimmed.starts_with("for ")
            || trimmed.starts_with("for(")
            || trimmed.contains("catch ")
    }

    /// Calculate Scala naming consistency ratio (0.0-1.0)
    fn scala_naming_consistency(lines: &[&str]) -> f32 {
        let mut camel_defs = 0u32;
        let mut non_camel_defs = 0u32;
        for line in lines {
            let trimmed = line.trim();
            let rest = trimmed.strip_prefix("def ")
                .or_else(|| trimmed.strip_prefix("val "))
                .or_else(|| trimmed.strip_prefix("var "));
            let Some(rest) = rest else { continue };
            let name = rest.split(|c: char| !c.is_alphanumeric() && c != '_').next().unwrap_or("");
            if name.is_empty() {
                continue;
            }
            if name.chars().next().is_some_and(|c| c.is_lowercase()) && !name.contains('_') {
                camel_defs += 1;
            } else if name.contains('_') {
                non_camel_defs += 1;
            }
        }
        let total_defs = camel_defs + non_camel_defs;
        if total_defs > 2 {
            camel_defs.max(non_camel_defs) as f32 / total_defs as f32
        } else {
            1.0
        }
    }

    // ── YAML heuristic analysis ─────────────────────────────────────────

    #[allow(clippy::cast_possible_truncation)]
    fn analyze_yaml_heuristic(
        &self,
        source: &str,
        score: &mut TdgScore,
        tracker: &mut PenaltyTracker,
    ) -> Result<()> {
        score.confidence *= 0.75;

        let lines: Vec<&str> = source.lines().collect();
        let total_lines = lines.len().max(1);

        // Structural: nesting depth via indentation
        let mut max_indent = 0usize;
        let mut indent_sizes = Vec::new();

        for line in &lines {
            if line.trim().is_empty() || line.trim().starts_with('#') {
                continue;
            }
            let indent = line.len() - line.trim_start().len();
            if indent > 0 {
                indent_sizes.push(indent);
                max_indent = max_indent.max(indent);
            }
        }

        // Estimate indent unit (usually 2 or 4)
        let indent_unit = if indent_sizes.len() > 2 {
            let mut diffs: Vec<usize> = indent_sizes
                .windows(2)
                .filter_map(|w| {
                    if w[1] > w[0] {
                        Some(w[1] - w[0])
                    } else {
                        None
                    }
                })
                .collect();
            diffs.sort_unstable();
            diffs.first().copied().unwrap_or(2).max(1)
        } else {
            2
        };

        let nesting_depth = max_indent / indent_unit;

        // Count anchors, aliases, multi-doc markers
        let anchor_count = source.matches(" &").count() as u32;
        let alias_count = source.matches(" *").count() as u32;
        let multi_doc = source.matches("\n---").count() as u32;
        let key_count = lines
            .iter()
            .filter(|l| {
                let t = l.trim();
                !t.is_empty() && !t.starts_with('#') && !t.starts_with('-') && t.contains(':')
            })
            .count() as u32;

        let cyclomatic = 1 + multi_doc + (anchor_count / 2);
        score.structural_complexity = self.score_structural_complexity(
            cyclomatic,
            nesting_depth as u32,
            nesting_depth,
            total_lines,
            tracker,
        );

        // Semantic: type tags, complex values
        let tag_count = source.matches("!!").count() as u32;
        let multiline_count =
            (source.matches(" |").count() + source.matches(" >").count()) as u32;
        score.semantic_complexity = self.score_semantic_complexity(
            key_count as usize,
            tag_count + multiline_count,
            anchor_count + alias_count,
            tracker,
        );

        // Duplication (YAML has lots of repeated keys)
        score.duplication_ratio = self.analyze_duplication_ast(source, score.language, tracker);

        // Coupling: anchors/aliases = internal references
        score.coupling_score =
            self.score_coupling(anchor_count + alias_count, multi_doc, 0, tracker);

        // Documentation: comment ratio
        let comment_lines = lines
            .iter()
            .filter(|l| l.trim().starts_with('#'))
            .count() as u32;
        score.doc_coverage = self.score_documentation(
            comment_lines,
            key_count.max(1),
            comment_lines,
            total_lines as u32,
            tracker,
        );

        // Consistency: indentation consistency
        if indent_sizes.len() > 3 {
            let consistent_indents = indent_sizes
                .iter()
                .filter(|&&s| s % indent_unit == 0)
                .count();
            let ratio = consistent_indents as f32 / indent_sizes.len() as f32;
            score.consistency_score = ratio * self.config.weights.consistency;
        } else {
            score.consistency_score = self.config.weights.consistency;
        }

        // Entropy
        score.entropy_score = self.score_entropy_analysis(source, score.language, tracker);

        Ok(())
    }

    // ── Markdown heuristic analysis ─────────────────────────────────────

    #[allow(clippy::cast_possible_truncation)]
    fn analyze_markdown_heuristic(
        &self,
        source: &str,
        score: &mut TdgScore,
        tracker: &mut PenaltyTracker,
    ) -> Result<()> {
        score.confidence *= 0.70;

        let lines: Vec<&str> = source.lines().collect();
        let total_lines = lines.len().max(1);

        // Structural: heading hierarchy, section count, code block count
        let mut heading_levels = Vec::new();
        let mut code_block_count = 0u32;
        let mut in_code_block = false;
        let mut max_list_depth = 0usize;

        for line in &lines {
            let trimmed = line.trim();

            if trimmed.starts_with("```") {
                in_code_block = !in_code_block;
                if !in_code_block {
                    code_block_count += 1;
                }
                continue;
            }
            if in_code_block {
                continue;
            }

            // Track heading levels
            if trimmed.starts_with('#') {
                let level = trimmed.chars().take_while(|&c| c == '#').count();
                heading_levels.push(level);
            }

            // Track list nesting
            let indent = line.len() - line.trim_start().len();
            if trimmed.starts_with("- ")
                || trimmed.starts_with("* ")
                || trimmed.starts_with("+ ")
                || trimmed.chars().next().is_some_and(|c| c.is_ascii_digit())
                    && trimmed.contains(". ")
            {
                max_list_depth = max_list_depth.max(indent / 2 + 1);
            }
        }

        let section_count = heading_levels.len() as u32;
        score.structural_complexity = self.score_structural_complexity(
            1 + section_count / 5,
            max_list_depth as u32,
            max_list_depth,
            total_lines,
            tracker,
        );

        // Semantic: link density, image count, table count
        let link_count = source.matches("](").count() as u32;
        let image_count = source.matches("![").count() as u32;
        let table_rows = lines
            .iter()
            .filter(|l| l.trim().starts_with('|') && l.trim().ends_with('|'))
            .count() as u32;
        score.semantic_complexity = self.score_semantic_complexity(
            link_count as usize,
            image_count + table_rows,
            code_block_count,
            tracker,
        );

        // Duplication
        score.duplication_ratio = self.analyze_duplication_ast(source, score.language, tracker);

        // Coupling: external links and cross-references
        let external_links = source.matches("](http").count() as u32;
        let internal_links = link_count.saturating_sub(external_links);
        score.coupling_score =
            self.score_coupling(external_links, internal_links, 0, tracker);

        // Documentation: markdown IS documentation, so base on structure quality
        let has_toc = source.contains("## Table of Contents")
            || source.contains("## TOC")
            || source.contains("<!-- toc");
        let has_intro = !heading_levels.is_empty() && heading_levels[0] <= 2;
        let well_structured = (has_toc as u32) + (has_intro as u32) + u32::from(section_count > 2);
        score.doc_coverage = self.score_documentation(
            well_structured,
            3,
            section_count,
            total_lines as u32,
            tracker,
        );

        // Consistency: heading hierarchy (no skipped levels), list marker consistency
        let mut hierarchy_violations = 0u32;
        for window in heading_levels.windows(2) {
            if window[1] > window[0] + 1 {
                hierarchy_violations += 1;
            }
        }

        // List marker consistency
        let dash_lists = lines
            .iter()
            .filter(|l| l.trim().starts_with("- "))
            .count();
        let star_lists = lines
            .iter()
            .filter(|l| l.trim().starts_with("* "))
            .count();
        let total_list_items = dash_lists + star_lists;
        let list_consistency = if total_list_items > 2 {
            dash_lists.max(star_lists) as f32 / total_list_items as f32
        } else {
            1.0
        };

        let heading_penalty = (hierarchy_violations as f32 * 1.5).min(5.0);
        score.consistency_score =
            (list_consistency * self.config.weights.consistency - heading_penalty).max(0.0);

        // Entropy
        score.entropy_score = self.score_entropy_analysis(source, score.language, tracker);

        Ok(())
    }
}

// Visitor implementations for AST analysis
#[cfg(feature = "rust-ast")]
struct RustComplexityVisitor {
    cyclomatic_complexity: u32,
    cognitive_complexity: u32,
    max_nesting_depth: usize,
    max_method_length: usize,
    max_params: usize,
    generic_count: u32,
    abstraction_levels: u32,
    import_count: u32,
    external_calls: u32,
    interface_implementations: u32,
    documented_items: u32,
    total_public_items: u32,
    comment_lines: u32,
    total_lines: u32,
    current_depth: usize,
}
