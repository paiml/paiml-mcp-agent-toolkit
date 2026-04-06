//! Classification logic for SATD debt patterns.

use regex::RegexSet;

use super::types::{AstContext, AstNodeType, DebtCategory, DebtClassifier, DebtPattern, Severity};

impl Default for DebtClassifier {
    fn default() -> Self {
        Self::new()
    }
}

impl DebtClassifier {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        // Default mode includes all patterns
        let patterns = vec![
            // High-confidence patterns with word boundaries
            DebtPattern {
                regex: r"(?i)\b(hack|kludge|smell|xxx)\b".to_string(),
                category: DebtCategory::Design,
                severity: Severity::Medium,
                description: "Architectural compromise".to_string(),
            },
            DebtPattern {
                regex: r"(?i)\b(fixme|broken|bug)\b".to_string(),
                category: DebtCategory::Defect,
                severity: Severity::High,
                description: "Known defect".to_string(),
            },
            DebtPattern {
                regex: r"(?i)\btodo\b".to_string(),
                category: DebtCategory::Requirement,
                severity: Severity::Low,
                description: "Missing feature".to_string(),
            },
            DebtPattern {
                regex: r"(?i)\b(security|vuln|vulnerability|cve|xss)\b".to_string(),
                category: DebtCategory::Security,
                severity: Severity::Critical,
                description: "Security concern".to_string(),
            },
            // Context-aware patterns
            DebtPattern {
                regex: r"(?i)\bperformance\s+(issue|problem)\b".to_string(),
                category: DebtCategory::Performance,
                severity: Severity::Medium,
                description: "Performance issue".to_string(),
            },
            DebtPattern {
                regex: r"(?i)\btest.*\b(disabled|skipped|failing)\b".to_string(),
                category: DebtCategory::Test,
                severity: Severity::Medium,
                description: "Test debt".to_string(),
            },
            // Multi-word patterns
            DebtPattern {
                regex: r"(?i)\btechnical\s+debt\b".to_string(),
                category: DebtCategory::Design,
                severity: Severity::Medium,
                description: "Explicit technical debt".to_string(),
            },
            DebtPattern {
                regex: r"(?i)\bcode\s+smell\b".to_string(),
                category: DebtCategory::Design,
                severity: Severity::Medium,
                description: "Code smell".to_string(),
            },
            // Additional common patterns
            DebtPattern {
                regex: r"(?i)\b(workaround|temp|temporary)\b".to_string(),
                category: DebtCategory::Design,
                severity: Severity::Low,
                description: "Temporary solution".to_string(),
            },
            DebtPattern {
                regex: r"(?i)\b(optimize|slow)\b".to_string(),
                category: DebtCategory::Performance,
                severity: Severity::Low,
                description: "Performance optimization needed".to_string(),
            },
        ];

        let regex_strings: Vec<&str> = patterns.iter().map(|p| p.regex.as_str()).collect();
        let compiled_patterns =
            RegexSet::new(&regex_strings).expect("Failed to compile SATD patterns");

        Self {
            patterns,
            compiled_patterns,
        }
    }

    /// Extended mode: includes euphemisms used by AI coding assistants to bypass SATD detection
    /// Detects: placeholder, stub, simplified, demo, mock, dummy, fake, hardcoded, "for now", WIP
    /// See issue #149: https://github.com/paiml/paiml-mcp-agent-toolkit/issues/149
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new_extended() -> Self {
        // Extended mode includes standard patterns PLUS euphemism patterns
        let mut patterns = vec![
            // Standard patterns (same as new())
            DebtPattern {
                regex: r"(?i)\b(hack|kludge|smell|xxx)\b".to_string(),
                category: DebtCategory::Design,
                severity: Severity::Medium,
                description: "Architectural compromise".to_string(),
            },
            DebtPattern {
                regex: r"(?i)\b(fixme|broken|bug)\b".to_string(),
                category: DebtCategory::Defect,
                severity: Severity::High,
                description: "Known defect".to_string(),
            },
            DebtPattern {
                regex: r"(?i)\btodo\b".to_string(),
                category: DebtCategory::Requirement,
                severity: Severity::Low,
                description: "Missing feature".to_string(),
            },
            DebtPattern {
                regex: r"(?i)\b(security|vuln|vulnerability|cve|xss)\b".to_string(),
                category: DebtCategory::Security,
                severity: Severity::Critical,
                description: "Security concern".to_string(),
            },
            DebtPattern {
                regex: r"(?i)\bperformance\s+(issue|problem)\b".to_string(),
                category: DebtCategory::Performance,
                severity: Severity::Medium,
                description: "Performance issue".to_string(),
            },
            DebtPattern {
                regex: r"(?i)\btest.*\b(disabled|skipped|failing)\b".to_string(),
                category: DebtCategory::Test,
                severity: Severity::Medium,
                description: "Test debt".to_string(),
            },
            DebtPattern {
                regex: r"(?i)\btechnical\s+debt\b".to_string(),
                category: DebtCategory::Design,
                severity: Severity::Medium,
                description: "Explicit technical debt".to_string(),
            },
            DebtPattern {
                regex: r"(?i)\bcode\s+smell\b".to_string(),
                category: DebtCategory::Design,
                severity: Severity::Medium,
                description: "Code smell".to_string(),
            },
            DebtPattern {
                regex: r"(?i)\b(workaround|temp|temporary)\b".to_string(),
                category: DebtCategory::Design,
                severity: Severity::Low,
                description: "Temporary solution".to_string(),
            },
            DebtPattern {
                regex: r"(?i)\b(optimize|slow)\b".to_string(),
                category: DebtCategory::Performance,
                severity: Severity::Low,
                description: "Performance optimization needed".to_string(),
            },
        ];

        // EXTENDED PATTERNS: Euphemisms that hide technical debt (issue #149)
        // These are commonly used by AI coding assistants to bypass SATD detection
        let extended_patterns = vec![
            // Placeholder patterns - indicate incomplete implementation
            DebtPattern {
                regex: r"(?i)\bplaceholder\b".to_string(),
                category: DebtCategory::Requirement,
                severity: Severity::Medium,
                description: "Placeholder - incomplete implementation".to_string(),
            },
            // Stub patterns - indicate missing implementation
            DebtPattern {
                regex: r"(?i)\bstub\b".to_string(),
                category: DebtCategory::Requirement,
                severity: Severity::Medium,
                description: "Stub - missing implementation".to_string(),
            },
            // Simplified patterns - indicate corners were cut
            DebtPattern {
                regex: r"(?i)\bsimplified\b".to_string(),
                category: DebtCategory::Design,
                severity: Severity::Low,
                description: "Simplified - corners cut".to_string(),
            },
            // Demo/demonstration patterns - indicate non-production code
            DebtPattern {
                regex: r"(?i)\b(for\s+)?demonstrat(e|ion)\b".to_string(),
                category: DebtCategory::Requirement,
                severity: Severity::Low,
                description: "Demo code - not production ready".to_string(),
            },
            // Mock/dummy/fake patterns - indicate fake implementations
            DebtPattern {
                regex: r"(?i)\b(mock|dummy|fake)\b".to_string(),
                category: DebtCategory::Test,
                severity: Severity::Low,
                description: "Mock/dummy - fake implementation".to_string(),
            },
            // Hardcoded patterns - indicate missing configuration
            DebtPattern {
                regex: r"(?i)\bhardcoded\b".to_string(),
                category: DebtCategory::Design,
                severity: Severity::Medium,
                description: "Hardcoded - missing configuration".to_string(),
            },
            // "For now" patterns - indicate temporary solutions
            DebtPattern {
                regex: r"(?i)\bfor\s+now\b".to_string(),
                category: DebtCategory::Design,
                severity: Severity::Medium,
                description: "For now - temporary solution".to_string(),
            },
            // WIP patterns - work in progress
            DebtPattern {
                regex: r"\bWIP\b".to_string(),
                category: DebtCategory::Requirement,
                severity: Severity::Medium,
                description: "WIP - work in progress".to_string(),
            },
            // Skip/bypass patterns - indicate missing validation
            DebtPattern {
                regex: r"(?i)\b(skip|bypass)\s+(for\s+now|this|validation)\b".to_string(),
                category: DebtCategory::Design,
                severity: Severity::High,
                description: "Skip/bypass - missing validation".to_string(),
            },
        ];

        patterns.extend(extended_patterns);

        let regex_strings: Vec<&str> = patterns.iter().map(|p| p.regex.as_str()).collect();
        let compiled_patterns =
            RegexSet::new(&regex_strings).expect("Failed to compile extended SATD patterns");

        Self {
            patterns,
            compiled_patterns,
        }
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new_strict() -> Self {
        // Strict mode only includes explicit SATD markers
        let patterns = vec![
            // Ultra-strict: ONLY actual comment-based SATD markers
            DebtPattern {
                regex: r"//\s*TODO:\s+.+".to_string(),
                category: DebtCategory::Requirement,
                severity: Severity::Low,
                description: "TODO task marker".to_string(),
            },
            DebtPattern {
                regex: r"//\s*FIXME:\s+.+".to_string(),
                category: DebtCategory::Defect,
                severity: Severity::High,
                description: "FIXME issue marker".to_string(),
            },
            DebtPattern {
                regex: r"//\s*HACK:\s+.+".to_string(),
                category: DebtCategory::Design,
                severity: Severity::Medium,
                description: "HACK workaround marker".to_string(),
            },
            DebtPattern {
                regex: r"//\s*XXX:\s+.+".to_string(),
                category: DebtCategory::Design,
                severity: Severity::Medium,
                description: "XXX problem marker".to_string(),
            },
            DebtPattern {
                regex: r"//\s*BUG:\s+.+".to_string(),
                category: DebtCategory::Defect,
                severity: Severity::High,
                description: "BUG issue marker".to_string(),
            },
        ];

        let regex_strings: Vec<&str> = patterns.iter().map(|p| p.regex.as_str()).collect();
        let compiled_patterns =
            RegexSet::new(&regex_strings).expect("Failed to compile strict SATD patterns");

        Self {
            patterns,
            compiled_patterns,
        }
    }

    /// Classify a comment text and return debt information
    ///
    /// # Examples
    ///
    /// ```
    /// use pmat::services::satd_detector::{DebtClassifier, DebtCategory, Severity};
    ///
    /// let classifier = DebtClassifier::new();
    ///
    /// // TODO comments are classified as requirements
    /// let result = classifier.classify_comment("TODO: implement this feature");
    /// assert_eq!(result, Some((DebtCategory::Requirement, Severity::Low)));
    ///
    /// // FIXME comments are defects with higher severity
    /// let result = classifier.classify_comment("FIXME: this crashes sometimes");
    /// assert_eq!(result, Some((DebtCategory::Defect, Severity::High)));
    ///
    /// // Normal comments return None
    /// let result = classifier.classify_comment("This is a regular comment");
    /// assert_eq!(result, None);
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn classify_comment(&self, text: &str) -> Option<(DebtCategory, Severity)> {
        debug_assert!(!text.is_empty(), "text must not be empty");
        let matches = self.compiled_patterns.matches(text);

        // Find the first matching pattern
        for match_idx in &matches {
            if let Some(pattern) = self.patterns.get(match_idx) {
                return Some((pattern.category, pattern.severity));
            }
        }

        None
    }

    /// Adjust severity based on context
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn adjust_severity(&self, base_severity: Severity, context: &AstContext) -> Severity {
        match context.node_type {
            // Critical paths escalate severity
            AstNodeType::SecurityFunction | AstNodeType::DataValidation => base_severity.escalate(),
            // Test code reduces severity
            AstNodeType::TestFunction | AstNodeType::MockImplementation => base_severity.reduce(),
            // Hot paths (high complexity) escalate performance issues
            AstNodeType::Regular if context.complexity > 20 => base_severity.escalate(),
            _ => base_severity,
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_extended_creates_valid_classifier() {
        let classifier = DebtClassifier::new_extended();
        // Should have 10 standard + 9 extended = 19 patterns
        assert_eq!(classifier.patterns.len(), 19);
        assert!(classifier.compiled_patterns.len() > 0);
    }

    #[test]
    fn test_new_extended_matches_standard_patterns() {
        let classifier = DebtClassifier::new_extended();
        // Standard SATD markers
        assert!(classifier.compiled_patterns.is_match("TODO: fix this"));
        assert!(classifier.compiled_patterns.is_match("FIXME: broken"));
        assert!(classifier.compiled_patterns.is_match("HACK workaround"));
        assert!(classifier
            .compiled_patterns
            .is_match("security vulnerability"));
        assert!(classifier.compiled_patterns.is_match("performance issue"));
        assert!(classifier.compiled_patterns.is_match("technical debt"));
        assert!(classifier.compiled_patterns.is_match("code smell"));
        assert!(classifier.compiled_patterns.is_match("temporary fix"));
        assert!(classifier.compiled_patterns.is_match("optimize this"));
    }

    #[test]
    fn test_new_extended_matches_euphemism_patterns() {
        let classifier = DebtClassifier::new_extended();
        // AI euphemism patterns (issue #149)
        assert!(classifier.compiled_patterns.is_match("placeholder value"));
        assert!(classifier.compiled_patterns.is_match("stub implementation"));
        assert!(classifier.compiled_patterns.is_match("simplified version"));
        assert!(classifier.compiled_patterns.is_match("for demonstration"));
        assert!(classifier.compiled_patterns.is_match("mock service"));
        assert!(classifier.compiled_patterns.is_match("dummy data"));
        assert!(classifier.compiled_patterns.is_match("fake response"));
        assert!(classifier.compiled_patterns.is_match("hardcoded path"));
        assert!(classifier.compiled_patterns.is_match("for now"));
        assert!(classifier.compiled_patterns.is_match("WIP"));
        assert!(classifier.compiled_patterns.is_match("skip validation"));
        assert!(classifier.compiled_patterns.is_match("bypass this"));
    }

    #[test]
    fn test_new_extended_does_not_match_normal_code() {
        let classifier = DebtClassifier::new_extended();
        assert!(!classifier
            .compiled_patterns
            .is_match("fn process_request()"));
        assert!(!classifier
            .compiled_patterns
            .is_match("let result = compute()"));
        assert!(!classifier.compiled_patterns.is_match("return Ok(value)"));
    }

    #[test]
    fn test_new_extended_has_more_patterns_than_standard() {
        let standard = DebtClassifier::new();
        let extended = DebtClassifier::new_extended();
        assert!(extended.patterns.len() > standard.patterns.len());
    }

    #[test]
    fn test_new_extended_pattern_severities() {
        let classifier = DebtClassifier::new_extended();
        // Check that security patterns are Critical
        let security_pattern = classifier
            .patterns
            .iter()
            .find(|p| p.description == "Security concern")
            .unwrap();
        assert_eq!(security_pattern.severity, Severity::Critical);

        // Check that skip/bypass is High severity
        let skip_pattern = classifier
            .patterns
            .iter()
            .find(|p| p.description.contains("Skip/bypass"))
            .unwrap();
        assert_eq!(skip_pattern.severity, Severity::High);
    }

    #[test]
    fn test_new_extended_pattern_categories() {
        let classifier = DebtClassifier::new_extended();
        let categories: Vec<&DebtCategory> =
            classifier.patterns.iter().map(|p| &p.category).collect();
        // Should contain diverse categories
        assert!(categories.contains(&&DebtCategory::Design));
        assert!(categories.contains(&&DebtCategory::Defect));
        assert!(categories.contains(&&DebtCategory::Requirement));
        assert!(categories.contains(&&DebtCategory::Security));
        assert!(categories.contains(&&DebtCategory::Performance));
        assert!(categories.contains(&&DebtCategory::Test));
    }

    #[test]
    fn test_new_strict_is_subset_of_extended() {
        let strict = DebtClassifier::new_strict();
        let extended = DebtClassifier::new_extended();
        assert!(strict.patterns.len() < extended.patterns.len());
    }

    #[test]
    fn test_classify_line_with_extended() {
        let classifier = DebtClassifier::new_extended();
        // Should classify euphemism patterns
        let matches: Vec<_> = classifier
            .compiled_patterns
            .matches("// placeholder for now")
            .into_iter()
            .collect();
        assert!(!matches.is_empty());
    }
}
