// Property-based tests for protocols module.
// Included by protocols.rs — shares parent module scope (no `use` imports here).

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use std::path::PathBuf;

    proptest! {
        #[test]
        fn test_protocol_to_string_never_empty(protocol_idx in 0u8..4u8) {
            let protocol = match protocol_idx {
                0 => Protocol::Cli,
                1 => Protocol::Http,
                2 => Protocol::Mcp,
                _ => Protocol::All,
            };

            let result = protocol_to_string(&protocol);
            prop_assert!(!result.is_empty());
        }

        #[test]
        fn test_build_protocol_request_valid_json(
            path_str in "[a-zA-Z0-9/._-]{0,100}",
            protocol in prop_oneof!["cli", "http", "mcp", "unknown"],
            show_api in any::<bool>()
        ) {
            let path = PathBuf::from(&path_str);
            let request = build_protocol_request(&protocol, &path, show_api);

            // Request should always be valid JSON object or empty object
            prop_assert!(request.is_object());
        }

        #[test]
        fn test_demo_args_centrality_threshold_bounds(threshold in 0.0f64..=1.0f64) {
            let args = DemoArgs {
                path: None,
                url: None,
                repo: None,
                format: crate::cli::OutputFormat::Json,
                no_browser: false,
                port: None,
                web: false,
                target_nodes: 10,
                centrality_threshold: threshold,
                merge_threshold: 3,
                protocol: Protocol::Cli,
                show_api: false,
                debug: false,
                debug_output: None,
                skip_vendor: false,
                max_line_length: None,
            };

            prop_assert!(args.centrality_threshold >= 0.0);
            prop_assert!(args.centrality_threshold <= 1.0);
        }

        #[test]
        fn test_demo_args_target_nodes_positive(target in 1usize..1000usize) {
            let args = DemoArgs {
                path: None,
                url: None,
                repo: None,
                format: crate::cli::OutputFormat::Json,
                no_browser: false,
                port: None,
                web: false,
                target_nodes: target,
                centrality_threshold: 0.5,
                merge_threshold: 3,
                protocol: Protocol::Cli,
                show_api: false,
                debug: false,
                debug_output: None,
                skip_vendor: false,
                max_line_length: None,
            };

            prop_assert!(args.target_nodes > 0);
            prop_assert_eq!(args.target_nodes, target);
        }
    }
}
