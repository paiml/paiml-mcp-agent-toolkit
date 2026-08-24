// Tool-based scoring: clippy, rustfmt, cargo-audit, cargo-deny
// Included into rust_tooling_scorer.rs

impl RustToolingScorer {
    /// Points each tool check is worth, so a check that did not run can leave
    /// the DENOMINATOR by exactly what it would have contributed.
    ///
    /// #1035: all three used to award invented partial credit instead —
    /// `Ok(5.0) // Fast mode: moderate credit`, `Ok(3.5) // Fast mode`,
    /// `Err(ToolNotFound(_)) => Ok(2.5)`. In fast mode, which is the default
    /// (`pmat rust-project-score` without `--full`), that is 22 of this
    /// category's 130 points handed out with no measurement taken: a project
    /// with twenty critical advisories and one with none scored the same 3.5/7
    /// for "cargo-audit (security)", and `score_audit_by_mode` in fast mode
    /// returned before it could look at the project at all — a check that
    /// could not fail.
    ///
    /// rustfmt's fast branch was worse than flat: it paid 3.0 rather than 2.5
    /// for the mere PRESENCE of a `rustfmt.toml`, so a config file nobody ran
    /// moved the score. That is the same "filesystem trinket" this repository
    /// already deleted from Code Quality's mutation heuristic.
    ///
    /// The counter-measure is the one Code Quality already uses
    /// (`code_quality_scoring_methods.rs`, `UNMEASURED_IN_FAST_MODE`): N/A, not
    /// half marks. A project scoring 100% of what was measured, over a smaller
    /// denominator, is an honest report; 50% of a check that never ran is not.
    pub(super) const CLIPPY_POINTS: f64 = 10.0;
    pub(super) const RUSTFMT_POINTS: f64 = 5.0;
    pub(super) const AUDIT_POINTS: f64 = 7.0;

    /// Run clippy and calculate tiered score
    pub(super) fn score_clippy(&self, project_path: &Path) -> ScorerResult<f64> {
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
    pub(super) fn score_rustfmt(&self, project_path: &Path) -> ScorerResult<f64> {
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
    pub(super) fn score_cargo_audit(&self, project_path: &Path) -> ScorerResult<f64> {
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
                    // cargo-audit is not installed, so no advisory database was
                    // consulted. This used to award 3.5 of 7 — "50% credit for
                    // clean Cargo.toml", over a Cargo.toml nothing had read.
                    // The caller turns this into N/A, not into half marks.
                    Err(ScorerError::ToolNotFound("cargo audit".to_string()))
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
    pub(super) fn score_cargo_deny(&self, project_path: &Path) -> ScorerResult<f64> {
        // Check if deny.toml exists
        if project_path.join("deny.toml").exists() {
            // Has deny.toml - give full 3 points
            Ok(3.0)
        } else {
            // No deny.toml - 0 points
            Ok(0.0)
        }
    }

    /// Score clippy based on mode (10pts)
    /// `Ok(None)` means NOT MEASURED — fast mode does not run clippy, and a
    /// machine without the toolchain cannot. See [`Self::CLIPPY_POINTS`].
    pub(super) fn score_clippy_by_mode(
        &self,
        project_path: &Path,
        mode: ScoringMode,
    ) -> ScorerResult<Option<f64>> {
        if !mode.is_full() {
            return Ok(None);
        }
        match self.score_clippy(project_path) {
            Ok(score) => Ok(Some(score)),
            Err(ScorerError::ToolNotFound(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Score rustfmt based on mode (5pts); `Ok(None)` means not measured.
    pub(super) fn score_rustfmt_by_mode(
        &self,
        project_path: &Path,
        mode: ScoringMode,
    ) -> ScorerResult<Option<f64>> {
        if !mode.is_full() {
            // The presence of a `rustfmt.toml` used to be worth half a point
            // here. A config file is not a formatted tree.
            return Ok(None);
        }
        match self.score_rustfmt(project_path) {
            Ok(score) => Ok(Some(score)),
            Err(ScorerError::ToolNotFound(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Score cargo-audit based on mode (7pts); `Ok(None)` means not measured.
    pub(super) fn score_audit_by_mode(
        &self,
        project_path: &Path,
        mode: ScoringMode,
    ) -> ScorerResult<Option<f64>> {
        if !mode.is_full() {
            return Ok(None);
        }
        match self.score_cargo_audit(project_path) {
            Ok(score) => Ok(Some(score)),
            // cargo-audit absent: no advisory database was consulted, so no
            // statement about this project's advisories can be made. It used
            // to be worth 3.5 of 7 — "50% credit for clean Cargo.toml", over a
            // Cargo.toml nothing had looked at.
            Err(ScorerError::ToolNotFound(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
