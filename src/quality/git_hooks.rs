#![cfg_attr(coverage_nightly, coverage(off))]
use crate::quality::gate::QualityGateRunner;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct GitHookManager {
    repo_path: PathBuf,
    quality_runner: QualityGateRunner,
}

impl GitHookManager {
    pub fn new(repo_path: impl AsRef<Path>) -> Self {
        Self {
            repo_path: repo_path.as_ref().to_path_buf(),
            quality_runner: QualityGateRunner::strict(),
        }
    }

    pub fn install_hooks(&self) -> Result<()> {
        let hooks_dir = self.repo_path.join(".git/hooks");

        if !hooks_dir.exists() {
            fs::create_dir_all(&hooks_dir).context("Failed to create hooks directory")?;
        }

        // Install pre-commit hook
        self.install_pre_commit_hook(&hooks_dir)?;

        // Install commit-msg hook
        self.install_commit_msg_hook(&hooks_dir)?;

        // Install pre-push hook
        self.install_pre_push_hook(&hooks_dir)?;

        println!("✅ Git hooks installed successfully");
        Ok(())
    }

    fn install_pre_commit_hook(&self, hooks_dir: &Path) -> Result<()> {
        let hook_path = hooks_dir.join("pre-commit");

        let hook_content = r#"#!/usr/bin/env bash
# PMAT Quality Gate Pre-Commit Hook

set -e

echo "🔍 Running PMAT quality gates..."

# Get staged Rust files
STAGED_FILES=$(git diff --cached --name-only --diff-filter=ACM | grep -E '\.(rs)$' || true)

if [ -z "$STAGED_FILES" ]; then
    echo "No Rust files to check"
    exit 0
fi

# Run quality checks on each file
for FILE in $STAGED_FILES; do
    echo "Checking: $FILE"

    # Run PMAT quality analysis
    pmat quality-gate "$FILE" || {
        echo "❌ Quality gate failed for $FILE"
        echo "Fix quality violations before committing."
        exit 1
    }
done

# Run tests
echo "Running tests..."
cargo test --quiet || {
    echo "❌ Tests failed"
    exit 1
}

# Check for SATD
SATD_COUNT=$(grep -r "TODO\|FIXME\|HACK\|XXX" --include="*.rs" . | wc -l)
if [ "$SATD_COUNT" -gt 0 ]; then
    echo "❌ Found $SATD_COUNT SATD markers (TODO, FIXME, HACK, XXX)"
    echo "Zero tolerance for technical debt!"
    exit 1
fi

echo "✅ All quality gates passed!"
"#;

        let mut file = fs::File::create(&hook_path).context("Failed to create pre-commit hook")?;

        file.write_all(hook_content.as_bytes())
            .context("Failed to write pre-commit hook")?;

        // Make executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&hook_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&hook_path, perms)?;
        }

        Ok(())
    }

    fn install_commit_msg_hook(&self, hooks_dir: &Path) -> Result<()> {
        let hook_path = hooks_dir.join("commit-msg");

        let hook_content = r#"#!/usr/bin/env bash
# PMAT Commit Message Quality Hook

COMMIT_MSG_FILE=$1
COMMIT_MSG=$(cat "$COMMIT_MSG_FILE")

# Check commit message format
if ! echo "$COMMIT_MSG" | grep -qE "^(PMAT-[0-9]+:|feat:|fix:|docs:|style:|refactor:|test:|chore:)"; then
    echo "❌ Invalid commit message format"
    echo "Commit message must start with:"
    echo "  - PMAT-XXX: for ticket work"
    echo "  - feat: for new features"
    echo "  - fix: for bug fixes"
    echo "  - docs: for documentation"
    echo "  - test: for tests"
    echo "  - refactor: for refactoring"
    exit 1
fi

# Check minimum length
if [ ${#COMMIT_MSG} -lt 10 ]; then
    echo "❌ Commit message too short"
    exit 1
fi

echo "✅ Commit message format valid"
"#;

        let mut file = fs::File::create(&hook_path)?;
        file.write_all(hook_content.as_bytes())?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&hook_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&hook_path, perms)?;
        }

        Ok(())
    }

    fn install_pre_push_hook(&self, hooks_dir: &Path) -> Result<()> {
        let hook_path = hooks_dir.join("pre-push");

        let hook_content = r#"#!/usr/bin/env bash
# PMAT Pre-Push Quality Verification

echo "🚀 Running pre-push quality verification..."

# Build in release mode
cargo build --release || {
    echo "❌ Release build failed"
    exit 1
}

# Run all tests
cargo test --all-features || {
    echo "❌ Tests failed"
    exit 1
}

# Run clippy
cargo clippy -- -D warnings || {
    echo "❌ Clippy warnings found"
    exit 1
}

# Check format
cargo fmt -- --check || {
    echo "❌ Code not formatted"
    echo "Run 'cargo fmt' to fix"
    exit 1
}

# Verify coverage
if command -v cargo-llvm-cov &> /dev/null; then
    COVERAGE=$(cargo llvm-cov report --summary-only | grep "TOTAL" | awk '{print $10}' | sed 's/%//')
    if (( $(echo "$COVERAGE < 95.0" | bc -l) )); then
        echo "❌ Coverage $COVERAGE% is below 95%"
        exit 1
    fi
fi

echo "✅ All pre-push checks passed!"
"#;

        let mut file = fs::File::create(&hook_path)?;
        file.write_all(hook_content.as_bytes())?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&hook_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&hook_path, perms)?;
        }

        Ok(())
    }

    pub fn validate_staged_files(&self) -> Result<Vec<QualityReport>> {
        let output = Command::new("git")
            .args(["diff", "--cached", "--name-only", "--diff-filter=ACM"])
            .output()
            .context("Failed to get staged files")?;

        let staged_files = String::from_utf8_lossy(&output.stdout);
        let mut reports = Vec::new();

        for file_path in staged_files.lines() {
            if file_path.ends_with(".rs") {
                let path = Path::new(file_path);
                match self.quality_runner.validate_module(path) {
                    Ok(_report) => reports.push(QualityReport {
                        file: file_path.to_string(),
                        passed: true,
                        violations: Vec::new(),
                    }),
                    Err(violation) => reports.push(QualityReport {
                        file: file_path.to_string(),
                        passed: false,
                        violations: vec![violation.to_string()],
                    }),
                }
            }
        }

        Ok(reports)
    }

    pub fn run_pre_commit_checks(&self) -> Result<bool> {
        let reports = self.validate_staged_files()?;

        let all_passed = reports.iter().all(|r| r.passed);

        if !all_passed {
            println!("❌ Quality gate violations found:");
            for report in reports.iter().filter(|r| !r.passed) {
                println!("  File: {}", report.file);
                for violation in &report.violations {
                    println!("    - {}", violation);
                }
            }
        }

        Ok(all_passed)
    }
}

#[derive(Debug)]
pub struct QualityReport {
    pub file: String,
    pub passed: bool,
    pub violations: Vec<String>,
}

pub struct IncrementalChecker {
    cache: HashMap<String, FileChecksum>,
}

#[derive(Debug, Clone)]
struct FileChecksum {
    _hash: String,
    last_checked: std::time::SystemTime,
    _passed: bool,
}

impl Default for IncrementalChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl IncrementalChecker {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn should_check(&self, file_path: &Path) -> Result<bool> {
        let metadata = fs::metadata(file_path)?;
        let modified = metadata.modified()?;

        if let Some(cached) = self.cache.get(file_path.to_str().unwrap_or("")) {
            Ok(modified > cached.last_checked)
        } else {
            Ok(true)
        }
    }

    pub fn update_cache(&mut self, file_path: &Path, passed: bool) -> Result<()> {
        use sha2::{Digest, Sha256};

        let content = fs::read_to_string(file_path)?;
        let hash = format!("{:x}", Sha256::digest(content.as_bytes()));

        self.cache.insert(
            file_path.to_str().unwrap_or("").to_string(),
            FileChecksum {
                _hash: hash,
                last_checked: std::time::SystemTime::now(),
                _passed: passed,
            },
        );

        Ok(())
    }
}

// Integration with CI/CD
pub fn generate_ci_config() -> String {
    r#"name: PMAT Quality Gates
on: [push, pull_request]

jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: rustfmt, clippy

      - name: Quality Gate Check
        run: |
          cargo build --release
          cargo test --all-features
          cargo clippy -- -D warnings
          cargo fmt -- --check

      - name: Complexity Analysis
        run: pmat analyze complexity --max 10

      - name: SATD Detection
        run: pmat analyze satd --zero-tolerance

      - name: Coverage Check
        run: |
          cargo install cargo-llvm-cov
          cargo llvm-cov test --no-report
          cargo llvm-cov report --lcov --output-path coverage/lcov.info

      - name: Upload Coverage
        uses: codecov/codecov-action@v3
        with:
          files: ./coverage/cobertura.xml
"#
    .to_string()
}

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
