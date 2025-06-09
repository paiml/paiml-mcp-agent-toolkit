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
                    message: format!("Target '{required}' should be declared .PHONY"),
                    fix_hint: Some(format!("Add '.PHONY: {required}' to your Makefile")),
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
                                "Target '{target}' should probably be declared .PHONY"
                            ),
                            fix_hint: Some(format!("Add '{target}' to .PHONY declaration")),
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
                            "Variable '{name}' uses immediate assignment with date command. \
                             This will be evaluated once at parse time"
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

/// Represents a variable reference found in text
#[derive(Debug)]
struct VariableRef {
    name: String,
    #[allow(dead_code)]
    position: usize,
    ref_type: VarRefType,
}

#[derive(Debug, PartialEq)]
enum VarRefType {
    Parenthesized, // $(VAR)
    Braced,        // ${VAR}
    Single,        // $V
}

/// Iterator that scans text for variable references
struct VariableScanner<'a> {
    text: &'a str,
    bytes: &'a [u8],
    position: usize,
}

fn check_undefined_in_text(
    text: &str,
    defined_vars: &HashSet<String>,
    violations: &mut Vec<Violation>,
    span: SourceSpan,
) {
    let scanner = VariableScanner::new(text);

    for var_ref in scanner {
        if should_check_variable(&var_ref) && !defined_vars.contains(&var_ref.name) {
            violations.push(create_undefined_violation(&var_ref.name, span));
        }
    }
}

impl<'a> VariableScanner<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            text,
            bytes: text.as_bytes(),
            position: 0,
        }
    }

    fn find_next_dollar(&mut self) -> Option<usize> {
        while self.position < self.bytes.len() {
            if self.bytes[self.position] == b'$' {
                return Some(self.position);
            }
            self.position += 1;
        }
        None
    }

    fn parse_parenthesized_var(&mut self, start: usize) -> Option<VariableRef> {
        let content_start = start + 2;
        if content_start >= self.text.len() {
            return None;
        }

        let remaining = &self.text[content_start..];

        if let Some(end) = remaining.find(')') {
            let var_content = &remaining[..end];
            let var_name = extract_var_name(var_content);

            self.position = content_start + end + 1;

            Some(VariableRef {
                name: var_name,
                position: start,
                ref_type: VarRefType::Parenthesized,
            })
        } else {
            None
        }
    }

    fn parse_braced_var(&mut self, start: usize) -> Option<VariableRef> {
        let content_start = start + 2;
        if content_start >= self.text.len() {
            return None;
        }

        let remaining = &self.text[content_start..];

        if let Some(end) = remaining.find('}') {
            let var_name = remaining[..end].to_string();

            self.position = content_start + end + 1;

            Some(VariableRef {
                name: var_name,
                position: start,
                ref_type: VarRefType::Braced,
            })
        } else {
            None
        }
    }

    fn parse_single_char_var(&mut self, start: usize) -> Option<VariableRef> {
        if start + 1 >= self.bytes.len() {
            return None;
        }

        let ch = self.bytes[start + 1];

        if ch.is_ascii_alphanumeric() || ch == b'_' {
            let var_name = std::str::from_utf8(&[ch]).unwrap().to_string();

            self.position = start + 2;

            Some(VariableRef {
                name: var_name,
                position: start,
                ref_type: VarRefType::Single,
            })
        } else {
            None
        }
    }
}

impl<'a> Iterator for VariableScanner<'a> {
    type Item = VariableRef;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let dollar_pos = self.find_next_dollar()?;

            if dollar_pos + 1 >= self.bytes.len() {
                return None;
            }

            let next_char = self.bytes[dollar_pos + 1];

            // Handle $$ escape sequence (literal $)
            if next_char == b'$' {
                self.position = dollar_pos + 2;
                continue;
            }

            let var_ref = match next_char {
                b'(' => self.parse_parenthesized_var(dollar_pos),
                b'{' => self.parse_braced_var(dollar_pos),
                _ => self.parse_single_char_var(dollar_pos),
            };

            if let Some(ref_) = var_ref {
                return Some(ref_);
            } else {
                // Skip this dollar sign and continue
                self.position = dollar_pos + 1;
            }
        }
    }
}

/// Extract variable name from a reference that might contain modifiers
fn extract_var_name(var_content: &str) -> String {
    // Handle default value syntax ${VAR:-default}
    if var_content.contains(":-") {
        if let Some(pos) = var_content.find(":-") {
            return var_content[..pos].trim().to_string();
        }
    }

    // Handle alternative value syntax ${VAR:+alt}
    if var_content.contains(":+") {
        if let Some(pos) = var_content.find(":+") {
            return var_content[..pos].trim().to_string();
        }
    }

    // Handle pattern substitution like $(VAR:old=new)
    if let Some(colon_pos) = var_content.find(':') {
        // But not if it's part of a shell command with spaces
        let before_colon = &var_content[..colon_pos];
        if !before_colon.contains(' ') && !before_colon.contains('|') && !before_colon.contains('{')
        {
            return before_colon.trim().to_string();
        }
    }

    // If it contains shell operators, it's likely a command not a variable
    if var_content.contains('|') || var_content.contains('>') || var_content.contains('<') {
        return String::new(); // Return empty to skip validation
    }

    var_content.trim().to_string()
}

/// Check if a variable reference should be validated
fn should_check_variable(var_ref: &VariableRef) -> bool {
    // Skip empty names (likely shell commands)
    if var_ref.name.is_empty() {
        return false;
    }

    // Skip automatic variables
    if is_automatic_var(&var_ref.name) {
        return false;
    }

    // Skip function calls (only applies to parenthesized refs)
    if var_ref.ref_type == VarRefType::Parenthesized && is_function_call(&var_ref.name) {
        return false;
    }

    // Skip shell commands (contain spaces or common shell operators)
    if var_ref.name.contains(' ') || var_ref.name.contains(';') || var_ref.name.contains('&') {
        return false;
    }

    // Skip single letter variables that are likely loop variables
    if var_ref.name.len() == 1 && var_ref.name.chars().all(|c| c.is_lowercase()) {
        return false;
    }

    true
}

fn create_undefined_violation(var_name: &str, span: SourceSpan) -> Violation {
    Violation {
        rule: "undefinedvariable".to_string(),
        severity: Severity::Warning,
        span,
        message: format!("Variable '{var_name}' may be undefined"),
        fix_hint: Some(format!("Define '{var_name}' before use")),
    }
}

fn is_automatic_var(var: &str) -> bool {
    matches!(var, "@" | "<" | "^" | "?" | "*" | "%" | "+" | "|" | "$")
}

fn is_function_call(text: &str) -> bool {
    const FUNCTION_PREFIXES: &[&str] = &[
        "shell ",
        "wildcard ",
        "patsubst ",
        "subst ",
        "strip ",
        "findstring ",
        "filter ",
        "sort ",
        "word ",
        "dir ",
        "notdir ",
        "suffix ",
        "basename ",
        "addprefix ",
        "addsuffix ",
        "join ",
        "foreach ",
        "if ",
        "or ",
        "and ",
        "call ",
        "eval ",
        "origin ",
        "error ",
        "warning ",
        "info ",
    ];

    FUNCTION_PREFIXES
        .iter()
        .any(|prefix| text.starts_with(prefix))
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
