#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    // === QualityConfigInput Tests ===

    #[test]
    fn test_default_quality_config() {
        let config = QualityConfigInput::default();
        assert_eq!(config.max_complexity, default_max_complexity());
        assert_eq!(config.allow_satd, default_allow_satd());
        assert_eq!(config.require_docs, default_require_docs());
        assert_eq!(config.auto_format, default_auto_format());
    }

    #[test]
    fn test_default_max_complexity() {
        assert_eq!(default_max_complexity(), 20);
    }

    #[test]
    fn test_default_allow_satd() {
        assert!(!default_allow_satd());
    }

    #[test]
    fn test_default_require_docs() {
        assert!(default_require_docs());
    }

    #[test]
    fn test_default_auto_format() {
        assert!(default_auto_format());
    }

    #[test]
    fn test_default_mode() {
        assert_eq!(default_mode(), "strict");
    }

    #[test]
    fn test_quality_config_deserialization_defaults() {
        let json = serde_json::json!({});
        let config: QualityConfigInput = serde_json::from_value(json).unwrap();
        assert_eq!(config.max_complexity, 20);
        assert!(!config.allow_satd);
        assert!(config.require_docs);
        assert!(config.auto_format);
    }

    #[test]
    fn test_quality_config_deserialization_custom() {
        let json = serde_json::json!({
            "max_complexity": 10,
            "allow_satd": true,
            "require_docs": false,
            "auto_format": false
        });
        let config: QualityConfigInput = serde_json::from_value(json).unwrap();
        assert_eq!(config.max_complexity, 10);
        assert!(config.allow_satd);
        assert!(!config.require_docs);
        assert!(!config.auto_format);
    }

    #[test]
    fn test_quality_config_debug() {
        let config = QualityConfigInput::default();
        let debug = format!("{:?}", config);
        assert!(debug.contains("QualityConfigInput"));
        assert!(debug.contains("max_complexity"));
    }

    // === QualityProxyInput Tests ===

    #[test]
    fn test_quality_proxy_input_deserialization() {
        let json = serde_json::json!({
            "file_path": "src/main.rs",
            "content": "fn main() {}",
            "mode": "strict"
        });
        let input: QualityProxyInput = serde_json::from_value(json).unwrap();
        assert_eq!(input.file_path, "src/main.rs");
        assert_eq!(input.content, "fn main() {}");
        assert_eq!(input.mode, "strict");
    }

    #[test]
    fn test_quality_proxy_input_default_mode() {
        let json = serde_json::json!({
            "file_path": "src/main.rs",
            "content": "pub fn a() {}"
        });
        let input: QualityProxyInput = serde_json::from_value(json).unwrap();
        assert_eq!(input.mode, "strict");
    }

    #[test]
    fn test_quality_proxy_input_edit_operation() {
        let json = serde_json::json!({
            "file_path": "src/lib.rs",
            "content": "pub fn a() {}"
        });
        let input: QualityProxyInput = serde_json::from_value(json).unwrap();
        assert!(!input.file_path.is_empty());
    }

    #[test]
    fn test_quality_proxy_input_advisory_mode() {
        let json = serde_json::json!({
            "file_path": "src/main.rs",
            "content": "pub fn a() {}",
            "mode": "advisory"
        });
        let input: QualityProxyInput = serde_json::from_value(json).unwrap();
        assert_eq!(input.mode, "advisory");
    }

    #[test]
    fn test_quality_proxy_input_auto_fix_mode() {
        let json = serde_json::json!({
            "file_path": "src/main.rs",
            "content": "pub fn a() {}",
            "mode": "auto_fix"
        });
        let input: QualityProxyInput = serde_json::from_value(json).unwrap();
        assert_eq!(input.mode, "auto_fix");
    }

    #[test]
    fn test_quality_proxy_input_with_quality_config() {
        let json = serde_json::json!({
            "file_path": "src/main.rs",
            "content": "pub fn a() {}",
            "quality_config": {
                "max_complexity": 5,
                "allow_satd": true
            }
        });
        let input: QualityProxyInput = serde_json::from_value(json).unwrap();
        assert_eq!(input.quality_config.max_complexity, 5);
        assert!(input.quality_config.allow_satd);
    }

    #[test]
    fn test_quality_proxy_input_debug() {
        let input = QualityProxyInput {
            file_path: "test.rs".to_string(),
            content: "test".to_string(),
            mode: "strict".to_string(),
            quality_config: QualityConfigInput::default(),
        };
        let debug = format!("{:?}", input);
        assert!(debug.contains("QualityProxyInput"));
    }

    #[test]
    fn test_quality_proxy_input_missing_required_fields() {
        let json = serde_json::json!({});
        let result: std::result::Result<QualityProxyInput, _> = serde_json::from_value(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_quality_proxy_input_append_operation() {
        let json = serde_json::json!({
            "file_path": "src/main.rs",
            "content": "// appended content"
        });
        let input: QualityProxyInput = serde_json::from_value(json).unwrap();
        assert_eq!(input.content, "// appended content");
    }

    // === QualityProxyTool Tests ===

    /// CRUX-10 R1: a request still carrying `operation` is refused at the
    /// schema, not silently accepted.
    #[test]
    fn a_stale_operation_field_is_rejected() {
        let json = serde_json::json!({
            "operation": "write",
            "file_path": "x.rs",
            "content": "pub fn a() {}"
        });
        let err = serde_json::from_value::<QualityProxyInput>(json).expect_err("must not parse");
        assert!(err.to_string().contains("operation"), "{err}");
    }

    /// CRUX-10 R2: the one-release alias advertises the same schema and
    /// description under the old name.
    #[test]
    fn the_alias_is_the_live_tool_under_the_old_name() {
        use pmcp::ToolHandler;
        let live = QualityProxyTool.metadata().expect("metadata");
        let alias = QualityProxyAliasTool.metadata().expect("metadata");
        assert_eq!(live.name, QUALITY_CHECK_CONTENT);
        assert_eq!(alias.name, QUALITY_PROXY_ALIAS);
        assert_eq!(live.description, alias.description);
        assert_eq!(live.input_schema, alias.input_schema);
        // The same substring rule the spec's audit script applies (R4):
        // no "write", "edit" or "append" anywhere, so "writes"/"writer" fail too.
        let desc = live.description.expect("description").to_lowercase();
        for verb in ["write", "edit", "append"] {
            assert!(
                !desc.contains(verb),
                "description still asserts a mutation ({verb}): {desc}"
            );
        }
    }

    #[test]
    fn test_quality_proxy_tool_creation() {
        let _tool = QualityProxyTool;
    }

    #[tokio::test]
    async fn test_quality_proxy_handle() {
        // Test disabled - requires pmcp internal types not exposed in API
    }

    // === Integration-like Tests (without pmcp types) ===




    #[test]
    fn test_mode_parsing_strict() {
        let mode_str = "strict";
        let mode = match mode_str {
            "strict" => ProxyMode::Strict,
            "advisory" => ProxyMode::Advisory,
            "auto_fix" | "auto-fix" => ProxyMode::AutoFix,
            _ => panic!("Invalid mode"),
        };
        assert!(matches!(mode, ProxyMode::Strict));
    }

    #[test]
    fn test_mode_parsing_advisory() {
        let mode_str = "advisory";
        let mode = match mode_str {
            "strict" => ProxyMode::Strict,
            "advisory" => ProxyMode::Advisory,
            "auto_fix" | "auto-fix" => ProxyMode::AutoFix,
            _ => panic!("Invalid mode"),
        };
        assert!(matches!(mode, ProxyMode::Advisory));
    }

    #[test]
    fn test_mode_parsing_auto_fix() {
        let mode_str = "auto_fix";
        let mode = match mode_str {
            "strict" => ProxyMode::Strict,
            "advisory" => ProxyMode::Advisory,
            "auto_fix" | "auto-fix" => ProxyMode::AutoFix,
            _ => panic!("Invalid mode"),
        };
        assert!(matches!(mode, ProxyMode::AutoFix));
    }

    #[test]
    fn test_mode_parsing_auto_fix_with_hyphen() {
        let mode_str = "auto-fix";
        let mode = match mode_str {
            "strict" => ProxyMode::Strict,
            "advisory" => ProxyMode::Advisory,
            "auto_fix" | "auto-fix" => ProxyMode::AutoFix,
            _ => panic!("Invalid mode"),
        };
        assert!(matches!(mode, ProxyMode::AutoFix));
    }

    #[test]
    fn test_quality_config_conversion() {
        let input_config = QualityConfigInput {
            max_complexity: 15,
            allow_satd: true,
            require_docs: false,
            auto_format: true,
        };

        let quality_config = QualityConfig {
            max_complexity: input_config.max_complexity,
            allow_satd: input_config.allow_satd,
            require_docs: input_config.require_docs,
            auto_format: input_config.auto_format,
        };

        assert_eq!(quality_config.max_complexity, 15);
        assert!(quality_config.allow_satd);
        assert!(!quality_config.require_docs);
        assert!(quality_config.auto_format);
    }

    #[test]
    fn test_proxy_request_construction() {
        let request = ProxyRequest {
            operation: ProxyOperation::Write,
            file_path: "test.rs".to_string(),
            content: Some("fn test() {}".to_string()),
            old_content: None,
            new_content: None,
            mode: ProxyMode::Strict,
            quality_config: QualityConfig {
                max_complexity: 20,
                allow_satd: false,
                require_docs: true,
                auto_format: true,
            },
        };

        assert_eq!(request.file_path, "test.rs");
        assert!(matches!(request.operation, ProxyOperation::Write));
        assert!(matches!(request.mode, ProxyMode::Strict));
    }
}

