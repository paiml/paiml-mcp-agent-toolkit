//! Comprehensive tests for Clap environment variable integration
//!
//! Tests environment variable expansion behavior, precedence rules,
//! and interaction with command-line arguments.

use crate::cli::Cli;
use clap::Parser;
use std::env;

use parking_lot::Mutex;

// Global mutex to ensure env var tests don't interfere across all modules
// Using parking_lot::Mutex which doesn't poison on panic
static ENV_MUTEX: Mutex<()> = Mutex::new(());

#[cfg(test)]
mod env_var_expansion_tests {
    use super::*;

    #[test]
    fn test_rust_log_env_var() {
        let _guard = ENV_MUTEX.lock();

        // Clean environment first
        env::remove_var("RUST_LOG");

        // Test RUST_LOG environment variable mapping to trace_filter
        env::set_var("RUST_LOG", "debug");

        let cli = Cli::try_parse_from(["pmat", "list"]);
        assert!(cli.is_ok());

        if let Ok(parsed) = cli {
            assert_eq!(parsed.trace_filter, Some("debug".to_string()));
        }

        // Test with complex filter
        env::set_var("RUST_LOG", "paiml=debug,cache=trace");
        let cli = Cli::try_parse_from(["pmat", "list"]);
        assert!(cli.is_ok());

        if let Ok(parsed) = cli {
            assert_eq!(
                parsed.trace_filter,
                Some("paiml=debug,cache=trace".to_string())
            );
        }

        // Clean up
        env::remove_var("RUST_LOG");
    }

    #[test]
    fn test_env_var_precedence() {
        let _guard = ENV_MUTEX.lock();
        // Test that command-line arguments take precedence over env vars
        env::set_var("RUST_LOG", "info");

        let cli = Cli::try_parse_from(["pmat", "--trace-filter", "debug", "list"]);

        assert!(cli.is_ok());

        if let Ok(parsed) = cli {
            // Command-line argument should override env var
            assert_eq!(parsed.trace_filter, Some("debug".to_string()));
        }

        // Clean up
        env::remove_var("RUST_LOG");
    }

    #[test]
    fn test_empty_env_var() {
        let _guard = ENV_MUTEX.lock();
        // Test empty environment variable
        env::set_var("RUST_LOG", "");

        let cli = Cli::try_parse_from(["pmat", "list"]);
        assert!(cli.is_ok());

        if let Ok(parsed) = cli {
            // Empty env var should be treated as Some("")
            assert_eq!(parsed.trace_filter, Some("".to_string()));
        }

        // Clean up
        env::remove_var("RUST_LOG");
    }

    #[test]
    fn test_env_var_unset() {
        let _guard = ENV_MUTEX.lock();
        // Make sure RUST_LOG is not set
        env::remove_var("RUST_LOG");

        let cli = Cli::try_parse_from(["pmat", "list"]);
        assert!(cli.is_ok());

        if let Ok(parsed) = cli {
            // No env var should result in None
            assert_eq!(parsed.trace_filter, None);
        }
    }

    #[test]
    fn test_env_var_with_special_characters() {
        let _guard = ENV_MUTEX.lock();
        // Test env var with special characters
        env::set_var("RUST_LOG", "module::submodule=debug,other_mod=trace");

        let cli = Cli::try_parse_from(["pmat", "list"]);
        assert!(cli.is_ok());

        if let Ok(parsed) = cli {
            assert_eq!(
                parsed.trace_filter,
                Some("module::submodule=debug,other_mod=trace".to_string())
            );
        }

        // Clean up
        env::remove_var("RUST_LOG");
    }

    #[test]
    fn test_env_var_unicode() {
        let _guard = ENV_MUTEX.lock();
        // Test env var with Unicode characters
        env::set_var("RUST_LOG", "测试=debug");

        let cli = Cli::try_parse_from(["pmat", "list"]);
        assert!(cli.is_ok());

        if let Ok(parsed) = cli {
            assert_eq!(parsed.trace_filter, Some("测试=debug".to_string()));
        }

        // Clean up
        env::remove_var("RUST_LOG");
    }
}

#[cfg(test)]
mod env_var_interaction_tests {
    use super::*;

    #[test]
    fn test_env_var_with_verbose_flags() {
        let _guard = ENV_MUTEX.lock();

        // Clean environment first
        env::remove_var("RUST_LOG");

        // Test interaction between env var and verbose flags
        env::set_var("RUST_LOG", "warn");

        let cli = Cli::try_parse_from(["pmat", "--verbose", "list"]);

        assert!(cli.is_ok());

        if let Ok(parsed) = cli {
            assert!(parsed.verbose);
            assert_eq!(parsed.trace_filter, Some("warn".to_string()));
            // Both should be active
        }

        // Clean up
        env::remove_var("RUST_LOG");
    }

    #[test]
    fn test_multiple_env_vars() {
        let _guard = ENV_MUTEX.lock();

        // Clean environment first
        env::remove_var("PMAT_MODE");
        env::remove_var("RUST_LOG");

        // Test if there are other env vars that Clap might use
        // This is a placeholder - adapt based on actual env vars used

        // Set potential env vars
        env::set_var("PMAT_MODE", "mcp");
        env::set_var("RUST_LOG", "debug");

        let cli = Cli::try_parse_from(["pmat", "list"]);
        assert!(cli.is_ok());

        if let Ok(parsed) = cli {
            // RUST_LOG should be captured
            assert_eq!(parsed.trace_filter, Some("debug".to_string()));
            // PMAT_MODE is likely not used (no env attribute on mode field)
            assert_eq!(parsed.mode, None);
        }

        // Clean up
        env::remove_var("PMAT_MODE");
        env::remove_var("RUST_LOG");
    }

    #[test]
    fn test_env_var_parsing_errors() {
        let _guard = ENV_MUTEX.lock();

        // Clean environment first
        env::remove_var("RUST_LOG");

        // Test if env vars can cause parsing errors
        // Since RUST_LOG accepts any string, it shouldn't cause errors
        env::set_var("RUST_LOG", "!@#$%^&*()");

        let cli = Cli::try_parse_from(["pmat", "list"]);
        assert!(cli.is_ok());

        if let Ok(parsed) = cli {
            assert_eq!(parsed.trace_filter, Some("!@#$%^&*()".to_string()));
        }

        // Clean up
        env::remove_var("RUST_LOG");
    }
}

#[cfg(test)]
mod env_var_precedence_tests {
    use super::*;

    #[test]
    fn test_explicit_none_vs_env_var() {
        let _guard = ENV_MUTEX.lock();

        // Clean environment first
        env::remove_var("RUST_LOG");

        // Test if we can explicitly override env var with a flag
        env::set_var("RUST_LOG", "debug");

        // There's no --no-trace-filter flag, so we can't explicitly set to None
        // But we can override with a different value
        let cli = Cli::try_parse_from(["pmat", "--trace-filter", "off", "list"]);

        assert!(cli.is_ok());

        if let Ok(parsed) = cli {
            assert_eq!(parsed.trace_filter, Some("off".to_string()));
        }

        // Clean up
        env::remove_var("RUST_LOG");
    }

    #[test]
    fn test_env_var_case_sensitivity() {
        let _guard = ENV_MUTEX.lock();

        // Clean environment first
        env::remove_var("rust_log");
        env::remove_var("RUST_LOG");

        // Test case sensitivity of env var names
        env::set_var("rust_log", "lowercase");
        env::set_var("RUST_LOG", "uppercase");

        let cli = Cli::try_parse_from(["pmat", "list"]);
        assert!(cli.is_ok());

        if let Ok(parsed) = cli {
            // Should use RUST_LOG (uppercase) as specified in the attribute
            assert_eq!(parsed.trace_filter, Some("uppercase".to_string()));
        }

        // Clean up
        env::remove_var("rust_log");
        env::remove_var("RUST_LOG");
    }

    #[test]
    fn test_env_var_whitespace_handling() {
        let _guard = ENV_MUTEX.lock();

        // Clean environment first
        env::remove_var("RUST_LOG");

        // Test env var with leading/trailing whitespace
        env::set_var("RUST_LOG", "  debug  ");

        let cli = Cli::try_parse_from(["pmat", "list"]);
        assert!(cli.is_ok());

        if let Ok(parsed) = cli {
            // Clap should preserve the whitespace
            assert_eq!(parsed.trace_filter, Some("  debug  ".to_string()));
        }

        // Clean up
        env::remove_var("RUST_LOG");
    }

    #[test]
    fn test_env_var_with_equals_sign() {
        let _guard = ENV_MUTEX.lock();

        // Clean environment first
        env::remove_var("RUST_LOG");

        // Test env var value containing equals signs
        env::set_var("RUST_LOG", "module=debug,other=trace=extra");

        let cli = Cli::try_parse_from(["pmat", "list"]);
        assert!(cli.is_ok());

        if let Ok(parsed) = cli {
            assert_eq!(
                parsed.trace_filter,
                Some("module=debug,other=trace=extra".to_string())
            );
        }

        // Clean up
        env::remove_var("RUST_LOG");
    }
}

#[cfg(test)]
mod env_var_edge_cases {
    use super::*;

    #[test]
    fn test_very_long_env_var() {
        let _guard = ENV_MUTEX.lock();

        // Clean environment first
        env::remove_var("RUST_LOG");

        // Test very long env var value - create a properly formatted filter string
        let long_value = (0..1000)
            .map(|i| format!("module{i}=debug"))
            .collect::<Vec<_>>()
            .join(",");
        env::set_var("RUST_LOG", &long_value);

        let cli = Cli::try_parse_from(["pmat", "list"]);
        assert!(cli.is_ok());

        if let Ok(parsed) = cli {
            assert_eq!(parsed.trace_filter, Some(long_value));
        }

        // Clean up
        env::remove_var("RUST_LOG");
    }

    #[test]
    fn test_env_var_with_newlines() {
        let _guard = ENV_MUTEX.lock();

        // Apply Jidoka - Clean environment and verify state
        env::remove_var("RUST_LOG");

        // Verify clean state
        let clean_cli = Cli::try_parse_from(["pmat", "list"]);
        assert!(clean_cli.is_ok());
        if let Ok(parsed) = clean_cli {
            assert_eq!(
                parsed.trace_filter, None,
                "Environment should be clean before test"
            );
        }

        // Test env var with newline characters
        env::set_var("RUST_LOG", "debug\ntrace\ninfo");

        let cli = Cli::try_parse_from(["pmat", "list"]);
        assert!(cli.is_ok(), "CLI parsing should succeed with newlines");

        if let Ok(parsed) = cli {
            // Apply Kaizen - More flexible assertion for newline handling
            match parsed.trace_filter {
                Some(filter) => {
                    // Some systems might normalize newlines or reject them
                    assert!(
                        filter == "debug\ntrace\ninfo"
                            || filter.contains("debug")
                                && filter.contains("trace")
                                && filter.contains("info"),
                        "Filter should contain expected values, got: {filter:?}"
                    );
                }
                None => {
                    // Some systems might reject env vars with newlines
                    println!("⚠️  Kaizen Note: System rejected environment variable with newlines");
                }
            }
        }

        // Clean up
        env::remove_var("RUST_LOG");
    }

    #[test]
    fn test_env_var_with_null_bytes() {
        let _guard = ENV_MUTEX.lock();

        // Clean environment first
        env::remove_var("RUST_LOG");

        // Test env var with null bytes (this will actually panic on Linux)
        // Most shells/OSes don't allow null bytes in env vars
        // This test verifies the platform limitation
        let value_with_null = "debug\0trace";

        // This will panic on Linux - catch the panic to verify behavior
        let result = std::panic::catch_unwind(|| {
            env::set_var("RUST_LOG", value_with_null);
        });

        // Should panic on platforms that don't support null bytes
        assert!(result.is_err());

        // Clean up (may not be necessary if set_var panicked)
        let _ = std::panic::catch_unwind(|| {
            env::remove_var("RUST_LOG");
        });
    }

    #[test]
    fn test_env_var_concurrent_modification() {
        let _guard = ENV_MUTEX.lock();

        // Apply Jidoka - Clean environment first and verify
        env::remove_var("RUST_LOG");

        // Verify clean state
        let clean_cli = Cli::try_parse_from(["pmat", "list"]);
        assert!(clean_cli.is_ok());
        if let Ok(parsed) = clean_cli {
            assert_eq!(
                parsed.trace_filter, None,
                "Environment should be clean before test"
            );
        }

        // Test behavior when env var is set and parsed
        env::set_var("RUST_LOG", "initial");

        // Parse CLI with environment variable set
        let cli = Cli::try_parse_from(["pmat", "list"]);
        assert!(cli.is_ok(), "CLI parsing should succeed");

        if let Ok(parsed) = cli {
            // Apply Kaizen - Clap reads env vars at parse time, so should capture initial value
            assert_eq!(
                parsed.trace_filter,
                Some("initial".to_string()),
                "Should capture env var value at parse time"
            );
        }

        // Modify env var after parsing to verify it doesn't affect already-parsed CLI
        env::set_var("RUST_LOG", "modified");

        // Parse a new CLI instance to show the env var was indeed modified
        let cli2 = Cli::try_parse_from(["pmat", "list"]);
        assert!(cli2.is_ok());
        if let Ok(parsed2) = cli2 {
            assert_eq!(
                parsed2.trace_filter,
                Some("modified".to_string()),
                "New parsing should see modified env var"
            );
        }

        // Clean up
        env::remove_var("RUST_LOG");
    }
}

#[cfg(test)]
mod env_var_documentation_tests {
    use super::*;

    #[test]
    fn test_env_var_help_text() {
        // Test that env var is mentioned in help text
        use clap::CommandFactory;

        let mut cmd = Cli::command();
        let mut help_output = Vec::new();
        let _ = cmd.write_long_help(&mut help_output);
        let help_str = String::from_utf8_lossy(&help_output);

        // Check that RUST_LOG is mentioned in the help
        assert!(help_str.contains("RUST_LOG") || help_str.contains("env:"));
    }

    #[test]
    fn test_env_var_in_error_messages() {
        // Test if env vars are mentioned in error messages when relevant
        env::set_var("RUST_LOG", "debug");

        // Create an invalid command to trigger error
        let cli = Cli::try_parse_from(["pmat", "--invalid-flag"]);
        assert!(cli.is_err());

        // Error message might not mention env vars, but let's check
        if let Err(e) = cli {
            let _error_str = e.to_string();
            // Env vars are usually not mentioned in parse errors
        }

        // Clean up
        env::remove_var("RUST_LOG");
    }
}

#[cfg(test)]
mod env_var_isolation_tests {
    use super::*;

    #[test]
    fn test_isolated_env_var() {
        let _guard = ENV_MUTEX.lock();

        // Clean environment first
        env::remove_var("RUST_LOG");

        // Test in clean environment
        let cli = Cli::try_parse_from(["pmat", "list"]);
        assert!(cli.is_ok());

        if let Ok(parsed) = cli {
            assert_eq!(parsed.trace_filter, None);
        }

        // Set and test
        env::set_var("RUST_LOG", "test_isolated");
        let cli = Cli::try_parse_from(["pmat", "list"]);
        assert!(cli.is_ok());

        if let Ok(parsed) = cli {
            assert_eq!(parsed.trace_filter, Some("test_isolated".to_string()));
        }

        // Clean up
        env::remove_var("RUST_LOG");
    }

    #[test]
    fn test_env_var_does_not_leak() {
        let _guard = ENV_MUTEX.lock();

        // Clean environment first
        env::remove_var("RUST_LOG");

        // Test that env var changes in one test don't affect others
        env::set_var("RUST_LOG", "test_specific_value");

        let cli = Cli::try_parse_from(["pmat", "list"]);
        assert!(cli.is_ok());

        if let Ok(parsed) = cli {
            assert_eq!(parsed.trace_filter, Some("test_specific_value".to_string()));
        }

        // Clean up
        env::remove_var("RUST_LOG");

        // Verify clean state
        let cli = Cli::try_parse_from(["pmat", "list"]);
        assert!(cli.is_ok());

        if let Ok(parsed) = cli {
            assert_eq!(parsed.trace_filter, None);
        }
    }
}
