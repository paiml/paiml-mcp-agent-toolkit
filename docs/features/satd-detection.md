# SATD Detection (Self-Admitted Technical Debt)

## Overview

SATD Detection identifies and analyzes Self-Admitted Technical Debt - instances where developers explicitly acknowledge suboptimal solutions through comments. This tool helps teams track, prioritize, and manage technical debt that developers have already identified but haven't yet addressed.

## What is SATD?

Self-Admitted Technical Debt appears in comments like:
- `// TODO: This is a hack, refactor when we have time`
- `// FIXME: This doesn't handle edge cases properly`
- `// HACK: Temporary workaround for production issue`
- `// XXX: This will break if data exceeds 1GB`
- `// REFACTOR: Extract this into a proper service`

## Usage

### Command Line

```bash
# Basic SATD detection
pmat analyze satd

# Filter by severity
pmat analyze satd --severity critical
pmat analyze satd --critical-only

# Include test files
pmat analyze satd --include-tests

# Evolution tracking
pmat analyze satd --evolution --days 90

# Output formats
pmat analyze satd --format json
pmat analyze satd --format sarif

# Top files with most SATD
pmat analyze satd --top-files 10
```

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--severity` | Filter by severity level | all |
| `--critical-only` | Show only critical debt | false |
| `--include-tests` | Include test files | false |
| `--evolution` | Track SATD over time | false |
| `--days` | Days to analyze for evolution | 30 |
| `--metrics` | Include debt metrics | false |
| `--top-files` | Show N files with most SATD | all |

## SATD Categories

### 1. **Design Debt**
Poor architectural decisions that need redesign.

```python
# TODO: This entire module needs to be refactored to use 
# the strategy pattern instead of this massive switch statement
def process_payment(payment_type, amount):
    if payment_type == "credit":
        # 200 lines of credit card logic
    elif payment_type == "debit":
        # 150 lines of debit card logic
    # ... continues for 1000+ lines
```

### 2. **Implementation Debt**
Quick and dirty implementations.

```rust
// HACK: Using a global mutex here because proper 
// dependency injection would require refactoring 
// the entire module
lazy_static! {
    static ref GLOBAL_STATE: Mutex<HashMap<String, String>> = 
        Mutex::new(HashMap::new());
}
```

### 3. **Testing Debt**
Missing or inadequate tests.

```javascript
// TODO: Add tests for error cases
// FIXME: This test is flaky and fails 30% of the time
// XXX: Skipping integration tests until we fix the test database
```

### 4. **Documentation Debt**
Missing or outdated documentation.

```java
// TODO: Document this algorithm - it's based on the paper
// "Efficient Algorithms for X" but with modifications
public double calculateComplexMetric(List<DataPoint> points) {
    // 100 lines of undocumented complex math
}
```

### 5. **Performance Debt**
Known performance issues.

```go
// FIXME: This is O(n²) but should be O(n log n)
// TODO: Replace with a more efficient algorithm when we
// have more than 1000 users
func findMatches(users []User) []Match {
    for i, user1 := range users {
        for j, user2 := range users[i+1:] {
            // Expensive comparison
        }
    }
}
```

## Severity Classification

### Critical (🔴)
- Security vulnerabilities
- Data loss risks
- Performance issues affecting users
- Blocking issues for other features

**Keywords**: `SECURITY`, `CRITICAL`, `URGENT`, `BLOCKER`

### High (🟠)
- Significant technical debt
- Architecture issues
- Missing error handling
- Performance degradation

**Keywords**: `FIXME`, `BUG`, `BROKEN`, `HACK`

### Medium (🟡)
- Code quality issues
- Missing features
- Refactoring needs
- Test debt

**Keywords**: `TODO`, `REFACTOR`, `CLEANUP`, `IMPROVE`

### Low (🟢)
- Nice-to-have improvements
- Documentation updates
- Code style issues

**Keywords**: `NOTE`, `IDEA`, `CONSIDER`, `MAYBE`

## Output Examples

### Default Format

```
🔍 Analyzing SATD in project...

📊 Summary:
- Total SATD comments: 127
- Critical: 5 (3.9%)
- High: 23 (18.1%)
- Medium: 67 (52.8%)
- Low: 32 (25.2%)

🔴 Critical Issues:

1. src/auth/token_validator.rs:45
   // SECURITY: Token validation is disabled for debugging
   // CRITICAL: Must be fixed before production release

2. src/database/migrations.rs:123
   // FIXME: This migration can cause data loss if run twice
   // Need to add idempotency check

📍 Top SATD Hotspots:
1. src/core/engine.rs (12 items)
2. src/api/handlers.rs (8 items)
3. src/utils/parser.rs (7 items)
```

### JSON Format

```json
{
  "summary": {
    "total": 127,
    "by_severity": {
      "critical": 5,
      "high": 23,
      "medium": 67,
      "low": 32
    },
    "by_category": {
      "design": 23,
      "implementation": 45,
      "testing": 34,
      "documentation": 20,
      "performance": 5
    }
  },
  "items": [
    {
      "file": "src/auth/token_validator.rs",
      "line": 45,
      "severity": "critical",
      "category": "implementation",
      "text": "SECURITY: Token validation is disabled for debugging",
      "author": "john.doe",
      "date": "2024-05-15",
      "age_days": 25,
      "context": {
        "function": "validate_token",
        "complexity": 15
      }
    }
  ],
  "evolution": {
    "trend": "increasing",
    "added_last_30_days": 23,
    "resolved_last_30_days": 8,
    "net_change": 15
  }
}
```

## Evolution Tracking

Track how SATD changes over time:

```bash
pmat analyze satd --evolution --days 90
```

**Output:**
```
📈 SATD Evolution (Last 90 Days)

Total SATD Trend:
150 ┤                                    ╱─
140 ┤                              ╱─────
130 ┤                        ╱─────
120 ┤                  ╱─────
110 ┤            ╱─────
100 ┤──────╲─────
    └─────────────────────────────────────
     Mar         Apr         May

By Severity:
- Critical: +2 (40% increase) ⚠️
- High: +8 (35% increase)
- Medium: +5 (8% increase)
- Low: -3 (9% decrease) ✅

New SATD Introduced:
- Week 1: 12 items (3 critical)
- Week 2: 8 items (1 critical)
- Week 3: 15 items (0 critical)
- Week 4: 6 items (1 critical)

SATD Resolved:
- Total: 18 items
- Average age when resolved: 45 days
- Oldest resolved: 234 days
```

## Integration

### GitHub Actions

```yaml
name: SATD Check
on: [pull_request]

jobs:
  satd-analysis:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
        with:
          fetch-depth: 0  # Full history for evolution
      
      - name: Install PMAT
        run: |
          curl -sSfL https://raw.githubusercontent.com/paiml/paiml-mcp-agent-toolkit/master/scripts/install.sh | sh
      
      - name: Check SATD
        id: satd
        run: |
          # Check for new critical SATD
          pmat analyze satd --critical-only --format json > satd.json
          
          new_critical=$(jq '.items | map(select(.age_days < 1)) | length' satd.json)
          
          if [ $new_critical -gt 0 ]; then
            echo "❌ New critical SATD detected!"
            echo "critical_found=true" >> $GITHUB_OUTPUT
          fi
      
      - name: Comment on PR
        if: steps.satd.outputs.critical_found == 'true'
        uses: actions/github-script@v6
        with:
          script: |
            const fs = require('fs');
            const satd = JSON.parse(fs.readFileSync('satd.json', 'utf8'));
            
            const criticalItems = satd.items
              .filter(item => item.age_days < 1)
              .map(item => `- \`${item.file}:${item.line}\`: ${item.text}`)
              .join('\n');
            
            const body = `### ⚠️ New Critical SATD Detected
            
            This PR introduces new critical technical debt:
            
            ${criticalItems}
            
            Please address these issues or create tracking tickets.`;
            
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: body
            });
```

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Check for new critical SATD
new_satd=$(pmat analyze satd --critical-only --format json | \
  jq '.items | map(select(.age_days < 0.01)) | length')

if [ $new_satd -gt 0 ]; then
    echo "❌ Commit blocked: New critical SATD detected"
    echo "Run 'pmat analyze satd --critical-only' for details"
    exit 1
fi

# Warn about increasing SATD
total_satd=$(pmat analyze satd --format json | jq '.summary.total')
baseline_satd=$(cat .satd-baseline 2>/dev/null || echo 0)

if [ $total_satd -gt $baseline_satd ]; then
    echo "⚠️  Warning: SATD increased from $baseline_satd to $total_satd"
    echo "Consider addressing technical debt before adding more"
fi
```

## Metrics and Reporting

### SATD Density

```bash
pmat analyze satd --metrics
```

**Output:**
```
📊 SATD Metrics

Density Metrics:
- SATD per KLOC: 2.8
- SATD per developer: 15.9
- Files with SATD: 23.5%

Debt Estimation:
- Total estimated hours: 234
- Critical debt hours: 45
- Average hours per SATD: 1.8

Quality Indicators:
- SATD/Complexity correlation: 0.72
- SATD in high-churn files: 65%
- Test file SATD ratio: 0.15
```

### Team Analytics

```json
{
  "by_author": {
    "john.doe": {
      "total": 34,
      "critical": 2,
      "resolved": 12,
      "resolution_rate": 0.35
    },
    "jane.smith": {
      "total": 23,
      "critical": 0,
      "resolved": 18,
      "resolution_rate": 0.78
    }
  },
  "by_team": {
    "backend": { "total": 67, "density": 3.2 },
    "frontend": { "total": 45, "density": 2.1 },
    "devops": { "total": 15, "density": 1.8 }
  }
}
```

## Best Practices

### 1. **Structured SATD Comments**

```python
# TODO(john.doe, 2024-06-01): Refactor this to use async/await
# JIRA: PROJ-1234
# Estimate: 4 hours
# Impact: Performance improvement ~20%
```

### 2. **SATD Templates**

```javascript
// TODO_TEMPLATE: [SEVERITY] ([AUTHOR], [DATE]): [DESCRIPTION]
// Ticket: [TICKET_ID]
// Blocked by: [DEPENDENCY]
// Estimate: [HOURS]
```

### 3. **Regular Reviews**

- Weekly SATD review meetings
- Quarterly debt reduction sprints
- SATD metrics in team dashboards

### 4. **Debt Budget**

- Allocate 20% of sprint capacity to debt reduction
- Priority queue for critical SATD
- Track resolution velocity

## Configuration

```toml
# satd-config.toml
[detection]
# Custom keywords to detect
keywords = ["TODO", "FIXME", "HACK", "XXX", "REFACTOR", "OPTIMIZE"]

# Additional patterns
custom_patterns = [
    "TECH[-_]?DEBT",
    "TEMPORARY",
    "WORKAROUND"
]

[classification]
# Severity mapping
critical_keywords = ["SECURITY", "DATA_LOSS", "CRITICAL", "URGENT"]
high_keywords = ["FIXME", "BUG", "BROKEN"]
medium_keywords = ["TODO", "REFACTOR", "CLEANUP"]
low_keywords = ["NOTE", "IDEA", "CONSIDER"]

[filtering]
# Exclude paths
exclude_paths = ["vendor/", "node_modules/", ".git/"]

# Include test files
include_tests = false

# Minimum comment length
min_length = 10

[reporting]
# Group by
group_by = "severity"  # severity, category, file, author

# Sort by
sort_by = "severity"   # severity, age, file

# Limit results
max_items = 100
```

## Troubleshooting

### Common Issues

**Q: Too many false positives**
A: Adjust `min_length` and add specific `exclude_paths` in configuration.

**Q: Missing SATD in some files**
A: Check file encoding and ensure files are UTF-8. Some unicode comments might be skipped.

**Q: Evolution tracking seems wrong**
A: Ensure full git history is available (`git fetch --unshallow`).

### Debug Mode

```bash
# Verbose output
RUST_LOG=debug pmat analyze satd

# Show all detected patterns
pmat analyze satd --show-patterns

# Validate detection
pmat analyze satd --validate --file src/main.rs
```

## Future Enhancements

- **AI-Powered Classification**: Automatic severity assessment
- **Resolution Suggestions**: AI-generated fixes for common SATD
- **IDE Integration**: Real-time SATD highlighting
- **Ticket Integration**: Auto-create JIRA/GitHub issues
- **Debt Forecasting**: Predict future SATD accumulation