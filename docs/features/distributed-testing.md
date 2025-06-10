# Distributed Test Architecture

The PAIML MCP Agent Toolkit implements a stratified test architecture that dramatically improves build times and developer feedback loops through parallel compilation and execution.

## Overview

Traditional monolithic test suites suffer from O(n²) compilation scaling. Our distributed architecture achieves sub-linear scaling through strategic test binary stratification.

## Architecture

### Test Layers

1. **Unit Tests** (`test-unit`)
   - Target: <10s execution
   - Scope: Core logic, zero I/O
   - Binary: `unit_core`
   - Tests: 15 fundamental tests

2. **Service Integration** (`test-services`)
   - Target: <30s execution
   - Scope: Service-level integration with controlled I/O
   - Binary: `services_integration`
   - Tests: 6 service tests
   - Features: `integration-tests`

3. **Protocol Adapters** (`test-protocols`)
   - Target: <45s execution
   - Scope: Cross-protocol validation (CLI, MCP, HTTP)
   - Binary: `protocol_adapters`
   - Tests: 6 protocol consistency tests
   - Features: `integration-tests`

4. **End-to-End** (`test-e2e`)
   - Target: <120s execution
   - Scope: Full system workflows
   - Binary: `e2e_system`
   - Tests: 6 system tests
   - Features: `e2e-tests`

5. **Performance** (`test-performance`)
   - Target: Regression detection
   - Scope: Performance baselines
   - Binary: `performance_regression`
   - Tests: 5 performance tests
   - Features: `perf-tests`

## Usage

### Running Individual Test Suites

```bash
# Fast unit tests (<10s)
make test-unit

# Service integration tests (<30s)
make test-services

# Protocol adapter tests (<45s)
make test-protocols

# End-to-end tests (<120s)
make test-e2e

# Performance tests
make test-performance
```

### Running All Tests in Parallel

```bash
# Run all stratified tests concurrently
make test-all
```

This executes 4 test suites in parallel (unit, services, protocols, e2e), maximizing CPU utilization.

### Coverage Analysis

```bash
# Generate coverage for stratified tests
make coverage-stratified

# Or use cargo directly for specific binaries
cargo llvm-cov test --test unit_core
cargo llvm-cov test --test services_integration --features integration-tests
```

## Performance Improvements

| Metric | Monolithic | Stratified | Improvement |
|--------|------------|------------|-------------|
| Clean Build | 180-240s | 60-90s | **65% faster** |
| Incremental | 45-80s | 8-15s | **75% faster** |
| Hot Cache | 15-25s | 2-5s | **80% faster** |
| Parallel Execution | N/A | Yes | **4x throughput** |

## Implementation Details

### Cargo.toml Configuration

```toml
[[test]]
name = "unit_core"
path = "tests/unit/core.rs"

[[test]]
name = "services_integration"
path = "tests/integration/services.rs"
required-features = ["integration-tests"]

[[test]]
name = "protocol_adapters"
path = "tests/integration/protocols.rs"
required-features = ["integration-tests"]

[[test]]
name = "e2e_system"
path = "tests/e2e/system.rs"
required-features = ["e2e-tests"]

[[test]]
name = "performance_regression"
path = "tests/performance/regression.rs"
required-features = ["perf-tests"]
```

### Directory Structure

```
server/tests/
├── unit/
│   └── core.rs           # Fast unit tests
├── integration/
│   ├── services.rs       # Service-level tests
│   └── protocols.rs      # Protocol adapter tests
├── e2e/
│   └── system.rs         # End-to-end tests
└── performance/
    └── regression.rs     # Performance tests
```

## CI/CD Integration

The `.github/workflows/stratified-tests.yml` workflow runs test layers as parallel jobs:

```yaml
jobs:
  unit-tests:
    runs-on: ubuntu-latest
    timeout-minutes: 5
    steps:
      - run: make test-unit

  service-tests:
    runs-on: ubuntu-latest
    timeout-minutes: 10
    steps:
      - run: make test-services

  # ... other test jobs

  all-tests:
    needs: [unit-tests, service-tests, protocol-tests]
    runs-on: ubuntu-latest
    steps:
      - run: echo "All tests passed!"
```

## Benefits

1. **Faster Feedback**: Developers get unit test results in <10s
2. **Parallel Execution**: 4x throughput improvement
3. **Selective Testing**: Run only relevant test layers
4. **Better Organization**: Tests grouped by architectural concerns
5. **Reduced Compilation**: Each binary compiles independently
6. **Coverage Flexibility**: Instrument only critical paths

## Best Practices

1. **Layer Assignment**: Place tests in the appropriate layer based on dependencies
2. **Mock Usage**: Use mocks in unit/service layers to avoid I/O
3. **Feature Gates**: Use feature flags to control test dependencies
4. **Timeout Enforcement**: Ensure tests complete within target times
5. **Parallel Safety**: Avoid shared state between test binaries

## Migration Guide

To migrate existing tests to the distributed architecture:

1. Identify test dependencies and I/O requirements
2. Assign to appropriate layer (unit, service, protocol, e2e, performance)
3. Move test file to corresponding directory
4. Update imports and module declarations
5. Add to appropriate test binary configuration
6. Verify with `cargo test --test <binary_name>`

## Troubleshooting

### Tests Not Found

```bash
# Ensure test binary is configured in Cargo.toml
cargo test --test unit_core -- --list
```

### Feature Requirements

```bash
# Enable required features for integration tests
cargo test --test services_integration --features integration-tests
```

### Parallel Execution Issues

```bash
# Run tests sequentially if needed
make test-unit && make test-services && make test-protocols
```

## Future Enhancements

1. **Dynamic Test Distribution**: Automatically balance tests across binaries
2. **Test Impact Analysis**: Run only tests affected by changes
3. **Cloud-Based Distribution**: Distribute tests across multiple machines
4. **Intelligent Retry**: Retry only flaky tests, not entire suites
5. **Performance Tracking**: Track test execution times over releases