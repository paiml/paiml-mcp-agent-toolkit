#![cfg_attr(coverage_nightly, coverage(off))]
//! Extended dispatch: demo, quality gate, reporting, and scoring commands

use super::super::CommandExecutor;
use crate::cli::Commands;
use anyhow::Result;

impl CommandExecutor {
    /// Execute demo, quality gate, reporting, and scoring commands
    pub(super) async fn execute_ext_scoring(&self, command: Commands) -> Result<()> {
        match command {
            Commands::Demo {
                path,
                url,
                repo,
                format,
                protocol,
                show_api,
                no_browser,
                port,
                cli,
                target_nodes,
                centrality_threshold,
                merge_threshold,
                debug,
                debug_output,
                skip_vendor,
                no_skip_vendor,
                max_line_length,
            } => {
                self.registry
                    .demo_handlers
                    .handle_demo(
                        self.server.clone(),
                        path,
                        url,
                        repo,
                        format,
                        protocol,
                        show_api,
                        no_browser,
                        port,
                        cli,
                        target_nodes,
                        centrality_threshold,
                        merge_threshold,
                        debug,
                        debug_output,
                        skip_vendor && !no_skip_vendor,
                        max_line_length,
                    )
                    .await
            }
            Commands::QualityGate {
                project_path,
                file,
                format,
                fail_on_violation,
                checks,
                max_dead_code,
                min_entropy,
                max_complexity_p99,
                include_provability,
                output,
                perf,
            } => {
                self.registry
                    .demo_handlers
                    .handle_quality_gate(
                        project_path,
                        file,
                        format,
                        fail_on_violation,
                        checks,
                        max_dead_code,
                        min_entropy,
                        max_complexity_p99,
                        include_provability,
                        output,
                        perf,
                    )
                    .await
            }
            Commands::Report {
                project_path,
                output_format,
                text,
                markdown,
                csv,
                include_visualizations,
                include_executive_summary,
                include_recommendations,
                analyses,
                confidence_threshold,
                output,
                perf,
            } => {
                crate::cli::handlers::enhanced_reporting_handlers::handle_generate_report(
                    project_path,
                    output_format,
                    text,
                    markdown,
                    csv,
                    include_visualizations,
                    include_executive_summary,
                    include_recommendations,
                    analyses,
                    confidence_threshold,
                    output,
                    perf,
                )
                .await
            }
            Commands::RepoScore {
                path,
                format,
                verbose,
                failures_only,
                output,
                update_badge,
                deep,
            } => {
                crate::cli::handlers::repo_score_handlers::handle_repo_score(
                    &path,
                    format,
                    verbose,
                    failures_only,
                    output.as_deref(),
                    update_badge,
                    deep,
                )
                .await
            }
            Commands::RustProjectScore {
                path,
                format,
                verbose,
                failures_only,
                output,
                full,
            } => {
                crate::cli::handlers::rust_project_score_handlers::handle_rust_project_score(
                    &path,
                    &format,
                    verbose,
                    failures_only,
                    output.as_deref(),
                    full,
                )
                .await
            }
            Commands::BrickScore {
                path,
                input,
                format,
                verbose,
                failures_only,
                threshold,
                output,
                hardware,
            } => {
                crate::cli::handlers::brick_score_handlers::handle_brick_score(
                    &path,
                    input.as_deref(),
                    &format,
                    verbose,
                    failures_only,
                    threshold,
                    output.as_deref(),
                    hardware.as_deref(),
                )
                .await
            }
            Commands::PopperScore {
                path,
                format,
                verbose,
                failures_only,
                output,
            } => {
                crate::cli::handlers::popper_score_handlers::handle_popper_score(
                    &path,
                    &format,
                    verbose,
                    failures_only,
                    output.as_deref(),
                )
                .await
            }
            Commands::DemoScore {
                path,
                format,
                verbose,
                failures_only,
                output,
            } => {
                crate::cli::handlers::demo_score_handlers::handle_demo_score(
                    &path,
                    &format,
                    verbose,
                    failures_only,
                    output.as_deref(),
                )
                .await
            }
            _ => unreachable!("execute_ext_scoring called with non-scoring command"),
        }
    }
}
