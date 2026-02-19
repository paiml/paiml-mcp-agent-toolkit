// CC-005: Example code duplication detection
// Output formatters: text, markdown, JSON
//
// Included into cross_crate_handlers.rs via include!()

/// CC-005: Detect when example code duplicates production source.
///
/// Walks `examples/` directories per crate, tokenizes example files,
/// and compares MinHash signatures against `src/` function signatures.
/// High similarity means the example is a near-copy of production code
/// rather than a curated demonstration.
fn detect_cc005_example_duplication(
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

// --- Output formatters ---

fn format_text(report: &CrossCrateReport) -> String {
    let mut out = String::new();

    out.push_str("\n\x1b[1mCross-Crate Duplication Report\x1b[0m\n");
    out.push_str(&format!(
        "Crates analyzed: {}\n\n",
        report.crates_analyzed.join(", ")
    ));

    if report.findings.is_empty() {
        out.push_str("\x1b[32mNo cross-crate duplication findings.\x1b[0m\n");
        return out;
    }

    // Group findings by rule
    let mut by_rule: HashMap<&str, Vec<&CrossCrateFinding>> = HashMap::new();
    for f in &report.findings {
        by_rule.entry(&f.rule).or_default().push(f);
    }

    let rule_order = ["CC-001", "CC-002", "CC-003", "CC-004", "CC-005"];

    for rule in &rule_order {
        if let Some(rule_findings) = by_rule.get(rule) {
            let icon = match *rule {
                "CC-001" => "\x1b[31m[CC-001 Clone]\x1b[0m",
                "CC-002" => "\x1b[33m[CC-002 Diverge]\x1b[0m",
                "CC-003" => "\x1b[33m[CC-003 Upstream]\x1b[0m",
                "CC-004" => "\x1b[36m[CC-004 Churn]\x1b[0m",
                "CC-005" => "\x1b[36m[CC-005 Example]\x1b[0m",
                _ => rule,
            };

            out.push_str(&format!("{} ({} findings)\n", icon, rule_findings.len()));

            for f in rule_findings.iter().take(20) {
                let sim_str = f
                    .similarity
                    .map(|s| format!(" ({:.0}%)", s * 100.0))
                    .unwrap_or_default();
                let severity_icon = match f.severity {
                    CcSeverity::Error => "\x1b[31m✗\x1b[0m",
                    CcSeverity::Warning => "\x1b[33m⚠\x1b[0m",
                    CcSeverity::Advisory => "\x1b[36mℹ\x1b[0m",
                };
                out.push_str(&format!(
                    "  {} {}/{}::{} ↔ {}/{}::{}{}\n",
                    severity_icon,
                    f.crate_a,
                    f.file_a,
                    f.function_a,
                    f.crate_b,
                    f.file_b,
                    f.function_b,
                    sim_str
                ));
                out.push_str(&format!("    → {}\n", f.recommendation));
            }

            if rule_findings.len() > 20 {
                out.push_str(&format!(
                    "  ... and {} more\n",
                    rule_findings.len() - 20
                ));
            }
            out.push('\n');
        }
    }

    // Summary
    out.push_str(&format!(
        "\x1b[1mSummary:\x1b[0m {} findings ({} errors, {} warnings, {} advisories)\n",
        report.summary.total_findings,
        report.summary.errors,
        report.summary.warnings,
        report.summary.advisories,
    ));

    out
}

fn format_markdown(report: &CrossCrateReport) -> String {
    let mut out = String::new();

    out.push_str("# Cross-Crate Duplication Report\n\n");
    out.push_str(&format!(
        "**Crates analyzed:** {}\n\n",
        report.crates_analyzed.join(", ")
    ));

    if report.findings.is_empty() {
        out.push_str("No cross-crate duplication findings.\n");
        return out;
    }

    out.push_str("| Rule | Severity | Crate A | Crate B | Function | Similarity | Recommendation |\n");
    out.push_str("|------|----------|---------|---------|----------|------------|----------------|\n");

    for f in &report.findings {
        let sim_str = f
            .similarity
            .map(|s| format!("{:.0}%", s * 100.0))
            .unwrap_or_else(|| "—".to_string());

        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            f.rule,
            f.severity,
            f.crate_a,
            f.crate_b,
            f.function_a,
            sim_str,
            f.recommendation,
        ));
    }

    out.push_str(&format!(
        "\n**Summary:** {} findings ({} errors, {} warnings, {} advisories)\n",
        report.summary.total_findings,
        report.summary.errors,
        report.summary.warnings,
        report.summary.advisories,
    ));

    out
}
