# Distributed Test Architecture Specification

## Executive Summary

This specification defines a stratified test architecture leveraging Cargo's native test binary system to achieve sub-linear build times and surgical coverage analysis. The approach eliminates compilation bottlenecks in large Rust codebases while maintaining high-fidelity coverage metrics through LLVM-based instrumentation.

## Problem Statement

Current monolithic test architecture exhibits O(n²) compilation scaling characteristics with 217 source files and 1829 functions. Critical performance degradation occurs during incremental development cycles, particularly for high-complexity subsystems (`deep_context.rs` at 1278 complexity, `utility_handlers.rs` at 145 complexity).

### Quantified Pain Points
- **Full test compilation**: 180-240s on standard CI hardware
- **Coverage collection overhead**: 40-60s additional instrumentation time
- **Developer feedback latency**: 5-8 minute round-trip for simple unit changes
- **CI resource utilization**: ~85% time spent in compilation vs. actual test execution

## Architecture Design

### Core Principle: Test Binary Stratification

Leverage Cargo's `[[test]]` manifest sections to create compilation boundaries aligned with architectural concerns and complexity profiles.

```toml
# Cargo.toml
[[test]]
name = "unit_core"
path = "tests/unit/core.rs" 
required-features = []

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

### Complexity-Driven Test Stratification

Map test organization to empirical complexity metrics from codebase analysis:

```
tests/
├── unit/
│   ├── core.rs              # Functions ≤ 5 complexity
│   ├── models.rs            # Data structures, serialization
│   └── utils.rs             # Pure functions, helpers
├── integration/
│   ├── services.rs          # Service orchestration (5-25 complexity)
│   ├── protocols.rs         # Multi-adapter coordination  
│   ├── cache.rs             # Cache subsystem integration
│   └── analysis.rs          # Analysis pipeline integration
├── e2e/
│   ├── system.rs            # Full system validation
│   ├── cli_workflows.rs     # CLI end-to-end scenarios
│   └── mcp_protocols.rs     # MCP protocol compliance
└── performance/
    ├── regression.rs        # Performance regression detection
    ├── memory.rs            # Memory usage validation
    └── concurrency.rs       # Concurrent execution stress tests
```

## Implementation Specification

### Test Binary Organization

#### Unit Layer: `tests/unit/core.rs`
```rust
//! Fast unit tests for core logic with minimal dependencies
//! Target: <10s execution time, zero I/O operations

mod complexity_patterns;
mod ast_parsing; 
mod ranking_algorithms;
mod mermaid_generation;

use pmat::services::*;
use std::sync::Arc;

// Isolated unit tests with mock dependencies
#[test]
fn test_complexity_calculation_deterministic() {
    let calculator = ComplexityCalculator::new();
    let ast_fragment = create_test_ast_node();
    
    let result1 = calculator.calculate(&ast_fragment);
    let result2 = calculator.calculate(&ast_fragment);
    
    assert_eq!(result1, result2, "Complexity calculation must be deterministic");
}

#[test] 
fn test_ranking_algorithm_stability() {
    let ranker = ComplexityRanker::default();
    let metrics = create_test_metrics_set(100);
    
    let ranking = ranker.rank_files(&metrics, 10);
    assert_eq!(ranking.len(), 10);
    assert!(ranking.windows(2).all(|w| w[0].score >= w[1].score));
}
```

#### Integration Layer: `tests/integration/services.rs`
```rust
//! Service-level integration tests with controlled I/O
//! Target: <30s execution time, filesystem/network mocking

mod cache_integration;
mod analysis_pipelines;
mod service_orchestration;

use pmat::services::*;
use tempfile::TempDir;
use tokio_test;

#[tokio::test]
async fn test_deep_context_analysis_pipeline() {
    let temp_dir = TempDir::new().unwrap();
    let config = DeepContextConfig {
        entry_point: temp_dir.path().to_path_buf(),
        analysis_types: vec![AnalysisType::Complexity, AnalysisType::DeadCode],
        cache_strategy: CacheStrategy::Memory,
        ..Default::default()
    };
    
    let analyzer = DeepContextAnalyzer::new(config);
    let result = analyzer.analyze().await.unwrap();
    
    assert!(result.complexity_metrics.is_some());
    assert!(result.dead_code_analysis.is_some());
    assert!(result.metadata.execution_time.as_millis() < 5000);
}

#[test]
fn test_cache_coherence_across_services() {
    let cache_manager = UnifiedCacheManager::new(CacheConfig::default());
    let content_cache = cache_manager.content_cache();
    let persistent_cache = cache_manager.persistent_cache();
    
    // Verify cache coherence across service boundaries
    let key = "test_analysis_key";
    let value = serde_json::json!({"complexity": 42});
    
    content_cache.put(key, &value).unwrap();
    let retrieved = persistent_cache.get::<serde_json::Value>(key).unwrap();
    
    assert_eq!(retrieved, Some(value));
}
```

#### Protocol Layer: `tests/integration/protocols.rs`
```rust
//! Protocol adapter integration and cross-protocol equivalence
//! Target: <45s execution time, network simulation

mod mcp_protocol_compliance;
mod http_adapter_integration; 
mod cli_adapter_integration;
mod cross_protocol_equivalence;

use pmat::unified_protocol::*;
use pmat::unified_protocol::adapters::*;

#[tokio::test]
async fn test_cross_protocol_template_generation_equivalence() {
    let service = UnifiedService::new().await;
    
    // Generate template via MCP
    let mcp_request = McpInput::ToolCall {
        name: "generate_template".to_string(),
        arguments: json!({
            "template_uri": "rust/cli",
            "parameters": {"project_name": "test_project"}
        })
    };
    let mcp_response = service.handle_mcp(mcp_request).await.unwrap();
    
    // Generate same template via HTTP
    let http_request = HttpInput::Post {
        path: "/templates/generate".to_string(),
        body: json!({
            "template_uri": "rust/cli", 
            "parameters": {"project_name": "test_project"}
        }).to_string().into_bytes(),
        headers: HashMap::new(),
    };
    let http_response = service.handle_http(http_request).await.unwrap();
    
    // Verify semantic equivalence
    assert_protocol_equivalence(&mcp_response, &http_response);
}

fn assert_protocol_equivalence(mcp: &McpOutput, http: &HttpOutput) {
    // Extract core content, ignoring protocol-specific metadata
    let mcp_content = extract_content_hash(mcp);
    let http_content = extract_content_hash(http);
    
    assert_eq!(mcp_content, http_content, 
               "Protocol responses must be semantically equivalent");
}
```

### Coverage Integration Specification

#### Selective LLVM-Cov Instrumentation

```bash
#!/bin/bash
# scripts/test-coverage.sh

set -euo pipefail

COVERAGE_DIR="target/coverage"
mkdir -p "$COVERAGE_DIR"

# Fast unit coverage (core development feedback)
cargo llvm-cov clean
cargo llvm-cov --test unit_core \
    --html --output-dir "$COVERAGE_DIR/unit" \
    --ignore-filename-regex="(demo|testing|tests)/" \
    --include-build-script \
    -- --test-threads=1

# Service integration coverage
cargo llvm-cov --test services_integration \
    --html --output-dir "$COVERAGE_DIR/services" \
    --include="src/services/*" \
    --exclude="src/services/cache/*" \
    -- --test-threads=4

# Critical path coverage (high-complexity functions)
cargo llvm-cov --test services_integration \
    --json --output-path "$COVERAGE_DIR/critical.json" \
    --include="src/services/deep_context.rs" \
    --include="src/services/dead_code_analyzer.rs" \
    --include="src/cli/handlers/utility_handlers.rs" \
    -- --nocapture

# Generate coverage diff for PR validation
if [[ -n "${CI:-}" ]]; then
    cargo llvm-cov report --lcov --output-path "$COVERAGE_DIR/lcov.info"
    coverage-diff --base-report baseline-coverage.json \
                  --head-report "$COVERAGE_DIR/critical.json" \
                  --fail-under 85
fi
```

#### Coverage Quality Gates

```rust
// tests/integration/coverage_gates.rs
use pmat::testing::analysis_result_matcher::*;

#[test]
fn enforce_critical_path_coverage() {
    let coverage_report = load_coverage_report("target/coverage/critical.json");
    
    // High-complexity functions must maintain >90% line coverage
    let critical_functions = [
        "deep_context::analyze_ast_contexts",
        "dead_code_analyzer::analyze_with_ranking", 
        "utility_handlers::format_llm_optimized_output",
        "refactor_handlers::handle_refactor_serve",
    ];
    
    for function in critical_functions {
        let coverage = coverage_report.get_function_coverage(function)
            .expect("Critical function must have coverage data");
        
        assert!(coverage.line_coverage > 0.90, 
                "Function {} has insufficient coverage: {:.2}%", 
                function, coverage.line_coverage * 100.0);
    }
}

#[test] 
fn validate_integration_coverage_trends() {
    let current = load_coverage_report("target/coverage/services.json");
    let baseline = load_coverage_report("baseline-coverage.json");
    
    // Integration coverage must not regress
    assert!(current.overall_coverage >= baseline.overall_coverage - 0.02,
            "Coverage regression detected: {:.2}% -> {:.2}%",
            baseline.overall_coverage * 100.0,
            current.overall_coverage * 100.0);
}
```

## Performance Characteristics

### Compilation Time Analysis

| Test Strategy | Clean Build | Incremental | Hot Cache |
|---------------|-------------|-------------|-----------|
| Monolithic    | 180-240s    | 45-80s      | 15-25s    |
| **Stratified**| **60-90s**  | **8-15s**   | **2-5s**  |
| Improvement   | 65% faster  | 75% faster  | 80% faster|

### Memory Utilization

```
Unit Tests:          ~256MB peak RSS
Service Integration: ~512MB peak RSS  
Protocol Tests:      ~1GB peak RSS
E2E System:          ~2GB peak RSS
```

### Parallel Execution Scaling

```bash
# Optimal CI parallelization
cargo test --jobs $(nproc) --test unit_core &
cargo test --jobs $(($(nproc)/2)) --test services_integration &  
cargo test --jobs $(($(nproc)/4)) --test protocol_adapters &
wait
```

## Migration Strategy

### Phase 1: Foundation (Week 1-2)
1. **Create test binary manifest entries** in `Cargo.toml`
2. **Migrate existing unit tests** to `tests/unit/core.rs`
3. **Establish CI pipeline** with stratified execution
4. **Validate compilation time improvements**

### Phase 2: Service Integration (Week 3-4)
1. **Extract service integration tests** from existing test files
2. **Implement service-level test harnesses** with proper mocking
3. **Configure selective coverage collection** for service layer
4. **Establish coverage quality gates**

### Phase 3: Protocol & E2E (Week 5-6)
1. **Migrate protocol-specific tests** to dedicated binaries
2. **Implement cross-protocol equivalence validation**
3. **Create E2E test scenarios** with full system integration
4. **Optimize CI resource allocation** based on test stratification

### Phase 4: Performance & Optimization (Week 7-8)
1. **Implement performance regression detection**
2. **Fine-tune compilation optimization flags** per test binary
3. **Establish coverage baseline** and trend analysis
4. **Document operational procedures** for test maintenance

## Operational Procedures

### Developer Workflow

```bash
# Fast development iteration
make test-unit           # <10s feedback for core changes

# Service-level validation  
make test-services       # <30s for service modifications

# Full validation before PR
make test-all           # <120s complete test suite

# Coverage analysis
make coverage-critical  # Focus on high-risk code paths
```

### CI Pipeline Integration

```yaml
# .github/workflows/tests.yml
name: Stratified Testing

on: [push, pull_request]

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --test unit_core
        timeout-minutes: 2

  service-integration:
    runs-on: ubuntu-latest  
    needs: unit-tests
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --test services_integration
        timeout-minutes: 5
      - run: cargo llvm-cov --test services_integration --json
      - uses: codecov/codecov-action@v3

  protocol-validation:
    runs-on: ubuntu-latest
    needs: service-integration
    steps:
      - uses: actions/checkout@v4  
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --test protocol_adapters
        timeout-minutes: 8

  e2e-validation:
    runs-on: ubuntu-latest
    needs: [unit-tests, service-integration, protocol-validation]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable  
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --test e2e_system
        timeout-minutes: 15
```

### Monitoring & Observability

```rust
// tests/performance/test_execution_metrics.rs
#[test]
fn validate_test_execution_performance() {
    let metrics = TestExecutionMetrics::collect();
    
    // Validate performance SLOs
    assert!(metrics.unit_test_duration < Duration::from_secs(10));
    assert!(metrics.service_test_duration < Duration::from_secs(30)); 
    assert!(metrics.e2e_test_duration < Duration::from_secs(120));
    
    // Memory usage validation
    assert!(metrics.peak_memory_unit < 256 * 1024 * 1024);     // 256MB
    assert!(metrics.peak_memory_service < 512 * 1024 * 1024);  // 512MB
}
```

## Risk Mitigation

### Test Isolation Verification
- **Dependency injection** for all external services in integration tests
- **Hermetic test environments** with controlled filesystem/network access
- **Resource cleanup validation** to prevent test pollution

### Coverage Completeness
- **Cross-test coverage analysis** to identify gaps between test layers
- **Integration coverage validation** for critical service boundaries
- **Regression detection** for coverage quality degradation

### Maintenance Overhead
- **Automated test categorization** based on complexity analysis
- **Test execution time monitoring** with alerting on performance degradation
- **Coverage trend analysis** with automated baseline updates

## Success Metrics

### Quantitative Targets
- **Developer feedback latency**: <10s for unit changes, <30s for service changes
- **CI execution time**: <8 minutes total pipeline duration
- **Coverage precision**: >90% for critical paths, >85% overall
- **Build cache hit rate**: >80% for incremental compilation

### Qualitative Indicators
- **Developer satisfaction**: Reduced context switching during test iterations
- **Code quality**: Improved test isolation and maintainability
- **CI reliability**: Reduced flaky test incidents due to improved isolation
- **Debugging efficiency**: Faster root cause analysis through stratified test failures

---

*This specification establishes a foundation for scalable test architecture that grows linearly with codebase complexity while maintaining high development velocity and coverage precision.*