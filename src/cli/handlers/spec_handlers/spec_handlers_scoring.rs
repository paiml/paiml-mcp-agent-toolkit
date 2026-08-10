fn count_citations(content: &str) -> usize {
    // Count unique citation references like [1], [2], etc. in body text
    let mut seen = std::collections::HashSet::new();
    let re = regex::Regex::new(r"\[(\d+)\]").expect("internal error");
    for caps in re.captures_iter(content) {
        if let Some(m) = caps.get(1) {
            seen.insert(m.as_str().to_string());
        }
    }
    seen.len()
}

/// Spec **completeness** score: counts the artefacts a spec carries (issue refs,
/// examples, criteria, claims, title, test requirements, citations).
///
/// This is not the same metric as `pmat qa-work spec`, which verifies each claim
/// and scores claim **falsifiability**. Both used to advertise themselves as "the
/// 100-point Popperian score" and print bare "Score: NN/100" lines with different
/// pass bars (95 here, 60 there), so the same file scored 41.0 FAIL and 50.0 FAIL
/// and readers had no way to tell the two numbers apart. The renderers below name
/// the metric they compute.
fn calculate_spec_score(spec: &ParsedSpec) -> f64 {
    // Scoring based on spec requirements (100 pts total)
    let mut score = 0.0;

    // Issue refs (10 pts)
    if !spec.issue_refs.is_empty() {
        score += 10.0;
    }

    // Code examples (20 pts, 4 pts each up to 5)
    score += (spec.code_examples.len().min(5) * 4) as f64;

    // Acceptance criteria (25 pts, 2.5 pts each up to 10)
    score += (spec.acceptance_criteria.len().min(10)) as f64 * 2.5;

    // Claims (15 pts based on count, cap at 15)
    score += (spec.claims.len().min(15)) as f64;

    // Title exists (5 pts)
    if !spec.title.is_empty() {
        score += 5.0;
    }

    // Test requirements (15 pts, 3 pts each up to 5)
    score += (spec.test_requirements.len().min(5) * 3) as f64;

    // Peer-reviewed citations (10 pts, 2 pts each up to 5)
    let citations = count_citations(&spec.raw_content);
    score += (citations.min(5) * 2) as f64;

    score.min(100.0)
}

fn format_spec_score_text(spec: &ParsedSpec, score: f64, verbose: bool) -> String {
    let mut out = String::new();

    out.push_str("📋 Specification Completeness Score\n");
    out.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n");

    if !spec.title.is_empty() {
        out.push_str(&format!("Title: {}\n", spec.title));
    }

    out.push_str(&format!("Completeness score: {:.1}/100\n", score));
    out.push_str(&format!(
        "Status: {}\n",
        if score >= 95.0 {
            "✅ PASS"
        } else {
            "❌ FAIL (needs ≥95)"
        }
    ));
    out.push_str(
        "Note: counts artefacts present in the spec. For claim verification, run\n      `pmat qa-work spec <file>` (claim falsifiability score, separate bar).\n",
    );

    if verbose {
        out.push_str(&format!("\nClaims: {}\n", spec.claims.len()));
        out.push_str(&format!("Code Examples: {}\n", spec.code_examples.len()));
        out.push_str(&format!(
            "Acceptance Criteria: {}\n",
            spec.acceptance_criteria.len()
        ));
        out.push_str(&format!("Issue Refs: {:?}\n", spec.issue_refs));
    }

    out
}

fn format_spec_score_json(spec: &ParsedSpec, score: f64) -> anyhow::Result<String> {
    let result = serde_json::json!({
        "title": spec.title,
        // Name the metric so consumers cannot conflate it with qa-work spec's
        // claim-falsifiability score, which has its own (60) threshold.
        "metric": "spec_completeness",
        "threshold": 95.0,
        "score": score,
        "passing": score >= 95.0,
        "claims": spec.claims.len(),
        "code_examples": spec.code_examples.len(),
        "acceptance_criteria": spec.acceptance_criteria.len(),
        "issue_refs": spec.issue_refs,
    });
    Ok(serde_json::to_string_pretty(&result)?)
}

fn format_spec_score_markdown(spec: &ParsedSpec, score: f64) -> String {
    let mut out = String::new();

    out.push_str("# Specification Completeness Score Report\n\n");

    if !spec.title.is_empty() {
        out.push_str(&format!("**Title:** {}\n\n", spec.title));
    }

    out.push_str("| Metric | Value |\n");
    out.push_str("|--------|-------|\n");
    out.push_str(&format!("| Completeness score | {:.1}/100 |\n", score));
    out.push_str(&format!(
        "| Status | {} |\n",
        if score >= 95.0 {
            "✅ PASS"
        } else {
            "❌ FAIL"
        }
    ));
    out.push_str(&format!("| Claims | {} |\n", spec.claims.len()));
    out.push_str(&format!(
        "| Code Examples | {} |\n",
        spec.code_examples.len()
    ));
    out.push_str(&format!(
        "| Acceptance Criteria | {} |\n",
        spec.acceptance_criteria.len()
    ));

    out
}
