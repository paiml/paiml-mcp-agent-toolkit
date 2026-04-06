// Rescue Protocol: Meyer §11 exception handling for postcondition violations
// Spec: docs/specifications/dbc.md §6

/// Rescue strategy — what to do when a postcondition fails
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RescueStrategy {
    /// Run coverage gap analysis, generate test stubs
    CoverageGapAnalysis,
    /// Run five-whys analysis, suggest refactoring
    FiveWhysAnalysis,
    /// Run dead code detection, suggest removal
    DeadCodeRemoval,
    /// Run SATD scan, list markers for resolution
    SatdResolution,
    /// Run complexity analysis, suggest extract-method
    ComplexityReduction,
    /// No automated rescue available — print guidance
    ManualIntervention { guidance: String },
}

impl std::fmt::Display for RescueStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RescueStrategy::CoverageGapAnalysis => write!(f, "CoverageGapAnalysis"),
            RescueStrategy::FiveWhysAnalysis => write!(f, "FiveWhysAnalysis"),
            RescueStrategy::DeadCodeRemoval => write!(f, "DeadCodeRemoval"),
            RescueStrategy::SatdResolution => write!(f, "SatdResolution"),
            RescueStrategy::ComplexityReduction => write!(f, "ComplexityReduction"),
            RescueStrategy::ManualIntervention { .. } => write!(f, "ManualIntervention"),
        }
    }
}

/// Outcome of a rescue attempt
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RescueOutcome {
    /// Rescue produced actionable diagnosis — developer must act
    ManualActionRequired,
    /// Rescue produced fix suggestions — developer can apply them
    FixSuggested,
    /// Rescue failed — no useful diagnosis could be produced
    DiagnosisFailed,
}

/// Record of a rescue attempt (DbC §8.3)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RescueRecord {
    /// Unique rescue ID
    pub rescue_id: String,
    /// Work item this rescue belongs to
    pub work_item_id: String,
    /// Timestamp of the rescue attempt
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Which postcondition clause was violated
    pub violated_clause: String,
    /// Strategy used for rescue
    pub strategy: RescueStrategy,
    /// Diagnosis details (free-form JSON-serializable data)
    pub diagnosis: RescueDiagnosis,
    /// Outcome of the rescue attempt
    pub outcome: RescueOutcome,
    /// Whether retry is allowed (one attempt per completion)
    pub retry_allowed: bool,
}

/// Structured diagnosis from a rescue attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RescueDiagnosis {
    /// Summary of the diagnosis
    pub summary: String,
    /// Suggested actions for the developer
    pub suggested_actions: Vec<String>,
    /// Path to generated artifacts (e.g., test stubs)
    pub generated_artifacts: Vec<String>,
}

impl RescueRecord {
    /// Save rescue record to .pmat-work/{item-id}/rescue/
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn save(&self, project_path: &Path) -> Result<PathBuf> {
        let rescue_dir = project_path
            .join(".pmat-work")
            .join(&self.work_item_id)
            .join("rescue");
        std::fs::create_dir_all(&rescue_dir)?;

        let filename = format!("rescue-{}.json", self.rescue_id);
        let rescue_path = rescue_dir.join(filename);
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&rescue_path, &json)?;
        Ok(rescue_path)
    }

    /// Load all rescue records for a work item
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn load_all(project_path: &Path, work_item_id: &str) -> Vec<Self> {
        let rescue_dir = project_path
            .join(".pmat-work")
            .join(work_item_id)
            .join("rescue");

        let mut records = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&rescue_dir) {
            for entry in entries.flatten() {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    if let Ok(record) = serde_json::from_str::<RescueRecord>(&content) {
                        records.push(record);
                    }
                }
            }
        }
        records.sort_by_key(|r| r.timestamp);
        records
    }
}

/// Map a FalsificationMethod to its rescue strategy.
///
/// Each postcondition violation type has a recommended rescue approach.
/// Returns None if no rescue is available for this method.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn rescue_strategy_for(method: &FalsificationMethod) -> Option<RescueStrategy> {
    match method {
        FalsificationMethod::AbsoluteCoverage
        | FalsificationMethod::DifferentialCoverage
        | FalsificationMethod::PerFileCoverage => Some(RescueStrategy::CoverageGapAnalysis),

        FalsificationMethod::TdgRegression => Some(RescueStrategy::FiveWhysAnalysis),

        FalsificationMethod::ComplexityRegression => Some(RescueStrategy::ComplexityReduction),

        FalsificationMethod::SatdDetection => Some(RescueStrategy::SatdResolution),

        FalsificationMethod::DeadCodeDetection => Some(RescueStrategy::DeadCodeRemoval),

        FalsificationMethod::SupplyChainIntegrity => {
            Some(RescueStrategy::ManualIntervention {
                guidance: "Run `cargo audit` to identify vulnerable dependencies, then update or replace them.".to_string(),
            })
        }

        FalsificationMethod::FileSizeRegression => {
            Some(RescueStrategy::ManualIntervention {
                guidance: "Split large files using extract-module refactoring. Target files exceeding the line limit.".to_string(),
            })
        }

        FalsificationMethod::GitHubSync => {
            Some(RescueStrategy::ManualIntervention {
                guidance: "Push all local commits: `git push origin HEAD`".to_string(),
            })
        }

        // Methods without a rescue strategy
        FalsificationMethod::ManifestIntegrity
        | FalsificationMethod::MetaFalsification
        | FalsificationMethod::CoverageGaming
        | FalsificationMethod::ExamplesCompile
        | FalsificationMethod::BookValidation
        | FalsificationMethod::LintPass
        | FalsificationMethod::VariantCoverage
        | FalsificationMethod::FixChainLimit
        | FalsificationMethod::CrossCrateParity
        | FalsificationMethod::RegressionGate
        | FalsificationMethod::FormalProofVerification
        | FalsificationMethod::SpecQuality
        | FalsificationMethod::RoadmapUpdate => None,
    }
}

/// Execute a rescue attempt for a violated postcondition.
///
/// This is the Phase 4 implementation: produces a structured diagnosis
/// with suggested actions. Rescue never modifies source code directly —
/// it only generates diagnostics and stubs.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn execute_rescue(
    _project_path: &Path,
    work_item_id: &str,
    violated_clause: &str,
    strategy: &RescueStrategy,
) -> RescueRecord {
    let diagnosis = match strategy {
        RescueStrategy::CoverageGapAnalysis => RescueDiagnosis {
            summary: "Coverage gap analysis: scan for uncovered functions in modified files".to_string(),
            suggested_actions: vec![
                "Run: pmat query --coverage-gaps --limit 10".to_string(),
                "Write tests targeting uncovered functions".to_string(),
                "Re-run: pmat work complete <id>".to_string(),
            ],
            generated_artifacts: vec![],
        },

        RescueStrategy::FiveWhysAnalysis => RescueDiagnosis {
            summary: "Root cause analysis via Five Whys methodology".to_string(),
            suggested_actions: vec![
                "Run: pmat five-whys \"TDG regression\"".to_string(),
                "Address the root cause identified".to_string(),
                "Re-run: pmat work complete <id>".to_string(),
            ],
            generated_artifacts: vec![],
        },

        RescueStrategy::DeadCodeRemoval => RescueDiagnosis {
            summary: "Dead code detected — remove unused functions".to_string(),
            suggested_actions: vec![
                "Run: pmat analyze dead-code".to_string(),
                "Remove identified dead code".to_string(),
                "Re-run: pmat work complete <id>".to_string(),
            ],
            generated_artifacts: vec![],
        },

        RescueStrategy::SatdResolution => RescueDiagnosis {
            summary: "Self-Admitted Technical Debt markers found".to_string(),
            suggested_actions: vec![
                "Run: pmat analyze satd".to_string(),
                "Resolve TODO/FIXME/HACK markers or create tickets".to_string(),
                "Re-run: pmat work complete <id>".to_string(),
            ],
            generated_artifacts: vec![],
        },

        RescueStrategy::ComplexityReduction => RescueDiagnosis {
            summary: "Complexity exceeds threshold — extract methods/functions".to_string(),
            suggested_actions: vec![
                "Run: pmat analyze complexity --violations-only".to_string(),
                "Apply extract-method refactoring on high-complexity functions".to_string(),
                "Re-run: pmat work complete <id>".to_string(),
            ],
            generated_artifacts: vec![],
        },

        RescueStrategy::ManualIntervention { guidance } => RescueDiagnosis {
            summary: format!("Manual intervention required: {}", guidance),
            suggested_actions: vec![
                guidance.clone(),
                "Re-run: pmat work complete <id>".to_string(),
            ],
            generated_artifacts: vec![],
        },
    };

    let outcome = RescueOutcome::ManualActionRequired;

    RescueRecord {
        rescue_id: uuid::Uuid::new_v4().to_string(),
        work_item_id: work_item_id.to_string(),
        timestamp: chrono::Utc::now(),
        violated_clause: violated_clause.to_string(),
        strategy: strategy.clone(),
        diagnosis,
        outcome,
        retry_allowed: true,
    }
}

/// Check if rescue is enabled for a given contract profile.
///
/// Per DbC §6.5: Pmat = enabled by default, others = opt-in via config.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn is_rescue_enabled(profile: &Option<ContractProfile>, config: &DbcConfig) -> bool {
    // Check config override first
    if config.rescue_enabled == Some(true) {
        return true;
    }
    if config.rescue_enabled == Some(false) {
        return false;
    }

    // Default: only Pmat profile has rescue enabled
    matches!(profile, Some(ContractProfile::Pmat))
}
