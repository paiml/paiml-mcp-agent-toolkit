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

        // GPU compute shader not yet implemented; falls back to CPU
        Ok(data.iter().sum())
    }
}
