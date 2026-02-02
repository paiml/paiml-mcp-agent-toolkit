//! Coverage boost tests for services/brick_score.rs
//! Tests: SimdWidth, GpuBackend, CpuCapability, GpuCapability, RooflineParams,
//! ByteBudget, HardwareCapability, Bottleneck, scale_budgets_for_hardware

use crate::services::brick_score::{
    Bottleneck, ByteBudget, CpuCapability, GpuBackend, GpuCapability, HardwareCapability,
    RooflineParams, SimdWidth,
};

// ============ SimdWidth Tests ============

#[test]
fn test_simd_width_scalar_lanes() {
    assert_eq!(SimdWidth::Scalar.lanes(), 1);
}

#[test]
fn test_simd_width_neon_lanes() {
    assert_eq!(SimdWidth::Neon128.lanes(), 4);
}

#[test]
fn test_simd_width_sse2_lanes() {
    assert_eq!(SimdWidth::Sse2.lanes(), 4);
}

#[test]
fn test_simd_width_avx2_lanes() {
    assert_eq!(SimdWidth::Avx2.lanes(), 8);
}

#[test]
fn test_simd_width_avx512_lanes() {
    assert_eq!(SimdWidth::Avx512.lanes(), 16);
}

#[test]
fn test_simd_width_wasm_lanes() {
    assert_eq!(SimdWidth::WasmSimd128.lanes(), 4);
}

#[test]
fn test_simd_width_scalar_speedup() {
    assert!((SimdWidth::Scalar.compute_speedup() - 1.0).abs() < f64::EPSILON);
}

#[test]
fn test_simd_width_neon_speedup() {
    assert!((SimdWidth::Neon128.compute_speedup() - 4.0).abs() < f64::EPSILON);
}

#[test]
fn test_simd_width_avx2_speedup() {
    assert!((SimdWidth::Avx2.compute_speedup() - 10.0).abs() < f64::EPSILON);
}

#[test]
fn test_simd_width_avx512_speedup() {
    assert!((SimdWidth::Avx512.compute_speedup() - 12.0).abs() < f64::EPSILON);
}

#[test]
fn test_simd_width_default() {
    assert_eq!(SimdWidth::default(), SimdWidth::Scalar);
}

#[test]
fn test_simd_width_serde() {
    let widths = vec![
        SimdWidth::Scalar,
        SimdWidth::Neon128,
        SimdWidth::Sse2,
        SimdWidth::Avx2,
        SimdWidth::Avx512,
        SimdWidth::WasmSimd128,
    ];
    for w in &widths {
        let json = serde_json::to_string(w).unwrap();
        let back: SimdWidth = serde_json::from_str(&json).unwrap();
        assert_eq!(*w, back);
    }
}

// ============ GpuBackend Tests ============

#[test]
fn test_gpu_backend_default() {
    assert_eq!(GpuBackend::default(), GpuBackend::None);
}

#[test]
fn test_gpu_backend_serde() {
    let backends = vec![
        GpuBackend::None,
        GpuBackend::Cuda,
        GpuBackend::Wgpu,
        GpuBackend::Metal,
        GpuBackend::Vulkan,
    ];
    for b in &backends {
        let json = serde_json::to_string(b).unwrap();
        let back: GpuBackend = serde_json::from_str(&json).unwrap();
        assert_eq!(*b, back);
    }
}

// ============ CpuCapability Tests ============

#[test]
fn test_cpu_capability_default() {
    let cpu = CpuCapability::default();
    assert_eq!(cpu.vendor, "Unknown");
    assert_eq!(cpu.model, "Unknown");
    assert_eq!(cpu.cores, 1);
    assert_eq!(cpu.threads, 1);
    assert_eq!(cpu.simd, SimdWidth::Scalar);
    assert!((cpu.base_freq_ghz - 3.0).abs() < f64::EPSILON);
    assert!((cpu.peak_gflops - 6.0).abs() < f64::EPSILON);
    assert!((cpu.memory_bw_gbps - 25.0).abs() < f64::EPSILON);
}

#[test]
fn test_cpu_capability_serde() {
    let cpu = CpuCapability {
        vendor: "AMD".to_string(),
        model: "Ryzen 9".to_string(),
        cores: 16,
        threads: 32,
        simd: SimdWidth::Avx2,
        base_freq_ghz: 3.7,
        peak_gflops: 1024.0,
        memory_bw_gbps: 76.8,
    };
    let json = serde_json::to_string(&cpu).unwrap();
    let back: CpuCapability = serde_json::from_str(&json).unwrap();
    assert_eq!(back.vendor, "AMD");
    assert_eq!(back.cores, 16);
    assert_eq!(back.simd, SimdWidth::Avx2);
}

#[test]
fn test_cpu_capability_clone() {
    let cpu = CpuCapability::default();
    let cloned = cpu.clone();
    assert_eq!(cloned.vendor, cpu.vendor);
    assert_eq!(cloned.cores, cpu.cores);
}

// ============ GpuCapability Tests ============

#[test]
fn test_gpu_capability_serde() {
    let gpu = GpuCapability {
        vendor: "NVIDIA".to_string(),
        model: "RTX 4090".to_string(),
        backend: GpuBackend::Cuda,
        compute_capability: Some("8.9".to_string()),
        peak_tflops_fp32: 82.6,
        peak_tflops_tensor: Some(330.0),
        memory_bw_gbps: 1008.0,
        vram_gb: 24.0,
    };
    let json = serde_json::to_string(&gpu).unwrap();
    let back: GpuCapability = serde_json::from_str(&json).unwrap();
    assert_eq!(back.vendor, "NVIDIA");
    assert_eq!(back.backend, GpuBackend::Cuda);
    assert!((back.vram_gb - 24.0).abs() < f64::EPSILON);
}

// ============ RooflineParams Tests ============

#[test]
fn test_roofline_params_default() {
    let params = RooflineParams::default();
    assert!((params.cpu_arithmetic_intensity - 0.24).abs() < f64::EPSILON);
    assert!(params.gpu_arithmetic_intensity.is_none());
}

#[test]
fn test_roofline_params_serde() {
    let params = RooflineParams {
        cpu_arithmetic_intensity: 0.5,
        gpu_arithmetic_intensity: Some(2.0),
    };
    let json = serde_json::to_string(&params).unwrap();
    let back: RooflineParams = serde_json::from_str(&json).unwrap();
    assert!((back.cpu_arithmetic_intensity - 0.5).abs() < f64::EPSILON);
    assert_eq!(back.gpu_arithmetic_intensity, Some(2.0));
}

// ============ ByteBudget Tests ============

#[test]
fn test_byte_budget_default() {
    let budget = ByteBudget::default();
    assert!((budget.gb_per_sec - 25.0).abs() < f64::EPSILON);
    assert_eq!(budget.page_size, 4096);
    assert!(budget.us_per_page > 0.0);
}

#[test]
fn test_byte_budget_serde() {
    let budget = ByteBudget {
        us_per_page: 0.5,
        gb_per_sec: 50.0,
        page_size: 8192,
    };
    let json = serde_json::to_string(&budget).unwrap();
    let back: ByteBudget = serde_json::from_str(&json).unwrap();
    assert!((back.gb_per_sec - 50.0).abs() < f64::EPSILON);
    assert_eq!(back.page_size, 8192);
}

#[test]
fn test_byte_budget_copy() {
    let budget = ByteBudget::default();
    let copied = budget;
    assert_eq!(copied.page_size, budget.page_size);
}

// ============ HardwareCapability Tests ============

#[test]
fn test_hardware_capability_default() {
    let hw = HardwareCapability::default();
    assert_eq!(hw.hostname, "unknown");
    assert_eq!(hw.cpu.vendor, "Unknown");
    assert!(hw.gpu.is_none());
    assert!(hw.byte_budget.is_some());
}

#[test]
fn test_hardware_capability_serde() {
    let hw = HardwareCapability::default();
    let json = serde_json::to_string(&hw).unwrap();
    let back: HardwareCapability = serde_json::from_str(&json).unwrap();
    assert_eq!(back.hostname, "unknown");
}

// ============ Bottleneck Tests ============

#[test]
fn test_bottleneck_memory_bound() {
    let hw = HardwareCapability::default();
    // Low AI → memory-bound
    assert_eq!(hw.bottleneck(0.1, false), Bottleneck::Memory);
}

#[test]
fn test_bottleneck_compute_bound() {
    let hw = HardwareCapability::default();
    // High AI → compute-bound
    assert_eq!(hw.bottleneck(10.0, false), Bottleneck::Compute);
}

#[test]
fn test_bottleneck_gpu_no_gpu_capability() {
    let hw = HardwareCapability::default();
    // GPU with no gpu_arithmetic_intensity → always memory-bound (threshold is f64::MAX)
    assert_eq!(hw.bottleneck(1000.0, true), Bottleneck::Memory);
}

#[test]
fn test_bottleneck_gpu_with_threshold() {
    let hw = HardwareCapability {
        roofline: RooflineParams {
            cpu_arithmetic_intensity: 0.24,
            gpu_arithmetic_intensity: Some(5.0),
        },
        ..Default::default()
    };
    assert_eq!(hw.bottleneck(3.0, true), Bottleneck::Memory);
    assert_eq!(hw.bottleneck(10.0, true), Bottleneck::Compute);
}

#[test]
fn test_bottleneck_serde() {
    let bottlenecks = vec![Bottleneck::Memory, Bottleneck::Compute];
    for b in &bottlenecks {
        let json = serde_json::to_string(b).unwrap();
        let back: Bottleneck = serde_json::from_str(&json).unwrap();
        assert_eq!(*b, back);
    }
}

// ============ scale_budgets_for_hardware Tests ============

#[test]
fn test_scale_budgets_for_hardware() {
    use crate::services::brick_score::{scale_budgets_for_hardware, BrickBudget};

    let base_budgets = vec![
        BrickBudget {
            name: "test_op".to_string(),
            max_us: 100.0,
        },
    ];

    // With default hardware (Scalar SIMD, 25 GB/s)
    let hw = HardwareCapability::default();
    let scaled = scale_budgets_for_hardware(&base_budgets, &hw);
    assert_eq!(scaled.len(), 1);
    // Scalar speedup = 1.0, mem_bw_factor = 25/25 = 1.0, scale = sqrt(1*1) = 1.0
    assert!((scaled[0].max_us - 100.0).abs() < 0.01);

    // With AVX2 hardware
    let hw_avx2 = HardwareCapability {
        cpu: CpuCapability {
            simd: SimdWidth::Avx2,
            memory_bw_gbps: 50.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let scaled_avx2 = scale_budgets_for_hardware(&base_budgets, &hw_avx2);
    // avx2 speedup = 10.0, mem_bw_factor = 50/25 = 2.0, scale = sqrt(20) ≈ 4.47
    assert!(scaled_avx2[0].max_us < 100.0); // Should be stricter
}

// ============ default_hardware_path Tests ============

#[test]
fn test_default_hardware_path() {
    let path = crate::services::brick_score::default_hardware_path();
    assert!(path.to_string_lossy().contains("hardware.toml"));
    assert!(path.to_string_lossy().contains(".pmat"));
}

// ============ load_hardware_capability Tests ============

#[test]
fn test_load_hardware_capability_nonexistent() {
    let result =
        crate::services::brick_score::load_hardware_capability(Some(std::path::Path::new(
            "/nonexistent/hardware.toml",
        )));
    assert!(result.is_none());
}
