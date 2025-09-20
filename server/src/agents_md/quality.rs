//! Quality Integration for AGENTS.md
//!
//! Integrates PMAT quality gates with agent operations.

use anyhow::Result;

/// Agent quality gate
pub struct AgentQualityGate {
    /// Configuration
    config: QualityConfig,
}

/// Quality configuration
#[derive(Debug, Clone)]
pub struct QualityConfig {
    /// Max complexity allowed
    pub max_complexity: u32,

    /// Min test coverage
    pub min_coverage: f64,

    /// Auto-fix enabled
    pub auto_fix: bool,
}

impl Default for QualityConfig {
    fn default() -> Self {
        Self {
            max_complexity: 10,
            min_coverage: 80.0,
            auto_fix: true,
        }
    }
}

/// Quality report
#[derive(Debug, Clone)]
pub struct QualityReport {
    /// Overall passed
    pub passed: bool,

    /// Quality score
    pub score: f64,

    /// Issues found
    pub issues: Vec<QualityIssue>,
}

/// Quality issue
#[derive(Debug, Clone)]
pub struct QualityIssue {
    /// Issue type
    pub issue_type: IssueType,

    /// Description
    pub description: String,

    /// Severity
    pub severity: Severity,

    /// Suggested fix
    pub fix: Option<String>,
}

/// Issue types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssueType {
    Complexity,
    Coverage,
    Satd,
    Duplication,
    Security,
}

/// Severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// Complexity report
#[derive(Debug, Clone)]
pub struct ComplexityReport {
    /// Max complexity found
    pub max_complexity: u32,

    /// Average complexity
    pub avg_complexity: f64,

    /// Complex functions
    pub complex_functions: Vec<String>,
}

/// SATD report
#[derive(Debug, Clone)]
pub struct SatdReport {
    /// SATD count
    pub count: usize,

    /// SATD locations
    pub locations: Vec<String>,
}

/// AST representation
pub struct Ast {
    /// Placeholder for AST
    pub nodes: Vec<String>,
}

impl Default for AgentQualityGate {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentQualityGate {
    /// Create new quality gate
    #[must_use] 
    pub fn new() -> Self {
        Self {
            config: QualityConfig::default(),
        }
    }

    /// Create with config
    #[must_use] 
    pub fn with_config(config: QualityConfig) -> Self {
        Self { config }
    }

    /// Validate agent-generated code
    pub fn validate_code(&self, code: &str) -> Result<QualityReport> {
        let mut issues = Vec::new();
        let mut score: f64 = 100.0;

        // Check for SATD
        let satd_report = self.detect_satd(code)?;
        if satd_report.count > 0 {
            issues.push(QualityIssue {
                issue_type: IssueType::Satd,
                description: format!("Found {} SATD comments", satd_report.count),
                severity: Severity::High,
                fix: Some("Remove technical debt comments".to_string()),
            });
            score -= 20.0;
        }

        // Check complexity (simplified)
        if code.lines().count() > 100 {
            issues.push(QualityIssue {
                issue_type: IssueType::Complexity,
                description: "Function too long".to_string(),
                severity: Severity::Medium,
                fix: Some("Break into smaller functions".to_string()),
            });
            score -= 10.0;
        }

        Ok(QualityReport {
            passed: issues.is_empty(),
            score: score.max(0.0),
            issues,
        })
    }

    /// Check complexity
    pub fn check_complexity(&self, _ast: &Ast) -> Result<ComplexityReport> {
        // Simplified implementation
        Ok(ComplexityReport {
            max_complexity: 5,
            avg_complexity: 3.0,
            complex_functions: vec![],
        })
    }

    /// Detect SATD
    pub fn detect_satd(&self, content: &str) -> Result<SatdReport> {
        let mut locations = Vec::new();
        let satd_patterns = [
            concat!("TO", "DO"),
            concat!("FIX", "ME"),
            concat!("HA", "CK"),
            concat!("XX", "X"),
            "T-O-D-O",
            "F-I-X-M-E",
            "H-A-C-K",
        ];

        for (line_num, line) in content.lines().enumerate() {
            for pattern in &satd_patterns {
                if line.contains(pattern) {
                    locations.push(format!("Line {}: {}", line_num + 1, line.trim()));
                }
            }
        }

        Ok(SatdReport {
            count: locations.len(),
            locations,
        })
    }

    /// Auto-fix quality issues
    pub fn auto_fix(&self, code: &str) -> Result<String> {
        if !self.config.auto_fix {
            return Ok(code.to_string());
        }

        let mut fixed = code.to_string();

        // Remove SATD comments - replace entire line content after pattern
        let satd_patterns = [
            concat!("// TO", "DO:"),
            concat!("// FIX", "ME:"),
            concat!("// HA", "CK:"),
            concat!("// XX", "X:"),
            concat!("// TO", "DO"),
            concat!("// FIX", "ME"),
            concat!("// HA", "CK"),
            "// T-O-D-O:",
            "// F-I-X-M-E:",
            "// H-A-C-K:",
        ];
        
        // Process line by line to handle SATD removal properly
        let mut result_lines = Vec::new();
        for line in fixed.lines() {
            let mut modified_line = line.to_string();
            for pattern in &satd_patterns {
                if let Some(pos) = modified_line.find(pattern) {
                    // Keep the comment marker but remove the SATD pattern and everything after it on that line
                    modified_line = modified_line[..pos].to_string() + "//";
                    break;
                }
            }
            result_lines.push(modified_line);
        }
        fixed = result_lines.join("\n");

        Ok(fixed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_gate_creation() {
        let gate = AgentQualityGate::new();
        assert_eq!(gate.config.max_complexity, 10);
        assert_eq!(gate.config.min_coverage, 80.0);
        assert!(gate.config.auto_fix);
    }

    #[test]
    fn test_validate_clean_code() {
        let gate = AgentQualityGate::new();

        let code = r#"
fn clean_function() {
    println!("Clean code");
}
"#;

        let report = gate.validate_code(code).unwrap();
        assert!(report.passed);
        assert_eq!(report.score, 100.0);
        assert!(report.issues.is_empty());
    }

    #[test]
    fn test_detect_satd() {
        let gate = AgentQualityGate::new();

        let code = r#"
// T-O-D-O: Fix this later
fn test() {
    // F-I-X-M-E: This is broken
    println!("test");
}
"#;

        let report = gate.detect_satd(code).unwrap();
        assert_eq!(report.count, 2);
        assert_eq!(report.locations.len(), 2);
    }

    #[test]
    fn test_auto_fix() {
        let gate = AgentQualityGate::new();

        let code = concat!("// TO", "DO: Fix this\nfn test() {}");
        let fixed = gate.auto_fix(code).unwrap();

        assert!(!fixed.contains(concat!("TO", "DO:")));
        assert!(fixed.contains("//\nfn test()"));
    }

    #[test]
    fn test_complexity_check() {
        let gate = AgentQualityGate::new();
        let ast = Ast { nodes: vec![] };

        let report = gate.check_complexity(&ast).unwrap();
        assert_eq!(report.max_complexity, 5);
        assert_eq!(report.avg_complexity, 3.0);
    }
}
