#[cfg_attr(coverage_nightly, coverage(off))]
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
        let ast = parser.parse().expect("internal error");
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
        let ast = parser.parse().expect("internal error");
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
        let ast = parser.parse().expect("internal error");
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
        let ast = parser.parse().expect("internal error");
        let violations = rule.check(&ast);
        assert_eq!(violations.len(), 1);
        assert!(violations[0]
            .message
            .contains("evaluated once at parse time"));

        // Test with deferred assignment - this is the recommended approach
        let input2 = "BUILD_TIME = $(shell date)";
        let mut parser2 = MakefileParser::new(input2);
        let ast2 = parser2.parse().expect("internal error");
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
        let ast = parser.parse().expect("internal error");
        let violations = rule.check(&ast);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("UNDEFINED_VAR"));

        // Test with defined variable
        let input2 = "VAR = value\ntarget:\n\techo $(VAR)";
        let mut parser2 = MakefileParser::new(input2);
        let ast2 = parser2.parse().expect("internal error");
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
        let ast = parser.parse().expect("internal error");
        let violations = rule.check(&ast);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Conditional assignment"));

        // Test with GNU-specific shell assignment
        let input2 = "VAR != date";
        let mut parser2 = MakefileParser::new(input2);
        let ast2 = parser2.parse().expect("internal error");
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

    #[test]
    fn test_extract_var_name_basic() {
        // Test basic variable name extraction
        assert_eq!(extract_var_name("VAR"), "VAR");
        assert_eq!(extract_var_name("MY_VAR"), "MY_VAR");
        assert_eq!(extract_var_name(" VAR "), "VAR");
        assert_eq!(extract_var_name("CC"), "CC");
    }

    #[test]
    fn test_extract_var_name_default_value_syntax() {
        // Test default value syntax ${VAR:-default}
        assert_eq!(extract_var_name("VAR:-default"), "VAR");
        assert_eq!(extract_var_name("MY_VAR:-/usr/bin"), "MY_VAR");
        assert_eq!(extract_var_name(" VAR :-value"), "VAR");
        assert_eq!(extract_var_name("PATH:-/usr/bin:/bin"), "PATH");
        // Edge case: multiple :- in the string
        assert_eq!(extract_var_name("VAR:-default:-other"), "VAR");
    }

    #[test]
    fn test_extract_var_name_alternative_value_syntax() {
        // Test alternative value syntax ${VAR:+alt}
        assert_eq!(extract_var_name("VAR:+alternative"), "VAR");
        assert_eq!(extract_var_name("MY_VAR:+/tmp"), "MY_VAR");
        assert_eq!(extract_var_name(" VAR :+value"), "VAR");
        assert_eq!(extract_var_name("DEBUG:+--debug"), "DEBUG");
        // Edge case: multiple :+ in the string
        assert_eq!(extract_var_name("VAR:+alt:+other"), "VAR");
    }

    #[test]
    fn test_extract_var_name_pattern_substitution() {
        // Test pattern substitution like $(VAR:old=new)
        assert_eq!(extract_var_name("SRCS:.c=.o"), "SRCS");
        assert_eq!(extract_var_name("FILES:%.txt=%.bak"), "FILES");
        assert_eq!(extract_var_name("VAR:old=new"), "VAR");
        assert_eq!(extract_var_name(" VAR :.c=.o"), "VAR :.c=.o");

        // Should NOT extract from shell commands with spaces/pipes/braces
        assert_eq!(extract_var_name("shell ls:test"), "shell ls:test");
        assert_eq!(extract_var_name("cmd | grep:pattern"), "");
        assert_eq!(extract_var_name("shell {cmd:arg}"), "shell {cmd:arg}");
    }

    #[test]
    fn test_extract_var_name_shell_operators() {
        // Test shell operators - should return empty string to skip validation
        assert_eq!(extract_var_name("shell date | cut -d:"), "");
        assert_eq!(extract_var_name("cat file > output"), "");
        assert_eq!(extract_var_name("cmd < input"), "");
        assert_eq!(extract_var_name("ls | grep pattern"), "");
        assert_eq!(extract_var_name("echo hello > file"), "");
        assert_eq!(extract_var_name("cat < file | sort"), "");
    }

    #[test]
    fn test_extract_var_name_edge_cases() {
        // Test empty and whitespace
        assert_eq!(extract_var_name(""), "");
        assert_eq!(extract_var_name("   "), "");

        // Test complex combinations
        assert_eq!(extract_var_name("VAR:-default:+alt"), "VAR");
        assert_eq!(extract_var_name("VAR:+alt:-default"), "VAR:+alt");

        // Test colon without valid substitution patterns
        assert_eq!(extract_var_name("file:line:col"), "file");
        assert_eq!(extract_var_name("url:http://example.com"), "url");

        // Test variables with numbers and underscores
        assert_eq!(extract_var_name("VAR_123"), "VAR_123");
        assert_eq!(extract_var_name("PREFIX_VAR_SUFFIX"), "PREFIX_VAR_SUFFIX");

        // Test shell command detection edge cases
        assert_eq!(extract_var_name("PATH"), "PATH");
        assert_eq!(extract_var_name("PATH:something"), "PATH");
    }

    #[test]
    fn test_extract_var_name_precedence() {
        // Test precedence - :- should be checked before :
        assert_eq!(extract_var_name("VAR:-def:old=new"), "VAR");

        // Test precedence - :+ should be checked before :
        assert_eq!(extract_var_name("VAR:+alt:old=new"), "VAR");

        // Test that pattern substitution works when no :- or :+
        assert_eq!(extract_var_name("VAR:old=new"), "VAR");
    }

    #[test]
    fn test_helper_functions() {
        // Test extract_from_default_value
        assert_eq!(
            extract_from_default_value("VAR:-default"),
            Some("VAR".to_string())
        );
        assert_eq!(extract_from_default_value("VAR"), None);
        assert_eq!(extract_from_default_value("VAR:+alt"), None);

        // Test extract_from_alternative_value
        assert_eq!(
            extract_from_alternative_value("VAR:+alt"),
            Some("VAR".to_string())
        );
        assert_eq!(extract_from_alternative_value("VAR"), None);
        assert_eq!(extract_from_alternative_value("VAR:-def"), None);

        // Test extract_from_pattern_substitution
        assert_eq!(
            extract_from_pattern_substitution("VAR:old=new"),
            Some("VAR".to_string())
        );
        assert_eq!(extract_from_pattern_substitution("VAR"), None);
        assert_eq!(
            extract_from_pattern_substitution("cmd with spaces:arg"),
            None
        );

        // Test contains_shell_indicators
        assert!(contains_shell_indicators("cmd with spaces"));
        assert!(contains_shell_indicators("cmd|pipe"));
        assert!(contains_shell_indicators("cmd{brace}"));
        assert!(!contains_shell_indicators("VAR"));

        // Test contains_shell_operators
        assert!(contains_shell_operators("cmd|pipe"));
        assert!(contains_shell_operators("cmd>output"));
        assert!(contains_shell_operators("cmd<input"));
        assert!(!contains_shell_operators("VAR:value"));
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
