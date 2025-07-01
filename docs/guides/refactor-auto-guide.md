# PMAT Refactor Auto - AI-Powered Automated Refactoring

## Overview

`pmat refactor auto` is an AI-powered automated refactoring tool that systematically improves code quality to meet extreme quality standards. It uses a state machine approach with intelligent heuristics to prioritize and fix quality violations.

## Quality Standards (EXTREME Profile)

The tool enforces the following rigid quality standards:

- **Test Coverage**: ≥ 80% per file
- **Cyclomatic Complexity**: ≤ 10 (target: 5)
- **SATD (Technical Debt)**: 0 (no TODO, FIXME, HACK, XXX)
- **Lint Violations**: 0 (including pedantic and nursery rules)

## Command Usage

```bash
# Basic usage - refactor current project
pmat refactor auto

# Specify project path
pmat refactor auto --project-path ./server

# Dry run mode (show what would be done)
pmat refactor auto --dry-run

# Detailed output format
pmat refactor auto --format detailed

# JSON output for CI/CD integration
pmat refactor auto --format json

# Set maximum iterations
pmat refactor auto --max-iterations 20

# Use checkpoint for resumable refactoring
pmat refactor auto --checkpoint ./refactor-checkpoint.json

# Skip compilation checks (faster but less safe)
pmat refactor auto --skip-compilation

# Skip test execution (not recommended)
pmat refactor auto --skip-tests
```

## Refactoring Heuristics

The tool uses a sophisticated prioritization algorithm:

### 0. HIGHEST PRIORITY: Compilation Errors
When the code doesn't compile:
- Automatically detects compilation failures
- Runs `cargo check` to identify error locations
- Treats compilation errors as critical violations
- Prioritizes files with most compilation errors first

### 1. Primary Mode: Lint Violations
When lint violations exist, the tool:
- Identifies the file with HIGHEST violation count
- Uses three-tier sorting:
  1. PRIMARY: Largest count of lint defects (descending)
  2. SECONDARY: Severity score of violations
  3. TERTIARY: Code coverage (lowest first)
- Groups violations by type for systematic fixes
- Applies automated fixes where possible
- Generates AI requests for complex refactoring

### 2. Fallback Mode 1: High Complexity
When no lint violations remain but complexity > 10:
- Finds functions with highest cyclomatic complexity
- Extracts methods to reduce complexity
- Simplifies conditional logic
- Removes dead code paths

### 3. Fallback Mode 2: SATD Cleanup
When complexity is acceptable but SATD exists:
- Locates TODO, FIXME, HACK comments
- Implements missing functionality
- Removes workarounds with proper solutions
- Documents resolved technical debt

### 4. Fallback Mode 3: Coverage-Driven
When other metrics pass but coverage < 80%:
- Prioritizes largest files with lowest coverage
- Generates comprehensive test suites
- Adds edge case testing
- Ensures 80% coverage per file

## State Machine Architecture

```
┌─────────────┐
│ Initialize  │
└──────┬──────┘
       ▼
┌─────────────┐
│  Analyze    │◄────────────┐
└──────┬──────┘             │
       ▼                    │
┌─────────────┐             │
│Select Target│             │
└──────┬──────┘             │
       ▼                    │
┌─────────────┐             │
│  Refactor   │             │
└──────┬──────┘             │
       ▼                    │
┌─────────────┐             │
│  Validate   │─────────────┘
└──────┬──────┘    (iterate)
       ▼
┌─────────────┐
│  Complete   │
└─────────────┘
```

## Output Formats

### Summary Format (Default)
```
🚀 Refactor Auto Progress
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Iteration: 3/10
Overall Progress: 45.2%

Quality Metrics:
  Lint Violations: 124 → 62 (-50.0%)
  Max Complexity: 45 → 18 (-60.0%)
  SATD Items: 12 → 4 (-66.7%)
  Test Coverage: 42.3% → 68.5% (+26.2%)

Current Phase: ComplexityReduction
Files Completed: 8/35
Estimated Time: ~15 minutes
```

### Detailed Format
Includes:
- File-by-file progress
- Specific violations being addressed
- Refactoring strategies applied
- Test generation details

### JSON Format
Machine-readable format for CI/CD integration with full state serialization.

## Integration with CI/CD

```yaml
# GitHub Actions example
- name: Run automated refactoring
  run: |
    pmat refactor auto \
      --format json \
      --max-iterations 5 \
      --checkpoint .refactor-state.json \
      > refactor-report.json

- name: Check quality gates
  run: |
    if jq -e '.quality_metrics.total_violations == 0' refactor-report.json; then
      echo "✅ All quality gates passed"
    else
      echo "❌ Quality violations remain"
      exit 1
    fi
```

## Best Practices

1. **Start with Dry Run**: Use `--dry-run` to preview changes
2. **Use Checkpoints**: Enable resumable refactoring with `--checkpoint`
3. **Review AI Suggestions**: The tool outputs AI requests for complex refactoring
4. **Incremental Approach**: Set reasonable `--max-iterations` limits
5. **Version Control**: Commit after each successful iteration

## Performance Considerations

- **AST Analysis**: Can be expensive on large codebases
- **Test Execution**: Adds time but ensures correctness
- **Compilation Checks**: Use `--skip-compilation` carefully
- **Parallelization**: Future versions will support parallel refactoring

## Troubleshooting

### "No more targets" Error
All quality gates have been met! Your code meets extreme quality standards.

### Timeout Issues
- Reduce scope with `--project-path src/specific/module`
- Use `--skip-tests` for faster iteration (not recommended for final runs)
- Increase system resources

### Compilation Failures After Refactoring
- The tool validates compilation by default
- Check the error output for specific issues
- Use `--checkpoint` to resume from last good state

## Example Workflow

```bash
# 1. Analyze current state
pmat analyze quality-gate

# 2. Run automated refactoring with checkpoint
pmat refactor auto \
  --checkpoint ./refactor-state.json \
  --format detailed \
  --max-iterations 10

# 3. If interrupted, resume from checkpoint
pmat refactor auto \
  --checkpoint ./refactor-state.json \
  --max-iterations 5

# 4. Verify all quality gates pass
pmat analyze quality-gate --strict
```

## Future Enhancements

- Parallel file processing
- Custom quality profiles
- Integration with more AI providers
- Incremental refactoring strategies
- Language-specific optimizations