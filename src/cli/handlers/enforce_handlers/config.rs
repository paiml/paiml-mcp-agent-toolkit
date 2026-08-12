#![cfg_attr(coverage_nightly, coverage(off))]
//! Configuration and initialization for quality enforcement

use super::types::QualityProfile;
use crate::cli::EnforceOutputFormat;
use anyhow::Result;
use std::path::PathBuf;

/// Configuration for enforcement loop
pub struct EnforcementConfig {
    pub max_iterations: u32,
    pub target_improvement: Option<f32>,
    pub max_time: Option<u64>,
    pub apply_suggestions: bool,
    pub specific_file: Option<PathBuf>,
    pub include_pattern: Option<String>,
    pub exclude_pattern: Option<String>,
    pub single_file_mode: bool,
    pub dry_run: bool,
    pub show_progress: bool,
    pub format: EnforceOutputFormat,
    pub ci_mode: bool,
}

/// The `--profile standard` thresholds: the loosest of the three.
///
/// `--help` advertises three profiles, but every arm of `load_quality_profile`
/// used to return `QualityProfile::default()` ("For now, return default extreme
/// profile"), so `--profile standard`, `--profile strict` and `--profile
/// extreme` enforced byte-identical thresholds. A flag that changes nothing is
/// worse than no flag: it reports a pass under "standard" that was actually
/// measured against the extreme limits.
fn standard_profile() -> QualityProfile {
    QualityProfile {
        coverage_min: 60.0,
        complexity_max: 30,
        complexity_target: 15,
        tdg_max: 2.5,
        satd_allowed: 20,
        duplication_max_lines: 200,
        big_o_max: "O(n^2)".to_string(),
        provability_min: 0.5,
    }
}

/// The `--profile strict` thresholds: between standard and extreme.
fn strict_profile() -> QualityProfile {
    QualityProfile {
        coverage_min: 70.0,
        complexity_max: 25,
        complexity_target: 12,
        tdg_max: 1.5,
        satd_allowed: 5,
        duplication_max_lines: 50,
        big_o_max: "O(n log n)".to_string(),
        provability_min: 0.7,
    }
}

/// Threshold overrides read from `--config FILE`.
///
/// Every field is optional and every field corresponds to one the enforcement
/// phases actually read, so a key that lands here changes a verdict. Unknown
/// keys are rejected rather than ignored: a config whose typo'd key silently
/// does nothing is the same defect as a flag that silently does nothing.
#[derive(Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfileOverrides {
    coverage_min: Option<f64>,
    complexity_max: Option<u16>,
    complexity_target: Option<u16>,
    tdg_max: Option<f64>,
    satd_allowed: Option<usize>,
    duplication_max_lines: Option<usize>,
    big_o_max: Option<String>,
    provability_min: Option<f64>,
}

impl ProfileOverrides {
    fn apply(self, base: &mut QualityProfile) {
        if let Some(v) = self.coverage_min {
            base.coverage_min = v;
        }
        if let Some(v) = self.complexity_max {
            base.complexity_max = v;
        }
        if let Some(v) = self.complexity_target {
            base.complexity_target = v;
        }
        if let Some(v) = self.tdg_max {
            base.tdg_max = v;
        }
        if let Some(v) = self.satd_allowed {
            base.satd_allowed = v;
        }
        if let Some(v) = self.duplication_max_lines {
            base.duplication_max_lines = v;
        }
        if let Some(v) = self.big_o_max {
            base.big_o_max = v;
        }
        if let Some(v) = self.provability_min {
            base.provability_min = v;
        }
    }
}

/// Load quality profile from name or config file
///
/// `config_path` used to be `_config_path`: `--config /nonexistent.toml` was
/// accepted in silence, exit 0, with a verdict measured against the built-in
/// thresholds — a path the binary never opened, reported as if it had. A path
/// given on the command line is read, and a path that cannot be read is an
/// error, the same rule `-p` follows.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn load_quality_profile(
    profile_name: &str,
    config_path: Option<PathBuf>,
) -> Result<QualityProfile> {
    let mut profile = match profile_name {
        "standard" => standard_profile(),
        "strict" => strict_profile(),
        // `extreme` is the RIGID profile `--help` describes, and is the default.
        "extreme" => QualityProfile::default(),
        other => anyhow::bail!(
            "Unknown quality profile: {other}. Valid profiles: standard, strict, extreme"
        ),
    };

    if let Some(path) = config_path {
        let text = std::fs::read_to_string(&path).map_err(|e| {
            anyhow::anyhow!(
                "cannot read quality config {}: {e} — enforce will not report a verdict measured against thresholds it could not load",
                path.display()
            )
        })?;
        let overrides: ProfileOverrides = toml::from_str(&text)
            .map_err(|e| anyhow::anyhow!("invalid quality config {}: {e}", path.display()))?;
        overrides.apply(&mut profile);
    }

    Ok(profile)
}

/// Initialize enforcement environment
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn initialize_enforcement_environment(
    profile_name: &str,
    config_path: Option<PathBuf>,
    cache_dir: &Option<PathBuf>,
    clear_cache: bool,
) -> Result<QualityProfile> {
    let profile = load_quality_profile(profile_name, config_path)?;

    if clear_cache {
        clear_enforcement_cache(cache_dir)?;
    }

    Ok(profile)
}

/// Clear the enforcement cache directory named by `--cache-dir`.
///
/// This used to be a stub: with `--cache-dir DIR` it printed
/// "🧹 Clearing cache at: DIR" above `// In real implementation, would clear
/// cache` (the entries were still there afterwards), and with no `--cache-dir`
/// — the default — it printed nothing and returned, so `enforce extreme
/// --clear-cache` was byte-identical to `enforce extreme`.
///
/// Now it really empties the directory via the one shared implementation
/// ([`crate::cli::cache_clearing::clear_cache_directory`]) and says so in every
/// branch, including the branch where there is nothing to clear.
///
/// # Errors
///
/// Returns an error when the cache directory exists but cannot be cleared;
/// `--clear-cache` that fails must not be reported as a clear.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn clear_enforcement_cache(cache_dir: &Option<PathBuf>) -> Result<()> {
    let Some(cache_path) = cache_dir else {
        eprintln!(
            "🧹 --clear-cache: no --cache-dir given and enforce keeps no cache of its own — \
             nothing to clear (every phase is recomputed on each run)"
        );
        return Ok(());
    };

    crate::cli::cache_clearing::clear_cache_directory_reporting(cache_path, "--clear-cache")?;
    Ok(())
}

#[cfg(test)]
mod profile_selection_tests {
    //! Regression tests for `--profile` being a no-op: `load_quality_profile`
    //! matched on the name and returned `QualityProfile::default()` in every
    //! arm, so all three documented profiles enforced the same thresholds.
    use super::load_quality_profile;

    #[test]
    fn test_named_profiles_have_distinct_thresholds() {
        let standard = load_quality_profile("standard", None).unwrap();
        let strict = load_quality_profile("strict", None).unwrap();
        let extreme = load_quality_profile("extreme", None).unwrap();

        // Ordered from loosest to strictest on every threshold the enforcement
        // phases actually read.
        assert!(
            standard.complexity_max > strict.complexity_max
                && strict.complexity_max > extreme.complexity_max,
            "complexity_max must tighten: {} / {} / {}",
            standard.complexity_max,
            strict.complexity_max,
            extreme.complexity_max
        );
        assert!(
            standard.tdg_max > strict.tdg_max && strict.tdg_max > extreme.tdg_max,
            "tdg_max must tighten"
        );
        assert!(
            standard.coverage_min < strict.coverage_min
                && strict.coverage_min < extreme.coverage_min,
            "coverage_min must rise"
        );
        assert!(
            standard.satd_allowed > strict.satd_allowed
                && strict.satd_allowed > extreme.satd_allowed,
            "satd_allowed must tighten"
        );
        assert!(
            standard.duplication_max_lines > strict.duplication_max_lines
                && strict.duplication_max_lines > extreme.duplication_max_lines,
            "duplication_max_lines must tighten"
        );
    }

    #[test]
    fn test_unknown_profile_is_an_error_not_a_silent_extreme() {
        let err = load_quality_profile("toyota", None)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("standard") && err.contains("strict") && err.contains("extreme"),
            "the error must name the valid profiles, got {err}"
        );
    }
}
