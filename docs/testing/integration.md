# Integration Testing Guide

*Reference: SPECIFICATION.md Section 29*

## Overview

PMAT's integration tests verify end-to-end functionality across all interfaces (CLI, MCP, HTTP) ensuring consistent behavior and quality enforcement.

## Test Categories

### 1. CLI Integration Tests

Located in `tests/cli_comprehensive_integration.rs`

Tests verify:
- **Command execution**: All CLI commands work correctly
- **Exit codes**: Proper exit codes on success/failure
- **Output formats**: JSON, SARIF, Markdown formatting
- **File operations**: Reading, analyzing, refactoring files

### 2. MCP Server Integration

Located in `tests/mcp_server_integration.rs`

Tests cover:
- **Tool discovery**: All tools properly registered
- **Request/response**: JSON-RPC protocol compliance
- **State management**: Stateless operation verification
- **Error handling**: Graceful failure modes

### 3. Quality Gate Integration

Located in `tests/quality_gate_integration.rs`

Validates:
- **Threshold enforcement**: Complexity limits applied
- **SATD detection**: Zero-tolerance policy works
- **Multi-check coordination**: All checks run together
- **Fail-fast behavior**: Stops on first violation

### 4. Refactor Engine Integration

Located in `tests/refactor_auto_property_integration.rs`

Ensures:
- **AST preservation**: Code remains valid after refactoring
- **Quality improvement**: Metrics improve post-refactor
- **Incremental application**: Step-by-step refactoring works

## Running Integration Tests

```bash
# Run all integration tests
make test-integration

# Run specific integration test suite
cargo test --test cli_comprehensive_integration

# Run with verbose output
cargo test --test mcp_server_integration -- --nocapture

# Run integration tests in parallel
cargo nextest run --test-threads 8
```

## Test Infrastructure

### Test Fixtures

Located in `tests/fixtures/`:
- Sample projects with known issues
- Golden files for output comparison
- Configuration files for different scenarios

### Test Utilities

Helper functions in `tests/common/`:
- Project setup/teardown
- Command execution helpers
- Output parsing utilities
- Assertion helpers

## Writing Integration Tests

### Structure

```rust
#[tokio::test]
async fn test_cli_analyze_complexity() {
    // Setup
    let project = TestProject::new("complex_code");
    
    // Execute
    let output = run_pmat(&["analyze", "complexity", "--format", "json"]);
    
    // Verify
    assert_eq!(output.status.code(), Some(0));
    let result: ComplexityResult = serde_json::from_str(&output.stdout)?;
    assert!(result.max_complexity <= 20);
    
    // Cleanup (automatic with Drop)
}
```

### Best Practices

1. **Isolation**: Each test creates its own test project
2. **Determinism**: Use fixed seeds and inputs
3. **Cleanup**: Ensure temporary files are removed
4. **Assertions**: Check both success and failure cases
5. **Documentation**: Explain what scenario is tested

## CI Integration

Integration tests run in CI with:
- **PR checks**: Fast subset of critical tests
- **Main branch**: Full integration suite
- **Nightly**: Extended integration with large projects

Configuration in `.github/workflows/test.yml`

## Performance Considerations

- Tests use `cargo-nextest` for parallel execution
- Shared fixtures are cached between test runs
- Heavy tests marked with `#[ignore]` for manual runs

## Related Documentation

- [Property-Based Testing](./property-based.md)
- [Performance Testing](./performance.md)
- [SPECIFICATION.md Section 29](../SPECIFICATION.md#29-integration-testing)