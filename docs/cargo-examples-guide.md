# PMAT Cargo Examples - Complete Guide

## Overview

PMAT includes 30+ comprehensive examples demonstrating all features, from basic analysis to advanced MCP integration. These examples are designed to help users understand and test PMAT capabilities without needing to write code.

## Running Examples

All examples can be run with:
```bash
cargo run --example <example_name>
```

Or from the project root:
```bash
cd paiml-mcp-agent-toolkit
cargo run --example <example_name>
```

## MCP Server Examples

These examples are crucial for understanding MCP integration with Claude Code.

### mcp_server_pmcp
**Purpose**: Demonstrates the unified MCP server using pmcp SDK
```bash
cargo run --example mcp_server_pmcp

# What it does:
# - Starts the MCP server
# - Shows available tools
# - Demonstrates pmcp SDK integration
# - Used by Claude Code for MCP connection
```

### test_pmcp_server
**Purpose**: Tests the MCP server with sample requests
```bash
cargo run --example test_pmcp_server

# What it does:
# - Sends test requests to MCP server
# - Validates responses
# - Tests all MCP tools
# - Great for debugging MCP issues
```

### unified_mcp_demo
**Purpose**: Comprehensive MCP demonstration
```bash
cargo run --example unified_mcp_demo

# What it does:
# - Shows all MCP capabilities
# - Demonstrates tool composition
# - Tests quality gates via MCP
# - Shows PDMT integration
```

### pmcp_analyze_workflow
**Purpose**: Complete analysis workflow via MCP
```bash
cargo run --example pmcp_analyze_workflow

# What it does:
# - Runs full project analysis
# - Generates PDMT todos
# - Checks quality gates
# - Creates comprehensive reports
```

### pmcp_refactor_session
**Purpose**: Automated refactoring session via MCP
```bash
cargo run --example pmcp_refactor_session

# What it does:
# - Identifies refactoring targets
# - Generates refactoring plans
# - Validates improvements
# - Shows before/after metrics
```

## Quality Gate Examples

Essential for understanding quality enforcement.

### quality_gate
**Purpose**: Basic quality gate demonstration
```bash
cargo run --example quality_gate

# What it does:
# - Runs all quality checks
# - Shows pass/fail status
# - Lists all metrics checked
# - Provides improvement suggestions
```

### quality_gate_custom
**Purpose**: Custom quality thresholds
```bash
cargo run --example quality_gate_custom

# What it does:
# - Demonstrates custom thresholds
# - Shows configuration options
# - Tests specific quality rules
# - Validates custom standards
```

### quality_gate_thresholds
**Purpose**: Threshold configuration examples
```bash
cargo run --example quality_gate_thresholds

# What it does:
# - Tests different threshold levels
# - Shows impact of each threshold
# - Demonstrates gradual tightening
# - Helps find optimal settings
```

### quality_gate_shows_checks
**Purpose**: Detailed view of all quality checks
```bash
cargo run --example quality_gate_shows_checks

# What it does:
# - Lists every quality check performed
# - Shows check execution order
# - Displays timing information
# - Useful for debugging
```

### quality_gate_perf
**Purpose**: Performance testing of quality gates
```bash
cargo run --example quality_gate_perf

# What it does:
# - Benchmarks quality gate speed
# - Tests with large codebases
# - Shows caching effectiveness
# - Measures parallel processing
```

### quality_proxy_demo
**Purpose**: Quality proxy in action
```bash
cargo run --example quality_proxy_demo

# What it does:
# - Intercepts code changes
# - Validates against quality standards
# - Shows auto-fix capabilities
# - Demonstrates enforcement modes
```

## Complexity Analysis Examples

Understanding code complexity metrics.

### complexity_demo
**Purpose**: Basic complexity analysis
```bash
cargo run --example complexity_demo

# What it does:
# - Analyzes cyclomatic complexity
# - Shows cognitive complexity
# - Lists complex functions
# - Provides refactoring hints
```

### complexity_validation
**Purpose**: Validates complexity calculations
```bash
cargo run --example complexity_validation

# What it does:
# - Tests complexity algorithms
# - Compares with known values
# - Validates edge cases
# - Shows calculation details
```

### complexity_isolation
**Purpose**: Isolated complexity testing
```bash
cargo run --example complexity_isolation

# What it does:
# - Tests single functions
# - Shows complexity breakdown
# - Demonstrates incremental analysis
# - Useful for debugging
```

### debug_complexity
**Purpose**: Debug complexity calculations
```bash
cargo run --example debug_complexity

# What it does:
# - Shows step-by-step calculation
# - Displays AST analysis
# - Identifies complexity sources
# - Helps understand metrics
```

### deep_context_complexity
**Purpose**: Deep contextual complexity analysis
```bash
cargo run --example deep_context_complexity

# What it does:
# - Analyzes complexity in context
# - Shows dependency impacts
# - Identifies complexity hotspots
# - Provides architectural insights
```

### one_function_only
**Purpose**: Analyze single function complexity
```bash
cargo run --example one_function_only

# What it does:
# - Focuses on one function
# - Shows detailed metrics
# - Tests isolated analysis
# - Quick complexity check
```

### single_function_test
**Purpose**: Test single function analysis
```bash
cargo run --example single_function_test

# What it does:
# - Validates function detection
# - Tests metric calculation
# - Shows function boundaries
# - Debugging tool
```

### single_if_test
**Purpose**: Test if-statement complexity
```bash
cargo run --example single_if_test

# What it does:
# - Tests conditional complexity
# - Shows nesting impact
# - Validates branch counting
# - Edge case testing
```

## Code Analysis Examples

### analyze_complexity
**Purpose**: Comprehensive complexity analysis
```bash
cargo run --example analyze_complexity

# What it does:
# - Full project complexity scan
# - Generates complexity report
# - Identifies refactoring targets
# - Shows complexity distribution
```

### analyze_dead_code
**Purpose**: Find unused code
```bash
cargo run --example analyze_dead_code

# What it does:
# - Detects unused functions
# - Finds dead structs/enums
# - Identifies unreachable code
# - Suggests removal candidates
```

### analyze_satd
**Purpose**: Find technical debt comments
```bash
cargo run --example analyze_satd

# What it does:
# - Scans for TODO/FIXME/HACK
# - Categorizes debt types
# - Shows debt distribution
# - Enforces zero-SATD policy
```

### satd_lint_analysis
**Purpose**: Combined SATD and lint analysis
```bash
cargo run --example satd_lint_analysis

# What it does:
# - Runs SATD detection
# - Performs lint checks
# - Shows correlation
# - Comprehensive debt analysis
```

### lint_hotspot_demo
**Purpose**: Find lint violation hotspots
```bash
cargo run --example lint_hotspot_demo

# What it does:
# - Identifies files with most violations
# - Shows violation types
# - Prioritizes fixes
# - Generates fix order
```

### lint_hotspot_enforce_flag
**Purpose**: Enforce lint standards
```bash
cargo run --example lint_hotspot_enforce_flag

# What it does:
# - Enforces zero violations
# - Shows enforcement modes
# - Tests strict checking
# - Validates lint fixes
```

## Integration Examples

### ci_integration
**Purpose**: CI/CD pipeline integration
```bash
cargo run --example ci_integration

# What it does:
# - Shows GitHub Actions setup
# - Demonstrates exit codes
# - Tests CI compatibility
# - Provides workflow templates
```

### check_github_repo
**Purpose**: Analyze GitHub repository
```bash
cargo run --example check_github_repo

# What it does:
# - Clones and analyzes repo
# - Generates quality report
# - Shows issue integration
# - Tests remote analysis
```

### exit_codes
**Purpose**: Demonstrate exit code usage
```bash
cargo run --example exit_codes

# What it does:
# - Shows all exit codes
# - Tests error conditions
# - Validates CI integration
# - Documents return values
```

## Scaffold Agent Examples

### scaffold_agent_basics
**Purpose**: Basic project scaffolding
```bash
cargo run --example scaffold_agent_basics

# What it does:
# - Creates project structure
# - Generates boilerplate
# - Sets up configuration
# - Initializes git
```

### scaffold_agent_interactive
**Purpose**: Interactive scaffolding
```bash
cargo run --example scaffold_agent_interactive

# What it does:
# - Prompts for options
# - Customizes generation
# - Validates inputs
# - Creates tailored project
```

### scaffold_agent_hybrid
**Purpose**: Hybrid scaffolding approach
```bash
cargo run --example scaffold_agent_hybrid

# What it does:
# - Combines templates
# - Mixes configurations
# - Creates complex projects
# - Advanced scaffolding
```

### scaffold_agent_course_project
**Purpose**: Educational project scaffolding
```bash
cargo run --example scaffold_agent_course_project

# What it does:
# - Creates learning projects
# - Includes documentation
# - Adds exercises
# - Educational focus
```

## Utility Examples

### check_code_quality
**Purpose**: Comprehensive quality check
```bash
cargo run --example check_code_quality

# What it does:
# - Runs all quality checks
# - Generates full report
# - Shows all metrics
# - One-stop quality analysis
```

## Running Multiple Examples

### Quick Test Suite
```bash
# Test core MCP functionality
cargo run --example mcp_server_pmcp
cargo run --example test_pmcp_server
cargo run --example unified_mcp_demo

# Test quality gates
cargo run --example quality_gate
cargo run --example quality_proxy_demo

# Test analysis
cargo run --example complexity_demo
cargo run --example analyze_dead_code
cargo run --example analyze_satd
```

### Comprehensive Test
```bash
# Run all examples (takes time)
for example in $(ls server/examples/*.rs | xargs -n1 basename | sed 's/\.rs//'); do
    echo "Running: $example"
    cargo run --example "$example"
    echo "---"
done
```

## Using Examples in Development

### Testing New Features
```rust
// Create new example: server/examples/my_feature.rs
use pmat::services::my_feature::MyFeature;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let feature = MyFeature::new();
    
    // Test the feature
    let result = feature.process("test input")?;
    
    println!("Result: {:?}", result);
    Ok(())
}
```

Then run:
```bash
cargo run --example my_feature
```

### Debugging with Examples
```bash
# Run with debug output
RUST_LOG=debug cargo run --example quality_gate

# Run with backtrace
RUST_BACKTRACE=1 cargo run --example mcp_server_pmcp

# Profile performance
cargo run --release --example quality_gate_perf
```

## Common Patterns in Examples

### Pattern 1: MCP Testing
Most MCP examples follow this pattern:
1. Initialize MCP server
2. Send test requests
3. Validate responses
4. Show results

### Pattern 2: Quality Validation
Quality examples typically:
1. Load test code
2. Run quality checks
3. Display metrics
4. Suggest improvements

### Pattern 3: Analysis Workflow
Analysis examples usually:
1. Scan project files
2. Calculate metrics
3. Generate reports
4. Provide actionable insights

## Troubleshooting Examples

### Example Not Running?
```bash
# Ensure you're in the right directory
cd paiml-mcp-agent-toolkit

# Check example exists
ls server/examples/ | grep your_example

# Run with full path
cargo run --manifest-path server/Cargo.toml --example your_example
```

### Example Failing?
```bash
# Check dependencies
cargo check

# Update dependencies
cargo update

# Run with verbose output
RUST_LOG=trace cargo run --example failing_example
```

## Best Practices

1. **Start Simple**: Begin with basic examples like `complexity_demo`
2. **Test MCP First**: Run `mcp_server_pmcp` to verify MCP setup
3. **Check Quality**: Use `quality_gate` before commits
4. **Learn by Example**: Study example code for implementation patterns
5. **Create Your Own**: Add custom examples for new features

## Related Documentation

- [MCP Setup Guide](./mcp-claude-code-setup.md)
- [PDMT Examples](./pdmt-detailed-examples.md)
- [Quality Gates Guide](./quality-gates-proxy-detailed.md)
- [Examples README](../server/examples/README.md)