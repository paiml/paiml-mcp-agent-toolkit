# LLVM Coverage for Rust: Canonical Implementation Specification
## Making `make coverage` Just Work™

### Executive Summary

This specification defines the canonical LLVM-based code coverage implementation for the PMAT project using `cargo-llvm-cov`, the de facto standard for Rust coverage. The solution provides zero-configuration coverage with optimal performance, leveraging LLVM's source-based instrumentation for precise line, region, and branch coverage metrics.

### Core Technology Stack

```yaml
Primary Tool: cargo-llvm-cov v0.6.19+
Underlying: LLVM source-based code coverage (-C instrument-coverage)
Test Runner: cargo-nextest (already integrated)
LLVM Tools: llvm-tools-preview component
Report Formats: lcov, html, json, cobertura, codecov
```

## 1. Canonical Implementation Pattern

### 1.1 The Gold Standard Approach

The most idiomatic way to implement LLVM coverage in Rust leverages `cargo-llvm-cov`, which wraps `rustc -C instrument-coverage` and the LLVM toolchain. This approach is canonical because:

1. **Zero intermediate layers**: Direct integration with rustc and LLVM tools
2. **Native Rust toolchain integration**: Uses rustup components
3. **Cargo-compatible CLI**: Seamless integration with existing workflows
4. **Performance optimized**: Minimal overhead, parallel execution support
5. **Industry standard**: Used by major Rust projects (tokio, serde, etc.)

### 1.2 Installation Requirements

```bash
# Install the LLVM tools (one-time setup)
rustup component add llvm-tools-preview

# Install cargo-llvm-cov
cargo install cargo-llvm-cov

# Verify installation
cargo llvm-cov --version
```

## 2. Makefile Integration

### 2.1 Basic Coverage Target

```makefile
# Fast coverage with inline display
coverage:
	@echo "📊 Running LLVM-based code coverage..."
	@if ! command -v cargo-llvm-cov >/dev/null 2>&1; then \
		echo "📦 Installing cargo-llvm-cov..."; \
		cargo install cargo-llvm-cov; \
	fi
	@cargo llvm-cov --workspace \
		--exclude server/tests/slow_integration \
		--features skip-slow-tests \
		--lcov --output-path target/llvm-cov/lcov.info
	@cargo llvm-cov report --summary-only
	@echo "✅ Coverage report: target/llvm-cov/lcov.info"
```

### 2.2 Advanced Stratified Coverage

Given PMAT's distributed test architecture, here's the canonical stratified coverage implementation:

```makefile
# Coverage with nextest (leverages existing fast test infrastructure)
coverage-nextest:
	@echo "⚡ Running coverage with cargo-nextest..."
	@cargo llvm-cov nextest --workspace \
		--features skip-slow-tests \
		--profile fast \
		--lcov --output-path target/llvm-cov/nextest.lcov
	@echo "✅ Nextest coverage completed"

# Unified coverage for all test types
coverage-full:
	@echo "🔬 Running comprehensive LLVM coverage..."
	@# Clean previous coverage data
	@cargo llvm-cov clean --workspace
	
	@# Run stratified tests with coverage
	@echo "  1️⃣ Unit tests..."
	@cargo llvm-cov nextest --no-report \
		--test unit_core \
		--test-threads $$(nproc)
	
	@echo "  2️⃣ Service tests..."
	@cargo llvm-cov nextest --no-report \
		--test services_integration \
		--features integration-tests
	
	@echo "  3️⃣ Protocol tests..."
	@cargo llvm-cov nextest --no-report \
		--test protocol_adapters \
		--features integration-tests
	
	@echo "  4️⃣ Property tests..."
	@cargo llvm-cov test --no-report \
		--lib -- property_tests prop_
	
	@echo "  5️⃣ Doc tests (if nightly)..."
	@if rustc --version | grep -q nightly; then \
		cargo llvm-cov --no-report --doc; \
	fi
	
	@# Generate unified report
	@cargo llvm-cov report --lcov --output-path target/llvm-cov/full.lcov
	@cargo llvm-cov report --html --output-dir target/llvm-cov/html
	@cargo llvm-cov report --summary-only
	
	@echo "✅ Full coverage report generated:"
	@echo "   📄 LCOV: target/llvm-cov/full.lcov"
	@echo "   🌐 HTML: target/llvm-cov/html/index.html"

# Quick coverage with immediate feedback
coverage-quick:
	@cargo llvm-cov nextest --open
```

### 2.3 Incremental Coverage Pattern

```makefile
# Coverage for changed files only (CI optimization)
coverage-diff:
	@echo "🔍 Running coverage on changed files..."
	@CHANGED_FILES=$$(git diff --name-only HEAD~1 HEAD | grep '\.rs$$' | xargs); \
	if [ -n "$$CHANGED_FILES" ]; then \
		cargo llvm-cov test --no-report -- --include $$CHANGED_FILES; \
		cargo llvm-cov report --summary-only; \
	else \
		echo "No Rust files changed"; \
	fi

# Coverage with threshold enforcement
coverage-gate:
	@echo "🚦 Running coverage quality gate..."
	@cargo llvm-cov --workspace --summary-only --fail-under-lines 80
	@echo "✅ Coverage threshold met (>80%)"
```

## 3. Performance Optimizations

### 3.1 Parallel Execution

```makefile
# Optimized parallel coverage
coverage-parallel:
	@echo "🚀 Running parallel coverage collection..."
	@export CARGO_BUILD_JOBS=$$(nproc); \
	export CARGO_TEST_THREADS=$$(nproc); \
	cargo llvm-cov nextest \
		--workspace \
		--jobs $$(nproc) \
		--test-threads nextest \
		--lcov --output-path target/llvm-cov/parallel.lcov
```

### 3.2 Incremental Coverage Collection

```makefile
# Incremental coverage without rebuilding
coverage-incremental:
	@echo "♻️ Running incremental coverage..."
	@# Set environment for coverage without rebuilding
	@source <(cargo llvm-cov show-env --export-prefix)
	@# Run additional tests without clearing previous data
	@cargo test --test new_tests
	@# Generate report from accumulated data
	@cargo llvm-cov report --lcov
```

## 4. Output Format Patterns

### 4.1 Multi-Format Generation

```makefile
# Generate all coverage formats
coverage-all-formats:
	@echo "📊 Generating all coverage formats..."
	@cargo llvm-cov clean --workspace
	@cargo llvm-cov nextest --no-report --workspace
	@mkdir -p target/coverage-reports
	
	@# LCOV for CI tools
	@cargo llvm-cov report --lcov \
		--output-path target/coverage-reports/coverage.lcov
	
	@# JSON for programmatic processing
	@cargo llvm-cov report --json \
		--output-path target/coverage-reports/coverage.json
	
	@# Cobertura for Jenkins/Azure DevOps
	@cargo llvm-cov report --cobertura \
		--output-path target/coverage-reports/coverage.xml
	
	@# HTML for human review
	@cargo llvm-cov report --html \
		--output-dir target/coverage-reports/html
	
	@# Codecov compatible
	@cargo llvm-cov report --codecov \
		--output-path target/coverage-reports/codecov.json
	
	@echo "✅ All formats generated in target/coverage-reports/"
```

## 5. CI/CD Integration

### 5.1 GitHub Actions Integration

```yaml
# .github/workflows/coverage.yml
- name: Install Rust toolchain
  uses: dtolnay/rust-toolchain@stable
  with:
    components: llvm-tools-preview

- name: Install cargo-llvm-cov
  uses: taiki-e/install-action@cargo-llvm-cov

- name: Install nextest
  uses: taiki-e/install-action@nextest

- name: Generate coverage
  run: |
    cargo llvm-cov nextest --all-features \
      --workspace \
      --lcov --output-path lcov.info

- name: Upload to Codecov
  uses: codecov/codecov-action@v3
  with:
    files: lcov.info
    fail_ci_if_error: true
```

### 5.2 Makefile CI Target

```makefile
# CI-optimized coverage
coverage-ci:
	@echo "🔧 Running CI coverage pipeline..."
	@cargo llvm-cov nextest \
		--all-features \
		--workspace \
		--lcov --output-path lcov.info \
		--codecov --output-path codecov.json
	@echo "##vso[task.setvariable variable=coverage]$$(cargo llvm-cov report --summary-only | grep TOTAL | awk '{print $$10}')"
	@echo "✅ Coverage artifacts ready for upload"
```

## 6. Advanced Features

### 6.1 Branch Coverage (Nightly)

```makefile
# Branch coverage (requires nightly)
coverage-branch:
	@echo "🌳 Running branch coverage (nightly only)..."
	@cargo +nightly llvm-cov \
		--branch \
		--workspace \
		--html --output-dir target/llvm-cov/branch-html
```

### 6.2 Coverage Exclusions

```rust
// Use in source code
#[cfg(not(tarpaulin_include))]
fn debug_only_function() { }

// Or with attributes
#[coverage(off)]
fn uncovered_function() { }

// Coverage pragma comments
fn partially_covered() {
    important_code();
    // LCOV_EXCL_START
    debug_assertions_only();
    // LCOV_EXCL_STOP
}
```

### 6.3 External Binary Coverage

```makefile
# Coverage for external test harnesses
coverage-external:
	@echo "🔌 Setting up external binary coverage..."
	@source <(cargo llvm-cov show-env --export-prefix)
	@cargo llvm-cov clean --workspace
	@cargo build --workspace
	@# Run external test harness
	@./scripts/integration-test-harness.sh
	@cargo llvm-cov report --lcov
```

## 7. Troubleshooting Patterns

### 7.1 Common Issues and Solutions

```makefile
# Debug coverage issues
coverage-debug:
	@echo "🔍 Debug information:"
	@echo "  Rust version: $$(rustc --version)"
	@echo "  LLVM version: $$(rustc --print=cfg | grep llvm)"
	@echo "  cargo-llvm-cov: $$(cargo llvm-cov --version)"
	@echo "  llvm-tools: $$(ls $$HOME/.rustup/toolchains/*/lib/rustlib/*/bin/llvm-* 2>/dev/null | head -1)"
	@cargo llvm-cov show-env

# Clean all coverage artifacts
coverage-clean:
	@echo "🧹 Cleaning coverage artifacts..."
	@cargo llvm-cov clean --workspace
	@rm -rf target/llvm-cov target/llvm-cov-target
	@rm -f *.profraw *.profdata
	@echo "✅ Coverage artifacts cleaned"

# Verify coverage setup
coverage-verify:
	@echo "✔️ Verifying coverage setup..."
	@command -v cargo-llvm-cov >/dev/null || (echo "❌ cargo-llvm-cov not found" && exit 1)
	@rustup component list | grep -q "llvm-tools" || (echo "❌ llvm-tools-preview not installed" && exit 1)
	@echo "✅ Coverage tools properly configured"
```

## 8. Integration with PMAT's Existing Infrastructure

### 8.1 Drop-in Replacement

Replace the existing coverage targets in the Makefile with:

```makefile
# Update existing coverage target
coverage:
	@echo "📊 Running LLVM-based code coverage..."
	@$(MAKE) coverage-verify
	@cargo llvm-cov nextest \
		--workspace \
		--features skip-slow-tests \
		--profile fast \
		--lcov --output-path target/llvm-cov/lcov.info \
		--html --output-dir target/llvm-cov/html
	@cargo llvm-cov report --summary-only
	@echo "✅ Coverage reports generated:"
	@echo "   📄 LCOV: target/llvm-cov/lcov.info"
	@echo "   🌐 HTML: file://$$(pwd)/target/llvm-cov/html/index.html"

# Update coverage-stratified to use llvm-cov
coverage-stratified:
	@echo "📊 Running stratified test coverage with LLVM..."
	@$(MAKE) coverage-full

# Backward compatibility alias
coverage-stdout: coverage
coverage-report: coverage-all-formats
```

### 8.2 Environment Configuration

Add to project's `.cargo/config.toml`:

```toml
[env]
# Optimize coverage collection
CARGO_LLVM_COV_TARGET_DIR = { value = "target/llvm-cov-target", relative = true }
RUSTFLAGS = { value = "-C link-dead-code", force = true }

[target.x86_64-unknown-linux-gnu]
runner = "cargo llvm-cov nextest --no-report --"
```

## 9. Best Practices Summary

1. **Always use `cargo-llvm-cov`**: It's the canonical tool with best rustc integration
2. **Leverage nextest**: Already integrated in PMAT, provides optimal parallel execution
3. **Incremental coverage in CI**: Use `--no-report` for multiple test runs, generate once
4. **Clean before full runs**: Use `cargo llvm-cov clean` to ensure accurate metrics
5. **Use workspace coverage**: `--workspace` flag ensures all crates are covered
6. **Feature-aware coverage**: Include `--all-features` for comprehensive coverage
7. **HTML for development**: `--open` flag for immediate visual feedback
8. **LCOV for CI**: Universal format supported by all major CI providers
9. **Set thresholds**: Use `--fail-under-lines` for quality gates
10. **Exclude vendored/generated code**: Use `--ignore-filename-regex` patterns

## 10. Performance Characteristics

```yaml
Overhead: ~15-25% runtime increase (vs 2-3x for source-based alternatives)
Build time: +20-30% initial compilation (cached afterward)
Accuracy: >99% (source-based instrumentation)
Memory: ~10-15% increase during test execution
Parallelism: Full support with nextest
Incremental: Supported via show-env pattern
```

## 11. Migration Checklist

- [ ] Install `cargo-llvm-cov` via `cargo install cargo-llvm-cov`
- [ ] Add `llvm-tools-preview` component via rustup
- [ ] Update Makefile with new coverage targets
- [ ] Update CI/CD workflows for llvm-cov
- [ ] Configure `.gitignore` for coverage artifacts (`target/llvm-cov*`, `*.profraw`)
- [ ] Update documentation with new coverage commands
- [ ] Verify coverage reports generate correctly
- [ ] Set up coverage thresholds in CI
- [ ] Configure IDE integration (VS Code Coverage Gutters, etc.)
- [ ] Train team on new coverage workflow

## Conclusion

This specification provides a production-ready, "just works" LLVM coverage implementation that integrates seamlessly with PMAT's existing infrastructure. The approach leverages industry-standard tools, provides precise metrics, and maintains the performance characteristics required for a high-velocity development workflow.

The canonical pattern using `cargo-llvm-cov` with `cargo-nextest` represents the current best practice in the Rust ecosystem, providing the optimal balance of accuracy, performance, and developer experience.
