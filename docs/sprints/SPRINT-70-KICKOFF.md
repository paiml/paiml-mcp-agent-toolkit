# Sprint 70: cargo-mutants Wrapper - KICKOFF

**Sprint**: 70
**Goal**: Replace defective generic mutation operators with cargo-mutants wrapper
**Duration**: 1-2 weeks
**Priority**: HIGH (fix 0% effectiveness defect)
**Status**: 🚀 STARTING

---

## Context

**Problem**: PMAT mutation testing has 0% kill rate due to generic operators that don't understand Rust semantics.

**Evidence** (from bashrs dogfooding):
- 178 mutants generated across 2 Rust modules
- 0 mutants killed by tests (100% survival rate)
- Generic operators (AOR, ROR, COR, UOR) produce invalid/irrelevant mutations
- See: `docs/specifications/enhance-pmat-mutation-spec.md`

**Solution**: Wrap cargo-mutants (proven ≥90% kill rate) instead of re-implementing language-specific operators.

---

## Sprint Goals

1. **Replace generic operators** with cargo-mutants subprocess wrapper
2. **Achieve ≥90% kill rate** on PMAT codebase (match cargo-mutants effectiveness)
3. **Maintain PMAT interface** (seamless user experience)
4. **Zero maintenance burden** (community maintains cargo-mutants)

---

## Architecture

### Current (Broken)
```
pmat mutate <file>
    ↓
Generic operators (AOR, ROR, COR, UOR)
    ↓
Generate 178 mutants
    ↓
Run tests: 0% kill rate ❌
```

### Target (Working)
```
pmat mutate <file>
    ↓
CargoMutantsWrapper
    ↓
cargo-mutants (subprocess)
    ↓
Parse JSON output
    ↓
Convert to PMAT format
    ↓
Display: ≥90% kill rate ✅
```

---

## Implementation Plan

### Week 1: Core Wrapper (Days 1-5)

#### Day 1-2: Infrastructure ✅
**Goal**: Basic wrapper that can call cargo-mutants

**Tasks**:
1. Create `server/src/mutation/cargo_mutants_wrapper.rs`
2. Add `which` dependency to Cargo.toml
3. Implement `CargoMutantsWrapper::new()`
4. Handle cargo-mutants not installed gracefully
5. Add basic subprocess execution

**Success Criteria**:
- ✅ Can detect cargo-mutants in PATH
- ✅ Can execute `cargo-mutants --version`
- ✅ Graceful error if not installed
- ✅ Unit tests passing

**Example**:
```rust
pub struct CargoMutantsWrapper {
    cargo_mutants_path: Option<PathBuf>,
}

impl CargoMutantsWrapper {
    pub fn new() -> Result<Self> {
        let path = which::which("cargo-mutants").ok();
        if path.is_none() {
            eprintln!("⚠️  cargo-mutants not found");
            eprintln!("   Install: cargo install cargo-mutants");
        }
        Ok(Self { cargo_mutants_path: path })
    }
}
```

#### Day 3-4: JSON Parsing ✅
**Goal**: Parse cargo-mutants output into PMAT format

**Tasks**:
1. Define `CargoMutantsReport` struct (mirrors cargo-mutants JSON)
2. Implement JSON parsing with serde
3. Implement `to_pmat_report()` conversion
4. Handle edge cases (no mutants, all killed, timeouts)
5. Add comprehensive parsing tests

**Success Criteria**:
- ✅ Can parse all cargo-mutants JSON output formats
- ✅ Handles mutant outcomes: killed, survived, timeout, unviable
- ✅ Converts to PMAT `MutationReport` correctly
- ✅ Edge case handling tested

**Example**:
```rust
#[derive(Debug, Deserialize)]
struct CargoMutantsReport {
    mutants: Vec<CargoMutant>,
}

#[derive(Debug, Deserialize)]
struct CargoMutant {
    scenario: String,
    outcome: MutantOutcome,
    #[serde(default)]
    phase: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum MutantOutcome {
    Caught,
    Missed,
    Timeout,
    Unviable,
}
```

#### Day 5: CLI Integration ✅
**Goal**: Wire up wrapper to PMAT CLI

**Tasks**:
1. Update `handle_mutate_command()` to use wrapper
2. Pass through cargo-mutants arguments
3. Add `--cargo-mutants-args` for advanced users
4. Display PMAT-formatted output
5. Add CLI integration tests

**Success Criteria**:
- ✅ `pmat mutate` calls cargo-mutants successfully
- ✅ Arguments passed through correctly
- ✅ PMAT-formatted output displayed
- ✅ Help text updated

**Example**:
```rust
pub fn handle_mutate_command(args: MutateArgs) -> Result<()> {
    let wrapper = CargoMutantsWrapper::new()?;

    println!("🧬 PMAT Mutation Testing (powered by cargo-mutants)");
    println!();

    let report = wrapper.run_mutation_testing(&args)?;

    display_mutation_report(&report);
    Ok(())
}
```

### Week 2: Testing & Release (Days 6-10)

#### Day 6-7: Testing ✅
**Goal**: Comprehensive test coverage

**Tasks**:
1. Unit tests for JSON parsing (all edge cases)
2. Integration tests with cargo-mutants
3. Error handling tests (not installed, invalid args, etc.)
4. Dogfood on PMAT codebase (target: ≥90% kill rate)
5. Compare results vs raw cargo-mutants

**Success Criteria**:
- ✅ All unit tests passing
- ✅ Integration tests working
- ✅ ≥90% kill rate on PMAT modules
- ✅ Results match raw cargo-mutants (within margin)

#### Day 8: Documentation ✅
**Goal**: Update all documentation

**Tasks**:
1. Update `docs/guides/mutation-testing.md`
2. Document cargo-mutants requirement
3. Add installation instructions
4. Create migration guide from generic operators
5. Update README with new approach

**Success Criteria**:
- ✅ Complete installation guide
- ✅ Usage examples with expected output
- ✅ Troubleshooting section
- ✅ Migration guide from v2.180.1

#### Day 9: Validation ✅
**Goal**: Final validation before release

**Tasks**:
1. Run on multiple PMAT modules
2. Verify ≥90% kill rate achieved
3. Performance benchmarks (should match cargo-mutants)
4. Compare vs generic operators (0% → ≥90%)
5. Final quality checks

**Success Criteria**:
- ✅ ≥90% kill rate on PMAT codebase
- ✅ Performance acceptable (~30s/mutant)
- ✅ All quality gates passing
- ✅ pmat-book validation passing

#### Day 10: Release Preparation ✅
**Goal**: Package and release v2.181.0 or v2.182.0

**Tasks**:
1. Update CHANGELOG
2. Version bump: v2.181.0 or v2.182.0
3. Create GitHub release
4. Update crates.io
5. Announce deprecation of generic operators

**Success Criteria**:
- ✅ Version bumped correctly
- ✅ CHANGELOG complete
- ✅ GitHub release published
- ✅ crates.io updated
- ✅ Announcement posted

---

## Success Criteria

### Functional
- ✅ cargo-mutants wrapper working
- ✅ ≥90% kill rate on PMAT codebase (vs 0% with generic operators)
- ✅ JSON parsing handles all cargo-mutants output
- ✅ PMAT CLI interface maintained (seamless for users)

### Quality (pmat-enforced)
- ✅ All modules: Cyclomatic complexity <10
- ✅ All modules: TDG score ≥90 (A grade)
- ✅ All tests passing (unit, integration, dogfooding)
- ✅ Zero security issues

### Performance
- ✅ 30-60s per mutant (match cargo-mutants)
- ✅ No significant overhead from wrapper
- ✅ Memory usage reasonable

### Documentation
- ✅ Installation guide complete
- ✅ Migration guide from generic operators
- ✅ Troubleshooting section
- ✅ pmat-book validation passing

---

## Files to Create/Modify

### New Files
- `server/src/mutation/cargo_mutants_wrapper.rs` (~300 lines)
- `server/tests/mutation/cargo_mutants_integration.rs` (~200 lines)
- `docs/guides/mutation-testing-cargo-mutants.md` (~400 lines)

### Modified Files
- `server/src/mutation/mod.rs` (export wrapper)
- `server/src/cli/handlers/mutate.rs` (use wrapper)
- `Cargo.toml` (add `which` dependency)
- `docs/guides/mutation-testing.md` (update for wrapper)
- `README.md` (mention cargo-mutants requirement)
- `CHANGELOG.md` (document changes)

**Estimated Total**: ~1,200 lines (code + docs + tests)

---

## Dependencies

### New Cargo Dependencies
```toml
[dependencies]
which = "6.0"  # Find cargo-mutants in PATH
```

### External Tool
- **cargo-mutants** v24.7.0+ (user must install)
- Installation: `cargo install cargo-mutants`

---

## Risk Mitigation

### Risk 1: cargo-mutants not installed
**Mitigation**: Clear error message with installation instructions
```
⚠️  cargo-mutants not found in PATH
   Install: cargo install cargo-mutants

   After installation, retry: pmat mutate <file>
```

### Risk 2: cargo-mutants API changes
**Mitigation**: Version check and error handling
```rust
fn check_cargo_mutants_version(&self) -> Result<Version> {
    let output = Command::new(&self.path)
        .arg("--version")
        .output()?;

    let version = parse_version(&output.stdout)?;

    if version < Version::new(24, 7, 0) {
        return Err(anyhow!(
            "cargo-mutants v24.7.0+ required (found {})", version
        ));
    }

    Ok(version)
}
```

### Risk 3: JSON parsing failures
**Mitigation**: Comprehensive error handling + fallback
```rust
match serde_json::from_slice(&output.stdout) {
    Ok(report) => Ok(report),
    Err(e) => {
        eprintln!("⚠️  Failed to parse cargo-mutants output");
        eprintln!("   Raw output saved to: /tmp/pmat-mutants-output.json");
        std::fs::write("/tmp/pmat-mutants-output.json", &output.stdout)?;
        Err(e.into())
    }
}
```

### Risk 4: Performance overhead
**Mitigation**: Minimal wrapper logic, direct subprocess passthrough
- No additional processing between cargo-mutants and user
- JSON parsing is O(n) where n = number of mutants
- Display formatting is negligible

---

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cargo_mutants_json_all_outcomes() {
        let json = r#"{
            "mutants": [
                {"scenario": "test1", "outcome": "caught"},
                {"scenario": "test2", "outcome": "missed"},
                {"scenario": "test3", "outcome": "timeout"},
                {"scenario": "test4", "outcome": "unviable"}
            ]
        }"#;

        let report: CargoMutantsReport = serde_json::from_str(json).unwrap();
        assert_eq!(report.mutants.len(), 4);
    }

    #[test]
    fn test_convert_to_pmat_report() {
        // Test conversion from cargo-mutants format to PMAT format
    }

    #[test]
    fn test_handle_not_installed() {
        // Test graceful handling when cargo-mutants not found
    }
}
```

### Integration Tests
```rust
#[test]
#[ignore] // Requires cargo-mutants installed
fn test_run_cargo_mutants_on_sample_file() {
    let wrapper = CargoMutantsWrapper::new().unwrap();

    // Run on simple Rust file
    let args = MutateArgs {
        path: "tests/fixtures/sample.rs".into(),
        ..Default::default()
    };

    let report = wrapper.run_mutation_testing(&args).unwrap();

    // Should have found some mutants
    assert!(report.total_mutants > 0);

    // Should have killed some (if tests exist)
    // Kill rate will vary, just check it ran
}
```

### Dogfooding Test
```bash
# Run on PMAT itself
pmat mutate server/src/tdg/baseline.rs

# Expected:
# - Generate ~50-100 mutants
# - ≥90% kill rate (vs 0% with generic operators)
# - ~30s per mutant
# - Total time: ~30-50 minutes
```

---

## Rollout Plan

### Phase 1: Development (Week 1)
- Implement core wrapper
- JSON parsing
- CLI integration
- Initial testing

### Phase 2: Validation (Week 2, Days 1-3)
- Comprehensive testing
- Documentation
- Dogfooding on PMAT

### Phase 3: Release (Week 2, Days 4-5)
- Final validation
- Version bump (v2.181.0 or v2.182.0)
- Publish to crates.io
- GitHub release

### Phase 4: Communication (Post-Release)
- Announce in README
- Update pmat-book
- Notify users of generic operator deprecation
- Social media announcement

---

## Communication Plan

### User Notification

**In-app Message** (for users of generic operators):
```
⚠️  PMAT Mutation Testing Update (v2.181.0+)

Generic mutation operators have been replaced with cargo-mutants wrapper
due to 0% effectiveness on Rust code.

Action Required:
1. Install cargo-mutants: cargo install cargo-mutants
2. Run mutation testing: pmat mutate <file>
3. Enjoy ≥90% kill rate (vs 0% previously)

See: docs/guides/mutation-testing-cargo-mutants.md
```

**Migration Guide**:
```markdown
# Migrating from Generic Operators to cargo-mutants Wrapper

## Why the Change?

Generic operators (AOR, ROR, COR, UOR) had 0% kill rate on Rust code:
- 178 mutants generated, 0 killed (bashrs dogfooding)
- Didn't understand Rust type system, ownership, semantics
- Produced invalid/irrelevant mutations

cargo-mutants has ≥90% kill rate (proven, battle-tested).

## Migration Steps

1. Install cargo-mutants:
   ```bash
   cargo install cargo-mutants
   ```

2. Run mutation testing (same command):
   ```bash
   pmat mutate <file>
   ```

3. Enjoy working mutation testing!

## What Changed?

- **Old**: Generic operators (broken, 0% kill rate)
- **New**: cargo-mutants wrapper (working, ≥90% kill rate)
- **Interface**: Same (seamless transition)
- **Dependencies**: Requires cargo-mutants installed
```

---

## Next Steps After Sprint 70

### Immediate (v2.181.1+)
- Monitor user feedback
- Fix any integration issues
- Improve error messages if needed

### Future (Sprint 71+)
- Python mutation testing (wrap `mutmut`)
- JavaScript mutation testing (wrap `stryker-js`)
- Plugin architecture for language-specific wrappers
- Caching layer for incremental mutation testing

---

## Conclusion

Sprint 70 will fix the critical 0% mutation testing effectiveness by wrapping proven cargo-mutants tool. This applies Toyota Way principles:

- **Muda** (Waste Elimination): Wrap proven tool vs re-implement
- **Jidoka** (Stop the Line): Fix 0% defect immediately
- **Genchi Genbutsu** (Go and See): cargo-mutants has ≥90% data, PMAT has 0%

**Timeline**: 1-2 weeks
**Outcome**: Working mutation testing (0% → ≥90% kill rate)
**Maintenance**: Zero (community maintains cargo-mutants)

---

**Document Status**: KICKOFF
**Created**: October 29, 2025
**Sprint**: Sprint 70 - cargo-mutants Wrapper
**Ready to Start**: ✅ YES
