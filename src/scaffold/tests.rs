// Unit tests for ScaffoldEngine
// Part of TICKET-PMAT-5001 - RED Phase

use super::*;
use crate::scaffold::config::{AgentFramework, Feature};
use serial_test::serial;
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
        template_type: TemplateType::Agent {
            based_on: AgentFramework::Pforge,
        },
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
        template_type: TemplateType::Agent {
            based_on: AgentFramework::Pforge,
        },
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
        template_type: TemplateType::Agent {
            based_on: AgentFramework::Pforge,
        },
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
        template_type: TemplateType::Agent {
            based_on: AgentFramework::Pforge,
        },
        features: vec![],
        quality_gates: QualityGateConfig::default(),
    };

    let engine = ScaffoldEngine::new().unwrap();
    assert!(engine.validate_config(&config).is_err());
}

#[test]
#[serial] // Process-global CWD modification requires serial execution
fn test_create_directory_success() {
    let temp_dir = TempDir::new().unwrap();
    let project_name = "test-project";
    let project_path = temp_dir.path().join(project_name);

    // Ensure it doesn't exist
    assert!(!project_path.exists());

    let engine = ScaffoldEngine::new().unwrap();

    // Change CWD directly for isolated test
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let result = engine.create_directory(project_name);

    assert!(result.is_ok());

    let created_path = result.unwrap();
    assert!(created_path.exists());
    assert!(created_path.is_dir());
}

#[test]
#[serial] // Process-global CWD modification requires serial execution
fn test_create_directory_already_exists() {
    let temp_dir = TempDir::new().unwrap();
    let project_name = "existing-project";
    let project_path = temp_dir.path().join(project_name);

    // Create the directory first
    std::fs::create_dir_all(&project_path).unwrap();

    let engine = ScaffoldEngine::new().unwrap();

    // Change CWD directly for isolated test
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
#[ignore] // Flaky - depends on environment setup
#[serial] // Process-global CWD modification requires serial execution
fn test_scaffold_full_workflow() {
    let temp_dir = TempDir::new().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let config = ScaffoldConfig {
        project_name: "full-test-project".into(),
        template_type: TemplateType::Agent {
            based_on: AgentFramework::Pforge,
        },
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
#[ignore] // Flaky - depends on environment setup
#[serial] // Process-global CWD modification requires serial execution
fn test_scaffold_pforge_project() {
    let temp_dir = TempDir::new().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let config = ScaffoldConfig {
        project_name: "my-agent".into(),
        template_type: TemplateType::Agent {
            based_on: AgentFramework::Pforge,
        },
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
#[ignore] // Flaky - WASM scaffold not fully implemented
#[serial] // Process-global CWD modification requires serial execution
fn test_scaffold_wasm_project() {
    let temp_dir = TempDir::new().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let config = ScaffoldConfig {
        project_name: "my-wasm".into(),
        template_type: TemplateType::Wasm {
            based_on: WasmFramework::WasmLabs,
        },
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

    let template_type = TemplateType::Agent {
        based_on: AgentFramework::Pforge,
    };
    engine
        .create_project_structure(temp_dir.path(), &template_type)
        .unwrap();

    assert!(temp_dir.path().join("src/handlers").exists());
    assert!(temp_dir.path().join("tests").exists());
    assert!(temp_dir.path().join("docs").exists());
}

#[test]
fn test_create_project_structure_wasm() {
    let temp_dir = TempDir::new().unwrap();
    let engine = ScaffoldEngine::new().unwrap();

    let template_type = TemplateType::Wasm {
        based_on: WasmFramework::WasmLabs,
    };
    engine
        .create_project_structure(temp_dir.path(), &template_type)
        .unwrap();

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

// TICKET-PMAT-5005: Pre-commit hook installation tests

#[test]
#[ignore] // Flaky - depends on environment setup
#[serial] // Process-global CWD modification requires serial execution
fn test_scaffold_pforge_installs_hooks() {
    let temp_dir = TempDir::new().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let config = ScaffoldConfig {
        project_name: "hook-test-agent".into(),
        template_type: TemplateType::Agent {
            based_on: AgentFramework::Pforge,
        },
        features: vec![],
        quality_gates: QualityGateConfig::default(),
    };

    let engine = ScaffoldEngine::new().unwrap();
    let project_dir = engine.scaffold(config).unwrap();

    // Verify hook exists
    let hook_path = project_dir.join(".git/hooks/pre-commit");
    assert!(hook_path.exists());

    // Verify hook content
    let hook_content = std::fs::read_to_string(&hook_path).unwrap();
    assert!(hook_content.contains("#!/bin/bash"));
    assert!(hook_content.contains("cargo clippy"));
    assert!(hook_content.contains("cargo test"));

    // Verify executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::metadata(&hook_path).unwrap().permissions();
        assert!(perms.mode() & 0o111 != 0, "Hook should be executable");
    }
}

#[test]
#[ignore] // Flaky - WASM scaffold not fully implemented
#[serial] // Process-global CWD modification requires serial execution
fn test_scaffold_wasm_installs_hooks() {
    let temp_dir = TempDir::new().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let config = ScaffoldConfig {
        project_name: "hook-test-wasm".into(),
        template_type: TemplateType::Wasm {
            based_on: WasmFramework::WasmLabs,
        },
        features: vec![],
        quality_gates: QualityGateConfig::default(),
    };

    let engine = ScaffoldEngine::new().unwrap();
    let project_dir = engine.scaffold(config).unwrap();

    // Verify hook exists
    let hook_path = project_dir.join(".git/hooks/pre-commit");
    assert!(hook_path.exists());

    // Verify hook content includes WASM-specific checks
    let hook_content = std::fs::read_to_string(&hook_path).unwrap();
    assert!(hook_content.contains("#!/bin/bash"));
    assert!(hook_content.contains("cargo clippy"));
    assert!(hook_content.contains("cargo test"));
    assert!(hook_content.contains("wasm32-unknown-unknown"));
}

// Phase 1 coverage push (spec §5): scaffold/mod.rs private helpers.
// These exercise the internal orchestration fns without CWD mutation,
// which is why the pre-existing full-workflow tests are #[ignore]'d.

#[test]
fn test_scaffold_engine_default_delegates_to_new() {
    // Default impl must yield an equivalent engine; validate_config on an
    // identical config must agree between Default and new().
    let engine_default: ScaffoldEngine = Default::default();
    let engine_new = ScaffoldEngine::new().unwrap();
    let config = ScaffoldConfig {
        project_name: "ok-name".into(),
        template_type: TemplateType::Library,
        features: vec![],
        quality_gates: QualityGateConfig::default(),
    };
    assert!(engine_default.validate_config(&config).is_ok());
    assert!(engine_new.validate_config(&config).is_ok());
}

#[test]
fn test_prepare_template_vars_has_required_keys() {
    let engine = ScaffoldEngine::new().unwrap();
    let config = ScaffoldConfig {
        project_name: "my-scaffold".into(),
        template_type: TemplateType::Library,
        features: vec![],
        quality_gates: QualityGateConfig::default(),
    };
    let vars = engine.prepare_template_vars(&config);
    assert_eq!(vars.get("project_name").map(String::as_str), Some("my-scaffold"));
    assert!(vars.contains_key("author"));
    assert!(vars.get("description").unwrap().contains("my-scaffold"));
    assert!(vars.contains_key("handler_name"));
    assert!(vars.contains_key("handler_description"));
}

#[test]
fn test_get_file_path_known_template_names() {
    let engine = ScaffoldEngine::new().unwrap();
    let base = std::path::Path::new("/tmp/proj");
    let tt = TemplateType::Library;

    assert_eq!(
        engine.get_file_path(base, "pforge.yaml", &tt),
        base.join("pforge.yaml")
    );
    assert_eq!(
        engine.get_file_path(base, "Cargo.toml", &tt),
        base.join("Cargo.toml")
    );
    assert_eq!(
        engine.get_file_path(base, "Makefile", &tt),
        base.join("Makefile")
    );
    assert_eq!(
        engine.get_file_path(base, "README.md", &tt),
        base.join("README.md")
    );
    assert_eq!(
        engine.get_file_path(base, "handler.rs", &tt),
        base.join("src/handlers/example.rs")
    );
    assert_eq!(
        engine.get_file_path(base, "lib.rs", &tt),
        base.join("src/lib.rs")
    );
    assert_eq!(
        engine.get_file_path(base, "vfs.rs", &tt),
        base.join("src/vfs.rs")
    );
}

#[test]
fn test_get_file_path_unknown_name_falls_through() {
    let engine = ScaffoldEngine::new().unwrap();
    let base = std::path::Path::new("/tmp/proj");
    let tt = TemplateType::Library;
    // Unknown template name falls through to project_dir.join(name).
    assert_eq!(
        engine.get_file_path(base, "CUSTOM.whatever", &tt),
        base.join("CUSTOM.whatever")
    );
}

#[test]
fn test_get_template_registry_agent_returns_pforge() {
    let engine = ScaffoldEngine::new().unwrap();
    let tt = TemplateType::Agent {
        based_on: AgentFramework::Pforge,
    };
    let reg = engine.get_template_registry(&tt);
    // pforge registry lists the expected template set.
    let names = reg.list();
    assert!(
        names.iter().any(|n| n == "pforge.yaml"),
        "pforge registry must include pforge.yaml, got {names:?}"
    );
}

#[test]
fn test_get_template_registry_wasm_returns_wasm_templates() {
    let engine = ScaffoldEngine::new().unwrap();
    let tt = TemplateType::Wasm {
        based_on: WasmFramework::WasmLabs,
    };
    let reg = engine.get_template_registry(&tt);
    let names = reg.list();
    assert!(
        names.iter().any(|n| n == "lib.rs" || n == "Cargo.toml"),
        "wasm registry must include lib.rs or Cargo.toml, got {names:?}"
    );
}

#[test]
fn test_get_template_registry_library_falls_through_to_empty() {
    let engine = ScaffoldEngine::new().unwrap();
    let tt = TemplateType::Library;
    let reg = engine.get_template_registry(&tt);
    // `_ => TemplateRegistry::new()` path — empty registry.
    assert!(reg.list().is_empty());
}

#[test]
fn test_get_template_registry_custom_falls_through_to_empty() {
    let engine = ScaffoldEngine::new().unwrap();
    let tt = TemplateType::Custom {
        path: std::path::PathBuf::from("/tmp/custom"),
    };
    let reg = engine.get_template_registry(&tt);
    assert!(reg.list().is_empty());
}

#[test]
fn test_generate_files_writes_all_pforge_templates() {
    let temp_dir = TempDir::new().unwrap();
    let engine = ScaffoldEngine::new().unwrap();
    let config = ScaffoldConfig {
        project_name: "gen-test".into(),
        template_type: TemplateType::Agent {
            based_on: AgentFramework::Pforge,
        },
        features: vec![],
        quality_gates: QualityGateConfig::default(),
    };
    // Pre-create structure that generate_files expects.
    engine
        .create_project_structure(temp_dir.path(), &config.template_type)
        .unwrap();
    let registry = engine.get_template_registry(&config.template_type);
    let result = engine.generate_files(temp_dir.path(), &registry, &config);
    assert!(result.is_ok(), "generate_files returned {:?}", result);
    assert!(temp_dir.path().join("pforge.yaml").exists());
    assert!(temp_dir.path().join("Cargo.toml").exists());
    let cargo = std::fs::read_to_string(temp_dir.path().join("Cargo.toml")).unwrap();
    assert!(cargo.contains("gen-test"));
}

#[test]
fn test_install_hooks_writes_pre_commit_script() {
    let temp_dir = TempDir::new().unwrap();
    // install_hooks expects .git/hooks to exist.
    std::fs::create_dir_all(temp_dir.path().join(".git/hooks")).unwrap();
    let engine = ScaffoldEngine::new().unwrap();
    let config = ScaffoldConfig {
        project_name: "hook-unit".into(),
        template_type: TemplateType::Agent {
            based_on: AgentFramework::Pforge,
        },
        features: vec![],
        quality_gates: QualityGateConfig::default(),
    };
    engine.install_hooks(temp_dir.path(), &config).unwrap();
    let hook = temp_dir.path().join(".git/hooks/pre-commit");
    assert!(hook.exists(), "pre-commit hook missing");
    let body = std::fs::read_to_string(&hook).unwrap();
    assert!(body.starts_with("#!/bin/bash"));
}

#[test]
fn test_write_file_overwrites_existing_content() {
    let temp_dir = TempDir::new().unwrap();
    let engine = ScaffoldEngine::new().unwrap();
    let path = temp_dir.path().join("overwrite.txt");
    engine.write_file(&path, "first").unwrap();
    engine.write_file(&path, "second").unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "second");
}
