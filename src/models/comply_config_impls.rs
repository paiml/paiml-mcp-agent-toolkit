// Impl blocks for comply config types

impl Default for CrossCrateConfig {
    fn default() -> Self {
        Self {
            excluded_functions: Vec::new(),
            excluded_crate_pairs: Vec::new(),
            min_body_lines: default_min_body_lines(),
            min_tokens: default_min_tokens(),
            cc003_min_similarity: default_cc003_similarity(),
        }
    }
}

impl Default for ComplyConfig {
    fn default() -> Self {
        Self {
            checks: default_checks(),
            thresholds: ComplyThresholds::default(),
            fail_fast: false,
            output: OutputConfig::default(),
            suppressions: Vec::new(),
        }
    }
}

impl Default for CheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            severity: CheckSeverity::Warning,
            threshold: None,
            options: HashMap::new(),
        }
    }
}

impl Default for ComplyThresholds {
    fn default() -> Self {
        Self {
            coverage: default_coverage(),
            per_file_coverage: default_per_file_coverage(),
            complexity: default_complexity(),
            dead_code_pct: default_dead_code(),
            max_file_lines: default_file_size(),
            max_function_lines: default_function_size(),
            slow_test_seconds: default_slow_test(),
            slow_coverage_minutes: default_slow_coverage(),
            min_tdg_grade: default_min_tdg_grade(),
            tdg_exclude_paths: Vec::new(),
        }
    }
}

impl PmatYamlConfig {
    /// Load configuration from .pmat.yaml in the given directory
    ///
    /// Returns default configuration if file doesn't exist.
    /// Returns error if file exists but is malformed.
    pub fn load(project_path: &Path) -> Result<Self, ConfigError> {
        let config_path = project_path.join(".pmat.yaml");

        if !config_path.exists() {
            // Also check for .pmat.yml
            let alt_path = project_path.join(".pmat.yml");
            if !alt_path.exists() {
                return Ok(Self::default());
            }
            return Self::load_from_path(&alt_path);
        }

        Self::load_from_path(&config_path)
    }

    /// Load configuration from a specific file path
    pub fn load_from_path(path: &Path) -> Result<Self, ConfigError> {
        let content =
            std::fs::read_to_string(path).map_err(|e| ConfigError::IoError(e.to_string()))?;

        serde_yaml_ng::from_str(&content).map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    /// Save configuration to .pmat.yaml in the given directory
    pub fn save(&self, project_path: &Path) -> Result<(), ConfigError> {
        let config_path = project_path.join(".pmat.yaml");
        let content =
            serde_yaml_ng::to_string(self).map_err(|e| ConfigError::SerializeError(e.to_string()))?;

        std::fs::write(config_path, content).map_err(|e| ConfigError::IoError(e.to_string()))
    }
}

impl ComplyConfig {
    /// Check if a specific check is enabled
    pub fn is_check_enabled(&self, check_id: &str) -> bool {
        self.checks.get(check_id).map(|c| c.enabled).unwrap_or(true) // Default to enabled for unknown checks
    }

    /// Get the severity for a check
    pub fn get_severity(&self, check_id: &str) -> CheckSeverity {
        self.checks
            .get(check_id)
            .map(|c| c.severity)
            .unwrap_or(CheckSeverity::Warning)
    }

    /// Get the threshold for a check
    pub fn get_threshold(&self, check_id: &str) -> Option<f64> {
        self.checks.get(check_id).and_then(|c| c.threshold)
    }

    /// Check if a severity level should cause failure
    pub fn should_fail(&self, severity: CheckSeverity, strict: bool) -> bool {
        match severity {
            CheckSeverity::Critical | CheckSeverity::Error => true,
            CheckSeverity::Warning => strict,
            CheckSeverity::Info => false,
        }
    }

    /// Check if a specific violation should be suppressed.
    ///
    /// Matches against the `suppressions` rules from `.pmat.yaml`.
    /// Returns `Some(reason)` if suppressed, `None` if not.
    pub fn is_suppressed(&self, check_id: &str, file_path: &str) -> Option<String> {
        let today = current_date_iso();
        for rule in &self.suppressions {
            // Check rule ID match (case-insensitive)
            let id_matches = rule.rules.iter().any(|r| r.eq_ignore_ascii_case(check_id));
            if !id_matches {
                continue;
            }

            // Check expiry
            if let Some(ref expires) = rule.expires {
                if expires.as_str() < today.as_str() {
                    continue; // Expired rule, skip
                }
            }

            // Check file glob match (if file globs specified)
            if !rule.files.is_empty() {
                let matches_any = rule.files.iter().any(|pattern| {
                    glob::Pattern::new(pattern)
                        .map(|p| {
                            let opts = glob::MatchOptions {
                                case_sensitive: true,
                                require_literal_separator: false,
                                require_literal_leading_dot: false,
                            };
                            p.matches_with(file_path, opts)
                        })
                        .unwrap_or(false)
                });
                if !matches_any {
                    continue;
                }
            }

            // All conditions matched
            return Some(rule.reason.clone());
        }
        None
    }
}

/// Get current date in ISO 8601 format for expiry comparison
fn current_date_iso() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Howard Hinnant's civil date algorithm
    let z = (secs / 86400) as i64 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::IoError(e) => write!(f, "IO error loading config: {}", e),
            ConfigError::ParseError(e) => write!(f, "Parse error in .pmat.yaml: {}", e),
            ConfigError::SerializeError(e) => write!(f, "Serialization error: {}", e),
        }
    }
}

impl std::error::Error for ConfigError {}
