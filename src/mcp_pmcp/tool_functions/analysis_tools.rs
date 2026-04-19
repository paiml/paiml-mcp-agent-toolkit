#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_complexity(
    paths: &[PathBuf],
    top_files: Option<usize>,
    threshold: Option<u64>,
) -> Result<Value> {
    use crate::services::complexity::analyze_file_complexity_uncached;

    // Validate input
    if paths.is_empty() {
        return Err(anyhow::anyhow!("At least one path must be provided"));
    }

    let threshold_value = threshold.unwrap_or(10);

    // Analyze all provided paths
    let mut all_functions = Vec::new();
    let mut total_files = 0;
    let mut total_complexity = 0u64;
    let mut violations = Vec::new();

    for path in paths {
        // Skip non-existent paths
        if !path.exists() {
            continue;
        }

        // Analyze single file
        if path.is_file() {
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

    let mut total_satd = 0;
    let mut file_results = Vec::new();

    for path in paths {
        if !path.exists() || !path.is_file() {
            continue;
        }

        match tokio::fs::read_to_string(path).await {
            Ok(content) => match detector.extract_from_content(&content, path) {
                Ok(debts) => {
                    // Filter out resolved debt markers (DONE, RESOLVED, FIXED) unless include_resolved
                    let debts: Vec<_> = if _include_resolved {
                        debts
                    } else {
                        debts.into_iter()
                            .filter(|d| {
                                let upper = d.text.to_uppercase();
                                !upper.contains("DONE") && !upper.contains("RESOLVED") && !upper.contains("FIXED")
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
pub async fn analyze_dead_code(paths: &[PathBuf], _include_tests: bool) -> Result<Value> {
    use crate::services::dead_code_multi_language::analyze_dead_code_multi_language;

    // Validate input
    if paths.is_empty() {
        return Err(anyhow::anyhow!("At least one path must be provided"));
    }

    let mut total_dead_code = 0;
    let mut file_results = Vec::new();

    for path in paths {
        if !path.exists() || !path.is_file() {
            continue;
        }

        match analyze_dead_code_multi_language(path) {
            Ok(result) => {
                let dead_count = result.dead_functions.len();
                total_dead_code += dead_count;

                if dead_count > 0 {
                    file_results.push(json!({
                        "file": path.display().to_string(),
                        "dead_code_count": dead_count,
                        "dead_functions": result.dead_functions.iter().map(|func| json!({
                            "name": func.name,
                            "line": func.line,
                        })).collect::<Vec<_>>(),
                    }));
                }
            }
            Err(_) => continue,
        }
    }

    Ok(json!({
        "status": "completed",
        "message": "Dead code analysis completed",
        "results": {
            "total_dead_code": total_dead_code,
            "files": file_results,
        }
    }))
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

// --- DAG analysis (R17-1) ---
//
// Dispatches to `services::deep_context::analysis_functions::analyze_dag`,
// which builds a ProjectContext + DagBuilder graph. This is the analysis
// powering `pmat analyze dag`.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_dag(paths: &[PathBuf], dag_type: Option<String>) -> Result<Value> {
    use crate::services::deep_context::analysis_functions::analyze_dag as svc_analyze_dag;
    use crate::services::deep_context::DagType;

    if paths.is_empty() {
        return Err(anyhow::anyhow!("At least one path must be provided"));
    }

    let dag_type_parsed = match dag_type.as_deref().unwrap_or("full-dependency") {
        "call-graph" | "call_graph" => DagType::CallGraph,
        "import-graph" | "import_graph" => DagType::ImportGraph,
        "inheritance" => DagType::Inheritance,
        _ => DagType::FullDependency,
    };
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
                "time_complexity": f.time_complexity,
                "space_complexity": f.space_complexity,
                "confidence": f.confidence,
            })
        })
        .collect();

    Ok(json!({
        "status": "completed",
        "message": format!("Big-O analysis completed ({} functions analyzed)", report.analyzed_functions),
        "results": {
            "analyzed_functions": report.analyzed_functions,
            "complexity_distribution": report.complexity_distribution,
            "high_complexity_functions": high_complexity,
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
    _include_patterns: Option<Vec<String>>,
) -> Result<Value> {
    use crate::services::deep_context::{DeepContextAnalyzer, DeepContextConfig};

    if paths.is_empty() {
        return Err(anyhow::anyhow!("At least one path must be provided"));
    }

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
            "quality_scorecard": {
                "overall_health": context.quality_scorecard.overall_health,
                "complexity_score": context.quality_scorecard.complexity_score,
                "maintainability_index": context.quality_scorecard.maintainability_index,
                "modularity_score": context.quality_scorecard.modularity_score,
                "technical_debt_hours": context.quality_scorecard.technical_debt_hours,
            },
            "file_count": context.file_tree.total_files,
            "ast_contexts": context.analyses.ast_contexts.len(),
        }
    }))
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
