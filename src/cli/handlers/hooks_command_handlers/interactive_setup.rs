//! Interactive setup and configuration management for hooks

#![cfg_attr(coverage_nightly, coverage(off))]

use super::hooks_command::HooksCommand;
use anyhow::Result;
use std::fs;

impl HooksCommand {
    /// Run interactive setup to configure hook preferences
    pub(super) fn run_interactive_setup(&self) -> Result<()> {
        println!("🔧 Interactive Pre-commit Hook Setup");
        println!("====================================\n");

        // Detect project type
        let project_type = self.detect_project_type();
        println!("📦 Detected project type: {project_type}");

        // Ask about complexity thresholds
        println!("\n⚙️  Quality Thresholds:");
        let max_complexity =
            self.prompt_number("Maximum cyclomatic complexity (default: 10)", 10)?;
        let max_cognitive = self.prompt_number("Maximum cognitive complexity (default: 15)", 15)?;

        // Ask about coverage
        let min_coverage = self.prompt_number("Minimum test coverage % (default: 80)", 80)?;

        // Ask about SATD
        let max_satd = self.prompt_number("Maximum SATD comments (default: 5)", 5)?;

        println!("\n📝 Updating configuration...");

        // Update pmat.toml with user preferences
        let config_path = std::env::current_dir()?.join("pmat.toml");
        if config_path.exists() {
            // Update existing config
            let config_content = fs::read_to_string(&config_path)?;
            let updated = self.update_config_values(
                &config_content,
                max_complexity,
                max_cognitive,
                min_coverage,
                max_satd,
            );
            fs::write(&config_path, updated)?;
            println!("✅ Updated pmat.toml with your preferences");
        } else {
            // Create new config
            let config_content =
                self.generate_config_content(max_complexity, max_cognitive, min_coverage, max_satd);
            fs::write(&config_path, config_content)?;
            println!("✅ Created pmat.toml with your preferences");
        }

        println!("\n✅ Interactive setup complete!\n");
        Ok(())
    }

    /// Prompt user for a number with default
    fn prompt_number(&self, prompt: &str, default: u32) -> Result<u32> {
        debug_assert!(!prompt.is_empty(), "prompt must not be empty");
        use std::io::{self, Write};

        print!("  {prompt}: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            Ok(default)
        } else {
            input
                .parse::<u32>()
                .map_err(|e| anyhow::anyhow!("Invalid number: {}", e))
        }
    }

    /// Detect project type from files in directory
    pub(crate) fn detect_project_type(&self) -> String {
        let current_dir = std::env::current_dir().ok();

        if let Some(dir) = current_dir {
            if dir.join("Cargo.toml").exists() {
                return "Rust".to_string();
            }
            if dir.join("package.json").exists() {
                return "JavaScript/TypeScript".to_string();
            }
            if dir.join("pyproject.toml").exists() || dir.join("setup.py").exists() {
                return "Python".to_string();
            }
            if dir.join("go.mod").exists() {
                return "Go".to_string();
            }
        }

        "Unknown".to_string()
    }

    /// Update config values in existing TOML content
    pub(crate) fn update_config_values(
        &self,
        content: &str,
        max_complexity: u32,
        max_cognitive: u32,
        min_coverage: u32,
        _max_satd: u32,
    ) -> String {
        debug_assert!(!content.is_empty(), "content must not be empty");
        // Simple string replacement preserves comments and formatting.
        // Full TOML parsing would lose comments - acceptable tradeoff for config updates.
        let old_complexity = self.extract_current_value(content, "max_complexity");
        let content = content.replace(
            &format!("max_complexity = {old_complexity}"),
            &format!("max_complexity = {max_complexity}"),
        );

        let old_cognitive = self.extract_current_value(&content, "max_cognitive_complexity");
        let content = content.replace(
            &format!("max_cognitive_complexity = {old_cognitive}"),
            &format!("max_cognitive_complexity = {max_cognitive}"),
        );

        let old_coverage = self.extract_current_value(&content, "min_coverage");
        content.replace(
            &format!("min_coverage = {old_coverage}"),
            &format!("min_coverage = {min_coverage}"),
        )
    }

    /// Extract current value from TOML content
    pub(crate) fn extract_current_value(&self, content: &str, key: &str) -> String {
        debug_assert!(!content.is_empty(), "content must not be empty");
        debug_assert!(!key.is_empty(), "key must not be empty");
        content
            .lines()
            .find(|line| line.contains(key))
            .and_then(|line| line.split('=').nth(1))
            .map(|val| val.trim().to_string())
            .unwrap_or_else(|| "10".to_string())
    }

    /// Generate new config content with specified values
    pub(crate) fn generate_config_content(
        &self,
        max_complexity: u32,
        max_cognitive: u32,
        min_coverage: u32,
        max_satd: u32,
    ) -> String {
        format!(
            r#"# PMAT Configuration File
# Generated by interactive setup

[quality]
max_complexity = {max_complexity}
max_cognitive_complexity = {max_cognitive}
min_coverage = {min_coverage}
max_satd_comments = {max_satd}
min_grade = "B+"

[hooks]
enabled = true
fail_on_warning = false
show_diff = true
auto_fix = false

[hooks.performance]
timeout = 30
max_files = 1000
incremental = true
"#
        )
    }
}
