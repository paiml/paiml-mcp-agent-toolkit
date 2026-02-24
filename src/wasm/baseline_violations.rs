// Violation and Severity method implementations
// Included from baseline.rs — shares parent module scope

impl Violation {
    #[must_use]
    pub fn severity(&self) -> &Severity {
        match self {
            Self::ComplexityRegression { severity, .. }
            | Self::ComplexityCreep { severity, .. }
            | Self::QualityErosion { severity, .. }
            | Self::BinarySizeIncrease { severity, .. }
            | Self::PerformanceRegression { severity, .. } => severity,
        }
    }

    #[must_use]
    pub fn description(&self) -> String {
        match self {
            Self::ComplexityRegression { current, limit, .. } => {
                format!("Complexity regression: {current} exceeds limit {limit}")
            }
            Self::ComplexityCreep {
                current, baseline, ..
            } => {
                format!("Complexity creep: {current} exceeds baseline {baseline}")
            }
            Self::QualityErosion { slope, .. } => {
                format!("Quality erosion detected with slope {slope:.2}")
            }
            Self::BinarySizeIncrease {
                increase_percent, ..
            } => {
                format!("Binary size increased by {increase_percent:.1}%")
            }
            Self::PerformanceRegression {
                metric,
                current,
                baseline,
                ..
            } => {
                format!("{metric} regression: {current:.1}ms exceeds baseline {baseline:.1}ms")
            }
        }
    }
}
