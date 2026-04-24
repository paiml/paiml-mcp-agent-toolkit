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
    fn test_compute_decay_score_grade_a_is_zero() {
        let h = hotspot_with(Some("A"), 10, 5, 0.0);
        assert_eq!(compute_decay_score(&h, 100), 0.0);
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
        for (g, expected_tdg) in [
            ("A", 0.0f32),
            ("B", 0.25),
            ("C", 0.5),
            ("D", 0.75),
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
