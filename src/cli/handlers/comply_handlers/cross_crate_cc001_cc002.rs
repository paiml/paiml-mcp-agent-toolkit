// CC-001: Cross-crate function clone detection
// CC-002: API signature divergence detection
//
// Included into cross_crate_handlers.rs via include!()

/// CC-001: Detect functions that are copy-pasted across crate boundaries.
///
/// Uses MinHash signatures on function source to find near-duplicates.
/// Only reports cross-crate pairs (same-crate duplicates are handled by
/// `pmat comply check` CB pattern detection).
fn detect_cc001_function_clones(
    crate_functions: &[(CrateInfo, Vec<FunctionEntry>)],
    threshold: f64,
    config: &DetectionConfig,
) -> Vec<CrossCrateFinding> {
    let signed = compute_signatures(crate_functions, config);
    if signed.len() < 2 {
        return Vec::new();
    }

    let mut findings = Vec::new();

    // Compare all cross-crate pairs
    // Group by crate for efficient pair enumeration
    let mut by_crate: HashMap<&str, Vec<usize>> = HashMap::new();
    for (idx, sf) in signed.iter().enumerate() {
        by_crate.entry(&sf.crate_name).or_default().push(idx);
    }

    let crate_names: Vec<&str> = by_crate.keys().copied().collect();

    for i in 0..crate_names.len() {
        for j in (i + 1)..crate_names.len() {
            // Skip excluded crate pairs
            if is_crate_pair_excluded(
                crate_names[i],
                crate_names[j],
                &config.excluded_crate_pairs,
            ) {
                continue;
            }

            let indices_a = &by_crate[crate_names[i]];
            let indices_b = &by_crate[crate_names[j]];

            for &idx_a in indices_a {
                for &idx_b in indices_b {
                    let sim = signed[idx_a].minhash.jaccard_similarity(&signed[idx_b].minhash);
                    if sim >= threshold {
                        let severity = if sim >= 0.95 {
                            CcSeverity::Error
                        } else {
                            CcSeverity::Warning
                        };

                        findings.push(CrossCrateFinding {
                            rule: "CC-001".to_string(),
                            severity,
                            crate_a: signed[idx_a].crate_name.clone(),
                            crate_b: signed[idx_b].crate_name.clone(),
                            function_a: signed[idx_a].function_name.clone(),
                            function_b: signed[idx_b].function_name.clone(),
                            file_a: signed[idx_a].file_path.clone(),
                            file_b: signed[idx_b].file_path.clone(),
                            similarity: Some(sim),
                            recommendation: format!(
                                "Extract shared function to common crate (similarity: {:.0}%)",
                                sim * 100.0
                            ),
                        });
                    }
                }
            }
        }
    }

    // Sort by similarity descending for most-impactful-first reporting
    findings.sort_by(|a, b| {
        b.similarity
            .unwrap_or(0.0)
            .partial_cmp(&a.similarity.unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    findings
}

/// CC-002: Detect functions with the same name but different signatures across crates.
///
/// When two crates define `rms_norm()` with different parameter types or return types,
/// this is an API divergence risk — callers may silently get wrong behavior.
fn detect_cc002_api_divergence(
    crate_functions: &[(CrateInfo, Vec<FunctionEntry>)],
    config: &DetectionConfig,
) -> Vec<CrossCrateFinding> {
    // Group public functions by name across crates
    struct FuncRef<'a> {
        crate_info: &'a CrateInfo,
        func: &'a FunctionEntry,
    }
    let mut by_name: HashMap<String, Vec<FuncRef<'_>>> = HashMap::new();

    for (crate_info, functions) in crate_functions {
        for func in functions {
            if is_excluded_function(&func.function_name, config) {
                continue;
            }
            // Only check public functions (pub fn, pub async fn)
            if func.signature.contains("pub ") || !func.signature.starts_with("fn ") {
                by_name
                    .entry(func.function_name.clone())
                    .or_default()
                    .push(FuncRef { crate_info, func });
            }
        }
    }

    let mut findings = Vec::new();

    for (name, impls) in &by_name {
        if impls.len() < 2 {
            continue;
        }

        // Count distinct crates — if a name appears in 3+ crates, it's
        // a natural polymorphic pattern, not an API divergence risk.
        let distinct_crates: HashSet<&str> = impls
            .iter()
            .map(|fr| fr.crate_info.name.as_str())
            .collect();
        if distinct_crates.len() > 2 {
            continue;
        }

        // Check cross-crate pairs that have a dependency relationship.
        // Without a dependency, different signatures are independent design choices.
        for i in 0..impls.len() {
            for j in (i + 1)..impls.len() {
                let fr_a = &impls[i];
                let fr_b = &impls[j];

                // Skip same-crate comparisons
                if fr_a.crate_info.name == fr_b.crate_info.name {
                    continue;
                }

                // Skip excluded crate pairs
                if is_crate_pair_excluded(
                    &fr_a.crate_info.name,
                    &fr_b.crate_info.name,
                    &config.excluded_crate_pairs,
                ) {
                    continue;
                }

                // Only check crates with a dependency relationship
                let a_deps_b = fr_a
                    .crate_info
                    .cargo_deps
                    .iter()
                    .any(|d| d == &fr_b.crate_info.name);
                let b_deps_a = fr_b
                    .crate_info
                    .cargo_deps
                    .iter()
                    .any(|d| d == &fr_a.crate_info.name);
                if !a_deps_b && !b_deps_a {
                    continue;
                }

                let norm_a = normalize_signature(&fr_a.func.signature);
                let norm_b = normalize_signature(&fr_b.func.signature);

                // Only flag if parameter counts are similar (within +/-1).
                // Completely different arities are independent APIs, not divergence.
                let params_a = count_signature_params(&norm_a);
                let params_b = count_signature_params(&norm_b);
                if params_a.abs_diff(params_b) > 1 {
                    continue;
                }

                if norm_a != norm_b {
                    findings.push(CrossCrateFinding {
                        rule: "CC-002".to_string(),
                        severity: CcSeverity::Warning,
                        crate_a: fr_a.crate_info.name.clone(),
                        crate_b: fr_b.crate_info.name.clone(),
                        function_a: name.clone(),
                        function_b: name.clone(),
                        file_a: fr_a.func.file_path.clone(),
                        file_b: fr_b.func.file_path.clone(),
                        similarity: None,
                        recommendation: format!(
                            "Align signatures: '{}' vs '{}'",
                            fr_a.func.signature, fr_b.func.signature
                        ),
                    });
                }
            }
        }
    }

    findings
}

/// Count parameters in a normalized signature.
fn count_signature_params(sig: &str) -> usize {
    if let Some(start) = sig.find('(') {
        if let Some(end) = sig.rfind(')') {
            let inner = sig[start + 1..end].trim();
            if inner.is_empty() {
                return 0;
            }
            // Handle &self, &mut self
            let params: Vec<&str> = inner.split(',').collect();
            return params
                .iter()
                .filter(|p| {
                    let t = p.trim();
                    t != "&self" && t != "&mut self" && t != "self" && t != "mut self"
                })
                .count();
        }
    }
    0
}

/// Normalize a function signature for comparison.
/// Strips `pub`, `async`, visibility modifiers, and normalizes whitespace.
fn normalize_signature(sig: &str) -> String {
    let mut s = sig.to_string();
    // Strip visibility
    for prefix in &["pub(crate) ", "pub(super) ", "pub "] {
        if let Some(rest) = s.strip_prefix(prefix) {
            s = rest.to_string();
        }
    }
    // Strip async
    if let Some(rest) = s.strip_prefix("async ") {
        s = rest.to_string();
    }
    // Normalize whitespace
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}
