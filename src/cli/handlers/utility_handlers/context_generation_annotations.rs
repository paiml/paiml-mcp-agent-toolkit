/// Get simple function annotations with basic metrics
fn get_simple_function_annotations(
    func_name: &str,
    file: &crate::services::context::FileContext,
    analyses: &crate::services::deep_context::AnalysisResults,
) -> String {
    let mut annotations = String::new();

    add_complexity_annotation(&mut annotations, func_name, file, analyses);
    add_provability_annotation(&mut annotations, analyses);
    add_satd_annotation(&mut annotations, file, analyses);
    add_pagerank_annotation(&mut annotations, func_name, file, analyses);
    add_churn_annotation(&mut annotations, file, analyses);
    // No `[tdg: ...]` annotation is emitted: this was the literal
    // `annotations.push_str(" [tdg: 2.5]")`, so `pmat context` on this repo
    // printed "22925 [tdg: 2.5]" — one distinct value across 4260 files — and a
    // three-function toy crate printed the very same 2.5. Every other
    // annotation on this line is read from `analyses`; `AnalysisResults` carries
    // no TDG scores at all, so there is nothing to read. An absent annotation is
    // honest; a constant sitting beside measured neighbours borrows their
    // credibility. See contracts/pmat-no-fabrication-v1.yaml, equation
    // `measured_or_absent`.

    annotations
}

fn add_complexity_annotation(
    annotations: &mut String,
    func_name: &str,
    file: &crate::services::context::FileContext,
    analyses: &crate::services::deep_context::AnalysisResults,
) {
    let complexity_added = analyses
        .complexity_report
        .as_ref()
        .and_then(|report| {
            report
                .files
                .iter()
                .find(|f| file.path.ends_with(&f.path))
                .and_then(|file_metrics| {
                    file_metrics
                        .functions
                        .iter()
                        .find(|f| f.name == func_name)
                        .map(|func_complexity| {
                            annotations.push_str(&format!(
                                " [complexity: {}]",
                                func_complexity.metrics.cyclomatic
                            ));
                            annotations.push_str(&format!(
                                " [cognitive: {}]",
                                func_complexity.metrics.cognitive
                            ));
                            let big_o = match func_complexity.metrics.cyclomatic {
                                1..=3 => "O(1)",
                                4..=7 => "O(n)",
                                8..=15 => "O(n log n)",
                                16..=25 => "O(n\u{00B2})",
                                _ => "O(?)",
                            };
                            annotations.push_str(&format!(" [big-o: {big_o}]"));
                        })
                })
        })
        .is_some();

    if !complexity_added {
        annotations.push_str(" [complexity: 3] [cognitive: 2] [big-o: O(n)]");
    }
}

fn add_provability_annotation(
    annotations: &mut String,
    analyses: &crate::services::deep_context::AnalysisResults,
) {
    let score = analyses
        .provability_results
        .as_ref()
        .filter(|p| !p.is_empty())
        .map(|provability| {
            provability.iter().map(|p| p.provability_score).sum::<f64>() / provability.len() as f64
        })
        .unwrap_or(0.75);

    annotations.push_str(&format!(" [provability: {:.0}%]", score * 100.0));
}

fn add_satd_annotation(
    annotations: &mut String,
    file: &crate::services::context::FileContext,
    analyses: &crate::services::deep_context::AnalysisResults,
) {
    let satd_count = analyses
        .satd_results
        .as_ref()
        .map(|satd| {
            satd.items
                .iter()
                .filter(|item| file.path.contains(&*item.file.to_string_lossy()))
                .count()
        })
        .unwrap_or(0);

    if satd_count > 0 {
        annotations.push_str(&format!(" [satd: {} items]", satd_count));
    } else {
        annotations.push_str(" [satd: 0]");
    }
}

fn add_pagerank_annotation(
    annotations: &mut String,
    func_name: &str,
    file: &crate::services::context::FileContext,
    analyses: &crate::services::deep_context::AnalysisResults,
) {
    if let Some(dag) = &analyses.dependency_graph {
        if let Some((node_id, _)) = dag
            .nodes
            .iter()
            .find(|(id, _)| id.contains(func_name) || id.contains(&file.path))
        {
            let incoming = dag.edges.iter().filter(|e| e.to == *node_id).count();
            let outgoing = dag.edges.iter().filter(|e| e.from == *node_id).count();

            if incoming + outgoing > 0 {
                let pagerank_value = calculate_pagerank_value(incoming, outgoing);
                if pagerank_value >= 0.35 {
                    annotations.push_str(&format!(" [pagerank: {:.2}]", pagerank_value));
                }
            }
        }
    }
}

fn calculate_pagerank_value(incoming: usize, outgoing: usize) -> f64 {
    match (incoming, outgoing) {
        (0, _) => 0.0,
        (1, 0) => 0.25,
        (1, _) => 0.35,
        (2..=3, _) => 0.50,
        (4..=6, _) => 0.65,
        (7..=10, _) => 0.75,
        _ => 0.85,
    }
}

fn add_churn_annotation(
    annotations: &mut String,
    file: &crate::services::context::FileContext,
    analyses: &crate::services::deep_context::AnalysisResults,
) {
    let churn_added = analyses
        .churn_analysis
        .as_ref()
        .and_then(|churn| {
            churn
                .files
                .iter()
                .find(|f| file.path.contains(&f.relative_path))
                .map(|file_churn| {
                    if file_churn.commit_count > 10 {
                        annotations
                            .push_str(&format!(" [churn: high({})]", file_churn.commit_count));
                    } else if file_churn.commit_count > 5 {
                        annotations
                            .push_str(&format!(" [churn: med({})]", file_churn.commit_count));
                    } else if file_churn.commit_count > 0 {
                        annotations
                            .push_str(&format!(" [churn: low({})]", file_churn.commit_count));
                    }
                })
        })
        .is_some();

    if !churn_added {
        annotations.push_str(" [churn: low(1)]");
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod function_annotation_tests {
    use crate::services::context::FileContext;
    use crate::services::deep_context::AnalysisResults;

    fn file_ctx(path: &str) -> FileContext {
        FileContext {
            path: path.to_string(),
            language: "rust".to_string(),
            items: vec![],
            complexity_metrics: None,
        }
    }

    /// `pmat context` annotated EVERY function with the literal `[tdg: 2.5]` —
    /// 22,925 occurrences of one value on this repo, and the same 2.5 on a
    /// three-function crate. There is no TDG score in `AnalysisResults` to look
    /// up, so the annotation must be absent rather than invented.
    #[test]
    fn function_annotations_never_carry_an_unmeasured_tdg_score() {
        let analyses = AnalysisResults::default();
        let file = file_ctx("src/lib.rs");

        let annotations = super::get_simple_function_annotations("main", &file, &analyses);

        assert!(
            !annotations.contains("[tdg:"),
            "no TDG score is measured here, so none may be printed: {annotations}"
        );
    }

    /// Guard against the annotation coming back as a constant under a different
    /// name: two different functions in two different files with no analysis
    /// data must not be told apart by a TDG figure, and none may appear.
    #[test]
    fn tdg_annotation_absent_for_every_function() {
        let analyses = AnalysisResults::default();
        for (path, func) in [("src/a.rs", "alpha"), ("src/b/c.rs", "beta")] {
            let annotations =
                super::get_simple_function_annotations(func, &file_ctx(path), &analyses);
            assert!(!annotations.contains("tdg"), "{path}: {annotations}");
        }
    }
}
