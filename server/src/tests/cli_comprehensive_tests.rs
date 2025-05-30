#[cfg(test)]
use crate::cli::{
    args::{expand_env_vars, parse_key_val, validate_params},
    AnalyzeCommands, Cli, Commands, ComplexityOutputFormat, ContextFormat, DagType, OutputFormat,
};
#[cfg(test)]
use crate::models::{
    churn::ChurnOutputFormat,
    template::{ParameterSpec, ParameterType},
};
#[cfg(test)]
use clap::Parser;
#[cfg(test)]
use serde_json::{json, Value};
#[cfg(test)]
use std::path::PathBuf;

// ===== Generate Command Tests =====

#[test]
fn test_generate_command_full_parsing() {
    let args = vec![
        "paiml-mcp-agent-toolkit",
        "generate",
        "makefile",
        "rust/cli",
        "-p",
        "project_name=test-project",
        "-p",
        "has_tests=true",
        "-p",
        "has_benchmarks=false",
        "-o",
        "/tmp/Makefile",
        "--create-dirs",
    ];

    let cli = Cli::try_parse_from(&args).unwrap();
    match cli.command {
        Commands::Generate {
            category,
            template,
            params,
            output,
            create_dirs,
        } => {
            assert_eq!(category, "makefile");
            assert_eq!(template, "rust/cli");
            assert_eq!(output, Some(PathBuf::from("/tmp/Makefile")));
            assert!(create_dirs);

            // Verify all parameters parsed correctly
            assert_eq!(params.len(), 3);
            let param_map: std::collections::HashMap<String, Value> = params.into_iter().collect();
            assert_eq!(param_map["project_name"], json!("test-project"));
            assert_eq!(param_map["has_tests"], json!(true));
            assert_eq!(param_map["has_benchmarks"], json!(false));
        }
        _ => panic!("Expected Generate command"),
    }
}

#[test]
fn test_generate_command_aliases() {
    // Test 'gen' alias
    let args = vec![
        "paiml-mcp-agent-toolkit",
        "gen",
        "readme",
        "deno/cli",
        "-p",
        "project_name=alias-test",
    ];
    assert!(Cli::try_parse_from(&args).is_ok());

    // Test 'g' alias
    let args = vec![
        "paiml-mcp-agent-toolkit",
        "g",
        "gitignore",
        "python-uv/cli",
        "-p",
        "project_name=short-alias",
    ];
    assert!(Cli::try_parse_from(&args).is_ok());
}

#[test]
fn test_generate_missing_required_args() {
    // Missing template path
    let args = vec!["paiml-mcp-agent-toolkit", "generate", "makefile"];
    assert!(Cli::try_parse_from(&args).is_err());

    // Missing category
    let args = vec!["paiml-mcp-agent-toolkit", "generate"];
    assert!(Cli::try_parse_from(&args).is_err());
}

// ===== Scaffold Command Tests =====

#[test]
fn test_scaffold_command_parsing() {
    let args = vec![
        "paiml-mcp-agent-toolkit",
        "scaffold",
        "rust",
        "--templates",
        "makefile,readme,gitignore",
        "-p",
        "project_name=scaffold-test",
        "-p",
        "description=Test scaffolding",
        "--parallel",
        "4",
    ];

    let cli = Cli::try_parse_from(&args).unwrap();
    match cli.command {
        Commands::Scaffold {
            toolchain,
            templates,
            params,
            parallel,
        } => {
            assert_eq!(toolchain, "rust");
            assert_eq!(templates, vec!["makefile", "readme", "gitignore"]);
            assert_eq!(parallel, 4);

            let param_map: std::collections::HashMap<String, Value> = params.into_iter().collect();
            assert_eq!(param_map["project_name"], json!("scaffold-test"));
            assert_eq!(param_map["description"], json!("Test scaffolding"));
        }
        _ => panic!("Expected Scaffold command"),
    }
}

#[test]
fn test_scaffold_template_delimiter() {
    // Test comma-separated templates
    let args = vec![
        "paiml-mcp-agent-toolkit",
        "scaffold",
        "deno",
        "-t",
        "makefile,readme",
        "-p",
        "project_name=test",
    ];

    let cli = Cli::try_parse_from(&args).unwrap();
    match cli.command {
        Commands::Scaffold { templates, .. } => {
            assert_eq!(templates.len(), 2);
            assert_eq!(templates[0], "makefile");
            assert_eq!(templates[1], "readme");
        }
        _ => panic!("Expected Scaffold command"),
    }
}

#[test]
fn test_scaffold_default_parallel() {
    let args = vec![
        "paiml-mcp-agent-toolkit",
        "scaffold",
        "python-uv",
        "-t",
        "readme",
        "-p",
        "project_name=test",
    ];

    let cli = Cli::try_parse_from(&args).unwrap();
    match cli.command {
        Commands::Scaffold { parallel, .. } => {
            // Should default to number of CPUs
            assert_eq!(parallel, num_cpus::get());
        }
        _ => panic!("Expected Scaffold command"),
    }
}

// ===== List Command Tests =====

#[test]
fn test_list_command_all_formats() {
    let formats = vec!["table", "json", "yaml"];

    for format in formats {
        let args = vec!["paiml-mcp-agent-toolkit", "list", "--format", format];
        let cli = Cli::try_parse_from(&args).unwrap();

        match cli.command {
            Commands::List {
                format: output_format,
                ..
            } => match format {
                "table" => assert_eq!(output_format, OutputFormat::Table),
                "json" => assert_eq!(output_format, OutputFormat::Json),
                "yaml" => assert_eq!(output_format, OutputFormat::Yaml),
                _ => unreachable!(),
            },
            _ => panic!("Expected List command"),
        }
    }
}

#[test]
fn test_list_command_filters() {
    let args = vec![
        "paiml-mcp-agent-toolkit",
        "list",
        "--toolchain",
        "rust",
        "--category",
        "makefile",
        "--format",
        "json",
    ];

    let cli = Cli::try_parse_from(&args).unwrap();
    match cli.command {
        Commands::List {
            toolchain,
            category,
            format,
        } => {
            assert_eq!(toolchain, Some("rust".to_string()));
            assert_eq!(category, Some("makefile".to_string()));
            assert_eq!(format, OutputFormat::Json);
        }
        _ => panic!("Expected List command"),
    }
}

#[test]
fn test_list_default_format() {
    let args = vec!["paiml-mcp-agent-toolkit", "list"];
    let cli = Cli::try_parse_from(&args).unwrap();

    match cli.command {
        Commands::List { format, .. } => {
            assert_eq!(format, OutputFormat::Table);
        }
        _ => panic!("Expected List command"),
    }
}

// ===== Search Command Tests =====

#[test]
fn test_search_command_parsing() {
    let args = vec![
        "paiml-mcp-agent-toolkit",
        "search",
        "rust makefile",
        "--toolchain",
        "rust",
        "--limit",
        "50",
    ];

    let cli = Cli::try_parse_from(&args).unwrap();
    match cli.command {
        Commands::Search {
            query,
            toolchain,
            limit,
        } => {
            assert_eq!(query, "rust makefile");
            assert_eq!(toolchain, Some("rust".to_string()));
            assert_eq!(limit, 50);
        }
        _ => panic!("Expected Search command"),
    }
}

#[test]
fn test_search_default_limit() {
    let args = vec!["paiml-mcp-agent-toolkit", "search", "template"];
    let cli = Cli::try_parse_from(&args).unwrap();

    match cli.command {
        Commands::Search { limit, .. } => {
            assert_eq!(limit, 20); // Default limit
        }
        _ => panic!("Expected Search command"),
    }
}

// ===== Validate Command Tests =====

#[test]
fn test_validate_command_parsing() {
    let args = vec![
        "paiml-mcp-agent-toolkit",
        "validate",
        "template://makefile/rust/cli",
        "-p",
        "project_name=validation-test",
        "-p",
        "has_tests=true",
    ];

    let cli = Cli::try_parse_from(&args).unwrap();
    match cli.command {
        Commands::Validate { uri, params } => {
            assert_eq!(uri, "template://makefile/rust/cli");
            assert_eq!(params.len(), 2);
        }
        _ => panic!("Expected Validate command"),
    }
}

// ===== Context Command Tests =====

#[test]
fn test_context_command_parsing() {
    let args = vec![
        "paiml-mcp-agent-toolkit",
        "context",
        "rust",
        "-p",
        "/tmp/project",
        "-o",
        "context.md",
        "--format",
        "markdown",
    ];

    let cli = Cli::try_parse_from(&args).unwrap();
    match cli.command {
        Commands::Context {
            toolchain,
            project_path,
            output,
            format,
        } => {
            assert_eq!(toolchain, "rust");
            assert_eq!(project_path, PathBuf::from("/tmp/project"));
            assert_eq!(output, Some(PathBuf::from("context.md")));
            assert_eq!(format, ContextFormat::Markdown);
        }
        _ => panic!("Expected Context command"),
    }
}

#[test]
fn test_context_formats() {
    let formats = vec![
        ("markdown", ContextFormat::Markdown),
        ("json", ContextFormat::Json),
    ];

    for (format_str, expected) in formats {
        let args = vec![
            "paiml-mcp-agent-toolkit",
            "context",
            "deno",
            "--format",
            format_str,
        ];
        let cli = Cli::try_parse_from(&args).unwrap();

        match cli.command {
            Commands::Context { format, .. } => {
                assert_eq!(format, expected);
            }
            _ => panic!("Expected Context command"),
        }
    }
}

#[test]
fn test_context_default_values() {
    let args = vec!["paiml-mcp-agent-toolkit", "context", "python-uv"];
    let cli = Cli::try_parse_from(&args).unwrap();

    match cli.command {
        Commands::Context {
            project_path,
            format,
            output,
            ..
        } => {
            assert_eq!(project_path, PathBuf::from("."));
            assert_eq!(format, ContextFormat::Markdown);
            assert!(output.is_none());
        }
        _ => panic!("Expected Context command"),
    }
}

// ===== Analyze Churn Tests =====

#[test]
fn test_analyze_churn_full_options() {
    let args = vec![
        "paiml-mcp-agent-toolkit",
        "analyze",
        "churn",
        "-p",
        "/tmp/repo",
        "-d",
        "60",
        "--format",
        "csv",
        "-o",
        "churn.csv",
    ];

    let cli = Cli::try_parse_from(&args).unwrap();
    match cli.command {
        Commands::Analyze(AnalyzeCommands::Churn {
            project_path,
            days,
            format,
            output,
        }) => {
            assert_eq!(project_path, PathBuf::from("/tmp/repo"));
            assert_eq!(days, 60);
            assert_eq!(format, ChurnOutputFormat::Csv);
            assert_eq!(output, Some(PathBuf::from("churn.csv")));
        }
        _ => panic!("Expected Analyze Churn command"),
    }
}

#[test]
fn test_analyze_churn_all_formats() {
    let formats = vec![
        ("summary", ChurnOutputFormat::Summary),
        ("json", ChurnOutputFormat::Json),
        ("markdown", ChurnOutputFormat::Markdown),
        ("csv", ChurnOutputFormat::Csv),
    ];

    for (format_str, expected) in formats {
        let args = vec![
            "paiml-mcp-agent-toolkit",
            "analyze",
            "churn",
            "--format",
            format_str,
        ];
        let cli = Cli::try_parse_from(&args).unwrap();

        match cli.command {
            Commands::Analyze(AnalyzeCommands::Churn { format, .. }) => {
                assert_eq!(format, expected);
            }
            _ => panic!("Expected Analyze Churn command"),
        }
    }
}

// ===== Analyze Complexity Tests =====

#[test]
fn test_analyze_complexity_full_options() {
    let args = vec![
        "paiml-mcp-agent-toolkit",
        "analyze",
        "complexity",
        "-p",
        "/workspace",
        "--toolchain",
        "rust",
        "--format",
        "sarif",
        "-o",
        "complexity.sarif",
        "--max-cyclomatic",
        "15",
        "--max-cognitive",
        "20",
        "--include",
        "**/*.rs",
        "--include",
        "src/**/*.rs",
        "--watch",
    ];

    let cli = Cli::try_parse_from(&args).unwrap();
    match cli.command {
        Commands::Analyze(AnalyzeCommands::Complexity {
            project_path,
            toolchain,
            format,
            output,
            max_cyclomatic,
            max_cognitive,
            include,
            watch,
        }) => {
            assert_eq!(project_path, PathBuf::from("/workspace"));
            assert_eq!(toolchain, Some("rust".to_string()));
            assert_eq!(format, ComplexityOutputFormat::Sarif);
            assert_eq!(output, Some(PathBuf::from("complexity.sarif")));
            assert_eq!(max_cyclomatic, Some(15));
            assert_eq!(max_cognitive, Some(20));
            assert_eq!(include, vec!["**/*.rs", "src/**/*.rs"]);
            assert!(watch);
        }
        _ => panic!("Expected Analyze Complexity command"),
    }
}

#[test]
fn test_analyze_complexity_formats() {
    let formats = vec![
        ("summary", ComplexityOutputFormat::Summary),
        ("full", ComplexityOutputFormat::Full),
        ("json", ComplexityOutputFormat::Json),
        ("sarif", ComplexityOutputFormat::Sarif),
    ];

    for (format_str, expected) in formats {
        let args = vec![
            "paiml-mcp-agent-toolkit",
            "analyze",
            "complexity",
            "--format",
            format_str,
        ];
        let cli = Cli::try_parse_from(&args).unwrap();

        match cli.command {
            Commands::Analyze(AnalyzeCommands::Complexity { format, .. }) => {
                assert_eq!(format, expected);
            }
            _ => panic!("Expected Analyze Complexity command"),
        }
    }
}

// ===== Analyze DAG Tests =====

#[test]
fn test_analyze_dag_full_options() {
    let args = vec![
        "paiml-mcp-agent-toolkit",
        "analyze",
        "dag",
        "--dag-type",
        "import-graph",
        "-p",
        "/project",
        "-o",
        "dependencies.mmd",
        "--max-depth",
        "5",
        "--filter-external",
        "--show-complexity",
    ];

    let cli = Cli::try_parse_from(&args).unwrap();
    match cli.command {
        Commands::Analyze(AnalyzeCommands::Dag {
            dag_type,
            project_path,
            output,
            max_depth,
            filter_external,
            show_complexity,
            include_duplicates: _,
            include_dead_code: _,
            enhanced: _,
        }) => {
            assert_eq!(dag_type, DagType::ImportGraph);
            assert_eq!(project_path, PathBuf::from("/project"));
            assert_eq!(output, Some(PathBuf::from("dependencies.mmd")));
            assert_eq!(max_depth, Some(5));
            assert!(filter_external);
            assert!(show_complexity);
        }
        _ => panic!("Expected Analyze DAG command"),
    }
}

#[test]
fn test_analyze_dag_types() {
    let dag_types = vec![
        ("call-graph", DagType::CallGraph),
        ("import-graph", DagType::ImportGraph),
        ("inheritance", DagType::Inheritance),
        ("full-dependency", DagType::FullDependency),
    ];

    for (type_str, expected) in dag_types {
        let args = vec![
            "paiml-mcp-agent-toolkit",
            "analyze",
            "dag",
            "--dag-type",
            type_str,
        ];
        let cli = Cli::try_parse_from(&args).unwrap();

        match cli.command {
            Commands::Analyze(AnalyzeCommands::Dag { dag_type, .. }) => {
                assert_eq!(dag_type, expected);
            }
            _ => panic!("Expected Analyze DAG command"),
        }
    }
}

// ===== Parameter Parsing Tests =====

#[test]
fn test_parse_key_val_basic() {
    let test_cases = vec![
        ("key=value", ("key", json!("value"))),
        ("name=test-project", ("name", json!("test-project"))),
        ("enabled=true", ("enabled", json!(true))),
        ("count=42", ("count", json!(42))),
        ("ratio=2.5", ("ratio", json!(2.5))),
    ];

    for (input, (expected_key, expected_val)) in test_cases {
        let (key, val) = parse_key_val(input).unwrap();
        assert_eq!(key, expected_key);
        assert_eq!(val, expected_val);
    }
}

#[test]
fn test_parse_key_val_edge_cases() {
    // Value with equals signs
    let (key, val) = parse_key_val("url=https://example.com?foo=bar&baz=qux").unwrap();
    assert_eq!(key, "url");
    assert_eq!(val, json!("https://example.com?foo=bar&baz=qux"));

    // Empty value
    let (key, val) = parse_key_val("empty=").unwrap();
    assert_eq!(key, "empty");
    assert_eq!(val, json!(true)); // Empty value is treated as boolean true

    // Invalid format (no equals)
    assert!(parse_key_val("invalid").is_err());
}

// ===== Validation Tests =====

#[test]
fn test_validate_params_comprehensive() {
    let specs = vec![
        ParameterSpec {
            name: "project_name".to_string(),
            param_type: ParameterType::String,
            required: true,
            default_value: None,
            description: "Project name".to_string(),
            validation_pattern: Some(r"^[a-z][a-z0-9-]*$".to_string()),
        },
        ParameterSpec {
            name: "has_tests".to_string(),
            param_type: ParameterType::Boolean,
            required: false,
            default_value: Some("true".to_string()),
            description: "Include tests".to_string(),
            validation_pattern: None,
        },
        ParameterSpec {
            name: "version".to_string(),
            param_type: ParameterType::String,
            required: false,
            default_value: Some("0.1.0".to_string()),
            description: "Version".to_string(),
            validation_pattern: Some(r"^\d+\.\d+\.\d+$".to_string()),
        },
    ];

    // Valid params
    let mut params = serde_json::Map::new();
    params.insert("project_name".to_string(), json!("my-project"));
    params.insert("has_tests".to_string(), json!(false));
    params.insert("version".to_string(), json!("1.0.0"));
    assert!(validate_params(&specs, &params).is_ok());

    // Note: validate_params in cli/args.rs only validates types and required fields
    // Pattern validation happens in the template service, not here

    // Invalid type test - providing number for string field
    let mut params = serde_json::Map::new();
    params.insert("project_name".to_string(), json!(123)); // Number instead of string
    let result = validate_params(&specs, &params);
    assert!(result.is_err());

    // Missing required parameter
    let mut params = serde_json::Map::new();
    params.insert("has_tests".to_string(), json!(true));
    params.insert("version".to_string(), json!("1.0.0"));
    // Missing required project_name
    let result = validate_params(&specs, &params);
    assert!(result.is_err());
}

// ===== Environment Variable Tests =====

#[test]
fn test_expand_env_vars_complex() {
    std::env::set_var("TEST_PROJECT", "my-project");
    std::env::set_var("TEST_VERSION", "1.0.0");
    std::env::set_var("TEST_AUTHOR", "Test Author");

    let template = r#"
Project: ${TEST_PROJECT}
Version: ${TEST_VERSION}
Author: ${TEST_AUTHOR}
Missing: ${NONEXISTENT_VAR}
Nested: ${TEST_PROJECT}-${TEST_VERSION}
"#;

    let expanded = expand_env_vars(template);
    assert!(expanded.contains("Project: my-project"));
    assert!(expanded.contains("Version: 1.0.0"));
    assert!(expanded.contains("Author: Test Author"));
    assert!(expanded.contains("Missing: ${NONEXISTENT_VAR}"));
    assert!(expanded.contains("Nested: my-project-1.0.0"));

    // Cleanup
    std::env::remove_var("TEST_PROJECT");
    std::env::remove_var("TEST_VERSION");
    std::env::remove_var("TEST_AUTHOR");
}

// ===== Error Handling Tests =====

#[test]
fn test_cli_error_scenarios() {
    // Unknown command
    let args = vec!["paiml-mcp-agent-toolkit", "unknown-command"];
    assert!(Cli::try_parse_from(&args).is_err());

    // Invalid option
    let args = vec!["paiml-mcp-agent-toolkit", "list", "--invalid-option"];
    assert!(Cli::try_parse_from(&args).is_err());

    // Missing required argument
    let args = vec!["paiml-mcp-agent-toolkit", "search"];
    assert!(Cli::try_parse_from(&args).is_err());

    // Invalid enum value
    let args = vec!["paiml-mcp-agent-toolkit", "list", "--format", "invalid"];
    assert!(Cli::try_parse_from(&args).is_err());
}

// ===== Help and Version Tests =====

#[test]
fn test_help_flags() {
    let args = vec!["paiml-mcp-agent-toolkit", "--help"];
    let result = Cli::try_parse_from(&args);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.kind(), clap::error::ErrorKind::DisplayHelp);
    }

    // Subcommand help
    let args = vec!["paiml-mcp-agent-toolkit", "generate", "--help"];
    let result = Cli::try_parse_from(&args);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.kind(), clap::error::ErrorKind::DisplayHelp);
    }
}

#[test]
fn test_version_flag() {
    let args = vec!["paiml-mcp-agent-toolkit", "--version"];
    let result = Cli::try_parse_from(&args);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.kind(), clap::error::ErrorKind::DisplayVersion);
    }
}

// ===== Mode Detection Tests =====

#[test]
fn test_mode_flag() {
    // Force CLI mode
    let args = vec!["paiml-mcp-agent-toolkit", "--mode", "cli", "list"];
    let cli = Cli::try_parse_from(&args).unwrap();
    assert!(matches!(cli.mode, Some(crate::cli::Mode::Cli)));

    // Force MCP mode
    let args = vec!["paiml-mcp-agent-toolkit", "--mode", "mcp", "list"];
    let cli = Cli::try_parse_from(&args).unwrap();
    assert!(matches!(cli.mode, Some(crate::cli::Mode::Mcp)));
}

// ===== Complex Scenario Tests =====

#[test]
fn test_multiple_parameter_types() {
    let args = vec![
        "paiml-mcp-agent-toolkit",
        "generate",
        "makefile",
        "rust/cli",
        "-p",
        "project_name=complex-test",
        "-p",
        "has_tests=true",
        "-p",
        "has_benchmarks=false",
        "-p",
        "rust_version=1.75.0",
        "-p",
        "optimization_level=3",
        "-p",
        "features=serde,tokio,clap",
    ];

    let cli = Cli::try_parse_from(&args).unwrap();
    match cli.command {
        Commands::Generate { params, .. } => {
            let param_map: std::collections::HashMap<String, Value> = params.into_iter().collect();

            assert_eq!(param_map["project_name"], json!("complex-test"));
            assert_eq!(param_map["has_tests"], json!(true));
            assert_eq!(param_map["has_benchmarks"], json!(false));
            assert_eq!(param_map["rust_version"], json!("1.75.0"));
            assert_eq!(param_map["optimization_level"], json!(3));
            assert_eq!(param_map["features"], json!("serde,tokio,clap"));
        }
        _ => panic!("Expected Generate command"),
    }
}

#[test]
fn test_nested_subcommand_parsing() {
    // Test all analyze subcommands
    let subcommands = vec![
        (vec!["analyze", "churn"], "churn"),
        (vec!["analyze", "complexity"], "complexity"),
        (vec!["analyze", "dag"], "dag"),
    ];

    for (subcmd, expected) in subcommands {
        let mut args = vec!["paiml-mcp-agent-toolkit"];
        args.extend(subcmd);

        let cli = Cli::try_parse_from(&args).unwrap();
        match cli.command {
            Commands::Analyze(analyze_cmd) => match (expected, analyze_cmd) {
                ("churn", AnalyzeCommands::Churn { .. }) => {}
                ("complexity", AnalyzeCommands::Complexity { .. }) => {}
                ("dag", AnalyzeCommands::Dag { .. }) => {}
                _ => panic!("Unexpected analyze subcommand"),
            },
            _ => panic!("Expected Analyze command"),
        }
    }
}
