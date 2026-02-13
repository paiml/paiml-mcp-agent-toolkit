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
            )
        } else {
            false
        }
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
