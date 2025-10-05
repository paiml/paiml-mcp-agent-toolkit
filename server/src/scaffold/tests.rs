// Unit tests for ScaffoldEngine
// Part of TICKET-PMAT-5001 - RED Phase

use super::*;
use crate::scaffold::config::{AgentFramework, Feature};
use tempfile::TempDir;

#[test]
fn test_scaffold_engine_creation() {
    let engine = ScaffoldEngine::new();
    assert!(engine.is_ok());
}

#[test]
fn test_validate_config_valid() {
    let config = ScaffoldConfig {
        project_name: "valid-project".into(),
        template: Template::Agent { based_on: AgentFramework::Pforge },
        features: vec![],
        quality_gates: QualityGateConfig::default(),
    };

    let engine = ScaffoldEngine::new().unwrap();
    assert!(engine.validate_config(&config).is_ok());
}

#[test]
fn test_validate_config_invalid_empty_name() {
    let config = ScaffoldConfig {
        project_name: "".into(),
        template: Template::Agent { based_on: AgentFramework::Pforge },
        features: vec![],
        quality_gates: QualityGateConfig::default(),
    };

    let engine = ScaffoldEngine::new().unwrap();
    assert!(engine.validate_config(&config).is_err());
}

#[test]
fn test_validate_config_invalid_slash_in_name() {
    let config = ScaffoldConfig {
        project_name: "invalid/project".into(),
        template: Template::Agent { based_on: AgentFramework::Pforge },
        features: vec![],
        quality_gates: QualityGateConfig::default(),
    };

    let engine = ScaffoldEngine::new().unwrap();
    assert!(engine.validate_config(&config).is_err());
}

#[test]
fn test_validate_config_invalid_too_long() {
    let config = ScaffoldConfig {
        project_name: "a".repeat(300),
        template: Template::Agent { based_on: AgentFramework::Pforge },
        features: vec![],
        quality_gates: QualityGateConfig::default(),
    };

    let engine = ScaffoldEngine::new().unwrap();
    assert!(engine.validate_config(&config).is_err());
}

#[test]
fn test_create_directory_success() {
    let temp_dir = TempDir::new().unwrap();
    let project_name = "test-project";
    let project_path = temp_dir.path().join(project_name);

    // Ensure it doesn't exist
    assert!(!project_path.exists());

    let engine = ScaffoldEngine::new().unwrap();

    // Change to temp dir so relative paths work
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let result = engine.create_directory(project_name);
    assert!(result.is_ok());

    let created_path = result.unwrap();
    assert!(created_path.exists());
    assert!(created_path.is_dir());
}

#[test]
fn test_create_directory_already_exists() {
    let temp_dir = TempDir::new().unwrap();
    let project_name = "existing-project";
    let project_path = temp_dir.path().join(project_name);

    // Create the directory first
    std::fs::create_dir_all(&project_path).unwrap();

    let engine = ScaffoldEngine::new().unwrap();

    // Change to temp dir
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let result = engine.create_directory(project_name);
    assert!(result.is_err());
}

#[test]
fn test_init_git_success() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("git-project");
    std::fs::create_dir_all(&project_dir).unwrap();

    let engine = ScaffoldEngine::new().unwrap();
    let result = engine.init_git(&project_dir);

    assert!(result.is_ok());
    assert!(project_dir.join(".git").exists());
}

#[test]
fn test_scaffold_full_workflow() {
    let temp_dir = TempDir::new().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let config = ScaffoldConfig {
        project_name: "full-test-project".into(),
        template: Template::Agent { based_on: AgentFramework::Pforge },
        features: vec![Feature::Logging],
        quality_gates: QualityGateConfig::extreme_tdd(),
    };

    let engine = ScaffoldEngine::new().unwrap();
    let result = engine.scaffold(config);

    assert!(result.is_ok());

    let project_dir = result.unwrap();
    assert!(project_dir.exists());
    assert!(project_dir.is_dir());
    assert!(project_dir.join(".git").exists());
}
