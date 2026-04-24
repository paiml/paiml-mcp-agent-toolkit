//! Formatting functions for proof annotations to reduce complexity

#![cfg_attr(coverage_nightly, coverage(off))]

use crate::models::unified_ast::{Location, ProofAnnotation};
use anyhow::Result;
use std::collections::HashMap;
use std::fmt::Write;

/// Format confidence level statistics
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_confidence_stats(
    annotations: &[(Location, ProofAnnotation)],
    output: &mut String,
) -> Result<()> {
    let mut confidence_counts = HashMap::new();
    for (_, ann) in annotations {
        let key = format!("{:?}", ann.confidence_level);
        *confidence_counts.entry(key).or_insert(0) += 1;
    }

    if !confidence_counts.is_empty() {
        writeln!(output, "\n## Confidence Levels\n")?;
        for (level, count) in confidence_counts {
            writeln!(output, "- {level}: {count} proofs")?;
        }
    }
    Ok(())
}

/// Format verification method statistics
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_method_stats(
    annotations: &[(Location, ProofAnnotation)],
    output: &mut String,
) -> Result<()> {
    use crate::models::unified_ast::VerificationMethod;

    let mut method_counts = HashMap::new();
    for (_, ann) in annotations {
        let key = match &ann.method {
            VerificationMethod::FormalProof { .. } => "Formal Proof",
            VerificationMethod::ModelChecking { .. } => "Model Checking",
            VerificationMethod::StaticAnalysis { .. } => "Static Analysis",
            VerificationMethod::AbstractInterpretation => "Abstract Interpretation",
            VerificationMethod::BorrowChecker => "Borrow Checker",
        };
        *method_counts.entry(key).or_insert(0) += 1;
    }

    if !method_counts.is_empty() {
        writeln!(output, "\n## Verification Methods\n")?;
        for (method, count) in method_counts {
            writeln!(output, "- {method}: {count} proofs")?;
        }
    }
    Ok(())
}

/// Format property type statistics
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_property_stats(
    annotations: &[(Location, ProofAnnotation)],
    output: &mut String,
) -> Result<()> {
    let mut property_counts = HashMap::new();
    for (_, ann) in annotations {
        let key = format!("{:?}", ann.property_proven);
        *property_counts.entry(key).or_insert(0) += 1;
    }

    if !property_counts.is_empty() {
        writeln!(output, "## Properties Proven\n")?;
        for (prop, count) in property_counts {
            writeln!(output, "- {prop}: {count} proofs")?;
        }
    }
    Ok(())
}

/// Group annotations by file
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn group_by_file(
    annotations: &[(Location, ProofAnnotation)],
) -> HashMap<std::path::PathBuf, Vec<(Location, ProofAnnotation)>> {
    let mut proofs_by_file = HashMap::new();
    for (loc, ann) in annotations {
        proofs_by_file
            .entry(loc.file_path.clone())
            .or_insert_with(Vec::new)
            .push((loc.clone(), ann.clone()));
    }

    // Sort each file's proofs by line number
    for proofs in proofs_by_file.values_mut() {
        proofs.sort_by_key(|(loc, _)| loc.span.start.0);
    }

    proofs_by_file
}

/// Format a single proof annotation
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_single_proof(
    location: &Location,
    annotation: &ProofAnnotation,
    output: &mut String,
    include_evidence: bool,
) -> Result<()> {
    format_proof_header(location, output)?;
    format_proof_metadata(annotation, output)?;
    format_proof_assumptions(&annotation.assumptions, output)?;

    if include_evidence {
        format_proof_evidence(annotation, output)?;
    }

    writeln!(output)?;
    Ok(())
}

fn format_proof_header(location: &Location, output: &mut String) -> Result<()> {
    writeln!(
        output,
        "### Position {}-{}\n",
        location.span.start.0, location.span.end.0
    )?;
    Ok(())
}

fn format_proof_metadata(annotation: &ProofAnnotation, output: &mut String) -> Result<()> {
    writeln!(output, "**Property**: {:?}", annotation.property_proven)?;
    writeln!(output, "**Method**: {:?}", annotation.method)?;
    writeln!(
        output,
        "**Tool**: {} v{}",
        annotation.tool_name, annotation.tool_version
    )?;
    writeln!(output, "**Confidence**: {:?}", annotation.confidence_level)?;
    writeln!(
        output,
        "**Verified**: {}",
        annotation.date_verified.format("%Y-%m-%d %H:%M:%S UTC")
    )?;
    Ok(())
}

fn format_proof_assumptions(assumptions: &[String], output: &mut String) -> Result<()> {
    if !assumptions.is_empty() {
        writeln!(output, "\n**Assumptions**:")?;
        for assumption in assumptions {
            writeln!(output, "- {assumption}")?;
        }
    }
    Ok(())
}

fn format_proof_evidence(annotation: &ProofAnnotation, output: &mut String) -> Result<()> {
    writeln!(output, "\n**Evidence**: {:?}", annotation.evidence_type)?;
    if let Some(ref spec_id) = annotation.specification_id {
        writeln!(output, "**Specification ID**: {spec_id}")?;
    }
    Ok(())
}

/// Format provability-specific output
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_provability_summary(
    summaries: &[crate::services::lightweight_provability_analyzer::ProofSummary],
    output: &mut String,
    _include_evidence: bool,
) -> Result<()> {
    let total_functions = summaries.len();
    let high_provability = summaries
        .iter()
        .filter(|s| s.provability_score >= 0.8)
        .count();
    let medium_provability = summaries
        .iter()
        .filter(|s| s.provability_score >= 0.5 && s.provability_score < 0.8)
        .count();
    let low_provability = summaries
        .iter()
        .filter(|s| s.provability_score < 0.5)
        .count();

    writeln!(output, "## Provability Analysis Summary\n")?;
    writeln!(output, "**Total Functions**: {total_functions}")?;
    writeln!(
        output,
        "**High Provability (≥80%)**: {} ({:.1}%)",
        high_provability,
        (high_provability as f64 / total_functions as f64) * 100.0
    )?;
    writeln!(
        output,
        "**Medium Provability (50-79%)**: {} ({:.1}%)",
        medium_provability,
        (medium_provability as f64 / total_functions as f64) * 100.0
    )?;
    writeln!(
        output,
        "**Low Provability (<50%)**: {} ({:.1}%)",
        low_provability,
        (low_provability as f64 / total_functions as f64) * 100.0
    )?;

    let avg_score =
        summaries.iter().map(|s| s.provability_score).sum::<f64>() / total_functions as f64;
    writeln!(output, "**Average Score**: {:.1}%\n", avg_score * 100.0)?;

    Ok(())
}

/// Generate SARIF rules for proof annotations
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn generate_proof_sarif_rules() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "id": "low-confidence-proof",
            "name": "Low Confidence Proof",
            "shortDescription": {
                "text": "Property verification has low confidence"
            },
            "fullDescription": {
                "text": "The verification method used has low confidence in the proof"
            },
            "defaultConfiguration": {
                "level": "warning"
            }
        }),
        serde_json::json!({
            "id": "medium-confidence-proof",
            "name": "Medium Confidence Proof",
            "shortDescription": {
                "text": "Property verification has medium confidence"
            },
            "fullDescription": {
                "text": "The verification method used has medium confidence in the proof"
            },
            "defaultConfiguration": {
                "level": "note"
            }
        }),
        serde_json::json!({
            "id": "high-confidence-proof",
            "name": "High Confidence Proof",
            "shortDescription": {
                "text": "Property verification has high confidence"
            },
            "fullDescription": {
                "text": "The verification method used has high confidence in the proof"
            },
            "defaultConfiguration": {
                "level": "none"
            }
        }),
        serde_json::json!({
            "id": "unverified-property",
            "name": "Unverified Safety Property",
            "shortDescription": {
                "text": "Critical safety property could not be verified"
            },
            "fullDescription": {
                "text": "Important properties like memory safety or null safety could not be formally verified"
            },
            "defaultConfiguration": {
                "level": "note"
            }
        }),
    ]
}

// Tests split for file health compliance (CB-040)
#[cfg(test)]
#[path = "property_tests.rs"]
mod property_tests;

// BROKEN: coverage_tests_part1.rs truncated mid-expression at line 400
#[cfg(all(test, feature = "broken-tests"))]
#[path = "coverage_tests.rs"]
mod coverage_tests;

#[cfg(test)]
mod proof_annotation_formatter_tests {
    //! Covers proof_annotation_formatter/mod.rs (37 uncov on broad, 0% cov).
    use super::*;
    use crate::models::unified_ast::{
        BytePos, ConfidenceLevel, EvidenceType, Location, ProofAnnotation, PropertyType, Span,
        VerificationMethod,
    };
    use chrono::Utc;
    use std::path::PathBuf;
    use uuid::Uuid;

    fn loc() -> Location {
        Location {
            file_path: PathBuf::from("src/a.rs"),
            span: Span {
                start: BytePos(10),
                end: BytePos(50),
            },
        }
    }

    fn ann_with(
        confidence: ConfidenceLevel,
        method: VerificationMethod,
        property: PropertyType,
        assumptions: Vec<String>,
    ) -> ProofAnnotation {
        ProofAnnotation {
            annotation_id: Uuid::new_v4(),
            property_proven: property,
            specification_id: Some("spec-001".into()),
            method,
            tool_name: "test-tool".into(),
            tool_version: "1.0".into(),
            confidence_level: confidence,
            assumptions,
            evidence_type: EvidenceType::ImplicitTypeSystemGuarantee,
            evidence_location: None,
            date_verified: Utc::now(),
        }
    }

    fn high_conf_ann() -> ProofAnnotation {
        ann_with(
            ConfidenceLevel::High,
            VerificationMethod::BorrowChecker,
            PropertyType::MemorySafety,
            vec![],
        )
    }

    // ── format_confidence_stats ──

    #[test]
    fn test_format_confidence_stats_empty_writes_nothing() {
        let mut out = String::new();
        format_confidence_stats(&[], &mut out).unwrap();
        assert!(out.is_empty());
    }

    #[test]
    fn test_format_confidence_stats_groups_by_confidence() {
        let anns = vec![
            (
                loc(),
                ann_with(
                    ConfidenceLevel::High,
                    VerificationMethod::BorrowChecker,
                    PropertyType::MemorySafety,
                    vec![],
                ),
            ),
            (
                loc(),
                ann_with(
                    ConfidenceLevel::High,
                    VerificationMethod::BorrowChecker,
                    PropertyType::MemorySafety,
                    vec![],
                ),
            ),
            (
                loc(),
                ann_with(
                    ConfidenceLevel::Low,
                    VerificationMethod::BorrowChecker,
                    PropertyType::MemorySafety,
                    vec![],
                ),
            ),
        ];
        let mut out = String::new();
        format_confidence_stats(&anns, &mut out).unwrap();
        assert!(out.contains("Confidence Levels"));
        assert!(out.contains("High: 2"));
        assert!(out.contains("Low: 1"));
    }

    // ── format_method_stats ──

    #[test]
    fn test_format_method_stats_empty_writes_nothing() {
        let mut out = String::new();
        format_method_stats(&[], &mut out).unwrap();
        assert!(out.is_empty());
    }

    #[test]
    fn test_format_method_stats_groups_by_method() {
        let anns = vec![
            (
                loc(),
                ann_with(
                    ConfidenceLevel::High,
                    VerificationMethod::BorrowChecker,
                    PropertyType::MemorySafety,
                    vec![],
                ),
            ),
            (
                loc(),
                ann_with(
                    ConfidenceLevel::High,
                    VerificationMethod::AbstractInterpretation,
                    PropertyType::MemorySafety,
                    vec![],
                ),
            ),
        ];
        let mut out = String::new();
        format_method_stats(&anns, &mut out).unwrap();
        assert!(!out.is_empty());
    }

    // ── format_property_stats ──

    #[test]
    fn test_format_property_stats_empty_writes_nothing() {
        let mut out = String::new();
        format_property_stats(&[], &mut out).unwrap();
        assert!(out.is_empty());
    }

    #[test]
    fn test_format_property_stats_with_anns_writes_section() {
        let anns = vec![(loc(), high_conf_ann())];
        let mut out = String::new();
        format_property_stats(&anns, &mut out).unwrap();
        assert!(!out.is_empty());
    }

    // ── group_by_file ──

    #[test]
    fn test_group_by_file_empty_returns_empty_map() {
        let anns: Vec<(Location, ProofAnnotation)> = vec![];
        let groups = group_by_file(&anns);
        assert!(groups.is_empty());
    }

    #[test]
    fn test_group_by_file_groups_by_path() {
        let mut l1 = loc();
        l1.file_path = PathBuf::from("src/a.rs");
        let mut l2 = loc();
        l2.file_path = PathBuf::from("src/b.rs");
        let anns = vec![
            (l1.clone(), high_conf_ann()),
            (l2.clone(), high_conf_ann()),
            (l1.clone(), high_conf_ann()),
        ];
        let groups = group_by_file(&anns);
        assert_eq!(groups.len(), 2);
        // src/a.rs should have 2 entries.
        let a_count = groups
            .get(&PathBuf::from("src/a.rs"))
            .map(|v| v.len())
            .unwrap_or(0);
        let b_count = groups
            .get(&PathBuf::from("src/b.rs"))
            .map(|v| v.len())
            .unwrap_or(0);
        assert_eq!(a_count, 2);
        assert_eq!(b_count, 1);
    }

    // ── format_single_proof ──

    #[test]
    fn test_format_single_proof_includes_position_and_metadata() {
        let mut out = String::new();
        format_single_proof(&loc(), &high_conf_ann(), &mut out, false).unwrap();
        assert!(out.contains("Position 10-50"));
        assert!(out.contains("Property"));
        assert!(out.contains("Method"));
        assert!(out.contains("Tool"));
        assert!(out.contains("Confidence"));
        assert!(out.contains("Verified"));
    }

    #[test]
    fn test_format_single_proof_with_assumptions_includes_them() {
        let ann = ann_with(
            ConfidenceLevel::High,
            VerificationMethod::BorrowChecker,
            PropertyType::MemorySafety,
            vec!["arr.len() > 0".into(), "no aliasing".into()],
        );
        let mut out = String::new();
        format_single_proof(&loc(), &ann, &mut out, false).unwrap();
        assert!(out.contains("Assumptions"));
        assert!(out.contains("arr.len() > 0"));
        assert!(out.contains("no aliasing"));
    }

    #[test]
    fn test_format_single_proof_with_evidence_includes_evidence_section() {
        let mut out = String::new();
        format_single_proof(&loc(), &high_conf_ann(), &mut out, true).unwrap();
        assert!(out.contains("Evidence"));
        // specification_id is Some("spec-001") in our fixture.
        assert!(out.contains("spec-001"));
    }

    #[test]
    fn test_format_single_proof_without_evidence_skips_section() {
        let mut out = String::new();
        format_single_proof(&loc(), &high_conf_ann(), &mut out, false).unwrap();
        // include_evidence=false → no Evidence section.
        assert!(!out.contains("Evidence"));
    }
}
