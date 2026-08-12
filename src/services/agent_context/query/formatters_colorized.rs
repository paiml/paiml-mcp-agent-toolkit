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
        let pv = r
            .contract_level
            .as_deref()
            .map(|l| format!(" │ PV:{YELLOW}{l}{RESET}"))
            .unwrap_or_default();
        output.push_str(&format!(
            "{CYAN}{}{RESET}:{YELLOW}{}-{}{RESET} │ {BOLD_WHITE}{}{RESET} │ TDG: {GREEN}{}{RESET} │ {MAGENTA}{}{RESET}{pv}\n",
            r.file_path, r.start_line, r.end_line, r.function_name, r.tdg_grade, r.big_o
        ));

        // Metrics line
        let metrics = build_rich_metrics(r);
        output.push_str(&format!("   {DIM}{}{RESET}\n", metrics.join(" │ ")));

        // Doc comment
        if let Some(doc) = &r.doc_comment {
            output.push_str(&format!(
                "   {ITALIC_WHITE}/// {}{RESET}\n",
                truncate_doc(doc)
            ));
        }

        // Call graph
        if let Some(graph) = format_call_graph(r) {
            output.push_str(&format!("   {DIM_CYAN}{}{RESET}\n", graph));
        }

        // Fault annotations
        format_fault_lines(&r.fault_annotations, &mut output);

        // Source code
        if let Some(source) = &r.source {
            highlight_source(source, &r.file_path, &mut output, r.start_line, highlight);
        } else {
            output.push_str(&format!(
                "{DIM}// (source hidden; omit --summary to show){RESET}\n"
            ));
        }

        output.push('\n');
    }

    output
}

fn build_text_metrics(r: &QueryResult) -> String {
    let grade_color = match r.tdg_grade.as_str() {
        "A" | "B" => GREEN,
        "C" => YELLOW,
        "D" => RED,
        "F" => BOLD_RED,
        _ => DIM,
    };
    let mut m = format!(
        "   TDG: {}{} ({:.1}){RESET} | Complexity: {} | Big-O: {MAGENTA}{}{RESET}",
        grade_color, r.tdg_grade, r.tdg_score, r.complexity, r.big_o
    );
    if r.satd_count > 0 {
        m.push_str(&format!(" | {BOLD_YELLOW}⚠️ SATD: {}{RESET}", r.satd_count));
    }
    if r.loc > 50 {
        m.push_str(&format!(" | LOC: {}", r.loc));
    }
    push_churn_text(r, &mut m);
    if r.clone_count > 0 {
        m.push_str(&format!(
            " | {BOLD_MAGENTA}📋 Clones: {} ({:.0}%){RESET}",
            r.clone_count,
            r.duplication_score * 100.0
        ));
    }
    if r.pattern_diversity > 0.0 && r.pattern_diversity < 0.3 {
        m.push_str(&format!(
            " | {DIM}🔄 Repetitive ({:.0}%){RESET}",
            (1.0 - r.pattern_diversity) * 100.0
        ));
    }
    format_coverage_metrics_text(r, &mut m);
    // Contract verification level
    if let Some(ref level) = r.contract_level {
        let pv_color = match level.as_str() {
            "L4" | "L5" => BOLD_GREEN, // green
            "L2" | "L3" => YELLOW,     // yellow
            _ => DIM,                  // dim
        };
        m.push_str(&format!(" | PV:{pv_color}{level}{RESET}"));
        if let Some(ref eq) = r.contract_equation {
            m.push_str(&format!("{DIM}({eq}){RESET}"));
        }
    }
    m
}

fn push_churn_text(r: &QueryResult, out: &mut String) {
    if r.churn_score > 0.5 {
        out.push_str(&format!(
            " | {BOLD_RED}🔥 Hot: {} commits ({:.0}%){RESET}",
            r.commit_count,
            r.churn_score * 100.0
        ));
    } else if r.commit_count > 0 {
        out.push_str(&format!(" | {DIM}{}c{RESET}", r.commit_count));
    }
}

fn format_text_details(r: &QueryResult, output: &mut String) {
    if !r.fault_annotations.is_empty() {
        for fault in &r.fault_annotations {
            output.push_str(&format!("   {BOLD_MAGENTA}⚠️ {}{RESET}\n", fault));
        }
    }
    if let Some(doc) = &r.doc_comment {
        output.push_str(&format!("   {ITALIC_WHITE}Doc: {}{RESET}\n", doc));
    }
    if !r.calls.is_empty() {
        output.push_str(&format!(
            "   {DIM_CYAN}Calls: {}{RESET}\n",
            r.calls.join(", ")
        ));
    }
    if !r.called_by.is_empty() {
        output.push_str(&format!(
            "   {DIM_CYAN}Called by: {}{RESET}\n",
            r.called_by.join(", ")
        ));
    }
    if r.pagerank > 0.0 || r.in_degree > 0 || r.out_degree > 0 {
        output.push_str(&format!(
            "   {DIM}Graph: PageRank {:.6} | In-Degree: {} | Out-Degree: {}{RESET}\n",
            r.pagerank, r.in_degree, r.out_degree
        ));
    }
}

/// Format results as text (colorized for terminal)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_text(results: &[QueryResult]) -> String {
    let mut output = String::new();
    output.push_str(&format!(
        "{BOLD}Found {} functions:{RESET}\n\n",
        results.len()
    ));

    for (i, r) in results.iter().enumerate() {
        output.push_str(&format!(
            "{DIM}{}.{RESET} {CYAN}{}{RESET}:{YELLOW}{}{RESET} {DIM}-{RESET} {BOLD_WHITE}{}{RESET}\n",
            i + 1, r.file_path, r.start_line, r.function_name
        ));
        output.push_str(&format!("   {DIM}Signature:{RESET} {}\n", r.signature));
        output.push_str(&build_text_metrics(r));
        output.push('\n');
        format_text_details(r, &mut output);
        let rel_color = if r.relevance_score > 0.7 {
            BOLD_GREEN
        } else if r.relevance_score > 0.3 {
            GREEN
        } else {
            DIM
        };
        output.push_str(&format!(
            "   Relevance: {}{:.2}{RESET}\n\n",
            rel_color, r.relevance_score
        ));
    }

    output
}
