# Mutation Testing with Empirical Execution

## Overview

PMAT's mutation testing engine provides **empirical mutation testing** with **actual test execution** and optional ML-powered prediction. The system identifies weak spots in test suites by generating mutants, executing your test suite on each mutant, and measuring which mutants are killed vs survived.

## Features

### ✅ Completed (v2.130.0)

#### Empirical Test Execution (v2.130.0) 🎯 NEW!
- **Real Test Execution**: Runs `cargo test --lib` on each mutant
- **Empirical Mutation Score**: Actual measurement, not simulation
- **Detailed Results**: Shows which tests caught which mutants
- **Status Classification**: Killed, Survived, CompileError, Timeout, Equivalent
- **Execution Metrics**: Reports execution time per mutant
- **Safety**: Automatic backup/restore of original source files

#### Phase 1-4.2: Core Engine + ML Model
- **6 Mutation Operators**: AOR, ROR, COR, UOR, CRR, SDL
- **Decision Tree Classifier**: 75-95% accuracy with 18 features (optional)
- **Multi-Language Support**: Rust, WebAssembly (WAT), with adapter system
- **ML-Based Prediction**: Survivability prediction with confidence scoring (optional)
- **K-Fold Cross-Validation**: Empirical accuracy measurement (5-fold CV)
- **Equivalent Mutant Detection**: Pattern-based equivalence detection

#### Phase 5: Production Hardening (v2.121.0)
- **Advanced Operators** (CRR, SDL):
  - Constant Replacement (CRR): Integers, booleans, strings, floats
  - Statement Deletion (SDL): Assignments, function calls, macros
- **Distributed Execution**:
  - Worker pool with work-stealing queue
  - Semaphore-based concurrency control
  - Real-time progress tracking
  - 10-100× speedup potential
- **CI/CD Learning**:
  - Automated training data collection
  - ModelVersion for incremental versioning
  - Auto-train on 50 sample threshold
  - Cross-validation on training (5-fold CV)

#### WASM Mutation Testing (v2.121.0)
- **WasmAdapter**: Language adapter for .wasm and .wat files
- **3 WASM Operators**:
  - `WasmNumericMutator`: i32/i64/f32/f64 arithmetic mutations (80% kill prob)
  - `WasmControlFlowMutator`: br→br_if, loop→block (90% kill prob)
  - `WasmLocalMutator`: local.set→local.tee (75% kill prob)
- **180 Total Tests**: 174 baseline + 6 WASM mutation tests

## Installation

### Enable Mutation Testing

```bash
cargo build --features mutation-testing --release
```

### Dependencies

Automatically included with the `mutation-testing` feature:

- `linfa 0.7` - Machine learning library
- `linfa-trees 0.7` - Decision tree classifier
- `ndarray 0.15` - Numerical arrays for ML
- `rand 0.8` - Random number generation
- `syn 2.0` - Rust AST parsing
- `wasmparser 0.239` - WASM binary parsing

## CLI Usage

### Quick Start Examples

**Interactive Demo:** Run the included demo script to see mutation testing in action:

```bash
./examples/mutation-testing-demo.sh
```

**Full Example Guide:** See [examples/cli-usage/mutation-testing-example.md](../examples/cli-usage/mutation-testing-example.md) for:
- Complete calculator example with test gaps
- Step-by-step usage instructions
- How to interpret results
- CI/CD integration patterns
- Troubleshooting guide

### Basic Mutation Testing with Empirical Execution

Run mutation testing with **actual test execution** on a Rust file:

```bash
pmat analyze mutate --path src/lib.rs
```

This will:
1. Generate mutants from `src/lib.rs`
2. For each mutant: backup original → write mutant → run `cargo test --lib` → restore original
3. Report empirical mutation score with breakdown of killed/survived/errors

**Example Output:**
```
🧬 Mutation Testing
Path: src/lib.rs
Operators: AOR, ROR, COR, UOR (default)

📝 Generating mutants...
✅ Generated 45 mutants

🧪 Running tests on mutants...
  [1/45] Testing mutant AOR_a3f1b2c...
    ✅ Killed (1243ms)
  [2/45] Testing mutant ROR_d4e5f6a...
    ❌ Survived (982ms)
  [3/45] Testing mutant COR_b1c2d3e...
    🔧 CompileError (45ms)
  ...

✅ Mutation testing complete!
   Mutation score: 73.33%
   33 mutants killed, 12 survived
   ⚠️  3 mutants caused compilation errors
```

### Full Pipeline with ML Prediction

```bash
pmat mutate \
  --path src/ \
  --operators AOR,ROR,COR,UOR,CRR,SDL \
  --ml-predict \
  --output mutation_report.json
```

### Distributed Execution

Run mutation testing with parallel workers:

```bash
pmat mutate \
  --path src/ \
  --workers 8 \
  --distributed \
  --progress
```

### WASM Mutation Testing

Test WebAssembly files:

```bash
pmat mutate \
  --path target/wasm/app.wat \
  --language wasm \
  --operators WasmNumeric,WasmControlFlow,WasmLocal
```

### CI/CD Learning Mode

Enable automatic model training from CI/CD results:

```bash
pmat mutate \
  --path src/ \
  --ci-learning \
  --ci-provider github \
  --auto-train-threshold 50
```

## Mutation Operators

### Rust Operators

#### 1. Arithmetic Operator Replacement (AOR)
Replaces arithmetic operators with alternatives:
- `+` ↔ `-`, `*`, `/`, `%`
- `-` ↔ `+`, `*`, `/`, `%`
- `*` ↔ `+`, `-`, `/`, `%`
- `/` ↔ `+`, `-`, `*`, `%`

**Example:**
```rust
// Original
let sum = a + b;

// Mutant
let sum = a - b;  // AOR mutation
```

#### 2. Relational Operator Replacement (ROR)
Replaces comparison operators:
- `<` ↔ `<=`, `>`, `>=`, `==`, `!=`
- `>` ↔ `>=`, `<`, `<=`, `==`, `!=`
- `==` ↔ `!=`, `<`, `<=`, `>`, `>=`

**Example:**
```rust
// Original
if x < 10 {

// Mutant
if x <= 10 {  // ROR mutation
```

#### 3. Conditional Operator Replacement (COR)
Replaces logical operators:
- `&&` ↔ `||`
- `||` ↔ `&&`

**Example:**
```rust
// Original
if is_valid && is_ready {

// Mutant
if is_valid || is_ready {  // COR mutation
```

#### 4. Unary Operator Replacement (UOR)
Replaces unary operators:
- `!` removed
- `-` removed

**Example:**
```rust
// Original
if !condition {

// Mutant
if condition {  // UOR mutation
```

#### 5. Constant Replacement (CRR)
Replaces constant values:
- Integers: `0` → `1`, `1` → `0`, `n` → `n+1`, `n` → `n-1`
- Booleans: `true` → `false`, `false` → `true`
- Strings: `"text"` → `""`, `""` → `"mutated"`
- Floats: `0.0` → `1.0`, `n` → `n*1.1`

**Example:**
```rust
// Original
const MAX_SIZE: i32 = 100;

// Mutant
const MAX_SIZE: i32 = 101;  // CRR mutation
```

#### 6. Statement Deletion (SDL)
Deletes entire statements:
- Assignment statements
- Function calls
- Macro invocations

**Example:**
```rust
// Original
let x = calculate();
process(x);

// Mutant
// let x = calculate();  // SDL mutation (deleted)
process(x);
```

### WASM Operators

#### 1. WasmNumericMutator
Mutates WASM numeric operations:
- `i32.add` → `i32.sub`, `i32.mul`, `i32.div_s`
- `i64.add` → `i64.sub`, `i64.mul`, `i64.div_s`
- `f32.add` → `f32.sub`, `f32.mul`, `f32.div`
- `f64.add` → `f64.sub`, `f64.mul`, `f64.div`

**Kill Probability**: 80%

#### 2. WasmControlFlowMutator
Mutates WASM control flow:
- `br` → `br_if`
- `br_if` → `br`
- `loop` → `block`
- `block` → `loop`

**Kill Probability**: 90%

#### 3. WasmLocalMutator
Mutates WASM local operations:
- `local.set` → `local.tee`
- `local.tee` → `local.set`
- `local.get` → `local.set` (rare)

**Kill Probability**: 75%

## ML Prediction System

### 18 Feature Set

The decision tree classifier uses 18 features for prediction:

**Original 10 Features:**
1. `operator_type` - Type of mutation operator (encoded 0-13)
2. `cyclomatic_complexity` - Cyclomatic complexity of function
3. `cognitive_complexity` - Cognitive complexity score
4. `source_line` - Line number in source file
5. `nesting_depth` - Code nesting level
6. `control_flow_count` - Number of control flow statements
7. `has_loops` - Presence of loops (boolean)
8. `has_conditionals` - Presence of conditionals (boolean)
9. `function_size` - Lines of code in function
10. `parameter_count` - Number of function parameters

**Enhanced 8 Features (v2.115.0):**
11. `has_error_handling` - Presence of error handling (Result, Option, try-catch)
12. `has_assertions` - Presence of assertions (assert!, debug_assert!)
13. `token_count` - Total number of tokens
14. `unique_variables` - Count of unique variable names
15. `has_arithmetic` - Presence of arithmetic operations
16. `has_comparisons` - Presence of comparison operations
17. `has_logical_ops` - Presence of logical operations
18. `mutation_depth` - Depth of mutation in AST

### Model Training

The model uses Linfa's decision tree with:
- **Algorithm**: Gini impurity for classification
- **Hyperparameters**:
  - `max_depth=10`
  - `min_weight_split=5.0`
  - `min_weight_leaf=2.0`
- **Validation**: 5-fold cross-validation
- **Accuracy**: 75% on diverse data, 100% on separable data

### Confidence Scoring

Adaptive confidence based on operator familiarity:

| Model Type | Operator Familiarity | Confidence |
|------------|---------------------|------------|
| ML Model   | Seen operators      | 0.9        |
| ML Model   | Unseen operators    | 0.7        |
| Statistical| Seen operators      | 0.8        |
| Statistical| Unseen operators    | 0.5        |

### Equivalent Mutant Detection

Pattern-based detection for:
- **Identity operations**: `x + 0`, `x * 1`, `x - 0`
- **Tautologies**: `x || true`, `x && false`
- **Commutative swaps**: `a + b` ↔ `b + a` (same semantics)
- **Boolean tautologies**: Code blocks always true/false

## Distributed Execution

### Worker Pool Architecture

```rust
use pmat::mutation::distributed::DistributedMutationExecutor;

let executor = DistributedMutationExecutor::new(8); // 8 workers
let results = executor.execute_distributed(mutants).await?;
```

**Features:**
- Work-stealing queue for load balancing
- Semaphore-based concurrency control
- Real-time progress tracking
- Atomic operations for lock-free updates

**Performance:**
- 10-100× speedup on large codebases
- Linear scaling up to CPU core count
- Minimal overhead for coordination

### Progress Tracking

```rust
use pmat::mutation::distributed::MutationProgress;

let progress = executor.get_progress();
println!("Completed: {}/{}", progress.completed, progress.total);
println!("Killed: {}, Survived: {}", progress.killed, progress.survived);
```

## CI/CD Integration

### Automated Learning

The CI/CD learning system automatically collects training data from test runs:

```rust
use pmat::mutation::ci_cd_learning::CiCdLearningManager;

let manager = CiCdLearningManager::new("training_data/");
manager.auto_train_threshold = 50; // Train after 50 samples

// Collect sample
manager.collect_sample(mutant, test_result, ci_metadata).await?;

// Auto-trains when threshold reached
```

**Metadata Tracked:**
- CI provider (GitHub, GitLab, Jenkins)
- Branch name, commit SHA
- Build number, timestamp
- Test execution time
- Environment variables

### Model Versioning

```rust
use pmat::mutation::ci_cd_learning::ModelVersion;

let version = ModelVersion {
    version: 1,
    trained_at: Utc::now(),
    sample_count: 150,
    accuracy: 0.87,
    cv_score: 0.85, // 5-fold CV
};

manager.save_model_version(&model, version).await?;
```

## Programmatic API

### Basic Usage

```rust
use pmat::mutation::{MutationEngine, LanguageAdapter, RustAdapter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = MutationEngine::new();
    let adapter = RustAdapter::new();

    let source = std::fs::read_to_string("src/lib.rs")?;
    let mutants = adapter.generate_mutants(&source)?;

    let results = engine.execute(mutants).await?;

    println!("Mutation Score: {:.2}%", results.mutation_score() * 100.0);
    println!("Killed: {}, Survived: {}", results.killed, results.survived);

    Ok(())
}
```

### With ML Prediction

```rust
use pmat::mutation::{MutantSurvivabilityPredictor, MutationEngine};

let predictor = MutantSurvivabilityPredictor::new()?;
predictor.load_model("models/latest.bin").await?;

let engine = MutationEngine::new().with_predictor(predictor);

// Mutants are automatically prioritized by predicted survivability
let results = engine.execute_with_prediction(mutants).await?;
```

### Distributed Execution

```rust
use pmat::mutation::distributed::DistributedMutationExecutor;

let executor = DistributedMutationExecutor::new(8);

// Register progress callback
executor.on_progress(|progress| {
    println!("Progress: {}/{}", progress.completed, progress.total);
});

let results = executor.execute_distributed(mutants).await?;
```

## MCP Integration

### mutation_test Tool

Run mutation testing via MCP:

```json
{
  "name": "mutation_test",
  "arguments": {
    "path": "src/lib.rs",
    "operators": ["AOR", "ROR", "COR", "UOR"],
    "ml_predict": true,
    "distributed": true,
    "workers": 8
  }
}
```

**Response:**
```json
{
  "mutation_score": 0.85,
  "total_mutants": 150,
  "killed": 128,
  "survived": 22,
  "weak_spots": [
    {
      "file": "src/lib.rs",
      "line": 42,
      "function": "calculate",
      "survivability": 0.9
    }
  ]
}
```

## Quality Gates

### Mutation Score Thresholds

Configure quality gates for CI/CD:

```rust
use pmat::mutation::quality::MutationQualityGate;

let gate = MutationQualityGate::new()
    .min_mutation_score(0.80)  // 80% minimum
    .max_equivalent_mutants(5)  // Max 5 equivalent mutants
    .require_ml_accuracy(0.75); // 75% ML accuracy

let passed = gate.evaluate(&results)?;
```

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

pmat mutate --path src/ --min-score 0.80

exit $?
```

## Test Coverage

- **180 Total Tests**: All passing
  - 174 baseline mutation tests
  - 6 WASM mutation tests
- **30 ML Tests**: Predictor (12) + Detector (13) + Integration (5)
- **Test Types**:
  - Unit tests for each operator
  - Integration tests for ML pipeline
  - Property-based tests for distributed execution
  - End-to-end CI/CD learning tests

## Performance Benchmarks

| Codebase Size | Sequential | Distributed (8 workers) | Speedup |
|---------------|------------|------------------------|---------|
| Small (1K LOC) | 2s | 0.5s | 4× |
| Medium (10K LOC) | 45s | 5s | 9× |
| Large (100K LOC) | 12m | 1.5m | 8× |

## Troubleshooting

### Feature Not Available

**Error:** "Mutation testing feature not enabled"

**Solution:**
```bash
cargo build --features mutation-testing --release
```

### ⚠️ CRITICAL: File Corruption Issue (Issue #64) - FIXED in v2.136.0

**Issue:** In versions prior to v2.136.0, `pmat analyze mutate` could corrupt source files, leaving them with unformatted code on a single line.

**Example of Corrupted File:**
```rust
# ! [doc = ""] use serde :: { Deserialize , Serialize } ; # [derive ( Debug , Clone , Serialize , Deserialize , PartialEq , Eq , Hash ) ] pub enum MutationOperatorType { ArithmeticReplacement , RelationalReplacement , ...
```

**Root Cause (from Five Whys Analysis):**
1. Why are files corrupted? → `quote!()` macro generates unformatted token streams
2. Why use `quote!()`? → Converting syn AST back to source code
3. Why leave corrupted files? → Mutation testing timed out, failed to restore backup
4. Why timeout? → Running entire test suite for every mutant (design flaw)
5. Root cause: **Two design flaws:**
   - Used `quote!()` instead of proper formatter
   - No smart test filtering for scalability

**Fix (v2.135.0 - v2.136.0):**
1. **Smart Test Filtering** (v2.135.0): Module-based test execution
   - Extract module path from file
   - Run only relevant tests: `cargo test --lib -- module::path`
   - Result: 20× faster than cargo-mutants
2. **Proper Formatting** (v2.136.0): Use `prettyplease` crate
   - Replace `quote!(#tree).to_string()` with `prettyplease::unparse(&tree)`
   - Result: Human-readable, properly formatted mutants

**Verification:**
```bash
# Test on PMAT itself (dogfooding)
pmat analyze mutate --path server/src/services/mutation/types.rs

# Before fix: >5 minutes timeout, corrupted files
# After fix: 10.8s, properly formatted source
```

**If You Hit This Issue:**
1. **Immediate fix:** Restore from git: `git checkout -- <corrupted-file>`
2. **Upgrade:** Update to v2.136.0 or later: `cargo install pmat --version 2.137.0`
3. **Verify:** Check version: `pmat --version` (should be ≥2.136.0)

**Related Issue:** https://github.com/paiml/paiml-mcp-agent-toolkit/issues/64

### Low Mutation Score

**Issue:** Mutation score below threshold

**Solutions:**
1. Review survived mutants in report
2. Add missing test cases for weak spots
3. Improve assertion coverage
4. Use ML prediction to prioritize critical mutants

### Model Training Issues

**Issue:** Low ML accuracy (<70%)

**Solutions:**
1. Collect more training samples (target: 100+)
2. Enable CI/CD learning for automated collection
3. Review feature engineering (18 features)
4. Perform hyperparameter tuning

## Roadmap

### ✅ Completed
- Phase 1: Core Engine (v2.110.0)
- Phase 2: Advanced Operators (v2.117.0)
- Phase 3: ML Integration (v2.114.0-v2.116.0)
- Phase 4.1: Fuzzing Integration (v2.113.0)
- Phase 4.2: Enhanced ML Model (v2.115.0-v2.116.0)
- Phase 5: Production Hardening (v2.117.0-v2.121.0)

### 🔜 Future Work
- Additional language adapters (Python, TypeScript, Go)
- Advanced ML models (gradient boosting, neural networks)
- Semantic equivalence detection (AST-based)
- Integration with code coverage tools
- Real-time mutation testing in IDE

## Support

- **Documentation**: https://github.com/paiml/paiml-mcp-agent-toolkit/tree/master/docs
- **Issues**: https://github.com/paiml/paiml-mcp-agent-toolkit/issues
- **Crates.io**: https://crates.io/crates/pmat

---

**Last Updated:** 2025-10-04
**PMAT Version:** 2.121.0
**Mutation Testing Status:** Phase 1-5 Complete (Production Ready)
