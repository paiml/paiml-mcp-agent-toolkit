// Extended tool handlers - split for file health (CB-040)
//
// Semantic submodules:
//   - extended_tools_relevance.rs     : Template relevance scoring
//   - extended_tools_complexity.rs    : Complexity analysis handlers
//   - extended_tools_dag.rs           : DAG analysis handlers
//   - extended_tools_context.rs       : Context generation handlers
//   - extended_tools_architecture.rs  : System architecture analysis handlers

// --- Template relevance scoring ---
include!("extended_tools_relevance.rs");

// --- Complexity analysis handlers ---
include!("extended_tools_complexity.rs");

// --- DAG analysis handlers ---
include!("extended_tools_dag.rs");

// --- Context generation handlers ---
include!("extended_tools_context.rs");

// --- System architecture analysis handlers ---
include!("extended_tools_architecture.rs");

// Tests extracted to tools_tests.rs for file health compliance (CB-040)
