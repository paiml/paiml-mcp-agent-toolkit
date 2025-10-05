// Property-based tests for ScaffoldEngine
// Part of TICKET-PMAT-5001 - REFACTOR Phase

use super::*;
use proptest::prelude::*;
use tempfile::TempDir;

// Generator for valid project names
fn valid_project_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9-]{0,63}"
        .prop_map(|s| s.to_string())
}

proptest! {
    #[test]
    fn prop_valid_names_accepted(name in valid_project_name()) {
        let config = ScaffoldConfig {
            project_name: name,
            template_type: TemplateType::Agent { based_on: AgentFramework::Pforge },
            features: vec![],
            quality_gates: QualityGateConfig::default(),
        };

        let engine = ScaffoldEngine::new().unwrap();
        prop_assert!(engine.validate_config(&config).is_ok());
    }

    #[test]
    fn prop_invalid_names_rejected_slash(name in "[a-z]+/[a-z]+") {
        let config = ScaffoldConfig {
            project_name: name,
            template_type: TemplateType::Agent { based_on: AgentFramework::Pforge },
            features: vec![],
            quality_gates: QualityGateConfig::default(),
        };

        let engine = ScaffoldEngine::new().unwrap();
        prop_assert!(engine.validate_config(&config).is_err());
    }

    #[test]
    fn prop_invalid_names_rejected_backslash(name in "[a-z]+", suffix in "[a-z]+") {
        let invalid_name = format!("{}\\{}", name, suffix);
        let config = ScaffoldConfig {
            project_name: invalid_name,
            template_type: TemplateType::Agent { based_on: AgentFramework::Pforge },
            features: vec![],
            quality_gates: QualityGateConfig::default(),
        };

        let engine = ScaffoldEngine::new().unwrap();
        prop_assert!(engine.validate_config(&config).is_err());
    }

    #[test]
    fn prop_directory_creation_creates_path(name in valid_project_name()) {
        let temp_dir = TempDir::new().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let engine = ScaffoldEngine::new().unwrap();
        let result = engine.create_directory(&name);

        prop_assert!(result.is_ok());
        let path = result.unwrap();
        prop_assert!(path.exists());
        prop_assert!(path.is_dir());
    }

    #[test]
    fn prop_git_init_idempotent(name in valid_project_name()) {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path().join(&name);
        std::fs::create_dir_all(&project_dir).unwrap();

        let engine = ScaffoldEngine::new().unwrap();

        // Initialize git twice
        let result1 = engine.init_git(&project_dir);
        let result2 = engine.init_git(&project_dir);

        // Both should succeed (git init is idempotent)
        prop_assert!(result1.is_ok());
        prop_assert!(result2.is_ok());

        // .git directory exists
        prop_assert!(project_dir.join(".git").exists());
    }
}
