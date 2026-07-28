/// Falsifiable claim structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FalsifiableClaim {
    /// Human-readable hypothesis
    pub hypothesis: String,

    /// Method to attempt falsification
    pub falsification_method: FalsificationMethod,

    /// Evidence required to validate
    pub evidence_required: EvidenceType,

    /// Result of falsification attempt
    pub result: Option<FalsificationResult>,

    /// Optional override (requires justification AND ticket)
    pub override_info: Option<OverrideInfo>,
}

/// Information for an override (Popperian "Immunizing Stratagem")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverrideInfo {
    /// Reason for the override
    pub reason: String,

    /// Mandatory ticket ID (e.g., DEBT-123)
    pub ticket_id: String,

    /// Timestamp of override
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Methods for attempting to falsify claims
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FalsificationMethod {
    /// Try to find files in baseline missing from completion
    ManifestIntegrity,

    /// Try to find uncovered lines in changed code
    DifferentialCoverage,

    /// Try to find total coverage below threshold
    AbsoluteCoverage,

    /// Try to find TDG score regression
    TdgRegression,

    /// Try to find complexity regression
    ComplexityRegression,

    /// Try to find file size regression
    FileSizeRegression,

    /// Try to find spec score below threshold
    SpecQuality,

    /// Try to find roadmap not updated
    RoadmapUpdate,

    /// Try to find unpushed commits
    GitHubSync,

    /// Try to find #[cfg(not(coverage))] gaming
    CoverageGaming,

    /// Try to find vulnerable dependencies added (New in v1.1)
    SupplyChainIntegrity,

    /// Try to find flaws in the falsifier itself (Meta-Check) (New in v1.1)
    MetaFalsification,

    /// Try to find examples that don't compile/run (New in v1.2)
    ExamplesCompile,

    /// Try to find pmat-book validation failures (New in v1.2)
    BookValidation,

    /// Try to find new SATD markers (TODO/FIXME/HACK) (New in v2.6 - comply spec)
    SatdDetection,

    /// Try to find new dead code (unreachable functions/modules) (New in v2.6 - comply spec)
    DeadCodeDetection,

    /// Try to find files below 95% coverage threshold (New in v2.6 - comply spec)
    PerFileCoverage,

    /// Try to find lint failures (make lint) (New in v2.6 - comply spec)
    LintPass,

    /// Try to find untested match arm variants (New in v3.1 - defect churn)
    VariantCoverage,

    /// Try to find consecutive fix-commit chains (New in v3.1 - defect churn)
    FixChainLimit,

    /// Try to find cross-crate integration failures (New in v3.1 - defect churn)
    CrossCrateParity,

    /// Try to find performance regressions via benchmark gate (New in v3.1 - defect churn)
    RegressionGate,

    /// Try to find incomplete proofs (sorry) in Lean 4 projects (New in v4.0 - provable contracts)
    FormalProofVerification,

    /// Execute a specific falsification test from a provable-contracts YAML
    /// (Component 29 §Schema Extension). Seeded automatically by Component 27
    /// binding; manual additions/removals are blocked by CB-1624.
    ///
    /// `expected` is canonical JSON of the YAML test's `expected` field at
    /// bind time. CB-1621 compares it against the current YAML value to
    /// detect silent drift.
    ProvableContract {
        /// Resolved YAML file, e.g. `contracts/rope-kernel-v1.yaml`.
        yaml_path: std::path::PathBuf,
        /// Equation key within the YAML, e.g. `rope`.
        equation: String,
        /// Test id from `falsification_tests[]`, e.g. `rope_periodicity_test`.
        test_id: String,
        /// Canonical JSON snapshot of the test's `expected` field.
        expected: String,
    },
}

/// Evidence types for falsification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceType {
    /// Numeric comparison (actual vs threshold)
    NumericComparison { actual: f64, threshold: f64 },

    /// File list (missing/added/modified)
    FileList(Vec<PathBuf>),

    /// Concrete counter-example details (better than boolean)
    CounterExample { details: String },

    /// Boolean check (Legacy, prefer CounterExample)
    BooleanCheck(bool),

    /// Git state
    GitState {
        unpushed_commits: usize,
        dirty_files: usize,
    },
}

/// Result of a falsification attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FalsificationResult {
    /// Did falsification succeed (found a problem)?
    pub falsified: bool,

    /// Evidence that caused falsification
    pub evidence: Option<EvidenceType>,

    /// Human-readable explanation
    pub explanation: String,

    /// Was the claim actually evaluated against evidence?
    ///
    /// A claim whose data source is missing is neither corroborated nor
    /// falsified — it was not tested. Counting those as passes let the ladder
    /// report "22/22 claims validated" when most had verified nothing, which is
    /// the one thing a falsification receipt must never say. Defaults to true
    /// so receipts written before this field stay readable.
    #[serde(default = "default_measured")]
    pub measured: bool,
}

fn default_measured() -> bool {
    true
}

impl FalsificationResult {
    /// Create a passing result (hypothesis holds)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn passed(explanation: impl Into<String>) -> Self {
        Self {
            falsified: false,
            evidence: None,
            explanation: explanation.into(),
            measured: true,
        }
    }

    /// Create a result for a claim that could not be evaluated.
    ///
    /// Not falsified — there is no counter-example — but not corroborated
    /// either. Reported separately so an absent data source can never be read
    /// as evidence that the property holds.
    pub fn unmeasured(explanation: impl Into<String>) -> Self {
        Self {
            falsified: false,
            evidence: None,
            explanation: explanation.into(),
            measured: false,
        }
    }

    /// Create a failing result (hypothesis falsified)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn failed(explanation: impl Into<String>, evidence: EvidenceType) -> Self {
        Self {
            falsified: true,
            evidence: Some(evidence),
            explanation: explanation.into(),
            measured: true,
        }
    }
}
