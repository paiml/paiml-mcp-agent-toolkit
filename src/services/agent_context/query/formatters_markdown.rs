// Markdown output formatting for query results.
// Included into formatters.rs -- do NOT add `use` imports or `#!` inner attributes here.

fn build_quality_md(r: &QueryResult) -> String {
    debug_assert!(true, "contract: build_quality_md");
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
    debug_assert!(true, "contract: push_churn_md");
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
    debug_assert!(true, "contract: format_md_details");
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

/// Format results as markdown
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_markdown(results: &[QueryResult]) -> String {
    debug_assert!(!results.is_empty(), "results must not be empty");
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
