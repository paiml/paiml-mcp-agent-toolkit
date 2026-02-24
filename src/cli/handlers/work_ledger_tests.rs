#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::handlers::work_contract::{
        EvidenceType, FalsificationMethod, FalsificationResult,
    };

    fn make_report(all_passed: bool) -> FalsificationReport {
        let mut results = vec![ClaimResult {
            index: 1,
            hypothesis: "All baseline files still exist".to_string(),
            method: FalsificationMethod::ManifestIntegrity,
            result: FalsificationResult::passed("All 10 files present"),
            is_blocking: true,
        }];

        if !all_passed {
            results.push(ClaimResult {
                index: 2,
                hypothesis: "Total coverage >= 95%".to_string(),
                method: FalsificationMethod::AbsoluteCoverage,
                result: FalsificationResult::failed(
                    "80.0% < 95.0%",
                    EvidenceType::NumericComparison {
                        actual: 80.0,
                        threshold: 95.0,
                    },
                ),
                is_blocking: true,
            });
        }

        let passed = results.iter().filter(|r| !r.result.falsified).count();
        let failed = results
            .iter()
            .filter(|r| r.result.falsified && r.is_blocking)
            .count();

        FalsificationReport {
            total_claims: results.len(),
            passed,
            failed,
            warnings: 0,
            all_passed,
            claim_results: results,
        }
    }

    #[test]
    fn receipt_from_report_all_passed() {
        let report = make_report(true);
        let receipt = FalsificationReceipt::from_report(
            &report,
            "abc123".to_string(),
            "PMAT-100".to_string(),
            FalsificationTrigger::WorkComplete,
            None,
            None,
        );

        assert!(receipt.summary.allows_completion);
        assert_eq!(receipt.summary.passed, 1);
        assert_eq!(receipt.summary.failed, 0);
        assert_eq!(receipt.summary.overridden, 0);
        assert_eq!(receipt.git_sha, "abc123");
        assert_eq!(receipt.work_item_id, "PMAT-100");
        assert!(!receipt.content_hash.is_empty());
    }

    #[test]
    fn receipt_from_report_with_failures() {
        let report = make_report(false);
        let receipt = FalsificationReceipt::from_report(
            &report,
            "def456".to_string(),
            "PMAT-101".to_string(),
            FalsificationTrigger::WorkComplete,
            None,
            None,
        );

        assert!(!receipt.summary.allows_completion);
        assert_eq!(receipt.summary.passed, 1);
        assert_eq!(receipt.summary.failed, 1);
        assert_eq!(receipt.verdicts.len(), 2);
    }

    #[test]
    fn receipt_from_report_with_overrides() {
        let report = make_report(false);
        let overrides = vec!["coverage".to_string()];
        let ticket = "DEBT-001".to_string();
        let receipt = FalsificationReceipt::from_report(
            &report,
            "ghi789".to_string(),
            "PMAT-102".to_string(),
            FalsificationTrigger::WorkComplete,
            Some(&overrides),
            Some(&ticket),
        );

        assert!(receipt.summary.allows_completion);
        assert_eq!(receipt.summary.overridden, 1);
        assert_eq!(receipt.overrides[0].claim_id, "coverage");
        assert_eq!(receipt.overrides[0].ticket, "DEBT-001");
    }

    #[test]
    fn content_hash_stable() {
        let report = make_report(true);
        let r1 = FalsificationReceipt::from_report(
            &report,
            "abc".to_string(),
            "X-1".to_string(),
            FalsificationTrigger::ManualCli,
            None,
            None,
        );
        // Hash is based on id + timestamp which are unique, so verify it's non-empty and valid hex
        assert!(!r1.content_hash.is_empty());
        assert!(r1.content_hash.chars().all(|c| c.is_ascii_hexdigit()));
        // Re-computing on same receipt should match
        assert!(r1.verify_integrity());
    }

    #[test]
    fn content_hash_detects_tampering() {
        let report = make_report(true);
        let mut receipt = FalsificationReceipt::from_report(
            &report,
            "abc".to_string(),
            "X-1".to_string(),
            FalsificationTrigger::CiPipeline,
            None,
            None,
        );

        assert!(receipt.verify_integrity());

        // Tamper with a field
        receipt.git_sha = "tampered".to_string();
        assert!(!receipt.verify_integrity());
    }

    #[test]
    fn freshness_matches_sha() {
        let report = make_report(true);
        let receipt = FalsificationReceipt::from_report(
            &report,
            "abc123".to_string(),
            "X-1".to_string(),
            FalsificationTrigger::WorkComplete,
            None,
            None,
        );

        assert!(receipt.is_fresh("abc123", MAX_RECEIPT_AGE_SECS));
    }

    #[test]
    fn freshness_stale_sha() {
        let report = make_report(true);
        let receipt = FalsificationReceipt::from_report(
            &report,
            "abc123".to_string(),
            "X-1".to_string(),
            FalsificationTrigger::WorkComplete,
            None,
            None,
        );

        assert!(!receipt.is_fresh("different_sha", MAX_RECEIPT_AGE_SECS));
    }

    #[test]
    fn freshness_expired() {
        let report = make_report(true);
        let mut receipt = FalsificationReceipt::from_report(
            &report,
            "abc123".to_string(),
            "X-1".to_string(),
            FalsificationTrigger::WorkComplete,
            None,
            None,
        );

        // Set timestamp to 48 hours ago
        let old_time = chrono::Utc::now() - chrono::Duration::hours(48);
        receipt.timestamp = old_time.to_rfc3339();

        assert!(!receipt.is_fresh("abc123", MAX_RECEIPT_AGE_SECS));
    }

    #[test]
    fn ledger_persist_and_load() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let ledger = FalsificationLedger::new(temp_dir.path());

        let report = make_report(true);
        let receipt = FalsificationReceipt::from_report(
            &report,
            "abc123".to_string(),
            "PMAT-200".to_string(),
            FalsificationTrigger::WorkComplete,
            None,
            None,
        );

        // Persist
        let path = ledger.persist_receipt(&receipt).unwrap();
        assert!(path.exists());

        // Load latest
        let loaded = ledger.latest_receipt("PMAT-200").unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.id, receipt.id);
        assert_eq!(loaded.git_sha, "abc123");
        assert!(loaded.verify_integrity());
    }

    #[test]
    fn ledger_append_only() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let ledger = FalsificationLedger::new(temp_dir.path());

        let report = make_report(true);
        let r1 = FalsificationReceipt::from_report(
            &report,
            "sha1".to_string(),
            "ITEM-1".to_string(),
            FalsificationTrigger::WorkComplete,
            None,
            None,
        );
        let r2 = FalsificationReceipt::from_report(
            &report,
            "sha2".to_string(),
            "ITEM-2".to_string(),
            FalsificationTrigger::ManualCli,
            None,
            None,
        );

        ledger.append_to_ledger(&r1).unwrap();
        ledger.append_to_ledger(&r2).unwrap();

        let ledger_path = temp_dir.path().join(".pmat-work/ledger.jsonl");
        let content = std::fs::read_to_string(&ledger_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2);

        // Verify each line is valid JSON
        let entry1: LedgerEntry = serde_json::from_str(lines[0]).unwrap();
        let entry2: LedgerEntry = serde_json::from_str(lines[1]).unwrap();
        assert_eq!(entry1.work_item_id, "ITEM-1");
        assert_eq!(entry2.work_item_id, "ITEM-2");
    }

    #[test]
    fn latest_receipt_returns_most_recent() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let ledger = FalsificationLedger::new(temp_dir.path());

        let report = make_report(true);
        let r1 = FalsificationReceipt::from_report(
            &report,
            "sha_old".to_string(),
            "PMAT-300".to_string(),
            FalsificationTrigger::WorkComplete,
            None,
            None,
        );

        // Small delay to ensure different timestamps in filenames
        ledger.persist_receipt(&r1).unwrap();

        // Create second receipt with different SHA
        let r2 = FalsificationReceipt::from_report(
            &report,
            "sha_new".to_string(),
            "PMAT-300".to_string(),
            FalsificationTrigger::WorkComplete,
            None,
            None,
        );
        ledger.persist_receipt(&r2).unwrap();

        let latest = ledger.latest_receipt("PMAT-300").unwrap().unwrap();
        assert_eq!(latest.git_sha, "sha_new");
    }

    #[test]
    fn has_fresh_receipt_true() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let ledger = FalsificationLedger::new(temp_dir.path());

        let report = make_report(true);
        let receipt = FalsificationReceipt::from_report(
            &report,
            "current_head".to_string(),
            "PMAT-400".to_string(),
            FalsificationTrigger::WorkComplete,
            None,
            None,
        );
        ledger.persist_receipt(&receipt).unwrap();

        assert!(ledger
            .has_fresh_receipt("PMAT-400", "current_head")
            .unwrap());
    }

    #[test]
    fn has_fresh_receipt_false_stale_sha() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let ledger = FalsificationLedger::new(temp_dir.path());

        let report = make_report(true);
        let receipt = FalsificationReceipt::from_report(
            &report,
            "old_sha".to_string(),
            "PMAT-401".to_string(),
            FalsificationTrigger::WorkComplete,
            None,
            None,
        );
        ledger.persist_receipt(&receipt).unwrap();

        assert!(!ledger.has_fresh_receipt("PMAT-401", "new_sha").unwrap());
    }

    #[test]
    fn has_fresh_receipt_false_no_receipts() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let ledger = FalsificationLedger::new(temp_dir.path());

        assert!(!ledger.has_fresh_receipt("PMAT-999", "any_sha").unwrap());
    }

    #[test]
    fn verify_integrity_report() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let ledger = FalsificationLedger::new(temp_dir.path());

        let report = make_report(true);
        let receipt = FalsificationReceipt::from_report(
            &report,
            "abc".to_string(),
            "PMAT-500".to_string(),
            FalsificationTrigger::WorkComplete,
            None,
            None,
        );
        ledger.persist_receipt(&receipt).unwrap();

        let integrity = ledger.verify_integrity("PMAT-500").unwrap();
        assert_eq!(integrity.total, 1);
        assert_eq!(integrity.valid, 1);
        assert_eq!(integrity.tampered, 0);
        assert_eq!(integrity.missing, 0);
    }

    #[test]
    fn verify_integrity_detects_tampering() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let ledger = FalsificationLedger::new(temp_dir.path());

        let report = make_report(true);
        let receipt = FalsificationReceipt::from_report(
            &report,
            "abc".to_string(),
            "PMAT-501".to_string(),
            FalsificationTrigger::WorkComplete,
            None,
            None,
        );
        let path = ledger.persist_receipt(&receipt).unwrap();

        // Tamper with the file
        let mut content: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        content["git_sha"] = serde_json::Value::String("tampered".to_string());
        std::fs::write(&path, serde_json::to_string_pretty(&content).unwrap()).unwrap();

        let integrity = ledger.verify_integrity("PMAT-501").unwrap();
        assert_eq!(integrity.total, 1);
        assert_eq!(integrity.tampered, 1);
        assert_eq!(integrity.valid, 0);
    }

    #[test]
    fn hypothesis_to_claim_id_mapping() {
        assert_eq!(
            hypothesis_to_claim_id("All baseline files still exist"),
            "manifest"
        );
        assert_eq!(hypothesis_to_claim_id("Total coverage >= 95%"), "coverage");
        assert_eq!(hypothesis_to_claim_id("TDG score >= baseline"), "tdg");
        assert_eq!(
            hypothesis_to_claim_id("No coverage exclusion gaming"),
            "coverage-gaming"
        );
        assert_eq!(hypothesis_to_claim_id("make lint passes"), "lint");
        assert_eq!(
            hypothesis_to_claim_id("No dead code introduced"),
            "dead-code"
        );
        // v3.1 defect churn claims
        assert_eq!(
            hypothesis_to_claim_id("All match arm variants have test coverage"),
            "variant-coverage"
        );
        assert_eq!(
            hypothesis_to_claim_id("No fix-after-fix chains exceed limit"),
            "fix-chain"
        );
        assert_eq!(
            hypothesis_to_claim_id("Cross-crate integration tests pass"),
            "cross-crate"
        );
        assert_eq!(
            hypothesis_to_claim_id("No performance regressions detected"),
            "regression-gate"
        );
    }

    #[test]
    fn ledger_entry_from_receipt() {
        let report = make_report(true);
        let receipt = FalsificationReceipt::from_report(
            &report,
            "abc".to_string(),
            "X-1".to_string(),
            FalsificationTrigger::PreCommit,
            None,
            None,
        );
        let entry = LedgerEntry::from_receipt(&receipt);
        assert_eq!(entry.receipt_id, receipt.id);
        assert_eq!(entry.work_item_id, "X-1");
        assert_eq!(entry.trigger, FalsificationTrigger::PreCommit);
        assert!(entry.allows_completion);
    }

    #[test]
    fn receipt_trigger_variants() {
        // Verify all trigger variants serialize/deserialize
        let triggers = vec![
            FalsificationTrigger::WorkComplete,
            FalsificationTrigger::ManualCli,
            FalsificationTrigger::CiPipeline,
            FalsificationTrigger::McpTool,
            FalsificationTrigger::PreCommit,
        ];
        for trigger in triggers {
            let json = serde_json::to_string(&trigger).unwrap();
            let deserialized: FalsificationTrigger = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, trigger);
        }
    }
}
