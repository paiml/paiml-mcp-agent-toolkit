// dependency_checks_analysis.rs — included by dependency_checks.rs
// CB-081 violation detection, scoring, Cargo.toml analysis, trend tracking

/// GH-292: Read max_transitive override from `.pmat-gates.toml [dependency_health]`
/// or `.pmat.yaml comply.thresholds.max_transitive`. Gates file wins when both exist.
fn load_dependency_max_transitive() -> Option<usize> {
    if let Some(v) = load_dependency_max_transitive_from_gates() {
        return Some(v);
    }
    load_dependency_max_transitive_from_yaml()
}

fn load_dependency_max_transitive_from_gates() -> Option<usize> {
    let path = std::path::Path::new(".pmat-gates.toml");
    let content = std::fs::read_to_string(path).ok()?;
    let table: toml::Table = content.parse().ok()?;
    table
        .get("dependency_health")
        .and_then(|dh| dh.get("max_transitive"))
        .and_then(|v| v.as_integer())
        .map(|v| v as usize)
}

fn load_dependency_max_transitive_from_yaml() -> Option<usize> {
    let cwd = std::env::current_dir().ok()?;
    let cfg = crate::models::comply_config::PmatYamlConfig::load(&cwd).ok()?;
    cfg.comply.thresholds.max_transitive
}

/// Build a CB-081-A threshold violation if dependency counts exceed the given
/// thresholds.  Returns `None` when both `direct` and `transitive` are within
/// limits.  The description lists failing metrics first, with passing metrics
/// shown in parentheses as context.
fn build_threshold_violation(
    cargo_toml: &str,
    direct: usize,
    transitive: usize,
    direct_max: usize,
    trans_max: usize,
    severity: Severity,
) -> Option<CbPatternViolation> {
    if direct <= direct_max && transitive <= trans_max {
        return None;
    }

    let mut parts = Vec::new();
    let mut ok_parts = Vec::new();

    if direct > direct_max {
        parts.push(if matches!(severity, Severity::Error) {
            format!("{} direct deps exceed max {}", direct, direct_max)
        } else {
            format!("{} direct deps (threshold {})", direct, direct_max)
        });
    } else {
        ok_parts.push(format!("{} direct OK", direct));
    }

    if transitive > trans_max {
        parts.push(if matches!(severity, Severity::Error) {
            format!(
                "{} prod transitive deps exceed max {}",
                transitive, trans_max
            )
        } else {
            format!(
                "{} prod transitive deps (threshold {})",
                transitive, trans_max
            )
        });
    } else {
        ok_parts.push(format!("{} transitive OK", transitive));
    }

    let description = if ok_parts.is_empty() {
        parts.join(", ")
    } else {
        format!("{} ({})", parts.join(", "), ok_parts.join(", "))
    };

    Some(CbPatternViolation {
        pattern_id: "CB-081-A".to_string(),
        file: cargo_toml.to_string(),
        line: 0,
        description,
        severity,
    })
}

/// Generate all CB-081 violations (A through E) from pre-computed metrics.
#[allow(clippy::too_many_arguments)]
fn check_dependency_count_violations(
    cargo_toml: &str,
    cargo_lock: &str,
    direct: usize,
    effective_transitive: usize,
    feature_gated_pct: f64,
    duplicate_crates: &[DuplicateCrate],
    trend: &Option<DependencyTrend>,
    transitive_count: usize,
    sovereign_count: usize,
) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();

    // CB-081-A: Count thresholds -- sovereign stack adjustment
    // GH-292: Read max_transitive override from .pmat-gates.toml [dependency_health]
    let config_max = load_dependency_max_transitive();
    let sovereign_allowance = sovereign_count.min(5) * 50;
    let trans_error_max = config_max.unwrap_or(250) + sovereign_allowance;
    let trans_warn_max = config_max.map(|m| m.saturating_sub(50)).unwrap_or(200) + sovereign_allowance;

    if let Some(v) = build_threshold_violation(
        cargo_toml,
        direct,
        effective_transitive,
        50,
        trans_error_max,
        Severity::Error,
    ) {
        violations.push(v);
    } else if let Some(v) = build_threshold_violation(
        cargo_toml,
        direct,
        effective_transitive,
        40,
        trans_warn_max,
        Severity::Warning,
    ) {
        violations.push(v);
    }

    // CB-081-B: Duplicate crates
    check_duplicate_crates_violation(cargo_lock, duplicate_crates, &mut violations);

    // CB-081-C: Low feature gating (only warn if deps exceed excellent tier threshold)
    if direct > 20 && feature_gated_pct < 30.0 {
        violations.push(CbPatternViolation {
            pattern_id: "CB-081-C".to_string(),
            file: cargo_toml.to_string(),
            line: 0,
            description: format!(
                "Only {:.0}% deps use default-features=false. Consider disabling unused features",
                feature_gated_pct
            ),
            severity: Severity::Info,
        });
    }

    // CB-081-E: Trend regression
    check_trend_regression_violation(cargo_toml, trend, transitive_count, &mut violations);

    violations
}

/// CB-081-B: Generate a violation for duplicate crates if any exist.
fn check_duplicate_crates_violation(
    cargo_lock: &str,
    duplicate_crates: &[DuplicateCrate],
    violations: &mut Vec<CbPatternViolation>,
) {
    if !duplicate_crates.is_empty() {
        let dup_names: Vec<_> = duplicate_crates.iter().map(|d| d.name.as_str()).collect();
        violations.push(CbPatternViolation {
            pattern_id: "CB-081-B".to_string(),
            file: cargo_lock.to_string(),
            line: 0,
            description: format!(
                "{} duplicate crates: {}. Run 'cargo tree --duplicates'",
                duplicate_crates.len(),
                dup_names.join(", ")
            ),
            severity: Severity::Warning,
        });
    }
}

/// CB-081-E: Generate a violation if transitive dependency count increased >10%.
fn check_trend_regression_violation(
    cargo_toml: &str,
    trend: &Option<DependencyTrend>,
    transitive_count: usize,
    violations: &mut Vec<CbPatternViolation>,
) {
    if let Some(ref t) = trend {
        let pct_increase = if t.transitive_delta > 0 {
            (t.transitive_delta as f64 / (transitive_count as i32 - t.transitive_delta) as f64)
                * 100.0
        } else {
            0.0
        };
        if pct_increase > 10.0 {
            violations.push(CbPatternViolation {
                pattern_id: "CB-081-E".to_string(),
                file: cargo_toml.to_string(),
                line: 0,
                description: format!(
                    "Dependency creep: +{} transitive deps ({:.0}% increase) since {}",
                    t.transitive_delta, pct_increase, t.previous_timestamp
                ),
                severity: Severity::Warning,
            });
        }
    }
}

/// CB-081: Detect excessive dependency counts (enhanced)
/// Thresholds from rust-project-score-v1.1-update.md:
/// - 5 points: <=20 direct, <=100 transitive
/// - 4 points: <=30 direct, <=150 transitive
/// - 3 points: <=40 direct, <=200 transitive
/// - 2 points: <=50 direct, <=250 transitive
/// - 0 points: >50 direct or >250 transitive
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn detect_cb081_dependency_count(project_path: &Path) -> DependencyCountReport {
    let cargo_toml_path = project_path.join("Cargo.toml");
    let cargo_lock_path = project_path.join("Cargo.lock");

    // CB-081-A: Count direct dependencies from Cargo.toml
    let (direct_count, feature_gated_count, sovereign_crates) =
        analyze_cargo_toml(&cargo_toml_path);

    // CB-081-A & CB-081-B: Use O(1) cached analysis (issue #148 fix)
    let (transitive_count, prod_transitive_count, duplicate_crates) =
        get_cached_dependency_analysis(project_path, &cargo_lock_path);

    // Use production-only count for scoring (excludes dev-dep transitive)
    // Fall back to total Cargo.lock count if cargo tree is unavailable
    let effective_transitive = prod_transitive_count.unwrap_or(transitive_count);

    // CB-081-C: Calculate feature gating percentage
    let feature_gated_pct = if direct_count > 0 {
        (feature_gated_count as f64 / direct_count as f64) * 100.0
    } else {
        0.0
    };

    // CB-081-D: Calculate sovereign bonus (max +5, GH-294)
    let sovereign_bonus = std::cmp::min(sovereign_crates.len() as u8, 5);

    // CB-081-E: Load trend data
    let trend = load_dependency_trend(project_path);

    // Calculate base score using production-only transitive count (sovereign-adjusted)
    let mut score =
        calculate_dependency_score(direct_count, effective_transitive, sovereign_crates.len());

    // Apply bonuses (capped at 5 total)
    if feature_gated_pct >= 50.0 && score < 5 {
        score = std::cmp::min(score + 1, 5);
    }

    // Generate all violations
    let violations = check_dependency_count_violations(
        &cargo_toml_path.display().to_string(),
        &cargo_lock_path.display().to_string(),
        direct_count,
        effective_transitive,
        feature_gated_pct,
        &duplicate_crates,
        &trend,
        transitive_count,
        sovereign_crates.len(),
    );

    // Save current metrics for future trend tracking (use effective count)
    let _ = save_dependency_metrics(project_path, direct_count, effective_transitive);

    DependencyCountReport {
        direct_count,
        transitive_count,
        prod_transitive_count,
        score,
        duplicate_crates,
        feature_gated_count,
        feature_gated_pct,
        sovereign_crates,
        sovereign_bonus,
        trend,
        violations,
    }
}

/// Parse a TOML section header to determine which dependency section we're in.
/// Returns (in_dependencies, in_dev_dependencies, in_build_dependencies).
fn is_dependency_section(trimmed: &str) -> (bool, bool, bool) {
    let in_dependencies = trimmed == "[dependencies]"
        || trimmed.starts_with("[dependencies.")
        || trimmed.starts_with("[target.")
        || trimmed == "[workspace.dependencies]";
    let in_dev_dependencies =
        trimmed == "[dev-dependencies]" || trimmed.starts_with("[dev-dependencies.");
    let in_build_dependencies =
        trimmed == "[build-dependencies]" || trimmed.starts_with("[build-dependencies.");
    (in_dependencies, in_dev_dependencies, in_build_dependencies)
}

/// Return true when the line is a scoreable (non-dev, non-build) dependency.
/// The line must be inside a `[dependencies]` section (not `[dev-dependencies]`
/// or `[build-dependencies]`), contain `=`, and not be a comment.
fn is_scoreable_dependency(in_deps: bool, in_dev: bool, in_build: bool, trimmed: &str) -> bool {
    in_deps && !in_dev && !in_build && trimmed.contains('=') && !trimmed.starts_with('#')
}

/// Process a single dependency line to determine if it is a direct (non-optional)
/// dependency and whether it uses feature gating (`default-features = false`).
/// Also checks for sovereign crates, appending any found to `sovereign_found`.
/// Returns (is_direct, is_feature_gated).
fn process_dependency_line(trimmed: &str, sovereign_found: &mut Vec<String>) -> (bool, bool) {
    let is_optional = trimmed.contains("optional") && trimmed.contains("true");
    let is_direct = !is_optional;

    let is_feature_gated = trimmed.contains("default-features") && trimmed.contains("false");

    for crate_name in SOVEREIGN_CRATES {
        if trimmed.starts_with(crate_name)
            && (trimmed.chars().nth(crate_name.len()) == Some(' ')
                || trimmed.chars().nth(crate_name.len()) == Some('='))
        {
            sovereign_found.push(crate_name.to_string());
        }
    }

    (is_direct, is_feature_gated)
}

/// Analyze Cargo.toml for dependencies, feature gating, and sovereign crates
pub(super) fn analyze_cargo_toml(cargo_toml_path: &Path) -> (usize, usize, Vec<String>) {
    let content = match fs::read_to_string(cargo_toml_path) {
        Ok(c) => c,
        Err(_) => return (0, 0, Vec::new()),
    };

    let mut direct_count = 0;
    let mut feature_gated_count = 0;
    let mut sovereign_found = Vec::new();
    let mut in_dependencies = false;
    let mut in_dev_dependencies = false;
    let mut in_build_dependencies = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Track section headers
        if trimmed.starts_with('[') {
            (in_dependencies, in_dev_dependencies, in_build_dependencies) =
                is_dependency_section(trimmed);
            continue;
        }

        // Count dependencies (excluding dev, build, and optional deps for scoring)
        if is_scoreable_dependency(
            in_dependencies,
            in_dev_dependencies,
            in_build_dependencies,
            trimmed,
        ) {
            let (is_direct, is_feature_gated) =
                process_dependency_line(trimmed, &mut sovereign_found);
            if is_direct {
                direct_count += 1;
            }
            if is_feature_gated {
                feature_gated_count += 1;
            }
        }
    }

    // GH-294: Also scan workspace member Cargo.toml files for sovereign deps.
    // Workspace members declare deps like `aprender-simulate` or `aprender-contracts-macros`
    // that aren't in the root [workspace.dependencies] section.
    if let Some(project_dir) = cargo_toml_path.parent() {
        let crates_dir = project_dir.join("crates");
        if crates_dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&crates_dir) {
                for entry in entries.flatten() {
                    let member_toml = entry.path().join("Cargo.toml");
                    if member_toml.exists() {
                        if let Ok(member_content) = std::fs::read_to_string(&member_toml) {
                            for line in member_content.lines() {
                                let t = line.trim();
                                for crate_name in SOVEREIGN_CRATES {
                                    if t.starts_with(crate_name)
                                        && (t.chars().nth(crate_name.len()) == Some(' ')
                                            || t.chars().nth(crate_name.len()) == Some('=')
                                            || t.chars().nth(crate_name.len()) == Some('.'))
                                        && !sovereign_found.contains(&crate_name.to_string())
                                    {
                                        sovereign_found.push(crate_name.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    (direct_count, feature_gated_count, sovereign_found)
}

/// Calculate dependency health score (0-5 points)
/// Sovereign stack projects get adjusted thresholds (each sovereign crate brings
/// its own ecosystem, e.g. trueno-graph -> arrow/wgpu, trueno-rag -> vector search).
pub(super) fn calculate_dependency_score(
    direct: usize,
    transitive: usize,
    sovereign_count: usize,
) -> u8 {
    let bonus = sovereign_count.min(5) * 50;
    // GH-292: Scale thresholds if max_transitive override is set
    let config_max = load_dependency_max_transitive().unwrap_or(250);
    let scale = config_max as f64 / 250.0;
    let t5 = (100.0 * scale) as usize + bonus;
    let t4 = (150.0 * scale) as usize + bonus;
    let t3 = (200.0 * scale) as usize + bonus;
    let t2 = (250.0 * scale) as usize + bonus;
    if direct <= 20 && transitive <= t5 {
        5
    } else if direct <= 30 && transitive <= t4 {
        4
    } else if direct <= 40 && transitive <= t3 {
        3
    } else if direct <= 50 && transitive <= t2 {
        2
    } else {
        0
    }
}

/// CB-081-E: Load previous dependency metrics for trend tracking
pub(super) fn load_dependency_trend(project_path: &Path) -> Option<DependencyTrend> {
    let metrics_path = project_path
        .join(".pmat")
        .join("metrics")
        .join("dependencies.json");

    let content = fs::read_to_string(&metrics_path).ok()?;

    #[derive(serde::Deserialize)]
     // Fields used for JSON deserialization structure matching
    struct PreviousMetrics {
        direct_count: usize,
        transitive_count: usize,
        timestamp: String,
    }

    let prev: PreviousMetrics = serde_json::from_str(&content).ok()?;

    // Return trend with previous timestamp - deltas calculated elsewhere
    Some(DependencyTrend {
        direct_delta: 0,
        transitive_delta: 0,
        previous_timestamp: prev.timestamp,
    })
}

/// CB-081-E: Save current dependency metrics for future trend tracking
pub(super) fn save_dependency_metrics(
    project_path: &Path,
    direct: usize,
    transitive: usize,
) -> std::io::Result<()> {
    let metrics_dir = project_path.join(".pmat").join("metrics");
    fs::create_dir_all(&metrics_dir)?;

    let metrics_path = metrics_dir.join("dependencies.json");

    // Load previous metrics to calculate deltas
    let previous = if metrics_path.exists() {
        fs::read_to_string(&metrics_path)
            .ok()
            .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok())
    } else {
        None
    };

    let timestamp = chrono::Utc::now().to_rfc3339();

    let metrics = serde_json::json!({
        "direct_count": direct,
        "transitive_count": transitive,
        "timestamp": timestamp,
        "previous": previous,
    });

    fs::write(&metrics_path, serde_json::to_string_pretty(&metrics)?)
}

/// Recalculate trend deltas with current counts
 // Reserved for future trend comparison feature
pub(super) fn calculate_trend_deltas(
    project_path: &Path,
    current_direct: usize,
    current_transitive: usize,
) -> Option<DependencyTrend> {
    let metrics_path = project_path
        .join(".pmat")
        .join("metrics")
        .join("dependencies.json");

    let content = fs::read_to_string(&metrics_path).ok()?;
    let prev: serde_json::Value = serde_json::from_str(&content).ok()?;

    let prev_direct = prev.get("previous")?.get("direct_count")?.as_u64()? as usize;
    let prev_transitive = prev.get("previous")?.get("transitive_count")?.as_u64()? as usize;
    let prev_timestamp = prev.get("previous")?.get("timestamp")?.as_str()?;

    Some(DependencyTrend {
        direct_delta: current_direct as i32 - prev_direct as i32,
        transitive_delta: current_transitive as i32 - prev_transitive as i32,
        previous_timestamp: prev_timestamp.to_string(),
    })
}
