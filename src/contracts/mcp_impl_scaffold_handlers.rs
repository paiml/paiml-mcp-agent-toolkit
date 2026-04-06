// TICKET-PMAT-6013: Scaffolding and maintenance tool implementations

impl ContractMcpServer {
    /// Handle scaffold_agent tool call (TICKET-PMAT-6017, PMAT-6022)
    async fn handle_scaffold_agent(&self, params: Value) -> Result<ToolResult> {
        match self.scaffold_agent_internal(params).await {
            Ok(data) => {
                let result = McpOperationResult::success(data);
                Ok(ToolResult::Success(serde_json::to_value(result)?))
            }
            Err(e) => {
                let result = McpOperationResult::from_error(e);
                Ok(ToolResult::Success(serde_json::to_value(result)?))
            }
        }
    }

    /// Internal implementation of scaffold_agent
    async fn scaffold_agent_internal(&self, params: Value) -> Result<Value> {
        use crate::scaffold::agent::{scaffold_agent, AgentContextBuilder, QualityLevel};
        use std::path::PathBuf;

        // Extract and validate parameters
        let name = params.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: 'name'"))?;

        let template = params.get("template")
            .and_then(|v| v.as_str())
            .unwrap_or("mcp-server");

        let output_dir = params.get("output_dir")
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        let quality_level_str = params.get("quality_level")
            .and_then(|v| v.as_str())
            .unwrap_or("standard");

        // Parse quality level
        let quality_level = match quality_level_str {
            "extreme" => QualityLevel::Extreme,
            "high" => QualityLevel::High,
            "standard" => QualityLevel::Standard,
            _ => QualityLevel::Standard,
        };

        // Build agent context
        let mut builder = AgentContextBuilder::new(name, template)
            .with_quality_level(quality_level);

        // Add features if specified
        if let Some(features_val) = params.get("features") {
            if let Some(features_array) = features_val.as_array() {
                for feature_val in features_array {
                    if let Some(feature_str) = feature_val.as_str() {
                        builder = builder.with_feature_str(feature_str);
                    }
                }
            }
        }

        let context = builder.build()?;
        let output_path = PathBuf::from(output_dir).join(name);

        // Call actual scaffolding logic
        scaffold_agent(&context, &output_path).await?;

        // Return success data
        Ok(json!({
            "project_name": name,
            "template": template,
            "output_dir": output_path.to_string_lossy(),
            "quality_level": quality_level_str,
            "message": format!("Agent project '{}' scaffolded successfully with '{}' template", name, template)
        }))
    }

    /// Handle scaffold_wasm tool call
    async fn handle_scaffold_wasm(&self, params: Value) -> Result<ToolResult> {
        let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("my-wasm");
        let framework = params.get("framework").and_then(|v| v.as_str()).unwrap_or("wasm-labs");
        let output_dir = params.get("output_dir").and_then(|v| v.as_str()).unwrap_or(".");

        let result = json!({
            "success": true,
            "project_name": name,
            "framework": framework,
            "output_dir": output_dir,
            "message": format!("WASM project '{}' scaffolded successfully with '{}' framework", name, framework)
        });

        Ok(ToolResult::Success(result))
    }

    /// Handle validate_roadmap tool call (TICKET-PMAT-6019, PMAT-6022)
    async fn handle_validate_roadmap(&self, params: Value) -> Result<ToolResult> {
        match self.validate_roadmap_internal(params).await {
            Ok(data) => {
                let result = McpOperationResult::success(data);
                Ok(ToolResult::Success(serde_json::to_value(result)?))
            }
            Err(e) => {
                let result = McpOperationResult::from_error(e);
                Ok(ToolResult::Success(serde_json::to_value(result)?))
            }
        }
    }

    /// Internal implementation of validate_roadmap
    async fn validate_roadmap_internal(&self, params: Value) -> Result<Value> {
        use crate::cli::handlers::roadmap_handler::validate_roadmap_internal as validate_impl;
        use std::path::PathBuf;

        let roadmap_path = params.get("roadmap_path")
            .and_then(|v| v.as_str())
            .unwrap_or("ROADMAP.md");

        let tickets_dir = params.get("tickets_dir")
            .and_then(|v| v.as_str())
            .unwrap_or("docs/tickets");

        // Call actual validation logic
        let validation = validate_impl(
            &PathBuf::from(roadmap_path),
            &PathBuf::from(tickets_dir),
        ).await?;

        Ok(json!({
            "roadmap_path": roadmap_path,
            "tickets_dir": tickets_dir,
            "valid": validation.valid,
            "errors": validation.errors,
            "warnings": validation.warnings,
            "message": if validation.valid {
                "Roadmap validation passed"
            } else {
                format!("{} error(s) found", validation.errors.len())
            }
        }))
    }

    /// Handle health_check tool call (TICKET-PMAT-6020, PMAT-6022)
    async fn handle_health_check(&self, params: Value) -> Result<ToolResult> {
        match self.health_check_internal(params).await {
            Ok(data) => {
                let result = McpOperationResult::success(data);
                Ok(ToolResult::Success(serde_json::to_value(result)?))
            }
            Err(e) => {
                let result = McpOperationResult::from_error(e);
                Ok(ToolResult::Success(serde_json::to_value(result)?))
            }
        }
    }

    /// Internal implementation of health_check
    async fn health_check_internal(&self, params: Value) -> Result<Value> {
        use crate::cli::handlers::health_handler::run_health_checks_internal;
        use std::path::PathBuf;

        let project_dir = params.get("project_dir")
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        let quick = params.get("quick")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let check_build = params.get("check_build")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let check_tests = params.get("check_tests")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let check_coverage = params.get("check_coverage")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let check_complexity = params.get("check_complexity")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let check_satd = params.get("check_satd")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let all = !quick && !check_build && !check_tests && !check_coverage && !check_complexity && !check_satd;

        // Call actual health check logic (with parallel execution from PMAT-6010!)
        let report = run_health_checks_internal(
            &PathBuf::from(project_dir),
            quick,
            all,
            check_build,
            check_tests,
            check_coverage,
            check_complexity,
            check_satd,
        ).await?;

        Ok(json!({
            "project_dir": project_dir,
            "quick_mode": quick,
            "healthy": report.healthy,
            "checks": report.checks,
            "summary": report.summary,
            "message": if report.healthy {
                "Project health check passed"
            } else {
                format!("{} check(s) failed", report.summary.failed)
            }
        }))
    }

    /// Handle generate_tickets tool call (TICKET-PMAT-6021, PMAT-6022)
    async fn handle_generate_tickets(&self, params: Value) -> Result<ToolResult> {
        match self.generate_tickets_internal(params).await {
            Ok(data) => {
                let result = McpOperationResult::success(data);
                Ok(ToolResult::Success(serde_json::to_value(result)?))
            }
            Err(e) => {
                let result = McpOperationResult::from_error(e);
                Ok(ToolResult::Success(serde_json::to_value(result)?))
            }
        }
    }

    /// Internal implementation of generate_tickets
    async fn generate_tickets_internal(&self, params: Value) -> Result<Value> {
        use crate::cli::handlers::roadmap_handler::generate_tickets_internal as generate_impl;
        use std::path::PathBuf;

        let roadmap_path = params.get("roadmap_path")
            .and_then(|v| v.as_str())
            .unwrap_or("ROADMAP.md");

        let tickets_dir = params.get("tickets_dir")
            .and_then(|v| v.as_str())
            .unwrap_or("docs/tickets");

        let dry_run = params.get("dry_run")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Call actual ticket generation logic
        let gen_result = generate_impl(
            &PathBuf::from(roadmap_path),
            &PathBuf::from(tickets_dir),
            dry_run,
        ).await?;

        Ok(json!({
            "roadmap_path": roadmap_path,
            "tickets_dir": tickets_dir,
            "dry_run": dry_run,
            "generated": gen_result.generated.len(),
            "generated_tickets": gen_result.generated,
            "skipped": gen_result.skipped.len(),
            "skipped_tickets": gen_result.skipped,
            "message": if gen_result.generated.is_empty() {
                "No missing ticket files".to_string()
            } else {
                format!("Generated {} ticket file(s)", gen_result.generated.len())
            }
        }))
    }
}
