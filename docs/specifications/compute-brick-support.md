# ComputeBrick Diagnostic Support Specification

**Version**: 1.0.0
**Status**: Active
**Authors**: PAIML Team
**Date**: 2026-01-09
**Ticket**: CB-IMPL-001

## Abstract

This specification extends the CUDA-SIMD Technical Debt Gradient (TDG) to provide comprehensive diagnostic support for the ComputeBrick paradigm—a WebGPU/WGSL shader generation system that produces GPU compute code from Rust type definitions. The specification integrates Toyota Production System (TPS) principles with Karl Popper's falsificationist methodology to create a 100-point quality scoring framework specifically designed for generated shader code.

## 1. Introduction

### 1.1 The ComputeBrick Paradigm

ComputeBrick (PROBAR-SPEC-009-P8) represents a paradigm shift in GPU compute development:

```
Traditional:  Hand-written WGSL → Manual testing → Runtime errors
ComputeBrick: Rust types → Generated WGSL → Static verification → Guaranteed safety
```

Key properties:
- **Zero hand-written shaders**: All WGSL derived from Rust type definitions
- **Compile-time verification**: Brick trait enforces pre-render validation
- **Jidoka enforcement**: Rendering blocked if verification fails

### 1.2 Theoretical Foundation

This specification draws from two complementary epistemological frameworks:

**Karl Popper's Falsificationism** (Popper, 1959):
> "The criterion of the scientific status of a theory is its falsifiability, or refutability, or testability." [1]

Applied to ComputeBrick: Every generated shader must be accompanied by tests capable of falsifying its correctness. Code that cannot be proven wrong cannot be trusted.

**Toyota Production System** (Ohno, 1988; Liker, 2004):
> "The Toyota Way is not about tools and techniques. It's about developing people who can use those tools and techniques." [2]

Applied to ComputeBrick: Static analysis embodies Jidoka (自働化)—automation with a human touch that stops on defect detection.

### 1.3 Problem Statement

Generated WGSL shaders present unique quality challenges:

| Challenge | Traditional WGSL | ComputeBrick | Risk |
|-----------|-----------------|--------------|------|
| Bounds checking | Manual | Must be generated | P0: OOB access |
| Workgroup sizing | Hardcoded | Declarative | P1: Occupancy loss |
| Barrier placement | Developer judgment | Generated | P0: Race conditions |
| Memory layout | Implicit | Type-derived | P2: Bank conflicts |

## 2. Fault Taxonomy

### 2.1 ComputeBrick-Specific Defects (Severity: P0-Critical)

| ID | Pattern | Description | Detection | Citation |
|----|---------|-------------|-----------|----------|
| CB-001 | WGPU_NO_BOUNDS_CHECK | `global_invocation_id` used without bounds validation | AST pattern match | [3] |
| CB-002 | WGSL_BARRIER_DIVERGENCE | `workgroupBarrier()` unreachable from some threads | CFG analysis | [4] |
| CB-003 | TILE_DIMENSION_MISMATCH | Tile size exceeds tensor dimensions | Constraint solving | [5] |
| CB-004 | SHARED_MEM_OVERFLOW | Workgroup shared memory exceeds 16KB limit | Static analysis | [6] |

### 2.2 Performance Defects (Severity: P1)

| ID | Pattern | Description | Detection | Citation |
|----|---------|-------------|-----------|----------|
| CB-010 | WGPU_SUBOPTIMAL_WORKGROUP | Workgroup size not multiple of 32 (warp) | Numeric check | [7] |
| CB-011 | WGSL_REDUNDANT_BARRIER | Barrier without preceding shared memory write | Data flow | [8] |
| CB-012 | LOW_VECTORIZATION_RATIO | <50% of operations use vector types | Type analysis | [9] |
| CB-013 | MISSING_COOPERATIVE_MATRIX | Matrix ops without subgroup cooperative usage | Pattern match | [10] |

### 2.3 Code Quality Defects (Severity: P2)

| ID | Pattern | Description | Detection | Citation |
|----|---------|-------------|-----------|----------|
| CB-020 | UNSAFE_NO_SAFETY_COMMENT | `unsafe` block without `// SAFETY:` | Regex + AST | [11] |
| CB-021 | MISSING_TARGET_FEATURE | SIMD intrinsics without `#[target_feature]` | Attribute check | [12] |
| CB-022 | EXCESSIVE_BARRIERS | >4 barriers per kernel suggests algorithmic issue | Count analysis | [13] |

## 3. 100-Point Popper Falsification Score

### 3.1 Scoring Framework

Following Popper's demarcation criterion [1], the score measures falsifiability:

```
ComputeBrick Score = A + B + C + D + E + F

Where:
  A = Falsifiability & Testability (25 points) - GATEWAY
  B = Reproducibility Infrastructure (25 points)
  C = Transparency & Openness (20 points)
  D = Statistical Rigor (15 points)
  E = Historical Integrity (10 points)
  F = ComputeBrick Specific (5 points)

GATEWAY RULE: If A < 15, Total Score = 0 (Lakatos, 1978) [14]
```

### 3.2 Category A: Falsifiability & Testability (25 points)

Derived from Popper's "Logic of Scientific Discovery" [1]:

| Criterion | Points | Description | Verification |
|-----------|--------|-------------|--------------|
| A.1 Bounds Safety | 5 | All array accesses bounds-checked | Generated WGSL inspection |
| A.2 Barrier Reachability | 5 | All barriers reachable from all threads | CFG analysis |
| A.3 Brick Verification | 5 | `can_render()` returns true | Runtime check |
| A.4 Tensor Shape Consistency | 5 | Input/output shapes validated | Type checking |
| A.5 Probar Test Coverage | 5 | ≥80% GUI coverage via probar | Coverage report |

**Toyota Way Alignment**: Criterion A embodies *Jidoka*—building quality in at the source [2].

### 3.3 Category B: Reproducibility Infrastructure (25 points)

Based on reproducibility crisis research (Baker, 2016) [15]:

| Criterion | Points | Description | Verification |
|-----------|--------|-------------|--------------|
| B.1 Deterministic WGSL | 8 | Generated shaders byte-identical across runs | Hash comparison |
| B.2 wgpu Version Lock | 5 | Exact wgpu/naga versions in Cargo.lock | Dependency check |
| B.3 GPU Capability Check | 5 | Feature detection before shader compilation | Runtime guard |
| B.4 Benchmark Harness | 4 | Criterion-style statistical benchmarking | Benchmark presence |
| B.5 CI/CD GPU Testing | 3 | Automated testing on GPU hardware | CI config check |

### 3.4 Category C: Transparency & Openness (20 points)

Following Open Science principles (Nosek et al., 2015) [16]:

| Criterion | Points | Description | Verification |
|-----------|--------|-------------|--------------|
| C.1 WGSL Inspection | 6 | Generated WGSL logged/accessible | `to_wgsl()` output |
| C.2 Binding Layout | 5 | Bind group layout explicitly documented | Rust bindings |
| C.3 Workgroup Calculation | 5 | Dispatch size derivation visible | `to_dispatch_js()` |
| C.4 Memory Budget | 4 | Shared memory usage computed | `BrickBudget` |

### 3.5 Category D: Statistical Rigor (15 points)

Based on ASA Statement on P-Values (Wasserstein & Lazar, 2016) [17]:

| Criterion | Points | Description | Verification |
|-----------|--------|-------------|--------------|
| D.1 Warmup Iterations | 4 | ≥3 warmup iterations before measurement | Benchmark config |
| D.2 Sample Count | 4 | ≥10 samples for throughput metrics | Showcase runner |
| D.3 Outlier Detection | 4 | 2σ outlier threshold documented | `BenchmarkStats` |
| D.4 Confidence Intervals | 3 | 95% CI on all reported metrics | `[CI_low, CI_high]` |

### 3.6 Category E: Historical Integrity (10 points)

Inspired by Toyota's *Hansei* (reflection) practice [2]:

| Criterion | Points | Description | Verification |
|-----------|--------|-------------|--------------|
| E.1 Defect Lineage | 4 | CB-XXX ticket references in code | Comment search |
| E.2 Regression Tests | 3 | Tests derived from historical bugs | Test naming |
| E.3 5-Why Documentation | 3 | Root cause analysis for P0 defects | Doc presence |

### 3.7 Category F: ComputeBrick Specific (5 points)

| Criterion | Points | Description | Verification |
|-----------|--------|-------------|--------------|
| F.1 Brick Trait Impl | 2 | Correct `Brick` trait implementation | Type check |
| F.2 TileStrategy Valid | 2 | Tile dimensions match tensor shapes | Constraint check |
| F.3 ElementwiseOp Safe | 1 | All ops have WGSL equivalents | Exhaustive match |

## 4. Detection Algorithms

### 4.1 Bounds Check Detection (CB-001)

The WGPU_NO_BOUNDS_CHECK detector identifies shaders using `global_invocation_id` without bounds validation:

```rust
// UNSAFE PATTERN (CB-001):
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let gid = global_id.x;
    output[gid] = input[gid];  // No bounds check!
}

// SAFE PATTERN:
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let gid = global_id.x;
    if (gid >= arrayLength(&input)) { return; }  // Bounds check
    output[gid] = input[gid];
}
```

**Detection Algorithm**:
1. Parse WGSL AST for `global_invocation_id` usage
2. Trace data flow to array index expressions
3. Check for dominating bounds guard (`if gid >= length { return }`)
4. Flag as CB-001 if guard absent

### 4.2 Barrier Divergence Detection (CB-002)

Based on PARITY-114 pattern from CUDA-SIMD TDG:

```rust
// UNSAFE PATTERN (CB-002):
fn main() {
    if (local_id.x == 0u) {
        shared_data[0] = compute();
    }
    workgroupBarrier();  // Thread 0 may not reach here!
}

// SAFE PATTERN:
fn main() {
    if (local_id.x == 0u) {
        shared_data[0] = compute();
    }
    workgroupBarrier();  // All threads reach barrier
    // Use shared_data[0]...
}
```

**Detection Algorithm**:
1. Build control flow graph of WGSL shader
2. Identify `workgroupBarrier()` call sites
3. For each barrier, compute dominating conditions
4. If any thread ID condition dominates, flag as CB-002

### 4.3 Tile Dimension Validation (CB-003)

```rust
// ComputeBrick validation:
impl ComputeBrick {
    fn validate_tiles(&self) -> Result<(), TileError> {
        for op in &self.operations {
            if let TileOp::LoadShared { src, tile_size } = op {
                let tensor = self.find_tensor(src)?;
                if tile_size.0 * tile_size.1 > tensor.element_count() {
                    return Err(TileError::DimensionMismatch {
                        tile: tile_size,
                        tensor: tensor.shape.clone(),
                    });
                }
            }
        }
        Ok(())
    }
}
```

## 5. pmat comply Integration

### 5.1 Compliance Enforcement

The `pmat comply` command integrates ComputeBrick diagnostics:

```bash
# Check ComputeBrick compliance
pmat comply check --compute-brick

# Enforce via git hooks
pmat comply enforce --compute-brick

# Generate compliance report
pmat comply report --compute-brick --format markdown
```

### 5.2 Compliance Rules

| Rule ID | Description | Enforcement | Block Level |
|---------|-------------|-------------|-------------|
| CB-COMPLY-001 | All ComputeBricks must pass `verify()` | Pre-commit | Hard |
| CB-COMPLY-002 | Generated WGSL must include bounds checks | Pre-push | Hard |
| CB-COMPLY-003 | Workgroup size ≤1024 total threads | Pre-commit | Hard |
| CB-COMPLY-004 | probar GUI coverage ≥80% | Pre-push | Soft |
| CB-COMPLY-005 | No P0 defects in cuda-tdg analysis | Pre-push | Hard |

### 5.3 .pmat-gates.toml Configuration

```toml
[compute-brick]
enabled = true
min_score = 70
block_on_p0 = true
require_probar_coverage = 80

[compute-brick.checks]
bounds_check = "hard"      # Block on CB-001
barrier_safety = "hard"    # Block on CB-002
tile_validation = "hard"   # Block on CB-003
workgroup_limit = "hard"   # Block on CB-004
vectorization = "soft"     # Warn on CB-012

[compute-brick.probar]
require_gui_coverage = true
min_coverage = 80
playbook_validation = true
mutation_testing = false   # Optional M1-M5 falsification
```

### 5.4 Git Hook Integration

```bash
#!/bin/bash
# .git/hooks/pre-push (installed by pmat comply enforce)

set -e

echo "Running ComputeBrick compliance checks..."

# Run cuda-tdg analysis on WGSL-generating code
pmat cuda-tdg --wgpu gate . --min-score 70 --fail-on-p0

# Check probar GUI coverage
if command -v probador &> /dev/null; then
    probador playbook --validate --min-coverage 80
fi

echo "ComputeBrick compliance: PASSED"
```

## 6. Probar Testing Enforcement

### 6.1 Mandatory Test Patterns

Following the "tests define interface" principle (PROBAR-SPEC-009):

```rust
// Every ComputeBrick MUST have:
#[cfg(test)]
mod tests {
    use super::*;
    use jugar_probar::prelude::*;

    #[test]
    fn test_brick_verification() {
        let brick = MyComputeBrick::new(/* ... */);
        assert!(brick.can_render(), "Brick verification failed");
    }

    #[test]
    fn test_bounds_safety() {
        let brick = MyComputeBrick::new(/* ... */);
        let wgsl = brick.to_wgsl();
        assert!(
            wgsl.contains("if (gid >= arrayLength"),
            "CB-001: Missing bounds check in generated WGSL"
        );
    }

    #[test]
    fn test_workgroup_limits() {
        let brick = MyComputeBrick::new(/* ... */);
        let (x, y, z) = brick.get_workgroup_size();
        assert!(
            x * y * z <= 1024,
            "CB-004: Workgroup size {} exceeds 1024", x * y * z
        );
    }
}
```

### 6.2 GUI Coverage Requirements

```rust
use jugar_probar::gui_coverage;

let mut gui = gui_coverage! {
    compute_bricks: ["mel-filterbank", "fft", "attention"],
    shaders: ["log-transform.wgsl", "softmax.wgsl"]
};

// Record shader generation during tests
gui.generate("mel-filterbank");
gui.compile("log-transform.wgsl");

assert!(gui.meets(80.0), "GUI coverage below 80%: {}", gui.summary());
```

## 7. Kaizen Continuous Improvement

### 7.1 Defect Tracking

All ComputeBrick defects follow the CB-XXX naming convention:

```
CB-001: WGPU_NO_BOUNDS_CHECK (discovered 2026-01-09)
  Root Cause: ComputeBrick::to_wgsl() generates gid without guard
  5-Why Analysis:
    1. Why no bounds check? Not in generation template
    2. Why not in template? Original design assumed fixed sizes
    3. Why fixed sizes? Simplicity over safety
    4. Why prioritize simplicity? Rapid prototyping phase
    5. Why no review? Missing Poka-Yoke gate
  Resolution: Add bounds check generation in to_wgsl()
  Regression Test: test_bounds_safety()
```

### 7.2 Score Improvement Tracking

```bash
# Generate Kaizen report
pmat cuda-tdg kaizen --compute-brick

# Output:
# ComputeBrick Kaizen Report
# ══════════════════════════════════════════════════════════════
# Current Score: 58.5/100 (Grade: D)
# Target Score:  85.0/100 (Grade: A)
# Gap: 26.5 points
#
# Improvement Opportunities:
# ┌────────────┬────────┬─────────────────────────────────────┐
# │ Category   │ Points │ Action                              │
# ├────────────┼────────┼─────────────────────────────────────┤
# │ A.1 Bounds │ +5     │ Add bounds check to to_wgsl()       │
# │ B.4 Bench  │ +4     │ Add criterion benchmarks            │
# │ C.1 WGSL   │ +6     │ Log generated shaders               │
# │ E.2 Regr   │ +3     │ Add regression tests for CB-001     │
# └────────────┴────────┴─────────────────────────────────────┘
```

## 8. References

[1] Popper, K. R. (1959). *The Logic of Scientific Discovery*. Hutchinson. ISBN 978-0-415-27844-7.

[2] Liker, J. K. (2004). *The Toyota Way: 14 Management Principles from the World's Greatest Manufacturer*. McGraw-Hill. ISBN 978-0-07-139231-0.

[3] Nickolls, J., Buck, I., Garland, M., & Skadron, K. (2008). Scalable parallel programming with CUDA. *ACM Queue*, 6(2), 40-53. https://doi.org/10.1145/1365490.1365500

[4] Habermaier, A., & Knapp, A. (2012). On the correctness of the SIMT execution model of GPUs. *European Symposium on Programming (ESOP)*, 316-335. https://doi.org/10.1007/978-3-642-28869-2_16

[5] Ragan-Kelley, J., Barnes, C., Adams, A., Paris, S., Durand, F., & Amarasinghe, S. (2013). Halide: A language and compiler for optimizing parallelism, locality, and recomputation in image processing pipelines. *ACM SIGPLAN Notices*, 48(6), 519-530. https://doi.org/10.1145/2491956.2462176

[6] NVIDIA Corporation. (2024). *CUDA C++ Programming Guide, v12.3*. Section 5.3: Shared Memory. https://docs.nvidia.com/cuda/cuda-c-programming-guide/

[7] Volkov, V. (2010). Better performance at lower occupancy. *GPU Technology Conference (GTC)*. https://www.nvidia.com/content/GTC-2010/pdfs/2238_GTC2010.pdf

[8] Sorensen, T., Evrard, H., & Donaldson, A. F. (2021). GPU schedulers: How fair is fair enough? *Proceedings of the 26th ACM SIGPLAN Symposium on Principles and Practice of Parallel Programming (PPoPP)*, 344-358. https://doi.org/10.1145/3437801.3441603

[9] Intel Corporation. (2023). *Intel Intrinsics Guide*. https://www.intel.com/content/www/us/en/docs/intrinsics-guide/

[10] Raihan, M. A., Goli, N., & Aamodt, T. M. (2019). Modeling deep learning accelerator enabled GPUs. *2019 IEEE International Symposium on Performance Analysis of Systems and Software (ISPASS)*, 79-92. https://doi.org/10.1109/ISPASS.2019.00016

[11] Klabnik, S., & Nichols, C. (2023). *The Rust Programming Language*. Chapter 19: Unsafe Rust. No Starch Press. ISBN 978-1-7185-0310-6.

[12] Rust RFC 2045: Target feature. https://rust-lang.github.io/rfcs/2045-target-feature.html

[13] Ohno, T. (1988). *Toyota Production System: Beyond Large-Scale Production*. Productivity Press. ISBN 978-0-915299-14-0. (Section on Muda - waste elimination)

[14] Lakatos, I. (1978). *The Methodology of Scientific Research Programmes*. Cambridge University Press. ISBN 978-0-521-28031-0.

[15] Baker, M. (2016). 1,500 scientists lift the lid on reproducibility. *Nature*, 533(7604), 452-454. https://doi.org/10.1038/533452a

[16] Nosek, B. A., et al. (2015). Promoting an open research culture. *Science*, 348(6242), 1422-1425. https://doi.org/10.1126/science.aab2374

[17] Wasserstein, R. L., & Lazar, N. A. (2016). The ASA statement on p-values: Context, process, and purpose. *The American Statistician*, 70(2), 129-133. https://doi.org/10.1080/00031305.2016.1154108

## Appendix A: Grade Thresholds

| Grade | Score Range | Description | Action |
|-------|-------------|-------------|--------|
| A | 85-100 | Excellent | Merge allowed |
| B | 70-84 | Good | Merge with review |
| C | 55-69 | Acceptable | Improvement required |
| D | 40-54 | Poor | Block merge |
| F | 0-39 | Failing | Immediate remediation |

## Appendix B: Integration Checklist

- [ ] ComputeBrick implements `Brick` trait
- [ ] `verify()` called before `to_wgsl()`
- [ ] Generated WGSL includes bounds checks
- [ ] Workgroup size ≤1024
- [ ] probar tests cover all bricks
- [ ] GUI coverage ≥80%
- [ ] No P0 defects in cuda-tdg analysis
- [ ] Criterion benchmarks present
- [ ] CB-XXX tickets referenced for known issues
