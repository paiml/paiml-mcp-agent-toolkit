// Decode methods for analyze subcommands (split for file health CB-040)
//
// Core decoders: complexity (with migration), churn, dag, dead_code,
//   satd, deep_context, tdg, provability
include!("decode_analyze_core.rs");

// Extended decoders: serve, demo, lint_hotspot, makefile, duplicates,
//   defect_prediction, comprehensive, graph_metrics, name_similarity,
//   proof_annotations, incremental_coverage, symbol_table, big_o,
//   assemblyscript, webassembly, and utility functions
include!("decode_analyze_extended.rs");
