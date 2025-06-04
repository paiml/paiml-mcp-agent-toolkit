pub mod checkmake;
pub mod performance;

use super::ast::*;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Performance,
    Info,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Violation {
    pub rule: String,
    pub severity: Severity,
    pub span: SourceSpan,
    pub message: String,
    pub fix_hint: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct LintResult {
    pub path: PathBuf,
    pub violations: Vec<Violation>,
    pub quality_score: f32,
}

impl LintResult {
    pub fn has_errors(&self) -> bool {
        self.violations
            .iter()
            .any(|v| v.severity == Severity::Error)
    }

    pub fn error_count(&self) -> usize {
        self.violations
            .iter()
            .filter(|v| v.severity == Severity::Error)
            .count()
    }

    pub fn max_severity(&self) -> Option<&Severity> {
        self.violations
            .iter()
            .map(|v| &v.severity)
            .max_by_key(|s| match s {
                Severity::Error => 3,
                Severity::Warning => 2,
                Severity::Performance => 1,
                Severity::Info => 0,
            })
    }
}

pub trait MakefileRule: Send + Sync {
    fn id(&self) -> &'static str;

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn check(&self, ast: &MakefileAst) -> Vec<Violation>;

    fn can_fix(&self) -> bool {
        false
    }

    fn fix(&self, _ast: &mut MakefileAst, _violation: &Violation) -> Option<String> {
        None
    }
}

#[derive(Default)]
pub struct RuleRegistry {
    rules: Vec<Box<dyn MakefileRule>>,
}

impl RuleRegistry {
    pub fn new() -> Self {
        let mut registry = Self::default();

        // Register all rules
        registry.register(Box::new(checkmake::MinPhonyRule::default()));
        registry.register(Box::new(checkmake::PhonyDeclaredRule::default()));
        registry.register(Box::new(checkmake::MaxBodyLengthRule::default()));
        registry.register(Box::new(checkmake::TimestampExpandedRule));
        registry.register(Box::new(checkmake::UndefinedVariableRule));
        registry.register(Box::new(performance::RecursiveExpansionRule::default()));
        registry.register(Box::new(checkmake::PortabilityRule));

        registry
    }

    pub fn register(&mut self, rule: Box<dyn MakefileRule>) {
        self.rules.push(rule);
    }

    pub fn check_all(&self, ast: &MakefileAst) -> Vec<Violation> {
        let mut violations = Vec::new();

        for rule in &self.rules {
            violations.extend(rule.check(ast));
        }

        // Sort by severity and line number
        violations.sort_by(|a, b| {
            match (a.severity == Severity::Error, b.severity == Severity::Error) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.span.line.cmp(&b.span.line),
            }
        });

        violations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::makefile_linter::MakefileParser;

    #[test]
    fn test_severity_ordering() {
        // With derive(Ord), the enum variants are ordered by their declaration order
        // Error < Warning < Performance < Info
        assert!(Severity::Error < Severity::Warning);
        assert!(Severity::Warning < Severity::Performance);
        assert!(Severity::Performance < Severity::Info);
    }

    #[test]
    fn test_violation_creation() {
        let violation = Violation {
            rule: "test_rule".to_string(),
            severity: Severity::Warning,
            span: SourceSpan::file_level(),
            message: "Test message".to_string(),
            fix_hint: Some("Fix hint".to_string()),
        };

        assert_eq!(violation.rule, "test_rule");
        assert_eq!(violation.severity, Severity::Warning);
        assert_eq!(violation.message, "Test message");
        assert_eq!(violation.fix_hint, Some("Fix hint".to_string()));
    }

    #[test]
    fn test_rule_registry_new() {
        let registry = RuleRegistry::new();
        // Should have 7 default rules registered
        assert!(registry.rules.len() >= 7);
    }

    #[test]
    fn test_rule_registry_register() {
        struct TestRule;
        impl MakefileRule for TestRule {
            fn id(&self) -> &'static str {
                "test"
            }
            fn check(&self, _ast: &MakefileAst) -> Vec<Violation> {
                vec![]
            }
        }

        let mut registry = RuleRegistry::default();
        let initial_count = registry.rules.len();
        registry.register(Box::new(TestRule));
        assert_eq!(registry.rules.len(), initial_count + 1);
    }

    #[test]
    fn test_check_all_empty_ast() {
        let registry = RuleRegistry::new();
        let ast = MakefileAst::new();
        let violations = registry.check_all(&ast);

        // With empty AST, no violations should be generated
        // MinPhonyRule only warns if targets exist but aren't .PHONY
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_check_all_sorting() {
        struct TestRule;
        impl MakefileRule for TestRule {
            fn id(&self) -> &'static str {
                "test"
            }
            fn check(&self, _ast: &MakefileAst) -> Vec<Violation> {
                vec![
                    Violation {
                        rule: "test".to_string(),
                        severity: Severity::Info,
                        span: SourceSpan::new(0, 10, 5, 1),
                        message: "Info on line 5".to_string(),
                        fix_hint: None,
                    },
                    Violation {
                        rule: "test".to_string(),
                        severity: Severity::Error,
                        span: SourceSpan::new(20, 30, 10, 1),
                        message: "Error on line 10".to_string(),
                        fix_hint: None,
                    },
                    Violation {
                        rule: "test".to_string(),
                        severity: Severity::Warning,
                        span: SourceSpan::new(10, 20, 3, 1),
                        message: "Warning on line 3".to_string(),
                        fix_hint: None,
                    },
                ]
            }
        }

        let mut registry = RuleRegistry::default();
        registry.register(Box::new(TestRule));
        let ast = MakefileAst::new();
        let violations = registry.check_all(&ast);

        // First violation should be the error
        let error_violations: Vec<_> = violations
            .iter()
            .filter(|v| v.severity == Severity::Error)
            .collect();
        assert!(!error_violations.is_empty());

        // Check that errors come before non-errors
        let first_error_idx = violations
            .iter()
            .position(|v| v.severity == Severity::Error);
        let last_non_error_idx = violations
            .iter()
            .rposition(|v| v.severity != Severity::Error);

        if let (Some(err_idx), Some(non_err_idx)) = (first_error_idx, last_non_error_idx) {
            assert!(
                err_idx < non_err_idx || violations.iter().all(|v| v.severity == Severity::Error)
            );
        }
    }

    #[test]
    fn test_default_trait_implementation() {
        struct MinimalRule;
        impl MakefileRule for MinimalRule {
            fn id(&self) -> &'static str {
                "minimal"
            }
            fn check(&self, _ast: &MakefileAst) -> Vec<Violation> {
                vec![]
            }
        }

        let rule = MinimalRule;
        assert_eq!(rule.default_severity(), Severity::Warning);
        assert!(!rule.can_fix());
        assert!(rule
            .fix(
                &mut MakefileAst::new(),
                &Violation {
                    rule: "test".to_string(),
                    severity: Severity::Warning,
                    span: SourceSpan::file_level(),
                    message: "Test".to_string(),
                    fix_hint: None,
                }
            )
            .is_none());
    }

    #[test]
    fn test_makefile_with_phony_targets() {
        let input =
            ".PHONY: all clean test\nall:\n\techo all\nclean:\n\trm -f *.o\ntest:\n\tpytest";
        let mut parser = MakefileParser::new(input);
        let ast = parser.parse().unwrap();

        let registry = RuleRegistry::new();
        let violations = registry.check_all(&ast);

        // Should not have minphony violations since all required targets are .PHONY
        assert!(!violations.iter().any(|v| v.rule == "minphony"));
    }
}
