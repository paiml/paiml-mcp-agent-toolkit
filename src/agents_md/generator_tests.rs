// Generator tests extracted from generator.rs for file health (CB-040).
// This file is include!()'d into generator.rs scope.
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents_md::{DocumentMetadata, SectionType};
    use std::fs;
    use tempfile::TempDir;

    // ==================== Generator Creation Tests ====================

    #[test]
    fn test_generator_creation() {
        let generator = AgentsMdGenerator::new();
        assert!(generator.config.include_quality);
        assert!(generator.config.include_security);
        assert_eq!(generator.config.max_commands, 20);
    }

    #[test]
    fn test_generator_default_implementation() {
        let generator = AgentsMdGenerator::default();
        assert!(generator.config.include_quality);
        assert!(generator.config.include_security);
        assert!(generator.config.include_pr_guidelines);
        assert_eq!(generator.config.max_commands, 20);
        assert!(generator.config.include_examples);
        // Templates should be loaded
        assert!(generator.templates.contains_key(&ProjectType::Rust));
        assert!(generator.templates.contains_key(&ProjectType::Generic));
    }

    #[test]
    fn test_config_customization() {
        let config = GeneratorConfig {
            include_quality: false,
            include_security: false,
            include_pr_guidelines: false,
            max_commands: 5,
            include_examples: false,
        };

        let generator = AgentsMdGenerator::with_config(config);
        assert!(!generator.config.include_quality);
        assert!(!generator.config.include_security);
        assert_eq!(generator.config.max_commands, 5);
    }

    #[test]
    fn test_generator_config_default() {
        let config = GeneratorConfig::default();
        assert!(config.include_quality);
        assert!(config.include_security);
        assert!(config.include_pr_guidelines);
        assert_eq!(config.max_commands, 20);
        assert!(config.include_examples);
    }

    // ==================== Template Tests ====================

    #[test]
    fn test_rust_template_loaded() {
        let generator = AgentsMdGenerator::new();
        let template = generator.templates.get(&ProjectType::Rust);
        assert!(template.is_some());

        let template = template.unwrap();
        assert!(!template.sections.is_empty());
        assert!(!template.default_commands.is_empty());

        // Verify default commands for Rust
        assert!(template
            .default_commands
            .iter()
            .any(|c| c.command.contains("cargo build")));
        assert!(template
            .default_commands
            .iter()
            .any(|c| c.command.contains("cargo test")));
    }

    #[test]
    fn test_generic_template_loaded() {
        let generator = AgentsMdGenerator::new();
        let template = generator.templates.get(&ProjectType::Generic);
        assert!(template.is_some());

        let template = template.unwrap();
        assert!(!template.sections.is_empty());
        // Generic template has no default commands
        assert!(template.default_commands.is_empty());
    }

    // ==================== Generate From Analysis Tests ====================

    #[test]
    fn test_generate_from_analysis() {
        let generator = AgentsMdGenerator::new();

        let analysis = PmatAnalysis {
            project_name: "Test Project".to_string(),
            description: "A test project".to_string(),
            project_type: ProjectType::Rust,
            commands: vec![Command {
                name: "Build".to_string(),
                command: "cargo build".to_string(),
                working_dir: None,
                env: vec![],
                timeout: Some(60),
                safe: true,
            }],
            quality_metrics: QualityMetrics {
                avg_complexity: 8.5,
                test_coverage: 85.0,
                satd_count: 0,
                grade: "A".to_string(),
            },
            dependencies: vec![],
        };

        let result = generator.generate_from_analysis(&analysis).unwrap();
        assert!(result.contains("# AGENTS.md"));
        assert!(result.contains("Test Project"));
        assert!(result.contains("cargo build"));
        assert!(result.contains("## Testing Instructions"));
    }

    #[test]
    fn test_generate_from_analysis_with_all_sections() {
        let generator = AgentsMdGenerator::new();

        let analysis = PmatAnalysis {
            project_name: "Full Project".to_string(),
            description: "A comprehensive project".to_string(),
            project_type: ProjectType::Generic,
            commands: vec![
                Command {
                    name: "Build".to_string(),
                    command: "make build".to_string(),
                    working_dir: None,
                    env: vec![],
                    timeout: Some(60),
                    safe: true,
                },
                Command {
                    name: "Test".to_string(),
                    command: "make test".to_string(),
                    working_dir: None,
                    env: vec![],
                    timeout: Some(120),
                    safe: true,
                },
            ],
            quality_metrics: QualityMetrics {
                avg_complexity: 5.0,
                test_coverage: 90.0,
                satd_count: 2,
                grade: "B".to_string(),
            },
            dependencies: vec!["dep1".to_string(), "dep2".to_string()],
        };

        let result = generator.generate_from_analysis(&analysis).unwrap();

        // Verify all sections are present
        assert!(result.contains("## Project Overview"));
        assert!(result.contains("## Development Setup"));
        assert!(result.contains("## Testing Instructions"));
        assert!(result.contains("## Code Style"));
        assert!(result.contains("## PR Guidelines"));
        assert!(result.contains("## Security Considerations"));

        // Verify content from analysis
        assert!(result.contains("Full Project"));
        assert!(result.contains("A comprehensive project"));
        assert!(result.contains("make build"));
        assert!(result.contains("make test"));
        assert!(result.contains("90%")); // Coverage
        assert!(result.contains("B")); // Grade
    }

    #[test]
    fn test_generate_from_analysis_without_optional_sections() {
        let config = GeneratorConfig {
            include_quality: false,
            include_security: false,
            include_pr_guidelines: false,
            max_commands: 20,
            include_examples: true,
        };
        let generator = AgentsMdGenerator::with_config(config);

        let analysis = PmatAnalysis {
            project_name: "Minimal Project".to_string(),
            description: "A minimal project".to_string(),
            project_type: ProjectType::Rust,
            commands: vec![],
            quality_metrics: QualityMetrics {
                avg_complexity: 10.0,
                test_coverage: 80.0,
                satd_count: 0,
                grade: "A".to_string(),
            },
            dependencies: vec![],
        };

        let result = generator.generate_from_analysis(&analysis).unwrap();

        // Verify required sections are present
        assert!(result.contains("## Project Overview"));
        assert!(result.contains("## Development Setup"));
        assert!(result.contains("## Testing Instructions"));

        // Verify optional sections are NOT present
        assert!(!result.contains("## Code Style"));
        assert!(!result.contains("## PR Guidelines"));
        assert!(!result.contains("## Security Considerations"));
    }

    #[test]
    fn test_generate_from_analysis_with_many_commands() {
        let generator = AgentsMdGenerator::new();

        // Create 10 commands
        let commands: Vec<Command> = (0..10)
            .map(|i| Command {
                name: format!("Command {}", i),
                command: format!("cmd{}", i),
                working_dir: None,
                env: vec![],
                timeout: Some(60),
                safe: true,
            })
            .collect();

        let analysis = PmatAnalysis {
            project_name: "Many Commands Project".to_string(),
            description: "Project with many commands".to_string(),
            project_type: ProjectType::Rust,
            commands,
            quality_metrics: QualityMetrics {
                avg_complexity: 10.0,
                test_coverage: 80.0,
                satd_count: 0,
                grade: "A".to_string(),
            },
            dependencies: vec![],
        };

        let result = generator.generate_from_analysis(&analysis).unwrap();

        // Only first 5 commands should be in dev setup
        assert!(result.contains("cmd0"));
        assert!(result.contains("cmd4"));
        // cmd5 and beyond should not be in the dev setup section
        // (The limit is 5 in generate_dev_setup)
    }

    #[test]
    fn test_generate_from_analysis_empty_commands() {
        let generator = AgentsMdGenerator::new();

        let analysis = PmatAnalysis {
            project_name: "Empty Commands Project".to_string(),
            description: "Project with no commands".to_string(),
            project_type: ProjectType::Rust,
            commands: vec![],
            quality_metrics: QualityMetrics {
                avg_complexity: 10.0,
                test_coverage: 80.0,
                satd_count: 0,
                grade: "A".to_string(),
            },
            dependencies: vec![],
        };

        let result = generator.generate_from_analysis(&analysis).unwrap();

        // Should still generate valid output
        assert!(result.contains("# AGENTS.md"));
        assert!(result.contains("## Development Setup"));
    }

    // ==================== Project Type Detection Tests ====================

    #[test]
    fn test_detect_project_type_rust() {
        let generator = AgentsMdGenerator::new();
        let temp = TempDir::new().unwrap();

        fs::write(temp.path().join("Cargo.toml"), "[package]").unwrap();
        assert_eq!(
            generator.detect_project_type(temp.path()).unwrap(),
            ProjectType::Rust
        );
    }

    #[test]
    fn test_detect_project_type_javascript() {
        let generator = AgentsMdGenerator::new();
        let temp = TempDir::new().unwrap();

        fs::write(temp.path().join("package.json"), "{}").unwrap();
        assert_eq!(
            generator.detect_project_type(temp.path()).unwrap(),
            ProjectType::JavaScript
        );
    }

    #[test]
    fn test_detect_project_type_python_setup_py() {
        let generator = AgentsMdGenerator::new();
        let temp = TempDir::new().unwrap();

        fs::write(temp.path().join("setup.py"), "from setuptools import setup").unwrap();
        assert_eq!(
            generator.detect_project_type(temp.path()).unwrap(),
            ProjectType::Python
        );
    }

    #[test]
    fn test_detect_project_type_python_pyproject() {
        let generator = AgentsMdGenerator::new();
        let temp = TempDir::new().unwrap();

        fs::write(temp.path().join("pyproject.toml"), "[project]").unwrap();
        assert_eq!(
            generator.detect_project_type(temp.path()).unwrap(),
            ProjectType::Python
        );
    }

    #[test]
    fn test_detect_project_type_go() {
        let generator = AgentsMdGenerator::new();
        let temp = TempDir::new().unwrap();

        fs::write(temp.path().join("go.mod"), "module example.com/test").unwrap();
        assert_eq!(
            generator.detect_project_type(temp.path()).unwrap(),
            ProjectType::Go
        );
    }

    #[test]
    fn test_detect_project_type_java_maven() {
        let generator = AgentsMdGenerator::new();
        let temp = TempDir::new().unwrap();

        fs::write(temp.path().join("pom.xml"), "<project></project>").unwrap();
        assert_eq!(
            generator.detect_project_type(temp.path()).unwrap(),
            ProjectType::Java
        );
    }

    #[test]
    fn test_detect_project_type_java_gradle() {
        let generator = AgentsMdGenerator::new();
        let temp = TempDir::new().unwrap();

        fs::write(temp.path().join("build.gradle"), "apply plugin: 'java'").unwrap();
        assert_eq!(
            generator.detect_project_type(temp.path()).unwrap(),
            ProjectType::Java
        );
    }

    #[test]
    fn test_detect_project_type_generic() {
        let generator = AgentsMdGenerator::new();
        let temp = TempDir::new().unwrap();

        // Empty directory should be generic
        assert_eq!(
            generator.detect_project_type(temp.path()).unwrap(),
            ProjectType::Generic
        );
    }

    #[test]
    fn test_detect_project_type_priority() {
        // When multiple project files exist, first match wins
        let generator = AgentsMdGenerator::new();
        let temp = TempDir::new().unwrap();

        // Rust has priority (checked first)
        fs::write(temp.path().join("Cargo.toml"), "[package]").unwrap();
        fs::write(temp.path().join("package.json"), "{}").unwrap();
        assert_eq!(
            generator.detect_project_type(temp.path()).unwrap(),
            ProjectType::Rust
        );
    }

    // ==================== Command Detection Tests ====================

    #[test]
    fn test_detect_commands_makefile() {
        let generator = AgentsMdGenerator::new();
        let temp = TempDir::new().unwrap();

        fs::write(
            temp.path().join("Makefile"),
            "build:\n\techo build\ntest:\n\techo test",
        )
        .unwrap();

        let commands = generator.detect_commands(temp.path()).unwrap();
        assert_eq!(commands.len(), 2);

        assert!(commands.iter().any(|c| c.name == "Build"));
        assert!(commands.iter().any(|c| c.name == "Test"));
        assert!(commands.iter().any(|c| c.command == "make build"));
        assert!(commands.iter().any(|c| c.command == "make test"));

        // Verify all commands are marked safe
        for cmd in &commands {
            assert!(cmd.safe);
        }
    }

    #[test]
    fn test_detect_commands_package_json() {
        let generator = AgentsMdGenerator::new();
        let temp = TempDir::new().unwrap();

        fs::write(
            temp.path().join("package.json"),
            r#"{"scripts": {"build": "webpack"}}"#,
        )
        .unwrap();

        let commands = generator.detect_commands(temp.path()).unwrap();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].name, "Install");
        assert_eq!(commands[0].command, "npm install");
        assert_eq!(commands[0].timeout, Some(300));
    }

    #[test]
    fn test_detect_commands_makefile_and_package_json() {
        let generator = AgentsMdGenerator::new();
        let temp = TempDir::new().unwrap();

        fs::write(temp.path().join("Makefile"), "build:\n\techo build").unwrap();
        fs::write(temp.path().join("package.json"), "{}").unwrap();

        let commands = generator.detect_commands(temp.path()).unwrap();
        // 2 from Makefile + 1 from package.json
        assert_eq!(commands.len(), 3);
    }

    #[test]
    fn test_detect_commands_empty_directory() {
        let generator = AgentsMdGenerator::new();
        let temp = TempDir::new().unwrap();

        let commands = generator.detect_commands(temp.path()).unwrap();
        assert!(commands.is_empty());
    }

    // ==================== Generate From Project Tests ====================

    #[test]
    fn test_generate_from_project_rust() {
        let generator = AgentsMdGenerator::new();
        let temp = TempDir::new().unwrap();

        fs::write(temp.path().join("Cargo.toml"), "[package]").unwrap();
        fs::write(temp.path().join("Makefile"), "build:\n\techo").unwrap();

        let project = ProjectInfo {
            root: temp.path().to_path_buf(),
            name: "My Rust Project".to_string(),
            version: "1.0.0".to_string(),
            description: "A Rust project".to_string(),
            readme: None,
        };

        let result = generator.generate_from_project(&project).unwrap();
        assert!(result.contains("# AGENTS.md"));
        assert!(result.contains("My Rust Project"));
        assert!(result.contains("A Rust project"));
        // Should detect Makefile commands
        assert!(result.contains("make build"));
    }

    #[test]
    fn test_generate_from_project_javascript() {
        let generator = AgentsMdGenerator::new();
        let temp = TempDir::new().unwrap();

        fs::write(temp.path().join("package.json"), "{}").unwrap();

        let project = ProjectInfo {
            root: temp.path().to_path_buf(),
            name: "My JS Project".to_string(),
            version: "2.0.0".to_string(),
            description: "A JavaScript project".to_string(),
            readme: Some("# README\nThis is a test".to_string()),
        };

        let result = generator.generate_from_project(&project).unwrap();
        assert!(result.contains("My JS Project"));
        assert!(result.contains("npm install"));
    }

    // ==================== Update Existing Tests ====================

    #[test]
    fn test_update_existing() {
        let generator = AgentsMdGenerator::new();

        let current = r#"# AGENTS.md

## Project Overview
Test Project

## Development Setup
```bash
cargo build
```
"#;

        let updates = Updates {
            new_commands: vec![Command {
                name: "Test".to_string(),
                command: "cargo test".to_string(),
                working_dir: None,
                env: vec![],
                timeout: Some(120),
                safe: true,
            }],
            quality_rules: Some(QualityRules {
                max_complexity: Some(10),
                min_coverage: Some(80.0),
                satd_allowed: false,
                custom_checks: vec![],
            }),
            new_sections: vec![],
        };

        let result = generator.update_existing(current, updates).unwrap();
        assert!(result.contains("# AGENTS.md"));
        assert!(result.contains("Project Overview"));
    }

    #[test]
    fn test_update_existing_with_new_sections() {
        let generator = AgentsMdGenerator::new();

        let current = r#"# AGENTS.md

## Project Overview
Test Project
"#;

        let updates = Updates {
            new_commands: vec![],
            quality_rules: None,
            new_sections: vec![Section {
                section_type: SectionType::Security,
                title: "Security Guidelines".to_string(),
                content: "Always validate input".to_string(),
                subsections: vec![],
            }],
        };

        let result = generator.update_existing(current, updates).unwrap();
        assert!(result.contains("Security Guidelines"));
        assert!(result.contains("Always validate input"));
    }

    #[test]
    fn test_update_existing_no_duplicate_commands() {
        let generator = AgentsMdGenerator::new();

        let current = r#"# AGENTS.md

## Project Overview
Test Project

## Development Setup
```bash
cargo build
```
"#;

        let updates = Updates {
            new_commands: vec![
                Command {
                    name: "Command from Development Setup".to_string(), // Same name as extracted
                    command: "cargo build".to_string(),
                    working_dir: None,
                    env: vec![],
                    timeout: Some(60),
                    safe: true,
                },
                Command {
                    name: "New Command".to_string(),
                    command: "cargo test".to_string(),
                    working_dir: None,
                    env: vec![],
                    timeout: Some(120),
                    safe: true,
                },
            ],
            quality_rules: None,
            new_sections: vec![],
        };

        // Should not fail and should not create duplicates
        let result = generator.update_existing(current, updates);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_existing_no_duplicate_sections() {
        let generator = AgentsMdGenerator::new();

        let current = r#"# AGENTS.md

## Project Overview
Test Project
"#;

        let updates = Updates {
            new_commands: vec![],
            quality_rules: None,
            new_sections: vec![
                Section {
                    section_type: SectionType::Overview,
                    title: "Project Overview".to_string(), // Same as existing
                    content: "New content".to_string(),
                    subsections: vec![],
                },
                Section {
                    section_type: SectionType::Testing,
                    title: "Testing".to_string(),
                    content: "Run tests".to_string(),
                    subsections: vec![],
                },
            ],
        };

        let result = generator.update_existing(current, updates).unwrap();
        // Should only contain one "Project Overview" section
        let overview_count = result.matches("Project Overview").count();
        assert_eq!(overview_count, 1);
    }

    // ==================== Format Document Tests ====================

    #[test]
    fn test_format_document() {
        let generator = AgentsMdGenerator::new();

        let doc = AgentsMdDocument {
            metadata: DocumentMetadata {
                path: PathBuf::from("AGENTS.md"),
                modified: std::time::SystemTime::now(),
                version: Some("1.0.0".to_string()),
                project: Some("Test".to_string()),
            },
            sections: vec![Section {
                section_type: SectionType::Overview,
                title: "Project Overview".to_string(),
                content: "Test project".to_string(),
                subsections: vec![],
            }],
            commands: vec![],
            guidelines: vec![],
            quality_rules: None,
        };

        let result = generator.format_document(&doc).unwrap();
        assert!(result.contains("# AGENTS.md"));
        assert!(result.contains("## Project Overview"));
        assert!(result.contains("Test project"));
    }

    #[test]
    fn test_format_document_with_nested_subsections() {
        let generator = AgentsMdGenerator::new();

        let doc = AgentsMdDocument {
            metadata: DocumentMetadata {
                path: PathBuf::from("AGENTS.md"),
                modified: std::time::SystemTime::now(),
                version: None,
                project: None,
            },
            sections: vec![Section {
                section_type: SectionType::DevEnvironment,
                title: "Development".to_string(),
                content: "Development content".to_string(),
                subsections: vec![
                    Section {
                        section_type: SectionType::Custom("Prerequisites".to_string()),
                        title: "Prerequisites".to_string(),
                        content: "Install Rust".to_string(),
                        subsections: vec![Section {
                            section_type: SectionType::Custom("Rust Version".to_string()),
                            title: "Rust Version".to_string(),
                            content: "Use Rust 1.70+".to_string(),
                            subsections: vec![],
                        }],
                    },
                    Section {
                        section_type: SectionType::Custom("IDE Setup".to_string()),
                        title: "IDE Setup".to_string(),
                        content: "Use VSCode".to_string(),
                        subsections: vec![],
                    },
                ],
            }],
            commands: vec![],
            guidelines: vec![],
            quality_rules: None,
        };

        let result = generator.format_document(&doc).unwrap();

        // Verify heading levels
        assert!(result.contains("## Development"));
        assert!(result.contains("### Prerequisites"));
        assert!(result.contains("#### Rust Version"));
        assert!(result.contains("### IDE Setup"));

        // Verify content
        assert!(result.contains("Install Rust"));
        assert!(result.contains("Use Rust 1.70+"));
        assert!(result.contains("Use VSCode"));
    }

    #[test]
    fn test_format_document_multiple_sections() {
        let generator = AgentsMdGenerator::new();

        let doc = AgentsMdDocument {
            metadata: DocumentMetadata {
                path: PathBuf::from("AGENTS.md"),
                modified: std::time::SystemTime::now(),
                version: None,
                project: None,
            },
            sections: vec![
                Section {
                    section_type: SectionType::Overview,
                    title: "Overview".to_string(),
                    content: "Project overview".to_string(),
                    subsections: vec![],
                },
                Section {
                    section_type: SectionType::Testing,
                    title: "Testing".to_string(),
                    content: "Run cargo test".to_string(),
                    subsections: vec![],
                },
                Section {
                    section_type: SectionType::Security,
                    title: "Security".to_string(),
                    content: "Security guidelines".to_string(),
                    subsections: vec![],
                },
            ],
            commands: vec![],
            guidelines: vec![],
            quality_rules: None,
        };

        let result = generator.format_document(&doc).unwrap();

        assert!(result.contains("## Overview"));
        assert!(result.contains("## Testing"));
        assert!(result.contains("## Security"));
    }

    #[test]
    fn test_format_document_empty_sections() {
        let generator = AgentsMdGenerator::new();

        let doc = AgentsMdDocument {
            metadata: DocumentMetadata {
                path: PathBuf::from("AGENTS.md"),
                modified: std::time::SystemTime::now(),
                version: None,
                project: None,
            },
            sections: vec![],
            commands: vec![],
            guidelines: vec![],
            quality_rules: None,
        };

        let result = generator.format_document(&doc).unwrap();
        assert!(result.contains("# AGENTS.md"));
        // Should be minimal with just the header
    }

    // ==================== Quality Metrics Tests ====================

    #[test]
    fn test_quality_metrics_in_output() {
        let generator = AgentsMdGenerator::new();

        let analysis = PmatAnalysis {
            project_name: "Metrics Test".to_string(),
            description: "Testing metrics".to_string(),
            project_type: ProjectType::Rust,
            commands: vec![],
            quality_metrics: QualityMetrics {
                avg_complexity: 15.5,
                test_coverage: 95.5,
                satd_count: 5,
                grade: "A+".to_string(),
            },
            dependencies: vec![],
        };

        let result = generator.generate_from_analysis(&analysis).unwrap();

        // Coverage percentage should be in testing section
        assert!(result.contains("95%"));
        // Grade should be in code style section
        assert!(result.contains("A+"));
    }

    // ==================== Project Type Enum Tests ====================

    #[test]
    fn test_project_type_equality() {
        assert_eq!(ProjectType::Rust, ProjectType::Rust);
        assert_ne!(ProjectType::Rust, ProjectType::Python);
        assert_ne!(ProjectType::JavaScript, ProjectType::Go);
    }

    #[test]
    fn test_project_type_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(ProjectType::Rust);
        set.insert(ProjectType::Python);
        set.insert(ProjectType::JavaScript);

        assert!(set.contains(&ProjectType::Rust));
        assert!(set.contains(&ProjectType::Python));
        assert!(!set.contains(&ProjectType::Go));
    }

    // ==================== Template Section Tests ====================

    #[test]
    fn test_template_section_with_variables() {
        let section = TemplateSection {
            title: "Test Section".to_string(),
            content: "{project_name}\n{description}".to_string(),
            variables: vec!["project_name".to_string(), "description".to_string()],
        };

        assert_eq!(section.variables.len(), 2);
        assert!(section.content.contains("{project_name}"));
        assert!(section.content.contains("{description}"));
    }

    #[test]
    fn test_template_creation() {
        let template = Template {
            sections: vec![TemplateSection {
                title: "Overview".to_string(),
                content: "Content".to_string(),
                variables: vec![],
            }],
            default_commands: vec![Command {
                name: "Build".to_string(),
                command: "make".to_string(),
                working_dir: None,
                env: vec![],
                timeout: Some(60),
                safe: true,
            }],
        };

        assert_eq!(template.sections.len(), 1);
        assert_eq!(template.default_commands.len(), 1);
    }

    // ==================== Command Structure Tests ====================

    #[test]
    fn test_command_with_environment() {
        let cmd = Command {
            name: "Build with ENV".to_string(),
            command: "cargo build".to_string(),
            working_dir: Some(PathBuf::from("/project")),
            env: vec![
                ("RUST_LOG".to_string(), "debug".to_string()),
                ("CARGO_TERM_COLOR".to_string(), "always".to_string()),
            ],
            timeout: Some(120),
            safe: true,
        };

        assert_eq!(cmd.env.len(), 2);
        assert_eq!(cmd.working_dir, Some(PathBuf::from("/project")));
    }

    // ==================== Updates Structure Tests ====================

    #[test]
    fn test_updates_creation() {
        let updates = Updates {
            new_commands: vec![Command {
                name: "New Cmd".to_string(),
                command: "new command".to_string(),
                working_dir: None,
                env: vec![],
                timeout: None,
                safe: true,
            }],
            quality_rules: Some(QualityRules {
                max_complexity: Some(15),
                min_coverage: Some(75.0),
                satd_allowed: true,
                custom_checks: vec!["custom_check".to_string()],
            }),
            new_sections: vec![Section {
                section_type: SectionType::Custom("New Section".to_string()),
                title: "New Section".to_string(),
                content: "Content".to_string(),
                subsections: vec![],
            }],
        };

        assert_eq!(updates.new_commands.len(), 1);
        assert!(updates.quality_rules.is_some());
        assert_eq!(updates.new_sections.len(), 1);
    }

    // ==================== ProjectInfo Structure Tests ====================

    #[test]
    fn test_project_info_creation() {
        let project = ProjectInfo {
            root: PathBuf::from("/path/to/project"),
            name: "Test Project".to_string(),
            version: "1.2.3".to_string(),
            description: "A test project".to_string(),
            readme: Some("# README\nContent".to_string()),
        };

        assert_eq!(project.root, PathBuf::from("/path/to/project"));
        assert_eq!(project.version, "1.2.3");
        assert!(project.readme.is_some());
    }

    #[test]
    fn test_project_info_without_readme() {
        let project = ProjectInfo {
            root: PathBuf::from("/project"),
            name: "No README".to_string(),
            version: "0.1.0".to_string(),
            description: "Project without readme".to_string(),
            readme: None,
        };

        assert!(project.readme.is_none());
    }

    // ==================== PmatAnalysis Structure Tests ====================

    #[test]
    fn test_pmat_analysis_creation() {
        let analysis = PmatAnalysis {
            project_name: "Analysis Test".to_string(),
            description: "Testing analysis".to_string(),
            project_type: ProjectType::Go,
            commands: vec![],
            quality_metrics: QualityMetrics {
                avg_complexity: 5.0,
                test_coverage: 100.0,
                satd_count: 0,
                grade: "A".to_string(),
            },
            dependencies: vec!["dep1".to_string()],
        };

        assert_eq!(analysis.project_type, ProjectType::Go);
        assert_eq!(analysis.dependencies.len(), 1);
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_generate_with_special_characters_in_name() {
        let generator = AgentsMdGenerator::new();

        let analysis = PmatAnalysis {
            project_name: "Test & <Project> \"Name\"".to_string(),
            description: "Description with 'quotes' and special chars: @#$%".to_string(),
            project_type: ProjectType::Rust,
            commands: vec![],
            quality_metrics: QualityMetrics {
                avg_complexity: 10.0,
                test_coverage: 80.0,
                satd_count: 0,
                grade: "B".to_string(),
            },
            dependencies: vec![],
        };

        let result = generator.generate_from_analysis(&analysis).unwrap();
        assert!(result.contains("Test & <Project> \"Name\""));
        assert!(result.contains("@#$%"));
    }

    #[test]
    fn test_generate_with_empty_description() {
        let generator = AgentsMdGenerator::new();

        let analysis = PmatAnalysis {
            project_name: "Empty Description".to_string(),
            description: "".to_string(),
            project_type: ProjectType::Rust,
            commands: vec![],
            quality_metrics: QualityMetrics {
                avg_complexity: 10.0,
                test_coverage: 80.0,
                satd_count: 0,
                grade: "A".to_string(),
            },
            dependencies: vec![],
        };

        let result = generator.generate_from_analysis(&analysis).unwrap();
        assert!(result.contains("Empty Description"));
        assert!(result.contains("# AGENTS.md"));
    }

    #[test]
    fn test_generate_with_unicode_content() {
        let generator = AgentsMdGenerator::new();

        let analysis = PmatAnalysis {
            project_name: "Unicode Project".to_string(),
            description: "A project with emojis and unicode chars".to_string(),
            project_type: ProjectType::Rust,
            commands: vec![],
            quality_metrics: QualityMetrics {
                avg_complexity: 10.0,
                test_coverage: 80.0,
                satd_count: 0,
                grade: "A".to_string(),
            },
            dependencies: vec![],
        };

        let result = generator.generate_from_analysis(&analysis);
        assert!(result.is_ok());
    }

    // ==================== Integration-Style Tests ====================

    #[test]
    fn test_full_workflow_rust_project() {
        let generator = AgentsMdGenerator::new();
        let temp = TempDir::new().unwrap();

        // Set up a Rust project
        fs::write(temp.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
        fs::write(
            temp.path().join("Makefile"),
            "build:\n\tcargo build\ntest:\n\tcargo test",
        )
        .unwrap();

        let project = ProjectInfo {
            root: temp.path().to_path_buf(),
            name: "Full Workflow Test".to_string(),
            version: "1.0.0".to_string(),
            description: "Testing full workflow".to_string(),
            readme: None,
        };

        // Generate initial document
        let result = generator.generate_from_project(&project).unwrap();
        assert!(result.contains("# AGENTS.md"));
        assert!(result.contains("Full Workflow Test"));
        assert!(result.contains("make build"));
        assert!(result.contains("make test"));
    }

    // ==================== Clone and Debug Tests ====================

    #[test]
    fn test_generator_config_clone() {
        let config = GeneratorConfig {
            include_quality: true,
            include_security: false,
            include_pr_guidelines: true,
            max_commands: 10,
            include_examples: false,
        };

        let cloned = config.clone();
        assert_eq!(config.include_quality, cloned.include_quality);
        assert_eq!(config.max_commands, cloned.max_commands);
    }

    #[test]
    fn test_generator_config_debug() {
        let config = GeneratorConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("GeneratorConfig"));
        assert!(debug_str.contains("include_quality"));
    }

    #[test]
    fn test_project_type_debug() {
        let pt = ProjectType::Rust;
        let debug_str = format!("{:?}", pt);
        assert!(debug_str.contains("Rust"));
    }

    #[test]
    fn test_project_type_clone() {
        let pt = ProjectType::JavaScript;
        let cloned = pt.clone();
        assert_eq!(pt, cloned);
    }

    #[test]
    fn test_template_clone() {
        let template = Template {
            sections: vec![TemplateSection {
                title: "Test".to_string(),
                content: "Content".to_string(),
                variables: vec!["var1".to_string()],
            }],
            default_commands: vec![],
        };

        let cloned = template.clone();
        assert_eq!(template.sections.len(), cloned.sections.len());
        assert_eq!(template.sections[0].title, cloned.sections[0].title);
    }

    #[test]
    fn test_template_debug() {
        let template = Template {
            sections: vec![],
            default_commands: vec![],
        };
        let debug_str = format!("{:?}", template);
        assert!(debug_str.contains("Template"));
    }

    #[test]
    fn test_template_section_clone() {
        let section = TemplateSection {
            title: "Title".to_string(),
            content: "Content".to_string(),
            variables: vec!["v1".to_string(), "v2".to_string()],
        };

        let cloned = section.clone();
        assert_eq!(section.title, cloned.title);
        assert_eq!(section.variables, cloned.variables);
    }

    #[test]
    fn test_template_section_debug() {
        let section = TemplateSection {
            title: "Test".to_string(),
            content: "Content".to_string(),
            variables: vec![],
        };
        let debug_str = format!("{:?}", section);
        assert!(debug_str.contains("TemplateSection"));
    }

    #[test]
    fn test_quality_metrics_clone() {
        let metrics = QualityMetrics {
            avg_complexity: 5.5,
            test_coverage: 90.0,
            satd_count: 2,
            grade: "A".to_string(),
        };

        let cloned = metrics.clone();
        assert_eq!(metrics.avg_complexity, cloned.avg_complexity);
        assert_eq!(metrics.test_coverage, cloned.test_coverage);
        assert_eq!(metrics.satd_count, cloned.satd_count);
        assert_eq!(metrics.grade, cloned.grade);
    }

    #[test]
    fn test_quality_metrics_debug() {
        let metrics = QualityMetrics {
            avg_complexity: 10.0,
            test_coverage: 80.0,
            satd_count: 0,
            grade: "B".to_string(),
        };
        let debug_str = format!("{:?}", metrics);
        assert!(debug_str.contains("QualityMetrics"));
        assert!(debug_str.contains("avg_complexity"));
    }
}
