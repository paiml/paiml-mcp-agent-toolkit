//! AST parsing strategies for multi-language code analysis
//! Split for file health compliance (CB-040)

include!("ast_strategies_impl.rs");

#[cfg(test)]
#[path = "ast_strategies_tests.rs"]
mod tests;
