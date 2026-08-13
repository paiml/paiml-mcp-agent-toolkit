#![cfg_attr(coverage_nightly, coverage(off))]
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

/// What red-team could conclude about one claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaimVerdict {
    /// Every source that this claim needs ran, and none contradicted it.
    Verified,
    /// At least one source that ran contradicts the claim.
    Contradicted,
    /// A source the claim depends on never ran. Not a pass and not a failure —
    /// see the NOT MEASURED evidence for the artefact that would settle it.
    Unverified,
}

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
    /// The single rule for reading a claim's evidence.
    ///
    /// "Verified" requires that something was actually measured *and* that
    /// nothing went unmeasured; anything else is stated as such. Previously a
    /// claim whose only capable source never ran printed "✅ All claims
    /// verified", and a claim whose source could not run printed 🔴
    /// HALLUCINATION — absence was rendered as both verdicts, never as itself.
    #[must_use]
    pub fn verdict_for(evidence: &[EvidenceResult]) -> ClaimVerdict {
        if evidence.iter().any(EvidenceResult::contradicts) {
            return ClaimVerdict::Contradicted;
        }
        let measured = evidence.iter().filter(|e| e.measured()).count();
        if measured == 0 || evidence.iter().any(|e| !e.measured()) {
            return ClaimVerdict::Unverified;
        }
        ClaimVerdict::Verified
    }

    /// Per-claim verdicts, in claim order.
    #[must_use]
    pub fn verdicts(&self) -> Vec<ClaimVerdict> {
        self.evidence_per_claim
            .iter()
            .map(|e| Self::verdict_for(e))
            .collect()
    }

    /// Claims red-team could not settle either way.
    #[must_use]
    pub fn unverified_claims(&self) -> Vec<usize> {
        self.verdicts()
            .into_iter()
            .enumerate()
            .filter(|(_, v)| *v == ClaimVerdict::Unverified)
            .map(|(i, _)| i)
            .collect()
    }

    /// Format as human-readable text report
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
            let verdicts = self.verdicts();
            let unverified = self.unverified_claims();

            if unverified.is_empty() {
                output.push_str("✅ All claims verified\n\n");
            } else if unverified.len() == self.claims.len() {
                output.push_str("⚠️  NOT VERIFIED — no claim could be checked\n\n");
            } else {
                output.push_str(&format!(
                    "⚠️  PARTIALLY VERIFIED — {} of {} claim(s) could not be checked\n\n",
                    unverified.len(),
                    self.claims.len()
                ));
            }
            output.push_str(&format!("Commit message: \"{}\"\n\n", self.commit_message));
            output.push_str(&format!("Found {} testable claim(s):\n", self.claims.len()));

            for (i, claim) in self.claims.iter().enumerate() {
                let mark = match verdicts[i] {
                    ClaimVerdict::Verified => "✅",
                    ClaimVerdict::Unverified => "❓",
                    ClaimVerdict::Contradicted => "❌",
                };
                output.push_str(&format!("  {} {}\n", mark, claim.text));
                for e in self.evidence_per_claim[i].iter().filter(|e| !e.measured()) {
                    output.push_str(&format!("      {}\n", e.details));
                }
            }

            if unverified.is_empty() {
                output.push_str("\nAll claims are supported by evidence.\n");
            } else {
                output.push_str(
                    "\nA claim marked ❓ was neither supported nor contradicted: the source \
                     that could settle it never ran.\n",
                );
            }
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
                .filter(|e| EvidenceResult::contradicts(e))
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
        output.push_str(
            "  3. Use qualified language (\"MVP\", \"Phase 1\", etc.) for incremental work\n",
        );

        output
    }

    /// Format as JSON
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
                        // `measured` travels with the datum: a consumer reading
                        // supports_claim alone would count an unread artefact
                        // as a contradiction.
                        "measured": e.measured(),
                        "supports_claim": e.supports_claim,
                        "contradicts_claim": e.contradicts(),
                        "confidence": e.confidence,
                        "details": e.details,
                        "timestamp": e.timestamp,
                    })
                }).collect::<Vec<_>>()
            }).collect::<Vec<_>>(),
            "claim_verdicts": self.verdicts().iter()
                .map(|v| format!("{v:?}"))
                .collect::<Vec<_>>(),
            "unverified_claims": self.unverified_claims(),
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            extractor: ClaimExtractor::new(),
            gatherer: EvidenceGatherer::new(),
            classifier: IntentClassifier::new(),
        }
    }

    /// Analyze a commit message for hallucinations
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn analyze_commit_message(
        &self,
        commit_message: &str,
        context: &RepositoryContext,
    ) -> RedTeamResult {
        debug_assert!(
            !commit_message.is_empty(),
            "commit_message must not be empty"
        );
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

            // Only *measured* evidence can contradict: a source that never ran
            // used to land here as `supports_claim: false` and fail the commit.
            let contradicting = evidence
                .iter()
                .filter(|e| EvidenceResult::contradicts(e))
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
                    crate::status_eprintln!("🔴 Red Team Mode: Analyzing commit message");
                    crate::status_eprintln!("📝 Message: {}", message);
                    crate::status_eprintln!("📁 Repository: {}", path.display());
                    crate::status_eprintln!(
                        "🔍 Scan mode: {}",
                        if *deep {
                            "Deep (entire git history)"
                        } else {
                            "Fast (recent commits only)"
                        }
                    );
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
                    crate::status_eprintln!(
                        "📊 Git history: {}",
                        if context.has_git_history() {
                            "✅ Found"
                        } else {
                            "⚠️  None"
                        }
                    );
                    crate::status_eprintln!(
                        "🧪 Test files: {} found",
                        context.get_test_files().len()
                    );
                    // "✅ Found" used to mean "a path exists", while the report
                    // itself went unread. Say what was read, or why not.
                    crate::status_eprintln!(
                        "📈 Coverage report: {}",
                        match (context.has_coverage_report(), context.actual_coverage) {
                            (_, Some(pct)) => format!("✅ Read: {pct:.1}% of lines covered"),
                            (true, None) => format!(
                                "⚠️  Found but unreadable: {}",
                                context
                                    .coverage_error
                                    .as_deref()
                                    .unwrap_or("no LF/LH records")
                            ),
                            (false, None) => "⚠️  None".to_string(),
                        }
                    );
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
                        println!("{}", serde_json::to_string_pretty(&result.format_json())?);
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

#[cfg_attr(coverage_nightly, coverage(off))]
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

    // ── #958: absence of benchmark data is not a hallucination ──

    /// REGRESSION (#958): every Performance claim exited 1 under a 🔴
    /// HALLUCINATION DETECTED banner, because the only reachable arm of
    /// `gather_performance_evidence` pushed `supports_claim: false` when no
    /// benchmark data had been read. `perf: 0% faster` is true by construction
    /// and was still reported a hallucination.
    #[test]
    fn a_perf_claim_without_benchmarks_is_unverified_not_hallucinated() {
        let handler = RedTeamHandler::new();
        let context = RepositoryContext::new_mock();

        for message in [
            "perf: 30% faster after SIMD",
            "perf: 0% faster",
            "perf: performance improved",
        ] {
            let result = handler.analyze_commit_message(message, &context);
            if result.claims.is_empty() {
                continue; // nothing testable was claimed
            }
            assert!(
                !result.hallucinations_detected,
                "{message:?} was reported a hallucination on absent benchmark data"
            );
            let text = result.format_text();
            assert!(
                !text.contains("HALLUCINATION"),
                "{message:?} still prints the hallucination banner:\n{text}"
            );
            assert_eq!(
                RedTeamResult::verdict_for(&result.evidence_per_claim[0]),
                ClaimVerdict::Unverified,
                "an unbenchmarked perf claim is neither verified nor contradicted"
            );
        }
    }

    // ── #957: an unread artefact may not read as verification ──

    /// A claim whose only capable source never ran used to print
    /// "✅ All claims verified".
    #[test]
    fn an_unmeasured_claim_is_not_reported_as_verified() {
        let unmeasured = vec![
            EvidenceResult {
                source: crate::red_team::EvidenceSource::GitHistory,
                supports_claim: true,
                confidence: 0.75,
                details: "No subsequent coverage fixes found".to_string(),
                timestamp: None,
            },
            EvidenceResult::not_measured("no coverage report found"),
        ];
        assert_eq!(
            RedTeamResult::verdict_for(&unmeasured),
            ClaimVerdict::Unverified
        );

        // Everything measured and consistent → verified.
        let measured = vec![EvidenceResult {
            source: crate::red_team::EvidenceSource::CoverageReport,
            supports_claim: true,
            confidence: 0.95,
            details: "Claimed: 95.0%, Actual: 95.2%".to_string(),
            timestamp: None,
        }];
        assert_eq!(
            RedTeamResult::verdict_for(&measured),
            ClaimVerdict::Verified
        );

        // A measured contradiction still wins.
        let contradicted = vec![
            EvidenceResult {
                source: crate::red_team::EvidenceSource::CoverageReport,
                supports_claim: false,
                confidence: 0.95,
                details: "Claimed: 95.0%, Actual: 10.0%".to_string(),
                timestamp: None,
            },
            EvidenceResult::not_measured("cargo audit was not run"),
        ];
        assert_eq!(
            RedTeamResult::verdict_for(&contradicted),
            ClaimVerdict::Contradicted
        );

        // No evidence at all is not a pass either.
        assert_eq!(RedTeamResult::verdict_for(&[]), ClaimVerdict::Unverified);
    }

    /// The banner must not claim verification the run did not perform.
    #[test]
    fn the_report_names_the_claims_it_could_not_check() {
        let handler = RedTeamHandler::new();
        let result =
            handler.analyze_commit_message("test: coverage at 95%", &RepositoryContext::new_mock());
        assert!(!result.claims.is_empty(), "the claim must be extracted");
        assert!(!result.hallucinations_detected);

        let text = result.format_text();
        assert!(
            !text.contains("All claims verified"),
            "a coverage claim adjudicated without a coverage report is not verified:\n{text}"
        );
        assert!(text.contains("NOT MEASURED"), "{text}");
        assert_eq!(result.unverified_claims(), vec![0]);

        let json = result.format_json();
        assert_eq!(json["claim_verdicts"][0], "Unverified");
        assert_eq!(json["evidence"][0][1]["measured"], false);
        assert_eq!(json["evidence"][0][1]["contradicts_claim"], false);
    }
}
