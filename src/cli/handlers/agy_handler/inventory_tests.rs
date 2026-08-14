//! Tests for the contract inventory `agy sync` reports.
//!
//! Every assertion here is on content the released 3.30.0 binary does not
//! produce at all: `agy sync` printed nothing on stdout and exited with a
//! refusal that named no input. Verified against the 1ac9feb5a binary — see the
//! evidence in MACS-017 (#984).

use super::*;
use std::fs;

/// A v5.0 contract with a Meyer triad, trimmed to the fields the projection
/// reads. Shaped after `.pmat-work/MACS-004/contract.json`.
fn v5_contract(id: &str) -> String {
    format!(
        r#"{{
  "version": "5.0",
  "work_item_id": "{id}",
  "created_at": "2026-07-02T14:01:30.420204037Z",
  "baseline_commit": "5cf0ac2f6ace904a90b891ce88194904f159060e",
  "baseline_file_manifest": {{ "files": {{}} }},
  "verification_level": "L3",
  "claims": [{{"hypothesis": "All baseline files still exist"}}],
  "require": [{{"id": "require.compiles", "description": "Project builds successfully"}}],
  "ensure": [
    {{"id": "ensure.tests_pass", "description": "All tests pass"}},
    {{"id": "ensure.coverage", "description": "Coverage does not regress"}}
  ],
  "invariant": [{{"id": "invariant.compiles", "description": "Project compiles at every checkpoint"}}],
  "implements": [{{"contract": "macs-ladder-v1", "equation": "parse_total_strict"}}],
  "agent": {{"model": "claude-fable-5", "harness": "claude_code"}}
}}"#
    )
}

fn write_contract(root: &std::path::Path, dir: &str, body: &str) {
    let d = root.join(dir);
    fs::create_dir_all(&d).expect("fixture dir");
    fs::write(d.join("contract.json"), body).expect("fixture contract");
}

#[test]
fn test_every_contract_read_is_named_in_the_report() {
    let temp = tempfile::TempDir::new().expect("tempdir");
    write_contract(temp.path(), "MACS-004", &v5_contract("MACS-004"));
    write_contract(temp.path(), "GH-230", &v5_contract("GH-230"));

    let inv = ContractInventory::scan(temp.path()).expect("scan");
    assert_eq!(inv.contracts.len(), 2, "both contracts must be read");

    let report = render(&inv);
    assert!(
        report.contains("contracts read: 2"),
        "report must state how many contracts it read:\n{report}"
    );
    assert!(
        report.contains("MACS-004") && report.contains("GH-230"),
        "every contract read must be named:\n{report}"
    );
}

#[test]
fn test_unparsable_contract_is_reported_not_silently_skipped() {
    let temp = tempfile::TempDir::new().expect("tempdir");
    write_contract(temp.path(), "GOOD-1", &v5_contract("GOOD-1"));
    write_contract(temp.path(), "BROKEN-1", "{ this is not json");

    let inv = ContractInventory::scan(temp.path()).expect("scan");
    assert_eq!(inv.contracts.len(), 1);
    assert_eq!(
        inv.unreadable.len(),
        1,
        "a contract that cannot be parsed is an input a transpiler would drop"
    );

    let report = render(&inv);
    assert!(
        report.contains("unreadable contracts: 1") && report.contains("BROKEN-1"),
        "the unreadable contract must be named in the report:\n{report}"
    );
}

#[test]
fn test_directory_without_a_contract_is_distinguished_from_an_unreadable_one() {
    let temp = tempfile::TempDir::new().expect("tempdir");
    write_contract(temp.path(), "HAS-ONE", &v5_contract("HAS-ONE"));
    fs::create_dir_all(temp.path().join("baseline-v1")).expect("bare dir");

    let inv = ContractInventory::scan(temp.path()).expect("scan");
    assert_eq!(inv.dirs_without_contract, vec!["baseline-v1".to_string()]);
    assert!(
        inv.unreadable.is_empty(),
        "a directory with no contract.json is not a parse failure"
    );
    assert!(render(&inv).contains("directories with no contract.json: 1"));
}

#[test]
fn test_triad_counts_and_rule_text_come_from_the_contract() {
    let temp = tempfile::TempDir::new().expect("tempdir");
    write_contract(temp.path(), "MACS-004", &v5_contract("MACS-004"));

    let inv = ContractInventory::scan(temp.path()).expect("scan");
    let c = &inv.contracts[0];
    assert_eq!((c.require, c.ensure, c.invariant), (1, 2, 1));
    assert_eq!(c.claims, 1);
    assert_eq!(c.harness.as_deref(), Some("claude_code"));
    assert_eq!(c.bindings, vec!["macs-ladder-v1::parse_total_strict"]);
    assert_eq!(
        c.rules.len(),
        4,
        "each clause description is a candidate rule"
    );

    let report = render(&inv);
    assert!(report.contains("1/2/1"), "triad counts:\n{report}");
    assert!(
        report.contains("- Project builds successfully"),
        "clause text is the only rule text a skill body could carry:\n{report}"
    );
    assert!(
        report.contains("4 distinct rules corpus-wide"),
        "the candidate rule set must be counted:\n{report}"
    );
}

#[test]
fn test_contract_with_no_rule_text_is_not_counted_as_ready() {
    let temp = tempfile::TempDir::new().expect("tempdir");
    write_contract(
        temp.path(),
        "PMAT-478",
        r#"{"version":"5.0","work_item_id":"PMAT-478",
            "falsifiable_claims":[{"hypothesis":"x"},{"hypothesis":"y"}],
            "verification_level":"L2"}"#,
    );

    let inv = ContractInventory::scan(temp.path()).expect("scan");
    let c = &inv.contracts[0];
    assert!(!c.has_rules(), "no require/ensure/invariant means no rules");
    assert!(c.legacy_claims_key, "claims live under the legacy key here");
    assert_eq!(c.claims, 2, "legacy claims still count as claims");
    assert_eq!(inv.with_rules(), 0);

    let report = render(&inv);
    assert!(
        report.contains("rules:       0/1"),
        "readiness must report 0 of 1, not omit the row:\n{report}"
    );
    assert!(
        report.contains("legacy `falsifiable_claims` key"),
        "a contract the reader only half-understands must say so:\n{report}"
    );
}

#[test]
fn test_report_states_that_no_description_can_be_sourced() {
    let temp = tempfile::TempDir::new().expect("tempdir");
    write_contract(temp.path(), "MACS-004", &v5_contract("MACS-004"));

    let report = render(&ContractInventory::scan(temp.path()).expect("scan"));
    assert!(
        report.contains("description: 0/1"),
        "the missing description is the transpiler's blocking gap, and it is a \
         measured 0, not an omitted row:\n{report}"
    );
}

#[test]
fn test_version_absent_is_rendered_as_absent_not_as_a_default() {
    let temp = tempfile::TempDir::new().expect("tempdir");
    write_contract(
        temp.path(),
        "PMAT-459",
        r#"{"work_item_id":"PMAT-459","claims":[],"verification_level":"L1"}"#,
    );

    let inv = ContractInventory::scan(temp.path()).expect("scan");
    assert_eq!(
        inv.contracts[0].version, "(none)",
        "a v4 contract has no version field; inventing \"4.0\" would be a claim \
         the file does not make"
    );
}

#[test]
fn test_scan_of_an_empty_work_dir_is_empty_not_a_pass() {
    let temp = tempfile::TempDir::new().expect("tempdir");
    let inv = ContractInventory::scan(temp.path()).expect("scan");
    assert!(inv.is_empty());
    assert_eq!(inv.bytes_read, 0);

    let report = render(&inv);
    assert!(report.contains("contracts read: 0"));
    assert!(
        report.contains("no contract.json found under"),
        "an empty corpus must say so, not render a table of 0/0 rows that reads \
         as everything checking out:\n{report}"
    );
    assert!(
        !report.contains("have a stable id") && !report.contains("candidate rules"),
        "an empty corpus must not render a readiness table or a rules section:\n{report}"
    );
}

#[test]
fn test_bytes_read_tracks_the_corpus_size() {
    let temp = tempfile::TempDir::new().expect("tempdir");
    let body = v5_contract("MACS-004");
    write_contract(temp.path(), "MACS-004", &body);
    let one = ContractInventory::scan(temp.path())
        .expect("scan")
        .bytes_read;
    write_contract(temp.path(), "MACS-005", &v5_contract("MACS-005"));
    let two = ContractInventory::scan(temp.path())
        .expect("scan")
        .bytes_read;

    assert_eq!(one, body.len() as u64);
    assert!(
        two > one,
        "bytes_read must respond to the corpus: {one} then {two}"
    );
}
