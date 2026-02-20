// CC-003: Primitive should be upstream detection (with MinHash similarity gate)
// CC-004: Cross-crate churn correlation detection
//
// Included into cross_crate_handlers.rs via include!()

struct CrateFuncRef<'a> {
    crate_info: &'a CrateInfo,
    func: &'a FunctionEntry,
}

/// CC-003: Detect when a downstream crate reimplements a function already in an upstream dep.
///
/// Uses MinHash similarity to reduce false positives: only flags when the source code
/// is actually similar (Jaccard >= cc003_min_similarity), not just same-named functions.
fn detect_cc003_primitive_upstream(
    crate_functions: &[(CrateInfo, Vec<FunctionEntry>)],
    config: &DetectionConfig,
) -> Vec<CrossCrateFinding> {
    let signatures = precompute_cc003_signatures(crate_functions, config);
    let func_by_name = build_func_by_name(crate_functions, config);

    let mut findings: Vec<CrossCrateFinding> = func_by_name
        .iter()
        .filter(|(_, impls)| impls.len() >= 2)
        .flat_map(|(func_name, impls)| check_cc003_pairs(func_name, impls, config, &signatures))
        .collect();

    // Deduplicate (A->B and B->A would both trigger, only downstream direction matters)
    findings.sort_by(|a, b| {
        (&a.crate_a, &a.crate_b, &a.function_a).cmp(&(&b.crate_a, &b.crate_b, &b.function_a))
    });
    findings.dedup_by(|a, b| {
        a.crate_a == b.crate_a && a.crate_b == b.crate_b && a.function_a == b.function_a
    });

    findings
}

/// Build function name -> crate references map, excluding generic names.
fn build_func_by_name<'a>(
    crate_functions: &'a [(CrateInfo, Vec<FunctionEntry>)],
    config: &DetectionConfig,
) -> HashMap<&'a str, Vec<CrateFuncRef<'a>>> {
    let mut map: HashMap<&str, Vec<CrateFuncRef<'_>>> = HashMap::new();
    for (crate_info, functions) in crate_functions {
        for func in functions {
            if !is_excluded_function(&func.function_name, config) {
                map.entry(func.function_name.as_str())
                    .or_default()
                    .push(CrateFuncRef { crate_info, func });
            }
        }
    }
    map
}

/// Check all upstream/downstream pairs for a given function name.
fn check_cc003_pairs(
    func_name: &str,
    impls: &[CrateFuncRef<'_>],
    config: &DetectionConfig,
    signatures: &HashMap<(&str, &str, &str), MinHashSignature>,
) -> Vec<CrossCrateFinding> {
    let mut findings = Vec::new();

    for i in 0..impls.len() {
        for j in 0..impls.len() {
            if i == j {
                continue;
            }
            if let Some(finding) =
                check_cc003_single_pair(func_name, &impls[i], &impls[j], config, signatures)
            {
                findings.push(finding);
            }
        }
    }

    findings
}

/// Check a single upstream/downstream pair for CC-003 violation.
fn check_cc003_single_pair(
    func_name: &str,
    upstream: &CrateFuncRef<'_>,
    downstream: &CrateFuncRef<'_>,
    config: &DetectionConfig,
    signatures: &HashMap<(&str, &str, &str), MinHashSignature>,
) -> Option<CrossCrateFinding> {
    if upstream.crate_info.name == downstream.crate_info.name {
        return None;
    }
    if is_crate_pair_excluded(
        &upstream.crate_info.name,
        &downstream.crate_info.name,
        &config.excluded_crate_pairs,
    ) {
        return None;
    }

    // Downstream must depend on upstream
    let has_dep = downstream
        .crate_info
        .cargo_deps
        .iter()
        .any(|d| d == &upstream.crate_info.name);
    if !has_dep {
        return None;
    }

    // Both must have non-trivial source
    let up_src = &upstream.func.source;
    let down_src = &downstream.func.source;
    if up_src.is_empty()
        || down_src.is_empty()
        || up_src.lines().count() < config.min_body_lines
        || down_src.lines().count() < config.min_body_lines
    {
        return None;
    }

    // MinHash similarity gate
    let similarity = compute_cc003_similarity(
        upstream.crate_info.name.as_str(),
        upstream.func.file_path.as_str(),
        downstream.crate_info.name.as_str(),
        downstream.func.file_path.as_str(),
        func_name,
        config.cc003_min_similarity,
        signatures,
    )?;

    Some(CrossCrateFinding {
        rule: "CC-003".to_string(),
        severity: CcSeverity::Warning,
        crate_a: upstream.crate_info.name.clone(),
        crate_b: downstream.crate_info.name.clone(),
        function_a: func_name.to_string(),
        function_b: func_name.to_string(),
        file_a: upstream.func.file_path.clone(),
        file_b: downstream.func.file_path.clone(),
        similarity,
        recommendation: format!(
            "Use {}::{} directly instead of reimplementing in {}",
            upstream.crate_info.name, func_name, downstream.crate_info.name
        ),
    })
}

/// Compute MinHash similarity between two function implementations.
/// Returns `Some(Some(sim))` if similar enough, `Some(None)` for name-only match,
/// `None` if below threshold.
fn compute_cc003_similarity(
    up_crate: &str,
    up_file: &str,
    down_crate: &str,
    down_file: &str,
    func_name: &str,
    min_similarity: f64,
    signatures: &HashMap<(&str, &str, &str), MinHashSignature>,
) -> Option<Option<f64>> {
    let up_key = (up_crate, up_file, func_name);
    let down_key = (down_crate, down_file, func_name);

    match (signatures.get(&up_key), signatures.get(&down_key)) {
        (Some(up_sig), Some(down_sig)) => {
            let sim = up_sig.jaccard_similarity(down_sig);
            if sim < min_similarity {
                None // Below threshold
            } else {
                Some(Some(sim))
            }
        }
        _ => Some(None), // No signatures — name-only match
    }
}

/// Pre-compute MinHash signatures for CC-003 similarity gating.
/// Key: (crate_name, file_path, function_name) -> MinHashSignature
fn precompute_cc003_signatures<'a>(
    crate_functions: &'a [(CrateInfo, Vec<FunctionEntry>)],
    config: &DetectionConfig,
) -> HashMap<(&'a str, &'a str, &'a str), MinHashSignature> {
    let dup_config = DuplicateDetectionConfig {
        normalize_identifiers: true,
        normalize_literals: true,
        ignore_comments: true,
        ..Default::default()
    };
    let extractor = UniversalFeatureExtractor::new(dup_config);
    let hasher = MinHashGenerator::new(128);

    let mut sigs = HashMap::new();

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
            sigs.insert(
                (
                    crate_info.name.as_str(),
                    func.file_path.as_str(),
                    func.function_name.as_str(),
                ),
                minhash,
            );
        }
    }

    sigs
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
