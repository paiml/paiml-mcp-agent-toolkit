#![cfg_attr(coverage_nightly, coverage(off))]

use crate::cli::commands::ComplyOutputFormat;
use crate::models::comply_config::{CrossCrateConfig, PmatYamlConfig};
use crate::services::agent_context::{parse_workspace_siblings, AgentContextIndex, FunctionEntry};
use crate::services::duplicate_detector::{
    DuplicateDetectionConfig, Language, MinHashGenerator, MinHashSignature,
    UniversalFeatureExtractor,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

// --- Types ---

#[derive(Debug, Clone, Serialize)]
pub struct CrateInfo {
    pub name: String,
    pub path: PathBuf,
    pub cargo_deps: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum CcSeverity {
    Error,
    Warning,
    Advisory,
}

impl std::fmt::Display for CcSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CcSeverity::Error => write!(f, "error"),
            CcSeverity::Warning => write!(f, "warning"),
            CcSeverity::Advisory => write!(f, "advisory"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CrossCrateFinding {
    pub rule: String,
    pub severity: CcSeverity,
    pub crate_a: String,
    pub crate_b: String,
    pub function_a: String,
    pub function_b: String,
    pub file_a: String,
    pub file_b: String,
    pub similarity: Option<f64>,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CrossCrateSummary {
    pub total_findings: usize,
    pub errors: usize,
    pub warnings: usize,
    pub advisories: usize,
    pub rules_triggered: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CrossCrateReport {
    pub findings: Vec<CrossCrateFinding>,
    pub summary: CrossCrateSummary,
    pub crates_analyzed: Vec<String>,
}

/// A function with its computed MinHash signature, grouped by crate.
struct SignedFunction {
    crate_name: String,
    function_name: String,
    #[allow(dead_code)]
    signature: String,
    file_path: String,
    minhash: MinHashSignature,
    #[allow(dead_code)]
    language: Language,
}

/// Ratchet baseline — persisted to `.pmat/cross-crate-baseline.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CrossCrateBaseline {
    version: String,
    generated: String,
    rule_counts: HashMap<String, usize>,
    total_findings: usize,
}

/// Configuration context passed through detection functions.
struct DetectionConfig {
    excluded_functions: HashSet<String>,
    excluded_crate_pairs: HashSet<(String, String)>,
    min_body_lines: usize,
    min_tokens: usize,
    cc003_min_similarity: f64,
}

impl DetectionConfig {
    fn from_yaml(cc: &CrossCrateConfig) -> Self {
        let excluded_functions: HashSet<String> =
            cc.excluded_functions.iter().map(|s| s.to_lowercase()).collect();
        let excluded_crate_pairs: HashSet<(String, String)> = cc
            .excluded_crate_pairs
            .iter()
            .filter_map(|pair| {
                let parts: Vec<&str> = pair.split(':').collect();
                if parts.len() == 2 {
                    Some((parts[0].to_string(), parts[1].to_string()))
                } else {
                    None
                }
            })
            .collect();
        Self {
            excluded_functions,
            excluded_crate_pairs,
            min_body_lines: cc.min_body_lines,
            min_tokens: cc.min_tokens,
            cc003_min_similarity: cc.cc003_min_similarity,
        }
    }
}

// --- Main handler ---

pub async fn handle_cross_crate(
    workspace_path: &Path,
    explicit_crates: Option<&[PathBuf]>,
    similarity_threshold: f64,
    churn_window_days: u32,
    rules_filter: Option<&str>,
    format: ComplyOutputFormat,
    output: Option<&Path>,
    strict: bool,
    save_baseline: bool,
) -> Result<()> {
    let yaml_config = PmatYamlConfig::load(workspace_path).unwrap_or_default();
    let det_config = DetectionConfig::from_yaml(&yaml_config.cross_crate);

    eprintln!("Discovering workspace crates...");
    let crates = discover_workspace_crates(workspace_path, explicit_crates);
    if crates.len() < 2 {
        print_discovery_help();
        return Ok(());
    }

    eprintln!("Loading functions from {} crates...", crates.len());
    let crate_functions = load_all_crate_functions(&crates);
    let crate_names: Vec<String> = crate_functions.iter().map(|(c, _)| c.name.clone()).collect();
    eprintln!("Analyzing {} crates: {}", crate_names.len(), crate_names.join(", "));

    let enabled_rules = parse_rules_filter(rules_filter);
    let findings = run_detection_rules(
        &crate_functions,
        &enabled_rules,
        &det_config,
        &yaml_config,
        similarity_threshold,
        churn_window_days,
    );
    let report = build_report(findings, crate_names);

    if save_baseline {
        save_ratchet_baseline(&report, workspace_path)?;
    }

    emit_report(&report, format, output)?;
    enforce_strict(&report, strict, workspace_path);

    Ok(())
}

fn print_discovery_help() {
    println!("Cross-crate analysis requires at least 2 crates.");
    println!("Discovery priority:");
    println!("  1. --crates ../foo,../bar  (explicit paths)");
    println!("  2. Cargo.toml [workspace]  (standard Cargo workspace)");
    println!("  3. batuta oracle --local    (batuta stack auto-discovery)");
    println!("  4. .pmat/workspace.toml     (manual siblings config)");
}

fn run_detection_rules(
    crate_functions: &[(CrateInfo, Vec<FunctionEntry>)],
    enabled_rules: &Option<HashSet<String>>,
    det_config: &DetectionConfig,
    yaml_config: &PmatYamlConfig,
    similarity_threshold: f64,
    churn_window_days: u32,
) -> Vec<CrossCrateFinding> {
    let mut findings = Vec::new();

    if is_rule_enabled("cc001", enabled_rules) {
        findings.extend(detect_cc001_function_clones(crate_functions, similarity_threshold, det_config));
    }
    if is_rule_enabled("cc002", enabled_rules) {
        findings.extend(detect_cc002_api_divergence(crate_functions, det_config));
    }
    if is_rule_enabled("cc003", enabled_rules) {
        findings.extend(detect_cc003_primitive_upstream(crate_functions, det_config));
    }
    if is_rule_enabled("cc004", enabled_rules) {
        findings.extend(detect_cc004_churn_correlation(crate_functions, churn_window_days));
    }
    if is_rule_enabled("cc005", enabled_rules) {
        findings.extend(detect_cc005_example_duplication(crate_functions, similarity_threshold));
    }

    // Apply suppressions and crate-pair exclusions
    findings.retain(|f| yaml_config.comply.is_suppressed(&f.rule, &f.file_b).is_none());
    findings.retain(|f| !is_crate_pair_excluded(&f.crate_a, &f.crate_b, &det_config.excluded_crate_pairs));
    findings
}

fn save_ratchet_baseline(report: &CrossCrateReport, workspace_path: &Path) -> Result<()> {
    let baseline = CrossCrateBaseline::from_report(report);
    baseline.save(workspace_path)?;
    eprintln!(
        "Baseline saved to .pmat/cross-crate-baseline.json ({} findings)",
        baseline.total_findings
    );
    Ok(())
}

fn emit_report(
    report: &CrossCrateReport,
    format: ComplyOutputFormat,
    output: Option<&Path>,
) -> Result<()> {
    let output_text = match format {
        ComplyOutputFormat::Text => format_text(report),
        ComplyOutputFormat::Json => serde_json::to_string_pretty(report)?,
        ComplyOutputFormat::Markdown => format_markdown(report),
    };

    if let Some(path) = output {
        std::fs::write(path, &output_text)?;
        eprintln!("Report written to {}", path.display());
    } else {
        println!("{output_text}");
    }
    Ok(())
}

fn enforce_strict(report: &CrossCrateReport, strict: bool, workspace_path: &Path) {
    if !strict {
        return;
    }

    match CrossCrateBaseline::load(workspace_path) {
        Some(baseline) => {
            let violations = baseline.check_ratchet(report);
            if violations.is_empty() {
                eprintln!("Ratchet check passed (no rule count increased)");
                return;
            }
            eprintln!("\nRatchet violations (finding count increased):");
            for (rule, old, new) in &violations {
                eprintln!("  {}: {} -> {} (+{})", rule, old, new, new - old);
            }
            std::process::exit(1);
        }
        None if report.summary.total_findings > 0 => std::process::exit(1),
        None => {}
    }
}

// --- Workspace discovery ---

/// Discover crates to analyze with priority chain:
/// 1. Explicit `--crates` paths
/// 2. Cargo.toml `[workspace]` members
/// 3. `batuta oracle --local` (batuta stack auto-discovery)
/// 4. `.pmat/workspace.toml` siblings (legacy)
/// 5. Single-crate fallback
pub fn discover_workspace_crates(
    workspace_path: &Path,
    explicit_crates: Option<&[PathBuf]>,
) -> Vec<CrateInfo> {
    // Priority 1: Explicit --crates flag
    if let Some(paths) = explicit_crates {
        if !paths.is_empty() {
            eprintln!("  Discovery: using explicit --crates paths");
            return discover_from_explicit(workspace_path, paths);
        }
    }

    // Priority 2: Cargo.toml [workspace] section
    let workspace_crates = discover_from_cargo_workspace(workspace_path);
    if workspace_crates.len() >= 2 {
        eprintln!("  Discovery: found Cargo workspace with {} members", workspace_crates.len());
        return workspace_crates;
    }

    // Priority 3: batuta oracle --local (batuta stack auto-discovery)
    let oracle_crates = discover_from_batuta_oracle(workspace_path);
    if oracle_crates.len() >= 2 {
        eprintln!(
            "  Discovery: batuta oracle found {} stack crates",
            oracle_crates.len()
        );
        return oracle_crates;
    }

    // Priority 4: .pmat/workspace.toml siblings (legacy, backward-compatible)
    let sibling_crates = discover_from_pmat_siblings(workspace_path);
    if sibling_crates.len() >= 2 {
        eprintln!(
            "  Discovery: .pmat/workspace.toml has {} siblings",
            sibling_crates.len()
        );
        return sibling_crates;
    }

    // Priority 5: Single-crate fallback
    vec![make_crate_info(workspace_path)]
}

/// Priority 1: Build CrateInfo from explicit paths.
fn discover_from_explicit(workspace_path: &Path, paths: &[PathBuf]) -> Vec<CrateInfo> {
    let mut crates = vec![make_crate_info(workspace_path)];

    for p in paths {
        let resolved = if p.is_absolute() {
            p.clone()
        } else {
            match workspace_path.join(p).canonicalize() {
                Ok(abs) => abs,
                Err(_) => continue,
            }
        };
        if !resolved.join("Cargo.toml").exists() {
            eprintln!("  Warning: {} has no Cargo.toml, skipping", resolved.display());
            continue;
        }
        // Skip if same as workspace_path
        if let (Ok(a), Ok(b)) = (workspace_path.canonicalize(), resolved.canonicalize()) {
            if a == b {
                continue;
            }
        }
        crates.push(make_crate_info(&resolved));
    }

    crates
}

/// Priority 2: Parse Cargo.toml [workspace] members with glob expansion.
fn discover_from_cargo_workspace(workspace_path: &Path) -> Vec<CrateInfo> {
    let cargo_toml = workspace_path.join("Cargo.toml");
    let Ok(content) = std::fs::read_to_string(&cargo_toml) else {
        return Vec::new();
    };

    // Check for [workspace] section
    if !content.contains("[workspace]") {
        return Vec::new();
    }

    // Extract members array using regex-free parsing
    let members = parse_workspace_members_with_globs(&content, workspace_path);
    if members.is_empty() {
        return Vec::new();
    }

    members.iter().map(|p| make_crate_info(p)).collect()
}

/// Parse `members = [...]` from workspace TOML, expanding glob patterns.
fn parse_workspace_members_with_globs(content: &str, base: &Path) -> Vec<PathBuf> {
    let members_buf = extract_members_array(content);
    let raw_members = extract_quoted_strings(&members_buf);
    resolve_member_paths(&raw_members, base)
}

/// Extract the raw `members = [...]` array content from TOML.
fn extract_members_array(content: &str) -> String {
    let mut in_members = false;
    let mut bracket_depth = 0;
    let mut buf = String::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if !in_members {
            if trimmed.starts_with("members") && trimmed.contains('=') {
                in_members = true;
                if let Some(after_eq) = trimmed.splitn(2, '=').nth(1) {
                    buf.push_str(after_eq);
                }
            }
            continue;
        }

        buf.push_str(trimmed);
        bracket_depth += trimmed.chars().filter(|&c| c == '[').count();
        bracket_depth -= trimmed.chars().filter(|&c| c == ']').count();

        if bracket_depth == 0 {
            break;
        }
    }

    buf
}

/// Extract double-quoted strings from a TOML array fragment.
fn extract_quoted_strings(buf: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut in_quote = false;
    let mut current = String::new();

    for ch in buf.chars() {
        match (ch, in_quote) {
            ('"', false) => in_quote = true,
            ('"', true) => {
                in_quote = false;
                if !current.is_empty() {
                    result.push(std::mem::take(&mut current));
                }
            }
            (_, true) => current.push(ch),
            _ => {}
        }
    }

    result
}

/// Resolve member path strings to absolute paths, expanding globs.
fn resolve_member_paths(raw_members: &[String], base: &Path) -> Vec<PathBuf> {
    let mut resolved = Vec::new();

    for member in raw_members {
        if member.contains('*') || member.contains('?') {
            let pattern = base.join(member).to_string_lossy().to_string();
            if let Ok(entries) = glob::glob(&pattern) {
                resolved.extend(entries.flatten().filter(|e| e.join("Cargo.toml").exists()));
            }
        } else {
            let member_path = base.join(member);
            if member_path.join("Cargo.toml").exists() {
                resolved.push(member_path);
            }
        }
    }

    resolved
}

/// Priority 3: Use `batuta oracle --local --format json` to discover PAIML stack crates.
///
/// The oracle knows the full batuta stack topology. We find the current crate in the
/// oracle's project list, then include all crates that share a dependency relationship
/// with it (direct deps or reverse deps within the PAIML stack).
fn discover_from_batuta_oracle(workspace_path: &Path) -> Vec<CrateInfo> {
    let Some(projects) = invoke_batuta_oracle() else {
        return Vec::new();
    };

    let canonical_ws = workspace_path
        .canonicalize()
        .unwrap_or_else(|_| workspace_path.to_path_buf());

    let Some(current_name) = find_current_project(&projects, &canonical_ws) else {
        return Vec::new();
    };

    let related = collect_related_crates(&projects, &current_name);
    projects_to_crate_infos(&projects, &related)
}

/// Run `batuta oracle --local --format json` and parse the projects map.
fn invoke_batuta_oracle() -> Option<serde_json::Map<String, serde_json::Value>> {
    let output = std::process::Command::new("batuta")
        .args(["oracle", "--local", "--format", "json"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json_start = stdout.find('{')?;
    let parsed: serde_json::Value = serde_json::from_str(&stdout[json_start..]).ok()?;
    parsed
        .get("projects")
        .and_then(|p| p.as_object().cloned())
}

/// Find the current project name by matching canonical paths.
fn find_current_project(
    projects: &serde_json::Map<String, serde_json::Value>,
    canonical_ws: &Path,
) -> Option<String> {
    projects.iter().find_map(|(name, info)| {
        let path_str = info.get("path")?.as_str()?;
        let project_path = PathBuf::from(path_str);
        let canonical = project_path.canonicalize().unwrap_or(project_path);
        (canonical == canonical_ws).then(|| name.clone())
    })
}

/// Extract PAIML dependency names from a project's JSON value.
fn extract_paiml_dep_names(info: &serde_json::Value) -> Vec<String> {
    info.get("paiml_dependencies")
        .and_then(|d| d.as_array())
        .map(|deps| {
            deps.iter()
                .filter_map(|dep| dep.get("name")?.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// Collect all crates related to `current_name` (forward + reverse deps).
fn collect_related_crates(
    projects: &serde_json::Map<String, serde_json::Value>,
    current_name: &str,
) -> HashSet<String> {
    let mut related = HashSet::new();
    related.insert(current_name.to_string());

    // Forward deps
    if let Some(info) = projects.get(current_name) {
        for dep_name in extract_paiml_dep_names(info) {
            related.insert(dep_name);
        }
    }

    // Reverse deps
    for (name, info) in projects {
        let dep_names = extract_paiml_dep_names(info);
        if dep_names.iter().any(|d| d == current_name) {
            related.insert(name.clone());
        }
    }

    related
}

/// Convert related crate names to CrateInfo, filtering to those with local paths.
fn projects_to_crate_infos(
    projects: &serde_json::Map<String, serde_json::Value>,
    related: &HashSet<String>,
) -> Vec<CrateInfo> {
    related
        .iter()
        .filter_map(|crate_name| {
            let info = projects.get(crate_name)?;
            let path_str = info.get("path")?.as_str()?;
            let crate_path = PathBuf::from(path_str);
            crate_path
                .join("Cargo.toml")
                .exists()
                .then(|| make_crate_info(&crate_path))
        })
        .collect()
}

/// Priority 4: Legacy `.pmat/workspace.toml` siblings.
fn discover_from_pmat_siblings(workspace_path: &Path) -> Vec<CrateInfo> {
    let mut crates = vec![make_crate_info(workspace_path)];

    let workspace_toml = workspace_path.join(".pmat").join("workspace.toml");
    if let Ok(content) = std::fs::read_to_string(&workspace_toml) {
        let siblings = parse_workspace_siblings(&content);
        for sibling_rel in siblings {
            let Ok(sibling_path) = workspace_path.join(&sibling_rel).canonicalize() else {
                continue;
            };
            if !sibling_path.join("Cargo.toml").exists() {
                continue;
            }
            crates.push(make_crate_info(&sibling_path));
        }
    }

    crates
}

/// Build a CrateInfo from a crate directory, reading its name from Cargo.toml.
fn make_crate_info(crate_path: &Path) -> CrateInfo {
    let cargo_toml = crate_path.join("Cargo.toml");
    let name = read_crate_name(&cargo_toml).unwrap_or_else(|| {
        crate_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string()
    });
    let cargo_deps = read_cargo_deps(&cargo_toml);
    CrateInfo {
        name,
        path: crate_path.to_path_buf(),
        cargo_deps,
    }
}

/// Extract `name = "..."` from [package] section of a Cargo.toml.
fn read_crate_name(cargo_toml: &Path) -> Option<String> {
    let content = std::fs::read_to_string(cargo_toml).ok()?;
    let mut in_package = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package = trimmed == "[package]";
            continue;
        }
        if in_package && trimmed.starts_with("name") {
            if let Some(eq_pos) = trimmed.find('=') {
                let value = trimmed[eq_pos + 1..].trim().trim_matches('"');
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
    }
    None
}

/// Parse dependency names from a Cargo.toml [dependencies] section.
/// Simple string parser — no full TOML parser needed.
pub fn read_cargo_deps(cargo_toml: &Path) -> Vec<String> {
    let Ok(content) = std::fs::read_to_string(cargo_toml) else {
        return Vec::new();
    };

    let mut deps = Vec::new();
    let mut in_deps_section = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with('[') {
            in_deps_section = trimmed == "[dependencies]"
                || trimmed.starts_with("[dependencies.")
                || trimmed == "[dev-dependencies]"
                || trimmed.starts_with("[dev-dependencies.");
            continue;
        }

        if in_deps_section {
            // Parse: crate_name = "version" or crate_name = { ... }
            if let Some(eq_pos) = trimmed.find('=') {
                let dep_name = trimmed[..eq_pos].trim().to_string();
                if !dep_name.is_empty() && !dep_name.starts_with('#') {
                    deps.push(dep_name);
                }
            }
        }
    }

    deps
}

/// Load functions from each crate's pmat index.
fn load_all_crate_functions(crates: &[CrateInfo]) -> Vec<(CrateInfo, Vec<FunctionEntry>)> {
    let mut result = Vec::new();

    for crate_info in crates {
        let index_path = crate_info.path.join(".pmat").join("context.idx");
        match AgentContextIndex::load(&index_path) {
            Ok(mut index) => {
                index.load_all_source();
                let functions: Vec<FunctionEntry> = index.all_functions().to_vec();
                eprintln!(
                    "  {} — {} functions loaded",
                    crate_info.name,
                    functions.len()
                );
                result.push((crate_info.clone(), functions));
            }
            Err(e) => {
                eprintln!("  {} — skipped (no index: {})", crate_info.name, e);
            }
        }
    }

    result
}

/// Parse a language string into the duplicate_detector Language enum, defaulting to Rust.
fn parse_language(lang: &str) -> Language {
    match lang.to_lowercase().as_str() {
        "rust" => Language::Rust,
        "typescript" => Language::TypeScript,
        "javascript" => Language::JavaScript,
        "python" => Language::Python,
        "c" => Language::C,
        "cpp" | "c++" => Language::Cpp,
        "kotlin" => Language::Kotlin,
        _ => Language::Rust,
    }
}

/// Parse --rules filter into a set of enabled rule IDs.
fn parse_rules_filter(rules: Option<&str>) -> Option<HashSet<String>> {
    rules.map(|r| {
        r.split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect()
    })
}

fn is_rule_enabled(rule: &str, filter: &Option<HashSet<String>>) -> bool {
    match filter {
        None => true,
        Some(set) => set.contains(rule),
    }
}

/// Check if a function name should be excluded from detection.
/// Combines hardcoded generic names with user-configured exclusions.
fn is_excluded_function(name: &str, config: &DetectionConfig) -> bool {
    is_generic_impl_name(name) || config.excluded_functions.contains(&name.to_lowercase())
}

/// Check if a crate pair is excluded from analysis.
fn is_crate_pair_excluded(
    crate_a: &str,
    crate_b: &str,
    excluded: &HashSet<(String, String)>,
) -> bool {
    excluded.contains(&(crate_a.to_string(), crate_b.to_string()))
        || excluded.contains(&(crate_b.to_string(), crate_a.to_string()))
}

/// Names too generic for meaningful cross-crate clone detection.
/// These are trait impls (Default, Display, From, etc.) that are
/// trivially duplicated and don't represent real copy-paste.
fn is_generic_impl_name(name: &str) -> bool {
    matches!(
        name,
        // Trait impls
        "default" | "new" | "fmt" | "clone" | "from" | "into"
            | "drop" | "deref" | "deref_mut" | "as_ref" | "as_mut"
            | "borrow" | "borrow_mut" | "try_from" | "try_into"
            | "hash" | "eq" | "partial_cmp" | "cmp" | "partial_eq"
            | "serialize" | "deserialize" | "display"
            | "index" | "index_mut" | "next" | "size_hint"
            | "poll" | "resume" | "init" | "build"
            // Trivial accessors (too short for meaningful clone detection)
            | "len" | "is_empty" | "is_full" | "capacity"
            | "get" | "set" | "push" | "pop" | "insert" | "remove"
            | "contains" | "clear" | "iter" | "name" | "id"
            | "width" | "height" | "size" | "count"
            // Additional trivial accessors (expanded for false-positive reduction)
            | "shape" | "dim" | "duration" | "alpha" | "beta" | "gamma"
            | "epsilon" | "rows" | "cols" | "dtype" | "ndim" | "rank"
            | "start" | "end" | "min" | "max" | "sum" | "mean"
            | "value" | "key" | "offset" | "stride" | "vocab_size"
    )
}

/// Compute MinHash signatures for all functions across all crates.
/// Filters out excluded functions and very short functions.
fn compute_signatures(
    crate_functions: &[(CrateInfo, Vec<FunctionEntry>)],
    config: &DetectionConfig,
) -> Vec<SignedFunction> {
    let dup_config = DuplicateDetectionConfig {
        normalize_identifiers: true,
        normalize_literals: true,
        ignore_comments: true,
        ..Default::default()
    };
    let extractor = UniversalFeatureExtractor::new(dup_config);
    let hasher = MinHashGenerator::new(128);

    let mut signed = Vec::new();

    for (crate_info, functions) in crate_functions {
        for func in functions {
            if func.source.is_empty() || func.source.lines().count() < config.min_body_lines {
                continue;
            }
            if is_excluded_function(&func.function_name, config) {
                continue;
            }
            let lang = parse_language(&func.language);
            let tokens = extractor.extract_features(&func.source, lang);
            if tokens.len() < config.min_tokens {
                continue;
            }
            let shingles = hasher.generate_shingles(&tokens, 3);
            if shingles.is_empty() {
                continue;
            }
            let minhash = hasher.compute_signature(&shingles);
            signed.push(SignedFunction {
                crate_name: crate_info.name.clone(),
                function_name: func.function_name.clone(),
                signature: func.signature.clone(),
                file_path: func.file_path.clone(),
                minhash,
                language: lang,
            });
        }
    }

    signed
}

fn build_report(
    findings: Vec<CrossCrateFinding>,
    crates_analyzed: Vec<String>,
) -> CrossCrateReport {
    let mut rules_triggered: HashMap<String, usize> = HashMap::new();
    let mut errors = 0;
    let mut warnings = 0;
    let mut advisories = 0;

    for f in &findings {
        *rules_triggered.entry(f.rule.clone()).or_insert(0) += 1;
        match f.severity {
            CcSeverity::Error => errors += 1,
            CcSeverity::Warning => warnings += 1,
            CcSeverity::Advisory => advisories += 1,
        }
    }

    CrossCrateReport {
        summary: CrossCrateSummary {
            total_findings: findings.len(),
            errors,
            warnings,
            advisories,
            rules_triggered,
        },
        findings,
        crates_analyzed,
    }
}

// --- Ratchet baseline ---

/// Compute the ratchet threshold for a given rule.
/// MinHash-based rules (CC-001, CC-003, CC-005) get 25% tolerance because
/// probabilistic signatures and lazy source loading cause ±15-30% variance.
/// Deterministic rules (CC-002, CC-004) use exact comparison.
fn ratchet_threshold(rule: &str, baseline_count: usize) -> usize {
    match rule {
        "CC-001" | "CC-003" | "CC-005" => baseline_count + baseline_count / 4,
        _ => baseline_count,
    }
}

impl CrossCrateBaseline {
    fn from_report(report: &CrossCrateReport) -> Self {
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
        let generated = format!("{y:04}-{m:02}-{d:02}");

        Self {
            version: "1.0".to_string(),
            generated,
            rule_counts: report.summary.rules_triggered.clone(),
            total_findings: report.summary.total_findings,
        }
    }

    fn save(&self, workspace_path: &Path) -> Result<()> {
        let pmat_dir = workspace_path.join(".pmat");
        std::fs::create_dir_all(&pmat_dir)?;
        let baseline_path = pmat_dir.join("cross-crate-baseline.json");
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(baseline_path, content)?;
        Ok(())
    }

    fn load(workspace_path: &Path) -> Option<Self> {
        let baseline_path = workspace_path
            .join(".pmat")
            .join("cross-crate-baseline.json");
        let content = std::fs::read_to_string(baseline_path).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Check if any rule count increased vs the baseline.
    /// Returns list of (rule, old_count, new_count) violations.
    ///
    /// Uses a 25% tolerance margin for MinHash-based rules (CC-001, CC-003, CC-005)
    /// because source lazy-loading and probabilistic signatures cause ±15-30% variance
    /// between runs. Deterministic rules (CC-002, CC-004) use exact comparison.
    fn check_ratchet(&self, report: &CrossCrateReport) -> Vec<(String, usize, usize)> {
        let mut violations = Vec::new();

        for (rule, &new_count) in &report.summary.rules_triggered {
            let old_count = self.rule_counts.get(rule).copied().unwrap_or(0);
            let threshold = ratchet_threshold(rule, old_count);
            if new_count > threshold {
                violations.push((rule.clone(), old_count, new_count));
            }
        }

        // Also check total with 25% tolerance
        let total_threshold = self.total_findings + self.total_findings / 4;
        if report.summary.total_findings > total_threshold {
            let already_has_total = violations.iter().any(|(r, _, _)| r == "TOTAL");
            if !already_has_total {
                violations.push((
                    "TOTAL".to_string(),
                    self.total_findings,
                    report.summary.total_findings,
                ));
            }
        }

        violations
    }
}

// --- Include sub-modules ---

include!("cross_crate_cc001_cc002.rs");
include!("cross_crate_cc003_cc004.rs");
include!("cross_crate_cc005_output.rs");

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover_workspace_crates_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let crates = discover_workspace_crates(tmp.path(), None);
        // Should have at least the base crate
        assert_eq!(crates.len(), 1);
        assert!(!crates[0].name.is_empty());
    }

    #[test]
    fn test_discover_from_cargo_workspace() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        // Create workspace Cargo.toml
        std::fs::write(
            root.join("Cargo.toml"),
            r#"[workspace]
members = ["crate-a", "crate-b"]
"#,
        )
        .unwrap();

        // Create member crates
        for name in &["crate-a", "crate-b"] {
            let dir = root.join(name);
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(
                dir.join("Cargo.toml"),
                format!(
                    "[package]\nname = \"{}\"\nversion = \"0.1.0\"\n",
                    name
                ),
            )
            .unwrap();
        }

        let crates = discover_workspace_crates(root, None);
        assert_eq!(crates.len(), 2);
        let names: HashSet<&str> = crates.iter().map(|c| c.name.as_str()).collect();
        assert!(names.contains("crate-a"));
        assert!(names.contains("crate-b"));
    }

    #[test]
    fn test_discover_from_cargo_workspace_with_glob() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        // Create workspace Cargo.toml with glob
        std::fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/*\"]\n",
        )
        .unwrap();

        // Create member crates under crates/
        let crates_dir = root.join("crates");
        std::fs::create_dir_all(&crates_dir).unwrap();
        for name in &["alpha", "beta"] {
            let dir = crates_dir.join(name);
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(
                dir.join("Cargo.toml"),
                format!("[package]\nname = \"{}\"\nversion = \"0.1.0\"\n", name),
            )
            .unwrap();
        }

        let crates = discover_workspace_crates(root, None);
        assert_eq!(crates.len(), 2);
        let names: HashSet<&str> = crates.iter().map(|c| c.name.as_str()).collect();
        assert!(names.contains("alpha"));
        assert!(names.contains("beta"));
    }

    #[test]
    fn test_discover_explicit_crates() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        // Create base Cargo.toml
        std::fs::write(
            root.join("Cargo.toml"),
            "[package]\nname = \"base\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();

        // Create sibling crate
        let sibling = root.join("sibling");
        std::fs::create_dir_all(&sibling).unwrap();
        std::fs::write(
            sibling.join("Cargo.toml"),
            "[package]\nname = \"sibling\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();

        let explicit = vec![sibling];
        let crates = discover_workspace_crates(root, Some(&explicit));
        assert_eq!(crates.len(), 2);
        assert_eq!(crates[0].name, "base");
        assert_eq!(crates[1].name, "sibling");
    }

    #[test]
    fn test_read_cargo_deps_parses_section() {
        let tmp = tempfile::tempdir().unwrap();
        let cargo_toml = tmp.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            r#"[package]
name = "test"
version = "0.1.0"

[dependencies]
serde = "1.0"
tokio = { version = "1", features = ["full"] }
anyhow = "1"

[dev-dependencies]
tempfile = "3"
"#,
        )
        .unwrap();

        let deps = read_cargo_deps(&cargo_toml);
        assert!(deps.contains(&"serde".to_string()));
        assert!(deps.contains(&"tokio".to_string()));
        assert!(deps.contains(&"anyhow".to_string()));
        assert!(deps.contains(&"tempfile".to_string()));
    }

    #[test]
    fn test_read_crate_name() {
        let tmp = tempfile::tempdir().unwrap();
        let cargo_toml = tmp.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            "[package]\nname = \"my-cool-crate\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        assert_eq!(
            read_crate_name(&cargo_toml),
            Some("my-cool-crate".to_string())
        );
    }

    #[test]
    fn test_normalize_signature_strips_pub() {
        assert_eq!(
            normalize_signature("pub fn foo(x: i32) -> bool"),
            "fn foo(x: i32) -> bool"
        );
        assert_eq!(normalize_signature("pub async fn bar()"), "fn bar()");
        assert_eq!(normalize_signature("fn baz(s: &str)"), "fn baz(s: &str)");
    }

    #[test]
    fn test_cc001_detects_identical_source() {
        let crate_a = CrateInfo {
            name: "crate_a".to_string(),
            path: PathBuf::from("/tmp/a"),
            cargo_deps: vec![],
        };
        let crate_b = CrateInfo {
            name: "crate_b".to_string(),
            path: PathBuf::from("/tmp/b"),
            cargo_deps: vec![],
        };

        let source = r#"pub fn silu_activation(input: &[f32], output: &mut [f32]) {
    assert_eq!(input.len(), output.len(), "Input and output lengths must match");
    for i in 0..input.len() {
        let x = input[i];
        let sigmoid = 1.0 / (1.0 + (-x).exp());
        output[i] = x * sigmoid;
    }
}"#
        .to_string();
        let func_a = make_test_func("silu_activation", &source, "src/a.rs");
        let func_b = make_test_func("silu_activation", &source, "src/b.rs");

        let crate_functions = vec![(crate_a, vec![func_a]), (crate_b, vec![func_b])];
        let det = default_detection_config();

        let findings = detect_cc001_function_clones(&crate_functions, 0.80, &det);
        assert!(
            !findings.is_empty(),
            "CC-001 should detect identical functions across crates"
        );
        assert_eq!(findings[0].rule, "CC-001");
    }

    #[test]
    fn test_cc001_no_finding_within_same_crate() {
        let crate_a = CrateInfo {
            name: "crate_a".to_string(),
            path: PathBuf::from("/tmp/a"),
            cargo_deps: vec![],
        };

        let source = r#"pub fn silu_activation(input: &[f32], output: &mut [f32]) {
    assert_eq!(input.len(), output.len(), "Input and output lengths must match");
    for i in 0..input.len() {
        let x = input[i];
        let sigmoid = 1.0 / (1.0 + (-x).exp());
        output[i] = x * sigmoid;
    }
}"#
        .to_string();
        let func_a = make_test_func("silu_activation", &source, "src/a.rs");
        let func_b = make_test_func("silu_activation_v2", &source, "src/b.rs");

        let crate_functions = vec![(crate_a, vec![func_a, func_b])];
        let det = default_detection_config();

        let findings = detect_cc001_function_clones(&crate_functions, 0.80, &det);
        assert!(
            findings.is_empty(),
            "CC-001 should NOT flag duplicates within the same crate"
        );
    }

    #[test]
    fn test_cc002_same_name_different_sig() {
        let crate_a = CrateInfo {
            name: "crate_a".to_string(),
            path: PathBuf::from("/tmp/a"),
            cargo_deps: vec![],
        };
        let crate_b = CrateInfo {
            name: "crate_b".to_string(),
            path: PathBuf::from("/tmp/b"),
            cargo_deps: vec!["crate_a".to_string()],
        };

        let func_a = make_test_func_with_sig(
            "rms_norm",
            "pub fn rms_norm(x: &[f32]) -> Vec<f32>",
            "src/a.rs",
        );
        let func_b = make_test_func_with_sig(
            "rms_norm",
            "pub fn rms_norm(x: &[f64], eps: f64) -> Vec<f64>",
            "src/b.rs",
        );

        let crate_functions = vec![(crate_a, vec![func_a]), (crate_b, vec![func_b])];
        let det = default_detection_config();

        let findings = detect_cc002_api_divergence(&crate_functions, &det);
        assert!(
            !findings.is_empty(),
            "CC-002 should detect divergent signatures"
        );
        assert_eq!(findings[0].rule, "CC-002");
    }

    #[test]
    fn test_cc002_same_name_same_sig_no_finding() {
        let crate_a = CrateInfo {
            name: "crate_a".to_string(),
            path: PathBuf::from("/tmp/a"),
            cargo_deps: vec![],
        };
        let crate_b = CrateInfo {
            name: "crate_b".to_string(),
            path: PathBuf::from("/tmp/b"),
            cargo_deps: vec!["crate_a".to_string()],
        };

        let func_a = make_test_func_with_sig("gelu", "pub fn gelu(x: f32) -> f32", "src/a.rs");
        let func_b = make_test_func_with_sig("gelu", "pub fn gelu(x: f32) -> f32", "src/b.rs");

        let crate_functions = vec![(crate_a, vec![func_a]), (crate_b, vec![func_b])];
        let det = default_detection_config();

        let findings = detect_cc002_api_divergence(&crate_functions, &det);
        assert!(
            findings.is_empty(),
            "CC-002 should NOT flag identical signatures"
        );
    }

    #[test]
    fn test_cc003_finding_when_dep_reimplements() {
        let crate_a = CrateInfo {
            name: "trueno".to_string(),
            path: PathBuf::from("/tmp/trueno"),
            cargo_deps: vec![],
        };
        let crate_b = CrateInfo {
            name: "aprender".to_string(),
            path: PathBuf::from("/tmp/aprender"),
            cargo_deps: vec!["trueno".to_string()],
        };

        // Use longer, realistic function bodies so MinHash signatures are computed
        let src_a = r#"pub fn f16_to_f32(input: &[u16], output: &mut [f32]) {
    assert_eq!(input.len(), output.len(), "Length mismatch");
    for i in 0..input.len() {
        let bits = input[i];
        let sign = (bits >> 15) & 1;
        let exponent = (bits >> 10) & 0x1F;
        let mantissa = bits & 0x3FF;
        let f32_bits = (sign as u32) << 31 | (exponent as u32 + 112) << 23 | (mantissa as u32) << 13;
        output[i] = f32::from_bits(f32_bits);
    }
}"#;
        let src_b = r#"pub fn f16_to_f32(input: &[u16], output: &mut [f32]) {
    assert_eq!(input.len(), output.len(), "Length mismatch");
    for idx in 0..input.len() {
        let raw = input[idx];
        let sign = (raw >> 15) & 1;
        let exponent = (raw >> 10) & 0x1F;
        let mantissa = raw & 0x3FF;
        let f32_bits = (sign as u32) << 31 | (exponent as u32 + 112) << 23 | (mantissa as u32) << 13;
        output[idx] = f32::from_bits(f32_bits);
    }
}"#;
        let func_a = make_test_func("f16_to_f32", src_a, "src/conv.rs");
        let func_b = make_test_func("f16_to_f32", src_b, "src/quant.rs");

        let crate_functions = vec![(crate_a, vec![func_a]), (crate_b, vec![func_b])];
        let det = default_detection_config();

        let findings = detect_cc003_primitive_upstream(&crate_functions, &det);
        assert!(
            !findings.is_empty(),
            "CC-003 should detect reimplementation of upstream function"
        );
        assert_eq!(findings[0].rule, "CC-003");
        assert!(
            findings[0].similarity.is_some(),
            "CC-003 should now include similarity score from MinHash"
        );
    }

    #[test]
    fn test_cc003_no_finding_when_no_dep() {
        let crate_a = CrateInfo {
            name: "trueno".to_string(),
            path: PathBuf::from("/tmp/trueno"),
            cargo_deps: vec![],
        };
        let crate_b = CrateInfo {
            name: "unrelated".to_string(),
            path: PathBuf::from("/tmp/unrelated"),
            cargo_deps: vec!["serde".to_string()],
        };

        let src_a = r#"pub fn f16_to_f32(input: &[u16], output: &mut [f32]) {
    assert_eq!(input.len(), output.len(), "Length mismatch");
    for i in 0..input.len() {
        let bits = input[i];
        let sign = (bits >> 15) & 1;
        let exponent = (bits >> 10) & 0x1F;
        output[i] = f32::from_bits((sign as u32) << 31 | (exponent as u32) << 23);
    }
}"#;
        let src_b = r#"pub fn f16_to_f32(input: &[u16], output: &mut [f32]) {
    assert_eq!(input.len(), output.len(), "Length mismatch");
    for idx in 0..input.len() {
        let raw = input[idx];
        let sign = (raw >> 15) & 1;
        let exponent = (raw >> 10) & 0x1F;
        output[idx] = f32::from_bits((sign as u32) << 31 | (exponent as u32) << 23);
    }
}"#;
        let func_a = make_test_func("f16_to_f32", src_a, "src/conv.rs");
        let func_b = make_test_func("f16_to_f32", src_b, "src/quant.rs");

        let crate_functions = vec![(crate_a, vec![func_a]), (crate_b, vec![func_b])];
        let det = default_detection_config();

        let findings = detect_cc003_primitive_upstream(&crate_functions, &det);
        assert!(
            findings.is_empty(),
            "CC-003 should NOT flag when no dependency relationship"
        );
    }

    #[test]
    fn test_cc005_no_examples_dir() {
        let crate_a = CrateInfo {
            name: "crate_a".to_string(),
            path: PathBuf::from("/tmp/nonexistent_crate_path"),
            cargo_deps: vec![],
        };

        let crate_functions = vec![(crate_a, vec![])];

        let findings = detect_cc005_example_duplication(&crate_functions, 0.80);
        assert!(
            findings.is_empty(),
            "CC-005 should gracefully skip missing examples/"
        );
    }

    #[test]
    fn test_build_report_summary() {
        let findings = vec![
            CrossCrateFinding {
                rule: "CC-001".to_string(),
                severity: CcSeverity::Error,
                crate_a: "a".to_string(),
                crate_b: "b".to_string(),
                function_a: "f".to_string(),
                function_b: "f".to_string(),
                file_a: "a.rs".to_string(),
                file_b: "b.rs".to_string(),
                similarity: Some(0.95),
                recommendation: "Consolidate".to_string(),
            },
            CrossCrateFinding {
                rule: "CC-002".to_string(),
                severity: CcSeverity::Warning,
                crate_a: "a".to_string(),
                crate_b: "c".to_string(),
                function_a: "g".to_string(),
                function_b: "g".to_string(),
                file_a: "a.rs".to_string(),
                file_b: "c.rs".to_string(),
                similarity: None,
                recommendation: "Align signatures".to_string(),
            },
            CrossCrateFinding {
                rule: "CC-004".to_string(),
                severity: CcSeverity::Advisory,
                crate_a: "a".to_string(),
                crate_b: "b".to_string(),
                function_a: "h".to_string(),
                function_b: "h".to_string(),
                file_a: "a.rs".to_string(),
                file_b: "b.rs".to_string(),
                similarity: None,
                recommendation: "Review".to_string(),
            },
        ];

        let report = build_report(
            findings,
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
        );
        assert_eq!(report.summary.total_findings, 3);
        assert_eq!(report.summary.errors, 1);
        assert_eq!(report.summary.warnings, 1);
        assert_eq!(report.summary.advisories, 1);
        assert_eq!(report.summary.rules_triggered["CC-001"], 1);
        assert_eq!(report.summary.rules_triggered["CC-002"], 1);
    }

    #[test]
    fn test_parse_rules_filter() {
        assert!(parse_rules_filter(None).is_none());

        let set = parse_rules_filter(Some("cc001,cc003")).unwrap();
        assert!(set.contains("cc001"));
        assert!(set.contains("cc003"));
        assert!(!set.contains("cc002"));

        let set2 = parse_rules_filter(Some(" CC001 , CC002 ")).unwrap();
        assert!(set2.contains("cc001"));
        assert!(set2.contains("cc002"));
    }

    #[test]
    fn test_excluded_function_combines_hardcoded_and_config() {
        let config = DetectionConfig {
            excluded_functions: HashSet::from(["my_accessor".to_string()]),
            excluded_crate_pairs: HashSet::new(),
            min_body_lines: 3,
            min_tokens: 15,
            cc003_min_similarity: 0.5,
        };
        // Hardcoded exclusion
        assert!(is_excluded_function("shape", &config));
        assert!(is_excluded_function("default", &config));
        // Config exclusion
        assert!(is_excluded_function("my_accessor", &config));
        // Not excluded
        assert!(!is_excluded_function("silu_activation", &config));
    }

    #[test]
    fn test_crate_pair_excluded() {
        let excluded = HashSet::from([("trueno".to_string(), "aprender".to_string())]);
        assert!(is_crate_pair_excluded("trueno", "aprender", &excluded));
        assert!(is_crate_pair_excluded("aprender", "trueno", &excluded));
        assert!(!is_crate_pair_excluded("trueno", "realizar", &excluded));
    }

    #[test]
    fn test_baseline_ratchet_passes_when_equal() {
        let baseline = CrossCrateBaseline {
            version: "1.0".to_string(),
            generated: "2026-02-20".to_string(),
            rule_counts: HashMap::from([("CC-001".to_string(), 10)]),
            total_findings: 10,
        };
        let report = CrossCrateReport {
            findings: Vec::new(),
            summary: CrossCrateSummary {
                total_findings: 10,
                errors: 0,
                warnings: 10,
                advisories: 0,
                rules_triggered: HashMap::from([("CC-001".to_string(), 10)]),
            },
            crates_analyzed: vec!["a".to_string()],
        };
        let violations = baseline.check_ratchet(&report);
        assert!(violations.is_empty(), "Same counts should pass ratchet");
    }

    #[test]
    fn test_baseline_ratchet_passes_when_decreased() {
        let baseline = CrossCrateBaseline {
            version: "1.0".to_string(),
            generated: "2026-02-20".to_string(),
            rule_counts: HashMap::from([("CC-001".to_string(), 10)]),
            total_findings: 10,
        };
        let report = CrossCrateReport {
            findings: Vec::new(),
            summary: CrossCrateSummary {
                total_findings: 5,
                errors: 0,
                warnings: 5,
                advisories: 0,
                rules_triggered: HashMap::from([("CC-001".to_string(), 5)]),
            },
            crates_analyzed: vec!["a".to_string()],
        };
        let violations = baseline.check_ratchet(&report);
        assert!(violations.is_empty(), "Decreased counts should pass ratchet");
    }

    #[test]
    fn test_baseline_ratchet_fails_when_increased() {
        let baseline = CrossCrateBaseline {
            version: "1.0".to_string(),
            generated: "2026-02-20".to_string(),
            rule_counts: HashMap::from([("CC-001".to_string(), 10)]),
            total_findings: 10,
        };
        let report = CrossCrateReport {
            findings: Vec::new(),
            summary: CrossCrateSummary {
                total_findings: 15,
                errors: 0,
                warnings: 15,
                advisories: 0,
                rules_triggered: HashMap::from([("CC-001".to_string(), 15)]),
            },
            crates_analyzed: vec!["a".to_string()],
        };
        let violations = baseline.check_ratchet(&report);
        assert!(!violations.is_empty(), "Increased counts should fail ratchet");
        assert_eq!(violations[0].0, "CC-001");
        assert_eq!(violations[0].1, 10);
        assert_eq!(violations[0].2, 15);
    }

    #[test]
    fn test_baseline_ratchet_tolerates_minhash_jitter() {
        // CC-001 has 25% tolerance: baseline=100, threshold=125
        let baseline = CrossCrateBaseline {
            version: "1.0".to_string(),
            generated: "2026-02-20".to_string(),
            rule_counts: HashMap::from([("CC-001".to_string(), 100)]),
            total_findings: 100,
        };
        let report = CrossCrateReport {
            findings: Vec::new(),
            summary: CrossCrateSummary {
                total_findings: 120,
                errors: 0,
                warnings: 120,
                advisories: 0,
                rules_triggered: HashMap::from([("CC-001".to_string(), 120)]),
            },
            crates_analyzed: vec!["a".to_string()],
        };
        let violations = baseline.check_ratchet(&report);
        assert!(violations.is_empty(), "20% increase in CC-001 should be within tolerance");
    }

    #[test]
    fn test_baseline_ratchet_cc002_exact() {
        // CC-002 is deterministic — no tolerance
        let baseline = CrossCrateBaseline {
            version: "1.0".to_string(),
            generated: "2026-02-20".to_string(),
            rule_counts: HashMap::from([("CC-002".to_string(), 100)]),
            total_findings: 100,
        };
        let report = CrossCrateReport {
            findings: Vec::new(),
            summary: CrossCrateSummary {
                total_findings: 101,
                errors: 0,
                warnings: 101,
                advisories: 0,
                rules_triggered: HashMap::from([("CC-002".to_string(), 101)]),
            },
            crates_analyzed: vec!["a".to_string()],
        };
        let violations = baseline.check_ratchet(&report);
        assert!(!violations.is_empty(), "CC-002 should fail on +1 increase (no tolerance)");
    }

    #[test]
    fn test_ratchet_threshold_function() {
        // MinHash-based rules get 25% tolerance
        assert_eq!(ratchet_threshold("CC-001", 100), 125);
        assert_eq!(ratchet_threshold("CC-003", 52), 65);
        assert_eq!(ratchet_threshold("CC-005", 60), 75);
        // Deterministic rules get exact comparison
        assert_eq!(ratchet_threshold("CC-002", 100), 100);
        assert_eq!(ratchet_threshold("CC-004", 50), 50);
    }

    #[test]
    fn test_cross_crate_config_yaml_parsing() {
        let yaml = r#"
cross_crate:
  excluded_functions: [shape, dim, duration]
  excluded_crate_pairs: ["trueno:aprender"]
  min_body_lines: 5
  min_tokens: 20
  cc003_min_similarity: 0.6
"#;
        let config: PmatYamlConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.cross_crate.excluded_functions.len(), 3);
        assert_eq!(config.cross_crate.excluded_crate_pairs.len(), 1);
        assert_eq!(config.cross_crate.min_body_lines, 5);
        assert_eq!(config.cross_crate.min_tokens, 20);
        assert!((config.cross_crate.cc003_min_similarity - 0.6).abs() < 0.001);
    }

    #[test]
    fn test_parse_workspace_members_with_globs_literal() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        // Create member directories
        for name in &["core", "utils"] {
            let dir = root.join(name);
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("Cargo.toml"), "[package]\nname = \"x\"\n").unwrap();
        }

        let content = "[workspace]\nmembers = [\"core\", \"utils\"]\n";
        let members = parse_workspace_members_with_globs(content, root);
        assert_eq!(members.len(), 2);
    }

    // --- Test helpers ---

    fn default_detection_config() -> DetectionConfig {
        DetectionConfig {
            excluded_functions: HashSet::new(),
            excluded_crate_pairs: HashSet::new(),
            min_body_lines: 3,
            min_tokens: 15,
            cc003_min_similarity: 0.5,
        }
    }

    fn make_test_func(name: &str, source: &str, file_path: &str) -> FunctionEntry {
        FunctionEntry {
            function_name: name.to_string(),
            signature: format!("fn {name}()"),
            source: source.to_string(),
            file_path: file_path.to_string(),
            doc_comment: None,
            definition_type: Default::default(),
            start_line: 1,
            end_line: 1,
            language: "Rust".to_string(),
            quality: Default::default(),
            checksum: String::new(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: vec![],
        }
    }

    fn make_test_func_with_sig(name: &str, sig: &str, file_path: &str) -> FunctionEntry {
        let mut func = make_test_func(name, "// placeholder source", file_path);
        func.signature = sig.to_string();
        func
    }
}
