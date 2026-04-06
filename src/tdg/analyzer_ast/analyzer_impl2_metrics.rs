impl TdgAnalyzerAst {
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
        debug_assert!(dir.exists(), "dir must exist: {}", dir.display());
        let files = self.discover_files(dir)?;
        let mut scores = Vec::new();

        for file in &files {
            // Skip include!() fragment files — they aren't standalone Rust modules
            if crate::cli::language_analyzer::is_include_fragment(file) {
                continue;
            }
            match self.analyze_file(file).await {
                Ok(score) => scores.push(score),
                Err(e) => {
                    eprintln!("Warning: Failed to analyze {}: {}", file.display(), e);
                }
            }
        }

        Ok(ProjectScore::aggregate(scores))
    }

    pub async fn compare(&self, path1: &Path, path2: &Path) -> Result<crate::tdg::Comparison> {
        debug_assert!(path1.exists(), "path1 must exist: {}", path1.display());
        debug_assert!(path2.exists(), "path2 must exist: {}", path2.display());
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
        debug_assert!(dir.exists(), "dir must exist: {}", dir.display());
        let mut files = Vec::new();
        self.discover_files_recursive(dir, &mut files)?;
        Ok(files)
    }

    fn discover_files_recursive(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        debug_assert!(dir.exists(), "dir must exist: {}", dir.display());
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
        debug_assert!(path.exists(), "path must exist: {}", path.display());
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
                    | ".lake"
                    | "tests"
            )
        } else {
            false
        }
    }

    fn should_analyze_file(&self, path: &Path) -> bool {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
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
                    | "lean"
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


}
