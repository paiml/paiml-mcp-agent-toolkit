/// Root configuration loaded from .pmat.yaml
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PmatYamlConfig {
    /// Comply-specific configuration
    #[serde(default)]
    pub comply: ComplyConfig,

    /// Quality gate configuration
    #[serde(default)]
    pub quality: QualityConfig,

    /// Work contract configuration
    #[serde(default)]
    pub work: WorkConfig,

    /// Project-specific scoring plugins
    #[serde(default)]
    pub scoring: ScoringPluginConfig,

    /// Cross-crate analysis configuration
    #[serde(default)]
    pub cross_crate: CrossCrateConfig,
}

/// Configuration for cross-crate duplication analysis (CC-001 through CC-005).
///
/// Example `.pmat.yaml`:
/// ```yaml
/// cross_crate:
///   excluded_functions: [shape, dim, duration, alpha, vocab_size]
///   excluded_crate_pairs: ["trueno:aprender"]
///   min_body_lines: 5
///   cc003_min_similarity: 0.5
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossCrateConfig {
    /// Function names to exclude from clone detection (e.g., trivial accessors)
    #[serde(default)]
    pub excluded_functions: Vec<String>,

    /// Crate pairs to exclude from analysis (format: "crate_a:crate_b")
    #[serde(default)]
    pub excluded_crate_pairs: Vec<String>,

    /// Minimum function body lines for clone detection (default: 3)
    #[serde(default = "default_min_body_lines")]
    pub min_body_lines: usize,

    /// Minimum token count for meaningful MinHash comparison (default: 15)
    #[serde(default = "default_min_tokens")]
    pub min_tokens: usize,

    /// Minimum Jaccard similarity for CC-003 upstream reimplementation (default: 0.5)
    #[serde(default = "default_cc003_similarity")]
    pub cc003_min_similarity: f64,
}

/// Configuration for pmat comply checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplyConfig {
    /// Individual check configurations keyed by check ID (e.g., "cb-050").
    ///
    /// PMAT-630: a **patch** over [`default_checks`], never a replacement. An id
    /// the map does not name keeps the severity and threshold the code declared
    /// for it; an entry that sets only `enabled:` keeps the rest of its
    /// declaration. See [`deserialize_checks_patch`].
    #[serde(
        default = "default_checks",
        deserialize_with = "deserialize_checks_patch"
    )]
    pub checks: HashMap<String, CheckConfig>,

    /// Global thresholds
    #[serde(default)]
    pub thresholds: ComplyThresholds,

    /// Whether to fail on first error or collect all errors
    #[serde(default)]
    pub fail_fast: bool,

    /// Output format preferences
    #[serde(default)]
    pub output: OutputConfig,

    /// Per-project suppression rules for false positive management
    #[serde(default)]
    pub suppressions: Vec<SuppressionYamlRule>,
}

/// A suppression rule loaded from .pmat.yaml
///
/// Example YAML:
/// ```yaml
/// comply:
///   suppressions:
///     - rules: ["CB-954"]
///       reason: "max_tokens is an LLM parameter, not a secret"
///     - rules: ["CB-501"]
///       files: ["examples/**"]
///       reason: "Examples use unwrap for brevity"
///     - rules: ["CB-516"]
///       files: ["src/constants.rs"]
///       reason: "Constants file legitimately contains magic numbers"
///       expires: "2026-12-31"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuppressionYamlRule {
    /// Check IDs to suppress (e.g., ["CB-954", "CB-501"])
    pub rules: Vec<String>,
    /// Optional glob patterns for file matching (e.g., ["examples/**"])
    #[serde(default)]
    pub files: Vec<String>,
    /// Required reason for audit trail
    pub reason: String,
    /// Optional expiry date (ISO 8601: "2026-12-31")
    #[serde(default)]
    pub expires: Option<String>,
}

/// Configuration for an individual check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckConfig {
    /// Whether the check is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Severity level for this check
    #[serde(default)]
    pub severity: CheckSeverity,

    /// Optional threshold for numeric checks
    #[serde(default)]
    pub threshold: Option<f64>,

    /// Additional check-specific options
    #[serde(default)]
    pub options: HashMap<String, serde_yaml_ng::Value>,
}

/// One `checks:` entry as written in `.pmat.yaml`: every field optional.
///
/// PMAT-630. Deserializing straight into [`CheckConfig`] made every omitted key
/// resolve to that type's own default, so `cb-2100: { enabled: false }` also
/// reset the severity from `Error` to `Warning`. A patch says the only thing the
/// YAML actually says — "these keys, and nothing about the others".
///
/// `deny_unknown_fields` is the fail-closed half: `severty: error` was silently
/// discarded, leaving the rule at its declared severity while the author
/// believed they had raised it. A key nobody reads is a config that lies.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct CheckConfigPatch {
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    severity: Option<CheckSeverity>,
    #[serde(default)]
    threshold: Option<f64>,
    #[serde(default)]
    options: Option<HashMap<String, serde_yaml_ng::Value>>,
}

impl CheckConfigPatch {
    /// Apply over what the code declared for this id.
    fn apply(self, base: CheckConfig) -> CheckConfig {
        CheckConfig {
            enabled: self.enabled.unwrap_or(base.enabled),
            severity: self.severity.unwrap_or(base.severity),
            threshold: self.threshold.or(base.threshold),
            options: self.options.unwrap_or(base.options),
        }
    }
}

/// Merge a `.pmat.yaml` `checks:` map over [`default_checks`].
///
/// The old behaviour was plain `#[serde(default)]`, which built the map from the
/// YAML alone. A `.pmat.yaml` naming seven ids therefore left `get_severity`
/// answering `Warning` for the other fifteen declared rules — CB-2100's own
/// `Error` among them — and `error_severity_rules` came back **empty**, which
/// made "every severity=error rule is enforced" vacuously true. That is the
/// absence-rendered-as-success defect CB-2100 exists to find, in CB-2100's own
/// configuration loader.
///
/// Merge, rather than "error on an id the map omits": the map's purpose is to
/// override a handful of rules, and a config that had to enumerate all 22 to
/// disable one would be unwritable and would rot on the next rule added. An id
/// the map does name is patched key-by-key; an id it does not name is untouched;
/// an id nobody declared is added.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "severity_resolution")]
fn deserialize_checks_patch<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, CheckConfig>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let patches = HashMap::<String, CheckConfigPatch>::deserialize(deserializer)?;
    let mut merged = default_checks();
    for (id, patch) in patches {
        let base = merged.get(&id).cloned().unwrap_or_default();
        merged.insert(id, patch.apply(base));
    }
    Ok(merged)
}

/// Severity levels for checks (matches check_handlers.rs Severity enum)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum CheckSeverity {
    /// Informational - logged but doesn't fail
    Info,
    /// Warning - logged, may fail in strict mode
    #[default]
    Warning,
    /// Error - always fails
    Error,
    /// Critical - blocks all further checks
    Critical,
}

/// Global thresholds for comply checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplyThresholds {
    /// Minimum coverage percentage (0-100)
    #[serde(default = "default_coverage")]
    pub coverage: f64,

    /// Minimum per-file coverage percentage (0-100)
    #[serde(default = "default_per_file_coverage")]
    pub per_file_coverage: f64,

    /// Maximum cyclomatic complexity per function
    #[serde(default = "default_complexity")]
    pub complexity: u32,

    /// Maximum allowed dead code percentage
    #[serde(default = "default_dead_code")]
    pub dead_code_pct: f64,

    /// Maximum file size in lines
    #[serde(default = "default_file_size")]
    pub max_file_lines: u32,

    /// Maximum function size in lines
    #[serde(default = "default_function_size")]
    pub max_function_lines: u32,

    /// Slow test threshold in seconds
    #[serde(default = "default_slow_test")]
    pub slow_test_seconds: f64,

    /// Slow coverage threshold in minutes
    #[serde(default = "default_slow_coverage")]
    pub slow_coverage_minutes: f64,

    /// Minimum TDG grade for CB-200 gate (A, B, C, D, F)
    #[serde(default = "default_min_tdg_grade")]
    pub min_tdg_grade: String,

    /// File path patterns to exclude from TDG grade gate (glob syntax)
    #[serde(default)]
    pub tdg_exclude_paths: Vec<String>,

    // --- Provable Contracts enforcement (L0.5 through L5) ---

    /// Treat pv lint failure as error (not warning). Default: false
    #[serde(default)]
    pub pv_lint_is_error: bool,

    /// Minimum binding existence percentage (0-100). Default: 80
    #[serde(default = "default_min_binding_existence")]
    pub min_binding_existence: f64,

    /// Require all 13 tier-1 contract traits for PASS. Default: false
    #[serde(default)]
    pub require_all_traits: bool,

    /// Minimum Kani coverage percentage (0-100) for CB-1206. Default: 0 (advisory)
    #[serde(default)]
    pub min_kani_coverage: f64,

    /// Minimum verification level: "L0", "L1", "L2", "L3", "L4", "L5". Default: "L0"
    #[serde(default = "default_min_verification_level")]
    pub min_verification_level: String,

    /// GH-292: Maximum transitive dependency count for CB-081.
    /// `None` uses the built-in default (250, with sovereign adjustment).
    #[serde(default)]
    pub max_transitive: Option<usize>,

    /// GH-292: File patterns excluded from File Health check (glob syntax).
    /// Patterns match either filename or path substring.
    #[serde(default)]
    pub file_health_exclude: Vec<String>,
}

fn default_min_binding_existence() -> f64 {
    80.0
}

fn default_min_verification_level() -> String {
    "L0".to_string()
}

/// Output format configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OutputConfig {
    /// Show only failures (hide passing checks)
    #[serde(default)]
    pub failures_only: bool,

    /// Use colors in output
    #[serde(default = "default_true")]
    pub colors: bool,

    /// Show verbose details
    #[serde(default)]
    pub verbose: bool,
}

/// Quality gate configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QualityConfig {
    /// Enable TDG scoring
    #[serde(default = "default_true")]
    pub tdg_enabled: bool,

    /// Minimum TDG score (0-100)
    #[serde(default = "default_tdg_score")]
    pub min_tdg_score: f64,

    /// Enable SATD detection
    #[serde(default = "default_true")]
    pub satd_enabled: bool,

    /// Block on new SATD markers
    #[serde(default)]
    pub block_on_new_satd: bool,
}

/// Work contract configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkConfig {
    /// Cache staleness warning threshold in hours
    #[serde(default = "default_cache_warn_hours")]
    pub cache_warn_hours: i64,

    /// Cache staleness block threshold in hours
    #[serde(default = "default_cache_block_hours")]
    pub cache_block_hours: i64,

    /// Skip quality gates on completion (not recommended)
    #[serde(default)]
    pub skip_quality_gates: bool,
}

/// Project-specific scoring plugins for domain scores (model accuracy, render quality, etc.)
///
/// Example .pmat.yaml:
/// ```yaml
/// scoring:
///   custom_scores:
///     - id: model-accuracy
///       name: "APR Model Accuracy"
///       command: "cargo test --test accuracy -- --nocapture 2>&1 | grep SCORE"
///       max_score: 100.0
///       min_score: 90.0
///       severity: error
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScoringPluginConfig {
    /// Custom score definitions
    #[serde(default)]
    pub custom_scores: Vec<CustomScoreDefinition>,
}

/// A single custom score definition for project-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomScoreDefinition {
    /// Unique identifier (e.g., "model-accuracy")
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Shell command that outputs JSON with {"score": N}
    pub command: String,
    /// Maximum possible score
    #[serde(default = "default_max_score")]
    pub max_score: f64,
    /// Minimum acceptable score (None = no minimum)
    #[serde(default)]
    pub min_score: Option<f64>,
    /// Severity when score is below minimum
    #[serde(default)]
    pub severity: CheckSeverity,
    /// Weight for composite scoring (default 1.0)
    #[serde(default = "default_weight")]
    pub weight: f64,
}

/// Configuration loading errors
#[derive(Debug, Clone)]
pub enum ConfigError {
    IoError(String),
    ParseError(String),
    SerializeError(String),
}
