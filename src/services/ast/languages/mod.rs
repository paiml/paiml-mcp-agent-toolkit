// Toyota Way: Language-Specific AST Strategies
//
// This module contains all language-specific AST parsing strategies,
// consolidating the functionality from individual ast_*.rs files

pub mod rust;

#[cfg(feature = "typescript-ast")]
pub mod typescript;

#[cfg(feature = "typescript-ast")]
pub mod javascript;

#[cfg(feature = "python-ast")]
pub mod python;

#[cfg(feature = "c-ast")]
pub mod c;

#[cfg(feature = "cpp-ast")]
pub mod cpp;

#[cfg(any(feature = "c-ast", feature = "cpp-ast"))]
pub mod c_cpp_strategy;

#[cfg(feature = "kotlin-ast")]
pub mod kotlin_strategy;
