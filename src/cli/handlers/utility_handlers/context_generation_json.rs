/// Generate JSON structured output (#216)
fn generate_json_context(
    toolchain: &str,
    project_path: &Path,
    context: &crate::services::deep_context::DeepContext,
) -> Result<String> {
    // Same rule as the markdown header: a header must count the document it
    // heads. `files` below is built by iterating `analyses.ast_contexts`, but
    // these totals were read from `complexity_report` — a separate pass with a
    // different file set and a wider notion of "function" — so a 19-entry
    // `files` array shipped under `total_files: 12`. #915 moved the markdown
    // header onto `count_emitted_body` for exactly this reason and left this
    // surface behind, which also made the two formats of one run disagree.
    let (total_files, total_functions) = count_emitted_body(context);

    let project = ContextJsonProject {
        language: toolchain.to_string(),
        path: project_path.display().to_string(),
        total_files,
        total_functions,
        overall_health: context
            .quality_scorecard
            .overall_health
            .map(|h| h.clamp(0.0, 100.0)),
        maintainability_index: context.quality_scorecard.maintainability_index,
    };

    let files: Vec<ContextJsonFile> = context
        .analyses
        .ast_contexts
        .iter()
        .map(|ec| build_json_file(&ec.base, &context.analyses))
        .collect();

    let output = ContextJsonOutput {
        version: "1.0",
        project,
        files,
    };

    serde_json::to_string_pretty(&output).map_err(|e| anyhow::anyhow!("JSON serialize: {e}"))
}

/// Build a single JSON file entry from AST context
fn build_json_file(
    file: &crate::services::context::FileContext,
    analyses: &crate::services::deep_context::AnalysisResults,
) -> ContextJsonFile {
    let file_metrics = find_file_metrics(file, analyses);

    let items: Vec<ContextJsonItem> = file
        .items
        .iter()
        .filter_map(|item| build_json_item(item, file_metrics))
        .collect();

    ContextJsonFile {
        path: file.path.clone(),
        items,
    }
}

/// Map an AstItem to a ContextJsonItem, enriching functions with complexity data
fn build_json_item(
    item: &crate::services::context::AstItem,
    file_metrics: Option<&crate::services::complexity::FileComplexityMetrics>,
) -> Option<ContextJsonItem> {
    use crate::services::context::AstItem;
    match item {
        AstItem::Function { name, line, .. } => {
            let (complexity, cognitive) = file_metrics
                .and_then(|fm| fm.functions.iter().find(|f| &f.name == name))
                .map(|fc| {
                    (
                        Some(fc.metrics.cyclomatic as u32),
                        Some(fc.metrics.cognitive as u32),
                    )
                })
                .unwrap_or((None, None));
            Some(ContextJsonItem {
                name: name.clone(),
                item_type: "function".to_string(),
                line: *line,
                complexity,
                cognitive_complexity: cognitive,
                fields_count: None,
                variants_count: None,
            })
        }
        AstItem::Struct {
            name,
            fields_count,
            line,
            ..
        } => Some(ContextJsonItem {
            name: name.clone(),
            item_type: "struct".to_string(),
            line: *line,
            complexity: None,
            cognitive_complexity: None,
            fields_count: Some(*fields_count),
            variants_count: None,
        }),
        AstItem::Enum {
            name,
            variants_count,
            line,
            ..
        } => Some(ContextJsonItem {
            name: name.clone(),
            item_type: "enum".to_string(),
            line: *line,
            complexity: None,
            cognitive_complexity: None,
            fields_count: None,
            variants_count: Some(*variants_count),
        }),
        AstItem::Trait { name, line, .. } => Some(ContextJsonItem {
            name: name.clone(),
            item_type: "trait".to_string(),
            line: *line,
            complexity: None,
            cognitive_complexity: None,
            fields_count: None,
            variants_count: None,
        }),
        AstItem::Impl {
            type_name,
            trait_name,
            line,
        } => {
            let name = if let Some(tr) = trait_name {
                format!("{tr} for {type_name}")
            } else {
                type_name.clone()
            };
            Some(ContextJsonItem {
                name,
                item_type: "impl".to_string(),
                line: *line,
                complexity: None,
                cognitive_complexity: None,
                fields_count: None,
                variants_count: None,
            })
        }
        _ => None,
    }
}
