#![cfg_attr(coverage_nightly, coverage(off))]

use super::types::QueryResult;

/// Format results as JSON
pub fn format_json(results: &[QueryResult]) -> Result<String, String> {
    serde_json::to_string_pretty(results).map_err(|e| format!("JSON serialization failed: {e}"))
}

// ─── Shared metric helpers (extracted for complexity reduction) ───

fn format_coverage_metrics_md(r: &QueryResult, out: &mut String) {
    match r.coverage_status.as_str() {
        "uncovered" => {
            out.push_str(&format!(" | 🛡️ **Uncovered (0/{} lines)**", r.lines_total));
        }
        "partial" => {
            out.push_str(&format!(
                " | 🛡️ **Coverage: {:.0}%** ({}/{})",
                r.line_coverage_pct, r.lines_covered, r.lines_total
            ));
            out.push_str(&format!(" | ⚠️ **{} missed lines**", r.missed_lines));
        }
        "full" => {
            out.push_str(&format!(" | 🛡️ **Fully covered** ({} lines)", r.lines_total));
        }
        _ => {}
    }
    if r.impact_score > 1.0 {
        out.push_str(&format!(" | 📈 **Impact: {:.1}**", r.impact_score));
    }
    format_coverage_diff_md(r.coverage_diff, out);
}

fn format_coverage_diff_md(diff: f32, out: &mut String) {
    if diff > 0.0 {
        out.push_str(&format!(" | ✅ **+{:.1}% coverage**", diff));
    } else if diff < 0.0 {
        out.push_str(&format!(" | ❌ **{:.1}% coverage**", diff));
    }
}

fn format_coverage_metrics_text(r: &QueryResult, out: &mut String) {
    match r.coverage_status.as_str() {
        "uncovered" => {
            out.push_str(&format!(" | \x1b[1;31m🛡️ Uncovered (0/{})\x1b[0m", r.lines_total));
        }
        "partial" => {
            let cov_color = if r.line_coverage_pct < 50.0 { "\x1b[1;31m" } else if r.line_coverage_pct < 80.0 { "\x1b[33m" } else { "\x1b[32m" };
            out.push_str(&format!(
                " | {}🛡️ Cov: {:.0}% ({}/{})\x1b[0m",
                cov_color, r.line_coverage_pct, r.lines_covered, r.lines_total
            ));
        }
        "full" => {
            out.push_str(&format!(" | \x1b[32m🛡️ Covered ({} lines)\x1b[0m", r.lines_total));
        }
        _ => {}
    }
    if r.impact_score > 1.0 {
        out.push_str(&format!(" | \x1b[1;33m📈 Impact: {:.1}\x1b[0m", r.impact_score));
    }
    format_coverage_diff_text(r.coverage_diff, out);
}

fn format_coverage_diff_text(diff: f32, out: &mut String) {
    if diff > 0.0 {
        out.push_str(&format!(" | \x1b[1;32m✅ +{:.1}% cov\x1b[0m", diff));
    } else if diff < 0.0 {
        out.push_str(&format!(" | \x1b[1;31m❌ {:.1}% cov\x1b[0m", diff));
    }
}

#[allow(clippy::incompatible_msrv)]
fn truncate_doc(doc: &str) -> String {
    let first_line = doc.lines().next().unwrap_or(doc);
    if first_line.len() > 100 {
        format!("{}...", &first_line[..first_line.floor_char_boundary(97)])
    } else {
        first_line.to_string()
    }
}

// ─── Metrics builders for format_text_with_code ───

fn build_rich_metrics(r: &QueryResult) -> Vec<String> {
    let mut metrics = Vec::new();
    metrics.push(format!("C:{}", r.complexity));
    metrics.push(format!("L:{}", r.loc));
    push_pagerank_metric(r, &mut metrics);
    push_indegree_metric(r, &mut metrics);
    push_churn_metric_rich(r, &mut metrics);
    push_entropy_metric(r, &mut metrics);
    if r.satd_count > 0 {
        metrics.push(format!("\x1b[1;33m⚠{}\x1b[0m", r.satd_count));
    }
    if r.clone_count > 0 {
        metrics.push(format!("\x1b[1;35m📋{}\x1b[0m", r.clone_count));
    }
    push_coverage_metric_rich(r, &mut metrics);
    push_fault_metric_rich(r, &mut metrics);
    metrics
}

fn push_pagerank_metric(r: &QueryResult, metrics: &mut Vec<String>) {
    if r.pagerank <= 0.0 {
        return;
    }
    let pr_scaled = r.pagerank * 10000.0;
    if pr_scaled >= 10.0 {
        metrics.push(format!("\x1b[1;36m★{:.0}\x1b[0m", pr_scaled));
    } else if pr_scaled >= 1.0 {
        metrics.push(format!("★{:.1}", pr_scaled));
    }
}

fn push_indegree_metric(r: &QueryResult, metrics: &mut Vec<String>) {
    if r.in_degree >= 5 {
        metrics.push(format!("\x1b[1;32m↓{}\x1b[0m", r.in_degree));
    } else if r.in_degree > 0 {
        metrics.push(format!("↓{}", r.in_degree));
    }
}

fn push_churn_metric_rich(r: &QueryResult, metrics: &mut Vec<String>) {
    if r.commit_count == 0 {
        return;
    }
    if r.churn_score > 0.7 {
        metrics.push(format!(
            "\x1b[1;31m🔥{}c {:.0}%\x1b[0m",
            r.commit_count,
            r.churn_score * 100.0
        ));
    } else if r.churn_score > 0.3 {
        metrics.push(format!("{}c {:.0}%", r.commit_count, r.churn_score * 100.0));
    } else {
        metrics.push(format!("{}c", r.commit_count));
    }
}

fn push_entropy_metric(r: &QueryResult, metrics: &mut Vec<String>) {
    if r.pattern_diversity <= 0.0 {
        return;
    }
    if r.pattern_diversity < 0.3 {
        metrics.push(format!(
            "\x1b[2m🔄{:.0}%\x1b[0m",
            r.pattern_diversity * 100.0
        ));
    } else if r.pattern_diversity > 0.8 {
        metrics.push(format!("H:{:.0}%", r.pattern_diversity * 100.0));
    }
}

fn push_coverage_metric_rich(r: &QueryResult, metrics: &mut Vec<String>) {
    match r.coverage_status.as_str() {
        "uncovered" => {
            metrics.push(format!(
                "\x1b[1;31m\u{1f6e1}\u{fe0f}0/{}\x1b[0m",
                r.lines_total
            ));
        }
        "partial" => {
            let fmt = if r.line_coverage_pct < 50.0 {
                format!(
                    "\x1b[1;31m\u{1f6e1}\u{fe0f}{:.0}%\x1b[0m",
                    r.line_coverage_pct
                )
            } else if r.line_coverage_pct < 80.0 {
                format!("\u{1f6e1}\u{fe0f}{:.0}%", r.line_coverage_pct)
            } else {
                format!(
                    "\x1b[32m\u{1f6e1}\u{fe0f}{:.0}%\x1b[0m",
                    r.line_coverage_pct
                )
            };
            metrics.push(fmt);
        }
        "full" => {
            metrics.push("\x1b[32m\u{1f6e1}\u{fe0f}100%\x1b[0m".to_string());
        }
        _ => {}
    }
    if r.impact_score > 1.0 {
        metrics.push(format!(
            "\x1b[1;33m\u{1f4c8}{:.1}\x1b[0m",
            r.impact_score
        ));
    }
    if r.coverage_diff > 0.0 {
        metrics.push(format!("\x1b[1;32m+{:.1}%\x1b[0m", r.coverage_diff));
    } else if r.coverage_diff < 0.0 {
        metrics.push(format!("\x1b[1;31m{:.1}%\x1b[0m", r.coverage_diff));
    }
}

fn push_fault_metric_rich(r: &QueryResult, metrics: &mut Vec<String>) {
    if r.fault_annotations.is_empty() {
        return;
    }
    let first = r
        .fault_annotations
        .first()
        .map_or("", |s| s.split(':').next().unwrap_or(s));
    metrics.push(format!(
        "\x1b[1;91m🐛{}:{}\x1b[0m",
        r.fault_annotations.len(),
        first
    ));
}

fn format_call_graph(r: &QueryResult) -> Option<String> {
    if r.calls.is_empty() && r.called_by.is_empty() {
        return None;
    }
    let mut parts = Vec::new();
    if !r.calls.is_empty() {
        let calls_str = if r.calls.len() <= 5 {
            r.calls.join(", ")
        } else {
            format!(
                "{}, (+{} more)",
                r.calls[..5].join(", "),
                r.calls.len() - 5
            )
        };
        parts.push(format!("calls: {}", calls_str));
    }
    if !r.called_by.is_empty() {
        let called_str = if r.called_by.len() <= 3 {
            r.called_by.join(", ")
        } else {
            format!(
                "{}, (+{} more)",
                r.called_by[..3].join(", "),
                r.called_by.len() - 3
            )
        };
        parts.push(format!("← {}", called_str));
    }
    Some(parts.join(" │ "))
}

fn format_fault_lines(faults: &[String], output: &mut String) {
    for fault in faults {
        if fault.contains("Boundary") || fault.contains("condition") {
            output.push_str(&format!("\x1b[1;33m⚠️  {}\x1b[0m\n", fault));
        } else if fault.contains("Arithmetic") {
            output.push_str(&format!("\x1b[1;31m⚠️  {}\x1b[0m\n", fault));
        } else {
            output.push_str(&format!("\x1b[1;35m⚠️  {}\x1b[0m\n", fault));
        }
    }
}

/// Highlight matching text in a single line for grep-like output.
/// For literal mode (`is_regex=false`), does case-insensitive substring matching.
/// For regex mode (`is_regex=true`), uses regex pattern matching.
fn highlight_matches_in_line(line: &str, pattern: &str, is_regex: bool) -> String {
    const HL_START: &str = "\x1b[1;43m"; // Bold + yellow background
    const HL_END: &str = "\x1b[0m";

    if is_regex {
        if let Ok(re) = regex::Regex::new(pattern) {
            let mut result = String::new();
            let mut last = 0;
            for m in re.find_iter(line) {
                result.push_str(&line[last..m.start()]);
                result.push_str(HL_START);
                result.push_str(m.as_str());
                result.push_str(HL_END);
                last = m.end();
            }
            result.push_str(&line[last..]);
            result
        } else {
            line.to_string()
        }
    } else {
        // Case-insensitive literal replacement preserving original case
        let lower_line = line.to_lowercase();
        let lower_pattern = pattern.to_lowercase();
        if lower_pattern.is_empty() {
            return line.to_string();
        }
        let mut result = String::new();
        let mut pos = 0;
        while let Some(idx) = lower_line[pos..].find(&lower_pattern) {
            let abs_idx = pos + idx;
            result.push_str(&line[pos..abs_idx]);
            result.push_str(HL_START);
            result.push_str(&line[abs_idx..abs_idx + pattern.len()]);
            result.push_str(HL_END);
            pos = abs_idx + pattern.len();
        }
        result.push_str(&line[pos..]);
        result
    }
}

fn highlight_source(source: &str, file_path: &str, output: &mut String, start_line: usize, highlight: Option<(&str, bool)>) {
    if let Some((pattern, is_regex)) = highlight {
        // Match highlighting mode: line numbers + yellow highlight on matches
        for (i, line) in source.lines().enumerate() {
            let line_num = start_line + i;
            let highlighted = highlight_matches_in_line(line, pattern, is_regex);
            output.push_str(&format!("\x1b[2m{:>4}\x1b[0m\u{2502} {}\n", line_num, highlighted));
        }
    } else {
        // Syntect syntax highlighting mode
        use syntect::easy::HighlightLines;
        use syntect::highlighting::ThemeSet;
        use syntect::parsing::SyntaxSet;
        use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

        let ps = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();
        let theme = &ts.themes["base16-ocean.dark"];

        let ext = file_path.rsplit('.').next().unwrap_or("rs");
        let syntax = ps
            .find_syntax_by_extension(ext)
            .unwrap_or_else(|| ps.find_syntax_plain_text());
        let mut h = HighlightLines::new(syntax, theme);

        for line in LinesWithEndings::from(source) {
            match h.highlight_line(line, &ps) {
                Ok(ranges) => output.push_str(&as_24_bit_terminal_escaped(&ranges[..], false)),
                Err(_) => output.push_str(line),
            }
        }
        if !source.ends_with('\n') {
            output.push('\n');
        }
        output.push_str("\x1b[0m");
    }
}

// ─── Public formatters ───

/// Format results as markdown
fn build_quality_md(r: &QueryResult) -> String {
    let mut q = format!(
        "**Quality:** TDG {} ({:.1}) | Complexity: {} | Big-O: {}",
        r.tdg_grade, r.tdg_score, r.complexity, r.big_o
    );
    if r.satd_count > 0 {
        q.push_str(&format!(" | ⚠️ **SATD: {}**", r.satd_count));
    }
    push_churn_md(r, &mut q);
    if r.clone_count > 0 {
        q.push_str(&format!(
            " | 📋 **Clones: {} ({:.0}%)**",
            r.clone_count,
            r.duplication_score * 100.0
        ));
    }
    if r.pattern_diversity > 0.0 && r.pattern_diversity < 0.3 {
        q.push_str(&format!(
            " | 🔄 **Repetitive ({:.0}%)**",
            (1.0 - r.pattern_diversity) * 100.0
        ));
    }
    format_coverage_metrics_md(r, &mut q);
    q
}

fn push_churn_md(r: &QueryResult, out: &mut String) {
    if r.churn_score > 0.5 {
        out.push_str(&format!(
            " | 🔥 **Hot: {} commits ({:.0}%)**",
            r.commit_count,
            r.churn_score * 100.0
        ));
    } else if r.commit_count > 0 {
        out.push_str(&format!(" | Commits: {}", r.commit_count));
    }
}

fn format_md_details(r: &QueryResult, output: &mut String) {
    if let Some(doc) = &r.doc_comment {
        output.push_str(&format!("**Documentation:** {}\n\n", doc));
    }
    if !r.calls.is_empty() {
        output.push_str(&format!("**Calls:** {}\n\n", r.calls.join(", ")));
    }
    if !r.called_by.is_empty() {
        output.push_str(&format!("**Called by:** {}\n\n", r.called_by.join(", ")));
    }
    if r.pagerank > 0.0 || r.in_degree > 0 || r.out_degree > 0 {
        output.push_str(&format!(
            "**Graph:** PageRank {:.6} | In-Degree: {} | Out-Degree: {}\n\n",
            r.pagerank, r.in_degree, r.out_degree
        ));
    }
}

pub fn format_markdown(results: &[QueryResult]) -> String {
    let mut output = String::new();
    output.push_str(&format!(
        "# Search Results ({} functions)\n\n",
        results.len()
    ));

    for (i, r) in results.iter().enumerate() {
        output.push_str(&format!("## {}. `{}`\n\n", i + 1, r.function_name));
        output.push_str(&format!(
            "**Location:** `{}:{}` ({} lines)\n\n",
            r.file_path, r.start_line, r.loc
        ));
        output.push_str(&format!("**Signature:**\n```\n{}\n```\n\n", r.signature));
        output.push_str(&build_quality_md(r));
        output.push_str("\n\n");
        format_md_details(r, &mut output);
        output.push_str(&format!("**Relevance:** {:.2}\n\n", r.relevance_score));
        output.push_str("---\n\n");
    }

    output
}

/// Format results as text with inline source code (agent-friendly)
/// Uses syntect for rich syntax highlighting, or match highlighting for literal/regex modes.
/// `highlight`: `Some((pattern, is_regex))` for grep-like match highlighting, `None` for syntect.
pub fn format_text_with_code(results: &[QueryResult], highlight: Option<(&str, bool)>) -> String {
    let mut output = String::new();

    for r in results.iter() {
        // Header line
        output.push_str(&format!(
            "\x1b[36m{}\x1b[0m:\x1b[33m{}-{}\x1b[0m │ \x1b[1;37m{}\x1b[0m │ TDG: \x1b[32m{}\x1b[0m │ \x1b[35m{}\x1b[0m\n",
            r.file_path, r.start_line, r.end_line, r.function_name, r.tdg_grade, r.big_o
        ));

        // Metrics line
        let metrics = build_rich_metrics(r);
        output.push_str(&format!("   \x1b[2m{}\x1b[0m\n", metrics.join(" │ ")));

        // Doc comment
        if let Some(doc) = &r.doc_comment {
            output.push_str(&format!("   \x1b[3;37m/// {}\x1b[0m\n", truncate_doc(doc)));
        }

        // Call graph
        if let Some(graph) = format_call_graph(r) {
            output.push_str(&format!("   \x1b[2;36m{}\x1b[0m\n", graph));
        }

        // Fault annotations
        format_fault_lines(&r.fault_annotations, &mut output);

        // Source code
        if let Some(source) = &r.source {
            highlight_source(source, &r.file_path, &mut output, r.start_line, highlight);
        } else {
            output.push_str("\x1b[2m// (source hidden; omit --summary to show)\x1b[0m\n");
        }

        output.push('\n');
    }

    output
}

fn build_text_metrics(r: &QueryResult) -> String {
    let grade_color = match r.tdg_grade.as_str() {
        "A" | "B" => "\x1b[32m",
        "C" => "\x1b[33m",
        "D" => "\x1b[31m",
        "F" => "\x1b[1;31m",
        _ => "\x1b[2m",
    };
    let mut m = format!(
        "   TDG: {}{} ({:.1})\x1b[0m | Complexity: {} | Big-O: \x1b[35m{}\x1b[0m",
        grade_color, r.tdg_grade, r.tdg_score, r.complexity, r.big_o
    );
    if r.satd_count > 0 {
        m.push_str(&format!(" | \x1b[1;33m⚠️ SATD: {}\x1b[0m", r.satd_count));
    }
    if r.loc > 50 {
        m.push_str(&format!(" | LOC: {}", r.loc));
    }
    push_churn_text(r, &mut m);
    if r.clone_count > 0 {
        m.push_str(&format!(
            " | \x1b[1;35m📋 Clones: {} ({:.0}%)\x1b[0m",
            r.clone_count,
            r.duplication_score * 100.0
        ));
    }
    if r.pattern_diversity > 0.0 && r.pattern_diversity < 0.3 {
        m.push_str(&format!(
            " | \x1b[2m🔄 Repetitive ({:.0}%)\x1b[0m",
            (1.0 - r.pattern_diversity) * 100.0
        ));
    }
    format_coverage_metrics_text(r, &mut m);
    m
}

fn push_churn_text(r: &QueryResult, out: &mut String) {
    if r.churn_score > 0.5 {
        out.push_str(&format!(
            " | \x1b[1;31m🔥 Hot: {} commits ({:.0}%)\x1b[0m",
            r.commit_count,
            r.churn_score * 100.0
        ));
    } else if r.commit_count > 0 {
        out.push_str(&format!(" | \x1b[2m{}c\x1b[0m", r.commit_count));
    }
}

fn format_text_details(r: &QueryResult, output: &mut String) {
    if !r.fault_annotations.is_empty() {
        for fault in &r.fault_annotations {
            output.push_str(&format!("   \x1b[1;35m⚠️ {}\x1b[0m\n", fault));
        }
    }
    if let Some(doc) = &r.doc_comment {
        output.push_str(&format!("   \x1b[3;37mDoc: {}\x1b[0m\n", doc));
    }
    if !r.calls.is_empty() {
        output.push_str(&format!("   \x1b[2;36mCalls: {}\x1b[0m\n", r.calls.join(", ")));
    }
    if !r.called_by.is_empty() {
        output.push_str(&format!("   \x1b[2;36mCalled by: {}\x1b[0m\n", r.called_by.join(", ")));
    }
    if r.pagerank > 0.0 || r.in_degree > 0 || r.out_degree > 0 {
        output.push_str(&format!(
            "   \x1b[2mGraph: PageRank {:.6} | In-Degree: {} | Out-Degree: {}\x1b[0m\n",
            r.pagerank, r.in_degree, r.out_degree
        ));
    }
}

/// Format results as text (colorized for terminal)
pub fn format_text(results: &[QueryResult]) -> String {
    let mut output = String::new();
    output.push_str(&format!("\x1b[1mFound {} functions:\x1b[0m\n\n", results.len()));

    for (i, r) in results.iter().enumerate() {
        output.push_str(&format!(
            "\x1b[2m{}.\x1b[0m \x1b[36m{}\x1b[0m:\x1b[33m{}\x1b[0m \x1b[2m-\x1b[0m \x1b[1;37m{}\x1b[0m\n",
            i + 1, r.file_path, r.start_line, r.function_name
        ));
        output.push_str(&format!("   \x1b[2mSignature:\x1b[0m {}\n", r.signature));
        output.push_str(&build_text_metrics(r));
        output.push('\n');
        format_text_details(r, &mut output);
        let rel_color = if r.relevance_score > 0.7 { "\x1b[1;32m" } else if r.relevance_score > 0.3 { "\x1b[32m" } else { "\x1b[2m" };
        output.push_str(&format!("   Relevance: {}{:.2}\x1b[0m\n\n", rel_color, r.relevance_score));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_result(name: &str, doc: Option<&str>) -> QueryResult {
        QueryResult {
            function_name: name.to_string(),
            file_path: "src/test.rs".to_string(),
            signature: format!("fn {}()", name),
            definition_type: "function".to_string(),
            doc_comment: doc.map(|s| s.to_string()),
            start_line: 1,
            end_line: 10,
            language: "Rust".to_string(),
            tdg_score: 80.0,
            tdg_grade: "A".to_string(),
            complexity: 5,
            big_o: "O(1)".to_string(),
            satd_count: 0,
            loc: 10,
            relevance_score: 0.95,
            source: None,
            calls: Vec::new(),
            called_by: Vec::new(),
            pagerank: 0.0,
            in_degree: 0,
            out_degree: 0,
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            duplication_score: 0.0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
            line_coverage_pct: 0.0,
            lines_covered: 0,
            lines_total: 0,
            missed_lines: 0,
            impact_score: 0.0,
            coverage_status: String::new(),
            coverage_diff: 0.0,
            coverage_exclusion: Default::default(),
            coverage_excluded: false,
            cross_project_callers: 0,
        }
    }

    /// Regression test for #157: UTF-8 multi-byte char boundary panic
    #[test]
    fn test_format_text_with_code_multibyte_doc_comment() {
        let result = make_result(
            "verify_output",
            Some("Verify output is correct: not empty, no garbage, contains expected answer (PMAT-QA-PROTOCOL-001 §7.5)  Order of checks is CRITICAL for safety"),
        );
        let output = format_text_with_code(&[result], None);
        assert!(output.contains("verify_output"));
        assert!(output.contains("..."));
    }

    #[test]
    fn test_format_text_short_doc_no_truncation() {
        let result = make_result("foo", Some("Short doc"));
        let output = format_text_with_code(&[result], None);
        assert!(output.contains("Short doc"));
    }

    #[test]
    fn test_highlight_matches_literal() {
        let line = "let result = unwrap();";
        let out = highlight_matches_in_line(line, "unwrap()", false);
        assert!(out.contains("\x1b[1;43m"), "missing highlight start");
        assert!(out.contains("unwrap()"), "missing matched text");
        assert!(out.contains("\x1b[0m"), "missing highlight end");
    }

    #[test]
    fn test_highlight_matches_literal_case_insensitive() {
        let line = "fn HandleRequest() {}";
        let out = highlight_matches_in_line(line, "handlerequest", false);
        // Should highlight preserving original case
        assert!(out.contains("HandleRequest"));
        assert!(out.contains("\x1b[1;43m"));
    }

    #[test]
    fn test_highlight_matches_regex() {
        let line = "fn handle_request(ctx: Context) {}";
        let out = highlight_matches_in_line(line, r"fn\s+handle_\w+", true);
        assert!(out.contains("\x1b[1;43m"));
        assert!(out.contains("fn handle_request"));
    }

    #[test]
    fn test_highlight_matches_no_match() {
        let line = "let x = 42;";
        let out = highlight_matches_in_line(line, "nonexistent", false);
        assert_eq!(out, line);
    }

    #[test]
    fn test_highlight_matches_invalid_regex() {
        let line = "some text here";
        let out = highlight_matches_in_line(line, "[invalid", true);
        assert_eq!(out, line);
    }

    #[test]
    fn test_format_text_with_code_literal_highlight() {
        let mut result = make_result("test_fn", None);
        result.source = Some("fn test_fn() {\n    unwrap();\n}".to_string());
        let output = format_text_with_code(&[result], Some(("unwrap()", false)));
        assert!(output.contains("\x1b[1;43m"), "missing highlight in output");
        // Should have line numbers in highlight mode
        assert!(output.contains("\u{2502}"), "missing line number separator");
    }
}
