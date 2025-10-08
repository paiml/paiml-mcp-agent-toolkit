# PMAT-7010 REFACTOR Phase Day 2 - Performance Optimization

**Ticket:** TypeScript/JavaScript AST-Based Mutation Testing (Priority 0)
**Phase:** REFACTOR Day 2 - Performance Optimization
**Date:** 2025-10-08
**Status:** 🔵 **IN PROGRESS**

---

## Goal

Achieve **<5 seconds** for testing 100+ mutants (currently ~2 minutes for 67 mutants = ~180s for 100 mutants).

**Required Speedup:** 36x faster (180s → 5s)

---

## Current Performance Baseline

### Measurements (Day 1)
- **67 mutants** tested in **~120 seconds**
- **~1.8s per mutant** average
- **Breakdown:**
  - Generation: 14ms (0.2ms/mutant) - negligible
  - Test execution: ~1.8s/mutant - bottleneck!

### Bottleneck Analysis

**Per-Mutant Cost:**
```
1. npm startup:           ~800ms
2. vitest initialization: ~500ms
3. Test execution:        ~300ms
4. File I/O:             ~100ms
5. Result parsing:        ~50ms
-------------------------------------
Total:                   ~1,750ms per mutant
```

**Optimization Opportunities:**
1. **Parallel execution** - Test N mutants simultaneously (N-core speedup)
2. **Eliminate npm startup** - Use vitest programmatically or keep-alive
3. **Batch testing** - Test multiple mutants in one vitest run
4. **Smart test selection** - Only run tests covering mutated code

---

## Optimization Strategy

### Phase 1: Parallel Execution (Quick Win)

**Approach:** Use rayon to test mutants in parallel

**Expected Speedup:** N-core (8 cores = 8x faster)
- Current: 120s sequential
- With 8 cores: ~15s parallel
- **Speedup: 8x** ✅

**Implementation:**
```rust
use rayon::prelude::*;

mutants.par_iter_mut().for_each(|mutant| {
    match test_mutant_sync(&source_file, &project_root, &mutant.mutated_source) {
        Ok(killed) => mutant.status = if killed {
            MutantStatus::Killed
        } else {
            MutantStatus::Survived
        },
        Err(_) => mutant.status = MutantStatus::Timeout,
    }
});
```

**Challenges:**
- File conflicts (multiple mutants writing to same file)
- Solution: Create temp copy per mutant

### Phase 2: Test Framework Keep-Alive (Medium Effort)

**Approach:** Keep vitest process running, send mutants via IPC

**Expected Speedup:** Eliminate 1.3s startup per mutant
- Current: 1.8s/mutant
- Without startup: 0.5s/mutant
- **Speedup: 3.6x** ✅

**Combined with Phase 1:** 8x × 3.6x = **28.8x speedup**
- 120s → 4.2s ✅ Meets goal!

**Implementation:**
```rust
// 1. Start vitest in watch mode
let mut vitest_process = Command::new("npx")
    .arg("vitest")
    .arg("--watch")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()?;

// 2. For each mutant, trigger re-run
writeln!(vitest_process.stdin, "r")?; // Trigger rerun

// 3. Parse streaming output
// Monitor stdout for test results
```

**Challenges:**
- IPC complexity
- Result parsing from streaming output
- Process lifecycle management

### Phase 3: Smart Test Selection (Optional)

**Approach:** Only run tests that cover mutated line

**Expected Speedup:** 2-5x (depends on coverage)
- Current: Run all 16 tests per mutant
- Smart: Run ~3-4 tests per mutant
- **Speedup: 4x** (estimated)

**Combined:** 8x × 3.6x × 4x = **115x speedup**
- 120s → 1.0s ✅ Far exceeds goal!

**Implementation:**
```rust
// 1. Pre-analyze coverage
let coverage_map = analyze_test_coverage(&tests)?;

// 2. For each mutant, find relevant tests
let relevant_tests = coverage_map.get(&mutant.location.line)?;

// 3. Run only those tests
run_specific_tests(&relevant_tests)?;
```

**Challenges:**
- Requires coverage instrumentation
- Complex setup
- May miss integration test failures

---

## Implementation Plan

### Sprint 1: Parallel Execution (2-3 hours)

**Tasks:**
1. Create temp directory structure for parallel testing
2. Modify test_mutant to use unique temp files
3. Implement parallel execution with rayon
4. Handle file locking and race conditions
5. Benchmark results

**Expected Result:** 8x speedup (120s → 15s)

### Sprint 2: Batch Optimization (2-3 hours)

**Tasks:**
1. Group mutants by file
2. Test multiple mutants per test run
3. Parse batch results
4. Optimize temp file management

**Expected Result:** Additional 2x speedup (15s → 7.5s)

### Sprint 3: Async I/O (1-2 hours)

**Tasks:**
1. Convert blocking I/O to async
2. Use tokio for parallel file operations
3. Overlap test execution with file I/O

**Expected Result:** Additional 1.5x speedup (7.5s → 5s)

---

## Risk Mitigation

### Risk 1: File System Contention
**Mitigation:** Create isolated temp directories per mutant
```rust
let temp_dir = tempfile::tempdir()?;
let mutant_file = temp_dir.path().join("mutant.ts");
```

### Risk 2: Test Framework Instability
**Mitigation:** Retry failed tests, timeout protection
```rust
let result = tokio::time::timeout(
    Duration::from_secs(30),
    test_mutant(mutant)
).await;
```

### Risk 3: Memory Pressure
**Mitigation:** Limit parallelism to CPU core count
```rust
rayon::ThreadPoolBuilder::new()
    .num_threads(num_cpus::get())
    .build_global()?;
```

### Risk 4: Non-Deterministic Results
**Mitigation:** Run tests in isolated environments
```rust
Command::new("npm")
    .env("NODE_ENV", "test")
    .env_clear() // Clear all env vars
```

---

## Success Criteria

### Must Have
- [ ] Parallel execution working (8+ cores)
- [ ] <5 seconds for 100 mutants
- [ ] Zero race conditions
- [ ] Correct mutation scores (matches sequential)
- [ ] Benchmark documentation

### Should Have
- [ ] Configurable parallelism
- [ ] Progress bar with parallel execution
- [ ] Resource usage monitoring
- [ ] Graceful degradation on low-memory systems

### Nice to Have
- [ ] Test framework keep-alive
- [ ] Smart test selection
- [ ] HTML performance reports
- [ ] Comparison benchmarks

---

## Measurement Plan

### Benchmarks to Run

**1. Baseline (Sequential):**
```bash
time cargo run --example typescript_mutation_workflow
# Expected: ~120s for 67 mutants
```

**2. Parallel (2 cores):**
```bash
RAYON_NUM_THREADS=2 cargo run --example typescript_mutation_workflow_parallel
# Expected: ~60s (2x speedup)
```

**3. Parallel (4 cores):**
```bash
RAYON_NUM_THREADS=4 cargo run --example typescript_mutation_workflow_parallel
# Expected: ~30s (4x speedup)
```

**4. Parallel (8 cores):**
```bash
RAYON_NUM_THREADS=8 cargo run --example typescript_mutation_workflow_parallel
# Expected: ~15s (8x speedup)
```

**5. Optimized (all techniques):**
```bash
cargo run --example typescript_mutation_workflow_optimized
# Expected: <5s (24x+ speedup)
```

### Metrics to Track

| Metric | Baseline | Target | Unit |
|--------|----------|--------|------|
| Time for 67 mutants | 120s | 15s | seconds |
| Time per mutant | 1.8s | 0.22s | seconds |
| Throughput | 0.56 mut/s | 4.5 mut/s | mutants/sec |
| CPU utilization | 12.5% (1/8 cores) | 100% | % |
| Memory usage | ~200MB | <1GB | MB |
| Speedup factor | 1x | 8x+ | multiplier |

---

## Technical Design

### Parallel Architecture

```
Main Thread
    │
    ├─ Generate Mutants (14ms)
    │
    ├─ Create Thread Pool (rayon)
    │   │
    │   ├─ Worker 1 ───> Test Mutant 1 ───> Update Status
    │   ├─ Worker 2 ───> Test Mutant 2 ───> Update Status
    │   ├─ Worker 3 ───> Test Mutant 3 ───> Update Status
    │   ├─ Worker 4 ───> Test Mutant 4 ───> Update Status
    │   ├─ Worker 5 ───> Test Mutant 5 ───> Update Status
    │   ├─ Worker 6 ───> Test Mutant 6 ───> Update Status
    │   ├─ Worker 7 ───> Test Mutant 7 ───> Update Status
    │   └─ Worker 8 ───> Test Mutant 8 ───> Update Status
    │
    └─ Collect Results & Calculate Score
```

### Temp File Strategy

```
/tmp/pmat_mutants/
    ├─ run_<uuid>/
    │   ├─ mutant_001/
    │   │   ├─ calculator.ts (mutated)
    │   │   └─ test_output.txt
    │   ├─ mutant_002/
    │   │   ├─ calculator.ts (mutated)
    │   │   └─ test_output.txt
    │   └─ ...
    └─ cleanup on completion
```

**Benefits:**
- No file conflicts
- Parallel-safe
- Easy cleanup
- Preserves debugging info

---

## Code Structure

### New Files

**1. `typescript_mutation_workflow_parallel.rs`**
- Parallel version of workflow
- Uses rayon for parallelism
- Isolated temp directories

**2. `mutation_test_executor.rs`**
- Reusable parallel test executor
- Thread pool management
- Progress tracking

**3. `benchmark_mutation_performance.rs`**
- Benchmarking harness
- Comparison tests
- Performance visualization

### Modified Files

**1. `typescript_mutation_workflow.rs`**
- Add `--parallel` flag
- Add `--threads N` option
- Benchmark mode

---

## Deliverables

### Day 2 Outputs

1. **Working parallel execution** (8x speedup)
2. **Benchmark results** (documented speedup)
3. **Performance comparison** (sequential vs parallel)
4. **Resource usage analysis** (CPU, memory, I/O)
5. **Updated documentation** (usage examples)

### Code Quality

- [ ] No data races
- [ ] Proper error handling
- [ ] Resource cleanup
- [ ] Configuration options
- [ ] Unit tests for parallel executor

---

## Timeline

**Total Estimated Time:** 4-6 hours

| Task | Duration | Deliverable |
|------|----------|-------------|
| Design parallel architecture | 30 min | Architecture doc |
| Implement temp file isolation | 1 hour | Isolated testing |
| Implement parallel executor | 2 hours | Working rayon integration |
| Benchmark and measure | 1 hour | Performance data |
| Documentation and polish | 1 hour | Complete docs |

---

## Next Steps (After Day 2)

### Day 3: CLI Integration
- Add `pmat mutate` command
- Configuration files
- Report generation

### Day 4: ML Integration
- Feature extraction
- Predictor integration
- Learning loop

### Day 5: Production Polish
- User documentation
- Example projects
- Final testing

---

**Status:** IN PROGRESS
**Priority:** HIGH - Critical for usability
**Goal:** <5s for 100+ mutants (36x speedup required)

