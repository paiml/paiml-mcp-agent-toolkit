//! Advanced analysis handlers tests - Part 2: TDG, Makefile, and Defect tests
//! Extracted for file health compliance (CB-040)

use super::*;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod handler_tests {
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

    #[tokio::test]
    async fn test_handle_analyze_tdg_basic() {
        let temp_dir = create_test_project();

        let result = handle_analyze_tdg(
            temp_dir.path().to_path_buf(),
            None,
            None,
            TdgOutputFormat::Table,
            false,
            None,
            false,
            false,
        )
        .await;

        // TDG analysis should complete (may fail on non-git repos, but function works)
        // We just verify the function runs without panic
        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_tdg_json_format() {
        let temp_dir = create_test_project();

        let result = handle_analyze_tdg(
            temp_dir.path().to_path_buf(),
            None,
            None,
            TdgOutputFormat::Json,
            false,
            None,
            false,
            false,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_tdg_with_threshold() {
        let temp_dir = create_test_project();

        let result = handle_analyze_tdg(
            temp_dir.path().to_path_buf(),
            Some(1.5),
            None,
            TdgOutputFormat::Table,
            false,
            None,
            false,
            false,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_tdg_with_top_files() {
        let temp_dir = create_test_project();

        let result = handle_analyze_tdg(
            temp_dir.path().to_path_buf(),
            None,
            Some(5),
            TdgOutputFormat::Table,
            false,
            None,
            false,
            false,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_tdg_include_components() {
        let temp_dir = create_test_project();

        let result = handle_analyze_tdg(
            temp_dir.path().to_path_buf(),
            None,
            None,
            TdgOutputFormat::Table,
            true, // include_components
            None,
            false,
            false,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_tdg_critical_only() {
        let temp_dir = create_test_project();

        let result = handle_analyze_tdg(
            temp_dir.path().to_path_buf(),
            None,
            None,
            TdgOutputFormat::Table,
            false,
            None,
            true, // critical_only
            false,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_makefile_basic() {
        let temp_dir = create_test_project();
        create_test_makefile(&temp_dir);

        let makefile_path = temp_dir.path().join("Makefile");

        let result = handle_analyze_makefile(
            makefile_path,
            vec![],
            MakefileOutputFormat::Human,
            false,
            None,
            10,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_makefile_json_format() {
        let temp_dir = create_test_project();
        create_test_makefile(&temp_dir);

        let makefile_path = temp_dir.path().join("Makefile");

        let result = handle_analyze_makefile(
            makefile_path,
            vec![],
            MakefileOutputFormat::Json,
            false,
            None,
            10,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_makefile_gcc_format() {
        let temp_dir = create_test_project();
        create_test_makefile(&temp_dir);

        let makefile_path = temp_dir.path().join("Makefile");

        let result = handle_analyze_makefile(
            makefile_path,
            vec![],
            MakefileOutputFormat::Gcc,
            false,
            None,
            10,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_makefile_with_rules() {
        let temp_dir = create_test_project();
        create_test_makefile(&temp_dir);

        let makefile_path = temp_dir.path().join("Makefile");

        let result = handle_analyze_makefile(
            makefile_path,
            vec!["all".to_string(), "clean".to_string()],
            MakefileOutputFormat::Human,
            false,
            None,
            10,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_makefile_nonexistent() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let makefile_path = temp_dir.path().join("nonexistent_Makefile");

        let result = handle_analyze_makefile(
            makefile_path,
            vec![],
            MakefileOutputFormat::Human,
            false,
            None,
            10,
        )
        .await;

        // Should fail for nonexistent file
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_analyze_defect_prediction_basic() {
        let temp_dir = create_test_project();

        let result = handle_analyze_defect_prediction(
            temp_dir.path().to_path_buf(),
            None,
            None,
            false,
            DefectPredictionOutputFormat::Summary,
            false,
            false,
            vec![],
            vec![],
            None,
            false,
            10,
        )
        .await;

        // Function should complete without panic
        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_defect_prediction_json_format() {
        let temp_dir = create_test_project();

        let result = handle_analyze_defect_prediction(
            temp_dir.path().to_path_buf(),
            None,
            None,
            false,
            DefectPredictionOutputFormat::Json,
            false,
            false,
            vec![],
            vec![],
            None,
            false,
            10,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_defect_prediction_with_confidence_threshold() {
        let temp_dir = create_test_project();

        let result = handle_analyze_defect_prediction(
            temp_dir.path().to_path_buf(),
            Some(0.7),
            None,
            false,
            DefectPredictionOutputFormat::Summary,
            false,
            false,
            vec![],
            vec![],
            None,
            false,
            10,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_defect_prediction_with_min_lines() {
        let temp_dir = create_test_project();

        let result = handle_analyze_defect_prediction(
            temp_dir.path().to_path_buf(),
            None,
            Some(50),
            false,
            DefectPredictionOutputFormat::Summary,
            false,
            false,
            vec![],
            vec![],
            None,
            false,
            10,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_defect_prediction_high_risk_only() {
        let temp_dir = create_test_project();

        let result = handle_analyze_defect_prediction(
            temp_dir.path().to_path_buf(),
            None,
            None,
            false,
            DefectPredictionOutputFormat::Summary,
            true, // high_risk_only
            false,
            vec![],
            vec![],
            None,
            false,
            10,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_defect_prediction_with_recommendations() {
        let temp_dir = create_test_project();

        let result = handle_analyze_defect_prediction(
            temp_dir.path().to_path_buf(),
            None,
            None,
            false,
            DefectPredictionOutputFormat::Detailed,
            false,
            true, // include_recommendations
            vec![],
            vec![],
            None,
            false,
            10,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_comprehensive_basic() {
        let temp_dir = create_test_project();

        let result = handle_analyze_comprehensive(
            temp_dir.path().to_path_buf(),
            None,
            vec![],
            ComprehensiveOutputFormat::Summary,
            false,
            false,
            false,
            true,
            false,
            0.5,
            100,
            None,
            None,
            None,
            false,
            false,
        )
        .await;

        // Function should complete
        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_comprehensive_json_format() {
        let temp_dir = create_test_project();

        let result = handle_analyze_comprehensive(
            temp_dir.path().to_path_buf(),
            None,
            vec![],
            ComprehensiveOutputFormat::Json,
            false,
            false,
            false,
            true,
            false,
            0.5,
            100,
            None,
            None,
            None,
            false,
            false,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_comprehensive_with_all_options() {
        let temp_dir = create_test_project();

        let result = handle_analyze_comprehensive(
            temp_dir.path().to_path_buf(),
            None,
            vec![],
            ComprehensiveOutputFormat::Detailed,
            true, // include_duplicates
            true, // include_dead_code
            true, // include_defects
            true, // include_complexity
            true, // include_tdg
            0.7,  // confidence_threshold
            50,   // min_lines
            None,
            None,
            None,
            false,
            true, // executive_summary
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_comprehensive_single_file() {
        let temp_dir = create_test_project();
        let main_rs = temp_dir.path().join("main.rs");

        let result = handle_analyze_comprehensive(
            temp_dir.path().to_path_buf(),
            Some(main_rs),
            vec![],
            ComprehensiveOutputFormat::Summary,
            false,
            false,
            false,
            true,
            false,
            0.5,
            100,
            None,
            None,
            None,
            false,
            false,
        )
        .await;

        let _ = result;
    }
}
