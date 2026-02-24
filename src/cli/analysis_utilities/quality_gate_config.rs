/// Default provability threshold when no config is found
const DEFAULT_PROVABILITY_THRESHOLD: f64 = 0.70;

/// Load the provability threshold from `.pmat-metrics.toml`.
///
/// Looks for `provability_min` under the `[thresholds]` section.
/// Falls back to `DEFAULT_PROVABILITY_THRESHOLD` (0.70) if the file
/// is missing, unreadable, or does not contain the key.
fn load_provability_threshold(project_path: &Path) -> f64 {
    let config_path = project_path.join(".pmat-metrics.toml");
    let content = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => return DEFAULT_PROVABILITY_THRESHOLD,
    };
    let table: toml::Table = match content.parse() {
        Ok(t) => t,
        Err(_) => return DEFAULT_PROVABILITY_THRESHOLD,
    };
    table
        .get("thresholds")
        .and_then(|t| t.get("provability_min"))
        .and_then(|v| v.as_float())
        .unwrap_or(DEFAULT_PROVABILITY_THRESHOLD)
}

/// Load entropy min_pattern_diversity from config files (#194, #219, #227).
///
/// Priority: `.pmat-gates.toml` > `.pmat-metrics.toml` > `pmat.toml` > CLI default.
/// Reads from `[entropy] min_pattern_diversity`, `[thresholds] entropy_min_diversity`,
/// or `[quality] min_pattern_diversity`.
/// Clamps result to 0.0-1.0 range to prevent unreachable thresholds.
fn load_entropy_threshold(project_path: &Path, cli_value: f64) -> f64 {
    let mut result = cli_value;

    // Load from pmat.toml [quality] (lowest config priority, #227)
    if let Some(val) = read_entropy_threshold_from_pmat_toml(project_path) {
        result = val;
    }

    // Load from .pmat-metrics.toml (medium priority)
    if let Some(val) = read_entropy_threshold_from_file(
        &project_path.join(".pmat-metrics.toml"),
    ) {
        result = val;
    }

    // Load from .pmat-gates.toml (highest priority, #219)
    if let Some(val) = read_entropy_threshold_from_file(
        &project_path.join(".pmat-gates.toml"),
    ) {
        result = val;
    }

    // Clamp to valid range (#219: prevent 200% unreachable thresholds)
    result.clamp(0.0, 1.0)
}

/// Read entropy threshold from `pmat.toml [quality] min_pattern_diversity` (#227).
fn read_entropy_threshold_from_pmat_toml(project_path: &Path) -> Option<f64> {
    let content = std::fs::read_to_string(project_path.join("pmat.toml")).ok()?;
    let table: toml::Table = content.parse().ok()?;
    table
        .get("quality")
        .and_then(|t| t.get("min_pattern_diversity"))
        .and_then(|v| v.as_float())
}

/// Read entropy threshold from a single TOML file.
/// Checks `[entropy] min_pattern_diversity` and `[thresholds] entropy_min_diversity`.
fn read_entropy_threshold_from_file(path: &Path) -> Option<f64> {
    let content = std::fs::read_to_string(path).ok()?;
    let table: toml::Table = content.parse().ok()?;

    // Check [entropy] min_pattern_diversity first (preferred key)
    if let Some(val) = table
        .get("entropy")
        .and_then(|t| t.get("min_pattern_diversity"))
        .and_then(|v| v.as_float())
    {
        return Some(val);
    }

    // Fallback: [thresholds] entropy_min_diversity (legacy key)
    table
        .get("thresholds")
        .and_then(|t| t.get("entropy_min_diversity"))
        .and_then(|v| v.as_float())
}

/// Entropy gate configuration loaded from `.pmat-gates.toml` (#220).
struct EntropyGateConfig {
    enabled: bool,
    max_violations: Option<usize>,
    exclude: Vec<String>,
}

/// Load entropy gate configuration from `.pmat-gates.toml`, with `pmat.toml` fallback (#220, #227).
///
/// Priority: `.pmat-gates.toml [entropy]` > `pmat.toml [quality]` > defaults.
/// Reads `enabled`, `max_violations`, `exclude` from `[entropy]` section.
fn load_entropy_gate_config(project_path: &Path) -> EntropyGateConfig {
    // Start with pmat.toml [quality] max_entropy_violations as lowest priority (#227)
    let mut max_violations_fallback: Option<usize> = None;
    if let Ok(content) = std::fs::read_to_string(project_path.join("pmat.toml")) {
        if let Ok(table) = content.parse::<toml::Table>() {
            max_violations_fallback = table
                .get("quality")
                .and_then(|t| t.get("max_entropy_violations"))
                .and_then(|v| v.as_integer())
                .map(|v| v.max(0) as usize);
        }
    }

    let path = project_path.join(".pmat-gates.toml");
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => {
            return EntropyGateConfig {
                enabled: true,
                max_violations: max_violations_fallback,
                exclude: Vec::new(),
            }
        }
    };
    let table: toml::Table = match content.parse() {
        Ok(t) => t,
        Err(_) => {
            return EntropyGateConfig {
                enabled: true,
                max_violations: max_violations_fallback,
                exclude: Vec::new(),
            }
        }
    };

    let entropy = table.get("entropy");

    let enabled = entropy
        .and_then(|t| t.get("enabled"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    // .pmat-gates.toml overrides pmat.toml if present
    let max_violations = entropy
        .and_then(|t| t.get("max_violations"))
        .and_then(|v| v.as_integer())
        .map(|v| v.max(0) as usize)
        .or(max_violations_fallback);

    let exclude = entropy
        .and_then(|t| t.get("exclude"))
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    EntropyGateConfig {
        enabled,
        max_violations,
        exclude,
    }
}

/// Extract exclude paths from a parsed TOML table.
///
/// Checks multiple patterns:
/// - `[exclude] paths = [...]`
/// - `exclude_paths = [...]`
/// - `[quality-gates] exclude = [...]`
fn extract_excludes_from_table(table: &toml::Table) -> Vec<String> {
    let arr = table
        .get("exclude")
        .and_then(|t| t.get("paths"))
        .and_then(|v| v.as_array())
        .or_else(|| table.get("exclude_paths").and_then(|v| v.as_array()))
        .or_else(|| {
            table
                .get("quality-gates")
                .and_then(|t| t.get("exclude"))
                .and_then(|v| v.as_array())
        });
    arr.map(|a| {
        a.iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect()
    })
    .unwrap_or_default()
}

/// Load exclude paths from `.pmat-metrics.toml` and `.pmat-gates.toml` (#195, #217).
///
/// Checks both config files and merges exclude patterns.
/// Returns an empty vec if neither file exists or no exclude config exists.
fn load_entropy_exclude_paths(project_path: &Path) -> Vec<String> {
    let mut excludes = Vec::new();

    // Load from .pmat-metrics.toml
    if let Ok(content) = std::fs::read_to_string(project_path.join(".pmat-metrics.toml")) {
        if let Ok(table) = content.parse::<toml::Table>() {
            excludes.extend(extract_excludes_from_table(&table));
        }
    }

    // Load from .pmat-gates.toml (#217)
    if let Ok(content) = std::fs::read_to_string(project_path.join(".pmat-gates.toml")) {
        if let Ok(table) = content.parse::<toml::Table>() {
            for pattern in extract_excludes_from_table(&table) {
                if !excludes.contains(&pattern) {
                    excludes.push(pattern);
                }
            }
        }
    }

    excludes
}

/// Filter violations whose file path matches any exclude path (#196).
///
/// Matches both exact prefix and glob patterns. Violations with `file = "project"`
/// or other non-path values are kept (project-level metrics).
fn filter_violations_by_exclude(violations: &mut Vec<QualityViolation>, exclude_paths: &[String]) {
    violations.retain(|v| {
        // Keep project-level violations (no file path)
        if v.file == "project" || v.file.is_empty() {
            return true;
        }
        // Check if the violation's file matches any exclude path
        !exclude_paths.iter().any(|excl| {
            let normalized = excl.trim_end_matches('/');
            v.file.starts_with(normalized)
                || v.file.starts_with(&format!("{normalized}/"))
                || v.file.starts_with(&format!("./{normalized}"))
                || glob::Pattern::new(excl).is_ok_and(|p| p.matches(&v.file))
        })
    });
}

