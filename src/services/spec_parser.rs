//! Specification Parser Service (Part C: Specification Parsing Enhancement)
//!
//! Parses markdown specification files from docs/specifications/*.md
//! and extracts validation criteria for the pmat qa command.
//!
//! # Architecture (Toyota Way - Genchi Genbutsu)
//!
//! Go to the source: extract validation criteria directly from specification files
//! rather than duplicating them in separate configuration.
//!
//! # References
//!
//! - Specification: docs/specifications/enhance-pmat-work.md
//! - Related Issues: #102, #113, #114, #116

use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Parsed specification with validation criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedSpec {
    /// Specification file path
    pub path: PathBuf,

    /// Title from YAML frontmatter or first H1
    pub title: String,

    /// Issue/ticket references (e.g., "#118", "GH-102")
    pub issue_refs: Vec<String>,

    /// Status from frontmatter
    pub status: Option<String>,

    /// Extracted validation claims
    pub claims: Vec<ValidationClaim>,

    /// Code examples that can be validated
    pub code_examples: Vec<CodeExample>,

    /// Acceptance criteria (checkbox items)
    pub acceptance_criteria: Vec<AcceptanceCriterion>,

    /// Test requirements mentioned
    pub test_requirements: Vec<TestRequirement>,
}

/// A falsifiable claim extracted from the specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationClaim {
    /// Unique ID for the claim (e.g., "A1", "B2")
    pub id: String,

    /// The claim text
    pub text: String,

    /// Source location (line number)
    pub line: usize,

    /// Claim category
    pub category: ClaimCategory,

    /// Whether this claim can be automatically validated
    pub automatable: bool,

    /// Validation command (if automatable)
    pub validation_cmd: Option<String>,

    /// Expected result pattern
    pub expected_pattern: Option<String>,
}

/// Claim categories for the 100-point Popperian framework
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClaimCategory {
    /// A. Falsifiability (25 pts) - GATEWAY
    Falsifiability,
    /// B. Implementation (25 pts)
    Implementation,
    /// C. Testing (20 pts)
    Testing,
    /// D. Documentation (15 pts)
    Documentation,
    /// E. Integration (15 pts)
    Integration,
}

impl ClaimCategory {
    pub fn max_points(&self) -> u32 {
        match self {
            Self::Falsifiability => 25,
            Self::Implementation => 25,
            Self::Testing => 20,
            Self::Documentation => 15,
            Self::Integration => 15,
        }
    }

    pub fn from_section(section: &str) -> Option<Self> {
        let lower = section.to_lowercase();
        if lower.contains("falsif") || lower.contains("testab") || lower.contains("claim") {
            Some(Self::Falsifiability)
        } else if lower.contains("implement")
            || lower.contains("code")
            || lower.contains("architecture")
        {
            Some(Self::Implementation)
        } else if lower.contains("test") || lower.contains("coverage") || lower.contains("mutation")
        {
            Some(Self::Testing)
        } else if lower.contains("doc") || lower.contains("readme") || lower.contains("changelog") {
            Some(Self::Documentation)
        } else if lower.contains("integrat") || lower.contains("ci") || lower.contains("deploy") {
            Some(Self::Integration)
        } else {
            None
        }
    }
}

/// Code example from specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExample {
    /// Language (rust, bash, etc.)
    pub language: String,

    /// Code content
    pub code: String,

    /// Line number in source
    pub line: usize,

    /// Whether this is executable
    pub executable: bool,
}

/// Acceptance criterion with completion status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceCriterion {
    /// Criterion text
    pub text: String,

    /// Whether marked as complete (checked)
    pub complete: bool,

    /// Line number in source
    pub line: usize,
}

/// Test requirement extracted from specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRequirement {
    /// Requirement text
    pub text: String,

    /// Type of test (unit, integration, property, e2e)
    pub test_type: String,

    /// Related code path if mentioned
    pub code_path: Option<String>,
}

/// Specification parser
pub struct SpecParser {
    /// Regex for YAML frontmatter
    frontmatter_regex: Regex,
    /// Regex for checkbox items
    checkbox_regex: Regex,
    /// Regex for issue references
    issue_ref_regex: Regex,
    /// Regex for claims (numbered items, MUST/SHALL/SHOULD)
    claim_regex: Regex,
}

impl Default for SpecParser {
    fn default() -> Self {
        Self::new()
    }
}

impl SpecParser {
    pub fn new() -> Self {
        Self {
            frontmatter_regex: Regex::new(r"(?s)^---\n(.*?)\n---").expect("internal error"),
            checkbox_regex: Regex::new(r"^\s*-\s*\[([ xX])\]\s*(.+)$").expect("internal error"),
            issue_ref_regex: Regex::new(r"(?:#(\d+)|GH-(\d+)|Issue\s+#?(\d+))")
                .expect("internal error"),
            claim_regex: Regex::new(r"(?i)(must|shall|should|will)\s+(.+)")
                .expect("internal error"),
        }
    }

    /// Parse a specification file
    pub fn parse_file(&self, path: &Path) -> Result<ParsedSpec> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read specification: {}", path.display()))?;

        self.parse_content(&content, path)
    }

    /// Parse specification content
    pub fn parse_content(&self, content: &str, path: &Path) -> Result<ParsedSpec> {
        let mut spec = ParsedSpec {
            path: path.to_path_buf(),
            title: String::new(),
            issue_refs: Vec::new(),
            status: None,
            claims: Vec::new(),
            code_examples: Vec::new(),
            acceptance_criteria: Vec::new(),
            test_requirements: Vec::new(),
        };

        // Extract frontmatter
        if let Some(caps) = self.frontmatter_regex.captures(content) {
            let frontmatter = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            self.parse_frontmatter(frontmatter, &mut spec);
        }

        // Extract title from first H1 if not in frontmatter
        if spec.title.is_empty() {
            for line in content.lines() {
                if line.starts_with("# ") {
                    spec.title = line.trim_start_matches("# ").to_string();
                    break;
                }
            }
        }

        // Extract issue references
        for caps in self.issue_ref_regex.captures_iter(content) {
            let issue_num = caps
                .get(1)
                .or_else(|| caps.get(2))
                .or_else(|| caps.get(3))
                .map(|m| m.as_str());
            if let Some(num) = issue_num {
                let ref_str = format!("#{}", num);
                if !spec.issue_refs.contains(&ref_str) {
                    spec.issue_refs.push(ref_str);
                }
            }
        }

        // Parse line by line for structured content
        let lines: Vec<&str> = content.lines().collect();
        let mut current_section = String::new();
        let mut in_code_block = false;
        let mut code_lang = String::new();
        let mut code_content = String::new();
        let mut code_start_line = 0;

        for (i, line) in lines.iter().enumerate() {
            let line_num = i + 1;

            // Track code blocks
            if line.starts_with("```") {
                if in_code_block {
                    // End of code block
                    let is_executable = matches!(
                        code_lang.as_str(),
                        "bash" | "sh" | "shell" | "rust" | "python" | "typescript" | "javascript"
                    );
                    spec.code_examples.push(CodeExample {
                        language: code_lang.clone(),
                        code: code_content.trim().to_string(),
                        line: code_start_line,
                        executable: is_executable,
                    });

                    // Code examples are falsifiable claims - they either compile/run or they don't
                    // Mark as manual validation (can be run by user, gives credit for having testable claims)
                    if is_executable && !code_content.trim().is_empty() {
                        let claim_text = format!(
                            "Code example ({}) at line {} compiles/runs correctly",
                            code_lang, code_start_line
                        );
                        spec.claims.push(ValidationClaim {
                            id: format!("CODE-{}", spec.code_examples.len()),
                            text: claim_text,
                            line: code_start_line,
                            category: ClaimCategory::Falsifiability,
                            automatable: false, // Manual validation - but still falsifiable!
                            validation_cmd: None,
                            expected_pattern: None,
                        });
                    }

                    in_code_block = false;
                    code_content.clear();
                } else {
                    // Start of code block
                    in_code_block = true;
                    code_lang = line.trim_start_matches("```").to_string();
                    code_start_line = line_num;
                }
                continue;
            }

            if in_code_block {
                code_content.push_str(line);
                code_content.push('\n');
                continue;
            }

            // Track sections
            if line.starts_with("## ") || line.starts_with("### ") {
                current_section = line.trim_start_matches('#').trim().to_string();
            }

            // Extract checkbox items (acceptance criteria)
            if let Some(caps) = self.checkbox_regex.captures(line) {
                let checked = caps.get(1).map(|m| m.as_str()) == Some("x")
                    || caps.get(1).map(|m| m.as_str()) == Some("X");
                let text = caps
                    .get(2)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();

                spec.acceptance_criteria.push(AcceptanceCriterion {
                    text: text.clone(),
                    complete: checked,
                    line: line_num,
                });

                // Also create a claim from acceptance criteria
                let category = ClaimCategory::from_section(&current_section)
                    .unwrap_or(ClaimCategory::Implementation);

                spec.claims.push(ValidationClaim {
                    id: format!("AC-{}", spec.acceptance_criteria.len()),
                    text,
                    line: line_num,
                    category,
                    automatable: false,
                    validation_cmd: None,
                    expected_pattern: None,
                });
            }

            // Extract Falsification Conditions (explicit falsifiability claims)
            // Format: "- If X, Y is falsified" or "- X is falsified when Y"
            if line.starts_with("- ") && line.to_lowercase().contains("falsified") {
                let claim_text = line.trim_start_matches("- ").trim().to_string();
                spec.claims.push(ValidationClaim {
                    id: format!("FC-{}", spec.claims.len() + 1),
                    text: claim_text,
                    line: line_num,
                    category: ClaimCategory::Falsifiability,
                    automatable: false, // Manual verification required
                    validation_cmd: None,
                    expected_pattern: None,
                });
            }

            // Extract Documentation requirements from bullet points in doc sections
            // Format: "- **Key**: Description" in Documentation sections
            if (current_section.to_lowercase().contains("documentation")
                || current_section.to_lowercase().contains("open science"))
                && line.starts_with("- ")
                && !line.contains("[ ]")
            {
                let claim_text = line.trim_start_matches("- ").trim().to_string();
                if !claim_text.is_empty() {
                    spec.claims.push(ValidationClaim {
                        id: format!("DOC-{}", spec.claims.len() + 1),
                        text: claim_text,
                        line: line_num,
                        category: ClaimCategory::Documentation,
                        automatable: false,
                        validation_cmd: None,
                        expected_pattern: None,
                    });
                }
            }

            // Extract MUST/SHALL/SHOULD claims
            if let Some(caps) = self.claim_regex.captures(line) {
                let verb = caps
                    .get(1)
                    .map(|m| m.as_str().to_uppercase())
                    .unwrap_or_default();
                let claim_text = caps
                    .get(2)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();

                // Content-based category detection (more accurate than section-based)
                let category = Self::categorize_claim(&claim_text, &current_section);

                // Determine if automatable - expanded detection
                let lower_claim = claim_text.to_lowercase();
                let automatable = lower_claim.contains("pmat ")
                    || lower_claim.contains("cargo ")
                    || lower_claim.contains("test")
                    || lower_claim.contains("coverage")
                    || lower_claim.contains("compile")
                    || lower_claim.contains("build")
                    || lower_claim.contains("pass")
                    || lower_claim.contains("fail")
                    || lower_claim.contains("%")
                    || lower_claim.contains("< ")
                    || lower_claim.contains("> ")
                    || lower_claim.contains("≥")
                    || lower_claim.contains("≤");

                let validation_cmd = if automatable {
                    self.extract_validation_command(&claim_text)
                } else {
                    None
                };

                spec.claims.push(ValidationClaim {
                    id: format!("{}-{}", &verb[..1], spec.claims.len() + 1),
                    text: format!("{} {}", verb, claim_text),
                    line: line_num,
                    category,
                    automatable,
                    validation_cmd,
                    expected_pattern: None,
                });
            }

            // Extract test requirements
            let lower = line.to_lowercase();
            if lower.contains("test")
                && (lower.contains("must") || lower.contains("should") || lower.contains("require"))
            {
                let test_type = if lower.contains("unit") {
                    "unit"
                } else if lower.contains("integration") {
                    "integration"
                } else if lower.contains("property") || lower.contains("proptest") {
                    "property"
                } else if lower.contains("e2e") || lower.contains("end-to-end") {
                    "e2e"
                } else {
                    "general"
                };

                spec.test_requirements.push(TestRequirement {
                    text: line.trim().to_string(),
                    test_type: test_type.to_string(),
                    code_path: self.extract_code_path(line),
                });
            }
        }

        Ok(spec)
    }

    /// Parse YAML frontmatter
    fn parse_frontmatter(&self, frontmatter: &str, spec: &mut ParsedSpec) {
        // Simple key: value parsing (not full YAML)
        for line in frontmatter.lines() {
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim().to_lowercase();
                let value = value
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();

                match key.as_str() {
                    "title" => spec.title = value,
                    "status" => spec.status = Some(value),
                    "issue" | "issues" | "related" => {
                        // Parse issue references from frontmatter
                        for part in value.split([',', ' ']) {
                            let part = part.trim();
                            if !part.is_empty() && !spec.issue_refs.contains(&part.to_string()) {
                                spec.issue_refs.push(part.to_string());
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    /// Extract a validation command from claim text
    fn extract_validation_command(&self, text: &str) -> Option<String> {
        // Look for `command` patterns
        let cmd_regex = Regex::new(r"`([^`]+)`").ok()?;
        for caps in cmd_regex.captures_iter(text) {
            let cmd = caps.get(1)?.as_str();
            if cmd.starts_with("pmat ") || cmd.starts_with("cargo ") {
                return Some(cmd.to_string());
            }
        }

        // Look for common patterns (case-insensitive)
        let lower = text.to_lowercase();
        if lower.contains("coverage") && text.contains("95%") {
            return Some("pmat analyze coverage --format json".to_string());
        }
        if lower.contains("complexity") {
            return Some("pmat analyze complexity --format json".to_string());
        }
        if lower.contains("test") && lower.contains("pass") {
            return Some("cargo test".to_string());
        }

        None
    }

    /// Extract code path from text
    fn extract_code_path(&self, text: &str) -> Option<String> {
        let path_regex = Regex::new(r"(?:`([^`]+\.[a-z]+)`|(\S+\.[a-z]+))").ok()?;
        for caps in path_regex.captures_iter(text) {
            let path = caps.get(1).or_else(|| caps.get(2))?.as_str();
            if path.ends_with(".rs") || path.ends_with(".py") || path.ends_with(".ts") {
                return Some(path.to_string());
            }
        }
        None
    }

    /// Content-based claim categorization (more accurate than section-based)
    fn categorize_claim(claim_text: &str, section: &str) -> ClaimCategory {
        let lower = claim_text.to_lowercase();
        let section_lower = section.to_lowercase();

        // Falsifiability: claims with concrete metrics, thresholds, or testable assertions
        if lower.contains('%')
            || lower.contains("≥")
            || lower.contains("≤")
            || lower.contains("< ")
            || lower.contains("> ")
            || lower.contains("within")
            || lower.contains("at least")
            || lower.contains("at most")
            || lower.contains("exactly")
            || lower.contains("zero ")
            || lower.contains("no ")
            || lower.contains("all ")
            || lower.contains("none ")
            || lower.contains("compile")
            || lower.contains("pass")
            || lower.contains("fail")
            || section_lower.contains("falsif")
            || section_lower.contains("testab")
            || section_lower.contains("acceptance")
        {
            return ClaimCategory::Falsifiability;
        }

        // Testing: test-related claims
        if lower.contains("test")
            || lower.contains("coverage")
            || lower.contains("mutation")
            || lower.contains("property")
            || section_lower.contains("test")
        {
            return ClaimCategory::Testing;
        }

        // Documentation: doc-related claims
        if lower.contains("document")
            || lower.contains("readme")
            || lower.contains("example")
            || lower.contains("changelog")
            || section_lower.contains("doc")
        {
            return ClaimCategory::Documentation;
        }

        // Integration: external system claims
        if lower.contains("api")
            || lower.contains("integrat")
            || lower.contains("github")
            || lower.contains("ci/cd")
            || lower.contains("deploy")
            || section_lower.contains("integrat")
        {
            return ClaimCategory::Integration;
        }

        // Default to Implementation
        ClaimCategory::from_section(section).unwrap_or(ClaimCategory::Implementation)
    }

    /// Find all specifications in a directory
    pub fn find_specs(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        let mut specs = Vec::new();

        if dir.is_file() {
            if dir.extension().map(|e| e == "md").unwrap_or(false) {
                specs.push(dir.to_path_buf());
            }
        } else if dir.is_dir() {
            let pattern = dir.join("**/*.md");
            for path in glob::glob(pattern.to_str().unwrap_or(""))?.flatten() {
                specs.push(path);
            }
        }

        Ok(specs)
    }
}

/// Validation result for a claim
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimValidation {
    pub claim_id: String,
    pub status: ValidationStatus,
    pub evidence: Option<String>,
    pub score: f64,
}

/// Validation status (Popperian)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValidationStatus {
    /// Criterion proven true through evidence
    Proven,
    /// Could not be validated (remains false per Popper)
    Unfalsified,
    /// Validation explicitly failed
    Falsified,
    /// Cannot be automatically validated
    ManualRequired,
    /// Skipped (not applicable)
    Skipped,
}

impl ValidationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proven => "PROVEN",
            Self::Unfalsified => "UNFALSIFIED",
            Self::Falsified => "FALSIFIED",
            Self::ManualRequired => "MANUAL",
            Self::Skipped => "SKIPPED",
        }
    }
}

/// Summary of spec validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    pub spec_path: PathBuf,
    pub total_claims: usize,
    pub proven: usize,
    pub falsified: usize,
    pub unfalsified: usize,
    pub manual_required: usize,
    pub category_scores: HashMap<String, f64>,
    pub total_score: f64,
    pub passed: bool,
    pub gateway_passed: bool,
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_spec() {
        let parser = SpecParser::new();
        let content = r#"---
title: Test Specification
status: Draft
issue: #123
---

# Test Specification

## Requirements

- [ ] First requirement
- [x] Second requirement (done)

## Implementation

The system MUST provide functionality.
The system SHOULD handle errors gracefully.

```rust
fn example() {
    println!("Hello");
}
```
"#;

        let spec = parser
            .parse_content(content, Path::new("test.md"))
            .expect("internal error");

        assert_eq!(spec.title, "Test Specification");
        assert_eq!(spec.status, Some("Draft".to_string()));
        assert!(spec.issue_refs.contains(&"#123".to_string()));
        assert_eq!(spec.acceptance_criteria.len(), 2);
        assert!(!spec.acceptance_criteria[0].complete);
        assert!(spec.acceptance_criteria[1].complete);
        assert!(!spec.code_examples.is_empty());
        assert_eq!(spec.code_examples[0].language, "rust");
    }

    #[test]
    fn test_claim_extraction() {
        let parser = SpecParser::new();
        let content = r#"
# Spec

The implementation MUST pass all tests.
Coverage SHOULD be at least 95%.
The API SHALL be backwards compatible.
"#;

        let spec = parser
            .parse_content(content, Path::new("test.md"))
            .expect("internal error");

        assert!(spec.claims.len() >= 3);
        assert!(spec.claims.iter().any(|c| c.text.contains("MUST")));
        assert!(spec.claims.iter().any(|c| c.text.contains("SHOULD")));
        assert!(spec.claims.iter().any(|c| c.text.contains("SHALL")));
    }

    #[test]
    fn test_category_detection() {
        assert_eq!(
            ClaimCategory::from_section("Testing Strategy"),
            Some(ClaimCategory::Testing)
        );
        assert_eq!(
            ClaimCategory::from_section("Implementation Plan"),
            Some(ClaimCategory::Implementation)
        );
        assert_eq!(
            ClaimCategory::from_section("Documentation"),
            Some(ClaimCategory::Documentation)
        );
        assert_eq!(
            ClaimCategory::from_section("CI/CD Integration"),
            Some(ClaimCategory::Integration)
        );
    }

    #[test]
    fn test_validation_command_extraction() {
        let parser = SpecParser::new();

        let cmd = parser.extract_validation_command("Run `pmat analyze complexity` to check");
        assert_eq!(cmd, Some("pmat analyze complexity".to_string()));

        let cmd = parser.extract_validation_command("Coverage must be at least 95%");
        assert!(cmd.is_some());
    }

    #[test]
    fn test_claim_category_max_points() {
        assert_eq!(ClaimCategory::Falsifiability.max_points(), 25);
        assert_eq!(ClaimCategory::Implementation.max_points(), 25);
        assert_eq!(ClaimCategory::Testing.max_points(), 20);
        assert_eq!(ClaimCategory::Documentation.max_points(), 15);
        assert_eq!(ClaimCategory::Integration.max_points(), 15);
    }

    #[test]
    fn test_claim_category_from_section_falsifiability() {
        assert_eq!(
            ClaimCategory::from_section("Falsifiable Claims"),
            Some(ClaimCategory::Falsifiability)
        );
        assert_eq!(
            ClaimCategory::from_section("Testability"),
            Some(ClaimCategory::Falsifiability)
        );
        assert_eq!(
            ClaimCategory::from_section("Claims to Validate"),
            Some(ClaimCategory::Falsifiability)
        );
    }

    #[test]
    fn test_claim_category_from_section_implementation() {
        assert_eq!(
            ClaimCategory::from_section("Code Changes"),
            Some(ClaimCategory::Implementation)
        );
        assert_eq!(
            ClaimCategory::from_section("Architecture Design"),
            Some(ClaimCategory::Implementation)
        );
    }

    #[test]
    fn test_claim_category_from_section_testing() {
        assert_eq!(
            ClaimCategory::from_section("Coverage Requirements"),
            Some(ClaimCategory::Testing)
        );
        assert_eq!(
            ClaimCategory::from_section("Mutation Testing"),
            Some(ClaimCategory::Testing)
        );
    }

    #[test]
    fn test_claim_category_from_section_documentation() {
        assert_eq!(
            ClaimCategory::from_section("README Updates"),
            Some(ClaimCategory::Documentation)
        );
        assert_eq!(
            ClaimCategory::from_section("Changelog"),
            Some(ClaimCategory::Documentation)
        );
    }

    #[test]
    fn test_claim_category_from_section_integration() {
        assert_eq!(
            ClaimCategory::from_section("CI Pipeline"),
            Some(ClaimCategory::Integration)
        );
        assert_eq!(
            ClaimCategory::from_section("Deployment"),
            Some(ClaimCategory::Integration)
        );
    }

    #[test]
    fn test_claim_category_from_section_unknown() {
        assert_eq!(ClaimCategory::from_section("Random Section"), None);
        assert_eq!(ClaimCategory::from_section("Something Else"), None);
    }

    #[test]
    fn test_spec_parser_default() {
        let parser = SpecParser::default();
        // Should be able to parse content
        let content = "# Simple Spec\n\nSome content.";
        let result = parser.parse_content(content, Path::new("test.md"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_parsed_spec_clone() {
        let spec = ParsedSpec {
            path: PathBuf::from("test.md"),
            title: "Test".to_string(),
            issue_refs: vec!["#1".to_string()],
            status: Some("Draft".to_string()),
            claims: vec![],
            code_examples: vec![],
            acceptance_criteria: vec![],
            test_requirements: vec![],
        };
        let cloned = spec.clone();
        assert_eq!(spec.title, cloned.title);
        assert_eq!(spec.issue_refs, cloned.issue_refs);
    }

    #[test]
    fn test_parsed_spec_serialization() {
        let spec = ParsedSpec {
            path: PathBuf::from("test.md"),
            title: "Test Spec".to_string(),
            issue_refs: vec!["#123".to_string()],
            status: Some("Complete".to_string()),
            claims: vec![],
            code_examples: vec![],
            acceptance_criteria: vec![],
            test_requirements: vec![],
        };
        let json = serde_json::to_string(&spec).unwrap();
        let deserialized: ParsedSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.title, "Test Spec");
        assert_eq!(deserialized.status, Some("Complete".to_string()));
    }

    #[test]
    fn test_validation_claim_fields() {
        let claim = ValidationClaim {
            id: "A1".to_string(),
            text: "MUST pass tests".to_string(),
            line: 10,
            category: ClaimCategory::Testing,
            automatable: true,
            validation_cmd: Some("cargo test".to_string()),
            expected_pattern: Some("ok".to_string()),
        };
        assert_eq!(claim.id, "A1");
        assert_eq!(claim.category, ClaimCategory::Testing);
        assert!(claim.automatable);
    }

    #[test]
    fn test_code_example_fields() {
        let example = CodeExample {
            language: "rust".to_string(),
            code: "fn main() {}".to_string(),
            line: 42,
            executable: true,
        };
        assert_eq!(example.language, "rust");
        assert!(example.executable);
    }

    #[test]
    fn test_acceptance_criterion_fields() {
        let criterion = AcceptanceCriterion {
            text: "Feature works correctly".to_string(),
            complete: true,
            line: 15,
        };
        assert!(criterion.complete);
        assert_eq!(criterion.line, 15);
    }

    #[test]
    fn test_test_requirement_fields() {
        let req = TestRequirement {
            text: "Integration tests required".to_string(),
            test_type: "integration".to_string(),
            code_path: Some("src/lib.rs".to_string()),
        };
        assert_eq!(req.test_type, "integration");
        assert!(req.code_path.is_some());
    }

    #[test]
    fn test_parse_spec_no_frontmatter() {
        let parser = SpecParser::new();
        let content = "# Simple Specification\n\nSome content here.";
        let result = parser.parse_content(content, Path::new("simple.md"));
        assert!(result.is_ok());
        let spec = result.unwrap();
        assert_eq!(spec.title, "Simple Specification");
    }

    #[test]
    fn test_claim_category_equality() {
        assert_eq!(ClaimCategory::Testing, ClaimCategory::Testing);
        assert_ne!(ClaimCategory::Testing, ClaimCategory::Documentation);
    }

    #[test]
    fn test_claim_category_serialization() {
        let cat = ClaimCategory::Implementation;
        let json = serde_json::to_string(&cat).unwrap();
        let deserialized: ClaimCategory = serde_json::from_str(&json).unwrap();
        assert_eq!(cat, deserialized);
    }
}
