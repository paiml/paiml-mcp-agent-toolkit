
/// Extract numeric level from "target_level: L3" or "current_level: L1" patterns.
/// Count `{` and `}` on a line, ignoring those inside string/char literals.
/// Handles Rust string forms: "...", '...', r"...", r#"..."#, r##"..."## etc.
/// Returns (open_count, close_count).
fn count_braces_outside_literals(line: &str) -> (i64, i64) {
    let mut opens = 0i64;
    let mut closes = 0i64;
    let mut in_string = false;
    let mut in_char = false;
    let mut escape = false;
    let mut raw_hashes = 0usize; // For raw strings, number of # delimiters; 0 = not raw
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        // Detect start of raw string: r#...#"  or r"
        if !in_string && !in_char && b == b'r' {
            // Look ahead for ' or " preceded by optional #s
            let mut j = i + 1;
            let mut hashes = 0;
            while j < bytes.len() && bytes[j] == b'#' {
                hashes += 1;
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'"' {
                in_string = true;
                raw_hashes = hashes;
                i = j + 1;
                continue;
            }
        }
        if escape {
            escape = false;
            i += 1;
            continue;
        }
        if b == b'\\' && (in_string || in_char) && raw_hashes == 0 {
            escape = true;
            i += 1;
            continue;
        }
        if in_string && b == b'"' {
            // Regular string ends at unescaped "
            if raw_hashes == 0 {
                in_string = false;
            } else {
                // Raw string ends at " followed by raw_hashes #s
                let mut k = i + 1;
                let mut matched = 0;
                while k < bytes.len() && matched < raw_hashes && bytes[k] == b'#' {
                    matched += 1;
                    k += 1;
                }
                if matched == raw_hashes {
                    in_string = false;
                    raw_hashes = 0;
                    i = k;
                    continue;
                }
            }
        } else if !in_string && !in_char && b == b'"' {
            in_string = true;
        } else if b == b'\'' && !in_string {
            in_char = !in_char;
        } else if !in_string && !in_char {
            if b == b'{' { opens += 1; }
            else if b == b'}' { closes += 1; }
        }
        i += 1;
    }
    (opens, closes)
}

fn extract_level(content: &str, field: &str) -> Option<u8> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(field) {
            // Parse "target_level: L3" or "target_level: \"L3\""
            if let Some(val) = trimmed.split(':').nth(1) {
                let val = val.trim().trim_matches('"').trim_matches('\'');
                if let Some(digit) = val.strip_prefix('L') {
                    return digit.parse().ok();
                }
                // Also try plain number
                return val.parse().ok();
            }
        }
    }
    None
}

/// CB-1338: No Ghost Bindings
///
/// Checks binding.yaml entries reference functions that exist in source.
/// A "ghost binding" is a binding.yaml entry for a function that doesn't exist.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_no_ghost_bindings(project_path: &Path) -> ComplianceCheck {
    let binding = project_path.join("binding.yaml");
    if !binding.exists() {
        // Also check contracts/ subdirectory
        let contracts_binding = project_path.join("contracts/binding.yaml");
        if !contracts_binding.exists() {
            return ComplianceCheck {
                name: "CB-1338: No Ghost Bindings".into(),
                status: CheckStatus::Skip,
                message: "No binding.yaml found".into(),
                severity: Severity::Info,
            };
        }
    }

    // Count binding entries and check if source files exist
    let binding_path = if project_path.join("binding.yaml").exists() {
        project_path.join("binding.yaml")
    } else {
        project_path.join("contracts/binding.yaml")
    };

    let content = match fs::read_to_string(&binding_path) {
        Ok(c) => c,
        Err(_) => {
            return ComplianceCheck {
                name: "CB-1338: No Ghost Bindings".into(),
                status: CheckStatus::Warn,
                message: "Could not read binding.yaml".into(),
                severity: Severity::Warning,
            };
        }
    };

    let mut total_bindings = 0usize;
    let mut ghost_count = 0usize;

    // Parse binding entries — look for "status: implemented" with source file refs
    let mut current_source: Option<String> = None;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("source_file:") || trimmed.starts_with("file:") {
            if let Some(val) = trimmed.split(':').nth(1) {
                current_source = Some(val.trim().trim_matches('"').trim_matches('\'').to_string());
            }
        }
        if trimmed.starts_with("status:") && trimmed.contains("implemented") {
            total_bindings += 1;
            if let Some(ref src) = current_source {
                let src_path = project_path.join(src);
                if !src_path.exists() {
                    ghost_count += 1;
                }
            }
        }
        if trimmed.starts_with("- name:") || trimmed.starts_with("- module_path:") {
            current_source = None;
        }
    }

    if total_bindings == 0 {
        ComplianceCheck {
            name: "CB-1338: No Ghost Bindings".into(),
            status: CheckStatus::Pass,
            message: "No implemented bindings to verify".into(),
            severity: Severity::Info,
        }
    } else if ghost_count == 0 {
        ComplianceCheck {
            name: "CB-1338: No Ghost Bindings".into(),
            status: CheckStatus::Pass,
            message: format!("{} binding(s) verified, 0 ghosts", total_bindings),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-1338: No Ghost Bindings".into(),
            status: CheckStatus::Warn,
            message: format!(
                "{}/{} binding(s) are ghosts (source files missing)",
                ghost_count, total_bindings
            ),
            severity: Severity::Warning,
        }
    }
}

/// CB-1339: No Placeholder Preconditions
///
/// Checks contracts for generic placeholder preconditions like !is_empty().
/// Domain-specific equations should have real preconditions, not boilerplate.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_no_placeholder_preconditions(project_path: &Path) -> ComplianceCheck {
    let contracts_dir = project_path.join("contracts");
    if !contracts_dir.exists() {
        return ComplianceCheck {
            name: "CB-1339: No Placeholder Preconditions".into(),
            status: CheckStatus::Skip,
            message: "No contracts/ directory".into(),
            severity: Severity::Info,
        };
    }

    let placeholders = [
        "!input.is_empty()",
        "!x.is_empty()",
        "input.len() > 0",
        "x.len() > 0",
        "!is_empty()",
    ];

    let mut total_preconditions = 0usize;
    let mut placeholder_count = 0usize;

    for entry in walkdir::WalkDir::new(&contracts_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() || path.extension().map_or(true, |e| e != "yaml" && e != "yml") {
            continue;
        }
        if let Ok(content) = fs::read_to_string(path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("precondition") || trimmed.starts_with("- \"") || trimmed.starts_with("- '") {
                    if placeholders.iter().any(|p| trimmed.contains(p)) {
                        placeholder_count += 1;
                    }
                    if trimmed.contains("precondition") || (trimmed.starts_with("- ") && trimmed.len() > 5) {
                        total_preconditions += 1;
                    }
                }
            }
        }
    }

    if total_preconditions == 0 {
        ComplianceCheck {
            name: "CB-1339: No Placeholder Preconditions".into(),
            status: CheckStatus::Pass,
            message: "No preconditions to check".into(),
            severity: Severity::Info,
        }
    } else if placeholder_count == 0 {
        ComplianceCheck {
            name: "CB-1339: No Placeholder Preconditions".into(),
            status: CheckStatus::Pass,
            message: format!("{} precondition(s), 0 placeholders", total_preconditions),
            severity: Severity::Info,
        }
    } else {
        let ratio = placeholder_count as f64 / total_preconditions.max(1) as f64;
        ComplianceCheck {
            name: "CB-1339: No Placeholder Preconditions".into(),
            status: if ratio > 0.5 { CheckStatus::Fail } else { CheckStatus::Warn },
            message: format!(
                "{}/{} precondition(s) are placeholders ({:.0}%)",
                placeholder_count,
                total_preconditions,
                ratio * 100.0
            ),
            severity: if ratio > 0.5 { Severity::Error } else { Severity::Warning },
        }
    }
}

/// Per-crate enforcement measurement result.
struct CratePenetration {
    name: String,
    call_sites: usize,
    total_fns: usize,
    is_cli: bool,
}

/// Count contract enforcement call sites (debug_assert!, contract macros) in a directory tree.
fn count_enforcement(dir: &Path, calls: &mut usize, fns: &mut usize) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() && !path.to_str().unwrap_or("").contains("test") {
            count_enforcement(&path, calls, fns);
        } else if path.extension().is_some_and(|e| e == "rs") {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.contains("test") || name.contains("_tests") {
                continue;
            }
            if let Ok(content) = fs::read_to_string(&path) {
                count_enforcement_in_source(&content, calls, fns);
            }
        }
    }
}

/// Count fn definitions and enforcement call sites in source text, skipping test modules.
fn count_enforcement_in_source(content: &str, calls: &mut usize, fns: &mut usize) {
    let mut pending_test = false;
    let mut in_test_module = false;
    let mut brace_depth_at_test = 0i32;
    let mut brace_depth = 0i32;
    for line in content.lines() {
        let t = line.trim();
        if t.contains("#[cfg(test)]") { pending_test = true; }
        let old_depth = brace_depth;
        let (opens, closes) = count_braces_outside_literals(line);
        brace_depth += (opens - closes) as i32;
        if pending_test && brace_depth > old_depth {
            in_test_module = true;
            pending_test = false;
            brace_depth_at_test = old_depth;
        }
        if in_test_module && brace_depth <= brace_depth_at_test { in_test_module = false; }
        if in_test_module { continue; }
        if t.starts_with("//") || t.starts_with("///") || t.starts_with("/*") || t.starts_with("*") {
            continue;
        }
        if is_fn_definition(t) { *fns += 1; }
        if is_enforcement_call(line) { *calls += 1; }
    }
}

/// Check if a trimmed line is a Rust function definition.
fn is_fn_definition(t: &str) -> bool {
    const PREFIXES: &[&str] = &[
        "fn ", "pub fn ", "async fn ", "pub async fn ",
        "const fn ", "pub const fn ", "unsafe fn ", "pub unsafe fn ",
        "pub(crate) fn ", "pub(super) fn ",
        "pub(crate) async fn ", "pub(crate) const fn ", "pub(crate) unsafe fn ",
    ];
    PREFIXES.iter().any(|p| t.starts_with(p)) && (t.contains('(') || t.contains('<'))
}

/// Check if a line contains contract enforcement patterns.
/// Matches actual enforcement macros, not data field names like contract_value.
fn is_enforcement_call(line: &str) -> bool {
    line.contains("debug_assert!") || line.contains("contract_pre_")
        || line.contains("contract_post_") || line.contains("#[contract")
        || line.contains("::contract(") || line.contains("requires!")
        || line.contains("ensures!")
}

/// Measure per-crate penetration for workspace projects.
fn measure_workspace_crates(project_path: &Path, members: &[String]) -> Vec<CratePenetration> {
    let mut results = Vec::new();
    // Root crate (the "." member)
    let root_src = project_path.join("src");
    if root_src.exists() {
        let mut calls = 0usize;
        let mut fns = 0usize;
        count_enforcement(&root_src, &mut calls, &mut fns);
        let pkg_name = project_path.file_name()
            .and_then(|n| n.to_str()).unwrap_or("root").to_string();
        let is_cli = pkg_name.ends_with("-cli") || pkg_name == "cli";
        results.push(CratePenetration { name: pkg_name, call_sites: calls, total_fns: fns, is_cli });
    }
    for member in members {
        if member == "." { continue; }
        let member_src = project_path.join(member).join("src");
        if !member_src.exists() { continue; }
        let mut calls = 0usize;
        let mut fns = 0usize;
        count_enforcement(&member_src, &mut calls, &mut fns);
        if fns == 0 { continue; }
        let crate_name = Path::new(member).file_name()
            .and_then(|n| n.to_str()).unwrap_or(member).to_string();
        let is_cli = crate_name.ends_with("-cli") || crate_name == "cli";
        results.push(CratePenetration { name: crate_name, call_sites: calls, total_fns: fns, is_cli });
    }
    results
}

/// Build per-crate detail string for CB-1340 message.
fn format_per_crate_detail(crates: &[CratePenetration]) -> String {
    if crates.len() <= 1 { return String::new(); }
    let parts: Vec<String> = crates.iter()
        .filter(|cr| cr.total_fns > 0)
        .map(|cr| {
            let pct = cr.call_sites as f64 / cr.total_fns as f64 * 100.0;
            let marker = if cr.is_cli { " [CLI]" } else { "" };
            format!("{}:{:.0}%{}", cr.name, pct, marker)
        })
        .collect();
    format!(" | per-crate: {}", parts.join(", "))
}

/// Find crates failing penetration thresholds.
/// CLI crates (*-cli): ≥95%. Significant crates (≥50 fns): ≥10%.
/// Small/bench crates: skip.
fn find_failing_crates(crates: &[CratePenetration]) -> (Vec<String>, Vec<String>) {
    let mut cli_fails = Vec::new();
    let mut non_cli_fails = Vec::new();
    for cr in crates {
        if cr.total_fns == 0 { continue; }
        let is_bench = cr.name.contains("bench");
        let is_small = cr.total_fns < 50;
        if !cr.is_cli && (is_bench || is_small) { continue; }
        let pct = cr.call_sites as f64 / cr.total_fns as f64 * 100.0;
        let threshold = if cr.is_cli { 95.0 } else { 10.0 };
        if pct < threshold {
            let msg = format!("{}: {:.1}% (need ≥{:.0}%)", cr.name, pct, threshold);
            if cr.is_cli { cli_fails.push(msg); } else { non_cli_fails.push(msg); }
        }
    }
    (cli_fails, non_cli_fails)
}
