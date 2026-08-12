/// Severity level with Andon-style color mapping
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Severity {
    /// Low severity - dimmed in output
    Low,
    /// Medium severity - cyan
    Medium,
    /// High severity - yellow
    High,
    /// Critical severity - red (Andon RED)
    Critical,
}

impl Severity {
    /// Get the ASCII indicator for this severity
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn indicator(&self) -> &'static str {
        match self {
            Severity::Critical => "●",
            Severity::High => "◐",
            Severity::Medium => "○",
            Severity::Low => "◌",
        }
    }

    /// Get the ANSI color code for this severity.
    ///
    /// Returns an [`Sgr`](crate::cli::colors::Sgr), not a `&'static str`: a
    /// `const &str` cannot consult
    /// [`colors_enabled`](crate::cli::colors::colors_enabled), which is why
    /// every printer that interpolated one of these wrote escapes under
    /// `--color never` and into redirected files. Interpolating the returned
    /// value renders nothing when colour is off; use
    /// [`Sgr::raw`](crate::cli::colors::Sgr::raw) where the bytes themselves are
    /// the subject.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn color_code(&self) -> crate::cli::colors::Sgr {
        use crate::cli::colors as c;
        match self {
            Severity::Critical => c::RED,
            Severity::High => c::YELLOW,
            Severity::Medium => c::CYAN,
            Severity::Low => c::DIM,
        }
    }
}

/// Andon status (Toyota Way visual signal)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AndonStatus {
    /// All checks pass - quality target met
    Green,
    /// Minor issues - attention needed
    Yellow,
    /// Critical issues - stop the line (Jidoka)
    Red,
}

impl AndonStatus {
    /// Get the ASCII representation
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn display(&self) -> &'static str {
        match self {
            AndonStatus::Green => "GREEN ✓",
            AndonStatus::Yellow => "YELLOW ⚠",
            AndonStatus::Red => "RED ✗",
        }
    }
}

/// Trend direction for time-series metrics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    /// Improving (value decreasing for costs/times)
    Improving,
    /// Stable (within tolerance)
    Stable,
    /// Degrading (value increasing for costs/times)
    Degrading,
}

impl TrendDirection {
    /// Get the ASCII arrow indicator
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn arrow(&self) -> &'static str {
        match self {
            TrendDirection::Improving => "↑",
            TrendDirection::Degrading => "↓",
            TrendDirection::Stable => "→",
        }
    }
}
