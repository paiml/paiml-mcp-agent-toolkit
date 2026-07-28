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
            unmeasured: 0,
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
    // ========================================================================
    // MACS-001 — provenance types, schema_version, canonical v2 hashing
    // RED tests named per spec card §5.2 MACS-001
    // ========================================================================

    fn make_provenance() -> AgentProvenance {
        AgentProvenance {
            model: "claude-fable-5".to_string(),
            effort: "high".to_string(),
            harness: AgentHarness::ClaudeCode,
            workflow_id: None,
            parent: None,
            task_id: None,
            conversation_id: None,
            source: ProvenanceSource::Declared,
        }
    }

    fn make_v2_receipt() -> FalsificationReceipt {
        let report = make_report(true);
        let mut receipt = FalsificationReceipt::from_report(
            &report,
            "abc123".to_string(),
            "MACS-TEST".to_string(),
            FalsificationTrigger::WorkComplete,
            None,
            None,
        );
        receipt.agent = Some(make_provenance());
        receipt.agent_events = vec![AgentEvent::ModelSwitch {
            at: "2026-07-02T12:00:00Z".to_string(),
            from: "claude-fable-5".to_string(),
            to: "claude-opus-4-8".to_string(),
        }];
        receipt.content_hash = receipt.compute_content_hash();
        receipt
    }

    #[test]
    fn new_receipts_are_schema_version_2() {
        let report = make_report(true);
        let receipt = FalsificationReceipt::from_report(
            &report,
            "abc123".to_string(),
            "MACS-TEST".to_string(),
            FalsificationTrigger::WorkComplete,
            None,
            None,
        );
        assert_eq!(receipt.schema_version, 2);
        assert!(receipt.agent.is_none());
        assert!(receipt.agent_events.is_empty());
        assert!(receipt.verify_integrity());
    }

    #[test]
    fn v2_receipt_with_agent_roundtrips_and_verifies() {
        let receipt = make_v2_receipt();
        assert!(receipt.verify_integrity());
        let json = serde_json::to_string(&receipt).expect("receipt serializes");
        let parsed: FalsificationReceipt = serde_json::from_str(&json).expect("receipt parses");
        assert!(parsed.verify_integrity());
        assert_eq!(parsed.agent, receipt.agent);
        assert_eq!(parsed.agent_events, receipt.agent_events);
        assert_eq!(parsed.schema_version, 2);
    }

    #[test]
    fn nine_field_v1_json_defaults_to_schema_version_1() {
        // A pre-MACS receipt (exactly the nine original fields) must parse
        // with schema_version=1, agent=None, agent_events=[].
        let report = make_report(true);
        let mut receipt = FalsificationReceipt::from_report(
            &report,
            "abc123".to_string(),
            "LEGACY-1".to_string(),
            FalsificationTrigger::WorkComplete,
            None,
            None,
        );
        // Rewrite as v1: strip new fields from the JSON entirely.
        receipt.schema_version = 1;
        receipt.content_hash = receipt.compute_content_hash();
        let mut value = serde_json::to_value(&receipt).expect("to_value");
        let obj = value.as_object_mut().expect("object");
        obj.remove("schema_version");
        obj.remove("agent");
        obj.remove("agent_events");
        let parsed: FalsificationReceipt =
            serde_json::from_value(value).expect("v1 receipt parses");
        assert_eq!(parsed.schema_version, 1);
        assert!(parsed.agent.is_none());
        assert!(parsed.agent_events.is_empty());
        assert!(parsed.verify_integrity(), "v1 rules must verify v1 receipts");
    }

    #[test]
    fn v1_receipts_still_verify() {
        // Golden fixtures: real pre-MACS receipts whose content_hash was
        // written by the v1 (byte-concat) rules.
        let root = env!("CARGO_MANIFEST_DIR");
        let dir = std::path::Path::new(root).join("fixtures/receipts/v1");
        let mut checked = 0usize;
        for entry in std::fs::read_dir(&dir).expect("fixtures/receipts/v1 exists") {
            let path = entry.expect("dir entry").path();
            if path.extension().is_none_or(|e| e != "json") {
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("fixture readable");
            let receipt: FalsificationReceipt =
                serde_json::from_str(&text).expect("v1 fixture parses");
            assert_eq!(receipt.schema_version, 1, "{}", path.display());
            assert!(receipt.verify_integrity(), "{} must verify", path.display());
            checked += 1;
        }
        assert!(checked >= 1, "at least one v1 fixture committed");
    }

    #[test]
    fn hash_stability_golden_v2() {
        // Golden fixtures: v2 receipts with pinned content hashes. Byte-stable
        // across OS/arch (contracts/macs-provenance-v1.yaml#hash_stability).
        let root = env!("CARGO_MANIFEST_DIR");
        let dir = std::path::Path::new(root).join("fixtures/receipts/v2");
        let mut checked = 0usize;
        for entry in std::fs::read_dir(&dir).expect("fixtures/receipts/v2 exists") {
            let path = entry.expect("dir entry").path();
            if path.extension().is_none_or(|e| e != "json") {
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("fixture readable");
            let receipt: FalsificationReceipt =
                serde_json::from_str(&text).expect("v2 fixture parses");
            assert_eq!(receipt.schema_version, 2, "{}", path.display());
            assert!(
                receipt.verify_integrity(),
                "{} content_hash must re-derive under v2 canonical rules",
                path.display()
            );
            checked += 1;
        }
        assert!(checked >= 2, "at least two v2 fixtures committed");
    }

    #[test]
    fn canonical_json_sorted_keys() {
        // Keys sort regardless of insertion order (serde_json in this
        // workspace is feature-unified with preserve_order).
        let a: serde_json::Value =
            serde_json::from_str(r#"{"zeta":1,"alpha":{"nested_z":true,"nested_a":null},"mid":[2,1]}"#)
                .expect("parse a");
        let b: serde_json::Value =
            serde_json::from_str(r#"{"alpha":{"nested_a":null,"nested_z":true},"mid":[2,1],"zeta":1}"#)
                .expect("parse b");
        assert_eq!(canonical_json(&a), canonical_json(&b));
        assert_eq!(
            canonical_json(&a),
            r#"{"alpha":{"nested_a":null,"nested_z":true},"mid":[2,1],"zeta":1}"#
        );
    }

    #[test]
    fn v2_hash_ignores_field_insertion_order() {
        let receipt = make_v2_receipt();
        // Round-trip through JSON with reversed key order: hash must not move.
        let json = serde_json::to_string(&receipt).expect("serializes");
        let parsed: FalsificationReceipt = serde_json::from_str(&json).expect("parses");
        assert_eq!(parsed.content_hash, receipt.content_hash);
        assert_eq!(parsed.compute_content_hash(), receipt.compute_content_hash());
    }

    #[test]
    fn v2_hash_covers_agent_fields() {
        // Tampering the agent block must break verify_integrity (unlike v1,
        // where provenance did not exist to cover).
        let mut receipt = make_v2_receipt();
        assert!(receipt.verify_integrity());
        if let Some(agent) = receipt.agent.as_mut() {
            agent.model = "tampered-model".to_string();
        }
        assert!(!receipt.verify_integrity(), "agent tamper must be detected");
    }

    #[test]
    fn legacy_jsonl_parse() {
        // Pre-MACS ledger.jsonl lines (10 fields, no agent summary) must parse.
        let line = r#"{"receipt_id":"0197-x","work_item_id":"PMAT-521","timestamp":"2026-04-16T19:42:51Z","git_sha":"abc","trigger":"WorkComplete","passed":17,"failed":5,"overridden":5,"allows_completion":true,"content_hash":"deadbeef"}"#;
        let entry: LedgerEntry = serde_json::from_str(line).expect("legacy ledger line parses");
        assert_eq!(entry.work_item_id, "PMAT-521");
        assert!(entry.allows_completion);
    }

    proptest::proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(16))]

        #[test]
        fn provenance_roundtrip_prop(
            model in "[a-z0-9-]{1,24}",
            effort in proptest::sample::select(vec!["low", "medium", "high", "xhigh", "max"]),
            harness_pick in 0usize..6,
            other in "[a-z-]{1,12}",
            workflow_id in proptest::option::of("[a-z0-9]{4,12}"),
            parent in proptest::option::of("[a-z0-9]{4,12}"),
            source_pick in 0usize..3,
            note in proptest::option::of("[ -~]{0,40}"),
            subagents in 0u32..1000,
        ) {
            let harness = match harness_pick {
                0 => AgentHarness::ClaudeCode,
                1 => AgentHarness::ClaudeAgentSdk,
                2 => AgentHarness::UltracodeWorkflow,
                3 => AgentHarness::CiPipeline,
                4 => AgentHarness::Human,
                _ => AgentHarness::Other(other.clone()),
            };
            let source = match source_pick {
                0 => ProvenanceSource::Declared,
                1 => ProvenanceSource::Detected,
                _ => ProvenanceSource::Mixed,
            };
            let provenance = AgentProvenance {
                model: model.clone(),
                effort: effort.to_string(),
                harness,
                workflow_id,
                parent,
                task_id: None,
                conversation_id: None,
                source,
            };
            let events = vec![
                AgentEvent::Refusal { at: "2026-07-02T00:00:00Z".to_string(), note },
                AgentEvent::WorkflowSpawn {
                    at: "2026-07-02T00:00:01Z".to_string(),
                    workflow_id: "wf-prop".to_string(),
                    subagents,
                },
            ];

            // Type-level roundtrip
            let pj = serde_json::to_string(&provenance).expect("provenance serializes");
            let p2: AgentProvenance = serde_json::from_str(&pj).expect("provenance parses");
            proptest::prop_assert_eq!(&p2, &provenance);
            let ej = serde_json::to_string(&events).expect("events serialize");
            let e2: Vec<AgentEvent> = serde_json::from_str(&ej).expect("events parse");
            proptest::prop_assert_eq!(&e2, &events);

            // Receipt-level roundtrip: deserialize(serialize(r)) = r (v2)
            let report = make_report(true);
            let mut receipt = FalsificationReceipt::from_report(
                &report,
                "sha-prop".to_string(),
                "MACS-PROP".to_string(),
                FalsificationTrigger::WorkComplete,
                None,
                None,
            );
            receipt.agent = Some(provenance);
            receipt.agent_events = events;
            receipt.content_hash = receipt.compute_content_hash();
            let rj = serde_json::to_string(&receipt).expect("receipt serializes");
            let r2: FalsificationReceipt = serde_json::from_str(&rj).expect("receipt parses");
            proptest::prop_assert!(r2.verify_integrity());
            proptest::prop_assert_eq!(r2.compute_content_hash(), receipt.content_hash);
        }
    }

    /// Regenerates the committed v2 golden fixtures. Run manually:
    /// cargo test --lib generate_v2_receipt_fixtures -- --ignored
    #[test]
    #[ignore = "fixture generator, not a test"]
    fn generate_v2_receipt_fixtures() {
        let root = env!("CARGO_MANIFEST_DIR");
        let dir = std::path::Path::new(root).join("fixtures/receipts/v2");
        std::fs::create_dir_all(&dir).expect("create fixtures dir");

        let mut a = make_v2_receipt();
        a.id = "0197f000-0000-7000-8000-00000000000a".to_string();
        a.git_sha = "bab0f9af9000000000000000000000000000000a".to_string();
        a.timestamp = "2026-07-02T12:00:00+00:00".to_string();
        a.content_hash = a.compute_content_hash();
        std::fs::write(
            dir.join("receipt-golden-a.json"),
            serde_json::to_string_pretty(&a).expect("serialize a"),
        )
        .expect("write a");

        let mut b = make_v2_receipt();
        b.id = "0197f000-0000-7000-8000-00000000000b".to_string();
        b.git_sha = "bab0f9af9000000000000000000000000000000b".to_string();
        b.timestamp = "2026-07-02T12:00:01+00:00".to_string();
        b.agent = Some(AgentProvenance {
            model: "claude-opus-4-8".to_string(),
            effort: "xhigh".to_string(),
            harness: AgentHarness::UltracodeWorkflow,
            workflow_id: Some("wf_golden".to_string()),
            parent: Some("session-root".to_string()),
            task_id: None,
            conversation_id: None,
            source: ProvenanceSource::Mixed,
        });
        b.agent_events = vec![
            AgentEvent::Refusal {
                at: "2026-07-02T11:59:00Z".to_string(),
                note: Some("flagged request".to_string()),
            },
            AgentEvent::SessionRestart { at: "2026-07-02T11:59:30Z".to_string() },
            AgentEvent::WorkflowSpawn {
                at: "2026-07-02T11:59:45Z".to_string(),
                workflow_id: "wf_golden".to_string(),
                subagents: 10,
            },
        ];
        b.content_hash = b.compute_content_hash();
        std::fs::write(
            dir.join("receipt-golden-b.json"),
            serde_json::to_string_pretty(&b).expect("serialize b"),
        )
        .expect("write b");
    }

    // ========================================================================
    // MACS-002 — provenance capture: declared-first resolution, ledger summary
    // ========================================================================

    #[test]
    fn declared_flags_win_over_env() {
        let declared = DeclaredAgent {
            model: Some("claude-fable-5".to_string()),
            effort: Some("high".to_string()),
            harness: Some("claude-code".to_string()),
            workflow_id: None,
            parent: None,
        };
        let env = |k: &str| {
            if k == "CLAUDE_CODE_EFFORT_LEVEL" {
                Some("low".to_string())
            } else {
                None
            }
        };
        let p = resolve_agent_provenance_with_env(&declared, &env).expect("provenance");
        assert_eq!(p.effort, "high", "declared effort beats detected env");
        assert_eq!(p.source, ProvenanceSource::Declared);
        assert_eq!(p.harness, AgentHarness::ClaudeCode);
    }

    #[test]
    fn env_detected_marked_advisory() {
        let env = |k: &str| {
            if k == "CLAUDE_CODE_EFFORT_LEVEL" {
                Some("xhigh".to_string())
            } else {
                None
            }
        };
        // Nothing declared: detection alone yields source=detected
        let p = resolve_agent_provenance_with_env(&DeclaredAgent::default(), &env)
            .expect("detected provenance");
        assert_eq!(p.effort, "xhigh");
        assert_eq!(p.source, ProvenanceSource::Detected);
        assert_eq!(p.harness, AgentHarness::ClaudeCode);
        assert_eq!(p.model, "unknown");
        // Partially declared: detection fills gaps and marks mixed
        let declared = DeclaredAgent {
            model: Some("claude-fable-5".to_string()),
            ..Default::default()
        };
        let p = resolve_agent_provenance_with_env(&declared, &env).expect("mixed provenance");
        assert_eq!(p.source, ProvenanceSource::Mixed);
        assert_eq!(p.effort, "xhigh");
        assert_eq!(p.model, "claude-fable-5");
    }

    #[test]
    fn missing_provenance_yields_none_v2() {
        let env = |_: &str| None;
        assert!(
            resolve_agent_provenance_with_env(&DeclaredAgent::default(), &env).is_none(),
            "no flags + no env => no provenance"
        );
        let receipt = FalsificationReceipt::from_report(
            &make_report(true),
            "sha".to_string(),
            "T-NONE".to_string(),
            FalsificationTrigger::WorkComplete,
            None,
            None,
        );
        assert_eq!(receipt.schema_version, 2);
        assert!(receipt.agent.is_none());
        assert!(receipt.verify_integrity());
    }

    #[test]
    fn ledger_entry_carries_agent() {
        let receipt = make_v2_receipt();
        let entry = LedgerEntry::from_receipt(&receipt);
        let agent = entry.agent.expect("agent summary on ledger entry");
        assert_eq!(agent.model, "claude-fable-5");
        assert_eq!(agent.effort, "high");
        assert_eq!(agent.harness, AgentHarness::ClaudeCode);
        // Receipt without provenance yields no summary
        let mut bare = make_v2_receipt();
        bare.agent = None;
        assert!(LedgerEntry::from_receipt(&bare).agent.is_none());
    }

    #[test]
    fn with_agent_reseals_content_hash() {
        let base = FalsificationReceipt::from_report(
            &make_report(true),
            "sha".to_string(),
            "T-SEAL".to_string(),
            FalsificationTrigger::WorkComplete,
            None,
            None,
        );
        let sealed = base.clone().with_agent(Some(make_provenance()), Vec::new());
        assert!(sealed.verify_integrity(), "hash must be recomputed");
        assert_ne!(sealed.content_hash, base.content_hash);
        assert!(sealed.agent.is_some());
    }
    // ========================================================================
    // MACS-003 — work events: append-only journal, refusal blocking, acks
    // ========================================================================

    #[test]
    fn event_appends_to_active_ticket() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let ledger = FalsificationLedger::new(temp_dir.path());
        let id = ledger
            .append_event(
                "MACS-EV",
                AgentEvent::Refusal {
                    at: "2026-07-02T00:00:00Z".to_string(),
                    note: Some("turn ended flagged".to_string()),
                },
            )
            .expect("append event");
        assert!(id.starts_with("ev-"));
        let events = ledger.load_events("MACS-EV").expect("load events");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, id);
        assert!(matches!(events[0].event, AgentEvent::Refusal { .. }));
    }

    #[test]
    fn event_refusal_blocks_complete() {
        // The gate is FalsificationLedger::unacked_refusals — non-empty means
        // handle_work_complete bails (wired before the fresh-receipt path).
        let temp_dir = tempfile::TempDir::new().unwrap();
        let ledger = FalsificationLedger::new(temp_dir.path());
        ledger
            .append_event(
                "MACS-EV",
                AgentEvent::Refusal {
                    at: "2026-07-02T00:00:00Z".to_string(),
                    note: None,
                },
            )
            .expect("append refusal");
        let unacked = ledger.unacked_refusals("MACS-EV").expect("query unacked");
        assert_eq!(unacked.len(), 1, "refusal must block until acknowledged");
    }

    #[test]
    fn ack_event_unblocks_with_reason() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let ledger = FalsificationLedger::new(temp_dir.path());
        let refusal_id = ledger
            .append_event(
                "MACS-EV",
                AgentEvent::Refusal {
                    at: "2026-07-02T00:00:00Z".to_string(),
                    note: None,
                },
            )
            .expect("append refusal");
        ledger
            .append_event(
                "MACS-EV",
                AgentEvent::Ack {
                    at: "2026-07-02T00:01:00Z".to_string(),
                    ack_of: refusal_id.clone(),
                    reason: "root cause: flagged phrasing; rephrased and continued".to_string(),
                },
            )
            .expect("append ack");
        assert!(
            ledger.unacked_refusals("MACS-EV").expect("query").is_empty(),
            "acked refusal must no longer block"
        );
        // The ack itself is a recorded, reason-carrying event
        let events = ledger.load_events("MACS-EV").expect("load");
        assert_eq!(events.len(), 2);
        match &events[1].event {
            AgentEvent::Ack { ack_of, reason, .. } => {
                assert_eq!(ack_of, &refusal_id);
                assert!(!reason.is_empty());
            }
            other => panic!("expected Ack, got {other:?}"),
        }
    }

    #[test]
    fn model_switch_recorded_from_to() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let ledger = FalsificationLedger::new(temp_dir.path());
        ledger
            .append_event(
                "MACS-EV",
                AgentEvent::ModelSwitch {
                    at: "2026-07-02T00:00:00Z".to_string(),
                    from: "claude-fable-5".to_string(),
                    to: "claude-opus-4-8".to_string(),
                },
            )
            .expect("append switch");
        let events = ledger.load_events("MACS-EV").expect("load");
        match &events[0].event {
            AgentEvent::ModelSwitch { from, to, .. } => {
                assert_eq!(from, "claude-fable-5");
                assert_eq!(to, "claude-opus-4-8");
            }
            other => panic!("expected ModelSwitch, got {other:?}"),
        }
        // Model switches never block completion
        assert!(ledger.unacked_refusals("MACS-EV").expect("query").is_empty());
    }

    #[test]
    fn events_append_only() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let ledger = FalsificationLedger::new(temp_dir.path());
        for i in 0..3 {
            ledger
                .append_event(
                    "MACS-EV",
                    AgentEvent::SessionRestart {
                        at: format!("2026-07-02T00:00:0{i}Z"),
                    },
                )
                .expect("append");
            let events = ledger.load_events("MACS-EV").expect("load");
            assert_eq!(events.len(), i + 1, "every append adds exactly one line");
        }
        // Prior records are byte-stable: ids and order preserved
        let events = ledger.load_events("MACS-EV").expect("load");
        let ids: Vec<&str> = events.iter().map(|r| r.id.as_str()).collect();
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        assert_eq!(ids, sorted, "uuid7 record ids are time-ordered");
    }

    // ========================================================================
    // MACS-016 — ledger verify capstone
    // ========================================================================

    fn persist_v2(ledger: &FalsificationLedger, ticket: &str, model: &str) -> FalsificationReceipt {
        let report = make_report(true);
        let mut receipt = FalsificationReceipt::from_report(
            &report,
            "sha".to_string(),
            ticket.to_string(),
            FalsificationTrigger::WorkComplete,
            None,
            None,
        );
        receipt = receipt.with_agent(
            Some(AgentProvenance {
                model: model.to_string(),
                effort: "xhigh".to_string(),
                harness: AgentHarness::ClaudeCode,
                workflow_id: None,
                parent: None,
                task_id: None,
                conversation_id: None,
                source: ProvenanceSource::Declared,
            }),
            Vec::new(),
        );
        ledger.persist_receipt(&receipt).unwrap();
        ledger.append_to_ledger(&receipt).unwrap();
        receipt
    }

    #[test]
    fn verify_recomputes_all_hashes_v1_and_v2() {
        let temp = tempfile::TempDir::new().unwrap();
        let ledger = FalsificationLedger::new(temp.path());
        // A v2 receipt with provenance...
        persist_v2(&ledger, "MACS-001", "claude-fable-5");
        // ...and a v1 receipt (schema_version 1, legacy hash rules).
        let mut v1 = FalsificationReceipt::from_report(
            &make_report(true),
            "sha".to_string(),
            "LEGACY-1".to_string(),
            FalsificationTrigger::WorkComplete,
            None,
            None,
        );
        v1.schema_version = 1;
        v1.content_hash = v1.compute_content_hash();
        ledger.persist_receipt(&v1).unwrap();

        let v = ledger.verify_all().unwrap();
        assert_eq!(v.total_receipts, 2);
        assert_eq!(v.verified, 2, "both v1 and v2 verify under their own rules");
        assert!(v.ok());
    }

    #[test]
    fn verify_red_on_tampered_entry() {
        let temp = tempfile::TempDir::new().unwrap();
        let ledger = FalsificationLedger::new(temp.path());
        let receipt = persist_v2(&ledger, "MACS-002", "claude-fable-5");
        // Tamper with the persisted receipt file: flip the model, keep the hash.
        let dir = temp.path().join(".pmat-work/MACS-002/falsification");
        let file = std::fs::read_dir(&dir).unwrap().next().unwrap().unwrap().path();
        let mut value: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
        value["agent"]["model"] = serde_json::Value::String("tampered".into());
        std::fs::write(&file, serde_json::to_string_pretty(&value).unwrap()).unwrap();

        let v = ledger.verify_all().unwrap();
        assert!(!v.ok());
        assert_eq!(v.tampered.len(), 1);
        assert!(v.tampered[0].contains(&receipt.id));
    }

    #[test]
    fn report_groups_by_model_effort_harness() {
        let temp = tempfile::TempDir::new().unwrap();
        let ledger = FalsificationLedger::new(temp.path());
        persist_v2(&ledger, "MACS-001", "claude-fable-5");
        persist_v2(&ledger, "MACS-002", "claude-fable-5");
        persist_v2(&ledger, "MACS-003", "claude-opus-4-8");
        let v = ledger.verify_all().unwrap();
        let fable = v
            .by_provenance
            .iter()
            .find(|(k, _)| k.contains("claude-fable-5"))
            .map(|(_, n)| *n);
        assert_eq!(fable, Some(2));
        let opus = v
            .by_provenance
            .iter()
            .find(|(k, _)| k.contains("claude-opus-4-8"))
            .map(|(_, n)| *n);
        assert_eq!(opus, Some(1));
    }

    #[test]
    fn rule_r1_order_violation_detected() {
        let temp = tempfile::TempDir::new().unwrap();
        let ledger = FalsificationLedger::new(temp.path());
        // Append MACS-005 then MACS-003: descending => R1 violation.
        persist_v2(&ledger, "MACS-005", "claude-fable-5");
        persist_v2(&ledger, "MACS-003", "claude-fable-5");
        let v = ledger.verify_all().unwrap();
        assert_eq!(v.r1_violations.len(), 1, "{:?}", v.r1_violations);
        assert!(v.r1_violations[0].contains("MACS-003"));
        assert!(!v.ok());
    }

    #[test]
    fn verify_readonly() {
        let temp = tempfile::TempDir::new().unwrap();
        let ledger = FalsificationLedger::new(temp.path());
        persist_v2(&ledger, "MACS-001", "claude-fable-5");
        let before = std::fs::read_dir(temp.path().join(".pmat-work/MACS-001/falsification"))
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| std::fs::read(e.path()).unwrap())
            .collect::<Vec<_>>();
        let _ = ledger.verify_all().unwrap();
        let after = std::fs::read_dir(temp.path().join(".pmat-work/MACS-001/falsification"))
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| std::fs::read(e.path()).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(before, after, "verify must not mutate receipts");
    }
}
