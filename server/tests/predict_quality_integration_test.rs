//! Integration tests for predict-quality command (Phase 4.1 O(1) Quality Gates)
//!
//! Tests the ML-based prediction engine for quality metrics threshold breaches.

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test basic threshold breach prediction
#[test]
#[ignore] // Slow test - requires pmat binary (>200s in coverage)
fn test_predict_quality_basic_prediction() {
    let temp_dir = TempDir::new().unwrap();
    create_test_metric_data(&temp_dir, "lint", 8);

    let binary_path = get_pmat_binary_path();

    let output = std::process::Command::new(&binary_path)
        .env("PMAT_METRICS_DIR", temp_dir.path())
        .args([
            "predict-quality",
            "--metric",
            "lint",
            "--threshold",
            "30000",
        ])
        .output()
        .expect("Failed to run predict-quality");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "Command should succeed. stderr:\n{}\nstdout:\n{}",
        stderr,
        stdout
    );

    // Should show prediction results
    assert!(
        stdout.contains("Quality Metrics Predictions") || stdout.contains("lint"),
        "Output should contain prediction results"
    );

    // Should show threshold breach prediction
    assert!(
        stdout.contains("Breach") || stdout.contains("days"),
        "Output should show breach prediction"
    );
}

/// Test JSON output format
#[test]
#[ignore] // Slow test - requires pmat binary (>200s in coverage)
fn test_predict_quality_json_output() {
    let temp_dir = TempDir::new().unwrap();
    create_test_metric_data(&temp_dir, "lint", 8);

    let binary_path = get_pmat_binary_path();

    let output = std::process::Command::new(&binary_path)
        .env("PMAT_METRICS_DIR", temp_dir.path())
        .args([
            "predict-quality",
            "--metric",
            "lint",
            "--threshold",
            "30000",
            "--format",
            "json",
        ])
        .output()
        .expect("Failed to run predict-quality with JSON output");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "Command should succeed. stderr:\n{}\nstdout:\n{}",
        stderr,
        stdout
    );

    // Parse JSON output
    let predictions: serde_json::Value =
        serde_json::from_str(&stdout).expect("Output should be valid JSON");

    assert!(predictions.is_array(), "JSON output should be an array");
    let first_prediction = &predictions[0];

    // Verify required fields
    assert!(
        first_prediction.get("metric").is_some(),
        "JSON should have 'metric' field"
    );
    assert!(
        first_prediction.get("current_value").is_some(),
        "JSON should have 'current_value' field"
    );
    assert!(
        first_prediction.get("threshold").is_some(),
        "JSON should have 'threshold' field"
    );
    assert!(
        first_prediction.get("confidence").is_some(),
        "JSON should have 'confidence' field"
    );
    assert!(
        first_prediction.get("recommendations").is_some(),
        "JSON should have 'recommendations' field"
    );
    assert!(
        first_prediction.get("forecast").is_some(),
        "JSON should have 'forecast' field"
    );

    // Verify forecast structure
    let forecast = first_prediction["forecast"]
        .as_array()
        .expect("Forecast should be an array");
    assert!(forecast.len() > 0, "Forecast should have data points");

    let first_point = &forecast[0];
    assert!(
        first_point.get("days_ahead").is_some(),
        "Forecast point should have 'days_ahead'"
    );
    assert!(
        first_point.get("predicted_value").is_some(),
        "Forecast point should have 'predicted_value'"
    );
    assert!(
        first_point.get("lower_bound").is_some(),
        "Forecast point should have 'lower_bound'"
    );
    assert!(
        first_point.get("upper_bound").is_some(),
        "Forecast point should have 'upper_bound'"
    );
}

/// Test --all flag for multiple metrics
#[test]
#[ignore] // Slow test - requires pmat binary (>180s in coverage)
fn test_predict_quality_all_metrics() {
    let temp_dir = TempDir::new().unwrap();
    create_test_metric_data(&temp_dir, "lint", 8);
    create_test_metric_data(&temp_dir, "test-fast", 8);

    let binary_path = get_pmat_binary_path();

    let output = std::process::Command::new(&binary_path)
        .env("PMAT_METRICS_DIR", temp_dir.path())
        .args(["predict-quality", "--all"])
        .output()
        .expect("Failed to run predict-quality --all");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "Command should succeed. stderr:\n{}\nstdout:\n{}",
        stderr,
        stdout
    );

    // Should show multiple metrics
    assert!(
        stdout.contains("lint") || stdout.contains("test-fast"),
        "Output should contain multiple metrics"
    );
}

/// Test insufficient data handling
#[test]
#[ignore] // Slow test - requires pmat binary (>180s in coverage)
fn test_predict_quality_insufficient_data() {
    let temp_dir = TempDir::new().unwrap();
    create_test_metric_data(&temp_dir, "lint", 5); // Only 5 observations (need 7+)

    let binary_path = get_pmat_binary_path();

    let output = std::process::Command::new(&binary_path)
        .env("PMAT_METRICS_DIR", temp_dir.path())
        .args([
            "predict-quality",
            "--metric",
            "lint",
            "--threshold",
            "30000",
        ])
        .output()
        .expect("Failed to run predict-quality");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should warn about insufficient data
    assert!(
        stderr.contains("at least 7 observations") || stderr.contains("Failed to predict"),
        "Should warn about insufficient observations. stderr:\n{}\nstdout:\n{}",
        stderr,
        stdout
    );
}

/// Test high confidence prediction (R² > 0.85)
#[test]
#[ignore] // Slow test - requires pmat binary (>180s in coverage)
fn test_predict_quality_high_confidence() {
    let temp_dir = TempDir::new().unwrap();
    create_test_metric_data(&temp_dir, "lint", 10); // More data = higher confidence

    let binary_path = get_pmat_binary_path();

    let output = std::process::Command::new(&binary_path)
        .env("PMAT_METRICS_DIR", temp_dir.path())
        .args([
            "predict-quality",
            "--metric",
            "lint",
            "--threshold",
            "30000",
            "--format",
            "json",
        ])
        .output()
        .expect("Failed to run predict-quality");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "Command should succeed. stderr:\n{}\nstdout:\n{}",
        stderr,
        stdout
    );

    let predictions: serde_json::Value =
        serde_json::from_str(&stdout).expect("Output should be valid JSON");
    let confidence = predictions[0]["confidence"]
        .as_f64()
        .expect("Confidence should be a number");

    assert!(
        confidence >= 0.85,
        "Confidence should be >= 0.85 (85%) for reliable predictions, got {}",
        confidence
    );
}

/// Test recommendations generation
#[test]
#[ignore] // Slow test - requires pmat binary (>180s in coverage)
fn test_predict_quality_recommendations() {
    let temp_dir = TempDir::new().unwrap();
    create_test_metric_data(&temp_dir, "lint", 8);

    let binary_path = get_pmat_binary_path();

    let output = std::process::Command::new(&binary_path)
        .env("PMAT_METRICS_DIR", temp_dir.path())
        .args([
            "predict-quality",
            "--metric",
            "lint",
            "--threshold",
            "30000",
            "--format",
            "json",
        ])
        .output()
        .expect("Failed to run predict-quality");

    let stdout = String::from_utf8_lossy(&output.stdout);

    let predictions: serde_json::Value =
        serde_json::from_str(&stdout).expect("Output should be valid JSON");
    let recommendations = predictions[0]["recommendations"]
        .as_array()
        .expect("Recommendations should be an array");

    assert!(
        !recommendations.is_empty(),
        "Should generate recommendations for predicted breach"
    );

    // Verify recommendations contain actionable advice
    let recommendations_text = recommendations
        .iter()
        .map(|r| r.as_str().unwrap())
        .collect::<Vec<_>>()
        .join(" ");

    assert!(
        recommendations_text.contains("dependencies")
            || recommendations_text.contains("optimization"),
        "Recommendations should contain actionable advice"
    );
}

// Helper functions

/// Creates test metric data with a regressing trend (increasing values)
fn create_test_metric_data(temp_dir: &TempDir, metric_name: &str, num_observations: usize) {
    let trends_dir = temp_dir.path().join("trends");
    fs::create_dir_all(&trends_dir).expect("Failed to create trends directory");

    let mut observations = Vec::new();
    let base_timestamp = 1761740295i64; // Fixed base timestamp
    let base_value = 20000.0; // 20 seconds

    for i in 0..num_observations {
        let timestamp = base_timestamp + (i as i64 * 5 * 86400); // 5 days apart
        let value = base_value + (i as f64 * 1000.0); // Increase by 1s per observation

        observations.push(serde_json::json!({
            "metric": metric_name,
            "value": value,
            "timestamp": timestamp
        }));
    }

    let metric_file = trends_dir.join(format!("{}.json", metric_name));
    fs::write(
        &metric_file,
        serde_json::to_string_pretty(&observations).unwrap(),
    )
    .expect("Failed to write metric data");
}

/// Gets the path to the pmat binary, building it if necessary
fn get_pmat_binary_path() -> PathBuf {
    // Get the workspace root (parent of server directory)
    let workspace_root = std::env::current_dir()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();

    let binary_path = workspace_root.join("target").join("debug").join("pmat");

    if !binary_path.exists() {
        // Build the binary if it doesn't exist from workspace root
        std::process::Command::new("cargo")
            .current_dir(&workspace_root)
            .args(["build", "--bin", "pmat", "-p", "pmat"])
            .output()
            .expect("Failed to build pmat binary");
    }

    assert!(
        binary_path.exists(),
        "pmat binary should exist at {:?}",
        binary_path
    );

    binary_path
}
