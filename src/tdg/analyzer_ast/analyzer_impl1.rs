// TdgAnalyzerAst impl part 1: constructors, configuration, git context
include!("analyzer_impl1_constructors.rs");

// TdgAnalyzerAst impl part 1: file analysis pipeline (analyze_file, caching, storage)
include!("analyzer_impl1_file_analysis.rs");

// TdgAnalyzerAst impl part 1: diagnostics, stats, resource monitoring
include!("analyzer_impl1_diagnostics.rs");

// TdgAnalyzerAst impl part 1: analyze_source dispatcher + Rust/Python AST analyzers
include!("analyzer_impl1_source_dispatch.rs");

// TdgAnalyzerAst impl part 1: JavaScript, Go, Java, Lua, C AST analyzers
include!("analyzer_impl1_language_extra.rs");
