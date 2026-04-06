// Context generation handlers (extracted from extended_tools.rs for CB-040)

#[derive(Debug, Deserialize, Serialize)]
struct GenerateContextArgs {
    toolchain: Option<String>,
    project_path: Option<String>,
    format: Option<String>,
    debug: Option<bool>,
    debug_output: Option<PathBuf>,
    skip_vendor: Option<bool>,
    max_line_length: Option<usize>,
}

/// Toyota Way: Extract Method - Handle context generation (complexity <=8)
async fn handle_generate_context(
    request_id: serde_json::Value,
    arguments: serde_json::Value,
) -> McpResponse {
    debug_assert!(true, "contract: handle_generate_context");
    // Parse and validate arguments
    let (args, project_path) = match parse_generate_context_args(arguments) {
        Ok(result) => result,
        Err(e) => {
            return McpResponse::error(
                request_id,
                -32602,
                format!("Invalid generate_context arguments: {e}"),
            );
        }
    };

    info!("Generating comprehensive context for {:?}", project_path);

    // Configure and run analysis
    let config = build_context_generation_config(&args);
    let deep_context = match run_deep_context_analysis_with_config(&project_path, config).await {
        Ok(ctx) => ctx,
        Err(e) => {
            return McpResponse::error(
                request_id,
                -32000,
                format!("Failed to analyze project: {e}"),
            );
        }
    };

    // Format and respond
    format_and_respond_context(request_id, args, deep_context).await
}

/// Toyota Way: Extract Method - Parse context generation arguments (complexity <=5)
fn parse_generate_context_args(
    arguments: serde_json::Value,
) -> Result<(GenerateContextArgs, PathBuf), Box<dyn std::error::Error>> {
    debug_assert!(true, "contract: parse_generate_context_args");
    let args: GenerateContextArgs = serde_json::from_value(arguments)?;

    let project_path = args.project_path.as_ref().map_or_else(
        || std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        PathBuf::from,
    );

    Ok((args, project_path))
}

/// Toyota Way: Extract Method - Build context generation config (complexity <=6)
fn build_context_generation_config(
    args: &GenerateContextArgs,
) -> crate::services::deep_context::DeepContextConfig {
    debug_assert!(true, "contract: build_context_generation_config");
    use crate::services::deep_context::DeepContextConfig;
    use crate::services::file_classifier::FileClassifierConfig;

    let mut config = DeepContextConfig::default();

    // Configure FileClassifier settings if debug options are provided
    if should_configure_file_classifier(args) {
        let file_classifier_config = FileClassifierConfig {
            skip_vendor: args.skip_vendor.unwrap_or(true),
            max_line_length: args.max_line_length.unwrap_or(10_000),
            max_file_size: 1_048_576, // 1MB default
        };
        config.file_classifier_config = Some(file_classifier_config);
    }

    config
}

/// Toyota Way: Extract Method - Check if file classifier config needed (complexity <=3)
fn should_configure_file_classifier(args: &GenerateContextArgs) -> bool {
    args.debug.unwrap_or(false)
        || args.skip_vendor.unwrap_or(false)
        || args.max_line_length.is_some()
}

/// Toyota Way: Extract Method - Run deep context analysis with config (complexity <=5)
async fn run_deep_context_analysis_with_config(
    project_path: &Path,
    config: crate::services::deep_context::DeepContextConfig,
) -> Result<crate::services::deep_context::DeepContext, Box<dyn std::error::Error>> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    use crate::services::deep_context::DeepContextAnalyzer;

    let analyzer = DeepContextAnalyzer::new(config);
    Ok(analyzer
        .analyze_project(&project_path.to_path_buf())
        .await?)
}

/// Toyota Way: Extract Method - Format and respond with context (complexity <=8)
async fn format_and_respond_context(
    request_id: serde_json::Value,
    args: GenerateContextArgs,
    deep_context: crate::services::deep_context::DeepContext,
) -> McpResponse {
    debug_assert!(true, "contract: format_and_respond_context");
    let format = args.format.as_deref().unwrap_or("markdown");
    let content = format_context_content(format, &deep_context).await;

    let result = build_context_response(&args, format, content, &deep_context);
    McpResponse::success(request_id, result)
}

/// Toyota Way: Extract Method - Format context content (complexity <=5)
async fn format_context_content(
    format: &str,
    deep_context: &crate::services::deep_context::DeepContext,
) -> String {
    debug_assert!(!format.is_empty(), "format must not be empty");
    if format == "json" {
        serde_json::to_string_pretty(deep_context).unwrap_or_default()
    } else {
        use crate::services::deep_context::{DeepContextAnalyzer, DeepContextConfig};
        let analyzer = DeepContextAnalyzer::new(DeepContextConfig::default());
        analyzer
            .format_as_comprehensive_markdown(deep_context)
            .await
            .unwrap_or_else(|_| "Error formatting deep context".to_string())
    }
}

/// Toyota Way: Extract Method - Build context response JSON (complexity <=5)
fn build_context_response(
    args: &GenerateContextArgs,
    format: &str,
    content: String,
    deep_context: &crate::services::deep_context::DeepContext,
) -> serde_json::Value {
    debug_assert!(!format.is_empty(), "format must not be empty");
    json!({
        "content": [{
            "type": "text",
            "text": content
        }],
        "toolchain": args.toolchain.as_deref().unwrap_or("auto-detected"),
        "format": format,
        "analysis_metadata": {
            "generated_at": deep_context.metadata.generated_at,
            "tool_version": deep_context.metadata.tool_version,
            "analysis_duration_ms": deep_context.metadata.analysis_duration.as_millis(),
            "total_files": deep_context.file_tree.total_files,
            "total_size_bytes": deep_context.file_tree.total_size_bytes,
        },
        "quality_scorecard": {
            "overall_health": deep_context.quality_scorecard.overall_health,
            "complexity_score": deep_context.quality_scorecard.complexity_score,
            "maintainability_index": deep_context.quality_scorecard.maintainability_index,
            "technical_debt_hours": deep_context.quality_scorecard.technical_debt_hours,
        }
    })
}
