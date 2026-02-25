fn calculate_spec_score(spec: &ParsedSpec) -> f64 {
    // Simplified scoring based on spec requirements
    let mut score = 0.0;

    // Issue refs (10 pts)
    if !spec.issue_refs.is_empty() {
        score += 10.0;
    }

    // Code examples (20 pts, 4 pts each up to 5)
    score += (spec.code_examples.len().min(5) * 4) as f64;

    // Acceptance criteria (30 pts, 3 pts each up to 10)
    score += (spec.acceptance_criteria.len().min(10) * 3) as f64;

    // Claims (20 pts based on count)
    score += (spec.claims.len().min(20)) as f64;

    // Title exists (5 pts)
    if !spec.title.is_empty() {
        score += 5.0;
    }

    // Test requirements (15 pts, 3 pts each up to 5)
    score += (spec.test_requirements.len().min(5) * 3) as f64;

    score.min(100.0)
}

fn format_spec_score_text(spec: &ParsedSpec, score: f64, verbose: bool) -> String {
    let mut out = String::new();

    out.push_str("📋 Specification Score\n");
    out.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n");

    if !spec.title.is_empty() {
        out.push_str(&format!("Title: {}\n", spec.title));
    }

    out.push_str(&format!("Score: {:.1}/100\n", score));
    out.push_str(&format!(
        "Status: {}\n",
        if score >= 95.0 {
            "✅ PASS"
        } else {
            "❌ FAIL (needs ≥95)"
        }
    ));

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

    out.push_str("# Specification Score Report\n\n");

    if !spec.title.is_empty() {
        out.push_str(&format!("**Title:** {}\n\n", spec.title));
    }

    out.push_str("| Metric | Value |\n");
    out.push_str("|--------|-------|\n");
    out.push_str(&format!("| Score | {:.1}/100 |\n", score));
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
