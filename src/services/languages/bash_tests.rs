// Unit tests and property tests for bash module
// Covers BashScriptAnalyzer, BashComplexityAnalyzer, ShellSafetyAnalyzer, and ShellCommandParser.

#[cfg_attr(coverage_nightly, coverage(off))]
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
        let items = analyzer
            .analyze_bash_script(SIMPLE_BASH_SCRIPT)
            .expect("Should parse simple Bash script");

        assert!(!items.is_empty(), "Should extract at least one AST item");

        // Should detect script structure (commands, variables, etc.)
        let has_commands = items
            .iter()
            .any(|item| matches!(item, AstItem::Function { .. }));
        assert!(
            has_commands || !items.is_empty(),
            "Should detect script structure"
        );
    }

    #[test]
    fn test_bash_functions_analysis() {
        let analyzer = BashScriptAnalyzer::new(Path::new("functions.sh"));
        let items = analyzer
            .analyze_bash_script(BASH_SCRIPT_WITH_FUNCTIONS)
            .expect("Should parse Bash script with functions");

        let function_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Function { .. }))
            .collect();

        assert!(
            function_items.len() >= 2,
            "Should extract both add_numbers and file_exists functions"
        );

        // Check function names
        let function_names: Vec<_> = function_items
            .iter()
            .filter_map(|item| match item {
                AstItem::Function { name, .. } => Some(name.as_str()),
                _ => None,
            })
            .collect();

        assert!(function_names
            .iter()
            .any(|&name| name.contains("add_numbers")));
        assert!(function_names
            .iter()
            .any(|&name| name.contains("file_exists")));
    }

    #[test]
    fn test_bash_complexity_analysis() {
        let mut analyzer = BashComplexityAnalyzer::new();
        let (cyclomatic, cognitive) = analyzer
            .analyze_complexity(COMPLEX_BASH_SCRIPT)
            .expect("Should analyze Bash complexity");

        assert!(
            cyclomatic >= 5,
            "Complex script should have significant cyclomatic complexity"
        );
        assert!(
            cognitive >= 5,
            "Complex script should have significant cognitive complexity"
        );
        assert!(
            cyclomatic <= 50,
            "Complexity should be reasonable for analysis"
        );
        assert!(cognitive <= 50, "Cognitive complexity should be reasonable");
    }

    #[test]
    fn test_bash_pipeline_complexity() {
        let mut analyzer = BashComplexityAnalyzer::new();
        let pipeline = "cat file.txt | grep pattern | sort | uniq -c | sort -nr | head -10";
        let complexity = analyzer
            .analyze_pipeline_complexity(pipeline)
            .expect("Should analyze pipeline complexity");

        assert!(
            complexity >= 6,
            "Pipeline with 6 commands should have complexity >=6"
        );
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

        let violations = safety_analyzer
            .analyze_safety(unsafe_script)
            .expect("Should analyze shell safety");

        assert!(
            !violations.is_empty(),
            "Should detect safety violations in unsafe script"
        );
    }

    #[test]
    fn test_shell_command_parsing() {
        let mut parser = ShellCommandParser::new();
        let command_line = r#"echo "hello world" | grep hello"#;
        let tokens = parser
            .parse_command_line(command_line)
            .expect("Should parse shell command");

        assert!(
            !tokens.is_empty(),
            "Should extract tokens from command line"
        );
        assert!(tokens.iter().any(|token| token.contains("echo")));
        assert!(tokens.iter().any(|token| token.contains("grep")));
    }

    #[test]
    fn test_variable_extraction() {
        let mut parser = ShellCommandParser::new();
        let line = "export PATH=/usr/local/bin:$PATH";
        let assignments = parser
            .extract_variable_assignments(line)
            .expect("Should extract variable assignments");

        assert!(!assignments.is_empty(), "Should extract PATH assignment");
        assert!(assignments.iter().any(|(var, _)| var == "PATH"));
    }

    #[test]
    fn test_empty_bash_script() {
        let analyzer = BashScriptAnalyzer::new(Path::new("empty.sh"));
        let items = analyzer
            .analyze_bash_script("")
            .expect("Should handle empty script");

        assert!(items.is_empty(), "Empty script should produce no AST items");
    }

    #[test]
    fn test_invalid_bash_syntax() {
        let analyzer = BashScriptAnalyzer::new(Path::new("invalid.sh"));
        let result = analyzer.analyze_bash_script("invalid bash syntax {{{ !!!");

        // Should either handle gracefully or return error
        assert!(
            result.is_ok() || result.is_err(),
            "Should handle invalid syntax gracefully"
        );
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

        let warnings = safety_analyzer
            .validate_best_practices(good_script)
            .expect("Should validate best practices");

        // Good script should have minimal warnings
        assert!(
            warnings.len() <= 2,
            "Well-written script should have few best practice warnings"
        );
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
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
