use super::*;
use crate::services::makefile_linter::ast::*;
use std::collections::HashSet;

/// MinPhony rule - ensures required targets are declared as .PHONY
pub struct MinPhonyRule {
    required_targets: Vec<String>,
    check_exists: bool,
}

impl Default for MinPhonyRule {
    fn default() -> Self {
        Self {
            required_targets: vec!["all".to_string(), "clean".to_string(), "test".to_string()],
            check_exists: true,
        }
    }
}

impl MakefileRule for MinPhonyRule {
    fn id(&self) -> &'static str {
        "minphony"
    }

    fn check(&self, ast: &MakefileAst) -> Vec<Violation> {
        let mut violations = Vec::new();
        let phony_targets: HashSet<_> = ast.get_phony_targets().into_iter().collect();

        // Collect all defined targets
        let mut defined_targets = HashSet::new();
        for node in &ast.nodes {
            if let NodeData::Rule { targets, .. } = &node.data {
                for target in targets {
                    if !target.starts_with('.') {
                        defined_targets.insert(target.clone());
                    }
                }
            }
        }

        // Check required targets
        for required in &self.required_targets {
            let exists = defined_targets.contains(required);
            let is_phony = phony_targets.contains(required);

            if (!self.check_exists || exists) && !is_phony {
                violations.push(Violation {
                    rule: self.id().to_string(),
                    severity: self.default_severity(),
                    span: SourceSpan::file_level(),
                    message: format!("Target '{}' should be declared .PHONY", required),
                    fix_hint: Some(format!("Add '.PHONY: {}' to your Makefile", required)),
                });
            }
        }

        violations
    }
}

/// PhonyDeclared rule - warns about targets that should be .PHONY
pub struct PhonyDeclaredRule {
    ignore_suffixes: Vec<String>,
}

impl Default for PhonyDeclaredRule {
    fn default() -> Self {
        Self {
            ignore_suffixes: vec![
                ".o".to_string(),
                ".a".to_string(),
                ".so".to_string(),
                ".exe".to_string(),
                ".ko".to_string(),
                ".mod".to_string(),
            ],
        }
    }
}

impl MakefileRule for PhonyDeclaredRule {
    fn id(&self) -> &'static str {
        "phonydeclared"
    }

    fn default_severity(&self) -> Severity {
        Severity::Info
    }

    fn check(&self, ast: &MakefileAst) -> Vec<Violation> {
        let mut violations = Vec::new();
        let phony_targets: HashSet<_> = ast.get_phony_targets().into_iter().collect();

        for node in &ast.nodes {
            if node.kind != MakefileNodeKind::Rule {
                continue;
            }

            if let NodeData::Rule { targets, .. } = &node.data {
                for target in targets {
                    // Skip special targets
                    if target.starts_with('.') || target.contains('/') || target.contains('%') {
                        continue;
                    }

                    // Skip files with known extensions
                    if self
                        .ignore_suffixes
                        .iter()
                        .any(|suffix| target.ends_with(suffix))
                    {
                        continue;
                    }

                    // Check if declared phony
                    if !phony_targets.contains(target) {
                        violations.push(Violation {
                            rule: self.id().to_string(),
                            severity: self.default_severity(),
                            span: node.span,
                            message: format!(
                                "Target '{}' should probably be declared .PHONY",
                                target
                            ),
                            fix_hint: Some(format!("Add '{}' to .PHONY declaration", target)),
                        });
                    }
                }
            }
        }

        violations
    }
}

/// MaxBodyLength rule - checks recipe complexity
pub struct MaxBodyLengthRule {
    max_lines: usize,
    count_logical: bool,
}

impl Default for MaxBodyLengthRule {
    fn default() -> Self {
        Self {
            max_lines: 10,
            count_logical: true,
        }
    }
}

impl MakefileRule for MaxBodyLengthRule {
    fn id(&self) -> &'static str {
        "maxbodylength"
    }

    fn default_severity(&self) -> Severity {
        Severity::Info
    }

    fn check(&self, ast: &MakefileAst) -> Vec<Violation> {
        let mut violations = Vec::new();

        for node in &ast.nodes {
            if node.kind != MakefileNodeKind::Recipe {
                continue;
            }

            if let NodeData::Recipe { lines } = &node.data {
                let line_count = if self.count_logical {
                    // Count logical lines (excluding continuations)
                    lines
                        .iter()
                        .filter(|line| !line.text.trim_end().ends_with('\\'))
                        .count()
                } else {
                    lines.len()
                };

                if line_count > self.max_lines {
                    violations.push(Violation {
                        rule: self.id().to_string(),
                        severity: self.default_severity(),
                        span: node.span,
                        message: format!(
                            "Recipe has {} lines (max: {}). Consider splitting into smaller targets",
                            line_count, self.max_lines
                        ),
                        fix_hint: Some("Break complex recipes into multiple targets or extract to scripts".to_string()),
                    });
                }
            }
        }

        violations
    }
}

/// TimestampExpanded rule - warns about timestamp issues
pub struct TimestampExpandedRule;

impl Default for TimestampExpandedRule {
    fn default() -> Self {
        Self
    }
}

impl MakefileRule for TimestampExpandedRule {
    fn id(&self) -> &'static str {
        "timestampexpanded"
    }

    fn check(&self, ast: &MakefileAst) -> Vec<Violation> {
        let mut violations = Vec::new();

        // Check for $(shell date) in immediate assignments
        for node in &ast.nodes {
            if let NodeData::Variable {
                name,
                assignment_op,
                value,
            } = &node.data
            {
                if *assignment_op == AssignmentOp::Immediate
                    && (value.contains("$(shell date") || value.contains("$(date"))
                {
                    violations.push(Violation {
                        rule: self.id().to_string(),
                        severity: self.default_severity(),
                        span: node.span,
                        message: format!(
                            "Variable '{}' uses immediate assignment with date command. \
                             This will be evaluated once at parse time",
                            name
                        ),
                        fix_hint: Some(
                            "Use deferred assignment (=) instead of immediate (:=)".to_string(),
                        ),
                    });
                }
            }
        }

        violations
    }
}

/// UndefinedVariable rule - warns about potentially undefined variables
pub struct UndefinedVariableRule;

impl Default for UndefinedVariableRule {
    fn default() -> Self {
        Self
    }
}

impl MakefileRule for UndefinedVariableRule {
    fn id(&self) -> &'static str {
        "undefinedvariable"
    }

    fn check(&self, ast: &MakefileAst) -> Vec<Violation> {
        let mut violations = Vec::new();
        let mut defined_vars = HashSet::new();

        // Collect all defined variables
        for (name, _, _) in ast.get_variables() {
            defined_vars.insert(name.clone());
        }

        // Add common built-in variables
        for builtin in &["CC", "CXX", "CFLAGS", "LDFLAGS", "MAKE", "SHELL", "PWD"] {
            defined_vars.insert(builtin.to_string());
        }

        // Check for undefined variable usage
        for node in &ast.nodes {
            match &node.data {
                NodeData::Variable { value, .. } => {
                    check_undefined_in_text(value, &defined_vars, &mut violations, node.span);
                }
                NodeData::Recipe { lines } => {
                    for line in lines {
                        check_undefined_in_text(
                            &line.text,
                            &defined_vars,
                            &mut violations,
                            node.span,
                        );
                    }
                }
                _ => {}
            }
        }

        violations
    }
}

fn check_undefined_in_text(
    text: &str,
    defined_vars: &HashSet<String>,
    violations: &mut Vec<Violation>,
    span: SourceSpan,
) {
    // Simple variable reference pattern matching
    let mut i = 0;
    let bytes = text.as_bytes();

    while i < bytes.len() - 1 {
        if bytes[i] == b'$' && i + 1 < bytes.len() {
            match bytes[i + 1] {
                b'(' => {
                    // Find closing paren
                    if let Some(end) = text[i + 2..].find(')') {
                        let var_ref = &text[i + 2..i + 2 + end];
                        // Skip automatic variables and functions
                        if !is_automatic_var(var_ref) && !is_function_call(var_ref) {
                            let var_name = var_ref.split(':').next().unwrap_or(var_ref);
                            if !defined_vars.contains(var_name) {
                                violations.push(Violation {
                                    rule: "undefinedvariable".to_string(),
                                    severity: Severity::Warning,
                                    span,
                                    message: format!("Variable '{}' may be undefined", var_name),
                                    fix_hint: Some(format!("Define '{}' before use", var_name)),
                                });
                            }
                        }
                        i += end + 3;
                        continue;
                    }
                }
                b'{' => {
                    // Find closing brace
                    if let Some(end) = text[i + 2..].find('}') {
                        let var_name = &text[i + 2..i + 2 + end];
                        if !is_automatic_var(var_name) && !defined_vars.contains(var_name) {
                            violations.push(Violation {
                                rule: "undefinedvariable".to_string(),
                                severity: Severity::Warning,
                                span,
                                message: format!("Variable '{}' may be undefined", var_name),
                                fix_hint: Some(format!("Define '{}' before use", var_name)),
                            });
                        }
                        i += end + 3;
                        continue;
                    }
                }
                c if c.is_ascii_alphanumeric() || c == b'_' => {
                    // Single character variable
                    let byte_slice = [c];
                    let var_name = std::str::from_utf8(&byte_slice).unwrap();
                    if !is_automatic_var(var_name) && !defined_vars.contains(var_name) {
                        violations.push(Violation {
                            rule: "undefinedvariable".to_string(),
                            severity: Severity::Warning,
                            span,
                            message: format!("Variable '{}' may be undefined", var_name),
                            fix_hint: Some(format!("Define '{}' before use", var_name)),
                        });
                    }
                    i += 2;
                    continue;
                }
                _ => {}
            }
        }
        i += 1;
    }
}

fn is_automatic_var(var: &str) -> bool {
    matches!(var, "@" | "<" | "^" | "?" | "*" | "%" | "+" | "|" | "$")
}

fn is_function_call(text: &str) -> bool {
    text.starts_with("shell ")
        || text.starts_with("wildcard ")
        || text.starts_with("patsubst ")
        || text.starts_with("subst ")
        || text.starts_with("strip ")
        || text.starts_with("findstring ")
        || text.starts_with("filter ")
        || text.starts_with("sort ")
        || text.starts_with("word ")
        || text.starts_with("dir ")
        || text.starts_with("notdir ")
        || text.starts_with("suffix ")
        || text.starts_with("basename ")
        || text.starts_with("addprefix ")
        || text.starts_with("addsuffix ")
        || text.starts_with("join ")
        || text.starts_with("foreach ")
        || text.starts_with("if ")
        || text.starts_with("or ")
        || text.starts_with("and ")
        || text.starts_with("call ")
        || text.starts_with("eval ")
        || text.starts_with("origin ")
        || text.starts_with("error ")
        || text.starts_with("warning ")
        || text.starts_with("info ")
}

/// Portability rule - checks for GNU Make specific features
pub struct PortabilityRule;

impl Default for PortabilityRule {
    fn default() -> Self {
        Self
    }
}

impl MakefileRule for PortabilityRule {
    fn id(&self) -> &'static str {
        "portability"
    }

    fn default_severity(&self) -> Severity {
        Severity::Info
    }

    fn check(&self, ast: &MakefileAst) -> Vec<Violation> {
        let mut violations = Vec::new();

        // Check for GNU-specific assignment operators
        for node in &ast.nodes {
            if let NodeData::Variable { assignment_op, .. } = &node.data {
                match assignment_op {
                    AssignmentOp::Conditional => {
                        violations.push(Violation {
                            rule: self.id().to_string(),
                            severity: self.default_severity(),
                            span: node.span,
                            message: "Conditional assignment (?=) is GNU Make specific".to_string(),
                            fix_hint: Some(
                                "Use ifdef/ifndef for portable conditional assignment".to_string(),
                            ),
                        });
                    }
                    AssignmentOp::Shell => {
                        violations.push(Violation {
                            rule: self.id().to_string(),
                            severity: self.default_severity(),
                            span: node.span,
                            message: "Shell assignment (!=) is GNU Make specific".to_string(),
                            fix_hint: Some(
                                "Use $(shell ...) for portable shell execution".to_string(),
                            ),
                        });
                    }
                    _ => {}
                }
            }
        }

        violations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::makefile_linter::MakefileParser;

    #[test]
    fn test_min_phony_rule() {
        let rule = MinPhonyRule::default();
        assert_eq!(rule.id(), "minphony");

        // Test with empty AST - MinPhonyRule only warns if targets exist but aren't phony
        let ast = MakefileAst::new();
        let violations = rule.check(&ast);
        assert_eq!(violations.len(), 0); // No targets exist, so no violations

        // Test with targets but no .PHONY
        let input = "all:\n\techo all\nclean:\n\trm -f *.o";
        let mut parser = MakefileParser::new(input);
        let ast = parser.parse().unwrap();
        let violations = rule.check(&ast);
        assert_eq!(violations.len(), 2); // all and clean exist but aren't .PHONY
        assert!(violations.iter().any(|v| v.message.contains("all")));
        assert!(violations.iter().any(|v| v.message.contains("clean")));

        // Test with custom rule that doesn't check existence
        let rule_no_check = MinPhonyRule {
            required_targets: vec!["all".to_string(), "clean".to_string()],
            check_exists: false,
        };
        let empty_ast = MakefileAst::new();
        let violations = rule_no_check.check(&empty_ast);
        assert_eq!(violations.len(), 2); // Should warn even if targets don't exist
    }

    #[test]
    fn test_phony_declared_rule() {
        let rule = PhonyDeclaredRule::default();
        assert_eq!(rule.id(), "phonydeclared");
        assert_eq!(rule.default_severity(), Severity::Info);

        // Test with non-file targets
        let input = "install:\n\tcp prog /usr/bin/\nhelp:\n\techo help";
        let mut parser = MakefileParser::new(input);
        let ast = parser.parse().unwrap();
        let violations = rule.check(&ast);
        assert_eq!(violations.len(), 2);
        assert!(violations.iter().all(|v| v.rule == "phonydeclared"));
    }

    #[test]
    fn test_max_body_length_rule() {
        let rule = MaxBodyLengthRule {
            max_lines: 5,
            count_logical: true,
        };
        assert_eq!(rule.id(), "maxbodylength");

        // Test with long recipe
        let input = "target:\n\tline1\n\tline2\n\tline3\n\tline4\n\tline5\n\tline6";
        let mut parser = MakefileParser::new(input);
        let ast = parser.parse().unwrap();
        let violations = rule.check(&ast);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("6 lines"));
    }

    #[test]
    fn test_timestamp_expanded_rule() {
        let rule = TimestampExpandedRule;
        assert_eq!(rule.id(), "timestampexpanded");

        // Test with immediate assignment - this is what the rule warns about
        let input = "BUILD_TIME := $(shell date)";
        let mut parser = MakefileParser::new(input);
        let ast = parser.parse().unwrap();
        let violations = rule.check(&ast);
        assert_eq!(violations.len(), 1);
        assert!(violations[0]
            .message
            .contains("evaluated once at parse time"));

        // Test with deferred assignment - this is the recommended approach
        let input2 = "BUILD_TIME = $(shell date)";
        let mut parser2 = MakefileParser::new(input2);
        let ast2 = parser2.parse().unwrap();
        let violations2 = rule.check(&ast2);
        assert_eq!(violations2.len(), 0);
    }

    #[test]
    fn test_undefined_variable_rule() {
        let rule = UndefinedVariableRule;
        assert_eq!(rule.id(), "undefinedvariable");

        // Test with undefined variable
        let input = "target:\n\techo $(UNDEFINED_VAR)";
        let mut parser = MakefileParser::new(input);
        let ast = parser.parse().unwrap();
        let violations = rule.check(&ast);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("UNDEFINED_VAR"));

        // Test with defined variable
        let input2 = "VAR = value\ntarget:\n\techo $(VAR)";
        let mut parser2 = MakefileParser::new(input2);
        let ast2 = parser2.parse().unwrap();
        let violations2 = rule.check(&ast2);
        assert_eq!(violations2.len(), 0);
    }

    #[test]
    fn test_portability_rule() {
        let rule = PortabilityRule;
        assert_eq!(rule.id(), "portability");

        // Test with GNU-specific conditional assignment
        let input = "VAR ?= value";
        let mut parser = MakefileParser::new(input);
        let ast = parser.parse().unwrap();
        let violations = rule.check(&ast);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Conditional assignment"));

        // Test with GNU-specific shell assignment
        let input2 = "VAR != date";
        let mut parser2 = MakefileParser::new(input2);
        let ast2 = parser2.parse().unwrap();
        let violations2 = rule.check(&ast2);
        assert_eq!(violations2.len(), 1);
        assert!(violations2[0].message.contains("Shell assignment"));
    }

    #[test]
    fn test_is_automatic_var() {
        assert!(is_automatic_var("@"));
        assert!(is_automatic_var("<"));
        assert!(is_automatic_var("^"));
        assert!(is_automatic_var("?"));
        assert!(is_automatic_var("*"));
        assert!(is_automatic_var("%"));
        assert!(is_automatic_var("+"));
        assert!(is_automatic_var("|"));
        assert!(!is_automatic_var("CC"));
        assert!(!is_automatic_var("CFLAGS"));
    }

    #[test]
    fn test_is_function_call() {
        assert!(is_function_call("shell date"));
        assert!(is_function_call("wildcard *.c"));
        assert!(is_function_call("patsubst %.c,%.o,$(SRCS)"));
        assert!(!is_function_call("CC"));
        assert!(!is_function_call("VARIABLE_NAME"));
    }
}
