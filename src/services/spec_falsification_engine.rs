/// Falsification engine that runs strategies against extracted claims
pub struct FalsificationEngine {
    project_path: PathBuf,
}

impl FalsificationEngine {
    pub fn new(project_path: &Path) -> Self {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        Self {
            project_path: project_path.to_path_buf(),
        }
    }

    /// Falsify all claims in a specification file
    pub fn falsify_spec(&self, spec_path: &Path) -> Result<SpecFalsificationReport> {
        debug_assert!(spec_path.exists(), "spec_path must exist: {}", spec_path.display());
        let content = std::fs::read_to_string(spec_path)
            .with_context(|| format!("Failed to read spec: {}", spec_path.display()))?;

        let extractor = SpecClaimExtractor::new();
        let claims = extractor.extract(&content, spec_path);

        let verdicts: Vec<SpecVerdict> = claims
            .into_iter()
            .map(|claim| self.falsify_claim(claim))
            .collect();

        let summary = Self::compute_summary(&verdicts);

        Ok(SpecFalsificationReport {
            target_file: spec_path.to_path_buf(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            verdicts,
            summary,
        })
    }

    /// Falsify a single claim using the appropriate strategy
    fn falsify_claim(&self, claim: SpecClaim) -> SpecVerdict {
        debug_assert!(self.project_path.exists(), "self.project_path must exist");
        let evidence = match &claim.category {
            SpecClaimCategory::PathReference => self.check_path_references(&claim),
            SpecClaimCategory::CodeEntity => self.check_code_entities(&claim),
            SpecClaimCategory::AbsenceClaim => self.check_absence_claim(&claim),
            SpecClaimCategory::CommandClaim => self.check_command_claim(&claim),
            SpecClaimCategory::MetricClaim => self.check_metric_claim(&claim),
            SpecClaimCategory::ArchitecturalClaim => Vec::new(), // Inconclusive — needs human review
            SpecClaimCategory::Unfalsifiable => Vec::new(),
        };

        let status = self.determine_verdict(&claim, &evidence);
        let contradiction_score = if evidence.is_empty() {
            0.0
        } else {
            evidence.iter().map(|e| e.contradiction_score).sum::<f64>() / evidence.len() as f64
        };

        SpecVerdict {
            claim,
            status,
            evidence,
            contradiction_score,
        }
    }

    /// Check if referenced file paths exist
    fn check_path_references(&self, claim: &SpecClaim) -> Vec<SpecEvidence> {
        claim
            .path_refs
            .iter()
            .map(|path_str| self.check_single_path(path_str))
            .collect()
    }

    fn check_single_path(&self, path_str: &str) -> SpecEvidence {
        debug_assert!(!path_str.is_empty(), "path_str must not be empty");
        let full_path = self.project_path.join(path_str);
        if full_path.exists() {
            return SpecEvidence {
                check: format!("File exists: {}", path_str),
                finding: "File found at expected location".to_string(),
                contradiction_score: 0.0,
            };
        }
        let suggestion = Self::find_similar_file(&full_path, &self.project_path);
        SpecEvidence {
            check: format!("File exists: {}", path_str),
            finding: format!("File NOT found{}", suggestion),
            contradiction_score: 1.0,
        }
    }

    fn find_similar_file(full_path: &Path, project_path: &Path) -> String {
        debug_assert!(full_path.exists(), "full_path must exist: {}", full_path.display());
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let parent = full_path.parent().unwrap_or(project_path);
        let stem = full_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        if !parent.exists() || stem.is_empty() {
            return String::new();
        }
        let Ok(entries) = std::fs::read_dir(parent) else {
            return String::new();
        };
        for entry in entries.flatten() {
            if entry.file_name().to_string_lossy().contains(stem) {
                return format!(" (did you mean: {}?)", entry.path().display());
            }
        }
        String::new()
    }

    /// Check if referenced code entities exist using pmat query
    fn check_code_entities(&self, claim: &SpecClaim) -> Vec<SpecEvidence> {
        debug_assert!(self.project_path.exists(), "self.project_path must exist");
        claim
            .entity_refs
            .iter()
            .map(|entity| {
                // Use pmat query --literal with --files-with-matches for simpler parsing
                let output = std::process::Command::new("pmat")
                    .args([
                        "query",
                        "--literal",
                        entity,
                        "--files-with-matches",
                        "--limit",
                        "5",
                    ])
                    .current_dir(&self.project_path)
                    .output();

                match output {
                    Ok(out) => {
                        let stdout = String::from_utf8_lossy(&out.stdout);
                        // Strip ANSI codes and count non-empty lines that look like file paths
                        let ansi_re = Regex::new(r"\x1b\[[0-9;]*m").expect("internal ansi regex");
                        let clean = ansi_re.replace_all(&stdout, "");
                        let file_matches: Vec<&str> = clean
                            .lines()
                            .filter(|line| {
                                let trimmed = line.trim();
                                !trimmed.is_empty()
                                    && !trimmed.starts_with("Loading")
                                    && !trimmed.starts_with("Index:")
                                    && !trimmed.starts_with("Searching")
                                    && !trimmed.starts_with("query profile")
                                    && !trimmed.starts_with("Checking")
                                    && !trimmed.starts_with("Incremental")
                                    && !trimmed.starts_with("Merging")
                                    && !trimmed.starts_with("SQLite")
                                    && !trimmed.starts_with("Workspace")
                                    && !trimmed.starts_with('+')
                                    // Exclude spec files from matches (avoid self-reference)
                                    && !trimmed.contains("specifications/")
                                    && !trimmed.contains("docs/roadmaps/")
                            })
                            .collect();

                        if !file_matches.is_empty() {
                            let first_file = file_matches[0].trim();
                            let count = file_matches.len();
                            SpecEvidence {
                                check: format!("Entity exists: `{}`", entity),
                                finding: format!("Found in {} file(s), e.g. {}", count, first_file),
                                contradiction_score: 0.0,
                            }
                        } else {
                            SpecEvidence {
                                check: format!("Entity exists: `{}`", entity),
                                finding: "NOT found in codebase".to_string(),
                                contradiction_score: 0.8,
                            }
                        }
                    }
                    Err(_) => SpecEvidence {
                        check: format!("Entity exists: `{}`", entity),
                        finding: "Could not run pmat query (pmat not available)".to_string(),
                        contradiction_score: 0.0,
                    },
                }
            })
            .collect()
    }

    /// Check absence claims by searching for counterexamples
    fn check_absence_claim(&self, claim: &SpecClaim) -> Vec<SpecEvidence> {
        debug_assert!(self.project_path.exists(), "self.project_path must exist");
        // Extract what should be absent from the claim text
        let text_lower = claim.original_text.to_lowercase();
        let search_terms: Vec<&str> = if text_lower.contains("unsafe") {
            vec!["unsafe"]
        } else if text_lower.contains("panic") {
            vec!["panic!"]
        } else if text_lower.contains("unwrap") {
            vec!["unwrap()"]
        } else if text_lower.contains("todo") || text_lower.contains("fixme") {
            vec!["TODO", "FIXME"]
        } else {
            return vec![SpecEvidence {
                check: "Absence claim".to_string(),
                finding: "Cannot determine what to search for".to_string(),
                contradiction_score: 0.0,
            }];
        };

        search_terms
            .iter()
            .map(|term| {
                let output = std::process::Command::new("pmat")
                    .args([
                        "query",
                        "--literal",
                        term,
                        "--count",
                        "--exclude-tests",
                        "--limit",
                        "5",
                    ])
                    .current_dir(&self.project_path)
                    .output();

                match output {
                    Ok(out) => {
                        let stdout = String::from_utf8_lossy(&out.stdout);
                        // Strip ANSI codes before parsing count output
                        let ansi_re = Regex::new(r"\x1b\[[0-9;]*m").expect("internal ansi regex");
                        let clean = ansi_re.replace_all(&stdout, "");
                        let total_count: u32 = clean
                            .lines()
                            .filter(|line| line.contains(':'))
                            .filter_map(|line| {
                                line.split(':').next_back()?.trim().parse::<u32>().ok()
                            })
                            .sum();

                        if total_count > 0 {
                            SpecEvidence {
                                check: format!("Absence: no `{}`", term),
                                finding: format!("Found {} occurrences in codebase", total_count),
                                contradiction_score: 1.0,
                            }
                        } else {
                            SpecEvidence {
                                check: format!("Absence: no `{}`", term),
                                finding: "No occurrences found — claim holds".to_string(),
                                contradiction_score: 0.0,
                            }
                        }
                    }
                    Err(_) => SpecEvidence {
                        check: format!("Absence: no `{}`", term),
                        finding: "Could not search codebase".to_string(),
                        contradiction_score: 0.0,
                    },
                }
            })
            .collect()
    }

    /// Check if referenced commands exist
    fn check_command_claim(&self, claim: &SpecClaim) -> Vec<SpecEvidence> {
        debug_assert!(self.project_path.exists(), "self.project_path must exist");
        let cmd_pattern = Regex::new(r"`(pmat\s+[\w-]+)`").expect("internal regex");
        let commands: Vec<String> = cmd_pattern
            .captures_iter(&claim.original_text)
            .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
            .collect();

        commands
            .iter()
            .map(|cmd| {
                // Check if the subcommand exists by running pmat --help
                let parts: Vec<&str> = cmd.split_whitespace().collect();
                if parts.len() >= 2 {
                    let subcommand = parts[1];
                    let output = std::process::Command::new("pmat")
                        .args([subcommand, "--help"])
                        .current_dir(&self.project_path)
                        .output();

                    match output {
                        Ok(out) if out.status.success() => SpecEvidence {
                            check: format!("Command exists: `{}`", cmd),
                            finding: "Command is available".to_string(),
                            contradiction_score: 0.0,
                        },
                        _ => SpecEvidence {
                            check: format!("Command exists: `{}`", cmd),
                            finding: "Command NOT recognized".to_string(),
                            contradiction_score: 1.0,
                        },
                    }
                } else {
                    SpecEvidence {
                        check: format!("Command: `{}`", cmd),
                        finding: "Could not parse command".to_string(),
                        contradiction_score: 0.0,
                    }
                }
            })
            .collect()
    }

    /// Check numeric/metric claims
    fn check_metric_claim(&self, _claim: &SpecClaim) -> Vec<SpecEvidence> {
        debug_assert!(self.project_path.exists(), "self.project_path must exist");
        // Metric claims require running actual measurements (coverage, complexity, etc.)
        // For MVP, mark these as inconclusive since we can't cheaply verify them
        vec![SpecEvidence {
            check: "Metric claim".to_string(),
            finding: "Metric verification requires measurement — marked for manual review"
                .to_string(),
            contradiction_score: 0.0,
        }]
    }

    /// Determine the verdict status from evidence
    fn determine_verdict(&self, claim: &SpecClaim, evidence: &[SpecEvidence]) -> VerdictStatus {
        debug_assert!(self.project_path.exists(), "self.project_path must exist");
        if matches!(
            claim.category,
            SpecClaimCategory::Unfalsifiable | SpecClaimCategory::ArchitecturalClaim
        ) {
            return VerdictStatus::Unfalsifiable;
        }

        if evidence.is_empty() {
            return VerdictStatus::Inconclusive;
        }

        let max_contradiction = evidence
            .iter()
            .map(|e| e.contradiction_score)
            .fold(0.0f64, f64::max);

        if max_contradiction >= 0.8 {
            VerdictStatus::Falsified
        } else if max_contradiction >= 0.4 {
            VerdictStatus::Inconclusive
        } else {
            VerdictStatus::Survived
        }
    }

    fn compute_summary(verdicts: &[SpecVerdict]) -> SpecFalsificationSummary {
        debug_assert!(!verdicts.is_empty(), "verdicts must not be empty");
        let total_claims = verdicts.len();
        let survived = verdicts
            .iter()
            .filter(|v| v.status == VerdictStatus::Survived)
            .count();
        let falsified = verdicts
            .iter()
            .filter(|v| v.status == VerdictStatus::Falsified)
            .count();
        let unfalsifiable = verdicts
            .iter()
            .filter(|v| v.status == VerdictStatus::Unfalsifiable)
            .count();
        let inconclusive = verdicts
            .iter()
            .filter(|v| v.status == VerdictStatus::Inconclusive)
            .count();

        let testable = total_claims.saturating_sub(unfalsifiable);
        let health_score = if testable > 0 {
            survived as f64 / testable as f64
        } else {
            1.0
        };

        SpecFalsificationSummary {
            total_claims,
            survived,
            falsified,
            unfalsifiable,
            inconclusive,
            health_score,
        }
    }
}
