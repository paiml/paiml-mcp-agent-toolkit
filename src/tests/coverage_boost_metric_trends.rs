#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for services/metric_trends.rs
//! Tests: MetricObservation, TrendAnalysis, TrendDirection, ForecastPoint,
//! PredictionResult, MetricTrendStore

use crate::services::metric_trends::{
    ForecastPoint, MetricObservation, MetricTrendStore, PredictionResult, TrendAnalysis,
    TrendDirection,
};

// ============ TrendDirection Tests ============

#[test]
fn test_trend_direction_serde() {
    let dirs = vec![
        TrendDirection::Improving,
        TrendDirection::Stable,
        TrendDirection::Regressing,
    ];
    for d in &dirs {
        let json = serde_json::to_string(d).unwrap();
        let back: TrendDirection = serde_json::from_str(&json).unwrap();
        assert_eq!(*d, back);
    }
}

#[test]
fn test_trend_direction_eq() {
    assert_eq!(TrendDirection::Improving, TrendDirection::Improving);
    assert_ne!(TrendDirection::Improving, TrendDirection::Regressing);
}

#[test]
fn test_trend_direction_copy() {
    let d = TrendDirection::Stable;
    let copied = d;
    assert_eq!(d, copied);
}

// ============ MetricObservation Tests ============

#[test]
fn test_metric_observation_serde() {
    let obs = MetricObservation {
        metric: "lint".to_string(),
        value: 24824.0,
        timestamp: 1700000000,
    };
    let json = serde_json::to_string(&obs).unwrap();
    let back: MetricObservation = serde_json::from_str(&json).unwrap();
    assert_eq!(back.metric, "lint");
    assert!((back.value - 24824.0).abs() < f64::EPSILON);
    assert_eq!(back.timestamp, 1700000000);
}

#[test]
fn test_metric_observation_clone() {
    let obs = MetricObservation {
        metric: "test-fast".to_string(),
        value: 150000.0,
        timestamp: 1700001000,
    };
    let cloned = obs.clone();
    assert_eq!(cloned.metric, obs.metric);
    assert_eq!(cloned.timestamp, obs.timestamp);
}

// ============ TrendAnalysis Tests ============

#[test]
fn test_trend_analysis_serde() {
    let analysis = TrendAnalysis {
        metric: "lint".to_string(),
        count: 30,
        mean: 25000.0,
        std_dev: 2000.0,
        min: 20000.0,
        max: 30000.0,
        direction: TrendDirection::Stable,
        slope: 0.5,
        p_value: 0.12,
    };
    let json = serde_json::to_string(&analysis).unwrap();
    let back: TrendAnalysis = serde_json::from_str(&json).unwrap();
    assert_eq!(back.metric, "lint");
    assert_eq!(back.count, 30);
    assert_eq!(back.direction, TrendDirection::Stable);
}

#[test]
fn test_trend_analysis_clone() {
    let analysis = TrendAnalysis {
        metric: "coverage".to_string(),
        count: 10,
        mean: 85.0,
        std_dev: 2.5,
        min: 80.0,
        max: 90.0,
        direction: TrendDirection::Improving,
        slope: -0.3,
        p_value: 0.01,
    };
    let cloned = analysis.clone();
    assert_eq!(cloned.direction, TrendDirection::Improving);
    assert!((cloned.slope - (-0.3)).abs() < f64::EPSILON);
}

// ============ ForecastPoint Tests ============

#[test]
fn test_forecast_point_serde() {
    let point = ForecastPoint {
        days_ahead: 7,
        predicted_value: 26000.0,
        lower_bound: 24000.0,
        upper_bound: 28000.0,
    };
    let json = serde_json::to_string(&point).unwrap();
    let back: ForecastPoint = serde_json::from_str(&json).unwrap();
    assert_eq!(back.days_ahead, 7);
    assert!((back.predicted_value - 26000.0).abs() < f64::EPSILON);
}

#[test]
fn test_forecast_point_clone() {
    let point = ForecastPoint {
        days_ahead: 14,
        predicted_value: 50000.0,
        lower_bound: 40000.0,
        upper_bound: 60000.0,
    };
    let cloned = point.clone();
    assert_eq!(cloned.days_ahead, point.days_ahead);
}

// ============ PredictionResult Tests ============

#[test]
fn test_prediction_result_serde() {
    let result = PredictionResult {
        metric: "lint".to_string(),
        current_value: 25000.0,
        threshold: 30000.0,
        breach_in_days: Some(14),
        predicted_value: Some(31000.0),
        confidence: 0.85,
        recommendations: vec!["Optimize lint step".to_string()],
        forecast: vec![ForecastPoint {
            days_ahead: 7,
            predicted_value: 28000.0,
            lower_bound: 26000.0,
            upper_bound: 30000.0,
        }],
    };
    let json = serde_json::to_string(&result).unwrap();
    let back: PredictionResult = serde_json::from_str(&json).unwrap();
    assert_eq!(back.metric, "lint");
    assert_eq!(back.breach_in_days, Some(14));
    assert_eq!(back.recommendations.len(), 1);
    assert_eq!(back.forecast.len(), 1);
}

#[test]
fn test_prediction_result_no_breach() {
    let result = PredictionResult {
        metric: "test-fast".to_string(),
        current_value: 100000.0,
        threshold: 300000.0,
        breach_in_days: None,
        predicted_value: None,
        confidence: 0.9,
        recommendations: vec![],
        forecast: vec![],
    };
    assert!(result.breach_in_days.is_none());
    assert!(result.predicted_value.is_none());
    assert!(result.recommendations.is_empty());
}

// ============ MetricTrendStore Tests ============

#[test]
fn test_metric_trend_store_from_path() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let store = MetricTrendStore::from_path(temp_dir.path().join("trends"));
    assert!(store.is_ok());
}

#[test]
fn test_metric_trend_store_record_and_trend() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let mut store = MetricTrendStore::from_path(temp_dir.path().join("trends")).unwrap();

    let now = chrono::Utc::now().timestamp();

    // Record multiple observations
    for i in 0..5 {
        store
            .record("lint", 25000.0 + (i as f64 * 100.0), now - (4 - i) * 86400)
            .unwrap();
    }

    // Get trend for last 30 days
    let trend = store.trend("lint", 30).unwrap();
    assert_eq!(trend.metric, "lint");
    assert_eq!(trend.count, 5);
    assert!(trend.mean > 0.0);
    assert!(trend.min <= trend.max);
}

#[test]
fn test_metric_trend_store_trend_no_data() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let mut store = MetricTrendStore::from_path(temp_dir.path().join("trends")).unwrap();

    let result = store.trend("nonexistent", 30);
    assert!(result.is_err());
}

#[test]
fn test_metric_trend_store_record_duplicate_timestamp() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let mut store = MetricTrendStore::from_path(temp_dir.path().join("trends")).unwrap();

    let now = chrono::Utc::now().timestamp();

    // Record same timestamp twice
    store.record("lint", 25000.0, now).unwrap();
    store.record("lint", 26000.0, now).unwrap();

    // Should not panic
    let trend = store.trend("lint", 30).unwrap();
    assert!(trend.count >= 1);
}

#[test]
fn test_metric_trend_store_multiple_metrics() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let mut store = MetricTrendStore::from_path(temp_dir.path().join("trends")).unwrap();

    let now = chrono::Utc::now().timestamp();

    store.record("lint", 25000.0, now).unwrap();
    store.record("test-fast", 150000.0, now).unwrap();
    store.record("coverage", 85.0, now).unwrap();

    let lint_trend = store.trend("lint", 30).unwrap();
    assert_eq!(lint_trend.metric, "lint");

    let test_trend = store.trend("test-fast", 30).unwrap();
    assert_eq!(test_trend.metric, "test-fast");

    let cov_trend = store.trend("coverage", 30).unwrap();
    assert_eq!(cov_trend.metric, "coverage");
}
