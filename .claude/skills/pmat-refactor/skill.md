---
name: Automated Refactoring with PMAT
description: |
  Provides automated refactoring suggestions and complexity reduction strategies using PMAT
  (Pragmatic AI Labs MCP Agent Toolkit). Use this skill when:
  - User requests code refactoring, optimization, or improvement
  - Complexity analysis reveals high-complexity functions (cyclomatic > 10)
  - Code review identifies maintainability issues
  - Technical debt needs to be addressed systematically
  - Preparing legacy code for modernization
  Supports 25+ languages with data-driven refactoring recommendations based on complexity metrics,
  mutation testing results, and industry best practices (Fowler's refactoring catalog).
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# PMAT Automated Refactoring Skill

You are an expert at identifying refactoring opportunities and applying systematic code improvements using PMAT (Pragmatic AI Labs MCP Agent Toolkit).

## When to Activate

This skill should automatically activate when:
1. User mentions "refactor", "optimize", "improve", or "simplify" code
2. Complexity analysis reveals functions with cyclomatic complexity > 10
3. User requests code modernization or technical debt reduction
4. Preparing code for new features (pre-refactoring)
5. Code review reveals maintainability concerns

## Core Refactoring Workflow

### Step 1: Baseline Analysis
Before any refactoring, establish baseline metrics:
```bash
# Analyze current complexity
pmat analyze complexity --path <target_file_or_directory> --output baseline_complexity.json

# Analyze quality metrics
pmat analyze quality --path <target_file_or_directory> --output baseline_quality.json

# Identify dead code
pmat analyze dead-code --path <target_file_or_directory> --output dead_code.json
```

### Step 2: Identify Refactoring Targets
Prioritize based on:
- **Cyclomatic Complexity > 10**: High risk, needs simplification
- **Cognitive Complexity > 15**: High mental load
- **Maintainability Index < 50**: Difficult to maintain
- **Code Duplication > 5%**: Extract common patterns
- **Dead Code**: Remove unused functions/imports

### Step 3: Apply Refactoring Patterns
Use industry-standard refactoring patterns (Fowler, 1999):

#### Pattern 1: Extract Method
**When**: Function has cyclomatic complexity > 10
**Goal**: Break down complex function into smaller, testable units

```bash
# Analyze function complexity
pmat analyze complexity --path src/services/parser.rs --format detailed

# Identify extraction candidates (high complexity blocks)
# Manual extraction using Edit tool, guided by complexity hotspots
```

#### Pattern 2: Simplify Conditionals
**When**: Deeply nested if/else statements (nesting depth > 3)
**Goal**: Reduce cognitive complexity using early returns, guard clauses

```bash
# Identify nesting depth issues
pmat analyze complexity --path src/handlers/request.rs --metrics cognitive

# Apply transformations:
# - Replace nested if with early returns
# - Extract condition logic into named functions
# - Use pattern matching where applicable
```

#### Pattern 3: Remove Dead Code
**When**: Unused functions, imports, or variables detected
**Goal**: Reduce codebase size, improve maintainability

```bash
# Detect dead code
pmat analyze dead-code --path src/ --output dead_code_report.json

# Review and remove confirmed dead code
# Use Edit tool to safely remove unused code
```

#### Pattern 4: Extract Class/Module
**When**: Single file exceeds 500 LOC or handles multiple responsibilities
**Goal**: Improve modularity, single responsibility principle

```bash
# Analyze file size and responsibilities
pmat context --path src/large_module.py --output module_analysis.md

# Identify cohesive responsibilities
# Create new modules/classes
# Migrate related functions
```

#### Pattern 5: Reduce Duplication
**When**: Code duplication > 5% detected
**Goal**: Extract common patterns, improve DRY principle

```bash
# Detect code duplication
pmat analyze duplication --path src/ --threshold 5 --output duplication.json

# Extract duplicated code into shared functions/modules
```

### Step 4: Verify Refactoring Impact
After refactoring, measure improvements:
```bash
# Re-run complexity analysis
pmat analyze complexity --path <refactored_files> --output after_complexity.json

# Compare before/after metrics
pmat compare --baseline baseline_complexity.json --current after_complexity.json

# Ensure quality improved
pmat analyze quality --path <refactored_files> --output after_quality.json
```

## Example Refactoring Workflows

### Example 1: Simplify High-Complexity Function

**Scenario**: Function has cyclomatic complexity of 18

```bash
# Step 1: Analyze current state
pmat analyze complexity --path src/services/validator.rs

# Output (hypothetical):
# Function: validate_request
# Cyclomatic Complexity: 18 (HIGH)
# Cognitive Complexity: 24 (VERY HIGH)
# Lines: 150
# Nesting Depth: 4 (HIGH)

# Step 2: Identify refactoring opportunities
# - Extract validation logic into separate functions (lines 45-80)
# - Replace nested if statements with early returns (lines 90-120)
# - Extract complex condition into named function (line 105)

# Step 3: Apply refactorings using Edit tool
# [Use Edit tool to implement changes]

# Step 4: Verify improvement
pmat analyze complexity --path src/services/validator.rs

# Expected output:
# Function: validate_request (refactored)
# Cyclomatic Complexity: 6 (GOOD) ← Reduced from 18
# Cognitive Complexity: 8 (GOOD) ← Reduced from 24
# Lines: 80 ← Reduced from 150
# Nesting Depth: 2 (GOOD) ← Reduced from 4

# Improvement: 67% reduction in cyclomatic complexity
```

### Example 2: Dead Code Cleanup

```bash
# Step 1: Identify dead code
pmat analyze dead-code --path src/

# Output:
# Found 15 unused functions:
# 1. src/utils/formatter.rs::legacy_format() - Last used in v1.2
# 2. src/helpers/validation.rs::old_validator() - Replaced by new_validator()
# 3. src/api/deprecated.rs::old_endpoint() - Deprecated in v2.0
# ...

# Step 2: Review and confirm removal safety
# Check git history, grep for usage, review tests

# Step 3: Remove dead code using Edit tool
# [Remove each unused function]

# Step 4: Measure impact
# - Binary size reduced by ~45KB
# - Maintainability index improved from 62 to 75
# - Reduced cognitive load for developers
```

### Example 3: Extract Method Refactoring

```bash
# Step 1: Identify extraction candidate
pmat analyze complexity --path src/core/processor.rs --format detailed

# Output:
# Function: process_data
# Lines 100-180: Complexity 12 (candidate for extraction)
# Lines 200-250: Complexity 8 (candidate for extraction)

# Step 2: Extract complex sections into named methods
# Before:
# fn process_data(input: &str) -> Result<Data> {
#     // Lines 100-180: Data validation (complexity 12)
#     // Lines 200-250: Data transformation (complexity 8)
# }

# After (using Edit tool):
# fn process_data(input: &str) -> Result<Data> {
#     let validated = validate_input_data(input)?;
#     transform_validated_data(validated)
# }
#
# fn validate_input_data(input: &str) -> Result<Data> {
#     // Extracted validation logic
# }
#
# fn transform_validated_data(data: Data) -> Result<Data> {
#     // Extracted transformation logic
# }

# Step 3: Verify improvement
pmat analyze complexity --path src/core/processor.rs

# Results:
# - process_data: Complexity 3 (SIMPLE) ← Reduced from 20
# - validate_input_data: Complexity 6 (MODERATE)
# - transform_validated_data: Complexity 4 (SIMPLE)
# Total complexity: 13 (distributed across 3 testable functions)
```

## Refactoring Decision Matrix

Use this matrix to prioritize refactoring efforts:

| Complexity | Churn (changes/month) | Priority | Action |
|------------|----------------------|----------|--------|
| High (>15) | High (>10) | CRITICAL | Refactor immediately |
| High (>15) | Low (<3) | HIGH | Refactor when modifying |
| Medium (8-15) | High (>10) | HIGH | Simplify hot paths |
| Medium (8-15) | Low (<3) | MEDIUM | Monitor, refactor if needed |
| Low (<8) | Any | LOW | No action needed |

**Generate this matrix using**:
```bash
# Combine complexity + churn analysis
pmat analyze complexity --path src/ --output complexity.json
pmat analyze churn --path src/ --since "6 months ago" --output churn.json
pmat refactor-matrix --complexity complexity.json --churn churn.json
```

## Refactoring Safety Checklist

Before applying refactorings:
1. ✅ **Baseline metrics captured** (complexity, quality, test coverage)
2. ✅ **Tests exist and pass** (run `pmat test` or project test suite)
3. ✅ **Git commit created** (safe rollback point)
4. ✅ **Refactoring scope defined** (one pattern at a time)
5. ✅ **Impact understood** (dependencies, callers identified)

After applying refactorings:
1. ✅ **Tests still pass** (regression check)
2. ✅ **Complexity reduced** (verify with `pmat analyze complexity`)
3. ✅ **Functionality preserved** (manual smoke test)
4. ✅ **Code review requested** (peer validation)

## Integration with PMAT Quality Analysis

Refactoring should be guided by quality metrics:

```bash
# Generate comprehensive quality report
pmat analyze quality --path src/ --output quality_report.json

# Identify refactoring targets from quality report
# - Functions with maintainability index < 50
# - Files with technical debt > 10 hours
# - Modules with high coupling scores

# Apply targeted refactorings
# [Use patterns above]

# Verify quality improvement
pmat analyze quality --path src/ --output quality_after.json
pmat compare-quality --before quality_report.json --after quality_after.json
```

## Scientific Foundation

PMAT refactoring implements peer-reviewed principles:

1. **Fowler's Refactoring Catalog** (1999)
   - Extract Method, Simplify Conditional, Remove Dead Code
   - Behavior-preserving transformations

2. **McCabe's Cyclomatic Complexity** (1976)
   - Threshold: 10 for well-structured code
   - Predictor of defect density

3. **Cognitive Complexity** (SonarSource, 2021)
   - Measures mental effort required to understand code
   - Guide for simplification priorities

4. **Technical Debt Quadrant** (Fowler, 2009)
   - Distinguish deliberate vs. inadvertent debt
   - Prioritize repayment strategies

## Performance Optimization Tips

For large codebases:
1. **Incremental Refactoring**: Refactor one file/function at a time
2. **Hot Path Priority**: Focus on frequently changed files first
3. **Complexity Threshold**: Only refactor complexity > 10 (Pareto principle)
4. **Measure Impact**: Track time saved via maintainability improvements

## Common Refactoring Patterns by Language

### Rust
- Replace `unwrap()` with proper error handling (`?` operator)
- Extract complex `match` arms into functions
- Use `impl Trait` to simplify return types
- Apply lifetime elision rules

### Python
- Extract nested functions for clarity
- Replace complex comprehensions with explicit loops
- Use `dataclasses` for data structures
- Apply type hints for better IDE support

### JavaScript/TypeScript
- Extract arrow function logic into named functions
- Replace callback hell with `async/await`
- Use destructuring to simplify parameter passing
- Apply optional chaining (`?.`) to reduce null checks

### Go
- Extract error handling into helper functions
- Simplify interface implementations
- Use `defer` for cleanup logic
- Apply table-driven tests

## When NOT to Refactor

Avoid refactoring in these scenarios:
1. **No Tests**: Without tests, refactoring is unsafe (write tests first)
2. **Unclear Requirements**: Understand the domain before changing code
3. **Hot Production Issues**: Fix bugs first, refactor later
4. **Large-Scale Changes**: Break into smaller, incremental refactorings
5. **Working Code**: If complexity is low and code is stable, leave it

## Limitations

- **Semantic Preservation**: PMAT suggests refactorings but cannot guarantee behavior preservation (test coverage required)
- **Context Awareness**: Automated suggestions may not account for domain-specific constraints
- **Language-Specific Idioms**: Some refactorings require language-specific expertise
- **External Dependencies**: Refactoring may require updating API contracts

## Error Handling

If refactoring causes issues:
1. **Rollback**: `git reset --hard HEAD` (use the commit you created before refactoring)
2. **Incremental Revert**: Use Edit tool to undo specific changes
3. **Test Isolation**: Identify which refactoring broke tests
4. **Consult Baseline**: Compare against baseline metrics to identify regressions

## Best Practices

1. **One Pattern at a Time**: Apply one refactoring pattern per commit
2. **Test After Each Change**: Verify tests pass after each refactoring
3. **Commit Frequently**: Create rollback points for safety
4. **Measure Impact**: Track complexity reduction and quality improvement
5. **Document Rationale**: Explain why refactoring was needed (commit messages, code comments)
6. **Peer Review**: Get feedback on refactoring approach before merging

## Integration with Other PMAT Skills

**Workflow Recommendation**:
1. **pmat-context**: Understand codebase structure
2. **pmat-quality**: Identify quality issues
3. **pmat-refactor**: Apply systematic improvements ← This skill
4. **pmat-tech-debt**: Track debt repayment progress

## Version Requirements

- **Minimum**: PMAT v2.170.0
- **Recommended**: Latest version for best refactoring suggestions
- **Check version**: `pmat --version`

---

**Remember**: Refactoring is about improving code structure WITHOUT changing behavior. Always start with tests, establish baselines, apply one pattern at a time, and verify improvements with data.
