/// Falsification engine that runs strategies against extracted claims
pub struct FalsificationEngine {
    project_path: PathBuf,
}

impl FalsificationEngine {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    /// Create a new instance.
    pub fn new(project_path: &Path) -> Self {
        Self {
            project_path: project_path.to_path_buf(),
        }
    }

    /// Falsify all claims in a specification file
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn falsify_spec(&self, spec_path: &Path) -> Result<SpecFalsificationReport> {
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
        // Average over *measured* evidence only — unmeasured checks contribute
        // no information and must not dilute a real contradiction toward zero.
        let measured: Vec<f64> = evidence
            .iter()
            .filter(|e| e.measured)
            .map(|e| e.contradiction_score)
            .collect();
        let contradiction_score = if measured.is_empty() {
            0.0
        } else {
            measured.iter().sum::<f64>() / measured.len() as f64
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
        let full_path = self.project_path.join(path_str);
        let check = format!("File exists: {}", path_str);
        if full_path.exists() {
            return SpecEvidence::supports(check, "File found at expected location");
        }
        let suggestion = Self::find_similar_file(&full_path, &self.project_path);
        SpecEvidence::contradicts_with(check, format!("File NOT found{}", suggestion))
    }

    fn find_similar_file(full_path: &Path, project_path: &Path) -> String {
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

                        let check = format!("Entity exists: `{}`", entity);
                        if !file_matches.is_empty() {
                            let first_file = file_matches[0].trim();
                            let count = file_matches.len();
                            SpecEvidence::supports(
                                check,
                                format!("Found in {} file(s), e.g. {}", count, first_file),
                            )
                        } else {
                            SpecEvidence::measured(
                                check,
                                "NOT found in codebase",
                                SpecEvidence::FALSIFYING,
                            )
                        }
                    }
                    // The search never ran — that is not evidence the entity exists.
                    Err(_) => SpecEvidence::unmeasured(
                        format!("Entity exists: `{}`", entity),
                        "NOT MEASURED: could not run `pmat query` (pmat not on PATH)",
                    ),
                }
            })
            .collect()
    }

    /// Check absence claims by searching for counterexamples
    fn check_absence_claim(&self, claim: &SpecClaim) -> Vec<SpecEvidence> {
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
            return vec![SpecEvidence::unmeasured(
                "Absence claim",
                "NOT MEASURED: cannot determine what to search for",
            )];
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

                        let check = format!("Absence: no `{}`", term);
                        if total_count > 0 {
                            SpecEvidence::contradicts_with(
                                check,
                                format!("Found {} occurrences in codebase", total_count),
                            )
                        } else {
                            SpecEvidence::supports(check, "No occurrences found — claim holds")
                        }
                    }
                    // The search never ran — absence was not demonstrated.
                    Err(_) => SpecEvidence::unmeasured(
                        format!("Absence: no `{}`", term),
                        "NOT MEASURED: could not search the codebase",
                    ),
                }
            })
            .collect()
    }

    /// Check if referenced commands exist
    fn check_command_claim(&self, claim: &SpecClaim) -> Vec<SpecEvidence> {
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

                    let check = format!("Command exists: `{}`", cmd);
                    match output {
                        Ok(out) if out.status.success() => {
                            SpecEvidence::supports(check, "Command is available")
                        }
                        Ok(_) => SpecEvidence::contradicts_with(check, "Command NOT recognized"),
                        // pmat itself could not be spawned — nothing was tested.
                        Err(_) => SpecEvidence::unmeasured(
                            check,
                            "NOT MEASURED: could not run `pmat` (not on PATH)",
                        ),
                    }
                } else {
                    SpecEvidence::unmeasured(
                        format!("Command: `{}`", cmd),
                        "NOT MEASURED: could not parse command",
                    )
                }
            })
            .collect()
    }

    /// Check numeric/metric claims.
    ///
    /// pmat does not measure the metric a spec line names — a coverage bound, a
    /// complexity ceiling and a latency budget need three different measurement
    /// harnesses, and guessing which one a sentence means is not measurement.
    /// So this refuses explicitly rather than returning a passing score: the
    /// evidence is flagged unmeasured, which [`Self::determine_verdict`] can
    /// only render as INCONCLUSIVE. An unrun check must never read as a pass.
    fn check_metric_claim(&self, claim: &SpecClaim) -> Vec<SpecEvidence> {
        let target = match (&claim.numeric_comparator, claim.numeric_value) {
            (Some(cmp), Some(val)) => format!("Metric claim ({} {})", cmp, val),
            _ => "Metric claim".to_string(),
        };
        vec![SpecEvidence::unmeasured(
            target,
            "NOT MEASURED: pmat does not measure spec metrics (coverage, complexity, \
             latency); this claim was never tested and is NOT a pass",
        )]
    }

    /// Determine the verdict status from evidence.
    ///
    /// A claim SURVIVES only when every check against it actually ran and none
    /// contradicted it. Evidence that was never measured yields INCONCLUSIVE —
    /// "we did not look" is not "we looked and it was fine".
    fn determine_verdict(&self, claim: &SpecClaim, evidence: &[SpecEvidence]) -> VerdictStatus {
        if matches!(
            claim.category,
            SpecClaimCategory::Unfalsifiable | SpecClaimCategory::ArchitecturalClaim
        ) {
            return VerdictStatus::Unfalsifiable;
        }

        if evidence.is_empty() {
            return VerdictStatus::Inconclusive;
        }

        // A measured contradiction falsifies even if a sibling check was skipped.
        if evidence.iter().any(SpecEvidence::contradicts) {
            return VerdictStatus::Falsified;
        }

        if evidence.iter().any(|e| !e.measured) || evidence.iter().any(SpecEvidence::is_ambiguous) {
            return VerdictStatus::Inconclusive;
        }

        VerdictStatus::Survived
    }

    fn compute_summary(verdicts: &[SpecVerdict]) -> SpecFalsificationSummary {
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

        let health_score = SpecFalsificationSummary::health(survived, total_claims, unfalsifiable);

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
