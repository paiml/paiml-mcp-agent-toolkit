// cli_checker_tests.rs — Unit tests for CLI documentation checker
// Included by cli_checker.rs — shares parent module scope (no `use` imports here)

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_flags_from_help() {
        let help_text = r#"
Usage: pmat scaffold agent [OPTIONS]

Options:
  -n, --name <NAME>      Agent name
  -t, --template <TEMPLATE>  Template type
  -v, --verbose          Enable verbose output
  -h, --help            Print help
"#;

        let flags = extract_flags_from_help(help_text);
        assert!(flags.contains(&"-n".to_string()));
        assert!(flags.contains(&"--name".to_string()));
        assert!(flags.contains(&"-t".to_string()));
        assert!(flags.contains(&"--template".to_string()));
        assert!(flags.contains(&"-v".to_string()));
        assert!(flags.contains(&"--verbose".to_string()));
    }

    #[test]
    fn test_find_undocumented_flags() {
        let expected = vec!["--name", "--template", "--output", "--force"];
        let documented = vec![
            "--name".to_string(),
            "--template".to_string(),
            "--output".to_string(),
        ];

        let missing = find_undocumented_flags(&expected, &documented);
        assert_eq!(missing, vec!["--force"]);
    }

    #[test]
    fn test_extract_flag_name() {
        assert_eq!(extract_flag_name("  -n, --name <NAME>"), "--name");
        assert_eq!(extract_flag_name("  -v, --verbose"), "--verbose");
        assert_eq!(extract_flag_name("  -h"), "-h");
    }

    #[test]
    fn test_cli_documentation_report_is_valid() {
        let valid_report = CliDocumentationReport {
            command: "test".to_string(),
            has_help: true,
            has_usage_section: true,
            has_options_section: true,
            has_examples_section: true,
            documented_flags: vec!["--help".to_string()],
            generic_descriptions: vec![],
            missing_descriptions: vec![],
            issues: vec![],
        };

        assert!(valid_report.is_valid());
    }

    #[test]
    fn test_cli_documentation_report_invalid_no_help() {
        let invalid_report = CliDocumentationReport {
            command: "test".to_string(),
            has_help: false, // No help
            has_usage_section: true,
            has_options_section: true,
            has_examples_section: false,
            documented_flags: vec![],
            generic_descriptions: vec![],
            missing_descriptions: vec![],
            issues: vec![],
        };

        assert!(!invalid_report.is_valid());
    }

    #[test]
    fn test_cli_documentation_report_invalid_generic_descriptions() {
        let invalid_report = CliDocumentationReport {
            command: "test".to_string(),
            has_help: true,
            has_usage_section: true,
            has_options_section: true,
            has_examples_section: false,
            documented_flags: vec![],
            generic_descriptions: vec!["Flag '--foo': Sets the foo".to_string()],
            missing_descriptions: vec![],
            issues: vec![],
        };

        assert!(!invalid_report.is_valid());
    }

    #[test]
    fn test_validate_sections_has_usage() {
        let mut report = CliDocumentationReport {
            command: "test".to_string(),
            has_help: true,
            has_usage_section: false,
            has_options_section: false,
            has_examples_section: false,
            documented_flags: vec![],
            generic_descriptions: vec![],
            missing_descriptions: vec![],
            issues: vec![],
        };

        let help_text = "Usage: test [OPTIONS]\n\nOptions:\n  --help Print help";
        validate_sections(help_text, &mut report);

        assert!(report.has_usage_section);
        assert!(report.has_options_section);
    }

    #[test]
    fn test_validate_sections_missing_usage() {
        let mut report = CliDocumentationReport {
            command: "test".to_string(),
            has_help: true,
            has_usage_section: false,
            has_options_section: false,
            has_examples_section: false,
            documented_flags: vec![],
            generic_descriptions: vec![],
            missing_descriptions: vec![],
            issues: vec![],
        };

        let help_text = "This is a test program";
        validate_sections(help_text, &mut report);

        assert!(!report.has_usage_section);
        assert!(report
            .issues
            .contains(&"Missing 'Usage:' section".to_string()));
    }

    #[test]
    fn test_validate_sections_has_flags() {
        let mut report = CliDocumentationReport {
            command: "test".to_string(),
            has_help: true,
            has_usage_section: false,
            has_options_section: false,
            has_examples_section: false,
            documented_flags: vec![],
            generic_descriptions: vec![],
            missing_descriptions: vec![],
            issues: vec![],
        };

        let help_text = "Usage: test\n\nFLAGS:\n  --verbose";
        validate_sections(help_text, &mut report);

        assert!(report.has_options_section);
    }

    #[test]
    fn test_validate_sections_has_examples() {
        let mut report = CliDocumentationReport {
            command: "test".to_string(),
            has_help: true,
            has_usage_section: false,
            has_options_section: false,
            has_examples_section: false,
            documented_flags: vec![],
            generic_descriptions: vec![],
            missing_descriptions: vec![],
            issues: vec![],
        };

        let help_text = "Usage: test\n\nExample:\n  test --help";
        validate_sections(help_text, &mut report);

        assert!(report.has_examples_section);
    }

    #[test]
    fn test_is_options_section_start() {
        assert!(is_options_section_start("Options:"));
        assert!(is_options_section_start("FLAGS:"));
        assert!(!is_options_section_start("Usage:"));
        assert!(!is_options_section_start("Example:"));
    }

    #[test]
    fn test_is_section_boundary() {
        // Not in section - always false
        assert!(!is_section_boundary("Some text", false));

        // In section with indented line - not boundary
        assert!(!is_section_boundary("  -h, --help", true));

        // In section with empty line - not boundary
        assert!(!is_section_boundary("", true));

        // In section with non-indented non-empty line - boundary
        assert!(is_section_boundary("Arguments:", true));
    }

    #[test]
    fn test_parse_flags_from_line() {
        assert_eq!(
            parse_flags_from_line("  -n, --name <NAME>"),
            Some(vec!["-n".to_string(), "--name".to_string()])
        );

        assert_eq!(
            parse_flags_from_line("  --verbose"),
            Some(vec!["--verbose".to_string()])
        );

        assert_eq!(parse_flags_from_line("This is not a flag line"), None);
    }

    #[test]
    fn test_extract_description_from_flag_line() {
        let description =
            extract_description_from_flag_line("  -n, --name <NAME>   The name to use");
        assert_eq!(description, Some("The name to use".to_string()));
    }

    #[test]
    fn test_extract_description_from_flag_line_no_description() {
        let description = extract_description_from_flag_line("  --verbose");
        assert!(description.is_none() || description.as_ref().is_some_and(|d| d.is_empty()));
    }

    #[test]
    fn test_extract_flag_name_unknown() {
        assert_eq!(extract_flag_name("not a flag line"), "unknown");
    }

    #[test]
    fn test_extract_options_section_lines() {
        let help_text = r#"Usage: test

Options:
  -h, --help     Print help
  -v, --verbose  Verbose output

Arguments:
  <FILE>  Input file
"#;

        let lines = extract_options_section_lines(help_text);
        assert!(lines.iter().any(|l| l.contains("--help")));
        assert!(lines.iter().any(|l| l.contains("--verbose")));
        // Should not include lines from Arguments section
        assert!(!lines.iter().any(|l| l.contains("<FILE>")));
    }

    #[test]
    fn test_find_undocumented_flags_all_documented() {
        let expected = vec!["--name", "--output"];
        let documented = vec!["--name".to_string(), "--output".to_string()];

        let missing = find_undocumented_flags(&expected, &documented);
        assert!(missing.is_empty());
    }
}
