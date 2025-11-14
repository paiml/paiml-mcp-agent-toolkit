// Red Team Mode CLI Handler
//
// Implements hallucination detection for commit messages, documentation, and code comments
// Based on specification: docs/specifications/red-team-mode-spec.md v1.1

use crate::cli::progress::MultiStageProgress;
use crate::red_team::{
    Claim, ClaimExtractor, CommitInfo, EvidenceGatherer, EvidenceResult, IntentClassifier,
    RepositoryContext,
};
use anyhow::Context;
use clap::{Parser, Subcommand, ValueEnum};
use std::fmt;
use std::path::PathBuf;
use std::process::ExitCode;

/// Red Team Mode handler result
#[derive(Debug)]
pub struct RedTeamResult {
    pub commit_message: String,
    pub claims: Vec<Claim>,
    pub evidence_per_claim: Vec<Vec<EvidenceResult>>,
    pub hallucinations_detected: bool,
    pub confidence: f64,
}

impl RedTeamResult {
    /// Format as human-readable text report
    pub fn format_text(&self) -> String {
        let mut output = String::new();

        if self.claims.is_empty() {
            output.push_str("✅ No testable claims found\n\n");
            output.push_str(&format!("Commit message: \"{}\"\n", self.commit_message));
            output.push_str("\nThis commit message makes no absolute or testable claims.\n");
            output.push_str("No hallucination detection needed.\n");
            return output;
        }

        if !self.hallucinations_detected {
            output.push_str("✅ All claims verified\n\n");
            output.push_str(&format!("Commit message: \"{}\"\n\n", self.commit_message));
            output.push_str(&format!("Found {} testable claim(s):\n", self.claims.len()));

            for claim in &self.claims {
                output.push_str(&format!("  • {}\n", claim.text));
            }

            output.push_str("\nAll claims are supported by evidence.\n");
            return output;
        }

        // Hallucination detected
        output.push_str("🔴 HALLUCINATION DETECTED\n\n");
        output.push_str(&format!("Commit message: \"{}\"\n\n", self.commit_message));
        output.push_str(&format!("Confidence: {:.2}\n\n", self.confidence));

        for (i, claim) in self.claims.iter().enumerate() {
            let evidence = &self.evidence_per_claim[i];
            let contradicting: Vec<_> = evidence
                .iter()
                .filter(|e| !e.supports_claim)
                .collect();

            if contradicting.is_empty() {
                continue;
            }

            output.push_str(&format!("Claim {}: \"{}\"\n", i + 1, claim.text));
            output.push_str(&format!("  Category: {:?}\n", claim.category));
            output.push_str(&format!("  Absolute: {}\n\n", claim.is_absolute));

            output.push_str("  Contradicting Evidence:\n");
            for (j, e) in contradicting.iter().enumerate() {
                output.push_str(&format!(
                    "    {}. {:?}: {} (confidence: {:.2})\n",
                    j + 1,
                    e.source,
                    e.details,
                    e.confidence
                ));
            }
            output.push('\n');
        }

        output.push_str("Verdict: POTENTIAL HALLUCINATION\n");
        output.push_str("\nRemediation:\n");
        output.push_str("  1. Run validation tests before committing\n");
        output.push_str("  2. Update commit message to reflect actual state\n");
        output.push_str("  3. Use qualified language (\"MVP\", \"Phase 1\", etc.) for incremental work\n");

        output
    }

    /// Format as JSON
    pub fn format_json(&self) -> serde_json::Value {
        serde_json::json!({
            "commit_message": self.commit_message,
            "claims": self.claims.iter().map(|c| {
                serde_json::json!({
                    "text": c.text,
                    "category": format!("{:?}", c.category),
                    "is_absolute": c.is_absolute,
                    "numeric_value": c.numeric_value,
                    "has_scope_qualifier": c.has_scope_qualifier,
                })
            }).collect::<Vec<_>>(),
            "evidence": self.evidence_per_claim.iter().map(|evidence_list| {
                evidence_list.iter().map(|e| {
                    serde_json::json!({
                        "source": format!("{:?}", e.source),
                        "supports_claim": e.supports_claim,
                        "confidence": e.confidence,
                        "details": e.details,
                        "timestamp": e.timestamp,
                    })
                }).collect::<Vec<_>>()
            }).collect::<Vec<_>>(),
            "hallucinations_detected": self.hallucinations_detected,
            "confidence": self.confidence,
        })
    }
}

impl fmt::Display for RedTeamResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_text())
    }
}

/// Red Team Mode handler
pub struct RedTeamHandler {
    extractor: ClaimExtractor,
    gatherer: EvidenceGatherer,
    classifier: IntentClassifier,
}

impl RedTeamHandler {
    pub fn new() -> Self {
        Self {
            extractor: ClaimExtractor::new(),
            gatherer: EvidenceGatherer::new(),
            classifier: IntentClassifier::new(),
        }
    }

    /// Analyze a commit message for hallucinations
    pub fn analyze_commit_message(
        &self,
        commit_message: &str,
        context: &RepositoryContext,
    ) -> RedTeamResult {
        // Step 1: Extract claims
        let claims = self.extractor.extract(commit_message);

        if claims.is_empty() {
            return RedTeamResult {
                commit_message: commit_message.to_string(),
                claims: vec![],
                evidence_per_claim: vec![],
                hallucinations_detected: false,
                confidence: 0.0,
            };
        }

        // Step 2: Gather evidence for each claim
        let mut evidence_per_claim = Vec::new();
        let mut any_hallucination = false;
        let mut max_contradiction_confidence: f64 = 0.0;

        for claim in &claims {
            let evidence = self.gatherer.gather_evidence(claim, context);

            // Check if any evidence contradicts the claim
            let contradicting = evidence
                .iter()
                .filter(|e| !e.supports_claim)
                .collect::<Vec<_>>();

            if !contradicting.is_empty() {
                any_hallucination = true;
                for e in &contradicting {
                    max_contradiction_confidence = max_contradiction_confidence.max(e.confidence);
                }
            }

            evidence_per_claim.push(evidence);
        }

        RedTeamResult {
            commit_message: commit_message.to_string(),
            claims,
            evidence_per_claim,
            hallucinations_detected: any_hallucination,
            confidence: max_contradiction_confidence,
        }
    }

    /// Analyze a pair of commits to classify intent
    pub fn analyze_commit_pair(
        &self,
        original: &CommitInfo,
        followup: &CommitInfo,
    ) -> crate::red_team::IntentClassification {
        self.classifier.classify(original, followup)
    }
}

impl Default for RedTeamHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Output format for Red Team analysis
#[derive(Debug, Clone, ValueEnum)]
pub enum RedTeamOutputFormat {
    /// Human-readable text output
    Text,
    /// JSON output for programmatic consumption
    Json,
}

/// Red Team subcommands
#[derive(Subcommand, Debug)]
pub enum RedTeamCommands {
    /// Analyze a commit message for hallucinations
    Analyze {
        /// Commit message to analyze
        #[arg(short, long, required = true)]
        message: String,

        /// Project path (defaults to current directory)
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        /// Output format (text or json)
        #[arg(short = 'f', long, default_value = "text")]
        format: RedTeamOutputFormat,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,

        /// Deep scan: Check entire git history (slower but more thorough)
        /// Default: false (scan recent commits only - last 30 days)
        #[arg(short = 'd', long)]
        deep: bool,
    },
}

/// Red Team Mode CLI command
///
/// # Example
///
/// ```bash
/// # Analyze a commit message
/// pmat red-team analyze --message "feat: All tests passing"
///
/// # Analyze with JSON output
/// pmat red-team analyze --message "test: Coverage at 85%" --format json
/// ```
#[derive(Parser, Debug)]
pub struct RedTeamCmd {
    #[command(subcommand)]
    pub command: RedTeamCommands,
}

impl RedTeamCmd {
    /// Execute the red-team command
    pub fn execute(&self) -> anyhow::Result<ExitCode> {
        match &self.command {
            RedTeamCommands::Analyze {
                message,
                path,
                format,
                verbose,
                deep,
            } => {
                if *verbose {
                    eprintln!("🔴 Red Team Mode: Analyzing commit message");
                    eprintln!("📝 Message: {}", message);
                    eprintln!("📁 Repository: {}", path.display());
                    eprintln!("🔍 Scan mode: {}", if *deep { "Deep (entire git history)" } else { "Fast (recent commits only)" });
                }

                // Initialize progress indicator (only for text output)
                let show_progress = matches!(format, RedTeamOutputFormat::Text);
                let mut progress = if show_progress {
                    let stages = vec![
                        "Building repository context".to_string(),
                        "Extracting claims".to_string(),
                        "Gathering evidence".to_string(),
                        "Analyzing results".to_string(),
                    ];
                    Some(MultiStageProgress::new(stages))
                } else {
                    None
                };

                // Stage 1: Build repository context
                if let Some(ref mut p) = progress {
                    p.next_stage("Building repository context");
                }

                let handler = RedTeamHandler::new();
                let context = RepositoryContext::from_path_with_config(path, *deep)
                    .context("Failed to build repository context")?;

                if *verbose {
                    eprintln!("📊 Git history: {}", if context.has_git_history() { "✅ Found" } else { "⚠️  None" });
                    eprintln!("🧪 Test files: {} found", context.get_test_files().len());
                    eprintln!("📈 Coverage report: {}", if context.has_coverage_report() { "✅ Found" } else { "⚠️  None" });
                }

                // Stage 2: Extract claims
                if let Some(ref mut p) = progress {
                    p.next_stage("Extracting claims");
                }

                let result = handler.analyze_commit_message(message, &context);

                // Stage 3: Analysis complete
                if let Some(ref mut p) = progress {
                    p.next_stage("Analyzing results");
                }

                // Clear progress before output
                if let Some(p) = progress {
                    p.finish("Analysis complete");
                }

                match format {
                    RedTeamOutputFormat::Text => {
                        println!("{}", result.format_text());
                    }
                    RedTeamOutputFormat::Json => {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&result.format_json())?
                        );
                    }
                }

                // Exit with error if hallucinations detected
                if result.hallucinations_detected {
                    Ok(ExitCode::from(1))
                } else {
                    Ok(ExitCode::SUCCESS)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handler_no_claims() {
        let handler = RedTeamHandler::new();
        let context = RepositoryContext::new_mock();

        let result = handler.analyze_commit_message("refactor: Improve code style", &context);

        assert_eq!(result.claims.len(), 0);
        assert!(!result.hallucinations_detected);
    }

    #[test]
    fn test_handler_with_hallucination() {
        let handler = RedTeamHandler::new();
        let context = RepositoryContext::new_mock().with_test_results(true, 5);

        let result = handler.analyze_commit_message("feat: All tests passing", &context);

        assert_eq!(result.claims.len(), 1);
        assert!(result.hallucinations_detected);
        assert!(result.confidence > 0.0);
    }

    #[test]
    fn test_handler_format_text() {
        let handler = RedTeamHandler::new();
        let context = RepositoryContext::new_mock().with_test_results(true, 5);

        let result = handler.analyze_commit_message("feat: All tests passing", &context);

        let text = result.format_text();
        assert!(text.contains("HALLUCINATION DETECTED"));
        assert!(text.contains("All tests passing"));
        assert!(text.contains("Contradicting Evidence"));
    }

    #[test]
    fn test_handler_format_json() {
        let handler = RedTeamHandler::new();
        let context = RepositoryContext::new_mock();

        let result = handler.analyze_commit_message("test: Add new tests", &context);

        let json = result.format_json();
        assert_eq!(json["commit_message"], "test: Add new tests");
        assert_eq!(json["hallucinations_detected"], false);
    }
}
