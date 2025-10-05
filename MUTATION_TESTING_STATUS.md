# Mutation Testing Implementation Status

## v2.130.0 - Empirical Test Execution ✅

### What Was Implemented

✅ **MutantExecutor Module** (`server/src/services/mutation/executor.rs`)
- Executes `cargo test --lib` on each mutant
- Backup/restore mechanism for safe file mutations
- Timeout handling (default 10 minutes per mutant)
- Status classification: Killed, Survived, CompileError, Timeout, Equivalent
- Test failure extraction from cargo output
- Sequential execution to avoid file conflicts

✅ **CLI Integration** (`server/src/cli/handlers/mutation_handlers.rs`)
- Updated `pmat analyze mutate` to use real test execution
- Removed "simulation mode" warnings
- Real mutation score calculation from empirical results
- Detailed output with per-mutant status

✅ **MCP Tool Integration** (`server/src/mcp_integration/mutation_tools.rs`)
- Updated `mutation_test` MCP tool to use empirical execution
- Changed mode from "simulation" to "empirical"
- Returns actual test results and execution metrics

✅ **Documentation** (`docs/mutation-testing.md`)
- Updated to reflect empirical execution
- Added example output showing real execution
- Clarified that ML prediction is optional

✅ **Unit Tests**
- 4 tests in `executor::tests`
- Test parsing of compilation errors, test failures, and success cases
- All tests passing

### Architecture

```
User runs: pmat analyze mutate --path file.rs
    ↓
MutationEngine generates mutants from AST
    ↓
MutantExecutor for each mutant:
    1. Backup original file → file.rs.pmat_backup
    2. Write mutated source → file.rs
    3. Run: cargo test --lib --nocapture
    4. Parse output for test failures
    5. Restore original file
    6. Classify: Killed/Survived/CompileError/Timeout
    ↓
MutationScore calculates empirical score
    ↓
Report JSON/text output with breakdown
```

### Current Limitations

#### 1. Cannot Test PMAT on Itself ❌
**Problem**: Running mutation testing on PMAT's own source code creates circular dependency:
- Mutating `server/src/services/mutation/*.rs` breaks PMAT's mutation tests
- Running `cargo test` would test the mutated PMAT code, not the original
- Infinite recursion: PMAT testing PMAT testing PMAT...

**Impact**: Cannot benchmark PMAT vs cargo-mutants using PMAT's own codebase

**Solution**: Test on **external projects** (e.g., simple calculator lib, parser, etc.)

#### 2. Sequential Execution Only
**Current**: Mutants run sequentially to avoid file write conflicts
**Future**: Parallel execution with file locking or temp directories

#### 3. Single File Only
**Current**: `pmat analyze mutate --path file.rs` (one file)
**Future**: Directory support `--path src/` for recursive mutation

#### 4. No Integration with ML Prediction
**Current**: ML prediction code exists but not integrated with test execution
**Future**: Run ML prediction first, then validate predictions vs empirical results

### Comparison to cargo-mutants

| Feature | PMAT v2.130.0 | cargo-mutants v25.3.1 |
|---------|---------------|------------------------|
| Test Execution | ✅ Yes (empirical) | ✅ Yes (empirical) |
| Mutation Score | ✅ Real measurement | ✅ Real measurement |
| Multi-file | ❌ Single file only | ✅ Full project |
| Parallel Execution | ❌ Sequential only | ✅ Parallel |
| ML Prediction | ✅ Available (not integrated) | ❌ No ML |
| Distributed Execution | ✅ Architecture exists | ❌ Local only |
| WASM Support | ✅ Yes (.wasm, .wat) | ❌ Rust only |
| Operator Count | 6 (AOR, ROR, COR, UOR, CRR, SDL) | ~12 operators |

### Next Steps for Full Parity

1. **External Project Benchmark**
   - Create simple Rust library (e.g., math operations)
   - Run both PMAT and cargo-mutants
   - Compare: speed, accuracy, mutant count

2. **Directory Support**
   - Implement recursive file discovery
   - Aggregate results across multiple files
   - Handle workspace/multi-crate projects

3. **Parallel Execution**
   - Use temp directories for each mutant
   - Or implement file-level locking
   - Target: 4-8× speedup with worker pool

4. **ML Integration with Validation**
   - Predict survivability before execution
   - Execute tests to validate predictions
   - Report ML accuracy metrics
   - Use ML to prioritize high-value mutants

5. **Location Metadata Fix** (GitHub Issue #63)
   - Currently all mutants show `line: 0, column: 0`
   - Need to extract actual line/column from AST
   - Required for "MISSED" output like cargo-mutants

### Verification Approach

Since we cannot test PMAT on itself, verification strategy:

1. **Create External Test Project**
   ```bash
   cargo new mutation_test_target --lib
   cd mutation_test_target
   # Add simple functions with tests
   ```

2. **Run Both Tools**
   ```bash
   pmat analyze mutate --path src/lib.rs
   cargo mutants --file src/lib.rs
   ```

3. **Compare Results**
   - Mutation score should be within 5% (accounting for different operators)
   - PMAT should be faster (ML prioritization potential)
   - Both should identify same test gaps

### Success Criteria (GitHub Issue #63 Resolution)

- ✅ **Priority 1**: Test execution implemented
- ⏳ **Priority 2**: Directory support (future)
- ⏳ **Priority 3**: Location metadata (future)
- ⏳ **Priority 4**: ML validation (future)

**Status**: Priority 1 complete. Issue #63 can be updated with progress.

### Example Usage (v2.130.0)

```bash
# Generate and test mutants on a single file
pmat analyze mutate --path src/calculator.rs --operators AOR,ROR,COR

# Output:
# 🧬 Mutation Testing
# Path: src/calculator.rs
#
# 📝 Generating mutants...
# ✅ Generated 24 mutants
#
# 🧪 Running tests on mutants...
#   [1/24] Testing mutant AOR_add_to_sub...
#     ✅ Killed (234ms)
#   [2/24] Testing mutant ROR_eq_to_ne...
#     ❌ Survived (187ms)
#   ...
#
# ✅ Mutation testing complete!
#    Mutation score: 83.33%
#    20 mutants killed, 4 survived
```

### Conclusion

PMAT v2.130.0 now has **functional empirical mutation testing** that:
- ✅ Actually runs tests (no more "simulation mode")
- ✅ Measures real mutation scores
- ✅ Reports which tests caught which mutants
- ✅ Handles errors, timeouts, and compile failures

**What's missing for full cargo-mutants parity:**
- Directory support
- Parallel execution
- Better location metadata

**What PMAT has that cargo-mutants doesn't:**
- ML prediction capability (not yet integrated with execution)
- WASM mutation support
- Distributed execution architecture
- MCP integration for AI workflows

The foundation is solid. Next phase: external project benchmarking and performance optimization.
