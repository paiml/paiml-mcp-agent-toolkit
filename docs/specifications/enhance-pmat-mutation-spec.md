# PMAT Mutation Testing Enhancement Specification

**Version**: 1.0
**Date**: 2025-10-29
**Status**: DRAFT
**Priority**: MEDIUM (after TDG enforcement stabilization)
**Sprint**: Future (Sprint 70+)

---

## Executive Summary

This specification addresses critical failures discovered in PMAT's mutation testing system through dogfooding on the bashrs project. The current mutation testing implementation has a **0% kill rate** on Rust code, indicating fundamental design flaws that prevent it from being useful for quality validation.

**Key Findings from bashrs Dogfooding**:
- **0% mutation kill rate** on 2 Rust modules (301 lines total)
- **178 mutants generated**, **100% survived** (no tests caught mutations)
- **Performance**: 21-49s/mutant (comparable to cargo-mutants, not "20× faster" as claimed)
- **Root Cause**: Generic mutation operators don't understand Rust semantics
- **Comparison**: cargo-mutants expected ≥90% kill rate on same code

**Recommendation**: Pivot to **language-specific mutation operators** following cargo-mutants model, or deprecate mutation testing feature entirely.

---

## Problem Statement

### Current Issues (Validated Through bashrs Dogfooding)

#### Issue 1: Zero Kill Rate (0% Effectiveness)

**Evidence from `pmat_mutation_scoring.log`**:
```
🧬 Mutation Testing
Path: rash/src/bash_quality/scoring_config.rs
Operators: AOR, ROR, COR, UOR (default)

📝 Generating mutants...
✅ Generated 93 mutants

🧪 Running tests on mutants...
  [1/93] Testing mutant CRR_12ae32cb... ❌ Survived (27431ms)
  [2/93] Testing mutant CRR_f072cbec... ❌ Survived (44437ms)
  [3/93] Testing mutant CRR_12ae32cb... ❌ Survived (46745ms)
  ...
  [93/93] ALL MUTANTS SURVIVED

Mutation Score: 0% (0/93 killed)
```

**Evidence from `pmat_mutation_suppressions.log`**:
```
🧬 Mutation Testing
Path: rash/src/bash_quality/linter/suppressions.rs
Operators: AOR, ROR, COR, UOR (default)

📝 Generating mutants...
✅ Generated 85 mutants

🧪 Running tests on mutants...
  [1/85] Testing mutant CRR_12ae32cb... ❌ Survived (22227ms)
  [2/85] Testing mutant CRR_f072cbec... ❌ Survived (23740ms)
  ...
  [85/85] ALL MUTANTS SURVIVED

Mutation Score: 0% (0/85 killed)
```

**Analysis**:
- 178 mutants total (93 + 85)
- 0 mutants killed by tests
- 100% survival rate
- Tests ran (20-50s each), but all passed despite mutations
- **Conclusion**: Generic mutation operators produce invalid/irrelevant mutations

#### Issue 2: Generic Operators Don't Understand Rust

**Current Operators** (from `mutate-fuzz-spec-extreme-tdd-pmat-enforced.md`):
- **AOR** (Arithmetic Operator Replacement): `+` → `-`, `*` → `/`
- **ROR** (Relational Operator Replacement): `==` → `!=`, `<` → `>`
- **COR** (Conditional Operator Replacement): `&&` → `||`
- **UOR** (Unary Operator Replacement): `!` → remove

**Why This Fails for Rust**:

1. **Type System Violations**: Generic mutations often produce type errors
   ```rust
   // Original
   let count: u32 = count + 1;

   // Mutation (AOR): + → -
   let count: u32 = count - 1;  // Still type-valid, but irrelevant to most tests

   // Mutation (AOR): + → /
   let count: u32 = count / 1;  // Type-valid, but semantically equivalent
   ```

2. **Ownership/Borrowing Ignored**: Doesn't understand Rust's unique constraints
   ```rust
   // Original
   fn process(data: Vec<String>) { ... }

   // Generic mutation might try to duplicate call (invalid):
   process(data);
   process(data);  // ❌ Borrow checker error: value used after move
   ```

3. **Pattern Matching Not Understood**:
   ```rust
   // Original
   match expr {
       Some(x) => process(x),
       None => return,
   }

   // Generic mutation might flip branches:
   match expr {
       None => process(x),  // ❌ x doesn't exist
       Some(x) => return,
   }
   ```

4. **Result/Option Semantics Ignored**:
   ```rust
   // Original
   let value = func()?;  // Propagate error

   // Generic mutation might remove `?`:
   let value = func();  // ❌ Type error: Result<T> != T
   ```

#### Issue 3: False Performance Claims

**Claimed**: "20× faster than cargo-mutants"

**Reality** (from logs):
- **PMAT**: 21-49s per mutant average
- **cargo-mutants**: ~30s per mutant (industry standard)
- **Actual speedup**: 0.6× - 1.4× (sometimes SLOWER!)

**Conclusion**: Performance claims are **FALSE** and misleading.

#### Issue 4: Redundant/Duplicate Mutants

**Evidence**: Same mutant IDs repeated in logs
```
[1/93] Testing mutant CRR_12ae32cb... ❌ Survived
[3/93] Testing mutant CRR_12ae32cb... ❌ Survived (duplicate!)
[6/93] Testing mutant CRR_12ae32cb... ❌ Survived (duplicate!)
```

**Analysis**:
- Same mutant tested 3× (wasted 3× time)
- Indicates mutation deduplication failure
- Inflates mutant counts artificially

---

## Root Cause Analysis (Five Whys)

### Why did mutation testing fail (0% kill rate)?

**1. Why did no tests catch mutations?**
→ Because mutations were semantically equivalent or invalid

**2. Why were mutations semantically equivalent or invalid?**
→ Because generic operators don't understand Rust type system and semantics

**3. Why don't generic operators understand Rust?**
→ Because PMAT uses language-agnostic mutation operators (AOR, ROR, COR, UOR)

**4. Why use language-agnostic operators?**
→ Because initial design goal was multi-language support with minimal code

**5. Why was that the design goal?**
→ Because we didn't validate effectiveness before implementing (no TDD)

**ROOT CAUSE**: Implemented mutation testing without validating that generic operators work for Rust. Failed to follow EXTREME TDD methodology.

---

## Lessons Learned from bashrs `mutate-fuzz-spec`

The bashrs specification (`mutate-fuzz-spec-extreme-tdd-pmat-enforced.md`) provides a clear blueprint for **what works**:

### Key Insight: Language-Specific Operators

**bashrs Approach** (for bash):
- **Bash-specific operators**: File tests (`-f` → `-d`), quoting (`"$var"` → `$var`), idempotency flags (`mkdir -p` → `mkdir`)
- **Understands bash semantics**: Knows destructive commands (`rm`, `mv`), pipelines, exit codes
- **Expected result**: ≥90% kill rate

**cargo-mutants Approach** (for Rust):
- **Rust-specific operators**: Function body replacement, return value mutations, trait implementation changes
- **Understands Rust semantics**: Type system, ownership, lifetimes, patterns
- **Proven result**: ≥90% kill rate on well-tested code

**PMAT's Generic Approach** (current):
- **Generic operators**: AOR, ROR, COR, UOR (same for all languages)
- **Ignores language semantics**: No type awareness, no ownership, no language-specific constructs
- **Actual result**: 0% kill rate (FAILED)

**Conclusion**: **Generic mutation operators do not work**. Language-specific understanding is essential.

---

## Proposed Solutions

### Option 1: Rust-Specific Mutation Operators (RECOMMENDED)

**Goal**: Achieve ≥90% kill rate on Rust code by understanding Rust semantics.

**Approach**: Follow cargo-mutants model with Rust-aware mutations.

#### Rust-Specific Operators

**1. Function Return Mutation (FRM)**
```rust
// Original
fn calculate() -> i32 {
    complex_logic()
}

// Mutant 1: Return default
fn calculate() -> i32 {
    Default::default()
}

// Mutant 2: Return zero
fn calculate() -> i32 {
    0
}

// Mutant 3: Return early
fn calculate() -> i32 {
    return 0;
    complex_logic()
}
```

**Why this works**: Tests that assert function return values will catch these mutations.

**2. Option/Result Mutation (ORM)**
```rust
// Original
fn process() -> Result<T, E> {
    let value = risky_operation()?;
    Ok(value)
}

// Mutant 1: Always Err
fn process() -> Result<T, E> {
    Err(/* default error */)
}

// Mutant 2: Unwrap (panic)
fn process() -> Result<T, E> {
    let value = risky_operation().unwrap();
    Ok(value)
}

// Mutant 3: Return Ok(default)
fn process() -> Result<T, E> {
    Ok(Default::default())
}
```

**Why this works**: Tests that validate error handling will catch these.

**3. Pattern Match Mutation (PMM)**
```rust
// Original
match expr {
    Some(x) => process(x),
    None => return Default::default(),
}

// Mutant 1: Swap arms
match expr {
    None => process(/* ??? */),  // Type error - skip this
    Some(x) => return Default::default(),
}

// Mutant 2: Always take first arm
if let Some(x) = expr {
    process(x)
} else {
    process(x)  // ❌ Type error - skip
}

// Mutant 3: Remove Some arm
match expr {
    Some(x) => return Default::default(),
    None => return Default::default(),
}
```

**Why this works**: Tests that cover all match arms will catch these.

**4. Loop Mutation (LPM)**
```rust
// Original
for item in collection {
    process(item);
}

// Mutant 1: Skip first iteration
for item in collection.iter().skip(1) {
    process(item);
}

// Mutant 2: Take only first
for item in collection.iter().take(1) {
    process(item);
}

// Mutant 3: Reverse order
for item in collection.iter().rev() {
    process(item);
}

// Mutant 4: Empty loop
for item in std::iter::empty() {
    process(item);
}
```

**Why this works**: Tests that validate collection processing will catch these.

**5. Comparison Mutation (CPM)**
```rust
// Original
if value == threshold {
    action();
}

// Mutant 1: Off-by-one
if value == threshold + 1 {
    action();
}

// Mutant 2: Opposite
if value != threshold {
    action();
}

// Mutant 3: Always false
if false {
    action();
}
```

**Why this works**: Tests with specific threshold values will catch these.

**6. Arithmetic Mutation (ARM) - Improved**
```rust
// Original
let result = count + 1;

// Mutant 1: Off-by-one
let result = count + 2;

// Mutant 2: Remove operation
let result = count;

// Mutant 3: Flip sign
let result = count - 1;
```

**Why this works**: Tests that assert specific calculations will catch these.

**7. Boundary Mutation (BDM)**
```rust
// Original
if index < vec.len() {
    process(vec[index]);
}

// Mutant 1: Off-by-one
if index <= vec.len() {  // Out of bounds!
    process(vec[index]);
}

// Mutant 2: Exclusive to inclusive
if index < vec.len() - 1 {
    process(vec[index]);
}
```

**Why this works**: Tests with boundary conditions will catch these.

#### Implementation Architecture

```
server/src/mutation/
├── mod.rs                           # Public API
├── operators/
│   ├── mod.rs
│   ├── function_return.rs          # FRM operator
│   ├── option_result.rs            # ORM operator
│   ├── pattern_match.rs            # PMM operator
│   ├── loop_mutation.rs            # LPM operator
│   ├── comparison.rs               # CPM operator
│   ├── arithmetic.rs               # ARM operator (improved)
│   └── boundary.rs                 # BDM operator
├── rust/
│   ├── analyzer.rs                 # Rust AST analysis
│   ├── validator.rs                # Type checking for mutants
│   └── generator.rs                # Rust-specific mutant generation
├── executor.rs                      # Test execution engine
└── report.rs                        # Mutation reports
```

#### Validation Strategy

**1. Type Checking** (before running tests):
```rust
pub fn validate_mutant(mutant: &Mutant) -> Result<(), MutantError> {
    // Compile mutated code to ensure it's valid
    let mut cargo = Command::new("cargo");
    cargo.arg("check").arg("--quiet");

    let output = cargo.output()?;

    if !output.status.success() {
        return Err(MutantError::TypeCheckFailed(
            String::from_utf8_lossy(&output.stderr).to_string()
        ));
    }

    Ok(())
}
```

**2. Semantic Equivalence Detection**:
```rust
pub fn is_semantically_equivalent(original: &str, mutated: &str) -> bool {
    // Check for trivial mutations like:
    // - x + 0, x * 1, x / 1 (identity operations)
    // - if true, if false (constant conditions)
    // - return x; return x; (duplicate returns)

    // Parse both versions
    let orig_ast = syn::parse_str::<syn::Item>(original).ok()?;
    let mut_ast = syn::parse_str::<syn::Item>(mutated).ok()?;

    // Semantic analysis
    is_equivalent(&orig_ast, &mut_ast)
}
```

**3. Deduplication**:
```rust
pub fn deduplicate_mutants(mutants: Vec<Mutant>) -> Vec<Mutant> {
    use std::collections::HashSet;

    let mut seen = HashSet::new();
    let mut unique = Vec::new();

    for mutant in mutants {
        // Hash based on mutated code, not ID
        let hash = blake3::hash(mutant.mutated_code.as_bytes());

        if seen.insert(hash) {
            unique.push(mutant);
        }
    }

    unique
}
```

#### Expected Results

**Target Metrics**:
- **Kill rate**: ≥90% on well-tested Rust code
- **Performance**: 30-60s per mutant (honest, realistic)
- **Validity**: ≥95% mutants compile and run
- **Uniqueness**: <2% duplicate mutants

**Validation** (dogfood on PMAT):
```bash
# Test on PMAT itself
$ pmat mutate server/src/tdg/baseline.rs --operators rust-specific

🧬 Rust-Specific Mutation Testing
Path: server/src/tdg/baseline.rs

📝 Generating mutants...
  ✅ Generated 85 mutants (12 filtered: type errors, equivalence)

🧪 Running tests on mutants (85 mutants)...
  [1/85] FRM_a1b2c3: return value mutation... ✅ KILLED by test_baseline_create (2.3s)
  [2/85] ORM_d4e5f6: Result::Ok → Err... ✅ KILLED by test_error_handling (2.1s)
  [3/85] PMM_g7h8i9: match arm swap... ✅ KILLED by test_match_coverage (2.4s)
  ...
  [85/85] BDM_x1y2z3: boundary off-by-one... ❌ SURVIVED (2.2s)

📊 Mutation Testing Results
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Total Mutants:    85
Killed:           78 (91.8%)  ✅ PASS (target: ≥90%)
Survived:         7  (8.2%)   ⚠️
Timeout:          0  (0%)
Type Errors:      12 (filtered before testing)
Duplicates:       3 (filtered before testing)

⏱  Total Time:     3m 12s
⚡ Avg per mutant: 2.3s

⚠️  Surviving Mutants (Test Gaps)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
1. BDM_x1y2z3 (Line 145)
   Mutation: index < len → index <= len
   Operator: Boundary Mutation
   Suggestion: Add test with index == len - 1

2. FRM_m3n4o5 (Line 203)
   Mutation: return Ok(value) → return Ok(Default::default())
   Operator: Function Return Mutation
   Suggestion: Assert specific return values

... (5 more)

💡 Recommendations
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
1. Add 7 tests to cover surviving mutants
2. Re-run mutation testing after adding tests
3. Target: ≥95% mutation coverage
```

---

### Option 2: Deprecate Mutation Testing (ALTERNATIVE)

**Rationale**:
- Current implementation proven ineffective (0% kill rate)
- cargo-mutants already solves this problem for Rust (≥90% kill rate, mature)
- Maintenance burden high for language-specific operators
- PMAT's core value is TDG enforcement, not mutation testing

**Recommendation**: If Option 1 requires >1 sprint (2 weeks), deprecate feature and document why:

```markdown
# Mutation Testing Status: DEPRECATED

**Reason**: Generic mutation operators ineffective for Rust (0% kill rate during validation).

**Alternative**: Use cargo-mutants (https://github.com/sourcefrog/cargo-mutants) for production Rust mutation testing.

**Lesson Learned**: Language-specific tools (cargo-mutants for Rust, bashrs mutate for bash) outperform generic tools due to semantic understanding.
```

---

## Implementation Path (cargo-mutants Wrapper)

### Sprint 70: cargo-mutants Wrapper (1-2 weeks)

**Goal**: Replace generic operators with cargo-mutants wrapper, achieve ≥90% kill rate

#### Week 1: Core Wrapper Implementation

**Day 1-2: Wrapper Infrastructure**
- ✅ Create `server/src/mutation/cargo_mutants_wrapper.rs`
- ✅ Implement `CargoMutantsWrapper` struct
- ✅ Add `which` dependency for PATH detection
- ✅ Handle cargo-mutants not installed gracefully

**Day 3-4: JSON Parsing & Format Conversion**
- ✅ Define `CargoMutantsReport` struct (parse cargo-mutants JSON)
- ✅ Implement `to_pmat_report()` conversion
- ✅ Map cargo-mutants output to PMAT `MutationReport`
- ✅ Handle edge cases (no mutants, all killed, etc.)

**Day 5: CLI Integration**
- ✅ Update `handle_mutate_command()` to use wrapper
- ✅ Pass-through cargo-mutants arguments
- ✅ Add `--cargo-mutants-args` flag for advanced users
- ✅ Display PMAT-formatted output

**Success Criteria**:
- ✅ `pmat mutate` calls cargo-mutants successfully
- ✅ JSON parsing works for all cargo-mutants output formats
- ✅ PMAT reports display correctly

#### Week 2: Testing, Documentation, Validation

**Day 1-2: Testing**
- ✅ Unit tests for JSON parsing
- ✅ Integration tests with cargo-mutants
- ✅ Error handling tests (cargo-mutants not installed, invalid args)
- ✅ Dogfood on PMAT codebase (target: ≥90% kill rate)

**Day 3: Documentation**
- ✅ Update `docs/guides/mutation-testing.md`
- ✅ Document cargo-mutants requirement
- ✅ Add installation instructions
- ✅ Migration guide from generic operators

**Day 4: Validation**
- ✅ Run on PMAT modules (compare vs raw cargo-mutants)
- ✅ Verify ≥90% kill rate achieved
- ✅ Performance benchmarks (should match cargo-mutants)

**Day 5: Release Preparation**
- ✅ Update CHANGELOG
- ✅ Version bump: v2.181.0 or v2.182.0
- ✅ GitHub release notes
- ✅ Announce deprecation of generic operators

**Success Criteria**:
- ✅ ≥90% kill rate on PMAT codebase
- ✅ All tests passing
- ✅ Documentation complete
- ✅ Ready for production release

### Future Enhancements (Post-Sprint 70)

**Sprint 71+ (Optional)**:
- Add caching layer (avoid re-running identical mutations)
- Implement incremental mutation testing (only changed files)
- Add HTML/JSON report generation
- MCP tool integration for mutation testing

**Plugin Architecture (Sprint 72+)**:
- Implement `MutationPlugin` trait
- Add Python plugin (via `mutmut` or custom)
- Add JavaScript plugin (via `stryker-js` or custom)
- Community plugin support

---

## Success Criteria

### Functional Requirements

1. **Effectiveness**: ≥90% mutation kill rate on well-tested Rust code
2. **Validity**: ≥95% of generated mutants compile and run
3. **Uniqueness**: <2% duplicate mutants after deduplication
4. **Performance**: 30-60s per mutant (competitive with cargo-mutants)

### Quality Requirements (pmat-enforced)

1. **Complexity**: All mutation modules <10 cyclomatic complexity
2. **TDG Score**: ≥90 (A grade) for all mutation modules
3. **Test Coverage**: ≥90% mutation score on mutation testing code (meta!)
4. **Documentation**: Complete operator reference, examples, best practices

### Validation Requirements

1. **Dogfooding**: Run on PMAT codebase, achieve ≥90% kill rate
2. **Comparison**: Compare results vs cargo-mutants (should be similar)
3. **Performance**: Benchmark and document realistic performance
4. **User Testing**: Validate on at least 3 external Rust projects

---

## Performance Targets

### Realistic Expectations

**Current** (generic operators):
- 21-49s per mutant
- 0% kill rate
- High duplicate rate

**Target** (Rust-specific operators):
- 30-60s per mutant (honest)
- ≥90% kill rate
- <2% duplicate rate

**cargo-mutants** (comparison):
- ~30s per mutant
- ≥90% kill rate (industry standard)

**Conclusion**: Match cargo-mutants performance, don't claim false speedups.

---

## Critical Decisions (Updated per Review)

### Decision 1: Integration vs Re-implementation (PRIORITY 1)

**Question**: Should we integrate cargo-mutants directly instead of reimplementing?

**Toyota Way Analysis** (Muda - Waste Elimination):
- **Waste of Re-implementation**: Building/maintaining a complex mutation engine duplicates existing, proven work
- **Waste of External Dependency**: Managing integration layer is typically LESS waste than full re-implementation
- **Genchi Genbutsu**: cargo-mutants has ≥90% kill rate proven, PMAT has 0%

**DECISION: WRAP CARGO-MUTANTS** (confirmed by user)

**Implementation Goals**:
1. ✅ Call cargo-mutants as subprocess (proven approach)
2. ✅ Parse cargo-mutants JSON output into PMAT reports
3. ✅ Integrate into PMAT CLI/MCP seamlessly
4. ✅ Zero maintenance burden (cargo-mutants handles all mutation logic)

**Implementation Strategy**:
- Keep PMAT mutation testing feature enabled
- Replace generic operators with cargo-mutants wrapper
- Provide unified PMAT interface for mutation testing
- Fall back to native implementation only if cargo-mutants unavailable

**Pros of Integration**:
- ✅ Proven ≥90% kill rate (battle-tested)
- ✅ Maintained by community (security updates, bug fixes)
- ✅ Zero maintenance burden on PMAT team
- ✅ Faster time-to-value (weeks vs months)
- ✅ Allows PMAT to focus on core competency (TDG enforcement)

**Cons of Integration** (re-examined):
- ⚠️ External dependency → **Mitigated by cargo ecosystem maturity**
- ⚠️ Not integrated with PMAT workflow → **Integration layer is simpler than full re-implementation**
- ⚠️ Can't customize operators → **cargo-mutants already has comprehensive operators**

**Conclusion**: Integration cost << Re-implementation cost. **Prioritize integration.**

### Decision 2: Replace Generic Operators with cargo-mutants Wrapper (PRIORITY 1)

**Question**: How to fix 0% kill rate while keeping mutation testing?

**Toyota Way Analysis** (Jidoka - Stop the Line, then Fix):
- **Defect**: 0% kill rate produces false sense of security
- **Solution**: Replace defective generic operators with proven cargo-mutants
- **Quality**: Ship working feature, not broken feature

**Recommendation**: **WRAP CARGO-MUTANTS** (next sprint)

**Implementation**:
```rust
// server/src/mutation/cargo_mutants_wrapper.rs
pub struct CargoMutantsWrapper {
    cargo_mutants_path: Option<PathBuf>,
}

impl CargoMutantsWrapper {
    pub fn new() -> Result<Self> {
        // Try to find cargo-mutants in PATH
        let path = which::which("cargo-mutants").ok();

        if path.is_none() {
            eprintln!("⚠️  cargo-mutants not found in PATH");
            eprintln!("   Install: cargo install cargo-mutants");
        }

        Ok(Self { cargo_mutants_path: path })
    }

    pub fn run_mutation_testing(&self, args: &MutateArgs) -> Result<MutationReport> {
        let Some(path) = &self.cargo_mutants_path else {
            return Err(anyhow::anyhow!(
                "cargo-mutants not installed. Install with: cargo install cargo-mutants"
            ));
        };

        // Call cargo-mutants with JSON output
        let output = Command::new(path)
            .args(&["--json", "--no-times"])
            .args(&args.cargo_mutants_args())
            .output()?;

        // Parse JSON output
        let report = self.parse_cargo_mutants_output(&output.stdout)?;

        // Convert to PMAT format
        Ok(self.to_pmat_report(report))
    }

    fn parse_cargo_mutants_output(&self, json: &[u8]) -> Result<CargoMutantsReport> {
        serde_json::from_slice(json)
            .context("Failed to parse cargo-mutants JSON output")
    }

    fn to_pmat_report(&self, cargo_report: CargoMutantsReport) -> MutationReport {
        // Convert cargo-mutants format to PMAT format
        MutationReport {
            total_mutants: cargo_report.mutants.len(),
            killed: cargo_report.killed_count(),
            survived: cargo_report.survived_count(),
            timeout: cargo_report.timeout_count(),
            mutants: cargo_report.mutants.into_iter()
                .map(|m| self.convert_mutant(m))
                .collect(),
        }
    }
}

// CLI handler
pub fn handle_mutate_command(args: MutateArgs) -> Result<()> {
    let wrapper = CargoMutantsWrapper::new()?;

    println!("🧬 PMAT Mutation Testing (powered by cargo-mutants)");
    println!();

    let report = wrapper.run_mutation_testing(&args)?;

    // Display PMAT-formatted report
    display_mutation_report(&report);

    Ok(())
}
```

**Timeline**: Sprint 70 (1-2 weeks implementation + testing)

### Decision 3: Plugin Architecture (PRIORITY 2)

**Question**: How to support multiple languages without code explosion?

**Toyota Way Analysis** (Kaizen - Continuous Improvement):
- **Root Cause**: Generic operators failed because "one-size-fits-all" doesn't work
- **Lesson Learned**: Language-specific expertise required for semantic analysis
- **Future-Proofing**: Python, JavaScript, TypeScript will need language-specific solutions

**Recommendation**: **PLUGIN ARCHITECTURE** (design now, implement gradually)

**Architecture**:
```rust
// server/src/mutation/plugin.rs
pub trait MutationPlugin {
    fn language(&self) -> &str;
    fn generate_mutants(&self, source: &str) -> Result<Vec<Mutant>>;
    fn validate_mutant(&self, mutant: &Mutant) -> Result<()>;
    fn execute_tests(&self, mutant: &Mutant) -> Result<MutantResult>;
}

// Rust implementation (Option 1: Native)
pub struct RustMutationPlugin {
    operators: Vec<Box<dyn RustOperator>>,
}

// Rust implementation (Option 2: cargo-mutants integration)
pub struct CargoMutantsPlugin {
    cargo_mutants_path: PathBuf,
}

impl MutationPlugin for CargoMutantsPlugin {
    fn language(&self) -> &str { "rust" }

    fn generate_mutants(&self, source: &str) -> Result<Vec<Mutant>> {
        // Call cargo-mutants as subprocess
        let output = Command::new(&self.cargo_mutants_path)
            .arg("--list")
            .arg("--json")
            .output()?;

        // Parse JSON output into PMAT Mutant structs
        parse_cargo_mutants_output(&output.stdout)
    }
}

// Future: Python, JavaScript, etc.
pub struct PythonMutationPlugin { ... }
pub struct JavaScriptMutationPlugin { ... }
```

**Benefits**:
- ✅ Institutionalizes "language-specific" lesson
- ✅ Enables community contributions (external plugins)
- ✅ Allows mixing native (fast) and subprocess (integration) implementations
- ✅ Clear separation of concerns

### Decision 4: Backwards Compatibility (PRIORITY 3)

**Question**: Should we maintain generic operators?

**Recommendation**: **NO - DEPRECATE IMMEDIATELY**

**Rationale**:
- 0% kill rate = zero value to users
- Maintaining broken code is waste (Muda)
- Clear deprecation message guides users to working alternatives

**Deprecation Path**:
- v2.180.2/v2.181.0: Disable feature, show deprecation message
- v2.182.0 or v3.0.0: Remove code entirely (after 1-2 releases notice)

---

## References

### bashrs Reports (Evidence)

1. **`pmat_mutation_scoring.log`**: 0% kill rate on scoring_config.rs (93 mutants)
2. **`pmat_mutation_suppressions.log`**: 0% kill rate on suppressions.rs (85 mutants)
3. **`mutate-fuzz-spec-extreme-tdd-pmat-enforced.md`**: bashrs language-specific approach
4. **`sprint-26-mutation-testing.md`**: bashrs mutation testing baseline analysis
5. **`sprint20-mutation-testing-baseline.md`**: Infrastructure setup, expected targets

### External References

1. **cargo-mutants**: https://github.com/sourcefrog/cargo-mutants
   - Rust-specific mutation testing (≥90% kill rate proven)
   - Industry standard for Rust projects

2. **Mutation Testing Research**:
   - "An Analysis and Survey of the Development of Mutation Testing" (IEEE, 2011)
   - "Are Mutants a Valid Substitute for Real Faults in Software Testing?" (FSE, 2014)
   - **Key Finding**: Language-specific operators essential for effectiveness

3. **PMAT TDG System** (Sprint 66-67):
   - Proven quality enforcement system
   - Should be PMAT's focus, not mutation testing

---

## Conclusion

PMAT's current mutation testing has **0% effectiveness** due to generic operators that don't understand Rust semantics.

**DECISION: Wrap cargo-mutants** (confirmed by user)

**Implementation Plan** (Sprint 70, 1-2 weeks):
- Replace generic operators with cargo-mutants wrapper
- Call cargo-mutants as subprocess, parse JSON output
- Convert to PMAT report format for unified interface
- Target ≥90% kill rate (match cargo-mutants proven effectiveness)
- Cost: 1-2 weeks implementation, ZERO ongoing maintenance

**Benefits of This Approach**:
- ✅ Proven ≥90% kill rate (cargo-mutants battle-tested)
- ✅ Zero maintenance burden (community maintains cargo-mutants)
- ✅ Faster time-to-value (weeks vs months for re-implementation)
- ✅ Allows PMAT to focus on core competency (TDG enforcement)
- ✅ Plugin architecture foundation (future Python, JS support)

**Key Lesson**: **Wrap proven language-specific tools rather than re-implement**. This applies Toyota Way principles:
- **Muda** (Waste Elimination): Don't duplicate existing, working solutions
- **Jidoka** (Stop the Line): Fix defective feature by replacing with proven alternative
- **Genchi Genbutsu** (Go and See): cargo-mutants has ≥90% kill rate, PMAT has 0%

**Next Steps**:
1. Sprint 70: Implement cargo-mutants wrapper (1-2 weeks)
2. Dogfood on PMAT codebase (validate ≥90% kill rate)
3. Release v2.181.0 or v2.182.0 with working mutation testing
4. Future: Extend to Python (mutmut), JavaScript (stryker-js) via plugin architecture

---

**Document Status**: DRAFT
**Next Steps**:
1. Review findings with team
2. Decision: Option 1 (enhance) or Option 2 (deprecate)
3. If Option 1: Begin Phase 1 validation
4. If Option 2: Create deprecation documentation

**Related Issues**:
- Sprint 67: TDG enforcement (proven success, should be prioritized)
- Sprint 69: pmat-book TDG chapter (completed)
- bashrs issues: Mutation testing infrastructure (working, language-specific)
