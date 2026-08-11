//! Tests for lint hotspot handlers
//! Split for file health compliance (CB-040)
//!
//! #701: the split used to cut `mod coverage_tests` and one `#[test] fn` in
//! half across `include!` boundaries. `include!` needs each included file to
//! parse as a complete item sequence, so the bundle could not compile in any
//! profile — it was parked behind the deliberately non-compiling
//! `broken-tests` feature and then silently rotted off the real types (every
//! `DiagnosticSpan` literal still carried a `_text` field that `types.rs` had
//! dropped) while people kept editing it, believing they were pinning
//! behaviour. The module boundary now lives here, in the parent, so each part
//! file is a valid item sequence on its own and the compiler keeps them honest.

use super::*;

include!("lint_hotspot_tests_part1.rs");

mod coverage_tests {
    // These live in private sibling modules; the fragments called them
    // unqualified back when they were one file.
    use super::clippy::{
        count_sloc, count_top_lints, extract_lint_name, find_primary_span, is_machine_applicable,
        is_target_file, resolve_absolute_path, update_severity_distribution,
    };
    use super::metrics::{calculate_defect_density, calculate_total_violations};
    use super::output::{format_detailed, format_json, format_sarif};
    use super::types::{DiagnosticCode, DiagnosticMessage, DiagnosticSpan, FileMetrics};

    include!("lint_hotspot_tests_part1_helpers.rs");
    include!("lint_hotspot_tests_part2.rs");
    include!("lint_hotspot_tests_part3.rs");
    include!("lint_hotspot_tests_part4.rs");
}
