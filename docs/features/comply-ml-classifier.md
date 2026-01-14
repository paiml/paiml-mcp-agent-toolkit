# Commit Message ML Classifier

**Command**: `pmat comply check`
**Status**: Production (v2.213.8+)

## Overview

ML-based classification of commit messages for Sovereign AI Stack projects. Uses Naive Bayes trained on commits from trueno, aprender, and realizar to identify development patterns.

## Classification Categories

| Category | Description | Example Commits |
|----------|-------------|-----------------|
| `ASTTransform` | Code transformations, refactoring | "refactor: extract method", "perf: optimize hot path" |
| `TraitBounds` | Type system, generics, bounds | "feat: add generic impl", "fix: trait bound issue" |
| `OwnershipBorrow` | Memory management, lifetimes | "fix: lifetime elision", "refactor: remove clone" |
| `ConcurrencyBugs` | Threading, async, race conditions | "fix: data race in worker", "feat: add mutex" |
| `ConfigurationErrors` | Config, build, CI issues | "fix: cargo.toml syntax", "ci: update workflow" |
| `MemorySafety` | Unsafe code, bounds checking | "fix: buffer overflow", "refactor: remove unsafe" |
| `StdlibMapping` | Standard library usage | "feat: use std::collections", "refactor: replace vec" |
| `IntegrationFailures` | Integration, API issues | "fix: API compatibility", "feat: add adapter" |

## Output

When running `pmat comply check` on a Sovereign Stack project:

```
✓ Sovereign Stack Patterns: ..., ML: ASTTransform dominant (6/10), ML: 90% high-confidence
```

- **Dominant pattern**: Most frequent classification in recent commits
- **High-confidence**: Percentage of commits with >60% classification confidence

## Model Details

| Property | Value |
|----------|-------|
| Algorithm | Multinomial Naive Bayes |
| Features | Bag-of-words (500 vocab) |
| Training samples | 65 |
| Test accuracy | 50% |
| Model size | 112 KB |

## Files

```
models/
├── training-data.json              # Labeled examples (65 commits)
├── train_classifier.py             # Training script
└── sovereign-stack-classifier.json # Exported model
```

## Retraining

To improve accuracy with more data:

```bash
# 1. Add examples to training-data.json
# Format: {"train": [{"message": "...", "label": "ASTTransform", ...}]}

# 2. Run training
cd models && python3 train_classifier.py

# Output:
# Training samples: N
# Test accuracy: X%
# Model exported to: sovereign-stack-classifier.json
```

## Integration

The classifier is used in `check_sovereign_stack_patterns()`:

```rust
use crate::services::commit_classifier::CommitClassifier;

if let Ok(classifier) = CommitClassifier::load_sovereign_stack() {
    let result = classifier.classify("fix: memory leak in parser");
    println!("Class: {}, Confidence: {:.0}%", result.class, result.confidence * 100.0);
}
```

## Limitations

- **Small dataset**: 65 examples limits accuracy (50%)
- **Domain-specific**: Trained only on Sovereign Stack commits
- **No online learning**: Must retrain to incorporate new patterns

## Future Improvements

1. **More training data**: Target 500+ examples for 80%+ accuracy
2. **Active learning**: Flag low-confidence commits for labeling
3. **Cross-project transfer**: Share training data across PAIML repos
