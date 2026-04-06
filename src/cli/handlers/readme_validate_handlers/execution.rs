#![cfg_attr(coverage_nightly, coverage(off))]
//! Command execution logic for README validation

use super::output::OutputFormat;
use super::types::ValidateReadmeCmd;
use crate::services::hallucination_detector::{
    CodeFactDatabase, DocAccuracyValidator, ValidationStatus,
};
use anyhow::{Context, Result};
use std::process::ExitCode;

impl ValidateReadmeCmd {
    /// Execute the validate-readme command
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn execute(&self) -> Result<ExitCode> {
        if self.verbose {
            eprintln!("🔍 Validating documentation for hallucinations...");
            eprintln!("📄 Targets: {:?}", self.targets);
            eprintln!("📊 Deep context: {}", self.deep_context.display());
            eprintln!("🎯 Verified threshold: {}", self.verified_threshold);
            eprintln!(
                "⚠️  Contradiction threshold: {}",
                self.contradiction_threshold
            );
        }

        // Load deep context
        let deep_context_markdown =
            std::fs::read_to_string(&self.deep_context).with_context(|| {
                format!(
                    "Failed to read deep context: {}",
                    self.deep_context.display()
                )
            })?;

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

            let results = validator
                .validate_documentation(&doc_content, &target.to_string_lossy())
                .with_context(|| format!("Failed to validate {}", target.display()))?;

            // Count statuses
            for result in &results {
                match result.status {
                    ValidationStatus::Verified => verified_count += 1,
                    ValidationStatus::Contradiction => contradiction_count += 1,
                    ValidationStatus::Unverified
                    | ValidationStatus::NotFound
                    | ValidationStatus::Outdated => unverified_count += 1,
                    ValidationStatus::Inconclusive => {}
                }
            }

            all_results.push((target.clone(), results));
        }

        // Output results
        match self.output {
            OutputFormat::Text | OutputFormat::Plain => self.print_text_summary(
                &all_results,
                verified_count,
                contradiction_count,
                unverified_count,
            ),
            OutputFormat::Json => self.print_json_summary(&all_results)?,
            OutputFormat::Junit => self.print_junit_summary(&all_results)?,
            _ => self.print_text_summary(
                &all_results,
                verified_count,
                contradiction_count,
                unverified_count,
            ),
        }

        // Determine exit code
        if self.fail_on_contradiction && contradiction_count > 0 {
            if self.verbose {
                eprintln!(
                    "\n❌ Exiting with failure: {} contradictions found",
                    contradiction_count
                );
            }
            Ok(ExitCode::FAILURE)
        } else if self.fail_on_unverified && unverified_count > 0 {
            if self.verbose {
                eprintln!(
                    "\n❌ Exiting with failure: {} unverified claims found",
                    unverified_count
                );
            }
            Ok(ExitCode::FAILURE)
        } else {
            if self.verbose {
                eprintln!("\n✅ Validation passed");
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}
