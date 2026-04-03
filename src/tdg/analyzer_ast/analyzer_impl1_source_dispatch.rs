impl TdgAnalyzerAst {
    pub fn analyze_source(
        &self,
        source: &str,
        language: Language,
        file_path: Option<PathBuf>,
    ) -> Result<TdgScore> {
        let mut tracker = PenaltyTracker::new();
        let mut score = TdgScore {
            language,
            confidence: language.confidence(),
            file_path,
            ..Default::default()
        };

        // Use proper AST-based analysis
        match language {
            Language::Rust => self.analyze_rust_ast(source, &mut score, &mut tracker)?,
            Language::Python => self.analyze_python_ast(source, &mut score, &mut tracker)?,
            Language::JavaScript | Language::TypeScript => {
                self.analyze_javascript_ast(source, &mut score, &mut tracker)?;
            }
            Language::Go => self.analyze_go_ast(source, &mut score, &mut tracker)?,
            Language::Java => self.analyze_java_ast(source, &mut score, &mut tracker)?,
            Language::C | Language::Cpp => self.analyze_c_ast(source, &mut score, &mut tracker)?,
            Language::Ruchy => self.analyze_ruchy_ast(source, &mut score, &mut tracker)?,
            Language::Lua => self.analyze_lua_ast(source, &mut score, &mut tracker)?,
            Language::Sql => self.analyze_sql_heuristic(source, &mut score, &mut tracker)?,
            Language::Scala => self.analyze_scala_heuristic(source, &mut score, &mut tracker)?,
            Language::Yaml => self.analyze_yaml_heuristic(source, &mut score, &mut tracker)?,
            Language::Lean => {
                self.analyze_lean_heuristic(source, &mut score, &mut tracker)?;
            }
            Language::Markdown => {
                self.analyze_markdown_heuristic(source, &mut score, &mut tracker)?;
            }
            _ => {
                // Fallback to heuristics for unsupported languages
                // but with reduced confidence
                score.confidence *= 0.5;
                self.analyze_heuristic(source, &mut score, &mut tracker)?;
            }
        }

        score.penalties_applied = tracker.get_attributions();

        // Known Defects v2.1: Detect critical defects for auto-fail
        if let Some(ref path) = score.file_path {
            let defects = match language {
                Language::Rust => {
                    let detector = RustDefectDetector::new();
                    detector.detect(source, path)
                }
                Language::Lua => {
                    let detector = LuaDefectDetector::new();
                    detector.detect(source, path)
                }
                _ => Vec::new(),
            };

            let critical_count: usize = defects
                .iter()
                .filter(|d| d.severity == DefectSeverity::Critical)
                .map(|d| d.instances.len())
                .sum();

            // Lean-specific: sorry = critical defect (proof incompleteness)
            let lean_sorry_count = if language == Language::Lean {
                count_lean_sorry_ast(source)
            } else {
                0
            };

            score.critical_defects_count = critical_count + lean_sorry_count;
            score.has_critical_defects = score.critical_defects_count > 0;

            // Issue #279: New files with no git history should not be auto-failed.
            // This prevents a circular dependency where new files can't be committed
            // because the quality gate blocks them, but they can't pass the gate
            // because they have no git history. Score based on code quality only.
            if score.has_critical_defects && !is_file_git_tracked(path) {
                score.has_critical_defects = false;
                // Keep critical_defects_count for informational reporting
            }
        }

        score.calculate_total();

        Ok(score)
    }

    fn analyze_rust_ast(
        &self,
        source: &str,
        score: &mut TdgScore,
        tracker: &mut PenaltyTracker,
    ) -> Result<()> {
        #[cfg(feature = "rust-ast")]
        {
            use syn::{visit::Visit, File};

            let ast = syn::parse_str::<File>(source)?;

            let mut visitor = RustComplexityVisitor::new();
            visitor.visit_file(&ast);

            // Calculate structural complexity based on AST
            let cyclomatic = visitor.cyclomatic_complexity;
            let cognitive = visitor.cognitive_complexity;

            score.structural_complexity = self.score_structural_complexity(
                cyclomatic,
                cognitive,
                visitor.max_nesting_depth,
                visitor.max_method_length,
                tracker,
            );

            // Calculate semantic complexity
            score.semantic_complexity = self.score_semantic_complexity(
                visitor.max_params,
                visitor.generic_count,
                visitor.abstraction_levels,
                tracker,
            );

            // Calculate duplication (requires AST comparison)
            score.duplication_ratio = self.analyze_duplication_ast(source, Language::Rust, tracker);

            // Calculate coupling
            score.coupling_score = self.score_coupling(
                visitor.import_count,
                visitor.external_calls,
                visitor.interface_implementations,
                tracker,
            );

            // Calculate documentation coverage
            score.doc_coverage = self.score_documentation(
                visitor.documented_items,
                visitor.total_public_items,
                visitor.comment_lines,
                visitor.total_lines,
                tracker,
            );

            // Calculate consistency
            score.consistency_score = self.score_consistency_rust(&ast, tracker);

            // Calculate entropy - pattern analysis for code quality
            score.entropy_score = self.score_entropy_analysis(source, Language::Rust, tracker);
        }
        #[cfg(not(feature = "rust-ast"))]
        {
            self.analyze_heuristic(source, score, tracker)?;
        }

        Ok(())
    }

    fn analyze_python_ast(
        &self,
        source: &str,
        score: &mut TdgScore,
        tracker: &mut PenaltyTracker,
    ) -> Result<()> {
        #[cfg(feature = "python-ast")]
        {
            // Modern tree-sitter-python parsing (replaces rustpython-parser)
            use tree_sitter::Parser as TsParser;

            let mut parser = TsParser::new();
            parser
                .set_language(&tree_sitter_python::LANGUAGE.into())
                .map_err(|e| anyhow::anyhow!("Failed to set Python language: {e}"))?;

            let tree = parser
                .parse(source, None)
                .ok_or_else(|| anyhow::anyhow!("Failed to parse Python code"))?;

            let mut visitor = PythonComplexityVisitor::new(source);
            visitor.analyze_tree(&tree);

            score.structural_complexity = self.score_structural_complexity(
                visitor.cyclomatic_complexity,
                visitor.cognitive_complexity,
                visitor.max_nesting_depth,
                visitor.max_method_length,
                tracker,
            );

            score.semantic_complexity = self.score_semantic_complexity(
                visitor.max_params,
                visitor.decorator_count,
                visitor.metaclass_count,
                tracker,
            );

            score.duplication_ratio =
                self.analyze_duplication_ast(source, Language::Python, tracker);

            score.coupling_score = self.score_coupling(
                visitor.import_count,
                visitor.external_calls,
                0, // Python doesn't have explicit interfaces
                tracker,
            );

            score.doc_coverage = self.score_documentation(
                visitor.documented_functions,
                visitor.total_functions,
                visitor.docstring_lines,
                visitor.total_lines,
                tracker,
            );

            score.consistency_score = self.score_consistency_python(source, tracker);

            score.entropy_score = self.score_entropy_analysis(source, Language::Python, tracker);
        }
        #[cfg(not(feature = "python-ast"))]
        {
            self.analyze_heuristic(source, score, tracker)?;
        }

        Ok(())
    }
}

/// Check if a file is tracked by git (has at least one commit).
/// Returns false for new/untracked files, enabling graceful degradation (issue #279).
fn is_file_git_tracked(path: &Path) -> bool {
    std::process::Command::new("git")
        .args(["log", "--oneline", "-1", "--"])
        .arg(path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
        .map(|o| o.status.success() && !o.stdout.is_empty())
        .unwrap_or(true) // If git unavailable, assume tracked (don't change behavior)
}
