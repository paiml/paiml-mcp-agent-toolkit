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
        let survived_ev = vec![SpecEvidence::supports("test", "ok")];
        assert_eq!(
            engine.determine_verdict(&claim, &survived_ev),
            VerdictStatus::Survived
        );

        // Falsified evidence
        let falsified_ev = vec![SpecEvidence::contradicts_with("test", "bad")];
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

    fn make_claim(
        id: &str,
        original_text: &str,
        category: SpecClaimCategory,
    ) -> SpecClaim {
        SpecClaim {
            id: id.to_string(),
            original_text: original_text.to_string(),
            source_line: 1,
            category,
            priority: ClaimPriority::P3Default,
            is_absolute: false,
            path_refs: vec![],
            entity_refs: vec![],
            numeric_value: None,
            numeric_comparator: None,
        }
    }

    // ── check_metric_claim: refuses explicitly, never certifies ──

    #[test]
    fn check_metric_claim_returns_unmeasured_evidence() {
        let engine = FalsificationEngine::new(Path::new("."));
        let claim = make_claim("m1", "coverage >= 95%", SpecClaimCategory::MetricClaim);
        let evidence = engine.check_metric_claim(&claim);
        assert_eq!(
            evidence.len(),
            1,
            "metric claims always return one evidence entry"
        );
        // PIN: pmat does not measure spec metrics. The evidence must say so
        // out loud rather than scoring 0.0 as if the check had passed.
        assert!(
            !evidence[0].measured,
            "an unrun metric check must be flagged unmeasured"
        );
        assert!(
            evidence[0].finding.contains("NOT MEASURED"),
            "refusal must be explicit, got: {}",
            evidence[0].finding
        );
        assert!(
            !evidence[0].contradicts(),
            "unmeasured evidence is not a contradiction either"
        );
    }

    /// REGRESSION (blocker: `pmat falsify` certified impossible metric claims).
    ///
    /// `check_metric_claim` was a stub returning contradiction_score 0.0, which
    /// `determine_verdict` mapped to SURVIVED — so every MetricClaim in every
    /// spec, including deliberately impossible ones, reported a green verdict.
    #[test]
    fn metric_claim_can_never_report_survived() {
        let engine = FalsificationEngine::new(Path::new("."));
        for text in [
            "The system MUST maintain test coverage >= 99.9% at all times.",
            "Every function MUST have cyclomatic complexity <= 1.",
            "The binary MUST start in < 0 ms.",
        ] {
            let claim = make_claim("m", text, SpecClaimCategory::MetricClaim);
            let evidence = engine.check_metric_claim(&claim);
            let verdict = engine.determine_verdict(&claim, &evidence);
            assert_eq!(
                verdict,
                VerdictStatus::Inconclusive,
                "unmeasured metric claim must not survive: {text}"
            );
            assert_ne!(verdict, VerdictStatus::Survived);
        }
    }

    /// A check that could not run (pmat missing, unparseable command) used to
    /// score 0.0 and therefore SURVIVE. Unmeasured must never certify.
    #[test]
    fn unmeasured_evidence_never_yields_survived() {
        let engine = FalsificationEngine::new(Path::new("."));
        let claim = make_claim("u", "t", SpecClaimCategory::CodeEntity);

        let unmeasured = vec![SpecEvidence::unmeasured("x", "NOT MEASURED: tool missing")];
        assert_eq!(
            engine.determine_verdict(&claim, &unmeasured),
            VerdictStatus::Inconclusive
        );

        // Even mixed with a real pass, one skipped check blocks SURVIVED.
        let mixed = vec![
            SpecEvidence::supports("a", "found"),
            SpecEvidence::unmeasured("b", "NOT MEASURED: tool missing"),
        ];
        assert_eq!(
            engine.determine_verdict(&claim, &mixed),
            VerdictStatus::Inconclusive
        );

        // A measured contradiction still wins over a skipped sibling check.
        let contradicted = vec![
            SpecEvidence::contradicts_with("a", "missing"),
            SpecEvidence::unmeasured("b", "NOT MEASURED: tool missing"),
        ];
        assert_eq!(
            engine.determine_verdict(&claim, &contradicted),
            VerdictStatus::Falsified
        );
    }

    /// The whole-spec report for a metric-only spec must not read as healthy.
    #[test]
    fn impossible_metric_spec_reports_zero_health() {
        let dir = tempfile::tempdir().unwrap();
        let spec = dir.path().join("impossible.md");
        std::fs::write(
            &spec,
            "# Impossible\n\nThe system MUST maintain test coverage >= 99.9% at all times.\n\
             The project MUST contain >= 100000000 lines of Rust.\n",
        )
        .unwrap();

        let engine = FalsificationEngine::new(dir.path());
        let report = engine.falsify_spec(&spec).unwrap();

        assert!(report.summary.total_claims >= 2, "claims must be extracted");
        assert_eq!(
            report.summary.survived, 0,
            "no impossible metric claim may survive"
        );
        assert_eq!(report.summary.inconclusive, report.summary.total_claims);
        assert_eq!(
            report.summary.health_score, 0.0,
            "an entirely unmeasured spec is not 100% healthy"
        );
    }

    // ── determine_verdict: unfalsifiable + inconclusive + boundary arms ──

    #[test]
    fn determine_verdict_unfalsifiable_category_short_circuits() {
        let engine = FalsificationEngine::new(Path::new("."));
        let claim = make_claim("u1", "t", SpecClaimCategory::Unfalsifiable);
        // Even a falsified-looking evidence vector is ignored for this category.
        let strong_ev = vec![SpecEvidence::contradicts_with("x", "y")];
        assert_eq!(
            engine.determine_verdict(&claim, &strong_ev),
            VerdictStatus::Unfalsifiable
        );
    }

    #[test]
    fn determine_verdict_architectural_claim_also_unfalsifiable() {
        let engine = FalsificationEngine::new(Path::new("."));
        let claim = make_claim("u2", "t", SpecClaimCategory::ArchitecturalClaim);
        assert_eq!(
            engine.determine_verdict(&claim, &[]),
            VerdictStatus::Unfalsifiable
        );
    }

    #[test]
    fn determine_verdict_empty_evidence_is_inconclusive() {
        let engine = FalsificationEngine::new(Path::new("."));
        let claim = make_claim("e1", "t", SpecClaimCategory::PathReference);
        assert_eq!(
            engine.determine_verdict(&claim, &[]),
            VerdictStatus::Inconclusive
        );
    }

    #[test]
    fn determine_verdict_contradiction_between_0_4_and_0_8_is_inconclusive() {
        let engine = FalsificationEngine::new(Path::new("."));
        let claim = make_claim("m1", "t", SpecClaimCategory::PathReference);
        let ev = vec![SpecEvidence::measured("x", "y", 0.5)];
        assert_eq!(
            engine.determine_verdict(&claim, &ev),
            VerdictStatus::Inconclusive
        );
    }

    // ── compute_summary health_score edge cases ──

    #[test]
    fn compute_summary_empty_verdicts_yields_perfect_health() {
        let summary = FalsificationEngine::compute_summary(&[]);
        assert_eq!(summary.total_claims, 0);
        // No testable claims → health_score defaults to 1.0 (perfect).
        assert!((summary.health_score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn compute_summary_all_unfalsifiable_yields_perfect_health() {
        let c = make_claim("u", "t", SpecClaimCategory::Unfalsifiable);
        let verdicts = vec![
            SpecVerdict {
                claim: c.clone(),
                status: VerdictStatus::Unfalsifiable,
                evidence: vec![],
                contradiction_score: 0.0,
            },
            SpecVerdict {
                claim: c,
                status: VerdictStatus::Unfalsifiable,
                evidence: vec![],
                contradiction_score: 0.0,
            },
        ];
        let s = FalsificationEngine::compute_summary(&verdicts);
        assert_eq!(s.unfalsifiable, 2);
        // 2 - 2 = 0 testable → defaults to 1.0.
        assert!((s.health_score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn compute_summary_counts_inconclusive() {
        let c = make_claim("i", "t", SpecClaimCategory::PathReference);
        let verdicts = vec![
            SpecVerdict {
                claim: c.clone(),
                status: VerdictStatus::Inconclusive,
                evidence: vec![],
                contradiction_score: 0.5,
            },
            SpecVerdict {
                claim: c,
                status: VerdictStatus::Survived,
                evidence: vec![],
                contradiction_score: 0.0,
            },
        ];
        let s = FalsificationEngine::compute_summary(&verdicts);
        assert_eq!(s.inconclusive, 1);
        assert_eq!(s.survived, 1);
        // testable = 2 (0 unfalsifiable); survived/testable = 0.5.
        assert!((s.health_score - 0.5).abs() < f64::EPSILON);
    }
}
