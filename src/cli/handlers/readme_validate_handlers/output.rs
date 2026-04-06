#![cfg_attr(coverage_nightly, coverage(off))]
//! Output formatting for README validation results
//!
//! Provides text, JSON, and JUnit XML output formats.

use super::types::ValidateReadmeCmd;
// Re-export OutputFormat from types for use in execution.rs
pub(crate) use super::types::OutputFormat;
use crate::services::hallucination_detector::{ValidationResult, ValidationStatus};
use anyhow::Result;
use std::path::PathBuf;

impl ValidateReadmeCmd {
    pub(crate) fn print_text_summary(
        &self,
        results: &[(PathBuf, Vec<ValidationResult>)],
        verified: usize,
        contradictions: usize,
        unverified: usize,
    ) {
        debug_assert!(!results.is_empty(), "results must not be empty");
        use crate::cli::colors as c;
        println!();
        println!(
            "{}",
            c::header("Documentation Hallucination Detection Summary")
        );
        println!();
        println!(
            "  Files validated:  {}",
            c::number(&results.len().to_string())
        );
        println!("  Verified claims:  {}{}{}", c::GREEN, verified, c::RESET);
        println!(
            "  Contradictions:   {}{}{}",
            c::RED,
            contradictions,
            c::RESET
        );
        println!(
            "  Unverified:       {}{}{}",
            c::YELLOW,
            unverified,
            c::RESET
        );
        println!();

        for (target, file_results) in results {
            println!("  {}", c::path(&target.display().to_string()));
            println!("  {}", c::separator());

            for (idx, result) in file_results.iter().enumerate() {
                if self.failures_only && matches!(result.status, ValidationStatus::Verified) {
                    continue;
                }

                let (status_icon, color) = match result.status {
                    ValidationStatus::Verified => ("✓", c::GREEN),
                    ValidationStatus::Contradiction => ("✗", c::RED),
                    ValidationStatus::Unverified => ("⚠", c::YELLOW),
                    ValidationStatus::NotFound => ("?", c::YELLOW),
                    ValidationStatus::Outdated => ("⏰", c::YELLOW),
                    ValidationStatus::Inconclusive => ("~", c::DIM),
                };

                println!(
                    "\n  {color}{status_icon}{} Claim {}{}{}: {:?}",
                    c::RESET,
                    c::BOLD_WHITE,
                    idx + 1,
                    c::RESET,
                    result.claim.claim_type
                );
                println!("     Text: {}\"{}\"{}", c::DIM, result.claim.text, c::RESET);
                println!(
                    "     Line: {}{}{}",
                    c::CYAN,
                    result.claim.line_number,
                    c::RESET
                );
                println!("     Status: {color}{:?}{}", result.status, c::RESET);
                println!(
                    "     Confidence: {}",
                    c::pct(f64::from(result.confidence) * 100.0, 80.0, 50.0)
                );

                if let Some(evidence) = &result.evidence {
                    println!("     Evidence: {}{}{}", c::DIM, evidence.content, c::RESET);
                }

                if self.verbose {
                    println!(
                        "     Entities: {}{:?}{}",
                        c::DIM,
                        result.claim.entities,
                        c::RESET
                    );
                }
            }

            println!();
        }

        if contradictions == 0 && unverified == 0 {
            println!("{}", c::pass("All documentation claims are verified!"));
        } else if contradictions > 0 {
            println!(
                "{}",
                c::fail(&format!(
                    "Found {} contradiction(s) — documentation contains hallucinations!",
                    contradictions
                ))
            );
        } else if unverified > 0 {
            println!(
                "{}",
                c::warn(&format!("Found {} unverified claim(s)", unverified))
            );
        }
    }

    pub(crate) fn print_json_summary(
        &self,
        results: &[(PathBuf, Vec<ValidationResult>)],
    ) -> Result<()> {
        debug_assert!(!results.is_empty(), "results must not be empty");
        use serde_json::json;

        let results_json: Vec<_> = results
            .iter()
            .map(|(target, file_results)| {
                let claims_json: Vec<_> = file_results
                    .iter()
                    .map(|r| {
                        json!({
                            "claim_text": r.claim.text,
                            "claim_type": format!("{:?}", r.claim.claim_type),
                            "line_number": r.claim.line_number,
                            "status": format!("{:?}", r.status),
                            "confidence": r.confidence,
                            "is_negative": r.claim.is_negative,
                            "entities": r.claim.entities.iter().map(|e| format!("{:?}", e)).collect::<Vec<_>>(),
                            "evidence": r.evidence.as_ref().map(|e| e.content.clone()),
                        })
                    })
                    .collect();

                json!({
                    "file": target.to_string_lossy(),
                    "claims": claims_json,
                })
            })
            .collect();

        let verified = results
            .iter()
            .flat_map(|(_, r)| r)
            .filter(|r| matches!(r.status, ValidationStatus::Verified))
            .count();

        let contradictions = results
            .iter()
            .flat_map(|(_, r)| r)
            .filter(|r| matches!(r.status, ValidationStatus::Contradiction))
            .count();

        let unverified = results
            .iter()
            .flat_map(|(_, r)| r)
            .filter(|r| {
                matches!(
                    r.status,
                    ValidationStatus::Unverified
                        | ValidationStatus::NotFound
                        | ValidationStatus::Outdated
                )
            })
            .count();

        let output = json!({
            "files_validated": results.len(),
            "verified_claims": verified,
            "contradictions": contradictions,
            "unverified_claims": unverified,
            "results": results_json,
        });

        println!("{}", serde_json::to_string_pretty(&output)?);
        Ok(())
    }

    pub(crate) fn print_junit_summary(
        &self,
        results: &[(PathBuf, Vec<ValidationResult>)],
    ) -> Result<()> {
        debug_assert!(!results.is_empty(), "results must not be empty");
        let total_claims: usize = results.iter().map(|(_, r)| r.len()).sum();
        let failures: usize = results
            .iter()
            .flat_map(|(_, r)| r)
            .filter(|r| {
                matches!(
                    r.status,
                    ValidationStatus::Contradiction
                        | ValidationStatus::Unverified
                        | ValidationStatus::NotFound
                        | ValidationStatus::Outdated
                )
            })
            .count();

        println!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
        println!(
            "<testsuites name=\"README Hallucination Detection\" tests=\"{}\" failures=\"{}\">",
            total_claims, failures
        );
        println!(
            "  <testsuite name=\"Documentation Validation\" tests=\"{}\" failures=\"{}\">",
            total_claims, failures
        );

        for (target, file_results) in results {
            for (idx, result) in file_results.iter().enumerate() {
                let test_name = format!(
                    "{} - Claim #{}: {}",
                    target.display(),
                    idx + 1,
                    result.claim.text.chars().take(50).collect::<String>()
                );

                print!(
                    "    <testcase name=\"{}\" classname=\"HallucinationDetection\"",
                    xml_escape(&test_name)
                );

                if matches!(
                    result.status,
                    ValidationStatus::Contradiction
                        | ValidationStatus::Unverified
                        | ValidationStatus::NotFound
                        | ValidationStatus::Outdated
                ) {
                    println!(">");
                    println!(
                        "      <failure message=\"{}: Confidence {:.2}\">",
                        xml_escape(&format!("{:?}", result.status)),
                        result.confidence
                    );
                    println!("Claim: {}", xml_escape(&result.claim.text));
                    if let Some(evidence) = &result.evidence {
                        println!("Evidence: {}", xml_escape(&evidence.content));
                    }
                    println!("      </failure>");
                    println!("    </testcase>");
                } else {
                    println!(" />");
                }
            }
        }

        println!("  </testsuite>");
        println!("</testsuites>");
        Ok(())
    }
}

pub(crate) fn xml_escape(s: &str) -> String {
    debug_assert!(!s.is_empty(), "s must not be empty");
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
