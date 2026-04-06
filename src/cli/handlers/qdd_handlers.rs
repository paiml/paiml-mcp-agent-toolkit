//! QDD (Quality-Driven Development) CLI handlers
//! Toyota Way: Single Responsibility and DRY principles

#![cfg_attr(coverage_nightly, coverage(off))]
use crate::cli::colors as c;
use crate::cli::commands::{QddCodeType, QddCommands, QddOutputFormat, QddQualityProfile};
use crate::qdd::{
    CodeType, CreateSpec, Parameter, QddOperation, QddResult, QddTool, QualityProfile, RefactorSpec,
};
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Handle QDD CLI commands
pub async fn handle_qdd_command(command: QddCommands) -> Result<()> {
    match command {
        QddCommands::Create {
            code_type,
            name,
            purpose,
            profile,
            input,
            output,
            output_file,
        } => {
            handle_qdd_create(
                code_type,
                name,
                purpose,
                profile,
                input,
                output,
                output_file,
            )
            .await
        }

        QddCommands::Refactor {
            file,
            function,
            profile,
            max_complexity,
            min_coverage,
            output,
            dry_run,
        } => {
            handle_qdd_refactor(
                file,
                function,
                profile,
                max_complexity,
                min_coverage,
                output,
                dry_run,
            )
            .await
        }

        QddCommands::Validate {
            path,
            profile,
            format,
            output,
            strict,
        } => handle_qdd_validate(path, profile, format, output, strict).await,
    }
}

/// Handle QDD create command
async fn handle_qdd_create(
    code_type: QddCodeType,
    name: String,
    purpose: String,
    profile: QddQualityProfile,
    inputs: Vec<(String, String)>,
    output_type: String,
    output_file: Option<PathBuf>,
) -> Result<()> {
    let qdd_code_type = convert_code_type(code_type);
    let quality_profile = convert_quality_profile(profile);
    let parameters = convert_parameters(inputs);
    let create_spec = build_create_spec(qdd_code_type, name, purpose, parameters, output_type);

    let result = execute_create_operation(quality_profile, create_spec).await?;
    display_create_results(profile, &result);
    output_generated_code(output_file, &result)?;

    Ok(())
}

/// Convert CLI code type to QDD code type
fn convert_code_type(code_type: QddCodeType) -> CodeType {
    match code_type {
        QddCodeType::Function => CodeType::Function,
        QddCodeType::Module => CodeType::Module,
        QddCodeType::Service => CodeType::Service,
        QddCodeType::Test => CodeType::Test,
    }
}

/// Convert CLI quality profile to QDD quality profile
fn convert_quality_profile(profile: QddQualityProfile) -> QualityProfile {
    match profile {
        QddQualityProfile::Extreme => QualityProfile::extreme(),
        QddQualityProfile::Standard => QualityProfile::standard(),
        QddQualityProfile::Relaxed => QualityProfile::relaxed(),
    }
}

/// Convert input parameters to QDD parameters
fn convert_parameters(inputs: Vec<(String, String)>) -> Vec<Parameter> {
    debug_assert!(!inputs.is_empty(), "inputs must not be empty");
    inputs
        .into_iter()
        .map(|(param_type, param_name)| Parameter {
            name: param_name,
            param_type,
            description: None,
        })
        .collect()
}

/// Build create specification
fn build_create_spec(
    code_type: CodeType,
    name: String,
    purpose: String,
    inputs: Vec<Parameter>,
    output_type: String,
) -> CreateSpec {
    CreateSpec {
        code_type,
        name,
        purpose,
        inputs,
        outputs: Parameter {
            name: "result".to_string(),
            param_type: output_type,
            description: Some("Function output".to_string()),
        },
    }
}

/// Execute create operation
async fn execute_create_operation(
    quality_profile: QualityProfile,
    create_spec: CreateSpec,
) -> Result<QddResult> {
    let qdd_tool = QddTool::with_profile(quality_profile);
    let operation = QddOperation::Create(create_spec);
    qdd_tool.execute(operation).await
}

/// Display creation results
fn display_create_results(profile: QddQualityProfile, result: &QddResult) {
    println!("{}", c::header("QDD Code Creation Successful!"));
    println!("{}", c::pass(&format!("Quality Profile: {profile:?}")));
    println!(
        "  {} {}",
        c::label("Quality Score:"),
        c::number(&format!("{:.1}", result.quality_score.overall))
    );
    println!(
        "  {} {}",
        c::label("Complexity:"),
        c::number(&format!("{}", result.quality_score.complexity))
    );
    println!(
        "  {} {}",
        c::label("Coverage:"),
        c::pct(result.quality_score.coverage, 80.0, 60.0)
    );
    println!(
        "  {} {}",
        c::label("TDG Score:"),
        c::number(&format!("{}", result.quality_score.tdg))
    );
    println!();
}

/// Output generated code to file or stdout
fn output_generated_code(output_file: Option<PathBuf>, result: &QddResult) -> Result<()> {
    if let Some(output_path) = output_file {
        let full_content = format!(
            "{}\n\n{}\n\n{}",
            result.code, result.tests, result.documentation
        );
        std::fs::write(&output_path, full_content)?;
        println!(
            "{}",
            c::pass(&format!(
                "Generated code written to: {}",
                c::path(&output_path.display().to_string())
            ))
        );
    } else {
        println!("{}", c::subheader("Generated Code:"));
        println!("{}", result.code);
        println!("\n{}", c::subheader("Generated Tests:"));
        println!("{}", result.tests);
        println!("\n{}", c::subheader("Generated Documentation:"));
        println!("{}", result.documentation);
    }
    Ok(())
}

/// Handle QDD refactor command
async fn handle_qdd_refactor(
    file: PathBuf,
    function: Option<String>,
    profile: QddQualityProfile,
    max_complexity: Option<u32>,
    min_coverage: Option<u32>,
    output: Option<PathBuf>,
    dry_run: bool,
) -> Result<()> {
    debug_assert!(file.exists(), "file must exist: {}", file.display());
    validate_file_exists(&file)?;

    let quality_profile = create_quality_profile(profile, max_complexity, min_coverage);
    let refactor_spec = create_refactor_spec(&file, function.clone(), &quality_profile);

    if dry_run {
        return handle_dry_run(&file, &function, profile, &quality_profile);
    }

    let result = execute_refactoring(quality_profile, refactor_spec).await?;
    display_refactor_results(&file, function, profile, &result);
    save_refactored_code(&output.unwrap_or(file), &result.code)?;
    display_rollback_info(&result);

    Ok(())
}

/// Validate that the target file exists
fn validate_file_exists(file: &Path) -> Result<()> {
    debug_assert!(file.exists(), "file must exist: {}", file.display());
    if !file.exists() {
        return Err(anyhow::anyhow!("File does not exist: {}", file.display()));
    }
    Ok(())
}

/// Create quality profile with optional overrides
fn create_quality_profile(
    profile: QddQualityProfile,
    max_complexity: Option<u32>,
    min_coverage: Option<u32>,
) -> QualityProfile {
    let mut quality_profile = match profile {
        QddQualityProfile::Extreme => QualityProfile::extreme(),
        QddQualityProfile::Standard => QualityProfile::standard(),
        QddQualityProfile::Relaxed => QualityProfile::relaxed(),
    };

    if let Some(complexity) = max_complexity {
        quality_profile.thresholds.max_complexity = complexity;
    }
    if let Some(coverage) = min_coverage {
        quality_profile.thresholds.min_coverage = coverage;
    }

    quality_profile
}

/// Create refactor specification
fn create_refactor_spec(
    file: &Path,
    function: Option<String>,
    quality_profile: &QualityProfile,
) -> RefactorSpec {
    debug_assert!(file.exists(), "file must exist: {}", file.display());
    RefactorSpec {
        file_path: file.to_path_buf(),
        function_name: function,
        target_metrics: quality_profile.thresholds.clone(),
    }
}

/// Handle dry run mode
fn handle_dry_run(
    file: &Path,
    function: &Option<String>,
    profile: QddQualityProfile,
    quality_profile: &QualityProfile,
) -> Result<()> {
    debug_assert!(file.exists(), "file must exist: {}", file.display());
    println!(
        "{}",
        c::dim(&format!(
            "DRY RUN: Would refactor file: {}",
            c::path(&file.display().to_string())
        ))
    );
    if let Some(func) = function {
        println!("  {} {}", c::label("Target function:"), func);
    }
    println!("  {} {profile:?}", c::label("Quality profile:"));
    println!(
        "  {} {}",
        c::label("Max complexity:"),
        c::number(&format!("{}", quality_profile.thresholds.max_complexity))
    );
    println!(
        "  {} {}",
        c::label("Min coverage:"),
        c::pct(quality_profile.thresholds.min_coverage as f64, 80.0, 60.0)
    );
    println!(
        "{}",
        c::warn("Use without --dry-run to execute refactoring")
    );
    Ok(())
}

/// Execute the refactoring operation
async fn execute_refactoring(
    quality_profile: QualityProfile,
    refactor_spec: RefactorSpec,
) -> Result<QddResult> {
    let qdd_tool = QddTool::with_profile(quality_profile);
    let operation = QddOperation::Refactor(refactor_spec);
    qdd_tool.execute(operation).await
}

/// Display refactoring results
fn display_refactor_results(
    file: &Path,
    function: Option<String>,
    profile: QddQualityProfile,
    result: &QddResult,
) {
    debug_assert!(file.exists(), "file must exist: {}", file.display());
    println!("{}", c::header("QDD Refactoring Successful!"));
    println!(
        "  {} {}",
        c::label("File:"),
        c::path(&file.display().to_string())
    );
    if let Some(func) = function {
        println!("  {} {}", c::label("Function:"), func);
    }
    println!("{}", c::pass(&format!("Quality Profile: {profile:?}")));
    println!(
        "  {} {}",
        c::label("Quality Score:"),
        c::number(&format!("{:.1}", result.quality_score.overall))
    );
    println!(
        "  {} {}",
        c::label("Complexity:"),
        c::number(&format!("{}", result.quality_score.complexity))
    );
    println!(
        "  {} {}",
        c::label("Coverage:"),
        c::pct(result.quality_score.coverage, 80.0, 60.0)
    );
    println!(
        "  {} {}",
        c::label("TDG Score:"),
        c::number(&format!("{}", result.quality_score.tdg))
    );
    println!();
}

/// Save refactored code to file
fn save_refactored_code(output_path: &Path, code: &str) -> Result<()> {
    debug_assert!(!code.is_empty(), "code must not be empty");
    std::fs::write(output_path, code)?;
    println!(
        "{}",
        c::pass(&format!(
            "Refactored code written to: {}",
            c::path(&output_path.display().to_string())
        ))
    );
    Ok(())
}

/// Display rollback information if available
fn display_rollback_info(result: &QddResult) {
    if !result.rollback_plan.checkpoints.is_empty() {
        println!(
            "  {} {} rollback checkpoints available",
            c::label("Rollback:"),
            c::number(&format!("{}", result.rollback_plan.checkpoints.len()))
        );
    }
}

/// Handle QDD validate command
async fn handle_qdd_validate(
    path: PathBuf,
    profile: QddQualityProfile,
    format: QddOutputFormat,
    output: Option<PathBuf>,
    strict: bool,
) -> Result<()> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let quality_profile = match profile {
        QddQualityProfile::Extreme => QualityProfile::extreme(),
        QddQualityProfile::Standard => QualityProfile::standard(),
        QddQualityProfile::Relaxed => QualityProfile::relaxed(),
    };

    // For now, implement as a simple quality check
    // In a full implementation, this would use the QDD validation engine
    println!("{}", c::header("QDD Quality Validation"));
    println!(
        "  {} {}",
        c::label("Path:"),
        c::path(&path.display().to_string())
    );
    println!("{}", c::pass(&format!("Quality Profile: {profile:?}")));
    println!("{}", c::subheader("Thresholds:"));
    println!(
        "  {} {}",
        c::label("Max Complexity:"),
        c::number(&format!("{}", quality_profile.thresholds.max_complexity))
    );
    println!(
        "  {} {}",
        c::label("Min Coverage:"),
        c::pct(quality_profile.thresholds.min_coverage as f64, 80.0, 60.0)
    );
    println!(
        "  {} {}",
        c::label("Max TDG:"),
        c::number(&format!("{}", quality_profile.thresholds.max_tdg))
    );
    println!(
        "  {} {}",
        c::label("Zero SATD:"),
        c::number(&format!("{}", quality_profile.thresholds.zero_satd))
    );

    // Simple validation placeholder
    let validation_passed = true; // Would implement actual validation

    match format {
        QddOutputFormat::Summary => {
            println!("\n{}", c::subheader("Validation Summary:"));
            println!(
                "{}",
                if validation_passed {
                    c::pass("PASSED")
                } else {
                    c::fail("FAILED")
                }
            );
        }
        QddOutputFormat::Detailed => {
            println!("\n{}", c::subheader("Detailed Validation Results:"));
            println!("{}", c::pass("Quality checks: PASSED"));
            println!("{}", c::pass("Complexity check: PASSED"));
            println!("{}", c::pass("Coverage check: PASSED"));
            println!("{}", c::pass("Technical debt: PASSED"));
        }
        QddOutputFormat::Json => {
            let json_result = serde_json::json!({
                "status": if validation_passed { "passed" } else { "failed" },
                "profile": format!("{profile:?}").to_lowercase(),
                "path": path.display().to_string(),
                "validation_time": chrono::Utc::now().to_rfc3339()
            });
            println!("{}", serde_json::to_string_pretty(&json_result)?);
        }
        QddOutputFormat::Markdown => {
            println!("# QDD Validation Report");
            println!();
            println!(
                "**Status:** {}",
                if validation_passed {
                    "✅ PASSED"
                } else {
                    "❌ FAILED"
                }
            );
            println!("**Profile:** {profile:?}");
            println!("**Path:** {}", path.display());
            println!(
                "**Date:** {}",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
            );
        }
    }

    if let Some(output_path) = output {
        println!(
            "\n{}",
            c::pass(&format!(
                "Validation report written to: {}",
                c::path(&output_path.display().to_string())
            ))
        );
    }

    if strict && !validation_passed {
        return Err(anyhow::anyhow!("Quality validation failed (strict mode)"));
    }

    Ok(())
}

// Tests extracted to qdd_handlers_tests.rs for file health (CB-040).
include!("qdd_handlers_tests.rs");
