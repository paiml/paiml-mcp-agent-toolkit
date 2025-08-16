# Quality Gates Proxy - Comprehensive Documentation

## Overview

The Quality Gates Proxy is a powerful feature in PMAT that acts as an intelligent intermediary between AI agents and your codebase. It intercepts, validates, and can automatically improve code changes to ensure they meet strict quality standards before being applied.

## Core Concepts

### What is Quality Gates Proxy?

The Quality Gates Proxy is a validation layer that:
1. **Intercepts** all code modifications from AI agents
2. **Validates** changes against configurable quality thresholds
3. **Enforces** zero-tolerance standards (0 SATD, complexity limits, test coverage)
4. **Provides** detailed feedback on violations
5. **Can automatically refactor** code to meet standards (in auto-fix mode)

### Why Use Quality Gates Proxy?

- **Prevent Technical Debt**: Stop low-quality code before it enters your codebase
- **Maintain Standards**: Enforce consistent quality across all contributors (human and AI)
- **Immediate Feedback**: Get instant validation results
- **Automated Improvement**: Let the proxy fix issues automatically
- **Comprehensive Checks**: Single tool for all quality metrics

## Quick Start

### Using in Claude Code

Simply ask Claude Code:
- "Check quality of src/main.rs"
- "Run quality gates on this project"
- "Validate my changes meet quality standards"
- "Enable strict quality enforcement"

### Command Line Examples

```bash
# Basic quality check
pmat quality-gate

# Strict mode (zero tolerance)
pmat quality-gate --strict

# Check specific file
pmat quality-gate --file src/main.rs

# Check with custom thresholds
pmat quality-gate --complexity 10 --coverage 90

# Run example demonstrations
cargo run --example quality_gate
cargo run --example quality_gate_custom
cargo run --example quality_gate_thresholds
cargo run --example quality_proxy_demo
cargo run --example quality_gate_shows_checks
```

## Detailed Examples

### Example 1: Basic Quality Gate Check

```bash
# Run quality gate on entire project
cargo run --example quality_gate

# Expected output:
# ✅ Quality Gate: PASSED
# 
# Summary:
#   Complexity: ✅ All functions ≤20 (max found: 8)
#   SATD: ✅ 0 comments found
#   Coverage: ✅ 87.3% (threshold: 80%)
#   Linting: ✅ 0 violations
#   Dead Code: ✅ 0 unused items
```

### Example 2: Strict Mode Enforcement

```rust
/// Example doctest for strict quality enforcement
/// 
/// ```rust
/// use pmat::services::quality_proxy::QualityProxy;
/// use pmat::models::proxy::{ProxyConfig, EnforcementMode};
/// 
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create proxy with strict configuration
/// let config = ProxyConfig {
///     enforcement_mode: EnforcementMode::Strict,
///     thresholds: Default::default(),
///     auto_fix: false,
///     detailed_report: true,
/// };
/// 
/// let proxy = QualityProxy::new(config);
/// 
/// // Validate a file
/// let result = proxy.validate_file("src/main.rs").await?;
/// 
/// if !result.passed {
///     println!("❌ Quality gate failed!");
///     for violation in result.violations {
///         println!("  - {}: {}", violation.category, violation.message);
///     }
/// } else {
///     println!("✅ All quality checks passed!");
/// }
/// # Ok(())
/// # }
/// ```
```

### Example 3: Custom Thresholds

```bash
# Configure custom quality thresholds
echo '{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "quality_gate",
    "arguments": {
      "file_path": "src/complex_module.rs",
      "thresholds": {
        "max_complexity": 15,
        "min_coverage": 85.0,
        "max_lint_violations": 0,
        "allow_satd": false,
        "max_function_length": 50
      }
    }
  },
  "id": 1
}' | pmat
```

### Example 4: Auto-Fix Mode

```rust
/// Auto-fix mode example
/// 
/// ```rust
/// use pmat::services::quality_proxy::{QualityProxy, AutoFixConfig};
/// 
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let proxy = QualityProxy::with_auto_fix(AutoFixConfig {
///     fix_complexity: true,
///     fix_naming: true,
///     fix_formatting: true,
///     fix_imports: true,
///     add_tests: false, // Don't auto-generate tests
/// });
/// 
/// // Process a file with automatic fixes
/// let result = proxy.process_file("src/messy_code.rs").await?;
/// 
/// println!("Fixed {} issues automatically", result.fixes_applied);
/// 
/// for fix in result.applied_fixes {
///     println!("  - {}: {}", fix.category, fix.description);
/// }
/// # Ok(())
/// # }
/// ```
```

## Quality Metrics Explained

### Complexity (Cyclomatic & Cognitive)

**What it measures**: How difficult code is to understand and test

**Thresholds**:
- **Excellent**: ≤ 5
- **Good**: ≤ 10
- **Acceptable**: ≤ 20
- **Needs Refactoring**: > 20

**Example Check**:
```bash
cargo run --example complexity_demo
cargo run --example quality_gate_thresholds
```

### SATD (Self-Admitted Technical Debt)

**What it measures**: TODO, FIXME, HACK, XXX comments

**Threshold**: Zero tolerance (0 comments)

**Example Check**:
```bash
cargo run --example analyze_satd
cargo run --example satd_lint_analysis
```

### Test Coverage

**What it measures**: Percentage of code covered by tests

**Thresholds**:
- **Minimum**: 80%
- **Recommended**: 85%
- **Excellent**: 90%+

**Example Check**:
```bash
# Run with coverage measurement
cargo tarpaulin --min 80
```

### Linting Violations

**What it measures**: Code style and potential bugs (via clippy)

**Threshold**: 0 violations

**Categories checked**:
- Correctness
- Performance
- Style
- Complexity
- Pedantic (optional)

### Dead Code

**What it measures**: Unused functions, structs, modules

**Threshold**: 0 unused items

**Example Check**:
```bash
cargo run --example analyze_dead_code
```

## Integration Patterns

### Pattern 1: Pre-Commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

echo "Running PMAT Quality Gates..."

# Check all staged Rust files
for file in $(git diff --cached --name-only --diff-filter=ACM | grep '\.rs$'); do
    echo "Checking: $file"
    
    if ! pmat quality-gate --file "$file" --strict; then
        echo "❌ Quality gate failed for $file"
        echo ""
        echo "To fix automatically, run:"
        echo "  pmat refactor auto --file $file"
        echo ""
        echo "Or to see detailed issues:"
        echo "  pmat quality-gate --file $file --verbose"
        exit 1
    fi
done

echo "✅ All quality gates passed!"
```

### Pattern 2: CI/CD Pipeline

```yaml
# .github/workflows/quality-gates.yml
name: Quality Gates

on:
  pull_request:
  push:
    branches: [main, develop]

jobs:
  quality-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install PMAT
        run: cargo install pmat
      
      - name: Run Quality Gates
        run: |
          pmat quality-gate --strict --output json > quality-report.json
          
      - name: Upload Quality Report
        uses: actions/upload-artifact@v3
        with:
          name: quality-report
          path: quality-report.json
      
      - name: Comment PR with Results
        if: github.event_name == 'pull_request'
        uses: actions/github-script@v6
        with:
          script: |
            const fs = require('fs');
            const report = JSON.parse(fs.readFileSync('quality-report.json'));
            
            const comment = `## 🔍 Quality Gate Results
            
            ${report.passed ? '✅ **PASSED**' : '❌ **FAILED**'}
            
            ### Metrics:
            - **Complexity**: ${report.metrics.complexity.status} (max: ${report.metrics.complexity.max})
            - **SATD**: ${report.metrics.satd.count} comments
            - **Coverage**: ${report.metrics.coverage.percentage}%
            - **Linting**: ${report.metrics.linting.violations} violations
            - **Dead Code**: ${report.metrics.dead_code.count} items
            
            ${!report.passed ? '### Required Actions:\n' + report.actions.join('\n') : ''}
            `;
            
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: comment
            });
```

### Pattern 3: VS Code Integration

```json
// .vscode/tasks.json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "Quality Gate: Current File",
      "type": "shell",
      "command": "pmat",
      "args": [
        "quality-gate",
        "--file",
        "${file}",
        "--verbose"
      ],
      "presentation": {
        "reveal": "always",
        "panel": "dedicated",
        "focus": false,
        "clear": true
      },
      "problemMatcher": {
        "owner": "pmat",
        "pattern": {
          "regexp": "^(.+):(\\d+):(\\d+):\\s+(warning|error):\\s+(.+)$",
          "file": 1,
          "line": 2,
          "column": 3,
          "severity": 4,
          "message": 5
        }
      }
    },
    {
      "label": "Quality Gate: Auto-Fix",
      "type": "shell",
      "command": "pmat",
      "args": [
        "refactor",
        "auto",
        "--file",
        "${file}"
      ],
      "presentation": {
        "reveal": "always",
        "panel": "dedicated"
      }
    }
  ]
}
```

### Pattern 4: Real-time Quality Monitoring

```rust
/// Real-time quality monitoring service
/// 
/// ```rust
/// use pmat::services::quality_proxy::QualityMonitor;
/// use std::time::Duration;
/// 
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let monitor = QualityMonitor::new()
///     .with_interval(Duration::from_secs(60))
///     .with_paths(vec!["src/", "tests/"])
///     .with_thresholds(Default::default());
/// 
/// // Start monitoring
/// let handle = monitor.start().await?;
/// 
/// // Subscribe to quality events
/// let mut receiver = monitor.subscribe();
/// 
/// while let Some(event) = receiver.recv().await {
///     match event {
///         QualityEvent::Degradation(file, metric) => {
///             eprintln!("⚠️ Quality degradation in {}: {:?}", file, metric);
///         }
///         QualityEvent::Improvement(file, metric) => {
///             println!("✅ Quality improved in {}: {:?}", file, metric);
///         }
///         QualityEvent::Violation(file, violation) => {
///             eprintln!("❌ Quality violation in {}: {}", file, violation);
///         }
///     }
/// }
/// # Ok(())
/// # }
/// ```
```

## Advanced Configuration

### Configuration File

Create `.pmat/quality.toml`:

```toml
[quality_gates]
enforcement_mode = "strict"  # strict, advisory, auto_fix
detailed_reports = true
cache_results = true

[thresholds]
max_complexity = 10
min_coverage = 85.0
max_lint_violations = 0
allow_satd = false
max_function_length = 40
max_file_length = 400
max_dependencies = 10

[auto_fix]
enabled = true
fix_complexity = true
fix_naming = true
fix_formatting = true
fix_imports = true
add_documentation = true
add_tests = false

[exclusions]
paths = ["tests/", "benches/", "examples/"]
patterns = ["*_test.rs", "*.generated.rs"]

[reporting]
format = "json"  # json, yaml, markdown, console
output_file = "quality-report.json"
include_suggestions = true
include_metrics = true
```

### Environment Variables

```bash
# Set quality gate defaults via environment
export PMAT_QUALITY_MODE=strict
export PMAT_QUALITY_COMPLEXITY=10
export PMAT_QUALITY_COVERAGE=85
export PMAT_QUALITY_AUTO_FIX=true

# Run with environment config
pmat quality-gate
```

## MCP Tool Interface

### Tool: `quality_gate`

```javascript
// MCP tool call structure
{
  "tool": "quality_gate",
  "parameters": {
    "file_path": "src/main.rs",  // Optional: specific file
    "strict": true,               // Optional: enforce zero tolerance
    "thresholds": {              // Optional: custom thresholds
      "max_complexity": 10,
      "min_coverage": 85.0,
      "max_lint_violations": 0,
      "allow_satd": false
    },
    "auto_fix": false,           // Optional: automatically fix issues
    "detailed_report": true      // Optional: include detailed analysis
  }
}
```

### Using in Claude Code

When using Claude Code with PMAT MCP server, you can simply ask:

- "Check the quality of my code"
- "Run quality gates with strict enforcement"
- "Fix quality issues in src/main.rs"
- "What's the complexity of this function?"
- "Find all SATD comments"

The Quality Gates Proxy will automatically be invoked when appropriate.

## Testing Quality Gates

### Unit Tests

```rust
/// Test quality gate validation
/// 
/// ```rust
/// use pmat::services::quality_gates::QualityGate;
/// 
/// #[test]
/// fn test_quality_validation() {
///     let gate = QualityGate::default();
///     
///     // Test complexity check
///     let complex_code = r#"
///         fn complex_function(x: i32) -> i32 {
///             if x > 0 {
///                 if x > 10 {
///                     if x > 20 {
///                         if x > 30 {
///                             return x * 2;
///                         }
///                     }
///                 }
///             }
///             x
///         }
///     "#;
///     
///     let result = gate.check_complexity(complex_code);
///     assert!(!result.passed);
///     assert!(result.complexity > 5);
/// }
/// ```
```

### Property Tests

```rust
/// Property test for quality gate consistency
/// 
/// ```rust
/// use proptest::prelude::*;
/// use pmat::services::quality_gates::QualityGate;
/// 
/// proptest! {
///     #[test]
///     fn quality_gate_deterministic(
///         code in "[a-z\n\t ]{10,100}",
///         seed in 0u64..100u64,
///     ) {
///         let gate = QualityGate::with_seed(seed);
///         
///         // Same code should always produce same quality results
///         let result1 = gate.check(&code);
///         let result2 = gate.check(&code);
///         
///         prop_assert_eq!(result1, result2);
///     }
/// }
/// ```
```

### Integration Tests

```bash
# Run all quality gate tests
cargo test quality_gate

# Run specific test suites
cargo test quality_proxy_integration
cargo test quality_gate_property_tests

# Run examples as tests
cargo run --example quality_gate
cargo run --example quality_proxy_demo
```

## Troubleshooting

### Issue: "Quality gate failing but code looks fine"

Debug with verbose output:
```bash
# Get detailed analysis
pmat quality-gate --file src/main.rs --verbose

# Check specific metrics
pmat analyze complexity --file src/main.rs
pmat analyze satd --file src/main.rs
pmat analyze dead-code --file src/main.rs
```

### Issue: "Auto-fix not working"

Ensure proper permissions and backup:
```bash
# Create backup before auto-fix
cp src/main.rs src/main.rs.backup

# Run auto-fix with debug info
RUST_LOG=debug pmat refactor auto --file src/main.rs
```

### Issue: "Different results in CI vs local"

Ensure consistent environment:
```bash
# Use exact same versions
cargo update -p pmat --precise 2.3.1

# Use same configuration
export PMAT_QUALITY_MODE=strict
export PMAT_QUALITY_COMPLEXITY=10
```

## Performance Considerations

### Caching

Quality Gates Proxy uses intelligent caching:

```rust
/// Enable caching for better performance
/// 
/// ```rust
/// use pmat::services::quality_proxy::QualityProxy;
/// use pmat::services::cache::CacheConfig;
/// 
/// let proxy = QualityProxy::with_cache(CacheConfig {
///     enabled: true,
///     ttl_seconds: 300,
///     max_size_mb: 100,
///     path: "/tmp/pmat-quality-cache",
/// });
/// ```
```

### Parallel Processing

For large codebases:

```bash
# Process files in parallel
find src -name "*.rs" | \
  parallel -j 8 'pmat quality-gate --file {}'
```

### Incremental Checks

Only check changed files:

```bash
# Check only modified files
git diff --name-only HEAD~1 | \
  grep '\.rs$' | \
  xargs -I {} pmat quality-gate --file {}
```

## Best Practices

### 1. Start with Advisory Mode

Begin with warnings before enforcing:

```bash
# Phase 1: Advisory mode
pmat quality-gate --mode advisory

# Phase 2: Fix issues
pmat refactor auto

# Phase 3: Enable strict mode
pmat quality-gate --strict
```

### 2. Gradual Threshold Tightening

```toml
# Week 1
[thresholds]
max_complexity = 20
min_coverage = 60

# Week 2
[thresholds]
max_complexity = 15
min_coverage = 70

# Week 3
[thresholds]
max_complexity = 10
min_coverage = 80
```

### 3. Team Onboarding

```bash
# Create team quality profile
cat > team-quality.toml << EOF
[quality_gates]
enforcement_mode = "advisory"
educational_messages = true

[thresholds]
max_complexity = 15
min_coverage = 75
EOF

# Share with team
pmat quality-gate --config team-quality.toml
```

## Additional Resources

- Example files: `server/examples/quality_*.rs`
- Implementation: `server/src/services/quality_proxy.rs`
- Models: `server/src/models/proxy.rs`
- Tests: `server/tests/quality_*.rs`
- Integration guide: `docs/guides/github-actions-quality-gate.md`

## Related Documentation

- [PDMT Integration](./pdmt-detailed-examples.md) - Todo generation with quality enforcement
- [MCP Setup Guide](./mcp-claude-code-setup.md) - Complete MCP configuration
- [Refactoring Guide](./guides/refactor-auto-guide.md) - Automated code improvement