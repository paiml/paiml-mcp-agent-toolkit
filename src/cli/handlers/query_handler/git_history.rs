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
#[allow(clippy::field_reassign_with_default)]
mod annotations_tests {
    //! PMAT-653: cover git_history_annotations.rs pure helpers.
    use super::*;
    use crate::services::git_history::{ChangeType, CommitInfo, FileChange};
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn fc(path: &str, add: u32, del: u32) -> FileChange {
        FileChange {
            path: path.to_string(),
            change_type: ChangeType::Modified,
            lines_added: add,
            lines_deleted: del,
        }
    }

    fn commit(
        hash: &str,
        author: &str,
        files: Vec<FileChange>,
        is_fix: bool,
        is_feat: bool,
    ) -> CommitInfo {
        CommitInfo {
            hash: hash.to_string(),
            message_subject: String::new(),
            message_body: None,
            author_name: author.to_string(),
            author_email: String::new(),
            timestamp: 0,
            is_merge: false,
            is_fix,
            is_feat,
            issue_refs: Vec::new(),
            files,
        }
    }

    fn write(p: &std::path::Path, content: &str) {
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(p, content).unwrap();
    }

    // --- count_pairwise_cochanges ---

    #[test]
    fn test_count_pairwise_cochanges_empty() {
        let mut m = HashMap::new();
        count_pairwise_cochanges(&[], &mut m);
        assert!(m.is_empty());
    }

    #[test]
    fn test_count_pairwise_cochanges_single_file_no_pair() {
        let mut m = HashMap::new();
        count_pairwise_cochanges(&["a.rs"], &mut m);
        assert!(m.is_empty());
    }

    #[test]
    fn test_count_pairwise_cochanges_two_files_sorted() {
        let mut m = HashMap::new();
        count_pairwise_cochanges(&["b.rs", "a.rs"], &mut m);
        assert_eq!(m.get(&("a.rs".to_string(), "b.rs".to_string())), Some(&1));
    }

    #[test]
    fn test_count_pairwise_cochanges_three_files_makes_3_pairs() {
        let mut m = HashMap::new();
        count_pairwise_cochanges(&["a.rs", "b.rs", "c.rs"], &mut m);
        assert_eq!(m.len(), 3);
        assert_eq!(m[&("a.rs".to_string(), "b.rs".to_string())], 1);
        assert_eq!(m[&("a.rs".to_string(), "c.rs".to_string())], 1);
        assert_eq!(m[&("b.rs".to_string(), "c.rs".to_string())], 1);
    }

    // --- aggregate_hotspots ---

    #[test]
    fn test_aggregate_hotspots_accumulates_commit_counts_and_flags() {
        let commits = vec![
            commit("h1", "alice", vec![fc("a.rs", 10, 5)], true, false),
            commit("h2", "alice", vec![fc("a.rs", 3, 1)], false, true),
        ];
        let (hotspots, cochange) = aggregate_hotspots(&commits);
        let a = hotspots.get("a.rs").unwrap();
        assert_eq!(a.commit_count, 2);
        assert_eq!(a.fix_count, 1);
        assert_eq!(a.feat_count, 1);
        assert_eq!(a.lines_added, 13);
        assert_eq!(a.lines_deleted, 6);
        assert_eq!(a.authors.get("alice"), Some(&2));
        assert!(cochange.is_empty());
    }

    #[test]
    fn test_aggregate_hotspots_cochange_skipped_over_15_files() {
        let files: Vec<FileChange> = (0..16).map(|i| fc(&format!("f{i}.rs"), 1, 0)).collect();
        let commits = vec![commit("h", "a", files, false, false)];
        let (_hotspots, cochange) = aggregate_hotspots(&commits);
        assert!(
            cochange.is_empty(),
            "expected co-change skipped for merge-like commit"
        );
    }

    #[test]
    fn test_aggregate_hotspots_cochange_counted_2_files() {
        let commits = vec![commit(
            "h",
            "a",
            vec![fc("a.rs", 1, 0), fc("b.rs", 1, 0)],
            false,
            false,
        )];
        let (_hotspots, cochange) = aggregate_hotspots(&commits);
        assert_eq!(cochange.len(), 1);
    }

    // --- compute_cochange_pairs ---

    /// Helper: build a hotspots map where every file has `commit_count` ≥ any
    /// observed co-change count. Prevents `ca + cb - count` underflow when
    /// the production code's `hotspots.get(...)` default-maps to 1.
    fn hotspots_with_counts(files: &[&str], commit_count: usize) -> HashMap<String, FileHotspot> {
        let mut m = HashMap::new();
        for f in files {
            let mut h = FileHotspot::default();
            h.commit_count = commit_count;
            m.insert((*f).to_string(), h);
        }
        m
    }

    #[test]
    fn test_compute_cochange_pairs_filters_below_3_threshold() {
        let mut cc = HashMap::new();
        cc.insert(("a.rs".to_string(), "b.rs".to_string()), 2);
        cc.insert(("a.rs".to_string(), "c.rs".to_string()), 5);
        let hotspots = hotspots_with_counts(&["a.rs", "b.rs", "c.rs"], 10);
        let pairs = compute_cochange_pairs(cc, &hotspots);
        // Only the (a,c)=5 pair passes >=3 threshold.
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].count, 5);
    }

    #[test]
    fn test_compute_cochange_pairs_sorts_desc_and_truncates_to_5() {
        let mut cc = HashMap::new();
        let mut paths: Vec<String> = Vec::new();
        for i in 0..8 {
            let a = format!("a{i}.rs");
            let b = format!("b{i}.rs");
            cc.insert((a.clone(), b.clone()), 10 + i as usize);
            paths.push(a);
            paths.push(b);
        }
        let path_refs: Vec<&str> = paths.iter().map(String::as_str).collect();
        let hotspots = hotspots_with_counts(&path_refs, 50);
        let pairs = compute_cochange_pairs(cc, &hotspots);
        assert_eq!(pairs.len(), 5);
        for i in 0..4 {
            assert!(pairs[i].count >= pairs[i + 1].count, "not sorted desc");
        }
        assert_eq!(pairs[0].count, 17);
    }

    #[test]
    fn test_compute_cochange_pairs_jaccard_calc() {
        let mut cc = HashMap::new();
        cc.insert(("a.rs".to_string(), "b.rs".to_string()), 10);
        let mut hotspots: HashMap<String, FileHotspot> = HashMap::new();
        let mut ha = FileHotspot::default();
        ha.commit_count = 20;
        hotspots.insert("a.rs".to_string(), ha);
        let mut hb = FileHotspot::default();
        hb.commit_count = 15;
        hotspots.insert("b.rs".to_string(), hb);
        let pairs = compute_cochange_pairs(cc, &hotspots);
        // union = 20+15-10 = 25; jaccard = 10/25 = 0.4
        assert!((pairs[0].jaccard - 0.4).abs() < 1e-6);
    }

    // --- compute_decay_score ---

    fn hotspot_with(
        grade: Option<&str>,
        commit_count: usize,
        fix_count: usize,
        dead_pct: f32,
    ) -> FileHotspot {
        let mut h = FileHotspot::default();
        h.annotation.tdg_grade = grade.map(String::from);
        h.commit_count = commit_count;
        h.fix_count = fix_count;
        h.annotation.dead_code_pct = dead_pct;
        h
    }

    #[test]
    fn test_compute_decay_score_grade_a_plus_is_near_zero() {
        // Risk is derived from crate::tdg::Grade's band floors, so only the
        // top band (A+, floor 95) is essentially risk-free. The old five-arm
        // table hard-coded A -> 0.0, which made a plain A indistinguishable
        // from a perfect score.
        let h = hotspot_with(Some("A+"), 10, 5, 0.0);
        assert!(compute_decay_score(&h, 100) < 0.01);
    }

    #[test]
    fn test_compute_decay_score_better_grade_decays_less() {
        // The property that matters: monotone in grade, over all eleven.
        let mut prev = -1.0f32;
        for g in ["A+", "A", "A-", "B+", "B", "B-", "C+", "C", "C-", "D", "F"] {
            let h = hotspot_with(Some(g), 10, 0, 0.0);
            let decay = compute_decay_score(&h, 10);
            assert!(
                decay > prev,
                "grade {g} decayed {decay}, not more than the better grade before it ({prev})"
            );
            prev = decay;
        }
    }

    #[test]
    fn test_compute_decay_score_grade_f_with_heavy_churn_clamps_to_1() {
        let h = hotspot_with(Some("F"), 100, 100, 50.0);
        assert_eq!(compute_decay_score(&h, 100), 1.0);
    }

    #[test]
    fn test_compute_decay_score_zero_commits_zero_score() {
        let h = hotspot_with(Some("F"), 0, 0, 0.0);
        assert_eq!(compute_decay_score(&h, 0), 0.0);
    }

    #[test]
    fn test_compute_decay_score_grades_mapping() {
        // 1 - band_floor/100, straight from crate::tdg::Grade::score_band.
        for (g, expected_tdg) in [
            ("A", 0.10f32),
            ("B", 0.25),
            ("C", 0.40),
            ("D", 0.50),
            ("F", 1.0),
        ] {
            let h = hotspot_with(Some(g), 10, 0, 0.0);
            let decay = compute_decay_score(&h, 10);
            assert!(
                (decay - expected_tdg).abs() < 1e-6,
                "grade {g}: expected {expected_tdg}, got {decay}"
            );
        }
    }

    #[test]
    fn test_compute_decay_score_unknown_grade_maps_to_0_5() {
        let h = hotspot_with(Some("X"), 10, 0, 0.0);
        assert!((compute_decay_score(&h, 10) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_compute_decay_score_missing_grade_defaults_to_0_5() {
        let h = hotspot_with(None, 10, 0, 0.0);
        assert!((compute_decay_score(&h, 10) - 0.5).abs() < 1e-6);
    }

    // --- compute_impact_risk ---

    #[test]
    fn test_compute_impact_risk_zero_commits_zero() {
        let h = FileHotspot::default();
        assert_eq!(compute_impact_risk(&h, 0), 0.0);
    }

    #[test]
    fn test_compute_impact_risk_with_pagerank_and_faults() {
        let mut h = FileHotspot::default();
        h.annotation.max_pagerank = Some(0.0005);
        h.annotation.fault_count = 2;
        h.commit_count = 10;
        // pagerank*10000 = 5.0, churn = 10/100 = 0.1, (1+2) = 3 → 5 * 0.1 * 3 = 1.5
        let r = compute_impact_risk(&h, 100);
        assert!((r - 1.5).abs() < 1e-6);
    }

    #[test]
    fn test_compute_impact_risk_clamps_to_100() {
        let mut h = FileHotspot::default();
        h.annotation.max_pagerank = Some(1.0);
        h.annotation.fault_count = 100;
        h.commit_count = 100;
        assert_eq!(compute_impact_risk(&h, 100), 100.0);
    }

    // --- load_work_ticket ---

    #[test]
    fn test_load_work_ticket_non_pmat_prefix_returns_none() {
        let tmp = TempDir::new().unwrap();
        assert!(load_work_ticket(tmp.path(), "random-ref").is_none());
    }

    #[test]
    fn test_load_work_ticket_missing_contract_returns_none() {
        let tmp = TempDir::new().unwrap();
        assert!(load_work_ticket(tmp.path(), "PMAT-99").is_none());
    }

    #[test]
    fn test_load_work_ticket_hash_prefix_maps_to_pmat() {
        let tmp = TempDir::new().unwrap();
        let contract = serde_json::json!({
            "claims": [
                {"result": {"falsified": false}},
                {"result": {"falsified": true}},
                {"result": {"falsified": false}},
            ],
            "baseline_tdg": 1.5,
        });
        write(
            &tmp.path().join(".pmat-work/PMAT-100/contract.json"),
            &contract.to_string(),
        );
        let t = load_work_ticket(tmp.path(), "#100").expect("some");
        assert_eq!(t.ticket_id, "PMAT-100");
        assert_eq!(t.claims_total, 3);
        assert_eq!(t.claims_passed, 2);
        assert!((t.baseline_tdg - 1.5).abs() < 1e-9);
    }

    #[test]
    fn test_load_work_ticket_lowercase_pmat_uppercased() {
        let tmp = TempDir::new().unwrap();
        let contract = serde_json::json!({
            "claims": [{"result": {"falsified": false}}],
        });
        write(
            &tmp.path().join(".pmat-work/PMAT-7/contract.json"),
            &contract.to_string(),
        );
        let t = load_work_ticket(tmp.path(), "pmat-7").expect("some");
        assert_eq!(t.ticket_id, "PMAT-7");
    }

    // --- load_commit_quality ---

    #[test]
    fn test_load_commit_quality_missing_returns_none() {
        let tmp = TempDir::new().unwrap();
        assert!(load_commit_quality(tmp.path(), "abc1234").is_none());
    }

    #[test]
    fn test_load_commit_quality_valid_parse() {
        let tmp = TempDir::new().unwrap();
        let meta = serde_json::json!({
            "work_item_id": "PMAT-1",
            "tdg_score": 85.5,
            "repo_score": 90.0,
        });
        write(
            &tmp.path().join(".pmat-metrics/commit-abc1234-meta.json"),
            &meta.to_string(),
        );
        let q = load_commit_quality(tmp.path(), "abc1234567").expect("some");
        assert!((q.tdg_score - 85.5).abs() < 1e-9);
    }

    // --- aggregate_bug_hunter_faults ---

    #[test]
    fn test_aggregate_bug_hunter_faults_missing_dir_empty() {
        let tmp = TempDir::new().unwrap();
        let counts = aggregate_bug_hunter_faults(&tmp.path().join("nonexistent"));
        assert!(counts.is_empty());
    }

    #[test]
    fn test_aggregate_bug_hunter_faults_counts_per_file() {
        let tmp = TempDir::new().unwrap();
        let cache = serde_json::json!({
            "findings": [
                {"file": "a.rs"},
                {"file": "a.rs"},
                {"file": "b.rs"},
            ],
        });
        write(&tmp.path().join("cache.json"), &cache.to_string());
        let counts = aggregate_bug_hunter_faults(tmp.path());
        assert_eq!(counts.get("a.rs"), Some(&2));
        assert_eq!(counts.get("b.rs"), Some(&1));
    }

    // --- load_bug_hunter_annotations ---

    #[test]
    fn test_load_bug_hunter_annotations_missing_dir_is_no_op() {
        let tmp = TempDir::new().unwrap();
        let mut annots: HashMap<String, FileAnnotation> = HashMap::new();
        load_bug_hunter_annotations(tmp.path(), &mut annots);
        assert!(annots.is_empty());
    }

    #[test]
    fn test_load_bug_hunter_annotations_updates_fault_count_when_higher() {
        let tmp = TempDir::new().unwrap();
        let cache = serde_json::json!({
            "findings": [{"file": "a.rs"}, {"file": "a.rs"}, {"file": "a.rs"}],
        });
        write(
            &tmp.path().join(".pmat/bug-hunter-cache/cache.json"),
            &cache.to_string(),
        );
        let mut annots: HashMap<String, FileAnnotation> = HashMap::new();
        annots.insert("a.rs".to_string(), FileAnnotation::default());
        load_bug_hunter_annotations(tmp.path(), &mut annots);
        assert_eq!(annots["a.rs"].fault_count, 3);
    }
}
#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
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
