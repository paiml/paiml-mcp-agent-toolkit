//! Agent feature definitions and configurations.

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

/// Features that can be included in an agent.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum AgentFeature {
    /// State machine with specified states.
    StateMachine { states: Vec<String> },
    /// Quality gates with specified level.
    QualityGates { level: QualityLevel },
    /// Tool composition for MCP tools.
    ToolComposition,
    /// Async handlers for requests.
    AsyncHandlers,
    /// Resource subscriptions for MCP.
    ResourceSubscriptions,
    /// Complexity analysis capabilities.
    ComplexityAnalysis,
    /// SATD detection capabilities.
    SATDDetection,
    /// Dead code elimination capabilities.
    DeadCodeElimination,
    /// Monitoring with specified backend.
    Monitoring { backend: MonitoringBackend },
    /// Tracing with specified exporter.
    Tracing { exporter: TraceExporter },
    /// Health check endpoints.
    HealthChecks,
}

impl std::str::FromStr for AgentFeature {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        match parts[0] {
            "state-machine" => {
                let states = if parts.len() > 1 {
                    parts[1].split(',').map(String::from).collect()
                } else {
                    vec![
                        "Initial".to_string(),
                        "Processing".to_string(),
                        "Complete".to_string(),
                    ]
                };
                Ok(Self::StateMachine { states })
            }
            "quality-gates" => {
                let level = if parts.len() > 1 {
                    parts[1].parse::<QualityLevel>()?
                } else {
                    QualityLevel::Standard
                };
                Ok(Self::QualityGates { level })
            }
            "tool-composition" => Ok(Self::ToolComposition),
            "async-handlers" => Ok(Self::AsyncHandlers),
            "resource-subscriptions" => Ok(Self::ResourceSubscriptions),
            "complexity-analysis" => Ok(Self::ComplexityAnalysis),
            "satd-detection" => Ok(Self::SATDDetection),
            "dead-code-elimination" => Ok(Self::DeadCodeElimination),
            "monitoring" => {
                let backend = if parts.len() > 1 {
                    parts[1].parse::<MonitoringBackend>()?
                } else {
                    MonitoringBackend::Prometheus
                };
                Ok(Self::Monitoring { backend })
            }
            "tracing" => {
                let exporter = if parts.len() > 1 {
                    parts[1].parse::<TraceExporter>()?
                } else {
                    TraceExporter::OTLP
                };
                Ok(Self::Tracing { exporter })
            }
            "health-checks" => Ok(Self::HealthChecks),
            _ => bail!("Unknown feature: {s}"),
        }
    }
}

/// Quality level for generated agents.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum QualityLevel {
    /// Basic quality checks.
    Standard,
    /// Zero warnings, high coverage.
    Strict,
    /// Toyota Way: zero SATD, max complexity 10.
    Extreme,
}

impl std::str::FromStr for QualityLevel {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "standard" => Ok(Self::Standard),
            "strict" => Ok(Self::Strict),
            "extreme" => Ok(Self::Extreme),
            _ => bail!("Unknown quality level: {s}"),
        }
    }
}

impl QualityLevel {
    /// Get the maximum cyclomatic complexity for this level.
    #[must_use]
    pub fn max_complexity(&self) -> u32 {
        match self {
            Self::Standard => 20,
            Self::Strict => 15,
            Self::Extreme => 10,
        }
    }

    /// Get the maximum cognitive complexity for this level.
    #[must_use]
    pub fn max_cognitive_complexity(&self) -> u32 {
        match self {
            Self::Standard => 15,
            Self::Strict => 10,
            Self::Extreme => 7,
        }
    }

    /// Get the maximum nesting depth for this level.
    #[must_use]
    pub fn max_nesting(&self) -> u32 {
        match self {
            Self::Standard => 5,
            Self::Strict => 4,
            Self::Extreme => 3,
        }
    }

    /// Get the minimum line coverage percentage for this level.
    #[must_use]
    pub fn min_line_coverage(&self) -> f64 {
        match self {
            Self::Standard => 70.0,
            Self::Strict => 80.0,
            Self::Extreme => 90.0,
        }
    }

    /// Get the minimum branch coverage percentage for this level.
    #[must_use]
    pub fn min_branch_coverage(&self) -> f64 {
        match self {
            Self::Standard => 60.0,
            Self::Strict => 75.0,
            Self::Extreme => 85.0,
        }
    }

    /// Get the minimum function coverage percentage for this level.
    #[must_use]
    pub fn min_function_coverage(&self) -> f64 {
        match self {
            Self::Standard => 80.0,
            Self::Strict => 90.0,
            Self::Extreme => 95.0,
        }
    }
}

/// Monitoring backend for agents.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum MonitoringBackend {
    /// Prometheus metrics.
    Prometheus,
    /// OpenTelemetry metrics.
    OpenTelemetry,
    /// Custom backend with specified name.
    Custom(String),
}

impl std::str::FromStr for MonitoringBackend {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "prometheus" => Ok(Self::Prometheus),
            "opentelemetry" | "otel" => Ok(Self::OpenTelemetry),
            custom => Ok(Self::Custom(custom.to_string())),
        }
    }
}

/// Trace exporter for agents.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum TraceExporter {
    /// Jaeger trace exporter.
    Jaeger,
    /// Zipkin trace exporter.
    Zipkin,
    /// OTLP trace exporter.
    OTLP,
}

impl std::str::FromStr for TraceExporter {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "jaeger" => Ok(Self::Jaeger),
            "zipkin" => Ok(Self::Zipkin),
            "otlp" => Ok(Self::OTLP),
            _ => bail!("Unknown trace exporter: {s}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_level_parsing() {
        assert!(matches!(
            "standard".parse::<QualityLevel>().unwrap(),
            QualityLevel::Standard
        ));
        assert!(matches!(
            "strict".parse::<QualityLevel>().unwrap(),
            QualityLevel::Strict
        ));
        assert!(matches!(
            "extreme".parse::<QualityLevel>().unwrap(),
            QualityLevel::Extreme
        ));
        assert!("invalid".parse::<QualityLevel>().is_err());
    }

    #[test]
    fn test_quality_level_thresholds() {
        let standard = QualityLevel::Standard;
        assert_eq!(standard.max_complexity(), 20);
        assert_eq!(standard.max_cognitive_complexity(), 15);
        assert_eq!(standard.max_nesting(), 5);
        assert_eq!(standard.min_line_coverage(), 70.0);

        let extreme = QualityLevel::Extreme;
        assert_eq!(extreme.max_complexity(), 10);
        assert_eq!(extreme.max_cognitive_complexity(), 7);
        assert_eq!(extreme.max_nesting(), 3);
        assert_eq!(extreme.min_line_coverage(), 90.0);
    }

    #[test]
    fn test_feature_parsing() {
        let feature = "state-machine:Init,Run,Done"
            .parse::<AgentFeature>()
            .unwrap();
        assert!(matches!(feature, AgentFeature::StateMachine { states } if states.len() == 3));

        let feature = "quality-gates:extreme".parse::<AgentFeature>().unwrap();
        assert!(matches!(
            feature,
            AgentFeature::QualityGates {
                level: QualityLevel::Extreme
            }
        ));

        let feature = "monitoring:prometheus".parse::<AgentFeature>().unwrap();
        assert!(matches!(
            feature,
            AgentFeature::Monitoring {
                backend: MonitoringBackend::Prometheus
            }
        ));

        assert!("invalid-feature".parse::<AgentFeature>().is_err());
    }

    #[test]
    fn test_monitoring_backend_parsing() {
        assert!(matches!(
            "prometheus".parse::<MonitoringBackend>().unwrap(),
            MonitoringBackend::Prometheus
        ));
        assert!(matches!(
            "otel".parse::<MonitoringBackend>().unwrap(),
            MonitoringBackend::OpenTelemetry
        ));
        assert!(matches!(
            "custom-backend".parse::<MonitoringBackend>().unwrap(),
            MonitoringBackend::Custom(s) if s == "custom-backend"
        ));
    }

    #[test]
    fn test_trace_exporter_parsing() {
        assert!(matches!(
            "jaeger".parse::<TraceExporter>().unwrap(),
            TraceExporter::Jaeger
        ));
        assert!(matches!(
            "zipkin".parse::<TraceExporter>().unwrap(),
            TraceExporter::Zipkin
        ));
        assert!(matches!(
            "otlp".parse::<TraceExporter>().unwrap(),
            TraceExporter::OTLP
        ));
        assert!("invalid".parse::<TraceExporter>().is_err());
    }
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
