//! Event system for quality monitoring

use crate::unified_quality::metrics::Metrics;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Quality events for notifications and tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityEvent {
    /// File added to monitoring
    FileAdded { path: PathBuf, metrics: Metrics },

    /// File removed from monitoring
    FileRemoved {
        path: PathBuf,
        last_metrics: Metrics,
    },

    /// Metrics updated for a file
    MetricsUpdated {
        path: PathBuf,
        old_metrics: Metrics,
        new_metrics: Metrics,
    },

    /// Quality threshold violated
    ThresholdViolated {
        path: PathBuf,
        violation: ThresholdViolation,
    },

    /// Quality improved significantly
    QualityImproved {
        path: PathBuf,
        old_score: f64,
        new_score: f64,
    },

    /// Quality degraded significantly
    QualityDegraded {
        path: PathBuf,
        old_score: f64,
        new_score: f64,
    },

    /// Batch analysis completed
    BatchAnalysisComplete {
        files_analyzed: usize,
        violations_found: usize,
        avg_quality_score: f64,
    },
}

/// Threshold violation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdViolation {
    pub metric: String,
    pub value: f64,
    pub threshold: f64,
    pub severity: ViolationSeverity,
}

/// Violation severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Warning,
    Error,
    Critical,
}

impl QualityEvent {
    /// Check if event is a violation
    #[must_use]
    pub fn is_violation(&self) -> bool {
        matches!(
            self,
            QualityEvent::ThresholdViolated { .. } | QualityEvent::QualityDegraded { .. }
        )
    }

    /// Check if event is an improvement
    #[must_use]
    pub fn is_improvement(&self) -> bool {
        matches!(self, QualityEvent::QualityImproved { .. })
    }

    /// Get event severity
    #[must_use]
    pub fn severity(&self) -> Option<ViolationSeverity> {
        match self {
            QualityEvent::ThresholdViolated { violation, .. } => Some(violation.severity.clone()),
            QualityEvent::QualityDegraded {
                old_score,
                new_score,
                ..
            } => {
                let delta = old_score - new_score;
                if delta > 0.3 {
                    Some(ViolationSeverity::Critical)
                } else if delta > 0.1 {
                    Some(ViolationSeverity::Error)
                } else {
                    Some(ViolationSeverity::Warning)
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use std::time::SystemTime;

    #[test]
    fn test_event_is_violation() {
        let event = QualityEvent::ThresholdViolated {
            path: PathBuf::from("test.rs"),
            violation: ThresholdViolation {
                metric: "complexity".to_string(),
                value: 25.0,
                threshold: 20.0,
                severity: ViolationSeverity::Error,
            },
        };

        assert!(event.is_violation());
        assert!(!event.is_improvement());
    }

    #[test]
    fn test_event_severity() {
        let event = QualityEvent::QualityDegraded {
            path: PathBuf::from("test.rs"),
            old_score: 0.9,
            new_score: 0.5,
        };

        match event.severity() {
            Some(ViolationSeverity::Critical) => (),
            _ => panic!("Expected critical severity"),
        }
    }
}
