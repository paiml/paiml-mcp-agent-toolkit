#![cfg_attr(coverage_nightly, coverage(off))]

use super::types::QueryResult;

/// Format results as JSON
pub fn format_json(results: &[QueryResult]) -> Result<String, String> {
    serde_json::to_string_pretty(results).map_err(|e| format!("JSON serialization failed: {e}"))
}

/// Format results as markdown
pub fn format_markdown(results: &[QueryResult]) -> String {
    let mut output = String::new();

    output.push_str(&format!("# Search Results ({} functions)\n\n", results.len()));

    for (i, r) in results.iter().enumerate() {
        output.push_str(&format!("## {}. `{}`\n\n", i + 1, r.function_name));
        output.push_str(&format!(
            "**Location:** `{}:{}` ({} lines)\n\n",
            r.file_path, r.start_line, r.loc
        ));
        output.push_str(&format!("**Signature:**\n```\n{}\n```\n\n", r.signature));
        // Quality metrics with SATD warning
        let mut quality = format!(
            "**Quality:** TDG {} ({:.1}) | Complexity: {} | Big-O: {}",
            r.tdg_grade, r.tdg_score, r.complexity, r.big_o
        );
        if r.satd_count > 0 {
            quality.push_str(&format!(" | ⚠️ **SATD: {}**", r.satd_count));
        }
        // Add churn indicator for volatile files
        if r.churn_score > 0.5 {
            quality.push_str(&format!(" | 🔥 **Hot: {} commits ({:.0}%)**", r.commit_count, r.churn_score * 100.0));
        } else if r.commit_count > 0 {
            quality.push_str(&format!(" | Commits: {}", r.commit_count));
        }
        // Add duplication indicator
        if r.clone_count > 0 {
            quality.push_str(&format!(" | 📋 **Clones: {} ({:.0}%)**", r.clone_count, r.duplication_score * 100.0));
        }
        // Add entropy indicator for low pattern diversity
        if r.pattern_diversity > 0.0 && r.pattern_diversity < 0.3 {
            quality.push_str(&format!(" | 🔄 **Repetitive ({:.0}%)**", (1.0 - r.pattern_diversity) * 100.0));
        }
        output.push_str(&quality);
        output.push_str("\n\n");

        if let Some(doc) = &r.doc_comment {
            output.push_str(&format!("**Documentation:** {}\n\n", doc));
        }

        if !r.calls.is_empty() {
            output.push_str(&format!("**Calls:** {}\n\n", r.calls.join(", ")));
        }
        if !r.called_by.is_empty() {
            output.push_str(&format!("**Called by:** {}\n\n", r.called_by.join(", ")));
        }

        // Show graph metrics if significant
        if r.pagerank > 0.0 || r.in_degree > 0 || r.out_degree > 0 {
            output.push_str(&format!(
                "**Graph:** PageRank {:.6} | In-Degree: {} | Out-Degree: {}\n\n",
                r.pagerank, r.in_degree, r.out_degree
            ));
        }

        output.push_str(&format!("**Relevance:** {:.2}\n\n", r.relevance_score));
        output.push_str("---\n\n");
    }

    output
}

/// Format results as text with inline source code (agent-friendly)
/// Uses syntect for rich syntax highlighting
pub fn format_text_with_code(results: &[QueryResult]) -> String {
    use syntect::easy::HighlightLines;
    use syntect::highlighting::ThemeSet;
    use syntect::parsing::SyntaxSet;
    use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

    let mut output = String::new();

    // Load syntax definitions and theme
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["base16-ocean.dark"];

    for r in results.iter() {
        // Header with rich colors
        // Cyan for file path, yellow for function name, green for TDG, magenta for Big-O
        output.push_str(&format!(
            "\x1b[36m{}\x1b[0m:\x1b[33m{}-{}\x1b[0m │ \x1b[1;37m{}\x1b[0m │ TDG: \x1b[32m{}\x1b[0m │ \x1b[35m{}\x1b[0m\n",
            r.file_path, r.start_line, r.end_line, r.function_name, r.tdg_grade, r.big_o
        ));

        // Metrics line - always show key metrics for agent decision-making
        let mut metrics = Vec::new();

        // Core metrics (always show)
        metrics.push(format!("C:{}", r.complexity));
        metrics.push(format!("L:{}", r.loc));

        // PageRank - show importance score (higher = more central to codebase)
        if r.pagerank > 0.0 {
            let pr_scaled = r.pagerank * 10000.0;
            if pr_scaled >= 10.0 {
                metrics.push(format!("\x1b[1;36m★{:.0}\x1b[0m", pr_scaled));
            } else if pr_scaled >= 1.0 {
                metrics.push(format!("★{:.1}", pr_scaled));
            }
        }

        // In-degree (callers) - shows how widely used
        if r.in_degree >= 5 {
            metrics.push(format!("\x1b[1;32m↓{}\x1b[0m", r.in_degree));
        } else if r.in_degree > 0 {
            metrics.push(format!("↓{}", r.in_degree));
        }

        // Churn - git volatility (commit count and churn score)
        if r.commit_count > 0 {
            if r.churn_score > 0.7 {
                metrics.push(format!("\x1b[1;31m🔥{}c {:.0}%\x1b[0m", r.commit_count, r.churn_score * 100.0));
            } else if r.churn_score > 0.3 {
                metrics.push(format!("{}c {:.0}%", r.commit_count, r.churn_score * 100.0));
            } else {
                metrics.push(format!("{}c", r.commit_count));
            }
        }

        // Pattern diversity / entropy (lower = more repetitive code patterns)
        if r.pattern_diversity > 0.0 {
            if r.pattern_diversity < 0.3 {
                metrics.push(format!("\x1b[2m🔄{:.0}%\x1b[0m", r.pattern_diversity * 100.0));
            } else if r.pattern_diversity > 0.8 {
                metrics.push(format!("H:{:.0}%", r.pattern_diversity * 100.0));
            }
        }

        // SATD (tech debt markers)
        if r.satd_count > 0 {
            metrics.push(format!("\x1b[1;33m⚠{}\x1b[0m", r.satd_count));
        }

        // Clone count (duplicates)
        if r.clone_count > 0 {
            metrics.push(format!("\x1b[1;35m📋{}\x1b[0m", r.clone_count));
        }

        // Fault annotations (Tarantula-style defect suspiciousness)
        if !r.fault_annotations.is_empty() {
            // Show count and first annotation type
            let first = r.fault_annotations.first().map_or("", |s| {
                s.split(':').next().unwrap_or(s)
            });
            metrics.push(format!("\x1b[1;91m🐛{}:{}\x1b[0m", r.fault_annotations.len(), first));
        }

        output.push_str(&format!("   \x1b[2m{}\x1b[0m\n", metrics.join(" │ ")));

        // Doc comment (important context for understanding intent)
        if let Some(doc) = &r.doc_comment {
            // Truncate long docs, show first line
            let first_line = doc.lines().next().unwrap_or(doc);
            let truncated = if first_line.len() > 100 {
                format!("{}...", &first_line[..97])
            } else {
                first_line.to_string()
            };
            output.push_str(&format!("   \x1b[3;37m/// {}\x1b[0m\n", truncated));
        }

        // Call graph (useful for navigation)
        if !r.calls.is_empty() || !r.called_by.is_empty() {
            let mut graph_parts = Vec::new();
            if !r.calls.is_empty() {
                let calls_str = if r.calls.len() <= 5 {
                    r.calls.join(", ")
                } else {
                    format!("{}, (+{} more)", r.calls[..5].join(", "), r.calls.len() - 5)
                };
                graph_parts.push(format!("calls: {}", calls_str));
            }
            if !r.called_by.is_empty() {
                let called_str = if r.called_by.len() <= 3 {
                    r.called_by.join(", ")
                } else {
                    format!("{}, (+{} more)", r.called_by[..3].join(", "), r.called_by.len() - 3)
                };
                graph_parts.push(format!("← {}", called_str));
            }
            output.push_str(&format!("   \x1b[2;36m{}\x1b[0m\n", graph_parts.join(" │ ")));
        }

        // Fault annotations with red/yellow warning colors
        for fault in &r.fault_annotations {
            if fault.contains("Boundary") || fault.contains("condition") {
                output.push_str(&format!("\x1b[1;33m⚠️  {}\x1b[0m\n", fault)); // Yellow for boundary
            } else if fault.contains("Arithmetic") {
                output.push_str(&format!("\x1b[1;31m⚠️  {}\x1b[0m\n", fault)); // Red for arithmetic
            } else {
                output.push_str(&format!("\x1b[1;35m⚠️  {}\x1b[0m\n", fault)); // Magenta for others
            }
        }

        // Source code with syntax highlighting
        if let Some(source) = &r.source {
            // Detect language from file extension
            let syntax = ps
                .find_syntax_by_extension(
                    r.file_path
                        .rsplit('.')
                        .next()
                        .unwrap_or("rs"),
                )
                .unwrap_or_else(|| ps.find_syntax_plain_text());

            let mut h = HighlightLines::new(syntax, theme);

            for line in LinesWithEndings::from(source) {
                match h.highlight_line(line, &ps) {
                    Ok(ranges) => {
                        let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
                        output.push_str(&escaped);
                    }
                    Err(_) => {
                        output.push_str(line);
                    }
                }
            }

            if !source.ends_with('\n') {
                output.push('\n');
            }
            // Reset colors after code block
            output.push_str("\x1b[0m");
        } else {
            output.push_str("\x1b[2m// (use --include-source to see code)\x1b[0m\n");
        }

        output.push('\n');
    }

    output
}

/// Format results as text
pub fn format_text(results: &[QueryResult]) -> String {
    let mut output = String::new();

    output.push_str(&format!("Found {} functions:\n\n", results.len()));

    for (i, r) in results.iter().enumerate() {
        output.push_str(&format!(
            "{}. {}:{} - {}\n",
            i + 1,
            r.file_path,
            r.start_line,
            r.function_name
        ));
        output.push_str(&format!("   Signature: {}\n", r.signature));
        // Core metrics line
        let mut metrics = format!(
            "   TDG: {} ({:.1}) | Complexity: {} | Big-O: {}",
            r.tdg_grade, r.tdg_score, r.complexity, r.big_o
        );

        // Add SATD warning if technical debt markers exist
        if r.satd_count > 0 {
            metrics.push_str(&format!(" | ⚠️ SATD: {}", r.satd_count));
        }

        // Add LOC for large functions
        if r.loc > 50 {
            metrics.push_str(&format!(" | LOC: {}", r.loc));
        }

        // Add churn indicator for volatile files
        if r.churn_score > 0.5 {
            metrics.push_str(&format!(" | 🔥 Hot: {} commits ({:.0}%)", r.commit_count, r.churn_score * 100.0));
        } else if r.commit_count > 0 {
            metrics.push_str(&format!(" | Commits: {}", r.commit_count));
        }

        // Add duplication indicator
        if r.clone_count > 0 {
            metrics.push_str(&format!(" | 📋 Clones: {} ({:.0}%)", r.clone_count, r.duplication_score * 100.0));
        }

        // Add entropy indicator for low pattern diversity
        if r.pattern_diversity > 0.0 && r.pattern_diversity < 0.3 {
            metrics.push_str(&format!(" | 🔄 Repetitive ({:.0}%)", (1.0 - r.pattern_diversity) * 100.0));
        }

        output.push_str(&metrics);
        output.push('\n');

        // Add fault annotations if present
        if !r.fault_annotations.is_empty() {
            for fault in &r.fault_annotations {
                output.push_str(&format!("   ⚠️ {}\n", fault));
            }
        }

        if let Some(doc) = &r.doc_comment {
            output.push_str(&format!("   Doc: {}\n", doc));
        }

        if !r.calls.is_empty() {
            output.push_str(&format!("   Calls: {}\n", r.calls.join(", ")));
        }
        if !r.called_by.is_empty() {
            output.push_str(&format!("   Called by: {}\n", r.called_by.join(", ")));
        }

        // Show graph metrics if significant
        if r.pagerank > 0.0 || r.in_degree > 0 || r.out_degree > 0 {
            output.push_str(&format!(
                "   Graph: PageRank {:.6} | In-Degree: {} | Out-Degree: {}\n",
                r.pagerank, r.in_degree, r.out_degree
            ));
        }

        output.push_str(&format!("   Relevance: {:.2}\n\n", r.relevance_score));
    }

    output
}
