impl EvidenceGatherer {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            git_history_window_days: 30,
            confidence_threshold: 0.7,
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Gather evidence.
    pub fn gather_evidence(
        &self,
        claim: &Claim,
        context: &RepositoryContext,
    ) -> Vec<EvidenceResult> {
        let mut evidence = Vec::new();

        match claim.category {
            ClaimCategory::TestStatus => {
                evidence.extend(self.gather_test_status_evidence(claim, context));
            }
            ClaimCategory::Documentation => {
                evidence.extend(self.gather_documentation_evidence(claim, context));
            }
            ClaimCategory::Coverage => {
                evidence.extend(self.gather_coverage_evidence(claim, context));
            }
            ClaimCategory::FeatureCompletion => {
                evidence.extend(self.gather_feature_completion_evidence(claim, context));
            }
            ClaimCategory::Migration => {
                evidence.extend(self.gather_migration_evidence(claim, context));
            }
            ClaimCategory::BugFix => {
                evidence.extend(self.gather_bugfix_evidence(claim, context));
            }
            ClaimCategory::Performance => {
                evidence.extend(self.gather_performance_evidence(claim, context));
            }
            ClaimCategory::Security => {
                evidence.extend(self.gather_security_evidence(claim, context));
            }
        }

        evidence
    }

    fn gather_test_status_evidence(
        &self,
        claim: &Claim,
        context: &RepositoryContext,
    ) -> Vec<EvidenceResult> {
        let mut evidence = Vec::new();

        // Evidence 1: Git history - check for subsequent test fixes
        if let Some(ref commits) = context.subsequent_commits {
            let test_fixes = commits
                .iter()
                .filter(|msg| {
                    let lower = msg.to_lowercase();
                    lower.contains("fix") && (lower.contains("test") || lower.contains("ignore"))
                })
                .count();

            let supports_claim = test_fixes == 0;
            let confidence = if test_fixes > 0 { 0.85 } else { 0.6 };

            evidence.push(EvidenceResult {
                source: EvidenceSource::GitHistory,
                supports_claim,
                confidence,
                details: if test_fixes > 0 {
                    format!("{} subsequent test fixes found", test_fixes)
                } else {
                    "No subsequent test fixes found".to_string()
                },
                timestamp: context.latest_commit_timestamp,
            });
        }

        // Evidence 2: Test execution results
        if let Some((passing, ignored)) = context.test_results {
            let all_passing = passing && ignored == 0;
            let supports_claim = if claim.is_absolute {
                all_passing // Absolute claim requires all passing
            } else {
                passing // Qualified claim just needs passing (some ignored OK)
            };

            evidence.push(EvidenceResult {
                source: EvidenceSource::TestExecution,
                supports_claim,
                confidence: 0.9, // High confidence in test execution
                details: if all_passing {
                    "All tests passing".to_string()
                } else if ignored > 0 {
                    format!("{} tests ignored", ignored)
                } else {
                    "Tests failing".to_string()
                },
                timestamp: None,
            });
        }

        evidence
    }

    fn gather_documentation_evidence(
        &self,
        _claim: &Claim,
        context: &RepositoryContext,
    ) -> Vec<EvidenceResult> {
        let mut evidence = Vec::new();

        // Evidence 1: Git history - check for subsequent doc fixes
        if let Some(ref commits) = context.subsequent_commits {
            let doc_fixes = commits
                .iter()
                .filter(|msg| {
                    let lower = msg.to_lowercase();
                    lower.contains("docs") || lower.contains("link") || lower.contains("404")
                })
                .count();

            let supports_claim = doc_fixes == 0;

            evidence.push(EvidenceResult {
                source: EvidenceSource::GitHistory,
                supports_claim,
                confidence: 0.75,
                details: if doc_fixes > 0 {
                    format!("{} subsequent documentation fixes found", doc_fixes)
                } else {
                    "No subsequent documentation fixes found".to_string()
                },
                timestamp: context.latest_commit_timestamp,
            });
        }

        // Evidence 2: Link validation
        if let Some(broken_links) = context.broken_links_count {
            let supports_claim = broken_links == 0;

            evidence.push(EvidenceResult {
                source: EvidenceSource::LinkValidation,
                supports_claim,
                confidence: 0.9, // High confidence in link validation
                details: if broken_links > 0 {
                    format!("{} broken links found", broken_links)
                } else {
                    "All links valid".to_string()
                },
                timestamp: None,
            });
        }

        evidence
    }

    fn gather_coverage_evidence(
        &self,
        claim: &Claim,
        context: &RepositoryContext,
    ) -> Vec<EvidenceResult> {
        let mut evidence = Vec::new();

        // Evidence 1: Git history - check for subsequent coverage fixes
        if let Some(ref commits) = context.subsequent_commits {
            let coverage_fixes = commits
                .iter()
                .filter(|msg| {
                    let lower = msg.to_lowercase();
                    lower.contains("coverage")
                        && (lower.contains("fix") || lower.contains("regress"))
                })
                .count();

            let supports_claim = coverage_fixes == 0;

            evidence.push(EvidenceResult {
                source: EvidenceSource::GitHistory,
                supports_claim,
                confidence: 0.75,
                details: if coverage_fixes > 0 {
                    format!("{} subsequent coverage fixes found", coverage_fixes)
                } else {
                    "No subsequent coverage fixes found".to_string()
                },
                timestamp: context.latest_commit_timestamp,
            });
        }

        // Evidence 2: Coverage report comparison
        if let Some(actual_coverage) = context.actual_coverage {
            if let Some(claimed_coverage) = claim.numeric_value {
                let diff = (actual_coverage - claimed_coverage).abs();
                let supports_claim = diff <= 2.0; // Within 2% tolerance

                evidence.push(EvidenceResult {
                    source: EvidenceSource::CoverageReport,
                    supports_claim,
                    confidence: 0.95, // Very high confidence in coverage reports
                    details: format!(
                        "Claimed: {:.1}%, Actual: {:.1}%",
                        claimed_coverage, actual_coverage
                    ),
                    timestamp: None,
                });
            }
        } else if let Some(ref error) = context.coverage_error {
            // Coverage tool failed
            evidence.push(EvidenceResult {
                source: EvidenceSource::CoverageReport,
                supports_claim: false, // Cannot verify
                confidence: 0.5,
                details: format!("Coverage tool error: {}", error),
                timestamp: None,
            });
        }

        evidence
    }

    fn gather_feature_completion_evidence(
        &self,
        _claim: &Claim,
        context: &RepositoryContext,
    ) -> Vec<EvidenceResult> {
        let mut evidence = Vec::new();

        // Evidence 1: Git history - check for subsequent fixes/reverts
        if let Some(ref commits) = context.subsequent_commits {
            let fixes = commits
                .iter()
                .filter(|msg| {
                    let lower = msg.to_lowercase();
                    lower.contains("fix") || lower.contains("bug") || lower.contains("revert")
                })
                .count();

            let supports_claim = fixes == 0;

            evidence.push(EvidenceResult {
                source: EvidenceSource::GitHistory,
                supports_claim,
                confidence: 0.8,
                details: if fixes > 0 {
                    format!("{} subsequent fixes found", fixes)
                } else {
                    "No subsequent fixes found".to_string()
                },
                timestamp: context.latest_commit_timestamp,
            });
        }

        evidence
    }

    fn gather_migration_evidence(
        &self,
        _claim: &Claim,
        context: &RepositoryContext,
    ) -> Vec<EvidenceResult> {
        let mut evidence = Vec::new();

        // Evidence 1: Git history - check for rollbacks
        if let Some(ref commits) = context.subsequent_commits {
            let rollbacks = commits
                .iter()
                .filter(|msg| {
                    let lower = msg.to_lowercase();
                    lower.contains("revert") || lower.contains("rollback")
                })
                .count();

            let supports_claim = rollbacks == 0;

            evidence.push(EvidenceResult {
                source: EvidenceSource::GitHistory,
                supports_claim,
                confidence: 0.85,
                details: if rollbacks > 0 {
                    format!("{} rollback commits found", rollbacks)
                } else {
                    "No rollbacks found".to_string()
                },
                timestamp: context.latest_commit_timestamp,
            });
        }

        // Evidence 2: Code grep - check if old system still referenced
        if let Some((ref old_system, count)) = context.code_grep_results {
            let supports_claim = count == 0;

            evidence.push(EvidenceResult {
                source: EvidenceSource::CodeGrep,
                supports_claim,
                confidence: 0.8,
                details: if count > 0 {
                    format!("{} files still reference '{}'", count, old_system)
                } else {
                    format!("No references to '{}' found", old_system)
                },
                timestamp: None,
            });
        }

        evidence
    }

    fn gather_bugfix_evidence(
        &self,
        claim: &Claim,
        context: &RepositoryContext,
    ) -> Vec<EvidenceResult> {
        let mut evidence = Vec::new();

        // Evidence 1: Issue tracker status
        if let Some(issue_num) = claim.issue_number {
            if let Some(ref status) = context.issue_status {
                let is_closed = status == "closed";
                let is_reopened = status == "reopened" || status == "open";

                evidence.push(EvidenceResult {
                    source: EvidenceSource::IssueTracker,
                    supports_claim: is_closed,
                    confidence: 0.85,
                    details: if is_reopened {
                        format!("Issue #{} was reopened", issue_num)
                    } else if is_closed {
                        format!("Issue #{} is closed", issue_num)
                    } else {
                        format!("Issue #{} status: {}", issue_num, status)
                    },
                    timestamp: None,
                });
            }
        }

        // Evidence 2: Git history - check for regressions
        if let Some(ref commits) = context.subsequent_commits {
            let regressions = commits
                .iter()
                .filter(|msg| {
                    let lower = msg.to_lowercase();
                    lower.contains("regression") || lower.contains("re-fix")
                })
                .count();

            let supports_claim = regressions == 0;

            evidence.push(EvidenceResult {
                source: EvidenceSource::GitHistory,
                supports_claim,
                confidence: 0.8,
                details: if regressions > 0 {
                    format!("{} regression commits found", regressions)
                } else {
                    "No regressions found".to_string()
                },
                timestamp: context.latest_commit_timestamp,
            });
        }

        evidence
    }

    fn gather_performance_evidence(
        &self,
        claim: &Claim,
        context: &RepositoryContext,
    ) -> Vec<EvidenceResult> {
        let mut evidence = Vec::new();

        // Evidence 1: Benchmark results
        if let Some(ref benchmark_data) = context.benchmark_results {
            evidence.push(EvidenceResult {
                source: EvidenceSource::BenchmarkResults,
                supports_claim: true, // Benchmark data exists
                confidence: 0.9,
                details: format!("Benchmark data: {}", benchmark_data),
                timestamp: None,
            });
        } else {
            // No benchmark data available
            evidence.push(EvidenceResult {
                source: EvidenceSource::BenchmarkResults,
                supports_claim: false, // Cannot verify without data
                confidence: 0.7,
                details: if claim.numeric_value.is_some() {
                    "No benchmark data found to support numeric claim".to_string()
                } else {
                    "No benchmark data available".to_string()
                },
                timestamp: None,
            });
        }

        // Evidence 2: Git history - check for performance regressions
        if let Some(ref commits) = context.subsequent_commits {
            let perf_regressions = commits
                .iter()
                .filter(|msg| {
                    let lower = msg.to_lowercase();
                    (lower.contains("perf") || lower.contains("performance"))
                        && (lower.contains("regress")
                            || lower.contains("slow")
                            || lower.contains("timeout"))
                })
                .count();

            let supports_claim = perf_regressions == 0;

            evidence.push(EvidenceResult {
                source: EvidenceSource::GitHistory,
                supports_claim,
                confidence: 0.75,
                details: if perf_regressions > 0 {
                    format!("{} performance regression commits found", perf_regressions)
                } else {
                    "No performance regressions found".to_string()
                },
                timestamp: context.latest_commit_timestamp,
            });
        }

        evidence
    }

    fn gather_security_evidence(
        &self,
        _claim: &Claim,
        context: &RepositoryContext,
    ) -> Vec<EvidenceResult> {
        let mut evidence = Vec::new();

        // Evidence 1: Cargo audit results
        if let Some(vuln_count) = context.vulnerabilities_count {
            let supports_claim = vuln_count == 0;

            evidence.push(EvidenceResult {
                source: EvidenceSource::CargoAudit,
                supports_claim,
                confidence: 0.95, // Very high confidence in cargo audit
                details: if vuln_count > 0 {
                    format!("{} vulnerabilities found", vuln_count)
                } else {
                    "No vulnerabilities found".to_string()
                },
                timestamp: None,
            });
        }

        // Evidence 2: Git history - check for subsequent security fixes
        if let Some(ref commits) = context.subsequent_commits {
            let security_fixes = commits
                .iter()
                .filter(|msg| {
                    let lower = msg.to_lowercase();
                    lower.contains("security") || lower.contains("vuln") || lower.contains("cve")
                })
                .count();

            let supports_claim = security_fixes == 0;

            evidence.push(EvidenceResult {
                source: EvidenceSource::GitHistory,
                supports_claim,
                confidence: 0.85,
                details: if security_fixes > 0 {
                    format!("{} subsequent security fixes found", security_fixes)
                } else {
                    "No subsequent security fixes found".to_string()
                },
                timestamp: context.latest_commit_timestamp,
            });
        }

        evidence
    }
}
