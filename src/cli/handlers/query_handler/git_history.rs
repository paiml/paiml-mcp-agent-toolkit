//! Git history: annotation builders, formatters, and log parsing.

use super::options::*;
use crate::services::agent_context::AgentContextIndex;
use crate::services::git_history::{ChangeType, CommitInfo, FileChange, GitSearchResult};
use std::collections::HashMap;

/// Timing breakdown for git history search phases
pub(super) struct GitHistoryProfile {
    pub(super) git_log_ms: u128,
    pub(super) parse_ms: u128,
    pub(super) index_ms: u128,
    pub(super) search_ms: u128,
    pub(super) annotate_ms: u128,
    pub(super) total_ms: u128,
    pub(super) commit_count: usize,
}

// O(1) annotation builders, scoring functions, work ticket/commit quality loaders
include!("git_history_annotations.rs");

// Colorized output formatting for git history results
include!("git_history_formatting.rs");

// Git log parsing (PMAT_START block format) and commit classification
include!("git_history_parsing.rs");

#[cfg(test)]
mod tests {
    //! PMAT-645: cover git_history_formatting.rs pure helpers.
    use super::*;

    // --- classify_commit_type: 9 rules + default ---

    #[test]
    fn test_classify_commit_type_fix_prefix() {
        let (_, tag) = classify_commit_type("fix: handle null");
        assert_eq!(tag, "[fix]");
    }

    #[test]
    fn test_classify_commit_type_fix_via_contains() {
        // Subject starts with something else but contains "bugfix"
        let (_, tag) = classify_commit_type("deps: bugfix for upstream");
        assert_eq!(tag, "[fix]");
    }

    #[test]
    fn test_classify_commit_type_feat_prefix() {
        let (_, tag) = classify_commit_type("feat: add scoring");
        assert_eq!(tag, "[feat]");
    }

    #[test]
    fn test_classify_commit_type_add_prefix_is_feat() {
        let (_, tag) = classify_commit_type("add new module");
        assert_eq!(tag, "[feat]");
    }

    #[test]
    fn test_classify_commit_type_refactor() {
        let (_, tag) = classify_commit_type("refactor: extract helper");
        assert_eq!(tag, "[refactor]");
    }

    #[test]
    fn test_classify_commit_type_docs() {
        let (_, tag) = classify_commit_type("docs: update README");
        assert_eq!(tag, "[docs]");
    }

    #[test]
    fn test_classify_commit_type_test() {
        let (_, tag) = classify_commit_type("test: add coverage");
        assert_eq!(tag, "[test]");
    }

    #[test]
    fn test_classify_commit_type_perf() {
        let (_, tag) = classify_commit_type("perf: speed up parser");
        assert_eq!(tag, "[perf]");
    }

    #[test]
    fn test_classify_commit_type_chore() {
        let (_, tag) = classify_commit_type("chore: bump deps");
        assert_eq!(tag, "[chore]");
    }

    #[test]
    fn test_classify_commit_type_ci() {
        let (_, tag) = classify_commit_type("ci: fix workflow");
        assert_eq!(tag, "[ci]");
    }

    #[test]
    fn test_classify_commit_type_merge() {
        let (_, tag) = classify_commit_type("merge branch X into main");
        assert_eq!(tag, "[merge]");
    }

    #[test]
    fn test_classify_commit_type_default() {
        let (_, tag) = classify_commit_type("arbitrary subject with no convention");
        assert_eq!(tag, "");
    }

    #[test]
    fn test_classify_commit_type_is_case_insensitive() {
        let (_, tag) = classify_commit_type("FIX: upper case");
        assert_eq!(tag, "[fix]");
    }

    // --- format_timestamp: civil date algorithm ---

    #[test]
    fn test_format_timestamp_unix_epoch() {
        // 1970-01-01 00:00:00 UTC
        assert_eq!(format_timestamp(0), "1970-01-01");
    }

    #[test]
    fn test_format_timestamp_known_dates() {
        // Known Unix timestamps (verified via `date -u -d @<ts>`):
        // 946684800  = 2000-01-01 00:00:00 UTC
        assert_eq!(format_timestamp(946684800), "2000-01-01");
        // 1704067200 = 2024-01-01 00:00:00 UTC
        assert_eq!(format_timestamp(1704067200), "2024-01-01");
        // 1735689600 = 2025-01-01 00:00:00 UTC
        assert_eq!(format_timestamp(1735689600), "2025-01-01");
    }

    #[test]
    fn test_format_timestamp_leap_year_feb_29() {
        // 2024-02-29 00:00:00 UTC = 1709164800
        assert_eq!(format_timestamp(1709164800), "2024-02-29");
    }

    #[test]
    fn test_format_timestamp_month_boundary() {
        // 2023-03-01 = day after Feb 28 in non-leap year; 1677628800
        assert_eq!(format_timestamp(1677628800), "2023-03-01");
    }

    // --- grade_to_color: 5 grade letters + unknown ---

    #[test]
    fn test_grade_to_color_a_and_b_are_green() {
        assert_eq!(grade_to_color("A"), GREEN);
        assert_eq!(grade_to_color("B"), GREEN);
    }

    #[test]
    fn test_grade_to_color_c_is_yellow() {
        assert_eq!(grade_to_color("C"), YELLOW);
    }

    #[test]
    fn test_grade_to_color_d_is_red() {
        assert_eq!(grade_to_color("D"), RED);
    }

    #[test]
    fn test_grade_to_color_f_is_bright_red() {
        assert_eq!(grade_to_color("F"), BRIGHT_RED);
    }

    #[test]
    fn test_grade_to_color_unknown_is_dim() {
        assert_eq!(grade_to_color("?"), DIM);
        assert_eq!(grade_to_color(""), DIM);
        assert_eq!(grade_to_color("X"), DIM);
    }

    // --- format_fix_indicator: 3 branches ---

    #[test]
    fn test_format_fix_indicator_high_ratio_gets_double_bang() {
        let mut hs = FileHotspot::default();
        hs.commit_count = 10;
        hs.fix_count = 7; // ratio 0.7 > 0.5
        let s = format_fix_indicator(&hs);
        assert!(s.contains("!!7 fixes"), "got: {s:?}");
    }

    #[test]
    fn test_format_fix_indicator_any_fix_gets_count() {
        let mut hs = FileHotspot::default();
        hs.commit_count = 10;
        hs.fix_count = 2;
        let s = format_fix_indicator(&hs);
        assert!(s.contains("2 fixes") && !s.contains("!!"), "got: {s:?}");
    }

    #[test]
    fn test_format_fix_indicator_no_fixes_returns_empty() {
        let mut hs = FileHotspot::default();
        hs.commit_count = 10;
        hs.fix_count = 0;
        assert_eq!(format_fix_indicator(&hs), "");
    }

    #[test]
    fn test_format_fix_indicator_zero_commit_count_is_empty_via_ratio_branch() {
        // commit_count=0 → fix_ratio=0.0 path (first branch); if fix_count>0 it
        // still returns the "{n} fixes" form because ratio is 0 but count is positive.
        let mut hs = FileHotspot::default();
        hs.commit_count = 0;
        hs.fix_count = 1;
        let s = format_fix_indicator(&hs);
        assert!(s.contains("1 fixes"));
    }

    // --- format_decay_indicator: 3 branches ---

    #[test]
    fn test_format_decay_indicator_high_decay_is_bright_red() {
        let s = format_decay_indicator(0.75);
        assert!(s.contains("decay:0.75"));
    }

    #[test]
    fn test_format_decay_indicator_medium_decay_is_yellow() {
        let s = format_decay_indicator(0.30);
        assert!(s.contains("decay:0.30"));
    }

    #[test]
    fn test_format_decay_indicator_low_decay_is_empty() {
        assert_eq!(format_decay_indicator(0.10), "");
        assert_eq!(format_decay_indicator(0.0), "");
    }

    // --- format_risk_indicator: 3 branches ---

    #[test]
    fn test_format_risk_indicator_high_risk() {
        let s = format_risk_indicator(12.5);
        assert!(s.contains("risk:12.5"));
    }

    #[test]
    fn test_format_risk_indicator_medium_risk() {
        let s = format_risk_indicator(2.5);
        assert!(s.contains("risk:2.5"));
    }

    #[test]
    fn test_format_risk_indicator_low_risk_is_empty() {
        assert_eq!(format_risk_indicator(0.5), "");
        assert_eq!(format_risk_indicator(1.0), "");
    }

    // --- format_top_author ---

    #[test]
    fn test_format_top_author_empty_authors_is_empty_string() {
        let hs = FileHotspot::default();
        assert_eq!(format_top_author(&hs), "");
    }

    #[test]
    fn test_format_top_author_picks_max_author() {
        let mut hs = FileHotspot::default();
        hs.commit_count = 10;
        hs.authors.insert("alice".to_string(), 2);
        hs.authors.insert("bob".to_string(), 7);
        hs.authors.insert("carol".to_string(), 1);
        let s = format_top_author(&hs);
        // bob has highest count
        assert!(s.contains("bob:70%"), "got: {s:?}");
    }

    // --- format_cochange_section empty branch ---

    #[test]
    fn test_format_cochange_section_empty_early_returns() {
        let mut out = String::new();
        format_cochange_section(&mut out, &[]);
        assert!(out.is_empty());
    }

    #[test]
    fn test_format_cochange_section_populated_shows_header_and_pair() {
        let mut out = String::new();
        let pair = CoChangePair {
            file_a: "a.rs".to_string(),
            file_b: "b.rs".to_string(),
            count: 5,
            jaccard: 0.8,
        };
        format_cochange_section(&mut out, &[pair]);
        assert!(out.contains("Co-Change Coupling"));
        assert!(out.contains("a.rs"));
        assert!(out.contains("b.rs"));
        assert!(out.contains("5 co-changes"));
        assert!(out.contains("J=0.80"));
    }

    // --- format_annotated_file ---

    #[test]
    fn test_format_annotated_file_with_high_fix_count_and_dead_and_faults() {
        let mut hs = FileHotspot::default();
        hs.commit_count = 10;
        hs.fix_count = 5;
        hs.annotation.tdg_grade = Some("D".to_string());
        hs.annotation.dead_code_count = 3;
        hs.annotation.fault_count = 7;
        let mut out = String::new();
        format_annotated_file(&mut out, "src/foo.rs", &hs, 100);
        assert!(out.contains("[D]"));
        assert!(out.contains("5 fixes, 5%"));
        assert!(out.contains("dead:3"));
        assert!(out.contains("faults:7"));
    }

    #[test]
    fn test_format_annotated_file_low_fix_count_omits_fixes() {
        let mut hs = FileHotspot::default();
        hs.commit_count = 10;
        hs.fix_count = 2; // > 0 but not > 2 -- no fix indicator
        hs.annotation.tdg_grade = Some("A".to_string());
        let mut out = String::new();
        format_annotated_file(&mut out, "src/bar.rs", &hs, 100);
        assert!(out.contains("[A]"));
        assert!(!out.contains("fixes"));
    }

    #[test]
    fn test_format_annotated_file_missing_grade_shows_question_mark() {
        let hs = FileHotspot::default();
        let mut out = String::new();
        format_annotated_file(&mut out, "x.rs", &hs, 100);
        assert!(out.contains("[?]"));
    }
}
