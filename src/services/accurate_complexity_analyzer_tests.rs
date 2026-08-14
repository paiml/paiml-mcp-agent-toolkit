// Included from accurate_complexity_analyzer.rs — NO use imports or #! attributes

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_basic_complexity() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        fs::write(
            &test_file,
            r#"
            fn simple() -> i32 {
                42
            }
            
            fn with_if(x: i32) -> i32 {
                if x > 0 {
                    x
                } else {
                    -x
                }
            }
        "#,
        )
        .unwrap();

        let analyzer = AccurateComplexityAnalyzer::new();
        let result = analyzer.analyze_file(&test_file).await.unwrap();

        assert_eq!(result.functions.len(), 2);
        assert_eq!(result.functions[0].cyclomatic_complexity, 1);
        assert_eq!(result.functions[1].cyclomatic_complexity, 2);
    }

    #[tokio::test]
    async fn test_line_numbers_are_accurate() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        // Write file with known line positions
        fs::write(
            &test_file,
            "fn first() -> i32 {\n    42\n}\n\nfn second() -> i32 {\n    99\n}\n",
        )
        .unwrap();
        // first() is at line 1, second() is at line 5

        let analyzer = AccurateComplexityAnalyzer::new();
        let result = analyzer.analyze_file(&test_file).await.unwrap();

        assert_eq!(result.functions.len(), 2);
        assert_eq!(result.functions[0].name, "first");
        assert_eq!(result.functions[0].line_start, 1);
        assert_eq!(result.functions[1].name, "second");
        assert_eq!(result.functions[1].line_start, 5);
    }

    /// #652/#656 regression: extents and file length must be measured.
    /// Pre-fix, `line_end` was not recorded at all and callers substituted
    /// `line_start + 50` for the last function.
    #[tokio::test]
    async fn test_line_end_and_total_lines_are_measured() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        fs::write(
            &test_file,
            "fn first() -> i32 {\n    42\n}\n\nfn second() -> i32 {\n    99\n}\n",
        )
        .unwrap();

        let analyzer = AccurateComplexityAnalyzer::new();
        let result = analyzer.analyze_file(&test_file).await.unwrap();

        assert_eq!(result.total_lines, 7, "file is 7 lines long");
        assert_eq!(result.functions[0].line_end, 3);
        assert_eq!(result.functions[1].line_end, 7);
        for func in &result.functions {
            assert!(
                func.line_end <= result.total_lines,
                "{} ends at {} but the file has {} lines",
                func.name,
                func.line_end,
                result.total_lines
            );
        }
    }

    /// #656 exact reproduction: a one-line file.
    #[tokio::test]
    async fn test_single_line_file_does_not_report_51_lines() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("one.rs");
        fs::write(&test_file, "fn only_one() -> i32 { 42 }\n").unwrap();

        let analyzer = AccurateComplexityAnalyzer::new();
        let result = analyzer.analyze_file(&test_file).await.unwrap();

        assert_eq!(result.total_lines, 1);
        assert_eq!(result.functions.len(), 1);
        assert_eq!(result.functions[0].line_start, 1);
        assert_eq!(result.functions[0].line_end, 1);
    }

    /// Nesting depth is measured, not derived as `cognitive / 3`.
    #[tokio::test]
    async fn test_max_nesting_is_measured() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("nest.rs");
        fs::write(
            &test_file,
            "fn deep(a: i32) -> i32 {\n    if a > 0 {\n        if a > 1 {\n            if a > 2 {\n                return 3;\n            }\n        }\n    }\n    0\n}\n",
        )
        .unwrap();

        let analyzer = AccurateComplexityAnalyzer::new();
        let result = analyzer.analyze_file(&test_file).await.unwrap();
        assert_eq!(result.functions[0].max_nesting, 3);
    }

    #[tokio::test]
    async fn test_line_numbers_with_attributes() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        fs::write(
            &test_file,
            "#[inline]\npub fn decorated() -> i32 {\n    42\n}\n\n/// doc comment\npub async fn async_fn() {}\n",
        )
        .unwrap();
        // decorated() fn keyword is at line 2, async_fn() fn keyword is at line 7

        let analyzer = AccurateComplexityAnalyzer::new();
        let result = analyzer.analyze_file(&test_file).await.unwrap();

        assert_eq!(result.functions.len(), 2);
        assert_eq!(result.functions[0].name, "decorated");
        assert_eq!(result.functions[0].line_start, 2);
        assert_eq!(result.functions[1].name, "async_fn");
        assert_eq!(result.functions[1].line_start, 7);
    }

    // Updated for the span API: the map now records the measured end line too,
    // because recording only the start is what forced callers to invent one.
    #[test]
    fn test_build_function_line_map() {
        let content = "fn foo() {}\n\npub fn bar() {}\n\nasync fn baz() {}\n";
        let map = build_function_line_map(content);
        assert_eq!(map.peek("foo"), Some(LineSpan { start: 1, end: 1 }));
        assert_eq!(map.peek("bar"), Some(LineSpan { start: 3, end: 3 }));
        assert_eq!(map.peek("baz"), Some(LineSpan { start: 5, end: 5 }));
        assert_eq!(map.total_lines(), 5);
    }

    #[test]
    fn test_build_function_line_map_skips_comments() {
        let content = "// fn not_a_function() {}\nfn real() {}\n";
        let map = build_function_line_map(content);
        assert_eq!(map.peek("real"), Some(LineSpan { start: 2, end: 2 }));
        assert_eq!(map.peek("not_a_function"), None);
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_analyzer_new_defaults() {
        let analyzer = AccurateComplexityAnalyzer::new();
        // Default values checked via builder methods
        assert!(!analyzer.exclude_tests);
        assert!(!analyzer.respect_annotations);
    }

    #[test]
    fn test_analyzer_builder_exclude_tests() {
        let analyzer = AccurateComplexityAnalyzer::new().exclude_tests(true);
        assert!(analyzer.exclude_tests);
    }

    #[test]
    fn test_analyzer_builder_respect_annotations() {
        let analyzer = AccurateComplexityAnalyzer::new().respect_annotations(true);
        assert!(analyzer.respect_annotations);
    }

    #[test]
    fn test_analyzer_builder_chaining() {
        let analyzer = AccurateComplexityAnalyzer::new()
            .exclude_tests(true)
            .respect_annotations(true);
        assert!(analyzer.exclude_tests);
        assert!(analyzer.respect_annotations);
    }

    #[test]
    fn test_is_test_file_test_suffix() {
        let analyzer = AccurateComplexityAnalyzer::new().exclude_tests(true);
        let path = Path::new("src/foo_test.rs");
        assert!(analyzer.is_test_file(path));
    }

    #[test]
    fn test_is_test_file_tests_suffix() {
        let analyzer = AccurateComplexityAnalyzer::new().exclude_tests(true);
        let path = Path::new("src/foo_tests.rs");
        assert!(analyzer.is_test_file(path));
    }

    #[test]
    fn test_is_test_file_tests_dir() {
        let analyzer = AccurateComplexityAnalyzer::new().exclude_tests(true);
        // Need full path with /tests/ to match
        let path = Path::new("src/tests/integration.rs");
        assert!(analyzer.is_test_file(path));
    }

    #[test]
    fn test_is_test_file_regular_file() {
        let analyzer = AccurateComplexityAnalyzer::new().exclude_tests(true);
        let path = Path::new("src/lib.rs");
        assert!(!analyzer.is_test_file(path));
    }

    #[test]
    fn test_is_test_file_always_checks() {
        let analyzer = AccurateComplexityAnalyzer::new().exclude_tests(false);
        let path = Path::new("src/foo_test.rs");
        // is_test_file always checks the path, regardless of exclude_tests flag
        // The flag is only used in analyze_project to skip files
        assert!(analyzer.is_test_file(path));
    }

    // `extract_fn_name` moved to `services::source_line_index`; its cases are
    // covered by `source_line_index::tests::extract_fn_name_handles_qualifiers_and_rejects_non_functions`.

    #[test]
    fn test_default_impl() {
        let analyzer = AccurateComplexityAnalyzer::default();
        assert!(!analyzer.exclude_tests);
        assert!(!analyzer.respect_annotations);
    }

    // --- analyze_function + has_suppress_annotation branch coverage ---
    //
    // accurate_complexity_analyzer_core.rs:63
    //   let suppressed = self.respect_annotations && self.has_suppress_annotation(&func.attrs);
    // is BH-MUT-0002 (`&&` → `||`). To kill it, assert all 4 truth-table cells
    // of (respect_annotations, has_suppress_annotation).

    fn parse_fn(src: &str) -> ItemFn {
        syn::parse_str::<ItemFn>(src).expect("parse fn")
    }

    #[test]
    fn test_analyze_function_suppressed_when_both_true() {
        // respect_annotations=true AND #[allow(complex_function)] → suppressed=true.
        let func = parse_fn("#[allow(complex_function)] fn f() { if true {} else {} }");
        let analyzer = AccurateComplexityAnalyzer::new().respect_annotations(true);
        let metrics = analyzer.analyze_function(&func, LineSpan { start: 1, end: 1 });
        assert_eq!(metrics.name, "f");
        assert!(metrics.suppressed, "both sides true: must be suppressed");
    }

    #[test]
    fn test_analyze_function_not_suppressed_when_only_flag_true() {
        // Kills `&&`→`||`: flag true but no attribute → suppressed must be false.
        let func = parse_fn("fn f() { if true {} else {} }");
        let analyzer = AccurateComplexityAnalyzer::new().respect_annotations(true);
        let metrics = analyzer.analyze_function(&func, LineSpan { start: 1, end: 1 });
        assert!(
            !metrics.suppressed,
            "respect_annotations alone must not suppress"
        );
    }

    #[test]
    fn test_analyze_function_not_suppressed_when_only_attr_true() {
        // Kills `&&`→`||`: attribute present but respect_annotations=false → suppressed=false.
        let func = parse_fn("#[allow(complex_function)] fn f() { if true {} else {} }");
        let analyzer = AccurateComplexityAnalyzer::new(); // respect_annotations defaults to false
        let metrics = analyzer.analyze_function(&func, LineSpan { start: 1, end: 1 });
        assert!(
            !metrics.suppressed,
            "attr alone must not suppress without respect_annotations=true"
        );
    }

    #[test]
    fn test_analyze_function_not_suppressed_when_both_false() {
        let func = parse_fn("fn f() {}");
        let analyzer = AccurateComplexityAnalyzer::new();
        let metrics = analyzer.analyze_function(&func, LineSpan { start: 42, end: 44 });
        assert!(!metrics.suppressed);
        assert_eq!(metrics.line_start, 42, "line_start must be propagated");
    }

    #[test]
    fn test_has_suppress_annotation_non_allow_attr() {
        // Branch: attribute is not `allow(..)` → the `else { false }` arm.
        let func = parse_fn("#[inline] fn f() {}");
        let analyzer = AccurateComplexityAnalyzer::new();
        assert!(!analyzer.has_suppress_annotation(&func.attrs));
    }

    #[test]
    fn test_has_suppress_annotation_allow_without_complex_function() {
        // Branch: allow(..) but not `complex_function` → returns false.
        let func = parse_fn("#[allow(dead_code)] fn f() {}");
        let analyzer = AccurateComplexityAnalyzer::new();
        assert!(!analyzer.has_suppress_annotation(&func.attrs));
    }

    #[test]
    fn test_has_suppress_annotation_allow_complex_function() {
        // Branch: allow(complex_function) → returns true.
        let func = parse_fn("#[allow(complex_function)] fn f() {}");
        let analyzer = AccurateComplexityAnalyzer::new();
        assert!(analyzer.has_suppress_annotation(&func.attrs));
    }

    #[test]
    fn test_has_suppress_annotation_mixed_multi_attr() {
        // Multiple attributes; any matching allow(complex_function) triggers true.
        let func = parse_fn("#[inline] #[allow(complex_function)] fn f() {}");
        let analyzer = AccurateComplexityAnalyzer::new();
        assert!(analyzer.has_suppress_annotation(&func.attrs));
    }

    #[test]
    fn test_has_suppress_annotation_allow_path_no_list() {
        // `#[allow = "x"]` has no token list → `require_list()` errs → falls to "" which
        // doesn't contain "complex_function" → returns false. Covers the `.unwrap_or_default()` arm.
        let func = parse_fn(r#"#[allow = "whatever"] fn f() {}"#);
        let analyzer = AccurateComplexityAnalyzer::new();
        assert!(!analyzer.has_suppress_annotation(&func.attrs));
    }

    // --- Functions are not only the free ones ---
    //
    // The walker matched top-level `Item::Fn` and nothing else, so a file whose
    // functions all lived in `impl` / `trait` / `mod` blocks measured as zero
    // functions with max cyclomatic 0 — and a complexity gate reading that
    // passes everything. Idiomatic Rust is mostly `impl` blocks.

    /// The same body must measure the same wherever it is declared.
    async fn measured(source: &str) -> FileComplexityResult {
        let dir = tempfile::TempDir::new().unwrap();
        let file = dir.path().join("lib.rs");
        std::fs::write(&file, source).unwrap();
        AccurateComplexityAnalyzer::new()
            .analyze_file(&file)
            .await
            .unwrap()
    }

    const BRANCHY_BODY: &str = "        let mut n = 0;\n        if x > 1 { n += 1; }\n        n\n";

    #[tokio::test]
    async fn test_impl_methods_are_counted_like_free_functions() {
        let free = measured(&format!("pub fn f(x: i32) -> i32 {{\n{BRANCHY_BODY}}}\n")).await;
        let in_impl = measured(&format!(
            "pub struct S;\nimpl S {{\n    pub fn f(&self, x: i32) -> i32 {{\n{BRANCHY_BODY}    }}\n}}\n"
        ))
        .await;

        assert_eq!(free.functions.len(), 1);
        assert_eq!(
            in_impl.functions.len(),
            1,
            "impl methods used to be invisible: 0 functions, max cyclomatic 0"
        );
        assert_eq!(
            in_impl.functions[0].cyclomatic_complexity, free.functions[0].cyclomatic_complexity,
            "the same body must score the same inside an impl block"
        );
        assert_eq!(in_impl.functions[0].name, "f");
        // Extent is measured, not invented: the method spans its own 5 lines.
        assert_eq!(
            (
                in_impl.functions[0].line_start,
                in_impl.functions[0].line_end
            ),
            (3, 7)
        );
    }

    #[tokio::test]
    async fn test_trait_default_methods_are_counted_and_required_ones_are_not() {
        let result = measured(&format!(
            "pub trait T {{\n    fn provided(&self, x: i32) -> i32 {{\n{BRANCHY_BODY}    }}\n    fn required(&self) -> u8;\n}}\n"
        ))
        .await;

        let names: Vec<&str> = result.functions.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(
            names,
            vec!["provided"],
            "a default method has a body to measure; a required one has none"
        );
        assert_eq!(result.functions[0].cyclomatic_complexity, 2);
    }

    #[tokio::test]
    async fn test_functions_inside_inline_modules_are_counted() {
        let result = measured(&format!(
            "pub mod outer {{\n    pub mod inner {{\n        pub fn deep(x: i32) -> i32 {{\n{BRANCHY_BODY}        }}\n    }}\n}}\n"
        ))
        .await;

        assert_eq!(
            result.functions.len(),
            1,
            "nested `mod` bodies were skipped"
        );
        assert_eq!(result.functions[0].name, "deep");
        assert_eq!(result.functions[0].cyclomatic_complexity, 2);
    }

    #[tokio::test]
    async fn test_all_declaration_sites_are_walked_in_source_order() {
        // Spans are consumed by name in textual order, so a name reused across
        // scopes must be visited in the order it appears or every extent shifts.
        let result = measured(concat!(
            "impl S {\n",
            "    fn go(&self) {}\n",
            "}\n",
            "mod m {\n",
            "    pub fn go() {\n",
            "        ()\n",
            "    }\n",
            "}\n",
            "fn go() {}\n",
        ))
        .await;

        let spans: Vec<(u32, u32)> = result
            .functions
            .iter()
            .map(|f| (f.line_start, f.line_end))
            .collect();
        assert_eq!(
            spans,
            vec![(2, 2), (5, 7), (9, 9)],
            "impl / mod / free `go` must be measured in source order"
        );
    }
}
