#![cfg_attr(coverage_nightly, coverage(off))]
//! Main handler entry point for enforce commands

use super::config::{initialize_enforcement_environment, EnforcementConfig};
use super::enforcement::{handle_special_modes, run_main_enforcement_loop};
use super::output::print_enforcement_header;
use crate::cli::{EnforceCommands, EnforceOutputFormat};
use anyhow::Result;
use std::path::PathBuf;

/// Route enforce commands to appropriate handlers
///
/// # Errors
///
/// Returns an error if the operation fails
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn route_enforce_command(cmd: EnforceCommands) -> Result<()> {
    match cmd {
        EnforceCommands::Extreme {
            project_path,
            single_file_mode,
            dry_run,
            profile,
            show_progress,
            format,
            output,
            max_iterations,
            target_improvement,
            max_time,
            apply_suggestions,
            validate_only,
            list_violations,
            config,
            ci_mode,
            file,
            include,
            exclude,
            cache_dir,
            clear_cache,
        } => {
            handle_enforce_extreme(
                project_path,
                single_file_mode,
                dry_run,
                profile.to_string(),
                show_progress,
                format,
                output,
                max_iterations,
                target_improvement,
                max_time,
                apply_suggestions,
                validate_only,
                list_violations,
                config,
                ci_mode,
                file,
                include,
                exclude,
                cache_dir,
                clear_cache,
            )
            .await
        }
    }
}

/// Handle enforce extreme command
#[allow(clippy::too_many_arguments)]
async fn handle_enforce_extreme(
    project_path: PathBuf,
    single_file_mode: bool,
    dry_run: bool,
    profile_name: String,
    show_progress: bool,
    format: EnforceOutputFormat,
    output: Option<PathBuf>,
    max_iterations: u32,
    target_improvement: Option<f32>,
    max_time: Option<u64>,
    apply_suggestions: bool,
    validate_only: bool,
    list_violations: bool,
    config_path: Option<PathBuf>,
    ci_mode: bool,
    specific_file: Option<PathBuf>,
    include_pattern: Option<String>,
    exclude_pattern: Option<String>,
    cache_dir: Option<PathBuf>,
    clear_cache: bool,
) -> Result<()> {
    // `--max-iterations 0` ran the loop zero times and still exited 0, which for
    // `--format json` meant zero bytes on stdout: an unparseable document and a
    // success code for a consumer that asked for JSON. There is no verdict to
    // report without a single measurement, so the argument is refused up front.
    if max_iterations == 0 {
        anyhow::bail!(
            "--max-iterations must be at least 1: enforce cannot report a verdict without running a single iteration"
        );
    }

    print_enforcement_header(&project_path);

    let profile =
        initialize_enforcement_environment(&profile_name, config_path, &cache_dir, clear_cache)?;

    if let Some(result) = handle_special_modes(
        list_violations,
        validate_only,
        &project_path,
        &profile,
        format,
        ci_mode,
        specific_file.as_ref(),
        output.as_deref(),
        include_pattern.as_ref(),
        exclude_pattern.as_ref(),
    )
    .await?
    {
        return result;
    }

    let enforcement_config = EnforcementConfig {
        max_iterations,
        target_improvement,
        max_time,
        apply_suggestions,
        specific_file,
        include_pattern,
        exclude_pattern,
        single_file_mode,
        dry_run,
        show_progress,
        format,
        ci_mode,
    };

    run_main_enforcement_loop(
        &project_path,
        &profile,
        enforcement_config,
        output.as_deref(),
    )
    .await
}
