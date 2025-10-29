# Sprint 70 - Phase 2 Kickoff: JSON Parsing

**Task**: PMAT-070-002
**Goal**: Parse cargo-mutants JSON output to PMAT format
**Duration**: Days 3-4
**Status**: 🚀 READY TO START

---

## Context

**Prerequisites**: ✅ PMAT-070-001 complete (CargoMutantsWrapper infrastructure)

**Problem**: cargo-mutants outputs JSON. We need to parse it and convert to PMAT's mutation report format.

**Solution**: Create parser with serde, map cargo-mutants outcomes to PMAT MutantStatus.

---

## Requirements (from Spec)

### 1. Data Structures

**CargoMutantsReport** - mirrors cargo-mutants JSON schema:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoMutantsReport {
    pub mutants: Vec<CargoMutant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoMutant {
    pub outcome: MutantOutcome,
    pub file: String,
    pub function: Option<String>,
    pub line: usize,
    // ... other fields from cargo-mutants
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MutantOutcome {
    Caught,
    Missed,
    Timeout,
    Unviable,
}
```

### 2. Conversion Logic

**Outcome Mapping**:
- `caught` → `MutantStatus::Killed`
- `missed` → `MutantStatus::Survived`
- `timeout` → `MutantStatus::Timeout`
- `unviable` → `MutantStatus::CompileError`

**Conversion Function**:
```rust
impl CargoMutantsReport {
    pub fn to_pmat_report(&self) -> Vec<Mutant> {
        // Convert each CargoMutant to PMAT Mutant
    }
}
```

### 3. Edge Cases

- ✅ Empty mutants list
- ✅ Invalid JSON (parse errors)
- ✅ Unknown outcomes (schema changes)
- ✅ Missing fields (graceful defaults)

---

## Tests to Write (RED Phase)

### Unit Tests

1. **test_parse_cargo_mutants_json_all_outcomes**
   - Parse JSON with all 4 outcomes
   - Verify correct deserialization

2. **test_parse_empty_mutants_list**
   - Parse JSON with empty `mutants: []`
   - Should succeed with empty Vec

3. **test_parse_invalid_json_returns_error**
   - Pass malformed JSON
   - Should return Err, not panic

4. **test_convert_caught_to_killed**
   - Verify `caught` → `MutantStatus::Killed`

5. **test_convert_missed_to_survived**
   - Verify `missed` → `MutantStatus::Survived`

6. **test_convert_timeout_outcome**
   - Verify `timeout` → `MutantStatus::Timeout`

7. **test_convert_unviable_outcome**
   - Verify `unviable` → `MutantStatus::CompileError`

8. **test_to_pmat_report_preserves_all_data**
   - Verify file, line, function preserved
   - Check mutant count matches

### Property Tests (Placeholders)

9. **proptest_json_parsing_round_trip**
   - Property: parse(serialize(data)) == data

10. **proptest_pmat_conversion_never_loses_mutants**
    - Property: to_pmat().len() == original.mutants.len()

---

## Sample cargo-mutants JSON

```json
{
  "mutants": [
    {
      "outcome": "caught",
      "file": "src/lib.rs",
      "function": "add",
      "line": 10,
      "replacement": "0"
    },
    {
      "outcome": "missed",
      "file": "src/lib.rs",
      "function": "subtract",
      "line": 15,
      "replacement": "1"
    },
    {
      "outcome": "timeout",
      "file": "src/lib.rs",
      "function": "multiply",
      "line": 20,
      "replacement": "panic!()"
    },
    {
      "outcome": "unviable",
      "file": "src/lib.rs",
      "function": "divide",
      "line": 25,
      "replacement": "compile_error!()"
    }
  ]
}
```

---

## Example Output (After Implementation)

```
📊 Parsed cargo-mutants JSON:
   Total mutants: 42
   Caught: 38 (90.5%)
   Missed: 3 (7.1%)
   Timeout: 1 (2.4%)

📊 Converted to PMAT format:
   Mutation Score: 90.5%
   Killed: 38, Survived: 3, Timeout: 1
```

---

## Development Workflow (Extreme TDD)

### RED Phase (~30 min)

1. Create test file: `server/tests/json_parsing_tests.rs`
2. Write all 10 tests with `unimplemented!()` in mock structs
3. Create example: `server/examples/parse_cargo_mutants_json.rs` (skeleton)
4. Verify all tests fail
5. Commit: "test: PMAT-070-002 RED phase - JSON parsing tests"

### GREEN Phase (~1 hour)

1. Create `server/src/services/mutation/json_parser.rs`
2. Define structs with serde derives
3. Implement `to_pmat_report()` conversion
4. Update example to use real parser
5. Verify all tests pass (100%)
6. Commit: "feat: PMAT-070-002 GREEN phase - JSON parser implementation"

### REFACTOR Phase (~45 min)

1. Extract outcome mapping to separate function
2. Add helper methods for statistics
3. Improve error messages
4. Add comprehensive documentation
5. Run cargo fmt
6. Verify tests still pass
7. Commit: "refactor: PMAT-070-002 REFACTOR phase - Clean up parser"

### VERIFY Phase (~30 min)

1. Run clippy (zero warnings)
2. Run cargo fmt --check (all formatted)
3. Run TDG scoring
4. Verify example works
5. Include metrics in commit

---

## Success Criteria

- ✅ All 10 tests passing (100%)
- ✅ Parse all cargo-mutants outcomes correctly
- ✅ Convert to PMAT Mutant format
- ✅ Handle edge cases (empty, invalid JSON)
- ✅ Working example
- ✅ Zero clippy warnings
- ✅ Code formatted
- ✅ TDG score ≥90

---

## Files to Create

1. `server/src/services/mutation/json_parser.rs` - Parser implementation
2. `server/tests/json_parsing_tests.rs` - Test suite
3. `server/examples/parse_cargo_mutants_json.rs` - Working example

---

## Dependencies

Already in Cargo.toml:
- ✅ `serde = { version = "1.0", features = ["derive"] }`
- ✅ `serde_json = "1.0"`

No new dependencies needed!

---

## Estimated Timeline

| Phase | Duration | Cumulative |
|-------|----------|------------|
| RED | 30 min | 30 min |
| GREEN | 1 hour | 1h 30min |
| REFACTOR | 45 min | 2h 15min |
| VERIFY | 30 min | 2h 45min |
| **Total** | **~3 hours** | **~3 hours** |

---

## Next Session Action Plan

**Start here**:
```bash
cd /home/noah/src/paiml-mcp-agent-toolkit

# 1. Create test file (RED phase)
touch server/tests/json_parsing_tests.rs

# 2. Write failing tests
# See "Tests to Write" section above

# 3. Create example skeleton
touch server/examples/parse_cargo_mutants_json.rs

# 4. Verify tests fail
cargo test --test json_parsing_tests

# 5. Commit RED phase
git add -A && git commit -m "test: PMAT-070-002 RED phase"
```

---

## Related Files

**Reference implementations**:
- `server/src/services/mutation/types.rs` - PMAT Mutant types
- `server/src/services/mutation/cargo_mutants_wrapper.rs` - Phase 1 implementation

**Test patterns**:
- `server/tests/cargo_mutants_wrapper_tests.rs` - Follow this structure

---

## Questions to Answer During Implementation

1. What additional fields does cargo-mutants JSON include?
2. Do we need to preserve all fields or just key ones?
3. Should we support multiple cargo-mutants versions?
4. How to handle future schema changes gracefully?

**Recommendation**: Start minimal (key fields only), expand later if needed.

---

## Completion Criteria

Before marking PMAT-070-002 complete:

- ✅ All RED tests written and failing
- ✅ All GREEN tests passing (100%)
- ✅ REFACTOR: Code quality improved
- ✅ VERIFY: All quality gates passed
- ✅ Example working correctly
- ✅ Documentation complete
- ✅ Completion report written

---

**Status**: Ready for RED phase implementation in next session.

**Previous Phase**: PMAT-070-001 ✅ COMPLETE
**Current Phase**: PMAT-070-002 🚀 STARTING
**Next Phase**: PMAT-070-003 (CLI Integration)
