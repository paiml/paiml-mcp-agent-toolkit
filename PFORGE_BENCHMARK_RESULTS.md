# PMAT vs cargo-mutants Benchmark on pforge

**Date**: October 5, 2025
**PMAT Version**: v2.130.0
**cargo-mutants Version**: v25.3.1
**Test File**: `/home/noah/src/pforge/crates/pforge-config/src/validator.rs` (147 lines)

## Executive Summary

❌ **PMAT FAILED to generate ANY mutants** (0 mutants)
✅ **cargo-mutants generated 4 mutants** (100% caught by tests)

**Root Cause**: PMAT's AST-based mutation operators are not detecting any mutation opportunities.

## Test Results

### cargo-mutants Results

```bash
$ cd /home/noah/src/pforge && cargo mutants --file crates/pforge-config/src/validator.rs --no-times --timeout 60

Found 4 mutants to test
ok       Unmutated baseline
4 mutants tested: 4 caught

Time: 20.4s
```

**Mutants Found by cargo-mutants**:
1. `validator.rs:6:5` - replace `validate_config -> Result<()>` with `Ok(())`
2. `validator.rs:9:12` - delete `!` in `validate_config`
3. `validator.rs:25:5` - replace `validate_handler_path -> Result<()>` with `Ok(())`
4. `validator.rs:30:8` - delete `!` in `validate_handler_path`

**Mutation Score**: 100% (4/4 caught)

### PMAT Results

```bash
$ pmat analyze mutate --path /home/noah/src/pforge/crates/pforge-config/src/validator.rs --operators AOR,ROR,COR
📝 Generating mutants...
✅ Generated 0 mutants
⚠️  No mutants generated - file may be too simple or no applicable operators
```

**Mutants Found**: 0

Tried operators:
- `AOR,ROR,COR` → 0 mutants
- `SDL,CRR,UOR` → 0 mutants
- All operators → 0 mutants

## Analysis

### What cargo-mutants Found

cargo-mutants uses **function-level mutations**:

**Return Value Replacement** (2 mutants):
```rust
// Original
pub fn validate_config(config: &ForgeConfig) -> Result<()> {
    // ... validation logic
    Ok(())
}

// Mutant: Always return Ok
pub fn validate_config(config: &ForgeConfig) -> Result<()> {
    Ok(())  // Bypasses all validation!
}
```

**Unary Operator Deletion** (2 mutants):
```rust
// Original
if !tool_names.insert(name) {
    return Err(ConfigError::DuplicateToolName(...));
}

// Mutant: Delete !
if tool_names.insert(name) {  // Logic inverted!
    return Err(ConfigError::DuplicateToolName(...));
}
```

### Why PMAT Found Nothing

PMAT's mutation operators are **expression-level** only:
- **AOR** (Arithmetic): `+`, `-`, `*`, `/` → None in this file
- **ROR** (Relational): `<`, `>`, `==`, `!=` → None in this file
- **COR** (Conditional): `&&`, `||` → None in this file
- **UOR** (Unary): Should detect `!` but didn't
- **SDL** (Statement Deletion): Should detect statements but didn't
- **CRR** (Constant Replacement): Should detect literals but didn't

**Critical Bug**: PMAT's operators are not being applied!

## Root Cause Investigation

Looking at PMAT's mutation engine, there are likely issues in:

1. **AST Traversal** - Not visiting all AST nodes
2. **Pattern Matching** - Not detecting mutation opportunities
3. **Operator Implementation** - UOR should find `!`, SDL should find statements
4. **Integration** - MutationEngine → RustAdapter → Operators not wired correctly

Specifically for this file:
- **UOR should detect**: `!tool_names.insert(name)`, `!path.is_empty()`, `!path.contains("::")`
- **SDL should detect**: Multiple statements in functions
- **CRR should detect**: String literals, method calls returning constants

## Comparison Matrix

| Metric | PMAT v2.130.0 | cargo-mutants v25.3.1 | Winner |
|--------|---------------|------------------------|---------|
| **Mutants Found** | 0 | 4 | cargo-mutants |
| **Mutation Score** | N/A | 100% | cargo-mutants |
| **Execution Time** | <1s | 20.4s | PMAT |
| **Return Mutations** | ❌ Not implemented | ✅ 2 found | cargo-mutants |
| **Unary Mutations** | ❌ 0 found (should be 2) | ✅ 2 found | cargo-mutants |
| **Test Execution** | ✅ Implemented | ✅ Implemented | Tie |

## Critical Findings

### ❌ **PMAT is NOT Production Ready for Mutation Testing**

**Severity**: **CRITICAL**

**Evidence**:
- 0 mutants generated on real-world Rust code
- cargo-mutants found 4 obvious mutation opportunities
- PMAT's operators claimed to work but generated nothing
- No validation in test suite that operators actually generate mutants

**Impact**:
- v2.130.0 claimed "empirical mutation testing" but operators don't work
- GitHub Issue #63 marked as "Priority 1 RESOLVED" but system non-functional
- MutantExecutor works, but has no mutants to execute

### Root Cause: Insufficient Integration Testing

PMAT has:
- ✅ Unit tests for individual operators
- ✅ Unit tests for MutantExecutor
- ❌ NO integration tests that verify end-to-end mutation generation
- ❌ NO tests using real Rust files to validate mutant generation

**Recommendation**: Add integration test that runs mutation on sample .rs file and asserts `mutants.len() > 0`

## Immediate Action Items

1. **Fix AST-based operator detection**
   - Debug why RustAdapter isn't visiting AST nodes correctly
   - Ensure syn AST traversal reaches all expressions
   - Add logging to operator apply() methods

2. **Implement Missing Operators**
   - **Return Value Replacement** (like cargo-mutants)
   - **Function Body Deletion** (replace function with default return)
   - **Negation Removal** (delete `!`)

3. **Add Integration Tests**
   ```rust
   #[test]
   fn test_generate_mutants_on_real_file() {
       let source = r#"
           fn validate(x: bool) -> bool {
               !x
           }
       "#;
       let mutants = generate_mutants(source);
       assert!(mutants.len() > 0, "Should generate at least one mutant");
   }
   ```

4. **Regression Test with pforge**
   - Add pforge/validator.rs as test fixture
   - Assert PMAT finds ≥4 mutants (matching cargo-mutants)

## Comparison Philosophy

### cargo-mutants Approach
- **Function-level mutations**: Replace entire function bodies
- **Broad coverage**: Catches high-level logic errors
- **Simple implementation**: Text-based replacements
- **High value**: Each mutant tests overall function correctness

### PMAT Approach
- **Expression-level mutations**: Modify individual operators
- **Fine-grained**: Tests specific code paths
- **Complex implementation**: AST-based transformations
- **High precision**: Each mutant tests specific logic branch

**Both are valuable!** cargo-mutants finds coarse-grained issues, PMAT should find fine-grained issues.

## Conclusion

**Benchmark Result**: cargo-mutants is the clear winner for actual mutation testing on pforge.

**PMAT Status**:
- ✅ Test execution infrastructure works (MutantExecutor)
- ❌ Mutation generation completely broken (0 mutants on real code)
- ❌ Cannot benchmark performance (no mutants to execute)
- ❌ v2.130.0 is non-functional for real-world use

**Next Steps**:
1. Fix mutation operator detection
2. Add integration tests
3. Re-run benchmark on pforge
4. Update GitHub Issue #63 with honest status

**Recommendation**: Downgrade issue #63 status from "RESOLVED" to "IN PROGRESS" until mutants are actually generated.

---

**Lesson Learned**: Always test on external projects, not just your own codebase. We discovered this bug only by testing on pforge. Testing PMAT on PMAT would never have caught this issue.
