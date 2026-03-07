// DBC Lint Configuration for pmat work contracts
// Spec: docs/specifications/dbc.md §13.7
//
// Per-project lint configuration via `.pmat-work/dbc-lint.toml`.
// Supports: rule severity overrides, rule suppression, score thresholds,
// trend tracking config, and diff-aware linting.

/// Lint configuration loaded from `.pmat-work/dbc-lint.toml`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintConfig {
    /// Minimum score threshold (default: 0.0, no threshold)
    #[serde(default)]
    pub min_score: f64,
    /// Promote all warnings to errors
    #[serde(default)]
    pub strict: bool,
    /// Per-rule severity overrides (rule_id -> "error"|"warning"|"info"|"off")
    #[serde(default)]
    pub rules: HashMap<String, String>,
    /// Suppressed rules (completely silenced)
    #[serde(default)]
    pub suppress: Vec<String>,
    /// Trend tracking configuration
    #[serde(default)]
    pub trend: LintTrendConfig,
}

/// Trend tracking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintTrendConfig {
    /// Enable trend tracking (default: true)
    #[serde(default = "lint_config_default_true")]
    pub enabled: bool,
    /// Retention in days (default: 90)
    #[serde(default = "lint_config_default_retention_days")]
    pub retention_days: u32,
    /// Drift threshold percentage (default: 0.05 = 5%)
    #[serde(default = "lint_config_default_drift_threshold")]
    pub drift_threshold: f64,
}

fn lint_config_default_true() -> bool {
    true
}
fn lint_config_default_retention_days() -> u32 {
    90
}
fn lint_config_default_drift_threshold() -> f64 {
    0.05
}

impl Default for LintTrendConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            retention_days: 90,
            drift_threshold: 0.05,
        }
    }
}

impl Default for LintConfig {
    fn default() -> Self {
        Self {
            min_score: 0.0,
            strict: false,
            rules: HashMap::new(),
            suppress: Vec::new(),
            trend: LintTrendConfig::default(),
        }
    }
}

impl LintConfig {
    /// Load lint configuration from `.pmat-work/dbc-lint.toml`.
    /// Returns default config if file doesn't exist.
    pub fn load(project_path: &Path) -> Self {
        let config_path = project_path.join(".pmat-work").join("dbc-lint.toml");
        if !config_path.exists() {
            return Self::default();
        }
        match std::fs::read_to_string(&config_path) {
            Ok(content) => Self::parse(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Parse lint config from TOML string
    pub fn parse(toml_str: &str) -> Result<Self> {
        // Manual TOML parsing (avoid adding toml dep just for this)
        let mut config = Self::default();

        for line in toml_str.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with('[') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_matches('"');

                match key {
                    "min_score" => {
                        if let Ok(v) = value.parse::<f64>() {
                            config.min_score = v;
                        }
                    }
                    "strict" => {
                        config.strict = value == "true";
                    }
                    "retention_days" => {
                        if let Ok(v) = value.parse::<u32>() {
                            config.trend.retention_days = v;
                        }
                    }
                    "drift_threshold" => {
                        if let Ok(v) = value.parse::<f64>() {
                            config.trend.drift_threshold = v;
                        }
                    }
                    "enabled" => {
                        config.trend.enabled = value == "true";
                    }
                    _ => {
                        // Rule overrides: DBC-VAL-001 = "error"
                        if key.starts_with("DBC-") {
                            config.rules.insert(key.to_string(), value.to_string());
                        }
                        // Suppress entries
                        if key == "suppress" || key == "rules" {
                            // Handle array-like: suppress = ["DBC-AUD-002", "DBC-AUD-003"]
                            let items: Vec<String> = value
                                .trim_matches(|c| c == '[' || c == ']')
                                .split(',')
                                .map(|s| s.trim().trim_matches('"').to_string())
                                .filter(|s| !s.is_empty())
                                .collect();
                            config.suppress.extend(items);
                        }
                    }
                }
            }
        }

        Ok(config)
    }

    /// Get the effective severity for a rule, considering overrides and strict mode
    pub fn effective_severity(&self, rule_id: &str, default: LintSeverity) -> Option<LintSeverity> {
        // Suppressed rules are completely silenced
        if self.suppress.contains(&rule_id.to_string()) {
            return None;
        }

        // Check for per-rule override
        if let Some(override_sev) = self.rules.get(rule_id) {
            return match override_sev.as_str() {
                "off" => None,
                "error" => Some(LintSeverity::Error),
                "warning" => Some(LintSeverity::Warning),
                "info" => Some(LintSeverity::Info),
                _ => Some(default),
            };
        }

        // Strict mode: promote warnings to errors
        if self.strict && default == LintSeverity::Warning {
            return Some(LintSeverity::Error);
        }

        Some(default)
    }

    /// Check if a rule is suppressed
    pub fn is_suppressed(&self, rule_id: &str) -> bool {
        self.suppress.contains(&rule_id.to_string())
            || self
                .rules
                .get(rule_id)
                .map(|v| v == "off")
                .unwrap_or(false)
    }
}

/// Apply lint config to a completed LintReport, filtering and adjusting findings.
pub fn apply_lint_config(report: &LintReport, config: &LintConfig) -> LintReport {
    let mut findings: Vec<LintFinding> = report
        .findings
        .iter()
        .filter_map(|f| {
            let default_severity = f.severity;
            config.effective_severity(&f.rule_id, default_severity).map(|new_severity| LintFinding {
                rule_id: f.rule_id.clone(),
                severity: new_severity,
                message: f.message.clone(),
                clause_id: f.clause_id.clone(),
            })
        })
        .collect();

    // Apply min_score from config if set and higher than what was passed
    if config.min_score > 0.0 {
        // Score threshold is handled by the caller (lint_contract already has min_score param)
        // This is for informational purposes
    }

    let error_count = findings
        .iter()
        .filter(|f| f.severity == LintSeverity::Error)
        .count();
    let warning_count = findings
        .iter()
        .filter(|f| f.severity == LintSeverity::Warning)
        .count();
    let info_count = findings
        .iter()
        .filter(|f| f.severity == LintSeverity::Info)
        .count();

    // Sort findings by severity (errors first, then warnings, then info)
    findings.sort_by_key(|f| match f.severity {
        LintSeverity::Error => 0,
        LintSeverity::Warning => 1,
        LintSeverity::Info => 2,
    });

    LintReport {
        passed: error_count == 0,
        findings,
        error_count,
        warning_count,
        info_count,
    }
}

/// List all changed contract IDs since a git ref (diff-aware linting, §13.6).
///
/// Scans `.pmat-work/*/contract.json` for contracts modified since `git_ref`.
/// Returns work item IDs of changed contracts.
pub fn changed_contracts_since(project_path: &Path, git_ref: &str) -> Vec<String> {
    let output = std::process::Command::new("git")
        .args(["diff", "--name-only", git_ref, "HEAD"])
        .current_dir(project_path)
        .output();

    let output = match output {
        Ok(o) if o.status.success() => o,
        _ => return Vec::new(),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut changed_ids = Vec::new();

    for line in stdout.lines() {
        // Match .pmat-work/<ID>/contract.json or .pmat-work/<ID>/checkpoints/*
        if let Some(rest) = line.strip_prefix(".pmat-work/") {
            if let Some(id) = rest.split('/').next() {
                if id != "ledger.jsonl" && id != "trusted-stacks.json" && !changed_ids.contains(&id.to_string()) {
                    changed_ids.push(id.to_string());
                }
            }
        }
    }

    changed_ids
}

/// Codebase-level scoring: aggregate across all active work contracts (§14.6).
///
/// Computes portfolio-level quality metrics across all work items.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodebaseScore {
    /// Number of active work contracts
    pub contract_count: usize,
    /// Fraction of contracts with score >= 0.60
    pub contract_coverage: f64,
    /// Mean contract score across all items
    pub mean_score: f64,
    /// Minimum contract score (weakest link)
    pub min_score: f64,
    /// Maximum contract score (best contract)
    pub max_score: f64,
    /// Mean drift bound across contracts
    pub mean_drift: f64,
    /// Fraction of contracts that pass lint (zero errors)
    pub lint_pass_rate: f64,
    /// Composite codebase score (weighted)
    pub composite: f64,
    /// Grade (A-F)
    pub grade: ScoreGrade,
}

/// Compute codebase-level scoring across all active work contracts.
pub fn compute_codebase_score(project_path: &Path) -> CodebaseScore {
    let pmat_work = project_path.join(".pmat-work");
    if !pmat_work.exists() {
        return CodebaseScore {
            contract_count: 0,
            contract_coverage: 0.0,
            mean_score: 0.0,
            min_score: 0.0,
            max_score: 0.0,
            mean_drift: 0.0,
            lint_pass_rate: 0.0,
            composite: 0.0,
            grade: ScoreGrade::F,
        };
    }

    let mut scores = Vec::new();
    let mut drift_bounds = Vec::new();
    let mut lint_passes = 0usize;
    let mut total_contracts = 0usize;

    // Scan .pmat-work/*/contract.json
    if let Ok(entries) = std::fs::read_dir(&pmat_work) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let contract_path = path.join("contract.json");
            if !contract_path.exists() {
                continue;
            }

            // Load and score the contract
            let item_id = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            if let Ok(contract) = WorkContract::load(project_path, item_id) {
                total_contracts += 1;

                let score = score_contract(&contract, project_path);
                scores.push(score.total);

                let drift = compute_drift_metrics(&contract, project_path);
                drift_bounds.push(drift.bounded_drift);

                let lint = lint_contract(&contract, project_path, 0.0);
                if lint.passed {
                    lint_passes += 1;
                }
            }
        }
    }

    if total_contracts == 0 {
        return CodebaseScore {
            contract_count: 0,
            contract_coverage: 0.0,
            mean_score: 0.0,
            min_score: 0.0,
            max_score: 0.0,
            mean_drift: 0.0,
            lint_pass_rate: 0.0,
            composite: 0.0,
            grade: ScoreGrade::F,
        };
    }

    let mean_score = scores.iter().sum::<f64>() / total_contracts as f64;
    let min_score = scores
        .iter()
        .cloned()
        .fold(f64::INFINITY, f64::min);
    let max_score = scores
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);
    let contract_coverage = scores.iter().filter(|s| **s >= 0.60).count() as f64
        / total_contracts as f64;
    let mean_drift = drift_bounds.iter().sum::<f64>() / total_contracts as f64;
    let lint_pass_rate = lint_passes as f64 / total_contracts as f64;

    // Composite: 40% mean_score + 25% contract_coverage + 20% lint_pass_rate + 15% (1 - mean_drift)
    let composite = 0.40 * mean_score
        + 0.25 * contract_coverage
        + 0.20 * lint_pass_rate
        + 0.15 * (1.0 - mean_drift).max(0.0);

    CodebaseScore {
        contract_count: total_contracts,
        contract_coverage,
        mean_score,
        min_score,
        max_score,
        mean_drift,
        lint_pass_rate,
        composite,
        grade: ScoreGrade::from_score(composite),
    }
}
