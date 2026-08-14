//! Regression tests for the `-- Document Results --` gate.
//!
//! Kept in their own file (declared with `#[path]` in `mod.rs` after the
//! `include!`) because `query_execution.rs` is `include!`d — a `#[cfg(test)]`
//! module inside it would put items after a test module.

use super::should_emit_docs_section;

/// `--files-with-matches` is `rg -l`: one path per line, nothing else.
///
/// The document printer used to ignore the flag entirely, so the paths were
/// followed by a `-- Document Results --` heading and numbered markdown-table
/// snippets — 1.6KB of prose fed to a consumer reading filenames.
#[test]
fn files_with_matches_suppresses_the_document_section() {
    assert!(
        !should_emit_docs_section(true, false, true, false),
        "--files-with-matches must print paths and nothing else"
    );
}

/// `--count` is `rg -c`: one `path:count` per line, nothing else.
#[test]
fn count_suppresses_the_document_section() {
    assert!(
        !should_emit_docs_section(true, false, false, true),
        "--count must print path:count lines and nothing else"
    );
}

/// The ordinary case is untouched: documents still follow code results.
#[test]
fn plain_query_still_gets_the_document_section() {
    assert!(should_emit_docs_section(true, false, false, false));
}

/// The pre-existing suppressions still hold.
#[test]
fn no_docs_and_lexical_modes_still_suppress() {
    assert!(
        !should_emit_docs_section(false, false, false, false),
        "--no-docs"
    );
    assert!(
        !should_emit_docs_section(true, true, false, false),
        "--regex/--literal search the raw tree, not the document index"
    );
}
