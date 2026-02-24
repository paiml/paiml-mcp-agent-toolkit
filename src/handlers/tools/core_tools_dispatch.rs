pub async fn handle_tool_call<T: TemplateServerTrait>(
    server: Arc<T>,
    request: McpRequest,
) -> McpResponse {
    let tool_params = match parse_tool_call_params(request.params, &request.id) {
        Ok(params) => params,
        Err(response) => return *response,
    };

    dispatch_tool_call(server, request.id, tool_params).await
}

fn parse_tool_call_params(
    params: Option<serde_json::Value>,
    request_id: &serde_json::Value,
) -> Result<ToolCallParams, Box<McpResponse>> {
    let params = match params {
        Some(p) => p,
        None => {
            return Err(Box::new(McpResponse::error(
                request_id.clone(),
                -32602,
                "Invalid params: missing tool call parameters".to_string(),
            )));
        }
    };

    match serde_json::from_value(params) {
        Ok(p) => Ok(p),
        Err(e) => Err(Box::new(McpResponse::error(
            request_id.clone(),
            -32602,
            format!("Invalid params: {e}"),
        ))),
    }
}

async fn dispatch_tool_call<T: TemplateServerTrait>(
    server: Arc<T>,
    request_id: serde_json::Value,
    tool_params: ToolCallParams,
) -> McpResponse {
    match tool_params.name.as_str() {
        "get_server_info" => handle_get_server_info(request_id).await,
        tool_name if is_template_tool(tool_name) => {
            handle_template_tools(server, request_id, tool_params).await
        }
        tool_name if is_analysis_tool(tool_name) => {
            handle_analysis_tools(request_id, tool_params).await
        }
        tool_name if super::vectorized_tools::is_vectorized_tool(tool_name) => {
            super::vectorized_tools::handle_vectorized_tools(request_id, tool_params).await
        }
        _ => McpResponse::error(
            request_id,
            -32602,
            format!("Unknown tool: {}", tool_params.name),
        ),
    }
}

/// Check if a tool name is a template tool
pub fn is_template_tool(tool_name: &str) -> bool {
    matches!(
        tool_name,
        "generate_template"
            | "list_templates"
            | "validate_template"
            | "scaffold_project"
            | "search_templates"
    )
}

/// Check if a tool name is an analysis tool
pub fn is_analysis_tool(tool_name: &str) -> bool {
    matches!(
        tool_name,
        "analyze_code_churn"
            | "analyze_complexity"
            | "analyze_dag"
            | "generate_context"
            | "analyze_system_architecture"
            | "analyze_defect_probability"
            | "analyze_dead_code"
            | "analyze_deep_context"
            | "analyze_tdg"
            | "analyze_makefile_lint"
            | "analyze_provability"
            | "analyze_satd"
            | "quality_driven_development"
            | "analyze_lint_hotspot"
    )
}

async fn handle_template_tools<T: TemplateServerTrait>(
    server: Arc<T>,
    request_id: serde_json::Value,
    tool_params: ToolCallParams,
) -> McpResponse {
    match tool_params.name.as_str() {
        "generate_template" => {
            handle_generate_template(server, request_id, tool_params.arguments).await
        }
        "list_templates" => handle_list_templates(server, request_id, tool_params.arguments).await,
        "validate_template" => {
            handle_validate_template(server, request_id, tool_params.arguments).await
        }
        "scaffold_project" => {
            handle_scaffold_project(server, request_id, tool_params.arguments).await
        }
        "search_templates" => {
            handle_search_templates(server, request_id, tool_params.arguments).await
        }
        _ => McpResponse::error(
            request_id,
            -32602,
            format!("Unsupported template tool: {}", tool_params.name),
        ),
    }
}

async fn handle_analysis_tools(
    request_id: serde_json::Value,
    tool_params: ToolCallParams,
) -> McpResponse {
    dispatch_analysis_tool(request_id, &tool_params.name, tool_params.arguments).await
}

/// Toyota Way: Extract Method - Dispatch analysis tools with grouped handling (complexity <=8)
async fn dispatch_analysis_tool(
    request_id: serde_json::Value,
    tool_name: &str,
    arguments: serde_json::Value,
) -> McpResponse {
    // Group 1: Core analysis tools
    if let Some(response) =
        handle_core_analysis_tools(request_id.clone(), tool_name, arguments.clone()).await
    {
        return response;
    }

    // Group 2: Advanced analysis tools
    if let Some(response) =
        handle_advanced_analysis_tools(request_id.clone(), tool_name, arguments.clone()).await
    {
        return response;
    }

    // Group 3: Specialized analysis tools
    if let Some(response) =
        handle_specialized_analysis_tools(request_id.clone(), tool_name, arguments).await
    {
        return response;
    }

    // Unknown tool
    McpResponse::error(
        request_id,
        -32602,
        format!("Unsupported analysis tool: {tool_name}"),
    )
}

/// Toyota Way: Extract Method - Handle core analysis tools (complexity <=5)
async fn handle_core_analysis_tools(
    request_id: serde_json::Value,
    tool_name: &str,
    arguments: serde_json::Value,
) -> Option<McpResponse> {
    match tool_name {
        "analyze_complexity" => Some(handle_analyze_complexity(request_id, arguments).await),
        "analyze_dead_code" => Some(handle_analyze_dead_code(request_id, arguments).await),
        "analyze_satd" => Some(handle_analyze_satd(request_id, arguments).await),
        "analyze_tdg" => Some(handle_analyze_tdg(request_id, arguments).await),
        _ => None,
    }
}

/// Toyota Way: Extract Method - Handle advanced analysis tools (complexity <=5)
async fn handle_advanced_analysis_tools(
    request_id: serde_json::Value,
    tool_name: &str,
    arguments: serde_json::Value,
) -> Option<McpResponse> {
    match tool_name {
        "analyze_code_churn" => Some(handle_analyze_code_churn(request_id, arguments).await),
        "analyze_dag" => Some(handle_analyze_dag(request_id, arguments).await),
        "generate_context" => Some(handle_generate_context(request_id, arguments).await),
        "analyze_deep_context" => Some(handle_analyze_deep_context(request_id, arguments).await),
        "analyze_defect_probability" => {
            // Deprecated - redirect to TDG analysis
            Some(handle_analyze_tdg(request_id, arguments).await)
        }
        _ => None,
    }
}

/// Toyota Way: Extract Method - Handle specialized analysis tools (complexity <=5)
async fn handle_specialized_analysis_tools(
    request_id: serde_json::Value,
    tool_name: &str,
    arguments: serde_json::Value,
) -> Option<McpResponse> {
    match tool_name {
        "analyze_system_architecture" => {
            Some(handle_analyze_system_architecture(request_id, arguments).await)
        }
        "analyze_makefile_lint" => {
            Some(handle_analyze_makefile_lint(request_id, Some(arguments)).await)
        }
        "analyze_provability" => {
            Some(handle_analyze_provability(request_id, Some(arguments)).await)
        }
        "analyze_lint_hotspot" => Some(handle_analyze_lint_hotspot(request_id, arguments).await),
        "quality_driven_development" => {
            Some(handle_quality_driven_development(request_id, arguments).await)
        }
        _ => None,
    }
}
