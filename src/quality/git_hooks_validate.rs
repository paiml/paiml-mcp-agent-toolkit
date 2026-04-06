// Validation, incremental checking, and CI config generation
// included from git_hooks.rs — no `use` imports or `#!` attributes

impl GitHookManager {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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

impl Default for IncrementalChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl IncrementalChecker {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn should_check(&self, file_path: &Path) -> Result<bool> {
        debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
        let metadata = fs::metadata(file_path)?;
        let modified = metadata.modified()?;

        if let Some(cached) = self.cache.get(file_path.to_str().unwrap_or("")) {
            Ok(modified > cached.last_checked)
        } else {
            Ok(true)
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn update_cache(&mut self, file_path: &Path, passed: bool) -> Result<()> {
        debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
