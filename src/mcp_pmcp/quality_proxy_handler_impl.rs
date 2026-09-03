#[async_trait]
impl ToolHandler for QualityProxyTool {
    fn metadata(&self) -> Option<ToolInfo> {
        let schema = json!({
            "type": "object",
            "properties": {
                "file_path":  { "type": "string", "description": "Path the content is destined for (decides the language and the project's pmat.toml)" },
                "content":     { "type": "string", "description": "The proposed file content to grade" },
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
            "required": ["file_path", "content"]
        });
        Some(build_tool_info(
            QUALITY_CHECK_CONTENT,
            QUALITY_CHECK_DESCRIPTION,
            schema,
        ))
    }

    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug!("Handling quality_proxy with args: {}", args);

        let input: QualityProxyInput = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        info!("Processing quality proxy request for {}", input.file_path);
        debug!("Proxy mode: {}", input.mode);

        // Convert input to ProxyRequest
        // Internally the service still models a "write" of the whole content —
        // that is the only shape a content check needs, and the enum is kept
        // for the service's own callers.
        let operation = ProxyOperation::Write;

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
            content: Some(input.content),
            old_content: None,
            new_content: None,
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

#[async_trait]
impl ToolHandler for QualityProxyAliasTool {
    fn metadata(&self) -> Option<ToolInfo> {
        // Identical to the live tool except the name, so a client that still
        // calls `quality_proxy` sees the same schema and the same description
        // (which no longer claims a write).
        let mut info = QualityProxyTool.metadata()?;
        info.name = QUALITY_PROXY_ALIAS.to_string();
        Some(info)
    }

    async fn handle(&self, args: Value, extra: RequestHandlerExtra) -> Result<Value> {
        QualityProxyTool.handle(args, extra).await
    }
}
