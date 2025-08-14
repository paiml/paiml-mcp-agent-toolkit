# Technical Debt Gradient (TDG) Analysis Examples

## Overview

The Technical Debt Gradient (TDG) is a comprehensive metric that quantifies accumulated technical debt in your codebase. Unlike simple metrics, TDG combines multiple factors to provide a holistic view of code quality.

## Basic Usage

### Analyze Current Directory

```bash
# Analyze current directory with default settings
pmat analyze tdg

# Output:
# Technical Debt Gradient Analysis
# 
# 📊 Total Files Analyzed: 451
# 🔴 Critical Files: 0 (0.0%)
# 🟡 Warning Files: 22 (4.9%)
# 📈 Average TDG: 0.73
# 📊 95th Percentile: 1.50
# 📊 99th Percentile: 1.80
# ⏱️ Estimated Debt: 1421.3 hours
```

### Show Top Hotspots

```bash
# Show top 5 files with highest TDG scores
pmat analyze tdg --top-files 5

# Output includes table of hotspots:
# | File | TDG Score | Primary Factor | Est. Hours |
# |------|-----------|----------------|------------|
# | ./server/src/cli/handlers/refactor_auto_handlers.rs | 2.19 | High Complexity | 7.2 |
# | ./server/src/cli/mod.rs | 2.05 | High Complexity | 6.7 |
```

## Advanced Usage

### JSON Output with Component Breakdown

```bash
# Get detailed JSON output with component weights
pmat analyze tdg --format json --include-components

# Output:
{
  "summary": {
    "total_files": 451,
    "critical_files": 0,
    "warning_files": 22,
    "average_tdg": 0.73,
    "p95_tdg": 1.50,
    "p99_tdg": 1.80,
    "estimated_debt_hours": 1421.3
  },
  "hotspots": [...],
  "components": {
    "complexity_weight": 0.30,
    "churn_weight": 0.35,
    "coupling_weight": 0.15,
    "domain_risk_weight": 0.10,
    "duplication_weight": 0.10
  }
}
```

### Critical Files Only

```bash
# Show only files with TDG > 2.5 (critical threshold)
pmat analyze tdg --critical-only --format markdown

# Output in markdown format for documentation
```

### CI/CD Integration with SARIF

```bash
# Generate SARIF output for GitHub/GitLab integration
pmat analyze tdg --format sarif --output tdg-report.sarif

# The SARIF format can be uploaded to GitHub Code Scanning
```

## TDG Score Interpretation

| Score Range | Risk Level | Action Required |
|-------------|------------|-----------------|
| 0.0 - 0.5 | Minimal | Maintain current practices |
| 0.5 - 1.0 | Low | Monitor trends |
| 1.0 - 1.5 | Moderate | Plan debt reduction |
| 1.5 - 2.0 | High | Prioritize refactoring |
| > 2.0 | Critical | Immediate action needed |

## Component Factors

TDG is calculated using weighted components:

1. **Complexity (30%)**: Cyclomatic and cognitive complexity
2. **Code Churn (35%)**: Frequency of changes over time
3. **Coupling (15%)**: Dependencies between modules
4. **Domain Risk (10%)**: Critical areas like auth, crypto
5. **Duplication (10%)**: Code duplication percentage

## Example: Analyzing a Complex File

Create a file with high complexity:

```rust
// examples/analysis/tdg_demo.rs
pub struct CacheManager {
    storage: Arc<Mutex<HashMap<String, CacheEntry>>>,
    config: CacheConfig,
    stats: Arc<Mutex<CacheStats>>,
}

impl CacheManager {
    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        let mut storage = self.storage.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();
        
        if let Some(entry) = storage.get_mut(key) {
            if entry.created_at.elapsed() > entry.ttl {
                stats.total_size -= entry.size;
                storage.remove(key);
                stats.misses += 1;
                return None;
            }
            // ... more complex logic
        }
    }
}
```

Analyze the file:

```bash
pmat analyze tdg --path examples/analysis --format json

# Output:
{
  "summary": {
    "total_files": 1,
    "warning_files": 1,
    "average_tdg": 2.49
  },
  "hotspots": [{
    "path": "examples/analysis/tdg_demo.rs",
    "tdg_score": 2.49,
    "primary_factor": "High Complexity",
    "estimated_hours": 8.7
  }]
}
```

## Monitoring TDG Over Time

```bash
# Save baseline
pmat analyze tdg --format json --output tdg-baseline.json

# After refactoring, compare
pmat analyze tdg --format json --output tdg-after.json

# Use jq to compare
jq -r '.summary.average_tdg' tdg-baseline.json tdg-after.json
```

## Integration with CI/CD

### GitHub Actions Example

```yaml
name: TDG Check
on: [push, pull_request]

jobs:
  tdg-analysis:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install pmat
        run: cargo install pmat
      
      - name: Run TDG Analysis
        run: |
          pmat analyze tdg --format json > tdg-report.json
          
          # Extract average TDG
          avg_tdg=$(jq -r '.summary.average_tdg' tdg-report.json)
          
          # Fail if TDG > 1.5
          if (( $(echo "$avg_tdg > 1.5" | bc -l) )); then
            echo "❌ Average TDG $avg_tdg exceeds threshold 1.5"
            exit 1
          fi
          
      - name: Upload SARIF
        if: always()
        run: |
          pmat analyze tdg --format sarif --output tdg.sarif
          # Upload to GitHub Code Scanning
```

## Best Practices

1. **Regular Monitoring**: Run TDG analysis weekly to track trends
2. **Set Thresholds**: Establish team-specific TDG thresholds
3. **Focus on Hotspots**: Address highest TDG files first
4. **Track Progress**: Monitor TDG reduction after refactoring
5. **Integrate with CI**: Fail builds when TDG exceeds limits

## Common Patterns

### High Complexity Files
- Long functions with nested conditionals
- Multiple responsibilities in single class
- Complex state management

### High Churn Files
- Frequently modified configuration files
- Unstable APIs under development
- Files with many bug fixes

### High Coupling Files
- God objects with many dependencies
- Circular dependencies between modules
- Tight coupling to external systems

## Remediation Strategies

Based on the primary factor:

### For High Complexity
```bash
# Use automated refactoring
pmat refactor auto --file high-complexity-file.rs
```

### For High Churn
- Add comprehensive tests
- Stabilize interfaces
- Consider architectural changes

### For High Coupling
- Introduce abstractions
- Apply dependency injection
- Break circular dependencies

## Troubleshooting

### No Git Repository
TDG analysis requires git history for churn calculation. If you see:
```
TDG analysis failed: Not found: No git repository found
```

Initialize a git repository or use the tool in a git-managed project.

### Performance Issues
For large codebases:
```bash
# Analyze specific directories
pmat analyze tdg --path src/core --top-files 20

# Exclude test files
pmat analyze tdg --exclude "**/test_*.rs"
```

## See Also

- [Complexity Analysis](./complexity-analysis.md)
- [Code Churn Analysis](./churn-analysis.md)
- [Quality Gate](./quality-gate.md)
- [Automated Refactoring](./refactor-auto.md)