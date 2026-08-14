// Tests for `pmat work delegate` (MACS-019, issue #985).
//
// Every test here fails on the pre-fix handler, which found the roadmap item,
// discarded it, printed "✅ MACS-019: Task delegated and provenance boundaries
// preserved." and returned Ok(()) — writing no bundle, no journal line, and
// producing byte-identical behaviour with and without `--agy`.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod work_ledger_delegate_tests {
    use super::*;

    fn item(id: &str) -> crate::models::roadmap::RoadmapItem {
        crate::models::roadmap::RoadmapItem {
            id: id.to_string(),
            github_issue: Some(985),
            item_type: crate::models::roadmap::ItemType::Task,
            title: "Delegate must delegate".to_string(),
            status: crate::models::roadmap::ItemStatus::InProgress,
            priority: crate::models::roadmap::Priority::High,
            assigned_to: None,
            created: "2026-08-13T00:00:00Z".to_string(),
            updated: "2026-08-13T00:00:00Z".to_string(),
            spec: Some(std::path::PathBuf::from("docs/specifications/macs.md")),
            acceptance_criteria: vec![
                "handoff bundle exists".to_string(),
                "provenance boundary is journalled".to_string(),
            ],
            phases: vec![],
            subtasks: vec![],
            estimated_effort: None,
            labels: vec![],
            notes: None,
        }
    }

    fn declared() -> DeclaredAgent {
        DeclaredAgent {
            model: Some("claude-opus-5".to_string()),
            effort: Some("high".to_string()),
            harness: Some("ultracode-workflow".to_string()),
            workflow_id: Some("wf-7".to_string()),
            parent: None,
        }
    }

    fn events_of(root: &std::path::Path, id: &str) -> Vec<WorkEventRecord> {
        FalsificationLedger::new(root)
            .load_events(id)
            .expect("load")
    }

    /// No ambient harness markers: the test asserts what was DECLARED, not
    /// what the runner happens to leak through `CLAUDE_CODE_*`.
    fn no_env(_: &str) -> Option<String> {
        None
    }

    fn delegate(
        it: &crate::models::roadmap::RoadmapItem,
        target: DelegationTarget,
        root: &std::path::Path,
        declared: &DeclaredAgent,
    ) -> Result<DelegationOutcome> {
        delegate_work_item_with_env(it, target, root, declared, &no_env)
    }

    /// The whole ticket: the sentence "task delegated and provenance
    /// boundaries preserved" now has two artifacts behind it.
    #[test]
    fn test_delegate_writes_a_handoff_bundle_and_a_boundary_event() {
        let temp = tempfile::TempDir::new().expect("tempdir");
        let root = temp.path();

        let outcome = delegate(
            &item("MACS-019"),
            DelegationTarget::Agent,
            root,
            &declared(),
        )
        .expect("a declared delegator must be able to delegate");

        assert!(
            outcome.handoff_path.exists(),
            "delegation must write the handoff bundle it reports: {}",
            outcome.handoff_path.display()
        );

        let bytes = std::fs::read_to_string(&outcome.handoff_path).expect("read bundle");
        let written: DelegationHandoff = serde_json::from_str(&bytes).expect("bundle is JSON");
        assert_eq!(
            written.acceptance_criteria,
            vec![
                "handoff bundle exists".to_string(),
                "provenance boundary is journalled".to_string()
            ],
            "task context forwarding means the criteria actually travel"
        );
        assert_eq!(written.title, "Delegate must delegate");
        assert_eq!(written.github_issue, Some(985));
        assert_eq!(written.delegated_by.model, "claude-opus-5");

        let events = events_of(root, "MACS-019");
        assert_eq!(events.len(), 1, "exactly one boundary event: {events:?}");
        match &events[0].event {
            AgentEvent::Delegation {
                to,
                handoff,
                digest,
                delegated_by,
                ..
            } => {
                assert_eq!(to, "agent");
                assert_eq!(handoff, &outcome.handoff_path.display().to_string());
                assert_eq!(digest, &outcome.digest);
                assert_eq!(delegated_by.model, "claude-opus-5");
            }
            other => panic!("expected a delegation event, got {other:?}"),
        }
    }

    /// The digest on the journal has to be the digest OF the bundle, or the
    /// boundary record proves nothing about what was forwarded.
    #[test]
    fn test_recorded_digest_is_the_digest_of_the_bundle_on_disk() {
        let temp = tempfile::TempDir::new().expect("tempdir");
        let outcome = delegate(
            &item("MACS-019"),
            DelegationTarget::GoogleAntiGravity,
            temp.path(),
            &declared(),
        )
        .expect("delegate");

        let bytes = std::fs::read(&outcome.handoff_path).expect("read bundle");
        let expected = sha256_hex(&bytes);
        assert_eq!(
            outcome.digest, expected,
            "the journalled sha256 must match the bytes actually written"
        );
    }

    /// #985's second half: `--agy` used to change one word of a banner. It now
    /// changes the artifacts — both the bundle and the journal line.
    #[test]
    fn test_agy_changes_the_written_artifacts_not_only_a_banner() {
        let plain_dir = tempfile::TempDir::new().expect("tempdir");
        let agy_dir = tempfile::TempDir::new().expect("tempdir");

        let plain = delegate(
            &item("MACS-019"),
            DelegationTarget::from_agy_flag(false),
            plain_dir.path(),
            &declared(),
        )
        .expect("delegate");
        let agy = delegate(
            &item("MACS-019"),
            DelegationTarget::from_agy_flag(true),
            agy_dir.path(),
            &declared(),
        )
        .expect("delegate --agy");

        assert_eq!(plain.handoff.target, "agent");
        assert_eq!(agy.handoff.target, "google-anti-gravity");
        assert_ne!(
            plain.handoff.receiving_harness, agy.handoff.receiving_harness,
            "--agy must select a different receiving harness"
        );
        assert_eq!(
            agy.handoff.receiving_harness,
            AgentHarness::GoogleAntiGravity
        );

        let plain_event = &events_of(plain_dir.path(), "MACS-019")[0].event;
        let agy_event = &events_of(agy_dir.path(), "MACS-019")[0].event;
        let to_of = |e: &AgentEvent| match e {
            AgentEvent::Delegation { to, .. } => to.clone(),
            other => panic!("expected delegation, got {other:?}"),
        };
        assert_ne!(
            to_of(plain_event),
            to_of(agy_event),
            "the target must reach the journal, not just stdout"
        );
    }

    /// A provenance boundary with one side unnamed is not a boundary. The
    /// refusal must be total: no bundle, no journal, non-zero.
    #[test]
    fn test_unidentified_delegator_is_refused_and_writes_nothing() {
        let temp = tempfile::TempDir::new().expect("tempdir");
        let root = temp.path();

        let err = delegate(
            &item("MACS-019"),
            DelegationTarget::Agent,
            root,
            &DeclaredAgent::default(),
        )
        .expect_err("an unidentified delegator must not produce a ✅");

        let msg = err.to_string();
        assert!(
            msg.contains("unidentified") && msg.contains("PMAT_AGENT_MODEL"),
            "the refusal must name what would unblock it: {msg}"
        );
        assert!(
            msg.contains("Nothing was written"),
            "the refusal must state that nothing was written: {msg}"
        );
        assert!(
            !root.join(".pmat-work").join("MACS-019").exists(),
            "a refused delegation must leave no partial tree behind"
        );
    }

    /// Env is declared provenance (MACS E9), and `work delegate` carries no
    /// `--agent-*` flags, so the env read is the only way in.
    #[test]
    fn test_declared_agent_is_read_from_env_and_blanks_do_not_count() {
        let env = |k: &str| match k {
            "PMAT_AGENT_MODEL" => Some("  claude-opus-5  ".to_string()),
            "PMAT_AGENT_HARNESS" => Some("   ".to_string()),
            _ => None,
        };
        let d = declared_agent_from_env(&env);
        assert_eq!(d.model.as_deref(), Some("claude-opus-5"), "trimmed");
        assert_eq!(
            d.harness, None,
            "a blank env var is absence, not a declared harness"
        );
    }

    /// The rendered transcript must name the artifacts, or a later verifier is
    /// back to trusting a sentence.
    #[test]
    fn test_render_names_both_artifacts_and_both_sides() {
        let temp = tempfile::TempDir::new().expect("tempdir");
        let outcome = delegate(
            &item("MACS-019"),
            DelegationTarget::GoogleAntiGravity,
            temp.path(),
            &declared(),
        )
        .expect("delegate");
        let text = render_delegation(&outcome);

        assert!(text.contains("google-anti-gravity"), "{text}");
        assert!(text.contains("claude-opus-5"), "{text}");
        assert!(text.contains(&outcome.digest), "{text}");
        assert!(text.contains(&outcome.event_id), "{text}");
        assert!(
            text.contains("handoff-"),
            "the bundle path must be in the transcript: {text}"
        );
    }

    /// `token()` is the inverse of `parse_token`, read from one table, so the
    /// two spellings cannot drift (defect shape (a)).
    #[test]
    fn test_harness_token_round_trips_through_parse_token() {
        for h in [
            AgentHarness::ClaudeCode,
            AgentHarness::ClaudeAgentSdk,
            AgentHarness::UltracodeWorkflow,
            AgentHarness::CiPipeline,
            AgentHarness::Human,
            AgentHarness::GoogleAntiGravity,
            AgentHarness::Other("weird-runner".to_string()),
        ] {
            assert_eq!(
                AgentHarness::parse_token(&h.token()),
                h,
                "token() must parse back to the same harness"
            );
        }
    }

    /// Old journals must keep loading: the new variant is additive.
    #[test]
    fn test_delegation_event_is_additive_to_the_journal_schema() {
        let temp = tempfile::TempDir::new().expect("tempdir");
        let root = temp.path();
        let ledger = FalsificationLedger::new(root);
        ledger
            .append_event(
                "MACS-019",
                AgentEvent::SessionRestart {
                    at: "2026-08-13T00:00:00Z".to_string(),
                },
            )
            .expect("append pre-existing event");

        delegate(
            &item("MACS-019"),
            DelegationTarget::Agent,
            root,
            &declared(),
        )
        .expect("delegate");

        let events = events_of(root, "MACS-019");
        assert_eq!(events.len(), 2, "both event kinds load: {events:?}");
        assert!(matches!(events[0].event, AgentEvent::SessionRestart { .. }));
        assert!(matches!(events[1].event, AgentEvent::Delegation { .. }));
    }
}
