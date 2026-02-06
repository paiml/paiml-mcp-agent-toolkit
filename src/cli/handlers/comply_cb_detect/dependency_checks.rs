use super::types::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

// =============================================================================
// CB-081: Dependency Count Detection (Enhanced v2.9)
// Per rust-project-score spec: Too many dependencies degrades build times,
// increases supply chain risk, and bloats binaries.
//
// Enhancements:
// - CB-081-A: Base dependency count scoring
// - CB-081-B: Duplicate crate detection
// - CB-081-C: Feature flag hygiene analysis
// - CB-081-D: Sovereign stack bonus
// - CB-081-E: Trend tracking
// =============================================================================

/// Sovereign stack crates (batuta ecosystem)
const SOVEREIGN_CRATES: &[&str] = &[
    "aprender", "trueno", "trueno-graph", "trueno-db", "trueno-rag",
    "trueno-viz", "trueno-zram-core", "pmcp", "presentar-core",
    "renacer", "certeza", "bashrs", "probar", "ruchy",
];

/// Dependency count analysis result (enhanced)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DependencyCountReport {
    pub direct_count: usize,
    /// Total packages in Cargo.lock (includes dev-dep transitive)
    pub transitive_count: usize,
    /// Production-only transitive count (excludes dev-dep transitive), used for scoring
    pub prod_transitive_count: Option<usize>,
    pub score: u8,  // 0-5 points based on rust-project-score thresholds
    /// Crates with multiple versions in Cargo.lock
    pub duplicate_crates: Vec<DuplicateCrate>,
    /// Dependencies using default-features = false
    pub feature_gated_count: usize,
    pub feature_gated_pct: f64,
    /// Sovereign stack crates used
    pub sovereign_crates: Vec<String>,
    pub sovereign_bonus: u8,  // 0-3 bonus points
    /// Delta from previous check (if available)
    pub trend: Option<DependencyTrend>,
    pub violations: Vec<CbPatternViolation>,
}

/// Duplicate crate info
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DuplicateCrate {
    pub name: String,
    pub versions: Vec<String>,
}

/// Trend tracking data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DependencyTrend {
    pub direct_delta: i32,
    pub transitive_delta: i32,
    pub previous_timestamp: String,
}

/// CB-081 Cache for O(1) dependency analysis (issue #148 fix)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(super) struct DependencyCache {
    /// Cargo.lock file mtime (unix timestamp)
    cargo_lock_mtime: u64,
    /// Cached transitive count (all packages from Cargo.lock)
    transitive_count: usize,
    /// Cached production-only transitive count (via cargo tree --no-dev)
    #[serde(default)]
    prod_transitive_count: Option<usize>,
    /// Cached duplicate crates
    duplicate_crates: Vec<DuplicateCrate>,
}

impl DependencyCache {
    /// Check if cache is valid (Cargo.lock unchanged)
    fn is_valid(&self, cargo_lock_path: &Path) -> bool {
        if let Ok(metadata) = fs::metadata(cargo_lock_path) {
            if let Ok(modified) = metadata.modified() {
                let mtime = modified
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                return mtime == self.cargo_lock_mtime;
            }
        }
        false
    }

    /// Load cache from .pmat/deps-cache.json
    fn load(project_path: &Path) -> Option<Self> {
        let cache_path = project_path.join(".pmat/deps-cache.json");
        fs::read_to_string(&cache_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
    }

    /// Save cache to .pmat/deps-cache.json
    fn save(&self, project_path: &Path) {
        let cache_path = project_path.join(".pmat/deps-cache.json");
        if let Some(parent) = cache_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(&cache_path, json);
        }
    }
}

/// Parse Cargo.lock once and return both transitive count and duplicates
pub(super) fn parse_cargo_lock(cargo_lock_path: &Path) -> (usize, Vec<DuplicateCrate>) {
    let content = match fs::read_to_string(cargo_lock_path) {
        Ok(c) => c,
        Err(_) => return (0, Vec::new()),
    };

    let mut crate_versions: HashMap<String, Vec<String>> =
        HashMap::new();
    let mut current_name: Option<String> = None;
    let mut current_version: Option<String> = None;
    let mut package_count = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "[[package]]" {
            package_count += 1;
            // Save previous package if complete
            if let (Some(name), Some(version)) = (current_name.take(), current_version.take()) {
                crate_versions.entry(name).or_default().push(version);
            }
        } else if let Some(name) = trimmed.strip_prefix("name = \"") {
            current_name = name.strip_suffix('"').map(|s| s.to_string());
        } else if let Some(version) = trimmed.strip_prefix("version = \"") {
            current_version = version.strip_suffix('"').map(|s| s.to_string());
        }
    }

    // Don't forget the last package
    if let (Some(name), Some(version)) = (current_name, current_version) {
        crate_versions.entry(name).or_default().push(version);
    }

    // Filter to only duplicates (>1 version)
    let duplicates: Vec<DuplicateCrate> = crate_versions
        .into_iter()
        .filter(|(_, versions)| versions.len() > 1)
        .map(|(name, mut versions)| {
            versions.sort();
            versions.dedup();
            DuplicateCrate { name, versions }
        })
        .filter(|d| d.versions.len() > 1)
        .collect();

    (package_count, duplicates)
}

/// Count production-only transitive dependencies using `cargo tree -e no-dev`
/// Returns None if cargo tree is unavailable or fails
pub(super) fn count_production_transitive(project_path: &Path) -> Option<usize> {
    let output = std::process::Command::new("cargo")
        .args(["tree", "-e", "no-dev", "--prefix=none"])
        .current_dir(project_path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut unique_packages: HashSet<String> =
        HashSet::new();
    for line in stdout.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            // cargo tree --prefix=none outputs "crate_name v1.2.3"
            if let Some(name) = trimmed.split_whitespace().next() {
                unique_packages.insert(name.to_string());
            }
        }
    }

    Some(unique_packages.len())
}

/// Get dependency analysis with O(1) caching (issue #148 fix)
pub(super) fn get_cached_dependency_analysis(
    project_path: &Path,
    cargo_lock_path: &Path,
) -> (usize, Option<usize>, Vec<DuplicateCrate>) {
    // Try to use cached results first
    if let Some(cache) = DependencyCache::load(project_path) {
        if cache.is_valid(cargo_lock_path) {
            return (cache.transitive_count, cache.prod_transitive_count, cache.duplicate_crates);
        }
    }

    // Cache miss or invalid - parse Cargo.lock
    let (transitive_count, duplicate_crates) = parse_cargo_lock(cargo_lock_path);

    // Get production-only count via cargo tree
    let prod_transitive_count = count_production_transitive(project_path);

    // Save to cache
    let mtime = fs::metadata(cargo_lock_path)
        .and_then(|m| m.modified())
        .map(|t| t.duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0))
        .unwrap_or(0);

    let cache = DependencyCache {
        cargo_lock_mtime: mtime,
        transitive_count,
        prod_transitive_count,
        duplicate_crates: duplicate_crates.clone(),
    };
    cache.save(project_path);

    (transitive_count, prod_transitive_count, duplicate_crates)
}

/// CB-081: Detect excessive dependency counts (enhanced)
/// Thresholds from rust-project-score-v1.1-update.md:
/// - 5 points: ≤20 direct, ≤100 transitive
/// - 4 points: ≤30 direct, ≤150 transitive
/// - 3 points: ≤40 direct, ≤200 transitive
/// - 2 points: ≤50 direct, ≤250 transitive
/// - 0 points: >50 direct or >250 transitive
pub fn detect_cb081_dependency_count(project_path: &Path) -> DependencyCountReport {
    let cargo_toml_path = project_path.join("Cargo.toml");
    let cargo_lock_path = project_path.join("Cargo.lock");

    let mut violations = Vec::new();

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

    // CB-081-D: Calculate sovereign bonus (max +3)
    let sovereign_bonus = std::cmp::min(sovereign_crates.len() as u8, 3);

    // CB-081-E: Load trend data
    let trend = load_dependency_trend(project_path);

    // Calculate base score using production-only transitive count
    let mut score = calculate_dependency_score(direct_count, effective_transitive);

    // Apply bonuses (capped at 5 total)
    if feature_gated_pct >= 50.0 && score < 5 {
        score = std::cmp::min(score + 1, 5);
    }

    // Generate violations based on severity (using production-only transitive count)
    // CB-081-A: Count thresholds
    if direct_count > 50 || effective_transitive > 250 {
        violations.push(CbPatternViolation {
            pattern_id: "CB-081-A".to_string(),
            file: cargo_toml_path.display().to_string(),
            line: 0,
            description: format!(
                "Critical: {} direct deps (max 50), {} prod transitive deps (max 250)",
                direct_count, effective_transitive
            ),
            severity: Severity::Error,
        });
    } else if direct_count > 40 || effective_transitive > 200 {
        violations.push(CbPatternViolation {
            pattern_id: "CB-081-A".to_string(),
            file: cargo_toml_path.display().to_string(),
            line: 0,
            description: format!(
                "High: {} direct (threshold 40), {} prod transitive (threshold 200)",
                direct_count, effective_transitive
            ),
            severity: Severity::Warning,
        });
    }

    // CB-081-B: Duplicate crates
    if !duplicate_crates.is_empty() {
        let dup_names: Vec<_> = duplicate_crates.iter().map(|d| d.name.as_str()).collect();
        violations.push(CbPatternViolation {
            pattern_id: "CB-081-B".to_string(),
            file: cargo_lock_path.display().to_string(),
            line: 0,
            description: format!(
                "{} duplicate crates: {}. Run 'cargo tree --duplicates'",
                duplicate_crates.len(),
                dup_names.join(", ")
            ),
            severity: Severity::Warning,
        });
    }

    // CB-081-C: Low feature gating (only warn if deps exceed excellent tier threshold)
    if direct_count > 20 && feature_gated_pct < 30.0 {
        violations.push(CbPatternViolation {
            pattern_id: "CB-081-C".to_string(),
            file: cargo_toml_path.display().to_string(),
            line: 0,
            description: format!(
                "Only {:.0}% deps use default-features=false. Consider disabling unused features",
                feature_gated_pct
            ),
            severity: Severity::Info,
        });
    }

    // CB-081-E: Trend regression
    if let Some(ref t) = trend {
        let pct_increase = if t.transitive_delta > 0 {
            (t.transitive_delta as f64 / (transitive_count as i32 - t.transitive_delta) as f64) * 100.0
        } else {
            0.0
        };
        if pct_increase > 10.0 {
            violations.push(CbPatternViolation {
                pattern_id: "CB-081-E".to_string(),
                file: cargo_toml_path.display().to_string(),
                line: 0,
                description: format!(
                    "Dependency creep: +{} transitive deps ({:.0}% increase) since {}",
                    t.transitive_delta, pct_increase, t.previous_timestamp
                ),
                severity: Severity::Warning,
            });
        }
    }

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
            in_dependencies = trimmed == "[dependencies]"
                || trimmed.starts_with("[dependencies.")
                || trimmed.starts_with("[target.");
            in_dev_dependencies = trimmed == "[dev-dependencies]"
                || trimmed.starts_with("[dev-dependencies.");
            in_build_dependencies = trimmed == "[build-dependencies]"
                || trimmed.starts_with("[build-dependencies.");
            continue;
        }

        // Count dependencies (excluding dev, build, and optional deps for scoring)
        if in_dependencies && !in_dev_dependencies && !in_build_dependencies
            && trimmed.contains('=') && !trimmed.starts_with('#')
        {
            // Skip optional dependencies - they don't count toward direct count
            let is_optional = trimmed.contains("optional") && trimmed.contains("true");
            if !is_optional {
                direct_count += 1;
            }

            // Check for default-features = false
            if trimmed.contains("default-features") && trimmed.contains("false") {
                feature_gated_count += 1;
            }

            // Check for sovereign crates
            for crate_name in SOVEREIGN_CRATES {
                if trimmed.starts_with(crate_name)
                    && (trimmed.chars().nth(crate_name.len()) == Some(' ')
                        || trimmed.chars().nth(crate_name.len()) == Some('='))
                {
                    sovereign_found.push(crate_name.to_string());
                }
            }
        }
    }

    (direct_count, feature_gated_count, sovereign_found)
}

/// Calculate dependency health score (0-5 points)
pub(super) fn calculate_dependency_score(direct: usize, transitive: usize) -> u8 {
    if direct <= 20 && transitive <= 100 {
        5
    } else if direct <= 30 && transitive <= 150 {
        4
    } else if direct <= 40 && transitive <= 200 {
        3
    } else if direct <= 50 && transitive <= 250 {
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
    #[allow(dead_code)] // Fields used for JSON deserialization structure matching
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
pub(super) fn save_dependency_metrics(project_path: &Path, direct: usize, transitive: usize) -> std::io::Result<()> {
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
#[allow(dead_code)] // Reserved for future trend comparison feature
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

// =============================================================================
// CB-130: Agent Context Adoption (PMAT-470)
// =============================================================================

/// CB-130 Agent Context Adoption report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContextReport {
    /// Whether the RAG index exists
    pub index_exists: bool,
    /// Index age in hours (None if no index)
    pub index_age_hours: Option<f64>,
    /// Whether index is considered stale (>24h)
    pub index_stale: bool,
    /// Number of functions indexed
    pub function_count: usize,
    /// Whether CLAUDE.md mentions pmat_query_code
    pub claude_md_configured: bool,
    /// Required patterns missing from CLAUDE.md
    pub missing_required_patterns: Vec<String>,
    /// Forbidden patterns found in CLAUDE.md
    pub forbidden_patterns_found: Vec<ForbiddenPatternMatch>,
}

/// A forbidden pattern match with location info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForbiddenPatternMatch {
    /// The pattern that was found
    pub pattern: String,
    /// Line number where it was found
    pub line: usize,
    /// The actual line content (truncated)
    pub context: String,
}

/// Required patterns that should be in CLAUDE.md for agent context adoption
const REQUIRED_PATTERNS: &[&str] = &[
    "pmat query",
    "NEVER use grep",
    "--faults",
];

/// Forbidden patterns that indicate agents might use grep instead of pmat query
const FORBIDDEN_PATTERNS: &[&str] = &[
    "grep -r",
    "grep -rn",
    "find . -name",
    "find . -type f",
    "rg \"",
    "rg '",
];

/// CB-130: Detect agent context adoption issues
///
/// Checks:
/// 1. RAG index exists at .pmat/context.idx
/// 2. Index is fresh (less than 24 hours old)
/// 3. CLAUDE.md references pmat_query_code (optional)
pub fn detect_cb130_agent_context_adoption(project_path: &Path) -> AgentContextReport {
    let index_path = project_path.join(".pmat/context.idx");

    let index_exists = index_path.exists();

    let (index_age_hours, index_stale) = if index_exists {
        match fs::metadata(&index_path) {
            Ok(metadata) => {
                if let Ok(modified) = metadata.modified() {
                    let age = std::time::SystemTime::now()
                        .duration_since(modified)
                        .unwrap_or_default();
                    let hours = age.as_secs_f64() / 3600.0;
                    (Some(hours), hours > 24.0)
                } else {
                    (None, false)
                }
            }
            Err(_) => (None, false),
        }
    } else {
        (None, false)
    };

    // Try to get function count from index
    let function_count = if index_exists {
        // Read index to get function count
        match crate::services::agent_context::AgentContextIndex::load(&index_path) {
            Ok(idx) => idx.manifest().function_count,
            Err(_) => 0,
        }
    } else {
        0
    };

    // Check CLAUDE.md for required and forbidden patterns
    let claude_md_path = project_path.join("CLAUDE.md");
    let (claude_md_configured, missing_required_patterns, forbidden_patterns_found) =
        if claude_md_path.exists() {
            match fs::read_to_string(&claude_md_path) {
                Ok(content) => {
                    let configured =
                        content.contains("pmat_query_code") || content.contains("pmat query");

                    // Check for missing required patterns
                    let missing: Vec<String> = REQUIRED_PATTERNS
                        .iter()
                        .filter(|&p| !content.to_lowercase().contains(&p.to_lowercase()))
                        .map(|s| s.to_string())
                        .collect();

                    // Check for forbidden patterns with line numbers
                    let mut forbidden: Vec<ForbiddenPatternMatch> = Vec::new();
                    for (line_num, line) in content.lines().enumerate() {
                        for &pattern in FORBIDDEN_PATTERNS {
                            if line.contains(pattern) {
                                // Skip if it's in a "DON'T DO THIS" example context
                                let lower_line = line.to_lowercase();
                                let is_negative_example = lower_line.contains("bad")
                                    || lower_line.contains("don't")
                                    || lower_line.contains("never")
                                    || lower_line.contains("avoid");
                                if !is_negative_example {
                                    forbidden.push(ForbiddenPatternMatch {
                                        pattern: pattern.to_string(),
                                        line: line_num + 1,
                                        context: line.chars().take(80).collect(),
                                    });
                                }
                            }
                        }
                    }

                    (configured, missing, forbidden)
                }
                Err(_) => (false, REQUIRED_PATTERNS.iter().map(|s| s.to_string()).collect(), vec![]),
            }
        } else {
            (false, REQUIRED_PATTERNS.iter().map(|s| s.to_string()).collect(), vec![])
        };

    AgentContextReport {
        index_exists,
        index_age_hours,
        index_stale,
        function_count,
        claude_md_configured,
        missing_required_patterns,
        forbidden_patterns_found,
    }
}
