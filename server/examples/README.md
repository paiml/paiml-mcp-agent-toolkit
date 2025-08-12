# PMAT Examples

This directory contains runnable examples demonstrating various features of PMAT.

## Running Examples

All examples can be run using `cargo run --example <example_name>`:

```bash
# Run a specific example
cargo run --example quality_gate
cargo run --example analyze_complexity
```

## Available Examples

### Quality Analysis Examples

#### `quality_gate`
Demonstrates comprehensive quality gate functionality with different configurations.
```bash
cargo run --example quality_gate
```

#### `analyze_complexity`
Shows how to use complexity analysis with CI/CD integration.
```bash
cargo run --example analyze_complexity
```

#### `analyze_dead_code`
Demonstrates dead code detection with configurable thresholds.
```bash
cargo run --example analyze_dead_code
```

#### `analyze_satd`
Shows Self-Admitted Technical Debt detection with various modes.
```bash
cargo run --example analyze_satd
```

#### `quality_proxy_demo`
**NEW** - Interactive demonstration of the Quality Proxy service for enforcing code quality on AI-generated code.
Shows all three enforcement modes (Strict, Advisory, Auto-Fix) with real examples.
```bash
cargo run --example quality_proxy_demo
```

Features demonstrated:
- High-quality code acceptance in strict mode
- SATD rejection and automatic removal
- Advisory mode with warnings
- Auto-fix mode with refactoring
- Complexity threshold enforcement

### CI/CD Integration Examples

#### `ci_integration`
Comprehensive CI/CD workflow examples for multiple platforms:
- GitHub Actions
- GitLab CI
- Jenkins
- CircleCI
- Pre-commit hooks
```bash
cargo run --example ci_integration
```

#### `exit_codes`
Reference guide for exit code behavior in CI/CD environments.
```bash
cargo run --example exit_codes
```

### Testing Examples

#### `complexity_demo`
Contains various complexity patterns for testing the analyzer:
- Simple functions (low complexity)
- Moderate complexity functions
- High complexity functions
- Very high complexity patterns
```bash
cargo run --example complexity_demo
# Then analyze it:
pmat analyze complexity --include "server/examples/complexity_demo.rs"
```

#### `lint_hotspot_demo`
Demonstrates lint hotspot analysis with intentional code quality issues.
```bash
cargo run --example lint_hotspot_demo
```

### MCP Server Examples

#### `mcp_server_pmcp` (NEW)
Demonstrates running the pmat MCP server using the pmcp Rust SDK.
```bash
# Start the MCP server
cargo run --example mcp_server_pmcp

# Connect with an MCP client in another terminal
npx @modelcontextprotocol/inspector tcp://127.0.0.1:3000
```

This example shows:
- Setting up an MCP server with the pmcp SDK
- Registering all pmat analysis tools
- Handling TCP connections
- Type-safe tool implementations

### Other Examples

#### `complexity_validation`
Validates complexity calculations against expected values.
```bash
cargo run --example complexity_validation
```

#### `complexity_isolation`
Tests complexity analysis in isolation.
```bash
cargo run --example complexity_isolation
```

## Example Patterns

### CI/CD Integration Pattern
```rust
// Check with custom thresholds and fail on violation
let result = handle_analyze_complexity(
    PathBuf::from("."),
    None,
    vec![],
    None,
    ComplexityOutputFormat::Json,
    None,
    Some(15),  // Max cyclomatic complexity
    Some(10),  // Max cognitive complexity
    vec![],
    false,
    10,
    true,      // fail_on_violation - will exit(1) if violated
).await;
```

### Quality Gate Pattern
```rust
// Run comprehensive quality checks
let result = handle_quality_gate(
    PathBuf::from("."),
    None,
    QualityGateOutputFormat::Json,
    true,  // fail_on_violation
    vec![],
    10.0,  // max dead code percentage
    2.0,   // min entropy
    20,    // max complexity
    false,
    None,
    false,
).await;
```

## Creating New Examples

When adding new examples:
1. Create a new `.rs` file in this directory
2. Add a `main` function (use `#[tokio::main]` for async)
3. Include helpful comments explaining what the example demonstrates
4. Add the example to this README
5. Test that it runs: `cargo run --example your_example`

## Best Practices

1. **Clear Output**: Use clear section headers and explanations
2. **Error Handling**: Show both success and failure cases
3. **Real-World Usage**: Demonstrate practical, real-world scenarios
4. **Documentation**: Include inline comments explaining each step
5. **Multiple Scenarios**: Show different configurations and use cases