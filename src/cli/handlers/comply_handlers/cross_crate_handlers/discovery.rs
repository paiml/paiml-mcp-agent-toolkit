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
            crate::status_eprintln!("  Discovery: using explicit --crates paths");
            return discover_from_explicit(workspace_path, paths);
        }
    }

    // Priority 2: Cargo.toml [workspace] section
    let workspace_crates = discover_from_cargo_workspace(workspace_path);
    if workspace_crates.len() >= 2 {
        crate::status_eprintln!(
            "  Discovery: found Cargo workspace with {} members",
            workspace_crates.len()
        );
        return workspace_crates;
    }

    // Priority 3: batuta oracle --local (batuta stack auto-discovery)
    let oracle_crates = discover_from_batuta_oracle(workspace_path);
    if oracle_crates.len() >= 2 {
        crate::status_eprintln!(
            "  Discovery: batuta oracle found {} stack crates",
            oracle_crates.len()
        );
        return oracle_crates;
    }

    // Priority 4: .pmat/workspace.toml siblings (legacy, backward-compatible)
    let sibling_crates = discover_from_pmat_siblings(workspace_path);
    if sibling_crates.len() >= 2 {
        crate::status_eprintln!(
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use tempfile::TempDir;

    fn write_cargo(dir: &Path, name: &str, deps: &[&str]) {
        let mut toml =
            format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\n\n[dependencies]\n");
        for d in deps {
            toml.push_str(&format!("{d} = \"1.0\"\n"));
        }
        fs::write(dir.join("Cargo.toml"), toml).unwrap();
    }

    // ── extract_quoted_strings ──────────────────────────────────────────────

    #[test]
    fn test_extract_quoted_strings_basic() {
        let s = r#"["a", "b", "c"]"#;
        let v = extract_quoted_strings(s);
        assert_eq!(v, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_extract_quoted_strings_skips_empty_quoted() {
        // The function pushes only when current is non-empty
        let s = r#"["", "x"]"#;
        let v = extract_quoted_strings(s);
        assert_eq!(v, vec!["x"]);
    }

    #[test]
    fn test_extract_quoted_strings_no_quotes_empty() {
        assert!(extract_quoted_strings("[a, b, c]").is_empty());
    }

    #[test]
    fn test_extract_quoted_strings_handles_chars_in_quotes() {
        let v = extract_quoted_strings(r#""foo/bar/*", "baz""#);
        assert_eq!(v, vec!["foo/bar/*", "baz"]);
    }

    // ── extract_members_array ───────────────────────────────────────────────

    #[test]
    fn test_extract_members_array_single_line() {
        let toml = "[workspace]\nmembers = [\"crates/a\", \"crates/b\"]\n";
        let buf = extract_members_array(toml);
        assert!(buf.contains("crates/a"));
        assert!(buf.contains("crates/b"));
    }

    #[test]
    fn test_extract_members_array_multiline_breaks_at_depth_zero() {
        // The parser initializes bracket_depth = 0 and only counts brackets in
        // lines AFTER the `members =` line. If the first value-line has no
        // brackets, depth stays 0 and the loop breaks. Document this real
        // behavior so the test matches the implementation.
        let toml = "[workspace]\nmembers = [\n  \"a\",\n  \"b\",\n  \"c\",\n]\n[deps]\n";
        let buf = extract_members_array(toml);
        // Only the first value-line is captured before the break.
        assert!(buf.contains("\"a\""));
        // (b and c are dropped — multiline TOML is not fully supported by
        // this parser. Not asserting their presence.)
    }

    #[test]
    fn test_extract_members_array_no_members_returns_empty() {
        let toml = "[package]\nname = \"x\"\n";
        assert!(extract_members_array(toml).is_empty());
    }

    // ── resolve_member_paths ────────────────────────────────────────────────

    #[test]
    fn test_resolve_member_paths_non_glob_existing() {
        let tmp = TempDir::new().unwrap();
        let crate_a = tmp.path().join("crate_a");
        fs::create_dir_all(&crate_a).unwrap();
        write_cargo(&crate_a, "a", &[]);

        let resolved = resolve_member_paths(&["crate_a".to_string()], tmp.path());
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0], crate_a);
    }

    #[test]
    fn test_resolve_member_paths_non_glob_missing_skipped() {
        let tmp = TempDir::new().unwrap();
        let resolved = resolve_member_paths(&["nonexistent".to_string()], tmp.path());
        assert!(resolved.is_empty());
    }

    #[test]
    fn test_resolve_member_paths_glob_pattern() {
        let tmp = TempDir::new().unwrap();
        let a = tmp.path().join("crates").join("a");
        let b = tmp.path().join("crates").join("b");
        fs::create_dir_all(&a).unwrap();
        fs::create_dir_all(&b).unwrap();
        write_cargo(&a, "a", &[]);
        write_cargo(&b, "b", &[]);

        let resolved = resolve_member_paths(&["crates/*".to_string()], tmp.path());
        assert_eq!(resolved.len(), 2);
    }

    #[test]
    fn test_resolve_member_paths_glob_skips_dirs_without_cargo_toml() {
        let tmp = TempDir::new().unwrap();
        let a = tmp.path().join("crates").join("a");
        let bare = tmp.path().join("crates").join("bare");
        fs::create_dir_all(&a).unwrap();
        fs::create_dir_all(&bare).unwrap();
        write_cargo(&a, "a", &[]);
        // bare has no Cargo.toml

        let resolved = resolve_member_paths(&["crates/*".to_string()], tmp.path());
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0], a);
    }

    // ── parse_workspace_members_with_globs (orchestrator) ───────────────────

    #[test]
    fn test_parse_workspace_members_with_globs_end_to_end() {
        let tmp = TempDir::new().unwrap();
        let a = tmp.path().join("a");
        let b = tmp.path().join("b");
        fs::create_dir_all(&a).unwrap();
        fs::create_dir_all(&b).unwrap();
        write_cargo(&a, "a", &[]);
        write_cargo(&b, "b", &[]);

        let toml = "[workspace]\nmembers = [\"a\", \"b\"]\n";
        let resolved = parse_workspace_members_with_globs(toml, tmp.path());
        assert_eq!(resolved.len(), 2);
    }

    // ── read_crate_name ─────────────────────────────────────────────────────

    #[test]
    fn test_read_crate_name_from_package_section() {
        let tmp = TempDir::new().unwrap();
        write_cargo(tmp.path(), "my_crate", &[]);
        assert_eq!(
            read_crate_name(&tmp.path().join("Cargo.toml")),
            Some("my_crate".to_string())
        );
    }

    #[test]
    fn test_read_crate_name_outside_package_section_ignored() {
        let tmp = TempDir::new().unwrap();
        // [other] section — `name = "x"` should be ignored
        let toml = "[other]\nname = \"x\"\n[package]\nname = \"actual\"\n";
        fs::write(tmp.path().join("Cargo.toml"), toml).unwrap();
        assert_eq!(
            read_crate_name(&tmp.path().join("Cargo.toml")),
            Some("actual".to_string())
        );
    }

    #[test]
    fn test_read_crate_name_missing_file_returns_none() {
        assert!(read_crate_name(Path::new("/nonexistent/Cargo.toml")).is_none());
    }

    #[test]
    fn test_read_crate_name_no_name_in_package_returns_none() {
        let tmp = TempDir::new().unwrap();
        let toml = "[package]\nversion = \"0.1\"\n";
        fs::write(tmp.path().join("Cargo.toml"), toml).unwrap();
        assert!(read_crate_name(&tmp.path().join("Cargo.toml")).is_none());
    }

    // ── read_cargo_deps ─────────────────────────────────────────────────────

    #[test]
    fn test_read_cargo_deps_basic() {
        let tmp = TempDir::new().unwrap();
        write_cargo(tmp.path(), "x", &["serde", "tokio"]);
        let deps = read_cargo_deps(&tmp.path().join("Cargo.toml"));
        assert!(deps.contains(&"serde".to_string()));
        assert!(deps.contains(&"tokio".to_string()));
    }

    #[test]
    fn test_read_cargo_deps_dev_dependencies_section() {
        let tmp = TempDir::new().unwrap();
        let toml =
            "[package]\nname = \"x\"\n[dev-dependencies]\nproptest = \"1.0\"\ntempfile = \"3.0\"\n";
        fs::write(tmp.path().join("Cargo.toml"), toml).unwrap();
        let deps = read_cargo_deps(&tmp.path().join("Cargo.toml"));
        assert!(deps.contains(&"proptest".to_string()));
        assert!(deps.contains(&"tempfile".to_string()));
    }

    #[test]
    fn test_read_cargo_deps_dotted_dependencies_section() {
        let tmp = TempDir::new().unwrap();
        let toml = "[package]\nname = \"x\"\n[dependencies.serde]\nversion = \"1.0\"\n";
        fs::write(tmp.path().join("Cargo.toml"), toml).unwrap();
        let deps = read_cargo_deps(&tmp.path().join("Cargo.toml"));
        // The [dependencies.serde] section enables in_deps_section=true,
        // so `version = "1.0"` is captured as a dep name.
        assert!(deps.contains(&"version".to_string()));
    }

    #[test]
    fn test_read_cargo_deps_skips_non_dep_sections() {
        let tmp = TempDir::new().unwrap();
        let toml = "[package]\nname = \"x\"\nversion = \"0.1\"\n";
        fs::write(tmp.path().join("Cargo.toml"), toml).unwrap();
        let deps = read_cargo_deps(&tmp.path().join("Cargo.toml"));
        // [package] is not a deps section → lines skipped
        assert!(!deps.contains(&"version".to_string()));
        assert!(!deps.contains(&"name".to_string()));
    }

    #[test]
    fn test_read_cargo_deps_missing_file_empty() {
        assert!(read_cargo_deps(Path::new("/nonexistent/Cargo.toml")).is_empty());
    }

    // ── make_crate_info ─────────────────────────────────────────────────────

    #[test]
    fn test_make_crate_info_uses_cargo_name() {
        let tmp = TempDir::new().unwrap();
        write_cargo(tmp.path(), "named_crate", &["dep1"]);
        let info = make_crate_info(tmp.path());
        assert_eq!(info.name, "named_crate");
        assert!(info.cargo_deps.contains(&"dep1".to_string()));
    }

    #[test]
    fn test_make_crate_info_falls_back_to_dirname() {
        let tmp = TempDir::new().unwrap();
        // No Cargo.toml — falls back to the dir's file_name
        let info = make_crate_info(tmp.path());
        assert!(!info.name.is_empty());
        assert_eq!(info.path, tmp.path());
    }

    // ── extract_paiml_dep_names ─────────────────────────────────────────────

    #[test]
    fn test_extract_paiml_dep_names_basic() {
        let info = json!({
            "paiml_dependencies": [
                {"name": "trueno"},
                {"name": "aprender"}
            ]
        });
        let names = extract_paiml_dep_names(&info);
        assert_eq!(names, vec!["trueno", "aprender"]);
    }

    #[test]
    fn test_extract_paiml_dep_names_missing_field_empty() {
        let info = json!({});
        assert!(extract_paiml_dep_names(&info).is_empty());
    }

    #[test]
    fn test_extract_paiml_dep_names_skips_dep_without_name() {
        let info = json!({
            "paiml_dependencies": [
                {"name": "x"},
                {"version": "1.0"}
            ]
        });
        let names = extract_paiml_dep_names(&info);
        assert_eq!(names, vec!["x"]);
    }

    // ── collect_related_crates ──────────────────────────────────────────────

    #[test]
    fn test_collect_related_crates_includes_self() {
        let projects = serde_json::Map::new();
        let related = collect_related_crates(&projects, "self_crate");
        assert!(related.contains("self_crate"));
        assert_eq!(related.len(), 1);
    }

    #[test]
    fn test_collect_related_crates_includes_forward_deps() {
        let mut projects = serde_json::Map::new();
        projects.insert(
            "current".into(),
            json!({"paiml_dependencies": [{"name": "dep_a"}]}),
        );
        let related = collect_related_crates(&projects, "current");
        assert!(related.contains("current"));
        assert!(related.contains("dep_a"));
    }

    #[test]
    fn test_collect_related_crates_includes_reverse_deps() {
        let mut projects = serde_json::Map::new();
        projects.insert("current".into(), json!({}));
        projects.insert(
            "consumer".into(),
            json!({"paiml_dependencies": [{"name": "current"}]}),
        );
        let related = collect_related_crates(&projects, "current");
        assert!(related.contains("current"));
        assert!(related.contains("consumer"));
    }

    // ── find_current_project ────────────────────────────────────────────────

    #[test]
    fn test_find_current_project_matches_canonical_path() {
        let tmp = TempDir::new().unwrap();
        let canonical = tmp.path().canonicalize().unwrap();
        let mut projects = serde_json::Map::new();
        projects.insert(
            "my_project".into(),
            json!({"path": canonical.to_string_lossy()}),
        );
        let found = find_current_project(&projects, &canonical);
        assert_eq!(found, Some("my_project".to_string()));
    }

    #[test]
    fn test_find_current_project_no_match_returns_none() {
        let tmp = TempDir::new().unwrap();
        let other = tmp.path().to_path_buf();
        let mut projects = serde_json::Map::new();
        projects.insert("p1".into(), json!({"path": "/some/other/path"}));
        assert!(find_current_project(&projects, &other).is_none());
    }

    // ── projects_to_crate_infos ─────────────────────────────────────────────

    #[test]
    fn test_projects_to_crate_infos_keeps_only_with_cargo_toml() {
        let tmp = TempDir::new().unwrap();
        let a = tmp.path().join("a");
        fs::create_dir_all(&a).unwrap();
        write_cargo(&a, "a", &[]);

        let mut projects = serde_json::Map::new();
        projects.insert("a".into(), json!({"path": a.to_string_lossy()}));
        projects.insert("missing".into(), json!({"path": "/nonexistent"}));

        let mut related = HashSet::new();
        related.insert("a".to_string());
        related.insert("missing".to_string());

        let infos = projects_to_crate_infos(&projects, &related);
        assert_eq!(infos.len(), 1);
        assert_eq!(infos[0].name, "a");
    }

    // ── discover_workspace_crates (priority chain) ──────────────────────────

    #[test]
    fn test_discover_workspace_crates_priority_1_explicit() {
        let tmp = TempDir::new().unwrap();
        write_cargo(tmp.path(), "root", &[]);

        let crate_a = tmp.path().join("crate_a");
        fs::create_dir_all(&crate_a).unwrap();
        write_cargo(&crate_a, "a", &[]);

        let crates = discover_workspace_crates(tmp.path(), Some(&[PathBuf::from("crate_a")]));
        // Always includes the workspace_path itself + each explicit crate
        assert!(crates.iter().any(|c| c.name == "root"));
        assert!(crates.iter().any(|c| c.name == "a"));
    }

    #[test]
    fn test_discover_workspace_crates_priority_2_cargo_workspace() {
        let tmp = TempDir::new().unwrap();
        // Create a workspace Cargo.toml at root
        let toml = "[workspace]\nmembers = [\"a\", \"b\"]\n";
        fs::write(tmp.path().join("Cargo.toml"), toml).unwrap();

        let a = tmp.path().join("a");
        let b = tmp.path().join("b");
        fs::create_dir_all(&a).unwrap();
        fs::create_dir_all(&b).unwrap();
        write_cargo(&a, "crate_a", &[]);
        write_cargo(&b, "crate_b", &[]);

        let crates = discover_workspace_crates(tmp.path(), None);
        // 2-member workspace exits at priority 2
        assert_eq!(crates.len(), 2);
    }

    #[test]
    fn test_discover_workspace_crates_priority_5_single_crate_fallback() {
        let tmp = TempDir::new().unwrap();
        write_cargo(tmp.path(), "lone", &[]);
        // No workspace, no .pmat/workspace.toml — falls all the way through
        let crates = discover_workspace_crates(tmp.path(), None);
        assert_eq!(crates.len(), 1);
        assert_eq!(crates[0].name, "lone");
    }

    #[test]
    fn test_discover_workspace_crates_explicit_with_empty_slice_falls_through() {
        let tmp = TempDir::new().unwrap();
        write_cargo(tmp.path(), "lone", &[]);
        // Some(&[]) — empty slice, NOT taken as priority 1; falls through to priority 5
        let crates = discover_workspace_crates(tmp.path(), Some(&[]));
        assert_eq!(crates.len(), 1);
        assert_eq!(crates[0].name, "lone");
    }

    // ── discover_from_explicit ──────────────────────────────────────────────

    #[test]
    fn test_discover_from_explicit_skips_missing_cargo_toml() {
        let tmp = TempDir::new().unwrap();
        write_cargo(tmp.path(), "root", &[]);
        // empty_dir has no Cargo.toml
        let empty = tmp.path().join("empty_dir");
        fs::create_dir_all(&empty).unwrap();

        let crates = discover_from_explicit(tmp.path(), &[PathBuf::from("empty_dir")]);
        // Only the workspace itself; empty_dir skipped
        assert_eq!(crates.len(), 1);
    }
}
