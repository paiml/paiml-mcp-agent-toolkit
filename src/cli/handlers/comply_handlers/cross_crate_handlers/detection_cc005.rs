#![cfg_attr(coverage_nightly, coverage(off))]

// CC-005: Example code duplication detection

use super::helpers::parse_language;
use super::types::{CcSeverity, CrateInfo, CrossCrateFinding};
use crate::services::agent_context::FunctionEntry;
use crate::services::duplicate_detector::{
    DuplicateDetectionConfig, Language, MinHashGenerator, MinHashSignature,
    UniversalFeatureExtractor,
};
use std::path::Path;

/// CC-005: Detect when example code duplicates production source.
///
/// Walks `examples/` directories per crate, tokenizes example files,
/// and compares MinHash signatures against `src/` function signatures.
/// High similarity means the example is a near-copy of production code
/// rather than a curated demonstration.
pub(super) fn detect_cc005_example_duplication(
    crate_functions: &[(CrateInfo, Vec<FunctionEntry>)],
    threshold: f64,
) -> Vec<CrossCrateFinding> {
    let config = DuplicateDetectionConfig {
        normalize_identifiers: true,
        normalize_literals: true,
        ignore_comments: true,
        ..Default::default()
    };
    let extractor = UniversalFeatureExtractor::new(config);
    let hasher = MinHashGenerator::new(128);

    let mut findings = Vec::new();

    for (crate_info, functions) in crate_functions {
        let examples_dir = crate_info.path.join("examples");
        if !examples_dir.exists() || !examples_dir.is_dir() {
            continue;
        }

        // Collect example file signatures
        let example_sigs = collect_example_signatures(&examples_dir, &extractor, &hasher);
        if example_sigs.is_empty() {
            continue;
        }

        // Compute src function signatures
        let mut src_sigs: Vec<(&FunctionEntry, MinHashSignature)> = Vec::new();
        for func in functions {
            if func.source.is_empty() || func.source.len() < 50 {
                continue;
            }
            let lang = parse_language(&func.language);
            let tokens = extractor.extract_features(&func.source, lang);
            if tokens.len() < 5 {
                continue;
            }
            let shingles = hasher.generate_shingles(&tokens, 3);
            if shingles.is_empty() {
                continue;
            }
            let sig = hasher.compute_signature(&shingles);
            src_sigs.push((func, sig));
        }

        // Compare example signatures against src function signatures
        for (example_path, example_sig) in &example_sigs {
            for (func, src_sig) in &src_sigs {
                let sim = example_sig.jaccard_similarity(src_sig);
                if sim >= threshold {
                    findings.push(CrossCrateFinding {
                        rule: "CC-005".to_string(),
                        severity: CcSeverity::Advisory,
                        crate_a: crate_info.name.clone(),
                        crate_b: crate_info.name.clone(),
                        function_a: example_path.clone(),
                        function_b: func.function_name.clone(),
                        file_a: example_path.clone(),
                        file_b: func.file_path.clone(),
                        similarity: Some(sim),
                        recommendation: format!(
                            "Example '{}' is {:.0}% similar to {}::{} — consider curating",
                            example_path,
                            sim * 100.0,
                            crate_info.name,
                            func.function_name
                        ),
                    });
                }
            }
        }
    }

    findings
}

/// Collect MinHash signatures from example source files.
fn collect_example_signatures(
    examples_dir: &Path,
    extractor: &UniversalFeatureExtractor,
    hasher: &MinHashGenerator,
) -> Vec<(String, MinHashSignature)> {
    let mut sigs = Vec::new();

    let entries = match std::fs::read_dir(examples_dir) {
        Ok(e) => e,
        Err(_) => return sigs,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let lang = match ext {
            "rs" => Language::Rust,
            "py" => Language::Python,
            "ts" => Language::TypeScript,
            "js" => Language::JavaScript,
            _ => continue,
        };

        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        if content.len() < 50 {
            continue;
        }

        let tokens = extractor.extract_features(&content, lang);
        if tokens.len() < 5 {
            continue;
        }
        let shingles = hasher.generate_shingles(&tokens, 3);
        if shingles.is_empty() {
            continue;
        }
        let sig = hasher.compute_signature(&shingles);

        let display_path = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        sigs.push((format!("examples/{display_path}"), sig));
    }

    sigs
}
