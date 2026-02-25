// Unit tests for AgentsMdParser
// Contains: all test cases for parsing, extraction, validation, and detection

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_document() {
        let parser = AgentsMdParser::new();
        let result = parser.parse("");
        assert!(result.is_ok());
        let doc = result.expect("internal error");
        assert!(doc.sections.is_empty());
        assert!(doc.commands.is_empty());
    }

    #[test]
    fn test_parse_basic_sections() {
        let content = r#"
# AGENTS.md

## Project Overview
This is a test project.

## Testing Instructions
Run `cargo test` to execute tests.

## Code Style
- Use rustfmt
- Follow clippy suggestions
"#;

        let parser = AgentsMdParser::new();
        let result = parser.parse(content);
        assert!(result.is_ok());

        let doc = result.expect("internal error");
        assert_eq!(doc.sections.len(), 4); // Including the AGENTS.md header section

        // Check section types
        assert!(doc
            .sections
            .iter()
            .any(|s| matches!(s.section_type, SectionType::Overview)));
        assert!(doc
            .sections
            .iter()
            .any(|s| matches!(s.section_type, SectionType::Testing)));
        assert!(doc
            .sections
            .iter()
            .any(|s| matches!(s.section_type, SectionType::CodeStyle)));
    }

    #[test]
    fn test_extract_commands() {
        let content = r#"
## Dev Setup

Install dependencies:
```bash
cargo build --all
cargo test
```

Run the application:
```sh
cargo run --release
```
"#;

        let parser = AgentsMdParser::new();
        let result = parser.parse(content);
        assert!(result.is_ok());

        let doc = result.expect("internal error");
        assert_eq!(doc.commands.len(), 3);
        assert_eq!(doc.commands[0].command, "cargo build --all");
        assert_eq!(doc.commands[1].command, "cargo test");
        assert_eq!(doc.commands[2].command, "cargo run --release");
    }

    #[test]
    fn test_extract_guidelines() {
        let content = r#"
## Code Style

- Must use rustfmt for formatting
- Should follow clippy recommendations
- Prefer explicit types over inference
* Document all public APIs
"#;

        let parser = AgentsMdParser::new();
        let result = parser.parse(content);
        assert!(result.is_ok());

        let doc = result.expect("internal error");
        assert_eq!(doc.guidelines.len(), 4);

        // Check priorities
        assert_eq!(doc.guidelines[0].priority, Priority::Critical); // "Must"
        assert_eq!(doc.guidelines[1].priority, Priority::High); // "Should"
        assert_eq!(doc.guidelines[2].priority, Priority::Medium); // "Prefer"
    }

    #[test]
    fn test_detect_unsafe_commands() {
        let parser = AgentsMdParser::new();

        assert!(!parser.is_command_safe("rm -rf /"));
        assert!(!parser.is_command_safe("sudo rm -rf /"));
        assert!(!parser.is_command_safe("chmod 777 /etc/passwd"));
        assert!(!parser.is_command_safe("eval $USER_INPUT"));

        assert!(parser.is_command_safe("cargo build"));
        assert!(parser.is_command_safe("npm test"));
        assert!(parser.is_command_safe("make clean"));
    }

    #[test]
    fn test_extract_quality_rules() {
        let content = r#"
## Quality Requirements

All functions must have complexity less than 10.
Maintain test coverage above 80%.
Technical debt is not allowed in production code.
"#;

        let parser = AgentsMdParser::new();
        let result = parser.parse(content);
        assert!(result.is_ok());

        let doc = result.expect("internal error");
        assert!(doc.quality_rules.is_some());

        let rules = doc.quality_rules.expect("internal error");
        assert_eq!(rules.max_complexity, Some(10));
        assert_eq!(rules.min_coverage, Some(80.0));
        assert!(!rules.satd_allowed);
    }

    #[test]
    fn test_validation_required_sections() {
        let parser = AgentsMdParser::with_rules(ValidationRules {
            require_overview: true,
            require_testing: true,
            ..Default::default()
        });

        let content = "## Code Style\nUse rustfmt";
        let doc = parser.parse(content).expect("internal error");
        let report = parser.validate(&doc).expect("internal error");

        assert!(!report.valid);
        assert_eq!(report.errors.len(), 2);
        assert!(report.errors.iter().any(|e| e.message.contains("Overview")));
        assert!(report.errors.iter().any(|e| e.message.contains("Testing")));
    }

    #[test]
    fn test_size_limit() {
        let parser = AgentsMdParser::with_rules(ValidationRules {
            max_size: 10,
            ..Default::default()
        });

        let content = "This content is too long for the limit";
        let result = parser.parse(content);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("exceeds maximum size"));
    }

    #[test]
    fn test_section_type_detection() {
        let _parser = AgentsMdParser::new();

        assert_eq!(
            AgentsMdParser::detect_section_type("Project Overview"),
            SectionType::Overview
        );
        assert_eq!(
            AgentsMdParser::detect_section_type("Development Environment"),
            SectionType::DevEnvironment
        );
        assert_eq!(
            AgentsMdParser::detect_section_type("Testing Instructions"),
            SectionType::Testing
        );
        assert_eq!(
            AgentsMdParser::detect_section_type("PR Guidelines"),
            SectionType::PRGuidelines
        );
        assert_eq!(
            AgentsMdParser::detect_section_type("Security Considerations"),
            SectionType::Security
        );
        assert_eq!(
            AgentsMdParser::detect_section_type("Custom Section"),
            SectionType::Custom("Custom Section".to_string())
        );
    }
}
