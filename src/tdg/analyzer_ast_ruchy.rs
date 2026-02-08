#![cfg_attr(coverage_nightly, coverage(off))]
//! Ruchy language analysis helpers for TDG AST analyzer
//!
//! Extracted from analyzer_ast.rs for file health compliance (CB-040).
//! Contains Ruchy-specific complexity, import, and documentation analysis.

use super::analyzer_ast::TdgAnalyzerAst;

impl TdgAnalyzerAst {
    #[cfg(feature = "ruchy-ast")]
    pub(crate) fn calculate_ruchy_semantic_complexity(&self, source: &str) -> f32 {
        let mut complexity_score = self.config.weights.semantic_complexity;

        // Count Ruchy-specific complex patterns
        let actor_count = source.matches("actor ").count();
        let receive_count = source.matches("receive ").count();
        let pipeline_count = source.matches("|>").count();
        let match_count = source.matches(" match ").count();
        let pattern_match_count = source.matches(" => ").count();

        // Actor model complexity
        complexity_score += (actor_count * 2) as f32;
        complexity_score += receive_count as f32 * 1.5;

        // Pipeline operator complexity
        complexity_score += pipeline_count as f32 * 0.5;

        // Pattern matching complexity
        complexity_score += match_count as f32 * 1.2;
        complexity_score += pattern_match_count as f32 * 0.3;

        complexity_score.min(self.config.weights.semantic_complexity)
    }

    #[cfg(feature = "ruchy-ast")]
    pub(crate) fn count_ruchy_imports(&self, source: &str) -> u32 {
        // Count Ruchy-style import statements
        source.matches("import ").count() as u32
            + source.matches("use ").count() as u32
            + source.matches("extern ").count() as u32
    }

    #[cfg(feature = "ruchy-ast")]
    pub(crate) fn count_ruchy_dependencies(&self, source: &str) -> u32 {
        // Count actor message dependencies and external calls
        source.matches(" <- ").count() as u32 +  // Message sends
        source.matches(" <? ").count() as u32 +  // Message queries
        source.matches("spawn ").count() as u32 // Actor spawns
    }

    #[cfg(feature = "ruchy-ast")]
    pub(crate) fn calculate_ruchy_doc_coverage(&self, source: &str) -> f32 {
        let line_count = source.lines().count() as f32;
        if line_count == 0.0 {
            return self.config.weights.documentation;
        }

        // Count documentation comments and doc strings
        let doc_comments = source.matches("///").count() as f32
            + source.matches("/**").count() as f32
            + source.matches("#[doc").count() as f32;

        let coverage_ratio = (doc_comments / line_count * 20.0).min(1.0);
        coverage_ratio * self.config.weights.documentation
    }

    #[cfg(feature = "ruchy-ast")]
    pub(crate) fn calculate_ruchy_consistency(&self, source: &str) -> f32 {
        let mut consistency_score = self.config.weights.consistency;

        // Check for consistent naming patterns
        let snake_case_functions = regex::Regex::new(r"fun [a-z][a-z0-9_]*\(")
            .expect("Invalid regex")
            .find_iter(source)
            .count();

        let pascal_case_types = regex::Regex::new(r"(struct|enum|actor) [A-Z][A-Za-z0-9]*")
            .expect("Invalid regex")
            .find_iter(source)
            .count();

        let snake_case_vars = regex::Regex::new(r"let [a-z][a-z0-9_]* =")
            .expect("Invalid regex")
            .find_iter(source)
            .count();

        let total_identifiers = snake_case_functions + pascal_case_types + snake_case_vars;

        // Reduce score for inconsistent naming
        if total_identifiers > 0 {
            let fun_upper_regex = regex::Regex::new(r"fun [A-Z]").expect("Invalid regex");
            let struct_lower_regex = regex::Regex::new(r"struct [a-z]").expect("Invalid regex");
            let let_upper_regex = regex::Regex::new(r"let [A-Z]").expect("Invalid regex");

            let inconsistent_count = fun_upper_regex.find_iter(source).count()
                + struct_lower_regex.find_iter(source).count()
                + let_upper_regex.find_iter(source).count();

            if inconsistent_count > 0 {
                let consistency_ratio =
                    1.0 - (inconsistent_count as f32 / total_identifiers as f32);
                consistency_score *= consistency_ratio;
            }
        }

        consistency_score
    }
}
