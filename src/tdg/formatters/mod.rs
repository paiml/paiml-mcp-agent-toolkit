#![cfg_attr(coverage_nightly, coverage(off))]

mod comparison;
pub(crate) mod helpers;
mod human;
mod json;
mod markdown;
mod project;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_coverage;

pub use comparison::format_comparison;
pub use human::format_human;
pub use json::format_json;
pub use markdown::format_markdown;
pub use project::format_project;
