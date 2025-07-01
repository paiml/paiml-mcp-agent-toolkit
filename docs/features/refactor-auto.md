# AI-Powered Automated Refactoring Guide

The `pmat refactor auto` command represents a breakthrough in automated code refactoring, leveraging AI to transform codebases to meet **EXTREME quality standards** without manual intervention.

## Table of Contents

1. [Overview](#overview)
2. [Quality Standards](#quality-standards)
3. [How It Works](#how-it-works)
4. [Usage Guide](#usage-guide)
5. [Configuration](#configuration)
6. [Output Formats](#output-formats)
7. [Integration Examples](#integration-examples)
8. [Troubleshooting](#troubleshooting)
9. [Advanced Features](#advanced-features)

## Overview

### What is Refactor Auto?

Refactor Auto is an AI-powered tool that:
- Automatically identifies quality violations in your codebase
- Generates comprehensive refactoring requests for AI processing
- Applies AI-generated improvements that meet extreme quality standards
- Iterates until the entire codebase achieves target quality metrics

### Key Benefits

- **Zero Manual Intervention**: Fully automated refactoring process
- **Extreme Quality Standards**: Enforces complexity ≤10, coverage ≥80%, zero SATD
- **AI-Driven**: Leverages advanced AI models for intelligent refactoring
- **Toyota Way Principles**: Focus on root cause fixes, not workarounds
- **Iterative Improvement**: Continuous refinement until standards are met

## Quality Standards

### Complexity Management
- **Maximum Cyclomatic Complexity**: 10 (target: 5)
- **Maximum Cognitive Complexity**: 15 (target: 8)
- **Function Length**: Prefer functions under 50 lines
- **Nesting Depth**: Maximum 4 levels of nesting

### Coverage Requirements
- **Minimum Test Coverage**: 80% per file
- **Meaningful Tests**: Not just placeholders, but comprehensive test scenarios
- **Edge Case Coverage**: Tests for error conditions and boundary cases
- **Integration Tests**: Tests for component interactions

### Technical Debt Elimination
- **Zero SATD**: No TODO, FIXME, HACK, XXX comments allowed
- **No Placeholder Code**: Complete implementations only
- **No Dead Code**: All code must be reachable and purposeful
- **No Duplicate Code**: DRY principles strictly enforced

### Code Quality
- **All Lints Fixed**: Clippy pedantic + nursery + restriction levels
- **Comprehensive Documentation**: All public items documented
- **Memory Safety**: No unsafe code without justification
- **Error Handling**: Proper error propagation and handling

## How It Works

### 1. Analysis Phase
```rust
// Scans codebase for violations
let violations = analyze_codebase(&project_path).await?;
let high_complexity_files = find_complexity_violations(&violations);
let coverage_gaps = analyze_test_coverage(&project_path).await?;
let satd_items = detect_technical_debt(&violations);
```

### 2. Prioritization
Files are prioritized by:
1. **Compilation errors** (highest priority - automatically detected)
2. **Lint violations** (sorted by highest count first)
   - Three-tier sorting: count → severity → coverage
3. **High complexity** (functions > 10 cyclomatic complexity)
4. **SATD items** (TODO, FIXME, HACK comments)
5. **Coverage gaps** (files < 80% coverage, prioritizing largest files)

### 3. AI Request Generation
```json
{
  "task": "unified_rewrite",
  "file": "src/complex_module.rs",
  "current_content": "...",
  "context": "...",
  "violations": [
    {
      "type": "complexity",
      "function": "handle_complex_operation",
      "current_complexity": 80,
      "target_complexity": 10
    }
  ],
  "coverage": {
    "current": 0.0,
    "target": 80.0,
    "needs_tests": true
  },
  "quality_requirements": {
    "max_complexity": 10,
    "min_coverage": 80,
    "zero_satd": true,
    "all_lints_fixed": true
  },
  "instructions": [
    "Break down complex functions into smaller, focused functions",
    "Extract complex logic into separate helper functions",
    "Add comprehensive error handling with proper propagation",
    "Create meaningful unit tests with ≥80% coverage",
    "Add comprehensive documentation for all public items"
  ]
}
```

### 4. AI Processing
The AI analyzes the request and generates:
- Refactored source code with reduced complexity
- Comprehensive test files
- Updated documentation
- Proper error handling

### 5. Verification & Iteration
```bash
# Verify the refactored code meets standards
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo llvm-cov report --json --output-path coverage.json
```

## Usage Guide

### Basic Usage
```bash
# Run automated refactoring with default settings
pmat refactor auto

# Specify project path
pmat refactor auto --project-path ./my-rust-project

# Limit iterations
pmat refactor auto --max-iterations 5
```

### Advanced Options
```bash
# Dry run - see what would be refactored
pmat refactor auto --dry-run --format json

# Verbose output for debugging
pmat refactor auto --verbose --debug

# CI/CD mode - fail if quality gates not met
pmat refactor auto --ci-mode

# Custom cache directory
pmat refactor auto --cache-dir ./custom-cache

# Custom trace filtering for debugging
pmat refactor auto --trace-filter "paiml=debug,cache=trace"
```

## Configuration

### Command Line Options

| Option | Description | Default |
|--------|-------------|---------|
| `--project-path`, `-p` | Root path of project to refactor | `.` |
| `--format` | Output format (summary, detailed, json) | `detailed` |
| `--max-iterations` | Maximum refactoring iterations | `10` |
| `--quality-profile` | Quality profile (standard, strict, extreme) | `extreme` |
| `--dry-run` | Show changes without applying them | `false` |
| `--skip-compilation` | Skip compilation checks (faster but less safe) | `false` |
| `--skip-tests` | Skip test execution (not recommended) | `false` |
| `--checkpoint` | Checkpoint file for resumable refactoring | None |
| `--verbose`, `-v` | Enable verbose output | `false` |

### Quality Thresholds

The tool uses these built-in thresholds (not configurable to maintain standards):
- **Max Complexity**: 10
- **Min Coverage**: 80%
- **SATD Tolerance**: 0 items
- **Lint Level**: pedantic + nursery + restriction

## Output Formats

### Summary Format (Default)
```
Iteration 1: Analyzing codebase...
Found 1 file with quality violations
Current file: server/src/complex_module.rs
Complexity: 80 -> Target: 10
Coverage: 0% -> Target: 80%
SATD items: 5 -> Target: 0

Generating AI refactoring request...
Request generated: 2.1KB JSON payload
```

### Detailed Format
```
=== REFACTOR AUTO ITERATION 1 ===

📊 Quality Metrics:
- Total violations: 15
- Max complexity: 80 (target: ≤10)
- Coverage: 42.3% (target: ≥80%)
- SATD count: 5 (target: 0)
- Files with issues: 3

🎯 Current Target: server/src/complex_module.rs
Violations:
  - Function 'handle_complex_operation': complexity 80 (target: ≤10)
  - Missing tests: 0% coverage (target: ≥80%)
  - SATD items: 2 TODO comments

📝 AI Request Generated (2,147 bytes):
  - Task: unified_rewrite
  - Context: 847 lines of surrounding code
  - Instructions: 8 specific refactoring guidelines
```

### JSON Format
```json
{
  "iteration": 1,
  "context_generated": true,
  "context_path": "./.pmat-cache/deep_context.md",
  "current_file": "server/src/complex_module.rs",
  "files_completed": [],
  "quality_metrics": {
    "total_violations": 15,
    "coverage_percent": 42.3,
    "max_complexity": 80,
    "satd_count": 5,
    "files_with_issues": 3
  },
  "start_time": {
    "secs_since_epoch": 1703123456,
    "nanos_since_epoch": 789012345
  }
}
```

## Integration Examples

### Pre-commit Hook
```bash
#!/bin/bash
# .git/hooks/pre-commit

echo "Running automated quality check..."
if ! pmat refactor auto --dry-run --ci-mode; then
    echo "❌ Quality standards not met. Run 'pmat refactor auto' to fix."
    exit 1
fi
echo "✅ Quality check passed"
```

### GitHub Actions CI/CD
```yaml
name: Quality Gate
on: [push, pull_request]

jobs:
  quality-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install pmat
        run: |
          curl -sSfL https://raw.githubusercontent.com/paiml/paiml-mcp-agent-toolkit/master/scripts/install.sh | sh
          
      - name: Run quality gate
        run: |
          pmat refactor auto --ci-mode --format json > quality-report.json
          
      - name: Upload quality report
        uses: actions/upload-artifact@v3
        with:
          name: quality-report
          path: quality-report.json
```

### Development Workflow
```bash
# Daily quality improvement
make quality-check:
	pmat refactor auto --max-iterations 1

# Pre-release quality assurance
make pre-release:
	pmat refactor auto --ci-mode
	cargo test --all-targets
	cargo clippy --all-targets -- -D warnings

# Continuous integration
make ci:
	pmat refactor auto --dry-run --format json | \
	  jq '.files_with_issues' | \
	  test "$$(cat)" -eq 0 || exit 1
```

## Troubleshooting

### Common Issues

#### 1. Compilation Errors After Refactoring
```bash
# Check for syntax errors
cargo check

# Run with verbose logging
pmat refactor auto --debug --trace-filter "paiml=debug"

# Restore from backup (automatically created)
cp .pmat-cache/backup/src/module.rs src/module.rs
```

#### 2. AI Request Generation Failures
```bash
# Check context generation
pmat context --verbose

# Verify file permissions
ls -la .pmat-cache/

# Clear cache and retry
rm -rf .pmat-cache/
pmat refactor auto
```

#### 3. Coverage Measurement Issues
```bash
# Install cargo-llvm-cov if missing
cargo install cargo-llvm-cov

# Check test compilation
cargo test --no-run

# Run coverage manually
cargo cargo llvm-cov report --json --output-path coverage.json
```

### Performance Optimization

#### Memory Usage
```bash
# Configure swap for large codebases
make config-swap

# Monitor memory usage
htop -p $(pgrep pmat)

# Use streaming mode for large files
pmat refactor auto --cache-dir /tmp/pmat-cache
```

#### Speed Optimization
```bash
# Parallel processing
export RAYON_NUM_THREADS=8

# Skip expensive analyses in development
pmat refactor auto --max-iterations 1

# Use incremental mode
pmat refactor auto --resume
```

## Advanced Features

### Custom Context Generation
```bash
# Generate custom context for AI requests
pmat context --format json --include-ast > custom-context.json

# Use custom context in refactoring
pmat refactor auto --context-file custom-context.json
```

### Integration with External Tools

#### SonarQube Integration
```bash
# Export quality metrics to SonarQube format
pmat refactor auto --format json | \
  jq '.quality_metrics' > sonar-quality.json
```

#### Metrics Collection
```bash
# Collect refactoring metrics over time
pmat refactor auto --format json | \
  jq '.quality_metrics + {"timestamp": now}' >> metrics.jsonl
```

### State Management
```bash
# View current refactoring state
cat .pmat-cache/refactor-state.json

# Resume interrupted refactoring
pmat refactor auto --resume

# Reset state and start fresh
rm -rf .pmat-cache/
pmat refactor auto
```

### AI Request Customization

The tool generates AI requests that can be customized for different AI providers:

```json
{
  "model_preferences": {
    "primary": "claude-3-opus",
    "fallback": "gpt-4-turbo",
    "local": "codellama-34b"
  },
  "request_template": {
    "system_prompt": "You are an expert Rust developer focused on extreme quality standards...",
    "max_tokens": 8192,
    "temperature": 0.1
  }
}
```

## Best Practices

### 1. Incremental Refactoring
```bash
# Daily incremental improvement
pmat refactor auto --max-iterations 1

# Weekly comprehensive refactoring
pmat refactor auto --max-iterations 10
```

### 2. Quality Gate Integration
```bash
# Block merges that don't meet standards
pmat refactor auto --ci-mode || exit 1
```

### 3. Documentation Updates
```bash
# Update documentation after refactoring
pmat refactor auto && cargo doc --no-deps
```

### 4. Monitoring Progress
```bash
# Track quality improvements over time
pmat refactor auto --format json | \
  jq '.quality_metrics' | \
  tee -a quality-history.json
```

## Conclusion

The `pmat refactor auto` command represents a paradigm shift in code quality management, providing:

- **Automated Excellence**: No manual intervention required
- **Consistent Standards**: Same quality bar across all projects
- **AI-Powered Intelligence**: Leverages cutting-edge AI for refactoring
- **Toyota Way Philosophy**: Focus on root causes and continuous improvement

By integrating this tool into your development workflow, you can maintain extremely high code quality standards with minimal effort, allowing your team to focus on feature development while ensuring technical excellence.