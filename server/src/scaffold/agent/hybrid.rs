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
    pub fn new() -> Self {
        Self {
            verification_method: VerificationMethod::PropertyTests,
            max_complexity: 10,
            invariants: Vec::new(),
        }
    }

    /// Set the verification method.
    pub fn verification_method(mut self, method: VerificationMethod) -> Self {
        self.verification_method = method;
        self
    }

    /// Set the maximum complexity.
    pub fn max_complexity(mut self, complexity: u32) -> Self {
        self.max_complexity = complexity;
        self
    }

    /// Add an invariant.
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
    pub fn new() -> Self {
        Self {
            model_type: ModelType::GPT4,
            fallback_strategy: FallbackStrategy::Deterministic,
            confidence_threshold: 0.95,
        }
    }

    /// Set the model type.
    pub fn model_type(mut self, model: ModelType) -> Self {
        self.model_type = model;
        self
    }

    /// Set the fallback strategy.
    pub fn fallback_strategy(mut self, strategy: FallbackStrategy) -> Self {
        self.fallback_strategy = strategy;
        self
    }

    /// Set the confidence threshold.
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelType {
    /// OpenAI GPT-4.
    GPT4,
    /// Anthropic Claude.
    Claude,
    /// Local model with specified path.
    Local(String),
}

/// Fallback strategies when the model fails.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// MessagePack format.
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
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            severity: InvariantSeverity::Error,
        }
    }

    /// Set the severity.
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
pub fn validate_complexity_for_quality(
    spec: &CoreSpec,
    quality: crate::scaffold::QualityLevel,
) -> bool {
    spec.max_complexity <= quality.max_complexity()
}

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
}