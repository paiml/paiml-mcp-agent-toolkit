//! CommandDispatcher Tests
//!
//! Extracted from command_dispatcher.rs for file health compliance (CB-040).
//! Split into include files for file health compliance (CB-040).

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use crate::cli::command_dispatcher::CommandDispatcher;
    use crate::cli::commands::{Commands, ScaffoldCommands};
    // `demo` is not in Cargo.toml's `default` feature list and is not built by
    // the acceptance_cmd this ticket runs; it IS built by CI's `full`
    // feature-matrix leg (.github/workflows/feature-matrix.yml). Gated to
    // match the tests below that use it (tests_metric_and_demo.rs,
    // tests_config_extended.rs), which mirror demo_commands.rs's own gate.
    #[cfg(feature = "demo")]
    use crate::cli::DemoProtocol;
    use crate::services::rich_reporter::OutputFormat;
    use crate::stateless_server::StatelessTemplateServer;
    use std::path::PathBuf;
    use std::sync::Arc;

    fn create_test_server() -> Arc<StatelessTemplateServer> {
        Arc::new(StatelessTemplateServer::new().expect("internal error"))
    }

    // --- Basic command routing tests (Generate, List, Scaffold, QualityGate, Report, Config) ---
    // --- Test config creation, performance summary, write results ---
    include!("tests_command_routing.rs");

    // --- Metric recommendations, demo protocol conversion, demo args creation ---
    include!("tests_metric_and_demo.rs");

    // --- Scaffold/memory/cache routing, quality gate check types ---
    include!("tests_scaffold_quality_gate.rs");

    // --- Report format variants, show/record metrics, metric edge cases ---
    include!("tests_report_and_metrics.rs");

    // --- Extended demo args, config variants, memory/cache/scaffold extended ---
    include!("tests_config_extended.rs");

    // --- Search, validate, context, analyze, qdd, refactor, roadmap, test, spec, work commands ---
    include!("tests_spec_and_work.rs");
}
