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
    fn calculate_max_function_length(&self, _source: &str) -> usize {
        // Simplified implementation for rust-only builds
        20 // Default approximation
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn analyze_project(&self, dir: &Path) -> Result<ProjectScore> {
        let found = self.discover_files(dir)?;
        let mut scores = Vec::new();
        // Seeded with what the WALK refused: source files this build has no
        // analyzer for never reach `analyze_file`, so they raise no error and
        // used to disappear from numerator and denominator alike.
        let mut ungraded: Vec<crate::tdg::UngradedFile> = found
            .ungraded
            .iter()
            .map(|(path, reason)| crate::tdg::UngradedFile {
                path: path.display().to_string(),
                reason: reason.clone(),
            })
            .collect();

        for file in &found.gradable {
            // Skip include!() fragment files — they aren't standalone Rust modules
            if crate::cli::language_analyzer::is_include_fragment(file) {
                continue;
            }
            match self.analyze_file(file).await {
                Ok(score) => scores.push(score),
                Err(e) => {
                    eprintln!("Warning: Failed to analyze {}: {}", file.display(), e);
                    // A refused file used to leave NO trace in the return value:
                    // stderr is not part of a `--format json` payload, so
                    // `analyze tdg` reported `average_score: 100.0` over the
                    // survivors of a tree whose only Rust file did not parse.
                    ungraded.push(crate::tdg::UngradedFile {
                        path: file.display().to_string(),
                        reason: e.to_string(),
                    });
                }
            }
        }

        // `discover_files` walks in filesystem order; this list is serialised
        // verbatim, so identical input must serialise identically.
        ungraded.sort_by(|a, b| a.path.cmp(&b.path));

        let mut project = ProjectScore::aggregate(scores);
        project.ungraded_files = ungraded;
        Ok(project)
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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

    /// Enumerate the files TDG will score under `dir`, and the source files it
    /// will not.
    ///
    /// This was a hand-rolled `read_dir` recursion that consulted nothing but a
    /// hardcoded directory-name skip list — no `.gitignore`, no hidden-directory
    /// handling — so it descended into ignored trees and scored them: on this
    /// repo it discovered 133,825 files for 4,260 tracked ones, because
    /// `.claude/worktrees/` holds whole copies of the very tree being analysed.
    /// The gitignore-aware walk that replaced it was then duplicated, with a
    /// second extension whitelist, in `analyzer_simple_helpers.rs`. Both are now
    /// `crate::tdg::file_discovery`, which also RETURNS the source files it
    /// refuses rather than dropping them.
    fn discover_files(&self, dir: &Path) -> Result<crate::tdg::file_discovery::Discovery> {
        crate::tdg::file_discovery::discover(dir, crate::tdg::file_discovery::Policy::ast())
    }

    #[cfg(test)]
    fn should_analyze_file(&self, path: &Path) -> bool {
        crate::tdg::file_discovery::is_gradable_path(
            path,
            crate::tdg::file_discovery::Policy::ast(),
        )
    }
}

#[cfg(test)]
mod discover_files_tests {
    use super::*;

    /// The 31x file inflation: `discover_files` used to walk gitignored and
    /// hidden trees, so `.claude/worktrees/` — whole copies of the tree being
    /// analysed — contributed their files to the project score.
    #[test]
    fn discover_files_honours_gitignore_and_hidden_directories() {
        let dir = tempfile::tempdir().expect("tempdir");
        // A named subdirectory, because `tempfile` roots are themselves dot-
        // prefixed and this walk skips hidden entries.
        let root = &dir.path().join("proj");
        std::fs::create_dir_all(root).expect("mkdir");

        std::fs::write(root.join(".gitignore"), "ignored/\n").expect("write gitignore");
        std::fs::write(root.join("kept.rs"), "pub fn kept() {}\n").expect("write");

        std::fs::create_dir_all(root.join("ignored")).expect("mkdir");
        std::fs::write(root.join("ignored/copy.rs"), "pub fn copy() {}\n").expect("write");

        std::fs::create_dir_all(root.join(".worktrees/wt")).expect("mkdir");
        std::fs::write(root.join(".worktrees/wt/copy.rs"), "pub fn copy() {}\n").expect("write");

        std::fs::create_dir_all(root.join("tests")).expect("mkdir");
        std::fs::write(root.join("tests/it.rs"), "pub fn it() {}\n").expect("write");

        let analyzer = TdgAnalyzerAst::new().expect("analyzer");
        let found = analyzer.discover_files(root).expect("discover");
        let names: Vec<String> = found
            .gradable
            .iter()
            .map(|p| p.strip_prefix(root).unwrap_or(p).display().to_string())
            .collect();

        assert!(names.iter().any(|n| n.ends_with("kept.rs")), "{names:?}");
        assert!(
            !names.iter().any(|n| n.contains("ignored")),
            "gitignored trees must not be scored: {names:?}"
        );
        assert!(
            !names.iter().any(|n| n.contains(".worktrees")),
            "hidden trees must not be scored: {names:?}"
        );
        assert!(
            !names.iter().any(|n| n.starts_with("tests")),
            "the historical tests/ skip must survive: {names:?}"
        );
    }
}
