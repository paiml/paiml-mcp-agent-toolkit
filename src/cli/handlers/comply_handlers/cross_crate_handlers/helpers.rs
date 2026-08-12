#![cfg_attr(coverage_nightly, coverage(off))]

use super::types::{
    CcSeverity, CrateInfo, CrossCrateFinding, CrossCrateReport, CrossCrateSummary, DetectionConfig,
    SignedFunction,
};
use crate::services::agent_context::{AgentContextIndex, FunctionEntry};
use crate::services::duplicate_detector::{
    DuplicateDetectionConfig, Language, MinHashGenerator, UniversalFeatureExtractor,
};
use std::collections::{HashMap, HashSet};

/// Parse a language string into the duplicate_detector Language enum, defaulting to Rust.
pub(super) fn parse_language(lang: &str) -> Language {
    match lang.to_lowercase().as_str() {
        "rust" => Language::Rust,
        "typescript" => Language::TypeScript,
        "javascript" => Language::JavaScript,
        "python" => Language::Python,
        "c" => Language::C,
        "cpp" | "c++" => Language::Cpp,
        "kotlin" => Language::Kotlin,
        _ => Language::Rust,
    }
}

/// Parse --rules filter into a set of enabled rule IDs.
pub(super) fn parse_rules_filter(rules: Option<&str>) -> Option<HashSet<String>> {
    rules.map(|r| {
        r.split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect()
    })
}

pub(super) fn is_rule_enabled(rule: &str, filter: &Option<HashSet<String>>) -> bool {
    match filter {
        None => true,
        Some(set) => set.contains(rule),
    }
}

/// Check if a function name should be excluded from detection.
/// Combines hardcoded generic names with user-configured exclusions.
pub(super) fn is_excluded_function(name: &str, config: &DetectionConfig) -> bool {
    is_generic_impl_name(name) || config.excluded_functions.contains(&name.to_lowercase())
}

/// Check if a crate pair is excluded from analysis.
pub(super) fn is_crate_pair_excluded(
    crate_a: &str,
    crate_b: &str,
    excluded: &HashSet<(String, String)>,
) -> bool {
    excluded.contains(&(crate_a.to_string(), crate_b.to_string()))
        || excluded.contains(&(crate_b.to_string(), crate_a.to_string()))
}

/// Names too generic for meaningful cross-crate clone detection.
/// These are trait impls (Default, Display, From, etc.) that are
/// trivially duplicated and don't represent real copy-paste.
pub(super) fn is_generic_impl_name(name: &str) -> bool {
    matches!(
        name,
        // Trait impls
        "default" | "new" | "fmt" | "clone" | "from" | "into"
            | "drop" | "deref" | "deref_mut" | "as_ref" | "as_mut"
            | "borrow" | "borrow_mut" | "try_from" | "try_into"
            | "hash" | "eq" | "partial_cmp" | "cmp" | "partial_eq"
            | "serialize" | "deserialize" | "display"
            | "index" | "index_mut" | "next" | "size_hint"
            | "poll" | "resume" | "init" | "build"
            // Trivial accessors (too short for meaningful clone detection)
            | "len" | "is_empty" | "is_full" | "capacity"
            | "get" | "set" | "push" | "pop" | "insert" | "remove"
            | "contains" | "clear" | "iter" | "name" | "id"
            | "width" | "height" | "size" | "count"
            // Additional trivial accessors (expanded for false-positive reduction)
            | "shape" | "dim" | "duration" | "alpha" | "beta" | "gamma"
            | "epsilon" | "rows" | "cols" | "dtype" | "ndim" | "rank"
            | "start" | "end" | "min" | "max" | "sum" | "mean"
            | "value" | "key" | "offset" | "stride" | "vocab_size"
    )
}

/// Compute MinHash signatures for all functions across all crates.
/// Filters out excluded functions and very short functions.
/// Uses parallel threads for CPU-bound feature extraction.
pub(super) fn compute_signatures(
    crate_functions: &[(CrateInfo, Vec<FunctionEntry>)],
    config: &DetectionConfig,
) -> Vec<SignedFunction> {
    // Collect candidate functions first (cheap filter pass)
    let candidates: Vec<_> = crate_functions
        .iter()
        .flat_map(|(crate_info, functions)| {
            functions.iter().filter_map(|func| {
                if func.source.is_empty() || func.source.lines().count() < config.min_body_lines {
                    return None;
                }
                if is_excluded_function(&func.function_name, config) {
                    return None;
                }
                Some((crate_info.name.clone(), func.clone()))
            })
        })
        .collect();

    // Process in parallel chunks for CPU-bound MinHash computation
    let chunk_size = (candidates.len() / num_cpus::get()).max(64);
    let chunks: Vec<_> = candidates.chunks(chunk_size).collect();

    let handles: Vec<_> = chunks
        .into_iter()
        .map(|chunk| {
            let chunk: Vec<_> = chunk.to_vec();
            std::thread::spawn(move || {
                let dup_config = DuplicateDetectionConfig {
                    normalize_identifiers: true,
                    normalize_literals: true,
                    ignore_comments: true,
                    ..Default::default()
                };
                let extractor = UniversalFeatureExtractor::new(dup_config);
                let hasher = MinHashGenerator::new(128);
                let mut signed = Vec::new();

                for (crate_name, func) in &chunk {
                    let lang = parse_language(&func.language);
                    let tokens = extractor.extract_features(&func.source, lang);
                    if tokens.len() < 5 {
                        continue;
                    }
                    let shingles = hasher.generate_shingles(&tokens, 3);
                    if shingles.is_empty() {
                        continue;
                    }
                    let minhash = hasher.compute_signature(&shingles);
                    signed.push(SignedFunction {
                        crate_name: crate_name.clone(),
                        function_name: func.function_name.clone(),
                        signature: func.signature.clone(),
                        file_path: func.file_path.clone(),
                        minhash,
                        language: lang,
                    });
                }
                signed
            })
        })
        .collect();

    handles
        .into_iter()
        .flat_map(|h| h.join().unwrap_or_default())
        .collect()
}

pub(super) fn build_report(
    findings: Vec<CrossCrateFinding>,
    crates_analyzed: Vec<String>,
) -> CrossCrateReport {
    let mut rules_triggered: HashMap<String, usize> = HashMap::new();
    let mut errors = 0;
    let mut warnings = 0;
    let mut advisories = 0;

    for f in &findings {
        *rules_triggered.entry(f.rule.clone()).or_insert(0) += 1;
        match f.severity {
            CcSeverity::Error => errors += 1,
            CcSeverity::Warning => warnings += 1,
            CcSeverity::Advisory => advisories += 1,
        }
    }

    CrossCrateReport {
        summary: CrossCrateSummary {
            total_findings: findings.len(),
            errors,
            warnings,
            advisories,
            rules_triggered,
        },
        findings,
        crates_analyzed,
    }
}

/// Load functions from each crate's pmat index (parallel).
pub(super) fn load_all_crate_functions(
    crates: &[CrateInfo],
) -> Vec<(CrateInfo, Vec<FunctionEntry>)> {
    let handles: Vec<_> = crates
        .iter()
        .map(|crate_info| {
            let ci = crate_info.clone();
            std::thread::spawn(move || {
                let index_path = ci.path.join(".pmat").join("context.idx");
                match AgentContextIndex::load(&index_path) {
                    Ok(mut index) => {
                        index.load_all_source();
                        let functions: Vec<FunctionEntry> = index.all_functions().to_vec();
                        crate::status_eprintln!(
                            "  {} — {} functions loaded",
                            ci.name,
                            functions.len()
                        );
                        Some((ci, functions))
                    }
                    Err(e) => {
                        eprintln!("  {} — skipped (no index: {})", ci.name, e);
                        None
                    }
                }
            })
        })
        .collect();

    handles
        .into_iter()
        .filter_map(|h| h.join().ok().flatten())
        .collect()
}
