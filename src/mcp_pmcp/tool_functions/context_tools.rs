#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn generate_context(
    paths: &[PathBuf],
    _max_depth: Option<usize>,
    _include_dependencies: bool,
) -> Result<Value> {
    debug_assert!(!paths.is_empty(), "paths must not be empty");
    use crate::services::deep_context::analyze_single_file;

    if paths.is_empty() {
        return Err(anyhow::anyhow!("At least one path must be provided"));
    }

    let mut all_files = Vec::new();
    let all_dependencies: Vec<String> = Vec::new();

    for path in paths {
        if !path.exists() {
            continue;
        }

        // Analyze each file
        match analyze_single_file(path).await {
            Ok(file_context) => {
                all_files.push(json!({
                    "path": file_context.path,
                    "language": file_context.language,
                    "items_count": file_context.items.len(),
                    "items": file_context.items.iter().map(|item| match item {
                        crate::services::context::AstItem::Function { name, visibility, is_async, line } => json!({
                            "type": "function",
                            "name": name,
                            "visibility": visibility,
                            "is_async": is_async,
                            "line": line,
                        }),
                        crate::services::context::AstItem::Struct { name, visibility, fields_count, derives, line } => json!({
                            "type": "struct",
                            "name": name,
                            "visibility": visibility,
                            "fields_count": fields_count,
                            "derives": derives,
                            "line": line,
                        }),
                        _ => json!({"type": "other"}),
                    }).collect::<Vec<_>>(),
                }));
            }
            Err(_) => continue,
        }
    }

    Ok(json!({
        "status": "completed",
        "message": "Context generation completed",
        "context": {
            "files": all_files,
            "dependencies": all_dependencies,
            "total_files": all_files.len(),
        }
    }))
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn generate_deep_context(paths: &[PathBuf], _format: Option<&str>) -> Result<Value> {
    debug_assert!(!paths.is_empty(), "paths must not be empty");
    use crate::services::deep_context::{DeepContextAnalyzer, DeepContextConfig};

    if paths.is_empty() {
        return Err(anyhow::anyhow!("At least one path must be provided"));
    }

    // Create deep context analyzer with default config
    let config = DeepContextConfig::default();
    let analyzer = DeepContextAnalyzer::new(config);

    // For simplicity, analyze the first path (typically the project root)
    let project_path = &paths[0];

    match analyzer.analyze_project(project_path).await {
        Ok(context) => {
            // Return simplified JSON representation
            Ok(json!({
                "status": "completed",
                "message": "Deep context generation completed",
                "context": {
                    "metadata": {
                        "project_root": context.metadata.project_root,
                        "tool_version": context.metadata.tool_version,
                        "generated_at": context.metadata.generated_at.to_rfc3339(),
                        "analysis_duration_ms": context.metadata.analysis_duration.as_millis(),
                    },
                    "quality_scorecard": {
                        "overall_health": context.quality_scorecard.overall_health,
                        "complexity_score": context.quality_scorecard.complexity_score,
                        "maintainability_index": context.quality_scorecard.maintainability_index,
                        "modularity_score": context.quality_scorecard.modularity_score,
                        "technical_debt_hours": context.quality_scorecard.technical_debt_hours,
                    },
                    "file_count": context.file_tree.total_files,
                }
            }))
        }
        Err(e) => Err(anyhow::anyhow!("Deep context analysis failed: {e}")),
    }
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_context(paths: &[PathBuf], analysis_types: &[String]) -> Result<Value> {
    debug_assert!(!paths.is_empty(), "paths must not be empty");
    use crate::services::deep_context::{DeepContextAnalyzer, DeepContextConfig};

    if paths.is_empty() {
        return Err(anyhow::anyhow!("At least one path must be provided"));
    }

    let project_path = &paths[0];
    let config = DeepContextConfig::default();
    let analyzer = DeepContextAnalyzer::new(config);

    // Analyze project
    let context = analyzer.analyze_project(project_path).await?;

    // Build analyses based on requested types (or all if none specified)
    let requested_all = analysis_types.is_empty();
    let mut analyses = serde_json::Map::new();

    if requested_all || analysis_types.iter().any(|t| t == "structure") {
        let file_count = context.file_tree.total_files;
        let function_count: usize = context
            .analyses
            .ast_contexts
            .iter()
            .map(|ast| {
                ast.base
                    .items
                    .iter()
                    .filter(|item| {
                        matches!(item, crate::services::context::AstItem::Function { .. })
                    })
                    .count()
            })
            .sum();
        analyses.insert(
            "structure".to_string(),
            json!({
                "total_files": file_count,
                "total_functions": function_count,
            }),
        );
    }

    if requested_all || analysis_types.iter().any(|t| t == "dependencies") {
        let import_count: usize = context
            .analyses
            .ast_contexts
            .iter()
            .map(|ast| {
                ast.base
                    .items
                    .iter()
                    .filter(|item| {
                        matches!(
                            item,
                            crate::services::context::AstItem::Use { .. }
                                | crate::services::context::AstItem::Import { .. }
                        )
                    })
                    .count()
            })
            .sum();
        analyses.insert(
            "dependencies".to_string(),
            json!({
                "total_imports": import_count,
            }),
        );
    }

    Ok(json!({
        "status": "completed",
        "message": "Context analysis completed using DeepContextAnalyzer",
        "analyses": analyses,
        "context": format!("Analyzed {} files", context.file_tree.total_files),
    }))
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn context_summary(paths: &[PathBuf], _level: Option<&str>) -> Result<Value> {
    debug_assert!(!paths.is_empty(), "paths must not be empty");
    use std::collections::HashSet;
    use std::fs;

    if paths.is_empty() {
        return Err(anyhow::anyhow!("At least one path must be provided"));
    }

    let project_path = &paths[0];

    // Count files and lines
    let mut total_files = 0;
    let mut total_lines = 0;
    let mut languages = HashSet::new();

    fn detect_lang(ext: &str) -> Option<&'static str> {
        debug_assert!(!ext.is_empty(), "ext must not be empty");
        match ext {
            "rs" => Some("Rust"),
            "py" => Some("Python"),
            "js" => Some("JavaScript"),
            "ts" => Some("TypeScript"),
            "java" => Some("Java"),
            "cpp" | "cc" | "cxx" | "cu" | "cuh" => Some("C++"),
            "c" | "h" => Some("C"),
            "go" => Some("Go"),
            "rb" => Some("Ruby"),
            "php" => Some("PHP"),
            "swift" => Some("Swift"),
            "kt" => Some("Kotlin"),
            "sh" => Some("Shell"),
            _ => None,
        }
    }

    fn is_excluded_dir(name: &str) -> bool {
        name.starts_with('.') || name == "target" || name == "node_modules"
    }

    fn traverse_dir(
        dir: &Path,
        total_files: &mut usize,
        total_lines: &mut usize,
        languages: &mut HashSet<String>,
    ) -> Result<()> {
        debug_assert!(dir.exists(), "dir must exist: {}", dir.display());
        if !dir.is_dir() {
            return Ok(());
        }
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();

            if path.is_dir() {
                let dominated = path.file_name().and_then(|n| n.to_str()).is_some_and(is_excluded_dir);
                if !dominated {
                    traverse_dir(&path, total_files, total_lines, languages)?;
                }
                continue;
            }

            let ext = match path.extension().and_then(|e| e.to_str()) {
                Some(e) => e,
                None => continue,
            };
            let Some(language) = detect_lang(ext) else { continue };

            languages.insert(language.to_string());
            *total_files += 1;
            if let Ok(content) = fs::read_to_string(&path) {
                *total_lines += content.lines().count();
            }
        }
        Ok(())
    }

    traverse_dir(
        project_path,
        &mut total_files,
        &mut total_lines,
        &mut languages,
    )?;

    let languages_vec: Vec<String> = languages.into_iter().collect();

    Ok(json!({
        "status": "completed",
        "message": "Context summary generated from file system analysis",
        "summary": {
            "total_files": total_files,
            "total_lines": total_lines,
            "languages": languages_vec,
        }
    }))
}
