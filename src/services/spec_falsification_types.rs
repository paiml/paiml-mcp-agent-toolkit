/// Priority level based on RFC-2119 keywords
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimPriority {
    /// MUST / SHALL / REQUIRED — single counterexample falsifies
    P0Critical,
    /// SHOULD / RECOMMENDED — needs pattern of violation
    P1High,
    /// MAY / OPTIONAL — informational only
    P2Low,
    /// No RFC-2119 keyword — default priority
    P3Default,
}

impl std::fmt::Display for ClaimPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::P0Critical => write!(f, "P0"),
            Self::P1High => write!(f, "P1"),
            Self::P2Low => write!(f, "P2"),
            Self::P3Default => write!(f, "P3"),
        }
    }
}

/// Category of falsifiable claim — determines which strategy to use
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpecClaimCategory {
    /// Claim references a file path that should exist
    PathReference,
    /// Claim asserts a function/struct/module exists
    CodeEntity,
    /// Claim contains a numeric threshold (coverage %, count, etc.)
    MetricClaim,
    /// Claim asserts something does NOT exist
    AbsenceClaim,
    /// Claim references a command that should work
    CommandClaim,
    /// Structural claim about architecture or patterns
    ArchitecturalClaim,
    /// Claims that cannot be mechanically falsified
    Unfalsifiable,
}

impl std::fmt::Display for SpecClaimCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PathReference => write!(f, "PathRef"),
            Self::CodeEntity => write!(f, "CodeEntity"),
            Self::MetricClaim => write!(f, "Metric"),
            Self::AbsenceClaim => write!(f, "Absence"),
            Self::CommandClaim => write!(f, "Command"),
            Self::ArchitecturalClaim => write!(f, "Arch"),
            Self::Unfalsifiable => write!(f, "Unfalsifiable"),
        }
    }
}

/// A single falsifiable claim extracted from a specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecClaim {
    /// Unique ID within this run
    pub id: String,
    /// Original text from the document
    pub original_text: String,
    /// Source location (file:line)
    pub source_line: usize,
    /// Claim category
    pub category: SpecClaimCategory,
    /// Priority (from RFC-2119 keywords)
    pub priority: ClaimPriority,
    /// Whether the claim uses absolute language ("all", "zero", "every")
    pub is_absolute: bool,
    /// Extracted file path references
    pub path_refs: Vec<String>,
    /// Extracted code entity references (function/struct names)
    pub entity_refs: Vec<String>,
    /// Extracted numeric value (if metric claim)
    pub numeric_value: Option<f64>,
    /// Numeric comparator text (e.g., ">=", "<", "≤")
    pub numeric_comparator: Option<String>,
}

/// Verdict status for a falsified claim
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerdictStatus {
    /// Claim survived falsification — no contradicting evidence found
    Survived,
    /// Claim actively contradicted by evidence
    Falsified,
    /// Claim could not be tested
    Unfalsifiable,
    /// Evidence found but inconclusive
    Inconclusive,
}

impl std::fmt::Display for VerdictStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Survived => write!(f, "SURVIVED"),
            Self::Falsified => write!(f, "FALSIFIED"),
            Self::Unfalsifiable => write!(f, "UNFALSIFIABLE"),
            Self::Inconclusive => write!(f, "INCONCLUSIVE"),
        }
    }
}

/// Evidence collected during falsification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecEvidence {
    /// What was checked
    pub check: String,
    /// What was found
    pub finding: String,
    /// How strongly this contradicts the claim (0.0 = supports, 1.0 = contradicts)
    pub contradiction_score: f64,
}

/// Per-claim verdict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecVerdict {
    pub claim: SpecClaim,
    pub status: VerdictStatus,
    pub evidence: Vec<SpecEvidence>,
    /// Overall contradiction score for this claim
    pub contradiction_score: f64,
}

/// Report summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecFalsificationSummary {
    pub total_claims: usize,
    pub survived: usize,
    pub falsified: usize,
    pub unfalsifiable: usize,
    pub inconclusive: usize,
    /// Health score: survived / (total - unfalsifiable)
    pub health_score: f64,
}

/// Complete falsification report for a spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecFalsificationReport {
    pub target_file: PathBuf,
    pub timestamp: String,
    pub verdicts: Vec<SpecVerdict>,
    pub summary: SpecFalsificationSummary,
}
