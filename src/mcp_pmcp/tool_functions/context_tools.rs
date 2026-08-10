/// Is `file` within `max_depth` levels of one of the supplied roots?
///
/// Depth is counted the way the tool documents it: a file sitting directly in a
/// supplied directory is depth 1. A file that is under none of the roots (a
/// glob expansion, say) cannot be attributed a depth and is kept.
fn within_max_depth(roots: &[PathBuf], file: &Path, max_depth: usize) -> bool {
    let mut attributable = false;
    for root in roots {
        if root == file {
            return true;
        }
        if let Ok(relative) = file.strip_prefix(root) {
            attributable = true;
            if relative.components().count() <= max_depth {
                return true;
            }
        }
    }
    !attributable
}

/// Module paths this file imports, in source order and de-duplicated.
fn collect_file_dependencies(items: &[crate::services::context::AstItem]) -> Vec<String> {
    use crate::services::context::AstItem;

    let mut out = Vec::new();
    for item in items {
        let dep = match item {
            AstItem::Use { path, .. } => path.clone(),
            AstItem::Import { module, .. } => module.clone(),
            _ => continue,
        };
        if !dep.is_empty() && !out.contains(&dep) {
            out.push(dep);
        }
    }
    out
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn generate_context(
    paths: &[PathBuf],
    max_depth: Option<usize>,
    include_dependencies: bool,
) -> Result<Value> {
    use crate::services::deep_context::analyze_single_file;

    if paths.is_empty() {
        return Err(anyhow::anyhow!("At least one path must be provided"));
    }

    // R17-2: Walk directories. Previously only `path.is_file()` was analyzed,
    // yielding instant empty responses for directory inputs (D82).
    let mut files = expand_paths_to_source_files(paths);

    // `max_depth` and `include_dependencies` were declared as `_max_depth` /
    // `_include_dependencies` and never read, so the two documented knobs did
    // nothing at all: max_depth 1, 2 and 99 over a four-level tree returned
    // byte-identical 1061-file responses, and `dependencies` was always [].
    // A parameter the schema advertises must either act or not be advertised.
    if let Some(depth) = max_depth {
        files.retain(|file| within_max_depth(paths, file, depth));
    }

    let mut all_files = Vec::new();
    let mut all_dependencies: Vec<String> = Vec::new();

    for path in &files {
        // Analyze each file
        match analyze_single_file(path).await {
            Ok(file_context) => {
                if include_dependencies {
                    for dep in collect_file_dependencies(&file_context.items) {
                        if !all_dependencies.contains(&dep) {
                            all_dependencies.push(dep);
                        }
                    }
                }
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

#[cfg(test)]
mod generate_context_knob_tests {
    use super::*;

    #[test]
    fn depth_one_keeps_only_the_top_level() {
        let root = PathBuf::from("/p/src");
        let roots = vec![root.clone()];

        assert!(within_max_depth(&roots, Path::new("/p/src/lib.rs"), 1));
        assert!(!within_max_depth(&roots, Path::new("/p/src/a/b.rs"), 1));
        assert!(within_max_depth(&roots, Path::new("/p/src/a/b.rs"), 2));
        assert!(!within_max_depth(&roots, Path::new("/p/src/a/b/c.rs"), 2));
    }

    #[test]
    fn an_explicitly_named_file_is_always_within_depth() {
        let file = PathBuf::from("/p/src/lib.rs");
        assert!(within_max_depth(std::slice::from_ref(&file), &file, 1));
    }

    #[test]
    fn a_file_under_no_supplied_root_is_kept() {
        // Glob expansions are not attributable to a root; dropping them would
        // silently lose results.
        let roots = vec![PathBuf::from("/other")];
        assert!(within_max_depth(&roots, Path::new("/p/src/lib.rs"), 1));
    }

    #[tokio::test]
    async fn max_depth_actually_narrows_the_result_set() {
        // The reported defect: max_depth 1, 2 and 99 returned byte-identical
        // responses over a four-level tree.
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_path_buf();
        std::fs::write(root.join("top.rs"), "pub fn top() {}\n").expect("write");
        std::fs::create_dir_all(root.join("a/b")).expect("mkdir");
        std::fs::write(root.join("a/mid.rs"), "pub fn mid() {}\n").expect("write");
        std::fs::write(root.join("a/b/deep.rs"), "pub fn deep() {}\n").expect("write");

        let paths = vec![root];
        let count = |v: &Value| v["context"]["total_files"].as_u64().unwrap();

        let d1 = generate_context(&paths, Some(1), false).await.expect("d1");
        let d2 = generate_context(&paths, Some(2), false).await.expect("d2");
        let d99 = generate_context(&paths, Some(99), false).await.expect("d99");

        assert_eq!(count(&d1), 1, "depth 1 is the top level only");
        assert_eq!(count(&d2), 2);
        assert_eq!(count(&d99), 3);
    }

    #[tokio::test]
    async fn include_dependencies_populates_the_dependencies_list() {
        // `dependencies` was always [] whatever the flag said.
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_path_buf();
        std::fs::write(
            root.join("lib.rs"),
            "use std::collections::HashMap;\npub fn f(_: HashMap<u8, u8>) {}\n",
        )
        .expect("write");

        let paths = vec![root];
        let with = generate_context(&paths, None, true).await.expect("with");
        let without = generate_context(&paths, None, false)
            .await
            .expect("without");

        assert!(
            !with["context"]["dependencies"]
                .as_array()
                .expect("array")
                .is_empty(),
            "include_dependencies=true must report the file's imports"
        );
        assert!(without["context"]["dependencies"]
            .as_array()
            .expect("array")
            .is_empty());
    }
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn generate_deep_context(paths: &[PathBuf], _format: Option<&str>) -> Result<Value> {
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
                    // Same renderer as analyze_deep_context, so the two MCP
                    // tools cannot disagree about what was measured (GH #667).
                    "quality_scorecard": quality_scorecard_json(&context.quality_scorecard),
                    "file_count": context.file_tree.total_files,
                }
            }))
        }
        Err(e) => Err(anyhow::anyhow!("Deep context analysis failed: {e}")),
    }
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_context(paths: &[PathBuf], analysis_types: &[String]) -> Result<Value> {
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
