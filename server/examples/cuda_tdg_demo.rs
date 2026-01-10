//! CUDA-SIMD Technical Debt Gradient (TDG) Example
//!
//! This example demonstrates how to use pmat's CUDA-SIMD TDG module for
//! analyzing GPU/SIMD code with the 100-point Karl Popper falsification
//! scoring system, integrated with Toyota Production System principles.
//!
//! Run with: `cargo run --example cuda_tdg_demo`
//!
//! # Features Demonstrated
//!
//! 1. Analyzing CUDA files for defects (PARITY-114, PAR-xxx patterns)
//! 2. 100-point Popper falsification scoring
//! 3. Barrier safety analysis
//! 4. Memory coalescing detection
//! 5. Quality gate for CI/CD integration
//! 6. Kaizen continuous improvement metrics
//!
//! # References
//!
//! - Popper, K. R. (1959). *The Logic of Scientific Discovery*. Routledge.
//! - Liker, J. K. (2004). *The Toyota Way*. McGraw-Hill.
//! - Volkov, V., & Demmel, J. W. (2008). "Benchmarking GPUs to tune dense linear algebra."

use anyhow::Result;
use pmat::tdg::{
    CudaSimdAnalyzer, CudaSimdConfig, CudaSimdTdgResult, DefectSeverity, DefectTaxonomy,
};

fn main() -> Result<()> {
    println!("CUDA-SIMD Technical Debt Gradient Demo");
    println!("{}", "=".repeat(60));

    // Example 1: Show Tauranta fault taxonomy
    println!("\nExample 1: Tauranta Fault Taxonomy");
    println!("{}", "-".repeat(40));
    demonstrate_taxonomy();

    // Example 2: Analyze sample CUDA code
    println!("\nExample 2: Analyzing CUDA Code");
    println!("{}", "-".repeat(40));
    demonstrate_analysis()?;

    // Example 3: Understanding the 100-point scoring system
    println!("\nExample 3: 100-Point Popper Falsification Score");
    println!("{}", "-".repeat(40));
    demonstrate_scoring();

    // Example 4: Quality gate for CI/CD
    println!("\nExample 4: Quality Gate for CI/CD");
    println!("{}", "-".repeat(40));
    demonstrate_quality_gate()?;

    // Example 5: Custom configuration
    println!("\nExample 5: Custom Configuration");
    println!("{}", "-".repeat(40));
    demonstrate_custom_config()?;

    println!("\n{}", "=".repeat(60));
    println!("CUDA-TDG demo completed!");

    Ok(())
}

fn demonstrate_taxonomy() {
    let taxonomy = DefectTaxonomy::with_tauranta_patterns();

    println!("\n  P0 Critical Defects (cause crashes/incorrect results):");
    for defect in taxonomy.all() {
        if defect.severity == DefectSeverity::P0Critical {
            println!("    - {}: {}", defect.ticket_id, defect.description);
            println!("      Detection: {}", defect.detection_method);
        }
    }

    println!("\n  P1 Performance Defects:");
    for defect in taxonomy.all() {
        if defect.severity == DefectSeverity::P1Performance {
            println!("    - {}: {}", defect.ticket_id, defect.description);
        }
    }

    println!("\n  P2 Efficiency Defects:");
    for defect in taxonomy.all() {
        if defect.severity == DefectSeverity::P2Efficiency {
            println!("    - {}: {}", defect.ticket_id, defect.description);
        }
    }
}

fn demonstrate_analysis() -> Result<()> {
    // Create sample CUDA file for analysis
    let temp_dir = tempfile::tempdir()?;
    let cuda_file = temp_dir.path().join("sample_kernel.cu");

    // Write sample CUDA code with intentional issues for demonstration
    std::fs::write(
        &cuda_file,
        r#"
// Sample CUDA kernel with common issues
// This demonstrates PARITY-114 barrier divergence risk

__global__ void naive_gemm_kernel(float *a, float *b, float *c, int n) {
    // PAR-034: Missing Tensor Core usage for matmul
    int row = blockIdx.y * blockDim.y + threadIdx.y;
    int col = blockIdx.x * blockDim.x + threadIdx.x;

    // Early exit creates barrier divergence risk (PARITY-114)
    if (row >= n || col >= n) return;

    __syncthreads();  // Barrier after conditional return!

    float sum = 0.0f;
    for (int k = 0; k < n; k++) {
        sum += a[row * n + k] * b[k * n + col];
    }
    c[row * n + col] = sum;
}
"#,
    )?;

    let analyzer = CudaSimdAnalyzer::new();
    let result = analyzer.analyze(&cuda_file)?;

    print_analysis_result(&result);

    Ok(())
}

fn print_analysis_result(result: &CudaSimdTdgResult) {
    println!("  Path: {}", result.path.display());
    println!("  Files analyzed: {}", result.files_analyzed);
    println!(
        "    CUDA: {}, SIMD: {}, WGPU: {}",
        result.cuda_files, result.simd_files, result.wgpu_files
    );

    println!(
        "\n  Score: {:.1}/100 (Grade: {})",
        result.score.total, result.score.grade
    );
    println!(
        "  Gateway: {}",
        if result.score.gateway_passed {
            "PASSED"
        } else {
            "FAILED"
        }
    );

    if !result.defects.is_empty() {
        println!("\n  Defects Found:");
        for defect in &result.defects {
            let severity_icon = match defect.defect_class.severity {
                DefectSeverity::P0Critical => "[P0]",
                DefectSeverity::P1Performance => "[P1]",
                DefectSeverity::P2Efficiency => "[P2]",
                DefectSeverity::P3Minor => "[P3]",
            };
            println!(
                "    {} {} - {}",
                severity_icon, defect.defect_class.ticket_id, defect.defect_class.description
            );
            if let Some(suggestion) = &defect.suggestion {
                println!("       Fix: {}", suggestion);
            }
        }
    }

    println!("\n  Barrier Safety:");
    println!(
        "    Total: {}, Safe: {}, Unsafe: {}",
        result.barrier_safety.total_barriers,
        result.barrier_safety.safe_barriers,
        result.barrier_safety.unsafe_barriers.len()
    );
    println!(
        "    Safety Score: {:.1}%",
        result.barrier_safety.safety_score * 100.0
    );
}

fn demonstrate_scoring() {
    println!("\n  The 100-Point Popper Falsification Scoring System:");
    println!();
    println!("  Category A: Falsifiability & Testability (25 pts) [GATEWAY]");
    println!("    - A.1: Barrier Safety (5 pts)");
    println!("    - A.2: Bounds Verification (5 pts)");
    println!("    - A.3: Divergence Testing (5 pts)");
    println!("    - A.4: Memory Race Detection (5 pts)");
    println!("    - A.5: Occupancy Bounds (5 pts)");
    println!();
    println!("  Category B: Reproducibility Infrastructure (25 pts)");
    println!("    - B.1: Deterministic Output (8 pts)");
    println!("    - B.2: Version Pinning (5 pts)");
    println!("    - B.3: Hardware Specification (5 pts)");
    println!("    - B.4: Benchmark Harness (4 pts)");
    println!("    - B.5: CI/CD Integration (3 pts)");
    println!();
    println!("  Category C: Transparency & Openness (20 pts)");
    println!("    - C.1: PTX Inspection (6 pts)");
    println!("    - C.2: Register Allocation (5 pts)");
    println!("    - C.3: Occupancy Calculation (5 pts)");
    println!("    - C.4: Memory Layout (4 pts)");
    println!();
    println!("  Category D: Statistical Rigor (15 pts)");
    println!("    - D.1: Warmup Iterations (4 pts)");
    println!("    - D.2: Sample Count (4 pts)");
    println!("    - D.3: Outlier Analysis (4 pts)");
    println!("    - D.4: Confidence Intervals (3 pts)");
    println!();
    println!("  Category E: Historical Integrity (10 pts)");
    println!("    - E.1: Fault Lineage (4 pts)");
    println!("    - E.2: Regression Tests (3 pts)");
    println!("    - E.3: Root Cause Documentation (3 pts)");
    println!();
    println!("  Category F: GPU/SIMD Specific (5 pts)");
    println!("    - F.1: Warp Efficiency (2 pts)");
    println!("    - F.2: Memory Throughput (2 pts)");
    println!("    - F.3: Instruction Mix (1 pt)");
    println!();
    println!("  Gateway Rule: If Category A < 15 points, total score = 0");
    println!("  (Implements Popper's demarcation criterion for falsifiability)");
}

fn demonstrate_quality_gate() -> Result<()> {
    // Create a clean CUDA file
    let temp_dir = tempfile::tempdir()?;
    let cuda_file = temp_dir.path().join("clean_kernel.cu");

    std::fs::write(
        &cuda_file,
        r#"
// Clean CUDA kernel using Tensor Cores
__global__ void tensor_core_gemm(half *a, half *b, float *c, int n) {
    // Using wmma for Tensor Core operations
    wmma::fragment<wmma::matrix_a, 16, 16, 16, half, wmma::row_major> a_frag;
    wmma::fragment<wmma::matrix_b, 16, 16, 16, half, wmma::col_major> b_frag;
    wmma::fragment<wmma::accumulator, 16, 16, 16, float> c_frag;

    wmma::fill_fragment(c_frag, 0.0f);

    // Safe barrier - all threads reach this point
    __syncthreads();

    // Tensor Core matrix multiply
    wmma::mma_sync(c_frag, a_frag, b_frag, c_frag);
}
"#,
    )?;

    let config = CudaSimdConfig {
        min_score: 70.0,
        fail_on_p0: true,
        ..Default::default()
    };
    let analyzer = CudaSimdAnalyzer::with_config(config);
    let result = analyzer.analyze(&cuda_file)?;

    let passes = analyzer.passes_quality_gate(&result);

    println!("  Quality Gate Configuration:");
    println!("    Minimum Score: 70.0");
    println!("    Fail on P0: true");
    println!();
    println!("  Analysis Result:");
    println!("    Score: {:.1}/100", result.score.total);
    println!("    Grade: {}", result.score.grade);
    println!(
        "    P0 Defects: {}",
        result
            .defects
            .iter()
            .filter(|d| d.defect_class.severity == DefectSeverity::P0Critical)
            .count()
    );
    println!();
    println!(
        "  Quality Gate: {}",
        if passes { "PASSED" } else { "FAILED" }
    );
    println!();
    println!("  CI/CD Integration:");
    println!("    $ pmat cuda-tdg gate --min-score 85 --fail-on-p0 ./src");

    Ok(())
}

fn demonstrate_custom_config() -> Result<()> {
    println!("  Custom Configuration Options:");
    println!();

    let config = CudaSimdConfig {
        min_score: 90.0,
        fail_on_p0: true,
        analyze_simd: true,
        analyze_wgpu: true,
        shared_memory_limit: 49152, // 48KB
        register_limit: 64,
    };

    println!(
        "    min_score: {} (quality gate threshold)",
        config.min_score
    );
    println!(
        "    fail_on_p0: {} (fail on critical defects)",
        config.fail_on_p0
    );
    println!(
        "    analyze_simd: {} (include AVX/NEON)",
        config.analyze_simd
    );
    println!(
        "    analyze_wgpu: {} (include WGPU/WGSL)",
        config.analyze_wgpu
    );
    println!(
        "    shared_memory_limit: {} bytes",
        config.shared_memory_limit
    );
    println!("    register_limit: {} per thread", config.register_limit);

    println!();
    println!("  Toyota Way Principles Applied:");
    println!("    - Jidoka: Automatic stop on P0 defect detection");
    println!("    - Kaizen: Track defect metrics over time");
    println!("    - Poka-Yoke: Static analysis prevents common errors");
    println!("    - Genchi Genbutsu: Analyze actual PTX/SIMD artifacts");
    println!("    - Hansei: 5-Why root cause analysis for each defect");

    // Analyze current directory with custom config
    let analyzer = CudaSimdAnalyzer::with_config(config);

    // Create a simple WGSL file to demonstrate WGPU analysis
    let temp_dir = tempfile::tempdir()?;
    let wgsl_file = temp_dir.path().join("compute.wgsl");
    std::fs::write(
        &wgsl_file,
        r#"
@group(0) @binding(0) var<storage, read_write> data: array<f32>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    workgroupBarrier();
    data[gid.x] = data[gid.x] * 2.0;
    storageBarrier();
}
"#,
    )?;

    let result = analyzer.analyze(&wgsl_file)?;

    println!();
    println!("  WGPU Analysis Result:");
    println!("    WGPU files: {}", result.wgpu_files);
    println!("    Barriers: {}", result.barrier_safety.total_barriers);
    println!("    Score: {:.1}/100", result.score.total);

    Ok(())
}
