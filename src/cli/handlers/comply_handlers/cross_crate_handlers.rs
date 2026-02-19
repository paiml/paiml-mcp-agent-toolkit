#![cfg_attr(coverage_nightly, coverage(off))]

use crate::cli::commands::ComplyOutputFormat;
use crate::services::agent_context::{parse_workspace_siblings, AgentContextIndex, FunctionEntry};
use crate::services::duplicate_detector::{
    DuplicateDetectionConfig, Language, MinHashGenerator, MinHashSignature,
    UniversalFeatureExtractor,
};
use anyhow::Result;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

// --- Types ---

#[derive(Debug, Clone, Serialize)]
pub struct CrateInfo {
    pub name: String,
    pub path: PathBuf,
    pub cargo_deps: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum CcSeverity {
    Error,
    Warning,
    Advisory,
}

impl std::fmt::Display for CcSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CcSeverity::Error => write!(f, "error"),
            CcSeverity::Warning => write!(f, "warning"),
            CcSeverity::Advisory => write!(f, "advisory"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CrossCrateFinding {
    pub rule: String,
    pub severity: CcSeverity,
    pub crate_a: String,
    pub crate_b: String,
    pub function_a: String,
    pub function_b: String,
    pub file_a: String,
    pub file_b: String,
    pub similarity: Option<f64>,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CrossCrateSummary {
    pub total_findings: usize,
    pub errors: usize,
    pub warnings: usize,
    pub advisories: usize,
    pub rules_triggered: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CrossCrateReport {
    pub findings: Vec<CrossCrateFinding>,
    pub summary: CrossCrateSummary,
    pub crates_analyzed: Vec<String>,
}

/// A function with its computed MinHash signature, grouped by crate.
struct SignedFunction {
    crate_name: String,
    function_name: String,
    #[allow(dead_code)]
    signature: String,
    file_path: String,
    minhash: MinHashSignature,
    #[allow(dead_code)]
    language: Language,
}

// --- Main handler ---

pub async fn handle_cross_crate(
    workspace_path: &Path,
    similarity_threshold: f64,
    churn_window_days: u32,
    rules_filter: Option<&str>,
    format: ComplyOutputFormat,
    output: Option<&Path>,
    strict: bool,
) -> Result<()> {
    let enabled_rules = parse_rules_filter(rules_filter);

    eprintln!("Discovering workspace crates...");
    let crates = discover_workspace_crates(workspace_path);
    if crates.len() < 2 {
        println!("Cross-crate analysis requires at least 2 crates in the workspace.");
        println!("Configure siblings in .pmat/workspace.toml:");
        println!("  siblings = [\"../aprender\", \"../trueno\"]");
        return Ok(());
    }

    eprintln!("Loading functions from {} crates...", crates.len());
    let crate_functions = load_all_crate_functions(&crates);
    let crate_names: Vec<String> = crate_functions
        .iter()
        .map(|(c, _)| c.name.clone())
        .collect();

    eprintln!(
        "Analyzing {} crates: {}",
        crate_names.len(),
        crate_names.join(", ")
    );

    let mut findings: Vec<CrossCrateFinding> = Vec::new();

    // CC-001: Function clone detection
    if is_rule_enabled("cc001", &enabled_rules) {
        let cc001 = detect_cc001_function_clones(&crate_functions, similarity_threshold);
        findings.extend(cc001);
    }

    // CC-002: API signature divergence
    if is_rule_enabled("cc002", &enabled_rules) {
        let cc002 = detect_cc002_api_divergence(&crate_functions);
        findings.extend(cc002);
    }

    // CC-003: Primitive should be upstream
    if is_rule_enabled("cc003", &enabled_rules) {
        let cc003 = detect_cc003_primitive_upstream(&crate_functions);
        findings.extend(cc003);
    }

    // CC-004: Churn correlation
    if is_rule_enabled("cc004", &enabled_rules) {
        let cc004 = detect_cc004_churn_correlation(&crate_functions, churn_window_days);
        findings.extend(cc004);
    }

    // CC-005: Example duplication
    if is_rule_enabled("cc005", &enabled_rules) {
        let cc005 = detect_cc005_example_duplication(&crate_functions, similarity_threshold);
        findings.extend(cc005);
    }

    let report = build_report(findings, crate_names);

    // Output
    let output_text = match format {
        ComplyOutputFormat::Text => format_text(&report),
        ComplyOutputFormat::Json => serde_json::to_string_pretty(&report)?,
        ComplyOutputFormat::Markdown => format_markdown(&report),
    };

    if let Some(path) = output {
        std::fs::write(path, &output_text)?;
        eprintln!("Report written to {}", path.display());
    } else {
        println!("{output_text}");
    }

    if strict && report.summary.total_findings > 0 {
        std::process::exit(1);
    }

    Ok(())
}

// --- Workspace discovery ---

pub fn discover_workspace_crates(workspace_path: &Path) -> Vec<CrateInfo> {
    let mut crates = Vec::new();

    // Add the base crate
    let base_name = workspace_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("base")
        .to_string();

    let base_deps = read_cargo_deps(&workspace_path.join("Cargo.toml"));
    crates.push(CrateInfo {
        name: base_name,
        path: workspace_path.to_path_buf(),
        cargo_deps: base_deps,
    });

    // Read workspace.toml for siblings
    let workspace_toml = workspace_path.join(".pmat").join("workspace.toml");
    if let Ok(content) = std::fs::read_to_string(&workspace_toml) {
        let siblings = parse_workspace_siblings(&content);
        for sibling_rel in siblings {
            let sibling_path = workspace_path.join(&sibling_rel).canonicalize();
            let Ok(sibling_path) = sibling_path else {
                continue;
            };
            if !sibling_path.join("Cargo.toml").exists() {
                continue;
            }
            let name = sibling_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            let deps = read_cargo_deps(&sibling_path.join("Cargo.toml"));
            crates.push(CrateInfo {
                name,
                path: sibling_path,
                cargo_deps: deps,
            });
        }
    }

    crates
}

/// Parse dependency names from a Cargo.toml [dependencies] section.
/// Simple string parser — no full TOML parser needed.
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

/// Load functions from each crate's pmat index.
fn load_all_crate_functions(crates: &[CrateInfo]) -> Vec<(CrateInfo, Vec<FunctionEntry>)> {
    let mut result = Vec::new();

    for crate_info in crates {
        let index_path = crate_info.path.join(".pmat").join("context.idx");
        match AgentContextIndex::load(&index_path) {
            Ok(mut index) => {
                index.load_all_source();
                let functions: Vec<FunctionEntry> = index.all_functions().to_vec();
                eprintln!(
                    "  {} — {} functions loaded",
                    crate_info.name,
                    functions.len()
                );
                result.push((crate_info.clone(), functions));
            }
            Err(e) => {
                eprintln!("  {} — skipped (no index: {})", crate_info.name, e);
            }
        }
    }

    result
}

/// Parse a language string into the duplicate_detector Language enum, defaulting to Rust.
fn parse_language(lang: &str) -> Language {
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
fn parse_rules_filter(rules: Option<&str>) -> Option<HashSet<String>> {
    rules.map(|r| {
        r.split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect()
    })
}

fn is_rule_enabled(rule: &str, filter: &Option<HashSet<String>>) -> bool {
    match filter {
        None => true,
        Some(set) => set.contains(rule),
    }
}

/// Names too generic for meaningful cross-crate clone detection.
/// These are trait impls (Default, Display, From, etc.) that are
/// trivially duplicated and don't represent real copy-paste.
fn is_generic_impl_name(name: &str) -> bool {
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
    )
}

/// Compute MinHash signatures for all functions across all crates.
/// Filters out generic trait impls and very short functions.
fn compute_signatures(crate_functions: &[(CrateInfo, Vec<FunctionEntry>)]) -> Vec<SignedFunction> {
    let config = DuplicateDetectionConfig {
        normalize_identifiers: true,
        normalize_literals: true,
        ignore_comments: true,
        ..Default::default()
    };
    let extractor = UniversalFeatureExtractor::new(config);
    let hasher = MinHashGenerator::new(128);

    let mut signed = Vec::new();

    for (crate_info, functions) in crate_functions {
        for func in functions {
            // Skip empty or very short source (< 3 lines)
            if func.source.is_empty() || func.source.lines().count() < 3 {
                continue;
            }
            // Skip generic trait impls that produce false positives
            if is_generic_impl_name(&func.function_name) {
                continue;
            }
            let lang = parse_language(&func.language);
            let tokens = extractor.extract_features(&func.source, lang);
            if tokens.len() < 15 {
                continue; // Too short for meaningful MinHash comparison
            }
            let shingles = hasher.generate_shingles(&tokens, 3);
            if shingles.is_empty() {
                continue;
            }
            let minhash = hasher.compute_signature(&shingles);
            signed.push(SignedFunction {
                crate_name: crate_info.name.clone(),
                function_name: func.function_name.clone(),
                signature: func.signature.clone(),
                file_path: func.file_path.clone(),
                minhash,
                language: lang,
            });
        }
    }

    signed
}

fn build_report(
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

// --- Include sub-modules ---

include!("cross_crate_cc001_cc002.rs");
include!("cross_crate_cc003_cc004.rs");
include!("cross_crate_cc005_output.rs");

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover_workspace_crates_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let crates = discover_workspace_crates(tmp.path());
        // Should have at least the base crate
        assert_eq!(crates.len(), 1);
        assert!(!crates[0].name.is_empty());
    }

    #[test]
    fn test_read_cargo_deps_parses_section() {
        let tmp = tempfile::tempdir().unwrap();
        let cargo_toml = tmp.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            r#"[package]
name = "test"
version = "0.1.0"

[dependencies]
serde = "1.0"
tokio = { version = "1", features = ["full"] }
anyhow = "1"

[dev-dependencies]
tempfile = "3"
"#,
        )
        .unwrap();

        let deps = read_cargo_deps(&cargo_toml);
        assert!(deps.contains(&"serde".to_string()));
        assert!(deps.contains(&"tokio".to_string()));
        assert!(deps.contains(&"anyhow".to_string()));
        assert!(deps.contains(&"tempfile".to_string()));
    }

    #[test]
    fn test_normalize_signature_strips_pub() {
        assert_eq!(
            normalize_signature("pub fn foo(x: i32) -> bool"),
            "fn foo(x: i32) -> bool"
        );
        assert_eq!(normalize_signature("pub async fn bar()"), "fn bar()");
        assert_eq!(normalize_signature("fn baz(s: &str)"), "fn baz(s: &str)");
    }

    #[test]
    fn test_cc001_detects_identical_source() {
        let crate_a = CrateInfo {
            name: "crate_a".to_string(),
            path: PathBuf::from("/tmp/a"),
            cargo_deps: vec![],
        };
        let crate_b = CrateInfo {
            name: "crate_b".to_string(),
            path: PathBuf::from("/tmp/b"),
            cargo_deps: vec![],
        };

        // Use a longer, realistic function body for meaningful MinHash tokenization
        let source = r#"pub fn silu_activation(input: &[f32], output: &mut [f32]) {
    assert_eq!(input.len(), output.len(), "Input and output lengths must match");
    for i in 0..input.len() {
        let x = input[i];
        let sigmoid = 1.0 / (1.0 + (-x).exp());
        output[i] = x * sigmoid;
    }
}"#
        .to_string();
        let func_a = make_test_func("silu_activation", &source, "src/a.rs");
        let func_b = make_test_func("silu_activation", &source, "src/b.rs");

        let crate_functions = vec![(crate_a, vec![func_a]), (crate_b, vec![func_b])];

        let findings = detect_cc001_function_clones(&crate_functions, 0.80);
        assert!(
            !findings.is_empty(),
            "CC-001 should detect identical functions across crates"
        );
        assert_eq!(findings[0].rule, "CC-001");
    }

    #[test]
    fn test_cc001_no_finding_within_same_crate() {
        let crate_a = CrateInfo {
            name: "crate_a".to_string(),
            path: PathBuf::from("/tmp/a"),
            cargo_deps: vec![],
        };

        let source = r#"pub fn silu_activation(input: &[f32], output: &mut [f32]) {
    assert_eq!(input.len(), output.len(), "Input and output lengths must match");
    for i in 0..input.len() {
        let x = input[i];
        let sigmoid = 1.0 / (1.0 + (-x).exp());
        output[i] = x * sigmoid;
    }
}"#
        .to_string();
        let func_a = make_test_func("silu_activation", &source, "src/a.rs");
        let func_b = make_test_func("silu_activation_v2", &source, "src/b.rs");

        let crate_functions = vec![(crate_a, vec![func_a, func_b])];

        let findings = detect_cc001_function_clones(&crate_functions, 0.80);
        assert!(
            findings.is_empty(),
            "CC-001 should NOT flag duplicates within the same crate"
        );
    }

    #[test]
    fn test_cc002_same_name_different_sig() {
        let crate_a = CrateInfo {
            name: "crate_a".to_string(),
            path: PathBuf::from("/tmp/a"),
            cargo_deps: vec![],
        };
        let crate_b = CrateInfo {
            name: "crate_b".to_string(),
            path: PathBuf::from("/tmp/b"),
            cargo_deps: vec!["crate_a".to_string()], // B depends on A
        };

        let func_a = make_test_func_with_sig(
            "rms_norm",
            "pub fn rms_norm(x: &[f32]) -> Vec<f32>",
            "src/a.rs",
        );
        let func_b = make_test_func_with_sig(
            "rms_norm",
            "pub fn rms_norm(x: &[f64], eps: f64) -> Vec<f64>",
            "src/b.rs",
        );

        let crate_functions = vec![(crate_a, vec![func_a]), (crate_b, vec![func_b])];

        let findings = detect_cc002_api_divergence(&crate_functions);
        assert!(
            !findings.is_empty(),
            "CC-002 should detect divergent signatures"
        );
        assert_eq!(findings[0].rule, "CC-002");
    }

    #[test]
    fn test_cc002_same_name_same_sig_no_finding() {
        let crate_a = CrateInfo {
            name: "crate_a".to_string(),
            path: PathBuf::from("/tmp/a"),
            cargo_deps: vec![],
        };
        let crate_b = CrateInfo {
            name: "crate_b".to_string(),
            path: PathBuf::from("/tmp/b"),
            cargo_deps: vec!["crate_a".to_string()], // B depends on A
        };

        let func_a = make_test_func_with_sig("gelu", "pub fn gelu(x: f32) -> f32", "src/a.rs");
        let func_b = make_test_func_with_sig("gelu", "pub fn gelu(x: f32) -> f32", "src/b.rs");

        let crate_functions = vec![(crate_a, vec![func_a]), (crate_b, vec![func_b])];

        let findings = detect_cc002_api_divergence(&crate_functions);
        assert!(
            findings.is_empty(),
            "CC-002 should NOT flag identical signatures"
        );
    }

    #[test]
    fn test_cc003_finding_when_dep_reimplements() {
        let crate_a = CrateInfo {
            name: "trueno".to_string(),
            path: PathBuf::from("/tmp/trueno"),
            cargo_deps: vec![],
        };
        let crate_b = CrateInfo {
            name: "aprender".to_string(),
            path: PathBuf::from("/tmp/aprender"),
            cargo_deps: vec!["trueno".to_string()],
        };

        // Source must be >= 3 lines to pass CC-003 filter
        let src_a = "fn f16_to_f32(x: u16) -> f32 {\n    let bits = f16::from_bits(x);\n    bits.to_f32()\n}";
        let src_b = "fn f16_to_f32(val: u16) -> f32 {\n    let h = half::f16::from_bits(val);\n    h.to_f32()\n}";
        let func_a = make_test_func("f16_to_f32", src_a, "src/conv.rs");
        let func_b = make_test_func("f16_to_f32", src_b, "src/quant.rs");

        let crate_functions = vec![(crate_a, vec![func_a]), (crate_b, vec![func_b])];

        let findings = detect_cc003_primitive_upstream(&crate_functions);
        assert!(
            !findings.is_empty(),
            "CC-003 should detect reimplementation of upstream function"
        );
        assert_eq!(findings[0].rule, "CC-003");
    }

    #[test]
    fn test_cc003_no_finding_when_no_dep() {
        let crate_a = CrateInfo {
            name: "trueno".to_string(),
            path: PathBuf::from("/tmp/trueno"),
            cargo_deps: vec![],
        };
        let crate_b = CrateInfo {
            name: "unrelated".to_string(),
            path: PathBuf::from("/tmp/unrelated"),
            cargo_deps: vec!["serde".to_string()],
        };

        let src_a = "fn f16_to_f32(x: u16) -> f32 {\n    let bits = x;\n    bits as f32\n}";
        let src_b = "fn f16_to_f32(v: u16) -> f32 {\n    let bits = v;\n    bits as f32\n}";
        let func_a = make_test_func("f16_to_f32", src_a, "src/conv.rs");
        let func_b = make_test_func("f16_to_f32", src_b, "src/quant.rs");

        let crate_functions = vec![(crate_a, vec![func_a]), (crate_b, vec![func_b])];

        let findings = detect_cc003_primitive_upstream(&crate_functions);
        assert!(
            findings.is_empty(),
            "CC-003 should NOT flag when no dependency relationship"
        );
    }

    #[test]
    fn test_cc005_no_examples_dir() {
        let crate_a = CrateInfo {
            name: "crate_a".to_string(),
            path: PathBuf::from("/tmp/nonexistent_crate_path"),
            cargo_deps: vec![],
        };

        let crate_functions = vec![(crate_a, vec![])];

        let findings = detect_cc005_example_duplication(&crate_functions, 0.80);
        assert!(
            findings.is_empty(),
            "CC-005 should gracefully skip missing examples/"
        );
    }

    #[test]
    fn test_build_report_summary() {
        let findings = vec![
            CrossCrateFinding {
                rule: "CC-001".to_string(),
                severity: CcSeverity::Error,
                crate_a: "a".to_string(),
                crate_b: "b".to_string(),
                function_a: "f".to_string(),
                function_b: "f".to_string(),
                file_a: "a.rs".to_string(),
                file_b: "b.rs".to_string(),
                similarity: Some(0.95),
                recommendation: "Consolidate".to_string(),
            },
            CrossCrateFinding {
                rule: "CC-002".to_string(),
                severity: CcSeverity::Warning,
                crate_a: "a".to_string(),
                crate_b: "c".to_string(),
                function_a: "g".to_string(),
                function_b: "g".to_string(),
                file_a: "a.rs".to_string(),
                file_b: "c.rs".to_string(),
                similarity: None,
                recommendation: "Align signatures".to_string(),
            },
            CrossCrateFinding {
                rule: "CC-004".to_string(),
                severity: CcSeverity::Advisory,
                crate_a: "a".to_string(),
                crate_b: "b".to_string(),
                function_a: "h".to_string(),
                function_b: "h".to_string(),
                file_a: "a.rs".to_string(),
                file_b: "b.rs".to_string(),
                similarity: None,
                recommendation: "Review".to_string(),
            },
        ];

        let report = build_report(
            findings,
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
        );
        assert_eq!(report.summary.total_findings, 3);
        assert_eq!(report.summary.errors, 1);
        assert_eq!(report.summary.warnings, 1);
        assert_eq!(report.summary.advisories, 1);
        assert_eq!(report.summary.rules_triggered["CC-001"], 1);
        assert_eq!(report.summary.rules_triggered["CC-002"], 1);
    }

    #[test]
    fn test_parse_rules_filter() {
        assert!(parse_rules_filter(None).is_none());

        let set = parse_rules_filter(Some("cc001,cc003")).unwrap();
        assert!(set.contains("cc001"));
        assert!(set.contains("cc003"));
        assert!(!set.contains("cc002"));

        let set2 = parse_rules_filter(Some(" CC001 , CC002 ")).unwrap();
        assert!(set2.contains("cc001"));
        assert!(set2.contains("cc002"));
    }

    // --- Test helpers ---

    fn make_test_func(name: &str, source: &str, file_path: &str) -> FunctionEntry {
        FunctionEntry {
            function_name: name.to_string(),
            signature: format!("fn {name}()"),
            source: source.to_string(),
            file_path: file_path.to_string(),
            doc_comment: None,
            definition_type: Default::default(),
            start_line: 1,
            end_line: 1,
            language: "Rust".to_string(),
            quality: Default::default(),
            checksum: String::new(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: vec![],
        }
    }

    fn make_test_func_with_sig(name: &str, sig: &str, file_path: &str) -> FunctionEntry {
        let mut func = make_test_func(name, "// placeholder source", file_path);
        func.signature = sig.to_string();
        func
    }
}
