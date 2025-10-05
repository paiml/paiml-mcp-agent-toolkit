pub mod analyzers;
pub mod complexity;
pub mod complexity_enhanced;
pub mod config; // TICKET-PMAT-5024: Configuration management
pub mod efficiency;
pub mod efficiency_enhanced;
pub mod entropy;
pub mod gate;
pub mod gate_runner;
pub mod gates; // TICKET-PMAT-5020: Quality gate executor
pub mod git_hooks;
pub mod satd;
pub mod satd_item;

// Re-export quality gate executor (TICKET-PMAT-5020)
pub use gates::{
    execute_all_gates, execute_clippy, execute_coverage, execute_complexity, execute_tests,
    format_report as format_quality_report, GateConfig, GateError, GateResult, QualityReport,
};

// Re-export configuration management (TICKET-PMAT-5024)
pub use config::{generate_config_toml, generate_default_config, validate_config};
