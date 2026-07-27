    proptest! {

        /// Property: McpResponse success always has jsonrpc "2.0"
        #[test]
        fn prop_mcp_response_success_jsonrpc(id in 1i64..1000i64) {
            let response = McpResponse::success(json!(id), json!({"ok": true}));
            prop_assert_eq!(response.jsonrpc, "2.0");
        }

        /// Property: McpResponse error always has jsonrpc "2.0"
        #[test]
        fn prop_mcp_response_error_jsonrpc(id in 1i64..1000i64, code in -32700i32..-32000i32) {
            let response = McpResponse::error(json!(id), code, "Error".to_string());
            prop_assert_eq!(response.jsonrpc, "2.0");
        }

        /// Property: McpResponse success has result and no error
        #[test]
        fn prop_mcp_response_success_structure(id in 1i64..1000i64) {
            let response = McpResponse::success(json!(id), json!({}));
            prop_assert!(response.result.is_some());
            prop_assert!(response.error.is_none());
        }

        /// Property: McpResponse error has error and no result
        #[test]
        fn prop_mcp_response_error_structure(id in 1i64..1000i64, code in -32700i32..-32000i32) {
            let response = McpResponse::error(json!(id), code, "Error".to_string());
            prop_assert!(response.error.is_some());
            prop_assert!(response.result.is_none());
        }

        /// Property: Serialization roundtrip preserves McpRequest
        #[test]
        fn prop_mcp_request_roundtrip(method in "[a-z/]+") {
            let request = McpRequest {
                jsonrpc: "2.0".to_string(),
                id: json!(1),
                method: method.clone(),
                params: None,
            };

            let json = serde_json::to_string(&request).unwrap();
            let roundtrip: McpRequest = serde_json::from_str(&json).unwrap();

            prop_assert_eq!(roundtrip.method, method);
            prop_assert_eq!(roundtrip.jsonrpc, "2.0");
        }

        /// Property: Error code is preserved in McpError
        #[test]
        fn prop_mcp_error_code_preserved(code in -40000i32..40000i32) {
            let error = McpError {
                code,
                message: "Test".to_string(),
                data: None,
            };

            let json = serde_json::to_string(&error).unwrap();
            let roundtrip: McpError = serde_json::from_str(&json).unwrap();

            prop_assert_eq!(roundtrip.code, code);
        }

        /// Property: PromptArgument required flag preserved
        #[test]
        fn prop_prompt_argument_required_preserved(name in "[a-z_]+", required in any::<bool>()) {
            let arg = PromptArgument {
                name: name.clone(),
                description: None,
                required,
            };

            let json = serde_json::to_string(&arg).unwrap();
            let roundtrip: PromptArgument = serde_json::from_str(&json).unwrap();

            prop_assert_eq!(roundtrip.name, name);
            prop_assert_eq!(roundtrip.required, required);
        }
    }
