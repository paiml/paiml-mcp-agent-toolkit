// Regression tests for the Big-O analyzer fabrications reported in
// GitHub issues #655 and #661.
//
// This module is declared inside `big_o_analyzer.rs`, so it can call the
// private per-file entry point and inspect *every* function — the public report
// only surfaces O(n^2)-and-worse functions.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod fabrication_tests {
    use crate::services::big_o_analyzer::{
        BigOAnalysisConfig, BigOAnalyzer, BigOClass, FunctionComplexity,
    };
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// The #655 fixture: one constant-time, one linear, one quadratic function
    /// and `main`.
    const MIXED_SOURCE: &str = concat!(
        "pub fn constant_time(a: i32, b: i32) -> i32 {\n",
        "    a + b\n",
        "}\n",
        "\n",
        "pub fn linear(v: &[i32]) -> i32 {\n",
        "    let mut s = 0;\n",
        "    for x in v {\n",
        "        s += x;\n",
        "    }\n",
        "    s\n",
        "}\n",
        "\n",
        "pub fn quadratic(v: &[i32]) -> i32 {\n",
        "    let mut s = 0;\n",
        "    for x in v {\n",
        "        for y in v {\n",
        "            s += x * y;\n",
        "        }\n",
        "    }\n",
        "    s\n",
        "}\n",
        "\n",
        "fn main() {\n",
        "    println!(\"{}\", constant_time(1, 2));\n",
        "}\n",
    );

    fn config(path: &std::path::Path, threshold: u8) -> BigOAnalysisConfig {
        BigOAnalysisConfig {
            project_path: path.to_path_buf(),
            include_patterns: vec!["*.rs".to_string()],
            exclude_patterns: vec![],
            confidence_threshold: threshold,
            analyze_space_complexity: false,
        }
    }

    /// Analyze one source string and return every function the analyzer kept.
    async fn functions_of(source: &str, threshold: u8) -> (TempDir, Vec<FunctionComplexity>) {
        let temp_dir = TempDir::new().unwrap();
        let file: PathBuf = temp_dir.path().join("lib.rs");
        fs::write(&file, source).unwrap();
        let analyzer = BigOAnalyzer::new();
        let cfg = config(temp_dir.path(), threshold);
        let functions = analyzer.analyze_file(&file, &cfg).await.unwrap();
        (temp_dir, functions)
    }

    fn find<'a>(functions: &'a [FunctionComplexity], name: &str) -> &'a FunctionComplexity {
        functions
            .iter()
            .find(|f| f.function_name == name)
            .unwrap_or_else(|| {
                panic!(
                    "{name} must be analyzed; got {:?}",
                    functions
                        .iter()
                        .map(|f| f.function_name.as_str())
                        .collect::<Vec<_>>()
                )
            })
    }

    /// #655: every function in a file was given the same complexity class,
    /// because each one was handed the entire rest of the file as its body.
    #[tokio::test]
    async fn test_each_function_gets_its_own_complexity_class() {
        let (_dir, functions) = functions_of(MIXED_SOURCE, 0).await;

        assert_eq!(
            find(&functions, "constant_time").time_complexity.class,
            BigOClass::Constant
        );
        assert_eq!(
            find(&functions, "linear").time_complexity.class,
            BigOClass::Linear
        );
        assert_eq!(
            find(&functions, "quadratic").time_complexity.class,
            BigOClass::Quadratic
        );
        assert_eq!(
            find(&functions, "main").time_complexity.class,
            BigOClass::Constant
        );
    }

    /// #655: the report used to claim three quadratic functions in this file.
    #[tokio::test]
    async fn test_distribution_counts_one_quadratic_not_three() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("lib.rs"), MIXED_SOURCE).unwrap();
        let report = BigOAnalyzer::new()
            .analyze(config(temp_dir.path(), 0))
            .await
            .unwrap();

        assert_eq!(report.analyzed_functions, 4, "main used to be dropped");
        assert_eq!(report.complexity_distribution.quadratic, 1);
        assert_eq!(report.complexity_distribution.linear, 1);
        assert_eq!(report.complexity_distribution.constant, 2);
    }

    /// #655: reported lines must be real source lines.
    #[tokio::test]
    async fn test_line_numbers_are_real_source_lines() {
        let (_dir, functions) = functions_of(MIXED_SOURCE, 0).await;

        assert_eq!(find(&functions, "constant_time").line_number, 1);
        assert_eq!(find(&functions, "linear").line_number, 5);
        assert_eq!(find(&functions, "quadratic").line_number, 13);
        assert_eq!(find(&functions, "main").line_number, 23);

        let total_lines = MIXED_SOURCE.lines().count();
        for func in &functions {
            assert!(
                func.line_number >= 1 && func.line_number <= total_lines,
                "{} reported line {}, outside a {}-line file",
                func.function_name,
                func.line_number,
                total_lines
            );
        }
    }

    /// #655: `/// fn 2: ...` doc comments were parsed as functions named "2".
    #[tokio::test]
    async fn test_function_names_never_come_from_comments() {
        let source = concat!(
            "/// fn 2: doubles the input\n",
            "/// fn 3: triples the input\n",
            "pub fn doubler(x: i32) -> i32 {\n",
            "    x * 2\n",
            "}\n",
        );
        let (_dir, functions) = functions_of(source, 0).await;

        let names: Vec<&str> = functions.iter().map(|f| f.function_name.as_str()).collect();
        assert_eq!(names, vec!["doubler"], "got {names:?}");
    }

    /// #661: loop-free functions were all dropped, so `analyze big-o` reported
    /// "Total Functions Analyzed: 0" on files that demonstrably have functions.
    #[tokio::test]
    async fn test_loop_free_functions_are_counted_at_the_default_threshold() {
        let source = concat!(
            "pub fn add(a: i32, b: i32) -> i32 { a + b }\n",
            "pub fn sub(a: i32, b: i32) -> i32 { a - b }\n",
        );
        // 50 is the shipped default; the bug produced confidence 45.
        let (_dir, functions) = functions_of(source, 50).await;

        assert_eq!(
            functions.len(),
            2,
            "both functions must survive the threshold"
        );
        for func in &functions {
            assert_eq!(func.time_complexity.class, BigOClass::Constant);
        }
    }

    /// #661: an if/else function with a multi-line body was silently dropped.
    #[tokio::test]
    async fn test_branching_function_is_not_dropped() {
        let source = concat!(
            "pub fn classify(a: i32) -> i32 {\n",
            "    if a > 0 {\n",
            "        1\n",
            "    } else {\n",
            "        -1\n",
            "    }\n",
            "}\n",
        );
        let (_dir, functions) = functions_of(source, 50).await;
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].function_name, "classify");
    }

    /// Real recursion must still be detected now that the declaration line no
    /// longer counts as a self-call.
    #[tokio::test]
    async fn test_real_recursion_is_still_detected() {
        let source = concat!(
            "pub fn fact(n: u64) -> u64 {\n",
            "    if n == 0 {\n",
            "        1\n",
            "    } else {\n",
            "        n * fact(n - 1)\n",
            "    }\n",
            "}\n",
        );
        let (_dir, functions) = functions_of(source, 0).await;
        let fact = find(&functions, "fact");
        assert!(
            fact.notes.iter().any(|n| n.contains("Recursive")),
            "notes: {:?}",
            fact.notes
        );
    }

    /// `format!(..)` starts with "for" but is not a loop.
    #[tokio::test]
    async fn test_format_macro_is_not_a_loop() {
        let source = concat!(
            "pub fn label(x: i32) -> String {\n",
            "    format!(\"{}\", x)\n",
            "}\n",
        );
        let (_dir, functions) = functions_of(source, 0).await;
        assert_eq!(
            find(&functions, "label").time_complexity.class,
            BigOClass::Constant
        );
    }

    /// Qualifiers that the old prefix list missed must not hide a function.
    #[tokio::test]
    async fn test_qualified_declarations_are_found() {
        let source = concat!(
            "pub(crate) async fn fetch(v: &[i32]) -> i32 {\n",
            "    v.len() as i32\n",
            "}\n",
            "\n",
            "const fn shift(x: i32) -> i32 {\n",
            "    x << 1\n",
            "}\n",
            "\n",
            "pub unsafe fn raw(x: i32) -> i32 {\n",
            "    x\n",
            "}\n",
        );
        let (_dir, functions) = functions_of(source, 0).await;
        let mut names: Vec<&str> = functions.iter().map(|f| f.function_name.as_str()).collect();
        names.sort_unstable();
        assert_eq!(names, vec!["fetch", "raw", "shift"]);
    }

    /// output_derived_from_input: an empty project yields an empty report.
    #[tokio::test]
    async fn test_empty_project_yields_empty_report() {
        let temp_dir = TempDir::new().unwrap();
        let report = BigOAnalyzer::new()
            .analyze(config(temp_dir.path(), 0))
            .await
            .unwrap();
        assert_eq!(report.analyzed_functions, 0);
        assert!(report.high_complexity_functions.is_empty());
    }
}
