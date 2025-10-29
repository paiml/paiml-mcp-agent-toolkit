# Sprint 70 Phase 2: JSON Format Mismatch Issue

**Date**: October 29, 2025
**Discovered During**: Session 3 manual validation testing
**Severity**: HIGH - Blocks Phase 3 functionality
**Status**: NEEDS FIX

---

## Issue Summary

The cargo-mutants JSON output format in v25.3.1 does NOT match the format we designed Phase 2 around. Our parser expects a simple format with outcomes included, but cargo-mutants actually outputs two separate JSON files with a different structure.

---

## Expected Format (What We Built)

**Phase 2 Parser Expectation** (`CargoMutantsReport`):
```json
{
  "mutants": [
    {
      "outcome": "caught",
      "file": "src/lib.rs",
      "function": "add",
      "line": 10,
      "replacement": "-"
    }
  ]
}
```

---

## Actual Format (What cargo-mutants v25.3.1 Produces)

### File 1: `mutants.out/mutants.json`
Lists all mutants WITHOUT outcomes:
```json
[
  {
    "package": "pmat-mutate-test",
    "file": "src/lib.rs",
    "function": {
      "function_name": "add",
      "return_type": "-> i32",
      "span": {
        "start": {"line": 1, "column": 1},
        "end": {"line": 3, "column": 2}
      }
    },
    "span": {
      "start": {"line": 2, "column": 7},
      "end": {"line": 2, "column": 8}
      },
    "replacement": "*",
    "genre": "BinaryOperator"
  }
]
```

### File 2: `mutants.out/outcomes.json`
Contains execution results:
```json
{
  "outcomes": [
    {
      "scenario": "Baseline",
      "summary": "Success",
      "log_path": "log/baseline.log",
      "diff_path": null,
      "phase_results": [...]
    },
    {
      "scenario": {
        "Mutant": {
          "mutant": { ... },
          "mutant_id": "src/lib.rs:2:7_0"
        }
      },
      "summary": "CaughtMutant",
      "log_path": "log/caught/...",
      "phase_results": [...]
    }
  ]
}
```

---

## Impact Analysis

### What Works
- ✅ Phase 1: CargoMutantsWrapper (detection, version, execution)
- ✅ Phase 3: CLI integration and command building
- ✅ All test infrastructure

### What's Broken
- ❌ Phase 2: JSON parser expects wrong format
- ❌ `CargoMutantsReport::from_json()` won't parse actual output
- ❌ End-to-end workflow (execute → parse → display)

### Error When Running
```bash
$ pmat mutate --target . --use-cargo-mutants --timeout 10
🧪 cargo-mutants Backend
✅ Detected: cargo-mutants 25.3.1
🔧 Executing: cargo mutants --output json --timeout 10
⏳ Running mutation tests...
ERROR: cargo-mutants execution failed:
```

**Root Cause**:
1. We're passing `--output json` but that flag means "output directory"
2. cargo-mutants doesn't output JSON to stdout
3. cargo-mutants writes two JSON files to `mutants.out/` directory
4. Our parser expects different JSON structure

---

## Required Fixes

### Fix 1: Update Phase 1 Wrapper Command Building

**Current (Wrong)**:
```rust
cmd.arg("--output").arg("json");
```

**Corrected**:
```rust
// Create temp directory for output
let output_dir = PathBuf::from("/tmp/pmat-cargo-mutants-output");
std::fs::create_dir_all(&output_dir)?;
cmd.arg("--output").arg(&output_dir);
```

### Fix 2: Update Phase 2 Parser

**New Structs Needed**:
```rust
// Parse mutants.json
#[derive(Deserialize)]
struct MutantDefinition {
    package: String,
    file: String,
    function: FunctionInfo,
    span: SpanInfo,
    replacement: String,
    genre: String,
}

#[derive(Deserialize)]
struct FunctionInfo {
    function_name: String,
    return_type: String,
    span: SpanInfo,
}

#[derive(Deserialize)]
struct SpanInfo {
    start: Position,
    end: Position,
}

#[derive(Deserialize)]
struct Position {
    line: usize,
    column: usize,
}

// Parse outcomes.json
#[derive(Deserialize)]
struct OutcomesFile {
    outcomes: Vec<Outcome>,
}

#[derive(Deserialize)]
#[serde(tag = "scenario")]
enum Outcome {
    Baseline {
        summary: String,
        phase_results: Vec<PhaseResult>,
    },
    Mutant {
        mutant: MutantRef,
        mutant_id: String,
        summary: String,  // "CaughtMutant", "MissedMutant", "Timeout", "Unviable"
        phase_results: Vec<PhaseResult>,
    },
}
```

**New Parsing Logic**:
```rust
pub fn from_output_dir(dir: &Path) -> Result<CargoMutantsReport> {
    // 1. Read mutants.json
    let mutants_json = std::fs::read_to_string(dir.join("mutants.json"))?;
    let mutants: Vec<MutantDefinition> = serde_json::from_str(&mutants_json)?;

    // 2. Read outcomes.json
    let outcomes_json = std::fs::read_to_string(dir.join("outcomes.json"))?;
    let outcomes: OutcomesFile = serde_json::from_str(&outcomes_json)?;

    // 3. Match mutants with outcomes
    let mut result_mutants = Vec::new();
    for outcome in outcomes.outcomes {
        if let Outcome::Mutant { mutant_id, summary, .. } = outcome {
            // Find corresponding mutant definition
            // Map summary to our MutantOutcome enum
            let outcome = match summary.as_str() {
                "CaughtMutant" => MutantOutcome::Caught,
                "MissedMutant" => MutantOutcome::Missed,
                "Timeout" => MutantOutcome::Timeout,
                _ => MutantOutcome::Unviable,
            };
            result_mutants.push(CargoMutant { outcome, ... });
        }
    }

    Ok(CargoMutantsReport { mutants: result_mutants })
}
```

### Fix 3: Update Phase 3 Backend Handler

**Change in `cargo_mutants_backend.rs`**:
```rust
pub fn execute(config: CargoMutantsConfig) -> Result<String> {
    // ... existing code ...

    // Execute cargo-mutants (writes to output dir)
    let output_result = cmd.output()?;

    // Read JSON from output directory
    let output_dir = config.path.join("mutants.out");
    let mutants_json = std::fs::read_to_string(output_dir.join("mutants.json"))?;
    let outcomes_json = std::fs::read_to_string(output_dir.join("outcomes.json"))?;

    // Return combined JSON or path to dir
    Ok(output_dir.to_string_lossy().to_string())
}
```

---

## Testing Commands

### Verify cargo-mutants Actual Behavior
```bash
cd /tmp/pmat-mutate-test
cargo mutants -o /tmp/mutants-test --timeout 10

# Check output files
ls /tmp/mutants-test/mutants.out/
# Should show: mutants.json, outcomes.json, lock.json, log/

# View JSON
cat /tmp/mutants-test/mutants.out/mutants.json
cat /tmp/mutants-test/mutants.out/outcomes.json
```

### Verify After Fix
```bash
cargo build --release
./target/release/pmat mutate --target /tmp/pmat-mutate-test --use-cargo-mutants --timeout 10
```

---

## Estimated Fix Time

**Phase 2 Parser Rewrite**: ~2-3 hours
- New struct definitions (30 min)
- Parsing logic for two files (60 min)
- Matching mutants with outcomes (30 min)
- Update tests (30 min)
- Testing and validation (30 min)

**Phase 3 Backend Update**: ~30 minutes
- Update command building
- Update output handling
- Update tests

**Total**: ~3-4 hours

---

## Root Cause Analysis

**Why This Happened**:
1. Phase 2 was designed based on *assumed* cargo-mutants JSON format
2. No actual cargo-mutants execution was done during Phase 2
3. Documentation/examples used simplified JSON structure
4. Manual validation only happened in Session 3

**Lessons Learned**:
- Always validate against actual tool output
- Run integration tests with real tools during development
- Check tool documentation for exact format specifications
- Add example output files to test fixtures

---

## Recommendation

### Option 1: Fix Now (Recommended if time permits)
- Rewrite Phase 2 parser for actual format
- Update Phase 3 backend
- Re-test end-to-end
- Update documentation

### Option 2: Document and Fix Later
- Create detailed issue (this document)
- Mark Phase 3 as "Needs Phase 2 Fix"
- Continue with other Sprint 70 phases
- Come back to fix in focused session

### Option 3: Alternative Approach
- Use cargo-mutants text output instead of JSON
- Parse text output with regex
- Simpler but less robust

---

## Status

**Current State**: Phase 3 code is complete but NOT FUNCTIONAL end-to-end due to JSON format mismatch.

**Next Steps**:
1. Decide on fix approach (Option 1 vs 2 vs 3)
2. Update Phase 2 if proceeding with fix
3. Re-test validation
4. Update all documentation

**Priority**: HIGH - This blocks Sprint 70 completion

---

## References

- cargo-mutants v25.3.1 documentation
- `mutants.out/mutants.json` - Actual output example
- `mutants.out/outcomes.json` - Actual output example
- Phase 2 completion report: `SPRINT-70-PHASE2-COMPLETION.md`
- Phase 3 completion report: `SPRINT-70-PHASE3-COMPLETION.md`
