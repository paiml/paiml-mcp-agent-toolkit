# Mutation Testing with ML Optimization

**Version**: 2.115.0
**Status**: Phase 4.2 REFACTOR Complete - Enhanced Feature Engineering
**Accuracy**: 60-70% (statistical baseline, ready for ML upgrade)

---

## Overview

PMAT's mutation testing engine combines traditional mutation testing with machine learning to intelligently prioritize mutants and detect equivalent mutants. The system uses 18 advanced features to predict which mutants are most likely to survive, helping you focus testing efforts where they matter most.

### Key Components

1. **ML-Based Survivability Predictor** - Predicts which mutants will be killed by tests
2. **Equivalent Mutant Detector** - Identifies semantically equivalent mutations
3. **Enhanced Feature Engineering** - Extracts 18 features from each mutant
4. **Statistical Baseline Model** - Current implementation (60-70% accuracy)

---

## Architecture

### Mutation Testing Pipeline

```
Source Code
    ↓
[Mutant Generation] ← 4 Operators: AOR, ROR, COR, UOR
    ↓
[Feature Extraction] ← 18 features per mutant
    ↓
[ML Prediction] ← Survivability score (0.0 - 1.0)
    ↓
[Prioritization] ← High-risk mutants first
    ↓
[Test Execution] ← Run tests on prioritized mutants
    ↓
[Results & Metrics] ← Mutation score, coverage
```

---

## Enhanced Feature Set (18 Features)

### Original Features (v1) - 10 Features

#### 1. **operator_type** (MutationOperatorType)
The type of mutation operator applied:
- `AOR` - Arithmetic Operator Replacement (+, -, *, /, %)
- `ROR` - Relational Operator Replacement (==, !=, <, >, <=, >=)
- `COR` - Conditional Operator Replacement (&&, ||)
- `UOR` - Unary Operator Replacement (!, -, ~)

#### 2. **cyclomatic_complexity** (u32)
Cyclomatic complexity at the mutation point. Higher values indicate more complex control flow with more test paths to cover.

#### 3. **cognitive_complexity** (u32)
Human-perceived complexity. Measures how difficult the code is to understand, which correlates with test difficulty.

#### 4. **source_line** (u32)
Line number where the mutation occurred. Can indicate code location patterns.

#### 5. **nesting_depth** (u32)
How deeply nested the mutation point is (if/while/for/match depth). Deeper nesting often means harder-to-kill mutants.

#### 6. **control_flow_count** (u32)
Number of control flow constructs (if/while/for/match) near the mutation point.

#### 7. **has_loops** (bool)
Whether loops exist near the mutation point. Loop-based mutants often require specific test inputs.

#### 8. **has_conditionals** (bool)
Whether conditional statements exist near the mutation. Affects test coverage requirements.

#### 9. **function_size** (u32)
Size of the containing function in lines of code. Larger functions may have less focused tests.

#### 10. **parameter_count** (u32)
Number of function parameters. More parameters = more test combinations needed.

### Enhanced Features (v2) - 8 New Features

#### 11. **has_error_handling** (bool)
Detects error handling patterns:
- `Result<T, E>` types
- `Option<T>` types
- `.unwrap()`, `.expect()`, `?` operator
- `try`/`catch` blocks

**Why it matters**: Error handling paths are often under-tested. Mutants in error paths have higher survival rates.

#### 12. **has_assertions** (bool)
Detects assertion and test code:
- `assert!()`, `debug_assert!()`
- `#[test]` attributes

**Why it matters**: Code with assertions nearby is already being validated, making mutants easier to kill.

#### 13. **token_count** (u32)
Total number of tokens (words) in the mutated code. Indicates code density.

**Why it matters**: Dense code with many tokens often has more complex logic requiring thorough testing.

#### 14. **unique_variables** (u32)
Count of unique variable identifiers (excluding keywords).

**Why it matters**: More variables = more state to track = harder to test comprehensively.

#### 15. **has_arithmetic** (bool)
Presence of arithmetic operators: `+`, `-`, `*`, `/`

**Why it matters**: Arithmetic operations require boundary testing and edge case coverage.

#### 16. **has_comparisons** (bool)
Presence of comparison operators: `==`, `!=`, `<`, `>`, `<=`, `>=`

**Why it matters**: Comparison mutants are classic targets requiring specific test values.

#### 17. **has_logical_ops** (bool)
Presence of logical operators: `&&`, `||`, `!`

**Why it matters**: Logical operators create multiple code paths requiring branch coverage.

#### 18. **mutation_depth** (u32)
Depth of the mutation in the control flow structure (same as `nesting_depth`).

**Why it matters**: Reinforces the importance of nesting depth for prediction models.

---

## Usage

### 1. ML Predictor API

```rust
use pmat::services::mutation::{SurvivabilityPredictor, MutantFeatures, Mutant};

// Create predictor
let mut predictor = SurvivabilityPredictor::new();

// Train on historical data
let training_data = vec![
    TrainingData {
        mutant: create_mutant(),
        was_killed: true,
    },
    // ... more samples
];
predictor.train(&training_data)?;

// Predict survivability
let mutant = generate_mutant(source_code);
let prediction = predictor.predict(&mutant)?;

println!("Kill probability: {:.2}%", prediction.kill_probability * 100.0);
println!("Confidence: {:.2}", prediction.confidence);
println!("Reason: {}", prediction.explanation);
```

### 2. Feature Extraction

```rust
use pmat::services::mutation::MutantFeatures;

let features = MutantFeatures::from_mutant(&mutant);

// Access individual features
println!("Operator: {:?}", features.operator_type);
println!("Complexity: {}", features.cyclomatic_complexity);
println!("Has error handling: {}", features.has_error_handling);
println!("Unique variables: {}", features.unique_variables);
```

### 3. Equivalent Mutant Detection

```rust
use pmat::services::mutation::EquivalentMutantDetector;

let mut detector = EquivalentMutantDetector::new();
detector.train(&equivalence_training_data)?;

let result = detector.detect_equivalent(&mutant, original_source)?;

if result.is_equivalent {
    println!("Equivalent mutant detected!");
    println!("Confidence: {:.2}", result.confidence);
    println!("Reason: {}", result.reason);
}
```

### 4. Prioritization

```rust
use pmat::services::mutation::MutationEngine;

let engine = MutationEngine::new(predictor, detector);

// Generate and prioritize mutants
let mutants = engine.generate_mutants(source_code)?;
let prioritized = engine.prioritize_mutants(mutants)?;

// Execute high-priority mutants first
for mutant in prioritized.iter().take(10) {
    run_tests_on_mutant(mutant)?;
}
```

---

## Equivalent Mutant Detection

The detector identifies 5 types of equivalent mutants:

### 1. Identity Operations
```rust
// Original
fn calculate(x: i32) -> i32 { x + 0 }

// Mutant (equivalent)
fn calculate(x: i32) -> i32 { x }
```

### 2. Boolean Tautologies
```rust
// Original
fn check(x: bool) -> bool { x || true }

// Mutant (equivalent)
fn check(x: bool) -> bool { true }
```

### 3. Commutative Operations
```rust
// Original
fn add(a: i32, b: i32) -> i32 { a + b }

// Mutant (equivalent)
fn add(a: i32, b: i32) -> i32 { b + a }
```

### 4. Associative Patterns
```rust
// Original
let result = (a + b) + c;

// Mutant (equivalent)
let result = a + (b + c);
```

### 5. Double Negation
```rust
// Original
let value = !!x;

// Mutant (equivalent)
let value = x;
```

---

## Model Performance

### Current Stats (v2.115.0)

| Metric | Value |
|--------|-------|
| Feature Count | 18 |
| Model Type | Statistical Baseline |
| Accuracy | 60-70% |
| Training Samples | Operator-based kill rates |
| Prediction Time | < 1ms per mutant |
| Memory Usage | Minimal (HashMap) |

### Future Improvements (Phase 4.2 REFACTOR)

Planned upgrade to gradient boosting:
- **Target Accuracy**: 85-95%
- **Model**: LightGBM or Linfa Random Forest
- **Features**: Same 18 features
- **Training**: Historical mutation test results
- **Feature Importance**: Identify most predictive features

---

## Feature Engineering Details

### Pattern Detection Algorithms

#### Unique Variables Counter
```rust
fn count_unique_variables(source: &str) -> u32 {
    let mut variables = HashSet::new();

    for token in source.split_whitespace() {
        let cleaned = token.trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
        if !cleaned.is_empty() {
            let first_char = cleaned.chars().next().unwrap();
            if (first_char.is_lowercase() || first_char == '_')
               && !is_rust_keyword(cleaned) {
                variables.insert(cleaned.to_string());
            }
        }
    }
    variables.len() as u32
}
```

#### Error Handling Detection
```rust
let has_error_handling = source.contains("Result<")
    || source.contains("Option<")
    || source.contains("unwrap")
    || source.contains("expect")
    || source.contains("?")
    || source.contains("try")
    || source.contains("catch");
```

---

## Integration with PMAT

### CLI Usage

```bash
# Run mutation testing with ML prioritization
pmat mutate --source src/ --ml-prioritize

# Generate mutation report
pmat mutate --source src/ --report mutation-report.json

# Filter equivalent mutants
pmat mutate --source src/ --filter-equivalent
```

### MCP Agent Integration

The mutation testing system is accessible via MCP tools:

```json
{
  "tool": "mutation_test",
  "parameters": {
    "source_path": "src/lib.rs",
    "enable_ml": true,
    "filter_equivalent": true,
    "max_mutants": 100
  }
}
```

---

## Testing

### Test Coverage

- **ML Predictor**: 12 tests
- **Equivalent Detector**: 13 tests
- **Integration**: 5 tests
- **Total**: 30 tests (100% passing)

### Test Categories

1. **Feature Extraction Tests**
   - Validate all 18 features extract correctly
   - Test edge cases (empty source, complex nesting)

2. **Prediction Tests**
   - Verify kill probability calculation
   - Test confidence scoring
   - Validate explanations

3. **Equivalence Detection Tests**
   - Identity operations
   - Boolean tautologies
   - Commutative swaps
   - Pattern matching

4. **Integration Tests**
   - End-to-end ML pipeline
   - Model persistence
   - Incremental learning

---

## Performance Considerations

### Scalability

- **Feature Extraction**: O(n) where n = source code length
- **Prediction**: O(1) for statistical baseline
- **Equivalence Detection**: O(m) where m = pattern count
- **Memory**: ~1KB per mutant (features + metadata)

### Optimization Tips

1. **Batch Processing**: Process mutants in batches for better cache utilization
2. **Parallel Execution**: Use Rayon for parallel feature extraction
3. **Incremental Learning**: Update model with new test results
4. **Caching**: Cache features for identical code patterns

---

## Future Roadmap

### Phase 4.2 REFACTOR (Next)
- [ ] Replace statistical baseline with gradient boosting
- [ ] Implement Linfa Random Forest or LightGBM
- [ ] Target 85-95% accuracy
- [ ] Feature importance analysis
- [ ] Cross-validation

### Phase 5 (Production Hardening)
- [ ] Distributed mutation testing
- [ ] Real-time learning from CI/CD
- [ ] Multi-language support (Python, Go, C++)
- [ ] Advanced operators (SDL, MCR)
- [ ] Visualization dashboard

---

## References

### Internal Modules

- `src/services/mutation/ml_predictor.rs` - ML predictor implementation
- `src/services/mutation/equivalent_detector.rs` - Equivalence detection
- `src/services/mutation/types.rs` - Core types and traits
- `src/services/mutation/operators.rs` - Mutation operators
- `src/services/mutation/engine.rs` - Mutation engine

### External Resources

- [Mutation Testing: A Comprehensive Survey](https://arxiv.org/abs/1805.05889)
- [Equivalent Mutant Detection](https://ieeexplore.ieee.org/document/8823896)
- [ML for Software Testing](https://dl.acm.org/doi/10.1145/3377811.3380424)

---

## Contributing

When adding new features:

1. Follow EXTREME TDD (RED → GREEN → REFACTOR)
2. Add tests for all 18 features
3. Update `MutantFeatures` struct
4. Document feature extraction logic
5. Validate against existing tests
6. Update this documentation

**Quality Gates**: All mutations must pass PMAT quality checks before merging.

---

**Last Updated**: October 4, 2025
**Maintainers**: PMAT Team
**License**: See project LICENSE file
