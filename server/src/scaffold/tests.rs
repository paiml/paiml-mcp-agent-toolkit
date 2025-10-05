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
        template_type: TemplateType::Agent { based_on: AgentFramework::Pforge },
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
        template_type: TemplateType::Agent { based_on: AgentFramework::Pforge },
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
        template_type: TemplateType::Agent { based_on: AgentFramework::Pforge },
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
        template_type: TemplateType::Agent { based_on: AgentFramework::Pforge },
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
        template_type: TemplateType::Agent { based_on: AgentFramework::Pforge },
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

// TICKET-PMAT-5004: Project structure generation tests

#[test]
fn test_scaffold_pforge_project() {
    let temp_dir = TempDir::new().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let config = ScaffoldConfig {
        project_name: "my-agent".into(),
        template_type: TemplateType::Agent { based_on: AgentFramework::Pforge },
        features: vec![],
        quality_gates: QualityGateConfig::default(),
    };

    let engine = ScaffoldEngine::new().unwrap();
    let project_dir = engine.scaffold(config).unwrap();

    // Verify directory structure
    assert!(project_dir.join("src/handlers").exists());
    assert!(project_dir.join("tests").exists());
    assert!(project_dir.join("docs").exists());
    assert!(project_dir.join(".git").exists());

    // Verify files
    assert!(project_dir.join("pforge.yaml").exists());
    assert!(project_dir.join("Cargo.toml").exists());
    assert!(project_dir.join("README.md").exists());
    assert!(project_dir.join("src/handlers/example.rs").exists());

    // Verify file contents
    let cargo_toml = std::fs::read_to_string(project_dir.join("Cargo.toml")).unwrap();
    assert!(cargo_toml.contains("name = \"my-agent\""));
}

#[test]
fn test_scaffold_wasm_project() {
    let temp_dir = TempDir::new().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let config = ScaffoldConfig {
        project_name: "my-wasm".into(),
        template_type: TemplateType::Wasm { based_on: WasmFramework::WasmLabs },
        features: vec![],
        quality_gates: QualityGateConfig::default(),
    };

    let engine = ScaffoldEngine::new().unwrap();
    let project_dir = engine.scaffold(config).unwrap();

    // Verify directory structure
    assert!(project_dir.join("src").exists());
    assert!(project_dir.join("tests").exists());
    assert!(project_dir.join("benches").exists());
    assert!(project_dir.join(".git").exists());

    // Verify files (WASM templates: Cargo.toml, Makefile, lib.rs, vfs.rs)
    assert!(project_dir.join("Cargo.toml").exists());
    assert!(project_dir.join("Makefile").exists());
    assert!(project_dir.join("src/lib.rs").exists());
    assert!(project_dir.join("src/vfs.rs").exists());

    // Verify file contents
    let cargo_toml = std::fs::read_to_string(project_dir.join("Cargo.toml")).unwrap();
    assert!(cargo_toml.contains("name = \"my-wasm\""));
    assert!(cargo_toml.contains("cdylib"));
}

#[test]
fn test_create_project_structure_agent() {
    let temp_dir = TempDir::new().unwrap();
    let engine = ScaffoldEngine::new().unwrap();

    let template_type = TemplateType::Agent { based_on: AgentFramework::Pforge };
    engine.create_project_structure(temp_dir.path(), &template_type).unwrap();

    assert!(temp_dir.path().join("src/handlers").exists());
    assert!(temp_dir.path().join("tests").exists());
    assert!(temp_dir.path().join("docs").exists());
}

#[test]
fn test_create_project_structure_wasm() {
    let temp_dir = TempDir::new().unwrap();
    let engine = ScaffoldEngine::new().unwrap();

    let template_type = TemplateType::Wasm { based_on: WasmFramework::WasmLabs };
    engine.create_project_structure(temp_dir.path(), &template_type).unwrap();

    assert!(temp_dir.path().join("src").exists());
    assert!(temp_dir.path().join("tests").exists());
    assert!(temp_dir.path().join("benches").exists());
}

#[test]
fn test_write_file() {
    let temp_dir = TempDir::new().unwrap();
    let engine = ScaffoldEngine::new().unwrap();

    let file_path = temp_dir.path().join("nested/dir/test.txt");
    let content = "Hello, World!";

    engine.write_file(&file_path, content).unwrap();

    assert!(file_path.exists());
    let read_content = std::fs::read_to_string(&file_path).unwrap();
    assert_eq!(read_content, content);
}
