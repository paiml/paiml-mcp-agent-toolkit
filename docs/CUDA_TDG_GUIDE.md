# CUDA-SIMD Technical Debt Gradient (TDG) Guide

The CUDA-SIMD TDG system provides comprehensive GPU/SIMD code quality scoring using a 100-point Karl Popper falsification methodology, integrated with Toyota Production System principles.

## Overview

CUDA-SIMD TDG analyzes compute code quality across six categories:

1. **Falsifiability & Testability** (25 points) - *GATEWAY* - Barrier safety, bounds verification
2. **Reproducibility Infrastructure** (25 points) - Deterministic output, version pinning
3. **Transparency & Openness** (20 points) - PTX inspection, register allocation
4. **Statistical Rigor** (15 points) - Warmup, sample count, confidence intervals
5. **Historical Integrity** (10 points) - Fault lineage from Tauranta taxonomy
6. **GPU/SIMD Specific** (5 points) - Warp efficiency, memory throughput

Total possible score: **100 points** with gateway rule.

### Gateway Rule

If Category A (Falsifiability) scores below 15 points, the total score is 0. This implements Karl Popper's demarcation criterion: code that cannot be tested for correctness is not production-ready.

## Quick Start

### Basic Analysis

```bash
# Analyze a single file
pmat cuda-tdg src/kernel.cu

# Analyze entire GPU project
pmat cuda-tdg ./cuda-kernels/

# Show score breakdown
pmat cuda-tdg score --breakdown .

# Quality gate for CI/CD
pmat cuda-tdg gate --min-score 85 --fail-on-p0 .
```

### Output Formats

```bash
# Terminal output with colors (default)
pmat cuda-tdg . --format terminal

# JSON for programmatic use
pmat cuda-tdg . --format json

# Markdown for documentation
pmat cuda-tdg . --format markdown

# SARIF for IDE integration
pmat cuda-tdg . --format sarif

# Write to file
pmat cuda-tdg . --output report.json --format json
```

### Subcommands

```bash
# Full analysis
pmat cuda-tdg analyze ./src

# Score with detailed breakdown
pmat cuda-tdg score --breakdown .

# Generate detailed report
pmat cuda-tdg report . --format markdown --output report.md

# Check barrier safety (PARITY-114 detection)
pmat cuda-tdg barrier-check kernel.cu

# Validate FlashAttention tile dimensions
pmat cuda-tdg validate-tiles --head-dim 128 --tile-kv 64

# CI/CD quality gate
pmat cuda-tdg gate --min-score 85 --fail-on-p0 .

# Kaizen continuous improvement metrics
pmat cuda-tdg kaizen .

# Show Tauranta fault taxonomy
pmat cuda-tdg taxonomy
```

## Understanding the 100-Point Scoring System

### Category A: Falsifiability & Testability (25 pts) - GATEWAY

| Criterion | Points | Description |
|-----------|--------|-------------|
| A.1 Barrier Safety | 5 | All `bar.sync` reachable from all threads |
| A.2 Bounds Verification | 5 | Shared memory indices within tile dimensions |
| A.3 Divergence Testing | 5 | Branch coverage includes warp divergence cases |
| A.4 Memory Race Detection | 5 | ThreadSanitizer or equivalent analysis |
| A.5 Occupancy Bounds | 5 | Register/shared memory within SM limits |

**Gateway Rule**: If Category A total < 15, the entire score becomes 0.

### Category B: Reproducibility Infrastructure (25 pts)

| Criterion | Points | Description |
|-----------|--------|-------------|
| B.1 Deterministic Output | 8 | Bitwise reproducible results |
| B.2 Version Pinning | 5 | CUDA/Driver/SM version locked |
| B.3 Hardware Specification | 5 | GPU model, compute capability documented |
| B.4 Benchmark Harness | 4 | Criterion-style statistical benchmarking |
| B.5 CI/CD Integration | 3 | Automated regression on GPU hardware |

### Category C: Transparency & Openness (20 pts)

| Criterion | Points | Description |
|-----------|--------|-------------|
| C.1 PTX Inspection | 6 | Generated PTX accessible and documented |
| C.2 Register Allocation | 5 | `--ptxas-options=-v` output analyzed |
| C.3 Occupancy Calculation | 5 | SM occupancy explicitly computed |
| C.4 Memory Layout | 4 | Shared memory bank mapping documented |

### Category D: Statistical Rigor (15 pts)

| Criterion | Points | Description |
|-----------|--------|-------------|
| D.1 Warmup Iterations | 4 | ≥3s warmup before measurement |
| D.2 Sample Count | 4 | ≥100 samples for statistical significance |
| D.3 Outlier Analysis | 4 | IQR-based outlier detection reported |
| D.4 Confidence Intervals | 3 | 95% CI on throughput metrics |

### Category E: Historical Integrity (10 pts)

| Criterion | Points | Description |
|-----------|--------|-------------|
| E.1 Fault Lineage | 4 | PARITY/PAR ticket references |
| E.2 Regression Tests | 3 | Tests derived from historical bugs |
| E.3 Root Cause Documentation | 3 | 5-Why analysis for each P0 defect |

### Category F: GPU/SIMD Specific (5 pts)

| Criterion | Points | Description |
|-----------|--------|-------------|
| F.1 Warp Efficiency | 2 | Active threads / warp size ratio |
| F.2 Memory Throughput | 2 | Achieved vs theoretical bandwidth |
| F.3 Instruction Mix | 1 | FMA/memory instruction ratio |

### Grade Scale

| Grade | Score Range | Description |
|-------|-------------|-------------|
| A+ | 90-100 | Production-ready, minimal technical debt |
| A | 80-89 | Production-ready with monitoring |
| B | 70-79 | Acceptable, prioritize improvements |
| C | 60-69 | Technical debt accumulating |
| D | 50-59 | Significant remediation needed |
| F | 0-49 | Not production-ready |
| FAIL | 0 (gateway) | Falsifiability requirements not met |

## Tauranta Fault Taxonomy

CUDA-TDG uses the PAIML Tauranta fault taxonomy for defect classification:

### P0 Critical Defects (Crash/Corruption Risk)

| Ticket | Description | Detection |
|--------|-------------|-----------|
| PARITY-114 | Barrier divergence: thread exits before bar.sync | PTX CFG analysis |
| PAR-002 | CUDA GEMV Error 700: illegal memory access | Bounds checking |
| PAR-041 | FlashAttention tile_kv < head_dim overflow | Tile dimension validation |

### P1 Performance Defects

| Ticket | Description | Detection |
|--------|-------------|-----------|
| PAR-001 | Q4_K dequantization inefficiency | Throughput analysis |
| PAR-034 | Tensor Core GEMM: unused capability | SM utilization analysis |
| PAR-036 | Non-persistent thread design | Thread lifecycle analysis |
| PAR-037 | Missing CUDA Graphs | API call pattern |
| PAR-039 | Megakernel fusion opportunity missed | Kernel launch count |

### P2 Efficiency Defects

| Ticket | Description | Detection |
|--------|-------------|-----------|
| PAR-005 | Redundant weight transfers | D2H/H2D ratio |
| PAR-028 | FP32 KV cache overhead | Precision analysis |

## CI/CD Integration

### GitHub Actions

```yaml
name: CUDA-TDG Quality Gate

on: [push, pull_request]

jobs:
  cuda-tdg:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install pmat
        run: cargo install pmat

      - name: Run CUDA-TDG Gate
        run: |
          pmat cuda-tdg gate \
            --min-score 85 \
            --fail-on-p0 \
            --format sarif \
            --output cuda-tdg.sarif \
            ./src

      - name: Upload SARIF
        uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: cuda-tdg.sarif
```

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Run CUDA-TDG on staged CUDA/PTX files
cuda_files=$(git diff --cached --name-only --diff-filter=ACM | grep -E '\.(cu|cuh|ptx|wgsl)$')

if [ -n "$cuda_files" ]; then
    echo "Running CUDA-TDG on staged GPU files..."
    pmat cuda-tdg gate --min-score 70 --fail-on-p0 $cuda_files
    if [ $? -ne 0 ]; then
        echo "CUDA-TDG quality gate failed. Fix defects before committing."
        exit 1
    fi
fi
```

## Toyota Way Integration

CUDA-TDG implements Toyota Production System principles:

### Jidoka (Autonomation)

Automatic quality gates stop the line when defects are detected:

```bash
# Stop on any P0 defect
pmat cuda-tdg gate --fail-on-p0 .
```

### Genchi Genbutsu (Go and See)

Analyze actual PTX/SIMD artifacts, not just source code:

```bash
# Inspect generated PTX
pmat cuda-tdg analyze --include-ptx kernel.cu
```

### Kaizen (Continuous Improvement)

Track defect metrics over time:

```bash
# Generate Kaizen report
pmat cuda-tdg kaizen --since 2024-01-01 .
```

### Poka-Yoke (Error Proofing)

Static analysis prevents common GPU errors:

```bash
# Check barrier safety before deployment
pmat cuda-tdg barrier-check kernel.cu
```

### Hansei (Reflection)

5-Why root cause analysis for each P0 defect:

```bash
# Show taxonomy with root causes
pmat cuda-tdg taxonomy
```

## Example: Analyzing FlashAttention

```bash
# Validate tile dimensions for attention kernel
pmat cuda-tdg validate-tiles \
    --head-dim 128 \
    --tile-kv 64 \
    --shared-memory 49152

# Output:
# Tile Dimension Validation
# =========================
#
# Head Dimension: 128
# Tile KV: 64
# Shared Memory Limit: 49152 bytes
# Shared Memory Required: 16384 bytes
#
# Status: INVALID
#
# Issue: PAR-041 - tile_kv < head_dim
# Fix: Set tile_kv >= 128 (currently 64)
```

## Programmatic Usage

```rust
use pmat::tdg::{CudaSimdAnalyzer, CudaSimdConfig, DefectSeverity};
use std::path::Path;

fn main() -> anyhow::Result<()> {
    // Configure analyzer
    let config = CudaSimdConfig {
        min_score: 85.0,
        fail_on_p0: true,
        analyze_simd: true,
        analyze_wgpu: true,
        shared_memory_limit: 49152,
        register_limit: 64,
    };

    let analyzer = CudaSimdAnalyzer::with_config(config);
    let result = analyzer.analyze(Path::new("./cuda-kernels"))?;

    // Check score
    println!("Score: {:.1}/100 (Grade: {})", result.score.total, result.score.grade);

    // Check for P0 defects
    let p0_count = result.defects.iter()
        .filter(|d| d.defect_class.severity == DefectSeverity::P0Critical)
        .count();

    if p0_count > 0 {
        eprintln!("Found {} P0 critical defects!", p0_count);
        for defect in &result.defects {
            if defect.defect_class.severity == DefectSeverity::P0Critical {
                eprintln!("  - {}: {}", defect.defect_class.ticket_id, defect.defect_class.description);
            }
        }
    }

    // Quality gate
    if analyzer.passes_quality_gate(&result) {
        println!("Quality gate: PASSED");
        Ok(())
    } else {
        anyhow::bail!("Quality gate: FAILED");
    }
}
```

## References

### Academic Citations

1. Popper, K. R. (1959). *The Logic of Scientific Discovery*. Routledge.
2. Liker, J. K. (2004). *The Toyota Way*. McGraw-Hill.
3. Volkov, V., & Demmel, J. W. (2008). "Benchmarking GPUs to tune dense linear algebra." *SC '08*.
4. Dao, T., et al. (2022). "FlashAttention: Fast and Memory-Efficient Exact Attention." *NeurIPS*.
5. Gregg, C., & Hazelwood, K. (2011). "Where is the data? Why you cannot debate GPU vs. CPU performance without the answer." *ISPASS*.
6. Narasiman, V., et al. (2011). "Improving GPU performance via large warps and two-level warp scheduling." *MICRO*.
7. Wong, H., et al. (2010). "Demystifying GPU microarchitecture through microbenchmarking." *ISPASS*.

### PAIML Tauranta Fault History

- PARITY-114: Barrier divergence in realizar GpuExecutor
- PAR-001 through PAR-041: GPU optimization and correctness issues
- Full taxonomy: `pmat cuda-tdg taxonomy`

## See Also

- [TDG Guide](./TDG_GUIDE.md) - General Technical Debt Gradient
- [CUDA-SIMD TDG Specification](./specifications/components/language-support.md) - Full specification
- [CI/CD Integration](./integrations/ci-cd-integration.md) - GitHub Actions setup
