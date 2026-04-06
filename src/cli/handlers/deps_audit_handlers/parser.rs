#![cfg_attr(coverage_nightly, coverage(off))]

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use super::types::DepEdge;

/// Parse Cargo.toml and extract dependencies
#[allow(clippy::type_complexity)]
pub fn parse_cargo_toml(
    path: &Path,
) -> Result<(Vec<(String, String, bool)>, Vec<(String, String, bool)>)> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let content =
        fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;

    let toml: toml::Value =
        toml::from_str(&content).with_context(|| "Failed to parse Cargo.toml")?;

    let mut deps = Vec::new();
    let mut dev_deps = Vec::new();

    // Regular dependencies
    if let Some(dependencies) = toml.get("dependencies").and_then(|d| d.as_table()) {
        for (name, value) in dependencies {
            let version = match value {
                toml::Value::String(v) => v.clone(),
                toml::Value::Table(t) => t
                    .get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("*")
                    .to_string(),
                _ => "*".to_string(),
            };
            deps.push((name.clone(), version, false));
        }
    }

    // Dev dependencies
    if let Some(dependencies) = toml.get("dev-dependencies").and_then(|d| d.as_table()) {
        for (name, value) in dependencies {
            let version = match value {
                toml::Value::String(v) => v.clone(),
                toml::Value::Table(t) => t
                    .get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("*")
                    .to_string(),
                _ => "*".to_string(),
            };
            dev_deps.push((name.clone(), version, true));
        }
    }

    Ok((deps, dev_deps))
}

/// Parse Cargo.lock and extract dependency graph
/// Looks in path, parent, and grandparent (for workspace roots)
pub fn parse_cargo_lock(path: &Path) -> Result<(Vec<String>, Vec<DepEdge>)> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    // Try path, parent, grandparent (workspace root)
    let candidates = [
        path.join("Cargo.lock"),
        path.parent()
            .map(|p| p.join("Cargo.lock"))
            .unwrap_or_default(),
        path.parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("Cargo.lock"))
            .unwrap_or_default(),
    ];

    let lock_path = candidates.iter().find(|p| p.exists());
    let Some(lock_path) = lock_path else {
        return Ok((Vec::new(), Vec::new()));
    };

    let content = fs::read_to_string(lock_path)
        .with_context(|| format!("Failed to read {}", lock_path.display()))?;

    let toml: toml::Value =
        toml::from_str(&content).with_context(|| "Failed to parse Cargo.lock")?;

    let mut all_packages = Vec::new();
    let mut edges = Vec::new();

    if let Some(packages) = toml.get("package").and_then(|p| p.as_array()) {
        for pkg in packages {
            if let Some(name) = pkg.get("name").and_then(|n| n.as_str()) {
                all_packages.push(name.to_string());

                // Extract dependencies for this package
                if let Some(deps) = pkg.get("dependencies").and_then(|d| d.as_array()) {
                    for dep in deps {
                        if let Some(dep_str) = dep.as_str() {
                            // Format: "name version" or "name version (source)"
                            let dep_name = dep_str.split_whitespace().next().unwrap_or(dep_str);
                            edges.push(DepEdge {
                                from: name.to_string(),
                                to: dep_name.to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    Ok((all_packages, edges))
}
