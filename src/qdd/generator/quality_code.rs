//! Quality code generation - orchestrates AST building, test generation, and documentation

use super::ast_builder::AstBuilder;
use super::doc_gen::DocGenerator;
use super::test_gen::TestGenerator;
use crate::qdd::{
    Checkpoint, CodeType, CreateSpec, QddResult, QualityMetrics, QualityProfile, QualityScore,
    RollbackPlan,
};
use anyhow::{anyhow, Result};

/// Quality-focused code generator
pub struct QualityCodeGenerator {
    pub(crate) profile: QualityProfile,
    ast_builder: AstBuilder,
    test_generator: TestGenerator,
    doc_generator: DocGenerator,
}

impl QualityCodeGenerator {
    /// Create generator with quality profile
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(profile: QualityProfile) -> Self {
        Self {
            ast_builder: AstBuilder::new(profile.clone()),
            test_generator: TestGenerator::new(profile.clone()),
            doc_generator: DocGenerator::new(profile.clone()),
            profile,
        }
    }
}

// Code creation methods: create, create_function, create_module, create_service, create_test
include!("quality_code_creation.rs");

// Quality analysis methods: estimate_complexity, calculate_quality_score, calculate_metrics, etc.
include!("quality_code_analysis.rs");

// Unit tests
include!("quality_code_tests.rs");

// Property-based tests
include!("quality_code_property_tests.rs");
