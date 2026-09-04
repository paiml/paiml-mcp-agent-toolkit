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

    /// 5-dimension contract score (DBC spec §13.4)
    #[serde(default)]
    pub contract_score: Option<ContractScore>,

    // === PROVABLE-CONTRACTS INTEGRATION (work-management spec §2) ===

    /// Verification level target: L0 (review) through L5 (Lean proof).
    /// Typed since MACS-004 (Component 32): the wire format stays the display
    /// string ("L3"), so legacy contracts parse unchanged; reads migrate
    /// leniently ("l4" -> L4, recovering intent) and values outside the
    /// ladder deserialize as L0 (`pmat work migrate --levels` rewrites the
    /// file with an audit note). Storing an unparsed level is unrepresentable.
    #[serde(
        default = "default_verification_level",
        deserialize_with = "deserialize_verification_level"
    )]
    pub verification_level: crate::cli::handlers::work_verification_level::VerificationLevel,

    /// Research and specification references
    #[serde(default)]
    pub references: WorkReferences,

    /// Chain-of-thought audit trail (mirrors pv-spec Section 23)
    #[serde(default)]
    pub chain_of_thought: Vec<ChainOfThoughtStep>,

    /// Provable-contracts bindings declaring which YAML equations this ticket implements.
    /// Component 27: pmat-work-contract-binding. Empty vec = unbound ticket.
    #[serde(default)]
    pub implements: Vec<ContractBinding>,

    /// Which agent configuration started this work item (MACS F1, Component 32).
    /// Declared-first: from `--agent-*` flags / `PMAT_AGENT_*` env; advisory
    /// detection is labeled via `source`. None = no provenance declared.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<crate::cli::handlers::work_ledger::AgentProvenance>,
}

/// Binding to a provable-contracts YAML equation.
///
/// Sub-spec: docs/specifications/components/pmat-work-contract-binding.md (Component 27).
/// A ticket's `implements: Vec<ContractBinding>` declares which formal equations
/// this work item is modifying. Inherited preconditions/postconditions from the
/// YAML flow into the ticket's `require`/`ensure`/`invariant` triad.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContractBinding {
    /// Contract name, e.g. "rope-kernel-v1"
    pub contract: String,

    /// Equation identifier within the contract, e.g. "rope"
    pub equation: String,

    /// Resolved path to the YAML file, e.g. "contracts/rope-kernel-v1.yaml"
    pub file: PathBuf,

    /// SHA-256 of the YAML bytes at bind time. Drift detector (CB-1601).
    pub sha: String,

    /// When the binding was recorded (for audit)
    pub bound_at: chrono::DateTime<chrono::Utc>,
}

impl ContractBinding {
    /// Parse a `<contract>/<equation>` token into (contract, equation).
    /// Returns None if the token is malformed.
    pub fn parse_token(token: &str) -> Option<(String, String)> {
        let mut parts = token.splitn(2, '/');
        let contract = parts.next()?.trim();
        let equation = parts.next()?.trim();
        if contract.is_empty() || equation.is_empty() {
            return None;
        }
        Some((contract.to_string(), equation.to_string()))
    }

    /// Stable textual key used for deduplication.
    pub fn key(&self) -> String {
        format!("{}/{}", self.contract, self.equation)
    }
}

/// The claim a ticket makes when nothing says otherwise. An unbound ticket can
/// evidence at most L1 (`src/quality/ladder_evidence.rs`), so L1 is the only
/// default the completion gate can honour; the shipped L3 made every ticket
/// created through `add → start → complete` uncompletable without a hand edit
/// (#1186). A ticket bound with `--implements` is created at L2; `--level` sets
/// the claim explicitly on `add`, `start` and `edit`.
fn default_verification_level() -> crate::cli::handlers::work_verification_level::VerificationLevel {
    crate::cli::handlers::work_verification_level::VerificationLevel::L1
}

/// The claim a freshly created contract makes: the explicit `--level` when
/// given, else L2 when it is bound to at least one equation, else L1.
pub fn initial_verification_level(
    explicit: Option<crate::cli::handlers::work_verification_level::VerificationLevel>,
    bound: bool,
) -> crate::cli::handlers::work_verification_level::VerificationLevel {
    use crate::cli::handlers::work_verification_level::VerificationLevel;
    explicit.unwrap_or(if bound {
        VerificationLevel::L2
    } else {
        VerificationLevel::L1
    })
}

/// Parse a `--level` argument strictly (`L0`..`L5`); anything else is refused
/// with the accepted spellings, never silently mapped.
pub fn parse_level_arg(
    raw: &str,
) -> anyhow::Result<crate::cli::handlers::work_verification_level::VerificationLevel> {
    crate::cli::handlers::work_verification_level::VerificationLevel::parse_strict(raw.trim())
        .ok_or_else(|| anyhow::anyhow!("invalid --level '{raw}': expected one of L0, L1, L2, L3, L4, L5"))
}

/// Migrating deserializer for `WorkContract::verification_level` (MACS-004).
/// Strict parse first; lenient recovers case/whitespace corruption ("l4" ->
/// L4); anything else becomes L0 so legacy contracts keep loading — the
/// migrate tool rewrites the stored value with an audit note.
fn deserialize_verification_level<'de, D>(
    deserializer: D,
) -> Result<crate::cli::handlers::work_verification_level::VerificationLevel, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use crate::cli::handlers::work_verification_level::VerificationLevel;
    let raw = String::deserialize(deserializer)?;
    Ok(VerificationLevel::parse_migrating(&raw))
}

/// Research references linking work item to papers and specs
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkReferences {
    /// arXiv paper IDs (e.g., ["2509.06250", "2510.12047"])
    #[serde(default)]
    pub arxiv: Vec<String>,

    /// Specification section references (e.g., "pv-spec.md §23")
    #[serde(default)]
    pub spec_sections: Vec<String>,

    /// Five-whys report ID that originated this work item
    #[serde(default)]
    pub five_whys_id: Option<String>,

    /// Batuta oracle query results that informed the approach
    #[serde(default)]
    pub oracle_context: Option<String>,
}

/// A single step in the chain-of-thought reasoning audit trail.
///
/// Two generations coexist (MACS-007, Component 32 implementing C31):
/// - legacy prose: `{step, question, answer}` — annotated L0 evidence,
///   still parsed and preserved, never dropped;
/// - v2 structured: `{id, assumption, implication, evidence_method,
///   discharged_by}` — checkable (CB-1640) and derivable (CB-1658) via
///   `crate::models::work_cot`.
///
/// `assumption`/`implication` stay raw `Value`s because two wire shapes are
/// legal: a plain string (MACS Appendix A) or the C31 object form
/// `{text, references[], expr}` that CB-1640/1643 introspect.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainOfThoughtStep {
    /// Step number (1-based; legacy shape)
    #[serde(default, skip_serializing_if = "cot_step_is_zero")]
    pub step: u32,
    /// The question being answered (legacy prose shape)
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub question: String,
    /// The answer/reasoning (legacy prose shape)
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub answer: String,
    /// v2: step id, e.g. "CoT-1"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// v2: input to the reasoning (string or `{text, references, expr}`)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assumption: Option<serde_json::Value>,
    /// v2: output of the reasoning (string or `{text, references, expr}`)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub implication: Option<serde_json::Value>,
    /// v2: how to falsify the implication
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_method: Option<String>,
    /// v2: what discharges the assumption (CoT id | contract#equation |
    /// E<n> | Axiomatic)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discharged_by: Option<crate::models::work_cot::DischargeRef>,
}

fn cot_step_is_zero(step: &u32) -> bool {
    *step == 0
}

fn default_contract_version() -> String {
    "4.0".to_string()
}

fn default_iteration() -> u32 {
    1
}

impl WorkContract {
    /// Create a new work contract with baseline capture (v4.0 flat claims, backward compat)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
            contract_score: None,
            verification_level: default_verification_level(),
            references: WorkReferences::default(),
            chain_of_thought: Vec::new(),
            implements: Vec::new(),
            agent: None,
        }
    }

    /// Create a v5.0 contract with Design by Contract triad.
    ///
    /// Detects the project profile, generates claims, verifies toolchain,
    /// applies exclusions, and calculates contract quality.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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
            contract_score: None,
            verification_level: default_verification_level(),
            references: WorkReferences::default(),
            chain_of_thought: Vec::new(),
            implements: Vec::new(),
            agent: None,
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_dbc(&self) -> bool {
        self.version == "5.0" && self.profile.is_some()
    }

    /// Get total active claim count across the triad
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn load(project_path: &Path, work_item_id: &str) -> Result<Self> {
        let contract_path = Self::contract_path(project_path, work_item_id);
        let content = std::fs::read_to_string(&contract_path)
            .with_context(|| format!("Failed to load contract from {}", contract_path.display()))?;
        serde_json::from_str(&content).context("Failed to parse contract JSON")
    }

    /// Save contract to file
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn save(&self, project_path: &Path) -> Result<PathBuf> {
        let contract_dir = project_path.join(".pmat-work").join(&self.work_item_id);
        std::fs::create_dir_all(&contract_dir)?;

        let contract_path = contract_dir.join("contract.json");
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&contract_path, json)?;

        Ok(contract_path)
    }

    /// Get contract path for a work item
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn contract_path(project_path: &Path, work_item_id: &str) -> PathBuf {
        project_path
            .join(".pmat-work")
            .join(work_item_id)
            .join("contract.json")
    }

    /// Check if contract exists for work item
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn exists(project_path: &Path, work_item_id: &str) -> bool {
        Self::contract_path(project_path, work_item_id).exists()
    }
}

#[cfg(test)]
mod ladder_claim_tests {
    //! #1186: the claim a ticket makes follows its evidence.
    use super::{initial_verification_level, parse_level_arg, WorkContract};
    use crate::cli::handlers::work_verification_level::VerificationLevel;

    #[test]
    fn an_unbound_contract_claims_l1_by_default() {
        let c = WorkContract::new("T-1".to_string(), "deadbeef".to_string());
        assert_eq!(c.verification_level, VerificationLevel::L1, "the shipped default was L3, which no unbound ticket can evidence");
        assert!(c.implements.is_empty());
    }

    #[test]
    fn the_initial_claim_follows_the_bindings_unless_told_otherwise() {
        assert_eq!(initial_verification_level(None, false), VerificationLevel::L1);
        assert_eq!(initial_verification_level(None, true), VerificationLevel::L2);
        assert_eq!(initial_verification_level(Some(VerificationLevel::L3), false), VerificationLevel::L3);
        assert_eq!(initial_verification_level(Some(VerificationLevel::L1), true), VerificationLevel::L1);
    }

    #[test]
    fn a_level_argument_is_parsed_strictly() {
        assert_eq!(parse_level_arg("L2").expect("L2"), VerificationLevel::L2);
        assert_eq!(parse_level_arg(" L4 ").expect("trimmed"), VerificationLevel::L4);
        let err = parse_level_arg("L9").expect_err("L9 is not a level");
        assert!(err.to_string().contains("L9") && err.to_string().contains("L0, L1"), "{err}");
        assert!(parse_level_arg("high").is_err());
    }
}
