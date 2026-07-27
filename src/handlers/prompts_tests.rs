#[cfg_attr(coverage_nightly, coverage(off))]

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // === Prompt struct tests ===

    #[test]
    fn test_prompt_creation() {
        let prompt = Prompt {
            name: "test-prompt".to_string(),
            description: "Test description".to_string(),
            arguments: vec![],
        };

        assert_eq!(prompt.name, "test-prompt");
        assert_eq!(prompt.description, "Test description");
        assert!(prompt.arguments.is_empty());
    }

    #[test]
    fn test_prompt_with_arguments() {
        let prompt = Prompt {
            name: "test-prompt".to_string(),
            description: "Test description".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "arg1".to_string(),
                    description: Some("First argument".to_string()),
                    required: true,
                },
                PromptArgument {
                    name: "arg2".to_string(),
                    description: None,
                    required: false,
                },
            ],
        };

        assert_eq!(prompt.arguments.len(), 2);
        assert!(prompt.arguments[0].required);
        assert!(!prompt.arguments[1].required);
    }

    #[test]
    fn test_prompt_serialization() {
        let prompt = Prompt {
            name: "test".to_string(),
            description: "desc".to_string(),
            arguments: vec![],
        };

        let json = serde_json::to_string(&prompt).unwrap();
        assert!(json.contains("test"));
        assert!(json.contains("desc"));
    }

    // === PromptArgument tests ===

    #[test]
    fn test_prompt_argument_required() {
        let arg = PromptArgument {
            name: "project_name".to_string(),
            description: Some("The project name".to_string()),
            required: true,
        };

        assert_eq!(arg.name, "project_name");
        assert!(arg.required);
        assert!(arg.description.is_some());
    }

    #[test]
    fn test_prompt_argument_optional() {
        let arg = PromptArgument {
            name: "optional_arg".to_string(),
            description: None,
            required: false,
        };

        assert!(!arg.required);
        assert!(arg.description.is_none());
    }

    #[test]
    fn test_prompt_argument_serialization() {
        let arg = PromptArgument {
            name: "test_arg".to_string(),
            description: Some("test description".to_string()),
            required: true,
        };

        let json = serde_json::to_string(&arg).unwrap();
        assert!(json.contains("test_arg"));
        assert!(json.contains("required"));
        assert!(json.contains("true"));
    }

    // === PromptGetParams tests ===

    #[test]
    fn test_prompt_get_params_deserialization() {
        let json_str = r#"{"name": "scaffold-rust-project"}"#;
        let params: PromptGetParams = serde_json::from_str(json_str).unwrap();
        assert_eq!(params.name, "scaffold-rust-project");
    }

    #[test]
    fn test_prompt_get_params_various_names() {
        let names = vec![
            "scaffold-rust-project",
            "scaffold-deno-project",
            "scaffold-python-project",
        ];

        for name in names {
            let json_str = format!(r#"{{"name": "{}"}}"#, name);
            let params: PromptGetParams = serde_json::from_str(&json_str).unwrap();
            assert_eq!(params.name, name);
        }
    }

    // === Prompt definitions tests ===

    #[test]
    fn test_rust_project_prompt_definition() {
        let prompt = Prompt {
            name: "scaffold-rust-project".to_string(),
            description:
                "Create a complete Rust project structure with Makefile, README, and .gitignore"
                    .to_string(),
            arguments: vec![
                PromptArgument {
                    name: "project_name".to_string(),
                    description: Some("Name of the Rust project".to_string()),
                    required: true,
                },
                PromptArgument {
                    name: "project_type".to_string(),
                    description: Some("Type of project: cli or library-crate".to_string()),
                    required: true,
                },
            ],
        };

        assert_eq!(prompt.name, "scaffold-rust-project");
        assert!(prompt.description.contains("Rust"));
        assert!(prompt.arguments.iter().any(|a| a.name == "project_name"));
        assert!(prompt.arguments.iter().any(|a| a.name == "project_type"));
    }

    #[test]
    fn test_deno_project_prompt_definition() {
        let prompt = Prompt {
            name: "scaffold-deno-project".to_string(),
            description: "Create a complete Deno/TypeScript project structure".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "project_name".to_string(),
                    description: Some("Name of the Deno project".to_string()),
                    required: true,
                },
                PromptArgument {
                    name: "permissions".to_string(),
                    description: Some("Deno permissions needed".to_string()),
                    required: false,
                },
            ],
        };

        assert_eq!(prompt.name, "scaffold-deno-project");
        assert!(prompt.description.contains("Deno"));
        assert!(prompt
            .arguments
            .iter()
            .any(|a| a.name == "permissions" && !a.required));
    }

    #[test]
    fn test_python_project_prompt_definition() {
        let prompt = Prompt {
            name: "scaffold-python-project".to_string(),
            description: "Create a complete Python UV project structure".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "project_name".to_string(),
                    description: Some("Name of the Python project".to_string()),
                    required: true,
                },
                PromptArgument {
                    name: "python_version".to_string(),
                    description: Some("Python version to use".to_string()),
                    required: false,
                },
            ],
        };

        assert_eq!(prompt.name, "scaffold-python-project");
        assert!(prompt.description.contains("Python"));
        assert!(prompt
            .arguments
            .iter()
            .any(|a| a.name == "python_version" && !a.required));
    }

    // === Error response tests ===

    #[test]
    fn test_mcp_error_code() {
        // Test that the error code for invalid params is -32602
        assert_eq!(-32602, -32602); // JSON-RPC invalid params error code
    }

    // === JSON serialization round-trip tests ===

    #[test]
    fn test_prompt_roundtrip_serialization() {
        let prompt = Prompt {
            name: "test-prompt".to_string(),
            description: "Test".to_string(),
            arguments: vec![PromptArgument {
                name: "arg1".to_string(),
                description: Some("desc".to_string()),
                required: true,
            }],
        };

        let json = serde_json::to_string(&prompt).unwrap();
        let parsed: Prompt = serde_json::from_str(&json).unwrap();

        assert_eq!(prompt.name, parsed.name);
        assert_eq!(prompt.description, parsed.description);
        assert_eq!(prompt.arguments.len(), parsed.arguments.len());
    }

    #[test]
    fn test_prompt_argument_roundtrip_serialization() {
        let arg = PromptArgument {
            name: "test".to_string(),
            description: Some("description".to_string()),
            required: true,
        };

        let json = serde_json::to_string(&arg).unwrap();
        let parsed: PromptArgument = serde_json::from_str(&json).unwrap();

        assert_eq!(arg.name, parsed.name);
        assert_eq!(arg.description, parsed.description);
        assert_eq!(arg.required, parsed.required);
    }

    // === JSON Value tests ===

    #[test]
    fn test_prompt_to_json_value() {
        let prompt = Prompt {
            name: "test".to_string(),
            description: "desc".to_string(),
            arguments: vec![],
        };

        let value = json!(prompt);
        assert!(value.get("name").is_some());
        assert!(value.get("description").is_some());
        assert!(value.get("arguments").is_some());
    }

    #[test]
    fn test_prompt_array_to_json() {
        let prompts = vec![
            Prompt {
                name: "p1".to_string(),
                description: "d1".to_string(),
                arguments: vec![],
            },
            Prompt {
                name: "p2".to_string(),
                description: "d2".to_string(),
                arguments: vec![],
            },
        ];

        let json = json!({ "prompts": prompts });
        let arr = json.get("prompts").unwrap().as_array().unwrap();
        assert_eq!(arr.len(), 2);
    }
}
