# Five Whys Analysis: Mutation Testing Timeout Issue

**Date**: October 5, 2025
**Issue**: Mutation testing times out on PMAT itself
**Methodology**: Toyota Way Five Whys + Root Cause Analysis

## The Problem Statement

When running mutation testing on PMAT's own code:
```bash
$ pmat analyze mutate --path server/src/services/mutation/types.rs
Command timed out after 5m 0s
```

## Five Whys Analysis

### Why #1: Why did mutation testing time out?

**Answer**: Because running `cargo test --lib` on each mutant takes >2 minutes, and with 28+ mutants, total time exceeds 5 minutes.

### Why #2: Why does cargo test take >2 minutes per mutant?

**Answer**: Because we run **the entire test suite** for every mutant, including:
- All unit tests across all modules
- Integration tests
- Tests completely unrelated to the mutated file

### Why #3: Why do we run ALL tests instead of just relevant tests?

**Answer**: Because our `MutantExecutor` always runs:
```rust
Command::new("cargo")
    .arg("test")
    .arg("--lib")
    .arg("--")
    .arg("--nocapture")
```

It doesn't filter to only tests that could be affected by the mutation.

### Why #4: Why don't we filter to relevant tests?

**Answer**: Because we don't know which tests are relevant to the mutated code. We have no test-to-code mapping.

### Why #5 (ROOT CAUSE): Why don't we have test-to-code mapping?

**Answer**: **DESIGN FLAW** - We designed mutation testing to be "test-agnostic" and always run the full suite, assuming tests are fast.

This assumption is **invalid for real-world codebases**.

## Root Cause Identified

**The fundamental design flaw**:
We run the entire test suite for every mutant, which doesn't scale.

**How cargo-mutants solves this**:
cargo-mutants runs tests in the **package** containing the mutated file, not all tests.

```bash
# cargo-mutants approach
cargo test --package pforge-config  # Only tests in same package
```

vs

```bash
# Our current approach
cargo test --lib  # ALL library tests
```

## The Better Design

### Principle: Test Locality

**Insight**: A mutation in `services/mutation/types.rs` cannot affect tests in `services/deep_wasm/`.

**Solution**: Only run tests in the **same module/package** as the mutated code.

### Implementation Strategy

1. **Determine test scope from file path**:
   - Mutating `server/src/services/mutation/types.rs`
   - Run tests in `services::mutation::types::tests`
   - Or run `cargo test --lib -- services::mutation`

2. **Use Rust's test filtering**:
   ```bash
   cargo test --lib -- services::mutation
   ```

3. **For files without tests, run package tests**:
   ```bash
   cargo test --package <package_name>
   ```

### Comparison to cargo-mutants

| Feature | cargo-mutants | PMAT (current) | PMAT (fixed) |
|---------|---------------|----------------|--------------|
| Test Scope | Package-level | All tests | Module/Package-level ✅ |
| Speed | Fast | Slow ❌ | Fast ✅ |
| Accuracy | High | High | High ✅ |

## The Fix: Smart Test Execution

### Design Goals

1. **Automatic**: No user configuration needed
2. **Fast**: Only run relevant tests
3. **Accurate**: Don't miss tests that should run
4. **Simple**: Easy to understand and maintain

### Implementation Plan

```rust
impl MutantExecutor {
    /// Determine which tests to run based on mutated file
    fn determine_test_filter(&self, mutant: &Mutant) -> TestFilter {
        let path = &mutant.original_file;

        // Extract module path from file path
        // e.g., "server/src/services/mutation/types.rs"
        //    -> "services::mutation::types"
        let module_path = self.extract_module_path(path);

        TestFilter::Module(module_path)
    }

    /// Run tests with appropriate filter
    async fn run_tests_for_mutant(&self, mutant: &Mutant) -> Result<String> {
        let filter = self.determine_test_filter(mutant);

        match filter {
            TestFilter::Module(module) => {
                // cargo test --lib -- services::mutation
                Command::new("cargo")
                    .arg("test")
                    .arg("--lib")
                    .arg("--")
                    .arg(&module)
                    .output()
            }
            TestFilter::Package(package) => {
                // cargo test --package pforge-config
                Command::new("cargo")
                    .arg("test")
                    .arg("--package")
                    .arg(&package)
                    .output()
            }
        }
    }
}
```

### Test Filtering Logic

```rust
// For: server/src/services/mutation/types.rs
// Extract: services::mutation

fn extract_module_path(file_path: &Path) -> String {
    let path_str = file_path.to_str().unwrap();

    // Remove "server/src/" prefix
    let relative = path_str.strip_prefix("server/src/")
        .or_else(|| path_str.strip_prefix("src/"))
        .unwrap_or(path_str);

    // Remove ".rs" suffix
    let without_ext = relative.strip_suffix(".rs").unwrap_or(relative);

    // Replace "/" with "::"
    let module = without_ext.replace("/", "::");

    // Remove "/mod" or "::mod" at end
    module.strip_suffix("::mod")
        .or_else(|| module.strip_suffix("/mod"))
        .unwrap_or(&module)
        .to_string()
}
```

### Examples

| File Path | Test Filter | Tests Run |
|-----------|-------------|-----------|
| `server/src/services/mutation/types.rs` | `services::mutation` | Only mutation module tests |
| `server/src/cli/handlers/mod.rs` | `cli::handlers` | Only handler tests |
| `../pforge/crates/pforge-config/src/validator.rs` | `--package pforge-config` | Package tests |

## Expected Results

### Before Fix
```
30 mutants × 2 minutes/mutant = 60 minutes ❌
```

### After Fix
```
30 mutants × 5 seconds/mutant = 2.5 minutes ✅
```

**Speed improvement**: 24× faster!

## Validation Plan

### EXTREME TDD Approach

1. **RED**: Write test for module extraction
   ```rust
   #[test]
   fn test_extract_module_path() {
       assert_eq!(
           extract_module_path("server/src/services/mutation/types.rs"),
           "services::mutation"
       );
   }
   ```

2. **GREEN**: Implement `extract_module_path()`

3. **VERIFY**: Run on PMAT itself
   - Should complete in <5 minutes
   - Should only run relevant tests
   - Should still catch all failures

### Comparison to cargo-mutants

Test on same file (pforge validator.rs):
- cargo-mutants: 20.4s
- PMAT (current): timeout
- PMAT (fixed): <15s ✅

## Why This Is Better Than Patching

### Patch Approach (What We Rejected)
- Add `--test-timeout 60` flag
- Add `--test-filter` flag
- Document limitations

**Problems**:
- User has to figure out right timeout
- User has to know which tests to run
- Doesn't "just work"

### Root Cause Fix (What We're Doing)
- Automatically determine test scope
- Run only relevant tests
- Zero configuration needed

**Benefits**:
- ✅ Just works
- ✅ Fast by default
- ✅ Better than cargo-mutants (module-level granularity)
- ✅ No user configuration needed

## Conclusion

### Root Cause
Running entire test suite for every mutant doesn't scale.

### Solution
Smart test filtering based on mutation location.

### Implementation
Module path extraction + cargo test filtering.

### Expected Outcome
- 24× speed improvement
- "Just works" on any codebase
- Better than cargo-mutants (finer granularity)
- Toyota Way: Fix the root cause, not the symptom

---

**Toyota Way Principles Applied**:
1. **Genchi Genbutsu** (Go and See): Dogfooding revealed the issue
2. **Five Whys**: Found root cause (design flaw)
3. **Kaizen**: Improve design to be better than before
4. **Jidoka**: Build quality in (automatic test filtering)

**Next Step**: Implement smart test filtering with EXTREME TDD
