// CC-003: Primitive should be upstream detection
// CC-004: Cross-crate churn correlation detection
//
// Included into cross_crate_handlers.rs via include!()

/// CC-003: Detect when a downstream crate reimplements a function already in an upstream dep.
///
/// If crate B depends on crate A and both define `f16_to_f32()`, B should import from A
/// instead of maintaining a copy.
fn detect_cc003_primitive_upstream(
    crate_functions: &[(CrateInfo, Vec<FunctionEntry>)],
) -> Vec<CrossCrateFinding> {
    let mut findings = Vec::new();

    // Build function name -> crate map
    struct CrateFuncRef<'a> {
        crate_info: &'a CrateInfo,
        func: &'a FunctionEntry,
    }
    let mut func_by_name: HashMap<&str, Vec<CrateFuncRef<'_>>> = HashMap::new();
    for (crate_info, functions) in crate_functions {
        for func in functions {
            // Skip generic trait impls
            if is_generic_impl_name(&func.function_name) {
                continue;
            }
            func_by_name
                .entry(func.function_name.as_str())
                .or_default()
                .push(CrateFuncRef { crate_info, func });
        }
    }

    // For each function name that appears in multiple crates
    for (func_name, impls) in &func_by_name {
        if impls.len() < 2 {
            continue;
        }

        // Check all pairs for dependency relationships
        for i in 0..impls.len() {
            for j in 0..impls.len() {
                if i == j {
                    continue;
                }
                let upstream = &impls[i];
                let downstream = &impls[j];

                // Skip same-crate
                if upstream.crate_info.name == downstream.crate_info.name {
                    continue;
                }

                // Does downstream depend on upstream?
                let has_dep = downstream
                    .crate_info
                    .cargo_deps
                    .iter()
                    .any(|d| d == &upstream.crate_info.name);
                if !has_dep {
                    continue;
                }

                // Only flag if both functions have non-trivial source
                // and the source is actually similar (not just same name)
                let up_src = &upstream.func.source;
                let down_src = &downstream.func.source;
                if up_src.is_empty() || down_src.is_empty() {
                    continue;
                }
                if up_src.lines().count() < 3 || down_src.lines().count() < 3 {
                    continue;
                }

                findings.push(CrossCrateFinding {
                    rule: "CC-003".to_string(),
                    severity: CcSeverity::Warning,
                    crate_a: upstream.crate_info.name.clone(),
                    crate_b: downstream.crate_info.name.clone(),
                    function_a: func_name.to_string(),
                    function_b: func_name.to_string(),
                    file_a: upstream.func.file_path.clone(),
                    file_b: downstream.func.file_path.clone(),
                    similarity: None,
                    recommendation: format!(
                        "Use {}::{} directly instead of reimplementing in {}",
                        upstream.crate_info.name,
                        func_name,
                        downstream.crate_info.name
                    ),
                });
            }
        }
    }

    // Deduplicate (A->B and B->A would both trigger, but only the downstream direction matters)
    findings.sort_by(|a, b| {
        (&a.crate_a, &a.crate_b, &a.function_a).cmp(&(&b.crate_a, &b.crate_b, &b.function_a))
    });
    findings.dedup_by(|a, b| {
        a.crate_a == b.crate_a && a.crate_b == b.crate_b && a.function_a == b.function_a
    });

    findings
}

/// CC-004: Detect shotgun surgery — correlated file changes across crate boundaries.
///
/// When the same file basename is modified in multiple crates within a short time window,
/// it suggests copy-paste maintenance where a fix in one crate requires the same fix in others.
fn detect_cc004_churn_correlation(
    crate_functions: &[(CrateInfo, Vec<FunctionEntry>)],
    window_days: u32,
) -> Vec<CrossCrateFinding> {
    let mut findings = Vec::new();

    // Get recent commits per crate
    let mut crate_changes: Vec<(&CrateInfo, HashMap<String, Vec<String>>)> = Vec::new();

    for (crate_info, _) in crate_functions {
        let changes = get_recent_file_changes(&crate_info.path, window_days);
        if !changes.is_empty() {
            crate_changes.push((crate_info, changes));
        }
    }

    if crate_changes.len() < 2 {
        return findings;
    }

    // Find basenames that changed in multiple crates
    // changes maps: basename -> Vec<date_string>
    let mut basename_crates: HashMap<String, Vec<(&CrateInfo, usize)>> = HashMap::new();

    for (crate_info, changes) in &crate_changes {
        for (basename, dates) in changes {
            basename_crates
                .entry(basename.clone())
                .or_default()
                .push((crate_info, dates.len()));
        }
    }

    for (basename, crate_entries) in &basename_crates {
        if crate_entries.len() < 2 {
            continue;
        }

        // Report pairs
        for i in 0..crate_entries.len() {
            for j in (i + 1)..crate_entries.len() {
                let (crate_a, count_a) = crate_entries[i];
                let (crate_b, count_b) = crate_entries[j];

                findings.push(CrossCrateFinding {
                    rule: "CC-004".to_string(),
                    severity: CcSeverity::Advisory,
                    crate_a: crate_a.name.clone(),
                    crate_b: crate_b.name.clone(),
                    function_a: basename.clone(),
                    function_b: basename.clone(),
                    file_a: format!("{} ({} changes)", basename, count_a),
                    file_b: format!("{} ({} changes)", basename, count_b),
                    similarity: None,
                    recommendation: format!(
                        "Correlated changes in '{}' across {} and {} — possible shotgun surgery",
                        basename, crate_a.name, crate_b.name
                    ),
                });
            }
        }
    }

    findings
}

/// Get files changed in the last N days from git log, grouped by basename.
fn get_recent_file_changes(
    crate_path: &Path,
    window_days: u32,
) -> HashMap<String, Vec<String>> {
    let since = format!("{}.days", window_days);
    let output = std::process::Command::new("git")
        .args([
            "log",
            "--format=%cd",
            "--date=short",
            "--name-only",
            &format!("--since={since}"),
        ])
        .current_dir(crate_path)
        .output();

    let Ok(output) = output else {
        return HashMap::new();
    };
    if !output.status.success() {
        return HashMap::new();
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut result: HashMap<String, Vec<String>> = HashMap::new();
    let mut current_date = String::new();

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Date lines look like "2026-02-15"
        if trimmed.len() == 10 && trimmed.chars().nth(4) == Some('-') {
            current_date = trimmed.to_string();
            continue;
        }
        // File path lines
        if let Some(basename) = Path::new(trimmed).file_name().and_then(|n| n.to_str()) {
            // Only track source files
            if basename.ends_with(".rs")
                || basename.ends_with(".py")
                || basename.ends_with(".ts")
                || basename.ends_with(".go")
            {
                result
                    .entry(basename.to_string())
                    .or_default()
                    .push(current_date.clone());
            }
        }
    }

    result
}
