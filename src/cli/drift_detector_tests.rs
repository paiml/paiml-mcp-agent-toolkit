// DriftDetector tests — included from drift_detector.rs

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::registry::{CommandMetadata, DeprecationInfo};

    fn sample_registry() -> CommandRegistry {
        let mut registry = CommandRegistry::new("2.0.0");

        let complexity_sub = CommandMetadata::builder("complexity")
            .short_description("Analyze complexity")
            .build();

        registry.register(
            CommandMetadata::builder("analyze")
                .short_description("Analyze code")
                .subcommand(complexity_sub)
                .category("analysis")
                .build(),
        );

        registry.register(
            CommandMetadata::builder("context")
                .short_description("Generate context")
                .aliases(["ctx"])
                .category("generation")
                .build(),
        );

        registry.register(
            CommandMetadata::builder("old-command")
                .short_description("Old deprecated command")
                .deprecated(DeprecationInfo {
                    since_version: "2.0.0".to_string(),
                    removal_version: Some("3.0.0".to_string()),
                    replacement: Some("new-command".to_string()),
                    reason: "Replaced".to_string(),
                })
                .category("internal")
                .build(),
        );

        registry
    }

    #[test]
    fn test_detector_creation() {
        let registry = sample_registry();
        let _detector = DriftDetector::new(registry);
    }

    #[test]
    fn test_detect_nonexistent_command() {
        let registry = sample_registry();
        let detector = DriftDetector::new(registry);

        let content = "Run `pmat mcp` to start the server";
        let errors = detector.detect_in_content(content, "README.md");

        assert_eq!(errors.len(), 1);
        assert!(
            matches!(&errors[0], DriftError::NonExistentCommand { mentioned, .. } if mentioned == "mcp")
        );
    }

    #[test]
    fn test_detect_valid_command() {
        let registry = sample_registry();
        let detector = DriftDetector::new(registry);

        let content = "Run `pmat analyze complexity` for metrics";
        let errors = detector.detect_in_content(content, "README.md");

        assert!(errors.is_empty());
    }

    #[test]
    fn test_detect_command_with_alias() {
        let registry = sample_registry();
        let detector = DriftDetector::new(registry);

        let content = "Use `pmat ctx` for quick context";
        let errors = detector.detect_in_content(content, "README.md");

        // ctx is an alias for context, should be valid
        assert!(errors.is_empty());
    }

    #[test]
    fn test_detect_invalid_example() {
        let registry = sample_registry();
        let detector = DriftDetector::new(registry);

        let content = r#"
```bash
pmat nonexistent --flag
```
"#;
        let errors = detector.detect_in_content(content, "README.md");

        assert!(!errors.is_empty());
    }

    #[test]
    fn test_suggest_similar_command() {
        let registry = sample_registry();
        let detector = DriftDetector::new(registry);

        let content = "Run `pmat analize` to check code"; // typo
        let errors = detector.detect_in_content(content, "README.md");

        assert_eq!(errors.len(), 1);
        if let DriftError::NonExistentCommand { suggestion, .. } = &errors[0] {
            assert_eq!(suggestion.as_deref(), Some("analyze"));
        }
    }

    #[test]
    fn test_generate_report() {
        let registry = sample_registry();
        let detector = DriftDetector::new(registry);

        // Create temp file
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_readme.md");
        std::fs::write(&temp_file, "Use `pmat analyze` and `pmat context`")
            .expect("internal error");

        let report = detector.generate_report(&[temp_file.as_path()]);

        assert!(report.documented_commands.contains("analyze"));
        assert!(report.documented_commands.contains("context"));

        // Cleanup
        std::fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_drift_report_format() {
        let report = DriftReport {
            errors: vec![DriftError::NonExistentCommand {
                mentioned: "mcp".to_string(),
                file: "README.md".to_string(),
                line: 10,
                suggestion: None,
            }],
            documented_commands: HashSet::from(["analyze".to_string()]),
            undocumented_commands: HashSet::from(["internal".to_string()]),
            total_commands: 3,
            coverage: 33.3,
        };

        let formatted = report.to_string_report();
        assert!(formatted.contains("1 errors detected"));
        assert!(formatted.contains("33.3% coverage"));
    }

    #[test]
    fn test_levenshtein_for_suggestions() {
        assert_eq!(levenshtein("analyze", "analyze"), 0);
        assert_eq!(levenshtein("analyze", "analize"), 1);
        assert_eq!(levenshtein("analyze", "analyz"), 1);
    }
}
