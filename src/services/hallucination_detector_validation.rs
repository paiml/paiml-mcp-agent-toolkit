// HallucinationDetector and DocAccuracyValidator implementation: claim validation
// against codebase facts, contradiction detection, and end-to-end documentation checking.

fn check_capability_contradiction(claim: &Claim) -> Option<ValidationResult> {
    let has_compile_cap = claim.entities.iter().any(|e| {
        matches!(e, Entity::Capability(cap) if cap == "compile")
    });
    if has_compile_cap && !claim.is_negative {
        return Some(ValidationResult {
            claim: claim.clone(),
            status: ValidationStatus::Contradiction,
            evidence: Some(Evidence {
                source: "CodeFactDatabase".to_string(),
                similarity: 0.2,
                content: "PMAT analyzes code but does not compile it".to_string(),
            }),
            error_message: Some("PMAT does not compile code - analysis only".to_string()),
            confidence: 0.2,
        });
    }
    None
}

fn check_language_support(
    claim: &Claim,
    code_facts: &CodeFactDatabase,
) -> Option<ValidationResult> {
    for entity in &claim.entities {
        let Entity::Language(lang) = entity else {
            continue;
        };
        if claim.is_negative {
            continue;
        }
        if code_facts.has_language_support(lang) {
            return Some(ValidationResult {
                claim: claim.clone(),
                status: ValidationStatus::Verified,
                evidence: Some(Evidence {
                    source: "CodeFactDatabase".to_string(),
                    similarity: 0.95,
                    content: format!("{} language analysis supported", lang),
                }),
                error_message: None,
                confidence: 0.95,
            });
        }
        return Some(ValidationResult {
            claim: claim.clone(),
            status: ValidationStatus::Unverified,
            evidence: None,
            error_message: Some(format!("{} language support not found in codebase", lang)),
            confidence: 0.5,
        });
    }
    None
}

impl HallucinationDetector {
    /// Create new detector with code facts
    pub fn new(code_facts: CodeFactDatabase) -> Self {
        Self {
            code_facts,
            similarity: SemanticSimilarity::new(),
        }
    }

    /// Validate a claim against codebase
    pub fn validate_claim(&self, claim: &Claim) -> Result<ValidationResult> {
        if let Some(result) = check_capability_contradiction(claim) {
            return Ok(result);
        }
        if let Some(result) = check_language_support(claim, &self.code_facts) {
            return Ok(result);
        }
        Ok(ValidationResult {
            claim: claim.clone(),
            status: ValidationStatus::Inconclusive,
            evidence: None,
            error_message: Some("Insufficient evidence to validate claim".to_string()),
            confidence: 0.5,
        })
    }
}

impl DocAccuracyValidator {
    /// Create new validator with code facts
    pub fn new(code_facts: CodeFactDatabase) -> Self {
        Self {
            extractor: ClaimExtractor::new(),
            detector: HallucinationDetector::new(code_facts),
        }
    }

    /// Validate all claims in documentation
    pub fn validate_documentation(
        &self,
        content: &str,
        filename: &str,
    ) -> Result<Vec<ValidationResult>> {
        debug_assert!(!content.is_empty(), "content must not be empty");
        // Extract claims
        let mut claims = self.extractor.extract_claims(content);

        // Set source file for all claims
        for claim in &mut claims {
            claim.source_file = PathBuf::from(filename);
        }

        // Validate each claim
        let mut results = Vec::new();
        for claim in claims {
            let result = self.detector.validate_claim(&claim)?;
            results.push(result);
        }

        Ok(results)
    }

    /// Check if results contain any contradictions
    pub fn has_contradictions(&self, results: &[ValidationResult]) -> bool {
        debug_assert!(!results.is_empty(), "results must not be empty");
        results
            .iter()
            .any(|r| r.status == ValidationStatus::Contradiction)
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claim_extractor_basic() {
        let extractor = ClaimExtractor::new();
        let doc = "PMAT can analyze Rust code complexity.";
        let claims = extractor.extract_claims(doc);

        assert_eq!(claims.len(), 1);
        assert_eq!(claims[0].claim_type, ClaimType::Capability);
        assert!(!claims[0].is_negative);
    }

    #[test]
    fn test_claim_extractor_negative() {
        let extractor = ClaimExtractor::new();
        let doc = "PMAT cannot compile code.";
        let claims = extractor.extract_claims(doc);

        assert_eq!(claims.len(), 1);
        assert!(claims[0].is_negative);
    }

    #[test]
    fn test_code_fact_database_from_markdown() {
        let markdown = r#"
Functions:
- main()
- run_server()

Supported languages:
- Rust
- TypeScript
        "#;

        let db = CodeFactDatabase::from_markdown(markdown).expect("internal error");
        assert!(db.has_function("main"));
        assert!(db.has_function("run_server"));
        assert!(db.has_language_support("Rust"));
        assert!(db.has_language_support("TypeScript"));
    }

    #[test]
    fn test_semantic_similarity_high_overlap() {
        let sim = SemanticSimilarity::new();
        let score = sim.calculate(
            "PMAT can analyze Rust code",
            "Rust language analysis supported",
        );
        assert!(score > 0.3, "Expected high similarity, got {}", score);
    }

    #[test]
    fn test_semantic_similarity_low_overlap() {
        let sim = SemanticSimilarity::new();
        let score = sim.calculate(
            "PMAT can compile Rust",
            "PMAT analyzes code but does not compile",
        );
        assert!(score < 0.5, "Expected low similarity, got {}", score);
    }
}
