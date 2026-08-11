#![cfg_attr(coverage_nightly, coverage(off))]
//! Stack sync handler: cross-repo dependency coordination for the sovereign AI stack
//!
//! Provides `pmat stack status` and `pmat stack sync` commands that discover
//! batuta stack repos, parse their Cargo.toml dependencies, check versions
//! against crates.io, and optionally apply updates.

use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::cli::colors as c;
use crate::cli::enums::OutputFormat;

// ── Constants ─────────────────────────────────────────────────────────────

const DEFAULT_STACK_REPOS: &[&str] = &[
    "trueno",
    "trueno-graph",
    "trueno-db",
    "trueno-rag",
    "trueno-viz",
    "trueno-zram-core",
    "aprender",
    "paiml-mcp-agent-toolkit",
    "certeza",
    "bashrs",
    "probar",
    "renacer",
    "presentar",
    "pmcp",
];

const BATUTA_CRATES: &[&str] = &[
    "trueno",
    "trueno-graph",
    "trueno-db",
    "trueno-rag",
    "trueno-viz",
    "trueno-zram-core",
    "aprender",
    "certeza",
    "bashrs",
    "probar",
    "renacer",
    "presentar",
    "presentar-core",
    "pmcp",
    "pmat",
];

// ── Data Types ────────────────────────────────────────────────────────────

#[derive(Debug, serde::Serialize)]
/// Information about repo.
pub struct RepoInfo {
    pub name: String,
    pub path: PathBuf,
    pub exists: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
/// Information about dep.
pub struct DepInfo {
    pub name: String,
    pub version: Option<String>,
    pub source: DepSource,
    pub section: String,
}

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
/// Dep source.
pub enum DepSource {
    Registry,
    Path(String),
    Git(String),
}

#[derive(Debug, serde::Serialize)]
/// Dep mismatch.
pub struct DepMismatch {
    pub repo: String,
    pub crate_name: String,
    pub current_version: String,
    pub latest_version: String,
}

#[derive(Debug, serde::Serialize)]
/// Stack status.
pub struct StackStatus {
    pub repos_found: usize,
    pub repos_missing: usize,
    pub total_deps: usize,
    pub outdated_deps: usize,
    pub mismatches: Vec<DepMismatch>,
}

#[derive(Debug, serde::Serialize)]
/// Stack graph.
pub struct StackGraph {
    pub repos: Vec<RepoInfo>,
    pub edges: Vec<(usize, usize, String)>, // (from_repo, to_repo, crate_name)
}

#[derive(Debug, serde::Serialize)]
struct StackManifest {
    repos: Vec<String>,
    base_path: PathBuf,
}

// ── Stack Manifest Discovery ──────────────────────────────────────────────

fn load_stack_manifest() -> Result<StackManifest> {
    let base_path = resolve_base_path();

    // Try .pmat/stack.toml first
    let stack_toml = PathBuf::from(".pmat/stack.toml");
    if stack_toml.exists() {
        let content = std::fs::read_to_string(&stack_toml)
            .with_context(|| format!("Failed to read {}", stack_toml.display()))?;
        let table: toml::Value =
            toml::from_str(&content).with_context(|| "Failed to parse .pmat/stack.toml")?;

        if let Some(repos) = table.get("repos").and_then(|v| v.as_array()) {
            let repo_names: Vec<String> = repos
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
            if !repo_names.is_empty() {
                return Ok(StackManifest {
                    repos: repo_names,
                    base_path,
                });
            }
        }
    }

    // Fall back to defaults
    Ok(StackManifest {
        repos: DEFAULT_STACK_REPOS
            .iter()
            .map(|s| (*s).to_string())
            .collect(),
        base_path,
    })
}

fn resolve_base_path() -> PathBuf {
    // Resolve repo paths by looking in parent of current project (~/src/)
    if let Ok(cwd) = std::env::current_dir() {
        if let Some(parent) = cwd.parent() {
            return parent.to_path_buf();
        }
    }
    // Fallback to ~/src
    if let Some(home) = dirs_next_home() {
        return home.join("src");
    }
    PathBuf::from(".")
}

fn dirs_next_home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn resolve_repo_path(base: &Path, repo_name: &str) -> PathBuf {
    base.join(repo_name)
}

// ── Cargo.toml Parsing ───────────────────────────────────────────────────

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
/// Parse cargo dependencies.
pub fn parse_cargo_dependencies(cargo_toml_path: &Path) -> Result<Vec<DepInfo>> {
    let content = std::fs::read_to_string(cargo_toml_path)
        .with_context(|| format!("Failed to read {}", cargo_toml_path.display()))?;
    parse_cargo_toml_content(&content)
}

fn parse_cargo_toml_content(content: &str) -> Result<Vec<DepInfo>> {
    let table: toml::Value =
        toml::from_str(content).with_context(|| "Failed to parse Cargo.toml")?;

    let mut deps = Vec::new();

    let sections = &[
        ("dependencies", "dependencies"),
        ("dev-dependencies", "dev-dependencies"),
        ("build-dependencies", "build-dependencies"),
    ];

    for &(key, section_name) in sections {
        if let Some(section) = table.get(key).and_then(|v| v.as_table()) {
            for (name, value) in section {
                let dep = parse_single_dep(name, value, section_name);
                deps.push(dep);
            }
        }
    }

    // Also check workspace dependencies
    if let Some(workspace) = table.get("workspace").and_then(|v| v.as_table()) {
        if let Some(ws_deps) = workspace.get("dependencies").and_then(|v| v.as_table()) {
            for (name, value) in ws_deps {
                let dep = parse_single_dep(name, value, "workspace.dependencies");
                deps.push(dep);
            }
        }
    }

    Ok(deps)
}

fn parse_single_dep(name: &str, value: &toml::Value, section: &str) -> DepInfo {
    match value {
        toml::Value::String(version) => DepInfo {
            name: name.to_string(),
            version: Some(version.clone()),
            source: DepSource::Registry,
            section: section.to_string(),
        },
        toml::Value::Table(tbl) => {
            let version = tbl
                .get("version")
                .and_then(|v| v.as_str())
                .map(String::from);
            let source = if let Some(path) = tbl.get("path").and_then(|v| v.as_str()) {
                DepSource::Path(path.to_string())
            } else if let Some(git) = tbl.get("git").and_then(|v| v.as_str()) {
                DepSource::Git(git.to_string())
            } else {
                DepSource::Registry
            };
            DepInfo {
                name: name.to_string(),
                version,
                source,
                section: section.to_string(),
            }
        }
        _ => DepInfo {
            name: name.to_string(),
            version: None,
            source: DepSource::Registry,
            section: section.to_string(),
        },
    }
}

// ── Version Checking ─────────────────────────────────────────────────────

fn check_latest_version(crate_name: &str) -> Result<Option<String>> {
    let output = std::process::Command::new("cargo")
        .args(["search", crate_name, "--limit", "1"])
        .output()
        .with_context(|| format!("Failed to run cargo search {crate_name}"))?;

    if !output.status.success() {
        return Ok(None);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_cargo_search_output(&stdout, crate_name)
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
/// Parse cargo search output.
pub fn parse_cargo_search_output(output: &str, crate_name: &str) -> Result<Option<String>> {
    // Format: `crate_name = "0.4.30"    # Description`
    for line in output.lines() {
        let line = line.trim();
        // Match exact crate name at start of line
        if let Some(rest) = line.strip_prefix(crate_name) {
            let rest = rest.trim();
            if let Some(rest) = rest.strip_prefix('=') {
                let rest = rest.trim();
                if let Some(rest) = rest.strip_prefix('"') {
                    if let Some(end) = rest.find('"') {
                        return Ok(Some(rest[..end].to_string()));
                    }
                }
            }
        }
    }
    Ok(None)
}

// ── Batuta Crate Detection ───────────────────────────────────────────────

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
/// Is batuta crate.
pub fn is_batuta_crate(name: &str) -> bool {
    BATUTA_CRATES.contains(&name)
}

// ── Mismatch Detection ──────────────────────────────────────────────────

/// Lowest version a manifest requirement admits, with omitted components
/// filled in as 0 (`"2.17"` -> 2.17.0, `"^0.4.22"` -> 0.4.22).
fn requirement_floor(req: &str) -> Option<semver::Version> {
    let head = req.trim().split(',').next()?.trim();
    let head = head.trim_start_matches(['^', '~', '=', '>', '<']).trim();

    let mut parts = head.split('.');
    let major = parts.next()?.parse::<u64>().ok()?;
    let component = |p: Option<&str>| {
        p.and_then(|p| p.split(['-', '+']).next())
            .and_then(|p| p.parse::<u64>().ok())
            .unwrap_or(0)
    };
    let minor = component(parts.next());
    let patch = component(parts.next());

    Some(semver::Version::new(major, minor, patch))
}

/// Is the manifest requirement `current` behind registry version `latest`?
///
/// This used to be a raw string inequality after stripping `^`/`~`, so
/// `pmcp = "2.17"` — a requirement 2.17.0 satisfies exactly — was reported as
/// `2.17 -> 2.17.0`, and the outdated count necessarily equalled the total dep
/// count (23 of 23 in this repo). Compare versions as versions: a requirement
/// whose floor already is the latest version is not outdated, whatever the two
/// strings look like. Unparseable input falls back to the string compare so an
/// unrecognised requirement is never silently reported as up to date.
fn requirement_is_outdated(current: &str, latest: &str) -> bool {
    match (requirement_floor(current), semver::Version::parse(latest)) {
        (Some(floor), Ok(latest_version)) => floor < latest_version,
        _ => {
            let current_clean = current.trim_start_matches('^').trim_start_matches('~');
            current_clean != latest
        }
    }
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
/// Detect mismatches.
pub fn detect_mismatches(
    repo_name: &str,
    deps: &[DepInfo],
    latest_versions: &BTreeMap<String, String>,
) -> Vec<DepMismatch> {
    let mut mismatches = Vec::new();

    for dep in deps {
        if !is_batuta_crate(&dep.name) {
            continue;
        }
        // Only check registry deps (path/git deps are local)
        if dep.source != DepSource::Registry {
            continue;
        }
        if let Some(current) = &dep.version {
            if let Some(latest) = latest_versions.get(&dep.name) {
                if requirement_is_outdated(current, latest) {
                    mismatches.push(DepMismatch {
                        repo: repo_name.to_string(),
                        crate_name: dep.name.clone(),
                        current_version: current.clone(),
                        latest_version: latest.clone(),
                    });
                }
            }
        }
    }

    mismatches
}

// ── Dependency Graph ─────────────────────────────────────────────────────

fn build_stack_graph(repos: &[RepoInfo], all_deps: &BTreeMap<String, Vec<DepInfo>>) -> StackGraph {
    let mut edges = Vec::new();

    // Build name -> repo index map
    let repo_index: BTreeMap<&str, usize> = repos
        .iter()
        .enumerate()
        .map(|(i, r)| (r.name.as_str(), i))
        .collect();

    for (from_idx, repo) in repos.iter().enumerate() {
        if let Some(deps) = all_deps.get(&repo.name) {
            for dep in deps {
                if is_batuta_crate(&dep.name) {
                    // Map crate name to repo name (crates often match repo names)
                    if let Some(&to_idx) = repo_index.get(dep.name.as_str()) {
                        if from_idx != to_idx {
                            edges.push((from_idx, to_idx, dep.name.clone()));
                        }
                    }
                }
            }
        }
    }

    StackGraph {
        repos: repos
            .iter()
            .map(|r| RepoInfo {
                name: r.name.clone(),
                path: r.path.clone(),
                exists: r.exists,
            })
            .collect(),
        edges,
    }
}

// ── Handler: stack status ────────────────────────────────────────────────

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn handle_stack_status(format: &OutputFormat) -> Result<()> {
    let manifest = load_stack_manifest()?;

    // Discover repos
    let repos: Vec<RepoInfo> = manifest
        .repos
        .iter()
        .map(|name| {
            let path = resolve_repo_path(&manifest.base_path, name);
            let exists = path.join("Cargo.toml").exists();
            RepoInfo {
                name: name.clone(),
                path,
                exists,
            }
        })
        .collect();

    let repos_found = repos.iter().filter(|r| r.exists).count();
    let repos_missing = repos.len() - repos_found;

    // Parse dependencies from each found repo
    let mut all_deps: BTreeMap<String, Vec<DepInfo>> = BTreeMap::new();
    let mut batuta_crate_names: std::collections::BTreeSet<String> =
        std::collections::BTreeSet::new();

    for repo in &repos {
        if !repo.exists {
            continue;
        }
        let cargo_toml = repo.path.join("Cargo.toml");
        match parse_cargo_dependencies(&cargo_toml) {
            Ok(deps) => {
                for dep in &deps {
                    if is_batuta_crate(&dep.name) && dep.source == DepSource::Registry {
                        batuta_crate_names.insert(dep.name.clone());
                    }
                }
                all_deps.insert(repo.name.clone(), deps);
            }
            Err(e) => {
                eprintln!(
                    "  {} Failed to parse {}: {}",
                    c::YELLOW,
                    cargo_toml.display(),
                    e
                );
            }
        }
    }

    // Check latest versions for batuta crates
    let mut latest_versions: BTreeMap<String, String> = BTreeMap::new();
    for crate_name in &batuta_crate_names {
        match check_latest_version(crate_name) {
            Ok(Some(version)) => {
                latest_versions.insert(crate_name.clone(), version);
            }
            Ok(None) => {
                // Crate not found on crates.io (might be unpublished)
            }
            Err(_) => {
                // cargo search failed, skip
            }
        }
    }

    // Detect mismatches
    let mut all_mismatches = Vec::new();
    let mut total_deps = 0;

    for repo in &repos {
        if let Some(deps) = all_deps.get(&repo.name) {
            let batuta_deps: Vec<&DepInfo> = deps
                .iter()
                .filter(|d| is_batuta_crate(&d.name) && d.source == DepSource::Registry)
                .collect();
            total_deps += batuta_deps.len();

            let mismatches = detect_mismatches(&repo.name, deps, &latest_versions);
            all_mismatches.extend(mismatches);
        }
    }

    let status = StackStatus {
        repos_found,
        repos_missing,
        total_deps,
        outdated_deps: all_mismatches.len(),
        mismatches: all_mismatches,
    };

    // Output.
    //
    // Six of the nine advertised `--format` values (markdown, csv, summary,
    // text, plain, junit) used to fall through a `_ =>` catch-all to the
    // coloured table, so `-f csv | column -s,` and `-f junit` handed a CI job
    // ANSI-decorated prose. Every variant is now spelled out — a new one is a
    // compile error here rather than another silent fall-through.
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&status)?);
        }
        OutputFormat::Yaml => {
            println!("{}", serde_yaml_ng::to_string(&status)?);
        }
        OutputFormat::Table => {
            print_status_table(&repos, &all_deps, &latest_versions, &status);
        }
        OutputFormat::Markdown
        | OutputFormat::Csv
        | OutputFormat::Summary
        | OutputFormat::Text
        | OutputFormat::Plain
        | OutputFormat::Junit => {
            print!(
                "{}",
                render_status_non_table(format, &repos, &all_deps, &latest_versions, &status)
            );
        }
    }

    Ok(())
}

fn print_status_table(
    repos: &[RepoInfo],
    all_deps: &BTreeMap<String, Vec<DepInfo>>,
    latest_versions: &BTreeMap<String, String>,
    status: &StackStatus,
) {
    println!();
    println!("{}", c::header("Stack Dependency Status"));
    println!("{}", c::rule());
    println!(
        "  Repos found: {}  Missing: {}",
        c::number(&status.repos_found.to_string()),
        if status.repos_missing > 0 {
            format!("{}{}{}", c::BOLD_RED, status.repos_missing, c::RESET)
        } else {
            c::dim(&status.repos_missing.to_string())
        }
    );
    println!(
        "  Batuta deps: {}  Outdated: {}",
        c::number(&status.total_deps.to_string()),
        if status.outdated_deps > 0 {
            format!("{}{}{}", c::BOLD_RED, status.outdated_deps, c::RESET)
        } else {
            c::pass(&status.outdated_deps.to_string())
        }
    );
    println!("{}", c::separator());

    // Per-repo breakdown
    for repo in repos {
        if !repo.exists {
            println!("  {} {}", c::dim(&repo.name), c::dim("(not found)"));
            continue;
        }

        let deps = match all_deps.get(&repo.name) {
            Some(d) => d,
            None => continue,
        };

        let batuta_deps: Vec<&DepInfo> = deps
            .iter()
            .filter(|d| is_batuta_crate(&d.name) && d.source == DepSource::Registry)
            .collect();

        if batuta_deps.is_empty() {
            println!("  {} {}", c::path(&repo.name), c::dim("(no batuta deps)"));
            continue;
        }

        println!("  {}", c::subheader(&repo.name));
        for dep in &batuta_deps {
            let current = dep.version.as_deref().unwrap_or("?");
            let latest = latest_versions
                .get(&dep.name)
                .map(|s| s.as_str())
                .unwrap_or("?");

            let is_outdated = latest != "?" && requirement_is_outdated(current, latest);

            if is_outdated {
                println!(
                    "    {} {} {} -> {latest}{}",
                    c::fail(&dep.name),
                    c::dim(current),
                    c::BOLD_GREEN,
                    c::RESET,
                );
            } else {
                println!("    {} {}", c::pass(&dep.name), c::dim(current),);
            }
        }
    }

    println!("{}", c::rule());
    println!();
}

// ── Handler: stack status — non-table renderers ──────────────────────────

/// Render `stack status` in one of the six formats that used to fall through to
/// the coloured table. `Table`/`Json`/`Yaml` are handled by the caller.
fn render_status_non_table(
    format: &OutputFormat,
    repos: &[RepoInfo],
    all_deps: &BTreeMap<String, Vec<DepInfo>>,
    latest_versions: &BTreeMap<String, String>,
    status: &StackStatus,
) -> String {
    match format {
        OutputFormat::Markdown => render_status_markdown(repos, all_deps, latest_versions, status),
        OutputFormat::Csv => render_status_csv(repos, all_deps, latest_versions),
        OutputFormat::Summary => format!("{}\n", render_status_summary(status)),
        OutputFormat::Junit => render_status_junit(repos, all_deps, latest_versions, status),
        // text/plain, plus the three the caller already handled — reaching here
        // with those would be a caller bug, and plain text is the honest answer.
        _ => render_status_text(repos, all_deps, latest_versions, status),
    }
}

/// One line of the status report: either a batuta dependency of a repo, or the
/// state of a repo that contributes no dependency row (missing / no deps).
struct StatusRow {
    repo: String,
    crate_name: String,
    current: String,
    latest: String,
    state: &'static str,
}

/// Flatten the discovered repos into report rows, in the order the table prints
/// them, so every format describes the same run.
fn status_rows(
    repos: &[RepoInfo],
    all_deps: &BTreeMap<String, Vec<DepInfo>>,
    latest_versions: &BTreeMap<String, String>,
) -> Vec<StatusRow> {
    let mut rows = Vec::new();

    for repo in repos {
        if !repo.exists {
            rows.push(StatusRow {
                repo: repo.name.clone(),
                crate_name: String::new(),
                current: String::new(),
                latest: String::new(),
                state: "missing",
            });
            continue;
        }

        let Some(deps) = all_deps.get(&repo.name) else {
            continue;
        };

        let batuta_deps: Vec<&DepInfo> = deps
            .iter()
            .filter(|d| is_batuta_crate(&d.name) && d.source == DepSource::Registry)
            .collect();

        if batuta_deps.is_empty() {
            rows.push(StatusRow {
                repo: repo.name.clone(),
                crate_name: String::new(),
                current: String::new(),
                latest: String::new(),
                state: "no-batuta-deps",
            });
            continue;
        }

        for dep in batuta_deps {
            let current = dep.version.as_deref().unwrap_or("?").to_string();
            let latest = latest_versions
                .get(&dep.name)
                .cloned()
                .unwrap_or_else(|| "?".to_string());
            // "?" is "crates.io was not reached / crate unpublished", which is
            // not the same as up to date and must not be reported as either.
            let state = if latest == "?" {
                "unknown"
            } else if requirement_is_outdated(&current, &latest) {
                "outdated"
            } else {
                "current"
            };

            rows.push(StatusRow {
                repo: repo.name.clone(),
                crate_name: dep.name.clone(),
                current,
                latest,
                state,
            });
        }
    }

    rows
}

fn render_status_summary(status: &StackStatus) -> String {
    format!(
        "repos_found={} repos_missing={} batuta_deps={} outdated={}",
        status.repos_found, status.repos_missing, status.total_deps, status.outdated_deps
    )
}

fn render_status_csv(
    repos: &[RepoInfo],
    all_deps: &BTreeMap<String, Vec<DepInfo>>,
    latest_versions: &BTreeMap<String, String>,
) -> String {
    let mut out = String::from("repo,crate,current,latest,state\n");
    for row in status_rows(repos, all_deps, latest_versions) {
        out.push_str(&format!(
            "{},{},{},{},{}\n",
            csv_field(&row.repo),
            csv_field(&row.crate_name),
            csv_field(&row.current),
            csv_field(&row.latest),
            row.state
        ));
    }
    out
}

/// Quote a CSV field that carries a separator, quote or newline (RFC 4180).
fn csv_field(value: &str) -> String {
    if value.contains([',', '"', '\n']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn render_status_markdown(
    repos: &[RepoInfo],
    all_deps: &BTreeMap<String, Vec<DepInfo>>,
    latest_versions: &BTreeMap<String, String>,
    status: &StackStatus,
) -> String {
    let mut out = String::from("# Stack Dependency Status\n\n");
    out.push_str(&format!(
        "- Repos found: {}\n- Repos missing: {}\n- Batuta deps: {}\n- Outdated: {}\n\n",
        status.repos_found, status.repos_missing, status.total_deps, status.outdated_deps
    ));
    out.push_str("| Repo | Crate | Current | Latest | State |\n");
    out.push_str("|------|-------|---------|--------|-------|\n");
    for row in status_rows(repos, all_deps, latest_versions) {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            row.repo,
            if row.crate_name.is_empty() {
                "-"
            } else {
                &row.crate_name
            },
            if row.current.is_empty() {
                "-"
            } else {
                &row.current
            },
            if row.latest.is_empty() {
                "-"
            } else {
                &row.latest
            },
            row.state
        ));
    }
    out
}

/// `text`/`plain`: the table's content with no ANSI, for pipes and logs.
fn render_status_text(
    repos: &[RepoInfo],
    all_deps: &BTreeMap<String, Vec<DepInfo>>,
    latest_versions: &BTreeMap<String, String>,
    status: &StackStatus,
) -> String {
    let mut out = String::from("Stack Dependency Status\n");
    out.push_str(&format!(
        "  Repos found: {}  Missing: {}\n  Batuta deps: {}  Outdated: {}\n",
        status.repos_found, status.repos_missing, status.total_deps, status.outdated_deps
    ));

    let mut current_repo = String::new();
    for row in status_rows(repos, all_deps, latest_versions) {
        if row.repo != current_repo {
            out.push_str(&format!("  {}\n", row.repo));
            current_repo = row.repo.clone();
        }
        match row.state {
            "missing" => out.push_str("    (not found)\n"),
            "no-batuta-deps" => out.push_str("    (no batuta deps)\n"),
            "outdated" => out.push_str(&format!(
                "    {} {} -> {}\n",
                row.crate_name, row.current, row.latest
            )),
            _ => out.push_str(&format!(
                "    {} {} ({})\n",
                row.crate_name, row.current, row.state
            )),
        }
    }

    out
}

fn render_status_junit(
    repos: &[RepoInfo],
    all_deps: &BTreeMap<String, Vec<DepInfo>>,
    latest_versions: &BTreeMap<String, String>,
    status: &StackStatus,
) -> String {
    let rows = status_rows(repos, all_deps, latest_versions);
    let failures = rows
        .iter()
        .filter(|r| r.state == "outdated" || r.state == "missing")
        .count();
    let skipped = rows.iter().filter(|r| r.state == "unknown").count();

    let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str(&format!(
        "<testsuite name=\"pmat stack status\" tests=\"{}\" failures=\"{}\" errors=\"0\" skipped=\"{}\">\n",
        rows.len(),
        failures,
        skipped
    ));

    for row in &rows {
        let name = if row.crate_name.is_empty() {
            row.repo.clone()
        } else {
            format!("{}::{}", row.repo, row.crate_name)
        };
        out.push_str(&format!(
            "  <testcase classname=\"stack-status\" name=\"{}\"",
            xml_escape(&name)
        ));
        match row.state {
            "outdated" => out.push_str(&format!(
                ">\n    <failure message=\"{} is {}, latest is {}\"/>\n  </testcase>\n",
                xml_escape(&row.crate_name),
                xml_escape(&row.current),
                xml_escape(&row.latest)
            )),
            "missing" => {
                out.push_str(">\n    <failure message=\"repo not found\"/>\n  </testcase>\n")
            }
            // Not measured is not a pass: crates.io was never reached for this
            // crate, so the case is skipped rather than reported green.
            "unknown" => out
                .push_str(">\n    <skipped message=\"latest version unknown\"/>\n  </testcase>\n"),
            _ => out.push_str("/>\n"),
        }
    }

    out.push_str(&format!(
        "</testsuite>\n<!-- outdated_deps={} -->\n",
        status.outdated_deps
    ));
    out
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// ── Handler: stack sync ──────────────────────────────────────────────────

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn handle_stack_sync(apply: bool, dry_run: bool) -> Result<()> {
    let manifest = load_stack_manifest()?;

    // Discover repos
    let repos: Vec<RepoInfo> = manifest
        .repos
        .iter()
        .map(|name| {
            let path = resolve_repo_path(&manifest.base_path, name);
            let exists = path.join("Cargo.toml").exists();
            RepoInfo {
                name: name.clone(),
                path,
                exists,
            }
        })
        .collect();

    // Parse dependencies from each found repo
    let mut all_deps: BTreeMap<String, Vec<DepInfo>> = BTreeMap::new();
    let mut batuta_crate_names: std::collections::BTreeSet<String> =
        std::collections::BTreeSet::new();

    for repo in &repos {
        if !repo.exists {
            continue;
        }
        let cargo_toml = repo.path.join("Cargo.toml");
        match parse_cargo_dependencies(&cargo_toml) {
            Ok(deps) => {
                for dep in &deps {
                    if is_batuta_crate(&dep.name) && dep.source == DepSource::Registry {
                        batuta_crate_names.insert(dep.name.clone());
                    }
                }
                all_deps.insert(repo.name.clone(), deps);
            }
            Err(e) => {
                eprintln!(
                    "  {} Failed to parse {}: {}",
                    c::YELLOW,
                    cargo_toml.display(),
                    e
                );
            }
        }
    }

    // Check latest versions
    let mut latest_versions: BTreeMap<String, String> = BTreeMap::new();
    for crate_name in &batuta_crate_names {
        match check_latest_version(crate_name) {
            Ok(Some(version)) => {
                latest_versions.insert(crate_name.clone(), version);
            }
            Ok(None) => {}
            Err(_) => {}
        }
    }

    // Build dependency graph
    let graph = build_stack_graph(&repos, &all_deps);

    // Detect all mismatches
    let mut all_mismatches = Vec::new();
    for repo in &repos {
        if let Some(deps) = all_deps.get(&repo.name) {
            let mismatches = detect_mismatches(&repo.name, deps, &latest_versions);
            all_mismatches.extend(mismatches);
        }
    }

    if all_mismatches.is_empty() {
        println!();
        println!(
            "{}",
            c::pass("All batuta stack dependencies are up to date")
        );
        println!();
        return Ok(());
    }

    // Print the sync plan
    println!();
    println!("{}", c::header("Stack Sync Plan"));
    println!("{}", c::rule());

    // Show dependency graph summary
    if !graph.edges.is_empty() {
        println!("  {}", c::subheader("Dependency Graph:"));
        for (from, to, crate_name) in &graph.edges {
            let from_name = &graph.repos[*from].name;
            let to_name = &graph.repos[*to].name;
            println!(
                "    {} -> {} ({})",
                c::path(from_name),
                c::path(to_name),
                c::dim(crate_name),
            );
        }
        println!("{}", c::separator());
    }

    // Show planned changes
    println!("  {}", c::subheader("Changes:"));
    for mismatch in &all_mismatches {
        println!(
            "    {} {} {} -> {}{}{}",
            c::path(&mismatch.repo),
            mismatch.crate_name,
            c::dim(&mismatch.current_version),
            c::BOLD_GREEN,
            mismatch.latest_version,
            c::RESET,
        );
    }
    println!("{}", c::separator());

    let is_dry_run = dry_run || !apply;

    if is_dry_run {
        println!(
            "  {} Run with {} to apply changes",
            c::dim("Dry run."),
            c::subheader("--apply"),
        );
        println!();
        return Ok(());
    }

    // Apply changes
    println!("  {}", c::subheader("Applying changes..."));

    // Group mismatches by repo
    let mut by_repo: BTreeMap<String, Vec<&DepMismatch>> = BTreeMap::new();
    for mismatch in &all_mismatches {
        by_repo
            .entry(mismatch.repo.clone())
            .or_default()
            .push(mismatch);
    }

    for (repo_name, mismatches) in &by_repo {
        let repo_info = repos.iter().find(|r| r.name == *repo_name);
        let repo_path = match repo_info {
            Some(r) => &r.path,
            None => continue,
        };

        let cargo_toml_path = repo_path.join("Cargo.toml");
        let mut content = std::fs::read_to_string(&cargo_toml_path)
            .with_context(|| format!("Failed to read {}", cargo_toml_path.display()))?;

        for mismatch in mismatches {
            // Simple text replacement for version strings
            // Handle both `crate = "version"` and `crate = { version = "version", ... }`
            let old_patterns = [
                format!("{} = \"{}\"", mismatch.crate_name, mismatch.current_version),
                format!("version = \"{}\"", mismatch.current_version),
            ];

            let new_version_simple =
                format!("{} = \"{}\"", mismatch.crate_name, mismatch.latest_version);
            let new_version_table = format!("version = \"{}\"", mismatch.latest_version);

            // Try simple form first
            if content.contains(&old_patterns[0]) {
                content = content.replace(&old_patterns[0], &new_version_simple);
                println!(
                    "    {} Updated {} in {}",
                    c::pass(&mismatch.crate_name),
                    mismatch.latest_version,
                    repo_name,
                );
            } else {
                // For table form, we need to be more careful: only replace within the dep's context
                // This is a best-effort approach - complex Cargo.toml edits should use cargo-edit
                content = content.replacen(&old_patterns[1], &new_version_table, 1);
                println!(
                    "    {} Updated {} in {} (table form)",
                    c::pass(&mismatch.crate_name),
                    mismatch.latest_version,
                    repo_name,
                );
            }
        }

        std::fs::write(&cargo_toml_path, &content)
            .with_context(|| format!("Failed to write {}", cargo_toml_path.display()))?;

        // Run cargo update in the repo
        println!(
            "    {} Running cargo update in {}...",
            c::dim(">>>"),
            repo_name
        );
        let update_result = std::process::Command::new("cargo")
            .arg("update")
            .current_dir(repo_path)
            .status();

        match update_result {
            Ok(status) if status.success() => {
                println!(
                    "    {}",
                    c::pass(&format!("cargo update succeeded in {repo_name}"))
                );
            }
            Ok(status) => {
                eprintln!(
                    "    {}",
                    c::fail(&format!(
                        "cargo update failed in {repo_name} (exit code: {})",
                        status.code().unwrap_or(-1)
                    ))
                );
            }
            Err(e) => {
                eprintln!(
                    "    {}",
                    c::fail(&format!("cargo update failed in {repo_name}: {e}"))
                );
            }
        }
    }

    println!("{}", c::rule());
    println!(
        "  {}",
        c::pass(&format!(
            "Applied {} updates across {} repos",
            all_mismatches.len(),
            by_repo.len()
        ))
    );
    println!();

    Ok(())
}

// ── Unit Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cargo_dependencies_simple() {
        let content = r#"
[package]
name = "my-project"
version = "0.1.0"

[dependencies]
trueno = "0.4.30"
serde = { version = "1.0", features = ["derive"] }
aprender = { version = "0.3.5", default-features = false }
local-crate = { path = "../local-crate" }
git-crate = { git = "https://github.com/example/git-crate" }

[dev-dependencies]
proptest = "1.0"
"#;

        let deps = parse_cargo_toml_content(content).unwrap();

        // Should have 6 deps total (5 deps + 1 dev-dep)
        assert_eq!(deps.len(), 6);

        // Check trueno
        let trueno = deps.iter().find(|d| d.name == "trueno").unwrap();
        assert_eq!(trueno.version.as_deref(), Some("0.4.30"));
        assert_eq!(trueno.source, DepSource::Registry);
        assert_eq!(trueno.section, "dependencies");

        // Check serde (table form)
        let serde = deps.iter().find(|d| d.name == "serde").unwrap();
        assert_eq!(serde.version.as_deref(), Some("1.0"));
        assert_eq!(serde.source, DepSource::Registry);

        // Check path dep
        let local = deps.iter().find(|d| d.name == "local-crate").unwrap();
        assert!(matches!(local.source, DepSource::Path(_)));

        // Check git dep
        let git = deps.iter().find(|d| d.name == "git-crate").unwrap();
        assert!(matches!(git.source, DepSource::Git(_)));

        // Check dev-dep
        let proptest = deps.iter().find(|d| d.name == "proptest").unwrap();
        assert_eq!(proptest.section, "dev-dependencies");
    }

    #[test]
    fn test_default_stack_repos() {
        assert!(DEFAULT_STACK_REPOS.contains(&"trueno"));
        assert!(DEFAULT_STACK_REPOS.contains(&"aprender"));
        assert!(DEFAULT_STACK_REPOS.contains(&"paiml-mcp-agent-toolkit"));
        assert!(DEFAULT_STACK_REPOS.contains(&"certeza"));
        assert_eq!(DEFAULT_STACK_REPOS.len(), 14);
    }

    #[test]
    fn test_resolve_repo_path() {
        let base = PathBuf::from("/home/user/src");
        let path = resolve_repo_path(&base, "trueno");
        assert_eq!(path, PathBuf::from("/home/user/src/trueno"));
    }

    #[test]
    fn test_parse_search_output() {
        let output = r#"trueno = "0.4.30"    # SIMD accelerated tensor operations
"#;
        let version = parse_cargo_search_output(output, "trueno").unwrap();
        assert_eq!(version, Some("0.4.30".to_string()));
    }

    #[test]
    fn test_parse_search_output_no_match() {
        let output = "other-crate = \"1.0.0\"    # Something else\n";
        let version = parse_cargo_search_output(output, "trueno").unwrap();
        assert_eq!(version, None);
    }

    #[test]
    fn test_parse_search_output_empty() {
        let version = parse_cargo_search_output("", "trueno").unwrap();
        assert_eq!(version, None);
    }

    #[test]
    fn test_detect_mismatches() {
        let deps = vec![
            DepInfo {
                name: "trueno".to_string(),
                version: Some("0.4.22".to_string()),
                source: DepSource::Registry,
                section: "dependencies".to_string(),
            },
            DepInfo {
                name: "serde".to_string(),
                version: Some("1.0".to_string()),
                source: DepSource::Registry,
                section: "dependencies".to_string(),
            },
            DepInfo {
                name: "aprender".to_string(),
                version: Some("0.3.5".to_string()),
                source: DepSource::Registry,
                section: "dependencies".to_string(),
            },
        ];

        let mut latest = BTreeMap::new();
        latest.insert("trueno".to_string(), "0.4.30".to_string());
        latest.insert("aprender".to_string(), "0.3.5".to_string());

        let mismatches = detect_mismatches("my-repo", &deps, &latest);

        // Only trueno should be outdated; serde is not batuta; aprender matches
        assert_eq!(mismatches.len(), 1);
        assert_eq!(mismatches[0].crate_name, "trueno");
        assert_eq!(mismatches[0].current_version, "0.4.22");
        assert_eq!(mismatches[0].latest_version, "0.4.30");
    }

    #[test]
    fn test_detect_mismatches_with_caret() {
        let deps = vec![DepInfo {
            name: "trueno".to_string(),
            version: Some("^0.4.30".to_string()),
            source: DepSource::Registry,
            section: "dependencies".to_string(),
        }];

        let mut latest = BTreeMap::new();
        latest.insert("trueno".to_string(), "0.4.30".to_string());

        let mismatches = detect_mismatches("my-repo", &deps, &latest);
        // ^0.4.30 should match 0.4.30 (caret stripped)
        assert_eq!(mismatches.len(), 0);
    }

    /// `pmcp = "2.17"` is satisfied exactly by 2.17.0, but the freshness check
    /// was a string inequality, so it printed `✗ pmcp 2.17 -> 2.17.0` and the
    /// outdated count came back equal to the dep count (23 of 23).
    #[test]
    fn test_detect_mismatches_two_component_requirement_is_not_outdated() {
        let deps = vec![DepInfo {
            name: "pmcp".to_string(),
            version: Some("2.17".to_string()),
            source: DepSource::Registry,
            section: "dependencies".to_string(),
        }];

        let mut latest = BTreeMap::new();
        latest.insert("pmcp".to_string(), "2.17.0".to_string());

        let mismatches = detect_mismatches("my-repo", &deps, &latest);
        assert!(
            mismatches.is_empty(),
            "2.17 is 2.17.0, yet it was reported as {:?}",
            mismatches
                .iter()
                .map(|m| format!("{} -> {}", m.current_version, m.latest_version))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_requirement_floor_pads_missing_components() {
        assert_eq!(
            requirement_floor("2.17"),
            Some(semver::Version::new(2, 17, 0))
        );
        assert_eq!(requirement_floor("1"), Some(semver::Version::new(1, 0, 0)));
        assert_eq!(
            requirement_floor("^0.4.22"),
            Some(semver::Version::new(0, 4, 22))
        );
        assert_eq!(requirement_floor("*"), None);
    }

    #[test]
    fn test_requirement_is_outdated_still_flags_a_real_lag() {
        // A genuinely older floor is still reported, so `stack sync` keeps working.
        assert!(requirement_is_outdated("0.61", "0.63.0"));
        assert!(requirement_is_outdated("^0.4.22", "0.4.30"));
        assert!(!requirement_is_outdated("0.4.30", "0.4.30"));
    }

    #[test]
    fn test_detect_mismatches_path_dep_skipped() {
        let deps = vec![DepInfo {
            name: "trueno".to_string(),
            version: Some("0.4.22".to_string()),
            source: DepSource::Path("../trueno".to_string()),
            section: "dependencies".to_string(),
        }];

        let mut latest = BTreeMap::new();
        latest.insert("trueno".to_string(), "0.4.30".to_string());

        let mismatches = detect_mismatches("my-repo", &deps, &latest);
        // Path deps should be skipped
        assert_eq!(mismatches.len(), 0);
    }

    #[test]
    fn test_is_batuta_crate() {
        assert!(is_batuta_crate("trueno"));
        assert!(is_batuta_crate("aprender"));
        assert!(is_batuta_crate("trueno-graph"));
        assert!(is_batuta_crate("certeza"));
        assert!(is_batuta_crate("presentar-core"));
        assert!(!is_batuta_crate("serde"));
        assert!(!is_batuta_crate("tokio"));
        assert!(!is_batuta_crate("rand"));
    }

    #[test]
    fn test_parse_cargo_dependencies_workspace() {
        let content = r#"
[workspace]
members = ["crates/*"]

[workspace.dependencies]
trueno = "0.4.30"
serde = "1.0"
"#;

        let deps = parse_cargo_toml_content(content).unwrap();
        assert_eq!(deps.len(), 2);

        let trueno = deps.iter().find(|d| d.name == "trueno").unwrap();
        assert_eq!(trueno.section, "workspace.dependencies");
    }

    #[test]
    fn test_build_stack_graph() {
        let repos = vec![
            RepoInfo {
                name: "aprender".to_string(),
                path: PathBuf::from("/tmp/aprender"),
                exists: true,
            },
            RepoInfo {
                name: "trueno".to_string(),
                path: PathBuf::from("/tmp/trueno"),
                exists: true,
            },
        ];

        let mut all_deps = BTreeMap::new();
        all_deps.insert(
            "aprender".to_string(),
            vec![DepInfo {
                name: "trueno".to_string(),
                version: Some("0.4.30".to_string()),
                source: DepSource::Registry,
                section: "dependencies".to_string(),
            }],
        );
        all_deps.insert("trueno".to_string(), vec![]);

        let graph = build_stack_graph(&repos, &all_deps);
        assert_eq!(graph.repos.len(), 2);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0], (0, 1, "trueno".to_string())); // aprender -> trueno
    }

    #[test]
    fn test_dep_source_equality() {
        assert_eq!(DepSource::Registry, DepSource::Registry);
        assert_ne!(DepSource::Registry, DepSource::Path("/foo".to_string()));
        assert_ne!(
            DepSource::Path("/a".to_string()),
            DepSource::Git("https://x".to_string())
        );
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod status_format_tests {
    use super::*;

    /// (repos, deps by repo, latest version by crate, overall status)
    type StatusFixture = (
        Vec<RepoInfo>,
        BTreeMap<String, Vec<DepInfo>>,
        BTreeMap<String, String>,
        StackStatus,
    );

    fn fixture() -> StatusFixture {
        let repos = vec![
            RepoInfo {
                name: "aprender".to_string(),
                path: PathBuf::from("/tmp/aprender"),
                exists: true,
            },
            RepoInfo {
                name: "ghost".to_string(),
                path: PathBuf::from("/tmp/ghost"),
                exists: false,
            },
        ];

        let mut all_deps = BTreeMap::new();
        all_deps.insert(
            "aprender".to_string(),
            vec![DepInfo {
                name: "trueno".to_string(),
                version: Some("0.4.30".to_string()),
                source: DepSource::Registry,
                section: "dependencies".to_string(),
            }],
        );

        let mut latest = BTreeMap::new();
        latest.insert("trueno".to_string(), "0.5.0".to_string());

        let status = StackStatus {
            repos_found: 1,
            repos_missing: 1,
            total_deps: 1,
            outdated_deps: 1,
            mismatches: vec![],
        };

        (repos, all_deps, latest, status)
    }

    /// markdown/csv/summary/text/plain/junit used to fall through a `_ =>`
    /// catch-all to the coloured table, so six of the nine advertised formats
    /// emitted identical ANSI output.
    #[test]
    fn every_format_renders_its_own_shape() {
        let (repos, all_deps, latest, status) = fixture();

        let render = |format: OutputFormat| {
            render_status_non_table(&format, &repos, &all_deps, &latest, &status)
        };
        let markdown = render(OutputFormat::Markdown);
        let csv = render(OutputFormat::Csv);
        let summary = render(OutputFormat::Summary);
        let text = render(OutputFormat::Text);
        let junit = render(OutputFormat::Junit);

        assert_eq!(
            text,
            render(OutputFormat::Plain),
            "text and plain are the same report"
        );

        let rendered = [&markdown, &csv, &summary, &text, &junit];
        for (i, a) in rendered.iter().enumerate() {
            for b in rendered.iter().skip(i + 1) {
                assert_ne!(a, b, "two formats produced the same bytes");
            }
            assert!(
                !a.contains('\u{1b}'),
                "machine-readable format must not carry ANSI escapes: {a}"
            );
        }

        assert!(markdown.starts_with("# Stack Dependency Status"));
        assert!(markdown.contains("| aprender | trueno | 0.4.30 | 0.5.0 | outdated |"));

        assert!(csv.starts_with("repo,crate,current,latest,state\n"));
        assert!(csv.contains("aprender,trueno,0.4.30,0.5.0,outdated\n"));
        assert!(csv.contains("ghost,,,,missing\n"));

        assert_eq!(
            summary,
            "repos_found=1 repos_missing=1 batuta_deps=1 outdated=1\n"
        );

        assert!(text.contains("trueno 0.4.30 -> 0.5.0"));
        assert!(text.contains("(not found)"));

        assert!(junit.starts_with("<?xml version=\"1.0\""));
        assert!(junit.contains("<testsuite name=\"pmat stack status\" tests=\"2\" failures=\"2\""));
        assert!(junit.contains("name=\"aprender::trueno\""));
        assert!(junit.contains("<failure message=\"repo not found\"/>"));
    }

    /// A crate whose latest version was never fetched is not "up to date"; the
    /// JUnit case is skipped rather than reported as a pass.
    #[test]
    fn unknown_latest_is_not_a_pass() {
        let (repos, all_deps, _latest, status) = fixture();
        let empty = BTreeMap::new();

        let junit = render_status_junit(&repos, &all_deps, &empty, &status);
        assert!(junit.contains("<skipped message=\"latest version unknown\"/>"));
        assert!(junit.contains("skipped=\"1\""));

        let csv = render_status_csv(&repos, &all_deps, &empty);
        assert!(csv.contains("aprender,trueno,0.4.30,?,unknown\n"));
    }
}
