//! QDD (Quality-Driven Development) CLI handlers
//! Toyota Way: Single Responsibility and DRY principles

#![cfg_attr(coverage_nightly, coverage(off))]
use crate::cli::colors as c;
use crate::cli::commands::{QddCodeType, QddCommands, QddOutputFormat, QddQualityProfile};
use crate::qdd::{
    CodeType, CreateSpec, Parameter, QddOperation, QddResult, QddTool, QualityProfile, RefactorSpec,
};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Handle QDD CLI commands
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    // `qdd validate -p /does/not/exist.rs` printed "✓ PASSED" and exited 0.
    crate::cli::ensure_analysis_path_exists(&path)?;

    let quality_profile = match profile {
        QddQualityProfile::Extreme => QualityProfile::extreme(),
        QddQualityProfile::Standard => QualityProfile::standard(),
        QddQualityProfile::Relaxed => QualityProfile::relaxed(),
    };
    let is_json = matches!(format, QddOutputFormat::Json);

    // JSON mode must keep stdout pure (jq-parseable): header goes to humans only
    if !is_json {
        print_validation_header(&path, profile, &quality_profile);
    }

    // This used to be `let validation_passed = true; // Would implement actual
    // validation`, and the Detailed arm printed four hardcoded PASSED lines with
    // no check behind any of them — so every input passed, including a path that
    // did not exist and this repository, whose own printed thresholds
    // ("Max Complexity: 10, Zero SATD: true") it violates. The verdict is now
    // derived from checks that actually run, and the two thresholds this command
    // cannot measure say so instead of passing.
    let outcome = run_validation_checks(&path, &quality_profile).await;
    let validation_passed = outcome.passed();

    match format {
        QddOutputFormat::Summary => {
            println!("\n{}", c::subheader("Validation Summary:"));
            println!("{}", render_status(&outcome));
            for (name, check) in &outcome.checks {
                println!("  {} {}", c::label(&format!("{name}:")), check.describe());
            }
        }
        QddOutputFormat::Detailed => {
            println!("\n{}", c::subheader("Detailed Validation Results:"));
            for (name, check) in &outcome.checks {
                println!("{}", check.render(name));
            }
            println!("{}", render_status(&outcome));
        }
        QddOutputFormat::Json => {
            let json_result = build_validation_json(&outcome, profile, &path);
            println!("{}", serde_json::to_string_pretty(&json_result)?);
        }
        QddOutputFormat::Markdown => {
            println!("# QDD Validation Report");
            println!();
            println!("**Status:** {}", markdown_status(&outcome));
            println!("**Profile:** {profile:?}");
            println!("**Path:** {}", path.display());
            println!(
                "**Date:** {}",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
            );
            println!();
            for (name, check) in &outcome.checks {
                println!("- **{name}:** {}", check.describe());
            }
        }
    }

    if let Some(output_path) = output {
        // Write the report before claiming it was written
        let report =
            serde_json::to_string_pretty(&build_validation_json(&outcome, profile, &path))?;
        std::fs::write(&output_path, report)
            .with_context(|| format!("Failed to write report: {}", output_path.display()))?;
        let message = c::pass(&format!(
            "Validation report written to: {}",
            c::path(&output_path.display().to_string())
        ));
        if is_json {
            eprintln!("{message}");
        } else {
            println!("\n{message}");
        }
    }

    if strict && !validation_passed {
        return Err(anyhow::anyhow!(
            "Quality validation did not pass (strict mode): {}",
            outcome.strict_reason()
        ));
    }

    Ok(())
}

/// The result of one threshold this command claims to enforce.
#[derive(Debug, Clone, PartialEq, Eq)]
enum CheckOutcome {
    Passed(String),
    Failed(String),
    /// The threshold is printed by the header but nothing here measures it.
    /// An unmeasured check is never a pass — see the `cuda-tdg` "not measured"
    /// reports for the same rule.
    NotMeasured(String),
}

impl CheckOutcome {
    fn describe(&self) -> String {
        match self {
            Self::Passed(detail) | Self::Failed(detail) | Self::NotMeasured(detail) => {
                detail.clone()
            }
        }
    }

    fn verdict(&self) -> &'static str {
        match self {
            Self::Passed(_) => "passed",
            Self::Failed(_) => "failed",
            Self::NotMeasured(_) => "not measured",
        }
    }

    fn render(&self, name: &str) -> String {
        let line = format!(
            "{name}: {} ({})",
            self.verdict().to_uppercase(),
            self.describe()
        );
        match self {
            Self::Passed(_) => c::pass(&line),
            Self::Failed(_) => c::fail(&line),
            Self::NotMeasured(_) => c::warn(&line),
        }
    }
}

/// Every check this run performed, in the order the header prints them.
struct ValidationOutcome {
    checks: Vec<(&'static str, CheckOutcome)>,
}

impl ValidationOutcome {
    /// `failed` beats `incomplete` beats `passed`: a run that could not measure
    /// a threshold it prints must not report a pass.
    fn status(&self) -> &'static str {
        if self.has(|c| matches!(c, CheckOutcome::Failed(_))) {
            "failed"
        } else if self.has(|c| matches!(c, CheckOutcome::NotMeasured(_))) {
            "incomplete"
        } else {
            "passed"
        }
    }

    fn has(&self, pred: impl Fn(&CheckOutcome) -> bool) -> bool {
        self.checks.iter().any(|(_, c)| pred(c))
    }

    fn passed(&self) -> bool {
        self.status() == "passed"
    }

    fn strict_reason(&self) -> String {
        self.checks
            .iter()
            .filter(|(_, c)| !matches!(c, CheckOutcome::Passed(_)))
            .map(|(name, c)| format!("{name} {}: {}", c.verdict(), c.describe()))
            .collect::<Vec<_>>()
            .join("; ")
    }

    fn violations(&self) -> Vec<serde_json::Value> {
        self.checks
            .iter()
            .filter(|(_, c)| matches!(c, CheckOutcome::Failed(_)))
            .map(|(name, c)| serde_json::json!({ "check": name, "detail": c.describe() }))
            .collect()
    }

    fn unmeasured(&self) -> Vec<serde_json::Value> {
        self.checks
            .iter()
            .filter(|(_, c)| matches!(c, CheckOutcome::NotMeasured(_)))
            .map(|(name, c)| serde_json::json!({ "check": name, "reason": c.describe() }))
            .collect()
    }
}

fn render_status(outcome: &ValidationOutcome) -> String {
    match outcome.status() {
        "passed" => c::pass("PASSED"),
        "failed" => c::fail("FAILED"),
        other => c::warn(&other.to_uppercase()),
    }
}

fn markdown_status(outcome: &ValidationOutcome) -> &'static str {
    match outcome.status() {
        "passed" => "✅ PASSED",
        "failed" => "❌ FAILED",
        _ => "⚠️ INCOMPLETE (some thresholds were not measured)",
    }
}

/// Run the checks behind the thresholds the header prints.
async fn run_validation_checks(path: &Path, profile: &QualityProfile) -> ValidationOutcome {
    let thresholds = &profile.thresholds;
    ValidationOutcome {
        checks: vec![
            ("complexity", check_complexity(path, thresholds.max_complexity).await),
            ("technical debt", check_satd(path, thresholds.zero_satd).await),
            (
                "coverage",
                CheckOutcome::NotMeasured(format!(
                    "min {}% required; coverage needs an instrumented test run (cargo llvm-cov), which this command does not perform",
                    thresholds.min_coverage
                )),
            ),
            (
                "tdg",
                CheckOutcome::NotMeasured(format!(
                    "max {} allowed; run `pmat tdg` — this command does not compute TDG",
                    thresholds.max_tdg
                )),
            ),
        ],
    }
}

/// Source files this command can analyse under `path`.
fn collect_analysable_files(path: &Path) -> Vec<PathBuf> {
    use crate::cli::language_analyzer::Language;

    let candidates = if path.is_file() {
        vec![path.to_path_buf()]
    } else {
        crate::services::file_discovery::ProjectFileDiscovery::new(path.to_path_buf())
            .discover_files()
            .unwrap_or_default()
    };

    candidates
        .into_iter()
        .filter(|p| {
            !matches!(
                Language::from_path(p),
                Language::Unknown | Language::Markdown | Language::Yaml
            )
        })
        .collect()
}

/// Worst cyclomatic complexity under `path` against the profile threshold.
async fn check_complexity(path: &Path, max_complexity: u32) -> CheckOutcome {
    let files = collect_analysable_files(path);
    let mut worst: Option<(String, u16)> = None;
    let mut analyzed = 0usize;

    for file in &files {
        let Ok(metrics) =
            crate::services::complexity::analyze_file_complexity_uncached(file, None).await
        else {
            continue;
        };
        analyzed += 1;
        for func in &metrics.functions {
            if worst
                .as_ref()
                .is_none_or(|(_, c)| func.metrics.cyclomatic > *c)
            {
                worst = Some((
                    format!("{}::{}", metrics.path, func.name),
                    func.metrics.cyclomatic,
                ));
            }
        }
    }

    match worst {
        // Nothing read means nothing measured; a clean pass over zero files is
        // the fabrication this whole command was guilty of.
        None => CheckOutcome::NotMeasured(format!(
            "no functions were read under {} ({analyzed} file(s) analysed)",
            path.display()
        )),
        Some((name, cyclomatic)) if u32::from(cyclomatic) > max_complexity => CheckOutcome::Failed(
            format!("{name} has cyclomatic complexity {cyclomatic}, over the limit of {max_complexity} ({analyzed} file(s) analysed)"),
        ),
        Some((name, cyclomatic)) => CheckOutcome::Passed(format!(
            "worst function {name} at cyclomatic {cyclomatic}, within {max_complexity} ({analyzed} file(s) analysed)"
        )),
    }
}

/// Self-admitted technical debt against the profile's `zero_satd` threshold.
async fn check_satd(path: &Path, zero_satd: bool) -> CheckOutcome {
    use crate::services::satd_detector::SATDDetector;

    if !zero_satd {
        return CheckOutcome::Passed("this profile does not require zero SATD".to_string());
    }

    let detector = SATDDetector::new();
    let debts = if path.is_file() {
        match std::fs::read_to_string(path) {
            Ok(content) => detector.extract_from_content(&content, path).ok(),
            Err(_) => None,
        }
    } else {
        detector.analyze_directory(path).await.ok()
    };

    match debts {
        None => CheckOutcome::NotMeasured(format!("could not scan {} for SATD", path.display())),
        Some(debts) if debts.is_empty() => {
            CheckOutcome::Passed("no self-admitted technical debt found".to_string())
        }
        Some(debts) => {
            let first = debts
                .first()
                .map(|d| format!("{}:{}", d.file.display(), d.line))
                .unwrap_or_default();
            CheckOutcome::Failed(format!(
                "{} self-admitted debt marker(s), first at {first}",
                debts.len()
            ))
        }
    }
}

/// Print the validation header and thresholds (human formats only)
fn print_validation_header(
    path: &Path,
    profile: QddQualityProfile,
    quality_profile: &QualityProfile,
) {
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
}

/// Build the JSON payload for validation results (stdout in JSON mode is this payload only)
///
/// The payload used to be `{status, profile, path, validation_time}` with no
/// field able to carry a violation — status was always "passed" and there was
/// nowhere for a failure to appear even if one had been found. `checks`,
/// `violations` and `not_measured` make the verdict auditable.
fn build_validation_json(
    outcome: &ValidationOutcome,
    profile: QddQualityProfile,
    path: &Path,
) -> serde_json::Value {
    serde_json::json!({
        "status": outcome.status(),
        "profile": format!("{profile:?}").to_lowercase(),
        "path": path.display().to_string(),
        "checks": outcome.checks.iter().map(|(name, check)| serde_json::json!({
            "check": name,
            "result": check.verdict(),
            "detail": check.describe(),
        })).collect::<Vec<_>>(),
        "violations": outcome.violations(),
        "not_measured": outcome.unmeasured(),
        "validation_time": chrono::Utc::now().to_rfc3339()
    })
}

// Tests extracted to qdd_handlers_tests.rs for file health (CB-040).
include!("qdd_handlers_tests.rs");
