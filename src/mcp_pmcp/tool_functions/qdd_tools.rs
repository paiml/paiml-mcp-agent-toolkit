/// Quality-Driven Development (QDD) tool for creating and refactoring code with guaranteed quality
// Helper function to select quality profile
fn select_quality_profile(profile_name: Option<&str>) -> QualityProfile {
    match profile_name.unwrap_or("standard") {
        "extreme" => QualityProfile::extreme(),
        "standard" => QualityProfile::standard(),
        "relaxed" => QualityProfile::relaxed(),
        _ => QualityProfile::standard(),
    }
}

// Helper function to parse code type
fn parse_code_type(code_type: Option<&str>) -> CodeType {
    match code_type.unwrap_or("function") {
        "function" => CodeType::Function,
        "module" => CodeType::Module,
        "service" => CodeType::Service,
        "test" => CodeType::Test,
        _ => CodeType::Function,
    }
}

// Helper function for create operation
async fn handle_qdd_create(
    qdd_tool: QddTool,
    quality_profile: Option<&str>,
    code_type: Option<&str>,
    name: Option<&str>,
    purpose: Option<&str>,
    inputs: Option<Vec<(String, String)>>,
    output_type: Option<&str>,
) -> Result<Value> {
    let code_type_enum = parse_code_type(code_type);

    let parameters = inputs
        .unwrap_or_default()
        .into_iter()
        .map(|(name, param_type)| Parameter {
            name,
            param_type,
            description: None,
        })
        .collect();

    let create_spec = CreateSpec {
        code_type: code_type_enum,
        name: name.unwrap_or("generated_code").to_string(),
        purpose: purpose
            .unwrap_or("Generated code with quality standards")
            .to_string(),
        inputs: parameters,
        outputs: Parameter {
            name: "result".to_string(),
            param_type: output_type.unwrap_or("()").to_string(),
            description: Some("Function output".to_string()),
        },
    };

    let operation = QddOperation::Create(create_spec);
    match qdd_tool.execute(operation).await {
        Ok(result) => Ok(json!({
            "status": "completed",
            "message": "QDD code creation successful",
            "result_type": "qdd_create",
            "quality_profile": quality_profile.unwrap_or("standard"),
            "code": result.code,
            "tests": result.tests,
            "documentation": result.documentation,
            "quality_score": {
                "overall": result.quality_score.overall,
                "complexity": result.quality_score.complexity,
                "coverage": result.quality_score.coverage,
                "tdg": result.quality_score.tdg
            },
            "metrics": {
                "complexity": result.metrics.complexity,
                "cognitive_complexity": result.metrics.cognitive_complexity,
                "coverage": result.metrics.coverage,
                "tdg": result.metrics.tdg,
                "satd_count": result.metrics.satd_count,
                "has_doctests": result.metrics.has_doctests,
                "has_property_tests": result.metrics.has_property_tests
            }
        })),
        Err(e) => Ok(json!({
            "status": "failed",
            "message": format!("QDD creation failed: {}", e),
            "result_type": "qdd_create_error",
            "error": e.to_string()
        })),
    }
}

// Helper function for refactor operation
async fn handle_qdd_refactor(
    qdd_tool: QddTool,
    profile: QualityProfile,
    quality_profile: Option<&str>,
    name: Option<&str>,
    file_path: Option<&PathBuf>,
) -> Result<Value> {
    if let Some(path) = file_path {
        let refactor_spec = RefactorSpec {
            file_path: path.clone(),
            function_name: name.map(std::string::ToString::to_string),
            target_metrics: profile.thresholds.clone(),
        };

        let operation = QddOperation::Refactor(refactor_spec);
        match qdd_tool.execute(operation).await {
            Ok(result) => Ok(json!({
                "status": "completed",
                "message": "QDD refactoring successful",
                "result_type": "qdd_refactor",
                "quality_profile": quality_profile.unwrap_or("standard"),
                "original_file": path.display().to_string(),
                "refactored_code": result.code,
                "quality_improvement": {
                    "overall_score": result.quality_score.overall,
                    "complexity": result.quality_score.complexity,
                    "coverage": result.quality_score.coverage,
                    "tdg": result.quality_score.tdg
                },
                "rollback_plan": {
                    "checkpoints": result.rollback_plan.checkpoints.len(),
                    "can_rollback": !result.rollback_plan.original.is_empty()
                }
            })),
            Err(e) => Ok(json!({
                "status": "failed",
                "message": format!("QDD refactoring failed: {}", e),
                "result_type": "qdd_refactor_error",
                "error": e.to_string(),
                "file_path": path.display().to_string()
            })),
        }
    } else {
        Ok(json!({
            "status": "failed",
            "message": "Refactor operation requires file_path parameter",
            "result_type": "qdd_refactor_error",
            "error": "Missing required parameter: file_path"
        }))
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn quality_driven_development(
    operation_type: &str,
    quality_profile: Option<&str>,
    code_type: Option<&str>,
    name: Option<&str>,
    purpose: Option<&str>,
    file_path: Option<&PathBuf>,
    inputs: Option<Vec<(String, String)>>,
    output_type: Option<&str>,
) -> Result<Value> {
    debug_assert!(!operation_type.is_empty(), "operation_type must not be empty");
    let profile = select_quality_profile(quality_profile);
    let qdd_tool = QddTool::with_profile(profile.clone());

    match operation_type {
        "create" => {
            handle_qdd_create(
                qdd_tool,
                quality_profile,
                code_type,
                name,
                purpose,
                inputs,
                output_type,
            )
            .await
        }
        "refactor" => {
            handle_qdd_refactor(qdd_tool, profile, quality_profile, name, file_path).await
        }
        _ => Ok(json!({
            "status": "failed",
            "message": format!("Unknown QDD operation: {}", operation_type),
            "result_type": "qdd_operation_error",
            "supported_operations": ["create", "refactor"],
            "error": format!("Operation '{}' not supported", operation_type)
        })),
    }
}

/// Generate defect-aware AI prompts from organizational intelligence
///
/// This MCP tool generates context-aware AI prompts that include organizational
/// quality standards and historical defect patterns from OIP (Organizational
/// Intelligence Plugin) analysis.
///
/// # Arguments
/// * `task` - The development task to generate a prompt for
/// * `context` - Additional context about the task
/// * `summary_path` - Path to OIP summary YAML file
///
/// # Returns
/// JSON with generated prompt and metadata
pub async fn generate_defect_aware_prompt(
    task: String,
    context: String,
    summary_path: PathBuf,
) -> Result<Value> {
    debug_assert!(summary_path.exists(), "summary_path must exist: {}", summary_path.display());
    use crate::prompts::DefectAwarePromptGenerator;

    // Validate summary file exists
    if !summary_path.exists() {
        return Ok(json!({
            "status": "failed",
            "message": format!("Summary file not found: {}", summary_path.display()),
            "error": "FILE_NOT_FOUND",
            "help": "Run OIP analysis first: cd organizational-intelligence-plugin && cargo run -- analyze --org <org> --output /tmp/analysis.yaml && cargo run -- summarize --input /tmp/analysis.yaml --output summary.yaml --strip-pii"
        }));
    }

    // Load organizational intelligence
    let generator = match DefectAwarePromptGenerator::from_file(&summary_path) {
        Ok(gen) => gen,
        Err(e) => {
            return Ok(json!({
                "status": "failed",
                "message": format!("Failed to load summary: {}", e),
                "error": "PARSE_ERROR",
                "summary_path": summary_path.display().to_string()
            }));
        }
    };
    let prompt = generator.generate_prompt(&task, &context);
    Ok(json!({
        "status": "completed",
        "message": "Defect-aware prompt generated successfully",
        "result_type": "defect_aware_prompt",
        "prompt": prompt,
        "metadata": {
            "repositories_analyzed": generator.metadata.repositories_analyzed,
            "commits_analyzed": generator.metadata.commits_analyzed,
            "analysis_date": generator.metadata.analysis_date,
            "defect_patterns_count": generator.defect_patterns.len(),
            "prompt_length": prompt.len(),
            "task": task,
            "context": context
        }
    }))
}
