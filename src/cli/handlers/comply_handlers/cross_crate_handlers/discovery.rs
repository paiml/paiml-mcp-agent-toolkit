#![cfg_attr(coverage_nightly, coverage(off))]

use super::types::CrateInfo;
use crate::services::agent_context::parse_workspace_siblings;
use serde_json;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

// --- Workspace discovery ---

/// Discover crates to analyze with priority chain:
/// 1. Explicit `--crates` paths
/// 2. Cargo.toml `[workspace]` members
/// 3. `batuta oracle --local` (batuta stack auto-discovery)
/// 4. `.pmat/workspace.toml` siblings (legacy)
/// 5. Single-crate fallback
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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
        eprintln!(
            "  Discovery: found Cargo workspace with {} members",
            workspace_crates.len()
        );
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
            eprintln!(
                "  Warning: {} has no Cargo.toml, skipping",
                resolved.display()
            );
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
pub(super) fn parse_workspace_members_with_globs(content: &str, base: &Path) -> Vec<PathBuf> {
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
                if let Some((_, after_eq)) = trimmed.split_once('=') {
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
    parsed.get("projects").and_then(|p| p.as_object().cloned())
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
pub(super) fn make_crate_info(crate_path: &Path) -> CrateInfo {
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
pub(super) fn read_crate_name(cargo_toml: &Path) -> Option<String> {
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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
