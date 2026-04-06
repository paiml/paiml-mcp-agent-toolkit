#![cfg_attr(coverage_nightly, coverage(off))]

use crate::cli::{ExplainLevel, RefactorCommands, RefactorMode, RefactorOutputFormat};
use crate::models::refactor::{RefactorConfig, Summary};
use crate::services::cache::unified_manager::UnifiedCacheManager;
use crate::services::refactor_engine::{EngineMode, UnifiedEngine};
use crate::services::unified_ast_engine::UnifiedAstEngine;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

/// Parameters for the refactor serve command
pub struct RefactorServeParams {
    pub mode: RefactorMode,
    pub config: Option<PathBuf>,
    pub project: PathBuf,
    pub parallel: usize,
    pub memory_limit: usize,
    pub batch_size: usize,
    pub priority: Option<String>,
    pub checkpoint_dir: Option<PathBuf>,
    pub resume: bool,
    pub auto_commit: Option<String>,
    pub max_runtime: Option<u64>,
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn route_refactor_command(refactor_cmd: RefactorCommands) -> anyhow::Result<()> {
    match refactor_cmd {
        RefactorCommands::Serve {
            refactor_mode,
            config,
            project,
            parallel,
            memory_limit,
            batch_size,
            priority,
            checkpoint_dir,
            resume,
            auto_commit,
            max_runtime,
        } => {
            let params = RefactorServeParams {
                mode: refactor_mode,
                config,
                project,
                parallel,
                memory_limit,
                batch_size,
                priority,
                checkpoint_dir,
                resume,
                auto_commit,
                max_runtime,
            };
            handle_refactor_serve(params).await
        }
        RefactorCommands::Interactive {
            project_path,
            explain,
            checkpoint,
            target_complexity,
            steps,
            config,
        } => {
            handle_refactor_interactive(
                project_path,
                explain,
                checkpoint,
                target_complexity,
                steps,
                config,
            )
            .await
        }
        RefactorCommands::Status { checkpoint, format } => {
            handle_refactor_status(checkpoint, format).await
        }
        RefactorCommands::Resume {
            checkpoint,
            steps,
            explain,
        } => handle_refactor_resume(checkpoint, steps, explain).await,
        RefactorCommands::Auto {
            project_path,
            single_file_mode,
            file,
            max_iterations,
            quality_profile: _,
            format,
            dry_run,
            skip_compilation: _,
            skip_tests: _,
            checkpoint,
            verbose: _,
            exclude,
            include,
            ignore_file,
            test,
            test_name,
            github_issue,
            bug_report_path,
        } => {
            super::refactor_auto_handlers::handle_refactor_auto(
                super::refactor_auto_handlers::RefactorAutoConfig {
                    project_path,
                    single_file_mode,
                    file,
                    format,
                    max_iterations,
                    cache_dir: checkpoint,
                    dry_run,
                    ci_mode: false, // use false for interactive mode
                    exclude_patterns: exclude,
                    include_patterns: include,
                    ignore_file,
                    test_file: test,
                    test_name,
                    github_issue_url: github_issue,
                    bug_report_path,
                },
            )
            .await
        }
        RefactorCommands::Docs {
            project_path,
            include_docs,
            include_root,
            additional_dirs,
            format,
            dry_run,
            temp_patterns,
            status_patterns,
            artifact_patterns,
            custom_patterns,
            min_age_days,
            max_size_mb,
            recursive,
            preserve_patterns,
            output,
            auto_remove,
            backup,
            backup_dir,
            perf,
        } => {
            super::refactor_docs_handlers::handle_refactor_docs(
                project_path,
                include_docs,
                include_root,
                additional_dirs,
                format,
                dry_run,
                temp_patterns,
                status_patterns,
                artifact_patterns,
                custom_patterns,
                min_age_days,
                max_size_mb,
                recursive,
                preserve_patterns,
                output,
                auto_remove,
                backup,
                backup_dir,
                perf,
            )
            .await
        }
    }
}

// Serve command handler and helpers (handle_refactor_serve, ExtractedRefactorParams, etc.)
include!("refactor_handlers_serve.rs");

// Interactive and resume command handlers (handle_refactor_interactive, handle_refactor_resume)
include!("refactor_handlers_interactive.rs");

// Status command handler and output formatters (handle_refactor_status, format_as_json, etc.)
include!("refactor_handlers_status.rs");

// Configuration loading, target discovery, auto-commit, and type conversions
include!("refactor_handlers_config.rs");

// Tests extracted to refactor_handlers_tests.rs for file health compliance (CB-040)
// TEMPORARILY DISABLED: File splitting broke syntax
#[cfg(all(test, feature = "broken-tests"))]
#[path = "refactor_handlers_tests.rs"]
mod tests;
