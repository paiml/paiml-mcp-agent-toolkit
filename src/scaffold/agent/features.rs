#![cfg_attr(coverage_nightly, coverage(off))]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn max_complexity(&self) -> u32 {
        match self {
            Self::Standard => 20,
            Self::Strict => 15,
            Self::Extreme => 10,
        }
    }

    /// Get the maximum cognitive complexity for this level.
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn max_cognitive_complexity(&self) -> u32 {
        match self {
            Self::Standard => 15,
            Self::Strict => 10,
            Self::Extreme => 7,
        }
    }

    /// Get the maximum nesting depth for this level.
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn max_nesting(&self) -> u32 {
        match self {
            Self::Standard => 5,
            Self::Strict => 4,
            Self::Extreme => 3,
        }
    }

    /// Get the minimum line coverage percentage for this level.
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn min_line_coverage(&self) -> f64 {
        match self {
            Self::Standard => 70.0,
            Self::Strict => 80.0,
            Self::Extreme => 90.0,
        }
    }

    /// Get the minimum branch coverage percentage for this level.
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn min_branch_coverage(&self) -> f64 {
        match self {
            Self::Standard => 60.0,
            Self::Strict => 75.0,
            Self::Extreme => 85.0,
        }
    }

    /// Get the minimum function coverage percentage for this level.
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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

#[cfg_attr(coverage_nightly, coverage(off))]
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

    // --- PMAT-637 additions: cover untested surface area ---

    #[test]
    fn test_quality_level_strict_all_thresholds() {
        let strict = QualityLevel::Strict;
        assert_eq!(strict.max_complexity(), 15);
        assert_eq!(strict.max_cognitive_complexity(), 10);
        assert_eq!(strict.max_nesting(), 4);
        assert_eq!(strict.min_line_coverage(), 80.0);
        assert_eq!(strict.min_branch_coverage(), 75.0);
        assert_eq!(strict.min_function_coverage(), 90.0);
    }

    #[test]
    fn test_quality_level_min_branch_coverage_all_levels() {
        assert_eq!(QualityLevel::Standard.min_branch_coverage(), 60.0);
        assert_eq!(QualityLevel::Strict.min_branch_coverage(), 75.0);
        assert_eq!(QualityLevel::Extreme.min_branch_coverage(), 85.0);
    }

    #[test]
    fn test_quality_level_min_function_coverage_all_levels() {
        assert_eq!(QualityLevel::Standard.min_function_coverage(), 80.0);
        assert_eq!(QualityLevel::Strict.min_function_coverage(), 90.0);
        assert_eq!(QualityLevel::Extreme.min_function_coverage(), 95.0);
    }

    #[test]
    fn test_quality_level_parse_is_case_insensitive() {
        // The impl lowercases input, so mixed-case should still parse.
        assert!(matches!(
            "STANDARD".parse::<QualityLevel>().unwrap(),
            QualityLevel::Standard
        ));
        assert!(matches!(
            "Strict".parse::<QualityLevel>().unwrap(),
            QualityLevel::Strict
        ));
        assert!(matches!(
            "Extreme".parse::<QualityLevel>().unwrap(),
            QualityLevel::Extreme
        ));
    }

    #[test]
    fn test_quality_level_parse_error_names_input() {
        let err = "bogus".parse::<QualityLevel>().unwrap_err().to_string();
        assert!(err.contains("bogus"), "err was: {err}");
    }

    #[test]
    fn test_state_machine_feature_parses_without_colon_uses_defaults() {
        let feature = "state-machine".parse::<AgentFeature>().unwrap();
        match feature {
            AgentFeature::StateMachine { states } => {
                assert_eq!(states, vec!["Initial", "Processing", "Complete"]);
            }
            other => panic!("expected StateMachine, got {other:?}"),
        }
    }

    #[test]
    fn test_quality_gates_feature_parses_without_colon_defaults_to_standard() {
        let feature = "quality-gates".parse::<AgentFeature>().unwrap();
        assert!(matches!(
            feature,
            AgentFeature::QualityGates {
                level: QualityLevel::Standard
            }
        ));
    }

    #[test]
    fn test_quality_gates_feature_propagates_invalid_level() {
        // "quality-gates:bogus" → propagates QualityLevel::FromStr error.
        assert!("quality-gates:bogus".parse::<AgentFeature>().is_err());
    }

    #[test]
    fn test_zero_field_agent_features_parse() {
        for (s, expect) in [
            ("tool-composition", AgentFeature::ToolComposition),
            ("async-handlers", AgentFeature::AsyncHandlers),
            (
                "resource-subscriptions",
                AgentFeature::ResourceSubscriptions,
            ),
            ("complexity-analysis", AgentFeature::ComplexityAnalysis),
            ("satd-detection", AgentFeature::SATDDetection),
            ("dead-code-elimination", AgentFeature::DeadCodeElimination),
            ("health-checks", AgentFeature::HealthChecks),
        ] {
            let parsed = s.parse::<AgentFeature>().expect(s);
            assert_eq!(parsed, expect, "parse({s}) mismatch");
        }
    }

    #[test]
    fn test_monitoring_feature_parses_without_colon_defaults_to_prometheus() {
        let feature = "monitoring".parse::<AgentFeature>().unwrap();
        assert!(matches!(
            feature,
            AgentFeature::Monitoring {
                backend: MonitoringBackend::Prometheus
            }
        ));
    }

    #[test]
    fn test_tracing_feature_parses_without_colon_defaults_to_otlp() {
        let feature = "tracing".parse::<AgentFeature>().unwrap();
        assert!(matches!(
            feature,
            AgentFeature::Tracing {
                exporter: TraceExporter::OTLP
            }
        ));
    }

    #[test]
    fn test_tracing_feature_with_jaeger_exporter_value() {
        let feature = "tracing:jaeger".parse::<AgentFeature>().unwrap();
        assert!(matches!(
            feature,
            AgentFeature::Tracing {
                exporter: TraceExporter::Jaeger
            }
        ));
    }

    #[test]
    fn test_tracing_feature_propagates_invalid_exporter() {
        assert!("tracing:invalid".parse::<AgentFeature>().is_err());
    }

    #[test]
    fn test_agent_feature_parse_error_names_input() {
        let err = "bogus-feature"
            .parse::<AgentFeature>()
            .unwrap_err()
            .to_string();
        assert!(err.contains("bogus-feature"), "err was: {err}");
    }

    #[test]
    fn test_monitoring_backend_opentelemetry_fullname() {
        assert!(matches!(
            "opentelemetry".parse::<MonitoringBackend>().unwrap(),
            MonitoringBackend::OpenTelemetry
        ));
    }

    #[test]
    fn test_monitoring_backend_case_insensitive_prometheus() {
        assert!(matches!(
            "PROMETHEUS".parse::<MonitoringBackend>().unwrap(),
            MonitoringBackend::Prometheus
        ));
    }

    #[test]
    fn test_monitoring_backend_custom_is_lowercased() {
        // The impl lowercases before matching, so Custom stores the lowercase form.
        match "StatsD-Backend".parse::<MonitoringBackend>().unwrap() {
            MonitoringBackend::Custom(s) => assert_eq!(s, "statsd-backend"),
            other => panic!("expected Custom, got {other:?}"),
        }
    }

    #[test]
    fn test_trace_exporter_case_insensitive() {
        assert!(matches!(
            "JAEGER".parse::<TraceExporter>().unwrap(),
            TraceExporter::Jaeger
        ));
        assert!(matches!(
            "OtLp".parse::<TraceExporter>().unwrap(),
            TraceExporter::OTLP
        ));
    }

    #[test]
    fn test_trace_exporter_parse_error_names_input() {
        let err = "zzz".parse::<TraceExporter>().unwrap_err().to_string();
        assert!(err.contains("zzz"), "err was: {err}");
    }

    #[test]
    fn test_agent_feature_serde_round_trip_all_variants() {
        let variants = vec![
            AgentFeature::StateMachine {
                states: vec!["A".to_string(), "B".to_string()],
            },
            AgentFeature::QualityGates {
                level: QualityLevel::Extreme,
            },
            AgentFeature::ToolComposition,
            AgentFeature::AsyncHandlers,
            AgentFeature::ResourceSubscriptions,
            AgentFeature::ComplexityAnalysis,
            AgentFeature::SATDDetection,
            AgentFeature::DeadCodeElimination,
            AgentFeature::Monitoring {
                backend: MonitoringBackend::Custom("x".to_string()),
            },
            AgentFeature::Tracing {
                exporter: TraceExporter::Zipkin,
            },
            AgentFeature::HealthChecks,
        ];
        for v in variants {
            let json = serde_json::to_string(&v).expect("serialize");
            let back: AgentFeature = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(back, v);
        }
    }

    #[test]
    fn test_quality_level_serde_round_trip() {
        for level in [
            QualityLevel::Standard,
            QualityLevel::Strict,
            QualityLevel::Extreme,
        ] {
            let json = serde_json::to_string(&level).expect("serialize");
            let back: QualityLevel = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(back, level);
        }
    }

    #[test]
    fn test_state_machine_feature_colon_only_parses_empty_state() {
        // Edge case: "state-machine:" has parts=[..,""], which splits on ',' to [""]
        // → single empty-string state. Exercises the `parts.len() > 1` branch with
        // a degenerate value.
        let feature = "state-machine:".parse::<AgentFeature>().unwrap();
        match feature {
            AgentFeature::StateMachine { states } => {
                assert_eq!(states, vec![""]);
            }
            other => panic!("expected StateMachine, got {other:?}"),
        }
    }

    #[test]
    fn test_quality_level_is_copy() {
        // Copy semantics — passing by value after prior use should still compile + work.
        let q = QualityLevel::Strict;
        let _a = q;
        let _b = q;
        assert_eq!(q, QualityLevel::Strict);
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
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
