// Complexity checking functions - extracted from quality_checks_part1.rs (CB-040)

// Quality check functions

/// Checks code complexity in a project and returns violations.
///
/// # Arguments
///
/// * `project_path` - Path to the project directory to analyze
/// * `max_complexity` - Maximum allowed cyclomatic complexity
///
/// # Returns
///
/// A vector of quality violations for functions exceeding the complexity threshold
///
/// # Examples
///
/// ```no_run
/// # use std::path::Path;
/// # use pmat::cli::analysis_utilities::{check_complexity, QualityViolation};
/// # async fn example() -> anyhow::Result<()> {
/// let violations = check_complexity(Path::new("."), 10).await?;
/// for violation in violations {
///     println!("Complex function: {} in {}", violation.message, violation.file);
/// }
/// # Ok(())
/// # }
/// ```
///
/// # Property Tests
///
/// ```rust,no_run
/// # tokio_test::block_on(async {
/// use std::path::Path;
/// use pmat::cli::analysis_utilities::check_complexity;
///
/// // Test with a specific threshold
/// let threshold = 10u32;
/// let violations = check_complexity(Path::new("."), threshold).await.unwrap();
///
/// // Property: All violations should have complexity > threshold
/// for violation in violations {
///     // Extract complexity from message
///     if let Some(complexity_str) = violation.message
///         .split("complexity ")
///         .nth(1)
///         .and_then(|s| s.split(' ').next())
///         .and_then(|s| s.parse::<u32>().ok()) {
///         assert!(complexity_str > threshold);
///     }
/// }
/// # });
/// ```
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn check_complexity(
    project_path: &Path,
    _max_complexity: u32,
) -> Result<Vec<QualityViolation>> {
    use crate::services::complexity::aggregate_results_with_thresholds;

    let mut violations = Vec::new();

    // Thresholds come from the project being analysed — NOT from the process CWD.
    let ComplexityThresholds {
        max_cyclomatic,
        max_cognitive,
        source,
    } = load_complexity_thresholds(project_path);

    // Name the resolved source, so a surprising verdict can be traced to the
    // file that produced it instead of to the shell it was typed in. On stderr:
    // the JSON on stdout has to stay byte-identical across working directories.
    crate::status_eprintln!(
        "  ⚙️  Complexity thresholds: cyclomatic {max_cyclomatic}, cognitive {max_cognitive} (from {})",
        source.as_ref().map_or_else(
            || "built-in defaults".to_string(),
            |p| p.display().to_string()
        )
    );

    // Load exclude_paths from .pmat-metrics.toml for filtering generated files
    let mut exclude_globs = load_exclude_paths(project_path);
    // Built-in excludes: non-production code where high complexity is expected
    for pattern in &[
        "**/examples/**", "**/benches/**", "**/scripts/**",
        "**/tests/**", "**/*_tests.rs", "**/*_tests_*.rs", "**/*tests_part*.rs",
        "**/fixtures/**",
        // Lint rule implementations have inherent pattern-matching complexity
        "**/comply_cb_detect/**", "**/comply_cb_detect.rs",
        // Language analysis infrastructure: inherently complex pattern matching
        "**/dead_code_multi_language.rs",
        "**/mcp_integration/**",
        // MCP tool functions: thin dispatch wrappers with many match arms
        "**/mcp_pmcp/tool_functions/**",
    ] {
        if let Ok(p) = glob::Pattern::new(pattern) {
            exclude_globs.push(p);
        }
    }

    // Use the existing analyze_project_files function - the ONE implementation
    let file_metrics = analyze_project_files(
        project_path,
        None, // Auto-detect toolchain
        &[],  // Empty include pattern means all files
        max_cyclomatic as u16,
        max_cognitive as u16,
    )
    .await?;

    // Check for violations using the same logic as analyze complexity
    let report = aggregate_results_with_thresholds(
        file_metrics,
        Some(max_cyclomatic as u16),
        Some(max_cognitive as u16),
    );

    // Convert violations to QualityViolation format
    // ONLY count actual violations where complexity exceeds threshold
    for violation in &report.violations {
        if !is_violation_excluded(violation, &exclude_globs) {
            process_complexity_violation(violation, &mut violations);
        }
    }

    Ok(violations)
}

/// The complexity thresholds the gate will enforce, plus where they came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ComplexityThresholds {
    pub(crate) max_cyclomatic: u32,
    pub(crate) max_cognitive: u32,
    /// `Some(path)` when a `pmat.toml` inside the analysed project supplied at
    /// least one threshold; `None` when the built-in defaults were used. Exposed
    /// so the verdict can say which file produced it.
    pub(crate) source: Option<PathBuf>,
}

/// Resolve the complexity thresholds for `project_path`.
///
/// # Resolution order (#1020)
///
/// 1. `<project_path>/pmat.toml` `[quality]` — `max_complexity`,
///    `max_cognitive_complexity`.
/// 2. The built-in defaults (`ConfigurationService::default_config()`).
///
/// **The process working directory is not consulted at any step.** This used to
/// read the global `configuration()` singleton, which is built from
/// `std::env::current_dir().join("pmat.toml")`, so the same
/// `pmat quality-gate --project-path X` returned a different verdict depending
/// on where it was invoked: run from this repo (whose `pmat.toml` sets
/// `max_cognitive_complexity = 100`) a fixture reported 1 violation, and run
/// from `/tmp` (no `pmat.toml`, so the default 25 applied) the identical command
/// on the identical tree reported 2. A gate whose answer depends on the caller's
/// shell is not reproducible: CI and a laptop disagree with nothing visible to
/// explain it.
///
/// A CWD config is *ignored outright* rather than kept as a lower-priority
/// fallback. A fallback would still make the verdict a function of the caller's
/// location — merely a rarer one, firing only when the project has no
/// `pmat.toml` of its own, which is precisely the case a fixture or a freshly
/// cloned repo hits. The gate's answer must be a function of (analysed tree, CLI
/// flags) alone. This also makes complexity consistent with every other config
/// reader in the gate — `load_exclude_paths`, `load_entropy_threshold`,
/// `load_max_pattern_repetition`, `load_provability_threshold`,
/// `load_entropy_gate_config` and `load_tdg_gate_overrides` all already resolve
/// against the project path.
///
/// `pmat config --set` still writes the CWD's `pmat.toml`; that is a per-project
/// edit and stays CWD-relative. Only the *gate's reading* of it is pinned to the
/// tree under analysis.
pub(crate) fn load_complexity_thresholds(project_path: &Path) -> ComplexityThresholds {
    use crate::services::configuration_service::ConfigurationService;

    let defaults = ConfigurationService::default_config().quality;
    let mut resolved = ComplexityThresholds {
        max_cyclomatic: defaults.max_complexity,
        max_cognitive: defaults.max_cognitive_complexity,
        source: None,
    };

    // Parsed as a generic table, not as `PmatConfig`: a project's `pmat.toml`
    // may set only the keys it cares about, and `PmatConfig` has no serde
    // defaults, so a strict deserialize of a partial file fails and would
    // silently drop the thresholds the user did write.
    let config_path = project_path.join("pmat.toml");
    let Ok(content) = std::fs::read_to_string(&config_path) else {
        return resolved;
    };
    let Ok(table) = content.parse::<toml::Table>() else {
        return resolved;
    };
    let Some(quality) = table.get("quality") else {
        return resolved;
    };

    // `try_from` rather than `as`: a negative or oversized value must fall back
    // to the default, not wrap into a ceiling (`-1` → 4294967295) that no
    // function could ever breach — a gate silently switched off by a typo.
    let read = |key: &str| {
        quality
            .get(key)
            .and_then(toml::Value::as_integer)
            .and_then(|v| u32::try_from(v).ok())
    };

    if let Some(v) = read("max_complexity") {
        resolved.max_cyclomatic = v;
        resolved.source = Some(config_path.clone());
    }
    if let Some(v) = read("max_cognitive_complexity") {
        resolved.max_cognitive = v;
        resolved.source = Some(config_path);
    }

    resolved
}

/// Load exclude_paths globs from `.pmat-metrics.toml`.
fn load_exclude_paths(project_path: &Path) -> Vec<glob::Pattern> {
    let config_path = project_path.join(".pmat-metrics.toml");
    let content = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let table: toml::Table = match content.parse() {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    table
        .get("exclude_paths")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .filter_map(|s| glob::Pattern::new(s).ok())
                .collect()
        })
        .unwrap_or_default()
}

/// Check if a complexity violation's file matches any exclude_paths glob.
fn is_violation_excluded(
    violation: &crate::services::complexity::Violation,
    exclude_globs: &[glob::Pattern],
) -> bool {
    use crate::services::complexity::Violation;
    let file_path = match violation {
        Violation::Error { file, .. } | Violation::Warning { file, .. } => file,
    };
    exclude_globs
        .iter()
        .any(|pat| pat.matches(file_path) || pat.matches_path(std::path::Path::new(file_path)))
}

/// Process a single complexity violation into `QualityViolation` format
fn process_complexity_violation(
    violation: &crate::services::complexity::Violation,
    violations: &mut Vec<QualityViolation>,
) {
    use crate::services::complexity::Violation;

    let (file, line, function, rule, message, value, threshold, severity) = match violation {
        Violation::Error {
            file,
            line,
            function,
            rule,
            message,
            value,
            threshold,
        } => (
            file, line, function, rule, message, value, threshold, "error",
        ),
        Violation::Warning {
            file,
            line,
            function,
            rule,
            message,
            value,
            threshold,
        } => (
            file, line, function, rule, message, value, threshold, "warning",
        ),
    };

    // Only add if this is an actual threshold violation
    if value > threshold {
        violations.push(QualityViolation {
            check_type: "complexity".to_string(),
            severity: severity.to_string(),
            file: file.clone(),
            line: Some(*line as usize),
            message: format!(
                "{}: {} - {} (complexity: {}, threshold: {})",
                function.as_deref().unwrap_or("global"),
                rule,
                message,
                value,
                threshold
            ),
            details: None,
        });
    }
}

#[cfg(test)]
mod part1_complexity_tests {
    //! Covers the pure-compute helpers in quality_checks_part1_complexity.rs
    //! (77 uncov on broad, 0% cov).
    use super::*;
    use crate::services::complexity::Violation;

    fn error_violation(file: &str, value: u16, threshold: u16) -> Violation {
        Violation::Error {
            rule: "cyclomatic".into(),
            message: "too complex".into(),
            value,
            threshold,
            file: file.into(),
            line: 42,
            function: Some("my_fn".into()),
        }
    }

    fn warning_violation(file: &str, value: u16, threshold: u16) -> Violation {
        Violation::Warning {
            rule: "cyclomatic".into(),
            message: "too complex".into(),
            value,
            threshold,
            file: file.into(),
            line: 42,
            function: None, // exercises `.unwrap_or("global")` branch
        }
    }

    // ── load_exclude_paths ──

    #[test]
    fn test_load_exclude_paths_no_config_returns_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let paths = load_exclude_paths(tmp.path());
        assert!(paths.is_empty());
    }

    #[test]
    fn test_load_exclude_paths_invalid_toml_returns_empty() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join(".pmat-metrics.toml"), "not = [valid toml").unwrap();
        let paths = load_exclude_paths(tmp.path());
        assert!(paths.is_empty());
    }

    #[test]
    fn test_load_exclude_paths_missing_section_returns_empty() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join(".pmat-metrics.toml"), "[thresholds]\n").unwrap();
        let paths = load_exclude_paths(tmp.path());
        assert!(paths.is_empty());
    }

    #[test]
    fn test_load_exclude_paths_parses_array_of_patterns() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join(".pmat-metrics.toml"),
            "exclude_paths = [\"tests/**\", \"benches/**\", \"vendor/*\"]\n",
        )
        .unwrap();
        let paths = load_exclude_paths(tmp.path());
        assert_eq!(paths.len(), 3);
    }

    #[test]
    fn test_load_exclude_paths_skips_invalid_globs() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join(".pmat-metrics.toml"),
            // `[` is an invalid glob start (unclosed char class).
            "exclude_paths = [\"tests/**\", \"[bad\"]\n",
        )
        .unwrap();
        let paths = load_exclude_paths(tmp.path());
        assert_eq!(paths.len(), 1, "invalid glob dropped via filter_map");
    }

    // ── is_violation_excluded ──

    #[test]
    fn test_is_violation_excluded_no_globs_never_excluded() {
        let v = error_violation("src/a.rs", 50, 10);
        assert!(!is_violation_excluded(&v, &[]));
    }

    #[test]
    fn test_is_violation_excluded_matching_glob_excluded() {
        let v = error_violation("tests/foo.rs", 50, 10);
        let globs = vec![glob::Pattern::new("tests/**").unwrap()];
        assert!(is_violation_excluded(&v, &globs));
    }

    #[test]
    fn test_is_violation_excluded_nonmatching_glob_not_excluded() {
        let v = error_violation("src/a.rs", 50, 10);
        let globs = vec![glob::Pattern::new("tests/**").unwrap()];
        assert!(!is_violation_excluded(&v, &globs));
    }

    #[test]
    fn test_is_violation_excluded_matches_warning_variant_too() {
        let v = warning_violation("benches/bench.rs", 20, 10);
        let globs = vec![glob::Pattern::new("benches/**").unwrap()];
        assert!(is_violation_excluded(&v, &globs));
    }

    // ── process_complexity_violation ──

    #[test]
    fn test_process_complexity_violation_error_above_threshold_pushes() {
        let v = error_violation("src/a.rs", 50, 10);
        let mut out = Vec::new();
        process_complexity_violation(&v, &mut out);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].check_type, "complexity");
        assert_eq!(out[0].severity, "error");
        assert_eq!(out[0].line, Some(42));
        // Function name embedded in message.
        assert!(out[0].message.contains("my_fn"));
        assert!(out[0].message.contains("complexity: 50"));
        assert!(out[0].message.contains("threshold: 10"));
    }

    #[test]
    fn test_process_complexity_violation_warning_above_threshold_uses_global_fn() {
        let v = warning_violation("src/b.rs", 20, 10);
        let mut out = Vec::new();
        process_complexity_violation(&v, &mut out);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].severity, "warning");
        // function=None → unwrap_or("global") arm.
        assert!(out[0].message.starts_with("global:"));
    }

    #[test]
    fn test_process_complexity_violation_below_threshold_not_pushed() {
        // value ≤ threshold → skipped.
        let v = error_violation("src/a.rs", 10, 10);
        let mut out = Vec::new();
        process_complexity_violation(&v, &mut out);
        assert!(out.is_empty());
    }
}

#[cfg(test)]
mod complexity_threshold_resolution_tests {
    //! #1020: the gate's complexity thresholds must be a function of the tree
    //! being analysed, never of the process working directory.
    //!
    //! `check_complexity` used to read the global `configuration()` singleton,
    //! which is constructed from `std::env::current_dir().join("pmat.toml")`.
    //! The same `pmat quality-gate --project-path X --checks complexity` on the
    //! same fixture therefore answered `complexity_violations: 1` when run from
    //! this repo and `2` when run from `/tmp`.
    use super::*;

    /// Ten flat `if`s: cyclomatic ≈ 11, cognitive ≈ 10. Comfortably under both
    /// built-in defaults (30 / 25) AND under this repo's `pmat.toml` (30 / 100),
    /// so any CWD a test runner could plausibly have reports zero violations for
    /// it. Only a threshold read out of the fixture itself flags it.
    fn write_fixture_project(dir: &Path) {
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(
            dir.join("Cargo.toml"),
            "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
        let mut body = String::from("pub fn branchy(n: i32) -> i32 {\n    let mut acc = 0;\n");
        for i in 0..10 {
            body.push_str(&format!("    if n > {i} {{ acc += 1; }}\n"));
        }
        body.push_str("    acc\n}\n");
        std::fs::write(dir.join("src").join("lib.rs"), body).unwrap();
    }

    #[tokio::test]
    async fn test_check_complexity_honours_the_analysed_projects_pmat_toml() {
        // RED without the fix: `branchy` is under every threshold reachable from
        // any working directory, so the gate found nothing at all and the
        // fixture's own `pmat.toml` was never opened.
        let tmp = tempfile::tempdir().unwrap();
        write_fixture_project(tmp.path());
        std::fs::write(
            tmp.path().join("pmat.toml"),
            "[quality]\nmax_complexity = 3\nmax_cognitive_complexity = 3\n",
        )
        .unwrap();

        let violations = check_complexity(tmp.path(), 0).await.unwrap();

        assert!(
            !violations.is_empty(),
            "the fixture's own pmat.toml sets max_complexity = 3; `branchy` \
             (cyclomatic ~11) must violate it. Empty means the thresholds came \
             from somewhere other than the analysed project — the CWD."
        );
        assert!(
            violations
                .iter()
                .any(|v| v.message.contains("threshold: 3")),
            "the reported threshold must be the fixture's 3, got: {:?}",
            violations.iter().map(|v| &v.message).collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn test_check_complexity_ignores_a_pmat_toml_outside_the_project() {
        // The converse: a project with NO config of its own gets the built-in
        // defaults, not whatever the caller happens to be standing in. Under
        // `cargo test --lib` the CWD is this repo, whose pmat.toml raises the
        // cognitive ceiling to 100; a fixture at cognitive 72 must still be
        // judged against the default 25.
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("src")).unwrap();
        std::fs::write(
            tmp.path().join("Cargo.toml"),
            "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
        let mut body = String::from("pub fn tangled(v: &[i32]) -> i32 {\n    let mut acc = 0;\n");
        for i in 0..12 {
            body.push_str(&format!(
                "    if v.len() > {i} {{\n        for x in v {{\n            \
                 if *x > {i} {{ acc += 1; }} else {{ acc -= 1; }}\n        }}\n    }}\n"
            ));
        }
        body.push_str("    acc\n}\n");
        std::fs::write(tmp.path().join("src").join("lib.rs"), body).unwrap();

        let violations = check_complexity(tmp.path(), 0).await.unwrap();

        assert!(
            violations
                .iter()
                .any(|v| v.message.contains("cognitive-complexity")),
            "no pmat.toml in the fixture ⇒ default cognitive ceiling 25 applies. \
             Missing means this repo's pmat.toml (100) leaked in from the CWD. \
             Got: {:?}",
            violations.iter().map(|v| &v.message).collect::<Vec<_>>()
        );
    }

    // ── load_complexity_thresholds ──

    #[test]
    fn test_thresholds_default_when_project_has_no_pmat_toml() {
        use crate::services::configuration_service::ConfigurationService;
        let defaults = ConfigurationService::default_config().quality;
        let tmp = tempfile::tempdir().unwrap();
        let t = load_complexity_thresholds(tmp.path());
        assert_eq!(t.max_cyclomatic, defaults.max_complexity);
        assert_eq!(t.max_cognitive, defaults.max_cognitive_complexity);
        assert_eq!(t.source, None, "defaults must report no config source");
    }

    #[test]
    fn test_thresholds_partial_file_keeps_defaults_for_the_absent_key() {
        // `PmatConfig` has no serde defaults, so a strict deserialize of this
        // file fails outright; the loader must still pick up the one key set.
        use crate::services::configuration_service::ConfigurationService;
        let defaults = ConfigurationService::default_config().quality;
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("pmat.toml"),
            "[quality]\nmax_complexity = 7\n",
        )
        .unwrap();
        let t = load_complexity_thresholds(tmp.path());
        assert_eq!(t.max_cyclomatic, 7);
        assert_eq!(t.max_cognitive, defaults.max_cognitive_complexity);
        assert_eq!(t.source, Some(tmp.path().join("pmat.toml")));
    }

    #[test]
    fn test_thresholds_both_keys_read() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("pmat.toml"),
            "[quality]\nmax_complexity = 11\nmax_cognitive_complexity = 13\n",
        )
        .unwrap();
        let t = load_complexity_thresholds(tmp.path());
        assert_eq!(t.max_cyclomatic, 11);
        assert_eq!(t.max_cognitive, 13);
    }

    #[test]
    fn test_thresholds_malformed_toml_falls_back_to_defaults() {
        use crate::services::configuration_service::ConfigurationService;
        let defaults = ConfigurationService::default_config().quality;
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("pmat.toml"), "not = [valid toml").unwrap();
        let t = load_complexity_thresholds(tmp.path());
        assert_eq!(t.max_cyclomatic, defaults.max_complexity);
        assert_eq!(t.source, None);
    }

    #[test]
    fn test_thresholds_out_of_range_values_rejected_not_wrapped() {
        // `-1 as u32` would silently become 4294967295, i.e. a gate that can
        // never fire; `u32::MAX + 1 as u32` truncates to 0, one that always
        // fires. Both must fall back to the default instead.
        use crate::services::configuration_service::ConfigurationService;
        let defaults = ConfigurationService::default_config().quality;
        for value in ["-1", "4294967296"] {
            let tmp = tempfile::tempdir().unwrap();
            std::fs::write(
                tmp.path().join("pmat.toml"),
                format!("[quality]\nmax_complexity = {value}\n"),
            )
            .unwrap();
            let t = load_complexity_thresholds(tmp.path());
            assert_eq!(
                t.max_cyclomatic, defaults.max_complexity,
                "max_complexity = {value} must be rejected, not coerced"
            );
            assert_eq!(t.source, None, "a rejected value names no source");
        }
    }

    #[test]
    fn test_thresholds_are_independent_of_the_process_cwd() {
        // The property under test, stated directly: the resolver takes exactly
        // one input. Two sibling directories with different configs must each
        // report their own numbers, whichever one the process is standing in.
        let root = tempfile::tempdir().unwrap();
        for (name, max) in [("a", 4u32), ("b", 44u32)] {
            let dir = root.path().join(name);
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(
                dir.join("pmat.toml"),
                format!("[quality]\nmax_complexity = {max}\n"),
            )
            .unwrap();
        }
        assert_eq!(
            load_complexity_thresholds(&root.path().join("a")).max_cyclomatic,
            4
        );
        assert_eq!(
            load_complexity_thresholds(&root.path().join("b")).max_cyclomatic,
            44
        );
        // ...and a directory with no config never inherits a sibling's.
        assert_eq!(load_complexity_thresholds(root.path()).source, None);
    }
}
