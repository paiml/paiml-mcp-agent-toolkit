//! AGENTS.md Generator
//!
//! Generates AGENTS.md files from PMAT analysis and project structure.

use super::*;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fmt::Write;

/// AGENTS.md generator with templates
pub struct AgentsMdGenerator {
    /// Templates for different project types
    templates: HashMap<ProjectType, Template>,

    /// Configuration
    config: GeneratorConfig,
}

/// Generator configuration
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    /// Include quality rules section
    pub include_quality: bool,

    /// Include security guidelines
    pub include_security: bool,

    /// Include PR guidelines
    pub include_pr_guidelines: bool,

    /// Maximum commands to include
    pub max_commands: usize,

    /// Include examples
    pub include_examples: bool,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            include_quality: true,
            include_security: true,
            include_pr_guidelines: true,
            max_commands: 20,
            include_examples: true,
        }
    }
}

/// Project types for templates
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProjectType {
    Rust,
    JavaScript,
    Python,
    Go,
    Java,
    Generic,
}

/// Template for generating AGENTS.md
#[derive(Debug, Clone)]
pub struct Template {
    /// Template sections
    pub sections: Vec<TemplateSection>,

    /// Default commands
    pub default_commands: Vec<Command>,
}

/// Template section
#[derive(Debug, Clone)]
pub struct TemplateSection {
    /// Section title
    pub title: String,

    /// Section content template
    pub content: String,

    /// Variables to replace
    pub variables: Vec<String>,
}

/// PMAT analysis for generation
pub struct PmatAnalysis {
    /// Project name
    pub project_name: String,

    /// Project description
    pub description: String,

    /// Project type
    pub project_type: ProjectType,

    /// Detected commands
    pub commands: Vec<Command>,

    /// Quality metrics
    pub quality_metrics: QualityMetrics,

    /// Dependencies
    pub dependencies: Vec<String>,
}

/// Quality metrics from analysis
#[derive(Debug, Clone)]
pub struct QualityMetrics {
    /// Average complexity
    pub avg_complexity: f64,

    /// Test coverage
    pub test_coverage: f64,

    /// SATD count
    pub satd_count: usize,

    /// Code quality grade
    pub grade: String,
}

/// Project info for generation
pub struct ProjectInfo {
    /// Root directory
    pub root: PathBuf,

    /// Project name
    pub name: String,

    /// Version
    pub version: String,

    /// Description
    pub description: String,

    /// README content if exists
    pub readme: Option<String>,
}

/// Updates to apply to existing AGENTS.md
pub struct Updates {
    /// New commands to add
    pub new_commands: Vec<Command>,

    /// Updated quality rules
    pub quality_rules: Option<QualityRules>,

    /// New sections to add
    pub new_sections: Vec<Section>,
}

impl Default for AgentsMdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentsMdGenerator {
    /// Create new generator
    pub fn new() -> Self {
        let mut generator = Self {
            templates: HashMap::new(),
            config: GeneratorConfig::default(),
        };

        generator.load_default_templates();
        generator
    }

    /// Create with custom config
    pub fn with_config(config: GeneratorConfig) -> Self {
        let mut generator = Self {
            templates: HashMap::new(),
            config,
        };

        generator.load_default_templates();
        generator
    }

    /// Load default templates
    fn load_default_templates(&mut self) {
        // Rust template
        self.templates.insert(ProjectType::Rust, Template {
            sections: vec![
                TemplateSection {
                    title: "Project Overview".to_string(),
                    content: "{project_name}\n{description}".to_string(),
                    variables: vec!["project_name".to_string(), "description".to_string()],
                },
                TemplateSection {
                    title: "Development Setup".to_string(),
                    content: "```bash\n# Install dependencies\ncargo build --all\n\n# Run tests\ncargo test --all\n```".to_string(),
                    variables: vec![],
                },
            ],
            default_commands: vec![
                Command {
                    name: "Build".to_string(),
                    command: "cargo build --all".to_string(),
                    working_dir: None,
                    env: vec![],
                    timeout: Some(60),
                    safe: true,
                },
                Command {
                    name: "Test".to_string(),
                    command: "cargo test --all".to_string(),
                    working_dir: None,
                    env: vec![],
                    timeout: Some(120),
                    safe: true,
                },
            ],
        });

        // Add other language templates
        self.templates.insert(
            ProjectType::Generic,
            Template {
                sections: vec![TemplateSection {
                    title: "Project Overview".to_string(),
                    content: "{project_name}\n{description}".to_string(),
                    variables: vec!["project_name".to_string(), "description".to_string()],
                }],
                default_commands: vec![],
            },
        );
    }

    /// Generate from PMAT analysis
    pub fn generate_from_analysis(&self, analysis: &PmatAnalysis) -> Result<String> {
        let _template = self
            .templates
            .get(&analysis.project_type)
            .or_else(|| self.templates.get(&ProjectType::Generic))
            .context("No template found")?;

        let mut output = String::new();
        writeln!(output, "# AGENTS.md")?;
        writeln!(output)?;

        // Generate overview
        writeln!(output, "## Project Overview")?;
        writeln!(
            output,
            "{}\n{}",
            analysis.project_name, analysis.description
        )?;
        writeln!(output)?;

        // Generate development setup
        self.generate_dev_setup(&mut output, analysis)?;

        // Generate testing section
        self.generate_testing(&mut output, analysis)?;

        // Generate code style
        if self.config.include_quality {
            self.generate_code_style(&mut output, analysis)?;
        }

        // Generate PR guidelines
        if self.config.include_pr_guidelines {
            self.generate_pr_guidelines(&mut output)?;
        }

        // Generate security
        if self.config.include_security {
            self.generate_security(&mut output)?;
        }

        Ok(output)
    }

    /// Generate from project structure
    pub fn generate_from_project(&self, project: &ProjectInfo) -> Result<String> {
        // Create basic analysis from project info
        let analysis = PmatAnalysis {
            project_name: project.name.clone(),
            description: project.description.clone(),
            project_type: self.detect_project_type(&project.root)?,
            commands: self.detect_commands(&project.root)?,
            quality_metrics: QualityMetrics {
                avg_complexity: 10.0,
                test_coverage: 80.0,
                satd_count: 0,
                grade: "A".to_string(),
            },
            dependencies: vec![],
        };

        self.generate_from_analysis(&analysis)
    }

    /// Update existing AGENTS.md
    pub fn update_existing(&self, current: &str, updates: Updates) -> Result<String> {
        let parser = super::parser::AgentsMdParser::new();
        let mut doc = parser.parse(current)?;

        // Add new commands
        for cmd in updates.new_commands {
            if !doc.commands.iter().any(|c| c.name == cmd.name) {
                doc.commands.push(cmd);
            }
        }

        // Update quality rules
        if let Some(rules) = updates.quality_rules {
            doc.quality_rules = Some(rules);
        }

        // Add new sections
        for section in updates.new_sections {
            if !doc.sections.iter().any(|s| s.title == section.title) {
                doc.sections.push(section);
            }
        }

        // Regenerate document
        self.format_document(&doc)
    }

    /// Format document back to markdown
    fn format_document(&self, doc: &AgentsMdDocument) -> Result<String> {
        let mut output = String::new();
        writeln!(output, "# AGENTS.md")?;
        writeln!(output)?;

        for section in &doc.sections {
            self.format_section(&mut output, section, 2)?;
        }

        Ok(output)
    }

    /// Format a section
    #[allow(clippy::only_used_in_recursion)]
    fn format_section(&self, output: &mut String, section: &Section, level: usize) -> Result<()> {
        let heading = "#".repeat(level);
        writeln!(output, "{} {}", heading, section.title)?;
        writeln!(output, "{}", section.content)?;
        writeln!(output)?;

        for subsection in &section.subsections {
            self.format_section(output, subsection, level + 1)?;
        }

        Ok(())
    }

    /// Generate development setup section
    fn generate_dev_setup(&self, output: &mut String, analysis: &PmatAnalysis) -> Result<()> {
        writeln!(output, "## Development Setup")?;
        writeln!(output, "```bash")?;

        let commands = &analysis.commands[..analysis.commands.len().min(5)];
        for cmd in commands {
            writeln!(output, "# {}", cmd.name)?;
            writeln!(output, "{}", cmd.command)?;
            writeln!(output)?;
        }

        writeln!(output, "```")?;
        writeln!(output)?;
        Ok(())
    }

    /// Generate testing section
    fn generate_testing(&self, output: &mut String, analysis: &PmatAnalysis) -> Result<()> {
        writeln!(output, "## Testing Instructions")?;
        writeln!(output, "- Run tests before committing")?;
        writeln!(
            output,
            "- Ensure {}%+ coverage maintained",
            analysis.quality_metrics.test_coverage as u32
        )?;
        writeln!(output, "- All functions must have complexity ≤{}", 10)?;
        writeln!(output)?;
        Ok(())
    }

    /// Generate code style section
    fn generate_code_style(&self, output: &mut String, analysis: &PmatAnalysis) -> Result<()> {
        writeln!(output, "## Code Style")?;
        writeln!(output, "- Follow project coding standards")?;
        writeln!(
            output,
            "- Current quality grade: {}",
            analysis.quality_metrics.grade
        )?;
        writeln!(output, "- Zero SATD tolerance")?;
        writeln!(output)?;
        Ok(())
    }

    /// Generate PR guidelines
    fn generate_pr_guidelines(&self, output: &mut String) -> Result<()> {
        writeln!(output, "## PR Guidelines")?;
        writeln!(output, "- Squash commits with conventional format")?;
        writeln!(output, "- Must pass all quality gates")?;
        writeln!(output, "- Include tests for new features")?;
        writeln!(output)?;
        Ok(())
    }

    /// Generate security section
    fn generate_security(&self, output: &mut String) -> Result<()> {
        writeln!(output, "## Security Considerations")?;
        writeln!(output, "- No secrets in code")?;
        writeln!(output, "- Validate all external input")?;
        writeln!(output, "- Use secure defaults")?;
        writeln!(output)?;
        Ok(())
    }

    /// Detect project type from directory
    fn detect_project_type(&self, path: &Path) -> Result<ProjectType> {
        if path.join("Cargo.toml").exists() {
            Ok(ProjectType::Rust)
        } else if path.join("package.json").exists() {
            Ok(ProjectType::JavaScript)
        } else if path.join("setup.py").exists() || path.join("pyproject.toml").exists() {
            Ok(ProjectType::Python)
        } else if path.join("go.mod").exists() {
            Ok(ProjectType::Go)
        } else if path.join("pom.xml").exists() || path.join("build.gradle").exists() {
            Ok(ProjectType::Java)
        } else {
            Ok(ProjectType::Generic)
        }
    }

    /// Detect available commands
    fn detect_commands(&self, path: &Path) -> Result<Vec<Command>> {
        let mut commands = Vec::new();

        // Check for Makefile
        if path.join("Makefile").exists() {
            commands.push(Command {
                name: "Build".to_string(),
                command: "make build".to_string(),
                working_dir: None,
                env: vec![],
                timeout: Some(60),
                safe: true,
            });
            commands.push(Command {
                name: "Test".to_string(),
                command: "make test".to_string(),
                working_dir: None,
                env: vec![],
                timeout: Some(120),
                safe: true,
            });
        }

        // Check for package.json scripts
        if path.join("package.json").exists() {
            commands.push(Command {
                name: "Install".to_string(),
                command: "npm install".to_string(),
                working_dir: None,
                env: vec![],
                timeout: Some(300),
                safe: true,
            });
        }

        Ok(commands)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_generator_creation() {
        let generator = AgentsMdGenerator::new();
        assert!(generator.config.include_quality);
        assert!(generator.config.include_security);
        assert_eq!(generator.config.max_commands, 20);
    }

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
    fn test_detect_project_type() {
        let generator = AgentsMdGenerator::new();
        let temp = TempDir::new().unwrap();

        // Test Rust detection
        fs::write(temp.path().join("Cargo.toml"), "[package]").unwrap();
        assert_eq!(
            generator.detect_project_type(temp.path()).unwrap(),
            ProjectType::Rust
        );

        // Test JavaScript detection
        fs::remove_file(temp.path().join("Cargo.toml")).unwrap();
        fs::write(temp.path().join("package.json"), "{}").unwrap();
        assert_eq!(
            generator.detect_project_type(temp.path()).unwrap(),
            ProjectType::JavaScript
        );
    }

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
}
