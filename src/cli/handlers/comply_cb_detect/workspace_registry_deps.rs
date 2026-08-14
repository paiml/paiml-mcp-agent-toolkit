//! CB-081-F: a workspace member pulled from the registry by a sibling.
//!
//! CB-081-B already counts crates that resolve to more than one version, but it
//! reports them as one undifferentiated list, and that list is dominated by
//! third-party conflicts a project usually cannot fix. Two causes with nothing
//! in common were being scored as one number:
//!
//! | | third-party major conflict | workspace member from registry |
//! |---|---|---|
//! | fixable by the project | often not | **always** |
//! | correct action | wait for upstream, or fork | repoint at the path |
//! | consequence | one extra crate | the same code compiled N times |
//!
//! The motivating tree (paiml/aprender, 78 crates) states the rule in prose in
//! its own `CLAUDE.md` and violates it in nine places: `trueno` resolves at
//! 0.16, 0.16.5 **and** the in-tree 0.63.0 simultaneously — three compilations
//! of the SIMD kernels in one binary — while `jugar-probar` spans seven declared
//! versions. `pmat comply check` did see it: `trueno` was in CB-081's
//! "176 duplicate crates" list, flattened in among `windows_i686_gnu`, `syn` and
//! `wasi`, under the generic remedy "run cargo tree --duplicates". The signal
//! was there; the severity was not, and a 176-item list reads as unfixable
//! noise (#989).
//!
//! # The trap this must not fall into
//!
//! Match on **both** `[package] name` and `[lib] name`. They diverge in real
//! workspaces, and that is precisely the case that matters here: package
//! `aprender-compute` has `[lib] name = "trueno"`, and package `aprender-db`
//! has `[lib] name = "trueno_db"`. A checker comparing package names alone
//! misses the `trueno` violation entirely — the single most costly one, since it
//! is the performance foundation every sibling depends on.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// One sibling declared with a registry version instead of a path.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegistryMemberDep {
    /// Member manifest holding the declaration, relative to the workspace root.
    pub file: PathBuf,
    /// 1-indexed line of the declaration.
    pub line: usize,
    /// The dependency key as written (`trueno`), which is not necessarily the
    /// package name.
    pub dep_key: String,
    /// The workspace member it resolves to (`aprender-compute`).
    pub member_package: String,
    /// Where that member lives in-tree, relative to the workspace root.
    pub member_path: PathBuf,
    /// True when the match was on `[lib] name` rather than `[package] name` —
    /// the case a package-name-only checker misses.
    pub matched_lib_name: bool,
    /// The declaration as written, trimmed.
    pub declaration: String,
}

/// A workspace member, indexed by every name a sibling could depend on it under.
#[derive(Debug, Clone)]
struct Member {
    package: String,
    path: PathBuf,
}

/// Scan a workspace for members pulled from the registry by their siblings.
///
/// Returns an empty vector for a non-workspace or an unreadable root, which is
/// correct: there are no members, so none can be mis-declared. It does NOT
/// return empty for "could not parse a member manifest" — an unreadable member
/// is skipped individually, and the others are still checked.
pub fn detect_workspace_members_from_registry(project_path: &Path) -> Vec<RegistryMemberDep> {
    let root_manifest = project_path.join("Cargo.toml");
    let Ok(root) = std::fs::read_to_string(&root_manifest) else {
        return Vec::new();
    };
    let member_dirs = workspace_member_dirs(&root, project_path);
    if member_dirs.is_empty() {
        return Vec::new();
    }

    // name -> member. One member contributes up to two keys: its package name
    // and, when they differ, its lib name.
    let mut by_name: BTreeMap<String, (Member, bool)> = BTreeMap::new();
    let mut manifests: Vec<(PathBuf, String)> = Vec::new();
    for dir in &member_dirs {
        let manifest = dir.join("Cargo.toml");
        let Ok(content) = std::fs::read_to_string(&manifest) else {
            continue;
        };
        let rel_dir = relative_to(dir, project_path);
        if let Some(package) = section_name(&content, "package") {
            let member = Member {
                package: package.clone(),
                path: rel_dir.clone(),
            };
            if let Some(lib) = section_name(&content, "lib") {
                if normalize(&lib) != normalize(&package) {
                    by_name.insert(normalize(&lib), (member.clone(), true));
                }
            }
            by_name.insert(normalize(&package), (member, false));
        }
        manifests.push((manifest, content));
    }
    if by_name.is_empty() {
        return Vec::new();
    }

    // The root manifest is scanned whether or not "." is listed as a member.
    // Its `[workspace.dependencies]` is the single highest-leverage place for
    // this defect — one registry version there reaches every sibling that
    // inherits it — and relying on "." being a member makes coverage depend on
    // a stylistic choice.
    if !manifests.iter().any(|(p, _)| p == &root_manifest) {
        manifests.push((root_manifest, root));
    }

    let mut found = Vec::new();
    for (manifest, content) in &manifests {
        let rel = relative_to(manifest, project_path);
        collect_from_manifest(content, &rel, &by_name, &mut found);
    }
    found.sort_by(|a, b| (&a.file, a.line).cmp(&(&b.file, b.line)));
    found
}

/// How many members this workspace has — 0 when it is not a workspace.
///
/// The caller needs this to tell "checked a workspace, found nothing wrong"
/// from "there was no workspace to check". Reporting those as the same Pass is
/// the defect this repository keeps finding.
#[must_use]
pub fn workspace_member_count(project_path: &Path) -> usize {
    std::fs::read_to_string(project_path.join("Cargo.toml"))
        .map(|root| workspace_member_dirs(&root, project_path).len())
        .unwrap_or(0)
}

/// Resolve `[workspace] members`, expanding a trailing `*` glob.
fn workspace_member_dirs(root: &str, base: &Path) -> Vec<PathBuf> {
    let Some(raw) = array_after_key(root, "members") else {
        return Vec::new();
    };
    let mut dirs = Vec::new();
    for pattern in quoted_strings(&raw) {
        if let Some(prefix) = pattern.strip_suffix("/*") {
            let Ok(entries) = std::fs::read_dir(base.join(prefix)) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.join("Cargo.toml").is_file() {
                    dirs.push(path);
                }
            }
        } else {
            let path = base.join(&pattern);
            if path.join("Cargo.toml").is_file() {
                dirs.push(path);
            }
        }
    }
    dirs.sort();
    dirs.dedup();
    dirs
}

/// Cargo treats `-` and `_` as the same character in a lib name, so a match must
/// too.
///
/// This is not cosmetic. `crates/aprender-test-lib` has `[lib] name =
/// "jugar_probar"`, and `crates/aprender-data` declares `jugar-probar = "1.0.1"`
/// — the registry package and the in-tree crate produce the SAME lib name, which
/// is the whole collision. Comparing the strings literally missed both sites
/// this check was written to catch (#989).
fn normalize(name: &str) -> String {
    name.replace('-', "_")
}

/// `name = "..."` from `[package]` or `[lib]`.
fn section_name(content: &str, section: &str) -> Option<String> {
    let header = format!("[{section}]");
    let mut inside = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            inside = trimmed == header;
            continue;
        }
        if !inside || trimmed.starts_with('#') {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("name") {
            let rest = rest.trim_start();
            if let Some(value) = rest.strip_prefix('=') {
                if let Some(name) = quoted_strings(value).into_iter().next() {
                    return Some(name);
                }
            }
        }
    }
    None
}

/// Walk one manifest, reporting sibling dependencies declared by version.
fn collect_from_manifest(
    content: &str,
    rel_manifest: &Path,
    by_name: &BTreeMap<String, (Member, bool)>,
    out: &mut Vec<RegistryMemberDep>,
) {
    // The member declaring the dependency, so a crate is never reported for
    // depending on itself via `[lib]`/`[[bin]]` sections.
    let own_package = section_name(content, "package");
    let lines: Vec<&str> = content.lines().collect();
    let mut in_deps = false;
    // `[dependencies.foo]` / `[target.'cfg(..)'.dev-dependencies.foo]` table form.
    let mut table_dep: Option<(String, usize)> = None;
    let mut table_buf = String::new();

    for (idx, raw) in lines.iter().enumerate() {
        let trimmed = raw.trim();
        if trimmed.starts_with('[') {
            // A new section closes any table-form declaration being gathered.
            if let Some((key, line)) = table_dep.take() {
                consider(
                    &key,
                    &table_buf,
                    line,
                    rel_manifest,
                    own_package.as_deref(),
                    by_name,
                    out,
                );
                table_buf.clear();
            }
            in_deps = is_dependency_header(trimmed);
            table_dep = dependency_table_key(trimmed).map(|k| (k, idx + 1));
            continue;
        }
        if table_dep.is_some() {
            if !trimmed.starts_with('#') {
                table_buf.push_str(trimmed);
                table_buf.push('\n');
            }
            continue;
        }
        if !in_deps || trimmed.starts_with('#') || !trimmed.contains('=') {
            continue;
        }
        let Some((key, value)) = split_dep_line(trimmed) else {
            continue;
        };
        consider(
            &key,
            value,
            idx + 1,
            rel_manifest,
            own_package.as_deref(),
            by_name,
            out,
        );
    }
    if let Some((key, line)) = table_dep {
        consider(
            &key,
            &table_buf,
            line,
            rel_manifest,
            own_package.as_deref(),
            by_name,
            out,
        );
    }
}

/// Is this header a dependency section of any kind?
///
/// `[workspace.dependencies]` is INCLUDED, and deliberately so. An earlier cut
/// of this check excluded it on the reasoning that it declares versions rather
/// than uses them — and then reported a clean pass over paiml/aprender while
/// `crates/aprender-test/Cargo.toml:82` said
/// `trueno = { version = "0.16.5", features = ["gpu"] }` (#989). That section is
/// the WORST place for a registry version, not an exempt one: every sibling
/// writing the correct-looking `{ workspace = true }` inherits it, so one line
/// pulls the duplicate into the whole tree while each member's own manifest
/// looks right. It is also how nested workspaces re-enter — `aprender-test` is
/// a member of the outer workspace and a workspace root in its own right.
fn is_dependency_header(trimmed: &str) -> bool {
    let inner = trimmed.trim_start_matches('[').trim_end_matches(']');
    // `[target.'cfg(unix)'.dependencies]` — take the trailing component.
    let tail = inner.rsplit('.').next().unwrap_or(inner);
    matches!(
        tail,
        "dependencies" | "dev-dependencies" | "build-dependencies"
    )
}

/// `[dependencies.foo]` / `[workspace.dependencies.foo]` -> `foo`.
/// Returns None for a plain section header.
fn dependency_table_key(trimmed: &str) -> Option<String> {
    let inner = trimmed.trim_start_matches('[').trim_end_matches(']');
    let mut parts: Vec<&str> = inner.split('.').collect();
    let key = parts.pop()?;
    let parent = parts.pop()?;
    if matches!(
        parent,
        "dependencies" | "dev-dependencies" | "build-dependencies"
    ) {
        Some(key.trim_matches('"').to_string())
    } else {
        None
    }
}

/// `foo = { version = "1" }` -> `("foo", "{ version = \"1\" }")`.
fn split_dep_line(trimmed: &str) -> Option<(String, &str)> {
    let eq = trimmed.find('=')?;
    let key = trimmed[..eq].trim();
    if key.is_empty() || key.contains(' ') || key.contains('"') {
        return None;
    }
    Some((key.to_string(), trimmed[eq + 1..].trim()))
}

/// Record a violation when `key`/`value` names a workspace member by version.
fn consider(
    key: &str,
    value: &str,
    line: usize,
    rel_manifest: &Path,
    own_package: Option<&str>,
    by_name: &BTreeMap<String, (Member, bool)>,
    out: &mut Vec<RegistryMemberDep>,
) {
    // A renaming key (`trueno-db = { package = "aprender-db" }`) means the real
    // crate is the `package` value, so both must be tried.
    let renamed = value_of(value, "package");
    let candidate = renamed.as_deref().unwrap_or(key);
    let Some((member, matched_lib_name)) = by_name.get(&normalize(candidate)) else {
        return;
    };
    if own_package == Some(member.package.as_str()) {
        return;
    }
    // A path or workspace-inherited declaration is the CORRECT form and must
    // never be reported: this check exists to distinguish exactly these.
    if has_key(value, "path") || value_of(value, "workspace").as_deref() == Some("true") {
        return;
    }
    // Only a registry version pulls a second copy. A bare `{ features = [..] }`
    // with neither version nor path is invalid TOML for cargo, not our finding.
    if !has_key(value, "version") && !is_bare_version(value) {
        return;
    }
    out.push(RegistryMemberDep {
        file: rel_manifest.to_path_buf(),
        line,
        dep_key: key.to_string(),
        member_package: member.package.clone(),
        member_path: member.path.clone(),
        matched_lib_name: *matched_lib_name,
        declaration: format!("{key} = {}", value.trim()).trim().to_string(),
    });
}

/// `foo = "1.2"` — the shorthand where the whole value is a version string.
fn is_bare_version(value: &str) -> bool {
    let t = value.trim();
    t.starts_with('"') && t.ends_with('"') && t.len() >= 2
}

/// Does the inline table declare `key`?
fn has_key(value: &str, key: &str) -> bool {
    value_of(value, key).is_some()
}

/// Read `key = <value>` out of an inline table or a table-form body.
fn value_of(value: &str, key: &str) -> Option<String> {
    let mut rest = value;
    while let Some(pos) = rest.find(key) {
        let after = &rest[pos + key.len()..];
        let before_ok = pos == 0
            || !rest[..pos]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_alphanumeric() || c == '-' || c == '_');
        let after_trim = after.trim_start();
        if before_ok {
            if let Some(v) = after_trim.strip_prefix('=') {
                let v = v.trim_start();
                if let Some(s) = quoted_strings(v).into_iter().next() {
                    // Only when the quote opens immediately, so `version` in a
                    // later field cannot be read off this one.
                    if v.starts_with('"') {
                        return Some(s);
                    }
                }
                let word: String = v
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '.')
                    .collect();
                if !word.is_empty() {
                    return Some(word);
                }
            }
        }
        rest = &rest[pos + key.len()..];
    }
    None
}

/// The `[...]` array following `key =`, brace-balanced across lines.
fn array_after_key(content: &str, key: &str) -> Option<String> {
    let mut lines = content.lines();
    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        let Some(rest) = trimmed.strip_prefix(key) else {
            continue;
        };
        let rest = rest.trim_start();
        let Some(rest) = rest.strip_prefix('=') else {
            continue;
        };
        let mut buf = rest.trim_start().to_string();
        if buf.matches('[').count() > buf.matches(']').count() {
            for next in lines.by_ref() {
                buf.push('\n');
                buf.push_str(next);
                if buf.matches('[').count() <= buf.matches(']').count() {
                    break;
                }
            }
        }
        return Some(buf);
    }
    None
}

/// Every `"…"` in `buf`, comments stripped.
fn quoted_strings(buf: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in buf.lines() {
        let line = match line.find('#') {
            Some(i) if !line[..i].contains('"') || line[..i].matches('"').count() % 2 == 0 => {
                &line[..i]
            }
            _ => line,
        };
        let mut chars = line.chars();
        while let Some(c) = chars.next() {
            if c != '"' {
                continue;
            }
            let s: String = chars.by_ref().take_while(|c| *c != '"').collect();
            out.push(s);
        }
    }
    out
}

fn relative_to(path: &Path, base: &Path) -> PathBuf {
    path.strip_prefix(base).unwrap_or(path).to_path_buf()
}

/// Render the finding the way the ticket asks: file:line, and the fix.
#[must_use]
pub fn format_registry_member_deps(found: &[RegistryMemberDep]) -> String {
    use std::fmt::Write;
    let mut s = format!(
        "{} sibling(s) pulled from crates.io instead of the in-tree copy",
        found.len()
    );
    for f in found {
        let _ = write!(
            s,
            "\n    {}:{}  {}",
            f.file.display(),
            f.line,
            f.declaration
        );
        let via = if f.matched_lib_name {
            format!(" (lib name of {})", f.member_package)
        } else {
            String::new()
        };
        let _ = write!(
            s,
            "\n      -> in-tree at {}{via}; use {{ workspace = true }} or {{ path = \"…\" }}",
            f.member_path.display()
        );
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a workspace on disk. `members` is (dir, manifest-contents).
    fn workspace(root_extra: &str, members: &[(&str, &str)]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        let list = members
            .iter()
            .map(|(d, _)| format!("\"{d}\""))
            .collect::<Vec<_>>()
            .join(", ");
        std::fs::write(
            dir.path().join("Cargo.toml"),
            format!("[workspace]\nmembers = [{list}]\n{root_extra}"),
        )
        .expect("root");
        for (d, content) in members {
            let p = dir.path().join(d);
            std::fs::create_dir_all(&p).expect("mkdir");
            std::fs::write(p.join("Cargo.toml"), content).expect("member");
        }
        dir
    }

    /// THE motivating case (#989): the sibling is depended on by its `[lib]`
    /// name, which differs from its package name. A package-name-only checker
    /// finds nothing here — and `trueno` is the one that matters most.
    #[test]
    fn matches_on_lib_name_not_just_package_name() {
        let ws = workspace(
            "",
            &[
                (
                    "crates/aprender-compute",
                    "[package]\nname = \"aprender-compute\"\n\n[lib]\nname = \"trueno\"\n",
                ),
                (
                    "crates/aprender-test",
                    "[package]\nname = \"aprender-test\"\n\n[dependencies]\ntrueno = { version = \"0.16.5\", features = [\"gpu\"] }\n",
                ),
            ],
        );
        let found = detect_workspace_members_from_registry(ws.path());
        assert_eq!(found.len(), 1, "{found:#?}");
        assert_eq!(found[0].dep_key, "trueno");
        assert_eq!(found[0].member_package, "aprender-compute");
        assert!(found[0].matched_lib_name, "must record the lib-name match");
        assert_eq!(found[0].line, 5);
    }

    /// The correct forms must never be reported — the whole point of splitting
    /// this out of CB-081's count is that it is unconditionally actionable.
    #[test]
    fn path_and_workspace_inherited_declarations_are_not_findings() {
        let ws = workspace(
            "",
            &[
                ("crates/core", "[package]\nname = \"core-crate\"\n"),
                (
                    "crates/a",
                    "[package]\nname = \"a\"\n\n[dependencies]\ncore-crate = { path = \"../core\" }\n",
                ),
                (
                    "crates/b",
                    "[package]\nname = \"b\"\n\n[dependencies]\ncore-crate = { workspace = true }\n",
                ),
            ],
        );
        assert!(
            detect_workspace_members_from_registry(ws.path()).is_empty(),
            "path/workspace declarations are the FIX, not the defect"
        );
    }

    /// A renaming key resolves through `package = "…"`.
    #[test]
    fn renamed_dependency_resolves_through_package_key() {
        let ws = workspace(
            "",
            &[
                ("crates/db", "[package]\nname = \"aprender-db\"\n"),
                (
                    "crates/user",
                    "[package]\nname = \"user\"\n\n[dependencies]\ntrueno-db = { version = \"0.61.0\", package = \"aprender-db\" }\n",
                ),
            ],
        );
        let found = detect_workspace_members_from_registry(ws.path());
        assert_eq!(found.len(), 1, "{found:#?}");
        assert_eq!(found[0].member_package, "aprender-db");
    }

    /// Third-party crates are somebody else's problem and must not appear here;
    /// that is exactly the noise this check exists to separate itself from.
    #[test]
    fn third_party_registry_deps_are_ignored() {
        let ws = workspace(
            "",
            &[(
                "crates/a",
                "[package]\nname = \"a\"\n\n[dependencies]\nserde = \"1.0\"\ntokio = { version = \"1\", features = [\"full\"] }\n",
            )],
        );
        assert!(detect_workspace_members_from_registry(ws.path()).is_empty());
    }

    /// Table form and `[target.…]` sections are declarations too.
    #[test]
    fn table_form_and_target_sections_are_scanned() {
        let ws = workspace(
            "",
            &[
                ("crates/core", "[package]\nname = \"core-crate\"\n"),
                (
                    "crates/a",
                    "[package]\nname = \"a\"\n\n[dependencies.core-crate]\nversion = \"0.1\"\nfeatures = [\"x\"]\n",
                ),
                (
                    "crates/b",
                    "[package]\nname = \"b\"\n\n[target.'cfg(unix)'.dependencies]\ncore-crate = \"0.1\"\n",
                ),
            ],
        );
        let found = detect_workspace_members_from_registry(ws.path());
        assert_eq!(found.len(), 2, "{found:#?}");
    }

    /// REGRESSION (#989): `[workspace.dependencies]` was excluded, and the check
    /// reported a clean pass over the very tree the ticket was filed about.
    ///
    /// This is the exact shape found in paiml/aprender: a member that is itself
    /// a workspace root (`crates/aprender-test`, a vendored `probar`) declaring
    /// an OUTER member by its `[lib]` name at a registry version. Every crate
    /// inheriting it writes the correct-looking `{ workspace = true }`, so the
    /// duplicate is invisible in each individual manifest.
    #[test]
    fn workspace_dependencies_section_is_scanned() {
        let ws = workspace(
            "",
            &[
                (
                    "crates/aprender-compute",
                    "[package]\nname = \"aprender-compute\"\n\n[lib]\nname = \"trueno\"\n",
                ),
                (
                    "crates/aprender-test",
                    "[workspace]\nmembers = [\"crates/probar\"]\n\n[workspace.dependencies]\ntrueno = { version = \"0.16.5\", features = [\"gpu\"] }\n",
                ),
            ],
        );
        let found = detect_workspace_members_from_registry(ws.path());
        assert_eq!(
            found.len(),
            1,
            "[workspace.dependencies] is where this defect hides — {found:#?}"
        );
        assert_eq!(found[0].member_package, "aprender-compute");
        assert!(found[0].matched_lib_name);
        assert_eq!(found[0].line, 5);
    }

    /// The root's own `[workspace.dependencies]` is checked even when "." is not
    /// listed as a member, so coverage does not hinge on a stylistic choice.
    #[test]
    fn root_workspace_dependencies_are_scanned_without_dot_member() {
        let ws = workspace(
            "[workspace.dependencies]\ncore-crate = \"0.1\"\n",
            &[("crates/core", "[package]\nname = \"core-crate\"\n")],
        );
        let found = detect_workspace_members_from_registry(ws.path());
        assert_eq!(found.len(), 1, "{found:#?}");
        assert_eq!(found[0].file, PathBuf::from("Cargo.toml"));
    }

    /// The correct root form — `path` alongside `version` and a renaming
    /// `package` — is what aprender's root actually writes, and must stay clean.
    #[test]
    fn root_path_plus_version_plus_package_is_correct_form() {
        let ws = workspace(
            "[workspace.dependencies]\ntrueno = { path = \"crates/aprender-compute\", version = \"0.63.0\", package = \"aprender-compute\" }\n",
            &[(
                "crates/aprender-compute",
                "[package]\nname = \"aprender-compute\"\n\n[lib]\nname = \"trueno\"\n",
            )],
        );
        assert!(
            detect_workspace_members_from_registry(ws.path()).is_empty(),
            "a path declaration is the fix, even with version and package present"
        );
    }

    /// REGRESSION (#989): `-` and `_` are the same character to Cargo in a lib
    /// name, and comparing literally missed BOTH real `jugar-probar` sites.
    #[test]
    fn hyphen_and_underscore_are_the_same_lib_name() {
        let ws = workspace(
            "",
            &[
                (
                    "crates/aprender-test-lib",
                    "[package]\nname = \"aprender-test-lib\"\n\n[lib]\nname = \"jugar_probar\"\n",
                ),
                (
                    "crates/aprender-data",
                    "[package]\nname = \"aprender-data\"\n\n[dependencies]\njugar-probar = \"1.0.1\"\n",
                ),
            ],
        );
        let found = detect_workspace_members_from_registry(ws.path());
        assert_eq!(found.len(), 1, "jugar-probar IS jugar_probar — {found:#?}");
        assert_eq!(found[0].member_package, "aprender-test-lib");
    }

    /// dev- and build-dependencies pull the duplicate just as surely.
    #[test]
    fn dev_and_build_dependencies_count() {
        let ws = workspace(
            "",
            &[
                ("crates/core", "[package]\nname = \"core-crate\"\n"),
                (
                    "crates/a",
                    "[package]\nname = \"a\"\n\n[dev-dependencies]\ncore-crate = \"0.1\"\n\n[build-dependencies]\ncore-crate = \"0.1\"\n",
                ),
            ],
        );
        assert_eq!(detect_workspace_members_from_registry(ws.path()).len(), 2);
    }

    /// A glob member list must expand, or a 78-crate workspace reports nothing.
    #[test]
    fn glob_members_expand() {
        let ws = workspace("", &[("crates/core", "[package]\nname = \"core-crate\"\n")]);
        std::fs::write(
            ws.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/*\"]\n",
        )
        .expect("root");
        let p = ws.path().join("crates/user");
        std::fs::create_dir_all(&p).expect("mkdir");
        std::fs::write(
            p.join("Cargo.toml"),
            "[package]\nname = \"user\"\n\n[dependencies]\ncore-crate = \"0.1\"\n",
        )
        .expect("member");
        let found = detect_workspace_members_from_registry(ws.path());
        assert_eq!(found.len(), 1, "glob members must expand — {found:#?}");
    }

    /// A crate is not in violation for naming itself.
    #[test]
    fn self_reference_is_not_a_finding() {
        let ws = workspace(
            "",
            &[(
                "crates/a",
                "[package]\nname = \"a\"\n\n[lib]\nname = \"a_lib\"\n\n[dependencies]\nserde = \"1\"\n",
            )],
        );
        assert!(detect_workspace_members_from_registry(ws.path()).is_empty());
    }

    /// Not a workspace: nothing to say, and it must not error.
    #[test]
    fn plain_crate_yields_nothing() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"solo\"\n\n[dependencies]\nserde = \"1\"\n",
        )
        .expect("write");
        assert!(detect_workspace_members_from_registry(dir.path()).is_empty());
    }

    /// The message must name file:line and the fix, per #989.
    #[test]
    fn message_names_location_and_remedy() {
        let found = vec![RegistryMemberDep {
            file: PathBuf::from("crates/aprender-test/Cargo.toml"),
            line: 82,
            dep_key: "trueno".into(),
            member_package: "aprender-compute".into(),
            member_path: PathBuf::from("crates/aprender-compute"),
            matched_lib_name: true,
            declaration: "trueno = { version = \"0.16.5\" }".into(),
        }];
        let msg = format_registry_member_deps(&found);
        assert!(msg.contains("crates/aprender-test/Cargo.toml:82"), "{msg}");
        assert!(msg.contains("crates/aprender-compute"), "{msg}");
        assert!(msg.contains("workspace = true"), "{msg}");
    }
}
