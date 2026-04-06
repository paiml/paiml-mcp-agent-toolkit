/// Maximum possible raw points for Rust Project Score
pub const RUST_PROJECT_MAX_POINTS: f64 = 106.0;

// ============================================================================
// ScoringMode - Performance vs Accuracy Tradeoff
// ============================================================================

/// Scoring mode determines speed vs accuracy tradeoff
///
/// Different modes skip or simplify expensive checks for faster results.
/// This enables sub-60s scoring while maintaining option for full analysis.
///
/// Performance targets:
/// - Quick: <10s - Filesystem only, no subprocesses, no external tools
/// - Fast:  <60s - Lightweight checks, minimal subprocess calls (default)
/// - Full:  <5m  - All checks including mutation testing, cargo audit, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ScoringMode {
    /// Quick mode: <10 seconds
    /// - Only filesystem-based heuristics
    /// - No subprocess spawning
    /// - No cargo commands
    /// - Uses simple pattern matching for complexity
    Quick,

    /// Fast mode: <60 seconds (default)
    /// - Skip expensive cargo operations (llvm-cov, mutants, clippy, audit)
    /// - Use heuristics where possible
    /// - Moderate credit for skipped checks
    #[default]
    Fast,

    /// Full mode: <5 minutes
    /// - All checks including mutation testing
    /// - Complete cargo tooling analysis
    /// - Maximum accuracy, slower execution
    Full,
}

impl ScoringMode {
    /// Check if this mode should skip subprocess calls
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn skip_subprocesses(&self) -> bool {
        matches!(self, ScoringMode::Quick)
    }

    /// Check if this mode should skip expensive cargo operations
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn skip_expensive_cargo(&self) -> bool {
        matches!(self, ScoringMode::Quick | ScoringMode::Fast)
    }

    /// Check if full analysis is enabled
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_full(&self) -> bool {
        matches!(self, ScoringMode::Full)
    }
}

impl fmt::Display for ScoringMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScoringMode::Quick => write!(f, "Quick (<10s)"),
            ScoringMode::Fast => write!(f, "Fast (<60s)"),
            ScoringMode::Full => write!(f, "Full (<5m)"),
        }
    }
}
