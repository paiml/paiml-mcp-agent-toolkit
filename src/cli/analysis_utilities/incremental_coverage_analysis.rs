// The producer that used to live here is gone (#954).
//
// `convert_coverage_update_to_report` turned a `CoverageUpdate` into an
// `IncrementalCoverageReport` by inventing the half of the comparison it did
// not have:
//
//     let base_coverage = file_coverage.line_coverage.max(50.0) - 10.0; // Simulate previous coverage
//
// so every file reported a fixed +10.0 delta, over a `line_coverage` that was
// itself fabricated upstream by `IncrementalCoverageAnalyzer::compute_coverage`
// (`// For now, return mock data`: 66.67% line, 75.0 branch, 80.0 function for
// every file in every project). Its only caller was the duplicate
// `handle_analyze_incremental_coverage` in `proof_coverage.rs`, which now
// forwards to the wired handler — the one that measures, and that reports
// "not measured" for the files it cannot.
//
// A simulated baseline is not a cheaper measurement, it is a different number
// wearing the same label, so this is deleted rather than defaulted. The report
// types and renderers in `incremental_coverage.rs` /
// `incremental_coverage_formatters.rs` remain: they are pure formatting and
// carry no claim of their own.
