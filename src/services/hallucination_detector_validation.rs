// HallucinationDetector and DocAccuracyValidator implementation: claim validation
// against codebase facts, contradiction detection, and end-to-end documentation checking.

/// Resolve a documentation-relative path against the places a reader would
/// look: the working directory, the document's own directory, and each of its
/// ancestors (docs cite repo-root-relative paths from nested files). Returns
/// the first location that exists.
fn resolve_documented_path(doc: &std::path::Path, referenced: &str) -> Option<PathBuf> {
    let mut roots: Vec<PathBuf> = vec![PathBuf::from(".")];
    let mut cursor = doc.parent();
    while let Some(dir) = cursor {
        roots.push(dir.to_path_buf());
        if roots.len() >= 8 {
            break;
        }
        cursor = dir.parent();
    }
    roots
        .into_iter()
        .map(|r| r.join(referenced))
        .find(|p| p.exists())
}

/// A documented file that does not exist is a contradiction: the filesystem is
/// ground truth, so this check can genuinely fail.
fn check_file_reference(claim: &Claim) -> Option<ValidationResult> {
    for entity in &claim.entities {
        let Entity::File(path) = entity else {
            continue;
        };
        return Some(match resolve_documented_path(&claim.source_file, path) {
            Some(found) => ValidationResult {
                claim: claim.clone(),
                status: ValidationStatus::Verified,
                evidence: Some(Evidence {
                    source: "filesystem".to_string(),
                    similarity: 1.0,
                    content: format!("file exists: {}", found.display()),
                }),
                error_message: None,
                confidence: 1.0,
            },
            None => ValidationResult {
                claim: claim.clone(),
                status: ValidationStatus::Contradiction,
                evidence: Some(Evidence {
                    source: "filesystem".to_string(),
                    similarity: 0.0,
                    content: format!("no file at `{path}` relative to the document or repo root"),
                }),
                error_message: Some(format!("documented path does not exist: {path}")),
                confidence: 1.0,
            },
        });
    }
    None
}

/// A documented function is checked against the deep-context fact database.
/// Absence there is only *unverified*, not a contradiction — the fact database
/// is a partial index. When it holds no functions at all there is no ground
/// truth to check against, and the claim is reported inconclusive rather than
/// silently passing.
fn check_function_reference(
    claim: &Claim,
    code_facts: &CodeFactDatabase,
) -> Option<ValidationResult> {
    for entity in &claim.entities {
        let Entity::Function(name) = entity else {
            continue;
        };
        if code_facts.has_function(name) {
            return Some(ValidationResult {
                claim: claim.clone(),
                status: ValidationStatus::Verified,
                evidence: Some(Evidence {
                    source: "CodeFactDatabase".to_string(),
                    similarity: 0.95,
                    content: format!("{name}() present in deep context"),
                }),
                error_message: None,
                confidence: 0.95,
            });
        }
        if code_facts.function_count() == 0 {
            return Some(ValidationResult {
                claim: claim.clone(),
                status: ValidationStatus::Inconclusive,
                evidence: None,
                error_message: Some(
                    "NOT MEASURED: deep context lists no functions, so function references \
                     cannot be checked"
                        .to_string(),
                ),
                confidence: 0.0,
            });
        }
        return Some(ValidationResult {
            claim: claim.clone(),
            status: ValidationStatus::Unverified,
            evidence: None,
            error_message: Some(format!("{name}() not found in codebase facts")),
            confidence: 0.3,
        });
    }
    None
}

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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(code_facts: CodeFactDatabase) -> Self {
        Self {
            code_facts,
            similarity: SemanticSimilarity::new(),
        }
    }

    /// Validate a claim against codebase
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn validate_claim(&self, claim: &Claim) -> Result<ValidationResult> {
        if let Some(result) = check_file_reference(claim) {
            return Ok(result);
        }
        if let Some(result) = check_function_reference(claim, &self.code_facts) {
            return Ok(result);
        }
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(code_facts: CodeFactDatabase) -> Self {
        Self {
            extractor: ClaimExtractor::new(),
            detector: HallucinationDetector::new(code_facts),
        }
    }

    /// Validate all claims in documentation
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn validate_documentation(
        &self,
        content: &str,
        filename: &str,
    ) -> Result<Vec<ValidationResult>> {
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn has_contradictions(&self, results: &[ValidationResult]) -> bool {
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

    // ── REGRESSION: the validator used to extract 0 claims from real docs and
    //    report "All documentation claims are verified!" for anything. ──

    fn facts_with_functions() -> CodeFactDatabase {
        CodeFactDatabase::from_markdown("Functions:\n- main()\n- run_server()\n")
            .expect("internal error")
    }

    #[test]
    fn extracts_claims_from_realistic_documentation() {
        let extractor = ClaimExtractor::new();
        let doc = "The dispatcher lives in `src/cli/mod.rs` and calls `run_server()`.\n\
                   See docs/specifications/example.md for the design.\n";
        let claims = extractor.extract_claims(doc);
        // PIN: prose that cites files and functions yields claims. Zero claims
        // from real documentation was the defect.
        assert!(
            claims.len() >= 3,
            "expected file + function claims, got {claims:?}"
        );
        assert!(claims
            .iter()
            .any(|c| c.entities.iter().any(|e| matches!(e, Entity::File(_)))));
        assert!(claims
            .iter()
            .any(|c| c.entities.iter().any(|e| matches!(e, Entity::Function(_)))));
    }

    #[test]
    fn fabricated_file_reference_is_a_contradiction() {
        let detector = HallucinationDetector::new(facts_with_functions());
        let claims = ClaimExtractor::new()
            .extract_claims("The entry point is `src/totally/made/up/nonexistent_module.rs`.");
        assert_eq!(claims.len(), 1, "one file claim expected: {claims:?}");
        let mut claim = claims.into_iter().next().expect("internal error");
        claim.source_file = PathBuf::from("BAD.md");

        let result = detector.validate_claim(&claim).expect("internal error");
        // PIN: a documented path that does not exist must fail, not pass.
        assert_eq!(result.status, ValidationStatus::Contradiction);
    }

    #[test]
    fn existing_file_reference_is_verified() {
        let dir = tempfile::tempdir().expect("internal error");
        std::fs::create_dir_all(dir.path().join("src")).expect("internal error");
        std::fs::write(dir.path().join("src/real.rs"), "fn x() {}").expect("internal error");

        let detector = HallucinationDetector::new(facts_with_functions());
        let claims = ClaimExtractor::new().extract_claims("Defined in `src/real.rs` today.");
        let mut claim = claims.into_iter().next().expect("internal error");
        // Paths are resolved relative to the document's own directory.
        claim.source_file = dir.path().join("README.md");

        let result = detector.validate_claim(&claim).expect("internal error");
        assert_eq!(result.status, ValidationStatus::Verified);
    }

    #[test]
    fn unknown_function_reference_is_not_verified() {
        let detector = HallucinationDetector::new(facts_with_functions());
        let claims = ClaimExtractor::new().extract_claims("Wired by `frobnicate_the_widget()`.");
        let claim = claims.into_iter().next().expect("internal error");
        let result = detector.validate_claim(&claim).expect("internal error");
        assert_eq!(result.status, ValidationStatus::Unverified);
        assert_ne!(result.status, ValidationStatus::Verified);
    }

    #[test]
    fn function_reference_without_ground_truth_is_inconclusive() {
        // PIN: an empty fact database cannot verify OR refute — it must say so
        // rather than letting the claim through as verified.
        let detector = HallucinationDetector::new(CodeFactDatabase::new());
        let claims = ClaimExtractor::new().extract_claims("Wired by `run_server()`.");
        let claim = claims.into_iter().next().expect("internal error");
        let result = detector.validate_claim(&claim).expect("internal error");
        assert_eq!(result.status, ValidationStatus::Inconclusive);
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
