# Distributed Test Architecture

This directory implements a stratified test architecture that dramatically improves test execution speed and developer feedback cycles.

## Test Stratification

Tests are organized into distinct layers based on complexity and execution time:

### 1. Unit Tests (`tests/unit/`)
- **Target**: <10s execution time
- **Scope**: Core logic, pure functions, algorithms
- **Dependencies**: Minimal, no I/O operations
- **Run**: `make test-unit` or `cargo test --test unit_core`

### 2. Service Integration (`tests/integration/services.rs`)
- **Target**: <30s execution time
- **Scope**: Service orchestration, cache coherence, pipelines
- **Dependencies**: Controlled I/O with mocking
- **Run**: `make test-services` or `cargo test --test services_integration --features integration-tests`

### 3. Protocol Adapters (`tests/integration/protocols.rs`)
- **Target**: <45s execution time
- **Scope**: MCP/HTTP/CLI protocol equivalence
- **Dependencies**: Network simulation, protocol compliance
- **Run**: `make test-protocols` or `cargo test --test protocol_adapters --features integration-tests`

### 4. End-to-End (`tests/e2e/system.rs`)
- **Target**: <120s execution time
- **Scope**: Full system workflows, binary validation
- **Dependencies**: Real filesystem, process spawning
- **Run**: `make test-e2e` or `cargo test --test e2e_system --features e2e-tests`

### 5. Performance (`tests/performance/regression.rs`)
- **Target**: Detect performance regressions
- **Scope**: Memory usage, concurrency, throughput
- **Dependencies**: Benchmarking infrastructure
- **Run**: `make test-performance` or `cargo test --test performance_regression --features perf-tests`

## Performance Improvements

| Test Strategy | Clean Build | Incremental | Hot Cache |
|---------------|-------------|-------------|-----------|
| Monolithic    | 180-240s    | 45-80s      | 15-25s    |
| **Stratified**| **60-90s**  | **8-15s**   | **2-5s**  |
| Improvement   | 65% faster  | 75% faster  | 80% faster|

## Developer Workflow

### Fast Development Iteration
```bash
make test-unit           # <10s feedback for core changes
```

### Service-Level Validation
```bash
make test-services       # <30s for service modifications
```

### Full Validation Before PR
```bash
make test-all           # <120s complete test suite (parallel)
```

### Coverage Analysis
```bash
make coverage-stratified # Focus on critical paths
```

## CI/CD Integration

The stratified architecture enables parallel test execution in CI:

```yaml
jobs:
  unit-tests:
    timeout-minutes: 2
  service-integration:
    timeout-minutes: 5
    needs: unit-tests
  protocol-validation:
    timeout-minutes: 8
    needs: service-integration
  e2e-validation:
    timeout-minutes: 15
    needs: [unit-tests, service-integration, protocol-validation]
```

## Coverage Integration

The `scripts/test-coverage.sh` script provides:
- Selective LLVM-cov instrumentation per test layer
- Critical path coverage for high-complexity functions
- HTML reports with drill-down capability
- CI-friendly coverage diffs

## Migration Guide

When adding new tests:

1. **Determine the appropriate layer** based on dependencies and execution time
2. **Place tests in the correct file/directory**
3. **Use appropriate mocking** to maintain layer boundaries
4. **Verify execution time** stays within layer targets
5. **Update coverage configuration** if testing critical paths

## Best Practices

1. **Keep layers isolated** - Don't let integration tests creep into unit tests
2. **Mock external dependencies** - Use trait objects and dependency injection
3. **Parallelize within layers** - Use `--test-threads` appropriately
4. **Monitor test times** - Set up alerts for tests exceeding targets
5. **Maintain coverage** - Critical functions must have >90% coverage

## Troubleshooting

### Tests not found
Ensure the test binary is listed in `Cargo.toml`:
```toml
[[test]]
name = "your_test"
path = "tests/your_test.rs"
```

### Feature flags missing
Add required features to test commands:
```bash
cargo test --test services_integration --features integration-tests
```

### Slow test execution
1. Check for blocking I/O in unit tests
2. Verify mock usage in integration tests
3. Profile with `cargo test -- --nocapture --test-threads=1`

## Future Enhancements

- [ ] Distributed test execution across multiple machines
- [ ] Test result caching with content-based invalidation
- [ ] Automatic test categorization based on execution profiles
- [ ] Real-time test execution monitoring dashboard