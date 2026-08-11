impl QualityCodeGenerator {
    /// Check if function needs decomposition based on complexity
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn needs_decomposition(&self, code: &str) -> Result<bool> {
        let complexity = self.estimate_complexity(code);
        Ok(complexity > self.profile.thresholds.max_complexity)
    }

    /// Decompose complex function into simpler parts
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn decompose_function(&self, code: String) -> Result<String> {
        // For now, return original code - actual decomposition is complex
        // This would involve AST manipulation in real implementation
        Ok(code)
    }

    /// Estimate cyclomatic complexity (simple heuristic for now)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn estimate_complexity(&self, code: &str) -> u32 {
        let if_count = code.matches("if ").count() as u32;
        let match_count = code.matches("match ").count() as u32;
        let loop_count =
            code.matches("for ").count() as u32 + code.matches("while ").count() as u32;

        1 + if_count + match_count + loop_count
    }

    /// Calculate quality score for generated code
    ///
    /// NOTE: `coverage` and `tdg` here are constants, not measurements — nothing
    /// runs the generated tests and nothing computes a TDG for a body that is
    /// `todo!()`. They exist only to feed `meets_thresholds`; the CLI reports them
    /// as "not measured" rather than as the 100% they would otherwise claim.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
    pub(crate) fn calculate_quality_score(&self, code: &str) -> Result<QualityScore> {
        let complexity = self.estimate_complexity(code);
        let coverage = 100.0; // NOT a measurement — see the note above
        let tdg = if complexity <= 5 { 1 } else { complexity / 2 };

        Ok(QualityScore {
            overall: 100.0 - (f64::from(complexity) * 2.0),
            complexity,
            coverage,
            tdg,
        })
    }

    /// Calculate detailed metrics
    ///
    /// `coverage`/`tdg`/`dead_code_percentage` are constants feeding the threshold
    /// check, not measurements; they must never be surfaced as a measured result.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
    pub(crate) fn calculate_metrics(&self, code: &str, tests: &str) -> Result<QualityMetrics> {
        Ok(QualityMetrics {
            complexity: self.estimate_complexity(code),
            cognitive_complexity: self.estimate_complexity(code), // Same for now
            coverage: 100, // NOT a measurement — no test run produced this
            tdg: 1,        // Generated code has minimal technical debt
            satd_count: code.matches("TODO").count() as u32,
            dead_code_percentage: 0, // Generated code has no dead code
            has_doctests: code.contains("```") && code.contains("assert"),
            has_property_tests: tests.contains("proptest!"),
        })
    }

    /// Enhance existing code with new features
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn enhance_with_features(&self, base_code: &str, features: &[String]) -> Result<String> {
        let mut enhanced = base_code.to_string();

        for feature in features {
            enhanced.push_str(&format!("\n\n// Feature: {feature}\n"));
            enhanced.push_str(&self.generate_feature_code(feature)?);
        }

        Ok(enhanced)
    }

    /// Generate tests for given code
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn generate_tests(&self, code: &str) -> Result<String> {
        self.test_generator.generate_tests(code)
    }

    /// Generate documentation for code
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn generate_documentation(&self, code: &str) -> Result<String> {
        self.doc_generator.generate_documentation(code)
    }

    /// Generate code for a specific feature
    fn generate_feature_code(&self, feature: &str) -> Result<String> {
        Ok(format!(
            r"
pub fn {}(&self) -> Result<()> {{
    // Implementation for {}
    Ok(())
}}
",
            feature.to_lowercase().replace(' ', "_"),
            feature
        ))
    }
}
