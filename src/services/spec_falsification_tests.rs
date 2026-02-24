#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_path_references() {
        let extractor = SpecClaimExtractor::new();
        let content = r#"
## Architecture
The main module is at `src/services/context.rs` which handles indexing.
Configuration lives in `docs/specifications/falsify-rag.md`.
"#;
        let claims = extractor.extract(content, Path::new("test.md"));
        let path_claims: Vec<_> = claims
            .iter()
            .filter(|c| c.category == SpecClaimCategory::PathReference)
            .collect();
        assert!(
            path_claims.len() >= 2,
            "Expected >=2 path claims, got {}: {:?}",
            path_claims.len(),
            path_claims
        );
        assert!(path_claims
            .iter()
            .any(|c| c.path_refs.iter().any(|p| p.contains("context.rs"))));
    }

    #[test]
    fn extract_rfc2119_priorities() {
        let extractor = SpecClaimExtractor::new();
        let content = r#"
## Requirements
- Implementations MUST validate all inputs before processing
- Clients SHOULD cache results for performance
- Servers MAY support optional compression
"#;
        let claims = extractor.extract(content, Path::new("test.md"));
        assert!(claims
            .iter()
            .any(|c| c.priority == ClaimPriority::P0Critical));
        assert!(claims.iter().any(|c| c.priority == ClaimPriority::P1High));
        assert!(claims.iter().any(|c| c.priority == ClaimPriority::P2Low));
    }

    #[test]
    fn extract_numeric_claims() {
        let extractor = SpecClaimExtractor::new();
        let content = "Coverage must be >= 95% across all modules.\n";
        let claims = extractor.extract(content, Path::new("test.md"));
        let metric_claims: Vec<_> = claims
            .iter()
            .filter(|c| {
                matches!(
                    c.category,
                    SpecClaimCategory::MetricClaim
                        | SpecClaimCategory::AbsenceClaim
                        | SpecClaimCategory::ArchitecturalClaim
                )
            })
            .collect();
        // Should find a claim with numeric value
        let has_numeric = claims.iter().any(|c| c.numeric_value.is_some());
        assert!(
            has_numeric,
            "Expected numeric claim, got: {:?}",
            metric_claims
        );
    }

    #[test]
    fn extract_code_entities() {
        let extractor = SpecClaimExtractor::new();
        let content = "The `FalsificationEngine` processes claims via `ClaimExtractor`.\n";
        let claims = extractor.extract(content, Path::new("test.md"));
        let entity_claims: Vec<_> = claims
            .iter()
            .filter(|c| c.category == SpecClaimCategory::CodeEntity)
            .collect();
        assert!(
            !entity_claims.is_empty(),
            "Expected entity claims, got none"
        );
        assert!(entity_claims
            .iter()
            .any(|c| c.entity_refs.contains(&"FalsificationEngine".to_string())));
    }

    #[test]
    fn extract_absence_claims() {
        let extractor = SpecClaimExtractor::new();
        let content = "There must be zero unsafe blocks in the parser module.\n";
        let claims = extractor.extract(content, Path::new("test.md"));
        let absence = claims
            .iter()
            .filter(|c| c.category == SpecClaimCategory::AbsenceClaim)
            .count();
        assert!(absence > 0, "Expected absence claim, got: {:?}", claims);
    }

    #[test]
    fn extract_command_claims() {
        let extractor = SpecClaimExtractor::new();
        let content = "Run `pmat falsify` to validate specs against the codebase.\n";
        let claims = extractor.extract(content, Path::new("test.md"));
        let cmd_claims: Vec<_> = claims
            .iter()
            .filter(|c| c.category == SpecClaimCategory::CommandClaim)
            .collect();
        assert!(!cmd_claims.is_empty(), "Expected command claims");
    }

    #[test]
    fn skip_code_blocks() {
        let extractor = SpecClaimExtractor::new();
        let content = r#"
## Example
```rust
// This MUST not be extracted as a claim
let x = src/foo/bar.rs;
```
This line SHOULD be extracted.
"#;
        let claims = extractor.extract(content, Path::new("test.md"));
        // Only the "SHOULD" line should be extracted, not the code block contents
        assert!(
            claims.iter().all(|c| !c.original_text.contains("let x =")),
            "Code block content should not be extracted as claims"
        );
        assert!(claims.iter().any(|c| c.original_text.contains("SHOULD")));
    }

    #[test]
    fn absolute_language_detection() {
        let extractor = SpecClaimExtractor::new();
        let content = "All modules MUST have complete test coverage.\n";
        let claims = extractor.extract(content, Path::new("test.md"));
        assert!(!claims.is_empty());
        assert!(claims[0].is_absolute);
        assert_eq!(claims[0].priority, ClaimPriority::P0Critical);
    }

    #[test]
    fn path_reference_validation_existing_file() {
        let engine = FalsificationEngine::new(Path::new(env!("CARGO_MANIFEST_DIR")));
        let claim = SpecClaim {
            id: "test-001".to_string(),
            original_text: "Config at src/lib.rs".to_string(),
            source_line: 1,
            category: SpecClaimCategory::PathReference,
            priority: ClaimPriority::P3Default,
            is_absolute: false,
            path_refs: vec!["src/lib.rs".to_string()],
            entity_refs: vec![],
            numeric_value: None,
            numeric_comparator: None,
        };
        let evidence = engine.check_path_references(&claim);
        assert!(!evidence.is_empty());
        assert_eq!(
            evidence[0].contradiction_score, 0.0,
            "src/lib.rs should exist"
        );
    }

    #[test]
    fn path_reference_validation_missing_file() {
        let engine = FalsificationEngine::new(Path::new(env!("CARGO_MANIFEST_DIR")));
        let claim = SpecClaim {
            id: "test-002".to_string(),
            original_text: "Config at src/nonexistent_file_xyz.rs".to_string(),
            source_line: 1,
            category: SpecClaimCategory::PathReference,
            priority: ClaimPriority::P3Default,
            is_absolute: false,
            path_refs: vec!["src/nonexistent_file_xyz.rs".to_string()],
            entity_refs: vec![],
            numeric_value: None,
            numeric_comparator: None,
        };
        let evidence = engine.check_path_references(&claim);
        assert!(!evidence.is_empty());
        assert_eq!(
            evidence[0].contradiction_score, 1.0,
            "Nonexistent file should be falsified"
        );
    }

    #[test]
    fn verdict_determination() {
        let engine = FalsificationEngine::new(Path::new("."));
        let claim = SpecClaim {
            id: "test".to_string(),
            original_text: "test".to_string(),
            source_line: 1,
            category: SpecClaimCategory::PathReference,
            priority: ClaimPriority::P3Default,
            is_absolute: false,
            path_refs: vec![],
            entity_refs: vec![],
            numeric_value: None,
            numeric_comparator: None,
        };

        // Surviving evidence
        let survived_ev = vec![SpecEvidence {
            check: "test".to_string(),
            finding: "ok".to_string(),
            contradiction_score: 0.0,
        }];
        assert_eq!(
            engine.determine_verdict(&claim, &survived_ev),
            VerdictStatus::Survived
        );

        // Falsified evidence
        let falsified_ev = vec![SpecEvidence {
            check: "test".to_string(),
            finding: "bad".to_string(),
            contradiction_score: 1.0,
        }];
        assert_eq!(
            engine.determine_verdict(&claim, &falsified_ev),
            VerdictStatus::Falsified
        );
    }

    #[test]
    fn summary_computation() {
        let claim = SpecClaim {
            id: "c1".to_string(),
            original_text: "test".to_string(),
            source_line: 1,
            category: SpecClaimCategory::PathReference,
            priority: ClaimPriority::P3Default,
            is_absolute: false,
            path_refs: vec![],
            entity_refs: vec![],
            numeric_value: None,
            numeric_comparator: None,
        };

        let verdicts = vec![
            SpecVerdict {
                claim: claim.clone(),
                status: VerdictStatus::Survived,
                evidence: vec![],
                contradiction_score: 0.0,
            },
            SpecVerdict {
                claim: claim.clone(),
                status: VerdictStatus::Falsified,
                evidence: vec![],
                contradiction_score: 1.0,
            },
            SpecVerdict {
                claim: claim.clone(),
                status: VerdictStatus::Unfalsifiable,
                evidence: vec![],
                contradiction_score: 0.0,
            },
        ];

        let summary = FalsificationEngine::compute_summary(&verdicts);
        assert_eq!(summary.total_claims, 3);
        assert_eq!(summary.survived, 1);
        assert_eq!(summary.falsified, 1);
        assert_eq!(summary.unfalsifiable, 1);
        // health = 1 survived / 2 testable = 0.5
        assert!((summary.health_score - 0.5).abs() < f64::EPSILON);
    }
}
