impl TdgAnalyzerAst {

    fn analyze_ruchy_ast(
        &self,
        source: &str,
        score: &mut TdgScore,
        tracker: &mut PenaltyTracker,
    ) -> Result<()> {
        debug_assert!(!source.is_empty(), "source must not be empty");
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
        debug_assert!(!source.is_empty(), "source must not be empty");
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
        debug_assert!(!source.is_empty(), "source must not be empty");
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
}

// Core scoring methods (structural, semantic, duplication, coupling, documentation)
include!("analyzer_impl2_scoring.rs");
// Language consistency scoring (Rust, Python, JavaScript, Lua)
include!("analyzer_impl2_consistency.rs");
// Entropy, tree-sitter metrics, project analysis
include!("analyzer_impl2_metrics.rs");
// SQL, Scala, YAML, Markdown heuristic analysis
include!("analyzer_impl2_heuristics.rs");

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
