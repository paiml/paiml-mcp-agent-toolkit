# Refactor Engine

## Overview

The Refactor Engine is an automated code refactoring system that reduces code complexity through intelligent transformations. It supports both batch processing and interactive modes, with checkpoint support for resumable operations and integration with version control systems.

## Architecture

The refactor engine operates in three layers:

1. **Analysis Layer**: Identifies refactoring opportunities using multiple metrics
2. **Transformation Layer**: Applies safe, semantic-preserving transformations
3. **Validation Layer**: Ensures correctness through testing and verification

## Command Reference

### Interactive Mode

Interactive refactoring provides step-by-step guidance with explanations:

```bash
# Start interactive session
pmat refactor interactive

# With custom complexity target
pmat refactor interactive --target-complexity 15

# Limited steps for incremental refactoring
pmat refactor interactive --steps 5 --checkpoint my-session.json

# Minimal explanations for experienced users
pmat refactor interactive --explain minimal
```

### Batch Mode (Server)

Batch processing for large-scale refactoring:

```bash
# Basic batch refactoring
pmat refactor serve --config refactor-config.json

# High-performance configuration
pmat refactor serve \
  --parallel 8 \
  --memory-limit 2048 \
  --batch-size 20

# Resume from checkpoint with auto-commit
pmat refactor serve \
  --resume \
  --checkpoint-dir ./checkpoints \
  --auto-commit "refactor: reduce complexity in {file}"

# Priority-based refactoring
pmat refactor serve \
  --priority "complexity * defect_probability" \
  --max-runtime 3600
```

### Status and Resume

Monitor and control refactoring progress:

```bash
# Check current status
pmat refactor status

# Detailed status from specific checkpoint
pmat refactor status \
  --checkpoint ./checkpoints/session-2024-01-15.json \
  --format summary

# Resume from checkpoint
pmat refactor resume --steps 10

# Resume with different explanation level
pmat refactor resume --explain detailed --checkpoint backup.json
```

## Configuration Format

### Batch Configuration (JSON)

```json
{
  "version": "1.0",
  "refactor_rules": {
    "max_complexity": 20,
    "max_function_length": 50,
    "max_parameter_count": 5,
    "max_nesting_depth": 4
  },
  "transformations": {
    "extract_method": {
      "enabled": true,
      "min_lines": 5,
      "max_lines": 20
    },
    "inline_variable": {
      "enabled": true,
      "single_use_only": true
    },
    "simplify_conditional": {
      "enabled": true,
      "de_morgan": true,
      "early_return": true
    },
    "loop_transformation": {
      "enabled": true,
      "iterator_conversion": true,
      "loop_fusion": false
    }
  },
  "safety": {
    "test_command": "cargo test",
    "require_tests_pass": true,
    "backup_enabled": true,
    "dry_run": false
  },
  "filters": {
    "include_patterns": ["src/**/*.rs"],
    "exclude_patterns": ["**/tests/**", "**/vendor/**"],
    "min_complexity": 10
  }
}
```

## Refactoring Strategies

### 1. Extract Method

Identifies and extracts cohesive code blocks into separate functions:

```rust
// Before
fn process_data(items: Vec<Item>) -> Result<Summary> {
    let mut total = 0;
    let mut count = 0;
    
    // Complex validation logic (15 lines)
    for item in &items {
        if item.value > 0 && item.value < 1000 {
            if item.category == Category::A || item.category == Category::B {
                if item.timestamp > cutoff_date {
                    total += item.value;
                    count += 1;
                }
            }
        }
    }
    
    // More processing...
}

// After
fn process_data(items: Vec<Item>) -> Result<Summary> {
    let (total, count) = calculate_totals(&items);
    // More processing...
}

fn calculate_totals(items: &[Item]) -> (i32, i32) {
    items.iter()
        .filter(|item| is_valid_item(item))
        .fold((0, 0), |(total, count), item| {
            (total + item.value, count + 1)
        })
}

fn is_valid_item(item: &Item) -> bool {
    item.value > 0 
        && item.value < 1000 
        && matches!(item.category, Category::A | Category::B)
        && item.timestamp > cutoff_date
}
```

### 2. Guard Clause Introduction

Reduces nesting by introducing early returns:

```rust
// Before
fn validate_user(user: &User) -> Result<()> {
    if user.is_active() {
        if user.has_permission("write") {
            if user.quota_remaining() > 0 {
                // Main logic
                Ok(())
            } else {
                Err("Quota exceeded")
            }
        } else {
            Err("Insufficient permissions")
        }
    } else {
        Err("User inactive")
    }
}

// After
fn validate_user(user: &User) -> Result<()> {
    if !user.is_active() {
        return Err("User inactive");
    }
    
    if !user.has_permission("write") {
        return Err("Insufficient permissions");
    }
    
    if user.quota_remaining() == 0 {
        return Err("Quota exceeded");
    }
    
    // Main logic
    Ok(())
}
```

### 3. Loop Simplification

Converts complex loops to functional style:

```python
# Before
def process_items(items):
    result = []
    for item in items:
        if item.is_valid():
            transformed = transform(item)
            if transformed.score > threshold:
                result.append(transformed)
    return result

# After
def process_items(items):
    return [
        transformed
        for item in items
        if item.is_valid()
        for transformed in [transform(item)]
        if transformed.score > threshold
    ]
```

### 4. Conditional Simplification

Applies logical simplifications:

```typescript
// Before
if (!(a && b) || !(c || d)) {
    return false;
}

// After (De Morgan's law)
if (!a || !b || (!c && !d)) {
    return false;
}
```

## Interactive Mode Features

### Real-time Feedback

```
🔍 Analyzing function: calculate_metrics (complexity: 35)

Suggested refactoring: Extract Method
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Current complexity: 35 (cognitive) / 28 (cyclomatic)
Target complexity: 20

The nested loops starting at line 45 can be extracted into a separate function.
This will:
- Reduce complexity by ~12 points
- Improve readability
- Enable easier testing

Preview:
  [Shows diff of proposed changes]

Apply this refactoring? (y/n/skip/explain/quit): 
```

### Explanation Levels

- **Minimal**: Just the facts - what changes and complexity reduction
- **Normal**: Includes rationale and expected benefits
- **Detailed**: Full explanation with examples and best practices

### Undo Support

All changes are tracked and can be undone:

```bash
# In interactive mode
> undo  # Reverts last change
> undo 3  # Reverts last 3 changes
> show history  # Shows all applied refactorings
```

## Checkpoint System

### Checkpoint Structure

```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2024-01-15T10:30:00Z",
  "progress": {
    "files_processed": 45,
    "files_total": 120,
    "refactorings_applied": 23,
    "complexity_reduced": 450
  },
  "current_file": "src/services/analyzer.rs",
  "pending_files": ["src/services/cache.rs", "..."],
  "applied_refactorings": [
    {
      "file": "src/main.rs",
      "type": "extract_method",
      "before_complexity": 35,
      "after_complexity": 18,
      "timestamp": "2024-01-15T10:15:00Z"
    }
  ]
}
```

### Checkpoint Management

```bash
# List available checkpoints
ls checkpoints/

# Backup checkpoint
cp refactor_state.json checkpoints/backup-$(date +%Y%m%d).json

# Merge checkpoints (for parallel runs)
pmat refactor merge-checkpoints checkpoint1.json checkpoint2.json
```

## Integration

### Version Control Integration

```bash
# Auto-commit after each file
pmat refactor serve \
  --auto-commit "refactor: reduce {function} complexity from {before} to {after}"

# Create separate branch
git checkout -b refactor/reduce-complexity
pmat refactor serve --config high-complexity.json
```

### CI/CD Integration

```yaml
# GitHub Actions workflow
name: Weekly Refactoring

on:
  schedule:
    - cron: '0 2 * * 0'  # Sunday 2 AM

jobs:
  refactor:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Run Refactoring
        run: |
          pmat refactor serve \
            --config .github/refactor-config.json \
            --max-runtime 3600 \
            --checkpoint-dir ./refactor-checkpoints
      
      - name: Create Pull Request
        uses: peter-evans/create-pull-request@v5
        with:
          title: "Automated Complexity Reduction"
          body: "This PR contains automated refactorings to reduce code complexity."
          branch: auto-refactor/reduce-complexity
```

### IDE Integration

The refactor engine can be integrated with IDEs through LSP:

```json
// VSCode settings.json
{
  "paiml.refactor.autoSuggest": true,
  "paiml.refactor.complexityThreshold": 20,
  "paiml.refactor.showPreview": true
}
```

## Safety Features

### Test Validation

Before applying any refactoring:

1. Runs existing tests
2. Applies transformation
3. Runs tests again
4. Reverts if tests fail

### Semantic Preservation

The engine ensures:

- No behavior changes
- Preserved API contracts
- Maintained type signatures
- Consistent error handling

### Backup System

```bash
# Automatic backups before each refactoring
checkpoints/
├── session-abc123/
│   ├── backup-001-main.rs
│   ├── backup-002-analyzer.rs
│   └── manifest.json
```

## Performance Optimization

### Parallel Processing

```bash
# Utilize all CPU cores
pmat refactor serve --parallel $(nproc)

# Memory-constrained environments
pmat refactor serve --parallel 4 --memory-limit 512
```

### Prioritization Strategies

```bash
# Focus on highest complexity first
--priority "complexity"

# Balance complexity and defect probability
--priority "complexity * defect_probability"

# Focus on frequently changed files
--priority "complexity * churn_rate"

# Custom expression
--priority "complexity^2 / (last_modified_days + 1)"
```

## Metrics and Reporting

### Progress Metrics

```
Refactoring Progress Report
═══════════════════════════════════════════════════════════════

Files processed:        45/120 (37.5%)
Refactorings applied:   23
Total complexity reduced: 450 points

Top improvements:
  src/analyzer.rs:     35 → 18 (-17 points)
  src/processor.rs:    42 → 20 (-22 points)
  src/validator.rs:    28 → 15 (-13 points)

Estimated time remaining: 25 minutes
Current file: src/services/cache.rs
```

### Final Report

```json
{
  "summary": {
    "total_files": 120,
    "files_refactored": 67,
    "total_refactorings": 134,
    "complexity_reduction": {
      "total": 892,
      "average_per_file": 13.3
    },
    "time_taken": "1h 23m",
    "test_results": {
      "before": {"passed": 450, "failed": 0},
      "after": {"passed": 450, "failed": 0}
    }
  },
  "detailed_changes": ["..."]
}
```

## Best Practices

### 1. Start Small

Begin with high-value targets:

```bash
# Find top complexity files
pmat analyze complexity --top-files 10

# Refactor worst offenders first
pmat refactor interactive --include "src/worst_file.rs"
```

### 2. Incremental Approach

```bash
# Daily complexity reduction
pmat refactor interactive --steps 5 --target-complexity 25

# Weekly batch runs
pmat refactor serve --max-runtime 3600 --auto-commit
```

### 3. Team Integration

- Review automated PRs carefully
- Maintain refactoring configuration in version control
- Document exemptions for necessarily complex code
- Track complexity trends over time

### 4. Continuous Improvement

```bash
# Track progress
pmat analyze complexity --format json > metrics/complexity-$(date +%Y%m%d).json

# Generate trend report
pmat excellence-tracker --metrics complexity
```

## Troubleshooting

### Common Issues

1. **Tests failing after refactoring**
   - Check test assumptions about internal structure
   - Verify mock configurations
   - Review integration test dependencies

2. **Checkpoint corruption**
   - Use backup checkpoints
   - Validate JSON structure
   - Start fresh if necessary

3. **Performance issues**
   - Reduce batch size
   - Limit parallelism
   - Exclude large files temporarily

### Debug Mode

```bash
# Verbose logging
RUST_LOG=debug pmat refactor serve --config debug.json

# Dry run mode
pmat refactor serve --dry-run --config test.json
```

## Related Features

- `pmat analyze complexity` - Identify refactoring targets
- `pmat analyze big-o` - Find algorithmic complexity issues
- `pmat analyze deep-context` - Comprehensive analysis
- `pmat excellence-tracker` - Track improvement over time