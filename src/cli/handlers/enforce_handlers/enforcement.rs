#![cfg_attr(coverage_nightly, coverage(off))]
//! Core enforcement loop orchestration and routing logic

use super::analysis::{
    run_complexity_analysis, run_coverage_analysis, run_dead_code_analysis,
    run_duplication_analysis, run_satd_analysis, run_tdg_analysis, AnalysisScope,
};
use super::config::EnforcementConfig;
use super::output::{
    emit_report, format_violations_output, handle_ci_mode_exit, output_result,
    print_enforcement_summary,
};
use super::states::{
    handle_analyzing_state, handle_complete_state, handle_refactoring_enforcement_state,
    handle_validating_enforcement_state, handle_violating_enforcement_state_proxy,
    violation_is_included,
};
use super::types::{
    EnforcementIterationResult, EnforcementLoopResult, EnforcementProgress, EnforcementResult,
    EnforcementState, QualityProfile, QualityViolation,
};
use crate::cli::colors as c;
use crate::cli::EnforceOutputFormat;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Handle special modes (list violations, validate only)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_special_modes(
    list_violations: bool,
    validate_only: bool,
    project_path: &PathBuf,
    profile: &QualityProfile,
    format: EnforceOutputFormat,
    ci_mode: bool,
    specific_file: Option<&PathBuf>,
    output: Option<&Path>,
    include_pattern: Option<&String>,
    exclude_pattern: Option<&String>,
) -> Result<Option<Result<()>>> {
    if list_violations {
        return Ok(Some(
            list_all_violations(
                project_path,
                profile,
                format,
                specific_file,
                output,
                include_pattern,
                exclude_pattern,
            )
            .await,
        ));
    }

    if validate_only {
        return Ok(Some(
            validate_current_state(
                project_path,
                profile,
                format,
                ci_mode,
                specific_file,
                output,
                include_pattern,
                exclude_pattern,
            )
            .await,
        ));
    }

    Ok(None)
}

/// Run the main enforcement loop - DEEPLY REFACTORED (complexity: ≤10)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn run_main_enforcement_loop(
    project_path: &PathBuf,
    profile: &QualityProfile,
    config: EnforcementConfig,
    output: Option<&Path>,
) -> Result<()> {
    let start_time = Instant::now();

    // Delegate entire loop logic to extracted function - COMPLEXITY NOW ≤10
    let loop_result = execute_main_loop(project_path, profile, &config, start_time, output).await?;

    finalize_enforcement_run(
        loop_result.final_score,
        loop_result.final_iteration,
        start_time.elapsed(),
        &config,
        loop_result.final_state,
    );

    Ok(())
}

/// Check if enforcement should continue
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn should_continue_enforcement(
    current_state: EnforcementState,
    iteration: u32,
    config: &EnforcementConfig,
    start_time: Instant,
) -> bool {
    if current_state == EnforcementState::Complete || iteration >= config.max_iterations {
        return false;
    }

    if let Some(max_seconds) = config.max_time {
        if start_time.elapsed().as_secs() > max_seconds {
            eprintln!("{}", c::warn("Time limit reached"));
            return false;
        }
    }

    true
}

/// Execute a single enforcement iteration
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn execute_enforcement_iteration(
    project_path: &PathBuf,
    profile: &QualityProfile,
    current_state: EnforcementState,
    config: &EnforcementConfig,
) -> Result<EnforcementResult> {
    run_enforcement_step(
        project_path,
        profile,
        current_state,
        config.single_file_mode,
        config.dry_run,
        config.apply_suggestions,
        config.specific_file.as_ref(),
        config.include_pattern.as_ref(),
        config.exclude_pattern.as_ref(),
    )
    .await
}

/// Check if should stop for target improvement
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn should_stop_for_target_improvement(
    target_improvement: Option<f32>,
    result_score: f64,
    current_score: f64,
) -> bool {
    if let Some(target_delta) = target_improvement {
        result_score >= current_score + f64::from(target_delta)
    } else {
        false
    }
}

/// Check improvement targets - extracted from `run_main_enforcement_loop` (complexity: ≤10)
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn check_improvement_targets(
    config: &EnforcementConfig,
    result_score: f64,
    current_score: f64,
) -> bool {
    if should_stop_for_target_improvement(config.target_improvement, result_score, current_score) {
        eprintln!("{}", c::pass("Target improvement achieved"));
        true
    } else {
        false
    }
}

/// Finalize enforcement run - extracted from `run_main_enforcement_loop` (complexity: ≤10)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn finalize_enforcement_run(
    current_score: f64,
    iteration: u32,
    elapsed: Duration,
    config: &EnforcementConfig,
    current_state: EnforcementState,
) {
    print_enforcement_summary(current_score, iteration, elapsed);
    handle_ci_mode_exit(config.ci_mode, current_state);
}

/// Handle single enforcement iteration - extracted from `run_main_enforcement_loop` (complexity: ≤10)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_enforcement_iteration(
    project_path: &PathBuf,
    profile: &QualityProfile,
    current_state: EnforcementState,
    config: &EnforcementConfig,
    iteration: u32,
    output: Option<&Path>,
) -> Result<EnforcementIterationResult> {
    eprintln!(
        "\n{} {}",
        c::label("Iteration"),
        c::number(&iteration.to_string())
    );

    let result =
        execute_enforcement_iteration(project_path, profile, current_state, config).await?;

    output_result(&result, config.format, config.show_progress, output)?;

    Ok(EnforcementIterationResult {
        iteration,
        state: result.state,
        score: result.score,
    })
}

/// Execute main enforcement loop - extracted from `run_main_enforcement_loop` (complexity: ≤10)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn execute_main_loop(
    project_path: &PathBuf,
    profile: &QualityProfile,
    config: &EnforcementConfig,
    start_time: Instant,
    output: Option<&Path>,
) -> Result<EnforcementLoopResult> {
    let mut current_state = EnforcementState::Analyzing;
    let mut iteration = 0;
    let mut current_score = 0.0;
    let mut previous: Option<(EnforcementState, f64)> = None;

    while should_continue_enforcement(current_state, iteration, config, start_time) {
        let loop_result = handle_enforcement_iteration(
            project_path,
            profile,
            current_state,
            config,
            iteration + 1,
            output,
        )
        .await?;

        iteration = loop_result.iteration;
        // The improvement check compared the new score against `current_score`
        // AFTER `current_score` had already been overwritten with it, i.e. the
        // score against itself, so `--target-improvement` could only ever fire
        // for a non-positive delta. Compare against the score we came in with.
        let score_before = current_score;
        current_state = loop_result.state;
        current_score = loop_result.score;

        if check_improvement_targets(config, loop_result.score, score_before) {
            break;
        }

        // Nothing in this loop edits the tree unless --apply-suggestions is on,
        // so a project that cannot converge re-measured the same unchanged tree
        // 100 times: `pmat enforce extreme` with no flags printed the identical
        // three-line summary a hundred times over 11 seconds for a 13-line
        // crate. An iteration that reproduces the previous verdict exactly has
        // demonstrated that iterating changes nothing; say so and stop.
        if previous.is_some_and(|(state, score)| {
            state == loop_result.state && (score - loop_result.score).abs() < f64::EPSILON
        }) {
            eprintln!(
                "{}",
                c::warn(&format!(
                    "no progress after {iteration} iterations (score unchanged at {:.2}); stopping",
                    loop_result.score
                ))
            );
            break;
        }
        previous = Some((loop_result.state, loop_result.score));

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    Ok(EnforcementLoopResult {
        final_iteration: iteration,
        final_state: current_state,
        final_score: current_score,
    })
}

/// Run a single enforcement step - REFACTORED (complexity: ≤10)
#[allow(clippy::too_many_arguments)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn run_enforcement_step(
    project_path: &PathBuf,
    profile: &QualityProfile,
    current_state: EnforcementState,
    single_file_mode: bool,
    dry_run: bool,
    apply_suggestions: bool,
    specific_file: Option<&PathBuf>,
    include_pattern: Option<&String>,
    exclude_pattern: Option<&String>,
) -> Result<EnforcementResult> {
    // Route to extracted state handlers - COMPLEXITY REDUCED FROM 62 TO ≤10
    match current_state {
        EnforcementState::Analyzing => {
            handle_analyzing_state(
                project_path,
                profile,
                single_file_mode,
                dry_run,
                specific_file,
                include_pattern,
                exclude_pattern,
            )
            .await
        }

        EnforcementState::Violating => {
            handle_violating_enforcement_state_proxy(
                project_path,
                profile,
                single_file_mode,
                dry_run,
                specific_file,
                apply_suggestions,
                include_pattern,
                exclude_pattern,
            )
            .await
        }

        EnforcementState::Refactoring => handle_refactoring_enforcement_state(0.7, specific_file),

        EnforcementState::Validating => {
            handle_validating_enforcement_state(
                project_path,
                profile,
                single_file_mode,
                dry_run,
                specific_file,
                include_pattern,
                exclude_pattern,
            )
            .await
        }

        EnforcementState::Complete => handle_complete_state(),
    }
}

/// List all violations in the project - REFACTORED (complexity: ≤10)
async fn list_all_violations(
    project_path: &Path,
    profile: &QualityProfile,
    format: EnforceOutputFormat,
    specific_file: Option<&PathBuf>,
    output: Option<&Path>,
    include_pattern: Option<&String>,
    exclude_pattern: Option<&String>,
) -> Result<()> {
    eprintln!("{}", c::header("Listing all quality violations..."));

    let scope = AnalysisScope::resolve(project_path, specific_file.map(PathBuf::as_path));
    if let AnalysisScope::SingleFile { module_dir, .. } = &scope {
        // Directory-walk phases cannot target a lone file
        eprintln!(
            "  {} Single-file mode: SATD/dead-code/duplication scoped to parent module {}",
            c::dim(">>"),
            module_dir.display()
        );
    }

    let mut all_violations: Vec<QualityViolation> = Vec::new();

    // Run all analyses using extracted functions - COMPLEXITY REDUCED FROM 48 TO ≤10
    eprintln!("  {} Analyzing complexity...", c::dim(">>"));
    let complexity_outcome =
        run_complexity_analysis(scope.walk_root(), profile, scope.single_file()).await?;
    all_violations.extend(complexity_outcome.violations);

    eprintln!("  {} Analyzing technical debt (SATD)...", c::dim(">>"));
    let satd_outcome = run_satd_analysis(scope.walk_root(), profile, scope.single_file()).await?;
    all_violations.extend(satd_outcome.violations);

    eprintln!("  {} Analyzing technical debt gradient...", c::dim(">>"));
    let tdg_outcome = run_tdg_analysis(scope.file_or_root(), profile).await?;
    all_violations.extend(tdg_outcome.violations);

    eprintln!("  {} Analyzing dead code...", c::dim(">>"));
    let dead_code_outcome = run_dead_code_analysis(scope.walk_root(), profile).await?;
    all_violations.extend(dead_code_outcome.violations);

    eprintln!("  {} Analyzing code duplication...", c::dim(">>"));
    let duplication_outcome = run_duplication_analysis(scope.walk_root(), profile).await?;
    all_violations.extend(duplication_outcome.violations);

    eprintln!("  {} Checking test coverage...", c::dim(">>"));
    let coverage_outcome = run_coverage_analysis(scope.walk_root(), profile).await?;
    all_violations.extend(coverage_outcome.violations);

    // Same filter, same helper as the state machine's: `--list-violations
    // --exclude 'src/*'` listed the excluded files too, because these six phases
    // never saw the patterns either.
    if include_pattern.is_some() || exclude_pattern.is_some() {
        let filter = crate::utils::file_filter::FileFilter::from_optional(
            &include_pattern.cloned(),
            &exclude_pattern.cloned(),
        )?;
        all_violations.retain(|v| violation_is_included(&filter, project_path, v));
    }

    eprintln!(
        "\n{} {} violations",
        c::label("Found"),
        c::number(&all_violations.len().to_string())
    );

    // Use extracted formatting function. `-o` is honoured here for the same
    // reason it is honoured for the iteration report: one flag, one meaning.
    let formatted_output = format_violations_output(&all_violations, profile, format)?;
    emit_report(&formatted_output, output)
}

/// Validate current state without making changes
async fn validate_current_state(
    project_path: &PathBuf,
    profile: &QualityProfile,
    format: EnforceOutputFormat,
    ci_mode: bool,
    specific_file: Option<&PathBuf>,
    output: Option<&Path>,
    include_pattern: Option<&String>,
    exclude_pattern: Option<&String>,
) -> Result<()> {
    eprintln!("{}", c::label("Validating current quality state..."));

    // Run the analysis step to get current state
    let result = run_enforcement_step(
        project_path,
        profile,
        EnforcementState::Analyzing,
        specific_file.is_some(), // single_file_mode
        true,                    // dry_run
        false,                   // apply_suggestions
        specific_file,
        include_pattern,
        exclude_pattern,
    )
    .await?;

    let passes = result.score >= result.target;
    let violations_count = result.violations.len();

    // Create summary result
    let validation_result = EnforcementResult {
        state: if passes {
            EnforcementState::Complete
        } else {
            EnforcementState::Violating
        },
        score: result.score,
        target: result.target,
        current_file: None,
        violations: result.violations,
        next_action: if passes {
            "none".to_string()
        } else {
            format!("fix_{violations_count}_violations")
        },
        progress: EnforcementProgress {
            files_completed: 0,
            files_remaining: 0,
            estimated_iterations: if passes {
                0
            } else {
                ((1.0 - result.score) * 10.0) as u32
            },
        },
    };

    output_result(&validation_result, format, false, output)?;

    if ci_mode && !passes {
        eprintln!("\n{}", c::fail("Quality validation failed!"));
        eprintln!(
            "   {} {}{:.2}{}/{}{:.2}{}",
            c::label("Score:"),
            c::BOLD_WHITE,
            result.score,
            c::RESET,
            c::DIM,
            result.target,
            c::RESET
        );
        eprintln!(
            "   {} {}",
            c::label("Violations:"),
            c::number(&validation_result.violations.len().to_string())
        );
        std::process::exit(1);
    }

    Ok(())
}
