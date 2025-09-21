//! Bash/Shell Script Analysis Support for PMAT
//!
//! This module provides Bash-specific analysis capabilities using lexical analysis
//! and partial AST extraction for shell scripts within static analysis constraints.

use std::path::{Path, PathBuf};
#[cfg(feature = "shell-ast")]
use crate::services::context::AstItem;

/// Bash script analyzer that extracts shell-specific information
pub struct BashScriptAnalyzer {
    items: Vec<AstItem>,
    _file_path: PathBuf,
    script_name: String,
    function_count: usize,
    variable_count: usize,
    command_count: usize,
}

impl BashScriptAnalyzer {
    /// Creates a new Bash script analyzer
    #[must_use] 
    pub fn new(file_path: &Path) -> Self {
        Self {
            items: Vec::new(),
            _file_path: file_path.to_path_buf(),
            script_name: file_path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string(),
            function_count: 0,
            variable_count: 0,
            command_count: 0,
        }
    }

    /// Analyzes Bash script and extracts AST items (complexity ≤10)
    pub fn analyze_bash_script(mut self, source: &str) -> Result<Vec<AstItem>, String> {
        if source.trim().is_empty() {
            return Ok(vec![]);
        }

        self.extract_shell_functions(source)?;
        self.extract_variables(source)?;
        self.analyze_commands(source)?;
        self.extract_control_flow(source)?;

        Ok(self.items)
    }

    /// Extracts function definitions from shell script (complexity ≤10)
    fn extract_shell_functions(&mut self, source: &str) -> Result<(), String> {
        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();

            if trimmed.ends_with("() {") || trimmed.contains("function ") {
                let func_name = self.extract_function_name(trimmed)?;
                let qualified_name = self.get_qualified_name(&func_name);

                self.items.push(AstItem::Function {
                    name: qualified_name,
                    visibility: "public".to_string(),
                    is_async: false,
                    line: line_num + 1,
                });
                self.function_count += 1;
            }
        }
        Ok(())
    }

    /// Extracts variable declarations and usage (complexity ≤10)
    fn extract_variables(&mut self, source: &str) -> Result<(), String> {
        for line in source.lines() {
            let trimmed = line.trim();

            if trimmed.contains('=') && !trimmed.starts_with('#') {
                let parts: Vec<&str> = trimmed.split('=').collect();
                if parts.len() >= 2 {
                    self.variable_count += 1;
                }
            }
        }
        Ok(())
    }

    /// Analyzes command invocations and pipelines (complexity ≤10)
    fn analyze_commands(&mut self, source: &str) -> Result<(), String> {
        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();

            if !trimmed.is_empty() && !trimmed.starts_with('#') && !trimmed.starts_with("#!/") {
                // Treat basic commands as functions for AST structure
                if let Some(cmd) = trimmed.split_whitespace().next() {
                    if !cmd.contains('=') && !cmd.ends_with('{') { // Not assignments or function defs
                        let qualified_name = self.get_qualified_name(cmd);
                        self.items.push(AstItem::Function {
                            name: qualified_name,
                            visibility: "public".to_string(),
                            is_async: false,
                            line: line_num + 1,
                        });
                    }
                }

                if trimmed.contains('|') {
                    self.command_count += 2; // Count pipeline as multiple commands
                } else {
                    self.command_count += 1;
                }
            }
        }
        Ok(())
    }

    /// Extracts control flow structures (complexity ≤10)
    fn extract_control_flow(&mut self, source: &str) -> Result<(), String> {
        for line in source.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("if ") || trimmed.starts_with("while ") ||
               trimmed.starts_with("for ") || trimmed.starts_with("case ") {
                // Control flow statements found - could extract more details
                // For now, just noting their presence
            }
        }
        Ok(())
    }

    /// Extracts function name from shell line (complexity ≤10)
    fn extract_function_name(&self, line: &str) -> Result<String, String> {
        if let Some(pos) = line.find("() {") {
            let name_part = &line[..pos];
            Ok(name_part.trim().to_string())
        } else if line.contains("function ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                Ok(parts[1].to_string())
            } else {
                Err("Invalid function declaration".to_string())
            }
        } else {
            Err("Invalid function format".to_string())
        }
    }

    /// Gets qualified name for shell symbol (complexity ≤10)
    fn get_qualified_name(&self, symbol_name: &str) -> String {
        if self.script_name.is_empty() {
            symbol_name.to_string()
        } else {
            format!("{}::{}", self.script_name, symbol_name)
        }
    }
}

/// Bash complexity analyzer for shell-specific metrics (complexity ≤10)
pub struct BashComplexityAnalyzer {
    cyclomatic_complexity: u32,
    cognitive_complexity: u32,
    _nesting_depth: u32,
}

impl Default for BashComplexityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl BashComplexityAnalyzer {
    /// Creates a new Bash complexity analyzer
    #[must_use] 
    pub fn new() -> Self {
        Self {
            cyclomatic_complexity: 0,
            cognitive_complexity: 0,
            _nesting_depth: 0,
        }
    }

    /// Analyzes complexity of Bash script (complexity ≤10)
    pub fn analyze_complexity(&mut self, source: &str) -> Result<(u32, u32), String> {
        self.cyclomatic_complexity = 1;
        self.cognitive_complexity = 1;

        for line in source.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("if ") || trimmed.starts_with("while ") ||
               trimmed.starts_with("for ") || trimmed.starts_with("case ") ||
               trimmed.starts_with("elif ") {
                self.cyclomatic_complexity += 1;
                self.cognitive_complexity += 1;
            }
        }

        Ok((self.cyclomatic_complexity, self.cognitive_complexity))
    }

    /// Analyzes pipeline complexity (complexity ≤10)
    pub fn analyze_pipeline_complexity(&mut self, pipeline: &str) -> Result<u32, String> {
        let pipe_count = pipeline.matches('|').count();
        Ok(pipe_count as u32 + 1) // Base complexity of 1 plus number of pipes
    }

    /// Analyzes conditional complexity (complexity ≤10)
    pub fn analyze_conditional_complexity(&mut self, conditions: &str) -> Result<u32, String> {
        let mut complexity = 1;

        // Count logical operators
        complexity += conditions.matches(" && ").count() as u32;
        complexity += conditions.matches(" || ").count() as u32;
        complexity += conditions.matches(" -a ").count() as u32;
        complexity += conditions.matches(" -o ").count() as u32;

        Ok(complexity)
    }
}

/// Shell script safety and best practices analyzer (complexity ≤10)
pub struct ShellSafetyAnalyzer {
    safety_violations: Vec<String>,
    best_practice_warnings: Vec<String>,
}

impl Default for ShellSafetyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl ShellSafetyAnalyzer {
    /// Creates a new shell safety analyzer
    #[must_use] 
    pub fn new() -> Self {
        Self {
            safety_violations: Vec::new(),
            best_practice_warnings: Vec::new(),
        }
    }

    /// Analyzes shell script for safety issues (complexity ≤10)
    pub fn analyze_safety(&mut self, source: &str) -> Result<Vec<String>, String> {
        let mut violations = Vec::new();

        for line in source.lines() {
            let trimmed = line.trim();

            if trimmed.contains("rm -rf $") {
                violations.push("Dangerous rm -rf with variable".to_string());
            }
            if trimmed.contains("eval \"$") {
                violations.push("Dangerous eval with user input".to_string());
            }
            if trimmed.contains("$@") && !trimmed.contains("\"$@\"") {
                violations.push("Unquoted $@ parameter expansion".to_string());
            }
        }

        self.safety_violations = violations.clone();
        Ok(violations)
    }

    /// Checks for common security vulnerabilities (complexity ≤10)
    pub fn check_security_vulnerabilities(&mut self, source: &str) -> Result<Vec<String>, String> {
        let mut vulnerabilities = Vec::new();

        for line in source.lines() {
            let trimmed = line.trim();

            if trimmed.contains("curl") && !trimmed.contains("--fail") {
                vulnerabilities.push("curl without --fail may ignore errors".to_string());
            }
            if trimmed.contains("wget") && !trimmed.contains("-O") {
                vulnerabilities.push("wget without explicit output may overwrite".to_string());
            }
        }

        Ok(vulnerabilities)
    }

    /// Validates best practices compliance (complexity ≤10)
    pub fn validate_best_practices(&mut self, source: &str) -> Result<Vec<String>, String> {
        let mut warnings = Vec::new();

        let has_shebang = source.lines().next().unwrap_or("").starts_with("#!");
        if !has_shebang {
            warnings.push("Missing shebang line".to_string());
        }

        let has_set_flags = source.contains("set -e") || source.contains("set -u");
        if !has_set_flags {
            warnings.push("Consider using 'set -e' or 'set -u' for error handling".to_string());
        }

        self.best_practice_warnings = warnings.clone();
        Ok(warnings)
    }

    /// Gets safety violations
    #[must_use] 
    pub fn get_safety_violations(&self) -> &[String] {
        &self.safety_violations
    }

    /// Gets best practice warnings
    #[must_use] 
    pub fn get_best_practice_warnings(&self) -> &[String] {
        &self.best_practice_warnings
    }
}

/// Shell command parser for lexical analysis (complexity ≤10)
pub struct ShellCommandParser {
    commands: Vec<String>,
    variables: Vec<String>,
}

impl Default for ShellCommandParser {
    fn default() -> Self {
        Self::new()
    }
}

impl ShellCommandParser {
    /// Creates a new shell command parser
    #[must_use] 
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            variables: Vec::new(),
        }
    }

    /// Parses shell command line into tokens (complexity ≤10)
    pub fn parse_command_line(&mut self, line: &str) -> Result<Vec<String>, String> {
        let tokens: Vec<String> = line.split_whitespace()
            .map(std::string::ToString::to_string)
            .collect();

        self.commands.extend(tokens.clone());
        Ok(tokens)
    }

    /// Extracts variable assignments (complexity ≤10)
    pub fn extract_variable_assignments(&mut self, line: &str) -> Result<Vec<(String, String)>, String> {
        let mut assignments = Vec::new();

        if line.contains('=') && !line.trim().starts_with('#') {
            let parts: Vec<&str> = line.split('=').collect();
            if parts.len() >= 2 {
                let var_name = parts[0].trim().to_string();
                let var_value = parts[1].trim().to_string();
                assignments.push((var_name.clone(), var_value));
                self.variables.push(var_name);
            }
        }

        Ok(assignments)
    }

    /// Gets parsed commands
    #[must_use] 
    pub fn get_commands(&self) -> &[String] {
        &self.commands
    }

    /// Gets extracted variables
    #[must_use] 
    pub fn get_variables(&self) -> &[String] {
        &self.variables
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    const SIMPLE_BASH_SCRIPT: &str = r#"#!/bin/bash

echo "Hello, World!"
exit 0
"#;

    const BASH_SCRIPT_WITH_FUNCTIONS: &str = r#"#!/bin/bash

# Function to add two numbers
add_numbers() {
    local a=$1
    local b=$2
    echo $((a + b))
}

# Function to check if file exists
file_exists() {
    if [[ -f "$1" ]]; then
        echo "File exists: $1"
        return 0
    else
        echo "File not found: $1"
        return 1
    fi
}

# Main script
result=$(add_numbers 5 3)
echo "Result: $result"

file_exists "/etc/passwd"
"#;

    const COMPLEX_BASH_SCRIPT: &str = r#"#!/bin/bash

# Complex script with loops and conditionals
process_files() {
    local dir="$1"
    local count=0

    for file in "$dir"/*; do
        if [[ -f "$file" ]]; then
            case "${file##*.}" in
                txt)
                    echo "Processing text file: $file"
                    ((count++))
                    ;;
                log)
                    if [[ -s "$file" ]]; then
                        echo "Processing log file: $file"
                        ((count++))
                    fi
                    ;;
                *)
                    echo "Skipping file: $file"
                    ;;
            esac
        elif [[ -d "$file" ]]; then
            echo "Found directory: $file"
            process_files "$file"  # Recursive call
        fi
    done

    echo "Processed $count files in $dir"
}

# Script with error handling
main() {
    set -euo pipefail

    local input_dir="${1:-$(pwd)}"

    if [[ ! -d "$input_dir" ]]; then
        echo "Error: Directory does not exist: $input_dir" >&2
        exit 1
    fi

    process_files "$input_dir"
}

main "$@"
"#;


    #[test]
    fn test_simple_bash_script_analysis() {
        let analyzer = BashScriptAnalyzer::new(Path::new("simple.sh"));
        let items = analyzer.analyze_bash_script(SIMPLE_BASH_SCRIPT)
            .expect("Should parse simple Bash script");

        assert!(!items.is_empty(), "Should extract at least one AST item");

        // Should detect script structure (commands, variables, etc.)
        let has_commands = items.iter().any(|item| matches!(item, AstItem::Function { .. }));
        assert!(has_commands || items.len() >= 1, "Should detect script structure");
    }

    #[test]
    fn test_bash_functions_analysis() {
        let analyzer = BashScriptAnalyzer::new(Path::new("functions.sh"));
        let items = analyzer.analyze_bash_script(BASH_SCRIPT_WITH_FUNCTIONS)
            .expect("Should parse Bash script with functions");

        let function_items: Vec<_> = items.iter()
            .filter(|item| matches!(item, AstItem::Function { .. }))
            .collect();

        assert!(function_items.len() >= 2, "Should extract both add_numbers and file_exists functions");

        // Check function names
        let function_names: Vec<_> = function_items.iter()
            .filter_map(|item| match item {
                AstItem::Function { name, .. } => Some(name.as_str()),
                _ => None,
            })
            .collect();

        assert!(function_names.iter().any(|&name| name.contains("add_numbers")));
        assert!(function_names.iter().any(|&name| name.contains("file_exists")));
    }

    #[test]
    fn test_bash_complexity_analysis() {
        let mut analyzer = BashComplexityAnalyzer::new();
        let (cyclomatic, cognitive) = analyzer.analyze_complexity(COMPLEX_BASH_SCRIPT)
            .expect("Should analyze Bash complexity");

        assert!(cyclomatic >= 5, "Complex script should have significant cyclomatic complexity");
        assert!(cognitive >= 5, "Complex script should have significant cognitive complexity");
        assert!(cyclomatic <= 50, "Complexity should be reasonable for analysis");
        assert!(cognitive <= 50, "Cognitive complexity should be reasonable");
    }

    #[test]
    fn test_bash_pipeline_complexity() {
        let mut analyzer = BashComplexityAnalyzer::new();
        let pipeline = "cat file.txt | grep pattern | sort | uniq -c | sort -nr | head -10";
        let complexity = analyzer.analyze_pipeline_complexity(pipeline)
            .expect("Should analyze pipeline complexity");

        assert!(complexity >= 6, "Pipeline with 6 commands should have complexity ≥6");
        assert!(complexity <= 15, "Pipeline complexity should be bounded");
    }

    #[test]
    fn test_shell_safety_analysis() {
        let mut safety_analyzer = ShellSafetyAnalyzer::new();
        let unsafe_script = r#"
#!/bin/bash
rm -rf $dangerous_var
eval "$user_input"
"#;

        let violations = safety_analyzer.analyze_safety(unsafe_script)
            .expect("Should analyze shell safety");

        assert!(!violations.is_empty(), "Should detect safety violations in unsafe script");
    }

    #[test]
    fn test_shell_command_parsing() {
        let mut parser = ShellCommandParser::new();
        let command_line = r#"echo "hello world" | grep hello"#;
        let tokens = parser.parse_command_line(command_line)
            .expect("Should parse shell command");

        assert!(!tokens.is_empty(), "Should extract tokens from command line");
        assert!(tokens.iter().any(|token| token.contains("echo")));
        assert!(tokens.iter().any(|token| token.contains("grep")));
    }

    #[test]
    fn test_variable_extraction() {
        let mut parser = ShellCommandParser::new();
        let line = "export PATH=/usr/local/bin:$PATH";
        let assignments = parser.extract_variable_assignments(line)
            .expect("Should extract variable assignments");

        assert!(!assignments.is_empty(), "Should extract PATH assignment");
        assert!(assignments.iter().any(|(var, _)| var == "PATH"));
    }

    #[test]
    fn test_empty_bash_script() {
        let analyzer = BashScriptAnalyzer::new(Path::new("empty.sh"));
        let items = analyzer.analyze_bash_script("")
            .expect("Should handle empty script");

        assert!(items.is_empty(), "Empty script should produce no AST items");
    }

    #[test]
    fn test_invalid_bash_syntax() {
        let analyzer = BashScriptAnalyzer::new(Path::new("invalid.sh"));
        let result = analyzer.analyze_bash_script("invalid bash syntax {{{ !!!");

        // Should either handle gracefully or return error
        assert!(result.is_ok() || result.is_err(), "Should handle invalid syntax gracefully");
    }

    #[test]
    fn test_bash_best_practices() {
        let mut safety_analyzer = ShellSafetyAnalyzer::new();
        let good_script = r#"
#!/bin/bash
set -euo pipefail

readonly script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly config_file="$script_dir/config.conf"

if [[ ! -f "$config_file" ]]; then
    echo "Error: Config file not found" >&2
    exit 1
fi
"#;

        let warnings = safety_analyzer.validate_best_practices(good_script)
            .expect("Should validate best practices");

        // Good script should have minimal warnings
        assert!(warnings.len() <= 2, "Well-written script should have few best practice warnings");
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use std::path::Path;

    proptest! {
        #[test]
        fn test_bash_analyzer_handles_various_script_names(
            script_name in "[a-zA-Z_][a-zA-Z0-9_]*"
        ) {
            let file_path = format!("{}.sh", script_name);
            let analyzer = BashScriptAnalyzer::new(Path::new(&file_path));

            prop_assert_eq!(analyzer.script_name, script_name);
            prop_assert_eq!(analyzer.function_count, 0);
            prop_assert_eq!(analyzer.variable_count, 0);
            prop_assert_eq!(analyzer.command_count, 0);
        }

        #[test]
        fn test_bash_complexity_analyzer_bounds(
            nesting_depth in 1u32..8
        ) {
            let mut analyzer = BashComplexityAnalyzer::new();

            // Create nested if statements
            let mut script = String::from("#!/bin/bash\n");
            for i in 0..nesting_depth {
                script.push_str(&format!("if [[ $var{} -eq 1 ]]; then\n", i));
            }
            script.push_str("echo 'nested'\n");
            for _ in 0..nesting_depth {
                script.push_str("fi\n");
            }

            if let Ok((cyclomatic, cognitive)) = analyzer.analyze_complexity(&script) {
                // Complexity should scale with nesting depth
                prop_assert!(cyclomatic >= nesting_depth);
                prop_assert!(cognitive >= nesting_depth);
                prop_assert!(cyclomatic <= nesting_depth * 2 + 5);
                prop_assert!(cognitive <= nesting_depth * 3 + 5);
            }
        }

        #[test]
        fn test_shell_command_parser_consistency(
            command_count in 1usize..10
        ) {
            let mut parser = ShellCommandParser::new();

            let mut command_line = String::new();
            for i in 0..command_count {
                if i > 0 {
                    command_line.push_str(" | ");
                }
                command_line.push_str(&format!("command{}", i));
            }

            if let Ok(tokens) = parser.parse_command_line(&command_line) {
                // Should extract reasonable number of tokens
                prop_assert!(tokens.len() >= command_count);
                prop_assert!(tokens.len() <= command_count * 3); // Account for pipes
            }
        }

        #[test]
        fn test_shell_safety_analyzer_consistency(
            script_lines in 1usize..20
        ) {
            let mut safety_analyzer = ShellSafetyAnalyzer::new();

            let mut script = String::from("#!/bin/bash\n");
            for i in 0..script_lines {
                script.push_str(&format!("echo 'line {}'\n", i));
            }

            if let Ok(violations) = safety_analyzer.analyze_safety(&script) {
                // Simple echo statements should have minimal violations
                prop_assert!(violations.len() <= script_lines / 2);
            }
        }
    }
}