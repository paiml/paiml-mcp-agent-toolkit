#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    // ═══════════════════════════════════════════════════════════════════════════
    // RED PHASE TESTS - These MUST fail until implementation is complete
    // ═══════════════════════════════════════════════════════════════════════════

    mod registry_core_tests {
        use super::*;

        #[test]
        fn test_registry_creation() {
            let registry = CommandRegistry::new("1.0.0");
            assert_eq!(registry.version, "1.0.0");
            assert!(registry.commands.is_empty());
            assert!(registry.global_flags.is_empty());
            assert!(registry.built_at.is_some());
        }

        #[test]
        fn test_register_command() {
            let mut registry = CommandRegistry::new("1.0.0");
            let cmd = CommandMetadata::builder("analyze")
                .short_description("Analyze code")
                .category("analysis")
                .build();

            registry.register(cmd);

            assert_eq!(registry.commands.len(), 1);
            assert!(registry.commands.contains_key("analyze"));
        }

        #[test]
        fn test_find_command_exact() {
            let mut registry = CommandRegistry::new("1.0.0");
            registry.register(
                CommandMetadata::builder("analyze")
                    .short_description("Analyze code")
                    .build(),
            );

            let found = registry.find_command("analyze");
            assert!(found.is_some());
            assert_eq!(found.unwrap().name, "analyze");
        }

        #[test]
        fn test_find_command_by_alias() {
            let mut registry = CommandRegistry::new("1.0.0");
            registry.register(
                CommandMetadata::builder("analyze")
                    .short_description("Analyze code")
                    .aliases(["a", "an"])
                    .build(),
            );

            let found = registry.find_command("a");
            assert!(found.is_some());
            assert_eq!(found.unwrap().name, "analyze");

            let found2 = registry.find_command("an");
            assert!(found2.is_some());
        }

        #[test]
        fn test_find_command_hierarchical() {
            let mut registry = CommandRegistry::new("1.0.0");
            let complexity_cmd = CommandMetadata::builder("complexity")
                .short_description("Analyze complexity")
                .build();

            registry.register(
                CommandMetadata::builder("analyze")
                    .short_description("Analyze code")
                    .subcommand(complexity_cmd)
                    .build(),
            );

            let found = registry.find_command("analyze complexity");
            assert!(found.is_some());
            assert_eq!(found.unwrap().name, "complexity");
        }

        #[test]
        fn test_find_by_tag() {
            let mut registry = CommandRegistry::new("1.0.0");
            registry.register(
                CommandMetadata::builder("analyze")
                    .tags(["quality", "metrics"])
                    .build(),
            );
            registry.register(
                CommandMetadata::builder("context")
                    .tags(["generation", "ast"])
                    .build(),
            );

            let quality_cmds = registry.find_by_tag("quality");
            assert_eq!(quality_cmds.len(), 1);
            assert_eq!(quality_cmds[0].name, "analyze");
        }

        #[test]
        fn test_find_by_category() {
            let mut registry = CommandRegistry::new("1.0.0");
            registry.register(
                CommandMetadata::builder("analyze")
                    .category("analysis")
                    .build(),
            );
            registry.register(
                CommandMetadata::builder("scaffold")
                    .category("generation")
                    .build(),
            );

            let analysis_cmds = registry.find_by_category("analysis");
            assert_eq!(analysis_cmds.len(), 1);
            assert_eq!(analysis_cmds[0].name, "analyze");
        }

        #[test]
        fn test_all_command_paths() {
            let mut registry = CommandRegistry::new("1.0.0");
            let sub = CommandMetadata::builder("complexity").build();
            registry.register(CommandMetadata::builder("analyze").subcommand(sub).build());
            registry.register(CommandMetadata::builder("context").build());

            let paths = registry.all_command_paths();
            assert!(paths.contains(&"analyze".to_string()));
            assert!(paths.contains(&"analyze complexity".to_string()));
            assert!(paths.contains(&"context".to_string()));
        }
    }

    mod registry_validation_tests {
        use super::*;

        #[test]
        fn test_validate_duplicate_alias() {
            let mut registry = CommandRegistry::new("1.0.0");
            registry.register(CommandMetadata::builder("analyze").alias("a").build());
            registry.register(
                CommandMetadata::builder("agent")
                    .alias("a") // Duplicate!
                    .build(),
            );

            let result = registry.validate();
            assert!(result.is_err());
            let errors = result.unwrap_err();
            assert!(errors
                .iter()
                .any(|e| matches!(e, RegistryError::DuplicateAlias { .. })));
        }

        #[test]
        fn test_validate_duplicate_mcp_tool() {
            let mut registry = CommandRegistry::new("1.0.0");
            registry.register(
                CommandMetadata::builder("analyze")
                    .mcp(McpToolMetadata {
                        tool_name: "pmat_analyze".to_string(),
                        ..Default::default()
                    })
                    .build(),
            );
            registry.register(
                CommandMetadata::builder("context")
                    .mcp(McpToolMetadata {
                        tool_name: "pmat_analyze".to_string(), // Duplicate!
                        ..Default::default()
                    })
                    .build(),
            );

            let result = registry.validate();
            assert!(result.is_err());
            let errors = result.unwrap_err();
            assert!(errors
                .iter()
                .any(|e| matches!(e, RegistryError::DuplicateMcpTool { .. })));
        }

        #[test]
        fn test_validate_invalid_related_command() {
            let mut registry = CommandRegistry::new("1.0.0");
            registry.register(
                CommandMetadata::builder("analyze")
                    .related("nonexistent") // Invalid reference!
                    .build(),
            );

            let result = registry.validate();
            assert!(result.is_err());
            let errors = result.unwrap_err();
            assert!(errors
                .iter()
                .any(|e| matches!(e, RegistryError::InvalidRelatedCommand { .. })));
        }

        #[test]
        fn test_validate_success() {
            let mut registry = CommandRegistry::new("1.0.0");
            registry.register(CommandMetadata::builder("analyze").alias("a").build());
            registry.register(
                CommandMetadata::builder("context")
                    .alias("ctx")
                    .related("analyze") // Valid reference
                    .build(),
            );

            let result = registry.validate();
            assert!(result.is_ok());
        }
    }

    mod serialization_tests {
        use super::*;

        #[test]
        fn test_to_json() {
            let mut registry = CommandRegistry::new("1.0.0");
            registry.register(
                CommandMetadata::builder("analyze")
                    .short_description("Analyze code")
                    .build(),
            );

            let json = registry.to_json().unwrap();
            assert!(json.contains("\"version\": \"1.0.0\""));
            assert!(json.contains("\"analyze\""));
        }

        #[test]
        fn test_from_json_roundtrip() {
            let mut registry = CommandRegistry::new("1.0.0");
            registry.register(
                CommandMetadata::builder("analyze")
                    .short_description("Analyze code")
                    .aliases(["a", "an"])
                    .tags(["quality"])
                    .build(),
            );

            let json = registry.to_json().unwrap();
            let restored = CommandRegistry::from_json(&json).unwrap();

            assert_eq!(restored.version, registry.version);
            assert_eq!(restored.commands.len(), registry.commands.len());
            let cmd = restored.commands.get("analyze").unwrap();
            assert_eq!(cmd.aliases, vec!["a", "an"]);
        }
    }

    mod builder_tests {
        use super::*;

        #[test]
        fn test_command_builder() {
            let cmd = CommandMetadata::builder("analyze")
                .short_description("Analyze code")
                .long_description("Analyze code for various metrics")
                .aliases(["a", "an"])
                .category("analysis")
                .tags(["quality", "metrics"])
                .is_mutation(false)
                .execution_time(ExecutionTime::Fast)
                .build();

            assert_eq!(cmd.name, "analyze");
            assert_eq!(cmd.short_description, "Analyze code");
            assert_eq!(cmd.aliases, vec!["a", "an"]);
            assert_eq!(cmd.category, "analysis");
            assert!(!cmd.is_mutation);
            assert_eq!(cmd.execution_time, ExecutionTime::Fast);
        }

        #[test]
        fn test_builder_with_arguments() {
            let cmd = CommandMetadata::builder("analyze")
                .argument(ArgumentMetadata {
                    name: "project-path".to_string(),
                    short: Some('p'),
                    long: Some("project-path".to_string()),
                    description: "Path to project".to_string(),
                    required: false,
                    default: Some(".".to_string()),
                    value_type: ValueType::Path,
                    ..Default::default()
                })
                .build();

            assert_eq!(cmd.arguments.len(), 1);
            assert_eq!(cmd.arguments[0].name, "project-path");
            assert_eq!(cmd.arguments[0].short, Some('p'));
        }

        #[test]
        fn test_builder_with_examples() {
            let cmd = CommandMetadata::builder("analyze")
                .example(ExampleMetadata {
                    description: "Analyze current project".to_string(),
                    command: "pmat analyze complexity".to_string(),
                    expected_exit_code: 0,
                    requires_project: false,
                    ..Default::default()
                })
                .build();

            assert_eq!(cmd.examples.len(), 1);
            assert_eq!(cmd.examples[0].command, "pmat analyze complexity");
        }

        #[test]
        fn test_builder_with_mcp() {
            let cmd = CommandMetadata::builder("analyze")
                .mcp(McpToolMetadata {
                    tool_name: "pmat_analyze_complexity".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "project_path": { "type": "string" }
                        }
                    }),
                    is_mutation: false,
                    execution_time: ExecutionTime::Medium,
                    annotations: McpAnnotations {
                        title: "Analyze Complexity".to_string(),
                        read_only_hint: true,
                        ..Default::default()
                    },
                })
                .build();

            assert!(cmd.mcp.is_some());
            let mcp = cmd.mcp.unwrap();
            assert_eq!(mcp.tool_name, "pmat_analyze_complexity");
            assert!(mcp.annotations.read_only_hint);
        }

        #[test]
        fn test_builder_with_subcommands() {
            let sub1 = CommandMetadata::builder("complexity")
                .short_description("Analyze complexity")
                .build();
            let sub2 = CommandMetadata::builder("satd")
                .short_description("Find SATD")
                .build();

            let cmd = CommandMetadata::builder("analyze")
                .subcommand(sub1)
                .subcommand(sub2)
                .build();

            assert_eq!(cmd.subcommands.len(), 2);
            assert_eq!(cmd.subcommands[0].name, "complexity");
            assert_eq!(cmd.subcommands[1].name, "satd");
        }

        #[test]
        fn test_builder_deprecated() {
            let cmd = CommandMetadata::builder("old-command")
                .deprecated(DeprecationInfo {
                    since_version: "2.0.0".to_string(),
                    removal_version: Some("3.0.0".to_string()),
                    replacement: Some("new-command".to_string()),
                    reason: "Replaced with better implementation".to_string(),
                })
                .build();

            assert!(cmd.deprecated.is_some());
            let dep = cmd.deprecated.unwrap();
            assert_eq!(dep.since_version, "2.0.0");
            assert_eq!(dep.replacement, Some("new-command".to_string()));
        }
    }

    mod metadata_tests {
        use super::*;

        #[test]
        fn test_find_subcommand() {
            let sub = CommandMetadata::builder("complexity")
                .aliases(["cx"])
                .build();
            let cmd = CommandMetadata::builder("analyze").subcommand(sub).build();

            // Find by name
            let found = cmd.find_subcommand("complexity");
            assert!(found.is_some());

            // Find by alias
            let found_alias = cmd.find_subcommand("cx");
            assert!(found_alias.is_some());
            assert_eq!(found_alias.unwrap().name, "complexity");
        }

        #[test]
        fn test_full_path() {
            let cmd = CommandMetadata::builder("complexity").build();

            assert_eq!(cmd.full_path(None), "complexity");
            assert_eq!(cmd.full_path(Some("analyze")), "analyze complexity");
        }
    }
}
