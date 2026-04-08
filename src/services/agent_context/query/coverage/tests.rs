#![cfg_attr(coverage_nightly, coverage(off))]

#[cfg(test)]
mod tests {
    use crate::services::agent_context::query::coverage::enrichment::{
        compute_impact_score, enrich_with_coverage, format_coverage_summary,
    };
    use crate::services::agent_context::query::coverage::parsing::try_load_lcov_info;
    use crate::services::agent_context::query::coverage::parsing::{
        build_coverage_map, parse_lcov_to_coverage_map, segments_to_line_hits,
    };
    use crate::services::agent_context::query::types::QueryResult;
    use std::collections::HashMap;
    use std::path::Path;

    #[test]
    fn test_segments_to_line_hits_empty() {
        let result = segments_to_line_hits(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_segments_to_line_hits_basic() {
        // Segments: [line, col, count, has_count, is_region_entry]
        let segments = vec![
            vec![
                json_u64(10),
                json_u64(1),
                json_u64(5),
                json_bool(true),
                json_bool(true),
            ],
            vec![
                json_u64(15),
                json_u64(1),
                json_u64(0),
                json_bool(true),
                json_bool(false),
            ],
            vec![
                json_u64(20),
                json_u64(1),
                json_u64(3),
                json_bool(true),
                json_bool(true),
            ],
        ];
        let hits = segments_to_line_hits(&segments);

        // Lines 10-15 should have count 5
        assert_eq!(hits.get(&10), Some(&5));
        assert_eq!(hits.get(&12), Some(&5));
        assert_eq!(hits.get(&15), Some(&5)); // inclusive of range

        // Lines 15-20 also get count from seg[1] (0) but seg[0] covers 10..=15 with 5
        // seg[1] covers 15..=20 with 0
        // For line 15, max(5, 0) = 5
        // For line 17, count from seg[1] is 0
        assert_eq!(hits.get(&17), Some(&0));

        // Lines 20+ should have count 3
        assert_eq!(hits.get(&20), Some(&3));
    }

    #[test]
    fn test_segments_to_line_hits_no_count() {
        // Segment with has_count=false should be skipped
        let segments = vec![vec![
            json_u64(10),
            json_u64(1),
            json_u64(5),
            json_bool(false),
            json_bool(true),
        ]];
        let hits = segments_to_line_hits(&segments);
        assert!(hits.is_empty());
    }

    #[test]
    fn test_enrich_with_coverage_basic() {
        let mut results = vec![make_result("src/main.rs", 10, 20, 0.5, 5)];

        let mut file_coverage = HashMap::new();
        let mut line_hits = HashMap::new();
        // Lines 10-15 covered, 16-20 uncovered
        for l in 10..=15 {
            line_hits.insert(l, 1);
        }
        for l in 16..=20 {
            line_hits.insert(l, 0);
        }
        file_coverage.insert("src/main.rs".to_string(), line_hits);

        enrich_with_coverage(&mut results, &file_coverage);

        assert_eq!(results[0].lines_covered, 6); // 10,11,12,13,14,15
        assert_eq!(results[0].lines_total, 11); // 10..=20
        assert_eq!(results[0].missed_lines, 5); // 16,17,18,19,20 — Coverage = 6/11 ~ 54.5%
        assert!((results[0].line_coverage_pct - 54.545).abs() < 1.0);
        assert!(results[0].impact_score > 0.0);
        assert_eq!(results[0].coverage_status, "partial");
    }

    #[test]
    fn test_enrich_with_coverage_no_file_match() {
        let mut results = vec![make_result("src/other.rs", 1, 10, 0.0, 1)];

        let file_coverage = HashMap::new();
        enrich_with_coverage(&mut results, &file_coverage);

        assert_eq!(results[0].lines_covered, 0);
        assert_eq!(results[0].lines_total, 0);
        assert_eq!(results[0].line_coverage_pct, 0.0);
        assert_eq!(results[0].coverage_status, "no_data");
    }

    #[test]
    fn test_enrich_with_coverage_non_instrumented_lines() {
        let mut results = vec![make_result("src/main.rs", 1, 10, 0.1, 3)];

        let mut file_coverage = HashMap::new();
        let mut line_hits = HashMap::new();
        // Only lines 3,5,7 are instrumented
        line_hits.insert(3, 1);
        line_hits.insert(5, 0);
        line_hits.insert(7, 1);
        file_coverage.insert("src/main.rs".to_string(), line_hits);

        enrich_with_coverage(&mut results, &file_coverage);

        assert_eq!(results[0].lines_total, 3); // only instrumented lines count
        assert_eq!(results[0].lines_covered, 2); // lines 3 and 7
        assert_eq!(results[0].missed_lines, 1); // line 5
    }

    #[test]
    fn test_compute_impact_score_zero_missed() {
        assert_eq!(compute_impact_score(0, 0.5, 10), 0.0);
    }

    #[test]
    fn test_compute_impact_score_basic() {
        // 10 missed lines, pagerank 0.001 (scaled to 10.0), complexity 5
        let score = compute_impact_score(10, 0.001, 5);
        // 10 * 10.0 / 5.0 = 20.0
        assert!((score - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_compute_impact_score_zero_pagerank() {
        // 0 pagerank floors to 0.1
        let score = compute_impact_score(5, 0.0, 1);
        // 5 * 0.1 / 1.0 = 0.5
        assert!((score - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_compute_impact_score_zero_complexity() {
        // 0 complexity floors to 1.0
        let score = compute_impact_score(10, 0.001, 0);
        // 10 * 10.0 / 1.0 = 100.0
        assert!((score - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_uncovered_only_filter() {
        let mut results = vec![
            make_result_with_coverage("src/a.rs", 100.0, 10, 10, 0),
            make_result_with_coverage("src/b.rs", 50.0, 5, 10, 5),
            make_result_with_coverage("src/c.rs", 0.0, 0, 10, 10),
            make_result_with_coverage("src/d.rs", 0.0, 0, 0, 0), // non-instrumented
        ];

        // Simulate --uncovered-only filter
        results.retain(|r| r.lines_total > 0 && r.line_coverage_pct < 100.0);

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].file_path, "src/b.rs");
        assert_eq!(results[1].file_path, "src/c.rs");
    }

    #[test]
    fn test_build_coverage_map_basic() {
        let json = r#"{
            "data": [{
                "files": [{
                    "filename": "/project/src/main.rs",
                    "segments": [
                        [10, 1, 5, true, true],
                        [15, 1, 0, true, false]
                    ],
                    "summary": {"lines": {"count": 20, "covered": 15}}
                }]
            }],
            "type": "llvm.coverage.json.export"
        }"#;

        let root = Path::new("/project");
        let map = build_coverage_map(json, root).unwrap();

        assert!(map.contains_key("src/main.rs"));
        let hits = &map["src/main.rs"];
        assert!(hits.contains_key(&10));
    }

    #[test]
    fn test_build_coverage_map_invalid_json() {
        let result = build_coverage_map("not json", Path::new("/project"));
        assert!(result.is_err());
    }

    // ── Test Helpers ────────────────────────────────────────────────────────

    fn json_u64(v: u64) -> serde_json::Value {
        serde_json::Value::Number(serde_json::Number::from(v))
    }

    fn json_bool(v: bool) -> serde_json::Value {
        serde_json::Value::Bool(v)
    }

    fn make_result(
        file_path: &str,
        start_line: usize,
        end_line: usize,
        pagerank: f32,
        complexity: u32,
    ) -> QueryResult {
        QueryResult {
            file_path: file_path.to_string(),
            function_name: "test_fn".to_string(),
            signature: "fn test_fn()".to_string(),
            definition_type: "function".to_string(),
            doc_comment: None,
            start_line,
            end_line,
            language: "rust".to_string(),
            tdg_score: 5.0,
            tdg_grade: "C".to_string(),
            complexity,
            big_o: "O(n)".to_string(),
            satd_count: 0,
            loc: (end_line - start_line + 1) as u32,
            relevance_score: 0.8,
            source: None,
            calls: Vec::new(),
            called_by: Vec::new(),
            pagerank,
            in_degree: 0,
            out_degree: 0,
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            duplication_score: 0.0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
            line_coverage_pct: 0.0,
            lines_covered: 0,
            lines_total: 0,
            missed_lines: 0,
            impact_score: 0.0,
            coverage_status: String::new(),
            coverage_diff: 0.0,
            coverage_exclusion: Default::default(),
            coverage_excluded: false,
            cross_project_callers: 0,
            io_classification: String::new(),
            io_patterns: Vec::new(),
            suggested_module: String::new(),
            contract_level: None,
            contract_equation: None,
        }
    }

    fn make_result_with_coverage(
        file_path: &str,
        coverage_pct: f32,
        covered: u32,
        total: u32,
        missed: u32,
    ) -> QueryResult {
        let mut r = make_result(file_path, 1, 10, 0.0, 1);
        r.line_coverage_pct = coverage_pct;
        r.lines_covered = covered;
        r.lines_total = total;
        r.missed_lines = missed;
        r
    }

    #[test]
    fn test_coverage_status_full() {
        let mut results = vec![make_result("src/main.rs", 1, 5, 0.1, 3)];
        let mut file_coverage = HashMap::new();
        let mut line_hits = HashMap::new();
        for l in 1..=5 {
            line_hits.insert(l, 1); // all covered
        }
        file_coverage.insert("src/main.rs".to_string(), line_hits);
        enrich_with_coverage(&mut results, &file_coverage);
        assert_eq!(results[0].coverage_status, "full");
        assert_eq!(results[0].line_coverage_pct, 100.0);
    }

    #[test]
    fn test_coverage_status_uncovered() {
        let mut results = vec![make_result("src/main.rs", 1, 5, 0.1, 3)];
        let mut file_coverage = HashMap::new();
        let mut line_hits = HashMap::new();
        for l in 1..=5 {
            line_hits.insert(l, 0); // all instrumented but zero hits
        }
        file_coverage.insert("src/main.rs".to_string(), line_hits);
        enrich_with_coverage(&mut results, &file_coverage);
        assert_eq!(results[0].coverage_status, "uncovered");
        assert_eq!(results[0].lines_covered, 0);
        assert_eq!(results[0].lines_total, 5);
    }

    #[test]
    fn test_coverage_status_no_data_when_no_instrumented_lines() {
        // File is in coverage map but no lines in the function's range are instrumented
        let mut results = vec![make_result("src/main.rs", 100, 110, 0.1, 3)];
        let mut file_coverage = HashMap::new();
        let mut line_hits = HashMap::new();
        line_hits.insert(1, 1); // only line 1 instrumented, function is at 100-110
        file_coverage.insert("src/main.rs".to_string(), line_hits);
        enrich_with_coverage(&mut results, &file_coverage);
        assert_eq!(results[0].coverage_status, "no_data");
        assert_eq!(results[0].lines_total, 0);
    }

    #[test]
    fn test_format_coverage_summary_basic() {
        let mut results = vec![
            make_result("src/a.rs", 1, 10, 0.01, 5),
            make_result("src/b.rs", 1, 10, 0.01, 5),
        ];
        results[0].coverage_status = "partial".to_string();
        results[0].lines_covered = 7;
        results[0].lines_total = 10;
        results[0].missed_lines = 3;
        results[0].impact_score = 5.0;
        results[0].function_name = "func_a".to_string();

        results[1].coverage_status = "uncovered".to_string();
        results[1].lines_covered = 0;
        results[1].lines_total = 10;
        results[1].missed_lines = 10;
        results[1].impact_score = 10.0;
        results[1].function_name = "func_b".to_string();

        let summary = format_coverage_summary(&results).unwrap();
        assert!(summary.contains("7/20 lines"));
        assert!(summary.contains("35.0%"));
        assert!(summary.contains("1 uncovered"));
        assert!(summary.contains("1 partial"));
        assert!(summary.contains("Top impact: func_b"));
    }

    #[test]
    fn test_format_coverage_summary_no_data() {
        let results = vec![make_result("src/a.rs", 1, 10, 0.0, 1)];
        // No coverage_status set (empty string) -> should return None
        assert!(format_coverage_summary(&results).is_none());
    }

    #[test]
    fn test_coverage_fault_annotation_uncovered() {
        let mut results = vec![make_result("src/main.rs", 1, 10, 0.1, 3)];
        let mut file_coverage = HashMap::new();
        let mut line_hits = HashMap::new();
        for l in 1..=10 {
            line_hits.insert(l, 0); // all instrumented, zero hits
        }
        file_coverage.insert("src/main.rs".to_string(), line_hits);
        enrich_with_coverage(&mut results, &file_coverage);

        // Should have NO_COVERAGE fault annotation
        assert!(
            results[0]
                .fault_annotations
                .iter()
                .any(|f| f.starts_with("NO_COVERAGE:")),
            "Expected NO_COVERAGE annotation, got: {:?}",
            results[0].fault_annotations
        );
        assert!(results[0].fault_annotations[0].contains("0/10 lines"));
    }

    #[test]
    fn test_coverage_fault_annotation_low_coverage() {
        let mut results = vec![make_result("src/main.rs", 1, 10, 0.1, 3)];
        let mut file_coverage = HashMap::new();
        let mut line_hits = HashMap::new();
        // 3 covered, 7 uncovered -> 30% coverage
        for l in 1..=3 {
            line_hits.insert(l, 1);
        }
        for l in 4..=10 {
            line_hits.insert(l, 0);
        }
        file_coverage.insert("src/main.rs".to_string(), line_hits);
        enrich_with_coverage(&mut results, &file_coverage);

        // Should have LOW_COVERAGE fault annotation (30% < 50%)
        assert!(
            results[0]
                .fault_annotations
                .iter()
                .any(|f| f.starts_with("LOW_COVERAGE:")),
            "Expected LOW_COVERAGE annotation, got: {:?}",
            results[0].fault_annotations
        );
    }

    #[test]
    fn test_coverage_fault_annotation_high_impact() {
        // High pagerank = high impact score when lines are missed
        let mut results = vec![make_result("src/main.rs", 1, 20, 0.01, 2)];
        let mut file_coverage = HashMap::new();
        let mut line_hits = HashMap::new();
        for l in 1..=5 {
            line_hits.insert(l, 1);
        }
        for l in 6..=20 {
            line_hits.insert(l, 0);
        }
        file_coverage.insert("src/main.rs".to_string(), line_hits);
        enrich_with_coverage(&mut results, &file_coverage);

        // Impact = 15 missed * (0.01*10000=100.0) / 2.0 = 750.0 -> COVERAGE_RISK
        assert!(results[0].impact_score > 5.0);
        assert!(
            results[0]
                .fault_annotations
                .iter()
                .any(|f| f.starts_with("COVERAGE_RISK:")),
            "Expected COVERAGE_RISK annotation for impact={:.1}, got: {:?}",
            results[0].impact_score,
            results[0].fault_annotations
        );
    }

    #[test]
    fn test_coverage_no_fault_annotation_when_fully_covered() {
        let mut results = vec![make_result("src/main.rs", 1, 5, 0.1, 3)];
        let mut file_coverage = HashMap::new();
        let mut line_hits = HashMap::new();
        for l in 1..=5 {
            line_hits.insert(l, 1);
        }
        file_coverage.insert("src/main.rs".to_string(), line_hits);
        enrich_with_coverage(&mut results, &file_coverage);

        // Fully covered -> no coverage fault annotations
        let coverage_faults: Vec<_> = results[0]
            .fault_annotations
            .iter()
            .filter(|f| {
                f.starts_with("NO_COVERAGE:")
                    || f.starts_with("LOW_COVERAGE:")
                    || f.starts_with("COVERAGE_RISK:")
            })
            .collect();
        assert!(
            coverage_faults.is_empty(),
            "Fully covered function should have no coverage faults, got: {:?}",
            coverage_faults
        );
    }

    #[test]
    fn test_build_coverage_map_skips_external_files() {
        // Files outside project root should be skipped (deps, registry, etc.)
        let json = r#"{
            "data": [{
                "files": [
                    {
                        "filename": "/project/src/main.rs",
                        "segments": [[10, 1, 5, true, true], [15, 1, 0, true, false]],
                        "summary": {"lines": {"count": 20, "covered": 15}}
                    },
                    {
                        "filename": "/home/user/.cargo/registry/src/crates.io-abc/dep-1.0.0/src/lib.rs",
                        "segments": [[1, 1, 3, true, true], [10, 1, 0, true, false]],
                        "summary": {"lines": {"count": 10, "covered": 5}}
                    }
                ]
            }],
            "type": "llvm.coverage.json.export"
        }"#;

        let root = Path::new("/project");
        let map = build_coverage_map(json, root).unwrap();

        // Only project file should be included
        assert_eq!(
            map.len(),
            1,
            "Should only contain project files, got: {:?}",
            map.keys().collect::<Vec<_>>()
        );
        assert!(map.contains_key("src/main.rs"));
    }

    #[test]
    fn test_parse_lcov_basic() {
        let lcov = "\
SF:/project/src/lib.rs
DA:1,5
DA:2,3
DA:3,0
end_of_record
SF:/project/src/main.rs
DA:10,1
end_of_record
";
        let root = Path::new("/project");
        let map = parse_lcov_to_coverage_map(lcov, root);
        assert_eq!(map.len(), 2);
        let lib = map.get("src/lib.rs").unwrap();
        assert_eq!(lib.get(&1), Some(&5));
        assert_eq!(lib.get(&2), Some(&3));
        assert_eq!(lib.get(&3), Some(&0));
        let main = map.get("src/main.rs").unwrap();
        assert_eq!(main.get(&10), Some(&1));
    }

    #[test]
    fn test_parse_lcov_empty() {
        let map = parse_lcov_to_coverage_map("", Path::new("/project"));
        assert!(map.is_empty());
    }

    #[test]
    fn test_parse_lcov_relative_paths() {
        let lcov = "\
SF:src/foo.rs
DA:1,10
end_of_record
";
        let root = Path::new("/project");
        let map = parse_lcov_to_coverage_map(lcov, root);
        assert_eq!(map.len(), 1);
        assert!(map.contains_key("src/foo.rs"));
    }

    #[test]
    fn test_try_load_lcov_info() {
        let temp = tempfile::TempDir::new().unwrap();
        let cov_dir = temp.path().join("target/coverage");
        std::fs::create_dir_all(&cov_dir).unwrap();
        std::fs::write(
            cov_dir.join("lcov.info"),
            format!(
                "SF:{}/src/lib.rs\nDA:1,5\nend_of_record\n",
                temp.path().display()
            ),
        )
        .unwrap();

        let result = try_load_lcov_info(temp.path());
        assert!(result.is_some());
        let map = result.unwrap();
        assert_eq!(map.len(), 1);
        assert!(map.contains_key("src/lib.rs"));
    }

    #[test]
    fn test_try_load_lcov_info_missing() {
        let temp = tempfile::TempDir::new().unwrap();
        let result = try_load_lcov_info(temp.path());
        assert!(result.is_none());
    }
}
