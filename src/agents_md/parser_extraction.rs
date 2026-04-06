// Extraction, detection, and validation methods for AgentsMdParser
// Contains: validate(), extract_sections(), detect_section_type(), extract_commands(),
//           extract_guidelines(), detect_priority(), is_command_safe(),
//           extract_quality_rules(), extract_metadata()

impl AgentsMdParser {
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
    #[must_use]
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
        debug_assert!(!text.is_empty(), "text must not be empty");
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
        debug_assert!(!text.is_empty(), "text must not be empty");
        // Simple extraction: lines starting with - or *
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                let content = trimmed.get(2..).unwrap_or_default();
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
        debug_assert!(!text.is_empty(), "text must not be empty");
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
        debug_assert!(!command.is_empty(), "command must not be empty");
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
        debug_assert!(!sections.is_empty(), "sections must not be empty");
        let mut rules = QualityRules {
            max_complexity: None,
            min_coverage: None,
            satd_allowed: false,
            custom_checks: Vec::new(),
        };

        let mut found_rules = false;

        // Compile regex patterns once before the loop
        let complexity_regex = Regex::new(r"complexity.*?(\d+)").expect("internal error");
        let coverage_regex = Regex::new(r"coverage.*?(\d+)").expect("internal error");

        for section in sections {
            let content = &section.content.to_lowercase();

            // Look for complexity limits
            if content.contains("complexity") {
                if let Some(captures) = complexity_regex.captures(content) {
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
                // Check if it's explicitly allowed (but not "not allowed" or "disallowed")
                rules.satd_allowed = (content.contains("allow") || content.contains("permitted"))
                    && !content.contains("not allow")
                    && !content.contains("disallow")
                    && !content.contains("is not");
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
        debug_assert!(!sections.is_empty(), "sections must not be empty");
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
