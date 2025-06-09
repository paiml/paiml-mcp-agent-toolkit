# Technical Debt Gradient (TDG)

## Overview

The Technical Debt Gradient (TDG) is a quantitative metric that measures the rate of technical debt accumulation in software projects. Unlike simple debt measurements, TDG captures the velocity and acceleration of debt, enabling teams to predict and prevent debt crises before they occur.

## Mathematical Foundation

### Core Formula

```
TDG = Σ(w_i × δ_i × t_i) / LOC
```

Where:
- `w_i` = Weight factor for debt type i
- `δ_i` = Debt density for type i
- `t_i` = Time decay factor
- `LOC` = Lines of code (normalized)

### Debt Types and Weights

| Debt Type | Weight | Description |
|-----------|--------|-------------|
| Complexity Debt | 0.30 | High cyclomatic/cognitive complexity |
| Design Debt | 0.25 | Poor architectural decisions |
| Test Debt | 0.20 | Missing or inadequate tests |
| Documentation Debt | 0.15 | Missing or outdated docs |
| Dependency Debt | 0.10 | Outdated or risky dependencies |

### Time Decay Function

```
t(age) = 1 + log₁₀(age_in_days + 1)
```

Older debt compounds, making it progressively more expensive to fix.

## Usage

### Command Line

```bash
# Calculate TDG for current project
pmat analyze tdg

# Detailed TDG analysis
pmat analyze tdg --detailed

# Track TDG over time
pmat analyze tdg --trend --days 90

# Compare branches
pmat analyze tdg --compare main..feature/new-ui

# Output formats
pmat analyze tdg --format json
pmat analyze tdg --format csv
```

### API Usage

```rust
use paiml_mcp_agent_toolkit::services::tdg_calculator::{
    TdgCalculator, TdgConfig, TdgResult
};

let calculator = TdgCalculator::new();
let config = TdgConfig {
    include_tests: false,
    time_decay: true,
    custom_weights: None,
};

let result = calculator.calculate(".", &config)?;
println!("TDG Score: {:.2}", result.score);
println!("Risk Level: {:?}", result.risk_level);
```

## Interpretation

### TDG Score Ranges

| Score | Risk Level | Action Required |
|-------|------------|-----------------|
| 0.0-0.5 | Minimal | Maintain current practices |
| 0.5-1.0 | Low | Monitor trends |
| 1.0-1.5 | Moderate | Plan debt reduction |
| 1.5-2.0 | High | Prioritize refactoring |
| >2.0 | Critical | Immediate action needed |

### Visualization

```
TDG Timeline (Last 90 Days)
2.5 ┤                                    ⚠️
2.0 ┤                              ╱─────
1.5 ┤                        ╱─────
1.0 ┤                  ╱─────  ✅ Refactoring
0.5 ┤            ╱─────              Sprint
0.0 ┤──────╲────
    └─────────────────────────────────────
     Mar         Apr         May
```

## Components of TDG

### 1. Complexity Debt

Measured using cyclomatic and cognitive complexity:

```rust
complexity_debt = Σ(functions where complexity > threshold) × age_factor
```

**Example:**
```rust
// High complexity debt (cyclomatic: 15)
fn process_order(order: Order) -> Result<Receipt> {
    if order.items.is_empty() {
        return Err("Empty order");
    }
    
    let mut total = 0.0;
    for item in &order.items {
        if item.quantity > 0 {
            if let Some(price) = get_price(item.id) {
                if item.discount > 0 {
                    total += price * item.quantity * (1.0 - item.discount);
                } else {
                    total += price * item.quantity;
                }
            } else {
                return Err("Invalid item");
            }
        }
    }
    // ... more nested logic ...
}
```

### 2. Design Debt

Architectural issues and anti-patterns:

```rust
design_debt = Σ(anti_patterns × severity × coupling_factor)
```

Common patterns detected:
- God objects/modules
- Circular dependencies
- Inappropriate intimacy
- Feature envy
- Shotgun surgery susceptibility

### 3. Test Debt

Missing or inadequate test coverage:

```rust
test_debt = (1 - coverage_ratio) × critical_path_weight × age
```

Factors:
- Line coverage < 80%
- Branch coverage < 70%
- Missing edge case tests
- No integration tests
- Outdated test fixtures

### 4. Documentation Debt

```rust
doc_debt = Σ(undocumented_public_apis × complexity × usage_frequency)
```

Includes:
- Missing function documentation
- Outdated README
- No architecture docs
- Missing API examples
- Incorrect documentation

### 5. Dependency Debt

```rust
dep_debt = Σ(outdated_deps × severity × transitive_factor)
```

Considers:
- Version lag (major/minor/patch)
- Security vulnerabilities
- Deprecated dependencies
- License risks
- Transitive dependency depth

## Advanced Analysis

### TDG Derivatives

**First Derivative (Velocity):**
```
TDG' = dTDG/dt
```
Indicates how fast debt is accumulating.

**Second Derivative (Acceleration):**
```
TDG'' = d²TDG/dt²
```
Shows if debt accumulation is speeding up or slowing down.

### Predictive Modeling

```bash
# Predict TDG in 30 days
pmat analyze tdg --predict 30

# Show confidence intervals
pmat analyze tdg --predict 30 --confidence 95
```

**Output:**
```json
{
  "current_tdg": 1.45,
  "predicted_tdg_30d": 1.78,
  "confidence_interval": [1.65, 1.91],
  "probability_exceeds_2": 0.23,
  "recommended_actions": [
    "Refactor high-complexity functions in src/core",
    "Add tests for uncovered critical paths",
    "Update 5 major version dependencies"
  ]
}
```

## Integration

### CI/CD Pipeline

```yaml
name: TDG Monitoring
on: [push, pull_request]

jobs:
  tdg-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
        with:
          fetch-depth: 0  # Full history for trend analysis
      
      - name: Calculate TDG
        run: |
          pmat analyze tdg --format json > tdg-report.json
          tdg_score=$(jq .score tdg-report.json)
          
          # Fail if TDG > 1.5
          if (( $(echo "$tdg_score > 1.5" | bc -l) )); then
            echo "❌ TDG Score $tdg_score exceeds threshold 1.5"
            exit 1
          fi
      
      - name: Comment on PR
        if: github.event_name == 'pull_request'
        run: |
          pmat analyze tdg --compare ${{ github.base_ref }}..${{ github.head_ref }} \
            --format markdown > tdg-diff.md
          # Post comment with TDG changes
```

### Monitoring Dashboard

```python
# tdg_monitor.py
import subprocess
import json
import matplotlib.pyplot as plt
from datetime import datetime, timedelta

def collect_tdg_history(days=90):
    """Collect TDG measurements over time"""
    history = []
    
    for i in range(days):
        date = datetime.now() - timedelta(days=i)
        # Checkout historical commit
        commit = get_commit_at_date(date)
        subprocess.run(['git', 'checkout', commit])
        
        # Calculate TDG
        result = subprocess.run(
            ['pmat', 'analyze', 'tdg', '--format', 'json'],
            capture_output=True,
            text=True
        )
        
        data = json.loads(result.stdout)
        history.append({
            'date': date,
            'tdg': data['score'],
            'components': data['components']
        })
    
    return history

def plot_tdg_trend(history):
    """Generate TDG trend chart"""
    dates = [h['date'] for h in history]
    tdg_scores = [h['tdg'] for h in history]
    
    plt.figure(figsize=(12, 6))
    plt.plot(dates, tdg_scores, 'b-', linewidth=2)
    plt.axhline(y=1.5, color='r', linestyle='--', label='Danger Zone')
    plt.fill_between(dates, 0, tdg_scores, alpha=0.3)
    
    plt.xlabel('Date')
    plt.ylabel('TDG Score')
    plt.title('Technical Debt Gradient Over Time')
    plt.legend()
    plt.grid(True, alpha=0.3)
    
    plt.savefig('tdg-trend.png', dpi=150)
```

## Remediation Strategies

### High Complexity Debt
1. **Extract Method**: Break down large functions
2. **Replace Conditional with Polymorphism**: Reduce branching
3. **Introduce Parameter Object**: Simplify signatures

### High Design Debt
1. **Facade Pattern**: Hide complex subsystems
2. **Dependency Injection**: Reduce coupling
3. **Module Extraction**: Break up god objects

### High Test Debt
1. **Test-Driven Refactoring**: Write tests before changes
2. **Mutation Testing**: Verify test quality
3. **Coverage Ratcheting**: Prevent regression

### High Documentation Debt
1. **Doc-Driven Development**: Write docs first
2. **Example-Based Documentation**: Show, don't tell
3. **Automated Doc Generation**: From code annotations

### High Dependency Debt
1. **Gradual Updates**: One major version at a time
2. **Dependency Pinning**: Lock known-good versions
3. **Regular Audits**: Weekly dependency checks

## Configuration

### Custom Weights

```toml
# tdg-config.toml
[weights]
complexity = 0.35
design = 0.30
test = 0.20
documentation = 0.10
dependencies = 0.05

[thresholds]
max_tdg = 1.5
complexity_threshold = 20
min_coverage = 80

[analysis]
include_tests = false
include_vendored = false
time_decay = true
decay_factor = 1.1
```

### Exclusions

```toml
[exclude]
paths = [
    "tests/",
    "vendor/",
    "generated/"
]

patterns = [
    "*.generated.rs",
    "*_test.go"
]
```

## Best Practices

1. **Monitor Trends**: Absolute values matter less than direction
2. **Set Ratchets**: Prevent TDG from increasing in PRs
3. **Budget Debt Work**: Allocate 20% of capacity to debt reduction
4. **Focus on High-Impact**: Address highest-weight components first
5. **Measure Progress**: Track TDG reduction sprint-over-sprint

## Case Studies

### Case 1: E-Commerce Platform
- **Initial TDG**: 2.3 (Critical)
- **Actions**: 3-month refactoring initiative
- **Final TDG**: 0.9 (Low)
- **Results**: 50% fewer bugs, 2x faster feature delivery

### Case 2: Mobile App
- **Initial TDG**: 1.7 (High)
- **Actions**: Incremental improvements each sprint
- **Final TDG**: 0.6 (Low)
- **Results**: 70% reduction in crash rate

## Future Enhancements

- **ML-Based Prediction**: More accurate trend forecasting
- **Team-Specific Weights**: Customize based on team priorities
- **Real-time Monitoring**: IDE plugin for live TDG tracking
- **Automated Remediation**: AI-suggested fixes for high TDG
- **Cross-Language Support**: Unified TDG across polyglot codebases