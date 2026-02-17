# PMAT Safety & Quality Improvement Specification

**Status**: Active Implementation
**Issues**: #226, #227, #228, #229, #230
**Date**: 2026-02-17

## Executive Summary

Cross-project analysis of 7 batuta stack projects (3,719 commits over 2.5 months) revealed systemic gaps in pmat's quality-gate reporting. This specification addresses both the macro trends discovered through analysis and 6 specific user-reported issues.

## Part 1: Cross-Project Trend Analysis

### Data Source

| Project | Commits | Fixes | Fix Rate | Fix:Feature Ratio |
|---------|---------|-------|----------|-------------------|
| aprender | 1,205 | 270 | 22% | 1.38:1 |
| realizar | 953 | 48 | 5% | 0.91:1 |
| trueno | 534 | 116 | 22% | 2.23:1 |
| whisper.apr | 409 | 29 | 7% | 0.64:1 |
| entrenar | 361 | 43 | 12% | 0.77:1 |
| resolve-pipeline | 144 | 27 | 19% | 0.54:1 |
| rmedia | 113 | 10 | 9% | 0.33:1 |
| **TOTAL** | **3,719** | **543** | **15%** | **1.13:1** |

### Trend 1: Entropy Kaizen Creates a Refactoring Treadmill

**Evidence**: trueno had 163 refactor + 116 fix commits out of 534 total (52%). aprender had 131 refactors. The cycle: pmat flags entropy -> files split -> module visibility breaks -> coverage drops -> tests rewritten -> entropy flagged again.

**Gap**: No metric tracks "net productivity" or whether quality gates are generating more work than they prevent.

**Action**: Add productivity metrics (see Part 3, Section A).

### Trend 2: Format/Serialization Is the #1 Bug Domain

**Evidence**: ~40+ fixes across aprender (GGUF, SafeTensors, ONNX, MLX, APR format bugs), realizar (quantization format confusion), whisper.apr (weight format mismatches).

**Gap**: No serialization round-trip invariant analysis. Each format tested in isolation.

**Action**: Future work - `pmat analyze format-parity` (out of scope for this sprint).

### Trend 3: Numerical Safety Bugs Across the Stack

**Evidence**: 15+ fixes total:
- `ln()` of zero/negative (entrenar)
- NaN in beam search from all-suppressed logits (whisper.apr)
- Division by zero in length-normalization (entrenar)
- BT.709 coefficient errors (rmedia)
- Epsilon guard fixes (trueno CB-530)

**Gap**: `pmat --faults` detects `unwrap()`/`panic!()` but NOT unguarded numerical operations.

**Action**: Future work - add numerical safety to `--faults` annotations.

### Trend 4: GPU State Lifecycle Bugs

**Evidence**: realizar had stale CUDA graph state causing KV cache corruption, 5GB memory waste from dead weight caching, batched prefill producing degenerate output.

**Gap**: No GPU lifecycle analysis in pmat.

**Action**: Future work - `pmat analyze resource-lifecycle --gpu`.

### Trend 5: Quality Gate Overhead Is Substantial

**Evidence**: whisper.apr dedicates ~15% of all commits to satisfying pmat quality gates. entrenar spent 20+ commits on CB-xxx compliance fixes. trueno's kaizen treadmill consumes 52% of commits.

**Root Cause**: Quality gates lack explainability, configurability, and accurate metrics - leading to wasted effort.

**Action**: The 6 issues below directly address this trend.

### Trend 6: Config/YAML Serialization Impedance

**Evidence**: entrenar had 4 commits in one day bouncing between quoting YAML booleans and reverting because serde broke. The YAML spec treats `true`/`false`/`yes`/`no` as booleans.

**Gap**: No config-file hazard detection in pmat.

**Action**: Future work - `pmat analyze config-safety`.

### Trend 7: Undocumented External API Trial-and-Error

**Evidence**: resolve-pipeline had 6 DaVinci Resolve API fixes from undocumented constraints.

**Gap**: pmat could track external API churn (functions calling FFI/external APIs with high fix rates).

**Action**: Future work - annotate external API boundaries in call graph.

## Part 2: User-Reported Issues (Priority Order)

### Issue 1: JSON Output Mixes Progress Lines (#230)

**Problem**: `pmat quality-gate --format json` produces output with emoji progress lines mixed into the JSON, making it unparseable by `jq`.

**Root Cause**: `eprintln!()` calls in `quality_gate.rs` print progress to stderr, but some output goes to stdout via `println!()` before the JSON block.

**Fix**: When `--format json`, suppress all progress output to stderr and ensure stdout contains ONLY the JSON payload. Use a guard pattern:

```rust
let is_json = matches!(format, QualityGateOutputFormat::Json);
if !is_json {
    eprintln!("...");
}
```

**Files**: `src/cli/analysis_utilities/quality_gate.rs`

### Issue 2: Entropy Thresholds Not Configurable (#227)

**Problem**: `pmat quality-gate` entropy check uses hardcoded thresholds with no override mechanism in `.pmat-gates.toml`.

**Root Cause**: `min_entropy` parameter exists as a CLI arg but `.pmat-gates.toml` has no `[entropy]` section read by the quality gate handler.

**Fix**: Add `[entropy]` section to `.pmat-gates.toml` parsing:

```toml
[entropy]
min_pattern_diversity = 0.30  # default threshold
max_violations = 10           # max files below threshold
enabled = true
```

Wire `load_quality_gate_config()` to read these values and pass them to the entropy check.

**Files**: `src/cli/analysis_utilities/quality_gate.rs`, `.pmat-gates.toml`

### Issue 3: Entropy Has No Explainability (#226)

**Problem**: Entropy violations say "ResourceManagement pattern repeated 10 times" but don't identify which functions, what the pattern looks like, or how to consolidate.

**Root Cause**: `EntropyReport` contains `ActionableViolation` with `PatternSummary` that has `pattern_type` and `repetitions` but only `example_code` (a single snippet). No function-level breakdown or consolidation guide.

**Fix**: Enhance `ActionableViolation` to include:
1. List of matched function names per pattern
2. AST structure description of the repeated pattern
3. Concrete consolidation suggestion with estimated savings
4. Score breakdown showing what would improve the metric

Add `--explain` flag to `pmat quality-gate` and `pmat analyze entropy`.

**Files**: `src/entropy/entropy_calculator.rs`, `src/cli/analysis_utilities/quality_gate.rs`

### Issue 4: Coverage Reports Wrong Percentage (#228)

**Problem**: Quality-gate reports 75% when actual coverage is 97.7%.

**Hypothesis**: The 75% is likely a function-level metric (% of functions with >0% coverage) rather than line coverage. Alternatively, it may be reading a stale or different cache.

**Fix**:
1. Verify what metric the quality gate actually reads
2. Ensure it uses line coverage from `.pmat/coverage-cache.json`
3. When cache is stale (git_hash mismatch), clearly indicate staleness
4. Label the metric correctly (line vs function coverage)

**Files**: `src/cli/analysis_utilities/quality_gate.rs`, coverage check functions

### Issue 5: Provability Has No Explainability (#229)

**Problem**: Provability score (0.52) with no breakdown of what drives it. 228/274 functions score below 50% with no guidance.

**Root Cause**: `format_provability_json` includes `verified_properties` with `property_type`, `confidence`, `evidence` - but only when `include_evidence` is true. The quality-gate path doesn't use this.

**Fix**:
1. In quality-gate output, include top contributing factors
2. Add `--explain` support for provability showing per-function breakdown
3. Document what provability measures (formal verification amenability: unsafe, FFI, mutation, side effects, complexity)

**Files**: `src/cli/provability_helpers.rs`, `src/cli/analysis_utilities/quality_gate.rs`

### Issue 6: No Entropy/Provability SQL Tables

**Problem**: `pmat sql` cannot query entropy violations or provability details because no tables exist for them.

**Fix**: Add two new tables to the SQLite schema:

```sql
CREATE TABLE IF NOT EXISTS entropy_violations (
    id INTEGER PRIMARY KEY,
    file_path TEXT NOT NULL,
    pattern_type TEXT NOT NULL,
    pattern_hash TEXT NOT NULL,
    repetitions INTEGER NOT NULL,
    variation_score REAL NOT NULL,
    estimated_loc_reduction INTEGER NOT NULL,
    severity TEXT NOT NULL,
    example_code TEXT,
    UNIQUE(file_path, pattern_hash)
);

CREATE TABLE IF NOT EXISTS provability_scores (
    id INTEGER PRIMARY KEY,
    function_id INTEGER NOT NULL,
    file_path TEXT NOT NULL,
    function_name TEXT NOT NULL,
    provability_score REAL NOT NULL,
    unsafe_count INTEGER DEFAULT 0,
    ffi_count INTEGER DEFAULT 0,
    mutation_count INTEGER DEFAULT 0,
    side_effect_count INTEGER DEFAULT 0,
    complexity_factor REAL DEFAULT 0.0,
    verified_properties INTEGER DEFAULT 0,
    FOREIGN KEY (function_id) REFERENCES functions(id)
);

CREATE INDEX IF NOT EXISTS idx_entropy_file ON entropy_violations(file_path);
CREATE INDEX IF NOT EXISTS idx_entropy_severity ON entropy_violations(severity);
CREATE INDEX IF NOT EXISTS idx_provability_score ON provability_scores(provability_score);
CREATE INDEX IF NOT EXISTS idx_provability_file ON provability_scores(file_path);
```

**Files**: `src/services/agent_context/function_index/sqlite_backend.rs`

## Part 3: Future Work (From Cross-Project Analysis)

### A. Productivity Metrics

| Metric | Description |
|--------|-------------|
| Fix Velocity Index | fix commits / feature commits over rolling 30-day window |
| Kaizen ROI | Track which pmat findings produce the most secondary fix commits |
| Refactoring Treadmill Score | Files refactored 3+ times in 30 days |
| Net Feature Velocity | Feature commits - (fix + refactor) per week |

Command: `pmat analyze productivity --since 30d`

### B. Numerical Safety Analysis

| Check | What it catches |
|-------|----------------|
| Unguarded `ln()`/`log()` | `x.ln()` without `x > 0` guard |
| Unguarded division | `a / b` where `b` could be zero |
| Unguarded `sqrt()` | `x.sqrt()` where x could be negative |
| NaN propagation | Functions producing NaN passed downstream |

Command: `pmat analyze numerical-safety` or integrate into `--faults`

### C. Cross-Project Fix Hotspot Dashboard

```
pmat workspace hotspots --since 30d --projects ../trueno,../aprender,...
```

Shows ranked (project, module, bug-count, fix-rate, churn-score) across the stack.

### D. Config File Hazard Analysis

| Check | What it catches |
|-------|----------------|
| YAML truthy hazards | Unquoted `yes`/`no`/`on`/`off` |
| TOML type mismatches | String where serde expects bool/int |
| Serde compatibility | Fields needing `#[serde(deserialize_with)]` |

Command: `pmat analyze config-safety`

### E. Resource Lifecycle Analysis (GPU)

| Check | What it catches |
|-------|----------------|
| Acquire-without-release | `cudaMalloc` without `cudaFree` |
| Stale state hazard | Resources cached across boundaries |
| Over-allocation | Per-call buffers that could be preallocated |

Command: `pmat analyze resource-lifecycle --gpu`

## Implementation Priority

1. **#230 JSON output** - Simplest fix, highest immediate user impact
2. **#227 Entropy configurability** - Unblocks projects stuck on entropy gates
3. **#228 Coverage accuracy** - Trust in metrics is fundamental
4. **#226 Entropy explainability** - Reduces wasted refactoring effort
5. **#229 Provability explainability** - Reduces confusion
6. **SQL tables** - Enables programmatic querying
