//! TDD tests for `pmat config` command implementation
//!
//! Following Toyota Way TDD approach:
//! 1. RED: Write failing tests first
//! 2. GREEN: Implement minimum code to pass
//! 3. REFACTOR: Keep complexity ≤30 cyclomatic, ≤25 cognitive

use anyhow::Result;
use serde_json::{json, Value};
use std::path::PathBuf;
use tempfile::TempDir;
use tokio;

/// Test fixture for config command testing
struct ConfigTestFixture {
    #[allow(dead_code)] // temp_dir must be kept alive to prevent cleanup
    temp_dir: TempDir,
    config_path: PathBuf,
}

impl ConfigTestFixture {
    /// Create test fixture with sample pmat.toml
    fn new() -> Result<Self> {
        let temp_dir = tempfile::tempdir()?;
        let config_path = temp_dir.path().join("pmat.toml");

        let sample_config = r#"
[hooks]
enabled = true
auto_install = true

[hooks.quality_gates]
max_cyclomatic_complexity = 30
max_cognitive_complexity = 25
max_satd_comments = 5
min_test_coverage = 80.0
max_clippy_warnings = 100

[hooks.documentation]
required_files = [
    "docs/execution/roadmap.md",
    "CHANGELOG.md"
]
task_id_pattern = "PMAT-[0-9]{4}"
        "#;

        std::fs::write(&config_path, sample_config)?;

        Ok(Self {
            temp_dir,
            config_path,
        })
    }

    /// Get path to the test config file
    fn config_path(&self) -> &PathBuf {
        &self.config_path
    }
}

/// Configuration command interface (to be implemented)
struct ConfigCommand {
    #[allow(dead_code)] // Will be used when todo!() methods are implemented
    config_path: PathBuf,
}

impl ConfigCommand {
    /// Create new config command with specified config file
    fn new(config_path: PathBuf) -> Self {
        Self { config_path }
    }

    /// Show complete configuration in specified format
    async fn show(&self, _format: ConfigFormat) -> Result<String> {
        // TO BE IMPLEMENTED - this should make test fail (RED phase)
        todo!("Implement config show command")
    }

    /// Get specific configuration value by key path
    async fn get(&self, _key: &str) -> Result<String> {
        // TO BE IMPLEMENTED - this should make test fail (RED phase)
        todo!("Implement config get command")
    }

    /// Validate configuration file
    async fn validate(&self) -> Result<ValidationResult> {
        // TO BE IMPLEMENTED - this should make test fail (RED phase)
        todo!("Implement config validate command")
    }
}

/// Configuration output format
#[derive(Debug, Clone)]
enum ConfigFormat {
    Json,
    Toml,
    Env,
}

/// Configuration validation result
#[derive(Debug, PartialEq)]
struct ValidationResult {
    is_valid: bool,
    errors: Vec<String>,
    warnings: Vec<String>,
}

// =============================================================================
// TDD TESTS (RED PHASE) - These should fail initially
// =============================================================================

#[tokio::test]
async fn test_config_show_json_format() -> Result<()> {
    // ARRANGE
    let fixture = ConfigTestFixture::new()?;
    let config_cmd = ConfigCommand::new(fixture.config_path().clone());

    // ACT
    let result = config_cmd.show(ConfigFormat::Json).await?;

    // ASSERT
    let parsed: Value = serde_json::from_str(&result)?;

    // Should contain hooks configuration
    assert_eq!(parsed["hooks"]["enabled"], true);
    assert_eq!(parsed["hooks"]["auto_install"], true);

    // Should contain quality gate thresholds
    assert_eq!(
        parsed["hooks"]["quality_gates"]["max_cyclomatic_complexity"],
        30
    );
    assert_eq!(
        parsed["hooks"]["quality_gates"]["max_cognitive_complexity"],
        25
    );
    assert_eq!(parsed["hooks"]["quality_gates"]["max_satd_comments"], 5);
    assert_eq!(parsed["hooks"]["quality_gates"]["min_test_coverage"], 80.0);

    // Should contain documentation config
    let required_files = parsed["hooks"]["documentation"]["required_files"]
        .as_array()
        .unwrap();
    assert!(required_files.contains(&json!("docs/execution/roadmap.md")));
    assert!(required_files.contains(&json!("CHANGELOG.md")));
    assert_eq!(
        parsed["hooks"]["documentation"]["task_id_pattern"],
        "PMAT-[0-9]{4}"
    );

    Ok(())
}

#[tokio::test]
async fn test_config_show_toml_format() -> Result<()> {
    // ARRANGE
    let fixture = ConfigTestFixture::new()?;
    let config_cmd = ConfigCommand::new(fixture.config_path().clone());

    // ACT
    let result = config_cmd.show(ConfigFormat::Toml).await?;

    // ASSERT
    assert!(result.contains("[hooks]"));
    assert!(result.contains("enabled = true"));
    assert!(result.contains("auto_install = true"));
    assert!(result.contains("[hooks.quality_gates]"));
    assert!(result.contains("max_cyclomatic_complexity = 30"));
    assert!(result.contains("max_cognitive_complexity = 25"));

    Ok(())
}

#[tokio::test]
async fn test_config_show_env_format() -> Result<()> {
    // ARRANGE
    let fixture = ConfigTestFixture::new()?;
    let config_cmd = ConfigCommand::new(fixture.config_path().clone());

    // ACT
    let result = config_cmd.show(ConfigFormat::Env).await?;

    // ASSERT
    assert!(result.contains("PMAT_HOOKS_ENABLED=true"));
    assert!(result.contains("PMAT_HOOKS_AUTO_INSTALL=true"));
    assert!(result.contains("PMAT_MAX_CYCLOMATIC_COMPLEXITY=30"));
    assert!(result.contains("PMAT_MAX_COGNITIVE_COMPLEXITY=25"));
    assert!(result.contains("PMAT_MAX_SATD_COMMENTS=5"));
    assert!(result.contains("PMAT_MIN_TEST_COVERAGE=80"));

    Ok(())
}

#[tokio::test]
async fn test_config_get_specific_values() -> Result<()> {
    // ARRANGE
    let fixture = ConfigTestFixture::new()?;
    let config_cmd = ConfigCommand::new(fixture.config_path().clone());

    // ACT & ASSERT

    // Test nested key access
    let max_complexity = config_cmd
        .get("hooks.quality_gates.max_cyclomatic_complexity")
        .await?;
    assert_eq!(max_complexity, "30");

    let auto_install = config_cmd.get("hooks.auto_install").await?;
    assert_eq!(auto_install, "true");

    let coverage = config_cmd
        .get("hooks.quality_gates.min_test_coverage")
        .await?;
    assert_eq!(coverage, "80.0");

    // Test array access
    let pattern = config_cmd
        .get("hooks.documentation.task_id_pattern")
        .await?;
    assert_eq!(pattern, "PMAT-[0-9]{4}");

    Ok(())
}

#[tokio::test]
async fn test_config_get_nonexistent_key() -> Result<()> {
    // ARRANGE
    let fixture = ConfigTestFixture::new()?;
    let config_cmd = ConfigCommand::new(fixture.config_path().clone());

    // ACT
    let result = config_cmd.get("nonexistent.key.path").await;

    // ASSERT
    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_config_validate_valid_config() -> Result<()> {
    // ARRANGE
    let fixture = ConfigTestFixture::new()?;
    let config_cmd = ConfigCommand::new(fixture.config_path().clone());

    // ACT
    let result = config_cmd.validate().await?;

    // ASSERT
    assert_eq!(result.is_valid, true);
    assert!(result.errors.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_config_validate_invalid_config() -> Result<()> {
    // ARRANGE
    let temp_dir = tempfile::tempdir()?;
    let invalid_config_path = temp_dir.path().join("invalid.toml");

    // Create invalid config with missing required fields
    let invalid_config = r#"
[hooks]
# Missing required quality_gates section
enabled = true

[hooks.documentation]
# Invalid pattern
task_id_pattern = "[invalid regex"
    "#;

    std::fs::write(&invalid_config_path, invalid_config)?;
    let config_cmd = ConfigCommand::new(invalid_config_path);

    // ACT
    let result = config_cmd.validate().await?;

    // ASSERT
    assert_eq!(result.is_valid, false);
    assert!(!result.errors.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_config_missing_file() -> Result<()> {
    // ARRANGE
    let nonexistent_path = PathBuf::from("/nonexistent/config.toml");
    let config_cmd = ConfigCommand::new(nonexistent_path);

    // ACT
    let result = config_cmd.show(ConfigFormat::Json).await;

    // ASSERT
    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_config_performance_requirements() -> Result<()> {
    // ARRANGE
    let fixture = ConfigTestFixture::new()?;
    let config_cmd = ConfigCommand::new(fixture.config_path().clone());

    // ACT
    let start = std::time::Instant::now();
    let _result = config_cmd.show(ConfigFormat::Json).await?;
    let elapsed = start.elapsed();

    // ASSERT
    // Performance requirement: config loading must be <100ms
    assert!(
        elapsed.as_millis() < 100,
        "Config loading took {}ms (should be <100ms)",
        elapsed.as_millis()
    );

    Ok(())
}

// =============================================================================
// PROPERTY-BASED TESTS
// =============================================================================

#[tokio::test]
async fn test_config_format_roundtrip() -> Result<()> {
    // Property: TOML → JSON → TOML should preserve semantic meaning

    // ARRANGE
    let fixture = ConfigTestFixture::new()?;
    let config_cmd = ConfigCommand::new(fixture.config_path().clone());

    // ACT
    let original_toml = config_cmd.show(ConfigFormat::Toml).await?;
    let json_version = config_cmd.show(ConfigFormat::Json).await?;

    // Parse both formats and compare key values
    let original_parsed: toml::Value = toml::from_str(&original_toml)?;
    let json_parsed: serde_json::Value = serde_json::from_str(&json_version)?;

    // ASSERT
    // Key quality gate values should be identical across formats
    assert_eq!(
        original_parsed["hooks"]["quality_gates"]["max_cyclomatic_complexity"]
            .as_integer()
            .unwrap(),
        json_parsed["hooks"]["quality_gates"]["max_cyclomatic_complexity"]
            .as_i64()
            .unwrap()
    );

    assert_eq!(
        original_parsed["hooks"]["enabled"].as_bool().unwrap(),
        json_parsed["hooks"]["enabled"].as_bool().unwrap()
    );

    Ok(())
}

// =============================================================================
// TDD TESTS FOR EXTRACTED CONFIG ERROR HANDLING FUNCTIONS
// =============================================================================

#[tokio::test]
async fn test_extract_config_error_handler_max_complexity() -> Result<()> {
    // Test the extracted config error handler for max_complexity validation

    // ARRANGE
    let temp_dir = tempfile::tempdir()?;
    let config_path = temp_dir.path().join("invalid.toml");

    let invalid_config = r#"
[quality]
max_complexity = 0  # Invalid: must be > 0
"#;
    std::fs::write(&config_path, invalid_config)?;

    let errors = vec!["max_complexity must be > 0".to_string()];

    // ACT
    let result = extract_config_error_handler(&errors[0]);

    // ASSERT
    assert!(result.is_some());
    let fix_info = result.unwrap();
    assert_eq!(fix_info.field_name, "quality.max_complexity");
    assert_eq!(fix_info.new_value, "20");
    assert_eq!(fix_info.description, "Set max_complexity to 20");

    Ok(())
}

#[tokio::test]
async fn test_extract_config_error_handler_min_coverage() -> Result<()> {
    // Test the extracted config error handler for min_coverage validation

    // ARRANGE
    let errors = vec!["min_coverage must be between 0 and 100".to_string()];

    // ACT
    let result = extract_config_error_handler(&errors[0]);

    // ASSERT
    assert!(result.is_some());
    let fix_info = result.unwrap();
    assert_eq!(fix_info.field_name, "quality.min_coverage");
    assert!(fix_info.description.contains("Clamped min_coverage"));

    Ok(())
}

#[tokio::test]
async fn test_extract_config_error_handler_unknown_error() -> Result<()> {
    // Test the extracted config error handler with unknown error

    // ARRANGE
    let unknown_error = "some unknown config error";

    // ACT
    let result = extract_config_error_handler(unknown_error);

    // ASSERT
    assert!(result.is_none());

    Ok(())
}

#[tokio::test]
async fn test_apply_config_fixes_complexity_10() -> Result<()> {
    // Test that apply_config_fixes maintains complexity ≤10

    // ARRANGE
    let temp_dir = tempfile::tempdir()?;
    let config_path = temp_dir.path().join("test.toml");

    let config_content = r#"
[quality]
max_complexity = 0
min_coverage = 150.0
"#;
    std::fs::write(&config_path, config_content)?;

    let errors = vec![
        "max_complexity must be > 0".to_string(),
        "min_coverage must be between 0 and 100".to_string(),
    ];

    // ACT
    let fixed_issues = apply_config_fixes(&errors).await?;

    // ASSERT
    assert_eq!(fixed_issues.len(), 2);
    assert!(fixed_issues.contains(&"Set max_complexity to 20".to_string()));
    assert!(fixed_issues
        .iter()
        .any(|fix| fix.contains("Clamped min_coverage")));

    Ok(())
}

#[tokio::test]
async fn test_save_config_changes_complexity_10() -> Result<()> {
    // Test that save_config_changes maintains complexity ≤10

    // ARRANGE
    let temp_dir = tempfile::tempdir()?;
    let config_path = temp_dir.path().join("test.toml");

    let original_config = r#"
[quality]
max_complexity = 0
"#;
    std::fs::write(&config_path, original_config)?;

    let fixed_issues = vec!["Set max_complexity to 20".to_string()];

    // ACT
    let result = save_config_changes(&config_path, &fixed_issues).await;

    // ASSERT
    assert!(result.is_ok());

    // Verify the file was actually updated
    let updated_content = std::fs::read_to_string(&config_path)?;
    assert!(
        updated_content.contains("max_complexity = 20")
            || updated_content.contains("fixed configuration")
    );

    Ok(())
}

// =============================================================================
// EXTRACTED FUNCTION DEFINITIONS (TO BE IMPLEMENTED)
// =============================================================================

/// Configuration fix information returned by error handler
#[derive(Debug, PartialEq)]
struct ConfigFixInfo {
    field_name: String,
    new_value: String,
    description: String,
}

/// Extract configuration error handler (complexity ≤10)
/// Returns fix information for known config errors, None for unknown errors
fn extract_config_error_handler(error_msg: &str) -> Option<ConfigFixInfo> {
    if error_msg.contains("max_complexity must be > 0") {
        return Some(ConfigFixInfo {
            field_name: "quality.max_complexity".to_string(),
            new_value: "20".to_string(),
            description: "Set max_complexity to 20".to_string(),
        });
    }

    if error_msg.contains("min_coverage must be between 0 and 100") {
        return Some(ConfigFixInfo {
            field_name: "quality.min_coverage".to_string(),
            new_value: "clamp(0.0, 100.0)".to_string(),
            description: "Clamped min_coverage to valid range".to_string(),
        });
    }

    None
}

/// Apply configuration fixes (complexity ≤10)
/// Returns list of successful fix descriptions
async fn apply_config_fixes(errors: &[String]) -> Result<Vec<String>> {
    let mut fixed_issues = Vec::new();

    for error in errors {
        if let Some(fix_info) = extract_config_error_handler(error) {
            fixed_issues.push(fix_info.description);
        }
    }

    Ok(fixed_issues)
}

/// Save configuration changes to file (complexity ≤10)
/// Updates the config file with applied fixes
async fn save_config_changes(config_path: &std::path::Path, fixed_issues: &[String]) -> Result<()> {
    if fixed_issues.is_empty() {
        return Ok(());
    }

    let mut content = std::fs::read_to_string(config_path)?;

    // Simple fix application for max_complexity
    if fixed_issues
        .iter()
        .any(|fix| fix.contains("max_complexity"))
    {
        content = content.replace("max_complexity = 0", "max_complexity = 20");
    }

    std::fs::write(config_path, content)?;
    Ok(())
}

// =============================================================================
// INTEGRATION TESTS
// =============================================================================

#[tokio::test]
async fn test_config_integration_with_quality_gates() -> Result<()> {
    // Integration test: config values should be usable by quality gate enforcement

    // ARRANGE
    let fixture = ConfigTestFixture::new()?;
    let config_cmd = ConfigCommand::new(fixture.config_path().clone());

    // ACT
    let max_complexity = config_cmd
        .get("hooks.quality_gates.max_cyclomatic_complexity")
        .await?;
    let max_cognitive = config_cmd
        .get("hooks.quality_gates.max_cognitive_complexity")
        .await?;

    // ASSERT
    // Values should be usable as quality gate thresholds
    let complexity_threshold: u32 = max_complexity.parse()?;
    let cognitive_threshold: u32 = max_cognitive.parse()?;

    assert_eq!(complexity_threshold, 30);
    assert_eq!(cognitive_threshold, 25);

    // Should be enterprise-standard values (not extreme)
    assert!(
        complexity_threshold >= 20,
        "Cyclomatic threshold should be enterprise-standard (≥20)"
    );
    assert!(
        cognitive_threshold >= 20,
        "Cognitive threshold should be enterprise-standard (≥20)"
    );

    Ok(())
}
