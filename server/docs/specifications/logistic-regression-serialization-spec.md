# LogisticRegression Serialization & Enhancement Specification

**Version**: 1.0
**Date**: 2025-11-19
**Status**: Ready for Implementation
**Target**: aprender v0.4.0

---

## Executive Summary

Implement SafeTensors serialization for `LogisticRegression` to enable deployment to realizar and complete the binary classification model suite. This specification is grounded in 10 peer-reviewed publications on logistic regression, binary classification, and model deployment.

**Key Deliverables**:
1. `save_safetensors()` / `load_safetensors()` methods for LogisticRegression
2. Model persistence with provenance tracking
3. Integration with realizar inference engine
4. Comprehensive testing (unit, integration, property-based)
5. Documentation and examples

---

## 1. Current State Analysis

### 1.1 LogisticRegression Implementation Status

**Location**: `/home/noah/src/aprender/src/classification/mod.rs`

**Implemented** ✅:
- `LogisticRegression` struct (lines 42-53)
- `fit()` method with gradient descent (lines 117-187)
- `predict()` method with 0.5 threshold (lines 189-199)
- `predict_proba()` method with sigmoid (lines 98-115)
- Serde traits: `Serialize`, `Deserialize` (line 41)

**Missing** ❌:
- `save_safetensors()` method
- `load_safetensors()` method
- Model persistence to disk
- Integration tests with realizar

**Current Structure**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogisticRegression {
    coefficients: Option<Vector<f32>>,  // Weights
    intercept: f32,                     // Bias
    learning_rate: f32,
    max_iter: usize,
    tol: f32,
}
```

### 1.2 Comparison with LinearRegression

**LinearRegression** (✅ HAS SafeTensors):
```rust
// src/linear_model/mod.rs:148-209
pub fn save_safetensors<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
    // Already implemented
}

pub fn load_safetensors<P: AsRef<Path>>(path: P) -> Result<Self, String> {
    // Already implemented
}
```

**LogisticRegression** (❌ NEEDS SafeTensors):
- Same tensor structure (coefficients + intercept)
- Same serialization requirements
- Can reuse SafeTensors infrastructure

---

## 2. Academic Foundation: 10 Peer-Reviewed Publications

### 2.1 Logistic Regression Theory

**[1] Hosmer, Lemeshow & Sturdivant (2013)**. *Applied Logistic Regression*, 3rd Edition. Wiley.

**Key Findings**:
- Maximum likelihood estimation (MLE) is optimal for binary classification
- Sigmoid function: σ(z) = 1 / (1 + e^(-z)) guarantees [0,1] probabilities
- Gradient descent converges to global optimum (convex loss function)

**Applied to Aprender**:
```rust
// Current implementation uses MLE via gradient descent
fn sigmoid(z: f32) -> f32 {
    1.0 / (1.0 + (-z).exp())  // ✅ Correct sigmoid
}

// Binary cross-entropy loss (implicit in gradients)
// L(y, ŷ) = -[y log(ŷ) + (1-y) log(1-ŷ)]
```

**Citation Justification**: Standard reference for logistic regression theory, cited 50,000+ times.

---

**[2] Bishop (2006)**. *Pattern Recognition and Machine Learning*. Springer.

**Key Findings**:
- Logistic regression is a generalized linear model (GLM)
- Probabilistic interpretation: models P(y=1|x)
- Regularization (L1/L2) prevents overfitting

**Applied to Aprender**:
```rust
// Future enhancement: Add regularization
pub struct LogisticRegression {
    // ... existing fields
    regularization: RegularizationType,  // L1, L2, or ElasticNet
    alpha: f32,                          // Regularization strength
}
```

**Citation Justification**: Classic ML textbook, provides theoretical foundation for probabilistic models.

---

### 2.2 Binary Classification Optimization

**[3] Bottou (2010)**. *Large-Scale Machine Learning with Stochastic Gradient Descent*. COMPSTAT 2010.

**Key Findings**:
- Stochastic Gradient Descent (SGD) scales to millions of samples
- Mini-batch SGD balances convergence speed and stability
- Adaptive learning rates (Adam, RMSprop) improve convergence

**Applied to Aprender**:
```rust
// Current: Batch gradient descent
// Future: Add SGD and mini-batch variants
pub enum Optimizer {
    GradientDescent,
    SGD { batch_size: usize },
    Adam { beta1: f32, beta2: f32 },
}
```

**Citation Justification**: Influential paper on SGD for large-scale ML (2,500+ citations).

---

**[4] Kingma & Ba (2015)**. *Adam: A Method for Stochastic Optimization*. ICLR 2015.

**Key Findings**:
- Adam combines momentum and adaptive learning rates
- Robust to hyperparameter choices
- 10-100x faster convergence vs vanilla SGD

**Applied to Aprender**:
```rust
// Future: Adam optimizer for faster convergence
impl LogisticRegression {
    pub fn with_optimizer(mut self, optimizer: Optimizer) -> Self {
        self.optimizer = optimizer;
        self
    }
}
```

**Citation Justification**: Adam is state-of-the-art optimizer (100,000+ citations).

---

### 2.3 Model Calibration & Probability Estimates

**[5] Platt (1999)**. *Probabilistic Outputs for Support Vector Machines*. Advances in Large Margin Classifiers.

**Key Findings**:
- Uncalibrated classifiers produce poor probability estimates
- Platt scaling calibrates probabilities via sigmoid fitting
- Critical for medical/financial applications

**Applied to Aprender**:
```rust
// Current: Raw sigmoid probabilities (well-calibrated for logistic regression)
pub fn predict_proba(&self, x: &Matrix<f32>) -> Vector<f32> {
    // ✅ Already returns calibrated probabilities (MLE property)
}

// Future: Add calibration diagnostics
pub fn calibration_curve(&self, x: &Matrix<f32>, y: &[usize]) -> CalibrationCurve {
    // Returns predicted vs actual probabilities in bins
}
```

**Citation Justification**: Standard reference for probability calibration (10,000+ citations).

---

**[6] Niculescu-Mizil & Caruana (2005)**. *Predicting Good Probabilities with Supervised Learning*. ICML 2005.

**Key Findings**:
- Logistic regression produces better calibrated probabilities than SVM, Naive Bayes
- Calibration curves diagnose probability quality
- Isotonic regression improves calibration

**Applied to Aprender**:
```rust
// Verification: LogisticRegression probabilities are well-calibrated
#[test]
fn test_calibration_quality() {
    // Train on large dataset
    let model = train_logistic_regression();

    // Compute calibration error (ECE)
    let ece = expected_calibration_error(&model, &test_data);

    assert!(ece < 0.05);  // Well-calibrated (ECE < 5%)
}
```

**Citation Justification**: Empirical study of calibration across ML models (3,000+ citations).

---

### 2.4 Model Persistence & Deployment

**[7] Baylor et al. (2017)**. *TFX: A TensorFlow-Based Production-Scale Machine Learning Platform*. KDD 2017.

**Key Findings**:
- Model versioning critical for A/B testing
- Provenance tracking ensures reproducibility
- Schema validation prevents deployment errors

**Applied to Aprender**:
```rust
// SafeTensors metadata for provenance
pub fn save_safetensors_with_provenance<P: AsRef<Path>>(
    &self,
    path: P,
    provenance: ModelProvenance,
) -> Result<(), String> {
    let metadata = hashmap! {
        "model.type" => "logistic_regression",
        "aprender.version" => env!("CARGO_PKG_VERSION"),
        "training.git_commit" => provenance.git_commit,
        "training.dataset_hash" => provenance.dataset_hash,
        "training.random_seed" => provenance.random_seed.to_string(),
        "hyperparams.learning_rate" => self.learning_rate.to_string(),
        "hyperparams.max_iter" => self.max_iter.to_string(),
    };
    // ... save with metadata
}
```

**Citation Justification**: Production ML deployment best practices from Google (1,500+ citations).

---

**[8] Sculley et al. (2015)**. *Hidden Technical Debt in Machine Learning Systems*. NeurIPS 2015.

**Key Findings**:
- Model format changes create technical debt
- Glue code and pipeline jungles slow development
- Schema-driven formats reduce integration issues

**Applied to Aprender**:
```rust
// SafeTensors schema validation
pub fn load_safetensors<P: AsRef<Path>>(path: P) -> Result<Self, String> {
    let (metadata, raw_data) = safetensors::load_safetensors(path)?;

    // Validate model type
    if metadata.get("model.type") != Some("logistic_regression") {
        return Err("Not a LogisticRegression model");
    }

    // Validate required tensors
    if !metadata.contains_key("coefficients") || !metadata.contains_key("intercept") {
        return Err("Missing required tensors");
    }

    // ... deserialize
}
```

**Citation Justification**: Influential paper on ML system design (5,000+ citations).

---

### 2.5 Verification & Testing

**[9] Zhang et al. (2018)**. *Machine Learning Testing: Survey, Landscapes and Horizons*. IEEE Trans. Software Engineering.

**Key Findings**:
- Property-based testing critical for ML models
- Invariants: predict(x1) == predict(x2) if x1 == x2
- Metamorphic testing: predict_proba(x) ∈ [0, 1]

**Applied to Aprender**:
```rust
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn probabilities_always_in_zero_one(
            x in prop::collection::vec(prop::num::f32::NORMAL, 10..100),
        ) {
            let model = train_model();
            let probas = model.predict_proba(&x);

            for &p in probas.as_slice() {
                prop_assert!(p >= 0.0 && p <= 1.0);
            }
        }

        #[test]
        fn serialization_roundtrip_preserves_predictions(
            x in prop::collection::vec(prop::num::f32::NORMAL, 10..100),
        ) {
            let model = train_model();
            let predictions_before = model.predict(&x);

            model.save_safetensors("/tmp/model.safetensors")?;
            let loaded = LogisticRegression::load_safetensors("/tmp/model.safetensors")?;
            let predictions_after = loaded.predict(&x);

            prop_assert_eq!(predictions_before, predictions_after);
        }
    }
}
```

**Citation Justification**: Comprehensive survey of ML testing (1,200+ citations).

---

**[10] Pei et al. (2017)**. *DeepXplore: Automated Whitebox Testing of Deep Learning Systems*. SOSP 2017.

**Key Findings**:
- Neuron coverage metrics detect undertested regions
- Differential testing finds inconsistencies
- Gradient-based fuzzing generates adversarial inputs

**Applied to Aprender**:
```rust
// Differential testing: LinearRegression vs LogisticRegression
#[test]
fn test_linear_vs_logistic_on_linearly_separable_data() {
    let (x, y) = generate_linearly_separable_data();

    // Both should achieve 100% accuracy
    let mut linear = LinearRegression::new();
    linear.fit(&x, &y.iter().map(|&label| label as f32).collect());

    let mut logistic = LogisticRegression::new();
    logistic.fit(&x, &y).unwrap();

    assert_eq!(linear.predict(&x), logistic.predict(&x));
}
```

**Citation Justification**: Pioneering work on ML system testing (1,800+ citations).

---

## 3. Implementation Specification

### 3.1 SafeTensors Format for LogisticRegression

**Tensor Structure**:
```
SafeTensors File:
  Header: [8 bytes] metadata length (u64 little-endian)
  Metadata: [JSON] {
    "coefficients": {
      "dtype": "F32",
      "shape": [n_features],
      "data_offsets": [0, n_features * 4]
    },
    "intercept": {
      "dtype": "F32",
      "shape": [1],
      "data_offsets": [n_features * 4, n_features * 4 + 4]
    },
    "__metadata__": {
      "model.type": "logistic_regression",
      "aprender.version": "0.4.0",
      "training.learning_rate": "0.01",
      "training.max_iter": "1000"
    }
  }
  Data: [n_features * 4 + 4 bytes] coefficients + intercept (f32 little-endian)
```

### 3.2 Implementation: save_safetensors()

```rust
impl LogisticRegression {
    /// Saves the trained model to SafeTensors format.
    ///
    /// # Arguments
    ///
    /// * `path` - File path to save the model
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Model is not fitted (call `fit()` first)
    /// - File writing fails
    /// - Serialization fails
    ///
    /// # Example
    ///
    /// ```
    /// use aprender::classification::LogisticRegression;
    /// # use aprender::prelude::*;
    ///
    /// let mut model = LogisticRegression::new();
    /// # let x = Matrix::from_vec(2, 2, vec![0.0, 0.0, 1.0, 1.0]).unwrap();
    /// # let y = vec![0, 1];
    /// model.fit(&x, &y).unwrap();
    ///
    /// model.save_safetensors("model.safetensors").unwrap();
    /// ```
    pub fn save_safetensors<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        use crate::serialization::safetensors;
        use std::collections::BTreeMap;

        // Verify model is fitted
        let coefficients = self
            .coefficients
            .as_ref()
            .ok_or("Cannot save unfitted model. Call fit() first.")?;

        // Prepare tensors (BTreeMap ensures deterministic ordering)
        let mut tensors = BTreeMap::new();
        tensors.insert(
            "coefficients".to_string(),
            coefficients.as_slice().to_vec(),
        );
        tensors.insert("intercept".to_string(), vec![self.intercept]);

        // Add model metadata
        let mut metadata = BTreeMap::new();
        metadata.insert("model.type".to_string(), "logistic_regression".to_string());
        metadata.insert(
            "aprender.version".to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
        );
        metadata.insert(
            "training.learning_rate".to_string(),
            self.learning_rate.to_string(),
        );
        metadata.insert(
            "training.max_iter".to_string(),
            self.max_iter.to_string(),
        );

        // Save to SafeTensors format
        safetensors::save_safetensors(path, tensors, Some(metadata))?;
        Ok(())
    }
}
```

### 3.3 Implementation: load_safetensors()

```rust
impl LogisticRegression {
    /// Loads a model from SafeTensors format.
    ///
    /// # Arguments
    ///
    /// * `path` - File path to load the model from
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - File reading fails
    /// - SafeTensors format is invalid
    /// - Required tensors are missing
    /// - Model type mismatch
    ///
    /// # Example
    ///
    /// ```
    /// use aprender::classification::LogisticRegression;
    ///
    /// let model = LogisticRegression::load_safetensors("model.safetensors").unwrap();
    /// ```
    pub fn load_safetensors<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        use crate::serialization::safetensors;

        // Load SafeTensors file
        let (metadata, raw_data) = safetensors::load_safetensors(path)?;

        // Validate model type
        if let Some(model_type) = metadata.get("__metadata__") {
            if let Some(model_type_str) = model_type.get("model.type") {
                if model_type_str != "logistic_regression" {
                    return Err(format!(
                        "Model type mismatch: expected 'logistic_regression', got '{}'",
                        model_type_str
                    ));
                }
            }
        }

        // Extract coefficients tensor
        let coef_meta = metadata
            .get("coefficients")
            .ok_or("Missing 'coefficients' tensor in SafeTensors file")?;
        let coef_data = safetensors::extract_tensor(&raw_data, coef_meta)?;
        let coefficients = Vector::from_vec(coef_data);

        // Extract intercept tensor
        let intercept_meta = metadata
            .get("intercept")
            .ok_or("Missing 'intercept' tensor in SafeTensors file")?;
        let intercept_data = safetensors::extract_tensor(&raw_data, intercept_meta)?;
        if intercept_data.len() != 1 {
            return Err("Intercept must be a single value".to_string());
        }
        let intercept = intercept_data[0];

        // Extract hyperparameters from metadata
        let learning_rate = metadata
            .get("__metadata__")
            .and_then(|m| m.get("training.learning_rate"))
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(0.01);

        let max_iter = metadata
            .get("__metadata__")
            .and_then(|m| m.get("training.max_iter"))
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(1000);

        Ok(Self {
            coefficients: Some(coefficients),
            intercept,
            learning_rate,
            max_iter,
            tol: 1e-4,
        })
    }
}
```

---

## 4. Testing Strategy

### 4.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_save_unfitted_model_fails() {
        let model = LogisticRegression::new();
        let result = model.save_safetensors("/tmp/model.safetensors");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unfitted"));
    }

    #[test]
    fn test_save_load_roundtrip() {
        // Train model
        let x = Matrix::from_vec(4, 2, vec![0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0]).unwrap();
        let y = vec![0, 0, 0, 1];

        let mut model = LogisticRegression::new();
        model.fit(&x, &y).unwrap();

        // Save
        let path = "/tmp/test_logistic_regression.safetensors";
        model.save_safetensors(path).unwrap();

        // Load
        let loaded = LogisticRegression::load_safetensors(path).unwrap();

        // Verify coefficients match
        assert_eq!(model.coefficients, loaded.coefficients);
        assert_eq!(model.intercept, loaded.intercept);

        // Verify predictions match
        let predictions_original = model.predict(&x);
        let predictions_loaded = loaded.predict(&x);
        assert_eq!(predictions_original, predictions_loaded);

        // Cleanup
        fs::remove_file(path).ok();
    }

    #[test]
    fn test_load_corrupted_file() {
        let path = "/tmp/corrupted.safetensors";
        fs::write(path, b"CORRUPTED DATA").unwrap();

        let result = LogisticRegression::load_safetensors(path);
        assert!(result.is_err());

        fs::remove_file(path).ok();
    }

    #[test]
    fn test_load_wrong_model_type() {
        // Create a LinearRegression model and try to load as LogisticRegression
        let mut linear = LinearRegression::new();
        let x = Matrix::from_vec(2, 1, vec![1.0, 2.0]).unwrap();
        let y = vec![2.0, 4.0];
        linear.fit(&x, &y).unwrap();

        let path = "/tmp/linear_model.safetensors";
        linear.save_safetensors(path).unwrap();

        let result = LogisticRegression::load_safetensors(path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Model type mismatch"));

        fs::remove_file(path).ok();
    }
}
```

### 4.2 Integration Tests

```rust
#[test]
fn test_aprender_logistic_regression_to_realizar() {
    // 1. Train LogisticRegression in aprender
    let x = Matrix::from_vec(100, 2, generate_training_data()).unwrap();
    let y = generate_labels();

    let mut model = LogisticRegression::new()
        .with_learning_rate(0.1)
        .with_max_iter(1000);
    model.fit(&x, &y).unwrap();

    // 2. Save to SafeTensors
    let path = "/tmp/logistic_regression_test.safetensors";
    model.save_safetensors(path).unwrap();

    // 3. Load in realizar (using existing SafeTensors parser)
    let realizar_model = realizar::SafetensorsModel::from_bytes(
        std::fs::read(path).unwrap()
    ).unwrap();

    // 4. Verify tensors exist
    assert!(realizar_model.get_tensor("coefficients").is_ok());
    assert!(realizar_model.get_tensor("intercept").is_ok());

    // 5. Verify metadata
    let metadata = realizar_model.metadata();
    assert_eq!(metadata.get("model.type"), Some(&"logistic_regression".to_string()));

    // Cleanup
    std::fs::remove_file(path).ok();
}
```

### 4.3 Property-Based Tests

```rust
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn serialization_preserves_predictions(
            coeffs in prop::collection::vec(prop::num::f32::NORMAL, 1..10),
            intercept in prop::num::f32::NORMAL,
        ) {
            // Create model with known parameters
            let mut model = LogisticRegression::new();
            model.coefficients = Some(Vector::from_vec(coeffs));
            model.intercept = intercept;

            // Test data
            let x = Matrix::from_vec(10, model.coefficients.as_ref().unwrap().len(),
                vec![0.0; 10 * model.coefficients.as_ref().unwrap().len()]).unwrap();

            let predictions_before = model.predict(&x);

            // Roundtrip
            let path = "/tmp/proptest_model.safetensors";
            model.save_safetensors(path)?;
            let loaded = LogisticRegression::load_safetensors(path)?;
            let predictions_after = loaded.predict(&x);

            prop_assert_eq!(predictions_before, predictions_after);

            std::fs::remove_file(path).ok();
        }

        #[test]
        fn probabilities_always_valid(
            coeffs in prop::collection::vec(prop::num::f32::NORMAL, 1..10),
            intercept in prop::num::f32::NORMAL,
        ) {
            let mut model = LogisticRegression::new();
            model.coefficients = Some(Vector::from_vec(coeffs.clone()));
            model.intercept = intercept;

            let x = Matrix::from_vec(10, coeffs.len(), vec![0.0; 10 * coeffs.len()]).unwrap();
            let probas = model.predict_proba(&x);

            for &p in probas.as_slice() {
                prop_assert!(p >= 0.0 && p <= 1.0);
                prop_assert!(!p.is_nan());
            }
        }
    }
}
```

---

## 5. Implementation Roadmap

### Sprint 1: Core Serialization (2 weeks)

**Tasks**:
- [ ] Implement `save_safetensors()` method
- [ ] Implement `load_safetensors()` method
- [ ] Add metadata support (model type, hyperparameters)
- [ ] Unit tests (save, load, roundtrip, error cases)
- [ ] Documentation with rustdoc examples

**Deliverables**:
- ✅ `LogisticRegression::save_safetensors()` functional
- ✅ `LogisticRegression::load_safetensors()` functional
- ✅ 10+ unit tests passing
- ✅ Zero clippy warnings

### Sprint 2: Integration & Testing (2 weeks)

**Tasks**:
- [ ] Integration test: aprender → realizar
- [ ] Property-based tests (roundtrip preservation)
- [ ] Error handling tests (corrupted files, wrong types)
- [ ] Performance benchmarks
- [ ] Book chapter example

**Deliverables**:
- ✅ Integration test: aprender → realizar passing
- ✅ 20+ tests total (unit + integration + property)
- ✅ Test coverage ≥85%
- ✅ Documentation complete

---

## 6. Success Criteria

### Phase 1: Serialization Complete

- ✅ `save_safetensors()` implemented and tested
- ✅ `load_safetensors()` implemented and tested
- ✅ Metadata includes model type and hyperparameters
- ✅ All tests passing (unit + integration + property)
- ✅ Zero clippy warnings
- ✅ Documentation with examples

### Phase 2: Integration Verified

- ✅ Realizar loads LogisticRegression models
- ✅ Predictions identical before/after serialization
- ✅ Test coverage ≥85%
- ✅ Performance: <1ms serialization for 1000 features

---

## 7. Dependencies

**No new dependencies required** - reuses existing SafeTensors infrastructure from LinearRegression implementation.

```toml
# Already in Cargo.toml
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

---

## 8. Use Cases

### 8.1 ML Predictor in paiml-mcp-agent-toolkit

**Current** (LinearRegression):
```rust
// server/src/services/mutation/ml_predictor.rs:275
model: Option<LinearRegression>
```

**Future** (LogisticRegression):
```rust
model: Option<LogisticRegression>  // Better for binary classification

// Proper probability estimates [0.0, 1.0]
let kill_probability = model.predict_proba(&features)[0];

// Save trained model
model.save_safetensors("survivability_model.safetensors")?;
```

### 8.2 Realizar Deployment

```bash
# Train in aprender
aprender train --model logistic_regression \
    --data mutations.csv \
    --output model.safetensors

# Deploy to realizar
realizar upload model.safetensors \
    --name "mutation-survivability" \
    --version "v1.0.0"

# Inference
curl -X POST http://realizar:8080/predict/mutation-survivability \
    -d '{"features": [1.0, 2.5, 3.7]}'
```

### 8.3 Ollama Integration (via GGUF conversion)

```bash
# Convert to GGUF
aprender convert model.safetensors --format gguf --output model.gguf

# Deploy via Ollama
ollama create mutation-predictor -f Modelfile
ollama run mutation-predictor "[1.0, 2.5, 3.7]"
```

---

## 9. References (10 Peer-Reviewed Publications)

1. **Hosmer, Lemeshow & Sturdivant (2013)**. *Applied Logistic Regression*, 3rd Ed. Wiley. [ISBN: 978-0470582473]
2. **Bishop (2006)**. *Pattern Recognition and Machine Learning*. Springer. [ISBN: 978-0387310732]
3. **Bottou (2010)**. *Large-Scale ML with SGD*. COMPSTAT 2010. [doi:10.1007/978-3-7908-2604-3_16]
4. **Kingma & Ba (2015)**. *Adam: Stochastic Optimization*. ICLR 2015. [arXiv:1412.6980]
5. **Platt (1999)**. *Probabilistic Outputs for SVMs*. Advances in Large Margin Classifiers. [ISBN: 978-0262194181]
6. **Niculescu-Mizil & Caruana (2005)**. *Predicting Good Probabilities*. ICML 2005. [doi:10.1145/1102351.1102430]
7. **Baylor et al. (2017)**. *TFX Production Platform*. KDD 2017. [doi:10.1145/3097983.3098021]
8. **Sculley et al. (2015)**. *Hidden Technical Debt in ML*. NeurIPS 2015. [Paper ID: 5656]
9. **Zhang et al. (2018)**. *ML Testing Survey*. IEEE TSE 2018. [doi:10.1109/TSE.2018.2897847]
10. **Pei et al. (2017)**. *DeepXplore Whitebox Testing*. SOSP 2017. [doi:10.1145/3132747.3132785]

---

## Appendix A: Current LogisticRegression Status

**File**: `/home/noah/src/aprender/src/classification/mod.rs`
**Lines**: 200 (41-240)

**Implemented Methods**:
- ✅ `new()` - Constructor
- ✅ `with_learning_rate()` - Builder
- ✅ `with_max_iter()` - Builder
- ✅ `with_tolerance()` - Builder
- ✅ `sigmoid()` - Activation function
- ✅ `predict_proba()` - Probability estimates
- ✅ `fit()` - Training with gradient descent
- ✅ `predict()` - Class predictions

**Missing Methods**:
- ❌ `save_safetensors()` - Model persistence
- ❌ `load_safetensors()` - Model loading

**Verification**:
- ✅ Serde traits present (`Serialize`, `Deserialize`)
- ✅ Struct matches LinearRegression pattern
- ✅ Can reuse SafeTensors infrastructure

---

**Generated**: 2025-11-19
**Status**: Ready for Implementation
**Target Version**: aprender v0.4.0
**Estimated Effort**: 4 weeks (2 sprints)
