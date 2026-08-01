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

/// Default entropy threshold when neither the CLI nor any config supplies one.
/// Matches `EntropyConfig::default().min_pattern_diversity`.
pub(crate) const DEFAULT_MIN_PATTERN_DIVERSITY: f64 = 0.3;

/// Resolve the pattern-diversity threshold the entropy check will apply.
///
/// #683: an explicit `--min-entropy` now wins outright. It used to be the *lowest*
/// priority input, so `--min-entropy 0.0` on a repo whose `pmat.toml` carries
/// `min_pattern_diversity = 0.30` silently ran at 0.30 — the user's request had
/// no effect on the result at all. An explicit value is also exempt from the
/// small-repo scaling below: scaling a number the user typed changes what they
/// asked for.
///
/// With no explicit value the order is `.pmat-gates.toml`, then
/// `.pmat-metrics.toml`, then `pmat.toml`, then `DEFAULT_MIN_PATTERN_DIVERSITY`;
/// the result is clamped to 0.0-1.0 (#219) and scaled down for small repos
/// (#248), which naturally show lower pattern diversity.
fn load_entropy_threshold(project_path: &Path, cli_value: Option<f64>) -> f64 {
    if let Some(explicit) = cli_value {
        return explicit.clamp(0.0, 1.0);
    }

    let mut result = DEFAULT_MIN_PATTERN_DIVERSITY;

    // Load from pmat.toml [quality] (lowest config priority, #227)
    if let Some(val) = read_entropy_threshold_from_pmat_toml(project_path) {
        result = val;
        source = "pmat.toml [quality] min_pattern_diversity";
    }

    // Load from .pmat-metrics.toml (medium priority)
    if let Some(val) = read_entropy_threshold_from_file(&project_path.join(".pmat-metrics.toml")) {
        result = val;
        source = ".pmat-metrics.toml entropy threshold";
    }

    // Load from .pmat-gates.toml (highest priority, #219)
    if let Some(val) = read_entropy_threshold_from_file(&project_path.join(".pmat-gates.toml")) {
        result = val;
        source = ".pmat-gates.toml [entropy] min_pattern_diversity";
    }

    if !(0.0..=1.0).contains(&result) {
        anyhow::bail!(
            "{source} = {result} is out of range: pattern diversity is a normalised \
             fraction, so the minimum must be between 0.0 and 1.0 (0.3 = 30% diversity)"
        );
    }

    // #248: Scale threshold for small repos to reduce false positives.
    // Small repos (<50 files) naturally have lower pattern diversity.
    let file_count = count_source_files(project_path);
    let effective = result * size_scale(file_count);
    if (effective - result).abs() > f64::EPSILON {
        eprintln!(
            "  \u{2139}\u{fe0f}  entropy minimum {result:.2} scaled to {effective:.2} \
             for a {file_count}-source-file project (#248)"
        );
    }
    Ok(effective)
}

/// Scale entropy threshold based on project size (#248).
///
/// Small repos (<50 source files) get a proportionally lower threshold:
/// - <10 files: threshold * 0.5
/// - <25 files: threshold * 0.7
/// - <50 files: threshold * 0.85
/// - >=50 files: no scaling (full threshold)
fn scale_entropy_for_project_size(project_path: &Path, threshold: f64) -> f64 {
    threshold * size_scale(count_source_files(project_path))
}

/// Scale factor for a project of `file_count` source files (#248).
/// Extracted so `load_entropy_threshold` can report the count it scaled by
/// without walking the tree twice.
fn size_scale(file_count: usize) -> f64 {
    if file_count < 10 {
        0.5
    } else if file_count < 25 {
        0.7
    } else if file_count < 50 {
        0.85
    } else {
        1.0
    }
}

/// Count source files in the project (quick heuristic, not a full walk).
/// Only counts files in common source directories with code extensions.
fn count_source_files(project_path: &Path) -> usize {
    let source_dirs = ["src", "lib", "app", "pkg", "crates"];
    let extensions = ["rs", "py", "js", "ts", "go", "java", "c", "cpp", "rb"];

    let mut count = 0usize;
    for dir_name in &source_dirs {
        let dir = project_path.join(dir_name);
        if dir.is_dir() {
            count += count_files_recursive(&dir, &extensions, 0);
        }
    }
    // If no standard source dirs found, count from project root (shallow)
    if count == 0 {
        count = count_files_recursive(project_path, &extensions, 0);
    }
    count
}

/// Check if a file has one of the given extensions.
fn has_matching_extension(path: &Path, extensions: &[&str]) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| extensions.contains(&ext))
}

/// Check if a directory should be traversed (skip hidden, target, node_modules).
fn is_traversable_dir(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|name| !name.starts_with('.') && name != "target" && name != "node_modules")
}

/// Recursively count files with given extensions (max depth 10).
fn count_files_recursive(dir: &Path, extensions: &[&str], depth: usize) -> usize {
    if depth > 10 {
        return 0;
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return 0,
    };
    let mut count = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && has_matching_extension(&path, extensions) {
            count += 1;
        } else if path.is_dir() && is_traversable_dir(&path) {
            count += count_files_recursive(&path, extensions, depth + 1);
        }
    }
    count
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

