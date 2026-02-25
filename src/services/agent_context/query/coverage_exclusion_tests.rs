// ── Tests ────────────────────────────────────────────────────────────────────
// Included from coverage_exclusion.rs — do NOT add `use` imports or `#!` attributes here.

#[cfg(test)]
mod tests {
    use super::*;

    fn make_result(file_path: &str, function_name: &str) -> QueryResult {
        QueryResult {
            file_path: file_path.to_string(),
            function_name: function_name.to_string(),
            signature: format!("fn {}()", function_name),
            definition_type: "function".to_string(),
            doc_comment: None,
            start_line: 1,
            end_line: 10,
            language: "rust".to_string(),
            tdg_score: 5.0,
            tdg_grade: "C".to_string(),
            complexity: 5,
            big_o: "O(n)".to_string(),
            satd_count: 0,
            loc: 10,
            relevance_score: 0.0,
            source: None,
            calls: Vec::new(),
            called_by: Vec::new(),
            pagerank: 0.0,
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
            lines_total: 10,
            missed_lines: 10,
            impact_score: 0.0,
            coverage_status: "uncovered".to_string(),
            coverage_diff: 0.0,
            coverage_exclusion: CoverageExclusion::None,
            coverage_excluded: false,
            cross_project_callers: 0,
            io_classification: String::new(),
            io_patterns: Vec::new(),
            suggested_module: String::new(),
        }
    }

    #[test]
    fn test_coverage_exclusion_default() {
        let e = CoverageExclusion::default();
        assert_eq!(e, CoverageExclusion::None);
        assert!(e.is_none());
    }

    #[test]
    fn test_coverage_exclusion_labels() {
        assert_eq!(CoverageExclusion::None.label(), "testable");
        assert_eq!(CoverageExclusion::CoverageOff.label(), "coverage(off)");
        assert_eq!(CoverageExclusion::DeadCode.label(), "dead code");
        assert_eq!(
            CoverageExclusion::MakefileExcluded.label(),
            "Makefile pattern"
        );
    }

    #[test]
    fn test_coverage_exclusion_is_none() {
        assert!(CoverageExclusion::None.is_none());
        assert!(!CoverageExclusion::CoverageOff.is_none());
        assert!(!CoverageExclusion::DeadCode.is_none());
        assert!(!CoverageExclusion::MakefileExcluded.is_none());
    }

    #[test]
    fn test_classify_coverage_off_file() {
        let temp = tempfile::TempDir::new().unwrap();
        let src_dir = temp.path().join("src/cli");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(
            src_dir.join("handler.rs"),
            "#![cfg_attr(coverage_nightly, coverage(off))]\nfn foo() {}\n",
        )
        .unwrap();

        let mut ctx = ExclusionContext {
            coverage_off_files: HashSet::new(),
            checked_files: HashSet::new(),
            makefile_regex: None,
            dead_functions: HashSet::new(),
            use_cached: false,
        };

        let r = make_result("src/cli/handler.rs", "foo");
        let excl = ctx.classify(&r, temp.path());
        assert_eq!(excl, CoverageExclusion::CoverageOff);
    }

    #[test]
    fn test_classify_dead_code() {
        let temp = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(temp.path().join("src")).unwrap();
        std::fs::write(temp.path().join("src/lib.rs"), "fn dead_fn() {}\n").unwrap();

        let mut dead = HashSet::new();
        dead.insert("src/lib.rs::dead_fn".to_string());

        let mut ctx = ExclusionContext {
            coverage_off_files: HashSet::new(),
            checked_files: HashSet::new(),
            makefile_regex: None,
            dead_functions: dead,
            use_cached: false,
        };

        let r = make_result("src/lib.rs", "dead_fn");
        let excl = ctx.classify(&r, temp.path());
        assert_eq!(excl, CoverageExclusion::DeadCode);
    }

    #[test]
    fn test_classify_makefile_excluded() {
        let temp = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(temp.path().join("src/cli/handlers")).unwrap();
        std::fs::write(temp.path().join("src/cli/handlers/foo.rs"), "fn bar() {}\n").unwrap();

        let re = regex::Regex::new(r"/(cli|mcp[^/]*)/").unwrap();
        let mut ctx = ExclusionContext {
            coverage_off_files: HashSet::new(),
            checked_files: HashSet::new(),
            makefile_regex: Some(re),
            dead_functions: HashSet::new(),
            use_cached: false,
        };

        let r = make_result("src/cli/handlers/foo.rs", "bar");
        let excl = ctx.classify(&r, temp.path());
        assert_eq!(excl, CoverageExclusion::MakefileExcluded);
    }

    #[test]
    fn test_classify_testable() {
        let temp = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(temp.path().join("src/services")).unwrap();
        std::fs::write(
            temp.path().join("src/services/core.rs"),
            "fn important() {}\n",
        )
        .unwrap();

        let mut ctx = ExclusionContext {
            coverage_off_files: HashSet::new(),
            checked_files: HashSet::new(),
            makefile_regex: None,
            dead_functions: HashSet::new(),
            use_cached: false,
        };

        let r = make_result("src/services/core.rs", "important");
        let excl = ctx.classify(&r, temp.path());
        assert_eq!(excl, CoverageExclusion::None);
    }

    #[test]
    fn test_classify_exclusions_batch() {
        let temp = tempfile::TempDir::new().unwrap();
        let src = temp.path().join("src");
        std::fs::create_dir_all(src.join("services")).unwrap();
        std::fs::create_dir_all(src.join("cli")).unwrap();
        std::fs::write(src.join("services/core.rs"), "fn testable() {}\n").unwrap();
        std::fs::write(
            src.join("cli/handler.rs"),
            "#![cfg_attr(coverage_nightly, coverage(off))]\nfn excluded() {}\n",
        )
        .unwrap();

        let mut results = vec![
            make_result("src/services/core.rs", "testable"),
            make_result("src/cli/handler.rs", "excluded"),
        ];

        classify_exclusions(&mut results, temp.path(), None);

        assert_eq!(results[0].coverage_exclusion, CoverageExclusion::None);
        assert!(!results[0].coverage_excluded);
        assert_eq!(
            results[1].coverage_exclusion,
            CoverageExclusion::CoverageOff
        );
        assert!(results[1].coverage_excluded);
    }

    #[test]
    fn test_exclusion_summary() {
        let mut r1 = make_result("src/cli/a.rs", "f1");
        r1.coverage_exclusion = CoverageExclusion::CoverageOff;
        let mut r2 = make_result("src/cli/b.rs", "f2");
        r2.coverage_exclusion = CoverageExclusion::CoverageOff;
        let mut r3 = make_result("src/cli/a.rs", "f3");
        r3.coverage_exclusion = CoverageExclusion::CoverageOff;
        let mut r4 = make_result("src/dead.rs", "f4");
        r4.coverage_exclusion = CoverageExclusion::DeadCode;
        let mut r5 = make_result("src/mcp/x.rs", "f5");
        r5.coverage_exclusion = CoverageExclusion::MakefileExcluded;

        let refs: Vec<&QueryResult> = vec![&r1, &r2, &r3, &r4, &r5];
        let summary = ExclusionSummary::from_results(&refs);

        assert_eq!(summary.coverage_off_count, 3);
        assert_eq!(summary.coverage_off_files, 2);
        assert_eq!(summary.dead_code_count, 1);
        assert_eq!(summary.dead_code_files, 1);
        assert_eq!(summary.makefile_count, 1);
        assert_eq!(summary.makefile_files, 1);
        assert_eq!(summary.total(), 5);
        assert!(!summary.is_empty());
    }

    #[test]
    fn test_exclusion_summary_empty() {
        let summary = ExclusionSummary::from_results(&[]);
        assert!(summary.is_empty());
        assert_eq!(summary.total(), 0);
    }

    #[test]
    fn test_parse_makefile_coverage_exclude() {
        let temp = tempfile::TempDir::new().unwrap();
        std::fs::write(
            temp.path().join("Makefile"),
            "COVERAGE_EXCLUDE := --ignore-filename-regex='(_tests?\\.rs|/(tests|benches)/|main\\.rs)'\n",
        ).unwrap();

        let re = parse_makefile_coverage_exclude(temp.path());
        assert!(re.is_some());
        let re = re.unwrap();
        assert!(re.is_match("src/foo_test.rs"));
        assert!(re.is_match("src/tests/bar.rs"));
        assert!(re.is_match("main.rs"));
        assert!(!re.is_match("src/services/core.rs"));
    }

    #[test]
    fn test_parse_makefile_double_backslash_dot() {
        // Real Makefiles use `\\.` (double-backslash-dot) which cargo-llvm-cov
        // interprets as literal dot. Verify our normalization handles this.
        let temp = tempfile::TempDir::new().unwrap();
        // Write raw bytes with double-backslash (0x5C 0x5C) + dot (0x2E)
        let content = b"COVERAGE_EXCLUDE := --ignore-filename-regex='(build_perf_impl\\\\.rs|storage_impl\\\\.rs)'\n";
        std::fs::write(temp.path().join("Makefile"), content).unwrap();

        let re = parse_makefile_coverage_exclude(temp.path());
        assert!(re.is_some(), "Should parse double-backslash regex");
        let re = re.unwrap();
        // Must match actual file paths (without backslashes)
        assert!(
            re.is_match("src/services/build_perf_impl.rs"),
            "Should match build_perf_impl.rs with literal dot"
        );
        assert!(
            re.is_match("src/tdg/storage_impl.rs"),
            "Should match storage_impl.rs with literal dot"
        );
        // Must NOT match paths without the exact filename
        assert!(
            !re.is_match("src/services/core.rs"),
            "Should not match unrelated files"
        );
    }

    #[test]
    fn test_parse_makefile_no_file() {
        let temp = tempfile::TempDir::new().unwrap();
        let re = parse_makefile_coverage_exclude(temp.path());
        assert!(re.is_none());
    }

    #[test]
    fn test_load_dead_code_functions() {
        let temp = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(temp.path().join(".pmat")).unwrap();
        std::fs::write(
            temp.path().join(".pmat/dead-code-cache.json"),
            r#"{
                "report": {
                    "files_with_dead_code": [
                        {
                            "file_path": "src/old.rs",
                            "dead_items": [
                                {"name": "unused_fn", "kind": "function"},
                                {"name": "OldStruct", "kind": "struct"}
                            ],
                            "file_dead_percentage": 50.0
                        }
                    ]
                }
            }"#,
        )
        .unwrap();

        let dead = load_dead_code_functions(temp.path());
        assert!(dead.contains("src/old.rs::unused_fn"));
        assert!(dead.contains("src/old.rs::OldStruct"));
        assert_eq!(dead.len(), 2);
    }

    #[test]
    fn test_load_dead_code_no_cache() {
        let temp = tempfile::TempDir::new().unwrap();
        let dead = load_dead_code_functions(temp.path());
        assert!(dead.is_empty());
    }

    #[test]
    fn test_coverage_exclusion_serde_roundtrip() {
        let variants = vec![
            CoverageExclusion::None,
            CoverageExclusion::CoverageOff,
            CoverageExclusion::DeadCode,
            CoverageExclusion::MakefileExcluded,
        ];
        for v in variants {
            let json = serde_json::to_string(&v).unwrap();
            let deserialized: CoverageExclusion = serde_json::from_str(&json).unwrap();
            assert_eq!(v, deserialized);
        }
    }

    #[test]
    fn test_dead_code_priority_over_coverage_off() {
        // Dead code should be classified as DeadCode even if file also has coverage(off)
        let temp = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(temp.path().join("src")).unwrap();
        std::fs::write(
            temp.path().join("src/mixed.rs"),
            "#![cfg_attr(coverage_nightly, coverage(off))]\nfn dead_fn() {}\n",
        )
        .unwrap();

        let mut dead = HashSet::new();
        dead.insert("src/mixed.rs::dead_fn".to_string());

        let mut ctx = ExclusionContext {
            coverage_off_files: HashSet::new(),
            checked_files: HashSet::new(),
            makefile_regex: None,
            dead_functions: dead,
            use_cached: false,
        };

        let r = make_result("src/mixed.rs", "dead_fn");
        let excl = ctx.classify(&r, temp.path());
        // Dead code has higher priority
        assert_eq!(excl, CoverageExclusion::DeadCode);
    }

    #[test]
    fn test_coverage_off_caching() {
        let temp = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(temp.path().join("src")).unwrap();
        std::fs::write(
            temp.path().join("src/cached.rs"),
            "#![cfg_attr(coverage_nightly, coverage(off))]\nfn a() {}\nfn b() {}\n",
        )
        .unwrap();

        let mut ctx = ExclusionContext {
            coverage_off_files: HashSet::new(),
            checked_files: HashSet::new(),
            makefile_regex: None,
            dead_functions: HashSet::new(),
            use_cached: false,
        };

        // First check reads file
        let r1 = make_result("src/cached.rs", "a");
        assert_eq!(
            ctx.classify(&r1, temp.path()),
            CoverageExclusion::CoverageOff
        );
        // Second check uses cache
        assert!(ctx.coverage_off_files.contains("src/cached.rs"));
        let r2 = make_result("src/cached.rs", "b");
        assert_eq!(
            ctx.classify(&r2, temp.path()),
            CoverageExclusion::CoverageOff
        );
    }
}
