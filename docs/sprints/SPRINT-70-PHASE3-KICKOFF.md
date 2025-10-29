# Sprint 70 - Phase 3 Kickoff: CLI Integration

**Task**: PMAT-070-003
**Goal**: Integrate cargo-mutants wrapper + JSON parser into `pmat mutate` CLI command
**Duration**: Day 5
**Status**: 🚀 READY TO START

---

## Context

**Prerequisites**:
- ✅ PMAT-070-001 complete (CargoMutantsWrapper infrastructure)
- ✅ PMAT-070-002 complete (JSON parser with outcome mapping)

**Problem**: Need a CLI command to execute cargo-mutants and display results

**Solution**: Create `pmat mutate` command that:
1. Detects cargo-mutants installation
2. Executes cargo-mutants with appropriate flags
3. Parses JSON output
4. Displays mutation score and statistics

---

## Requirements (from Spec)

### 1. CLI Command Structure

**Command**: `pmat mutate`

**Aliases**: `pmat mut`, `pmat m`

**Arguments**:
```bash
pmat mutate [OPTIONS]

Options:
  -p, --path <PATH>           Path to Rust project (default: current directory)
  -o, --output <FILE>         Save JSON output to file
  --timeout <SECONDS>         Timeout per test (default: 300)
  --jobs <N>                  Number of parallel jobs (default: num_cpus)
  --no-shuffle                Don't shuffle test execution order
  --features <FEATURES>       Cargo features to enable
  --all-features              Enable all features
  --no-default-features       Disable default features
  -v, --verbose               Show detailed output
  -h, --help                  Print help
```

### 2. Command Flow

```
1. Validate cargo-mutants installed (via CargoMutantsWrapper)
2. Validate minimum version (v24.7.0+)
3. Execute: cargo mutants --output json [OPTIONS]
4. Parse JSON output (via CargoMutantsReport)
5. Convert to PMAT format (via to_pmat_report)
6. Display statistics
7. Exit with appropriate code (0 if score ≥ threshold, 1 otherwise)
```

### 3. Output Format

**Console Output**:
```
🧪 Running mutation testing with cargo-mutants...

✅ cargo-mutants v25.3.1 detected
🔧 Executing: cargo mutants --output json --timeout 300

⏳ Running mutation tests... (this may take several minutes)

📊 Mutation Testing Results:
   Total mutants: 42
   Caught: 38 (90.5%)
   Missed: 3 (7.1%)
   Timeout: 1 (2.4%)
   Unviable: 0 (0.0%)

📈 Mutation Score: 90.5%
✅ Excellent! Test suite quality is very high

💾 Results saved to: mutation-report.json
```

**Error Handling**:
```
❌ cargo-mutants not found in PATH
   Install: cargo install cargo-mutants

❌ cargo-mutants version 24.6.0 is too old
   Minimum required: v24.7.0
   Upgrade: cargo install --force cargo-mutants

❌ Failed to parse mutation results
   Run with --verbose for details
```

### 4. Integration Points

**Files to Create/Modify**:
1. `server/src/cli/commands.rs` - Add `Mutate` variant to Commands enum
2. `server/src/cli/handlers/mutate_command_handler.rs` - NEW: Command handler
3. `server/src/cli/command_structure.rs` - Add mutate to UtilityCommandGroup
4. `server/tests/mutate_command_tests.rs` - NEW: Integration tests

---

## Tests to Write (RED Phase)

### Unit Tests

1. **test_mutate_command_requires_cargo_mutants**
   - Verify error when cargo-mutants not installed

2. **test_mutate_command_validates_version**
   - Verify version check (v24.7.0+)

3. **test_mutate_command_executes_successfully**
   - Mock cargo-mutants execution
   - Verify JSON parsing
   - Verify statistics display

4. **test_mutate_command_handles_timeout**
   - Verify timeout handling

5. **test_mutate_command_handles_parse_error**
   - Verify graceful error when JSON is malformed

6. **test_mutate_command_saves_output_file**
   - Verify --output flag saves JSON

7. **test_mutate_command_passes_flags_to_cargo_mutants**
   - Verify --features, --timeout, --jobs flags passed correctly

8. **test_mutate_command_displays_statistics**
   - Verify console output formatting

### Integration Tests

9. **integration_test_mutate_command_end_to_end**
   - Requires actual cargo-mutants installation
   - End-to-end workflow test

10. **integration_test_mutate_command_with_real_project**
    - Run against small test Rust project
    - Verify results are accurate

---

## Implementation Plan

### RED Phase (~45 min)

1. Create test file: `server/tests/mutate_command_tests.rs`
2. Write 10 tests with `unimplemented!()` in mock command handler
3. Add `Mutate` variant to Commands enum
4. Verify all tests fail
5. Commit: "test: PMAT-070-003 RED phase - Mutate command tests"

### GREEN Phase (~1.5 hours)

1. Create `server/src/cli/handlers/mutate_command_handler.rs`
2. Implement command handler:
   - Use CargoMutantsWrapper::new()
   - Execute cargo mutants via std::process::Command
   - Parse JSON output via CargoMutantsReport::from_json()
   - Display statistics
3. Add command to Commands enum
4. Wire up in command_structure.rs
5. Update mod.rs exports
6. Verify all tests pass (10/10 = 100%)
7. Commit: "feat: PMAT-070-003 GREEN phase - Mutate command implementation"

### REFACTOR Phase (~45 min)

1. Extract display_statistics() helper
2. Extract build_cargo_mutants_command() helper
3. Add error context for better debugging
4. Enhance documentation
5. Run cargo fmt
6. Verify tests still pass
7. Commit: "refactor: PMAT-070-003 REFACTOR phase - Code quality improvements"

### VERIFY Phase (~30 min)

1. Run clippy (zero warnings)
2. Run cargo fmt --check (all formatted)
3. Run integration test with actual cargo-mutants
4. Test CLI manually: `cargo run --bin pmat -- mutate --help`
5. Include metrics in commit

---

## Sample Implementation (Skeleton)

```rust
// server/src/cli/handlers/mutate_command_handler.rs

use crate::services::mutation::cargo_mutants_wrapper::CargoMutantsWrapper;
use crate::services::mutation::json_parser::CargoMutantsReport;
use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;

pub struct MutateCommandHandler;

impl MutateCommandHandler {
    pub fn execute(
        path: Option<PathBuf>,
        output: Option<PathBuf>,
        timeout: Option<u64>,
        jobs: Option<usize>,
        features: Option<Vec<String>>,
        verbose: bool,
    ) -> Result<()> {
        // 1. Validate cargo-mutants installation
        let wrapper = CargoMutantsWrapper::new()?;
        wrapper.validate_version()?;

        // 2. Build cargo mutants command
        let mut cmd = Command::new("cargo");
        cmd.arg("mutants");
        cmd.arg("--output").arg("json");

        if let Some(t) = timeout {
            cmd.arg("--timeout").arg(t.to_string());
        }

        if let Some(j) = jobs {
            cmd.arg("--jobs").arg(j.to_string());
        }

        // 3. Execute cargo mutants
        println!("🧪 Running mutation testing with cargo-mutants...");
        let output_result = cmd.output()?;

        // 4. Parse JSON
        let json = String::from_utf8(output_result.stdout)?;
        let report = CargoMutantsReport::from_json(&json)?;

        // 5. Display statistics
        Self::display_statistics(&report);

        // 6. Save output if requested
        if let Some(file) = output {
            std::fs::write(file, json)?;
        }

        Ok(())
    }

    fn display_statistics(report: &CargoMutantsReport) {
        println!("\n📊 Mutation Testing Results:");
        println!("   Total mutants: {}", report.mutants.len());
        println!("   Mutation Score: {:.1}%", report.mutation_score());
        // ... more statistics
    }
}
```

---

## Success Criteria

- ✅ `pmat mutate` command implemented
- ✅ All command-line flags working
- ✅ cargo-mutants detection and version validation
- ✅ JSON parsing and PMAT conversion
- ✅ Statistics display
- ✅ Error handling (installation, version, parsing)
- ✅ 100% test pass rate (10/10)
- ✅ Integration test passing
- ✅ Zero clippy warnings
- ✅ Code formatted

---

## Files to Create

1. `server/src/cli/handlers/mutate_command_handler.rs` - Command handler
2. `server/tests/mutate_command_tests.rs` - Test suite

---

## Files to Modify

1. `server/src/cli/commands.rs` - Add Mutate variant
2. `server/src/cli/command_structure.rs` - Wire up command
3. `server/src/cli/handlers/mod.rs` - Export handler
4. `server/src/cli/mod.rs` - Re-export if needed

---

## Dependencies

Already in Cargo.toml:
- ✅ `clap` (CLI argument parsing)
- ✅ `anyhow` (error handling)
- ✅ `serde` and `serde_json` (JSON parsing)

No new dependencies needed!

---

## Estimated Timeline

| Phase | Duration | Cumulative |
|-------|----------|------------|
| RED | 45 min | 45 min |
| GREEN | 1.5 hours | 2h 15min |
| REFACTOR | 45 min | 3h |
| VERIFY | 30 min | 3h 30min |
| **Total** | **~3.5 hours** | **~3.5 hours** |

---

## Next Session Action Plan

**Start here**:
```bash
cd /home/noah/src/paiml-mcp-agent-toolkit

# 1. Create test file (RED phase)
touch server/tests/mutate_command_tests.rs

# 2. Create handler file (skeleton)
touch server/src/cli/handlers/mutate_command_handler.rs

# 3. Write failing tests
# See "Tests to Write" section above

# 4. Verify tests fail
cargo test --test mutate_command_tests

# 5. Commit RED phase
git add -A && git commit -m "test: PMAT-070-003 RED phase"
```

---

## Related Files

**Reference implementations**:
- `server/src/services/mutation/cargo_mutants_wrapper.rs` - Wrapper API
- `server/src/services/mutation/json_parser.rs` - Parser API
- `server/src/cli/handlers/*_command_handler.rs` - Existing command handlers

**Test patterns**:
- `server/tests/cargo_mutants_wrapper_tests.rs` - Follow this structure
- `server/tests/json_parsing_tests.rs` - Follow this structure

---

## Questions to Answer During Implementation

1. Should mutation testing be part of quality gates (`pmat quality-gate`)?
2. Should we cache mutation results for incremental runs?
3. Should we support custom mutation operators alongside cargo-mutants?
4. Should we integrate with CI/CD via exit codes based on thresholds?

**Recommendation**: Start minimal (just `pmat mutate`), expand later if needed.

---

## Completion Criteria

Before marking PMAT-070-003 complete:

- ✅ All RED tests written and failing
- ✅ All GREEN tests passing (100%)
- ✅ REFACTOR: Code quality improved
- ✅ VERIFY: All quality gates passed
- ✅ Integration test working
- ✅ Manual CLI test successful
- ✅ Documentation complete
- ✅ Completion report written

---

**Status**: Ready for RED phase implementation in next session.

**Previous Phases**:
- PMAT-070-001 ✅ COMPLETE (Infrastructure)
- PMAT-070-002 ✅ COMPLETE (JSON Parsing)

**Current Phase**: PMAT-070-003 🚀 STARTING
**Next Phase**: PMAT-070-004 (Comprehensive Testing)
