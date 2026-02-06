
use super::*;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use tempfile::TempDir;

// ==========================================================================
// Helper Functions for Test Setup
// ==========================================================================

/// Create a minimal Cargo.toml for testing
fn create_cargo_toml(dir: &Path, content: &str) {
    let cargo_toml = dir.join("Cargo.toml");
    std::fs::write(&cargo_toml, content).expect("Failed to create Cargo.toml");
}

/// Create a file cache with predefined content
fn create_test_cache(files: Vec<(PathBuf, String)>) -> FileCache {
    let mut cache = FileCache::new();
    for (path, content) in files {
        cache.insert(path, content);
    }
    cache
}

// ==========================================================================
// Basic Construction Tests
// ==========================================================================

mod construction_tests {
    use super::*;

    #[test]
    fn test_scorer_creation() {
        let scorer = BuildPerfScorer::new();
        assert_eq!(scorer.name(), "Build Performance");
        assert_eq!(scorer.max_points(), 15.0);
    }

    #[test]
    fn test_default_implementation() {
        let scorer = BuildPerfScorer::default();
        assert_eq!(scorer.name(), "Build Performance");
        assert_eq!(scorer.max_points(), 15.0);
    }

    #[test]
    fn test_scorer_implements_trait() {
        let scorer = BuildPerfScorer::new();
        let _trait_obj: &dyn Scorer = &scorer;
    }

    #[test]
    fn test_scorer_clone() {
        let scorer = BuildPerfScorer::new();
        let cloned = scorer.clone();
        assert_eq!(scorer.name(), cloned.name());
        assert_eq!(scorer.max_points(), cloned.max_points());
    }

    #[test]
    fn test_scorer_debug() {
        let scorer = BuildPerfScorer::new();
        let debug_str = format!("{:?}", scorer);
        assert!(debug_str.contains("BuildPerfScorer"));
        assert!(debug_str.contains("Build Performance"));
    }
}

// ==========================================================================
// LTO Scoring Tests
// ==========================================================================

mod lto_tests {
    use super::*;

    #[test]
    fn test_lto_enabled_true() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"
[package]
name = "test"
version = "0.1.0"

[profile.release]
lto = true
"#;
        create_cargo_toml(temp_dir.path(), content);

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_lto(temp_dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2.0);
    }

    #[test]
    fn test_lto_thin() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"
[package]
name = "test"
version = "0.1.0"

[profile.release]
lto = "thin"
"#;
        create_cargo_toml(temp_dir.path(), content);

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_lto(temp_dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2.0);
    }

    #[test]
    fn test_lto_fat() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"
[package]
name = "test"
version = "0.1.0"

[profile.release]
lto = "fat"
"#;
        create_cargo_toml(temp_dir.path(), content);

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_lto(temp_dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2.0);
    }

    #[test]
    fn test_lto_disabled() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"
[package]
name = "test"
version = "0.1.0"

[profile.release]
lto = false
"#;
        create_cargo_toml(temp_dir.path(), content);

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_lto(temp_dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0.0);
    }

    #[test]
    fn test_lto_missing() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"
[package]
name = "test"
version = "0.1.0"

[profile.release]
opt-level = 3
"#;
        create_cargo_toml(temp_dir.path(), content);

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_lto(temp_dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0.0);
    }

    #[test]
    fn test_lto_no_release_profile() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"
[package]
name = "test"
version = "0.1.0"
"#;
        create_cargo_toml(temp_dir.path(), content);

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_lto(temp_dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0.0);
    }

    #[test]
    fn test_lto_with_cache() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");
        let content = r#"
[package]
name = "test"

[profile.release]
lto = true
"#;
        let cache = create_test_cache(vec![(cargo_toml_path, content.to_string())]);

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_lto(temp_dir.path(), Some(&cache));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2.0);
    }

    #[test]
    fn test_lto_no_cargo_toml() {
        let temp_dir = TempDir::new().unwrap();
        let scorer = BuildPerfScorer::new();
        let result = scorer.score_lto(temp_dir.path(), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_lto_in_dev_profile_not_counted() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"
[package]
name = "test"

[profile.dev]
lto = true

[profile.release]
debug = true
"#;
        create_cargo_toml(temp_dir.path(), content);

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_lto(temp_dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0.0);
    }

    #[test]
    fn test_lto_multiple_profiles() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"
[package]
name = "test"

[profile.dev]
opt-level = 1

[profile.release]
lto = "thin"

[profile.bench]
debug = true
"#;
        create_cargo_toml(temp_dir.path(), content);

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_lto(temp_dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2.0);
    }
}

// ==========================================================================
// Target Directory Size Tests
// ==========================================================================

mod target_dir_tests {
    use super::*;

    #[test]
    fn test_no_target_dir_gives_full_points() {
        let temp_dir = TempDir::new().unwrap();

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_target_dir_size(temp_dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2.0);
    }

    #[test]
    fn test_empty_target_dir_gives_full_points() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::create_dir(temp_dir.path().join("target")).unwrap();

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_target_dir_size(temp_dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2.0);
    }

    #[test]
    fn test_small_target_dir_gives_full_points() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path().join("target");
        std::fs::create_dir(&target).unwrap();
        // Create a small file (1KB)
        std::fs::write(target.join("test.txt"), vec![0u8; 1024]).unwrap();

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_target_dir_size(temp_dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2.0);
    }

    #[test]
    fn test_nested_target_dir_calculation() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path().join("target");
        let nested = target.join("debug");
        std::fs::create_dir_all(&nested).unwrap();
        // Create files in nested directory
        std::fs::write(nested.join("file1.txt"), vec![0u8; 1024]).unwrap();
        std::fs::write(nested.join("file2.txt"), vec![0u8; 2048]).unwrap();

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_target_dir_size(temp_dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2.0); // Still under 2GB
    }
}

// ==========================================================================
// Cargo.lock Tests
// ==========================================================================

mod cargo_lock_tests {
    use super::*;

    #[test]
    fn test_cargo_lock_exists() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("Cargo.lock"), "# Cargo.lock").unwrap();

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_cargo_lock(temp_dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2.0);
    }

    #[test]
    fn test_cargo_lock_missing() {
        let temp_dir = TempDir::new().unwrap();

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_cargo_lock(temp_dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0.0);
    }

    #[test]
    fn test_cargo_lock_empty() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("Cargo.lock"), "").unwrap();

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_cargo_lock(temp_dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2.0);
    }
}

// ==========================================================================
// Cargo Config Tests
// ==========================================================================

mod cargo_config_tests {
    use super::*;

    #[test]
    fn test_config_toml_with_build_section() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_dir = temp_dir.path().join(".cargo");
        std::fs::create_dir(&cargo_dir).unwrap();
        std::fs::write(cargo_dir.join("config.toml"), "[build]\njobs = 4").unwrap();

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_cargo_config(temp_dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2.0);
    }

    #[test]
    fn test_config_toml_with_target_section() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_dir = temp_dir.path().join(".cargo");
        std::fs::create_dir(&cargo_dir).unwrap();
        std::fs::write(
            cargo_dir.join("config.toml"),
            "[target.x86_64-unknown-linux-gnu]\nlinker = \"clang\"",
        )
        .unwrap();

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_cargo_config(temp_dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2.0);
    }

    #[test]
    fn test_config_toml_with_env_section() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_dir = temp_dir.path().join(".cargo");
        std::fs::create_dir(&cargo_dir).unwrap();
        std::fs::write(
            cargo_dir.join("config.toml"),
            "[env]\nRUST_BACKTRACE = \"1\"",
        )
        .unwrap();

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_cargo_config(temp_dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2.0);
    }

    #[test]
    fn test_config_toml_with_alias_section() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_dir = temp_dir.path().join(".cargo");
        std::fs::create_dir(&cargo_dir).unwrap();
        std::fs::write(cargo_dir.join("config.toml"), "[alias]\nb = \"build\"").unwrap();

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_cargo_config(temp_dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2.0);
    }

    #[test]
    fn test_config_toml_minimal() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_dir = temp_dir.path().join(".cargo");
        std::fs::create_dir(&cargo_dir).unwrap();
        std::fs::write(cargo_dir.join("config.toml"), "# Just a comment\n").unwrap();

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_cargo_config(temp_dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1.0); // Exists but minimal
    }

    #[test]
    fn test_legacy_config_file() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_dir = temp_dir.path().join(".cargo");
        std::fs::create_dir(&cargo_dir).unwrap();
        std::fs::write(cargo_dir.join("config"), "[build]\njobs = 2").unwrap();

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_cargo_config(temp_dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2.0);
    }

    #[test]
    fn test_no_cargo_config() {
        let temp_dir = TempDir::new().unwrap();

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_cargo_config(temp_dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0.0);
    }

    #[test]
    fn test_cargo_config_with_cache() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_dir = temp_dir.path().join(".cargo");
        std::fs::create_dir(&cargo_dir).unwrap();
        let config_path = cargo_dir.join("config.toml");
        std::fs::write(&config_path, "").unwrap(); // Create empty file

        let cache = create_test_cache(vec![(config_path, "[build]\njobs = 4".to_string())]);

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_cargo_config(temp_dir.path(), Some(&cache));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2.0);
    }

    #[test]
    fn test_config_toml_preferred_over_legacy() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_dir = temp_dir.path().join(".cargo");
        std::fs::create_dir(&cargo_dir).unwrap();
        // Both files exist
        std::fs::write(cargo_dir.join("config.toml"), "[build]\njobs = 4").unwrap();
        std::fs::write(cargo_dir.join("config"), "# legacy").unwrap();

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_cargo_config(temp_dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2.0); // Uses config.toml
    }
}

// ==========================================================================
// Incremental Builds Tests
// ==========================================================================

mod incremental_builds_tests {
    use super::*;

    #[test]
    fn test_incremental_enabled_in_config() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_dir = temp_dir.path().join(".cargo");
        std::fs::create_dir(&cargo_dir).unwrap();
        std::fs::write(cargo_dir.join("config.toml"), "[build]\nincremental = true").unwrap();

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_incremental_builds(temp_dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2.0);
    }

    #[test]
    fn test_incremental_disabled_in_cargo_toml() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"
[package]
name = "test"

[profile.dev]
incremental = false
"#;
        create_cargo_toml(temp_dir.path(), content);

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_incremental_builds(temp_dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0.0);
    }

    #[test]
    fn test_incremental_default_behavior() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"
[package]
name = "test"
"#;
        create_cargo_toml(temp_dir.path(), content);

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_incremental_builds(temp_dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1.5); // Partial credit for default
    }

    #[test]
    fn test_incremental_with_cache() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_dir = temp_dir.path().join(".cargo");
        std::fs::create_dir(&cargo_dir).unwrap();
        let config_path = cargo_dir.join("config.toml");
        std::fs::write(&config_path, "").unwrap();

        let cache = create_test_cache(vec![(
            config_path,
            "[build]\nincremental = true".to_string(),
        )]);

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_incremental_builds(temp_dir.path(), Some(&cache));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2.0);
    }

    #[test]
    fn test_incremental_no_config_no_cargo_toml() {
        let temp_dir = TempDir::new().unwrap();

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_incremental_builds(temp_dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1.5); // Default partial credit
    }
}

// ==========================================================================
// Codegen Units Tests
// ==========================================================================

mod codegen_units_tests {
    use super::*;

    #[test]
    fn test_codegen_units_optimal() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"
[package]
name = "test"

[profile.release]
codegen-units = 1
"#;
        create_cargo_toml(temp_dir.path(), content);

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_codegen_units(temp_dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2.0);
    }

    #[test]
    fn test_codegen_units_missing() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"
[package]
name = "test"

[profile.release]
lto = true
"#;
        create_cargo_toml(temp_dir.path(), content);

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_codegen_units(temp_dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0.0);
    }

    #[test]
    fn test_codegen_units_no_release_profile() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"
[package]
name = "test"
"#;
        create_cargo_toml(temp_dir.path(), content);

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_codegen_units(temp_dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0.0);
    }

    #[test]
    fn test_codegen_units_with_cache() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");
        let content = r#"
[package]
name = "test"

[profile.release]
codegen-units = 1
"#;
        let cache = create_test_cache(vec![(cargo_toml_path, content.to_string())]);

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_codegen_units(temp_dir.path(), Some(&cache));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2.0);
    }

    #[test]
    fn test_codegen_units_no_cargo_toml() {
        let temp_dir = TempDir::new().unwrap();

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_codegen_units(temp_dir.path(), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_codegen_units_in_dev_profile_not_counted() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"
[package]
name = "test"

[profile.dev]
codegen-units = 1

[profile.release]
debug = true
"#;
        create_cargo_toml(temp_dir.path(), content);

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_codegen_units(temp_dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0.0);
    }
}

// ==========================================================================
// Build System Tests
// ==========================================================================

mod build_system_tests {
    use super::*;

    #[test]
    fn test_makefile_detection() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("Makefile"), "all:\n\techo hello").unwrap();

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_build_system(temp_dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1.5);
    }

    #[test]
    fn test_justfile_lowercase_detection() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("justfile"), "all:\n\techo hello").unwrap();

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_build_system(temp_dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1.5);
    }

    #[test]
    fn test_justfile_uppercase_detection() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("Justfile"), "all:\n\techo hello").unwrap();

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_build_system(temp_dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1.5);
    }

    #[test]
    fn test_build_rs_detection() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("build.rs"), "fn main() {}").unwrap();

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_build_system(temp_dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1.0);
    }

    #[test]
    fn test_multiple_build_systems() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("Makefile"), "all:").unwrap();
        std::fs::write(temp_dir.path().join("justfile"), "all:").unwrap();
        std::fs::write(temp_dir.path().join("build.rs"), "fn main() {}").unwrap();

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_build_system(temp_dir.path(), None);
        assert!(result.is_ok());
        // 1.5 + 1.5 + 1.0 = 4.0, but capped at 3.0
        assert_eq!(result.unwrap(), 3.0);
    }

    #[test]
    fn test_no_build_system() {
        let temp_dir = TempDir::new().unwrap();

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_build_system(temp_dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0.0);
    }

    #[test]
    fn test_makefile_and_build_rs() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("Makefile"), "all:").unwrap();
        std::fs::write(temp_dir.path().join("build.rs"), "fn main() {}").unwrap();

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_build_system(temp_dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2.5); // 1.5 + 1.0
    }
}

// ==========================================================================
// Full Score Integration Tests
// ==========================================================================

mod integration_tests {
    use super::*;

    #[test]
    fn test_invalid_project_no_cargo_toml() {
        let temp_dir = TempDir::new().unwrap();

        let scorer = BuildPerfScorer::new();
        let result = scorer.score(temp_dir.path());
        assert!(result.is_err());
        if let Err(ScorerError::InvalidProject(msg)) = result {
            assert!(msg.contains("Cargo.toml"));
        } else {
            panic!("Expected InvalidProject error");
        }
    }

    #[test]
    fn test_minimal_project() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"
[package]
name = "test"
version = "0.1.0"
"#;
        create_cargo_toml(temp_dir.path(), content);

        let scorer = BuildPerfScorer::new();
        let result = scorer.score(temp_dir.path());
        assert!(result.is_ok());
        let score = result.unwrap();
        // Target dir (2.0) + Incremental default (1.5) = 3.5
        assert!(score.earned >= 3.0);
        assert!(score.earned <= 4.0);
    }

    #[test]
    fn test_well_configured_project() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"
[package]
name = "test"
version = "0.1.0"

[profile.release]
lto = true
codegen-units = 1
"#;
        create_cargo_toml(temp_dir.path(), content);
        std::fs::write(temp_dir.path().join("Cargo.lock"), "# lock").unwrap();
        std::fs::write(temp_dir.path().join("Makefile"), "all:").unwrap();

        let cargo_dir = temp_dir.path().join(".cargo");
        std::fs::create_dir(&cargo_dir).unwrap();
        std::fs::write(cargo_dir.join("config.toml"), "[build]\nincremental = true").unwrap();

        let scorer = BuildPerfScorer::new();
        let result = scorer.score(temp_dir.path());
        assert!(result.is_ok());
        let score = result.unwrap();
        // LTO (2) + Target (2) + Lock (2) + Config (2) + Incremental (2) + Codegen (2) + Make (1.5)
        assert!(score.earned >= 13.0);
    }

    #[test]
    fn test_score_with_mode_fast() {
        let temp_dir = TempDir::new().unwrap();
        let content = "[package]\nname = \"test\"";
        create_cargo_toml(temp_dir.path(), content);

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_with_mode(temp_dir.path(), ScoringMode::Fast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_score_with_mode_full() {
        let temp_dir = TempDir::new().unwrap();
        let content = "[package]\nname = \"test\"";
        create_cargo_toml(temp_dir.path(), content);

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_with_mode(temp_dir.path(), ScoringMode::Full);
        assert!(result.is_ok());
    }

    #[test]
    fn test_score_with_mode_quick() {
        let temp_dir = TempDir::new().unwrap();
        let content = "[package]\nname = \"test\"";
        create_cargo_toml(temp_dir.path(), content);

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_with_mode(temp_dir.path(), ScoringMode::Quick);
        assert!(result.is_ok());
    }

    #[test]
    fn test_score_with_cache() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");
        let content = r#"
[package]
name = "test"

[profile.release]
lto = true
codegen-units = 1
"#;
        std::fs::write(&cargo_toml_path, content).unwrap();

        let cache = create_test_cache(vec![(cargo_toml_path, content.to_string())]);

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_with_cache(temp_dir.path(), ScoringMode::Fast, Some(&cache));
        assert!(result.is_ok());
    }
}

// ==========================================================================
// Recommendations Tests
// ==========================================================================

mod recommendations_tests {
}

// ==========================================================================
// Thread Safety Tests
// ==========================================================================

mod thread_safety_tests {
    use super::*;

    #[test]
    fn test_scorer_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<BuildPerfScorer>();
    }

    #[test]
    fn test_scorer_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<BuildPerfScorer>();
    }

    #[test]
    fn test_concurrent_scoring() {
        let temp_dir = TempDir::new().unwrap();
        let content = "[package]\nname = \"test\"";
        create_cargo_toml(temp_dir.path(), content);

        let scorer = Arc::new(BuildPerfScorer::new());
        let path = temp_dir.path().to_path_buf();

        let handles: Vec<_> = (0..4)
            .map(|_| {
                let scorer = Arc::clone(&scorer);
                let path = path.clone();
                thread::spawn(move || scorer.score(&path))
            })
            .collect();

        for handle in handles {
            let result = handle.join().unwrap();
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_trait_object() {
        let scorer: Box<dyn Scorer> = Box::new(BuildPerfScorer::new());
        assert_eq!(scorer.name(), "Build Performance");
        assert_eq!(scorer.max_points(), 15.0);
    }
}

// ==========================================================================
// dir_size Helper Function Tests
// ==========================================================================

mod dir_size_tests {
    use super::*;

    #[test]
    fn test_dir_size_empty() {
        let temp_dir = TempDir::new().unwrap();
        let result = dir_size(temp_dir.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_dir_size_single_file() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("test.txt"), vec![0u8; 1000]).unwrap();

        let result = dir_size(temp_dir.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1000);
    }

    #[test]
    fn test_dir_size_multiple_files() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("file1.txt"), vec![0u8; 500]).unwrap();
        std::fs::write(temp_dir.path().join("file2.txt"), vec![0u8; 300]).unwrap();

        let result = dir_size(temp_dir.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 800);
    }

    #[test]
    fn test_dir_size_nested_directories() {
        let temp_dir = TempDir::new().unwrap();
        let nested = temp_dir.path().join("nested");
        std::fs::create_dir(&nested).unwrap();
        std::fs::write(temp_dir.path().join("root.txt"), vec![0u8; 100]).unwrap();
        std::fs::write(nested.join("child.txt"), vec![0u8; 200]).unwrap();

        let result = dir_size(temp_dir.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 300);
    }

    #[test]
    fn test_dir_size_deeply_nested() {
        let temp_dir = TempDir::new().unwrap();
        let deep = temp_dir.path().join("a").join("b").join("c");
        std::fs::create_dir_all(&deep).unwrap();
        std::fs::write(deep.join("deep.txt"), vec![0u8; 500]).unwrap();

        let result = dir_size(temp_dir.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 500);
    }

    #[test]
    fn test_dir_size_nonexistent_path() {
        let result = dir_size(Path::new("/nonexistent/path"));
        // Should return Ok(0) or error depending on implementation
        // Current implementation returns Ok(0) for non-directories
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_dir_size_file_path() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "content").unwrap();

        let result = dir_size(&file_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0); // Not a directory
    }
}

// ==========================================================================
// Edge Cases and Error Handling Tests
// ==========================================================================

mod edge_case_tests {
    use super::*;

    #[test]
    fn test_empty_cargo_toml() {
        let temp_dir = TempDir::new().unwrap();
        create_cargo_toml(temp_dir.path(), "");

        let scorer = BuildPerfScorer::new();
        let result = scorer.score(temp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_malformed_cargo_toml() {
        let temp_dir = TempDir::new().unwrap();
        create_cargo_toml(temp_dir.path(), "not valid toml {{{}}}");

        let scorer = BuildPerfScorer::new();
        // Should still work - we just do string matching
        let result = scorer.score(temp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_unicode_in_cargo_toml() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"
[package]
name = "测试项目"
version = "0.1.0"

[profile.release]
lto = true
"#;
        create_cargo_toml(temp_dir.path(), content);

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_lto(temp_dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2.0);
    }

    #[test]
    fn test_whitespace_variations_in_lto() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"
[package]
name = "test"

[profile.release]
lto   =   true
"#;
        create_cargo_toml(temp_dir.path(), content);

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_lto(temp_dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2.0);
    }

    #[test]
    fn test_comments_in_cargo_toml() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"
[package]
name = "test"

# Enable LTO for release builds
[profile.release]
# lto = false  # commented out
lto = true  # actual setting
"#;
        create_cargo_toml(temp_dir.path(), content);

        let scorer = BuildPerfScorer::new();
        let result = scorer.score_lto(temp_dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2.0);
    }

    #[test]
    fn test_cache_miss() {
        let temp_dir = TempDir::new().unwrap();
        let cache = FileCache::new(); // Empty cache

        let scorer = BuildPerfScorer::new();
        // With cache but file not in cache
        let result = scorer.score_lto(temp_dir.path(), Some(&cache));
        assert!(result.is_err());
    }

    #[test]
    fn test_max_score_possible() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"
[package]
name = "test"

[profile.release]
lto = true
codegen-units = 1
"#;
        create_cargo_toml(temp_dir.path(), content);
        std::fs::write(temp_dir.path().join("Cargo.lock"), "# lock").unwrap();
        std::fs::write(temp_dir.path().join("Makefile"), "all:").unwrap();
        std::fs::write(temp_dir.path().join("justfile"), "all:").unwrap();
        std::fs::write(temp_dir.path().join("build.rs"), "fn main() {}").unwrap();

        let cargo_dir = temp_dir.path().join(".cargo");
        std::fs::create_dir(&cargo_dir).unwrap();
        std::fs::write(cargo_dir.join("config.toml"), "[build]\nincremental = true").unwrap();

        let scorer = BuildPerfScorer::new();
        let result = scorer.score(temp_dir.path());
        assert!(result.is_ok());
        let score = result.unwrap();
        assert_eq!(score.max, 15.0);
        // LTO (2) + Target (2) + Lock (2) + Config (2) + Incremental (2) + Codegen (2) + Build (3)
        assert_eq!(score.earned, 15.0);
    }

    #[test]
    fn test_special_characters_in_path() {
        // Create temp dir with special characters manually is tricky
        // Test with a path that contains dashes and underscores
        let temp_dir = TempDir::new().unwrap();
        let special_dir = temp_dir.path().join("test-project_v1.0");
        std::fs::create_dir(&special_dir).unwrap();
        create_cargo_toml(&special_dir, "[package]\nname = \"test\"");

        let scorer = BuildPerfScorer::new();
        let result = scorer.score(&special_dir);
        assert!(result.is_ok());
    }
}

// ==========================================================================
// Category Score Integration Tests
// ==========================================================================

mod category_score_tests {
    use super::*;

    #[test]
    fn test_category_score_percentage() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"
[package]
name = "test"

[profile.release]
lto = true
"#;
        create_cargo_toml(temp_dir.path(), content);

        let scorer = BuildPerfScorer::new();
        let result = scorer.score(temp_dir.path());
        assert!(result.is_ok());
        let score = result.unwrap();
        let percentage = score.percentage();
        assert!(percentage >= 0.0);
        assert!(percentage <= 100.0);
    }

    #[test]
    fn test_category_score_is_perfect() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"
[package]
name = "test"

[profile.release]
lto = true
codegen-units = 1
"#;
        create_cargo_toml(temp_dir.path(), content);
        std::fs::write(temp_dir.path().join("Cargo.lock"), "# lock").unwrap();
        std::fs::write(temp_dir.path().join("Makefile"), "all:").unwrap();
        std::fs::write(temp_dir.path().join("justfile"), "all:").unwrap();

        let cargo_dir = temp_dir.path().join(".cargo");
        std::fs::create_dir(&cargo_dir).unwrap();
        std::fs::write(cargo_dir.join("config.toml"), "[build]\nincremental = true").unwrap();

        let scorer = BuildPerfScorer::new();
        let result = scorer.score(temp_dir.path());
        assert!(result.is_ok());
        let score = result.unwrap();
        assert!(score.is_perfect());
    }

    #[test]
    fn test_category_score_not_perfect() {
        let temp_dir = TempDir::new().unwrap();
        let content = "[package]\nname = \"test\"";
        create_cargo_toml(temp_dir.path(), content);

        let scorer = BuildPerfScorer::new();
        let result = scorer.score(temp_dir.path());
        assert!(result.is_ok());
        let score = result.unwrap();
        assert!(!score.is_perfect());
    }
}
