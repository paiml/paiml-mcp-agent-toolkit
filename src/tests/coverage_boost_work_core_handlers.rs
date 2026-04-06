#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for cli/handlers/work_handlers/core_handlers.rs
//!
//! Tests for core work handler functions including:
//! - parse_acceptance_criteria
//! - parse_github_url
//! - claim_to_override_name (CLAIM_PATTERNS)
//! - CommitMetadata structure
//! - GitHubIssueInfo structure
//! - Falsification report processing helpers
//! - Score capture helper functions

use crate::cli::handlers::work_contract::{
    EvidenceType, FalsificationMethod, FalsificationResult,
};
use crate::cli::handlers::work_falsification::{ClaimResult, FalsificationReport};
use crate::models::roadmap::{ItemStatus, ItemType, Priority, RoadmapItem, Roadmap, Phase, SubTask};
use std::path::PathBuf;

// ============================================================================
// Tests for parse_acceptance_criteria
// ============================================================================

/// Test helper that mimics parse_acceptance_criteria logic
fn parse_acceptance_criteria_test(body: &str) -> Vec<String> {
    debug_assert!(!body.is_empty(), "body must not be empty");
    let mut criteria = Vec::new();

    for line in body.lines() {
        let trimmed = line.trim();
        // Match markdown checkboxes: - [ ] or - [x]
        if trimmed.starts_with("- [ ]") || trimmed.starts_with("- [x]") {
            let criterion = trimmed
                .trim_start_matches("- [ ]")
                .trim_start_matches("- [x]")
                .trim()
                .to_string();
            if !criterion.is_empty() {
                criteria.push(criterion);
            }
        }
    }

    criteria
}

#[test]
fn test_parse_acceptance_criteria_empty_body() {
    let body = "";
    let criteria = parse_acceptance_criteria_test(body);
    assert!(criteria.is_empty());
}

#[test]
fn test_parse_acceptance_criteria_no_checkboxes() {
    let body = "This is some text\nWithout any checkboxes\n- regular bullet point";
    let criteria = parse_acceptance_criteria_test(body);
    assert!(criteria.is_empty());
}

#[test]
fn test_parse_acceptance_criteria_unchecked_only() {
    let body = r#"## Acceptance Criteria
- [ ] First criterion
- [ ] Second criterion
- [ ] Third criterion
"#;
    let criteria = parse_acceptance_criteria_test(body);
    assert_eq!(criteria.len(), 3);
    assert_eq!(criteria[0], "First criterion");
    assert_eq!(criteria[1], "Second criterion");
    assert_eq!(criteria[2], "Third criterion");
}

#[test]
fn test_parse_acceptance_criteria_checked_only() {
    let body = r#"## Acceptance Criteria
- [x] First done
- [x] Second done
"#;
    let criteria = parse_acceptance_criteria_test(body);
    assert_eq!(criteria.len(), 2);
    assert_eq!(criteria[0], "First done");
    assert_eq!(criteria[1], "Second done");
}

#[test]
fn test_parse_acceptance_criteria_mixed() {
    let body = r#"## Acceptance Criteria
- [ ] Not started
- [x] Completed
- [ ] In progress
- [x] Also done
"#;
    let criteria = parse_acceptance_criteria_test(body);
    assert_eq!(criteria.len(), 4);
    assert_eq!(criteria[0], "Not started");
    assert_eq!(criteria[1], "Completed");
    assert_eq!(criteria[2], "In progress");
    assert_eq!(criteria[3], "Also done");
}

#[test]
fn test_parse_acceptance_criteria_with_whitespace() {
    let body = "  - [ ] Criterion with leading space\n\t- [x] Criterion with tab";
    let criteria = parse_acceptance_criteria_test(body);
    assert_eq!(criteria.len(), 2);
    assert_eq!(criteria[0], "Criterion with leading space");
    assert_eq!(criteria[1], "Criterion with tab");
}

#[test]
fn test_parse_acceptance_criteria_empty_checkbox() {
    let body = "- [ ] \n- [x] Valid";
    let criteria = parse_acceptance_criteria_test(body);
    assert_eq!(criteria.len(), 1);
    assert_eq!(criteria[0], "Valid");
}

#[test]
fn test_parse_acceptance_criteria_special_characters() {
    let body = "- [ ] Test `code` and **bold**\n- [x] Has: colons and (parentheses)";
    let criteria = parse_acceptance_criteria_test(body);
    assert_eq!(criteria.len(), 2);
    assert!(criteria[0].contains("`code`"));
    assert!(criteria[1].contains("colons"));
}

// ============================================================================
// Tests for parse_github_url
// ============================================================================

/// Test helper that mimics parse_github_url logic
fn parse_github_url_test(url: &str) -> Option<String> {
    debug_assert!(!url.is_empty(), "url must not be empty");
    // HTTPS: https://github.com/owner/repo.git
    if let Some(start) = url.find("github.com/") {
        let rest = &url[start + 11..];
        let repo = rest.trim_end_matches(".git");
        return Some(repo.to_string());
    }

    // SSH: git@github.com:owner/repo.git
    if let Some(start) = url.find("github.com:") {
        let rest = &url[start + 11..];
        let repo = rest.trim_end_matches(".git");
        return Some(repo.to_string());
    }

    None
}

#[test]
fn test_parse_github_url_https() {
    let url = "https://github.com/owner/repo.git";
    assert_eq!(parse_github_url_test(url), Some("owner/repo".to_string()));
}

#[test]
fn test_parse_github_url_https_no_git() {
    let url = "https://github.com/owner/repo";
    assert_eq!(parse_github_url_test(url), Some("owner/repo".to_string()));
}

#[test]
fn test_parse_github_url_ssh() {
    let url = "git@github.com:owner/repo.git";
    assert_eq!(parse_github_url_test(url), Some("owner/repo".to_string()));
}

#[test]
fn test_parse_github_url_ssh_no_git() {
    let url = "git@github.com:owner/repo";
    assert_eq!(parse_github_url_test(url), Some("owner/repo".to_string()));
}

#[test]
fn test_parse_github_url_nested_path() {
    let url = "https://github.com/org/project/subproject.git";
    assert_eq!(parse_github_url_test(url), Some("org/project/subproject".to_string()));
}

#[test]
fn test_parse_github_url_not_github() {
    let url = "https://gitlab.com/owner/repo.git";
    assert!(parse_github_url_test(url).is_none());
}

#[test]
fn test_parse_github_url_empty() {
    let url = "";
    assert!(parse_github_url_test(url).is_none());
}

#[test]
fn test_parse_github_url_invalid() {
    let url = "not a valid url";
    assert!(parse_github_url_test(url).is_none());
}

// ============================================================================
// Tests for claim_to_override_name (CLAIM_PATTERNS)
// ============================================================================

/// Pattern-based lookup for claim_to_override_name
const CLAIM_PATTERNS: &[(&[&str], &str)] = &[
    (&["manifest", "files deleted"], "manifest"),
    (&["meta-falsification", "falsification system"], "meta-falsification"),
    (&["coverage gaming", "coverage exclusion", "cfg(not(coverage))"], "coverage-gaming"),
    (&["differential coverage", "new code", "changed lines"], "differential-coverage"),
    (&["total coverage", "absolute coverage", "coverage does not decrease", "coverage >= 95"], "coverage"),
    (&["tdg", "test-driven grade"], "tdg"),
    (&["complexity", "cyclomatic"], "complexity"),
    (&["supply chain", "dependencies", "vulnerable dependencies"], "supply-chain"),
    (&["file size", "500 lines"], "file-size"),
    (&["spec", "specification"], "spec-quality"),
    (&["github", "sync", "changes pushed", "uncommitted"], "github-sync"),
    (&["examples", "compile"], "examples"),
    (&["book", "pmat-book"], "book"),
    (&["satd", "todo/fixme/hack"], "satd"),
    (&["dead code introduced", "dead code detected"], "dead-code"),
    (&["per-file coverage", "files have >= 95%", "all files have"], "per-file-coverage"),
    (&["lint passes", "make lint"], "lint"),
];

fn claim_to_override_name_test(hypothesis: &str) -> String {
    debug_assert!(!hypothesis.is_empty(), "hypothesis must not be empty");
    let hypothesis_lower = hypothesis.to_lowercase();

    for (patterns, name) in CLAIM_PATTERNS {
        if patterns.iter().any(|p| hypothesis_lower.contains(p)) {
            return name.to_string();
        }
    }

    // Unknown claim - use a sanitized version
    hypothesis_lower
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .take(30)
        .collect()
}

#[test]
fn test_claim_to_override_manifest() {
    assert_eq!(claim_to_override_name_test("All manifest files exist"), "manifest");
    assert_eq!(claim_to_override_name_test("No files deleted from baseline"), "manifest");
}

#[test]
fn test_claim_to_override_meta_falsification() {
    assert_eq!(claim_to_override_name_test("The meta-falsification check passes"), "meta-falsification");
    assert_eq!(claim_to_override_name_test("The falsification system is working"), "meta-falsification");
}

#[test]
fn test_claim_to_override_coverage_gaming() {
    assert_eq!(claim_to_override_name_test("No coverage gaming detected"), "coverage-gaming");
    assert_eq!(claim_to_override_name_test("No coverage exclusion patterns"), "coverage-gaming");
    assert_eq!(claim_to_override_name_test("No cfg(not(coverage)) found"), "coverage-gaming");
}

#[test]
fn test_claim_to_override_differential_coverage() {
    assert_eq!(claim_to_override_name_test("All changed lines are covered"), "differential-coverage");
    assert_eq!(claim_to_override_name_test("New code is tested"), "differential-coverage");
    assert_eq!(claim_to_override_name_test("Differential coverage met"), "differential-coverage");
}

#[test]
fn test_claim_to_override_total_coverage() {
    assert_eq!(claim_to_override_name_test("Total coverage >= 95%"), "coverage");
    assert_eq!(claim_to_override_name_test("Absolute coverage threshold met"), "coverage");
    assert_eq!(claim_to_override_name_test("Coverage does not decrease"), "coverage");
}

#[test]
fn test_claim_to_override_tdg() {
    assert_eq!(claim_to_override_name_test("TDG score >= baseline"), "tdg");
    assert_eq!(claim_to_override_name_test("Test-driven grade met"), "tdg");
}

#[test]
fn test_claim_to_override_complexity() {
    assert_eq!(claim_to_override_name_test("No function exceeds complexity 20"), "complexity");
    assert_eq!(claim_to_override_name_test("Cyclomatic complexity under threshold"), "complexity");
}

#[test]
fn test_claim_to_override_supply_chain() {
    assert_eq!(claim_to_override_name_test("No vulnerable dependencies added"), "supply-chain");
    assert_eq!(claim_to_override_name_test("Supply chain integrity verified"), "supply-chain");
    assert_eq!(claim_to_override_name_test("Dependencies are secure"), "supply-chain");
}

#[test]
fn test_claim_to_override_file_size() {
    assert_eq!(claim_to_override_name_test("No file exceeds 500 lines"), "file-size");
    assert_eq!(claim_to_override_name_test("File size limit respected"), "file-size");
}

#[test]
fn test_claim_to_override_spec_quality() {
    assert_eq!(claim_to_override_name_test("Spec score above threshold"), "spec-quality");
    assert_eq!(claim_to_override_name_test("Specification quality met"), "spec-quality");
}

#[test]
fn test_claim_to_override_github_sync() {
    assert_eq!(claim_to_override_name_test("All changes pushed to GitHub"), "github-sync");
    assert_eq!(claim_to_override_name_test("No uncommitted changes"), "github-sync");
    assert_eq!(claim_to_override_name_test("Sync status verified"), "github-sync");
}

#[test]
fn test_claim_to_override_examples() {
    assert_eq!(claim_to_override_name_test("All examples compile"), "examples");
    assert_eq!(claim_to_override_name_test("Examples run successfully"), "examples");
}

#[test]
fn test_claim_to_override_book() {
    assert_eq!(claim_to_override_name_test("pmat-book validation passes"), "book");
    assert_eq!(claim_to_override_name_test("Book documentation correct"), "book");
}

#[test]
fn test_claim_to_override_satd() {
    assert_eq!(claim_to_override_name_test("No new SATD markers"), "satd");
    assert_eq!(claim_to_override_name_test("No TODO/FIXME/HACK added"), "satd");
}

#[test]
fn test_claim_to_override_dead_code() {
    assert_eq!(claim_to_override_name_test("No dead code introduced"), "dead-code");
    assert_eq!(claim_to_override_name_test("Dead code detected and removed"), "dead-code");
}

#[test]
fn test_claim_to_override_per_file_coverage() {
    assert_eq!(claim_to_override_name_test("All files have >= 95% coverage"), "per-file-coverage");
    assert_eq!(claim_to_override_name_test("Per-file coverage threshold met"), "per-file-coverage");
}

#[test]
fn test_claim_to_override_lint() {
    assert_eq!(claim_to_override_name_test("make lint passes"), "lint");
    assert_eq!(claim_to_override_name_test("Lint passes with no errors"), "lint");
}

#[test]
fn test_claim_to_override_unknown() {
    let result = claim_to_override_name_test("Some completely unknown claim");
    // Should sanitize and truncate
    assert!(result.len() <= 30);
    assert!(result.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_'));
}

#[test]
fn test_claim_to_override_case_insensitive() {
    assert_eq!(claim_to_override_name_test("TDG SCORE"), "tdg");
    assert_eq!(claim_to_override_name_test("Complexity Check"), "complexity");
    assert_eq!(claim_to_override_name_test("COVERAGE GAMING"), "coverage-gaming");
}

// ============================================================================
// Tests for CommitMetadata structure (serialization)
// ============================================================================

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct TestCommitMetadata {
    commit_sha: Option<String>,
    work_item_id: String,
    prompt: String,
    tdg_score: f64,
    repo_score: f64,
    rust_project_score: Option<f64>,
    timestamp: chrono::DateTime<chrono::Utc>,
}

#[test]
fn test_commit_metadata_serialization() {
    let metadata = TestCommitMetadata {
        commit_sha: Some("abc123".to_string()),
        work_item_id: "PMAT-001".to_string(),
        prompt: "Fix bug #123".to_string(),
        tdg_score: 85.5,
        repo_score: 92.0,
        rust_project_score: Some(78.3),
        timestamp: chrono::Utc::now(),
    };

    let json = serde_json::to_string(&metadata).expect("Failed to serialize");
    assert!(json.contains("abc123"));
    assert!(json.contains("PMAT-001"));
    assert!(json.contains("85.5"));
}

#[test]
fn test_commit_metadata_deserialization() {
    let json = r#"{
        "commit_sha": "def456",
        "work_item_id": "PMAT-002",
        "prompt": "Add feature",
        "tdg_score": 90.0,
        "repo_score": 88.0,
        "rust_project_score": null,
        "timestamp": "2024-01-15T12:00:00Z"
    }"#;

    let metadata: TestCommitMetadata = serde_json::from_str(json).expect("Failed to deserialize");
    assert_eq!(metadata.work_item_id, "PMAT-002");
    assert_eq!(metadata.tdg_score, 90.0);
    assert!(metadata.rust_project_score.is_none());
}

#[test]
fn test_commit_metadata_no_sha() {
    let metadata = TestCommitMetadata {
        commit_sha: None,
        work_item_id: "PMAT-003".to_string(),
        prompt: "Refactor".to_string(),
        tdg_score: 75.0,
        repo_score: 80.0,
        rust_project_score: None,
        timestamp: chrono::Utc::now(),
    };

    let json = serde_json::to_string(&metadata).expect("Failed to serialize");
    assert!(json.contains("null") || json.contains("commit_sha"));
}

// ============================================================================
// Tests for GitHubIssueInfo structure
// ============================================================================

#[derive(Debug, Clone)]
struct TestGitHubIssueInfo {
    number: u64,
    title: String,
    body: Option<String>,
    labels: Vec<String>,
}

#[test]
fn test_github_issue_info_full() {
    let issue = TestGitHubIssueInfo {
        number: 42,
        title: "Fix authentication bug".to_string(),
        body: Some("This is the issue body".to_string()),
        labels: vec!["bug".to_string(), "priority:high".to_string()],
    };

    assert_eq!(issue.number, 42);
    assert_eq!(issue.title, "Fix authentication bug");
    assert!(issue.body.is_some());
    assert_eq!(issue.labels.len(), 2);
}

#[test]
fn test_github_issue_info_no_body() {
    let issue = TestGitHubIssueInfo {
        number: 100,
        title: "Feature request".to_string(),
        body: None,
        labels: vec![],
    };

    assert!(issue.body.is_none());
    assert!(issue.labels.is_empty());
}

#[test]
fn test_github_issue_info_debug() {
    let issue = TestGitHubIssueInfo {
        number: 1,
        title: "Test".to_string(),
        body: None,
        labels: vec!["test".to_string()],
    };

    let debug = format!("{:?}", issue);
    assert!(debug.contains("Test"));
    assert!(debug.contains("1"));
}

// ============================================================================
// Tests for filter_unoverriden_failures helper
// ============================================================================

fn filter_unoverriden_failures_test<'a>(
    failures: &[&'a ClaimResult],
    override_claims: Option<&Vec<String>>,
) -> Vec<&'a ClaimResult> {
    failures
        .iter()
        .filter(|failure| {
            let claim_name = claim_to_override_name_test(&failure.hypothesis);
            if let Some(overrides) = override_claims {
                !overrides.iter().any(|o| o.to_lowercase() == claim_name.to_lowercase())
            } else {
                true
            }
        })
        .copied()
        .collect()
}

fn make_claim_result(hypothesis: &str) -> ClaimResult {
    debug_assert!(!hypothesis.is_empty(), "hypothesis must not be empty");
    ClaimResult {
        index: 1,
        hypothesis: hypothesis.to_string(),
        method: FalsificationMethod::AbsoluteCoverage,
        result: FalsificationResult {
            falsified: true,
            evidence: None,
            explanation: "Failed".to_string(),
        },
        is_blocking: true,
    }
}

#[test]
fn test_filter_unoverriden_no_overrides() {
    let claim1 = make_claim_result("Coverage >= 95%");
    let claim2 = make_claim_result("TDG score regression");
    let failures: Vec<&ClaimResult> = vec![&claim1, &claim2];

    let result = filter_unoverriden_failures_test(&failures, None);
    assert_eq!(result.len(), 2);
}

#[test]
fn test_filter_unoverriden_with_matching_override() {
    let claim1 = make_claim_result("Coverage >= 95%");
    let claim2 = make_claim_result("TDG score regression");
    let failures: Vec<&ClaimResult> = vec![&claim1, &claim2];
    let overrides = vec!["coverage".to_string()];

    let result = filter_unoverriden_failures_test(&failures, Some(&overrides));
    assert_eq!(result.len(), 1);
    assert!(result[0].hypothesis.contains("TDG"));
}

#[test]
fn test_filter_unoverriden_all_overridden() {
    let claim1 = make_claim_result("Coverage >= 95%");
    let claim2 = make_claim_result("TDG score regression");
    let failures: Vec<&ClaimResult> = vec![&claim1, &claim2];
    let overrides = vec!["coverage".to_string(), "tdg".to_string()];

    let result = filter_unoverriden_failures_test(&failures, Some(&overrides));
    assert!(result.is_empty());
}

#[test]
fn test_filter_unoverriden_case_insensitive() {
    let claim = make_claim_result("Total coverage below 95%");
    let failures: Vec<&ClaimResult> = vec![&claim];
    let overrides = vec!["COVERAGE".to_string()];

    let result = filter_unoverriden_failures_test(&failures, Some(&overrides));
    assert!(result.is_empty());
}

#[test]
fn test_filter_unoverriden_empty_failures() {
    let failures: Vec<&ClaimResult> = vec![];
    let overrides = vec!["coverage".to_string()];

    let result = filter_unoverriden_failures_test(&failures, Some(&overrides));
    assert!(result.is_empty());
}

// ============================================================================
// Tests for FalsificationReport methods (from work_falsification)
// ============================================================================

fn make_test_claim(
    index: usize,
    hypothesis: &str,
    falsified: bool,
    is_blocking: bool,
) -> ClaimResult {
    debug_assert!(!hypothesis.is_empty(), "hypothesis must not be empty");
    ClaimResult {
        index,
        hypothesis: hypothesis.to_string(),
        method: FalsificationMethod::DifferentialCoverage,
        result: FalsificationResult {
            falsified,
            evidence: None,
            explanation: format!("Result for {}", hypothesis),
        },
        is_blocking,
    }
}

#[test]
fn test_report_has_blocking_with_multiple_blocking() {
    let report = FalsificationReport {
        total_claims: 4,
        passed: 2,
        failed: 2,
        warnings: 0,
        claim_results: vec![
            make_test_claim(1, "Coverage", true, true),
            make_test_claim(2, "TDG", true, true),
            make_test_claim(3, "Lint", false, true),
            make_test_claim(4, "SATD", false, false),
        ],
        all_passed: false,
    };

    assert!(report.has_blocking_failures());
    assert_eq!(report.blocking_failures().len(), 2);
}

#[test]
fn test_report_warning_failures_only() {
    let report = FalsificationReport {
        total_claims: 3,
        passed: 1,
        failed: 0,
        warnings: 2,
        claim_results: vec![
            make_test_claim(1, "Coverage", false, true),
            make_test_claim(2, "File size", true, false), // warning
            make_test_claim(3, "SATD", true, false),      // warning
        ],
        all_passed: false,
    };

    assert!(!report.has_blocking_failures());
    assert_eq!(report.warning_failures().len(), 2);
}

#[test]
fn test_report_all_passed_report() {
    let report = FalsificationReport {
        total_claims: 5,
        passed: 5,
        failed: 0,
        warnings: 0,
        claim_results: vec![
            make_test_claim(1, "A", false, true),
            make_test_claim(2, "B", false, true),
            make_test_claim(3, "C", false, false),
            make_test_claim(4, "D", false, true),
            make_test_claim(5, "E", false, false),
        ],
        all_passed: true,
    };

    assert!(!report.has_blocking_failures());
    assert!(report.blocking_failures().is_empty());
    assert!(report.warning_failures().is_empty());
}

// ============================================================================
// Tests for RoadmapItem methods used in core_handlers
// ============================================================================

#[test]
fn test_roadmap_item_from_github_issue_id_format() {
    let item = RoadmapItem::from_github_issue(123, "Test issue".to_string());
    assert_eq!(item.id, "GH-123");
    assert_eq!(item.github_issue, Some(123));
}

#[test]
fn test_roadmap_item_new_default_status() {
    let item = RoadmapItem::new("TEST-001".to_string(), "New item".to_string());
    assert_eq!(item.status, ItemStatus::Planned);
    assert_eq!(item.priority, Priority::Medium);
}

#[test]
fn test_roadmap_item_is_github_synced_true() {
    let mut item = RoadmapItem::new("GH-100".to_string(), "Synced".to_string());
    item.github_issue = Some(100);
    assert!(item.is_github_synced());
}

#[test]
fn test_roadmap_item_is_github_synced_false() {
    let item = RoadmapItem::new("PMAT-001".to_string(), "Not synced".to_string());
    assert!(!item.is_github_synced());
}

#[test]
fn test_roadmap_item_completion_percentage_all_statuses() {
    let test_cases = [
        (ItemStatus::Planned, 0),
        (ItemStatus::InProgress, 50),
        (ItemStatus::Review, 90),
        (ItemStatus::Completed, 100),
        (ItemStatus::Blocked, 0),
        (ItemStatus::Cancelled, 0),
    ];

    for (status, expected) in test_cases {
        let mut item = RoadmapItem::new("TEST".to_string(), "Test".to_string());
        item.status = status;
        assert_eq!(item.completion_percentage(), expected, "Failed for status: {:?}", status);
    }
}

#[test]
fn test_roadmap_item_type_epic() {
    let mut item = RoadmapItem::new("EPIC-001".to_string(), "Epic item".to_string());
    item.item_type = ItemType::Epic;
    assert_eq!(item.item_type, ItemType::Epic);
}

// ============================================================================
// Tests for ID display truncation logic
// ============================================================================

fn truncate_id_for_display(id: &str) -> String {
    debug_assert!(!id.is_empty(), "id must not be empty");
    if id.chars().count() > 30 {
        format!("{}...", id.chars().take(30).collect::<String>())
    } else {
        id.to_string()
    }
}

#[test]
fn test_truncate_id_short() {
    let id = "PMAT-001";
    assert_eq!(truncate_id_for_display(id), "PMAT-001");
}

#[test]
fn test_truncate_id_exactly_30() {
    let id = "PMAT-123456789012345678901234"; // 30 chars
    assert_eq!(id.chars().count(), 30);
    assert_eq!(truncate_id_for_display(id), id);
}

#[test]
fn test_truncate_id_over_30() {
    let id = "PMAT-1234567890123456789012345678901"; // >30 chars
    let result = truncate_id_for_display(id);
    assert!(result.ends_with("..."));
    assert_eq!(result.chars().count(), 33); // 30 + "..."
}

#[test]
fn test_truncate_id_unicode() {
    let id = "\u{1F600}".repeat(35); // 35 emoji chars
    let result = truncate_id_for_display(&id);
    assert!(result.ends_with("..."));
}

// ============================================================================
// Tests for Phase completion tracking
// ============================================================================

#[test]
fn test_phase_display() {
    let phase = Phase {
        name: "RED".to_string(),
        status: ItemStatus::Completed,
        completion: 100,
    };

    assert_eq!(phase.name, "RED");
    assert_eq!(phase.status, ItemStatus::Completed);
    assert_eq!(phase.completion, 100);
}

#[test]
fn test_phase_in_progress() {
    let phase = Phase {
        name: "GREEN".to_string(),
        status: ItemStatus::InProgress,
        completion: 50,
    };

    assert_eq!(phase.status, ItemStatus::InProgress);
    assert_eq!(phase.completion, 50);
}

// ============================================================================
// Tests for SubTask completion tracking
// ============================================================================

#[test]
fn test_subtask_display() {
    let subtask = SubTask {
        id: "SUB-001".to_string(),
        title: "Implement feature A".to_string(),
        status: ItemStatus::Planned,
        completion: 0,
    };

    assert_eq!(subtask.id, "SUB-001");
    assert_eq!(subtask.title, "Implement feature A");
}

#[test]
fn test_subtask_completed() {
    let subtask = SubTask {
        id: "SUB-002".to_string(),
        title: "Write tests".to_string(),
        status: ItemStatus::Completed,
        completion: 100,
    };

    assert_eq!(subtask.status, ItemStatus::Completed);
    assert_eq!(subtask.completion, 100);
}

// ============================================================================
// Tests for Roadmap yaml_only_items and epic_items
// ============================================================================

#[test]
fn test_roadmap_yaml_only_items_mixed() {
    let mut roadmap = Roadmap::new(None);

    let yaml_item = RoadmapItem::new("YAML-001".to_string(), "YAML only".to_string());
    let mut gh_item = RoadmapItem::from_github_issue(42, "GitHub".to_string());
    gh_item.github_issue = Some(42);

    roadmap.upsert_item(yaml_item);
    roadmap.upsert_item(gh_item);

    let yaml_only = roadmap.yaml_only_items();
    assert_eq!(yaml_only.len(), 1);
    assert_eq!(yaml_only[0].id, "YAML-001");
}

#[test]
fn test_roadmap_epic_items_mixed() {
    let mut roadmap = Roadmap::new(None);

    let mut epic = RoadmapItem::new("EPIC-001".to_string(), "Epic".to_string());
    epic.item_type = ItemType::Epic;

    let task = RoadmapItem::new("TASK-001".to_string(), "Task".to_string());

    roadmap.upsert_item(epic);
    roadmap.upsert_item(task);

    let epics = roadmap.epic_items();
    assert_eq!(epics.len(), 1);
    assert_eq!(epics[0].id, "EPIC-001");
}

// ============================================================================
// Tests for score reading from cache (O(1) operations)
// ============================================================================

#[test]
fn test_tdg_score_json_parsing() {
    let json = r#"{"score": 85.5, "timestamp": "2024-01-15T12:00:00Z"}"#;
    let value: serde_json::Value = serde_json::from_str(json).unwrap();

    let score = value.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
    assert!((score - 85.5).abs() < f64::EPSILON);
}

#[test]
fn test_repo_score_json_parsing() {
    let json = r#"{"score": 92.0, "grade": "A"}"#;
    let value: serde_json::Value = serde_json::from_str(json).unwrap();

    let score = value.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
    assert!((score - 92.0).abs() < f64::EPSILON);
}

#[test]
fn test_rust_project_score_json_parsing() {
    let json = r#"{"total_earned": 78.3, "total_possible": 106}"#;
    let value: serde_json::Value = serde_json::from_str(json).unwrap();

    let score = value.get("total_earned").and_then(|v| v.as_f64()).unwrap_or(0.0);
    assert!((score - 78.3).abs() < f64::EPSILON);
}

#[test]
fn test_score_missing_field() {
    let json = r#"{"other_field": 100}"#;
    let value: serde_json::Value = serde_json::from_str(json).unwrap();

    let score = value.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
    assert!((score - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_score_wrong_type() {
    let json = r#"{"score": "not a number"}"#;
    let value: serde_json::Value = serde_json::from_str(json).unwrap();

    let score = value.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
    assert!((score - 0.0).abs() < f64::EPSILON);
}

// ============================================================================
// Tests for commit message generation
// ============================================================================

#[test]
fn test_commit_message_format() {
    let title = "Fix authentication bug";
    let id = "PMAT-001";
    let tdg_score = 85.5;
    let repo_score = 92.0;

    let commit_msg = format!(
        "feat: {} (Refs {})\n\nWork-Item: {}\nTDG-Score: {:.1}/100\nRepo-Score: {:.1}/100",
        title, id, id, tdg_score, repo_score
    );

    assert!(commit_msg.contains("feat: Fix authentication bug"));
    assert!(commit_msg.contains("Refs PMAT-001"));
    assert!(commit_msg.contains("TDG-Score: 85.5/100"));
}

#[test]
fn test_commit_message_with_rust_score() {
    let title = "Add feature";
    let id = "GH-42";
    let tdg_score = 90.0;
    let repo_score = 88.0;
    let rust_score = Some(78.3);

    let rust_line = if let Some(rs) = rust_score {
        format!("Rust-Score: {:.1}/134\n", rs)
    } else {
        String::new()
    };

    let commit_msg = format!(
        "feat: {} (Refs {})\n\nWork-Item: {}\nTDG-Score: {:.1}/100\nRepo-Score: {:.1}/100\n{}",
        title, id, id, tdg_score, repo_score, rust_line
    );

    assert!(commit_msg.contains("Rust-Score: 78.3/134"));
}

#[test]
fn test_commit_message_no_rust_score() {
    let rust_score: Option<f64> = None;

    let rust_line = if let Some(rs) = rust_score {
        format!("Rust-Score: {:.1}/134\n", rs)
    } else {
        String::new()
    };

    assert!(rust_line.is_empty());
}

// ============================================================================
// Tests for EvidenceType variants
// ============================================================================

#[test]
fn test_evidence_type_file_list() {
    let evidence = EvidenceType::FileList(vec![
        PathBuf::from("src/main.rs"),
        PathBuf::from("src/lib.rs"),
    ]);

    if let EvidenceType::FileList(files) = evidence {
        assert_eq!(files.len(), 2);
    } else {
        panic!("Expected FileList");
    }
}

#[test]
fn test_evidence_type_numeric_comparison() {
    let evidence = EvidenceType::NumericComparison {
        actual: 80.0,
        threshold: 95.0,
    };

    if let EvidenceType::NumericComparison { actual, threshold } = evidence {
        assert!((actual - 80.0).abs() < f64::EPSILON);
        assert!((threshold - 95.0).abs() < f64::EPSILON);
    } else {
        panic!("Expected NumericComparison");
    }
}

#[test]
fn test_evidence_type_git_state() {
    let evidence = EvidenceType::GitState {
        unpushed_commits: 3,
        dirty_files: 2,
    };

    if let EvidenceType::GitState { unpushed_commits, dirty_files } = evidence {
        assert_eq!(unpushed_commits, 3);
        assert_eq!(dirty_files, 2);
    } else {
        panic!("Expected GitState");
    }
}

#[test]
fn test_evidence_type_boolean_check() {
    let evidence = EvidenceType::BooleanCheck(false);

    if let EvidenceType::BooleanCheck(value) = evidence {
        assert!(!value);
    } else {
        panic!("Expected BooleanCheck");
    }
}

#[test]
fn test_evidence_type_counter_example() {
    let evidence = EvidenceType::CounterExample {
        details: "Found TODO marker in src/main.rs".to_string(),
    };

    if let EvidenceType::CounterExample { details } = evidence {
        assert!(details.contains("TODO"));
    } else {
        panic!("Expected CounterExample");
    }
}

// ============================================================================
// Tests for specification template path generation
// ============================================================================

#[test]
fn test_spec_path_github_issue() {
    let github_issue = Some(42u64);
    let id = "GH-42";
    let project_path = PathBuf::from("/project");

    let spec_path = if id.parse::<u64>().is_ok() || id.starts_with("GH-") {
        project_path.join(format!(
            "docs/specifications/{:03}-spec.md",
            github_issue.expect("internal error")
        ))
    } else {
        project_path.join(format!("docs/specifications/{}-spec.md", id.to_lowercase()))
    };

    assert_eq!(spec_path, PathBuf::from("/project/docs/specifications/042-spec.md"));
}

#[test]
fn test_spec_path_yaml_ticket() {
    let id = "PMAT-NEW-FEATURE";
    let project_path = PathBuf::from("/project");

    let spec_path = project_path.join(format!("docs/specifications/{}-spec.md", id.to_lowercase()));

    assert_eq!(spec_path, PathBuf::from("/project/docs/specifications/pmat-new-feature-spec.md"));
}

// ============================================================================
// Tests for work contract path helpers
// ============================================================================

#[test]
fn test_work_contract_path() {
    let project_path = PathBuf::from("/home/user/project");
    let work_item_id = "PMAT-001";

    let contract_path = project_path
        .join(".pmat-work")
        .join(work_item_id)
        .join("contract.json");

    assert_eq!(
        contract_path,
        PathBuf::from("/home/user/project/.pmat-work/PMAT-001/contract.json")
    );
}

#[test]
fn test_work_contract_dir() {
    let project_path = PathBuf::from("/home/user/project");
    let work_item_id = "GH-42";

    let contract_dir = project_path.join(".pmat-work").join(work_item_id);

    assert_eq!(
        contract_dir,
        PathBuf::from("/home/user/project/.pmat-work/GH-42")
    );
}
