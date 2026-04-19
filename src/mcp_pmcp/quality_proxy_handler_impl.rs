#[async_trait]
impl ToolHandler for QualityProxyTool {
    fn metadata(&self) -> Option<ToolInfo> {
        let schema = json!({
            "type": "object",
            "properties": {
                "operation":  { "type": "string", "enum": ["write", "edit", "append"], "description": "File operation to proxy" },
                "file_path":  { "type": "string", "description": "Target file path" },
                "content":     { "type": "string", "description": "New content for `write`/`append`" },
                "old_content": { "type": "string", "description": "Old string for `edit` operations" },
                "new_content": { "type": "string", "description": "New string for `edit` operations" },
                "mode":        { "type": "string", "enum": ["strict", "advisory", "auto_fix", "auto-fix"], "description": "Proxy enforcement mode" },
                "quality_config": {
                    "type": "object",
                    "properties": {
                        "max_complexity": { "type": "integer" },
                        "allow_satd":     { "type": "boolean" },
                        "require_docs":   { "type": "boolean" },
                        "auto_format":    { "type": "boolean" }
                    }
                }
            },
            "required": ["operation", "file_path"]
        });
        Some(build_tool_info(
            "quality_proxy",
            "Proxy a file operation (write/edit/append) through the quality gate, optionally auto-fixing violations.",
            schema,
        ))
    }

    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug!("Handling quality_proxy with args: {}", args);

        let input: QualityProxyInput = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        info!("Processing quality proxy request for {}", input.file_path);
        debug!("Proxy mode: {}, Operation: {}", input.mode, input.operation);

        // Convert input to ProxyRequest
        let operation = match input.operation.as_str() {
            "write" => ProxyOperation::Write,
            "edit" => ProxyOperation::Edit,
            "append" => ProxyOperation::Append,
            _ => {
                return Err(Error::validation(format!(
                    "Invalid operation: {}",
                    input.operation
                )))
            }
        };

        let mode = match input.mode.as_str() {
            "strict" => ProxyMode::Strict,
            "advisory" => ProxyMode::Advisory,
            "auto_fix" | "auto-fix" => ProxyMode::AutoFix,
            _ => return Err(Error::validation(format!("Invalid mode: {}", input.mode))),
        };

        let quality_config = QualityConfig {
            max_complexity: input.quality_config.max_complexity,
            allow_satd: input.quality_config.allow_satd,
            require_docs: input.quality_config.require_docs,
            auto_format: input.quality_config.auto_format,
        };

        let request = ProxyRequest {
            operation,
            file_path: input.file_path.clone(),
            content: input.content,
            old_content: input.old_content,
            new_content: input.new_content,
            mode,
            quality_config,
        };

        // Process the request
        let service = QualityProxyService::new();
        let response = service
            .proxy_operation(request)
            .await
            .map_err(|e| Error::internal(format!("Failed to process request: {e}")))?;

        // Convert response to JSON
        let result = serde_json::to_value(response)
            .map_err(|e| Error::internal(format!("Failed to serialize response: {e}")))?;

        info!("Quality proxy request completed");
        Ok(result)
    }
}
