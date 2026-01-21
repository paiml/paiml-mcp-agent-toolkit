//! Analytics Backend Abstraction (Issue #79, P0-3)
//!
//! Provides unified backend selection for GPU/SIMD/Scalar compute operations.
//! Implements graceful degradation: GPU → SIMD → Scalar.
//!
//! # Backend Selection
//!
//! ```rust
//! use pmat::services::analytics_backend::{Backend, BackendSelector};
//!
//! // Automatic backend selection (graceful degradation)
//! let backend = BackendSelector::auto_select();
//!
//! // Manual backend selection
//! let backend = Backend::Simd;
//! ```
//!
//! # Statistical Equivalence Testing
//!
//! The primary use case is validating GPU floating-point results against SIMD
//! to ensure CI stability despite GPU non-associativity.
//!
//! Reference: Higham (1993) - "The Accuracy of Floating Point Summation" (SIAM)

/// Compute backend selection for analytics operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    /// GPU-accelerated compute using wgpu (requires analytics-gpu feature)
    #[cfg(feature = "analytics-gpu")]
    Gpu,

    /// SIMD-accelerated compute using trueno (requires analytics-simd feature)
    #[cfg(feature = "analytics-simd")]
    Simd,

    /// Scalar fallback (always available)
    Scalar,
}

/// Backend selector with graceful degradation
pub struct BackendSelector;

impl BackendSelector {
    /// Automatically select best available backend
    ///
    /// Preference order: GPU > SIMD > Scalar
    ///
    /// # Example
    ///
    /// ```rust
    /// use pmat::services::analytics_backend::BackendSelector;
    ///
    /// let backend = BackendSelector::auto_select();
    /// println!("Selected backend: {:?}", backend);
    /// ```
    pub fn auto_select() -> Backend {
        #[cfg(feature = "analytics-gpu")]
        {
            // TODO: Check if GPU is actually available
            // For now, fall through to SIMD
        }

        #[cfg(feature = "analytics-simd")]
        {
            return Backend::Simd;
        }

        #[allow(unreachable_code)]
        Backend::Scalar
    }

    /// Check if GPU backend is available
    #[cfg(feature = "analytics-gpu")]
    pub fn is_gpu_available() -> bool {
        // TODO: Implement GPU device detection
        false
    }

    /// Check if SIMD backend is available
    #[cfg(feature = "analytics-simd")]
    pub fn is_simd_available() -> bool {
        true // Always available when feature is enabled
    }
}

/// Statistical helper functions for equivalence testing
pub mod stats {
    use super::Backend;
    use anyhow::Result;

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
        // Use trueno for SIMD-accelerated sum
        // For now, use scalar as placeholder
        // TODO: Integrate trueno::simd::sum() when available
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
}

/// GPU compute backend (Issue #79, P0-3 and P0-5)
///
/// Implements GPU-accelerated compute operations using wgpu.
/// Provides PCIe bandwidth calibration for cost-based query optimization.
#[cfg(feature = "analytics-gpu")]
pub mod gpu {
    use anyhow::{bail, Context, Result};
    use std::sync::Once;
    use wgpu::util::DeviceExt;

    /// Global GPU device instance (initialized on first use)
    static mut GPU_DEVICE: Option<GpuDevice> = None;
    static INIT: Once = Once::new();

    /// GPU compute device for analytics operations
    ///
    /// Manages wgpu device lifecycle and compute shader dispatch.
    /// Includes PCIe bandwidth calibration for query optimization.
    pub struct GpuDevice {
        #[allow(dead_code)] // Used for GPU compute operations
        device: wgpu::Device,
        #[allow(dead_code)] // Used for GPU command submission
        queue: wgpu::Queue,
        pcie_bandwidth_gbps: f64,
    }

    impl GpuDevice {
        /// Get or initialize the global GPU device
        #[allow(static_mut_refs)]
        pub fn get_or_init() -> Result<&'static GpuDevice> {
            // SAFETY: INIT.call_once ensures single initialization; GPU_DEVICE is only written once
            unsafe {
                INIT.call_once(|| match Self::new() {
                    Ok(device) => GPU_DEVICE = Some(device),
                    Err(e) => panic!("Failed to initialize GPU: {}", e),
                });

                GPU_DEVICE
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("GPU device not initialized"))
            }
        }

        /// Initialize GPU device with PCIe calibration
        fn new() -> Result<Self> {
            // Create wgpu instance
            let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
                backends: wgpu::Backends::all(),
                ..Default::default()
            });

            // Request adapter (GPU device)
            let adapter =
                pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    force_fallback_adapter: false,
                    compatible_surface: None,
                }))
                .context("Failed to find GPU adapter. Ensure GPU drivers are installed.")?;

            // Get adapter info for logging
            let adapter_info = adapter.get_info();
            eprintln!(
                "🔍 GPU Detected: {} ({:?})",
                adapter_info.name, adapter_info.backend
            );

            // Request device and queue
            let (device, queue) = pollster::block_on(adapter.request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("PMAT Analytics GPU"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: Default::default(),
                },
                None,
            ))
            .context("Failed to create GPU device")?;

            // Calibrate PCIe bandwidth
            let pcie_bandwidth_gbps = Self::calibrate_pcie_bandwidth(&device, &queue)?;

            Ok(GpuDevice {
                device,
                queue,
                pcie_bandwidth_gbps,
            })
        }

        /// Calibrate PCIe bandwidth (P0-5)
        ///
        /// Measures actual bandwidth instead of assuming 32 GB/s.
        /// Uses 50ms micro-benchmark for accuracy.
        ///
        /// Reference: Gregg & Hazelwood (2011) ISPASS
        fn calibrate_pcie_bandwidth(device: &wgpu::Device, queue: &wgpu::Queue) -> Result<f64> {
            const CALIBRATION_SIZE: usize = 30_000_000; // 30M f64 = 240 MB (under 256 MB limit)

            let start = std::time::Instant::now();

            // Create test buffer (CPU → GPU transfer)
            let test_data: Vec<f64> = (0..CALIBRATION_SIZE).map(|i| i as f64).collect();
            let test_bytes = bytemuck::cast_slice(&test_data);

            let gpu_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("PCIe Calibration Buffer (GPU)"),
                contents: test_bytes,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            });

            // Create staging buffer for readback (GPU → CPU)
            let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("PCIe Calibration Buffer (Staging)"),
                size: test_bytes.len() as u64,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            // Copy from GPU to staging (GPU → CPU transfer)
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("PCIe Calibration Encoder"),
            });
            encoder.copy_buffer_to_buffer(
                &gpu_buffer,
                0,
                &staging_buffer,
                0,
                test_bytes.len() as u64,
            );
            queue.submit(std::iter::once(encoder.finish()));

            // Wait for GPU operations to complete
            let buffer_slice = staging_buffer.slice(..);
            let (tx, rx) = std::sync::mpsc::channel();
            buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
                tx.send(result).ok();
            });
            device.poll(wgpu::Maintain::Wait);
            rx.recv()
                .context("Failed to map buffer")?
                .context("Buffer mapping failed")?;

            let elapsed = start.elapsed();

            // Unmap staging buffer (BufferSlice is Copy, so just let it go out of scope)
            staging_buffer.unmap();

            // Calculate bandwidth
            let bytes_transferred = test_bytes.len() as f64;
            let seconds = elapsed.as_secs_f64();
            let bandwidth_gbps = (bytes_transferred / seconds) / 1_000_000_000.0;

            // Validate bandwidth is within realistic range
            // Note: wgpu overhead can dominate for small transfers, so we use conservative limits
            if bandwidth_gbps < 0.1 || bandwidth_gbps > 35.0 {
                bail!(
                    "PCIe calibration out of range: {:.2} GB/s (expected 0.1-35 GB/s). \
                     This may indicate severe driver issues or GPU unavailability.",
                    bandwidth_gbps
                );
            }

            // Warn if bandwidth seems unusually low (may indicate wgpu overhead dominating)
            if bandwidth_gbps < 2.0 {
                eprintln!(
                    "⚠️  Low measured bandwidth ({:.2} GB/s). This is normal for wgpu's command \
                     submission overhead. Actual PCIe bandwidth may be higher.",
                    bandwidth_gbps
                );
            }

            // Warn if calibration took too long
            if elapsed.as_millis() > 100 {
                eprintln!(
                    "⚠️  PCIe calibration took {:?} (target: <100ms). \
                     Consider reducing CALIBRATION_SIZE.",
                    elapsed
                );
            }

            eprintln!(
                "📊 PCIe Bandwidth: {:.2} GB/s (calibrated in {:?})",
                bandwidth_gbps, elapsed
            );

            // Drop buffers to free GPU memory
            drop(gpu_buffer);
            drop(staging_buffer);

            Ok(bandwidth_gbps)
        }

        /// Get calibrated PCIe bandwidth
        pub fn pcie_bandwidth(&self) -> f64 {
            self.pcie_bandwidth_gbps
        }

        /// Compute sum of f64 array using GPU
        pub fn compute_sum(&self, data: &[f64]) -> Result<f64> {
            // For small datasets, GPU overhead isn't worth it
            if data.len() < 10_000 {
                return Ok(data.iter().sum());
            }

            // TODO: Implement GPU compute shader for parallel sum
            // For now, fall back to CPU
            // This is a placeholder until we implement the WGSL shader
            Ok(data.iter().sum())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::stats::*;
    use super::*;

    #[test]
    fn test_backend_auto_select() {
        let backend = BackendSelector::auto_select();

        #[cfg(feature = "analytics-simd")]
        assert_eq!(backend, Backend::Simd);

        #[cfg(not(feature = "analytics-simd"))]
        assert_eq!(backend, Backend::Scalar);
    }

    #[test]
    fn test_generate_dataset() {
        let dataset = generate_test_dataset(1000);
        assert_eq!(dataset.len(), 1000);

        // Check for mix of positive/negative values
        let positive = dataset.iter().filter(|&&x| x > 0.0).count();
        let negative = dataset.iter().filter(|&&x| x < 0.0).count();
        assert!(positive > 0);
        assert!(negative > 0);
    }

    #[test]
    fn test_mean_and_std() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let (mean, std) = mean_and_std(&values);

        // Mean should be 3.0
        assert!((mean - 3.0).abs() < 0.01);

        // Std should be ~1.58 (sample std)
        assert!((std - 1.58).abs() < 0.1);
    }

    #[test]
    fn test_mean_and_std_empty() {
        let values: Vec<f64> = vec![];
        let (mean, std) = mean_and_std(&values);
        assert_eq!(mean, 0.0);
        assert_eq!(std, 0.0);
    }

    #[test]
    fn test_compute_avg_scalar() {
        let dataset = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let avg = compute_avg(&dataset, Backend::Scalar).unwrap();
        assert!((avg - 3.0).abs() < 0.01);
    }

    #[test]
    #[cfg(feature = "analytics-simd")]
    fn test_compute_avg_simd() {
        let dataset = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let avg = compute_avg(&dataset, Backend::Simd).unwrap();
        assert!((avg - 3.0).abs() < 0.01);
    }
}
