#![cfg_attr(coverage_nightly, coverage(off))]
//! Prune the parallel analysis results down to the configured file scope.
//!
//! The individual analyses (complexity, SATD, dead code, …) each walk the whole
//! project on their own; none of them takes the deep-context pattern lists.
//! Rather than thread the patterns through nine analyzers — nine more places for
//! the rule to drift — the results are pruned once, here, through the single
//! `FileScope` predicate that the file-tree walk also uses. That is what keeps
//! `file_count` and the reported findings describing the same set of files.

use crate::services::deep_context::analyzer_core::types::ParallelAnalysisResults;
use crate::services::deep_context::scope::FileScope;
use crate::services::deep_context::DeepContextAnalyzer;

impl DeepContextAnalyzer {
    /// Drop every finding that names a file outside the scope.
    pub(crate) fn retain_analyses_in_scope(&self, results: &mut ParallelAnalysisResults) {
        let scope = FileScope::from_config(&self.config);
        if scope.is_unrestricted() {
            return;
        }
        Self::retain_ast_in_scope(results, &scope);
        Self::retain_complexity_in_scope(results, &scope);
        Self::retain_churn_in_scope(results, &scope);
        Self::retain_dead_code_in_scope(results, &scope);
        Self::retain_satd_in_scope(results, &scope);
        Self::retain_duplicates_in_scope(results, &scope);
        Self::retain_big_o_in_scope(results, &scope);
        Self::retain_dag_in_scope(results, &scope);
    }

    fn retain_ast_in_scope(results: &mut ParallelAnalysisResults, scope: &FileScope) {
        if let Some(contexts) = results.ast_contexts.as_mut() {
            contexts.retain(|c| scope.contains_file_str(&c.base.path));
        }
    }

    fn retain_complexity_in_scope(results: &mut ParallelAnalysisResults, scope: &FileScope) {
        use crate::services::complexity::Violation;

        let Some(report) = results.complexity_report.as_mut() else {
            return;
        };
        report.files.retain(|f| scope.contains_file_str(&f.path));
        report.hotspots.retain(|h| scope.contains_file_str(&h.file));
        report.violations.retain(|v| {
            let file = match v {
                Violation::Error { file, .. } | Violation::Warning { file, .. } => file,
            };
            scope.contains_file_str(file)
        });
        report.summary.total_files = report.files.len();
        report.summary.total_functions = report.files.iter().map(|f| f.functions.len()).sum();
    }

    fn retain_churn_in_scope(results: &mut ParallelAnalysisResults, scope: &FileScope) {
        if let Some(churn) = results.churn_analysis.as_mut() {
            churn.files.retain(|f| scope.contains_file(&f.path));
            churn.summary.total_files_changed = churn.files.len();
        }
    }

    fn retain_dead_code_in_scope(results: &mut ParallelAnalysisResults, scope: &FileScope) {
        let Some(dead) = results.dead_code_results.as_mut() else {
            return;
        };
        dead.ranked_files
            .retain(|f| scope.contains_file_str(&f.path));
        dead.summary.total_files_analyzed = dead.ranked_files.len();
        dead.summary.files_with_dead_code = dead
            .ranked_files
            .iter()
            .filter(|f| f.dead_lines > 0)
            .count();
        dead.summary.total_dead_lines = dead.ranked_files.iter().map(|f| f.dead_lines).sum();
        dead.summary.dead_functions = dead.ranked_files.iter().map(|f| f.dead_functions).sum();
        dead.summary.dead_classes = dead.ranked_files.iter().map(|f| f.dead_classes).sum();
        let total_lines: usize = dead.ranked_files.iter().map(|f| f.total_lines).sum();
        dead.summary.dead_percentage = if total_lines == 0 {
            0.0
        } else {
            dead.summary.total_dead_lines as f32 / total_lines as f32 * 100.0
        };
    }

    fn retain_satd_in_scope(results: &mut ParallelAnalysisResults, scope: &FileScope) {
        let Some(satd) = results.satd_results.as_mut() else {
            return;
        };
        satd.items.retain(|i| scope.contains_file(&i.file));
        satd.summary.total_items = satd.items.len();
        let distinct: std::collections::BTreeSet<_> =
            satd.items.iter().map(|i| i.file.clone()).collect();
        satd.summary.files_with_satd = distinct.len();
        satd.files_with_debt = distinct.len();
        // `total_files_analyzed` counts what the detector actually scanned; it
        // is deliberately left alone rather than back-fitted to the pruned
        // items, because inventing a number here would be the same defect this
        // whole round is about.
    }

    fn retain_duplicates_in_scope(results: &mut ParallelAnalysisResults, scope: &FileScope) {
        if let Some(dupes) = results.duplicate_code_results.as_mut() {
            dupes.hotspots.retain(|h| scope.contains_file(&h.file));
        }
    }

    fn retain_big_o_in_scope(results: &mut ParallelAnalysisResults, scope: &FileScope) {
        if let Some(big_o) = results.big_o_analysis.as_mut() {
            big_o
                .high_complexity_functions
                .retain(|f| scope.contains_file(&f.file_path));
        }
    }

    fn retain_dag_in_scope(results: &mut ParallelAnalysisResults, scope: &FileScope) {
        let Some(dag) = results.dependency_graph.as_mut() else {
            return;
        };
        dag.nodes
            .retain(|_, node| scope.contains_file_str(&node.file_path));
        dag.edges
            .retain(|e| dag.nodes.contains_key(&e.from) && dag.nodes.contains_key(&e.to));
    }
}
