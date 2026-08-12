// R21-4 (D98) / R22-2 (D102): Glob expansion + source-tree walking live in
// the shared `crate::services::path_glob` module so the parallel
// `src/handlers/tools/` dispatcher can use the same implementation.
use crate::services::path_glob::expand_paths_to_source_files;

// Re-export for the existing `coverage_tests` suite, which references
// `resolve_paths_with_globs` at module scope (R21-4 test fixture carried
// over from before the service extraction).
#[cfg(test)]
use crate::services::path_glob::resolve_paths_with_globs;

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_complexity(
    paths: &[PathBuf],
    top_files: Option<usize>,
    threshold: Option<u64>,
) -> Result<Value> {
    // `analyze_file_complexity_uncached` used to run the heuristic counter
    // while `pmat analyze complexity` ran the AST one, so this tool reported
    // cyclomatic 10 / cognitive 18 (plus a threshold violation) for the same
    // function the CLI scored 6 / 9 with no violation. Both surfaces now share
    // the analyzer; keep them on one entry point.
    use crate::services::complexity::analyze_file_complexity_uncached;

    // Validate input
    if paths.is_empty() {
        return Err(anyhow::anyhow!("At least one path must be provided"));
    }

    let threshold_value = threshold.unwrap_or(10);
    let files = expand_paths_to_source_files(paths);

    // Analyze all expanded files
    let mut all_functions = Vec::new();
    let mut total_files = 0;
    let mut total_complexity = 0u64;
    let mut violations = Vec::new();

    for path in &files {
        match analyze_file_complexity_uncached(path, None).await {
            Ok(metrics) => {
                total_files += 1;

                for func in &metrics.functions {
                    let cc = func.metrics.cyclomatic as u64;
                    total_complexity += cc;

                    if cc >= threshold_value {
                        violations.push(json!({
                            "file": metrics.path.clone(),
                            "function": func.name.clone(),
                            "complexity": cc,
                            "threshold": threshold_value,
                            "line_start": func.line_start,
                            "line_end": func.line_end,
                        }));
                    }

                    all_functions.push(json!({
                        "file": metrics.path.clone(),
                        "function": func.name.clone(),
                        "cyclomatic_complexity": func.metrics.cyclomatic,
                        "cognitive_complexity": func.metrics.cognitive,
                        "line_start": func.line_start,
                        "line_end": func.line_end,
                    }));
                }
            }
            Err(_) => continue, // Skip files that fail to analyze
        }
    }

    // Sort by complexity and apply top_files limit
    let mut sorted_functions = all_functions;
    if let Some(limit) = top_files {
        sorted_functions.sort_by(|a, b| {
            let a_cc = a["cyclomatic_complexity"].as_u64().unwrap_or(0);
            let b_cc = b["cyclomatic_complexity"].as_u64().unwrap_or(0);
            b_cc.cmp(&a_cc) // Descending order
        });
        sorted_functions.truncate(limit);
    }

    let average_complexity = if total_files > 0 {
        total_complexity / total_files as u64
    } else {
        0
    };

    Ok(json!({
        "status": "completed",
        "message": "Complexity analysis completed",
        "results": {
            "total_files": total_files,
            "total_complexity": total_complexity,
            "average_complexity": average_complexity,
            "violations": violations,
            "top_files": sorted_functions,
        }
    }))
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_satd(paths: &[PathBuf], _include_resolved: bool) -> Result<Value> {
    use crate::services::satd_detector::SATDDetector;

    // Validate input
    if paths.is_empty() {
        return Err(anyhow::anyhow!("At least one path must be provided"));
    }

    let detector = SATDDetector::new();
    let files = expand_paths_to_source_files(paths);

    let mut total_satd = 0;
    let mut file_results = Vec::new();

    for path in &files {
        match tokio::fs::read_to_string(path).await {
            Ok(content) => match detector.extract_from_content(&content, path) {
                Ok(debts) => {
                    // Filter out resolved debt markers (DONE, RESOLVED, FIXED) unless include_resolved
                    let debts: Vec<_> = if _include_resolved {
                        debts
                    } else {
                        debts
                            .into_iter()
                            .filter(|d| {
                                let upper = d.text.to_uppercase();
                                !upper.contains("DONE")
                                    && !upper.contains("RESOLVED")
                                    && !upper.contains("FIXED")
                            })
                            .collect()
                    };
                    let satd_count = debts.len();
                    total_satd += satd_count;

                    if satd_count > 0 {
                        file_results.push(json!({
                            "file": path.display().to_string(),
                            "satd_count": satd_count,
                            "debts": debts.iter().map(|debt| json!({
                                "line": debt.line,
                                "category": format!("{:?}", debt.category),
                                "severity": format!("{:?}", debt.severity),
                                "text": debt.text,
                            })).collect::<Vec<_>>(),
                        }));
                    }
                }
                Err(_) => continue,
            },
            Err(_) => continue,
        }
    }

    Ok(json!({
        "status": "completed",
        "message": "SATD analysis completed",
        "results": {
            "total_satd": total_satd,
            "files": file_results,
        }
    }))
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_dead_code(paths: &[PathBuf], include_tests: bool) -> Result<Value> {
    use crate::services::dead_code_multi_language::analyze_dead_code_multi_language;
    use std::collections::HashMap;

    // Validate input
    if paths.is_empty() {
        return Err(anyhow::anyhow!("At least one path must be provided"));
    }

    let mut total_dead_code = 0;
    let mut total_functions = 0;
    // R17-2: analyze_dead_code_multi_language() accepts dirs OR files; call it
    // once per provided path (dir aggregation handled by per-language strategies).
    // Group results by file rather than by path to keep JSON structure stable.
    let mut dead_by_file: HashMap<String, Vec<(String, usize)>> = HashMap::new();
    let mut languages: Vec<String> = Vec::new();

    for path in paths {
        if !path.exists() {
            continue;
        }
        match analyze_dead_code_multi_language(path) {
            Ok(result) => {
                // `include_tests` used to be `_include_tests` — an inert
                // parameter: passing include_tests:true returned a byte-identical
                // response. The CLI's default (false) keeps test code out of the
                // dead-code list, so honour the same default here instead of
                // advertising a flag that does nothing.
                let dead: Vec<_> = result
                    .dead_functions
                    .iter()
                    .filter(|func| include_tests || !is_test_path(&func.file))
                    .collect();

                total_dead_code += dead.len();
                total_functions += result.total_functions;
                if !languages.contains(&result.language) {
                    languages.push(result.language.clone());
                }
                for func in dead {
                    dead_by_file
                        .entry(func.file.clone())
                        .or_default()
                        .push((func.name.clone(), func.line));
                }
            }
            Err(_) => continue,
        }
    }

    let file_results: Vec<Value> = dead_by_file
        .into_iter()
        .map(|(file, funcs)| {
            json!({
                "file": file,
                "dead_code_count": funcs.len(),
                "dead_functions": funcs.iter().map(|(name, line)| json!({
                    "name": name,
                    "line": line,
                })).collect::<Vec<_>>(),
            })
        })
        .collect();

    Ok(json!({
        "status": "completed",
        "message": "Dead code analysis completed",
        "results": {
            "total_dead_code": total_dead_code,
            "total_functions": total_functions,
            "languages": languages,
            "files": file_results,
            // This tool and `pmat analyze dead-code` disagreed (2 vs 0 dead
            // functions on the same path) with nothing in either payload saying
            // why: this is the language-agnostic reachability heuristic, which
            // calls an un-called item dead, while the CLI runs cargo's own
            // dead-code pass on Rust projects. Name the producer so a client can
            // reconcile the two numbers instead of guessing which is wrong.
            "analyzer": "multi-language-reachability",
            "analyzer_note": "Un-called items are reported dead. `pmat analyze dead-code` \
                              uses the cargo/AST analyzer on Rust projects and may report fewer.",
            "include_tests": include_tests,
        }
    }))
}

/// Does this path look like test code? Used to honour `include_tests` for the
/// reachability analyzer, which has no test-awareness of its own.
fn is_test_path(file: &str) -> bool {
    let path = std::path::Path::new(file);
    let in_test_dir = path.components().any(|component| {
        matches!(
            component.as_os_str().to_str(),
            Some("tests") | Some("test") | Some("testing")
        )
    });
    let name = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default();

    in_test_dir || name.starts_with("test_") || name.ends_with("_test") || name.ends_with("_tests")
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_lint_hotspots(paths: &[PathBuf], top_files: Option<usize>) -> Result<Value> {
    use crate::tdg::analyzer_simple::TdgAnalyzer;

    if paths.is_empty() {
        return Err(anyhow::anyhow!("At least one path must be provided"));
    }

    let top_files_limit = top_files.unwrap_or(10);
    let analyzer = TdgAnalyzer::new()?;
    let project_path = &paths[0];

    // Analyze project with TDG
    let project_score = if project_path.is_dir() {
        analyzer.analyze_project(project_path)?
    } else {
        return Err(anyhow::anyhow!("Path must be a directory"));
    };

    // Sort files by score (lower score = worse quality = hotspot)
    let mut file_scores = project_score.files.clone();
    file_scores.sort_by(|a, b| {
        a.total
            .partial_cmp(&b.total)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Take top N hotspots (lowest scores)
    file_scores.truncate(top_files_limit);

    // Build hotspot entries
    let hotspots: Vec<Value> = file_scores
        .iter()
        .filter_map(|file_score| {
            file_score.file_path.as_ref().map(|path| {
                json!({
                    "file": path.display().to_string(),
                    "score": file_score.total,
                    "grade": file_score.grade.to_string(),
                    "violation_count": file_score.penalties_applied.len(),
                    "complexity": file_score.structural_complexity,
                    "satd_count": file_score.penalties_applied.iter()
                        .filter(|p| p.issue.to_lowercase().contains("satd") || p.issue.to_lowercase().contains("todo"))
                        .count(),
                    "total_penalty": file_score.penalties_applied.iter()
                        .map(|p| p.amount)
                        .sum::<f32>(),
                })
            })
        })
        .collect();

    Ok(json!({
        "status": "completed",
        "message": format!("Lint hotspot analysis completed ({} hotspots found)", hotspots.len()),
        "results": {
            "hotspots": hotspots,
            "total_files_analyzed": project_score.files.len(),
            "top_files_limit": top_files_limit,
        }
    }))
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_churn(
    paths: &[PathBuf],
    days: Option<u32>,
    top_files: Option<usize>,
) -> Result<Value> {
    use crate::services::git_analysis::GitAnalysisService;

    if paths.is_empty() {
        return Err(anyhow::anyhow!("At least one path must be provided"));
    }

    let days_value = days.unwrap_or(30);
    let top_files_value = top_files.unwrap_or(10);

    // Analyze churn for the first path (typically repository root)
    let repo_path = &paths[0];

    match GitAnalysisService::analyze_code_churn(repo_path, days_value) {
        Ok(mut analysis) => {
            // Apply top_files filtering
            analysis.files.sort_by(|a, b| {
                b.churn_score
                    .partial_cmp(&a.churn_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            analysis.files.truncate(top_files_value);

            // Transform to JSON
            Ok(json!({
                "status": "completed",
                "message": format!("Churn analysis completed for last {days_value} days"),
                "results": {
                    "period_days": analysis.period_days,
                    "total_commits": analysis.summary.total_commits,
                    "total_files_changed": analysis.summary.total_files_changed,
                    "files": analysis.files.iter().map(|f| json!({
                        "path": f.relative_path,
                        "commit_count": f.commit_count,
                        "unique_authors": f.unique_authors.len(),
                        "additions": f.additions,
                        "deletions": f.deletions,
                        "churn_score": f.churn_score,
                        "last_modified": f.last_modified.to_rfc3339(),
                    })).collect::<Vec<_>>(),
                    "hotspot_files": analysis.summary.hotspot_files.len(),
                }
            }))
        }
        Err(e) => Err(anyhow::anyhow!("Churn analysis failed: {e}")),
    }
}

/// The `dag_type` values `analyze_dag` accepts, in the spelling the tool's
/// `inputSchema` advertises.
pub const DAG_TYPES: [&str; 4] = [
    "call-graph",
    "import-graph",
    "inheritance",
    "full-dependency",
];

/// Resolve the `dag_type` argument, REJECTING anything not in [`DAG_TYPES`].
///
/// The match used to end in `_ => DagType::FullDependency`, so `"BOGUS"`, `""`
/// and `"12345"` all came back `status: "completed"` with
/// `results.dag_type: "FullDependency"` and no warning: a client that typo'd the
/// mode got a successful-looking result for a DIFFERENT analysis than the one it
/// asked for, and no way to tell. `generate_context` already rejects an
/// unsupported `format` and `scaffold_project` an unsupported `level`; silently
/// coercing an enum is the one behaviour a schema-declared `enum` promises will
/// not happen.
///
/// The underscore spellings stay accepted — they were accepted before, and
/// removing them would break callers for no gain.
pub fn parse_dag_type(dag_type: Option<&str>) -> Result<crate::services::deep_context::DagType> {
    use crate::services::deep_context::DagType;
    match dag_type.unwrap_or("full-dependency") {
        "call-graph" | "call_graph" => Ok(DagType::CallGraph),
        "import-graph" | "import_graph" => Ok(DagType::ImportGraph),
        "inheritance" => Ok(DagType::Inheritance),
        "full-dependency" | "full_dependency" => Ok(DagType::FullDependency),
        other => Err(anyhow::anyhow!(
            "Unsupported dag_type: {other:?} (expected one of {})",
            DAG_TYPES.join(", ")
        )),
    }
}

// --- DAG analysis (R17-1) ---
//
// Dispatches to `services::deep_context::analysis_functions::analyze_dag`,
// which builds a ProjectContext + DagBuilder graph. This is the analysis
// powering `pmat analyze dag`.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_dag(paths: &[PathBuf], dag_type: Option<String>) -> Result<Value> {
    use crate::services::deep_context::analysis_functions::analyze_dag as svc_analyze_dag;

    if paths.is_empty() {
        return Err(anyhow::anyhow!("At least one path must be provided"));
    }

    let dag_type_parsed = parse_dag_type(dag_type.as_deref())?;
    let dag_type_label = format!("{:?}", dag_type_parsed);

    let project_path = &paths[0];
    let graph = svc_analyze_dag(project_path, dag_type_parsed)
        .await
        .map_err(|e| anyhow::anyhow!("DAG analysis failed: {e}"))?;

    let node_count = graph.nodes.len();
    let edge_count = graph.edges.len();

    // Emit a compact summary plus the raw graph. Callers that want full
    // mermaid output can use the CLI's `pmat analyze dag` path.
    let top_nodes: Vec<Value> = graph
        .nodes
        .values()
        .take(25)
        .map(|n| {
            json!({
                "id": n.id,
                "label": n.label,
                "node_type": n.node_type,
                "file_path": n.file_path,
                "line_number": n.line_number,
                "complexity": n.complexity,
            })
        })
        .collect();

    Ok(json!({
        "status": "completed",
        "message": format!("DAG analysis completed ({node_count} nodes, {edge_count} edges)"),
        "results": {
            "dag_type": dag_type_label,
            "node_count": node_count,
            "edge_count": edge_count,
            "top_nodes": top_nodes,
        }
    }))
}

// --- Big-O analysis (R17-1) ---
//
// Dispatches to `services::deep_context::analysis_functions::analyze_big_o`,
// which classifies function time complexity via BigOAnalyzer. This is the
// analysis powering `pmat analyze big-o`.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_big_o(paths: &[PathBuf], top_files: Option<usize>) -> Result<Value> {
    use crate::services::deep_context::analysis_functions::analyze_big_o as svc_analyze_big_o;

    if paths.is_empty() {
        return Err(anyhow::anyhow!("At least one path must be provided"));
    }

    let project_path = &paths[0];
    let report = svc_analyze_big_o(project_path)
        .await
        .map_err(|e| anyhow::anyhow!("Big-O analysis failed: {e}"))?;

    let top_limit = top_files.unwrap_or(25);
    let high_complexity: Vec<Value> = report
        .high_complexity_functions
        .iter()
        .take(top_limit)
        .map(|f| {
            json!({
                "file_path": f.file_path,
                "function_name": f.function_name,
                "line_number": f.line_number,
                // `time_complexity` / `space_complexity` used to be the raw
                // `#[repr(C)]` ComplexityBound, so the payload carried the
                // struct's alignment padding and its packed flag byte
                // (`{"class":"Quadratic","coefficient":1,"input_var":"N",
                // "confidence":75,"flags":2,"_padding":[0,0]}`) to a reader
                // that cannot tell layout from measurement. The CLI's JSON
                // emits the notation and the confidence; emit the same.
                "time_complexity": f.time_complexity.notation(),
                "time_complexity_confidence": f.time_complexity.confidence,
                "space_complexity": f.space_complexity.notation(),
                "space_complexity_confidence": f.space_complexity.confidence,
                "confidence": f.confidence,
            })
        })
        .collect();

    // A LIST THAT IS SECRETLY A CAP: `top_files` truncated this silently, so
    // a caller asking for 2 saw 2 and had no way to learn there were 8. The
    // CLI names both numbers (`high_complexity_count` / `_found` /
    // `_truncated`); keep the two surfaces telling the same story.
    let listed = high_complexity.len();
    let dist = &report.complexity_distribution;
    let found = (dist.quadratic + dist.cubic + dist.exponential).max(report.high_complexity_functions.len());

    Ok(json!({
        "status": "completed",
        "message": format!("Big-O analysis completed ({} functions analyzed)", report.analyzed_functions),
        "results": {
            "analyzed_functions": report.analyzed_functions,
            "complexity_distribution": report.complexity_distribution,
            "high_complexity_functions": high_complexity,
            "high_complexity_count": listed,
            "high_complexity_found": found,
            "high_complexity_truncated": listed < found,
            "recommendations": report.recommendations,
        }
    }))
}

// --- Deep context analysis (R17-1) ---
//
// Dispatches to `services::deep_context::DeepContextAnalyzer`, which runs the
// full multi-phase deep context pipeline. This is the analysis powering
// `pmat context` / `pmat analyze deep-context`.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_deep_context(
    paths: &[PathBuf],
    include_patterns: Option<Vec<String>>,
) -> Result<Value> {
    use crate::services::deep_context::{DeepContextAnalyzer, DeepContextConfig};

    if paths.is_empty() {
        return Err(anyhow::anyhow!("At least one path must be provided"));
    }

    reject_unsupported_include_patterns(include_patterns.as_deref())?;

    let project_path = &paths[0];
    let config = DeepContextConfig::default();
    let analyzer = DeepContextAnalyzer::new(config);
    let context = analyzer
        .analyze_project(&project_path.to_path_buf())
        .await
        .map_err(|e| anyhow::anyhow!("Deep context analysis failed: {e}"))?;

    Ok(json!({
        "status": "completed",
        "message": format!("Deep context analysis completed ({} files)", context.file_tree.total_files),
        "results": {
            "metadata": {
                "project_root": context.metadata.project_root,
                "tool_version": context.metadata.tool_version,
                "generated_at": context.metadata.generated_at.to_rfc3339(),
                "analysis_duration_ms": context.metadata.analysis_duration.as_millis(),
            },
            "quality_scorecard": quality_scorecard_json(&context.quality_scorecard),
            "file_count": context.file_tree.total_files,
            "ast_contexts": context.analyses.ast_contexts.len(),
        }
    }))
}

/// Refuse `include_patterns` rather than accept it and ignore it.
///
/// `analyze_deep_context {"paths":[dir]}` and
/// `analyze_deep_context {"paths":[dir],"include_patterns":["*.py"]}` returned
/// the SAME `file_count: 3` over a directory holding `a.go app.ts main.py`: the
/// argument parsed, was bound to `_include_patterns`, and was thrown away. The
/// tool's own schema described it as "accepted but not yet applied as a filter",
/// which is a defect annotated and shipped rather than fixed.
///
/// It cannot be wired up from here. `DeepContextConfig::include_patterns` exists
/// but is READ BY NOBODY — `grep -rn include_patterns src/services/deep_context/`
/// finds the declaration, the `Default` initialiser and nothing else, so setting
/// it would be a second no-op stacked on the first. Only `exclude_patterns` is
/// honoured, and only by the file-tree walk (`analyzer_core/file_tree.rs`), not
/// by the analysis phase that produces `ast_contexts` and the scorecard. Post-
/// filtering the results here would be worse than useless: `quality_scorecard`
/// is computed inside the pipeline over the WHOLE tree, so a filtered
/// `file_count` beside an unfiltered scorecard is a payload that reads as
/// measured and is not.
///
/// So the parameter is refused, loudly and by name, with the operation that
/// actually narrows the analysis. Wiring it for real needs
/// `src/services/deep_context/analyzer_core/` to honour the config field — the
/// CLI's `pmat analyze deep-context --include` sets that same dead field
/// (`src/cli/handlers/advanced_analysis_handlers.rs:272`) and is equally inert.
fn reject_unsupported_include_patterns(patterns: Option<&[String]>) -> Result<()> {
    let Some(patterns) = patterns.filter(|p| !p.is_empty()) else {
        return Ok(());
    };
    Err(anyhow::anyhow!(
        "include_patterns is not supported by analyze_deep_context: the deep-context \
         pipeline has no file filter, so applying {patterns:?} would change nothing about \
         the analysis. Rejecting instead of silently ignoring it. Narrow the analysis by \
         passing the subdirectory you want in `paths`, or use analyze_complexity / \
         analyze_satd, which do filter."
    ))
}

/// Serialise a quality scorecard for MCP consumers.
///
/// Unmeasured fields are `null` AND listed by name in `not_measured`, because
/// this payload is read by models that cannot tell a measurement from a
/// placeholder. Every one of these was a constant: six wildly different code
/// bases — a 5-file toy fixture and pmat's own 3891-file tree among them — all
/// returned maintainability_index 70.0, modularity_score 85.0 and
/// technical_debt_hours 40.0, while file_count and ast_contexts varied
/// correctly, which is exactly what made the scorecard read as measured
/// (GH #667, same root cause as #643 on the CLI side).
///
/// `test_coverage` is included here; the MCP payload used to omit it entirely
/// while the CLI reported it.
fn quality_scorecard_json(
    scorecard: &crate::services::deep_context::QualityScorecard,
) -> serde_json::Value {
    let fields = [
        ("overall_health", scorecard.overall_health),
        ("complexity_score", scorecard.complexity_score),
        ("maintainability_index", scorecard.maintainability_index),
        ("modularity_score", scorecard.modularity_score),
        ("test_coverage", scorecard.test_coverage),
        ("technical_debt_hours", scorecard.technical_debt_hours),
    ];

    let mut object = serde_json::Map::new();
    let mut not_measured = Vec::new();
    for (name, value) in fields {
        if value.is_none() {
            not_measured.push(serde_json::Value::from(name));
        }
        object.insert(name.to_string(), json!(value));
    }
    object.insert(
        "not_measured".to_string(),
        serde_json::Value::Array(not_measured),
    );
    serde_json::Value::Object(object)
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_coupling(paths: &[PathBuf], threshold: Option<f64>) -> Result<Value> {
    use crate::services::deep_context::{DeepContextAnalyzer, DeepContextConfig};
    use std::collections::HashMap;

    if paths.is_empty() {
        return Err(anyhow::anyhow!("At least one path must be provided"));
    }

    let project_path = &paths[0];
    let threshold_value = threshold.unwrap_or(0.5);

    // Use deep context analyzer to get AST contexts
    let config = DeepContextConfig::default();
    let analyzer = DeepContextAnalyzer::new(config);
    let context = analyzer.analyze_project(project_path).await?;

    // Analyze coupling from AST contexts
    let mut file_metrics: HashMap<String, (usize, usize, f64)> = HashMap::new();

    // Build import map for afferent coupling calculation
    let mut all_imports: HashMap<String, Vec<String>> = HashMap::new();
    for ast_context in &context.analyses.ast_contexts {
        let file_path = ast_context.base.path.clone();
        let imports: Vec<String> = ast_context
            .base
            .items
            .iter()
            .filter_map(|item| match item {
                crate::services::context::AstItem::Use { path, .. } => Some(path.clone()),
                crate::services::context::AstItem::Import { module, .. } => Some(module.clone()),
                _ => None,
            })
            .collect();
        all_imports.insert(file_path, imports);
    }

    // Calculate metrics
    for (file, imports) in &all_imports {
        let efferent = imports.len();
        let afferent = all_imports
            .values()
            .filter(|deps| deps.iter().any(|d| d.contains(file) || file.contains(d)))
            .count();
        let total = afferent + efferent;
        let instability = if total > 0 {
            efferent as f64 / total as f64
        } else {
            0.0
        };

        file_metrics.insert(file.clone(), (afferent, efferent, instability));
    }

    // Filter by threshold and build coupling entries
    let couplings: Vec<Value> = file_metrics
        .iter()
        .filter(|(_, (_, _, instability))| *instability >= threshold_value)
        .map(|(file, (afferent, efferent, instability))| {
            json!({
                "file": file,
                "afferent_coupling": afferent,
                "efferent_coupling": efferent,
                "instability": instability,
                "strength": afferent + efferent,
            })
        })
        .collect();

    // Calculate project-level metrics
    let avg_afferent = if !file_metrics.is_empty() {
        file_metrics.values().map(|(a, _, _)| *a).sum::<usize>() as f64 / file_metrics.len() as f64
    } else {
        0.0
    };
    let avg_efferent = if !file_metrics.is_empty() {
        file_metrics.values().map(|(_, e, _)| *e).sum::<usize>() as f64 / file_metrics.len() as f64
    } else {
        0.0
    };
    let max_afferent = file_metrics.values().map(|(a, _, _)| *a).max().unwrap_or(0);
    let max_efferent = file_metrics.values().map(|(_, e, _)| *e).max().unwrap_or(0);

    Ok(json!({
        "status": "completed",
        "message": format!("Coupling analysis completed ({} files analyzed)", file_metrics.len()),
        "results": {
            "couplings": couplings,
            "total_files": file_metrics.len(),
            "threshold": threshold_value,
            "project_metrics": {
                "avg_afferent": avg_afferent,
                "avg_efferent": avg_efferent,
                "max_afferent": max_afferent,
                "max_efferent": max_efferent,
            }
        }
    }))
}

#[cfg(test)]
mod big_o_payload_tests {
    //! The tool serialised the raw `#[repr(C)]` ComplexityBound and truncated
    //! its function list without saying so.
    use super::*;

    /// Three functions with a doubly-nested loop each: enough for `top_files`
    /// to bite.
    fn quadratic_fixture() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut src = String::new();
        for i in 0..3 {
            src.push_str(&format!(
                "pub fn pairs{i}(xs: &[usize]) -> usize {{\n    let mut acc = 0;\n    for a in xs {{\n        for b in xs {{\n            acc += a * b;\n        }}\n    }}\n    acc\n}}\n\n"
            ));
        }
        std::fs::write(dir.path().join("lib.rs"), src).expect("write lib.rs");
        dir
    }

    #[tokio::test]
    async fn test_big_o_payload_has_no_struct_layout_fields() {
        let dir = quadratic_fixture();
        let value = analyze_big_o(&[dir.path().to_path_buf()], Some(1))
            .await
            .expect("analysis");
        let payload = serde_json::to_string(&value).expect("serialise");

        assert!(
            !payload.contains("_padding"),
            "alignment filler leaked into the MCP payload: {payload}"
        );
        let functions = value["results"]["high_complexity_functions"]
            .as_array()
            .expect("array");
        assert!(
            !functions.is_empty(),
            "the fixture must produce a high-complexity function, or this asserts nothing"
        );
        for func in functions {
            // A notation string ("O(n²)"), not the packed bound struct.
            let time = func["time_complexity"].as_str().unwrap_or_else(|| {
                panic!("time_complexity must be a notation string: {func}")
            });
            assert!(time.starts_with("O("), "unexpected notation {time:?}");
            assert!(func["time_complexity_confidence"].is_number());
            assert!(func["space_complexity"].is_string());
        }
    }

    /// `top_files` capped the list silently: asking for 1 of 3 returned 1,
    /// with nothing in the payload naming the other two.
    #[tokio::test]
    async fn test_big_o_payload_discloses_truncation() {
        let dir = quadratic_fixture();
        let results = analyze_big_o(&[dir.path().to_path_buf()], Some(1))
            .await
            .expect("analysis");
        let results = &results["results"];

        let listed = results["high_complexity_functions"]
            .as_array()
            .expect("array")
            .len();
        let count = results["high_complexity_count"].as_u64().expect("count");
        let found = results["high_complexity_found"].as_u64().expect("found");

        assert_eq!(count as usize, listed, "count must be the listed length");
        // The fixture holds three quadratic functions and asks for one, so the
        // cap must be visible rather than inferred.
        assert_eq!(count, 1, "top_files=1 must list exactly one");
        assert!(
            found > count,
            "found ({found}) must exceed the listed count ({count}) for this fixture"
        );
        assert_eq!(
            results["high_complexity_truncated"],
            serde_json::Value::Bool(count < found),
            "the truncation flag must follow the two counts"
        );
    }
}

#[cfg(test)]
mod dead_code_include_tests_tests {
    //! `include_tests` used to be an unused parameter (`_include_tests`).
    use super::*;

    #[test]
    fn test_is_test_path_recognises_the_usual_layouts() {
        assert!(is_test_path("tests/integration.rs"));
        assert!(is_test_path("crate/tests/helpers/mod.rs"));
        assert!(is_test_path("src/foo_tests.rs"));
        assert!(is_test_path("src/test_foo.py"));
        assert!(!is_test_path("src/lib.rs"));
        assert!(!is_test_path("src/latest.rs"));
    }

    /// Passing include_tests:true returned a byte-identical response, because
    /// the tool signature was `analyze_dead_code(paths, _include_tests: bool)`.
    #[tokio::test]
    async fn test_include_tests_changes_the_result() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join("src")).expect("mkdir src");
        std::fs::create_dir_all(dir.path().join("tests")).expect("mkdir tests");
        std::fs::write(
            dir.path().join("src/lib.rs"),
            "pub fn used() {}\npub fn never_called() {}\nfn main() { used(); }\n",
        )
        .expect("write lib.rs");
        std::fs::write(
            dir.path().join("tests/helper.rs"),
            "fn only_in_tests() {}\n",
        )
        .expect("write tests/helper.rs");

        let paths = vec![dir.path().to_path_buf()];
        let without = analyze_dead_code(&paths, false).await.expect("analysis");
        let with = analyze_dead_code(&paths, true).await.expect("analysis");

        let count = |v: &Value| v["results"]["total_dead_code"].as_u64().unwrap_or(0);
        assert!(
            count(&with) > count(&without),
            "include_tests made no difference: {} vs {}",
            count(&without),
            count(&with)
        );
        assert_eq!(
            without["results"]["analyzer"].as_str(),
            Some("multi-language-reachability"),
            "the payload must name which analyzer produced these numbers"
        );
    }
}
