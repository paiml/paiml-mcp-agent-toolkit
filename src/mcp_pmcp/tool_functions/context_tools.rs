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

    // Issue #1090. A walk that found nothing produced
    // `{"status":"completed","context":{"files":[],"total_files":0}}` — absence
    // rendered as success, over the very directory `pmat analyze complexity`
    // refuses with exit 5. Same sentence, same exit code; see
    // `ensure_mcp_population_was_analyzed`.
    ensure_mcp_population_was_analyzed("source files", "context", paths, files.len())?;
    let discovered = files.len();

    // `max_depth` and `include_dependencies` were declared as `_max_depth` /
    // `_include_dependencies` and never read, so the two documented knobs did
    // nothing at all: max_depth 1, 2 and 99 over a four-level tree returned
    // byte-identical 1061-file responses, and `dependencies` was always [].
    // A parameter the schema advertises must either act or not be advertised.
    if let Some(depth) = max_depth {
        files.retain(|file| within_max_depth(paths, file, depth));
        // A depth that excluded EVERY file is the caller narrowing the request,
        // not an empty tree, so it gets its own sentence: "no source files were
        // found under …" would be false here and would send the caller to look
        // at the wrong thing. It is still a refusal — nothing was measured.
        if files.is_empty() {
            return Err(crate::cli_exit::analysis_error(anyhow::anyhow!(
                "max_depth {depth} excluded all {discovered} source file(s) found under {}, \
                 so no context measurement was taken. This is not a clean result.",
                walked_roots(paths)
            )));
        }
    }

    let mut all_files = Vec::new();
    let mut all_dependencies: Vec<String> = Vec::new();
    // The same disclosure `analyze_complexity` publishes, for the same reason:
    // `Err(_) => continue` below took the file out of `total_files` and said so
    // nowhere, so a tree whose files all failed to parse and a tree with
    // nothing in it produced the identical `total_files: 0` under
    // `"status": "completed"`. One bucket, because every candidate here was
    // admitted by this walk's own extension list — a failure is a supported
    // file that yielded no measurement.
    let mut not_analyzed = UnmeasuredFiles::with_buckets(&["supported_but_unmeasured"]);

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
            // NAMED, not skipped — see `UnmeasuredFiles`.
            Err(error) => not_analyzed.record("supported_but_unmeasured", path, &error),
        }
    }

    // A population that was found and that the parser then refused in full is
    // not a clean result either.
    ensure_some_candidate_was_measured(
        "context",
        paths,
        files.len(),
        all_files.len(),
        not_analyzed.first_failure(),
    )?;

    Ok(json!({
        "status": "completed",
        "message": "Context generation completed",
        "context": {
            "files": all_files,
            "dependencies": all_dependencies,
            "total_files": all_files.len(),
            // Issue #1090 / #1058: the denominator, under the spellings
            // `analyze complexity --format json` and this module's other
            // analysis tools already use, so one consumer parser covers every
            // surface. `total_files` above is retained because clients read it;
            // it is `files_analyzed` by another name and always equal to it.
            "files_analyzed": all_files.len(),
            "files_discovered": files.len(),
            "files_not_analyzed": not_analyzed.to_json(),
            // What `max_depth` removed before the parser ever saw it — the
            // caller's own narrowing, reported separately from a failure. It is
            // `null` when no depth was supplied, because a 0 there would claim
            // a filter ran and excluded nothing.
            "files_excluded_by_max_depth": max_depth.map(|_| discovered - files.len()),
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

    // The two populations this payload mixes, named once so neither can be
    // read for the other: what the walk SAW, and what was actually parsed.
    let files_discovered = context.file_tree.total_files;
    let files_analyzed = context.analyses.ast_contexts.len();

    // Issue #1090. Every number this tool reports is summed over
    // `ast_contexts`, so an empty one makes `total_functions: 0` and
    // `total_imports: 0` the answer for a tree in which nothing could be
    // parsed — while the payload's other number, `total_files`, comes from the
    // FILE TREE, which counts a README. A numerator drawn from one population
    // beside a denominator drawn from another is precisely how "0 functions"
    // comes to read as a measurement rather than as a miss.
    ensure_mcp_population_was_analyzed(
        "source files pmat could parse",
        "context",
        std::slice::from_ref(project_path),
        files_analyzed,
    )?;

    // Build analyses based on requested types (or all if none specified)
    let requested_all = analysis_types.is_empty();
    let mut analyses = serde_json::Map::new();

    if requested_all || analysis_types.iter().any(|t| t == "structure") {
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
                "total_files": files_discovered,
                "total_functions": function_count,
                // Issue #1090 / #1058. `total_files` is the FILE TREE's count —
                // every file in scope, README included — while
                // `total_functions` is summed over the files that actually
                // parsed. Naming both populations stops the pair reading as
                // one, in the spellings the CLI and this module's other
                // analysis tools use.
                "files_discovered": files_discovered,
                "files_analyzed": files_analyzed,
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
                // The denominator for the count above, for the same reason it
                // is given for `total_functions`: this sum is over parsed
                // files, not over the tree.
                "files_analyzed": files_analyzed,
            }),
        );
    }

    Ok(json!({
        "status": "completed",
        "message": "Context analysis completed using DeepContextAnalyzer",
        "analyses": analyses,
        // "Analyzed N files" claimed the whole walk had been analysed, using
        // the file tree's count — the population the analyses above are NOT
        // computed over. Both numbers, so the sentence is true of whichever one
        // the reader takes it for.
        "context": format!("Analyzed {files_analyzed} of {files_discovered} file(s) walked"),
    }))
}

/// Detail levels the `scaffold_project` schema advertises for `level`.
const SUMMARY_LEVELS: [&str; 3] = ["brief", "normal", "detailed"];

/// Resolve the advertised `level` enum, rejecting anything outside it.
///
/// `level` was bound to `_level` and never read, so `brief` and `detailed`
/// returned byte-identical summaries and a value like `bogus-level-xyz` was
/// accepted in silence. A schema `enum` the tool does not enforce is a
/// documented contract the tool does not keep.
///
/// `pub` so the MCP handler can apply the SAME rule before dispatching and
/// report the refusal as `-32602 Invalid params`. It is a schema-enum
/// violation, and the handler's blanket `Error::internal` turned it into
/// `-32603 Internal error`, which is reserved for genuine server faults — a
/// client cannot tell "you sent a bad enum" from "the server crashed", and
/// retries the call. Enforcing it in one place keeps the two answers identical.
pub fn resolve_summary_level(level: Option<&str>) -> Result<&'static str> {
    let Some(requested) = level else {
        return Ok("normal");
    };
    SUMMARY_LEVELS
        .iter()
        .find(|known| known.eq_ignore_ascii_case(requested))
        .copied()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Unsupported level `{requested}`; expected one of: {}",
                SUMMARY_LEVELS.join(", ")
            )
        })
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn context_summary(paths: &[PathBuf], level: Option<&str>) -> Result<Value> {
    use std::collections::HashSet;
    use std::fs;

    if paths.is_empty() {
        return Err(anyhow::anyhow!("At least one path must be provided"));
    }

    let level = resolve_summary_level(level)?;

    let project_path = &paths[0];

    // Count files and lines
    let mut total_files = 0;
    let mut total_lines = 0;
    let mut languages = HashSet::new();
    // Per-language breakdown, only rendered above `brief`.
    let mut per_language: std::collections::BTreeMap<String, (usize, usize)> =
        std::collections::BTreeMap::new();

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
        per_language: &mut std::collections::BTreeMap<String, (usize, usize)>,
    ) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();

            if path.is_dir() {
                let dominated = path.file_name().and_then(|n| n.to_str()).is_some_and(is_excluded_dir);
                if !dominated {
                    traverse_dir(&path, total_files, total_lines, languages, per_language)?;
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
            let mut lines = 0;
            if let Ok(content) = fs::read_to_string(&path) {
                lines = content.lines().count();
                *total_lines += lines;
            }
            let bucket = per_language.entry(language.to_string()).or_insert((0, 0));
            bucket.0 += 1;
            bucket.1 += lines;
        }
        Ok(())
    }

    traverse_dir(
        project_path,
        &mut total_files,
        &mut total_lines,
        &mut languages,
        &mut per_language,
    )?;

    let mut languages_vec: Vec<String> = languages.into_iter().collect();
    languages_vec.sort();

    // Levels are nested (brief ⊂ normal ⊂ detailed) so a consumer that asked for
    // more never loses a key it already parsed.
    let mut summary = serde_json::Map::new();
    summary.insert("level".into(), json!(level));
    summary.insert("total_files".into(), json!(total_files));
    summary.insert("total_lines".into(), json!(total_lines));
    summary.insert("languages".into(), json!(languages_vec));
    if level != "brief" {
        let by_files: serde_json::Map<String, Value> = per_language
            .iter()
            .map(|(lang, (files, _))| (lang.clone(), json!(files)))
            .collect();
        summary.insert("files_by_language".into(), Value::Object(by_files));
    }
    if level == "detailed" {
        let by_lines: serde_json::Map<String, Value> = per_language
            .iter()
            .map(|(lang, (_, lines))| (lang.clone(), json!(lines)))
            .collect();
        summary.insert("lines_by_language".into(), Value::Object(by_lines));
    }

    Ok(json!({
        "status": "completed",
        "message": "Context summary generated from file system analysis",
        "summary": summary,
    }))
}

#[cfg(test)]
mod context_summary_level_tests {
    use super::*;

    #[test]
    fn out_of_enum_levels_are_rejected() {
        // The schema advertises an enum; accepting `bogus-level-xyz` in silence
        // is the defect.
        assert!(resolve_summary_level(Some("bogus-level-xyz")).is_err());
        assert_eq!(resolve_summary_level(None).expect("default"), "normal");
        for known in ["brief", "normal", "detailed", "DETAILED"] {
            assert!(resolve_summary_level(Some(known)).is_ok(), "{known}");
        }
    }

    #[tokio::test]
    async fn level_changes_the_summary() {
        // The reported defect: brief and detailed returned identical summaries.
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_path_buf();
        std::fs::write(root.join("a.rs"), "fn a() {}\n").expect("write");
        std::fs::write(root.join("b.py"), "def b():\n    pass\n").expect("write");
        let paths = vec![root];

        let brief = context_summary(&paths, Some("brief")).await.expect("brief");
        let normal = context_summary(&paths, None).await.expect("normal");
        let detailed = context_summary(&paths, Some("detailed"))
            .await
            .expect("detailed");

        assert_ne!(brief["summary"], detailed["summary"]);
        assert_eq!(brief["summary"]["level"], "brief");
        assert!(brief["summary"]["files_by_language"].is_null());
        assert!(normal["summary"]["files_by_language"].is_object());
        assert!(normal["summary"]["lines_by_language"].is_null());
        assert_eq!(detailed["summary"]["files_by_language"]["Rust"], 1);
        assert!(detailed["summary"]["lines_by_language"]["Python"]
            .as_u64()
            .expect("line count")
            >= 2);
    }

    #[tokio::test]
    async fn an_unknown_level_is_an_error_not_a_default() {
        let dir = tempfile::tempdir().expect("tempdir");
        let paths = vec![dir.path().to_path_buf()];
        let err = context_summary(&paths, Some("verbose"))
            .await
            .expect_err("out-of-enum level must be rejected");
        assert!(
            err.to_string().contains("Unsupported level"),
            "got: {err}"
        );
    }
}

#[cfg(test)]
mod unmeasured_context_population_tests {
    //! Issue #1090. `generate_context` and `analyze_context` reported
    //! `"status": "completed"` over populations they had measured nothing in,
    //! for directories `pmat analyze complexity` refuses with exit 5. The
    //! refusal is the CLI's own sentence; see
    //! `ensure_mcp_population_was_analyzed`.
    use super::*;

    /// The brief's fixture: a directory holding nothing the walk admits.
    /// `{"status":"completed","context":{"files":[],"total_files":0}}` was the
    /// old answer — absence rendered as success.
    #[tokio::test]
    async fn generate_context_refuses_a_population_of_zero() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("README.md"), "# nothing to parse\n")
            .expect("write README");

        let error = generate_context(&[dir.path().to_path_buf()], None, false)
            .await
            .expect_err("an empty population must be refused, not reported as completed");

        let message = format!("{error:#}");
        assert!(
            message.contains("no source files were found"),
            "the refusal must be the CLI's own sentence: {message}"
        );
        assert!(
            message.contains("no context measurement was taken")
                && message.contains("This is not a clean result"),
            "the refusal must name the measurement and the verdict: {message}"
        );
        assert_eq!(
            crate::cli_exit::code_for(&error),
            crate::cli_exit::ExitCode::AnalysisError,
            "the refusal must carry the CLI's own exit code: {message}"
        );
    }

    /// A `max_depth` that excludes every file found is still a run that
    /// measured nothing — but it is the CALLER's narrowing, so it must not be
    /// reported as an empty tree.
    #[tokio::test]
    async fn a_max_depth_that_excludes_everything_is_refused_by_its_own_name() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join("a/b")).expect("mkdir");
        std::fs::write(dir.path().join("a/b/deep.rs"), "pub fn deep() {}\n").expect("write");

        let error = generate_context(&[dir.path().to_path_buf()], Some(1), false)
            .await
            .expect_err("a depth filter that empties the population must be refused");

        let message = format!("{error:#}");
        assert!(
            message.contains("max_depth 1 excluded all 1 source file(s)"),
            "the refusal must name the filter and what it removed: {message}"
        );
        assert!(
            !message.contains("no source files were found"),
            "the walk DID find a file; blaming the walk sends the caller to the \
             wrong place: {message}"
        );
        assert!(
            message.contains("This is not a clean result"),
            "it is still a run that measured nothing: {message}"
        );
    }

    /// A measured run publishes its denominator under the spellings the CLI and
    /// the other MCP analysis tools use, so one consumer parser covers every
    /// surface (#1058). `total_files` used to be the only number in the
    /// payload.
    #[tokio::test]
    async fn a_measured_run_publishes_its_denominator() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("lib.rs"), "pub fn measurable() {}\n").expect("write");

        let json = generate_context(&[dir.path().to_path_buf()], None, false)
            .await
            .expect("one measurable file is a measurement");
        let context = &json["context"];

        assert_eq!(
            context["files_analyzed"], context["total_files"],
            "the retained legacy key must not drift from the canonical one: {json}"
        );
        assert_eq!(
            context["files_discovered"].as_u64(),
            Some(1),
            "the walk's count must be published: {json}"
        );
        assert_eq!(
            context["files_not_analyzed"]["total"].as_u64(),
            Some(0),
            "nothing was dropped here, and that is a claim the payload must \
             make rather than leave to be assumed: {json}"
        );
        assert!(
            context["files_not_analyzed"]["files"]
                .as_array()
                .is_some_and(|files| files.is_empty()),
            "the named-drop list must exist and be empty: {json}"
        );
        // No depth was supplied, so nothing was narrowed — and 0 there would
        // claim a filter ran and excluded nothing.
        assert!(
            context["files_excluded_by_max_depth"].is_null(),
            "an unrequested filter must report null, not 0: {json}"
        );
    }

    /// `analyze_context` sums `total_functions` and `total_imports` over
    /// `ast_contexts`, so an empty one made "0 functions" the answer for a tree
    /// nothing was parsed in — beside a `total_files` drawn from the FILE TREE,
    /// which counts a README.
    ///
    /// The assertion is on the REFUSAL and not on its wording: the deep-context
    /// pipeline may itself fail on such a tree, and either outcome is a
    /// refusal. What must not happen — what did happen — is a success payload
    /// of zeros.
    #[tokio::test]
    async fn analyze_context_refuses_a_population_it_parsed_nothing_in() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("README.md"), "# nothing to parse\n")
            .expect("write README");

        let refusal = match analyze_context(&[dir.path().to_path_buf()], &[]).await {
            Ok(json) => Err(format!(
                "analyze_context reported success over a tree it parsed nothing in: {json}"
            )),
            Err(error) => Ok(format!("{error:#}")),
        };
        refusal.expect("an unmeasured context run must not report completion");
    }
}
