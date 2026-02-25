// Tests for refactor docs handlers
// Extracted for file health compliance (CB-040)
// Split via include!() pattern for size compliance (PMAT-503)

use super::*;

mod tests {
    use super::*;
    use std::time::Duration;
    use tempfile::TempDir;

    // FileCategory, CleanupSummary, CruftFile, RefactorDocsResult, should_preserve,
    // matches_pattern, collect_scan_directories, combine_patterns,
    // should_use_interactive_mode, should_create_backup, should_remove_files
    include!("refactor_docs_tests_types_patterns.rs");

    // passes_file_filters, calculate_age_days, update_summary_for_cruft,
    // merge_summary, finalize_summary
    include!("refactor_docs_tests_summary.rs");

    // format_output, format_summary, format_detailed, format_json,
    // create_cruft_file, get_file_metadata
    include!("refactor_docs_tests_formatting.rs");

    // Async I/O tests, edge cases, and serialization
    include!("refactor_docs_tests_async_serial.rs");
}

// Property-based stability tests
include!("refactor_docs_tests_property.rs");
