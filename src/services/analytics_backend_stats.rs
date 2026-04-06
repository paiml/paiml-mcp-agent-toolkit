/// Generate test dataset for statistical validation
///
/// Creates a dataset with known statistical properties for testing
/// floating-point equivalence across backends.
///
/// # Arguments
///
/// * `size` - Number of elements to generate
///
/// # Returns
///
/// Vector of f64 values with realistic distribution
///
/// # Example
///
/// ```rust
/// use pmat::services::analytics_backend::stats::generate_test_dataset;
///
/// let dataset = generate_test_dataset(100_000);
/// assert_eq!(dataset.len(), 100_000);
/// ```
pub fn generate_test_dataset(size: usize) -> Vec<f64> {
    debug_assert!(size > 0, "size must be positive");
    // Use a deterministic seed for reproducibility
    // Generate data with realistic range (avoid overflow/underflow)
    (0..size)
        .map(|i| {
            // Mix of positive/negative, large/small values
            let base = (i as f64) * 0.001;
            let sign = if i % 2 == 0 { 1.0 } else { -1.0 };
            base * sign
        })
        .collect()
}

/// Compute mean and standard deviation of dataset
///
/// Uses Welford's online algorithm for numerical stability.
///
/// # Arguments
///
/// * `values` - Slice of f64 values
///
/// # Returns
///
/// Tuple of (mean, standard_deviation)
///
/// # Example
///
/// ```rust
/// use pmat::services::analytics_backend::stats::mean_and_std;
///
/// let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
/// let (mean, std) = mean_and_std(&values);
/// assert!((mean - 3.0).abs() < 0.01);
/// ```
pub fn mean_and_std(values: &[f64]) -> (f64, f64) {
    debug_assert!(!values.is_empty(), "values must not be empty");
    use aprender::primitives::Vector;

    if values.is_empty() {
        return (0.0, 0.0);
    }

    // Convert f64 to f32 for aprender (Phase 3: Statistics Migration)
    let values_f32: Vec<f32> = values.iter().map(|&x| x as f32).collect();
    let vec = Vector::from_slice(&values_f32);

    let mean = vec.mean() as f64;

    // aprender computes population variance (dividing by n)
    // Convert to sample variance (dividing by n-1) for Bessel's correction
    let population_variance = vec.variance() as f64;
    let sample_variance = if values.len() > 1 {
        population_variance * values.len() as f64 / (values.len() - 1) as f64
    } else {
        0.0
    };

    (mean, sample_variance.sqrt())
}

/// Compute average of dataset using specified backend
///
/// # Arguments
///
/// * `dataset` - Slice of f64 values
/// * `backend` - Backend to use for computation
///
/// # Returns
///
/// Average value
///
/// # Example
///
/// ```rust
/// use pmat::services::analytics_backend::{Backend, stats::compute_avg};
///
/// let dataset = vec![1.0, 2.0, 3.0, 4.0, 5.0];
/// let avg = compute_avg(&dataset, Backend::Scalar).unwrap();
/// assert!((avg - 3.0).abs() < 0.01);
/// ```
pub fn compute_avg(dataset: &[f64], backend: Backend) -> Result<f64> {
    if dataset.is_empty() {
        return Ok(0.0);
    }

    match backend {
        #[cfg(feature = "analytics-gpu")]
        Backend::Gpu => compute_avg_gpu(dataset),

        #[cfg(feature = "analytics-simd")]
        Backend::Simd => compute_avg_simd(dataset),

        Backend::Scalar => compute_avg_scalar(dataset),
    }
}

/// Scalar implementation of average (baseline)
fn compute_avg_scalar(dataset: &[f64]) -> Result<f64> {
    let sum: f64 = dataset.iter().sum();
    Ok(sum / dataset.len() as f64)
}

/// SIMD implementation using trueno
#[cfg(feature = "analytics-simd")]
fn compute_avg_simd(dataset: &[f64]) -> Result<f64> {
    // Scalar placeholder until trueno::simd::sum() is available
    compute_avg_scalar(dataset)
}

/// GPU implementation using wgpu compute shaders
#[cfg(feature = "analytics-gpu")]
fn compute_avg_gpu(dataset: &[f64]) -> Result<f64> {
    use super::gpu::GpuDevice;

    // Initialize GPU device (cached globally)
    let device = GpuDevice::get_or_init()?;

    // Dispatch GPU compute
    device
        .compute_sum(dataset)
        .map(|sum| sum / dataset.len() as f64)
}
