/// Validation result for a claim
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimValidation {
    pub claim_id: String,
    pub status: ValidationStatus,
    pub evidence: Option<String>,
    pub score: f64,
}

/// Validation status (Popperian)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValidationStatus {
    /// Criterion proven true through evidence
    Proven,
    /// Could not be validated (remains false per Popper)
    Unfalsified,
    /// Validation explicitly failed
    Falsified,
    /// Cannot be automatically validated
    ManualRequired,
    /// Skipped (not applicable)
    Skipped,
}

impl ValidationStatus {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// As str.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proven => "PROVEN",
            Self::Unfalsified => "UNFALSIFIED",
            Self::Falsified => "FALSIFIED",
            Self::ManualRequired => "MANUAL",
            Self::Skipped => "SKIPPED",
        }
    }
}

/// Summary of spec validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    pub spec_path: PathBuf,
    pub total_claims: usize,
    pub proven: usize,
    pub falsified: usize,
    pub unfalsified: usize,
    pub manual_required: usize,
    pub category_scores: HashMap<String, f64>,
    pub total_score: f64,
    pub passed: bool,
    pub gateway_passed: bool,
}
