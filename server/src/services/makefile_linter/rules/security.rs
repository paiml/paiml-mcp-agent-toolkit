//! Security rules for Makefile linting
//!
//! Implements high-priority security rules following TDD:
//! - ShellInjectionRule: Detect potential shell injection vulnerabilities
//! - SensitiveDataRule: Detect hardcoded secrets and credentials
//! - UnsafeCommandRule: Detect unsafe command usage
//! - PrivilegeEscalationRule: Detect potential privilege escalation

use super::{MakefileRule, Severity, Violation};
use crate::services::makefile_linter::ast::{MakefileAst, NodeData};
use regex::Regex;
use std::sync::OnceLock;

/// Detects potential shell injection vulnerabilities
#[derive(Debug, Default)]
pub struct ShellInjectionRule;

impl MakefileRule for ShellInjectionRule {
    fn id(&self) -> &'static str {
        "security/shell-injection"
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn check(&self, ast: &MakefileAst) -> Vec<Violation> {
        let mut violations = Vec::new();

        // Check all recipe lines for unquoted variables
        for node in &ast.nodes {
            if let NodeData::Recipe { lines } = &node.data {
                for line in lines {
                    if contains_shell_injection(&line.text) {
                        violations.push(Violation {
                            rule: self.id().to_string(),
                            severity: self.default_severity(),
                            span: node.span.clone(),
                            message: format!(
                                "Potential shell injection: unquoted variable in command '{}'",
                                truncate_command(&line.text)
                            ),
                            fix_hint: Some(quote_variables(&line.text)),
                        });
                    }
                }
            }
        }

        violations
    }

    fn can_fix(&self) -> bool {
        true
    }

    fn fix(&self, _ast: &mut MakefileAst, violation: &Violation) -> Option<String> {
        violation.fix_hint.clone()
    }
}

/// Detects hardcoded secrets and credentials
#[derive(Debug, Default)]
pub struct SensitiveDataRule;

impl MakefileRule for SensitiveDataRule {
    fn id(&self) -> &'static str {
        "security/sensitive-data"
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn check(&self, ast: &MakefileAst) -> Vec<Violation> {
        let mut violations = Vec::new();

        // Check variables for sensitive data
        for node in &ast.nodes {
            match &node.data {
                NodeData::Variable {
                    name,
                    value,
                    assignment_op: _,
                } => {
                    if let Some(secret_type) = detect_secret(name, value) {
                        violations.push(Violation {
                            rule: self.id().to_string(),
                            severity: self.default_severity(),
                            span: node.span.clone(),
                            message: format!(
                                "Hardcoded {} detected in variable '{}'",
                                secret_type, name
                            ),
                            fix_hint: Some(format!(
                                "{} = $${{{}}}  # Use environment variable",
                                name, name
                            )),
                        });
                    }
                }
                NodeData::Recipe { lines } => {
                    for line in lines {
                        if let Some(secret_type) = detect_secret_in_command(&line.text) {
                            violations.push(Violation {
                                rule: self.id().to_string(),
                                severity: self.default_severity(),
                                span: node.span.clone(),
                                message: format!(
                                    "Hardcoded {} in command: '{}'",
                                    secret_type,
                                    truncate_command(&line.text)
                                ),
                                fix_hint: Some(
                                    "Store secret in environment variable or secure vault"
                                        .to_string(),
                                ),
                            });
                        }
                    }
                }
                _ => {}
            }
        }

        violations
    }
}

/// Detects unsafe command usage
#[derive(Debug, Default)]
pub struct UnsafeCommandRule;

impl MakefileRule for UnsafeCommandRule {
    fn id(&self) -> &'static str {
        "security/unsafe-command"
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn check(&self, ast: &MakefileAst) -> Vec<Violation> {
        let mut violations = Vec::new();

        for node in &ast.nodes {
            if let NodeData::Recipe { lines } = &node.data {
                for line in lines {
                    if let Some((pattern, severity)) = detect_unsafe_command(&line.text) {
                        violations.push(Violation {
                            rule: self.id().to_string(),
                            severity,
                            span: node.span.clone(),
                            message: format!(
                                "Unsafe command pattern detected: {}",
                                pattern
                            ),
                            fix_hint: Some(suggest_safe_alternative(pattern)),
                        });
                    }
                }
            }
        }

        violations
    }
}

/// Detects potential privilege escalation
#[derive(Debug, Default)]
pub struct PrivilegeEscalationRule;

impl MakefileRule for PrivilegeEscalationRule {
    fn id(&self) -> &'static str {
        "security/privilege-escalation"
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn check(&self, ast: &MakefileAst) -> Vec<Violation> {
        let mut violations = Vec::new();

        for node in &ast.nodes {
            if let NodeData::Recipe { lines } = &node.data {
                for line in lines {
                    if let Some(issue) = detect_privilege_escalation(&line.text) {
                        violations.push(Violation {
                            rule: self.id().to_string(),
                            severity: self.default_severity(),
                            span: node.span.clone(),
                            message: format!(
                                "Potential privilege escalation: {}",
                                issue
                            ),
                            fix_hint: Some(
                                "Review privilege requirements and use least privilege principle"
                                    .to_string(),
                            ),
                        });
                    }
                }
            }
        }

        violations
    }
}

// Helper functions

fn contains_shell_injection(command: &str) -> bool {
    static UNQUOTED_VAR_REGEX: OnceLock<Regex> = OnceLock::new();
    let regex = UNQUOTED_VAR_REGEX.get_or_init(|| {
        // Match $(VAR) or ${VAR} not inside quotes
        Regex::new(r#"(?:^|[^"'])\$\([^)]+\)|(?:^|[^"'])\$\{[^}]+\}"#).unwrap()
    });

    // Skip safe commands
    if command.trim_start().starts_with('@') {
        let cmd = &command.trim_start()[1..];
        if cmd.starts_with("echo") || cmd.starts_with("printf") {
            return false;
        }
    }

    // Check for dangerous patterns
    let dangerous_commands = ["rm", "find", "curl", "wget", "tar", "install"];
    let has_dangerous = dangerous_commands
        .iter()
        .any(|&cmd| command.contains(cmd));

    if has_dangerous && regex.is_match(command) {
        // Check if it's not already quoted
        !command.contains('"') || regex.is_match(command)
    } else {
        false
    }
}

fn quote_variables(command: &str) -> String {
    static VAR_REGEX: OnceLock<Regex> = OnceLock::new();
    let regex = VAR_REGEX.get_or_init(|| Regex::new(r#"\$\(([^)]+)\)"#).unwrap());

    regex
        .replace_all(command, "\"$($1)\"")
        .to_string()
}

fn detect_secret(name: &str, value: &str) -> Option<String> {
    let name_lower = name.to_lowercase();
    let value_trimmed = value.trim();

    // Check for common secret variable names
    if name_lower.contains("password")
        || name_lower.contains("secret")
        || name_lower.contains("token")
        || name_lower.contains("api_key")
        || name_lower.contains("access_key")
    {
        // Check if it's not a variable reference
        if !value_trimmed.starts_with('$') && value_trimmed.len() > 4 {
            return Some("credential".to_string());
        }
    }

    // Check for AWS credentials pattern
    if value_trimmed.starts_with("AKIA") && value_trimmed.len() == 20 {
        return Some("AWS access key".to_string());
    }

    // Check for GitHub token pattern
    if value_trimmed.starts_with("ghp_") || value_trimmed.starts_with("github_pat_") {
        return Some("GitHub token".to_string());
    }

    // Check for JWT pattern
    if value_trimmed.starts_with("eyJ") {
        return Some("JWT token".to_string());
    }

    // Check for API key patterns
    if value_trimmed.starts_with("sk-") || value_trimmed.starts_with("pk-") {
        return Some("API key".to_string());
    }

    None
}

fn detect_secret_in_command(command: &str) -> Option<String> {
    // Check for Bearer tokens
    if command.contains("Bearer ") {
        if let Some(pos) = command.find("Bearer ") {
            let token_part = &command[pos + 7..];
            if !token_part.starts_with('$') && token_part.len() > 10 {
                return Some("Bearer token".to_string());
            }
        }
    }

    // Check for Authorization headers
    if command.contains("Authorization:") {
        return Some("authorization credential".to_string());
    }

    None
}

fn detect_unsafe_command(command: &str) -> Option<(&'static str, Severity)> {
    let cmd_trimmed = command.trim();

    // Critical: rm -rf /
    if cmd_trimmed.contains("rm ") && cmd_trimmed.contains("-rf /") {
        if cmd_trimmed.contains("-rf /") || cmd_trimmed.contains("-rf /*") {
            return Some(("rm -rf / - extremely dangerous", Severity::Error));
        }
    }

    // High: curl | bash pattern
    if (cmd_trimmed.contains("curl") || cmd_trimmed.contains("wget"))
        && (cmd_trimmed.contains("| bash") || cmd_trimmed.contains("| sh"))
    {
        return Some(("downloading and piping to shell", Severity::Error));
    }

    // High: eval with untrusted input
    if cmd_trimmed.starts_with("eval") && cmd_trimmed.contains('$') {
        return Some(("eval with variable input", Severity::Error));
    }

    // High: chmod 777
    if cmd_trimmed.contains("chmod 777") {
        return Some(("chmod 777 - overly permissive", Severity::Warning));
    }

    None
}

fn detect_privilege_escalation(command: &str) -> Option<String> {
    let cmd_trimmed = command.trim();

    // Check for sudo with variables
    if cmd_trimmed.starts_with("sudo ") && cmd_trimmed.contains('$') {
        return Some("sudo with untrusted variable input".to_string());
    }

    // Check for setuid
    if cmd_trimmed.contains("chmod") && cmd_trimmed.contains("+s") {
        return Some("creating setuid binary".to_string());
    }

    // Check for su command
    if cmd_trimmed.starts_with("su ") {
        return Some("using su command".to_string());
    }

    // Check for pkexec
    if cmd_trimmed.contains("pkexec") {
        return Some("using pkexec for privilege escalation".to_string());
    }

    None
}

fn truncate_command(command: &str) -> &str {
    if command.len() > 50 {
        &command[..50]
    } else {
        command
    }
}

fn suggest_safe_alternative(pattern: &str) -> String {
    match pattern {
        "rm -rf / - extremely dangerous" => {
            "Use specific paths and add safety checks".to_string()
        }
        "downloading and piping to shell" => {
            "Download file first, verify checksum, then execute".to_string()
        }
        "eval with variable input" => "Avoid eval; use direct command execution".to_string(),
        "chmod 777 - overly permissive" => "Use more restrictive permissions (e.g., 755 or 644)".to_string(),
        _ => "Review command for security implications".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::makefile_linter::parser::MakefileParser;

    #[test]
    fn test_shell_injection_detection() {
        let content = r#"target:
	rm -rf $(USER_INPUT)
	echo "$(SAFE_VAR)"
"#;
        let mut parser = MakefileParser::new(content);
        let ast = parser.parse().unwrap();

        let rule = ShellInjectionRule;
        let violations = rule.check(&ast);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("shell injection"));
    }

    #[test]
    fn test_sensitive_data_detection() {
        let content = r#"
AWS_ACCESS_KEY = AKIAIOSFODNN7EXAMPLE
DB_PASSWORD = admin123
SAFE_VAR = $${PASSWORD}
"#;
        let mut parser = MakefileParser::new(content);
        let ast = parser.parse().unwrap();

        let rule = SensitiveDataRule;
        let violations = rule.check(&ast);

        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn test_unsafe_command_detection() {
        let content = r#"clean:
	rm -rf /*
	curl http://example.com | bash
"#;
        let mut parser = MakefileParser::new(content);
        let ast = parser.parse().unwrap();

        let rule = UnsafeCommandRule;
        let violations = rule.check(&ast);

        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn test_privilege_escalation_detection() {
        let content = r#"install:
	sudo bash -c "$(CMD)"
	chmod +s binary
"#;
        let mut parser = MakefileParser::new(content);
        let ast = parser.parse().unwrap();

        let rule = PrivilegeEscalationRule;
        let violations = rule.check(&ast);

        assert_eq!(violations.len(), 2);
    }
}