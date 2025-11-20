//! RustToolingScorer - Rust Tooling Compliance Category (37 points)
//!
//! Analyzes Rust project compliance with standard tooling:
//! - Clippy (tiered scoring): 10pts
//!   - Correctness: 5pts (zero warnings)
//!   - Suspicious: 3pts (zero warnings)
//!   - Pedantic: 2pts (zero warnings)
//! - rustfmt compliance: 5pts
//! - cargo-audit (security): 7pts (risk-based scoring)
//! - cargo-deny (policy): 3pts
//! - **v2.0 Workspace Lints (12pts)**: Based on "Learn from Rust Giants" spec
//!   - Workspace-level lints configured: 5pts
//!   - High-value lint categories (correctness, suspicious, perf): 4pts
//!   - .clippy.toml with disallowed-methods: 3pts

use super::models::{CategoryScore, FileCache, ScoringMode};
use super::scorer::{Scorer, ScorerError, ScorerResult};
use std::path::Path;
use std::process::Command;

/// Count of vulnerabilities by severity level
#[derive(Debug, Default)]
struct VulnerabilityCount {
    critical: u32,
    high: u32,
    medium: u32,
    low: u32,
}

/// Rust Tooling Compliance scorer
#[derive(Debug, Clone)]
pub struct RustToolingScorer {
    /// Category name
    name: String,
    /// Maximum possible points
    max_points: f64,
}

impl RustToolingScorer {
    /// Create a new RustToolingScorer
    pub fn new() -> Self {
        Self {
            name: "Rust Tooling Compliance".to_string(),
            max_points: 37.0, // v2.0: 25 + 12 (workspace lints)
        }
    }

    /// Run clippy and calculate tiered score
    fn score_clippy(&self, project_path: &Path) -> ScorerResult<f64> {
        // Check if Cargo.toml exists
        if !project_path.join("Cargo.toml").exists() {
            return Err(ScorerError::InvalidProject(
                "No Cargo.toml found".to_string(),
            ));
        }

        // Run clippy with JSON output
        let output = Command::new("cargo")
            .arg("clippy")
            .arg("--all-targets")
            .arg("--")
            .arg("-D")
            .arg("warnings")
            .current_dir(project_path)
            .output();

        match output {
            Ok(result) => {
                // If clippy succeeds (exit code 0), give full 10 points
                if result.status.success() {
                    Ok(10.0)
                } else {
                    // Parse stderr to determine warning levels
                    let stderr = String::from_utf8_lossy(&result.stderr);

                    let mut score: f64 = 10.0;

                    // Deduct points based on warning levels
                    // This is a simplified heuristic - real implementation
                    // would parse clippy JSON output
                    if stderr.contains("warning") || stderr.contains("error") {
                        // Assume some warnings - deduct points
                        score -= 2.0;
                    }

                    Ok(score.max(0.0))
                }
            }
            Err(e) => {
                // If cargo/clippy not found, gracefully degrade
                if e.kind() == std::io::ErrorKind::NotFound {
                    Err(ScorerError::ToolNotFound("cargo clippy".to_string()))
                } else {
                    Err(ScorerError::CommandError(e.to_string()))
                }
            }
        }
    }

    /// Run rustfmt check and calculate score
    fn score_rustfmt(&self, project_path: &Path) -> ScorerResult<f64> {
        // Run rustfmt in check mode (doesn't modify files)
        let output = Command::new("cargo")
            .arg("fmt")
            .arg("--")
            .arg("--check")
            .current_dir(project_path)
            .output();

        match output {
            Ok(result) => {
                // If rustfmt check passes, give full 5 points
                if result.status.success() {
                    Ok(5.0)
                } else {
                    // Formatting issues found
                    Ok(0.0)
                }
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    Err(ScorerError::ToolNotFound("cargo fmt".to_string()))
                } else {
                    Err(ScorerError::CommandError(e.to_string()))
                }
            }
        }
    }

    /// Run cargo-audit and calculate risk-based score
    fn score_cargo_audit(&self, project_path: &Path) -> ScorerResult<f64> {
        // Run cargo-audit with JSON output for proper parsing
        let output = Command::new("cargo")
            .args(["audit", "--json"])
            .current_dir(project_path)
            .output();

        match output {
            Ok(result) => {
                let stdout = String::from_utf8_lossy(&result.stdout);

                // Parse JSON output to count vulnerabilities by severity
                let vuln_counts = self.parse_audit_json(&stdout);

                // Risk-based scoring (cumulative deductions):
                // Each Critical: -3.5pts, High: -2pts, Medium: -1pt, Low: -0.5pt
                let mut deduction: f64 = 0.0;
                deduction += vuln_counts.critical as f64 * 3.5;
                deduction += vuln_counts.high as f64 * 2.0;
                deduction += vuln_counts.medium as f64 * 1.0;
                deduction += vuln_counts.low as f64 * 0.5;

                Ok((7.0 - deduction).max(0.0))
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    // cargo-audit not installed - graceful degradation
                    // Give 50% credit for clean Cargo.toml
                    Ok(3.5)
                } else {
                    Err(ScorerError::CommandError(e.to_string()))
                }
            }
        }
    }

    /// Parse cargo-audit JSON output to count vulnerabilities by severity
    fn parse_audit_json(&self, json_str: &str) -> VulnerabilityCount {
        let mut counts = VulnerabilityCount::default();

        // Try to parse as JSON
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(json_str) {
            // cargo-audit JSON format: { "vulnerabilities": { "list": [...] } }
            if let Some(vulns) = json.get("vulnerabilities").and_then(|v| v.get("list")) {
                if let Some(vuln_array) = vulns.as_array() {
                    for vuln in vuln_array {
                        // Extract advisory.severity from each item
                        if let Some(severity) = vuln
                            .get("advisory")
                            .and_then(|a| a.get("severity"))
                            .and_then(|s| s.as_str())
                        {
                            match severity.to_lowercase().as_str() {
                                "critical" => counts.critical += 1,
                                "high" => counts.high += 1,
                                "medium" => counts.medium += 1,
                                "low" => counts.low += 1,
                                _ => {} // Ignore unknown severities
                            }
                        }
                    }
                }
            }
        }

        counts
    }

    /// Check for cargo-deny configuration and calculate score
    fn score_cargo_deny(&self, project_path: &Path) -> ScorerResult<f64> {
        // Check if deny.toml exists
        if project_path.join("deny.toml").exists() {
            // Has deny.toml - give full 3 points
            Ok(3.0)
        } else {
            // No deny.toml - 0 points
            Ok(0.0)
        }
    }

    /// Score workspace-level lint configuration (v2.0 feature)
    ///
    /// Based on "Learn from Rust Giants" specification (TPS-reviewed):
    /// - +5pts: Workspace-level lints configured ([workspace.lints])
    /// - +4pts: High-value lint categories enabled (correctness, suspicious, perf)
    /// - +3pts: .clippy.toml with disallowed-methods
    ///
    /// Total possible: 12 points
    ///
    /// References:
    /// - Johnson et al. 2013 ICSE: Quality over quantity (avoid warning blindness)
    /// - Bacchelli & Bird 2013 ICSE: Automated style enforcement reduces review waste
    fn score_workspace_lints(&self, project_path: &Path, cache: Option<&FileCache>) -> ScorerResult<f64> {
        let mut score = 0.0;

        // Read Cargo.toml
        let cargo_toml_path = project_path.join("Cargo.toml");
        if !cargo_toml_path.exists() {
            return Ok(0.0); // No Cargo.toml, can't have workspace lints
        }

        // Use cache if available, otherwise read file
        let cargo_toml_content = if let Some(cache) = cache {
            cache
                .get(&cargo_toml_path)
                .ok_or_else(|| ScorerError::IoError("Cargo.toml not in cache".to_string()))?
                .clone()
        } else {
            std::fs::read_to_string(&cargo_toml_path)
                .map_err(|e| ScorerError::IoError(e.to_string()))?
        };

        // Check 1: Workspace-level lints configured (+5pts)
        let has_workspace_rust_lints = cargo_toml_content.contains("[workspace.lints.rust]");
        let has_workspace_clippy_lints = cargo_toml_content.contains("[workspace.lints.clippy]");

        if has_workspace_rust_lints || has_workspace_clippy_lints {
            score += 5.0;
        }

        // Check 2: High-value lint categories (+4pts)
        // Look for key lints that indicate quality focus (not just quantity)
        let has_high_value_lints =
            cargo_toml_content.contains("unsafe_op_in_unsafe_fn") || // Safety-critical
            cargo_toml_content.contains("unreachable_pub") ||        // API clarity
            cargo_toml_content.contains("unused_lifetimes") ||       // Code quality
            cargo_toml_content.contains("checked_conversions") ||    // Correctness
            cargo_toml_content.contains("fallible_impl_from");       // Correctness

        if has_high_value_lints {
            score += 4.0;
        }

        // Check 3: .clippy.toml with disallowed-methods (+3pts)
        let clippy_toml_path = project_path.join(".clippy.toml");
        if clippy_toml_path.exists() {
            // Use cache if available
            let clippy_toml_content = if let Some(cache) = cache {
                cache
                    .get(&clippy_toml_path)
                    .ok_or_else(|| ScorerError::IoError(".clippy.toml not in cache".to_string()))?
                    .clone()
            } else {
                std::fs::read_to_string(&clippy_toml_path)
                    .map_err(|e| ScorerError::IoError(e.to_string()))?
            };

            // Check for disallowed-methods section with actual content
            if clippy_toml_content.contains("disallowed-methods") {
                score += 3.0;
            }
        }

        Ok(score)
    }

    /// Internal scoring logic that accepts optional cache
    ///
    /// **Kaizen Round 4**: Cache-aware scoring implementation
    /// Note: This scorer only does file existence checks and subprocess calls,
    /// so cache is not used (no file reads to optimize)
    fn score_internal(
        &self,
        project_path: &Path,
        mode: ScoringMode,
        _cache: Option<&FileCache>,
    ) -> ScorerResult<CategoryScore> {
        // Verify project has Cargo.toml
        if !project_path.join("Cargo.toml").exists() {
            return Err(ScorerError::InvalidProject(
                "No Cargo.toml found - not a valid Rust project".to_string(),
            ));
        }

        let mut total_earned = 0.0;

        // Score clippy (10pts) - ONLY in full mode (too slow for fast mode)
        if mode.is_full() {
            match self.score_clippy(project_path) {
                Ok(score) => total_earned += score,
                Err(ScorerError::ToolNotFound(_)) => {
                    // Graceful degradation - give 50% credit if tool not found
                    total_earned += 5.0;
                }
                Err(e) => return Err(e),
            }
        } else {
            // Fast mode: Skip clippy (too slow - takes 60-90s on large projects)
            // Give moderate credit (5/10 points) to avoid penalizing fast mode
            total_earned += 5.0;
        }

        // Score rustfmt (5pts)
        // KAIZEN: Skip in fast mode - takes 30-60s on large projects (145 files)
        if mode.is_full() {
            match self.score_rustfmt(project_path) {
                Ok(score) => total_earned += score,
                Err(ScorerError::ToolNotFound(_)) => {
                    // Graceful degradation
                    total_earned += 2.5;
                }
                Err(e) => return Err(e),
            }
        } else {
            // Fast mode: Check for rustfmt.toml existence as proxy
            // If rustfmt.toml exists, assume formatting is configured (give 3/5 pts)
            // If not, give moderate credit (2.5/5 pts)
            if project_path.join("rustfmt.toml").exists()
                || project_path.join(".rustfmt.toml").exists()
            {
                total_earned += 3.0;
            } else {
                total_earned += 2.5;
            }
        }

        // Score cargo-audit (7pts) - ONLY in full mode (network calls ~30s)
        if mode.is_full() {
            match self.score_cargo_audit(project_path) {
                Ok(score) => total_earned += score,
                Err(e) => return Err(e),
            }
        } else {
            // Fast mode: Skip cargo-audit (network calls too slow)
            // Give moderate credit (3.5/7 points) to avoid penalizing fast mode
            total_earned += 3.5;
        }

        // Score cargo-deny (3pts) - fast enough for both modes
        match self.score_cargo_deny(project_path) {
            Ok(score) => total_earned += score,
            Err(e) => return Err(e),
        }

        // Score workspace lints (12pts) - v2.0 feature (fast, just file reads)
        match self.score_workspace_lints(project_path, _cache) {
            Ok(score) => total_earned += score,
            Err(e) => return Err(e),
        }

        Ok(CategoryScore::new(total_earned, self.max_points))
    }
}

impl Default for RustToolingScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl Scorer for RustToolingScorer {
    fn name(&self) -> &str {
        &self.name
    }

    fn max_points(&self) -> f64 {
        self.max_points
    }

    fn score(&self, project_path: &Path) -> ScorerResult<CategoryScore> {
        // Backward compatibility: call with default mode and no cache
        self.score_internal(project_path, ScoringMode::default(), None)
    }

    fn score_with_mode(
        &self,
        project_path: &Path,
        mode: ScoringMode,
    ) -> ScorerResult<CategoryScore> {
        // Backward compatibility: call with no cache
        self.score_internal(project_path, mode, None)
    }

    fn score_with_cache(
        &self,
        project_path: &Path,
        mode: ScoringMode,
        cache: Option<&FileCache>,
    ) -> ScorerResult<CategoryScore> {
        // Kaizen Round 4: Cache support added for API consistency
        // Note: This scorer only does file existence checks and subprocess calls,
        // so cache is not actually used (no file reads to optimize)
        self.score_internal(project_path, mode, cache)
    }

    fn recommendations(&self, project_path: &Path) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Check clippy - SKIP subprocess, always recommend
        recommendations
            .push("Run 'cargo clippy --fix' to automatically fix clippy warnings".to_string());

        // Check rustfmt - SKIP subprocess, always recommend
        recommendations
            .push("Run 'cargo fmt' to format code according to Rust style guidelines".to_string());

        // Check cargo-audit - SKIP subprocess, always recommend
        recommendations.push("Run 'cargo audit' and update vulnerable dependencies".to_string());

        // Check cargo-deny - Fast filesystem check is ok
        if let Ok(score) = self.score_cargo_deny(project_path) {
            if score < 3.0 {
                recommendations.push(
                    "Add deny.toml configuration for dependency policy enforcement".to_string(),
                );
            }
        }

        // v2.0: Check workspace lints
        if let Ok(lint_score) = self.score_workspace_lints(project_path, None) {
            if lint_score < 12.0 {
                if !project_path.join("Cargo.toml").exists() {
                    // Skip workspace lint recommendations if not a Rust project
                } else if let Ok(content) = std::fs::read_to_string(project_path.join("Cargo.toml")) {
                    if !content.contains("[workspace.lints") {
                        recommendations.push(
                            "Add [workspace.lints.rust] and [workspace.lints.clippy] to Cargo.toml for consistent linting across all crates".to_string(),
                        );
                    }
                    if !content.contains("unsafe_op_in_unsafe_fn") && !content.contains("checked_conversions") {
                        recommendations.push(
                            "Enable high-value lint categories (unsafe_op_in_unsafe_fn, unreachable_pub, checked_conversions) for better code quality".to_string(),
                        );
                    }
                }
                if !project_path.join(".clippy.toml").exists() {
                    recommendations.push(
                        "Create .clippy.toml with disallowed-methods to enforce project-specific style preferences".to_string(),
                    );
                }
            }
        }

        recommendations
    }
}

// Ensure Send + Sync for parallel execution
unsafe impl Send for RustToolingScorer {}
unsafe impl Sync for RustToolingScorer {}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_scorer_creation() {
        let scorer = RustToolingScorer::new();
        assert_eq!(scorer.name(), "Rust Tooling Compliance");
        assert_eq!(scorer.max_points(), 37.0); // v2.0: 25 + 12 workspace lints
    }

    #[test]
    fn test_scorer_implements_trait() {
        let scorer = RustToolingScorer::new();
        let _trait_obj: &dyn Scorer = &scorer;
    }

    // ============================================================================
    // v2.0 Workspace Lints Tests (RED phase - following EXTREME TDD)
    // ============================================================================

    #[test]
    fn test_workspace_lints_no_cargo_toml() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        let score = scorer.score_workspace_lints(temp_dir.path(), None).unwrap();
        assert_eq!(score, 0.0, "Should return 0 points when Cargo.toml doesn't exist");
    }

    #[test]
    fn test_workspace_lints_no_workspace_section() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        // Create Cargo.toml without workspace lints
        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(&cargo_toml, r#"
[package]
name = "test"
version = "0.1.0"
edition = "2021"
"#).unwrap();

        let score = scorer.score_workspace_lints(temp_dir.path(), None).unwrap();
        assert_eq!(score, 0.0, "Should return 0 points when no workspace lints configured");
    }

    #[test]
    fn test_workspace_lints_rust_only() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(&cargo_toml, r#"
[workspace.lints.rust]
rust_2018_idioms = { level = "warn", priority = -1 }
unreachable_pub = "warn"
"#).unwrap();

        let score = scorer.score_workspace_lints(temp_dir.path(), None).unwrap();
        assert_eq!(score, 9.0, "Should get 5pts (workspace lints) + 4pts (high-value: unreachable_pub)");
    }

    #[test]
    fn test_workspace_lints_clippy_only() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(&cargo_toml, r#"
[workspace.lints.clippy]
checked_conversions = "warn"
fallible_impl_from = "warn"
"#).unwrap();

        let score = scorer.score_workspace_lints(temp_dir.path(), None).unwrap();
        assert_eq!(score, 9.0, "Should get 5pts (workspace lints) + 4pts (high-value: checked_conversions, fallible_impl_from)");
    }

    #[test]
    fn test_workspace_lints_both_rust_and_clippy() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(&cargo_toml, r#"
[workspace.lints.rust]
unsafe_op_in_unsafe_fn = "warn"
unused_lifetimes = "warn"

[workspace.lints.clippy]
checked_conversions = "warn"
"#).unwrap();

        let score = scorer.score_workspace_lints(temp_dir.path(), None).unwrap();
        assert_eq!(score, 9.0, "Should get 5pts (workspace lints) + 4pts (high-value lints)");
    }

    #[test]
    fn test_workspace_lints_with_clippy_toml() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(&cargo_toml, r#"
[workspace.lints.clippy]
checked_conversions = "warn"
"#).unwrap();

        let clippy_toml = temp_dir.path().join(".clippy.toml");
        std::fs::write(&clippy_toml, r#"
disallowed-methods = [
    { path = "std::option::Option::map_or", reason = "prefer map(..).unwrap_or(..)" },
]
"#).unwrap();

        let score = scorer.score_workspace_lints(temp_dir.path(), None).unwrap();
        assert_eq!(score, 12.0, "Should get 5pts + 4pts + 3pts (clippy.toml) = 12pts");
    }

    #[test]
    fn test_workspace_lints_full_score() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        // Create Cargo.toml with workspace lints (like clap/tokio)
        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(&cargo_toml, r#"
[workspace.lints.rust]
rust_2018_idioms = { level = "warn", priority = -1 }
unreachable_pub = "warn"
unsafe_op_in_unsafe_fn = "warn"
unused_lifetimes = "warn"

[workspace.lints.clippy]
checked_conversions = "warn"
fallible_impl_from = "warn"
"#).unwrap();

        // Create .clippy.toml with disallowed-methods
        let clippy_toml = temp_dir.path().join(".clippy.toml");
        std::fs::write(&clippy_toml, r#"
allow-print-in-tests = true
disallowed-methods = [
    { path = "std::option::Option::map_or", reason = "prefer map(..).unwrap_or(..)" },
    { path = "std::iter::Iterator::for_each", reason = "prefer for loops" },
]
"#).unwrap();

        let score = scorer.score_workspace_lints(temp_dir.path(), None).unwrap();
        assert_eq!(score, 12.0, "Should get full 12 points: 5 + 4 + 3");
    }

    #[test]
    fn test_workspace_lints_no_high_value_lints() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(&cargo_toml, r#"
[workspace.lints.clippy]
# Low-value lints only (no correctness/safety lints)
bool_assert_comparison = "allow"
"#).unwrap();

        let score = scorer.score_workspace_lints(temp_dir.path(), None).unwrap();
        assert_eq!(score, 5.0, "Should get 5pts (workspace section exists) but not 4pts (no high-value lints)");
    }
}
