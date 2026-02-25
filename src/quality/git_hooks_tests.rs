// Tests for git_hooks module
// included from git_hooks.rs — no `use` imports or `#!` attributes

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_hook_installation() {
        let temp_dir = TempDir::new().unwrap();
        let git_dir = temp_dir.path().join(".git/hooks");
        fs::create_dir_all(&git_dir).unwrap();

        let manager = GitHookManager::new(temp_dir.path());
        manager.install_hooks().unwrap();

        assert!(git_dir.join("pre-commit").exists());
        assert!(git_dir.join("commit-msg").exists());
        assert!(git_dir.join("pre-push").exists());
    }

    #[test]
    fn test_incremental_checker() {
        let mut checker = IncrementalChecker::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");

        fs::write(&file_path, "fn test() {}").unwrap();

        assert!(checker.should_check(&file_path).unwrap());

        checker.update_cache(&file_path, true).unwrap();

        // Should not need check if not modified
        assert!(!checker.should_check(&file_path).unwrap());
    }

    #[test]
    fn test_git_hook_manager_new() {
        let temp_dir = TempDir::new().unwrap();
        let manager = GitHookManager::new(temp_dir.path());
        assert_eq!(manager.repo_path, temp_dir.path());
    }

    #[test]
    fn test_git_hook_manager_from_string_path() {
        let manager = GitHookManager::new("/tmp/test_repo");
        assert_eq!(manager.repo_path, PathBuf::from("/tmp/test_repo"));
    }

    #[test]
    fn test_install_pre_commit_hook() {
        let temp_dir = TempDir::new().unwrap();
        let hooks_dir = temp_dir.path().join(".git/hooks");
        fs::create_dir_all(&hooks_dir).unwrap();

        let manager = GitHookManager::new(temp_dir.path());
        manager.install_pre_commit_hook(&hooks_dir).unwrap();

        let hook_path = hooks_dir.join("pre-commit");
        assert!(hook_path.exists());

        let content = fs::read_to_string(&hook_path).unwrap();
        assert!(content.contains("#!/usr/bin/env bash"));
        assert!(content.contains("PMAT Quality Gate"));
        assert!(content.contains("cargo test"));
    }

    #[test]
    fn test_install_commit_msg_hook() {
        let temp_dir = TempDir::new().unwrap();
        let hooks_dir = temp_dir.path().join(".git/hooks");
        fs::create_dir_all(&hooks_dir).unwrap();

        let manager = GitHookManager::new(temp_dir.path());
        manager.install_commit_msg_hook(&hooks_dir).unwrap();

        let hook_path = hooks_dir.join("commit-msg");
        assert!(hook_path.exists());

        let content = fs::read_to_string(&hook_path).unwrap();
        assert!(content.contains("COMMIT_MSG_FILE"));
        assert!(content.contains("feat:|fix:|docs:"));
    }

    #[test]
    fn test_install_pre_push_hook() {
        let temp_dir = TempDir::new().unwrap();
        let hooks_dir = temp_dir.path().join(".git/hooks");
        fs::create_dir_all(&hooks_dir).unwrap();

        let manager = GitHookManager::new(temp_dir.path());
        manager.install_pre_push_hook(&hooks_dir).unwrap();

        let hook_path = hooks_dir.join("pre-push");
        assert!(hook_path.exists());

        let content = fs::read_to_string(&hook_path).unwrap();
        assert!(content.contains("cargo build --release"));
        assert!(content.contains("cargo clippy"));
    }

    #[test]
    fn test_hook_installation_creates_hooks_dir() {
        let temp_dir = TempDir::new().unwrap();
        // Only create .git, not hooks
        fs::create_dir_all(temp_dir.path().join(".git")).unwrap();

        let manager = GitHookManager::new(temp_dir.path());
        manager.install_hooks().unwrap();

        let hooks_dir = temp_dir.path().join(".git/hooks");
        assert!(hooks_dir.exists());
    }

    #[test]
    fn test_quality_report_struct() {
        let report = QualityReport {
            file: "src/main.rs".to_string(),
            passed: true,
            violations: vec![],
        };
        assert_eq!(report.file, "src/main.rs");
        assert!(report.passed);
        assert!(report.violations.is_empty());
    }

    #[test]
    fn test_quality_report_with_violations() {
        let report = QualityReport {
            file: "src/lib.rs".to_string(),
            passed: false,
            violations: vec![
                "Complexity too high".to_string(),
                "Missing tests".to_string(),
            ],
        };
        assert!(!report.passed);
        assert_eq!(report.violations.len(), 2);
    }

    #[test]
    fn test_incremental_checker_default() {
        let checker = IncrementalChecker::default();
        assert!(checker.cache.is_empty());
    }

    #[test]
    fn test_incremental_checker_new_file() {
        let checker = IncrementalChecker::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("new_file.rs");

        fs::write(&file_path, "fn main() {}").unwrap();

        // New file should always need check
        assert!(checker.should_check(&file_path).unwrap());
    }

    #[test]
    fn test_incremental_checker_update_and_check() {
        let mut checker = IncrementalChecker::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("checked.rs");

        fs::write(&file_path, "fn test_fn() {}").unwrap();

        // Update cache
        checker.update_cache(&file_path, true).unwrap();

        // File not modified - should not need check
        assert!(!checker.should_check(&file_path).unwrap());

        // Modify file
        std::thread::sleep(std::time::Duration::from_millis(10));
        fs::write(&file_path, "fn test_fn() { println!(\"modified\"); }").unwrap();

        // Modified file should need check
        assert!(checker.should_check(&file_path).unwrap());
    }

    #[test]
    fn test_incremental_checker_nonexistent_file() {
        let checker = IncrementalChecker::new();
        let result = checker.should_check(Path::new("/nonexistent/file.rs"));
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_ci_config() {
        let config = generate_ci_config();

        assert!(config.contains("name: PMAT Quality Gates"));
        assert!(config.contains("cargo build --release"));
        assert!(config.contains("cargo test --all-features"));
        assert!(config.contains("cargo clippy -- -D warnings"));
        assert!(config.contains("cargo fmt -- --check"));
        assert!(config.contains("pmat analyze complexity"));
        assert!(config.contains("pmat analyze satd"));
    }

    #[test]
    fn test_generate_ci_config_has_coverage() {
        let config = generate_ci_config();

        assert!(config.contains("cargo-llvm-cov"));
        assert!(config.contains("codecov/codecov-action"));
    }

    #[test]
    fn test_generate_ci_config_triggers() {
        let config = generate_ci_config();

        assert!(config.contains("on: [push, pull_request]"));
    }

    #[test]
    fn test_quality_report_debug() {
        let report = QualityReport {
            file: "test.rs".to_string(),
            passed: true,
            violations: vec![],
        };
        let debug = format!("{:?}", report);
        assert!(debug.contains("QualityReport"));
        assert!(debug.contains("test.rs"));
    }

    #[test]
    fn test_hooks_are_executable() {
        let temp_dir = TempDir::new().unwrap();
        let hooks_dir = temp_dir.path().join(".git/hooks");
        fs::create_dir_all(&hooks_dir).unwrap();

        let manager = GitHookManager::new(temp_dir.path());
        manager.install_hooks().unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let pre_commit = hooks_dir.join("pre-commit");
            let perms = fs::metadata(&pre_commit).unwrap().permissions();
            assert!(perms.mode() & 0o111 != 0, "pre-commit should be executable");

            let commit_msg = hooks_dir.join("commit-msg");
            let perms = fs::metadata(&commit_msg).unwrap().permissions();
            assert!(perms.mode() & 0o111 != 0, "commit-msg should be executable");

            let pre_push = hooks_dir.join("pre-push");
            let perms = fs::metadata(&pre_push).unwrap().permissions();
            assert!(perms.mode() & 0o111 != 0, "pre-push should be executable");
        }
    }

    #[test]
    fn test_validate_staged_files_no_git() {
        let temp_dir = TempDir::new().unwrap();
        let manager = GitHookManager::new(temp_dir.path());

        // This may fail or succeed depending on git availability
        // We're just testing it doesn't panic
        let _ = manager.validate_staged_files();
    }

    #[test]
    fn test_run_pre_commit_checks() {
        let temp_dir = TempDir::new().unwrap();
        let manager = GitHookManager::new(temp_dir.path());

        // May fail without git, just ensure no panic
        let _ = manager.run_pre_commit_checks();
    }

    #[test]
    fn test_file_checksum_clone() {
        let checksum = FileChecksum {
            _hash: "abc123".to_string(),
            last_checked: std::time::SystemTime::now(),
            _passed: true,
        };
        let cloned = checksum.clone();
        assert_eq!(checksum._hash, cloned._hash);
        assert_eq!(checksum._passed, cloned._passed);
    }

    #[test]
    fn test_incremental_checker_multiple_files() {
        let mut checker = IncrementalChecker::new();
        let temp_dir = TempDir::new().unwrap();

        let file1 = temp_dir.path().join("file1.rs");
        let file2 = temp_dir.path().join("file2.rs");

        fs::write(&file1, "fn f1() {}").unwrap();
        fs::write(&file2, "fn f2() {}").unwrap();

        checker.update_cache(&file1, true).unwrap();
        checker.update_cache(&file2, false).unwrap();

        assert_eq!(checker.cache.len(), 2);
    }
}
