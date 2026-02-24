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
    pub fn indicator(&self) -> &'static str {
        match self {
            Severity::Critical => "●",
            Severity::High => "◐",
            Severity::Medium => "○",
            Severity::Low => "◌",
        }
    }

    /// Get the ANSI color code for this severity
    pub fn color_code(&self) -> &'static str {
        match self {
            Severity::Critical => "\x1b[31m", // Red
            Severity::High => "\x1b[33m",     // Yellow
            Severity::Medium => "\x1b[36m",   // Cyan
            Severity::Low => "\x1b[2m",       // Dim
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
    pub fn arrow(&self) -> &'static str {
        match self {
            TrendDirection::Improving => "↑",
            TrendDirection::Degrading => "↓",
            TrendDirection::Stable => "→",
        }
    }
}
