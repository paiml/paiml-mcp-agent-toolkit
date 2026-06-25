#![cfg_attr(coverage_nightly, coverage(off))]

#[cfg(test)]
mod tests {
    use crate::cli::handlers::work_contract::{
        EvidenceType, FalsificationMethod, FalsificationResult,
    };
    use crate::cli::handlers::work_falsification::cache::{
        find_cache_file, read_cached_metric, read_deny_cache_fallback, read_lint_cache_fallback,
    };
    use crate::cli::handlers::work_falsification::churn_checks::{
        detect_fix_chains, extract_large_match_variants, extract_test_section,
    };
    use crate::cli::handlers::work_falsification::pmat_owned_state::is_pmat_owned_state;
    use crate::cli::handlers::work_falsification::supply_chain_checks::{
        is_rust_project, test_supply_chain_integrity,
    };
    use crate::cli::handlers::work_falsification::types::{
        CachedMetric, ClaimResult, FalsificationReport, CACHE_BLOCK_HOURS, CACHE_WARN_HOURS,
    };
    use std::path::PathBuf;

    #[test]
    fn test_falsification_report_blocking() {
        let report = FalsificationReport {
            total_claims: 3,
            passed: 2,
            failed: 1,
            warnings: 0,
            claim_results: vec![
                ClaimResult {
                    index: 1,
                    hypothesis: "Test 1".to_string(),
                    method: FalsificationMethod::ManifestIntegrity,
                    result: FalsificationResult::passed("OK"),
                    is_blocking: true,
                },
                ClaimResult {
                    index: 2,
                    hypothesis: "Test 2".to_string(),
                    method: FalsificationMethod::AbsoluteCoverage,
                    result: FalsificationResult::failed(
                        "Coverage low",
                        EvidenceType::NumericComparison {
                            actual: 80.0,
                            threshold: 95.0,
                        },
                    ),
                    is_blocking: true,
                },
            ],
            all_passed: false,
        };

        assert!(report.has_blocking_failures());
        assert_eq!(report.blocking_failures().len(), 1);
    }

    #[test]
    fn test_github_sync_parsing() {
        // This tests the parsing logic for git status output
        let status = "## main...origin/main [ahead 2]\n M file.rs\n?? new.rs";

        let ahead = if status.contains("ahead") {
            status
                .lines()
                .next()
                .and_then(|l| {
                    l.find("ahead")
                        .and_then(|i| l.get(i..))
                        .and_then(|s| s.split_whitespace().nth(1))
                        .and_then(|n| n.trim_end_matches(']').parse::<usize>().ok())
                })
                .unwrap_or(0)
        } else {
            0
        };

        assert_eq!(ahead, 2);
    }

    #[test]
    fn test_github_sync_excludes_untracked_files() {
        // Untracked files (??) should NOT count as dirty (#224)
        let status = "## master...origin/master\n?? target\n?? docs/spec.md\n?? scripts/";
        let dirty_count = status
            .lines()
            .skip(1)
            .filter(|l| !l.is_empty() && !l.starts_with("??"))
            .count();
        assert_eq!(dirty_count, 0, "Untracked files should not count as dirty");
    }

    #[test]
    fn test_github_sync_counts_modified_files() {
        // Modified/staged files should count as dirty
        let status = "## master...origin/master\n M src/lib.rs\nA  src/new.rs\n?? untracked.txt";
        let dirty_count = status
            .lines()
            .skip(1)
            .filter(|l| !l.is_empty() && !l.starts_with("??"))
            .count();
        assert_eq!(dirty_count, 2, "Modified and staged files should count");
    }

    // PMAT-154: pmat's own post-commit hook regenerates .pmat/baseline.json,
    // .pmat/context.db and .pmat/*.cache during `work complete`. These are
    // self-generated artifacts and MUST NOT be counted as unpushed dirty work,
    // or the github-sync falsifier hits the self-dirty loop on retries.
    #[test]
    fn test_github_sync_ignores_self_generated_pmat_artifacts() {
        // Exactly the files the post-commit hook regenerates.
        assert!(
            is_pmat_owned_state(" M .pmat/baseline.json"),
            ".pmat/baseline.json is a pmat-owned artifact"
        );
        assert!(
            is_pmat_owned_state(" M .pmat/context.db"),
            ".pmat/context.db is a pmat-owned artifact"
        );
        assert!(
            is_pmat_owned_state(" M .pmat/tdg.cache"),
            ".pmat/*.cache is a pmat-owned artifact"
        );
        assert!(
            is_pmat_owned_state(" M .pmat/context.idx/manifest.cache"),
            "nested .pmat/*.cache is a pmat-owned artifact"
        );

        // Look-alike user-owned files must still count as real dirty work.
        assert!(
            !is_pmat_owned_state(" M .pmat-baseline.json"),
            ".pmat-baseline.json (user-owned look-alike) must NOT be ignored"
        );
        assert!(
            !is_pmat_owned_state(" M src/baseline.json"),
            "user source files must NOT be ignored"
        );

        // Full self-dirty scenario: HEAD == origin, only pmat artifacts dirty.
        let status = concat!(
            "## master...origin/master\n",
            " M .pmat/baseline.json\n",
            " M .pmat/context.db\n",
            " M .pmat/tdg.cache\n"
        );
        let dirty_count = status
            .lines()
            .skip(1)
            .filter(|l| !l.is_empty() && !l.starts_with("??"))
            .filter(|l| !is_pmat_owned_state(l))
            .count();
        assert_eq!(
            dirty_count, 0,
            "pmat-self-generated artifacts must not trip the github-sync dirty check"
        );
    }

    // PMAT-058: Rust-only supply-chain gates must be skipped (not failed) on a
    // repo with no Cargo.toml (e.g. the infra fleet repo).
    #[test]
    fn test_is_rust_project_detection() {
        use tempfile::TempDir;
        let non_rust = TempDir::new().unwrap();
        assert!(
            !is_rust_project(non_rust.path()),
            "dir without Cargo.toml is not a Rust project"
        );

        let rust = TempDir::new().unwrap();
        std::fs::write(rust.path().join("Cargo.toml"), "[package]\nname = \"x\"\n").unwrap();
        assert!(
            is_rust_project(rust.path()),
            "dir with Cargo.toml is a Rust project"
        );
    }

    #[tokio::test]
    async fn test_supply_chain_skips_non_rust_repo() {
        use tempfile::TempDir;
        // No Cargo.toml and no deny cache. Without the PMAT-058 guard this would
        // hard-fail with "No deny cache. Run 'cargo deny check' first".
        let temp = TempDir::new().unwrap();
        let result = test_supply_chain_integrity(temp.path()).await.unwrap();
        assert!(
            !result.falsified,
            "supply-chain gate must not fail on a non-Rust repo"
        );
        assert!(
            result.explanation.contains("N/A (non-Rust repo"),
            "non-Rust skip must be clearly labeled, got: {}",
            result.explanation
        );
    }

    #[tokio::test]
    async fn test_supply_chain_still_fails_rust_repo_without_cache() {
        use tempfile::TempDir;
        // A Rust repo (Cargo.toml present) with no deny cache must STILL fail —
        // PMAT-058 must not weaken the gate for real Rust projects.
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("Cargo.toml"), "[package]\nname = \"x\"\n").unwrap();
        let result = test_supply_chain_integrity(temp.path()).await.unwrap();
        assert!(
            result.falsified,
            "Rust repo with no deny cache must still fail the supply-chain gate"
        );
    }

    #[test]
    fn test_find_cache_file_empty_dir() {
        let tmp = std::env::temp_dir().join("pmat-test-cache-find");
        let _ = std::fs::create_dir_all(&tmp);
        let result = find_cache_file(&tmp, "nonexistent.txt");
        assert!(result.is_empty());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_find_cache_file_in_pmat_dir() {
        let tmp = std::env::temp_dir().join("pmat-test-cache-pmat");
        let pmat_dir = tmp.join(".pmat");
        let _ = std::fs::create_dir_all(&pmat_dir);
        std::fs::write(pmat_dir.join("deny-cache.txt"), "ok").unwrap();

        let result = find_cache_file(&tmp, "deny-cache.txt");
        assert_eq!(result.len(), 1);
        assert!(result[0].ends_with(".pmat/deny-cache.txt"));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_find_cache_file_in_work_dir() {
        let tmp = std::env::temp_dir().join("pmat-test-cache-work");
        let work_dir = tmp.join(".pmat-work").join("PMAT-260");
        let _ = std::fs::create_dir_all(&work_dir);
        std::fs::write(work_dir.join("lint-cache.txt"), "passed").unwrap();

        let result = find_cache_file(&tmp, "lint-cache.txt");
        assert_eq!(result.len(), 1);
        assert!(result[0].to_string_lossy().contains("PMAT-260"));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_read_deny_cache_fallback_with_pass() {
        let tmp = std::env::temp_dir().join("pmat-test-deny-fallback");
        let pmat_dir = tmp.join(".pmat");
        let _ = std::fs::create_dir_all(&pmat_dir);
        std::fs::write(
            pmat_dir.join("deny-cache.txt"),
            "all checks passed\n0 warnings",
        )
        .unwrap();

        let result = read_deny_cache_fallback(&tmp);
        assert!(result.is_some());
        let cache = result.unwrap();
        assert_eq!(
            cache.value.get("passed").and_then(|v| v.as_bool()),
            Some(true)
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_read_lint_cache_fallback_with_errors() {
        let tmp = std::env::temp_dir().join("pmat-test-lint-fallback");
        let pmat_dir = tmp.join(".pmat");
        let _ = std::fs::create_dir_all(&pmat_dir);
        std::fs::write(
            pmat_dir.join("lint-cache.txt"),
            "error[E0001]: unused var\nerror[E0002]: missing type",
        )
        .unwrap();

        let result = read_lint_cache_fallback(&tmp);
        assert!(result.is_some());
        let cache = result.unwrap();
        assert_eq!(
            cache.value.get("passed").and_then(|v| v.as_bool()),
            Some(false)
        );
        // "error" appears twice (once per line)
        assert!(
            cache
                .value
                .get("error_count")
                .and_then(|v| v.as_u64())
                .unwrap()
                >= 2
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_falsification_report_warnings() {
        let report = FalsificationReport {
            total_claims: 2,
            passed: 1,
            failed: 1,
            warnings: 1,
            claim_results: vec![
                ClaimResult {
                    index: 1,
                    hypothesis: "Test OK".to_string(),
                    method: FalsificationMethod::ManifestIntegrity,
                    result: FalsificationResult::passed("OK"),
                    is_blocking: false,
                },
                ClaimResult {
                    index: 2,
                    hypothesis: "Warning test".to_string(),
                    method: FalsificationMethod::LintPass,
                    result: FalsificationResult::failed(
                        "Size above warning threshold",
                        EvidenceType::NumericComparison {
                            actual: 45.0,
                            threshold: 40.0,
                        },
                    ),
                    is_blocking: false,
                },
            ],
            all_passed: false,
        };

        assert!(!report.has_blocking_failures());
        assert_eq!(report.blocking_failures().len(), 0);
        assert_eq!(report.warning_failures().len(), 1);
    }

    #[test]
    fn test_falsification_report_all_passed() {
        let report = FalsificationReport {
            total_claims: 2,
            passed: 2,
            failed: 0,
            warnings: 0,
            claim_results: vec![
                ClaimResult {
                    index: 1,
                    hypothesis: "Test 1".to_string(),
                    method: FalsificationMethod::ManifestIntegrity,
                    result: FalsificationResult::passed("OK"),
                    is_blocking: true,
                },
                ClaimResult {
                    index: 2,
                    hypothesis: "Test 2".to_string(),
                    method: FalsificationMethod::AbsoluteCoverage,
                    result: FalsificationResult::passed("Coverage OK"),
                    is_blocking: true,
                },
            ],
            all_passed: true,
        };

        assert!(!report.has_blocking_failures());
        assert_eq!(report.blocking_failures().len(), 0);
        assert_eq!(report.warning_failures().len(), 0);
    }

    #[test]
    fn test_falsification_result_constructors() {
        let passed = FalsificationResult::passed("Success");
        assert!(!passed.falsified);
        assert_eq!(passed.explanation, "Success");

        let failed = FalsificationResult::failed("Error", EvidenceType::BooleanCheck(false));
        assert!(failed.falsified);
        assert_eq!(failed.explanation, "Error");
    }

    #[test]
    fn test_evidence_type_display() {
        let bool_check = EvidenceType::BooleanCheck(true);
        let numeric = EvidenceType::NumericComparison {
            actual: 80.0,
            threshold: 95.0,
        };

        // Just verify these don't panic when formatted
        let _ = format!("{:?}", bool_check);
        let _ = format!("{:?}", numeric);
    }

    #[test]
    fn test_cached_metric_staleness_levels() {
        // Test cache staleness calculation
        let fresh = CachedMetric {
            value: serde_json::json!({"coverage": 85.0}),
            age_minutes: 30,
            is_stale_warn: false,
            is_stale_block: false,
        };
        assert!(!fresh.is_stale_warn);
        assert!(!fresh.is_stale_block);

        let warning = CachedMetric {
            value: serde_json::json!({"coverage": 85.0}),
            age_minutes: 90,
            is_stale_warn: true,
            is_stale_block: false,
        };
        assert!(warning.is_stale_warn);
        assert!(!warning.is_stale_block);

        let blocked = CachedMetric {
            value: serde_json::json!({"coverage": 85.0}),
            age_minutes: 1500, // 25 hours
            is_stale_warn: true,
            is_stale_block: true,
        };
        assert!(blocked.is_stale_warn);
        assert!(blocked.is_stale_block);
    }

    #[test]
    fn test_read_cached_metric_nonexistent() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let result = read_cached_metric(temp_dir.path(), "nonexistent.json");
        assert!(result.is_none());
    }

    #[test]
    fn test_read_cached_metric_invalid_json() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join(".pmat-metrics");
        std::fs::create_dir_all(&cache_dir).unwrap();
        std::fs::write(cache_dir.join("test.json"), "not valid json").unwrap();
        let result = read_cached_metric(temp_dir.path(), "test.json");
        assert!(result.is_none());
    }

    #[test]
    fn test_read_cached_metric_valid() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join(".pmat-metrics");
        std::fs::create_dir_all(&cache_dir).unwrap();
        std::fs::write(
            cache_dir.join("coverage.json"),
            r#"{"line_coverage": 85.5}"#,
        )
        .unwrap();
        let result = read_cached_metric(temp_dir.path(), "coverage.json");
        assert!(result.is_some());
        let metric = result.unwrap();
        assert_eq!(metric.value["line_coverage"], 85.5);
        assert!(!metric.is_stale_warn); // Just created
        assert!(!metric.is_stale_block);
    }

    #[test]
    fn test_claim_result_debug() {
        let claim = ClaimResult {
            index: 1,
            hypothesis: "Test hypothesis".to_string(),
            method: FalsificationMethod::ManifestIntegrity,
            result: FalsificationResult::passed("OK"),
            is_blocking: true,
        };
        let debug_str = format!("{:?}", claim);
        assert!(debug_str.contains("hypothesis"));
        assert!(debug_str.contains("Test hypothesis"));
    }

    #[test]
    fn test_falsification_report_empty() {
        let report = FalsificationReport {
            total_claims: 0,
            passed: 0,
            failed: 0,
            warnings: 0,
            claim_results: vec![],
            all_passed: true,
        };
        assert!(!report.has_blocking_failures());
        assert!(report.blocking_failures().is_empty());
        assert!(report.warning_failures().is_empty());
    }

    #[test]
    fn test_falsification_report_mixed_results() {
        let report = FalsificationReport {
            total_claims: 4,
            passed: 2,
            failed: 1,
            warnings: 1,
            claim_results: vec![
                ClaimResult {
                    index: 1,
                    hypothesis: "Passed".to_string(),
                    method: FalsificationMethod::ManifestIntegrity,
                    result: FalsificationResult::passed("OK"),
                    is_blocking: true,
                },
                ClaimResult {
                    index: 2,
                    hypothesis: "Blocking failure".to_string(),
                    method: FalsificationMethod::AbsoluteCoverage,
                    result: FalsificationResult::failed(
                        "Coverage low",
                        EvidenceType::NumericComparison {
                            actual: 70.0,
                            threshold: 95.0,
                        },
                    ),
                    is_blocking: true,
                },
                ClaimResult {
                    index: 3,
                    hypothesis: "Warning".to_string(),
                    method: FalsificationMethod::FileSizeRegression,
                    result: FalsificationResult::failed(
                        "File too large",
                        EvidenceType::NumericComparison {
                            actual: 600.0,
                            threshold: 500.0,
                        },
                    ),
                    is_blocking: false,
                },
                ClaimResult {
                    index: 4,
                    hypothesis: "Another pass".to_string(),
                    method: FalsificationMethod::LintPass,
                    result: FalsificationResult::passed("Lint OK"),
                    is_blocking: true,
                },
            ],
            all_passed: false,
        };
        assert!(report.has_blocking_failures());
        assert_eq!(report.blocking_failures().len(), 1);
        assert_eq!(report.warning_failures().len(), 1);
    }

    #[test]
    fn test_evidence_type_variants() {
        let file_list =
            EvidenceType::FileList(vec![PathBuf::from("file1.rs"), PathBuf::from("file2.rs")]);
        let _ = format!("{:?}", file_list);

        let counter_example = EvidenceType::CounterExample {
            details: "Some evidence text".to_string(),
        };
        let _ = format!("{:?}", counter_example);
    }

    #[test]
    fn test_falsification_method_variants() {
        // Verify all falsification methods can be formatted
        let methods = vec![
            FalsificationMethod::ManifestIntegrity,
            FalsificationMethod::MetaFalsification,
            FalsificationMethod::CoverageGaming,
            FalsificationMethod::DifferentialCoverage,
            FalsificationMethod::AbsoluteCoverage,
            FalsificationMethod::TdgRegression,
            FalsificationMethod::ComplexityRegression,
            FalsificationMethod::SupplyChainIntegrity,
            FalsificationMethod::FileSizeRegression,
            FalsificationMethod::SpecQuality,
            FalsificationMethod::RoadmapUpdate,
            FalsificationMethod::GitHubSync,
            FalsificationMethod::ExamplesCompile,
            FalsificationMethod::BookValidation,
            FalsificationMethod::SatdDetection,
            FalsificationMethod::DeadCodeDetection,
            FalsificationMethod::PerFileCoverage,
            FalsificationMethod::LintPass,
            FalsificationMethod::VariantCoverage,
            FalsificationMethod::FixChainLimit,
            FalsificationMethod::CrossCrateParity,
            FalsificationMethod::RegressionGate,
            FalsificationMethod::FormalProofVerification,
        ];
        for method in methods {
            let _ = format!("{:?}", method);
        }
    }

    #[test]
    fn test_extract_large_match_variants() {
        let code = r#"
            match dtype {
                Dtype::F32 => do_f32(),
                Dtype::F16 => do_f16(),
                Dtype::Q4_K => do_q4k(),
                Dtype::Q6_K => do_q6k(),
                Dtype::Q8_0 => do_q80(),
                _ => panic!("unsupported"),
            }
        "#;
        let variants = extract_large_match_variants(code);
        assert!(variants.contains(&"F32".to_string()));
        assert!(variants.contains(&"Q4_K".to_string()));
        assert!(variants.contains(&"Q8_0".to_string()));
        assert_eq!(variants.len(), 5);
    }

    #[test]
    fn test_extract_large_match_variants_small_match() {
        // Match with <5 arms should return empty
        let code = r#"
            match color {
                Color::Red => 1,
                Color::Blue => 2,
            }
        "#;
        let variants = extract_large_match_variants(code);
        assert!(variants.is_empty());
    }

    #[test]
    fn test_detect_fix_chains_basic() {
        let log = "\
abc1234 fix: broken Q4K dispatch
src/quantize.rs
abc1235 fix: also broken Q6K
src/quantize.rs
abc1236 fix: and Q8_0 too
src/quantize.rs
abc1237 fix: final Q2_K fix
src/quantize.rs
abc1238 feat: add new feature
src/other.rs
";
        let chains = detect_fix_chains(log, 3);
        assert_eq!(chains.len(), 1);
        assert_eq!(chains[0].0, "src/quantize.rs");
        assert_eq!(chains[0].1, 4);
    }

    #[test]
    fn test_detect_fix_chains_no_violations() {
        let log = "\
abc1234 fix: one fix
src/a.rs
abc1235 feat: feature
src/b.rs
abc1236 fix: another fix
src/c.rs
";
        let chains = detect_fix_chains(log, 3);
        assert!(chains.is_empty());
    }

    #[test]
    fn test_extract_test_section() {
        let code = r#"
fn production_code() {}

#[cfg(test)]
mod tests {
    #[test]
    fn test_something() {
        assert!(true);
    }
}
"#;
        let section = extract_test_section(code);
        assert!(section.contains("test_something"));
        assert!(!section.contains("production_code"));
    }

    #[test]
    fn test_cache_staleness_thresholds() {
        // Verify constants are reasonable
        assert_eq!(CACHE_WARN_HOURS, 1);
        assert_eq!(CACHE_BLOCK_HOURS, 24);
        assert!(CACHE_WARN_HOURS < CACHE_BLOCK_HOURS);
    }
}
