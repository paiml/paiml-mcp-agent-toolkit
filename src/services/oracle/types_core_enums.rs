/// Severity level for defects
/// Note: Order is lowest to highest for correct Ord derivation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Severity {
    /// Minor issue, cosmetic or style
    Low,
    /// Moderate impact, workaround exists
    Medium,
    /// Major functionality impact
    High,
    /// Blocks compilation or causes runtime crash
    Critical,
}

/// Source of a quality signal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SignalSource {
    Rustc,
    Clippy,
    CargoTest,
    CargoBuild,
    LlvmCov,
    CargoMutants,
    PmatTdg,
    PmatComplexity,
    PmatSatd,
    PmatDeadCode,
    PmatRustProjectScore,
    PmatFiveWhys,
    PmatChurn,
}

/// Evidence from a signal source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalEvidence {
    pub source: SignalSource,
    pub raw_message: String,
    pub error_code: Option<String>,
    pub weight: f32,
}

/// Code location for a defect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeLocation {
    pub file_path: PathBuf,
    pub line: usize,
    pub column: Option<usize>,
    pub span_end_line: Option<usize>,
}

/// Suggested fix for a defect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedFix {
    pub description: String,
    pub confidence: f32,
    pub fix_type: FixType,
}

/// Type of fix to apply
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FixType {
    /// Use `cargo clippy --fix`
    ClippyAutoFix,
    /// Apply a unified diff patch
    DiffPatch(String),
    /// Simple text replacement
    Replacement { old: String, new: String },
    /// Insert content after a line
    InsertAfter { anchor: String, content: String },
    /// Delete lines
    DeleteLines { start: usize, end: usize },
}

/// Oracle decision for a defect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OracleDecision {
    /// Auto-apply fix (confidence >= threshold)
    AutoApply,
    /// Queue for human review
    HumanReview,
    /// Skip (confidence too low)
    Skip,
}
