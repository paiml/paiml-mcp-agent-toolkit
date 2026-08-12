//! How a file is named inside a "top N files" list.
//!
//! Every one of these lists used to print `Path::file_name()`, so
//! `analyze duplicates` on this repo rendered
//!
//! ```text
//!   1. core_tests_properties.rs - 100.0% duplication (292 / 292 lines)
//!   4. core_tests_properties.rs - 100.0% duplication (292 / 292 lines)
//! ```
//!
//! which reads as one file listed twice. They are two different files —
//! `./src/ast/core_tests_properties.rs` and
//! `./src/ast/core/core_tests_properties.rs` — and the basename was the only
//! thing separating them, so it separated nothing. `analyze symbol-table` was
//! worse still: three of its ten slots were `mod.rs`, `tests.rs` and
//! `types.rs`, names this tree carries hundreds of copies of.
//!
//! A report row a reader cannot resolve back to a file on disk is not
//! actionable, so these lists now print the path the analyzer actually keyed
//! the statistics by — the same string the `--format json` document carries.

/// The path to print for `file_path` in a report row.
///
/// The analyzers key their per-file statistics by a project-relative path
/// (`./src/ast/core.rs`); the leading `./` is noise in a rendered list, so it
/// is trimmed. Nothing else is removed: whatever is left is unique per file,
/// which is the entire point.
#[must_use]
pub fn report_path(file_path: &str) -> &str {
    file_path.strip_prefix("./").unwrap_or(file_path)
}

#[cfg(test)]
#[path = "report_paths_tests.rs"]
mod tests;
