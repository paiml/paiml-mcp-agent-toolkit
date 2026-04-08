impl From<f64> for TDGSeverity {
    fn from(value: f64) -> Self {
        if value > 2.5 {
            TDGSeverity::Critical
        } else if value > 1.5 {
            TDGSeverity::Warning
        } else {
            TDGSeverity::Normal
        }
    }
}

impl TDGSeverity {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// As str.
    pub fn as_str(&self) -> &'static str {
        match self {
            TDGSeverity::Normal => "normal",
            TDGSeverity::Warning => "warning",
            TDGSeverity::Critical => "critical",
        }
    }
}

fn default_dead_code_weight() -> f64 {
    0.20
}

impl Default for TDGConfig {
    fn default() -> Self {
        Self {
            // CB-128: Rebalanced weights to add dead_code component
            // Total still sums to 1.0
            complexity_weight: 0.25, // Was 0.30
            churn_weight: 0.20,      // Was 0.35
            coupling_weight: 0.15,
            domain_risk_weight: 0.10,
            duplication_weight: 0.10,
            dead_code_weight: 0.20, // NEW: CB-128 6th dimension
            critical_threshold: 2.5,
            warning_threshold: 1.5,
        }
    }
}
