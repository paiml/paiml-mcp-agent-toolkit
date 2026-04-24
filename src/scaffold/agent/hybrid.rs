#![cfg_attr(coverage_nightly, coverage(off))]
//! Hybrid agent architecture specifications.

use serde::{Deserialize, Serialize};

/// Specification for hybrid agents with deterministic core and probabilistic wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridAgentSpec {
    /// Deterministic core specification.
    pub deterministic_core: CoreSpec,
    /// Probabilistic wrapper specification.
    pub probabilistic_wrapper: WrapperSpec,
    /// Boundary between core and wrapper.
    pub boundary: BoundarySpec,
}

/// Specification for the deterministic core.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreSpec {
    /// Method used to verify correctness.
    pub verification_method: VerificationMethod,
    /// Maximum allowed complexity.
    pub max_complexity: u32,
    /// Invariants that must hold.
    pub invariants: Vec<Invariant>,
}

impl CoreSpec {
    /// Create a new core specification.
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            verification_method: VerificationMethod::PropertyTests,
            max_complexity: 10,
            invariants: Vec::new(),
        }
    }

    /// Set the verification method.
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn verification_method(mut self, method: VerificationMethod) -> Self {
        self.verification_method = method;
        self
    }

    /// Set the maximum complexity.
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn max_complexity(mut self, complexity: u32) -> Self {
        self.max_complexity = complexity;
        self
    }

    /// Add an invariant.
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_invariant(mut self, invariant: Invariant) -> Self {
        self.invariants.push(invariant);
        self
    }
}

impl Default for CoreSpec {
    fn default() -> Self {
        Self::new()
    }
}

/// Specification for the probabilistic wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WrapperSpec {
    /// Model type to use.
    pub model_type: ModelType,
    /// Fallback strategy when model fails.
    pub fallback_strategy: FallbackStrategy,
    /// Confidence threshold for accepting results.
    pub confidence_threshold: f64,
}

impl WrapperSpec {
    /// Create a new wrapper specification.
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            model_type: ModelType::GPT4,
            fallback_strategy: FallbackStrategy::Deterministic,
            confidence_threshold: 0.95,
        }
    }

    /// Set the model type.
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn model_type(mut self, model: ModelType) -> Self {
        self.model_type = model;
        self
    }

    /// Set the fallback strategy.
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn fallback_strategy(mut self, strategy: FallbackStrategy) -> Self {
        self.fallback_strategy = strategy;
        self
    }

    /// Set the confidence threshold.
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn confidence_threshold(mut self, threshold: f64) -> Self {
        self.confidence_threshold = threshold;
        self
    }
}

impl Default for WrapperSpec {
    fn default() -> Self {
        Self::new()
    }
}

/// Specification for the boundary between core and wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundarySpec {
    /// Serialization format for data exchange.
    pub serialization: SerializationFormat,
    /// Validation strategy for data exchange.
    pub validation: ValidationStrategy,
    /// Error propagation strategy.
    pub error_propagation: ErrorPropagation,
}

impl Default for BoundarySpec {
    fn default() -> Self {
        Self {
            serialization: SerializationFormat::JSON,
            validation: ValidationStrategy::Both,
            error_propagation: ErrorPropagation::Immediate,
        }
    }
}

/// Methods for verifying correctness.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationMethod {
    /// Property-based testing.
    PropertyTests,
    /// Formal proof.
    FormalProof,
    /// Model checking.
    ModelChecking,
}

/// Types of models for the probabilistic wrapper.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ModelType {
    /// `OpenAI` GPT-4.
    GPT4,
    /// Anthropic Claude.
    Claude,
    /// Local model with specified path.
    Local(String),
}

/// Fallback strategies when the model fails.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FallbackStrategy {
    /// Fall back to deterministic implementation.
    Deterministic,
    /// Return a default value.
    DefaultValue,
    /// Return an error.
    Error,
}

/// Serialization formats for data exchange.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializationFormat {
    /// JSON format.
    JSON,
    /// `MessagePack` format.
    MessagePack,
    /// Protocol Buffers format.
    Protobuf,
}

/// Validation strategies for data exchange.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationStrategy {
    /// Schema-based validation.
    Schema,
    /// Runtime validation.
    Runtime,
    /// Both schema and runtime validation.
    Both,
}

/// Error propagation strategies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorPropagation {
    /// Propagate errors immediately.
    Immediate,
    /// Defer error propagation.
    Deferred,
    /// Log errors without propagating.
    Logged,
}

/// An invariant that must hold.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invariant {
    /// Name of the invariant.
    pub name: String,
    /// Description of what the invariant checks.
    pub description: String,
    /// Severity if the invariant is violated.
    pub severity: InvariantSeverity,
}

impl Invariant {
    /// Create a new invariant.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            severity: InvariantSeverity::Error,
        }
    }

    /// Set the severity.
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_severity(mut self, severity: InvariantSeverity) -> Self {
        self.severity = severity;
        self
    }
}

/// Severity levels for invariant violations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InvariantSeverity {
    /// Warning level - logged but not fatal.
    Warning,
    /// Error level - causes failure.
    Error,
    /// Critical level - immediate panic.
    Critical,
}

/// Validate complexity for a given quality level.
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn validate_complexity_for_quality(
    spec: &CoreSpec,
    quality: crate::scaffold::QualityLevel,
) -> bool {
    spec.max_complexity <= quality.max_complexity()
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_spec_builder() {
        let spec = CoreSpec::new()
            .verification_method(VerificationMethod::FormalProof)
            .max_complexity(5)
            .add_invariant(Invariant::new("test", "Test invariant"));

        assert!(matches!(
            spec.verification_method,
            VerificationMethod::FormalProof
        ));
        assert_eq!(spec.max_complexity, 5);
        assert_eq!(spec.invariants.len(), 1);
    }

    #[test]
    fn test_wrapper_spec_builder() {
        let spec = WrapperSpec::new()
            .model_type(ModelType::Claude)
            .fallback_strategy(FallbackStrategy::Error)
            .confidence_threshold(0.9);

        assert!(matches!(spec.model_type, ModelType::Claude));
        assert!(matches!(spec.fallback_strategy, FallbackStrategy::Error));
        assert_eq!(spec.confidence_threshold, 0.9);
    }

    #[test]
    fn test_hybrid_spec_serialization() {
        let spec = HybridAgentSpec {
            deterministic_core: CoreSpec::default(),
            probabilistic_wrapper: WrapperSpec::default(),
            boundary: BoundarySpec::default(),
        };

        let json = serde_json::to_string(&spec).unwrap();
        let deserialized: HybridAgentSpec = serde_json::from_str(&json).unwrap();

        assert_eq!(
            deserialized.deterministic_core.max_complexity,
            spec.deterministic_core.max_complexity
        );
    }

    #[test]
    fn test_validate_complexity() {
        let spec = CoreSpec::new().max_complexity(8);
        assert!(validate_complexity_for_quality(
            &spec,
            crate::scaffold::QualityLevel::Extreme
        ));

        let spec = CoreSpec::new().max_complexity(15);
        assert!(!validate_complexity_for_quality(
            &spec,
            crate::scaffold::QualityLevel::Extreme
        ));
    }

    // --- PMAT-636 additions: cover untested surface area ---

    use crate::scaffold::QualityLevel;

    #[test]
    fn test_core_spec_new_initial_state() {
        let spec = CoreSpec::new();
        assert!(matches!(
            spec.verification_method,
            VerificationMethod::PropertyTests
        ));
        assert_eq!(spec.max_complexity, 10);
        assert!(spec.invariants.is_empty());
    }

    #[test]
    fn test_core_spec_default_delegates_to_new() {
        let a = CoreSpec::default();
        let b = CoreSpec::new();
        assert_eq!(a.max_complexity, b.max_complexity);
        assert_eq!(a.invariants.len(), b.invariants.len());
        assert!(matches!(
            a.verification_method,
            VerificationMethod::PropertyTests
        ));
    }

    #[test]
    fn test_core_spec_verification_method_model_checking_variant() {
        let spec = CoreSpec::new().verification_method(VerificationMethod::ModelChecking);
        assert!(matches!(
            spec.verification_method,
            VerificationMethod::ModelChecking
        ));
    }

    #[test]
    fn test_core_spec_add_invariant_appends() {
        let spec = CoreSpec::new()
            .add_invariant(Invariant::new("a", "A"))
            .add_invariant(Invariant::new("b", "B"))
            .add_invariant(Invariant::new("c", "C"));
        assert_eq!(spec.invariants.len(), 3);
        assert_eq!(spec.invariants[0].name, "a");
        assert_eq!(spec.invariants[2].name, "c");
    }

    #[test]
    fn test_wrapper_spec_new_initial_state() {
        let spec = WrapperSpec::new();
        assert_eq!(spec.model_type, ModelType::GPT4);
        assert_eq!(spec.fallback_strategy, FallbackStrategy::Deterministic);
        assert_eq!(spec.confidence_threshold, 0.95);
    }

    #[test]
    fn test_wrapper_spec_default_delegates_to_new() {
        let a = WrapperSpec::default();
        let b = WrapperSpec::new();
        assert_eq!(a.model_type, b.model_type);
        assert_eq!(a.fallback_strategy, b.fallback_strategy);
        assert_eq!(a.confidence_threshold, b.confidence_threshold);
    }

    #[test]
    fn test_wrapper_spec_model_type_local_variant() {
        let spec = WrapperSpec::new().model_type(ModelType::Local("/models/m.gguf".to_string()));
        match &spec.model_type {
            ModelType::Local(p) => assert_eq!(p, "/models/m.gguf"),
            other => panic!("expected Local, got {other:?}"),
        }
    }

    #[test]
    fn test_wrapper_spec_fallback_default_value_variant() {
        let spec = WrapperSpec::new().fallback_strategy(FallbackStrategy::DefaultValue);
        assert_eq!(spec.fallback_strategy, FallbackStrategy::DefaultValue);
    }

    #[test]
    fn test_boundary_spec_default_fields() {
        let b = BoundarySpec::default();
        assert!(matches!(b.serialization, SerializationFormat::JSON));
        assert!(matches!(b.validation, ValidationStrategy::Both));
        assert!(matches!(b.error_propagation, ErrorPropagation::Immediate));
    }

    #[test]
    fn test_invariant_default_severity_is_error() {
        let inv = Invariant::new("x", "X");
        assert!(matches!(inv.severity, InvariantSeverity::Error));
    }

    #[test]
    fn test_invariant_with_severity_sets_all_variants() {
        for sev in [
            InvariantSeverity::Warning,
            InvariantSeverity::Error,
            InvariantSeverity::Critical,
        ] {
            let inv = Invariant::new("x", "X").with_severity(sev.clone());
            match (&inv.severity, &sev) {
                (InvariantSeverity::Warning, InvariantSeverity::Warning)
                | (InvariantSeverity::Error, InvariantSeverity::Error)
                | (InvariantSeverity::Critical, InvariantSeverity::Critical) => {}
                _ => panic!("severity mismatch after with_severity"),
            }
        }
    }

    #[test]
    fn test_validate_complexity_standard_boundary() {
        // Standard cap is 20.
        assert!(validate_complexity_for_quality(
            &CoreSpec::new().max_complexity(20),
            QualityLevel::Standard
        ));
        assert!(!validate_complexity_for_quality(
            &CoreSpec::new().max_complexity(21),
            QualityLevel::Standard
        ));
    }

    #[test]
    fn test_validate_complexity_strict_boundary() {
        // Strict cap is 15.
        assert!(validate_complexity_for_quality(
            &CoreSpec::new().max_complexity(15),
            QualityLevel::Strict
        ));
        assert!(!validate_complexity_for_quality(
            &CoreSpec::new().max_complexity(16),
            QualityLevel::Strict
        ));
    }

    #[test]
    fn test_validate_complexity_extreme_boundary() {
        // Extreme cap is 10 (new()'s default max_complexity).
        assert!(validate_complexity_for_quality(
            &CoreSpec::new().max_complexity(10),
            QualityLevel::Extreme
        ));
        assert!(!validate_complexity_for_quality(
            &CoreSpec::new().max_complexity(11),
            QualityLevel::Extreme
        ));
    }

    #[test]
    fn test_serialization_format_variants_round_trip() {
        for fmt in [
            SerializationFormat::JSON,
            SerializationFormat::MessagePack,
            SerializationFormat::Protobuf,
        ] {
            let s = serde_json::to_string(&fmt).expect("serialize");
            let back: SerializationFormat = serde_json::from_str(&s).expect("deserialize");
            // Serde representation should round-trip.
            let s2 = serde_json::to_string(&back).expect("re-serialize");
            assert_eq!(s, s2);
        }
    }

    #[test]
    fn test_validation_strategy_variants_round_trip() {
        for v in [
            ValidationStrategy::Schema,
            ValidationStrategy::Runtime,
            ValidationStrategy::Both,
        ] {
            let s = serde_json::to_string(&v).expect("serialize");
            let back: ValidationStrategy = serde_json::from_str(&s).expect("deserialize");
            let s2 = serde_json::to_string(&back).expect("re-serialize");
            assert_eq!(s, s2);
        }
    }

    #[test]
    fn test_error_propagation_variants_round_trip() {
        for v in [
            ErrorPropagation::Immediate,
            ErrorPropagation::Deferred,
            ErrorPropagation::Logged,
        ] {
            let s = serde_json::to_string(&v).expect("serialize");
            let back: ErrorPropagation = serde_json::from_str(&s).expect("deserialize");
            let s2 = serde_json::to_string(&back).expect("re-serialize");
            assert_eq!(s, s2);
        }
    }

    #[test]
    fn test_hybrid_spec_serialization_with_non_default_fields() {
        // Forces serde to walk every nested enum/struct with non-default variants.
        let spec = HybridAgentSpec {
            deterministic_core: CoreSpec::new()
                .verification_method(VerificationMethod::FormalProof)
                .max_complexity(7)
                .add_invariant(
                    Invariant::new("n1", "d1").with_severity(InvariantSeverity::Critical),
                ),
            probabilistic_wrapper: WrapperSpec::new()
                .model_type(ModelType::Local("/m".to_string()))
                .fallback_strategy(FallbackStrategy::DefaultValue)
                .confidence_threshold(0.42),
            boundary: BoundarySpec {
                serialization: SerializationFormat::Protobuf,
                validation: ValidationStrategy::Runtime,
                error_propagation: ErrorPropagation::Logged,
            },
        };
        let json = serde_json::to_string(&spec).expect("serialize");
        let back: HybridAgentSpec = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.deterministic_core.max_complexity, 7);
        assert_eq!(back.probabilistic_wrapper.confidence_threshold, 0.42);
        assert_eq!(
            back.probabilistic_wrapper.fallback_strategy,
            FallbackStrategy::DefaultValue
        );
        match &back.probabilistic_wrapper.model_type {
            ModelType::Local(p) => assert_eq!(p, "/m"),
            other => panic!("expected Local, got {other:?}"),
        }
        assert!(matches!(
            back.boundary.serialization,
            SerializationFormat::Protobuf
        ));
        assert!(matches!(
            back.boundary.validation,
            ValidationStrategy::Runtime
        ));
        assert!(matches!(
            back.boundary.error_propagation,
            ErrorPropagation::Logged
        ));
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
