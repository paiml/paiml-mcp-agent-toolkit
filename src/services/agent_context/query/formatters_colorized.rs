// Colorized terminal text formatting for query results: format_text and format_text_with_code.
// Included into formatters.rs -- do NOT add `use` imports or `#!` inner attributes here.

/// Format results as text with inline source code (agent-friendly)
/// Uses syntect for rich syntax highlighting, or match highlighting for literal/regex modes.
/// `highlight`: `Some((pattern, is_regex))` for grep-like match highlighting, `None` for syntect.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
        output.push_str(&format!(
            "   \x1b[2;36mCalls: {}\x1b[0m\n",
            r.calls.join(", ")
        ));
    }
    if !r.called_by.is_empty() {
        output.push_str(&format!(
            "   \x1b[2;36mCalled by: {}\x1b[0m\n",
            r.called_by.join(", ")
        ));
    }
    if r.pagerank > 0.0 || r.in_degree > 0 || r.out_degree > 0 {
        output.push_str(&format!(
            "   \x1b[2mGraph: PageRank {:.6} | In-Degree: {} | Out-Degree: {}\x1b[0m\n",
            r.pagerank, r.in_degree, r.out_degree
        ));
    }
}

/// Format results as text (colorized for terminal)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_text(results: &[QueryResult]) -> String {
    let mut output = String::new();
    output.push_str(&format!(
        "\x1b[1mFound {} functions:\x1b[0m\n\n",
        results.len()
    ));

    for (i, r) in results.iter().enumerate() {
        output.push_str(&format!(
            "\x1b[2m{}.\x1b[0m \x1b[36m{}\x1b[0m:\x1b[33m{}\x1b[0m \x1b[2m-\x1b[0m \x1b[1;37m{}\x1b[0m\n",
            i + 1, r.file_path, r.start_line, r.function_name
        ));
        output.push_str(&format!("   \x1b[2mSignature:\x1b[0m {}\n", r.signature));
        output.push_str(&build_text_metrics(r));
        output.push('\n');
        format_text_details(r, &mut output);
        let rel_color = if r.relevance_score > 0.7 {
            "\x1b[1;32m"
        } else if r.relevance_score > 0.3 {
            "\x1b[32m"
        } else {
            "\x1b[2m"
        };
        output.push_str(&format!(
            "   Relevance: {}{:.2}\x1b[0m\n\n",
            rel_color, r.relevance_score
        ));
    }

    output
}
