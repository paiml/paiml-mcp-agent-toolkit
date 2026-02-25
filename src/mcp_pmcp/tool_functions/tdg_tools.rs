// TDG tools - split into include files for file health (CB-040)
//
// tdg_tools_analysis.rs    - analyze_tdg, compare_tdg, single/multi path analysis
// tdg_tools_storage.rs     - storage management, analyze_with_storage, record creation
// tdg_tools_diagnostics.rs - system diagnostics, performance metrics, health check

include!("tdg_tools_analysis.rs");
include!("tdg_tools_storage.rs");
include!("tdg_tools_diagnostics.rs");
