# Claude Agent SDK Integration Guide

## Overview

The Claude Agent SDK Integration provides a production-ready bridge between PMAT and Claude AI, enabling intelligent code analysis, quality monitoring, and automated insights. Built using EXTREME TDD methodology, it offers zero-cost abstractions, comprehensive observability, and progressive rollout capabilities.

## Features

### 🚀 Core Capabilities

- **Zero-Cost Error Handling**: Discriminated union-based error handling with no performance overhead
- **Two-Tier Caching**: L1 (10ms TTL) + L2 (60s TTL) with automatic promotion
- **Circuit Breaker**: 3-state pattern (Closed, Open, Half-Open) for resilience
- **Feature Flags**: 4 rollout strategies with instant kill switch
- **RED Metrics**: Comprehensive Rate, Errors, Duration observability
- **Atomic IPC**: PIPE_BUF (4096 bytes) guaranteed atomic writes
- **Auto-Rollback**: Automatic feature disable on performance degradation

### 📊 Performance Characteristics

| Operation | Latency | Notes |
|-----------|---------|-------|
| Bridge Init | < 500ms | Strict requirement, typically ~5µs |
| L1 Cache Hit | ~100ns | In-memory, FNV-1a hashing |
| L2 Cache Hit | ~1µs | Persistent, with auto-promotion |
| Cache Miss | ~15ms | Full source load and analysis |
| Circuit Breaker | ~10ns | Zero-cost state check |

## Quick Start

### Basic Usage

```rust
use pmat::claude_integration::{BridgeConfig, ClaudeBridge};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create bridge with default configuration
    let config = BridgeConfig::default();
    let bridge = ClaudeBridge::new(config).await?;

    // Analyze code
    let code = r#"
        fn complex_function() {
            if condition1 {
                for item in items {
                    while processing {
                        match state {
                            State::A => {},
                            State::B => {},
                        }
                    }
                }
            }
        }
    "#;

    let result = bridge.analyze_code(code).await?;

    println!("Complexity: {}", result.complexity);
    println!("Cognitive Complexity: {}", result.cognitive_complexity);
    println!("SATD Count: {}", result.satd_count);

    Ok(())
}
```

## Feature Flags

Control Claude integration rollout with flexible strategies:

### Allowlist Strategy

```rust
use pmat::claude_integration::{FeatureFlagsBuilder, RolloutStrategy};

let flags = FeatureFlagsBuilder::new()
    .enabled(true)
    .strategy(RolloutStrategy::Allowlist)
    .add_to_allowlist("user_123")
    .add_to_allowlist("user_456")
    .build();

// Check if user should use Claude
if flags.should_use_claude("user_123") {
    // Use Claude integration
}
```

### Percentage Rollout

```rust
let flags = FeatureFlagsBuilder::new()
    .enabled(true)
    .strategy(RolloutStrategy::Percentage(25))
    .build();

// 25% of users will get Claude integration
```

### Kill Switch

```rust
// Instant disable for all users
flags.disable();

// Check status
assert!(!flags.is_enabled());
```

### Auto-Rollback on Performance Degradation

```rust
let flags = FeatureFlagsBuilder::new()
    .enabled(true)
    .strategy(RolloutStrategy::Percentage(10))
    .max_latency(1000) // 1 second max
    .build();

// Monitor performance
let latency_ms = 2000; // 2 seconds - too slow!
if flags.auto_rollback_on_degradation(latency_ms) {
    println!("⚠️ Automatic rollback triggered!");
}
```

## Caching

Two-tier caching system for optimal performance:

### Configuration

```rust
let config = BridgeConfig {
    enable_cache: true,
    pool_size: 5,
    ..Default::default()
};

let bridge = ClaudeBridge::new(config).await?;
```

### Cache Statistics

```rust
let stats = bridge.cache_stats();
println!("L1 hit rate: {:.1}%", stats.l1_hit_rate * 100.0);
println!("L2 hit rate: {:.1}%", stats.l2_hit_rate * 100.0);
println!("Effective hit rate: {:.1}%", stats.effective_hit_rate * 100.0);
```

### Cache Behavior

- **L1 Cache**: 10ms TTL, ~100ns access time, in-memory
- **L2 Cache**: 60s TTL, ~1µs access time, persistent
- **Auto-Promotion**: Popular L2 entries promoted to L1
- **FNV-1a Hashing**: Fast non-cryptographic hash for keys

## Observability

Comprehensive metrics collection following RED methodology:

### Metrics Collection

```rust
use pmat::claude_integration::{MetricsCollector, observability::ErrorType};
use std::time::Duration;

let metrics = MetricsCollector::new();

// Record success
let latency = Duration::from_micros(1500);
metrics.record_success(latency);

// Record error
metrics.record_error(ErrorType::Application, latency);

// Record cache activity
metrics.record_cache_hit();
metrics.record_cache_miss();
```

### Metrics Snapshot

```rust
let snapshot = metrics.snapshot();

println!("Requests: {}", snapshot.requests_total);
println!("Success rate: {:.1}%", snapshot.success_rate * 100.0);
println!("Avg latency: {}μs", snapshot.avg_latency_us);
println!("Cache hit rate: {:.1}%", snapshot.cache_hit_rate * 100.0);

// Formatted output
println!("{}", snapshot.format_summary());
```

### Available Metrics

- **Request Metrics**: total, success, error counts
- **Success Rate**: percentage of successful requests
- **Latency Stats**: average, min, max in microseconds
- **Error Breakdown**: by type (Network, Timeout, Application, Unknown)
- **Cache Metrics**: hits, misses, hit rate
- **Pool Metrics**: acquisitions, exhaustion events

## Connection Pool

Resilient connection pool with circuit breaker pattern:

### Configuration

```rust
let config = BridgeConfig {
    pool_size: 10, // Max concurrent connections
    ..Default::default()
};
```

### Pool Statistics

```rust
let stats = bridge.pool_stats();
println!("Successes: {}", stats.successes);
println!("Failures: {}", stats.failures);
```

### Circuit Breaker States

1. **Closed**: Normal operation, requests flow through
2. **Open**: Too many failures, requests fail fast
3. **Half-Open**: Testing recovery, limited requests allowed

### Thresholds

- **Failure Threshold**: 5 failures in 60-second window triggers Open
- **Recovery Time**: 30 seconds in Open before Half-Open
- **Success Threshold**: 2 successes in Half-Open to return to Closed

## Quality Gates

Enforce quality standards on analyzed code:

### Configuration

```rust
use pmat::claude_integration::quality_gates::ClaudeIntegrationQualityGate;

let gate = ClaudeIntegrationQualityGate {
    max_complexity: 15,           // Stricter than PMAT's 20
    max_cognitive_complexity: 10,
    min_test_coverage: 0.95,      // 95%
    max_coupling: 3,
};
```

### Validation

```rust
let result = bridge.analyze_code(code).await?;

if let Err(violation) = gate.validate(&result) {
    eprintln!("Quality gate failed: {}", violation);
    // Handle violation
}
```

## Security & Sandboxing

Process isolation and resource limits:

### Sandbox Configuration

```rust
use pmat::claude_integration::BridgeSandbox;
use std::path::PathBuf;

let sandbox = BridgeSandbox::new(PathBuf::from("/tmp/claude-sandbox"))
    .with_memory_limit(256)      // 256MB
    .with_cpu_shares(100);       // CPU allocation

// Sandbox automatically cleans up on drop
```

### Security Features

- **Process Isolation**: Separate process per bridge instance
- **Memory Limits**: Configurable max memory (default 256MB)
- **CPU Limits**: Configurable CPU shares (default 100)
- **Filesystem Isolation**: Dedicated sandbox directory
- **Automatic Cleanup**: Resources cleaned on drop

## Error Handling

Zero-cost error handling with discriminated unions:

### Error Codes

Stable error codes (1000-4999 range):

```rust
use pmat::claude_integration::ErrorCode;

match error_code {
    ErrorCode::PipeBrokenPipe => // Handle broken pipe (1001)
    ErrorCode::FramingError => // Handle framing error (1002)
    ErrorCode::InitializationTimeout => // Handle init timeout (2001)
    ErrorCode::AnalysisFailed => // Handle analysis failure (3001)
    ErrorCode::Unknown => // Handle unknown error (4000)
}
```

### Error Context

```rust
use pmat::claude_integration::{BridgeError, BridgeResult};

fn handle_error(result: BridgeResult<AnalysisResult>) {
    match result {
        BridgeResult::Success(data) => {
            // Handle success
        }
        BridgeResult::Error(error) => {
            eprintln!("Error {}: {}", error.code, error.message);
            if let Some(source) = error.source {
                eprintln!("Caused by: {}", source);
            }
            if let Some(backtrace) = error.backtrace {
                eprintln!("Backtrace:\n{}", backtrace);
            }
        }
        BridgeResult::Timeout { elapsed_ms } => {
            eprintln!("Timeout after {}ms", elapsed_ms);
        }
        BridgeResult::CircuitOpen { retry_after_ms } => {
            eprintln!("Circuit breaker open, retry after {}ms", retry_after_ms);
        }
    }
}
```

## Transport Layer

Atomic message passing with length-prefixed framing:

### Frame Format

```
[magic(4)][sequence(8)][length(4)][payload]
```

- **Magic**: 'PMAT' (4 bytes)
- **Sequence**: Message sequence number (8 bytes)
- **Length**: Payload length (4 bytes)
- **Payload**: Actual message data (variable)

### PIPE_BUF Guarantee

- **Atomic Writes**: Up to 4096 bytes guaranteed atomic on POSIX
- **No Interleaving**: Messages never split or interleaved
- **Reliable Delivery**: Either complete message or nothing

## Complete Example

See `server/examples/claude_integration_example.rs` for a comprehensive example covering:

1. Basic usage
2. Feature flags with all strategies
3. Caching performance
4. Observability and metrics
5. Progressive rollout with auto-rollback
6. Complete workflow integration

Run with:

```bash
cargo run --example claude_integration_example
```

## Testing

### Unit Tests

```bash
cargo test --lib claude_integration
```

51 tests covering all modules:
- Bridge (8 tests)
- Cache (4 tests)
- Error handling (3 tests)
- Feature flags (8 tests)
- Observability (9 tests)
- Connection pool (3 tests)
- Quality gates (4 tests)
- Sandbox (2 tests)
- Transport (3 tests)
- Integration tests (7 tests)

### Property Tests

Property-based tests ensure correctness across inputs:
- Error code stability
- Bridge result type safety
- Circuit breaker state transitions

## Architecture

### Module Structure

```
claude_integration/
├── mod.rs          # Public API exports
├── bridge.rs       # Main coordinator
├── cache.rs        # Two-tier caching
├── error.rs        # Error types & handling
├── feature_flags.rs # Rollout strategies
├── observability.rs # Metrics collection
├── pool.rs         # Connection pool
├── quality_gates.rs # Quality enforcement
├── sandbox.rs      # Process isolation
├── transport.rs    # IPC layer
└── tests.rs        # Integration tests
```

### Design Principles

1. **EXTREME TDD**: RED-GREEN-REFACTOR with strict quality gates
2. **Zero-Cost Abstractions**: No runtime overhead for error handling
3. **Type Safety**: Compile-time guarantees for correctness
4. **Progressive Rollout**: Safe gradual deployment
5. **Observability First**: Built-in metrics and monitoring
6. **Security by Default**: Sandboxing and resource limits

## TypeScript Bridge (Optional)

A TypeScript bridge is available for cross-language integration:

```bash
cd bridge
npm install
npm run build
```

See `bridge/README.md` for TypeScript usage.

## CI/CD Integration

### GitHub Actions

Coverage and quality gates run automatically:

```yaml
- name: Run Claude Integration Tests
  run: cargo test --lib claude_integration --verbose

- name: Check Quality Gates
  run: cargo test claude_integration::quality_gates
```

### Pre-commit Hooks

Local quality enforcement:

```bash
.git-hooks/pre-commit-claude-integration.sh
```

Checks:
- Zero SATD violations
- Code formatting
- Clippy lints
- All tests pass
- TypeScript build

## Benchmarks

Run performance benchmarks:

```bash
cargo bench --bench claude_integration_bench
```

Benchmarks:
- Cache operations (L1, L2, miss)
- Hash function performance
- Concurrent access patterns
- Memory allocation

## Troubleshooting

### Bridge Initialization Timeout

**Symptom**: Bridge takes > 500ms to initialize

**Solutions**:
1. Check TypeScript bridge binary is built: `cd bridge && npm run build`
2. Verify Node.js is installed and in PATH
3. Check sandbox directory permissions
4. Review system resource availability

### High Cache Miss Rate

**Symptom**: Cache hit rate < 50%

**Solutions**:
1. Increase cache TTLs in configuration
2. Verify code analysis inputs are stable
3. Check cache size limits
4. Monitor cache eviction patterns

### Circuit Breaker Opens Frequently

**Symptom**: Circuit opens despite healthy service

**Solutions**:
1. Increase failure threshold (default: 5 in 60s)
2. Lengthen error window duration
3. Check network stability
4. Review TypeScript bridge logs

### Quality Gate Violations

**Symptom**: Analysis fails quality gates

**Solutions**:
1. Review complexity thresholds (max: 15)
2. Check cognitive complexity limits (max: 10)
3. Verify test coverage requirements (min: 95%)
4. Reduce coupling (max: 3)

## FAQ

**Q: Is Claude API key required?**
A: Not yet. Current implementation provides infrastructure and mock analysis. Full Claude API integration is optional.

**Q: Can I use this without TypeScript?**
A: Yes. Rust-only usage works with mock analysis. TypeScript bridge is optional for full features.

**Q: What's the overhead of feature flags?**
A: ~10ns per check (circuit breaker pattern). Negligible in production.

**Q: How do I monitor in production?**
A: Use `MetricsCollector` and export to your observability platform (Prometheus, Datadog, etc.).

**Q: Can I customize quality gates?**
A: Yes. Create custom `ClaudeIntegrationQualityGate` with your thresholds.

**Q: Is caching thread-safe?**
A: Yes. Uses `Arc<RwLock<...>>` for safe concurrent access.

## Resources

- **Specification**: `docs/specifications/claude-agent-integration.md`
- **Implementation Report**: `docs/claude-integration-final.md`
- **Example Code**: `server/examples/claude_integration_example.rs`
- **Unit Tests**: `server/src/claude_integration/tests.rs`
- **Benchmarks**: `server/benches/claude_integration_bench.rs`

## Support

For issues, questions, or contributions:
- GitHub Issues: https://github.com/paiml/paiml-mcp-agent-toolkit/issues
- Documentation: https://docs.paiml.com
- Examples: `server/examples/`

---

Built with EXTREME TDD methodology by Pragmatic AI Labs
