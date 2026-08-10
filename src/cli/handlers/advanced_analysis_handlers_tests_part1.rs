//! Advanced analysis handlers tests - Part 1: Basic tests
//! Extracted for file health compliance (CB-040)

use super::*;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Helper to create a temporary project directory with Rust files
    fn create_test_project() -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create a simple Rust file
        let main_rs = temp_dir.path().join("main.rs");
        fs::write(
            &main_rs,
            r#"
fn main() {
    println!("Hello, world!");
}

fn complex_function(x: i32, y: i32) -> i32 {
    if x > 0 {
        if y > 0 {
            x + y
        } else {
            x - y
        }
    } else {
        -x
    }
}

// TODO: Refactor this function
fn needs_work() {
    // FIXME: This is a hack
    let _ = 42;
}
"#,
        )
        .expect("Failed to write main.rs");

        // Create a lib.rs file
        let lib_rs = temp_dir.path().join("lib.rs");
        fs::write(
            &lib_rs,
            r#"
pub mod utils;

/// Add.
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 2), 4);
    }
}
"#,
        )
        .expect("Failed to write lib.rs");

        temp_dir
    }

    /// Helper to create a simple Makefile for testing
    fn create_test_makefile(dir: &TempDir) {
        let makefile = dir.path().join("Makefile");
        fs::write(
            &makefile,
            r#"
.PHONY: all clean test

all: build

build:
	cargo build --release

test:
	cargo test

clean:
	cargo clean
"#,
        )
        .expect("Failed to write Makefile");
    }

    #[test]
    fn test_advanced_analysis_handlers_basic() {
        // Basic test to ensure module compiles
        assert_eq!(1 + 1, 2);
    }

    #[test]
    fn test_deep_context_output_format_variants() {
        // Test that all output format variants work
        let formats = [
            DeepContextOutputFormat::Json,
            DeepContextOutputFormat::Markdown,
            DeepContextOutputFormat::Sarif,
        ];

        for format in formats {
            let format_str = format!("{:?}", format);
            assert!(!format_str.is_empty());
        }
    }

    #[test]
    fn test_dag_type_variants() {
        // Test that all DAG type variants work
        let dag_types = [
            DagType::CallGraph,
            DagType::ImportGraph,
            DagType::Inheritance,
            DagType::FullDependency,
        ];

        for dag_type in dag_types {
            let dag_str = format!("{}", dag_type);
            assert!(!dag_str.is_empty());
        }
    }

    #[test]
    fn test_tdg_output_format_variants() {
        // Test TDG output format variants
        let formats = [
            TdgOutputFormat::Table,
            TdgOutputFormat::Json,
            TdgOutputFormat::Markdown,
            TdgOutputFormat::Sarif,
        ];

        for format in formats {
            let format_str = format!("{}", format);
            assert!(!format_str.is_empty());
        }
    }

    #[test]
    fn test_makefile_output_format_variants() {
        // Test Makefile output format variants
        let formats = [
            MakefileOutputFormat::Human,
            MakefileOutputFormat::Json,
            MakefileOutputFormat::Gcc,
            MakefileOutputFormat::Sarif,
        ];

        for format in formats {
            let format_str = format!("{}", format);
            assert!(!format_str.is_empty());
        }
    }

    #[test]
    fn test_defect_prediction_output_format_variants() {
        // Test defect prediction output format variants
        let formats = [
            DefectPredictionOutputFormat::Summary,
            DefectPredictionOutputFormat::Detailed,
            DefectPredictionOutputFormat::Json,
            DefectPredictionOutputFormat::Csv,
            DefectPredictionOutputFormat::Sarif,
        ];

        for format in formats {
            let format_str = format!("{}", format);
            assert!(!format_str.is_empty());
        }
    }

    #[test]
    fn test_comprehensive_output_format_variants() {
        // Test comprehensive output format variants
        let formats = [
            ComprehensiveOutputFormat::Summary,
            ComprehensiveOutputFormat::Detailed,
            ComprehensiveOutputFormat::Json,
            ComprehensiveOutputFormat::Markdown,
            ComprehensiveOutputFormat::Sarif,
        ];

        for format in formats {
            let format_str = format!("{}", format);
            assert!(!format_str.is_empty());
        }
    }

    #[test]
    fn test_graph_metric_type_variants() {
        // Test graph metric type variants
        let metrics = [
            GraphMetricType::Centrality,
            GraphMetricType::Betweenness,
            GraphMetricType::Closeness,
            GraphMetricType::PageRank,
            GraphMetricType::Clustering,
            GraphMetricType::Components,
            GraphMetricType::All,
        ];

        for metric in metrics {
            let metric_str = format!("{}", metric);
            assert!(!metric_str.is_empty());
        }
    }

    #[test]
    fn test_graph_metrics_output_format_variants() {
        // Test graph metrics output format variants
        let formats = [
            GraphMetricsOutputFormat::Summary,
            GraphMetricsOutputFormat::Detailed,
            GraphMetricsOutputFormat::Human,
            GraphMetricsOutputFormat::Json,
            GraphMetricsOutputFormat::Csv,
            GraphMetricsOutputFormat::GraphML,
            GraphMetricsOutputFormat::Markdown,
        ];

        for format in formats {
            let format_str = format!("{}", format);
            assert!(!format_str.is_empty());
        }
    }

    #[test]
    fn test_symbol_table_output_format_variants() {
        // Test symbol table output format variants
        let formats = [
            SymbolTableOutputFormat::Summary,
            SymbolTableOutputFormat::Detailed,
            SymbolTableOutputFormat::Human,
            SymbolTableOutputFormat::Json,
            SymbolTableOutputFormat::Csv,
        ];

        for format in formats {
            let format_str = format!("{}", format);
            assert!(!format_str.is_empty());
        }
    }

    #[test]
    fn test_symbol_type_filter_variants() {
        // Test symbol type filter variants
        let filters = [
            SymbolTypeFilter::Functions,
            SymbolTypeFilter::Classes,
            SymbolTypeFilter::Types,
            SymbolTypeFilter::Variables,
            SymbolTypeFilter::Modules,
            SymbolTypeFilter::All,
        ];

        for filter in filters {
            let filter_str = format!("{}", filter);
            assert!(!filter_str.is_empty());
        }
    }

    #[tokio::test]
    async fn test_handle_analyze_deep_context_with_empty_project() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create an empty directory
        let result = handle_analyze_deep_context(
            temp_dir.path().to_path_buf(),
            None,
            DeepContextOutputFormat::Json,
            false,
            vec![],
            vec![],
            30,
            None,
            None,
            vec![],
            vec![],
            None,
            false,
            false,
            10,
        )
        .await;

        // Should succeed even with no files
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_deep_context_json_format() {
        let temp_dir = create_test_project();

        let result = handle_analyze_deep_context(
            temp_dir.path().to_path_buf(),
            None,
            DeepContextOutputFormat::Json,
            false,
            vec![],
            vec![],
            30,
            None,
            None,
            vec![],
            vec![],
            None,
            false,
            false,
            10,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_deep_context_markdown_format() {
        let temp_dir = create_test_project();

        let result = handle_analyze_deep_context(
            temp_dir.path().to_path_buf(),
            None,
            DeepContextOutputFormat::Markdown,
            false,
            vec![],
            vec![],
            30,
            None,
            None,
            vec![],
            vec![],
            None,
            false,
            false,
            10,
        )
        .await;

        assert!(result.is_ok());
    }

    /// Issue #659 regression: `analyze deep-context --format sarif` used to
    /// emit pmat's internal JSON (top-level keys summary/files/
    /// recommendations), byte-identical to `--format json` — no `$schema`, no
    /// `version`, no `runs`. This test used to assert only `is_ok()`, which
    /// the defect satisfied.
    #[tokio::test]
    async fn test_handle_analyze_deep_context_sarif_format() {
        let temp_dir = create_test_project();
        let sarif_path = temp_dir.path().join("out.sarif");
        let json_path = temp_dir.path().join("out.json");

        handle_analyze_deep_context(
            temp_dir.path().to_path_buf(),
            Some(sarif_path.clone()),
            DeepContextOutputFormat::Sarif,
            false,
            vec![],
            vec![],
            30,
            None,
            None,
            vec![],
            vec![],
            None,
            false,
            false,
            10,
        )
        .await
        .expect("sarif deep-context should succeed");

        handle_analyze_deep_context(
            temp_dir.path().to_path_buf(),
            Some(json_path.clone()),
            DeepContextOutputFormat::Json,
            false,
            vec![],
            vec![],
            30,
            None,
            None,
            vec![],
            vec![],
            None,
            false,
            false,
            10,
        )
        .await
        .expect("json deep-context should succeed");

        let sarif_text = fs::read_to_string(&sarif_path).expect("sarif file");
        let json_text = fs::read_to_string(&json_path).expect("json file");
        assert_ne!(
            sarif_text, json_text,
            "--format sarif must not be byte-identical to --format json"
        );

        let sarif: serde_json::Value =
            serde_json::from_str(&sarif_text).expect("SARIF must be JSON");
        assert_eq!(sarif["version"], "2.1.0");
        assert!(sarif["$schema"].as_str().unwrap().contains("sarif"));
        assert!(sarif["runs"].as_array().is_some_and(|r| !r.is_empty()));
        assert!(sarif["runs"][0]["tool"]["driver"]["name"].is_string());
        assert!(sarif["runs"][0]["results"].is_array());
    }

    #[tokio::test]
    async fn test_handle_analyze_deep_context_full_mode() {
        let temp_dir = create_test_project();

        let result = handle_analyze_deep_context(
            temp_dir.path().to_path_buf(),
            None,
            DeepContextOutputFormat::Json,
            true, // full mode
            vec![],
            vec![],
            30,
            None,
            None,
            vec![],
            vec![],
            None,
            false,
            false,
            10,
        )
        .await;

        // This used to assert `is_ok()`, which is precisely the defect: `--full`
        // was accepted and discarded, so "full mode" produced a report
        // byte-identical to the default one. An unimplemented flag is refused.
        let err = result.expect_err("--full is not implemented");
        assert!(err.to_string().contains("--full"), "{err}");
    }

    #[tokio::test]
    async fn test_handle_analyze_deep_context_with_include_features() {
        let temp_dir = create_test_project();

        let result = handle_analyze_deep_context(
            temp_dir.path().to_path_buf(),
            None,
            DeepContextOutputFormat::Json,
            false,
            vec!["complexity".to_string(), "dependencies".to_string()],
            vec![],
            30,
            None,
            None,
            vec![],
            vec![],
            None,
            false,
            false,
            10,
        )
        .await;

        // `--include` selected no analyses — the report was the same with and
        // without it — so it is refused rather than silently dropped.
        let err = result.expect_err("--include is not implemented");
        assert!(err.to_string().contains("--include"), "{err}");
    }

    #[tokio::test]
    async fn test_handle_analyze_deep_context_with_include_patterns() {
        let temp_dir = create_test_project();

        let result = handle_analyze_deep_context(
            temp_dir.path().to_path_buf(),
            None,
            DeepContextOutputFormat::Json,
            false,
            vec![],
            vec![],
            30,
            None,
            None,
            vec!["**/*.rs".to_string()],
            vec![],
            None,
            false,
            false,
            10,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_deep_context_with_exclude_patterns() {
        let temp_dir = create_test_project();

        let result = handle_analyze_deep_context(
            temp_dir.path().to_path_buf(),
            None,
            DeepContextOutputFormat::Json,
            false,
            vec![],
            vec![],
            30,
            None,
            None,
            vec![],
            vec!["**/tests/**".to_string()],
            None,
            false,
            false,
            10,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_deep_context_verbose_mode() {
        let temp_dir = create_test_project();

        let result = handle_analyze_deep_context(
            temp_dir.path().to_path_buf(),
            None,
            DeepContextOutputFormat::Json,
            false,
            vec![],
            vec![],
            30,
            None,
            None,
            vec![],
            vec![],
            None,
            false,
            true, // verbose
            10,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_deep_context_with_output_file() {
        let temp_dir = create_test_project();
        let output_file = temp_dir.path().join("output.json");

        let result = handle_analyze_deep_context(
            temp_dir.path().to_path_buf(),
            Some(output_file.clone()),
            DeepContextOutputFormat::Json,
            false,
            vec![],
            vec![],
            30,
            None,
            None,
            vec![],
            vec![],
            None,
            false,
            false,
            10,
        )
        .await;

        assert!(result.is_ok());
        assert!(output_file.exists());
    }

    #[tokio::test]
    async fn test_handle_analyze_deep_context_with_dag_type() {
        let temp_dir = create_test_project();

        let result = handle_analyze_deep_context(
            temp_dir.path().to_path_buf(),
            None,
            DeepContextOutputFormat::Json,
            false,
            vec![],
            vec![],
            30,
            Some(DagType::CallGraph),
            None,
            vec![],
            vec![],
            None,
            false,
            false,
            10,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_deep_context_with_max_depth() {
        let temp_dir = create_test_project();

        let result = handle_analyze_deep_context(
            temp_dir.path().to_path_buf(),
            None,
            DeepContextOutputFormat::Json,
            false,
            vec![],
            vec![],
            30,
            None,
            Some(5),
            vec![],
            vec![],
            None,
            false,
            false,
            10,
        )
        .await;

        // `--max-depth` never reached the file walk, so depth 5 and no limit at
        // all produced the same report; it is refused until it is threaded in.
        let err = result.expect_err("--max-depth is not implemented");
        assert!(err.to_string().contains("--max-depth"), "{err}");
    }

    #[tokio::test]
    async fn test_handle_analyze_deep_context_with_period_days() {
        let temp_dir = create_test_project();

        let result = handle_analyze_deep_context(
            temp_dir.path().to_path_buf(),
            None,
            DeepContextOutputFormat::Json,
            false,
            vec![],
            vec![],
            90, // 90 day period
            None,
            None,
            vec![],
            vec![],
            None,
            false,
            false,
            10,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_deep_context_with_top_files() {
        let temp_dir = create_test_project();

        let result = handle_analyze_deep_context(
            temp_dir.path().to_path_buf(),
            None,
            DeepContextOutputFormat::Json,
            false,
            vec![],
            vec![],
            30,
            None,
            None,
            vec![],
            vec![],
            None,
            false,
            false,
            20, // top 20 files
        )
        .await;

        assert!(result.is_ok());
    }
}
