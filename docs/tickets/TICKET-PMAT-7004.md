# TICKET-PMAT-7004: Mutation Testing ML Upgrade to Production Grade

**Status**: 🔨 TODO
**Priority**: High
**Complexity**: Medium (3-5 days)
**Sprint**: Sprint 23
**Created**: 2025-10-07
**Dependencies**: Phase 4.2 ML Model ✅ Complete (v2.116.0)

## Objective

Upgrade mutation testing from statistical baseline (60-70% accuracy) to production-grade machine learning with gradient boosting (85-95% accuracy), using the 18 enhanced features from Phase 4.2.

## Current State (✅ Complete)

### Phase 4.2 Enhanced Features (v2.115.0-v2.116.0)
- ✅ **18 enhanced features** for mutant survivability prediction
- ✅ Statistical baseline model (operator-based kill rates)
- ✅ Equivalent mutant detector with pattern-based detection
- ✅ Model persistence and incremental learning
- ✅ 30 ML tests passing
- ✅ Comprehensive documentation

### Accuracy: 60-70% (statistical baseline)

## Requirements

### 1. LightGBM/Linfa Integration (2-3 days)
- [ ] Evaluate Linfa (pure Rust) vs LightGBM (binding)
- [ ] Add dependency to Cargo.toml
- [ ] Implement gradient boosting model trainer
- [ ] Train on 18 enhanced features:
  - operator_type, cyclomatic_complexity, cognitive_complexity
  - source_line, nesting_depth, control_flow_count
  - has_loops, has_conditionals, function_size, parameter_count
  - has_error_handling, has_assertions, token_count
  - unique_variables, has_arithmetic, has_comparisons
  - has_logical_ops, mutation_depth
- [ ] Cross-validation with k-fold (k=5)
- [ ] Hyperparameter tuning (learning rate, max depth, num trees)
- [ ] Model evaluation metrics (precision, recall, F1, AUC-ROC)

### 2. Advanced Equivalence Detection (1-2 days)
- [ ] AST-based semantic equivalence analysis
- [ ] Dynamic execution pattern comparison
- [ ] Feature importance analysis from trained model
- [ ] Enhanced tautology detection
- [ ] Commutative operation detection improvements

### 3. Integration & Testing (1 day)
- [ ] Replace statistical model with trained GB model
- [ ] Update prediction pipeline
- [ ] Benchmark accuracy improvements
- [ ] Performance profiling (inference time)
- [ ] Update documentation

## Implementation Plan

### Files to Create/Modify
- `server/src/services/mutation/ml/gradient_boosting.rs` (new)
- `server/src/services/mutation/ml/model_trainer.rs` (extend)
- `server/src/services/mutation/ml/feature_extractor.rs` (extend for AST)
- `server/src/services/mutation/ml/predictor.rs` (update)
- `server/src/services/mutation/ml/equivalence_detector.rs` (extend)

### Gradient Boosting Model
```rust
pub struct GradientBoostingPredictor {
    model: Box<dyn GBModel>,
    feature_extractor: FeatureExtractor,
    config: GBConfig,
}

impl GradientBoostingPredictor {
    pub fn train(training_data: &[MutantExample]) -> Result<Self>;
    pub fn predict(&self, mutant: &Mutant) -> Result<KillProbability>;
    pub fn feature_importance(&self) -> HashMap<String, f64>;
}
```

### AST-Based Equivalence
```rust
pub struct AstEquivalenceDetector {
    parser: syn::Parser,
    cache: LruCache<String, bool>,
}

impl AstEquivalenceDetector {
    pub fn are_equivalent(&self, original: &str, mutated: &str) -> Result<bool>;
    pub fn detect_tautologies(&self, code: &str) -> Vec<TautologyPattern>;
    pub fn detect_identity_ops(&self, code: &str) -> Vec<IdentityOp>;
}
```

## Success Criteria

- [ ] Accuracy improves from 60-70% to 85-95%
- [ ] Precision > 90% (few false positives)
- [ ] Recall > 80% (catch most killable mutants)
- [ ] Inference time < 10ms per mutant
- [ ] Model size < 50MB
- [ ] Feature importance analysis available
- [ ] All existing tests still pass
- [ ] New tests for GB model (10+ tests)
- [ ] Documentation updated

## Training Data Strategy

1. **Collect Historical Data**: Use existing mutation test results
2. **Synthetic Generation**: Generate mutants with known outcomes
3. **Cross-validation**: 80/20 train/test split with 5-fold CV
4. **Balanced Classes**: Equal representation of killed/survived mutants
5. **Feature Normalization**: Scale features to [0, 1]

## Hyperparameter Tuning

- **Learning Rate**: [0.01, 0.05, 0.1, 0.2]
- **Max Depth**: [3, 5, 7, 10]
- **Num Trees**: [50, 100, 200, 500]
- **Min Samples Leaf**: [1, 5, 10]
- **Subsample**: [0.8, 1.0]

Use grid search or Bayesian optimization.

## Evaluation Metrics

- **Accuracy**: Overall correctness
- **Precision**: TP / (TP + FP) - Avoid false alarms
- **Recall**: TP / (TP + FN) - Catch all killable mutants
- **F1 Score**: Harmonic mean of precision and recall
- **AUC-ROC**: Area under ROC curve
- **Confusion Matrix**: Detailed error analysis

## Technology Choice: Linfa vs LightGBM

### Linfa (Recommended)
- ✅ Pure Rust (no external dependencies)
- ✅ Type-safe, no FFI overhead
- ✅ Good integration with Rust ecosystem
- ❌ Smaller community
- ❌ Less mature than LightGBM

### LightGBM
- ✅ Industry-proven, highly optimized
- ✅ Extensive documentation
- ✅ Best-in-class performance
- ❌ Requires C++ bindings
- ❌ External dependency

**Decision**: Start with Linfa for simplicity, benchmark, consider LightGBM if performance insufficient.

## Value Delivered

**Before**: 60-70% accuracy with statistical baseline
**After**: 85-95% accuracy with gradient boosting ML
**Impact**: Significantly reduces false positives/negatives, improves test quality
**ROI**: High - Better mutant prioritization saves testing time, improves code quality

## Estimated Effort

3-5 days

## Notes

- 18 features already implemented and validated
- Model persistence infrastructure exists
- Focus on accuracy improvements first, then inference speed
- Consider ensemble methods if single model insufficient
- Monitor memory usage during training
- Document feature importance for interpretability
