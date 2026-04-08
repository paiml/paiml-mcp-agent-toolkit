// Provable-contracts enforcement helper functions
// Included from check.rs — do NOT add `use` imports or `#!` attributes here.

/// Extract equation names from contract YAMLs that have preconditions or postconditions.
fn collect_contract_equation_names(contracts_dir: &Path) -> Vec<String> {
    let mut eq_names = Vec::new();
    let headers = [
        "equations",
        "metadata",
        "falsification_tests",
        "kani_harnesses",
        "proof_obligations",
        "qa_gate",
        "implementation",
        "enforcement",
        "version",
        "created",
        "author",
        "description",
        "references",
        "issues",
    ];
    let Ok(entries) = std::fs::read_dir(contracts_dir) else {
        return eq_names;
    };
    for entry in entries.flatten() {
        if entry.path().extension().map_or(true, |e| e != "yaml") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(entry.path()) else {
            continue;
        };
        let lines: Vec<&str> = content.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if !trimmed.ends_with(':')
                || trimmed.starts_with('#')
                || trimmed.starts_with('-')
                || trimmed.contains(' ')
                || !line.starts_with("  ")
                || line.starts_with("    ")
            {
                continue;
            }
            let name = trimmed.trim_end_matches(':');
            if headers.contains(&name) {
                continue;
            }
            // Look ahead for preconditions/postconditions
            let has_pre_post = lines[i + 1..]
                .iter()
                .take_while(|next| {
                    let nt = next.trim();
                    !(next.starts_with("  ")
                        && !next.starts_with("    ")
                        && nt.ends_with(':')
                        && !nt.starts_with('#')
                        && !nt.starts_with('-'))
                })
                .any(|next| {
                    let nt = next.trim();
                    nt == "preconditions:" || nt == "postconditions:"
                });
            if has_pre_post {
                eq_names.push(name.to_string());
            }
        }
    }
    eq_names
}

/// Quick check if a directory contains any contract YAML files (not just binding.yaml).
fn has_contract_yamls(dir: &Path) -> bool {
    std::fs::read_dir(dir).into_iter().flatten().flatten().any(|e| {
        let p = e.path();
        p.is_file()
            && p.extension().is_some_and(|ext| ext == "yaml" || ext == "yml")
            && !p.file_name().is_some_and(|n| n.to_string_lossy().contains("binding"))
    })
}

/// Resolve the contracts directory for a project.
/// Checks local `contracts/` first (if it has YAMLs), then sibling `../provable-contracts/contracts/<name>/`.
/// Tries both the directory name and the Cargo.toml package name (e.g., paiml-mcp-agent-toolkit → pmat).
fn resolve_contracts_dir(project_path: &Path) -> Option<std::path::PathBuf> {
    // Prefer sibling provable-contracts repo — contains only provable-contracts YAMLs.
    // Local contracts/ may contain pmat work contracts (different schema) that pv lint
    // cannot parse.
    let abs = std::fs::canonicalize(project_path).ok()?;
    let parent = abs.parent()?;
    let pv_contracts = parent.join("provable-contracts").join("contracts");
    if pv_contracts.exists() {
        // Try 1: directory name
        let dir_name = abs.file_name()?.to_str()?;
        let sibling = pv_contracts.join(dir_name);
        if sibling.exists() {
            return Some(sibling);
        }
        // Try 2: Cargo.toml package name (handles paiml-mcp-agent-toolkit → pmat)
        let cargo_toml = project_path.join("Cargo.toml");
        if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("name") && trimmed.contains('=') {
                    if let Some(name) = trimmed.split('=').nth(1) {
                        let pkg = name.trim().trim_matches('"');
                        let by_pkg = pv_contracts.join(pkg);
                        if by_pkg.exists() {
                            return Some(by_pkg);
                        }
                    }
                    break;
                }
            }
        }
    }
    // Fallback: local contracts/ if it has provable-contracts YAMLs
    let local = project_path.join("contracts");
    if local.exists() && has_contract_yamls(&local) {
        return Some(local);
    }
    None
}

/// Collect contract YAML stems from the project's contracts directory (recursive).
/// Returns a set of stems (e.g., "softmax-kernel-v1") for filtering proof-status.json.
/// Checks local `contracts/` then sibling `../provable-contracts/contracts/<project>/`.
fn collect_project_contract_stems(project_path: &Path) -> std::collections::HashSet<String> {
    let mut stems = std::collections::HashSet::new();
    if let Some(contracts_dir) = resolve_contracts_dir(project_path) {
        collect_stems_recursive(&contracts_dir, &mut stems);
    }
    stems
}

fn collect_stems_recursive(dir: &Path, stems: &mut std::collections::HashSet<String>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            collect_stems_recursive(&path, stems);
        } else if path.extension().is_some_and(|e| e == "yaml" || e == "yml") {
            // Skip binding files
            if path
                .file_name()
                .is_some_and(|n| n.to_string_lossy().contains("binding"))
            {
                continue;
            }
            if let Some(stem) = path.file_stem() {
                stems.insert(stem.to_string_lossy().into_owned());
            }
        }
    }
}


/// Detect if build.rs has contract enforcement (reads binding.yaml or contracts/)
fn detect_buildrs_enforcement(project_path: &Path) -> bool {
    // Check root build.rs
    let build_rs = project_path.join("build.rs");
    if build_rs.exists() {
        if let Ok(c) = std::fs::read_to_string(&build_rs) {
            if c.contains("binding") || c.contains("contract") || c.contains("AllImplemented") {
                return true;
            }
        }
    }
    // Check workspace member crates (crates/*/build.rs)
    if let Ok(entries) = std::fs::read_dir(project_path.join("crates")) {
        for entry in entries.flatten() {
            let member_build = entry.path().join("build.rs");
            if member_build.exists() {
                if let Ok(c) = std::fs::read_to_string(&member_build) {
                    if c.contains("binding")
                        || c.contains("contract")
                        || c.contains("AllImplemented")
                    {
                        return true;
                    }
                }
            }
        }
    }
    false
}


/// Resolve binding.yaml file locations (local + sibling provable-contracts)
fn resolve_binding_files(
    contracts_dir: &Path,
    abs_path: &Path,
    project_name: &str,
) -> Vec<std::path::PathBuf> {
    let mut files: Vec<std::path::PathBuf> = Vec::new();
    for entry in walkdir::WalkDir::new(contracts_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.path().is_file()
            && entry
                .path()
                .file_name()
                .is_some_and(|n| n.to_string_lossy().contains("binding"))
        {
            files.push(entry.path().to_path_buf());
        }
    }
    let has_implemented = files.iter().any(|f| {
        std::fs::read_to_string(f)
            .map(|c| c.contains("status: implemented"))
            .unwrap_or(false)
    });
    if files.is_empty() || !has_implemented {
        if let Some(sb) = abs_path.parent().map(|p| {
            p.join("provable-contracts")
                .join("contracts")
                .join(project_name)
                .join("binding.yaml")
        }) {
            if sb.exists() {
                files.push(sb);
            }
        }
    }
    files
}

/// Collect all fn/const/static names from Rust source files
fn collect_known_fn_names(
    project_path: &Path,
) -> Option<std::collections::HashSet<String>> {
    let src_dir = project_path.join("src");
    let crates_dir = project_path.join("crates");
    let mut search_dirs: Vec<std::path::PathBuf> = Vec::new();
    // Search both src/ and crates/*/src/ — workspaces have both
    if src_dir.exists() {
        search_dirs.push(src_dir);
    }
    if crates_dir.exists() {
        for entry in std::fs::read_dir(&crates_dir).into_iter().flatten().flatten() {
            let s = entry.path().join("src");
            if s.exists() {
                search_dirs.push(s);
            }
        }
    }
    if search_dirs.is_empty() {
        return None;
    }

    let mut known = std::collections::HashSet::new();
    for sdir in &search_dirs {
        for entry in walkdir::WalkDir::new(sdir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if !entry.path().extension().is_some_and(|e| e == "rs") {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                extract_names_from_source(&content, &mut known);
            }
        }
    }
    Some(known)
}

/// Extract fn/const/static names from a single Rust source file
fn extract_names_from_source(content: &str, names: &mut std::collections::HashSet<String>) {
    for line in content.lines() {
        let t = line.trim();
        // fn declarations (all visibility/async/unsafe/const variants)
        let fn_rest = t
            .strip_prefix("pub fn ")
            .or_else(|| t.strip_prefix("fn "))
            .or_else(|| t.strip_prefix("pub async fn "))
            .or_else(|| t.strip_prefix("pub(crate) fn "))
            .or_else(|| t.strip_prefix("async fn "))
            .or_else(|| t.strip_prefix("unsafe fn "))
            .or_else(|| t.strip_prefix("pub unsafe fn "))
            .or_else(|| t.strip_prefix("pub(crate) unsafe fn "))
            .or_else(|| t.strip_prefix("pub(crate) async fn "))
            .or_else(|| t.strip_prefix("pub const unsafe fn "))
            .or_else(|| t.strip_prefix("const fn "))
            .or_else(|| t.strip_prefix("pub const fn "));
        if let Some(rest) = fn_rest {
            if let Some(name) = rest.split('(').next() {
                let name = name.trim();
                if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    names.insert(name.to_string());
                }
            }
        }
        // const/static declarations
        let const_rest = t
            .strip_prefix("pub const ")
            .or_else(|| t.strip_prefix("pub(crate) const "))
            .or_else(|| t.strip_prefix("const "))
            .or_else(|| t.strip_prefix("pub static "))
            .or_else(|| t.strip_prefix("static "));
        if let Some(rest) = const_rest {
            if let Some(name) = rest.split(':').next() {
                let name = name.trim();
                if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    names.insert(name.to_string());
                }
            }
        }
    }
}

/// Cross-reference binding entries against known fn names
fn cross_reference_bindings(
    entries: &[(String, String)],
    known_fns: &std::collections::HashSet<String>,
) -> (usize, usize, usize, Vec<String>) {
    let total = entries.len();
    let mut missing = Vec::new();
    let mut verified = 0usize;
    let mut seen = std::collections::HashSet::new();
    for (func, _) in entries {
        if !seen.insert(func.clone()) {
            continue;
        }
        if known_fns.contains(func.as_str()) {
            verified += 1;
        } else if missing.len() < 5 {
            missing.push(func.clone());
        }
    }
    (total, seen.len(), verified, missing)
}

/// Parse binding entries from a binding.yaml file content
fn parse_binding_entries(
    content: &str,
    path: &Path,
    contracts_dir: &Path,
    entries: &mut Vec<(String, String)>,
) {
    let mut current_fn = None;
    let mut current_status = None;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("function:") {
            let val = trimmed
                .strip_prefix("function:")
                .unwrap_or("")
                .trim()
                .trim_matches('"');
            // Extract bare function name (strip Type:: prefix)
            let bare = if let Some(pos) = val.rfind("::") {
                &val[pos + 2..]
            } else {
                val
            };
            current_fn = Some(bare.to_string());
        } else if trimmed.starts_with("status:") {
            current_status = Some(
                trimmed
                    .strip_prefix("status:")
                    .unwrap_or("")
                    .trim()
                    .to_string(),
            );
        } else if trimmed.starts_with("- contract:") || trimmed.is_empty() {
            if let (Some(func), Some(status)) = (current_fn.take(), current_status.take()) {
                if status == "implemented" && !func.is_empty() {
                    let rel = path
                        .strip_prefix(contracts_dir)
                        .unwrap_or(path)
                        .display()
                        .to_string();
                    entries.push((func, rel));
                }
            }
        }
    }
    // Handle last entry
    if let (Some(func), Some(status)) = (current_fn, current_status) {
        if status == "implemented" && !func.is_empty() {
            let rel = path
                .strip_prefix(contracts_dir)
                .unwrap_or(path)
                .display()
                .to_string();
            entries.push((func, rel));
        }
    }
}


fn count_contract_test_refs(project_path: &Path) -> (usize, usize, usize) {
    let contracts_dir = project_path.join("contracts");
    let src_dir = project_path.join("src");
    let mut refs = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&contracts_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().map_or(true, |e| e != "yaml" && e != "yml") {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(&p) {
                for line in content.lines() {
                    if let Some(pos) = line.find("test:") {
                        let rest = line[pos + 5..].trim().trim_matches('"');
                        let name = rest
                            .split(|c: char| !c.is_alphanumeric() && c != '_')
                            .next()
                            .unwrap_or("");
                        if name.starts_with("test_") || name.starts_with("prop_") {
                            refs.push(name.to_string());
                        }
                    }
                }
            }
        }
    }

    if refs.is_empty() {
        return (0, 0, 0);
    }

    let mut src_tests = std::collections::HashSet::new();
    // Scan src/, tests/, crates/, and workspace member directories
    let mut scan_dirs = vec![src_dir.clone()];
    scan_dirs.push(project_path.join("tests"));
    scan_dirs.push(project_path.join("crates"));
    // Workspace members: root-level dirs with Cargo.toml
    if let Ok(entries) = std::fs::read_dir(project_path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() && p.join("Cargo.toml").exists() {
                let name = p.file_name().unwrap_or_default();
                if name != "src" && name != "crates" && name != "tests" && name != "contracts" {
                    scan_dirs.push(p.join("src"));
                    scan_dirs.push(p.join("tests"));
                }
            }
        }
    }
    for dir in &scan_dirs {
        if !dir.exists() {
            continue;
        }
        for entry in walkdir::WalkDir::new(dir).into_iter().flatten() {
            if entry.path().extension().map_or(true, |e| e != "rs") {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                for line in content.lines() {
                    if let Some(pos) = line.find("fn test_").or_else(|| line.find("fn prop_")) {
                        let rest = &line[pos + 3..];
                        let name = rest
                            .split(|c: char| !c.is_alphanumeric() && c != '_')
                            .next()
                            .unwrap_or("");
                        if !name.is_empty() {
                            src_tests.insert(name.to_string());
                        }
                    }
                }
            }
        }
    }

    let existing = refs
        .iter()
        .filter(|t| src_tests.contains(t.as_str()))
        .count();
    let missing = refs.len() - existing;
    (refs.len(), existing, missing)
}

