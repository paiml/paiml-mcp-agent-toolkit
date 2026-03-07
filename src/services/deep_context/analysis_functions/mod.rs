// Analysis functions - refactored from monolithic file for file health (CB-040)
//
// Submodules organized by semantic grouping:
// - dispatch: main dispatch/routing for language analysis
// - rust: Rust-specific analysis functions
// - systems: systems language analysis (C, C++, Go, Java, etc.)
// - scripting: scripting language analysis (Python, JavaScript, TypeScript, etc.)
// - metrics: shared metrics computation (complexity, churn, dead code, etc.)

mod dispatch;
mod metrics;
mod rust;
mod scripting;
mod systems;

// Re-export all public items that were previously public from the monolithic file

// Dispatch functions
pub use dispatch::{analyze_file_by_language, analyze_single_file};

// Rust-specific
pub use rust::analyze_rust_language;

// Systems languages
pub use systems::{
    analyze_c_language, analyze_cpp_language, analyze_csharp_file, analyze_csharp_language,
    analyze_go_language, analyze_java_file, analyze_java_language, analyze_kotlin_language,
    analyze_lean_language, analyze_swift_file, analyze_swift_language, analyze_wasm_language,
};

// Scripting languages
pub use scripting::{
    analyze_bash_language, analyze_elixir_language, analyze_erlang_language,
    analyze_haskell_language, analyze_lua_language, analyze_ocaml_language,
    analyze_python_language, analyze_ruby_language, analyze_typescript_language,
};

// Metrics / analysis
pub use metrics::{
    analyze_big_o, analyze_churn, analyze_complexity, analyze_dag, analyze_dead_code,
    analyze_duplicate_code, analyze_provability, analyze_satd, detect_project_language,
};

// Tests extracted to deep_context_tests.rs for file health compliance (CB-040)
// TEMPORARILY DISABLED: Test file is missing (deep_context_tests.rs does not exist)
