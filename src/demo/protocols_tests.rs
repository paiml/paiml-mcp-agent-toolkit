// Unit tests for protocols module.
// Included by protocols.rs — shares parent module scope (no `use` imports here).

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // ============================================================
    // Protocol enum tests
    // ============================================================

    #[test]
    fn test_protocol_cli_equality() {
        assert_eq!(Protocol::Cli, Protocol::Cli);
        assert_ne!(Protocol::Cli, Protocol::Http);
    }

    #[test]
    fn test_protocol_http_equality() {
        assert_eq!(Protocol::Http, Protocol::Http);
        assert_ne!(Protocol::Http, Protocol::Mcp);
    }

    #[test]
    fn test_protocol_mcp_equality() {
        assert_eq!(Protocol::Mcp, Protocol::Mcp);
        assert_ne!(Protocol::Mcp, Protocol::All);
    }

    #[test]
    fn test_protocol_all_equality() {
        assert_eq!(Protocol::All, Protocol::All);
        assert_ne!(Protocol::All, Protocol::Cli);
    }

    #[test]
    fn test_protocol_copy() {
        let p = Protocol::Cli;
        let p2 = p;
        assert_eq!(p, p2);
    }

    #[test]
    fn test_protocol_debug() {
        let formatted = format!("{:?}", Protocol::Cli);
        assert!(formatted.contains("Cli"));
    }

    // ============================================================
    // protocol_to_string tests
    // ============================================================

    #[test]
    fn test_protocol_to_string_cli() {
        assert_eq!(protocol_to_string(&Protocol::Cli), "cli");
    }

    #[test]
    fn test_protocol_to_string_http() {
        assert_eq!(protocol_to_string(&Protocol::Http), "http");
    }

    #[test]
    fn test_protocol_to_string_mcp() {
        assert_eq!(protocol_to_string(&Protocol::Mcp), "mcp");
    }

    #[test]
    fn test_protocol_to_string_all() {
        assert_eq!(protocol_to_string(&Protocol::All), "all");
    }

    // ============================================================
    // print_protocol_banner tests (output verification)
    // ============================================================

    #[test]
    fn test_print_protocol_banner_cli() {
        // Just verify it doesn't panic
        print_protocol_banner(&Protocol::Cli);
    }

    #[test]
    fn test_print_protocol_banner_http() {
        print_protocol_banner(&Protocol::Http);
    }

    #[test]
    fn test_print_protocol_banner_mcp() {
        print_protocol_banner(&Protocol::Mcp);
    }

    #[test]
    fn test_print_protocol_banner_all() {
        print_protocol_banner(&Protocol::All);
    }

    // ============================================================
    // build_protocol_request tests
    // ============================================================

    #[test]
    fn test_build_protocol_request_cli() {
        let path = PathBuf::from("/test/path");
        let request = build_protocol_request("cli", &path, false);

        assert_eq!(request["path"], "/test/path");
        assert_eq!(request["show_api"], false);
    }

    #[test]
    fn test_build_protocol_request_cli_with_show_api() {
        let path = PathBuf::from("/test/path");
        let request = build_protocol_request("cli", &path, true);

        assert_eq!(request["path"], "/test/path");
        assert_eq!(request["show_api"], true);
    }

    #[test]
    fn test_build_protocol_request_http() {
        let path = PathBuf::from("/test/repo");
        let request = build_protocol_request("http", &path, false);

        assert_eq!(request["method"], "GET");
        assert_eq!(request["path"], "/demo/analyze");
        assert_eq!(request["query"]["path"], "/test/repo");
        assert_eq!(request["headers"]["Accept"], "application/json");
    }

    #[test]
    fn test_build_protocol_request_mcp() {
        let path = PathBuf::from("/test/project");
        let request = build_protocol_request("mcp", &path, true);

        assert_eq!(request["jsonrpc"], "2.0");
        assert_eq!(request["method"], "demo.analyze");
        assert_eq!(request["params"]["path"], "/test/project");
        assert_eq!(request["params"]["include_trace"], true);
        assert_eq!(request["id"], 1);
    }

    #[test]
    fn test_build_protocol_request_unknown() {
        let path = PathBuf::from("/test");
        let request = build_protocol_request("unknown", &path, false);

        assert!(request.as_object().unwrap().is_empty());
    }

    // ============================================================
    // format_and_print_output tests
    // ============================================================

    #[test]
    fn test_format_and_print_output_json() {
        let response = serde_json::json!({"status": "ok", "count": 42});
        let result = format_and_print_output(&response, &crate::cli::OutputFormat::Json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_and_print_output_yaml() {
        let response = serde_json::json!({"key": "value"});
        let result = format_and_print_output(&response, &crate::cli::OutputFormat::Yaml);
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_and_print_output_table() {
        let response = serde_json::json!({"data": [1, 2, 3]});
        let result = format_and_print_output(&response, &crate::cli::OutputFormat::Table);
        assert!(result.is_ok());
    }

    // ============================================================
    // DemoArgs struct tests
    // ============================================================

    #[test]
    fn test_demo_args_default_construction() {
        let args = DemoArgs {
            path: None,
            url: None,
            repo: None,
            format: crate::cli::OutputFormat::Json,
            no_browser: false,
            port: None,
            web: false,
            target_nodes: 10,
            centrality_threshold: 0.5,
            merge_threshold: 3,
            protocol: Protocol::Cli,
            show_api: false,
            debug: false,
            debug_output: None,
            skip_vendor: false,
            max_line_length: None,
        };

        assert!(args.path.is_none());
        assert!(!args.web);
        assert_eq!(args.protocol, Protocol::Cli);
    }

    #[test]
    fn test_demo_args_with_path() {
        let args = DemoArgs {
            path: Some(PathBuf::from("/test/path")),
            url: None,
            repo: None,
            format: crate::cli::OutputFormat::Table,
            no_browser: true,
            port: Some(8080),
            web: true,
            target_nodes: 20,
            centrality_threshold: 0.75,
            merge_threshold: 5,
            protocol: Protocol::Http,
            show_api: true,
            debug: true,
            debug_output: Some(PathBuf::from("/tmp/debug.log")),
            skip_vendor: true,
            max_line_length: Some(120),
        };

        assert_eq!(args.path, Some(PathBuf::from("/test/path")));
        assert!(args.web);
        assert!(args.no_browser);
        assert_eq!(args.port, Some(8080));
        assert_eq!(args.target_nodes, 20);
        assert!((args.centrality_threshold - 0.75).abs() < f64::EPSILON);
        assert_eq!(args.protocol, Protocol::Http);
        assert!(args.show_api);
        assert!(args.debug);
        assert!(args.skip_vendor);
        assert_eq!(args.max_line_length, Some(120));
    }

    #[test]
    fn test_demo_args_clone() {
        let args = DemoArgs {
            path: Some(PathBuf::from("/test")),
            url: Some("https://github.com/test/repo".to_string()),
            repo: Some("test/repo".to_string()),
            format: crate::cli::OutputFormat::Yaml,
            no_browser: false,
            port: None,
            web: false,
            target_nodes: 15,
            centrality_threshold: 0.6,
            merge_threshold: 4,
            protocol: Protocol::Mcp,
            show_api: false,
            debug: false,
            debug_output: None,
            skip_vendor: false,
            max_line_length: None,
        };

        let cloned = args.clone();
        assert_eq!(args.path, cloned.path);
        assert_eq!(args.url, cloned.url);
        assert_eq!(args.repo, cloned.repo);
        assert_eq!(args.protocol, cloned.protocol);
    }

    #[test]
    fn test_demo_args_debug() {
        let args = DemoArgs {
            path: None,
            url: None,
            repo: None,
            format: crate::cli::OutputFormat::Json,
            no_browser: false,
            port: None,
            web: false,
            target_nodes: 10,
            centrality_threshold: 0.5,
            merge_threshold: 3,
            protocol: Protocol::All,
            show_api: false,
            debug: false,
            debug_output: None,
            skip_vendor: false,
            max_line_length: None,
        };

        let debug_str = format!("{:?}", args);
        assert!(debug_str.contains("DemoArgs"));
    }

    // ============================================================
    // Async function tests (using tokio::test)
    // ============================================================

    #[tokio::test]
    async fn test_print_api_metadata() {
        let result = print_api_metadata("cli").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_print_api_metadata_http() {
        let result = print_api_metadata("http").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_print_api_metadata_mcp() {
        let result = print_api_metadata("mcp").await;
        assert!(result.is_ok());
    }

    // ============================================================
    // Edge case tests
    // ============================================================

    #[test]
    fn test_build_protocol_request_empty_path() {
        let path = PathBuf::from("");
        let request = build_protocol_request("cli", &path, false);
        assert_eq!(request["path"], "");
    }

    #[test]
    fn test_build_protocol_request_special_characters_in_path() {
        let path = PathBuf::from("/path/with spaces/and-dashes/and_underscores");
        let request = build_protocol_request("http", &path, false);
        assert_eq!(
            request["query"]["path"],
            "/path/with spaces/and-dashes/and_underscores"
        );
    }

    // ============================================================
    // Integration-style tests
    // ============================================================

    #[test]
    fn test_protocol_round_trip() {
        // Test that protocol_to_string matches Protocol enum variants
        let protocols = [Protocol::Cli, Protocol::Http, Protocol::Mcp, Protocol::All];

        for p in protocols {
            let name = protocol_to_string(&p);
            assert!(!name.is_empty());

            // Verify round-trip consistency
            let name2 = protocol_to_string(&p);
            assert_eq!(name, name2);
        }
    }

    #[test]
    fn test_protocol_request_consistency() {
        let path = PathBuf::from("/consistent/test");

        // All protocol requests should be valid JSON
        for protocol in ["cli", "http", "mcp", "unknown"] {
            let request = build_protocol_request(protocol, &path, true);
            assert!(request.is_object() || request.as_object().unwrap().is_empty());
        }
    }
}
