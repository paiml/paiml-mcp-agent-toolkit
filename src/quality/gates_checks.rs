// Individual quality gate check implementations.
// Contains: execute_clippy, execute_tests, execute_coverage, execute_complexity,
// and coverage artifact cleanup helpers.

/// Pure-compute message builder extracted from execute_clippy for R5 testability.
/// Builds the clippy result message from the success flag and stderr capture.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
fn build_clippy_message(passed: bool, stderr: &str) -> String {
    if passed {
        "✓ Clippy passed".to_string()
    } else {
        format!(
            "✗ Clippy failed:\n{}",
            stderr.lines().take(10).collect::<Vec<_>>().join("\n")
        )
    }
}

/// Pure-compute message builder extracted from execute_tests for R5 testability.
/// Selects between stdout failure-line filter and stderr fallback based on which
/// has more signal. Pinned behavior: filters lines containing FAILED/panicked/
/// error[/failures:/indented-thread/qualified-path patterns.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
fn build_test_message(passed: bool, stdout: &str, stderr: &str) -> String {
    if passed {
        return "✓ Tests passed".to_string();
    }
    let failure_lines: Vec<&str> = stdout
        .lines()
        .filter(|line| {
            line.contains("FAILED")
                || line.contains("panicked")
                || line.contains("error[")
                || line.starts_with("failures:")
                || line.starts_with("    ")
                    && (line.contains("::") || line.trim().starts_with("thread"))
        })
        .take(15)
        .collect();
    if !failure_lines.is_empty() {
        format!("✗ Tests failed:\n{}", failure_lines.join("\n"))
    } else {
        format!(
            "✗ Tests failed:\n{}",
            stderr.lines().take(10).collect::<Vec<_>>().join("\n")
        )
    }
}

/// Pure-compute decision builder extracted from execute_coverage for R5 testability.
/// Returns (passed, message) given the parsed coverage % and the configured threshold.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
fn build_coverage_decision(coverage: f64, min_coverage: f64) -> (bool, String) {
    let passed = coverage >= min_coverage;
    let message = if passed {
        format!("✓ Coverage: {:.1}% (>= {:.1}%)", coverage, min_coverage)
    } else {
        format!("✗ Coverage: {:.1}% (< {:.1}%)", coverage, min_coverage)
    };
    (passed, message)
}

/// Execute clippy gate
///
/// # Complexity
/// - Time: O(codebase size)
/// - Cyclomatic: 4
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn execute_clippy(config: &GateConfig, project_dir: &Path) -> Result<GateResult> {
    use std::time::Instant;

    let start = Instant::now();
    let mut cmd = Command::new("cargo");
    cmd.arg("clippy")
        .arg("--lib") // Avoid --all-targets which causes module duplication with include!()
        .current_dir(project_dir);

    if config.clippy_strict {
        cmd.arg("--").arg("-D").arg("warnings");
    }

    let output = cmd.output()?;
    let duration = start.elapsed();

    let passed = output.status.success();
    let stderr = String::from_utf8_lossy(&output.stderr);
    let message = build_clippy_message(passed, &stderr);

    Ok(GateResult {
        name: "clippy".to_string(),
        passed,
        duration,
        message,
    })
}

/// Execute test gate
///
/// Runs `cargo test --lib` to test library code only. This matches user
/// expectations when they say "tests pass" (typically meaning unit tests).
/// Integration tests, doc tests, and examples are excluded for reliability.
///
/// # Issue #143 Fix
/// Previously ran `cargo test --all-features` which included doc tests,
/// integration tests, etc. that could fail independently of the main test suite.
/// Now uses `--lib` flag to match typical user workflow.
///
/// # Complexity
/// - Time: O(test suite size)
/// - Cyclomatic: 3
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn execute_tests(config: &GateConfig, project_dir: &Path) -> Result<GateResult> {
    use std::time::Instant;

    let start = Instant::now();
    let output = Command::new("cargo")
        .arg("test")
        .arg("--lib")
        .env("RUST_MIN_STACK", "33554432") // 32MB stack for clap parsing tests
        .current_dir(project_dir)
        .output()?;
    let duration = start.elapsed();

    // Check timeout
    if duration.as_secs() > config.test_timeout {
        return Err(GateError::Timeout(config.test_timeout));
    }

    let passed = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let message = build_test_message(passed, &stdout, &stderr);

    Ok(GateResult {
        name: "tests".to_string(),
        passed,
        duration,
        message,
    })
}

/// Execute coverage gate
///
/// # Complexity
/// - Time: O(codebase size)
/// - Cyclomatic: 5
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn execute_coverage(config: &GateConfig, project_dir: &Path) -> Result<GateResult> {
    use std::time::Instant;

    let start = Instant::now();

    // Run cargo llvm-cov
    // Coverage requires nightly for #[coverage(off)] attribute support
    let output = Command::new("cargo")
        .arg("+nightly")
        .arg("llvm-cov")
        .arg("--lib")
        .arg("--summary-only")
        .env("RUST_MIN_STACK", "33554432") // 32MB stack for clap parsing tests
        .current_dir(project_dir)
        .output()?;
    let duration = start.elapsed();

    // Clean up coverage artifacts to prevent zram bloat (TICKET-PMAT-9)
    cleanup_coverage_artifacts(project_dir);

    // Try to parse coverage even on test failure (flaky tests shouldn't block coverage)
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let coverage = parse_coverage_from_output(&stdout);

    // If no coverage data found AND exit was non-zero, report failure
    if coverage == 0.0 && !output.status.success() {
        let err_snippet = stderr.lines().rev().take(3).collect::<Vec<_>>();
        return Ok(GateResult {
            name: "coverage".to_string(),
            passed: false,
            duration,
            message: format!("✗ Coverage check failed to run: {}", err_snippet.join(" | ")),
        });
    }

    let (passed, message) = build_coverage_decision(coverage, config.min_coverage);

    Ok(GateResult {
        name: "coverage".to_string(),
        passed,
        duration,
        message,
    })
}

/// Clean up coverage artifacts to prevent memory bloat
///
/// Removes stale llvm-cov-target directories and cleans zram cache.
/// This prevents the issue documented in TICKET-PMAT-9 where coverage
/// artifacts in /mnt/zram accumulated to 70GB+ consuming RAM.
///
/// # Complexity
/// - Time: O(n) where n is number of files to clean
/// - Cyclomatic: 3
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
fn cleanup_coverage_artifacts(project_dir: &Path) {
    // Clean llvm-cov-target in project dir
    let llvm_cov_target = project_dir.join("target").join("llvm-cov-target");
    if llvm_cov_target.exists() {
        let _ = std::fs::remove_dir_all(&llvm_cov_target);
    }

    // Clean zram coverage cache if it exists (>1 hour old)
    let zram_coverage = Path::new("/mnt/zram/coverage");
    if zram_coverage.exists() {
        clean_old_files(zram_coverage, 3600); // 1 hour
    }

    // Clean zram targets cache if it exists (>1 hour old)
    let zram_targets = Path::new("/mnt/zram/targets");
    if zram_targets.exists() {
        clean_old_files(zram_targets, 3600); // 1 hour
    }
}

/// Remove files older than max_age_secs from a directory
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
fn clean_old_files(dir: &Path, max_age_secs: u64) {
    use std::time::{Duration, SystemTime};

    let max_age = Duration::from_secs(max_age_secs);
    let now = SystemTime::now();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                let should_delete = metadata
                    .modified()
                    .ok()
                    .and_then(|mtime| now.duration_since(mtime).ok())
                    .is_some_and(|age| age > max_age);

                if should_delete {
                    let path = entry.path();
                    if path.is_dir() {
                        let _ = std::fs::remove_dir_all(&path);
                    } else {
                        let _ = std::fs::remove_file(&path);
                    }
                }
            }
        }
    }
}

/// Parse coverage percentage from llvm-cov output
///
/// # Complexity
/// - Time: O(n) where n is output length
/// - Cyclomatic: 4
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
fn parse_coverage_from_output(output: &str) -> f64 {
    // Look for "TOTAL.*X.XX%"
    for line in output.lines() {
        if line.contains("TOTAL") {
            if let Some(pct) = line
                .split_whitespace()
                .find(|s| s.ends_with('%'))
                .and_then(|s| s.trim_end_matches('%').parse::<f64>().ok())
            {
                return pct;
            }
        }
    }
    0.0
}

/// What the complexity gate actually measured over a project tree.
///
/// `worst` is `(file, function, cyclomatic)` for the highest-scoring function
/// found; `None` means nothing was measurable.
#[derive(Debug, Default, PartialEq)]
pub struct ComplexityScan {
    /// Rust files discovered under the project directory.
    pub files_seen: usize,
    /// Rust files that parsed and were actually measured.
    pub files_measured: usize,
    /// Functions measured across those files.
    pub functions_measured: usize,
    /// Highest-complexity function seen: (file, function, cyclomatic).
    pub worst: Option<(String, String, u32)>,
}

/// Per-function cyclomatic collector for the complexity gate.
///
/// Visits free functions and inherent/trait impl methods, the same two shapes
/// `analyze complexity` counts, and delegates the count itself to
/// `measure_block` so the gate cannot drift away from the analyzer.
struct GateComplexityVisitor {
    functions: Vec<(String, u32)>,
}

impl<'ast> syn::visit::Visit<'ast> for GateComplexityVisitor {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let name = node.sig.ident.to_string();
        let measured =
            crate::services::accurate_complexity_analyzer::measure_block(&name, &node.block);
        self.functions.push((name, measured.cyclomatic));
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        let name = node.sig.ident.to_string();
        let measured =
            crate::services::accurate_complexity_analyzer::measure_block(&name, &node.block);
        self.functions.push((name, measured.cyclomatic));
        syn::visit::visit_impl_item_fn(self, node);
    }
}

/// Measure every parsable Rust file under `project_dir`.
///
/// Discovery goes through `ProjectFileDiscovery` so .gitignore/.pmatignore are
/// honoured and `target/` is not walked, matching `analyze complexity`.
fn scan_rust_complexity(project_dir: &Path) -> ComplexityScan {
    use crate::services::file_discovery::{FileDiscoveryConfig, ProjectFileDiscovery};
    use syn::visit::Visit;

    let discovery_config = FileDiscoveryConfig {
        respect_gitignore: true,
        filter_external_repos: true,
        ..Default::default()
    };
    let discovery =
        ProjectFileDiscovery::new(project_dir.to_path_buf()).with_config(discovery_config);
    let files = discovery.discover_files().unwrap_or_default();

    let mut scan = ComplexityScan::default();
    for file in files {
        if file.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        scan.files_seen += 1;

        let Ok(content) = std::fs::read_to_string(&file) else {
            continue;
        };
        let Ok(tree) = syn::parse_file(&content) else {
            continue; // include!() fragments and non-standalone sources
        };

        let mut visitor = GateComplexityVisitor {
            functions: Vec::new(),
        };
        visitor.visit_file(&tree);
        scan.files_measured += 1;
        scan.functions_measured += visitor.functions.len();

        for (name, cyclomatic) in visitor.functions {
            let worse = scan.worst.as_ref().is_none_or(|(_, _, w)| cyclomatic > *w);
            if worse {
                scan.worst = Some((file.display().to_string(), name, cyclomatic));
            }
        }
    }
    scan
}

/// Pure-compute decision builder for the complexity gate.
///
/// A scan that measured nothing must NOT report a pass — that was the whole
/// defect below.
fn build_complexity_decision(scan: &ComplexityScan, max_complexity: u32) -> (bool, String) {
    let Some((file, function, worst)) = scan.worst.as_ref() else {
        return (
            false,
            format!(
                "✗ Complexity: not measured — 0 of {} Rust file(s) could be parsed",
                scan.files_seen
            ),
        );
    };

    if *worst > max_complexity {
        return (
            false,
            format!(
                "✗ Complexity: {} in {} has cyclomatic {} (> {}); measured {} function(s) in {} of {} file(s)",
                function, file, worst, max_complexity,
                scan.functions_measured, scan.files_measured, scan.files_seen
            ),
        );
    }

    (
        true,
        format!(
            "✓ Complexity: max cyclomatic {} (≤ {}); measured {} function(s) in {} of {} file(s)",
            worst, max_complexity, scan.functions_measured, scan.files_measured, scan.files_seen
        ),
    )
}

/// Execute complexity gate
///
/// This gate used to ignore `project_dir` entirely: it set `let passed = true;`
/// under the comment "Simplified: Assume complexity passes" and then printed
/// the *configured* threshold back as if it had been measured. With
/// `max_complexity = 0` pointed at a 4000-file tree it still reported
/// `passed: true, "✓ Complexity: All functions <0"`. It now parses the tree and
/// compares the real worst cyclomatic count, and refuses to pass when it could
/// not measure anything.
///
/// # Complexity
/// - Time: O(codebase size)
/// - Cyclomatic: 2
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn execute_complexity(config: &GateConfig, project_dir: &Path) -> Result<GateResult> {
    use std::time::Instant;

    let start = Instant::now();
    let scan = scan_rust_complexity(project_dir);
    let duration = start.elapsed();

    let (passed, message) = build_complexity_decision(&scan, config.max_complexity);

    Ok(GateResult {
        name: "complexity".to_string(),
        passed,
        duration,
        message,
    })
}

#[cfg(test)]
mod complexity_gate_tests {
    use super::*;

    fn scan_with_worst(cyclomatic: u32) -> ComplexityScan {
        ComplexityScan {
            files_seen: 1,
            files_measured: 1,
            functions_measured: 1,
            worst: Some(("src/lib.rs".to_string(), "complex".to_string(), cyclomatic)),
        }
    }

    #[test]
    fn test_complexity_gate_fails_when_worst_exceeds_threshold() {
        // Regression: the gate used to hardcode `passed = true`, so a
        // max_complexity of 0 passed over any tree.
        let (passed, message) = build_complexity_decision(&scan_with_worst(6), 0);
        assert!(!passed, "cyclomatic 6 must not pass a threshold of 0");
        assert!(message.contains("complex"), "message names the offender: {message}");
    }

    #[test]
    fn test_complexity_gate_passes_when_under_threshold() {
        let (passed, message) = build_complexity_decision(&scan_with_worst(6), 10);
        assert!(passed);
        assert!(message.contains("max cyclomatic 6"), "{message}");
    }

    #[test]
    fn test_complexity_gate_refuses_to_pass_when_nothing_measured() {
        let scan = ComplexityScan {
            files_seen: 3,
            ..Default::default()
        };
        let (passed, message) = build_complexity_decision(&scan, 10);
        assert!(!passed, "a gate that measured nothing has not passed");
        assert!(message.contains("not measured"), "{message}");
    }

    #[test]
    fn test_execute_complexity_measures_the_real_tree() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("lib.rs"),
            "pub fn complex(n: u32) -> u32 {\n    let mut acc = 0;\n    for i in 0..n {\n        if i % 2 == 0 { acc += i; }\n        else if i % 3 == 0 { acc += i * 2; }\n        else if i % 5 == 0 { acc += i * 3; }\n        else if i % 7 == 0 { acc += i * 4; }\n        else { acc += 1; }\n    }\n    acc\n}\n",
        )
        .unwrap();

        let config = GateConfig {
            max_complexity: 0,
            ..Default::default()
        };
        let result = execute_complexity(&config, dir.path()).unwrap();
        assert!(
            !result.passed,
            "every function exceeds max_complexity 0: {}",
            result.message
        );
        assert!(result.message.contains("complex"), "{}", result.message);

        let lenient = GateConfig {
            max_complexity: 100,
            ..Default::default()
        };
        let result = execute_complexity(&lenient, dir.path()).unwrap();
        assert!(result.passed, "{}", result.message);
    }

    #[test]
    fn test_execute_complexity_does_not_pass_an_empty_tree() {
        let dir = tempfile::TempDir::new().unwrap();
        let result = execute_complexity(&GateConfig::default(), dir.path()).unwrap();
        assert!(!result.passed);
        assert!(result.message.contains("not measured"), "{}", result.message);
    }
}
