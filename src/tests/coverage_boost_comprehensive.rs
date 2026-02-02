//! Coverage boost tests for cli/analysis_utilities/comprehensive.rs
//! Tests: determine_satd_severity pure function

use crate::cli::analysis_utilities::determine_satd_severity;

#[test]
fn test_determine_satd_severity_hack() {
    assert_eq!(determine_satd_severity("HACK"), "high");
}

#[test]
fn test_determine_satd_severity_xxx() {
    assert_eq!(determine_satd_severity("XXX"), "high");
}

#[test]
fn test_determine_satd_severity_fixme() {
    assert_eq!(determine_satd_severity("FIXME"), "medium");
}

#[test]
fn test_determine_satd_severity_refactor() {
    assert_eq!(determine_satd_severity("REFACTOR"), "medium");
}

#[test]
fn test_determine_satd_severity_todo() {
    assert_eq!(determine_satd_severity("TODO"), "low");
}

#[test]
fn test_determine_satd_severity_unknown() {
    assert_eq!(determine_satd_severity("UNKNOWN"), "low");
    assert_eq!(determine_satd_severity(""), "low");
    assert_eq!(determine_satd_severity("OPTIMIZE"), "low");
}
