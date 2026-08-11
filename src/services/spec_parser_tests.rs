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
            raw_content: String::new(),
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
            raw_content: String::new(),
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

    #[test]
    fn test_find_specs_errors_on_missing_path() {
        // Regression: a typo'd path used to yield Ok(vec![]), so `spec list`
        // over /does/not/exist printed "0 specs, 0 passing" and exited 0.
        let parser = SpecParser::new();
        let err = parser
            .find_specs(Path::new("/does/not/exist/pmat-spec-dir"))
            .expect_err("nonexistent spec path must be an error, not an empty spec set");
        assert!(
            err.to_string().contains("does not exist"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_find_specs_empty_dir_is_ok() {
        // An existing-but-empty directory is still legitimately zero specs.
        let dir = std::env::temp_dir().join("pmat_spec_parser_empty_dir_test");
        std::fs::create_dir_all(&dir).unwrap();
        let parser = SpecParser::new();
        let specs = parser.find_specs(&dir).expect("empty dir must be Ok");
        assert!(specs.is_empty());
        let _ = std::fs::remove_dir(&dir);
    }
}
