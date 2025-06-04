pub mod ast;
pub mod parser;
pub mod rules;

use crate::models::error::AnalysisError;
use std::path::Path;

pub use self::ast::{MakefileAst, MakefileNode, MakefileNodeKind};
pub use self::parser::MakefileParser;
pub use self::rules::{LintResult, MakefileRule, RuleRegistry, Severity, Violation};

/// Main entry point for linting a Makefile
pub async fn lint_makefile(path: &Path) -> Result<LintResult, AnalysisError> {
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(AnalysisError::Io)?;

    let mut parser = MakefileParser::new(&content);
    let ast = parser
        .parse()
        .map_err(|e| AnalysisError::ParseError(format!("Makefile parse error: {:?}", e)))?;

    let registry = RuleRegistry::new();
    let violations = registry.check_all(&ast);
    let quality_score = calculate_quality_score(&violations);

    Ok(LintResult {
        path: path.to_path_buf(),
        violations,
        quality_score,
    })
}

fn calculate_quality_score(violations: &[Violation]) -> f32 {
    let critical_count = violations
        .iter()
        .filter(|v| v.severity == Severity::Error)
        .count();
    let warning_count = violations
        .iter()
        .filter(|v| v.severity == Severity::Warning)
        .count();
    let info_count = violations
        .iter()
        .filter(|v| v.severity == Severity::Info)
        .count();

    let score = 1.0
        - (critical_count as f32 * 0.3)
        - (warning_count as f32 * 0.1)
        - (info_count as f32 * 0.02);

    score.max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::makefile_linter::ast::SourceSpan;
    use std::path::PathBuf;

    #[test]
    fn test_calculate_quality_score_perfect() {
        let violations = vec![];
        assert_eq!(calculate_quality_score(&violations), 1.0);
    }

    #[test]
    fn test_calculate_quality_score_with_errors() {
        let violations = vec![Violation {
            rule: "test".to_string(),
            severity: Severity::Error,
            span: SourceSpan::file_level(),
            message: "Test error".to_string(),
            fix_hint: None,
        }];
        assert_eq!(calculate_quality_score(&violations), 0.7);
    }

    #[test]
    fn test_calculate_quality_score_with_warnings() {
        let violations = vec![Violation {
            rule: "test".to_string(),
            severity: Severity::Warning,
            span: SourceSpan::file_level(),
            message: "Test warning".to_string(),
            fix_hint: None,
        }];
        assert_eq!(calculate_quality_score(&violations), 0.9);
    }

    #[test]
    fn test_calculate_quality_score_minimum() {
        let mut violations = vec![];
        for i in 0..10 {
            violations.push(Violation {
                rule: "test".to_string(),
                severity: Severity::Error,
                span: SourceSpan::file_level(),
                message: format!("Test error {}", i),
                fix_hint: None,
            });
        }
        assert_eq!(calculate_quality_score(&violations), 0.0);
    }

    #[test]
    fn test_lint_result_methods() {
        let violations = vec![
            Violation {
                rule: "error_rule".to_string(),
                severity: Severity::Error,
                span: SourceSpan::file_level(),
                message: "Error".to_string(),
                fix_hint: None,
            },
            Violation {
                rule: "warning_rule".to_string(),
                severity: Severity::Warning,
                span: SourceSpan::file_level(),
                message: "Warning".to_string(),
                fix_hint: None,
            },
        ];

        let result = LintResult {
            path: PathBuf::from("test.mk"),
            violations: violations.clone(),
            quality_score: 0.6,
        };

        assert!(result.has_errors());
        assert_eq!(result.error_count(), 1);
        assert_eq!(result.max_severity(), Some(&Severity::Error));
    }

    #[tokio::test]
    async fn test_lint_makefile_async() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create a temporary Makefile
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "all:").unwrap();
        writeln!(temp_file, "\techo hello").unwrap();

        let result = lint_makefile(temp_file.path()).await;
        assert!(result.is_ok());

        let lint_result = result.unwrap();
        assert_eq!(lint_result.path, temp_file.path());

        // Should have violations for missing .PHONY
        assert!(lint_result.violations.iter().any(|v| v.rule == "minphony"));
    }

    #[tokio::test]
    async fn test_lint_makefile_file_not_found() {
        let result = lint_makefile(Path::new("/nonexistent/makefile")).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AnalysisError::Io(_)));
    }
}
