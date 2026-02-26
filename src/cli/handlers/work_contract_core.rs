/// Popperian Work Contract with Meyer Design by Contract triad
///
/// Every claim made by `pmat work complete` must be falsifiable.
/// If ANY claim cannot be verified, work is BLOCKED.
///
/// v5.0: Claims are classified into require/ensure/invariant (Meyer triad).
/// v4.0: Flat claims list (backward-compatible).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkContract {
    /// Contract version ("5.0" for triad, "4.0" for flat)
    #[serde(default = "default_contract_version")]
    pub version: String,

    /// Work item ID
    pub work_item_id: String,

    /// Contract creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    // === BASELINE (captured at work start, immutable via git) ===
    /// Git SHA of baseline commit (tamper-proof)
    pub baseline_commit: String,

    /// TDG score at baseline
    pub baseline_tdg: f64,

    /// Coverage percentage at baseline
    pub baseline_coverage: f64,

    /// Rust project score at baseline (if Rust project)
    pub baseline_rust_score: Option<f64>,

    /// File manifest for anti-gaming detection
    pub baseline_file_manifest: FileManifest,

    // === THRESHOLDS ===
    /// Quality thresholds for this contract
    pub thresholds: ContractThresholds,

    // === FALSIFICATION CLAIMS (v4.0 flat list, retained for backward compat) ===
    /// Claims that must survive falsification
    pub claims: Vec<FalsifiableClaim>,

    // === MEYER TRIAD (v5.0 — Design by Contract) ===
    /// Contract profile that generated these claims
    #[serde(default)]
    pub profile: Option<ContractProfile>,

    /// Preconditions: must hold at work start
    #[serde(default)]
    pub require: Vec<ContractClause>,

    /// Postconditions: must hold at work complete
    #[serde(default)]
    pub ensure: Vec<ContractClause>,

    /// Invariants: must hold at every checkpoint
    #[serde(default)]
    pub invariant: Vec<ContractClause>,

    /// Claims explicitly excluded via --without
    #[serde(default)]
    pub excluded_claims: Vec<ExcludedClaim>,

    /// Current iteration number (for subcontracting)
    #[serde(default = "default_iteration")]
    pub iteration: u32,

    /// Postconditions inherited from prior iteration
    #[serde(default)]
    pub inherited_postconditions: Vec<ContractClause>,

    /// Contract quality metric
    #[serde(default)]
    pub contract_quality: Option<ContractQuality>,
}

fn default_contract_version() -> String {
    "4.0".to_string()
}

fn default_iteration() -> u32 {
    1
}

impl WorkContract {
    /// Create a new work contract with baseline capture (v4.0 flat claims, backward compat)
    pub fn new(work_item_id: String, baseline_commit: String) -> Self {
        Self {
            version: "4.0".to_string(),
            work_item_id,
            created_at: chrono::Utc::now(),
            baseline_commit,
            baseline_tdg: 0.0,
            baseline_coverage: 0.0,
            baseline_rust_score: None,
            baseline_file_manifest: FileManifest::default(),
            thresholds: ContractThresholds::default(),
            claims: Self::default_claims(),
            // v5.0 triad fields (empty for v4.0 contracts)
            profile: None,
            require: Vec::new(),
            ensure: Vec::new(),
            invariant: Vec::new(),
            excluded_claims: Vec::new(),
            iteration: 1,
            inherited_postconditions: Vec::new(),
            contract_quality: None,
        }
    }

    /// Create a v5.0 contract with Design by Contract triad.
    ///
    /// Detects the project profile, generates claims, verifies toolchain,
    /// applies exclusions, and calculates contract quality.
    pub fn with_dbc(
        work_item_id: String,
        baseline_commit: String,
        project_path: &Path,
        without: &[String],
        iteration: u32,
    ) -> Result<Self, anyhow::Error> {
        // Detect profile (or load from config)
        let config = DbcConfig::load(project_path);
        let profile = config.profile_override.clone()
            .unwrap_or_else(|| ContractProfile::detect(project_path));

        // §2.4: Verify toolchain preconditions (fail-fast, no silent skipping)
        let missing_tools = check_toolchain(&profile, project_path);
        if !missing_tools.is_empty() {
            let missing_names: Vec<_> = missing_tools.iter().map(|t| t.name.as_str()).collect();
            anyhow::bail!(
                "Toolchain precondition failure for {} profile: {} missing tool(s): {}\n\
                 Options:\n\
                 1. Install missing tools\n\
                 2. Downgrade: pmat work start <id> --profile universal\n\
                 3. Exclude: pmat work start <id> --without {}",
                profile.name(),
                missing_tools.len(),
                missing_names.join(", "),
                missing_tools.iter().map(|t| t.claim_id.as_str()).collect::<Vec<_>>().join(","),
            );
        }

        // Generate claims for the detected profile
        let all_clauses = claims_for_profile(&profile, &config);
        let applicable_count = all_clauses.len();

        // Classify into triad
        let (require, ensure, invariant) = classify_claims(&all_clauses);

        // Apply explicit exclusions
        let (require, excluded_r) = apply_exclusions(require, without);
        let (ensure, excluded_e) = apply_exclusions(ensure, without);
        let (invariant, excluded_i) = apply_exclusions(invariant, without);
        let excluded_claims: Vec<ExcludedClaim> =
            [excluded_r, excluded_e, excluded_i].concat();

        let active_count = require.len() + ensure.len() + invariant.len();
        let quality = ContractQuality::calculate(active_count, applicable_count);

        // Also generate the flat v4.0 claims for backward compat
        let flat_claims = Self::default_claims();

        let mut contract = Self {
            version: "5.0".to_string(),
            work_item_id,
            created_at: chrono::Utc::now(),
            baseline_commit,
            baseline_tdg: 0.0,
            baseline_coverage: 0.0,
            baseline_rust_score: None,
            baseline_file_manifest: FileManifest::default(),
            thresholds: ContractThresholds::default(),
            claims: flat_claims,
            profile: Some(profile),
            require,
            ensure,
            invariant,
            excluded_claims,
            iteration,
            inherited_postconditions: Vec::new(),
            contract_quality: Some(quality),
        };

        // §5.3-5.4: Subcontracting validation for iteration > 1
        if iteration > 1 {
            let prior_iteration = iteration - 1;
            // Load the prior contract to inherit postconditions
            if let Ok(prior) = Self::load(project_path, &contract.work_item_id) {
                if prior.iteration == prior_iteration {
                    // Validate monotonic postcondition strengthening
                    if let Err(violation) = validate_subcontracting(&prior.ensure, &contract.ensure) {
                        anyhow::bail!(
                            "Subcontracting violation (iteration {} vs {}): {}\n\
                             Postconditions must not weaken between iterations.\n\
                             Fix: strengthen or maintain all postconditions from iteration {}.",
                            prior_iteration,
                            iteration,
                            violation,
                            prior_iteration,
                        );
                    }
                    // Inherit postconditions from prior iteration
                    contract.inherited_postconditions = prior.ensure.clone();
                }
            }
        }

        Ok(contract)
    }

    /// Check if this is a v5.0 (triad) contract
    pub fn is_dbc(&self) -> bool {
        self.version == "5.0" && self.profile.is_some()
    }

    /// Get total active claim count across the triad
    pub fn triad_claim_count(&self) -> usize {
        self.require.len() + self.ensure.len() + self.invariant.len()
    }

    /// Generate default falsifiable claims
    fn default_claims() -> Vec<FalsifiableClaim> {
        vec![
            FalsifiableClaim {
                hypothesis: "All baseline files still exist".to_string(),
                falsification_method: FalsificationMethod::ManifestIntegrity,
                evidence_required: EvidenceType::FileList(vec![]),
                result: None,
                override_info: None,
            },
            FalsifiableClaim {
                hypothesis: "The falsifier is active and detecting (Meta-Check)".to_string(),
                falsification_method: FalsificationMethod::MetaFalsification,
                evidence_required: EvidenceType::CounterExample { details: "".into() },
                result: None,
                override_info: None,
            },
            FalsifiableClaim {
                hypothesis: "No coverage exclusion gaming".to_string(),
                falsification_method: FalsificationMethod::CoverageGaming,
                evidence_required: EvidenceType::FileList(vec![]),
                result: None,
                override_info: None,
            },
            FalsifiableClaim {
                hypothesis: "All changed lines are covered".to_string(),
                falsification_method: FalsificationMethod::DifferentialCoverage,
                evidence_required: EvidenceType::NumericComparison {
                    actual: 0.0,
                    threshold: 100.0,
                },
                result: None,
                override_info: None,
            },
            FalsifiableClaim {
                hypothesis: "Total coverage >= 95%".to_string(),
                falsification_method: FalsificationMethod::AbsoluteCoverage,
                evidence_required: EvidenceType::NumericComparison {
                    actual: 0.0,
                    threshold: 95.0,
                },
                result: None,
                override_info: None,
            },
            FalsifiableClaim {
                hypothesis: "TDG score >= baseline".to_string(),
                falsification_method: FalsificationMethod::TdgRegression,
                evidence_required: EvidenceType::NumericComparison {
                    actual: 0.0,
                    threshold: 0.0,
                },
                result: None,
                override_info: None,
            },
            FalsifiableClaim {
                hypothesis: "No function exceeds complexity 20".to_string(),
                falsification_method: FalsificationMethod::ComplexityRegression,
                evidence_required: EvidenceType::NumericComparison {
                    actual: 0.0,
                    threshold: 20.0,
                },
                result: None,
                override_info: None,
            },
            FalsifiableClaim {
                hypothesis: "No vulnerable dependencies added".to_string(),
                falsification_method: FalsificationMethod::SupplyChainIntegrity,
                evidence_required: EvidenceType::CounterExample { details: "".into() },
                result: None,
                override_info: None,
            },
            FalsifiableClaim {
                hypothesis: "No file exceeds 500 lines".to_string(),
                falsification_method: FalsificationMethod::FileSizeRegression,
                evidence_required: EvidenceType::NumericComparison {
                    actual: 0.0,
                    threshold: 500.0,
                },
                result: None,
                override_info: None,
            },
            FalsifiableClaim {
                hypothesis: "Spec & Roadmap Quality".to_string(),
                falsification_method: FalsificationMethod::SpecQuality,
                evidence_required: EvidenceType::NumericComparison {
                    actual: 0.0,
                    threshold: 95.0,
                },
                result: None,
                override_info: None,
            },
            FalsifiableClaim {
                hypothesis: "All changes pushed".to_string(),
                falsification_method: FalsificationMethod::GitHubSync,
                evidence_required: EvidenceType::GitState {
                    unpushed_commits: 0,
                    dirty_files: 0,
                },
                result: None,
                override_info: None,
            },
            FalsifiableClaim {
                hypothesis: "All examples compile and run".to_string(),
                falsification_method: FalsificationMethod::ExamplesCompile,
                evidence_required: EvidenceType::CounterExample { details: "".into() },
                result: None,
                override_info: None,
            },
            FalsifiableClaim {
                hypothesis: "pmat-book validation passes".to_string(),
                falsification_method: FalsificationMethod::BookValidation,
                evidence_required: EvidenceType::CounterExample { details: "".into() },
                result: None,
                override_info: None,
            },
            // v2.6 comply spec additions
            FalsifiableClaim {
                hypothesis: "No new SATD markers (TODO/FIXME/HACK)".to_string(),
                falsification_method: FalsificationMethod::SatdDetection,
                evidence_required: EvidenceType::FileList(vec![]),
                result: None,
                override_info: None,
            },
            FalsifiableClaim {
                hypothesis: "No new dead code introduced".to_string(),
                falsification_method: FalsificationMethod::DeadCodeDetection,
                evidence_required: EvidenceType::FileList(vec![]),
                result: None,
                override_info: None,
            },
            FalsifiableClaim {
                hypothesis: "All files have >= 95% coverage".to_string(),
                falsification_method: FalsificationMethod::PerFileCoverage,
                evidence_required: EvidenceType::FileList(vec![]),
                result: None,
                override_info: None,
            },
            FalsifiableClaim {
                hypothesis: "make lint passes".to_string(),
                falsification_method: FalsificationMethod::LintPass,
                evidence_required: EvidenceType::BooleanCheck(false),
                result: None,
                override_info: None,
            },
            // v3.1 defect churn prevention
            FalsifiableClaim {
                hypothesis: "All match arm variants have test coverage".to_string(),
                falsification_method: FalsificationMethod::VariantCoverage,
                evidence_required: EvidenceType::FileList(vec![]),
                result: None,
                override_info: None,
            },
            FalsifiableClaim {
                hypothesis: "No fix-after-fix chains exceed limit".to_string(),
                falsification_method: FalsificationMethod::FixChainLimit,
                evidence_required: EvidenceType::NumericComparison {
                    actual: 0.0,
                    threshold: 3.0,
                },
                result: None,
                override_info: None,
            },
            FalsifiableClaim {
                hypothesis: "Cross-crate integration tests pass".to_string(),
                falsification_method: FalsificationMethod::CrossCrateParity,
                evidence_required: EvidenceType::BooleanCheck(false),
                result: None,
                override_info: None,
            },
            FalsifiableClaim {
                hypothesis: "No performance regressions detected".to_string(),
                falsification_method: FalsificationMethod::RegressionGate,
                evidence_required: EvidenceType::NumericComparison {
                    actual: 0.0,
                    threshold: 0.0,
                },
                result: None,
                override_info: None,
            },
            // v4.0 provable contracts
            FalsifiableClaim {
                hypothesis: "No incomplete proofs (sorry) introduced".to_string(),
                falsification_method: FalsificationMethod::FormalProofVerification,
                evidence_required: EvidenceType::NumericComparison {
                    actual: 0.0,
                    threshold: 0.0,
                },
                result: None,
                override_info: None,
            },
        ]
    }

    /// Load contract from file
    pub fn load(project_path: &Path, work_item_id: &str) -> Result<Self> {
        let contract_path = Self::contract_path(project_path, work_item_id);
        let content = std::fs::read_to_string(&contract_path)
            .with_context(|| format!("Failed to load contract from {}", contract_path.display()))?;
        serde_json::from_str(&content).context("Failed to parse contract JSON")
    }

    /// Save contract to file
    pub fn save(&self, project_path: &Path) -> Result<PathBuf> {
        let contract_dir = project_path.join(".pmat-work").join(&self.work_item_id);
        std::fs::create_dir_all(&contract_dir)?;

        let contract_path = contract_dir.join("contract.json");
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&contract_path, json)?;

        Ok(contract_path)
    }

    /// Get contract path for a work item
    pub fn contract_path(project_path: &Path, work_item_id: &str) -> PathBuf {
        project_path
            .join(".pmat-work")
            .join(work_item_id)
            .join("contract.json")
    }

    /// Check if contract exists for work item
    pub fn exists(project_path: &Path, work_item_id: &str) -> bool {
        Self::contract_path(project_path, work_item_id).exists()
    }

    /// Acknowledge legacy debt by comparing current metrics against thresholds
    ///
    /// For projects that don't meet strict Popperian thresholds (95% coverage, etc.),
    /// this method creates tracked debt tickets and adds overrides to claims.
    ///
    /// This is the "managed migration path" for existing projects.
    pub fn acknowledge_legacy_debt(&mut self, project_path: &Path) -> Result<()> {
        let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();
        let mut debt_items: Vec<DebtItem> = Vec::new();

        // Check coverage against 95% threshold
        if self.baseline_coverage < self.thresholds.min_coverage_pct {
            let ticket_id = format!("DEBT-COV-{}", timestamp);
            debt_items.push(DebtItem {
                ticket_id: ticket_id.clone(),
                category: "coverage".to_string(),
                description: format!(
                    "Coverage {:.1}% is below required {:.1}%",
                    self.baseline_coverage, self.thresholds.min_coverage_pct
                ),
                current_value: self.baseline_coverage,
                required_value: self.thresholds.min_coverage_pct,
            });

            // Add override to AbsoluteCoverage claim
            for claim in &mut self.claims {
                if claim.falsification_method == FalsificationMethod::AbsoluteCoverage {
                    claim.override_info = Some(OverrideInfo {
                        reason: "Legacy Debt: Project predates strict coverage requirements"
                            .to_string(),
                        ticket_id: ticket_id.clone(),
                        timestamp: chrono::Utc::now(),
                    });
                }
            }
        }

        // Check TDG baseline (if we have one, treat 0 as "no data yet")
        if self.baseline_tdg > 0.0 {
            // TDG already captured - no debt for this
        }

        // Check for large files in the manifest
        let large_files: Vec<_> = self
            .baseline_file_manifest
            .files
            .iter()
            .filter(|(_, entry)| entry.lines > self.thresholds.max_file_lines)
            .map(|(path, entry)| (path.clone(), entry.lines))
            .collect();

        if !large_files.is_empty() {
            let ticket_id = format!("DEBT-SIZE-{}", timestamp);
            let details: Vec<String> = large_files
                .iter()
                .take(10)
                .map(|(p, lines)| format!("{}: {} lines", p.display(), lines))
                .collect();

            debt_items.push(DebtItem {
                ticket_id: ticket_id.clone(),
                category: "file_size".to_string(),
                description: format!(
                    "{} file(s) exceed {} line limit: {}",
                    large_files.len(),
                    self.thresholds.max_file_lines,
                    details.join(", ")
                ),
                current_value: large_files.len() as f64,
                required_value: 0.0,
            });

            // Add override to FileSizeRegression claim
            for claim in &mut self.claims {
                if claim.falsification_method == FalsificationMethod::FileSizeRegression {
                    claim.override_info = Some(OverrideInfo {
                        reason: "Legacy Debt: Large files predate strict size requirements"
                            .to_string(),
                        ticket_id: ticket_id.clone(),
                        timestamp: chrono::Utc::now(),
                    });
                }
            }
        }

        // Create debt tickets directory and write YAML files
        if !debt_items.is_empty() {
            let tickets_dir = project_path.join(".pmat-tickets");
            std::fs::create_dir_all(&tickets_dir)?;

            for item in &debt_items {
                let ticket_path = tickets_dir.join(format!("{}.yaml", item.ticket_id));
                let yaml_content = format!(
                    r#"# PMAT Legacy Debt Ticket
# Auto-generated by `pmat comply upgrade`
# DO NOT DELETE - Required for Popperian falsification override

ticket_id: "{}"
category: "{}"
created_at: "{}"
status: "open"

description: |
  {}

metrics:
  current: {:.2}
  required: {:.2}
  gap: {:.2}

resolution:
  # Update this section when addressing the debt
  plan: "TBD"
  target_date: null
  completed_at: null
"#,
                    item.ticket_id,
                    item.category,
                    chrono::Utc::now().to_rfc3339(),
                    item.description,
                    item.current_value,
                    item.required_value,
                    item.required_value - item.current_value,
                );

                std::fs::write(&ticket_path, yaml_content)?;
                println!("   📝 Created debt ticket: {}", ticket_path.display());
            }
        }

        Ok(())
    }
}

/// Internal struct for tracking debt items
#[derive(Debug)]
struct DebtItem {
    ticket_id: String,
    category: String,
    description: String,
    current_value: f64,
    required_value: f64,
}
