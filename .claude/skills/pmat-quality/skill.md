---
name: Code Quality Analysis with PMAT
description: |
  Analyzes code quality, complexity, and technical debt using PMAT
  (Pragmatic AI Labs MCP Agent Toolkit). Use this skill when:
  - User mentions "code quality", "complexity", "technical debt", or "maintainability"
  - Reviewing code or conducting code review
  - Modifying or refactoring existing code files
  - Creating pull requests or preparing commits
  - Investigating performance or quality issues
  Supports 25+ languages including Rust, Python, TypeScript, JavaScript, Go, C++, Java, Ruby, PHP, Swift, and more.
  Provides cyclomatic complexity, cognitive complexity, maintainability index, dead code detection,
  and technical debt annotations (SATD: TODO, FIXME, HACK comments).
allowed-tools: Bash, Read, Glob, Grep
---

# PMAT Code Quality Analysis Skill

You are an expert code quality analyzer powered by PMAT (Pragmatic AI Labs MCP Agent Toolkit).

## When to Activate

This skill should automatically activate when:
1. User asks about code quality, complexity, or technical debt
2. You are reviewing code files before making changes
3. User requests refactoring or optimization suggestions
4. Creating or reviewing pull requests
5. Investigating performance or maintainability concerns

## Available PMAT Commands

### 1. Quick Quality Analysis
```bash
pmat analyze quality --path <file_or_directory>
```
**Use when**: Quick overview of quality metrics
**Output**: Overall health score, complexity scores, maintainability index

### 2. Complexity Analysis
```bash
pmat analyze complexity --path <file_or_directory>
```
**Use when**: Detailed complexity breakdown
**Output**: Cyclomatic complexity, cognitive complexity per function

### 3. Dead Code Detection
```bash
pmat analyze dead-code --path <file_or_directory>
```
**Use when**: Finding unused code
**Output**: Unused functions, variables, imports

### 4. Technical Debt Detection (SATD)
```bash
pmat analyze satd --path <file_or_directory>
```
**Use when**: Finding technical debt annotations
**Output**: TODO, FIXME, HACK comments with context

### 5. Deep Context Generation
```bash
pmat context --output context.md --format llm-optimized
```
**Use when**: Comprehensive codebase understanding
**Output**: LLM-optimized markdown with architecture, complexity distribution, hotspots

## Usage Workflow

### Step 1: Analyze Before Changes
Before suggesting or making code changes, run quality analysis:
```bash
pmat analyze quality --path <target_directory>
```

### Step 2: Identify Hotspots
Look for:
- Functions with cyclomatic complexity > 10 (McCabe's threshold)
- Cognitive complexity > 15 (high mental load)
- Maintainability index < 50 (difficult to maintain)
- Files with >5 SATD annotations (high technical debt)

### Step 3: Provide Actionable Recommendations
Based on PMAT output, suggest:
- **Extract Method**: Break down complex functions (complexity > 10)
- **Simplify Conditionals**: Reduce nesting depth
- **Remove Dead Code**: Delete unused functions/imports
- **Address Technical Debt**: Prioritize TODO/FIXME comments
- **Improve Documentation**: Add missing docstrings

### Step 4: Measure Impact
After refactoring, re-run analysis to show improvement:
```bash
pmat analyze complexity --path <refactored_file>
```

## Example Usage

### Example 1: Pre-Refactoring Analysis
```bash
# User asks: "Can you help optimize src/services/parser.rs?"

# Step 1: Analyze current state
pmat analyze complexity --path src/services/parser.rs

# Step 2: Review output (hypothetical)
# Function: parse_complex_ast
# Cyclomatic Complexity: 18 (HIGH)
# Cognitive Complexity: 24 (VERY HIGH)
# Lines: 150

# Step 3: Suggest refactoring
# "The parse_complex_ast function has high complexity (cyclomatic: 18, cognitive: 24).
# I recommend:
# 1. Extract method for AST node validation (lines 45-80)
# 2. Simplify conditional logic (lines 90-120)
# 3. Use early returns to reduce nesting"
```

### Example 2: Technical Debt Inventory
```bash
# User asks: "What are the main technical debt issues in the project?"

# Step 1: Run SATD analysis
pmat analyze satd --path .

# Step 2: Categorize and prioritize
# "Found 47 technical debt annotations:
# - 23 TODO comments (deferred features)
# - 18 FIXME comments (known bugs/issues)
# - 6 HACK comments (workarounds that need proper solutions)
#
# High Priority:
# 1. src/core/database.rs:145 - FIXME: SQL injection vulnerability
# 2. src/api/auth.rs:67 - HACK: Temporary token validation bypass
# ..."
```

### Example 3: Dead Code Cleanup
```bash
# User asks: "Are there any unused functions I can remove?"

# Step 1: Detect dead code
pmat analyze dead-code --path src/

# Step 2: Present findings
# "Found 12 unused functions:
# 1. src/utils/formatter.rs::legacy_format() - Last used in v1.2
# 2. src/helpers/validation.rs::old_validator() - Replaced by new_validator()
# ...
#
# Removing these functions would:
# - Reduce binary size by ~45KB
# - Improve maintainability
# - Reduce cognitive load for developers"
```

## Integration with Code Review

When reviewing code:
1. **Automatic Quality Check**: Run `pmat analyze quality` on changed files
2. **Complexity Threshold**: Flag functions with complexity > 10
3. **Technical Debt**: Check for new SATD annotations
4. **Dead Code**: Verify no unused code introduced

## Scientific Foundation

PMAT implements peer-reviewed metrics:
- **Cyclomatic Complexity** (McCabe, 1976): Threshold 10 for well-structured code
- **Cognitive Complexity** (SonarSource, 2021): Measures mental effort required
- **Maintainability Index** (Oman & Hagemeister, 1992): 0-100 scale
- **Technical Debt Annotations** (Potdar & Shihab, 2014): SATD detection

## Output Interpretation

### Quality Scorecard
```
Overall Health: 78/100    (Good)
├─ Complexity Score: 82    (Good - low cyclomatic complexity)
├─ Maintainability: 75     (Fair - room for improvement)
├─ Modularity: 88          (Excellent - well-structured)
└─ Technical Debt: 45 hrs  (Moderate)
```

### Complexity Thresholds
- **1-5**: Simple (low risk)
- **6-10**: Moderate (acceptable)
- **11-20**: High (refactor recommended)
- **21+**: Very High (refactor urgently)

### Maintainability Index
- **85-100**: Excellent (highly maintainable)
- **65-84**: Good (maintainable)
- **50-64**: Fair (moderate effort to maintain)
- **0-49**: Poor (difficult to maintain)

## Best Practices

1. **Run Before Commits**: Check quality before creating commits
2. **Set Quality Gates**: Fail builds if complexity exceeds thresholds
3. **Track Over Time**: Monitor quality trends across sprints
4. **Prioritize Hotspots**: Fix high-complexity, high-churn files first
5. **Document Decisions**: If high complexity is justified, add comments explaining why

## Limitations

- **Binary Files**: PMAT analyzes source code only (not compiled binaries)
- **Generated Code**: May report false positives for auto-generated files
- **DSLs**: Domain-specific languages may have limited support
- **Macros**: Rust procedural macros expanded before analysis

## Error Handling

If PMAT command fails:
1. Check file path exists: `ls -la <path>`
2. Verify language support: `pmat analyze quality --help`
3. Check pmat version: `pmat --version` (requires v2.170.0+)
4. Review error message for specific guidance

## Performance Notes

- **Small files (<1000 LOC)**: <100ms
- **Medium projects (1K-10K LOC)**: <2s
- **Large codebases (100K+ LOC)**: 30-60s
- Use `--path` to analyze specific subdirectories for faster results

## When NOT to Use This Skill

- **Syntax Errors**: PMAT requires syntactically valid code
- **Proprietary Formats**: Binary or encrypted files not supported
- **Real-time Editing**: PMAT analyzes files on disk (not in-memory buffers)
- **Non-Code Files**: Documentation, configs, etc. are not analyzed

## Integration with Other PMAT Features

This skill complements:
- **pmat-context**: Deep codebase understanding
- **pmat-refactor**: Automated refactoring suggestions
- **pmat-tech-debt**: Technical debt tracking
- **pmat-multi-lang**: Multi-language project analysis

## Version Requirements

- **Minimum**: PMAT v2.170.0
- **Recommended**: Latest version for best language support
- **Check version**: `pmat --version`

---

**Remember**: Always analyze code quality BEFORE suggesting changes. Use PMAT to provide data-driven, scientifically grounded recommendations.
