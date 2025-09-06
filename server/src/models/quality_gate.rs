//! Quality gate models for the PMAT system

use serde::{Deserialize, Serialize};
use std::fmt;

/// Result of quality gate execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateResult {
    pub violations: Option<u32>,
    pub passed: bool,
}

/// Comprehensive quality gate results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateResults {
    pub passed: bool,
    pub total_violations: usize,
    pub complexity_violations: usize,
    pub dead_code_violations: usize,
    pub satd_violations: usize,
    pub entropy_violations: usize,
    pub security_violations: usize,
    pub duplicate_violations: usize,
    pub coverage_violations: usize,
    pub section_violations: usize,
    pub violations: Vec<QualityViolation>,
}

/// Individual quality violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityViolation {
    pub file: String,
    pub line: Option<usize>,
    pub violation_type: String,
    pub message: String,
    pub severity: ViolationSeverity,
}

/// Severity levels for violations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ViolationSeverity {
    Error,
    Warning,
    Info,
}

impl Default for QualityGateResults {
    fn default() -> Self {
        Self {
            passed: true,
            total_violations: 0,
            complexity_violations: 0,
            dead_code_violations: 0,
            satd_violations: 0,
            entropy_violations: 0,
            security_violations: 0,
            duplicate_violations: 0,
            coverage_violations: 0,
            section_violations: 0,
            violations: Vec::new(),
        }
    }
}

/// Types of quality checks that can be performed
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum QualityCheckType {
    Complexity,
    DeadCode,
    Satd,
    Security,
    Entropy,
    Duplicates,
    Coverage,
}

impl fmt::Display for QualityCheckType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QualityCheckType::Complexity => write!(f, "complexity"),
            QualityCheckType::DeadCode => write!(f, "dead_code"),
            QualityCheckType::Satd => write!(f, "satd"),
            QualityCheckType::Security => write!(f, "security"),
            QualityCheckType::Entropy => write!(f, "entropy"),
            QualityCheckType::Duplicates => write!(f, "duplicates"),
            QualityCheckType::Coverage => write!(f, "coverage"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_gate_result_creation() {
        let result = QualityGateResult {
            violations: Some(5),
            passed: false,
        };
        
        assert_eq!(result.violations, Some(5));
        assert!(!result.passed);
        
        let passing_result = QualityGateResult {
            violations: None,
            passed: true,
        };
        
        assert!(passing_result.violations.is_none());
        assert!(passing_result.passed);
    }

    #[test]
    fn test_quality_gate_results_default() {
        let results = QualityGateResults::default();
        
        assert!(results.passed);
        assert_eq!(results.total_violations, 0);
        assert_eq!(results.complexity_violations, 0);
        assert_eq!(results.dead_code_violations, 0);
        assert_eq!(results.satd_violations, 0);
        assert_eq!(results.entropy_violations, 0);
        assert_eq!(results.security_violations, 0);
        assert_eq!(results.duplicate_violations, 0);
        assert_eq!(results.coverage_violations, 0);
        assert_eq!(results.section_violations, 0);
        assert!(results.violations.is_empty());
    }

    #[test]
    fn test_quality_violation_creation() {
        let violation = QualityViolation {
            file: "test.rs".to_string(),
            line: Some(42),
            violation_type: "complexity".to_string(),
            message: "Function too complex".to_string(),
            severity: ViolationSeverity::Error,
        };
        
        assert_eq!(violation.file, "test.rs");
        assert_eq!(violation.line, Some(42));
        assert_eq!(violation.violation_type, "complexity");
        assert_eq!(violation.message, "Function too complex");
        assert_eq!(violation.severity, ViolationSeverity::Error);
    }

    #[test]
    fn test_violation_severity_equality() {
        assert_eq!(ViolationSeverity::Error, ViolationSeverity::Error);
        assert_eq!(ViolationSeverity::Warning, ViolationSeverity::Warning);
        assert_eq!(ViolationSeverity::Info, ViolationSeverity::Info);
        assert_ne!(ViolationSeverity::Error, ViolationSeverity::Warning);
        assert_ne!(ViolationSeverity::Warning, ViolationSeverity::Info);
    }

    #[test]
    fn test_quality_check_type_display() {
        assert_eq!(QualityCheckType::Complexity.to_string(), "complexity");
        assert_eq!(QualityCheckType::DeadCode.to_string(), "dead_code");
        assert_eq!(QualityCheckType::Satd.to_string(), "satd");
        assert_eq!(QualityCheckType::Security.to_string(), "security");
        assert_eq!(QualityCheckType::Entropy.to_string(), "entropy");
        assert_eq!(QualityCheckType::Duplicates.to_string(), "duplicates");
        assert_eq!(QualityCheckType::Coverage.to_string(), "coverage");
    }

    #[test]
    fn test_quality_check_type_equality() {
        assert_eq!(QualityCheckType::Complexity, QualityCheckType::Complexity);
        assert_ne!(QualityCheckType::Complexity, QualityCheckType::DeadCode);
        
        let check1 = QualityCheckType::Satd;
        let check2 = QualityCheckType::Satd;
        assert_eq!(check1, check2);
    }

    #[test]
    fn test_quality_gate_results_with_violations() {
        let mut results = QualityGateResults::default();
        
        results.passed = false;
        results.total_violations = 3;
        results.complexity_violations = 2;
        results.satd_violations = 1;
        
        results.violations.push(QualityViolation {
            file: "main.rs".to_string(),
            line: Some(10),
            violation_type: "complexity".to_string(),
            message: "Cyclomatic complexity of 25 exceeds limit of 20".to_string(),
            severity: ViolationSeverity::Error,
        });
        
        assert!(!results.passed);
        assert_eq!(results.total_violations, 3);
        assert_eq!(results.violations.len(), 1);
        assert_eq!(results.violations[0].file, "main.rs");
    }

    #[test]
    fn test_quality_violation_without_line_number() {
        let violation = QualityViolation {
            file: "project".to_string(),
            line: None,
            violation_type: "coverage".to_string(),
            message: "Test coverage below 80%".to_string(),
            severity: ViolationSeverity::Warning,
        };
        
        assert_eq!(violation.file, "project");
        assert!(violation.line.is_none());
        assert_eq!(violation.severity, ViolationSeverity::Warning);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let original = QualityGateResults {
            passed: false,
            total_violations: 5,
            complexity_violations: 3,
            dead_code_violations: 2,
            satd_violations: 0,
            entropy_violations: 0,
            security_violations: 0,
            duplicate_violations: 0,
            coverage_violations: 0,
            section_violations: 0,
            violations: vec![
                QualityViolation {
                    file: "test.rs".to_string(),
                    line: Some(100),
                    violation_type: "complexity".to_string(),
                    message: "Too complex".to_string(),
                    severity: ViolationSeverity::Error,
                }
            ],
        };
        
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: QualityGateResults = serde_json::from_str(&json).unwrap();
        
        assert_eq!(original.passed, deserialized.passed);
        assert_eq!(original.total_violations, deserialized.total_violations);
        assert_eq!(original.violations.len(), deserialized.violations.len());
        assert_eq!(original.violations[0].file, deserialized.violations[0].file);
    }
}
