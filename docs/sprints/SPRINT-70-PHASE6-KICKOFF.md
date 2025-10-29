# Sprint 70 Phase 6: Performance Validation - KICKOFF

**Phase**: 6/7
**Status**: Starting
**Date**: October 29, 2025
**Estimated Duration**: 2-3 hours

---

## Overview

Phase 6 focuses on validating that the cargo-mutants integration performs well at scale. We'll benchmark parsing performance, test with real-world projects, profile memory usage, and optimize if needed.

---

## Prerequisites (All Complete ✅)

- ✅ Phase 1: CargoMutantsWrapper implementation
- ✅ Phase 2: JSON parser (fixed for v25.3.1 format)
- ✅ Phase 3: CLI integration
- ✅ Phase 4: Comprehensive testing (10/10 tests passing)
- ✅ Phase 5: Documentation (user-ready)

---

## Goals

1. **Parsing Performance**: Verify parser handles large outputs efficiently
2. **Real-World Validation**: Test on actual Rust projects
3. **Memory Profiling**: Ensure memory usage is reasonable
4. **Scalability**: Validate parallel execution scales well
5. **Documentation**: Document performance characteristics

---

## Tasks Breakdown

### Task 1: Benchmark Parsing Performance (30-45 min)

**Goal**: Measure parser performance with varying output sizes

**Test Cases**:
1. **Small Output** (5-10 mutants)
   - Expected: <10ms parsing time
   - Use existing `some-missed` fixture

2. **Medium Output** (50-100 mutants)
   - Expected: <100ms parsing time
   - Create synthetic fixture or use real output

3. **Large Output** (500+ mutants)
   - Expected: <1s parsing time
   - Create synthetic fixture

4. **Very Large Output** (1000+ mutants)
   - Expected: <5s parsing time
   - Test scalability limits

**Metrics to Measure**:
- Parse time (from file read to `CargoMutantsReport` creation)
- Memory allocation
- CPU usage
- Scalability (time vs mutant count)

**Tools**:
- `std::time::Instant` for timing
- `cargo bench` for reproducible benchmarks
- Manual profiling with real cargo-mutants runs

**Deliverable**: Benchmark results document

---

### Task 2: Real-World Project Testing (45-60 min)

**Goal**: Validate integration on actual Rust projects

**Test Projects**:

1. **Small Project** (< 1000 LOC)
   - Example: Simple CLI tool, library
   - Expected: Fast execution, accurate results

2. **Medium Project** (1000-10000 LOC)
   - Example: PMAT itself (this project)
   - Expected: Reasonable execution time, comprehensive results

3. **Large Project** (> 10000 LOC)
   - Example: Well-known open-source Rust project
   - Expected: Handles complexity, doesn't crash

**Validation Criteria**:
- ✅ cargo-mutants detection works
- ✅ Version validation passes
- ✅ Execution completes successfully
- ✅ Parsing handles output correctly
- ✅ Statistics accurate
- ✅ No crashes or errors

**Test Commands**:
```bash
# Small project
cd /tmp/small-rust-project
pmat mutate --use-cargo-mutants --timeout 300

# Medium project (PMAT)
cd ~/src/paiml-mcp-agent-toolkit/server
pmat mutate --target src/cli --use-cargo-mutants --timeout 600

# Large project (if available)
cd ~/rust-analyzer  # Example
pmat mutate --target crates/ide --use-cargo-mutants --timeout 900
```

**Deliverable**: Real-world test results

---

### Task 3: Memory Profiling (30-45 min)

**Goal**: Ensure memory usage is reasonable

**Profiling Approach**:

1. **Baseline Memory**:
   - Measure PMAT memory before mutation testing
   - Expected: ~10-50 MB baseline

2. **During Parsing**:
   - Measure memory while parsing large outputs
   - Expected: Linear growth with mutant count

3. **Peak Memory**:
   - Identify peak memory usage
   - Expected: <500 MB for 1000 mutants

4. **Memory Leaks**:
   - Check for memory leaks (repeated runs)
   - Expected: No leaks, stable memory

**Tools**:
- `/usr/bin/time -v` for memory stats
- `heaptrack` (if available)
- `valgrind --tool=massif` (if needed)
- Manual observation with `htop`

**Test Scenarios**:
```bash
# Memory test with small output
/usr/bin/time -v pmat mutate --use-cargo-mutants --timeout 60

# Memory test with large output
/usr/bin/time -v pmat mutate --use-cargo-mutants --timeout 600
```

**Deliverable**: Memory profile report

---

### Task 4: Parallel Execution Scaling (30-45 min)

**Goal**: Validate parallel execution scales efficiently

**Test Matrix**:

| Jobs | Expected Speedup | Notes |
|------|------------------|-------|
| 1    | Baseline         | Sequential execution |
| 2    | ~1.8x            | Near-linear speedup |
| 4    | ~3.5x            | Good parallelism |
| 8    | ~6x              | Some overhead |

**Test Command**:
```bash
# Benchmark different job counts
for jobs in 1 2 4 8; do
  echo "Testing with $jobs jobs..."
  time pmat mutate --use-cargo-mutants --jobs $jobs --timeout 300
done
```

**Metrics**:
- Wall clock time
- CPU utilization
- Speedup ratio
- Efficiency (speedup / cores)

**Deliverable**: Scaling benchmark results

---

### Task 5: Performance Documentation (30 min)

**Goal**: Document performance characteristics

**Create**: `docs/performance/cargo-mutants-performance.md`

**Sections**:
1. **Overview**
   - Performance summary
   - Key metrics

2. **Parsing Performance**
   - Time vs mutant count graph
   - Memory usage analysis
   - Scalability limits

3. **Real-World Results**
   - Small/medium/large project timings
   - Accuracy validation
   - Known limitations

4. **Parallel Execution**
   - Scaling characteristics
   - Recommended job counts
   - CPU/memory tradeoffs

5. **Optimization Tips**
   - When to use `--jobs`
   - Timeout recommendations
   - Feature selection impact

6. **Benchmarks**
   - Reproducible benchmark commands
   - Expected results
   - Comparison with built-in mutation testing

**Deliverable**: Performance documentation

---

## Success Criteria

### Performance Requirements

**Parsing**:
- [ ] <100ms for 100 mutants
- [ ] <1s for 500 mutants
- [ ] Linear or sub-linear scaling

**Memory**:
- [ ] <200 MB for typical usage
- [ ] <500 MB for large projects
- [ ] No memory leaks

**Real-World**:
- [ ] Works on small projects
- [ ] Works on medium projects (PMAT)
- [ ] Gracefully handles large projects

**Parallel Execution**:
- [ ] 2x speedup with 2 jobs
- [ ] 3.5x+ speedup with 4 jobs
- [ ] No crashes with high parallelism

---

## Timeline

**Total Estimated Time**: 2-3 hours

| Task | Duration | Status |
|------|----------|--------|
| Task 1: Parsing Benchmarks | 30-45 min | Pending |
| Task 2: Real-World Testing | 45-60 min | Pending |
| Task 3: Memory Profiling | 30-45 min | Pending |
| Task 4: Parallel Scaling | 30-45 min | Pending |
| Task 5: Documentation | 30 min | Pending |

**Order of Execution**:
1. Task 1 (Parsing) - Baseline performance
2. Task 2 (Real-World) - Practical validation
3. Task 3 (Memory) - Resource usage
4. Task 4 (Parallel) - Scaling validation
5. Task 5 (Documentation) - Results summary

---

## Key Decisions

### Optimization Strategy
- **Measure First**: Don't optimize without data
- **Targeted Fixes**: Only optimize bottlenecks
- **Acceptable Thresholds**: Define "good enough" performance
- **Document Limitations**: Be honest about constraints

### Test Projects
- **Available Projects**: Use what's accessible
- **Representative**: Choose typical Rust projects
- **Diverse**: Test different sizes and complexities

### Profiling Depth
- **Lightweight First**: Start with simple timing
- **Deep Dive If Needed**: Use heaptrack/valgrind if issues found
- **Production Focus**: Prioritize real-world scenarios

---

## Quality Checks

### Before Declaring Success
1. **All Benchmarks Run**: Complete all test cases
2. **Results Documented**: Clear performance characteristics
3. **No Critical Issues**: No crashes, memory leaks, or slowdowns
4. **Real-World Validated**: Works on actual projects
5. **Scaling Confirmed**: Parallel execution provides benefit

### Optimization Triggers
Optimize if:
- Parsing >1s for 500 mutants
- Memory >500 MB for typical usage
- Parallel execution doesn't scale
- Real-world projects fail or crash

Don't optimize if:
- Performance meets requirements
- Issues are edge cases
- Optimization adds complexity

---

## Notes

- Focus on **real-world performance** over micro-optimizations
- Document **limitations honestly** (e.g., not suitable for monorepos)
- Provide **tuning guidance** for users
- Compare with **built-in mutation testing** where applicable

---

## References

**Cargo-mutants Performance**:
- https://mutants.rs/performance.html
- https://github.com/sourcefrog/cargo-mutants/discussions

**Rust Profiling**:
- `cargo bench` documentation
- `heaptrack` usage guide
- `perf` for Linux profiling

**Related**:
- Phase 5 completion: `docs/sprints/SPRINT-70-PHASE5-PARTIAL-COMPLETION.md`
- Phase 4 completion: `docs/sprints/SPRINT-70-PHASE4-COMPLETION.md`

---

## Next Steps After Phase 6

1. **If Performance Good**: Move to Phase 7 (Release Preparation)
2. **If Issues Found**: Optimize and re-validate
3. **If Blockers**: Document limitations and mitigations

---

**Status**: Ready to begin Task 1 (Parsing Benchmarks)
