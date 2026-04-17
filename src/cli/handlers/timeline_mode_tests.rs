#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;

    mod timeline_mode_enum_tests {
        use super::*;

        #[test]
        fn test_default_is_non_interactive() {
            let mode: TimelineMode = Default::default();
            assert_eq!(mode, TimelineMode::NonInteractive);
        }

        #[test]
        fn test_debug_impl() {
            let interactive = TimelineMode::Interactive;
            let non_interactive = TimelineMode::NonInteractive;
            assert_eq!(format!("{:?}", interactive), "Interactive");
            assert_eq!(format!("{:?}", non_interactive), "NonInteractive");
        }

        #[test]
        fn test_clone() {
            let original = TimelineMode::Interactive;
            let cloned = original;
            assert_eq!(original, cloned);
        }

        #[test]
        fn test_copy() {
            let original = TimelineMode::NonInteractive;
            let copied = original;
            assert_eq!(original, copied);
        }

        #[test]
        fn test_eq_same_variants() {
            assert_eq!(TimelineMode::Interactive, TimelineMode::Interactive);
            assert_eq!(TimelineMode::NonInteractive, TimelineMode::NonInteractive);
        }

        #[test]
        fn test_eq_different_variants() {
            assert_ne!(TimelineMode::Interactive, TimelineMode::NonInteractive);
        }
    }

    mod from_args_tests {
        use super::*;

        #[test]
        fn test_from_args_interactive_long_flag() {
            let args = vec!["--interactive"];
            let mode = TimelineMode::from_args(&args);
            assert_eq!(mode, TimelineMode::Interactive);
        }

        #[test]
        fn test_from_args_interactive_short_flag() {
            let args = vec!["-i"];
            let mode = TimelineMode::from_args(&args);
            assert_eq!(mode, TimelineMode::Interactive);
        }

        #[test]
        fn test_from_args_non_interactive_empty() {
            let args: Vec<&str> = vec![];
            let mode = TimelineMode::from_args(&args);
            assert_eq!(mode, TimelineMode::NonInteractive);
        }

        #[test]
        fn test_from_args_non_interactive_other_flags() {
            let args = vec!["--json", "--verbose", "--output", "file.txt"];
            let mode = TimelineMode::from_args(&args);
            assert_eq!(mode, TimelineMode::NonInteractive);
        }

        #[test]
        fn test_from_args_interactive_with_other_flags() {
            let args = vec!["recording.pmat", "--interactive", "--verbose"];
            let mode = TimelineMode::from_args(&args);
            assert_eq!(mode, TimelineMode::Interactive);
        }

        #[test]
        fn test_from_args_short_flag_with_other_args() {
            let args = vec!["-i", "recording.pmat"];
            let mode = TimelineMode::from_args(&args);
            assert_eq!(mode, TimelineMode::Interactive);
        }
    }

    mod is_interactive_tests {
        use super::*;

        #[test]
        fn test_is_interactive_true() {
            let mode = TimelineMode::Interactive;
            assert!(mode.is_interactive());
        }

        #[test]
        fn test_is_interactive_false() {
            let mode = TimelineMode::NonInteractive;
            assert!(!mode.is_interactive());
        }
    }

    mod is_non_interactive_tests {
        use super::*;

        #[test]
        fn test_is_non_interactive_true() {
            let mode = TimelineMode::NonInteractive;
            assert!(mode.is_non_interactive());
        }

        #[test]
        fn test_is_non_interactive_false() {
            let mode = TimelineMode::Interactive;
            assert!(!mode.is_non_interactive());
        }
    }

    mod requires_terminal_tests {
        use super::*;

        #[test]
        fn test_requires_terminal_interactive() {
            let mode = TimelineMode::Interactive;
            assert!(mode.requires_terminal());
        }

        #[test]
        fn test_requires_terminal_non_interactive() {
            let mode = TimelineMode::NonInteractive;
            assert!(!mode.requires_terminal());
        }
    }

    mod validate_terminal_availability_tests {
        use super::*;

        #[test]
        fn test_interactive_with_tty_success() {
            let mode = TimelineMode::Interactive;
            let result = mode.validate_terminal_availability(true);
            assert!(result.is_ok());
        }

        #[test]
        fn test_interactive_without_tty_error() {
            let mode = TimelineMode::Interactive;
            let result = mode.validate_terminal_availability(false);
            assert!(result.is_err());
            let err_msg = result.unwrap_err().to_string();
            assert!(err_msg.contains("Interactive mode requires a TTY"));
            assert!(err_msg.contains("terminal"));
        }

        #[test]
        fn test_non_interactive_with_tty_success() {
            let mode = TimelineMode::NonInteractive;
            let result = mode.validate_terminal_availability(true);
            assert!(result.is_ok());
        }

        #[test]
        fn test_non_interactive_without_tty_success() {
            let mode = TimelineMode::NonInteractive;
            let result = mode.validate_terminal_availability(false);
            assert!(result.is_ok());
        }
    }

    mod description_tests {
        use super::*;

        #[test]
        fn test_description_interactive() {
            let mode = TimelineMode::Interactive;
            assert_eq!(mode.description(), "Interactive TUI mode");
        }

        #[test]
        fn test_description_non_interactive() {
            let mode = TimelineMode::NonInteractive;
            assert_eq!(mode.description(), "Non-interactive batch mode");
        }
    }

    mod validate_args_tests {
        use super::*;

        #[test]
        fn test_validate_args_no_conflict_empty() {
            let args: Vec<&str> = vec![];
            let result = TimelineMode::validate_args(&args);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_args_no_conflict_interactive_only() {
            let args = vec!["--interactive"];
            let result = TimelineMode::validate_args(&args);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_args_no_conflict_short_interactive_only() {
            let args = vec!["-i"];
            let result = TimelineMode::validate_args(&args);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_args_no_conflict_json_only() {
            let args = vec!["--json"];
            let result = TimelineMode::validate_args(&args);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_args_conflict_interactive_and_json() {
            let args = vec!["--interactive", "--json"];
            let result = TimelineMode::validate_args(&args);
            assert!(result.is_err());
            let err_msg = result.unwrap_err().to_string();
            assert!(err_msg.contains("Conflicting flags"));
            assert!(err_msg.contains("--interactive"));
            assert!(err_msg.contains("--json"));
        }

        #[test]
        fn test_validate_args_conflict_short_interactive_and_json() {
            let args = vec!["-i", "--json"];
            let result = TimelineMode::validate_args(&args);
            assert!(result.is_err());
        }

        #[test]
        fn test_validate_args_no_conflict_other_flags() {
            let args = vec!["--verbose", "--output", "file.txt", "recording.pmat"];
            let result = TimelineMode::validate_args(&args);
            assert!(result.is_ok());
        }
    }

    mod check_feature_availability_tests {
        use super::*;

        #[test]
        fn test_check_feature_non_interactive_always_ok() {
            let mode = TimelineMode::NonInteractive;
            let result = mode.check_feature_availability();
            assert!(result.is_ok());
        }

        #[test]
        #[cfg(feature = "tui")]
        fn test_check_feature_interactive_with_tui_feature_ok() {
            let mode = TimelineMode::Interactive;
            let result = mode.check_feature_availability();
            assert!(result.is_ok());
        }

        #[test]
        #[cfg(not(feature = "tui"))]
        fn test_check_feature_interactive_without_tui_feature_error() {
            let mode = TimelineMode::Interactive;
            let result = mode.check_feature_availability();
            assert!(result.is_err());
            let err_msg = result.unwrap_err().to_string();
            assert!(err_msg.contains("tui"));
            assert!(err_msg.contains("feature"));
        }
    }

    mod handle_timeline_tests {
        use super::*;

        #[test]
        fn test_handle_timeline_empty_args() {
            let args: Vec<&str> = vec![];
            let result = handle_timeline(&args);
            assert!(result.is_ok());
        }

        #[test]
        fn test_handle_timeline_with_args() {
            let args = vec!["recording.pmat", "--interactive"];
            let result = handle_timeline(&args);
            assert!(result.is_ok());
        }

        #[test]
        fn test_handle_timeline_with_json() {
            let args = vec!["recording.pmat", "--json"];
            let result = handle_timeline(&args);
            assert!(result.is_ok());
        }
    }

    mod get_timeline_help_text_tests {
        use super::*;

        #[test]
        fn test_get_timeline_help_text_contains_usage() {
            let help = get_timeline_help_text();
            assert!(help.contains("USAGE:"));
        }

        #[test]
        fn test_get_timeline_help_text_contains_options() {
            let help = get_timeline_help_text();
            assert!(help.contains("OPTIONS:"));
        }

        #[test]
        fn test_get_timeline_help_text_contains_interactive_flag() {
            let help = get_timeline_help_text();
            assert!(help.contains("--interactive"));
            assert!(help.contains("-i"));
        }

        #[test]
        fn test_get_timeline_help_text_contains_json_flag() {
            let help = get_timeline_help_text();
            assert!(help.contains("--json"));
        }

        #[test]
        fn test_get_timeline_help_text_contains_examples() {
            let help = get_timeline_help_text();
            assert!(help.contains("EXAMPLES:"));
        }

        #[test]
        fn test_get_timeline_help_text_contains_pmat_command() {
            let help = get_timeline_help_text();
            assert!(help.contains("pmat timeline"));
        }

        #[test]
        fn test_get_timeline_help_text_not_empty() {
            let help = get_timeline_help_text();
            assert!(!help.is_empty());
            assert!(help.len() > 100); // Should have substantial content
        }
    }

    mod integration_tests {
        use super::*;

        #[test]
        fn test_full_workflow_interactive() {
            let args = vec!["recording.pmat", "--interactive"];

            // Parse mode
            let mode = TimelineMode::from_args(&args);
            assert_eq!(mode, TimelineMode::Interactive);

            // Check properties
            assert!(mode.is_interactive());
            assert!(!mode.is_non_interactive());
            assert!(mode.requires_terminal());

            // Validate with TTY
            assert!(mode.validate_terminal_availability(true).is_ok());

            // Description
            assert_eq!(mode.description(), "Interactive TUI mode");
        }

        #[test]
        fn test_full_workflow_non_interactive() {
            let args = vec!["recording.pmat", "--json"];

            // Validate no conflicts
            assert!(TimelineMode::validate_args(&args).is_ok());

            // Parse mode
            let mode = TimelineMode::from_args(&args);
            assert_eq!(mode, TimelineMode::NonInteractive);

            // Check properties
            assert!(!mode.is_interactive());
            assert!(mode.is_non_interactive());
            assert!(!mode.requires_terminal());

            // Validate without TTY (should still work for non-interactive)
            assert!(mode.validate_terminal_availability(false).is_ok());

            // Description
            assert_eq!(mode.description(), "Non-interactive batch mode");
        }

        #[test]
        fn test_conflicting_args_detected() {
            let args = vec!["recording.pmat", "--interactive", "--json"];

            // Should detect conflict
            let result = TimelineMode::validate_args(&args);
            assert!(result.is_err());
        }

        #[test]
        fn test_help_text_matches_functionality() {
            let help = get_timeline_help_text();

            // Help mentions both modes
            assert!(help.contains("Interactive"));
            assert!(help.contains("Non-interactive"));

            // Help mentions conflicts
            assert!(help.contains("conflicts"));
        }
    }
}
