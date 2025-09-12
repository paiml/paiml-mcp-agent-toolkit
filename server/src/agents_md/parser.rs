//! AGENTS.md Parser Implementation
//!
//! Parses markdown files following the AGENTS.md specification with quality validation.

use super::*;
use anyhow::Result;
use pulldown_cmark::{Event, HeadingLevel, Parser as MarkdownParser, Tag, TagEnd};
use regex::Regex;
use std::collections::HashMap;

/// Parser for AGENTS.md documents
pub struct AgentsMdParser {
    /// Validation rules
    validation_rules: ValidationRules,

    /// Command patterns
    command_patterns: Vec<Regex>,
}

/// Validation rules for parsing
#[derive(Debug, Clone)]
pub struct ValidationRules {
    /// Require project overview section
    pub require_overview: bool,

    /// Require testing instructions
    pub require_testing: bool,

    /// Maximum document size in bytes
    pub max_size: usize,

    /// Allowed section types
    pub allowed_sections: Vec<SectionType>,
}

impl Default for ValidationRules {
    fn default() -> Self {
        Self {
            require_overview: false,
            require_testing: false,
            max_size: 1024 * 1024, // 1MB
            allowed_sections: vec![
                SectionType::Overview,
                SectionType::DevEnvironment,
                SectionType::Testing,
                SectionType::CodeStyle,
                SectionType::PRGuidelines,
                SectionType::Security,
            ],
        }
    }
}

/// Validation report
#[derive(Debug, Clone)]
pub struct ValidationReport {
    /// Whether validation passed
    pub valid: bool,

    /// Validation errors
    pub errors: Vec<ValidationError>,

    /// Validation warnings
    pub warnings: Vec<ValidationWarning>,
}

/// Validation error
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Error message
    pub message: String,

    /// Line number if applicable
    pub line: Option<usize>,

    /// Section where error occurred
    pub section: Option<String>,
}

/// Validation warning
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    /// Warning message
    pub message: String,

    /// Severity level
    pub severity: WarningSeverity,
}

/// Warning severity levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WarningSeverity {
    Low,
    Medium,
    High,
}

impl Default for AgentsMdParser {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentsMdParser {
    /// Create new parser with default rules
    pub fn new() -> Self {
        Self::with_rules(ValidationRules::default())
    }

    /// Create parser with custom rules
    pub fn with_rules(rules: ValidationRules) -> Self {
        Self {
            validation_rules: rules,
            command_patterns: Self::init_command_patterns(),
        }
    }

    /// Initialize command detection patterns
    fn init_command_patterns() -> Vec<Regex> {
        vec![
            Regex::new(r"^```(?:bash|sh|shell)\n(.*?)\n```").unwrap(),
            Regex::new(r"^\$ (.+)$").unwrap(),
            Regex::new(r"^> (.+)$").unwrap(),
        ]
    }

    /// Parse AGENTS.md content
    pub fn parse(&self, content: &str) -> Result<AgentsMdDocument> {
        // Check size limit
        if content.len() > self.validation_rules.max_size {
            return Err(anyhow::anyhow!(
                "Document exceeds maximum size of {} bytes",
                self.validation_rules.max_size
            ));
        }

        let mut document = AgentsMdDocument {
            metadata: DocumentMetadata {
                path: PathBuf::new(),
                modified: std::time::SystemTime::now(),
                version: None,
                project: None,
            },
            sections: Vec::new(),
            commands: Vec::new(),
            guidelines: Vec::new(),
            quality_rules: None,
        };

        // Parse markdown
        let parser = MarkdownParser::new(content);
        let mut current_section: Option<Section> = None;
        let mut current_heading_level = 0;
        let mut in_code_block = false;
        let mut code_block_content = String::new();
        let mut code_block_lang = String::new();

        for event in parser {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    current_heading_level = match level {
                        HeadingLevel::H1 => 1,
                        HeadingLevel::H2 => 2,
                        HeadingLevel::H3 => 3,
                        HeadingLevel::H4 => 4,
                        HeadingLevel::H5 => 5,
                        HeadingLevel::H6 => 6,
                    };
                }
                Event::Text(text) => {
                    if in_code_block {
                        code_block_content.push_str(&text);
                    } else if current_heading_level > 0 {
                        // Start new section
                        if let Some(section) = current_section.take() {
                            document.sections.push(section);
                        }

                        let section_type = Self::detect_section_type(&text);
                        current_section = Some(Section {
                            section_type,
                            title: text.to_string(),
                            content: String::new(),
                            subsections: Vec::new(),
                        });
                        current_heading_level = 0;
                    } else if let Some(ref mut section) = current_section {
                        section.content.push_str(&text);
                        section.content.push('\n');

                        // Extract commands from content
                        self.extract_commands(&text, &mut document.commands);

                        // Extract guidelines
                        self.extract_guidelines(
                            &text,
                            &section.section_type,
                            &mut document.guidelines,
                        );
                    }
                }
                Event::Start(Tag::CodeBlock(kind)) => {
                    in_code_block = true;
                    if let pulldown_cmark::CodeBlockKind::Fenced(lang) = kind {
                        code_block_lang = lang.to_string();
                    }
                }
                Event::End(TagEnd::CodeBlock) => {
                    if in_code_block {
                        if code_block_lang == "bash"
                            || code_block_lang == "sh"
                            || code_block_lang == "shell"
                        {
                            // Extract command from code block
                            for line in code_block_content.lines() {
                                if !line.trim().is_empty() && !line.trim().starts_with('#') {
                                    document.commands.push(Command {
                                        name: format!(
                                            "Command from {}",
                                            current_section
                                                .as_ref()
                                                .map(|s| &s.title)
                                                .unwrap_or(&"Unknown".to_string())
                                        ),
                                        command: line.trim().to_string(),
                                        working_dir: None,
                                        env: Vec::new(),
                                        timeout: Some(60),
                                        safe: self.is_command_safe(line),
                                    });
                                }
                            }
                        }

                        // Add code block to current section content
                        if let Some(ref mut section) = current_section {
                            section.content.push_str(&format!(
                                "```{code_block_lang}\n{code_block_content}\n```\n"
                            ));
                        }

                        in_code_block = false;
                        code_block_content.clear();
                        code_block_lang.clear();
                    }
                }
                _ => {}
            }
        }

        // Add final section
        if let Some(section) = current_section {
            document.sections.push(section);
        }

        // Extract quality rules
        document.quality_rules = self.extract_quality_rules(&document.sections);

        // Extract metadata
        self.extract_metadata(&document.sections, &mut document.metadata);

        Ok(document)
    }

    /// Validate parsed document
    pub fn validate(&self, doc: &AgentsMdDocument) -> Result<ValidationReport> {
        let mut report = ValidationReport {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        };

        // Check required sections
        if self.validation_rules.require_overview
            && !doc
                .sections
                .iter()
                .any(|s| matches!(s.section_type, SectionType::Overview))
            {
                report.errors.push(ValidationError {
                    message: "Missing required Overview section".to_string(),
                    line: None,
                    section: None,
                });
                report.valid = false;
            }

        if self.validation_rules.require_testing
            && !doc
                .sections
                .iter()
                .any(|s| matches!(s.section_type, SectionType::Testing))
            {
                report.errors.push(ValidationError {
                    message: "Missing required Testing section".to_string(),
                    line: None,
                    section: None,
                });
                report.valid = false;
            }

        // Check for unsafe commands
        for command in &doc.commands {
            if !command.safe {
                report.warnings.push(ValidationWarning {
                    message: format!("Potentially unsafe command: {}", command.command),
                    severity: WarningSeverity::High,
                });
            }
        }

        // Check quality rules consistency
        if let Some(ref rules) = doc.quality_rules {
            if let Some(coverage) = rules.min_coverage {
                if !(0.0..=100.0).contains(&coverage) {
                    report.errors.push(ValidationError {
                        message: format!("Invalid coverage requirement: {coverage}%"),
                        line: None,
                        section: Some("Quality Rules".to_string()),
                    });
                    report.valid = false;
                }
            }
        }

        Ok(report)
    }

    /// Extract sections by type
    pub fn extract_sections(&self, doc: &AgentsMdDocument) -> HashMap<SectionType, Section> {
        let mut map = HashMap::new();
        for section in &doc.sections {
            map.insert(section.section_type.clone(), section.clone());
        }
        map
    }

    /// Detect section type from title
    fn detect_section_type(title: &str) -> SectionType {
        let lower = title.to_lowercase();

        if lower.contains("overview") || lower.contains("introduction") || lower.contains("about") {
            SectionType::Overview
        } else if lower.contains("dev") || lower.contains("environment") || lower.contains("setup")
        {
            SectionType::DevEnvironment
        } else if lower.contains("test") {
            SectionType::Testing
        } else if lower.contains("style") || lower.contains("format") || lower.contains("lint") {
            SectionType::CodeStyle
        } else if lower.contains("pr") || lower.contains("pull request") || lower.contains("commit")
        {
            SectionType::PRGuidelines
        } else if lower.contains("security") || lower.contains("safety") {
            SectionType::Security
        } else {
            SectionType::Custom(title.to_string())
        }
    }

    /// Extract commands from text
    fn extract_commands(&self, text: &str, commands: &mut Vec<Command>) {
        for pattern in &self.command_patterns {
            if let Some(captures) = pattern.captures(text) {
                if let Some(cmd) = captures.get(1) {
                    commands.push(Command {
                        name: "Extracted command".to_string(),
                        command: cmd.as_str().to_string(),
                        working_dir: None,
                        env: Vec::new(),
                        timeout: Some(60),
                        safe: self.is_command_safe(cmd.as_str()),
                    });
                }
            }
        }
    }

    /// Extract guidelines from text
    fn extract_guidelines(
        &self,
        text: &str,
        section_type: &SectionType,
        guidelines: &mut Vec<Guideline>,
    ) {
        // Simple extraction: lines starting with - or *
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                let content = &trimmed[2..];
                let priority = self.detect_priority(content);

                guidelines.push(Guideline {
                    category: format!("{section_type:?}"),
                    text: content.to_string(),
                    priority,
                });
            }
        }
    }

    /// Detect priority from guideline text
    fn detect_priority(&self, text: &str) -> Priority {
        let lower = text.to_lowercase();

        if lower.contains("must") || lower.contains("critical") || lower.contains("required") {
            Priority::Critical
        } else if lower.contains("should") || lower.contains("important") {
            Priority::High
        } else if lower.contains("recommend") || lower.contains("prefer") {
            Priority::Medium
        } else {
            Priority::Low
        }
    }

    /// Check if command is safe to execute
    fn is_command_safe(&self, command: &str) -> bool {
        let dangerous_patterns = [
            "rm -rf",
            "sudo",
            "chmod 777",
            "eval",
            "exec",
            "> /dev/",
            "dd if=",
        ];

        let lower = command.to_lowercase();
        !dangerous_patterns
            .iter()
            .any(|pattern| lower.contains(pattern))
    }

    /// Extract quality rules from sections
    fn extract_quality_rules(&self, sections: &[Section]) -> Option<QualityRules> {
        let mut rules = QualityRules {
            max_complexity: None,
            min_coverage: None,
            satd_allowed: false,
            custom_checks: Vec::new(),
        };

        let mut found_rules = false;

        // Compile regex patterns once before the loop
        let complexity_regex = Regex::new(r"complexity.*?(\d+)").unwrap();
        let coverage_regex = Regex::new(r"coverage.*?(\d+)").unwrap();

        for section in sections {
            let content = &section.content.to_lowercase();

            // Look for complexity limits
            if content.contains("complexity") {
                if let Some(captures) = complexity_regex.captures(content)
                {
                    if let Some(num) = captures.get(1) {
                        rules.max_complexity = num.as_str().parse().ok();
                        found_rules = true;
                    }
                }
            }

            // Look for coverage requirements
            if content.contains("coverage") {
                if let Some(captures) = coverage_regex.captures(content) {
                    if let Some(num) = captures.get(1) {
                        rules.min_coverage = num.as_str().parse::<f64>().ok();
                        found_rules = true;
                    }
                }
            }

            // Check SATD policy
            if content.contains("satd") || content.contains("technical debt") {
                rules.satd_allowed = content.contains("allow") || content.contains("permitted");
                found_rules = true;
            }
        }

        if found_rules {
            Some(rules)
        } else {
            None
        }
    }

    /// Extract metadata from document
    fn extract_metadata(&self, sections: &[Section], metadata: &mut DocumentMetadata) {
        // Look for project name in overview
        for section in sections {
            if matches!(section.section_type, SectionType::Overview) {
                // Simple extraction: first line often contains project name
                if let Some(first_line) = section.content.lines().next() {
                    metadata.project = Some(first_line.trim().to_string());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_document() {
        let parser = AgentsMdParser::new();
        let result = parser.parse("");
        assert!(result.is_ok());
        let doc = result.unwrap();
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

        let doc = result.unwrap();
        assert_eq!(doc.sections.len(), 3);

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

        let doc = result.unwrap();
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

        let doc = result.unwrap();
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

        let doc = result.unwrap();
        assert!(doc.quality_rules.is_some());

        let rules = doc.quality_rules.unwrap();
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
        let doc = parser.parse(content).unwrap();
        let report = parser.validate(&doc).unwrap();

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
