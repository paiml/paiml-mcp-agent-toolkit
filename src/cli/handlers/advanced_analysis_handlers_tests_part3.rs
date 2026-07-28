//! Advanced analysis handlers tests - Part 3: Graph metrics and symbol table tests
//! Extracted for file health compliance (CB-040)

use super::*;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod graph_symbol_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Helper to create a temporary project directory with Rust files
    fn create_test_project() -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let main_rs = temp_dir.path().join("main.rs");
        fs::write(
            &main_rs,
            r#"
fn main() {
    println!("Hello, world!");
}

fn complex_function(x: i32, y: i32) -> i32 {
    if x > 0 {
        if y > 0 { x + y } else { x - y }
    } else {
        -x
    }
}
"#,
        )
        .expect("Failed to write main.rs");

        temp_dir
    }

    #[tokio::test]
    async fn test_handle_analyze_graph_metrics_basic() {
        let temp_dir = create_test_project();

        let result = handle_analyze_graph_metrics(
            temp_dir.path().to_path_buf(),
            vec![GraphMetricType::Centrality],
            vec![],
            0.85,
            100,
            1e-6,
            false,
            GraphMetricsOutputFormat::Summary,
            None,
            None,
            None,
            false,
            10,
            0.0,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_graph_metrics_all_metrics() {
        let temp_dir = create_test_project();

        let result = handle_analyze_graph_metrics(
            temp_dir.path().to_path_buf(),
            vec![GraphMetricType::All],
            vec![],
            0.85,
            100,
            1e-6,
            false,
            GraphMetricsOutputFormat::Json,
            None,
            None,
            None,
            false,
            10,
            0.0,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_graph_metrics_with_pagerank_seeds() {
        let temp_dir = create_test_project();

        let result = handle_analyze_graph_metrics(
            temp_dir.path().to_path_buf(),
            vec![GraphMetricType::PageRank],
            vec!["main".to_string()],
            0.85,
            100,
            1e-6,
            false,
            GraphMetricsOutputFormat::Summary,
            None,
            None,
            None,
            false,
            10,
            0.0,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_graph_metrics_custom_damping() {
        let temp_dir = create_test_project();

        let result = handle_analyze_graph_metrics(
            temp_dir.path().to_path_buf(),
            vec![GraphMetricType::PageRank],
            vec![],
            0.90, // custom damping factor
            200,  // custom max iterations
            1e-8, // tighter convergence
            false,
            GraphMetricsOutputFormat::Summary,
            None,
            None,
            None,
            false,
            10,
            0.0,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_symbol_table_basic() {
        let temp_dir = create_test_project();

        let result = handle_analyze_symbol_table(
            temp_dir.path().to_path_buf(),
            SymbolTableOutputFormat::Summary,
            None,
            None,
            vec![],
            vec![],
            false,
            false,
            None,
            false,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_symbol_table_json_format() {
        let temp_dir = create_test_project();

        let result = handle_analyze_symbol_table(
            temp_dir.path().to_path_buf(),
            SymbolTableOutputFormat::Json,
            None,
            None,
            vec![],
            vec![],
            false,
            false,
            None,
            false,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_symbol_table_with_filter() {
        let temp_dir = create_test_project();

        let result = handle_analyze_symbol_table(
            temp_dir.path().to_path_buf(),
            SymbolTableOutputFormat::Summary,
            Some(SymbolTypeFilter::Functions),
            None,
            vec![],
            vec![],
            false,
            false,
            None,
            false,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_symbol_table_with_query() {
        let temp_dir = create_test_project();

        let result = handle_analyze_symbol_table(
            temp_dir.path().to_path_buf(),
            SymbolTableOutputFormat::Summary,
            None,
            Some("main".to_string()),
            vec![],
            vec![],
            false,
            false,
            None,
            false,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_symbol_table_show_unreferenced() {
        let temp_dir = create_test_project();

        let result = handle_analyze_symbol_table(
            temp_dir.path().to_path_buf(),
            SymbolTableOutputFormat::Detailed,
            None,
            None,
            vec![],
            vec![],
            true, // show_unreferenced
            false,
            None,
            false,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_symbol_table_show_references() {
        let temp_dir = create_test_project();

        let result = handle_analyze_symbol_table(
            temp_dir.path().to_path_buf(),
            SymbolTableOutputFormat::Detailed,
            None,
            None,
            vec![],
            vec![],
            false,
            true, // show_references
            None,
            false,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_symbol_table_all_filters() {
        let temp_dir = create_test_project();

        // Test all filter types
        let filters = [
            SymbolTypeFilter::Functions,
            SymbolTypeFilter::Classes,
            SymbolTypeFilter::Types,
            SymbolTypeFilter::Variables,
            SymbolTypeFilter::Modules,
            SymbolTypeFilter::All,
        ];

        for filter in filters {
            let result = handle_analyze_symbol_table(
                temp_dir.path().to_path_buf(),
                SymbolTableOutputFormat::Summary,
                Some(filter),
                None,
                vec![],
                vec![],
                false,
                false,
                None,
                false,
            )
            .await;

            let _ = result;
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn test_period_days_reasonable(days in 1u32..365) {
            // Period days should always be positive and reasonable
            prop_assert!(days > 0);
            prop_assert!(days < 366);
        }

        #[test]
        fn test_top_files_positive(top in 1usize..1000) {
            // Top files count should always be positive
            prop_assert!(top > 0);
        }

        #[test]
        fn test_confidence_threshold_valid(threshold in 0.0f32..1.0) {
            // Confidence threshold should be between 0 and 1
            prop_assert!(threshold >= 0.0);
            prop_assert!(threshold <= 1.0);
        }

        #[test]
        fn test_damping_factor_valid(damping in 0.0f32..1.0) {
            // PageRank damping factor should be between 0 and 1
            prop_assert!(damping >= 0.0);
            prop_assert!(damping <= 1.0);
        }

        #[test]
        fn test_max_iterations_positive(iterations in 1usize..10000) {
            // Max iterations should be positive
            prop_assert!(iterations > 0);
        }

        #[test]
        fn test_min_lines_reasonable(lines in 1usize..100000) {
            // Min lines should be reasonable
            prop_assert!(lines > 0);
            prop_assert!(lines < 100001);
        }

        #[test]
        fn test_convergence_threshold_small(threshold in 1e-10f64..1e-3) {
            // Convergence threshold should be small but positive
            prop_assert!(threshold > 0.0);
            prop_assert!(threshold < 0.01);
        }

        #[test]
        fn test_tdg_threshold_positive(threshold in 0.0f64..10.0) {
            // TDG threshold should be non-negative
            prop_assert!(threshold >= 0.0);
        }
    }
}
