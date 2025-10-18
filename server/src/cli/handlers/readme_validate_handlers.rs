//! README and documentation hallucination detection CLI handlers
//!
//! Validates AI-generated documentation against codebase facts using semantic
//! entropy-based hallucination detection.
//!
//! Based on peer-reviewed research:
//! - Semantic Entropy (Farquhar et al., Nature 2024)
//! - MIND framework (IJCAI 2025)
//! - Unified Detection Framework (Complex & Intelligent Systems 2025)

use crate::services::hallucination_detector::{
    CodeFactDatabase, DocAccuracyValidator, ValidationResult, ValidationStatus,
};
use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use std::path::PathBuf;
use std::process::ExitCode;

/// Output format for validation results
#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable text output
    Text,
    /// JSON output for programmatic consumption
    Json,
    /// JUnit XML for CI integration
    Junit,
}

/// Validate README and documentation for hallucinations
///
/// # Example
///
/// ```bash
/// # Generate deep context first
/// pmat context --output deep_context.md --format llm-optimized
///
/// # Validate README against codebase facts
/// pmat validate-readme \
///     --targets README.md CLAUDE.md \
///     --deep-context deep_context.md \
///     --fail-on-contradiction
/// ```
#[derive(Parser, Debug)]
pub struct ValidateReadmeCmd {
    /// Documentation files to validate (e.g., README.md, CLAUDE.md)
    #[arg(short, long, num_args = 1.., required = true)]
    pub targets: Vec<PathBuf>,

    /// Deep context markdown file (output from `pmat context`)
    #[arg(short, long, required = true)]
    pub deep_context: PathBuf,

    /// Confidence threshold for verification (0.0-1.0)
    #[arg(long, default_value = "0.9")]
    pub verified_threshold: f32,

    /// Confidence threshold for contradictions (0.0-1.0)
    #[arg(long, default_value = "0.3")]
    pub contradiction_threshold: f32,

    /// Fail if contradictions found
    #[arg(long, default_value = "true")]
    pub fail_on_contradiction: bool,

    /// Fail if unverified claims found
    #[arg(long, default_value = "false")]
    pub fail_on_unverified: bool,

    /// Output format (text, json, junit)
    #[arg(short, long, default_value = "text")]
    pub output: OutputFormat,

    /// Show only failures (contradictions and unverified)
    #[arg(long)]
    pub failures_only: bool,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

impl ValidateReadmeCmd {
    /// Execute the validate-readme command
    pub fn execute(&self) -> Result<ExitCode> {
        if self.verbose {
            eprintln!("🔍 Validating documentation for hallucinations...");
            eprintln!("📄 Targets: {:?}", self.targets);
            eprintln!("📊 Deep context: {}", self.deep_context.display());
            eprintln!("🎯 Verified threshold: {}", self.verified_threshold);
            eprintln!("⚠️  Contradiction threshold: {}", self.contradiction_threshold);
        }

        // Load deep context
        let deep_context_markdown = std::fs::read_to_string(&self.deep_context)
            .with_context(|| format!("Failed to read deep context: {}", self.deep_context.display()))?;

        // Build code fact database
        let code_facts = CodeFactDatabase::from_markdown(&deep_context_markdown)
            .with_context(|| "Failed to parse deep context markdown")?;

        // Create validator
        let validator = DocAccuracyValidator::new(code_facts);

        // Validate each target file
        let mut all_results = Vec::new();
        let mut contradiction_count = 0;
        let mut unverified_count = 0;
        let mut verified_count = 0;

        for target in &self.targets {
            if self.verbose {
                eprintln!("\n📖 Validating {}...", target.display());
            }

            let doc_content = std::fs::read_to_string(target)
                .with_context(|| format!("Failed to read target file: {}", target.display()))?;

            let results = validator.validate_documentation(&doc_content, &target.to_string_lossy())
                .with_context(|| format!("Failed to validate {}", target.display()))?;

            // Count statuses
            for result in &results {
                match result.status {
                    ValidationStatus::Verified => verified_count += 1,
                    ValidationStatus::Contradiction => contradiction_count += 1,
                    ValidationStatus::Unverified | ValidationStatus::NotFound | ValidationStatus::Outdated => unverified_count += 1,
                    ValidationStatus::Inconclusive => {}
                }
            }

            all_results.push((target.clone(), results));
        }

        // Output results
        match self.output {
            OutputFormat::Text => self.print_text_summary(&all_results, verified_count, contradiction_count, unverified_count),
            OutputFormat::Json => self.print_json_summary(&all_results)?,
            OutputFormat::Junit => self.print_junit_summary(&all_results)?,
        }

        // Determine exit code
        if self.fail_on_contradiction && contradiction_count > 0 {
            if self.verbose {
                eprintln!("\n❌ Exiting with failure: {} contradictions found", contradiction_count);
            }
            Ok(ExitCode::FAILURE)
        } else if self.fail_on_unverified && unverified_count > 0 {
            if self.verbose {
                eprintln!("\n❌ Exiting with failure: {} unverified claims found", unverified_count);
            }
            Ok(ExitCode::FAILURE)
        } else {
            if self.verbose {
                eprintln!("\n✅ Validation passed");
            }
            Ok(ExitCode::SUCCESS)
        }
    }

    fn print_text_summary(
        &self,
        results: &[(PathBuf, Vec<ValidationResult>)],
        verified: usize,
        contradictions: usize,
        unverified: usize,
    ) {
        println!("\n🔬 Documentation Hallucination Detection Summary");
        println!("================================================");
        println!();
        println!("📄 Files validated:  {}", results.len());
        println!("✅ Verified claims:  {}", verified);
        println!("❌ Contradictions:   {}", contradictions);
        println!("⚠️  Unverified:       {}", unverified);
        println!();

        for (target, file_results) in results {
            println!("📖 {}", target.display());
            println!("{}", "─".repeat(50));

            for (idx, result) in file_results.iter().enumerate() {
                // Skip verified claims if failures_only is true
                if self.failures_only && matches!(result.status, ValidationStatus::Verified) {
                    continue;
                }

                let status_icon = match result.status {
                    ValidationStatus::Verified => "✅",
                    ValidationStatus::Contradiction => "❌",
                    ValidationStatus::Unverified => "⚠️",
                    ValidationStatus::NotFound => "🔍",
                    ValidationStatus::Outdated => "⏰",
                    ValidationStatus::Inconclusive => "❓",
                };

                println!("\n{} Claim #{}: {:?}", status_icon, idx + 1, result.claim.claim_type);
                println!("   Text: \"{}\"", result.claim.text);
                println!("   Line: {}", result.claim.line_number);
                println!("   Status: {:?}", result.status);
                println!("   Confidence: {:.2}", result.confidence);

                if let Some(evidence) = &result.evidence {
                    println!("   Evidence: {}", evidence.content);
                }

                if self.verbose {
                    println!("   Entities: {:?}", result.claim.entities);
                }
            }

            println!();
        }

        if contradictions == 0 && unverified == 0 {
            println!("🎉 All documentation claims are verified!");
        } else if contradictions > 0 {
            println!("💥 Found {} contradiction(s) - documentation contains hallucinations!", contradictions);
        } else if unverified > 0 {
            println!("⚠️  Found {} unverified claim(s)", unverified);
        }
    }

    fn print_json_summary(&self, results: &[(PathBuf, Vec<ValidationResult>)]) -> Result<()> {
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

        let verified = results.iter()
            .flat_map(|(_, r)| r)
            .filter(|r| matches!(r.status, ValidationStatus::Verified))
            .count();

        let contradictions = results.iter()
            .flat_map(|(_, r)| r)
            .filter(|r| matches!(r.status, ValidationStatus::Contradiction))
            .count();

        let unverified = results.iter()
            .flat_map(|(_, r)| r)
            .filter(|r| matches!(r.status, ValidationStatus::Unverified | ValidationStatus::NotFound | ValidationStatus::Outdated))
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

    fn print_junit_summary(&self, results: &[(PathBuf, Vec<ValidationResult>)]) -> Result<()> {
        let total_claims: usize = results.iter().map(|(_, r)| r.len()).sum();
        let failures: usize = results.iter()
            .flat_map(|(_, r)| r)
            .filter(|r| matches!(r.status, ValidationStatus::Contradiction | ValidationStatus::Unverified | ValidationStatus::NotFound | ValidationStatus::Outdated))
            .count();

        println!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
        println!(
            "<testsuites name=\"README Hallucination Detection\" tests=\"{}\" failures=\"{}\">",
            total_claims, failures
        );
        println!("  <testsuite name=\"Documentation Validation\" tests=\"{}\" failures=\"{}\">",
            total_claims, failures);

        for (target, file_results) in results {
            for (idx, result) in file_results.iter().enumerate() {
                let test_name = format!(
                    "{} - Claim #{}: {}",
                    target.display(),
                    idx + 1,
                    result.claim.text.chars().take(50).collect::<String>()
                );

                print!("    <testcase name=\"{}\" classname=\"HallucinationDetection\"",
                    xml_escape(&test_name));

                if matches!(result.status, ValidationStatus::Contradiction | ValidationStatus::Unverified | ValidationStatus::NotFound | ValidationStatus::Outdated) {
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

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xml_escape() {
        assert_eq!(xml_escape("hello"), "hello");
        assert_eq!(xml_escape("<tag>"), "&lt;tag&gt;");
        assert_eq!(xml_escape("a & b"), "a &amp; b");
        assert_eq!(xml_escape("\"quoted\""), "&quot;quoted&quot;");
    }

    #[test]
    fn test_validate_readme_cmd_defaults() {
        let cmd = ValidateReadmeCmd {
            targets: vec![PathBuf::from("README.md")],
            deep_context: PathBuf::from("deep_context.md"),
            verified_threshold: 0.9,
            contradiction_threshold: 0.3,
            fail_on_contradiction: true,
            fail_on_unverified: false,
            output: OutputFormat::Text,
            failures_only: false,
            verbose: false,
        };

        assert_eq!(cmd.verified_threshold, 0.9);
        assert_eq!(cmd.contradiction_threshold, 0.3);
        assert!(cmd.fail_on_contradiction);
        assert!(!cmd.fail_on_unverified);
    }
}
