#![cfg_attr(coverage_nightly, coverage(off))]
//! TypeScript and JavaScript language AST parsing strategies

mod javascript;
mod strategy;
pub(crate) mod visitor;

#[cfg(test)]
mod tests;

pub use javascript::JavaScriptStrategy;
pub use strategy::TypeScriptStrategy;
