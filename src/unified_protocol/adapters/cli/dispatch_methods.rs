// Dispatch methods for CliAdapter - split for file health (CB-040)
// Basic + Advanced analysis dispatch (churn, complexity, dead code, SATD, TDG, etc.)
include!("dispatch_methods_basic.rs");
// Structural + Specialized analysis dispatch (DAG, graph metrics, symbol table, provability, etc.)
include!("dispatch_methods_structural.rs");
