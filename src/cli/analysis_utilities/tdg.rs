// TDG handlers - extracted for file health (CB-040)
// See handle_analyze_tdg() in tdg_analysis.rs for full documentation.

// Helper utilities: percentile, primary factor identification, filtering
include!("tdg_helpers.rs");

// Output formatting: table, json, markdown, sarif
include!("tdg_formatting.rs");

// Core analysis: single file, multiple files, project
include!("tdg_analysis.rs");

// Watch mode (feature-gated)
include!("tdg_watch.rs");

// Main entry point and orchestration
include!("tdg_handler.rs");
