
pub(crate) async fn handle_analyze_deep_context(
    request_id: serde_json::Value,
    arguments: serde_json::Value,
) -> McpResponse {
    let args = match parse_deep_context_args(arguments) {
        Ok(args) => args,
        Err(e) => return McpResponse::error(request_id, -32602, e),
    };

    let project_path = resolve_deep_context_project_path(args.project_path.clone());
    info!("Running deep context analysis for {:?}", project_path);

    let config = build_deep_context_config(&args);
    let analyzer = create_deep_context_analyzer(config);

    match analyzer.analyze_project(&project_path).await {
        Ok(context) => {
            let result = format_deep_context_response(&context, &args);
            McpResponse::success(request_id, result)
        }
        Err(e) => {
            error!("Deep context analysis failed: {}", e);
            McpResponse::error(request_id, -32000, e.to_string())
        }
    }
}

fn parse_deep_context_args(arguments: serde_json::Value) -> Result<AnalyzeDeepContextArgs, String> {
    serde_json::from_value(arguments)
        .map_err(|e| format!("Invalid analyze_deep_context arguments: {e}"))
}

fn resolve_deep_context_project_path(project_path: Option<String>) -> PathBuf {
    project_path.map_or_else(
        || std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        PathBuf::from,
    )
}

fn default_project_path() -> String {
    ".".to_string()
}

fn default_top_files() -> usize {
    10
}

fn get_default_analysis_types() -> Vec<crate::services::deep_context::AnalysisType> {
    use crate::services::deep_context::AnalysisType;
    vec![
        AnalysisType::Ast,
        AnalysisType::Complexity,
        AnalysisType::Churn,
    ]
}

fn parse_analysis_type_string(s: &str) -> Option<crate::services::deep_context::AnalysisType> {
    use crate::services::deep_context::AnalysisType;

    match s {
        "ast" => Some(AnalysisType::Ast),
        "complexity" => Some(AnalysisType::Complexity),
        "churn" => Some(AnalysisType::Churn),
        "dag" => Some(AnalysisType::Dag),
        "dead_code" => Some(AnalysisType::DeadCode),
        "satd" => Some(AnalysisType::Satd),
        "tdg" => Some(AnalysisType::TechnicalDebtGradient),
        _ => None,
    }
}

fn parse_analysis_types(
    include_analyses: Option<Vec<String>>,
) -> Vec<crate::services::deep_context::AnalysisType> {
    match include_analyses {
        Some(analyses) => analyses
            .iter()
            .filter_map(|s| parse_analysis_type_string(s))
            .collect(),
        None => get_default_analysis_types(),
    }
}

fn parse_deep_context_dag_type(dag_type: Option<String>) -> crate::services::deep_context::DagType {
    use crate::services::deep_context::DagType;

    match dag_type.as_deref() {
        Some("import-graph") => DagType::ImportGraph,
        Some("inheritance") => DagType::Inheritance,
        Some("full-dependency") => DagType::FullDependency,
        Some("call-graph") | None => DagType::CallGraph,
        _ => DagType::CallGraph,
    }
}

fn parse_cache_strategy(
    cache_strategy: Option<String>,
) -> crate::services::deep_context::CacheStrategy {
    use crate::services::deep_context::CacheStrategy;

    match cache_strategy.as_deref() {
        Some("force-refresh") => CacheStrategy::ForceRefresh,
        Some("offline") => CacheStrategy::Offline,
        Some("normal") | None => CacheStrategy::Normal,
        _ => CacheStrategy::Normal,
    }
}

fn build_deep_context_config(
    args: &AnalyzeDeepContextArgs,
) -> crate::services::deep_context::DeepContextConfig {
    use crate::services::deep_context::{ComplexityThresholds, DeepContextConfig};

    DeepContextConfig {
        include_analyses: parse_analysis_types(args.include_analyses.clone()),
        period_days: args.period_days.unwrap_or(30),
        dag_type: parse_deep_context_dag_type(args.dag_type.clone()),
        complexity_thresholds: Some(ComplexityThresholds {
            max_cyclomatic: 10,
            max_cognitive: 15,
        }),
        max_depth: args.max_depth,
        include_patterns: args.include_pattern.clone().unwrap_or_default(),
        exclude_patterns: args.exclude_pattern.clone().unwrap_or_default(),
        cache_strategy: parse_cache_strategy(args.cache_strategy.clone()),
        parallel: args.parallel.unwrap_or(4),
        file_classifier_config: None, // Default to None for deep context analysis
    }
}

fn create_deep_context_analyzer(
    config: crate::services::deep_context::DeepContextConfig,
) -> crate::services::deep_context::DeepContextAnalyzer {
    crate::services::deep_context::DeepContextAnalyzer::new(config)
}

fn format_deep_context_response(
    context: &crate::services::deep_context::DeepContext,
    args: &AnalyzeDeepContextArgs,
) -> serde_json::Value {
    let format = args.format.as_deref().unwrap_or("markdown");
    let content_text = match format {
        "json" => serde_json::to_string_pretty(context).unwrap_or_default(),
        "sarif" => format_deep_context_as_sarif(context),
        _ => {
            // Note: This is a sync context, so we can't easily use async here
            // The format_deep_context_as_markdown function has been updated to include
            // README and Makefile metadata when available
            format_deep_context_as_markdown(context)
        }
    };

    json!({
        "content": [{
            "type": "text",
            "text": content_text
        }],
        "context": context,
        "format": format!("{:?}", format),
        "analysis_duration_ms": context.metadata.analysis_duration.as_millis(),
    })
}

fn format_deep_context_as_sarif(_context: &crate::services::deep_context::DeepContext) -> String {
    // Simple SARIF implementation for MCP
    use serde_json::json;

    let sarif = json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "pmat",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/mcp-agent-toolkit"
                }
            },
            "results": []
        }]
    });

    serde_json::to_string_pretty(&sarif).unwrap_or_default()
}

/// Toyota Way: Extract Method - Format deep context analysis as markdown (complexity ≤8)
fn format_deep_context_as_markdown(context: &crate::services::deep_context::DeepContext) -> String {
    use crate::cli::formatting_helpers::{
        format_defect_summary, format_executive_summary, format_quality_scorecard,
        format_recommendations,
    };

    let mut output = String::with_capacity(1024);
    output.push_str("# Deep Context Analysis\n\n");

    // Reuse helper functions from cli module
    output.push_str(&format_executive_summary(context));
    format_essential_metadata(&mut output, context);

    // Quality Scorecard and other sections
    output.push_str(&format_quality_scorecard(context));
    output.push_str(&format_defect_summary(context));
    output.push_str(&format_recommendations(context));

    format_analysis_results(&mut output, context);
    format_deep_context_recommendations(&mut output, context);

    output
}

/// Toyota Way: Extract Method - Format essential project metadata section (complexity ≤5)
fn format_essential_metadata(
    output: &mut String,
    context: &crate::services::deep_context::DeepContext,
) {
    use crate::cli::formatting_helpers::{format_build_info, format_project_overview};

    if context.project_overview.is_some() || context.build_info.is_some() {
        output.push_str("\n## Essential Project Metadata\n\n");

        if let Some(ref overview) = context.project_overview {
            output.push_str(&format_project_overview(overview));
        }

        if let Some(ref build_info) = context.build_info {
            output.push_str(&format_build_info(build_info));
        }
    }
}

/// Toyota Way: Extract Method - Format analysis results section (complexity ≤8)
fn format_analysis_results(
    output: &mut String,
    context: &crate::services::deep_context::DeepContext,
) {
    output.push_str("\n## Analysis Results\n\n");
    output.push_str(&format!(
        "**Total Defects:** {}\n",
        context.defect_summary.total_defects
    ));
    output.push_str(&format!(
        "**Defect Density:** {:.2}\n",
        context.defect_summary.defect_density
    ));

    format_defects_by_type(output, &context.defect_summary.by_type);
    format_defects_by_severity(output, &context.defect_summary.by_severity);

    output.push_str(&format!(
        "**Total Files:** {}\n\n",
        context.file_tree.total_files
    ));
}

/// Toyota Way: Extract Method - Format defects by type (complexity ≤5)
fn format_defects_by_type(output: &mut String, by_type: &rustc_hash::FxHashMap<String, usize>) {
    if !by_type.is_empty() {
        output.push_str("**By Type:**\n");
        for (defect_type, count) in by_type {
            output.push_str(&format!("- {defect_type}: {count}\n"));
        }
    }
}

/// Toyota Way: Extract Method - Format defects by severity (complexity ≤5)
fn format_defects_by_severity(
    output: &mut String,
    by_severity: &rustc_hash::FxHashMap<String, usize>,
) {
    if !by_severity.is_empty() {
        output.push_str("**By Severity:**\n");
        for (severity, count) in by_severity {
            output.push_str(&format!("- {severity}: {count}\n"));
        }
    }
}

/// Toyota Way: Extract Method - Format recommendations section (complexity ≤5)
fn format_deep_context_recommendations(
    output: &mut String,
    context: &crate::services::deep_context::DeepContext,
) {
    if !context.recommendations.is_empty() {
        output.push_str("## Recommendations\n\n");
        for (i, rec) in context.recommendations.iter().take(5).enumerate() {
            output.push_str(&format!(
                "{}. **{}** (Priority: {:?})\n",
                i + 1,
                rec.title,
                rec.priority
            ));
            output.push_str(&format!("   {}\n\n", rec.description));
        }
    }
}

#[derive(Deserialize)]
struct MakefileLintArgs {
    path: String,
    #[serde(default)]
    rules: Vec<String>,
    #[serde(default)]
    #[allow(dead_code)]
    fix: bool,
    #[serde(default)]
    #[allow(dead_code)]
    gnu_version: String,
}

fn parse_makefile_lint_args(
    arguments: Option<serde_json::Value>,
) -> Result<MakefileLintArgs, String> {
    match arguments {
        Some(args) => serde_json::from_value(args)
            .map_err(|e| format!("Invalid analyze_makefile_lint arguments: {e}")),
        None => Err("Missing required arguments for analyze_makefile_lint".to_string()),
    }
}

async fn execute_makefile_linting(
    makefile_path: &std::path::Path,
) -> Result<crate::services::makefile_linter::LintResult, String> {
    use crate::services::makefile_linter;

    makefile_linter::lint_makefile(makefile_path)
        .await
        .map_err(|e| format!("Makefile linting failed: {e}"))
}

fn map_severity(severity: &crate::services::makefile_linter::Severity) -> &'static str {
    use crate::services::makefile_linter::Severity;

    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Performance => "performance",
        Severity::Info => "info",
    }
}

fn format_violation(violation: &crate::services::makefile_linter::Violation) -> serde_json::Value {
    json!({
        "rule": violation.rule,
        "severity": map_severity(&violation.severity),
        "line": violation.span.line,
        "column": violation.span.column,
        "message": violation.message,
        "fix_hint": violation.fix_hint,
    })
}

fn count_violations_by_severity(
    violations: &[crate::services::makefile_linter::Violation],
    _target_severity: crate::services::makefile_linter::Severity,
) -> usize {
    violations
        .iter()
        .filter(|v| matches!(&v.severity, _target_severity))
        .count()
}

fn build_makefile_analysis(
    args: &MakefileLintArgs,
    lint_result: &crate::services::makefile_linter::LintResult,
) -> serde_json::Value {
    use crate::services::makefile_linter::Severity;

    json!({
        "path": args.path,
        "violations": lint_result.violations.iter().map(format_violation).collect::<Vec<_>>(),
        "quality_score": lint_result.quality_score,
        "rules_applied": args.rules,
        "total_violations": lint_result.violations.len(),
        "error_count": count_violations_by_severity(&lint_result.violations, Severity::Error),
        "warning_count": count_violations_by_severity(&lint_result.violations, Severity::Warning),
    })
}

pub(crate) async fn handle_analyze_makefile_lint(
    request_id: serde_json::Value,
    arguments: Option<serde_json::Value>,
) -> McpResponse {
    let args = match parse_makefile_lint_args(arguments) {
        Ok(args) => args,
        Err(e) => return McpResponse::error(request_id, -32602, e),
    };

    let makefile_path = std::path::Path::new(&args.path);
    info!("Analyzing Makefile at {:?}", makefile_path);

    let lint_result = match execute_makefile_linting(makefile_path).await {
        Ok(result) => result,
        Err(e) => return McpResponse::error(request_id, -32000, e),
    };

    let analysis = build_makefile_analysis(&args, &lint_result);
    McpResponse::success(request_id, analysis)
}

pub(crate) async fn handle_analyze_provability(
    request_id: serde_json::Value,
    arguments: Option<serde_json::Value>,
) -> McpResponse {
    #[derive(Deserialize)]
    struct ProvabilityArgs {
        project_path: String,
        #[serde(default)]
        functions: Option<Vec<String>>,
        #[serde(default)]
        analysis_depth: Option<usize>,
    }

    let args: ProvabilityArgs = match arguments {
        Some(args) => match serde_json::from_value(args) {
            Ok(args) => args,
            Err(e) => {
                return McpResponse::error(
                    request_id,
                    -32602,
                    format!("Invalid analyze_provability arguments: {e}"),
                );
            }
        },
        None => {
            return McpResponse::error(
                request_id,
                -32602,
                "Missing required arguments for analyze_provability".to_string(),
            );
        }
    };

    info!("Analyzing provability for project: {:?}", args.project_path);

    // Use the existing provability analyzer service
    use crate::services::lightweight_provability_analyzer::{
        FunctionId, LightweightProvabilityAnalyzer,
    };

    let analyzer = LightweightProvabilityAnalyzer::new();

    // Extract functions from parameters or mock discovery from project path
    let functions = if let Some(function_names) = args.functions {
        function_names
            .into_iter()
            .enumerate()
            .map(|(i, name)| FunctionId {
                file_path: format!("{}/src/lib.rs", args.project_path),
                function_name: name,
                line_number: i * 10, // Mock line numbers
            })
            .collect()
    } else {
        // Mock function discovery from project path
        vec![FunctionId {
            file_path: format!("{}/src/main.rs", args.project_path),
            function_name: "main".to_string(),
            line_number: 1,
        }]
    };

    let summaries = analyzer.analyze_incrementally(&functions).await;

    let average_score = if summaries.is_empty() {
        0.0
    } else {
        summaries.iter().map(|s| s.provability_score).sum::<f64>() / summaries.len() as f64
    };

    let analysis = json!({
        "project_path": args.project_path,
        "analysis_depth": args.analysis_depth.unwrap_or(10),
        "functions_analyzed": summaries.len(),
        "average_provability_score": average_score,
        "summaries": summaries.iter().map(|s| json!({
            "function_id": format!("{}:{}", s.version, "main"), // Mock function ID
            "provability_score": s.provability_score,
            "verified_properties": s.verified_properties,
            "analysis_time_us": s.analysis_time_us,
        })).collect::<Vec<_>>(),
        "confidence_breakdown": {
            "high_confidence": summaries.iter().filter(|s| s.provability_score > 0.8).count(),
            "medium_confidence": summaries.iter().filter(|s| s.provability_score > 0.5 && s.provability_score <= 0.8).count(),
            "low_confidence": summaries.iter().filter(|s| s.provability_score <= 0.5).count(),
        }
    });

    McpResponse::success(request_id, analysis)
}
