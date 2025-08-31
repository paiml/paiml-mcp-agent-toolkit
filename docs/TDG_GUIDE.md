# Technical Debt Grading (TDG) Guide

The Technical Debt Grading (TDG) system provides comprehensive code quality scoring using six orthogonal metrics. This guide covers everything you need to know about using TDG for code analysis.

## Overview

TDG analyzes code quality across six dimensions:

1. **Structural Complexity** (25 points) - Cyclomatic complexity, nesting depth
2. **Semantic Complexity** (20 points) - Cognitive complexity patterns  
3. **Code Duplication** (20 points) - Duplicate code detection and quantification
4. **Coupling Score** (15 points) - Import/dependency analysis
5. **Documentation Coverage** (10 points) - Language-specific documentation patterns
6. **Consistency Score** (10 points) - Naming conventions and code style

Total possible score: **100 points** with letter grades A+ through F.

## Quick Start

### Basic Analysis

```bash
# Analyze a single file
pmat tdg src/main.rs

# Analyze entire project with top files
pmat tdg . --top-files 10

# Include component breakdown
pmat tdg . --include-components

# Web dashboard (NEW v2.39.0)
pmat tdg dashboard --port 8081 --open
```

### Output Formats (8 Formats in v2.39.0)

```bash
# Human-readable table (default)
pmat tdg . --format table

# JSON for programmatic use
pmat tdg . --format json

# SARIF for CI/CD integration
pmat tdg . --format sarif

# CSV for spreadsheet analysis
pmat tdg . --format csv

# HTML report with visualizations
pmat tdg . --format html

# Markdown for documentation
pmat tdg . --format markdown

# XML for enterprise systems
pmat tdg . --format xml

# Prometheus metrics format
pmat tdg . --format prometheus
```

### Filtering Results

```bash
# Show only files above threshold
pmat tdg . --threshold 2.0

# Show only critical issues (grade below C)
pmat tdg . --critical-only

# Limit number of results
pmat tdg . --top-files 5

# Filter by storage backend (NEW v2.39.0)
pmat tdg . --storage-backend sled
pmat tdg . --storage-backend inmemory
pmat tdg . --storage-backend rocksdb
```

## Understanding TDG Scores

### Grade Scale

| Grade | Score Range | Description |
|-------|-------------|-------------|
| A+    | 95-100      | Exceptional code quality |
| A     | 90-94.9     | Excellent code quality |
| A-    | 85-89.9     | Very good code quality |
| B+    | 80-84.9     | Good code quality |
| B     | 75-79.9     | Acceptable code quality |
| B-    | 70-74.9     | Below average quality |
| C+    | 65-69.9     | Poor code quality |
| C     | 60-64.9     | Very poor code quality |
| C-    | 55-59.9     | Critical quality issues |
| D     | 50-54.9     | Severe quality problems |
| F     | < 50        | Unacceptable code quality |

### Metric Breakdown

#### Structural Complexity (25 points)
Measures code structure complexity through:
- **Cyclomatic Complexity**: Decision points in code (if, for, while, match)
- **Nesting Depth**: Maximum depth of nested control structures

**Penalties Applied For:**
- Cyclomatic complexity > 10 (configurable)
- Nesting depth > 3 levels (configurable)

#### Semantic Complexity (20 points)
Evaluates cognitive load through:
- **Deep Nesting**: Heavily nested code structures
- **Complex Conditional Logic**: Intricate boolean expressions

**Penalties Applied For:**
- Nesting depth > 4 levels
- Complex pattern matching

#### Code Duplication (20 points)
Detects and measures code duplication:
- **Line-based Duplication**: Identical or similar lines
- **Pattern Detection**: Repeated code patterns

**Penalties Applied For:**
- Duplication ratio > 10%
- Lines longer than 10 characters that appear multiple times

#### Coupling Score (15 points)
Analyzes dependencies and coupling:
- **Import Analysis**: Number and complexity of imports
- **Dependency Tracking**: External library usage

**Penalties Applied For:**
- More than 20 imports in a single file
- High external dependency usage

#### Documentation Coverage (10 points)
Measures documentation quality:

**Language-Specific Patterns:**
- **Rust**: `///` doc comments, `//!` module docs
- **Python**: `"""` docstrings, `'''` docstrings  
- **JavaScript/TypeScript**: `/**` JSDoc comments
- **Other Languages**: `//` and `/*` comments

**Scoring:** Based on ratio of documentation lines to total lines

#### Consistency Score (10 points)
Evaluates code style consistency:
- **Indentation**: Tab vs. space consistency
- **Naming Conventions**: Variable and function naming patterns
- **Code Style**: Consistent formatting patterns

## Language Support

TDG supports analysis for 10+ programming languages:

| Language     | Extension | Documentation Pattern | Confidence |
|--------------|-----------|----------------------|------------|
| Rust         | `.rs`     | `///`, `//!`         | High       |
| Python       | `.py`     | `"""`, `'''`         | High       |
| JavaScript   | `.js`     | `/**`, `*`           | High       |
| TypeScript   | `.ts`     | `/**`, `*`           | High       |
| Go           | `.go`     | `//`, `/*`           | Medium     |
| Java         | `.java`   | `/**`, `*`           | Medium     |
| C            | `.c, .h`  | `//`, `/*`           | Medium     |
| C++          | `.cpp, .hpp` | `//`, `/*`        | Medium     |
| Ruby         | `.rb`     | `#`                  | Medium     |
| Swift        | `.swift`  | `///`, `/**`         | Medium     |
| Kotlin       | `.kt`     | `/**`, `*`           | Medium     |

## MCP Integration (v2.39.0 Enterprise Tools)

TDG provides 6 enterprise-grade MCP tools for external integration:

### Available MCP Tools

#### `tdg_analyze_with_storage` (NEW v2.39.0)
Analyze files with configurable storage backends.

**Parameters:**
```json
{
  "paths": ["src/", "tests/"],
  "storage_backend": "sled",
  "priority": "high"
}
```

#### `tdg_system_diagnostics` (NEW v2.39.0)
Comprehensive system health monitoring.

**Parameters:**
```json
{
  "detailed": true,
  "components": ["storage", "performance", "alerts"]
}
```

#### `tdg_storage_management` (NEW v2.39.0)
Storage operations and management.

**Parameters:**
```json
{
  "action": "flush",
  "options": {"force": true}
}
```

#### `tdg_performance_profiling` (NEW v2.39.0)
Generate performance profiles and flame graphs.

**Parameters:**
```json
{
  "target_path": "src/tdg/",
  "profile_type": "flame_graph",
  "duration_seconds": 30
}
```

#### `tdg_alert_management` (NEW v2.39.0)
Configure and manage alerts.

**Parameters:**
```json
{
  "action": "configure",
  "threshold_type": "cpu_usage",
  "threshold_value": 85.0
}
```

#### `tdg_export_data` (NEW v2.39.0)
Export TDG data in multiple formats.

**Parameters:**
```json
{
  "paths": ["."],
  "format": "prometheus",
  "output_path": "./tdg-metrics.prom"
}
```

#### Legacy Tools
#### `analyze_tdg`
Basic TDG analysis (legacy interface).

#### `analyze_tdg_compare`
Compare TDG scores between files.

### Usage in Claude Code (v2.39.0)

```bash
# Start MCP server with TDG tools
pmat mcp serve --port 3000

# In Claude Code, use tools:
# - tdg_analyze_with_storage for cached analysis
# - tdg_system_diagnostics for health monitoring
# - tdg_storage_management for storage operations
# - tdg_performance_profiling for flame graphs
# - tdg_alert_management for alert configuration
# - tdg_export_data for multi-format export
```

## Advanced Usage

### Configuration

TDG behavior can be customized through configuration files:

```toml
# ~/.pmat/tdg-config.toml
[weights]
structural_complexity = 30.0
semantic_complexity = 20.0
duplication = 15.0
coupling = 15.0
documentation = 10.0
consistency = 10.0

[thresholds]
max_cyclomatic_complexity = 15
max_nesting_depth = 4
min_doc_coverage = 0.7
```

### Integration with Quality Gates

```bash
# Use TDG in quality gates
pmat quality-gate --tdg-threshold 75.0

# Fail build on poor TDG scores
pmat tdg . --critical-only --format json | \
  jq '.results.files[] | select(.total < 60)' && exit 1

# Enforce grade thresholds (NEW v2.39.0)
pmat tdg . --enforce-thresholds --fail-on-grade-below A-

# Dashboard-based monitoring
pmat tdg dashboard --background --alert-on-regression
```

### CI/CD Integration (v2.39.0)

```yaml
# GitHub Actions example with multiple formats
- name: TDG Analysis
  run: |
    # Install latest from crates.io
    cargo install pmat --version 2.39.0 --force
    
    # Run TDG analysis with multiple exports
    pmat tdg . --format json > tdg-results.json
    pmat tdg . --format sarif > tdg-results.sarif
    pmat tdg . --format prometheus > tdg-metrics.prom
    
    # Fail if average score below threshold
    SCORE=$(jq '.results.average_score' tdg-results.json)
    [ $(echo "$SCORE > 70" | bc -l) -eq 1 ] || exit 1
    
    # Upload SARIF results
    - uses: github/codeql-action/upload-sarif@v2
      with:
        sarif_file: tdg-results.sarif
```

## Example Output

### Human-Readable Format

```
╭─────────────────────────────────────────────────╮
│  TDG Score Report: src/main.rs                 │
├─────────────────────────────────────────────────┤
│  Overall Score: 78.5/100 (B)                  │
│  Language: Rust (confidence: 100%)             │
│                                                 │
│  📊 Breakdown:                                  │
│  ├─ Structural:     14.5/25  █████░░░░░        │
│  ├─ Semantic:       19.0/20  █████████░        │
│  ├─ Duplication:    20.0/20  ██████████        │
│  ├─ Coupling:       15.0/15  ██████████        │
│  ├─ Documentation:   0.0/10  ░░░░░░░░░░        │
│  └─ Consistency:    10.0/10  ██████████        │
│                                                 │
│  🔍 Issues Found:                               │
│    • High cyclomatic complexity: 31              │
│    • Deep nesting: 4 levels                      │
│                                                 │
│  👍 Good code quality with minor improvements. │
╰─────────────────────────────────────────────────╯
```

### JSON Format

```json
{
  "status": "completed",
  "message": "TDG file analysis completed",
  "result_type": "file",
  "results": {
    "structural_complexity": 14.5,
    "semantic_complexity": 19.0,
    "duplication_ratio": 20.0,
    "coupling_score": 15.0,
    "doc_coverage": 0.0,
    "consistency_score": 10.0,
    "total": 78.5,
    "grade": "B",
    "confidence": 1.0,
    "language": "Rust",
    "file_path": "src/main.rs",
    "penalties_applied": [
      {
        "source_metric": "StructuralComplexity",
        "amount": 10.5,
        "applied_to": ["StructuralComplexity"],
        "issue": "High cyclomatic complexity: 31"
      }
    ]
  }
}
```

## Best Practices

### Improving TDG Scores

1. **Reduce Structural Complexity**
   - Break down large functions
   - Reduce nesting depth
   - Simplify conditional logic

2. **Enhance Documentation**
   - Add doc comments to public functions
   - Document complex algorithms
   - Include usage examples

3. **Eliminate Duplication**
   - Extract common code to functions
   - Use design patterns appropriately
   - Refactor similar code blocks

4. **Reduce Coupling**
   - Minimize dependencies
   - Use dependency injection
   - Apply SOLID principles

5. **Improve Consistency**
   - Use consistent naming conventions
   - Follow language style guides
   - Use automated formatters

### Continuous Monitoring (v2.39.0)

```bash
# Weekly TDG reports with dashboard
pmat tdg . --format markdown > reports/tdg-$(date +%Y-%m-%d).md

# Real-time dashboard monitoring
pmat tdg dashboard --background --port 8081 &

# Automated alert monitoring
pmat tdg alerts --start-monitoring --config-file alerts.toml

# Export time-series metrics
pmat tdg export . --format prometheus --time-series

# Track improvements over time
git log --oneline | head -10 | while read commit; do
  echo "=== $commit ==="
  git checkout $(echo $commit | cut -d' ' -f1)
  pmat tdg . --format json | jq '.results.average_score'
done
```

## Troubleshooting

### Common Issues

**"Language not supported"**
- Check file extension is recognized
- Verify file contains actual code (not just comments)

**"No files found for analysis"**
- Ensure path exists and contains supported file types
- Check directory permissions

**"Low confidence score"**
- File may have unusual structure
- Mixed language content
- Generated code detection

### Performance Considerations

- Large projects: Use `--top-files` to limit results
- CI/CD: Consider using `--critical-only` for faster execution
- Network filesystems: Run analysis locally when possible

## API Reference

See the [TDG specification](../specifications/tdg-simplified-spec.md) for complete implementation details.

For integration examples and advanced usage, see:
- [MCP Integration Guide](api-guide.md#mcp-tools)
- [CLI Reference](cli-reference.md#analyze-tdg)
- [Quality Gates Guide](execution/quality-gates.md#tdg-integration)