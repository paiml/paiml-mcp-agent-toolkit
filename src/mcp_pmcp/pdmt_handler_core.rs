// Core ToolHandler implementation for PdmtTool
// Included from pdmt_handler.rs — no `use` imports or `#!` inner attributes allowed

#[async_trait]
impl ToolHandler for PdmtTool {
    fn metadata(&self) -> Option<ToolInfo> {
        Some(tool_info_for("pdmt_deterministic_todos"))
    }

    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug!("Handling pdmt_deterministic_todos with args: {}", args);

        let input: PdmtInput = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        if input.requirements.is_empty() {
            return Err(Error::validation("Requirements list cannot be empty"));
        }

        info!(
            "Generating deterministic todos for {} requirements",
            input.requirements.len()
        );

        // Convert input quality config to internal format
        let enforcement_mode = match input.quality_config.enforcement_mode.as_str() {
            "strict" => EnforcementMode::Strict,
            "advisory" => EnforcementMode::Advisory,
            "auto_fix" | "autofix" => EnforcementMode::AutoFix,
            _ => {
                return Err(Error::validation(format!(
                    "Invalid enforcement mode: {}. Must be 'strict', 'advisory', or 'auto_fix'",
                    input.quality_config.enforcement_mode
                )))
            }
        };

        let quality_config = PdmtQualityConfig {
            enforcement_mode,
            coverage_threshold: input.quality_config.coverage_threshold,
            max_complexity: input.quality_config.max_complexity,
            require_doctests: input.quality_config.require_doctests,
            require_property_tests: input.quality_config.require_property_tests,
            require_examples: input.quality_config.require_examples,
            zero_satd_tolerance: input.quality_config.zero_satd_tolerance,
        };

        // Generate deterministic todos
        let todo_list = self
            .service
            .generate_todos(
                input.requirements,
                input.project_name,
                &input.granularity,
                quality_config,
            )
            .map_err(|e| Error::internal(format!("Failed to generate todos: {e}")))?;

        // Run quality enforcement validation
        let quality_validation = self
            .quality_enforcer
            .enforce_quality_standards(&todo_list)
            .await
            .map_err(|e| Error::internal(format!("Quality validation failed: {e}")))?;

        // Calculate summary statistics
        let total_todos = todo_list.todos.len();
        let estimated_total_hours: f32 = todo_list.todos.iter().map(|t| t.estimated_hours).sum();

        // Prepare output
        let output = PdmtOutput {
            success: quality_validation.overall_passed,
            message: if quality_validation.overall_passed {
                format!(
                        "Successfully generated {total_todos} deterministic todos with quality enforcement"
                    )
            } else {
                format!(
                        "Generated {total_todos} todos but quality validation failed. Review violations."
                    )
            },
            todo_list: Some(
                serde_json::to_value(&todo_list)
                    .map_err(|e| Error::internal(format!("Failed to serialize todo list: {e}")))?,
            ),
            quality_validation: Some(serde_json::to_value(&quality_validation).map_err(|e| {
                Error::internal(format!("Failed to serialize validation results: {e}"))
            })?),
            total_todos,
            estimated_total_hours,
        };

        if quality_validation.overall_passed {
            info!(
                "Successfully generated {} todos totaling {} hours",
                total_todos, estimated_total_hours
            );
        } else {
            error!(
                "Quality validation failed with {} recommendations",
                quality_validation.recommendations.len()
            );
        }

        serde_json::to_value(output)
            .map_err(|e| Error::internal(format!("Serialization error: {e}")))
    }
}
