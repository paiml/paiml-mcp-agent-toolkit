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
                            span: node.span,
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
                            span: node.span,
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
                                span: node.span,
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
                            span: node.span,
                            message: format!("Unsafe command pattern detected: {}", pattern),
                            fix_hint: Some(suggest_safe_alternative(pattern)),
                        });
                    }
                }
            }
        }

        violations
    }
}

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
                            span: node.span,
                            message: format!("Potential privilege escalation: {}", issue),
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
