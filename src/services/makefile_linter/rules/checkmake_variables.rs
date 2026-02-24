/// `UndefinedVariable` rule - warns about potentially undefined variables
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
            defined_vars.insert((*builtin).to_string());
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

        let remaining = self.text.get(content_start..).unwrap_or_default();

        if let Some(end) = remaining.find(')') {
            let var_content = remaining.get(..end).unwrap_or_default();
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

        let remaining = self.text.get(content_start..).unwrap_or_default();

        if let Some(end) = remaining.find('}') {
            let var_name = remaining.get(..end).unwrap_or_default().to_string();

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
            let var_name = std::str::from_utf8(&[ch])
                .expect("internal error")
                .to_string();

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

impl Iterator for VariableScanner<'_> {
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
            }
            // Skip this dollar sign and continue
            self.position = dollar_pos + 1;
        }
    }
}

/// Extracts variable name from default value syntax ${VAR:-default}
fn extract_from_default_value(var_content: &str) -> Option<String> {
    if var_content.contains(":-") {
        if let Some(pos) = var_content.find(":-") {
            return Some(
                var_content
                    .get(..pos)
                    .unwrap_or_default()
                    .trim()
                    .to_string(),
            );
        }
    }
    None
}

/// Extracts variable name from alternative value syntax ${VAR:+alt}
fn extract_from_alternative_value(var_content: &str) -> Option<String> {
    if var_content.contains(":+") {
        if let Some(pos) = var_content.find(":+") {
            return Some(
                var_content
                    .get(..pos)
                    .unwrap_or_default()
                    .trim()
                    .to_string(),
            );
        }
    }
    None
}

/// Extracts variable name from pattern substitution like $(VAR:old=new)
fn extract_from_pattern_substitution(var_content: &str) -> Option<String> {
    if let Some(colon_pos) = var_content.find(':') {
        // But not if it's part of a shell command with spaces
        let before_colon = var_content.get(..colon_pos).unwrap_or_default();
        if !contains_shell_indicators(before_colon) {
            return Some(before_colon.trim().to_string());
        }
    }
    None
}

/// Checks if text contains shell command indicators
fn contains_shell_indicators(text: &str) -> bool {
    text.contains(' ') || text.contains('|') || text.contains('{')
}

/// Checks if content contains shell operators that should skip validation
fn contains_shell_operators(var_content: &str) -> bool {
    var_content.contains('|') || var_content.contains('>') || var_content.contains('<')
}

/// Extract variable name from a reference that might contain modifiers
fn extract_var_name(var_content: &str) -> String {
    // Handle default value syntax ${VAR:-default}
    if let Some(var_name) = extract_from_default_value(var_content) {
        return var_name;
    }

    // Handle alternative value syntax ${VAR:+alt}
    if let Some(var_name) = extract_from_alternative_value(var_content) {
        return var_name;
    }

    // Handle pattern substitution like $(VAR:old=new)
    if let Some(var_name) = extract_from_pattern_substitution(var_content) {
        return var_name;
    }

    // If it contains shell operators, it's likely a command not a variable
    if contains_shell_operators(var_content) {
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
    if var_ref.name.len() == 1 && var_ref.name.chars().all(char::is_lowercase) {
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
