#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::scaffold::config::AgentFramework;
    use crate::scaffold::config::WasmFramework;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_generate_pforge_hook() {
        let config = HookConfig {
            project_type: TemplateType::Agent {
                based_on: AgentFramework::Pforge,
            },
            quality_gates: QualityGateConfig::default(),
        };

        let script = generate_pre_commit_hook(&config).unwrap();

        assert!(script.contains("#!/bin/bash"));
        assert!(script.contains("cargo clippy"));
        assert!(script.contains("cargo test"));
        assert!(script.contains("pforge project"));
    }

    #[test]
    fn test_generate_wasm_hook() {
        let config = HookConfig {
            project_type: TemplateType::Wasm {
                based_on: WasmFramework::WasmLabs,
            },
            quality_gates: QualityGateConfig::default(),
        };

        let script = generate_pre_commit_hook(&config).unwrap();

        assert!(script.contains("#!/bin/bash"));
        assert!(script.contains("cargo clippy"));
        assert!(script.contains("cargo test"));
        assert!(script.contains("wasm32-unknown-unknown"));
        assert!(script.contains("WASM project"));
    }

    #[test]
    fn test_install_pre_commit_hook() {
        let temp_dir = TempDir::new().unwrap();
        let git_dir = temp_dir.path().join(".git/hooks");
        fs::create_dir_all(&git_dir).unwrap();

        let script = "#!/bin/bash\necho test";
        install_pre_commit_hook(temp_dir.path(), script).unwrap();

        let hook_path = git_dir.join("pre-commit");
        assert!(hook_path.exists());

        let content = fs::read_to_string(&hook_path).unwrap();
        assert_eq!(content, script);

        // Check executable permission on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::metadata(&hook_path).unwrap().permissions();
            assert!(perms.mode() & 0o111 != 0, "Hook should be executable");
        }
    }

    #[test]
    fn test_hook_script_format() {
        let config = HookConfig {
            project_type: TemplateType::Agent {
                based_on: AgentFramework::Pforge,
            },
            quality_gates: QualityGateConfig::extreme_tdd(),
        };

        let script = generate_pre_commit_hook(&config).unwrap();
        assert!(script.contains("All quality gates passed"));
        assert!(script.contains("set -e")); // Fail fast
    }

    #[test]
    fn test_generate_post_commit_hook() {
        let hook = generate_post_commit_hook();

        assert!(hook.starts_with("#!/bin/bash"));
        assert!(hook.contains("TICKET-PMAT-"));
        assert!(hook.contains("pmat maintain update-roadmap"));
    }

    #[test]
    fn test_install_post_commit_hook() {
        let temp_dir = TempDir::new().unwrap();
        let git_dir = temp_dir.path().join(".git/hooks");
        fs::create_dir_all(&git_dir).unwrap();

        install_post_commit_hook(temp_dir.path()).unwrap();

        let hook_path = git_dir.join("post-commit");
        assert!(hook_path.exists());

        let content = fs::read_to_string(&hook_path).unwrap();
        assert!(content.contains("#!/bin/bash"));
        assert!(content.contains("TICKET-PMAT-"));

        // Check executable permission on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::metadata(&hook_path).unwrap().permissions();
            assert!(perms.mode() & 0o111 != 0, "Hook should be executable");
        }
    }

    #[test]
    fn test_post_commit_hook_silent_failure() {
        let hook = generate_post_commit_hook();
        // Property: Hook should not block commits on failure
        assert!(hook.contains("|| true"));
        assert!(hook.contains("2>/dev/null"));
    }

    // TICKET-PMAT-5021 tests
    #[test]
    fn test_generate_hook_with_gates_fast() {
        let hook = generate_pre_commit_hook_with_gates_fast();

        assert!(hook.starts_with("#!/bin/bash"));
        assert!(hook.contains("TICKET-PMAT-5021"));
        assert!(hook.contains("cargo clippy"));
        assert!(hook.contains("cargo test"));
        assert!(hook.contains("fast mode"));
        assert!(hook.contains("--no-verify"));
    }

    #[test]
    fn test_generate_gate_config_toml() {
        let toml = generate_gate_config_toml();

        assert!(toml.contains("run_clippy = true"));
        assert!(toml.contains("min_coverage = 80.0"));
        assert!(toml.contains("max_complexity = 10"));
        assert!(toml.contains("[gates]"));
    }

    #[test]
    fn test_install_gate_config() {
        let temp_dir = TempDir::new().unwrap();

        install_gate_config(temp_dir.path()).unwrap();

        let config_path = temp_dir.path().join(".pmat-gates.toml");
        assert!(config_path.exists());

        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("run_clippy"));
        assert!(content.contains("min_coverage"));
    }

    #[test]
    fn test_install_all_hooks_with_gates() {
        let temp_dir = TempDir::new().unwrap();
        let git_dir = temp_dir.path().join(".git/hooks");
        fs::create_dir_all(&git_dir).unwrap();

        install_all_hooks_with_gates(temp_dir.path()).unwrap();

        // Check pre-commit hook
        let pre_commit = git_dir.join("pre-commit");
        assert!(pre_commit.exists());
        let content = fs::read_to_string(&pre_commit).unwrap();
        assert!(content.contains("fast mode"));

        // Check post-commit hook
        let post_commit = git_dir.join("post-commit");
        assert!(post_commit.exists());

        // Check gate config
        let config = temp_dir.path().join(".pmat-gates.toml");
        assert!(config.exists());

        // Check executability
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::metadata(&pre_commit).unwrap().permissions();
            assert!(perms.mode() & 0o111 != 0, "Pre-commit should be executable");
        }
    }

    #[test]
    fn test_hook_bypass_documentation() {
        let hook = generate_pre_commit_hook_with_gates_fast();
        assert!(hook.contains("--no-verify"));
        assert!(hook.contains("To bypass:"));
    }

    #[test]
    fn test_hook_execution_time_hint() {
        let hook = generate_pre_commit_hook_with_gates_fast();
        assert!(hook.contains("<30s") || hook.contains("fast"));
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use super::*;
    use crate::scaffold::config::{AgentFramework, WasmFramework};
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_hooks_are_valid_bash(is_agent in any::<bool>()) {
            let project_type = if is_agent {
                TemplateType::Agent { based_on: AgentFramework::Pforge }
            } else {
                TemplateType::Wasm { based_on: WasmFramework::WasmLabs }
            };

            let config = HookConfig {
                project_type,
                quality_gates: QualityGateConfig::default(),
            };

            let script = generate_pre_commit_hook(&config).unwrap();

            // Property: All hooks are valid bash scripts
            prop_assert!(script.starts_with("#!/bin/bash"));
            prop_assert!(script.contains("set -e"));
            prop_assert!(script.contains("exit 0"));
        }

        #[test]
        fn prop_hooks_check_clippy_and_tests(is_agent in any::<bool>()) {
            let project_type = if is_agent {
                TemplateType::Agent { based_on: AgentFramework::Pforge }
            } else {
                TemplateType::Wasm { based_on: WasmFramework::WasmLabs }
            };

            let config = HookConfig {
                project_type,
                quality_gates: QualityGateConfig::default(),
            };

            let script = generate_pre_commit_hook(&config).unwrap();

            // Property: All hooks enforce clippy and tests
            prop_assert!(script.contains("cargo clippy"));
            prop_assert!(script.contains("cargo test"));
        }
    }

    #[test]
    fn prop_wasm_hooks_verify_wasm_build() {
        // Simple property test (not using proptest macro)
        let config = HookConfig {
            project_type: TemplateType::Wasm {
                based_on: WasmFramework::WasmLabs,
            },
            quality_gates: QualityGateConfig::default(),
        };

        let script = generate_pre_commit_hook(&config).unwrap();

        // Property: WASM hooks verify WASM builds
        assert!(script.contains("wasm32-unknown-unknown"));
    }
}
