//! Quality and analysis command routing.
//!
//! Routes ProjectDiag, TestDiscovery, DebugFiveWhys, Oracle, PerfectionScore,
//! Spec, Localize, CudaTdg, DepsAudit, and Kaizen commands to their handlers.
#![cfg_attr(coverage_nightly, coverage(off))]

use super::CommandDispatcher;
use crate::cli::commands::Commands;
use crate::cli::handlers;

impl CommandDispatcher {
    /// Route quality and analysis commands (extracted to reduce route_command cognitive complexity)
    pub(super) async fn route_quality_command(command: Commands) -> anyhow::Result<()> {
        match command {
            Commands::ProjectDiag {
                path,
                format,
                category,
                failures_only,
                output,
                quiet,
            } => {
                let config = handlers::project_diag_handlers::ProjectDiagConfig {
                    path,
                    format,
                    category,
                    failures_only,
                    output,
                    quiet,
                };
                handlers::project_diag_handlers::handle_project_diag(config).await
            }
            Commands::TestDiscovery { command } => {
                handlers::test_discovery_handlers::handle_test_discovery_command(command).await
            }
            Commands::TestStability {
                path,
                runs,
                filter,
                format,
                output,
            } => {
                crate::cli::handlers::test_stability_handler::handle_test_stability(
                    &path,
                    runs,
                    filter.as_deref(),
                    &format,
                    output.as_deref(),
                )
                .await
            }
            Commands::DebugFiveWhys {
                issue,
                depth,
                format,
                output,
                path,
                context,
                auto_analyze,
            } => {
                crate::cli::handlers::five_whys_handlers::handle_debug(
                    &issue,
                    depth,
                    format,
                    output.as_deref(),
                    &path,
                    context.as_deref(),
                    auto_analyze,
                )
                .await
            }
            Commands::Oracle { command } => {
                crate::cli::handlers::oracle_handlers::handle_oracle_command(command).await
            }
            Commands::PerfectionScore {
                path,
                breakdown,
                target,
                format,
                output,
                fast,
            } => {
                handlers::perfection_score_handlers::handle_perfection_score(
                    &path,
                    breakdown,
                    target,
                    format,
                    output.as_deref(),
                    fast,
                )
                .await
            }
            Commands::Spec { command } => Self::handle_spec_command(command).await,
            Commands::Localize {
                passed_coverage,
                failed_coverage,
                passed_count,
                failed_count,
                formula,
                top_n,
                output,
                format,
            } => {
                crate::cli::handlers::localize_handlers::handle_localize(
                    &passed_coverage,
                    &failed_coverage,
                    passed_count,
                    failed_count,
                    &formula,
                    top_n,
                    output.as_deref(),
                    &format,
                )
                .await
            }
            Commands::CudaTdg {
                path,
                command,
                format,
                min_score,
                fail_on_p0,
                simd,
                wgpu,
                output,
                quiet,
            } => {
                let config = handlers::CudaTdgCommandConfig {
                    path,
                    command,
                    format,
                    min_score,
                    fail_on_p0,
                    simd,
                    wgpu,
                    output,
                    quiet,
                };
                handlers::handle_cuda_tdg_command(config).await
            }
            Commands::DepsAudit {
                path,
                format,
                all,
                pareto,
                sort_by,
            } => handlers::deps_audit_handlers::handle_deps_audit(
                &path, &format, all, pareto, &sort_by,
            ),
            Commands::Kaizen {
                path,
                dry_run,
                no_commit,
                no_issues,
                push,
                agent,
                max_agents,
                format,
                output,
                skip_clippy,
                skip_fmt,
                skip_comply,
                skip_github,
                skip_defects,
                cross_stack,
            } => {
                let config = handlers::kaizen_handler::KaizenConfig {
                    path,
                    dry_run,
                    commit: !no_commit,
                    create_issues: !no_issues,
                    push,
                    auto_agent: agent,
                    max_agents,
                    format,
                    output,
                    skip_clippy,
                    skip_fmt,
                    skip_comply,
                    skip_github,
                    skip_defects,
                    cross_stack,
                };
                handlers::kaizen_handler::handle_kaizen(config).await
            }
            _ => unreachable!("route_quality_command called with non-quality command"),
        }
    }
}
