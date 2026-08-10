/// Get simple function annotations with basic metrics
fn get_simple_function_annotations(
    func_name: &str,
    file: &crate::services::context::FileContext,
    analyses: &crate::services::deep_context::AnalysisResults,
) -> String {
    let mut annotations = String::new();

    add_complexity_annotation(&mut annotations, func_name, file, analyses);
    // No `[provability: N%]` annotation is emitted: it used to average
    // `provability_score` over the WHOLE `provability_results` vector and stamp
    // that one number onto every row — 22,925 functions on this repo all read
    // "[provability: 71%]", and a three-language toy corpus all read 65%.
    // `ProofSummary` carries no function or file identity, so there is no
    // per-function record to look up; a project-wide mean printed on a function
    // row is not that function's provability. Absent beats attributed-to-the-
    // wrong-thing. See contracts/pmat-no-fabrication-v1.yaml, equation
    // `measured_or_absent`.
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
    // When the lookup misses, nothing was measured for this function, so nothing
    // is printed. This used to fall back to the literal
    // " [complexity: 3] [cognitive: 2] [big-o: O(n)]", so a phantom `anonymous`
    // row in a one-line TypeScript file was published carrying three invented
    // metrics beside the real ones on the row above. The JSON path already emits
    // no complexity keys in this case; the markdown path now matches it. See
    // contracts/pmat-no-fabrication-v1.yaml, equation `measured_or_absent`.
    let _measured = analyses
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

    /// A function the complexity report says nothing about used to be published
    /// as `[complexity: 3] [cognitive: 2] [big-o: O(n)]` — a hardcoded string,
    /// not a measurement. It is what made the phantom `anonymous` row in a
    /// one-line TypeScript file look measured.
    #[test]
    fn unmeasured_function_gets_no_invented_complexity() {
        let analyses = AnalysisResults::default();
        let file = file_ctx("app.ts");

        let annotations = super::get_simple_function_annotations("anonymous", &file, &analyses);

        assert!(
            !annotations.contains("[complexity:"),
            "no complexity was measured, so none may be printed: {annotations}"
        );
        assert!(
            !annotations.contains("[cognitive:"),
            "no cognitive complexity was measured: {annotations}"
        );
        assert!(
            !annotations.contains("[big-o:"),
            "big-o is derived from an unmeasured cyclomatic value: {annotations}"
        );
    }

    /// Measured functions must still be annotated — the fix above removes the
    /// fabricated fallback, not the real lookup.
    #[test]
    fn measured_function_still_gets_its_own_complexity() {
        use crate::services::complexity::{
            ComplexityMetrics, ComplexityReport, ComplexitySummary, FileComplexityMetrics,
            FunctionComplexity,
        };

        let metrics = ComplexityMetrics {
            cyclomatic: 6,
            cognitive: 4,
            nesting_max: 1,
            lines: 10,
            halstead: None,
        };

        let analyses = AnalysisResults {
            complexity_report: Some(ComplexityReport {
                summary: ComplexitySummary {
                    total_files: 1,
                    total_functions: 1,
                    median_cyclomatic: 6.0,
                    median_cognitive: 4.0,
                    max_cyclomatic: 6,
                    max_cognitive: 4,
                    p90_cyclomatic: 6,
                    p90_cognitive: 4,
                    technical_debt_hours: 0.0,
                },
                violations: vec![],
                hotspots: vec![],
                files: vec![FileComplexityMetrics {
                    path: "src/lib.rs".to_string(),
                    total_complexity: metrics,
                    functions: vec![FunctionComplexity {
                        name: "complex".to_string(),
                        line_start: 1,
                        line_end: 10,
                        metrics,
                    }],
                    classes: vec![],
                }],
            }),
            ..Default::default()
        };

        let annotations =
            super::get_simple_function_annotations("complex", &file_ctx("src/lib.rs"), &analyses);

        assert!(annotations.contains("[complexity: 6]"), "{annotations}");
        assert!(annotations.contains("[cognitive: 4]"), "{annotations}");
    }

    /// `[provability: N%]` was a project-wide mean (or the 0.75 default) stamped
    /// onto every row: one distinct value across 22,925 functions, and 92% on a
    /// toy crate whose functions include one that unwraps. `ProofSummary` has no
    /// function identity, so no per-function provability exists to print.
    #[test]
    fn provability_annotation_is_never_a_project_wide_constant() {
        use crate::services::lightweight_provability_analyzer::ProofSummary;

        // No analysis at all: the old code still printed the 0.75 default.
        let empty = AnalysisResults::default();
        let annotations = super::get_simple_function_annotations("add", &file_ctx("l.rs"), &empty);
        assert!(
            !annotations.contains("provability"),
            "provability is not measured per function: {annotations}"
        );

        // Analysis present: the old code averaged it and stamped the mean on
        // every function, however different the functions were.
        let with_results = AnalysisResults {
            provability_results: Some(vec![
                ProofSummary {
                    provability_score: 0.9,
                    verified_properties: vec![],
                    analysis_time_us: 0,
                    version: 1,
                },
                ProofSummary {
                    provability_score: 0.1,
                    verified_properties: vec![],
                    analysis_time_us: 0,
                    version: 1,
                },
            ]),
            ..Default::default()
        };
        for func in ["add", "dangerous"] {
            let annotations =
                super::get_simple_function_annotations(func, &file_ctx("l.rs"), &with_results);
            assert!(
                !annotations.contains("provability"),
                "{func}: {annotations}"
            );
        }
    }
}
