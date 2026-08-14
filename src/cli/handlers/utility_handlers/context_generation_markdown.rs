/// Generate markdown context (existing path)
fn generate_markdown_context(
    toolchain: &str,
    project_path: &Path,
    context: &crate::services::deep_context::DeepContext,
) -> Result<String> {
    let mut builder = MarkdownBuilder::new();

    // Add header
    builder.content.push_str("# Project Context\n\n");
    builder
        .content
        .push_str(&format!("**Language**: {}\n", toolchain));
    builder
        .content
        .push_str(&format!("**Project Path**: {}\n\n", project_path.display()));

    // Add project structure section
    builder.content.push_str("## Project Structure\n\n");

    // Add project-level metrics summary
    //
    // These counts must describe the document they head. They used to be read
    // from `complexity_report` while the body below is rendered from
    // `analyses.ast_contexts`, so one document carried three mutually different
    // function totals (32420 in this header, 22925 `- **Function**` entries in
    // the body, 33754 summed from the per-file lines). Count what is actually
    // emitted instead.
    let (total_files, total_functions) = count_emitted_body(context);
    builder
        .content
        .push_str(&format!("- **Total Files**: {}\n", total_files));
    builder
        .content
        .push_str(&format!("- **Total Functions**: {}\n", total_functions));
    if let Some(complexity_report) = &context.analyses.complexity_report {
        builder.content.push_str(&format!(
            "- **Median Cyclomatic**: {:.2}\n",
            complexity_report.summary.median_cyclomatic
        ));
        builder.content.push_str(&format!(
            "- **Median Cognitive**: {:.2}\n\n",
            complexity_report.summary.median_cognitive
        ));
    } else {
        builder.content.push('\n');
    }

    // Add quality scorecard
    builder.content.push_str("## Quality Scorecard\n\n");
    use crate::services::deep_context::QualityScorecard;
    // Normalize Overall Health as TDG score (0-100 range)
    let tdg_score = context
        .quality_scorecard
        .overall_health
        .map(|h| h.clamp(0.0, 100.0));
    builder.content.push_str(&format!(
        "- **Overall Health**: {}\n",
        QualityScorecard::render(tdg_score, "%")
    ));
    builder.content.push_str(&format!(
        "- **Maintainability Index**: {}\n",
        QualityScorecard::render(context.quality_scorecard.maintainability_index, "")
    ));
    builder.content.push_str(&format!(
        "- **Complexity Score**: {}\n",
        QualityScorecard::render(context.quality_scorecard.complexity_score, "")
    ));
    if let Some(coverage) = context.quality_scorecard.test_coverage {
        // Normalize test coverage to 0-100 range (remove meaningless percentages)
        let normalized_coverage = coverage.min(100.0).max(0.0);
        builder.content.push_str(&format!(
            "- **Test Coverage**: {:.1}%\n",
            normalized_coverage
        ));
    } else {
        builder.content.push_str("- **Test Coverage**: N/A\n");
    }
    builder.content.push('\n');

    // Add file-level AST with annotations
    builder.content.push_str("## Files\n\n");

    // Use ast_contexts from analyses
    for enhanced_context in &context.analyses.ast_contexts {
        add_simple_file_section(&mut builder, &enhanced_context.base, &context.analyses);
    }

    Ok(builder.content)
}

/// Count the file sections and function entries `generate_markdown_context`
/// will emit into the body, so the header can report them rather than a
/// different analysis pass's numbers.
fn count_emitted_body(context: &crate::services::deep_context::DeepContext) -> (usize, usize) {
    let files = context.analyses.ast_contexts.len();
    let functions = context
        .analyses
        .ast_contexts
        .iter()
        .map(|c| count_listed_functions(&c.base))
        .sum();
    (files, functions)
}

/// Number of `- **Function**` entries a file section will emit.
fn count_listed_functions(file: &crate::services::context::FileContext) -> usize {
    file.items
        .iter()
        .filter(|item| matches!(item, crate::services::context::AstItem::Function { .. }))
        .count()
}

/// Add simple file section with annotated AST
fn add_simple_file_section(
    builder: &mut MarkdownBuilder,
    file: &crate::services::context::FileContext,
    analyses: &crate::services::deep_context::AnalysisResults,
) {
    // File header
    builder.content.push_str(&format!("### {}\n\n", file.path));

    // File-level metrics from analyses (BUG-007 FIX: shared path matching)
    let file_metrics = find_file_metrics(file, analyses);

    // **Functions** counts the entries listed immediately below it, not the
    // complexity report's own tally for the file: the two disagree (the report
    // also counts methods and nested items this listing does not emit), and
    // summing the old per-file line gave a third project-wide total that
    // matched neither the header nor the body.
    let function_count = count_listed_functions(file);

    if let Some(file_metrics) = file_metrics {
        builder.content.push_str(&format!(
            "**File Complexity**: {} | **Functions**: {}\n\n",
            file_metrics.total_complexity.cyclomatic, function_count
        ));
    } else {
        // BUG-007 FIX: Fallback - complexity report missing for this file
        if function_count > 0 || !file.items.is_empty() {
            builder.content.push_str(&format!(
                "**File Complexity**: N/A | **Functions**: {}\n\n",
                function_count
            ));
        }
    }

    // Add AST items with rich annotations
    for item in &file.items {
        builder.content.push_str(&format_ast_item_line(item, file, analyses));
    }

    builder.content.push('\n');
}

fn format_ast_item_line(
    item: &crate::services::context::AstItem,
    file: &crate::services::context::FileContext,
    analyses: &crate::services::deep_context::AnalysisResults,
) -> String {
    match item {
        crate::services::context::AstItem::Function { name, .. } => {
            let annotations = get_simple_function_annotations(name, file, analyses);
            format!("- **Function**: `{}`{}\n", name, annotations)
        }
        crate::services::context::AstItem::Struct { name, fields_count, .. } => {
            format!("- **Struct**: `{}` [fields: {}]\n", name, fields_count)
        }
        crate::services::context::AstItem::Trait { name, .. } => {
            format!("- **Trait**: `{}`\n", name)
        }
        crate::services::context::AstItem::Enum { name, variants_count, .. } => {
            format!("- **Enum**: `{}` [variants: {}]\n", name, variants_count)
        }
        crate::services::context::AstItem::Impl { type_name, trait_name, .. } => {
            if let Some(trait_name) = trait_name {
                format!("- **Impl**: `{}` for `{}`\n", trait_name, type_name)
            } else {
                format!("- **Impl**: `{}`\n", type_name)
            }
        }
        _ => String::new(),
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod header_body_agreement_tests {
    use super::*;
    use crate::services::complexity::{ComplexityReport, ComplexitySummary};
    use crate::services::context::{AstItem, FileContext};
    use crate::services::deep_context::{DeepContext, DefectAnnotations, EnhancedFileContext};

    fn file_with_functions(path: &str, names: &[&str]) -> EnhancedFileContext {
        EnhancedFileContext {
            base: FileContext {
                path: path.to_string(),
                language: "rust".to_string(),
                items: names
                    .iter()
                    .map(|n| AstItem::Function {
                        name: (*n).to_string(),
                        visibility: "pub".to_string(),
                        is_async: false,
                        line: 1,
                    })
                    .collect(),
                complexity_metrics: None,
            },
            complexity_metrics: None,
            churn_metrics: None,
            defects: DefectAnnotations {
                dead_code: None,
                technical_debt: vec![],
                complexity_violations: vec![],
                tdg_score: None,
            },
            symbol_id: path.to_string(),
        }
    }

    /// The `Total Files` / `Total Functions` header must count what the document
    /// below it actually contains. It used to be read from the complexity report
    /// while the body was rendered from `ast_contexts`, so one document carried
    /// three mutually different function totals.
    #[test]
    fn header_totals_match_the_body_they_head() {
        let mut context = DeepContext::default();
        context.analyses.ast_contexts = vec![
            file_with_functions("src/a.rs", &["one", "two"]),
            file_with_functions("src/b.rs", &["three"]),
        ];
        // A complexity report that disagrees with the body, as the real one does.
        context.analyses.complexity_report = Some(ComplexityReport {
            summary: ComplexitySummary {
                total_files: 7,
                total_functions: 999,
                ..ComplexitySummary::default()
            },
            violations: vec![],
            hotspots: vec![],
            files: vec![],
        });

        let md =
            generate_markdown_context("rust", std::path::Path::new("/repo"), &context).unwrap();

        assert!(
            md.contains("- **Total Files**: 2\n"),
            "header must count the emitted file sections:\n{md}"
        );
        assert!(
            md.contains("- **Total Functions**: 3\n"),
            "header must count the emitted function entries:\n{md}"
        );
        assert!(
            !md.contains("999"),
            "header must not report a count from a different analysis pass:\n{md}"
        );

        let sections = md.matches("\n### ").count();
        let functions = md.matches("- **Function**").count();
        assert_eq!(sections, 2);
        assert_eq!(functions, 3);
    }

    /// The same rule for the JSON surface. #915 fixed the markdown header and
    /// left this one reading `complexity_report`, so `--format json` shipped a
    /// 19-entry `files` array under `total_files: 12`.
    #[test]
    fn json_header_totals_match_the_body_they_head() {
        let mut context = DeepContext::default();
        context.analyses.ast_contexts = vec![
            file_with_functions("src/a.rs", &["one", "two"]),
            file_with_functions("src/b.rs", &["three"]),
        ];
        context.analyses.complexity_report = Some(ComplexityReport {
            summary: ComplexitySummary {
                total_files: 7,
                total_functions: 999,
                ..ComplexitySummary::default()
            },
            violations: vec![],
            hotspots: vec![],
            files: vec![],
        });

        let json = generate_json_context("rust", std::path::Path::new("/repo"), &context).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("parseable");

        assert_eq!(parsed["project"]["total_files"], 2, "{json}");
        assert_eq!(parsed["project"]["total_functions"], 3, "{json}");
        assert_eq!(
            parsed["files"].as_array().expect("files array").len(),
            2,
            "header must count the array it heads:\n{json}"
        );
    }

    /// Two formats of one command, on one project, in one run.
    #[test]
    fn markdown_and_json_headers_agree() {
        let mut context = DeepContext::default();
        context.analyses.ast_contexts = vec![
            file_with_functions("src/a.rs", &["one", "two"]),
            file_with_functions("src/b.rs", &["three"]),
            file_with_functions("src/c.rs", &["four", "five", "six"]),
        ];
        context.analyses.complexity_report = Some(ComplexityReport {
            summary: ComplexitySummary {
                total_files: 7,
                total_functions: 999,
                ..ComplexitySummary::default()
            },
            violations: vec![],
            hotspots: vec![],
            files: vec![],
        });
        let path = std::path::Path::new("/repo");

        let md = generate_markdown_context("rust", path, &context).unwrap();
        let json = generate_json_context("rust", path, &context).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("parseable");

        assert!(md.contains("- **Total Files**: 3\n"), "{md}");
        assert!(md.contains("- **Total Functions**: 6\n"), "{md}");
        assert_eq!(parsed["project"]["total_files"], 3, "{json}");
        assert_eq!(parsed["project"]["total_functions"], 6, "{json}");
    }
}
