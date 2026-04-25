#![cfg_attr(coverage_nightly, coverage(off))]

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use super::types::DepEdge;

/// Parse Cargo.toml and extract dependencies
#[allow(clippy::type_complexity)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn parse_cargo_toml(
    path: &Path,
) -> Result<(Vec<(String, String, bool)>, Vec<(String, String, bool)>)> {
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn parse_cargo_lock(path: &Path) -> Result<(Vec<String>, Vec<DepEdge>)> {
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

#[cfg(test)]
mod parser_tests {
    //! Covers parse_cargo_toml + parse_cargo_lock in
    //! deps_audit_handlers/parser.rs (13 uncov on broad, 0% cov).
    use super::*;

    // ── parse_cargo_toml ──

    #[test]
    fn test_parse_cargo_toml_missing_file_returns_err() {
        let missing = std::path::Path::new("/tmp/pmat_missing_cargo_xyz.toml");
        assert!(parse_cargo_toml(missing).is_err());
    }

    #[test]
    fn test_parse_cargo_toml_invalid_toml_returns_err() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), "not = [valid toml").unwrap();
        assert!(parse_cargo_toml(tmp.path()).is_err());
    }

    #[test]
    fn test_parse_cargo_toml_empty_file_returns_empty_lists() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), "[package]\nname = \"x\"\n").unwrap();
        let (deps, dev_deps) = parse_cargo_toml(tmp.path()).unwrap();
        assert!(deps.is_empty());
        assert!(dev_deps.is_empty());
    }

    #[test]
    fn test_parse_cargo_toml_string_dep_version() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(
            tmp.path(),
            "[dependencies]\nserde = \"1.0\"\n[dev-dependencies]\nproptest = \"1.5\"\n",
        )
        .unwrap();
        let (deps, dev_deps) = parse_cargo_toml(tmp.path()).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0], ("serde".to_string(), "1.0".to_string(), false));
        assert_eq!(dev_deps.len(), 1);
        assert_eq!(
            dev_deps[0],
            ("proptest".to_string(), "1.5".to_string(), true)
        );
    }

    #[test]
    fn test_parse_cargo_toml_table_dep_with_version() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(
            tmp.path(),
            "[dependencies.serde]\nversion = \"1.0\"\nfeatures = [\"derive\"]\n",
        )
        .unwrap();
        let (deps, _) = parse_cargo_toml(tmp.path()).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].0, "serde");
        assert_eq!(deps[0].1, "1.0");
    }

    #[test]
    fn test_parse_cargo_toml_table_dep_without_version_defaults_to_star() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(
            tmp.path(),
            "[dependencies.local-thing]\npath = \"../local\"\n",
        )
        .unwrap();
        let (deps, _) = parse_cargo_toml(tmp.path()).unwrap();
        assert_eq!(deps[0].1, "*");
    }

    // ── parse_cargo_lock ──

    #[test]
    fn test_parse_cargo_lock_no_lock_returns_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let (packages, edges) = parse_cargo_lock(tmp.path()).unwrap();
        assert!(packages.is_empty());
        assert!(edges.is_empty());
    }

    #[test]
    fn test_parse_cargo_lock_finds_in_parent_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let sub = tmp.path().join("subcrate");
        std::fs::create_dir(&sub).unwrap();
        // Create Cargo.lock in parent.
        std::fs::write(
            tmp.path().join("Cargo.lock"),
            "[[package]]\nname = \"x\"\nversion = \"1.0\"\n",
        )
        .unwrap();
        let (packages, _) = parse_cargo_lock(&sub).unwrap();
        assert_eq!(packages, vec!["x".to_string()]);
    }

    #[test]
    fn test_parse_cargo_lock_extracts_packages_and_edges() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("Cargo.lock"),
            "[[package]]\nname = \"foo\"\nversion = \"1.0\"\ndependencies = [\n \"bar 2.0\",\n \"baz 0.5 (registry+https://github.com/rust-lang/crates.io-index)\",\n]\n[[package]]\nname = \"bar\"\nversion = \"2.0\"\n",
        )
        .unwrap();
        let (packages, edges) = parse_cargo_lock(tmp.path()).unwrap();
        assert_eq!(packages.len(), 2);
        assert!(packages.contains(&"foo".to_string()));
        assert!(packages.contains(&"bar".to_string()));
        // 2 edges from foo (bar + baz, name only).
        assert_eq!(edges.len(), 2);
        assert!(edges.iter().any(|e| e.from == "foo" && e.to == "bar"));
        assert!(edges.iter().any(|e| e.from == "foo" && e.to == "baz"));
    }

    #[test]
    fn test_parse_cargo_lock_invalid_toml_returns_err() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("Cargo.lock"), "garbage [not toml").unwrap();
        assert!(parse_cargo_lock(tmp.path()).is_err());
    }

    #[test]
    fn test_parse_cargo_lock_no_package_section_returns_empty() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("Cargo.lock"), "version = 3\n").unwrap();
        let (packages, edges) = parse_cargo_lock(tmp.path()).unwrap();
        assert!(packages.is_empty());
        assert!(edges.is_empty());
    }
}
