#[cfg(test)]
mod tests {
    use crate::cli::{AnalyzeCommands, Cli, Commands};
    use clap::Parser;

    #[test]
    fn test_analyze_complexity_with_include_patterns() {
        // Test that include patterns are parsed correctly
        let args = vec![
            "pmat",
            "analyze",
            "complexity",
            "--include",
            "**/*.rs",
            "--include",
            "src/**/*.rs",
            "--include",
            "tests/*.rs",
        ];

        let cli = Cli::try_parse_from(&args).unwrap();

        match cli.command {
            Commands::Analyze(AnalyzeCommands::Complexity { include, .. }) => {
                assert_eq!(include.len(), 3);
                assert_eq!(include[0], "**/*.rs");
                assert_eq!(include[1], "src/**/*.rs");
                assert_eq!(include[2], "tests/*.rs");
            }
            _ => panic!("Expected Analyze::Complexity command"),
        }
    }

    #[test]
    fn test_glob_pattern_matching() {
        use glob::Pattern;

        // Test various glob patterns
        let test_cases = vec![
            ("**/*.rs", "src/main.rs", true),
            ("**/*.rs", "src/lib/mod.rs", true),
            ("**/*.rs", "src/main.py", false),
            ("src/**/*.rs", "src/main.rs", true),
            ("src/**/*.rs", "src/lib/mod.rs", true),
            ("src/**/*.rs", "tests/test.rs", false),
            ("tests/*.rs", "tests/test.rs", true),
            ("tests/*/*.rs", "tests/unit/test.rs", true), // Need explicit path depth
            ("tests/**/*.rs", "tests/unit/test.rs", true),
        ];

        for (pattern_str, path, expected) in test_cases {
            let pattern = Pattern::new(pattern_str).unwrap();
            assert_eq!(
                pattern.matches(path),
                expected,
                "Pattern '{}' matching '{}' expected {}, got {}",
                pattern_str,
                path,
                expected,
                pattern.matches(path)
            );
        }
    }
}
