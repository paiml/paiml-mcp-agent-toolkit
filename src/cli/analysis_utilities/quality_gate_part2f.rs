#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod quality_gate_threshold_tests {
    use super::*;

    // ===================
    // load_provability_threshold Tests (GH-172)
    // ===================

    #[test]
    fn test_load_provability_threshold_no_file() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let threshold = load_provability_threshold(temp_dir.path());
        assert!(
            (threshold - DEFAULT_PROVABILITY_THRESHOLD).abs() < f64::EPSILON,
            "Should fall back to default when file is missing"
        );
    }

    #[test]
    fn test_load_provability_threshold_from_config() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config_content = r#"
[thresholds]
provability_min = 0.60
"#;
        std::fs::write(temp_dir.path().join(".pmat-metrics.toml"), config_content).unwrap();

        let threshold = load_provability_threshold(temp_dir.path());
        assert!(
            (threshold - 0.60).abs() < f64::EPSILON,
            "Should read provability_min from config, got {threshold}"
        );
    }

    #[test]
    fn test_load_provability_threshold_missing_key() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config_content = r#"
[thresholds]
lint_max_ms = 150000
"#;
        std::fs::write(temp_dir.path().join(".pmat-metrics.toml"), config_content).unwrap();

        let threshold = load_provability_threshold(temp_dir.path());
        assert!(
            (threshold - DEFAULT_PROVABILITY_THRESHOLD).abs() < f64::EPSILON,
            "Should fall back to default when key is missing"
        );
    }

    #[test]
    fn test_load_provability_threshold_invalid_toml() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        std::fs::write(
            temp_dir.path().join(".pmat-metrics.toml"),
            "this is not valid toml {{{{",
        )
        .unwrap();

        let threshold = load_provability_threshold(temp_dir.path());
        assert!(
            (threshold - DEFAULT_PROVABILITY_THRESHOLD).abs() < f64::EPSILON,
            "Should fall back to default when TOML is invalid"
        );
    }

    #[test]
    fn test_load_provability_threshold_no_thresholds_section() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config_content = r#"
[quality_gates]
min_coverage_pct = 85.0
"#;
        std::fs::write(temp_dir.path().join(".pmat-metrics.toml"), config_content).unwrap();

        let threshold = load_provability_threshold(temp_dir.path());
        assert!(
            (threshold - DEFAULT_PROVABILITY_THRESHOLD).abs() < f64::EPSILON,
            "Should fall back to default when [thresholds] section is missing"
        );
    }

    // --- Entropy threshold tests (#194) ---

    #[test]
    fn test_load_entropy_threshold_no_file() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let threshold = load_entropy_threshold(temp_dir.path(), 0.3);
        // #248: Empty temp dir has 0 source files (<10), so threshold is scaled by 0.5
        assert!(
            (threshold - 0.15).abs() < f64::EPSILON,
            "Should fall back to CLI value (0.3) scaled for small repo (0.5) = 0.15, got {threshold}"
        );
    }

    #[test]
    fn test_load_entropy_threshold_from_config() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config_content = r#"
[thresholds]
entropy_min_diversity = 0.0
"#;
        std::fs::write(temp_dir.path().join(".pmat-metrics.toml"), config_content).unwrap();

        let threshold = load_entropy_threshold(temp_dir.path(), 0.3);
        assert!(
            threshold.abs() < f64::EPSILON,
            "Should read entropy_min_diversity=0.0 from config, got {threshold}"
        );
    }

    #[test]
    fn test_load_entropy_threshold_missing_key_falls_back_to_cli() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config_content = r#"
[thresholds]
provability_min = 0.70
"#;
        std::fs::write(temp_dir.path().join(".pmat-metrics.toml"), config_content).unwrap();

        let threshold = load_entropy_threshold(temp_dir.path(), 0.5);
        // #248: Empty temp dir has 0 source files (<10), so threshold is scaled by 0.5
        assert!(
            (threshold - 0.25).abs() < f64::EPSILON,
            "Should fall back to CLI value (0.5) scaled for small repo (0.5) = 0.25, got {threshold}"
        );
    }

    // --- Exclude paths tests (#195) ---

    #[test]
    fn test_load_entropy_exclude_paths_no_file() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let paths = load_entropy_exclude_paths(temp_dir.path());
        assert!(paths.is_empty(), "Should return empty when file is missing");
    }

    #[test]
    fn test_load_entropy_exclude_paths_from_exclude_section() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config_content = r#"
[exclude]
paths = ["reference/", "vendor/"]
"#;
        std::fs::write(temp_dir.path().join(".pmat-metrics.toml"), config_content).unwrap();

        let paths = load_entropy_exclude_paths(temp_dir.path());
        assert_eq!(paths, vec!["reference/", "vendor/"]);
    }

    #[test]
    fn test_load_entropy_exclude_paths_from_top_level() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config_content = r#"
exclude_paths = ["third_party/"]
"#;
        std::fs::write(temp_dir.path().join(".pmat-metrics.toml"), config_content).unwrap();

        let paths = load_entropy_exclude_paths(temp_dir.path());
        assert_eq!(paths, vec!["third_party/"]);
    }

    #[test]
    fn test_load_entropy_exclude_paths_section_takes_precedence() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config_content = r#"
exclude_paths = ["old/"]

[exclude]
paths = ["new/"]
"#;
        std::fs::write(temp_dir.path().join(".pmat-metrics.toml"), config_content).unwrap();

        let paths = load_entropy_exclude_paths(temp_dir.path());
        assert_eq!(paths, vec!["new/"], "[exclude] paths should take precedence over top-level exclude_paths");
    }

    #[test]
    fn test_load_entropy_exclude_paths_from_gates_toml() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let gates_content = r#"
[quality-gates]
exclude = [
    "**/*_generated.rs",
    "demos/**",
    "examples/**",
]
"#;
        std::fs::write(temp_dir.path().join(".pmat-gates.toml"), gates_content).unwrap();

        let paths = load_entropy_exclude_paths(temp_dir.path());
        assert_eq!(paths.len(), 3, "Should load excludes from .pmat-gates.toml [quality-gates] exclude");
        assert!(paths.contains(&"**/*_generated.rs".to_string()));
        assert!(paths.contains(&"demos/**".to_string()));
        assert!(paths.contains(&"examples/**".to_string()));
    }

    #[test]
    fn test_load_entropy_exclude_paths_merges_both_files() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let metrics_content = r#"
[exclude]
paths = ["target/**"]
"#;
        let gates_content = r#"
[quality-gates]
exclude = ["demos/**", "examples/**"]
"#;
        std::fs::write(temp_dir.path().join(".pmat-metrics.toml"), metrics_content).unwrap();
        std::fs::write(temp_dir.path().join(".pmat-gates.toml"), gates_content).unwrap();

        let paths = load_entropy_exclude_paths(temp_dir.path());
        assert_eq!(paths.len(), 3, "Should merge excludes from both config files");
        assert!(paths.contains(&"target/**".to_string()));
        assert!(paths.contains(&"demos/**".to_string()));
        assert!(paths.contains(&"examples/**".to_string()));
    }

    // --- Entropy gate config tests (#220) ---

    #[test]
    fn test_load_entropy_gate_config_no_file() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config = load_entropy_gate_config(temp_dir.path());
        assert!(config.enabled);
        assert!(config.max_violations.is_none());
        assert!(config.exclude.is_empty());
    }

    #[test]
    fn test_load_entropy_gate_config_enabled_false() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let content = r#"
[entropy]
enabled = false
"#;
        std::fs::write(temp_dir.path().join(".pmat-gates.toml"), content).unwrap();
        let config = load_entropy_gate_config(temp_dir.path());
        assert!(!config.enabled);
    }

    #[test]
    fn test_load_entropy_gate_config_max_violations() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let content = r#"
[entropy]
max_violations = 5
"#;
        std::fs::write(temp_dir.path().join(".pmat-gates.toml"), content).unwrap();
        let config = load_entropy_gate_config(temp_dir.path());
        assert!(config.enabled);
        assert_eq!(config.max_violations, Some(5));
    }

    #[test]
    fn test_load_entropy_gate_config_excludes() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let content = r#"
[entropy]
exclude = ["**/gqa.rs", "benches/**"]
"#;
        std::fs::write(temp_dir.path().join(".pmat-gates.toml"), content).unwrap();
        let config = load_entropy_gate_config(temp_dir.path());
        assert_eq!(config.exclude.len(), 2);
        assert!(config.exclude.contains(&"**/gqa.rs".to_string()));
        assert!(config.exclude.contains(&"benches/**".to_string()));
    }

    #[test]
    fn test_load_entropy_gate_config_full() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let content = r#"
[entropy]
enabled = true
max_pattern_repetition = 12
min_pattern_diversity = 0.3
max_violations = 3
exclude = ["**/gqa.rs"]
"#;
        std::fs::write(temp_dir.path().join(".pmat-gates.toml"), content).unwrap();
        let config = load_entropy_gate_config(temp_dir.path());
        assert!(config.enabled);
        assert_eq!(config.max_violations, Some(3));
        assert_eq!(config.exclude, vec!["**/gqa.rs"]);
    }

    // --- Filter violations by exclude paths tests (#196) ---

    #[test]
    fn test_filter_violations_excludes_matching_files() {
        let mut violations = vec![
            QualityViolation { check_type: "satd".into(), severity: "warning".into(), file: "reference/kong/init.lua".into(), line: Some(10), message: "TODO".into(), details: None },
            QualityViolation { check_type: "satd".into(), severity: "warning".into(), file: "src/main.rs".into(), line: Some(5), message: "TODO".into(), details: None },
            QualityViolation { check_type: "duplicates".into(), severity: "info".into(), file: "reference/apisix/core.lua".into(), line: None, message: "dup".into(), details: None },
        ];
        let excludes = vec!["reference/".to_string()];
        filter_violations_by_exclude(&mut violations, &excludes);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].file, "src/main.rs");
    }

    #[test]
    fn test_filter_violations_keeps_project_level() {
        let mut violations = vec![
            QualityViolation { check_type: "entropy".into(), severity: "warning".into(), file: "project".into(), line: None, message: "low diversity".into(), details: None },
            QualityViolation { check_type: "satd".into(), severity: "info".into(), file: "vendor/lib.lua".into(), line: Some(1), message: "hack".into(), details: None },
        ];
        let excludes = vec!["vendor/".to_string()];
        filter_violations_by_exclude(&mut violations, &excludes);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].file, "project");
    }

    #[test]
    fn test_filter_violations_empty_excludes_no_change() {
        let mut violations = vec![
            QualityViolation { check_type: "satd".into(), severity: "info".into(), file: "src/lib.rs".into(), line: Some(1), message: "fixme".into(), details: None },
        ];
        filter_violations_by_exclude(&mut violations, &[]);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_recalculate_from_violations() {
        let violations = vec![
            QualityViolation { check_type: "satd".into(), severity: "warning".into(), file: "a.rs".into(), line: Some(1), message: "todo".into(), details: None },
            QualityViolation { check_type: "satd".into(), severity: "info".into(), file: "b.rs".into(), line: Some(2), message: "hack".into(), details: None },
            QualityViolation { check_type: "complexity".into(), severity: "error".into(), file: "c.rs".into(), line: Some(3), message: "high".into(), details: None },
        ];
        let mut results = QualityGateResults::default();
        results.recalculate_from(&violations);
        assert_eq!(results.satd_violations, 2);
        assert_eq!(results.complexity_violations, 1);
        assert_eq!(results.total_violations, 3);
        assert_eq!(results.entropy_violations, 0);
    }

    // ===================
    // Filesystem helpers: count_source_files / has_matching_extension /
    // is_traversable_dir / count_files_recursive / scale_entropy_for_project_size
    // (0%-covered leaves of load_entropy_threshold; quality_gate_config.rs:91..142)
    // ===================

    #[test]
    fn test_has_matching_extension_hit_and_miss() {
        let exts = &["rs", "py"];
        assert!(has_matching_extension(std::path::Path::new("foo.rs"), exts));
        assert!(has_matching_extension(std::path::Path::new("bar/baz.py"), exts));
        assert!(!has_matching_extension(
            std::path::Path::new("foo.md"),
            exts
        ));
        // No extension at all — extension() returns None, so is_some_and is false.
        assert!(!has_matching_extension(std::path::Path::new("README"), exts));
    }

    #[test]
    fn test_is_traversable_dir_rejects_hidden_target_node_modules() {
        assert!(is_traversable_dir(std::path::Path::new("src")));
        assert!(is_traversable_dir(std::path::Path::new("lib/nested")));
        // Hidden dot-dir
        assert!(!is_traversable_dir(std::path::Path::new(".git")));
        assert!(!is_traversable_dir(std::path::Path::new("foo/.hidden")));
        // Explicit disallow list
        assert!(!is_traversable_dir(std::path::Path::new("target")));
        assert!(!is_traversable_dir(std::path::Path::new("node_modules")));
        // Non-UTF8 / no file name (root) — and_then(to_str) → None → is_some_and = false.
        assert!(!is_traversable_dir(std::path::Path::new("/")));
    }

    #[test]
    fn test_count_files_recursive_counts_only_matching_extensions() {
        let tmp = tempfile::TempDir::new().unwrap();
        // Matching: 3 .rs
        std::fs::write(tmp.path().join("a.rs"), "").unwrap();
        std::fs::write(tmp.path().join("b.rs"), "").unwrap();
        let nested = tmp.path().join("nested");
        std::fs::create_dir(&nested).unwrap();
        std::fs::write(nested.join("c.rs"), "").unwrap();
        // Non-matching
        std::fs::write(tmp.path().join("d.md"), "").unwrap();
        std::fs::write(nested.join("e.txt"), "").unwrap();

        let count = count_files_recursive(tmp.path(), &["rs"], 0);
        assert_eq!(count, 3, "should count only .rs files, not .md/.txt");
    }

    #[test]
    fn test_count_files_recursive_skips_target_and_hidden_dirs() {
        let tmp = tempfile::TempDir::new().unwrap();
        // Top-level match counts.
        std::fs::write(tmp.path().join("top.rs"), "").unwrap();
        // target/ is skipped by is_traversable_dir.
        let target = tmp.path().join("target");
        std::fs::create_dir(&target).unwrap();
        std::fs::write(target.join("inside.rs"), "").unwrap();
        // .git/ (hidden) is skipped.
        let git = tmp.path().join(".git");
        std::fs::create_dir(&git).unwrap();
        std::fs::write(git.join("hook.rs"), "").unwrap();

        let count = count_files_recursive(tmp.path(), &["rs"], 0);
        assert_eq!(count, 1, "only the top-level .rs should be counted");
    }

    #[test]
    fn test_count_files_recursive_depth_limit_returns_zero() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(tmp.path().join("x.rs"), "").unwrap();
        // Pass depth=11 (> max 10) — hits the early-return arm.
        assert_eq!(count_files_recursive(tmp.path(), &["rs"], 11), 0);
    }

    #[test]
    fn test_count_files_recursive_read_dir_error_returns_zero() {
        // Point at a non-existent path so std::fs::read_dir returns Err.
        let missing = std::path::Path::new("/nonexistent/pmat/dir/0xC0FFEE");
        assert_eq!(count_files_recursive(missing, &["rs"], 0), 0);
    }

    #[test]
    fn test_count_source_files_uses_src_dir_when_present() {
        let tmp = tempfile::TempDir::new().unwrap();
        let src = tmp.path().join("src");
        std::fs::create_dir(&src).unwrap();
        std::fs::write(src.join("lib.rs"), "").unwrap();
        std::fs::write(src.join("main.rs"), "").unwrap();
        // Outside src/ — should NOT be counted once src/ exists.
        std::fs::write(tmp.path().join("root.rs"), "").unwrap();

        let count = count_source_files(tmp.path());
        assert_eq!(count, 2, "only src/ contents should count");
    }

    #[test]
    fn test_count_source_files_falls_back_to_root_when_no_source_dir() {
        let tmp = tempfile::TempDir::new().unwrap();
        // No src/lib/app/pkg/crates — fall back to root shallow count.
        std::fs::write(tmp.path().join("script.py"), "").unwrap();
        std::fs::write(tmp.path().join("README.md"), "").unwrap();

        let count = count_source_files(tmp.path());
        assert_eq!(count, 1, ".py counts, .md does not");
    }

    #[test]
    fn test_scale_entropy_for_project_size_all_bands() {
        let tmp = tempfile::TempDir::new().unwrap();
        let src = tmp.path().join("src");
        std::fs::create_dir(&src).unwrap();

        // < 10 files → scale 0.5
        for i in 0..5 {
            std::fs::write(src.join(format!("a{i}.rs")), "").unwrap();
        }
        let scaled = scale_entropy_for_project_size(tmp.path(), 1.0);
        assert!(
            (scaled - 0.5).abs() < f64::EPSILON,
            "< 10 files → 0.5 (got {scaled})"
        );

        // 10–24 files → scale 0.7
        for i in 5..15 {
            std::fs::write(src.join(format!("a{i}.rs")), "").unwrap();
        }
        let scaled = scale_entropy_for_project_size(tmp.path(), 1.0);
        assert!(
            (scaled - 0.7).abs() < f64::EPSILON,
            "10–24 files → 0.7 (got {scaled})"
        );

        // 25–49 files → scale 0.85
        for i in 15..35 {
            std::fs::write(src.join(format!("a{i}.rs")), "").unwrap();
        }
        let scaled = scale_entropy_for_project_size(tmp.path(), 1.0);
        assert!(
            (scaled - 0.85).abs() < f64::EPSILON,
            "25–49 files → 0.85 (got {scaled})"
        );

        // ≥ 50 files → scale 1.0 (no scaling)
        for i in 35..60 {
            std::fs::write(src.join(format!("a{i}.rs")), "").unwrap();
        }
        let scaled = scale_entropy_for_project_size(tmp.path(), 0.42);
        assert!(
            (scaled - 0.42).abs() < f64::EPSILON,
            "≥ 50 files → identity (got {scaled})"
        );
    }
}
