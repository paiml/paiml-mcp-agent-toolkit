# Complexity Analysis Example

This example demonstrates how to use `pmat analyze complexity` to identify complex code that may need refactoring.

## Basic Usage

### Analyze Entire Codebase
```bash
# Get overview of complexity across the project
pmat analyze complexity

# Expected output:
# Complexity Analysis Summary
# 📊 Files analyzed: 3
# 🔧 Total functions: 374
# 
# Complexity Metrics
# - Median Cyclomatic: 3.0
# - Median Cognitive: 3.0
# - Max Cyclomatic: 10
# - Max Cognitive: 31
# - 90th Percentile Cyclomatic: 6
# - 90th Percentile Cognitive: 11
# 
# ⏱️ Estimated Refactoring Time: 13.2 hours
```

### Find Most Complex Files
```bash
# Show top 3 most complex files
pmat analyze complexity --top-files 3

# Expected output:
# Top Files by Complexity
# 1. refactor_auto_handlers.rs - Cyclomatic: 377, Cognitive: 667, Functions: 93
# 2. deep_context.rs - Cyclomatic: 409, Cognitive: 583, Functions: 144
# 3. stubs.rs - Cyclomatic: 332, Cognitive: 443, Functions: 137
```

### Analyze Specific File
```bash
# Analyze a single file for detailed complexity metrics
pmat analyze complexity --include "server/src/services/complexity.rs"

# Expected output:
# 📊 Files analyzed: 1
# 🔧 Total functions: 56
# 
# Complexity Metrics
# - Median Cyclomatic: 1.0
# - Median Cognitive: 0.0
# - Max Cyclomatic: 6
# - Max Cognitive: 14
```

## Advanced Usage

### Custom Thresholds
```bash
# Set custom complexity thresholds
pmat analyze complexity --max-cyclomatic 5 --max-cognitive 10
```

### Different Output Formats
```bash
# JSON output for tooling integration
pmat analyze complexity --format json

# SARIF format for IDE integration
pmat analyze complexity --format sarif
```

### Save Results to File
```bash
# Save analysis to file
pmat analyze complexity --output complexity-report.json --format json
```

## Interpreting Results

### Complexity Metrics Explained
- **Cyclomatic Complexity**: Number of independent paths through code (decision points)
- **Cognitive Complexity**: How difficult code is to understand (nesting, logic)
- **Functions**: Total number of functions analyzed

### Quality Thresholds
- **Low Complexity**: Cyclomatic ≤ 5, Cognitive ≤ 5
- **Moderate Complexity**: Cyclomatic 6-10, Cognitive 6-15  
- **High Complexity**: Cyclomatic > 10, Cognitive > 15

### Refactoring Priority
Focus refactoring efforts on:
1. Files with highest total complexity scores
2. Functions with cognitive complexity > 15
3. Functions with cyclomatic complexity > 10

## Integration with Quality Gates

```bash
# Use in CI/CD pipelines
pmat analyze complexity --max-cyclomatic 10 --format json > complexity.json

# Check if any functions exceed thresholds
if grep -q '"cyclomatic": [0-9][0-9]' complexity.json; then
    echo "❌ High complexity functions found"
    exit 1
fi
```

## Tips

1. **Start with Overview**: Run `pmat analyze complexity` first to understand overall codebase health
2. **Focus on Hotspots**: Use `--top-files` to identify the most problematic files first
3. **Set Realistic Thresholds**: Start with higher thresholds and gradually lower them
4. **Track Progress**: Save reports over time to measure improvement
5. **Combine with Other Analysis**: Use alongside `pmat analyze satd` and `pmat analyze lint-hotspot` for comprehensive quality assessment