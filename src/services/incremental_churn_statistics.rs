// Churn statistics calculation (scalar and SIMD)
// Included from incremental_churn.rs - do NOT add `use` imports or `#!` attributes here.

/// Scalar implementation of churn statistics calculation
fn calculate_churn_statistics_scalar(files: &[FileChurnMetrics]) -> (f64, f64, f64) {
    debug_assert!(!files.is_empty(), "files must not be empty");
    if files.is_empty() {
        return (0.0, 0.0, 0.0);
    }

    let scores: Vec<f64> = files.iter().map(|f| f64::from(f.churn_score)).collect();
    let n = scores.len() as f64;

    // Calculate mean
    let sum: f64 = scores.iter().sum();
    let mean = sum / n;

    // Calculate variance (population variance)
    let variance = scores.iter().map(|&s| (s - mean).powi(2)).sum::<f64>() / n;

    // Calculate standard deviation
    let stddev = variance.sqrt();

    (mean, variance, stddev)
}

/// SIMD implementation of churn statistics using Trueno
#[cfg(feature = "simd")]
fn calculate_churn_statistics_simd(files: &[FileChurnMetrics]) -> (f64, f64, f64) {
    debug_assert!(!files.is_empty(), "files must not be empty");
    use trueno::Vector;

    if files.is_empty() {
        return (0.0, 0.0, 0.0);
    }

    // Convert churn scores to f32 for Trueno
    let scores_f32: Vec<f32> = files.iter().map(|f| f.churn_score).collect();
    let vec = Vector::from_slice(&scores_f32);

    // Use Trueno's mean and variance functions
    let mean = match vec.mean() {
        Ok(m) => f64::from(m),
        Err(_) => return calculate_churn_statistics_scalar(files),
    };

    let variance = match vec.variance() {
        Ok(v) => f64::from(v),
        Err(_) => return calculate_churn_statistics_scalar(files),
    };

    let stddev = variance.sqrt();

    (mean, variance, stddev)
}

/// Calculate churn statistics - dispatches to SIMD or scalar implementation
#[cfg(feature = "simd")]
fn calculate_churn_statistics(files: &[FileChurnMetrics]) -> (f64, f64, f64) {
    debug_assert!(!files.is_empty(), "files must not be empty");
    calculate_churn_statistics_simd(files)
}

#[cfg(not(feature = "simd"))]
fn calculate_churn_statistics(files: &[FileChurnMetrics]) -> (f64, f64, f64) {
    debug_assert!(!files.is_empty(), "files must not be empty");
    calculate_churn_statistics_scalar(files)
}
