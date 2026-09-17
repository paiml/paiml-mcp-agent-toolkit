// Tests for `pmat work estimate` (PMAT-1366).

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod work_estimate_ledger_tests {
    use super::*;

    fn row(ticket: &str) -> EstimateRow {
        EstimateRow {
            repo: "paiml-mcp-agent-toolkit".into(),
            ticket: ticket.into(),
            phase: "all".into(),
            mode: "direct".into(),
            est: Some(4),
            actual: Some(9),
            unit: Some("turn".into()),
            basis: Some("docs/audits/impl-estimates.jsonl:L19-L29".into()),
            note: None,
        }
    }

    fn lines_of(path: &Path) -> Vec<String> {
        std::fs::read_to_string(path)
            .unwrap()
            .lines()
            .map(str::to_string)
            .collect()
    }

    fn assert_json_object(line: &str) {
        let value: serde_json::Value =
            serde_json::from_str(line).unwrap_or_else(|e| panic!("line is not JSON ({e}): {line}"));
        assert!(value.is_object(), "line is not a JSON object: {line}");
    }

    fn refusal(mutate: impl FnOnce(&mut EstimateRow)) -> Vec<String> {
        let mut r = row("PMAT-1");
        mutate(&mut r);
        gate_new_row(&r)
    }

    fn assert_refused_with(defects: &[String], needle: &str) {
        assert!(
            defects.iter().any(|d| d.contains(needle)),
            "expected a defect containing {needle:?}, got: {defects:?}"
        );
    }

    // ---- 1. a unit-less row is never written --------------------------------

    #[test]
    fn estimate_ledger_a_row_without_unit_is_refused_and_creates_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let ledger = dir.path().join("docs/audits/impl-estimates.jsonl");
        let mut r = row("PMAT-1");
        r.unit = None;
        let err = append_row(&ledger, &r).unwrap_err().to_string();
        assert!(err.contains("unit is required"), "got: {err}");
        assert!(!ledger.exists(), "a refused row must not create the ledger");
        assert!(
            !ledger.parent().unwrap().exists(),
            "a refused row must not create the ledger's directory"
        );
    }

    #[test]
    fn estimate_ledger_a_refused_row_leaves_an_existing_ledger_byte_identical() {
        let dir = tempfile::tempdir().unwrap();
        let ledger = dir.path().join("impl-estimates.jsonl");
        let before = b"{\"repo\":\"a\",\"ticket\":\"T-1\",\"unit\":\"turn\"}\n{\"x\":1}";
        std::fs::write(&ledger, before).unwrap();
        let mut r = row("PMAT-1");
        r.unit = Some("   ".into());
        assert!(append_row(&ledger, &r).is_err());
        assert_eq!(std::fs::read(&ledger).unwrap(), before.to_vec());
    }

    #[test]
    fn estimate_ledger_an_empty_unit_is_the_same_refusal_as_a_missing_one() {
        let defects = refusal(|r| r.unit = Some(String::new()));
        assert_refused_with(&defects, "unit is required");
    }

    // ---- 2. every refusal names its reason ----------------------------------

    #[test]
    fn estimate_ledger_unit_unknown_is_refused_for_a_new_row() {
        let defects = refusal(|r| r.unit = Some("unknown".into()));
        assert_refused_with(&defects, "unknown marks a legacy row");
    }

    #[test]
    fn estimate_ledger_an_unrecognised_unit_is_refused() {
        let defects = refusal(|r| r.unit = Some("banana".into()));
        assert_refused_with(&defects, "banana");
    }

    #[test]
    fn estimate_ledger_session_is_a_measured_unit() {
        assert!(refusal(|r| r.unit = Some("session".into())).is_empty());
    }

    #[test]
    fn estimate_ledger_an_estimate_without_basis_is_refused() {
        assert_refused_with(&refusal(|r| r.basis = None), "without basis=");
        assert_refused_with(&refusal(|r| r.basis = Some("  ".into())), "without basis=");
        assert!(refusal(|r| {
            r.est = None;
            r.basis = None;
        })
        .is_empty());
    }

    #[test]
    fn estimate_ledger_range_phases_are_refused() {
        assert_refused_with(&refusal(|r| r.phase = "1-4".into()), "1-4");
        assert_refused_with(&refusal(|r| r.phase = "P1-P3".into()), "P1-P3");
        assert_refused_with(&refusal(|r| r.phase = String::new()), "phase");
        assert!(refusal(|r| r.phase = "3".into()).is_empty());
    }

    #[test]
    fn estimate_ledger_empty_ticket_repo_and_mode_are_refused() {
        assert_refused_with(&refusal(|r| r.ticket = " ".into()), "ticket");
        assert_refused_with(&refusal(|r| r.repo = String::new()), "repo");
        assert_refused_with(&refusal(|r| r.mode = String::new()), "mode");
    }

    #[test]
    fn estimate_ledger_every_defect_is_reported_at_once() {
        let defects = refusal(|r| {
            r.unit = None;
            r.phase = "1-4".into();
            r.basis = None;
        });
        assert_eq!(defects.len(), 3, "got: {defects:?}");
    }

    // ---- 3. append-only: never truncates or reorders ------------------------

    #[test]
    fn estimate_ledger_appends_never_truncate_or_reorder() {
        let dir = tempfile::tempdir().unwrap();
        let ledger = dir.path().join("impl-estimates.jsonl");
        let before = concat!(
            "{\"repo\":\"paiml-mcp-agent-toolkit\",\"ticket\":\"T-1\",\"unit\":\"turn\"}\n",
            "{\"repo\":\"paiml-mcp-agent-toolkit\",\"ticket\":\"T-2\",\"unit\":\"turn\"}\n",
            "{\"repo\":\"paiml-mcp-agent-toolkit\",\"ticket\":\"T-3\",\"unit\":\"turn\"}\n",
        );
        std::fs::write(&ledger, before).unwrap();
        assert_eq!(append_row(&ledger, &row("PMAT-4")).unwrap(), 4);
        assert_eq!(append_row(&ledger, &row("PMAT-5")).unwrap(), 5);

        let after = std::fs::read(&ledger).unwrap();
        assert!(
            after.starts_with(before.as_bytes()),
            "old bytes are not a prefix"
        );
        let lines = lines_of(&ledger);
        assert_eq!(lines.len(), 5, "got: {lines:?}");
        lines.iter().for_each(|l| assert_json_object(l));
        let l4: EstimateRow = serde_json::from_str(&lines[3]).unwrap();
        let l5: EstimateRow = serde_json::from_str(&lines[4]).unwrap();
        assert_eq!(l4.ticket, "PMAT-4");
        assert_eq!(l5.ticket, "PMAT-5");
    }

    #[test]
    fn estimate_ledger_first_append_creates_file_and_parent() {
        let dir = tempfile::tempdir().unwrap();
        let ledger = dir.path().join("docs/audits/impl-estimates.jsonl");
        assert_eq!(append_row(&ledger, &row("PMAT-1")).unwrap(), 1);
        assert_eq!(lines_of(&ledger).len(), 1);
    }

    #[test]
    fn estimate_ledger_a_row_that_would_split_the_key_is_refused_byte_identical() {
        let dir = tempfile::tempdir().unwrap();
        let ledger = dir.path().join("impl-estimates.jsonl");
        let before =
            "{\"repo\":\"paiml-mcp-agent-toolkit\",\"ticket\":\"T-1\",\"unit\":\"turn\"}\n";
        std::fs::write(&ledger, before).unwrap();
        let mut r = row("PMAT-2");
        r.repo = "pmat".into();
        let err = append_row(&ledger, &r).unwrap_err().to_string();
        assert!(
            err.contains("would split the ledger") && err.contains("paiml-mcp-agent-toolkit"),
            "got: {err}"
        );
        assert_eq!(std::fs::read(&ledger).unwrap(), before.as_bytes().to_vec());
        assert_eq!(append_row(&ledger, &row("PMAT-3")).unwrap(), 2);
    }

    // ---- 4. a missing trailing newline is not glued onto --------------------

    #[test]
    fn estimate_ledger_missing_trailing_newline_starts_a_new_line() {
        let dir = tempfile::tempdir().unwrap();
        let ledger = dir.path().join("impl-estimates.jsonl");
        let before = "{\"repo\":\"paiml-mcp-agent-toolkit\",\"ticket\":\"T-1\",\"unit\":\"turn\"}";
        std::fs::write(&ledger, before).unwrap();
        assert_eq!(append_row(&ledger, &row("PMAT-2")).unwrap(), 2);

        let after = std::fs::read(&ledger).unwrap();
        assert!(
            after.starts_with(before.as_bytes()),
            "old bytes are not a prefix"
        );
        let lines = lines_of(&ledger);
        assert_eq!(lines.len(), 2, "got: {lines:?}");
        lines.iter().for_each(|l| assert_json_object(l));
    }

    // ---- 5. one record per line ---------------------------------------------

    #[test]
    fn estimate_ledger_one_record_per_line_with_nulls_and_unit() {
        let dir = tempfile::tempdir().unwrap();
        let ledger = dir.path().join("impl-estimates.jsonl");
        let mut r = row("PMAT-1");
        r.est = None;
        r.actual = None;
        r.basis = None;
        r.note = Some("first line\nsecond line".into());
        append_row(&ledger, &r).unwrap();

        let lines = lines_of(&ledger);
        assert_eq!(
            lines.len(),
            1,
            "an embedded newline split the record: {lines:?}"
        );
        let value: serde_json::Value = serde_json::from_str(&lines[0]).unwrap();
        assert!(value["est"].is_null() && value.get("est").is_some());
        assert!(value["actual"].is_null() && value.get("actual").is_some());
        assert_eq!(value["unit"], "turn");
        assert!(value.get("basis").is_none(), "basis None must be skipped");
        assert_eq!(value["note"], "first line\nsecond line");
    }

    #[test]
    fn estimate_ledger_fields_serialise_in_ledger_order() {
        let line = serde_json::to_string(&row("PMAT-1")).unwrap();
        let keys = [
            "repo", "ticket", "phase", "mode", "est", "actual", "unit", "basis",
        ];
        let positions: Vec<usize> = keys
            .iter()
            .map(|k| line.find(&format!("\"{k}\":")).unwrap())
            .collect();
        assert!(positions.windows(2).all(|w| w[0] < w[1]), "got: {line}");
    }

    // ---- 6. the check over committed text -----------------------------------

    const CHECK_FIXTURE: &str = concat!(
        "{\"repo\":\"r\",\"ticket\":\"NOUNIT\",\"phase\":\"all\",\"mode\":\"d\",\"est\":1,\"actual\":5}\n",
        "{\"repo\":\"r\",\"ticket\":\"NOUNIT-NULL\",\"phase\":\"all\",\"mode\":\"d\",\"est\":1,\"actual\":null}\n",
        "{\"repo\":\"r\",\"ticket\":\"UNKNOWN\",\"phase\":\"all\",\"unit\":\"unknown\",\"mode\":\"d\",\"est\":1,\"actual\":5}\n",
        "{\"repo\":\"r\",\"ticket\":\"ALL\",\"phase\":\"all\",\"unit\":\"turn\",\"mode\":\"d\",\"est\":1,\"actual\":5}\n",
        "\n",
        "{\"repo\":\"r\",\"ticket\":\"NUM\",\"phase\":3,\"unit\":\"turn\",\"mode\":\"d\",\"est\":1,\"actual\":5}\n",
        "{\"repo\":\"r\",\"ticket\":\"STR\",\"phase\":\"3\",\"unit\":\"turn\",\"mode\":\"d\",\"est\":1,\"actual\":5}\n",
        "{\"repo\":\"r\",\"ticket\":\"RANGE\",\"phase\":\"1-4\",\"unit\":\"turn\",\"mode\":\"d\",\"est\":1,\"actual\":5}\n",
        "{\"repo\":\"r\",\"ticket\":\"SESSION\",\"phase\":\"all\",\"unit\":\"session\",\"mode\":\"d\",\"est\":1,\"actual\":5}\n",
        "{\"repo\":\"other\",\"ticket\":\"OTHER\",\"phase\":\"all\",\"unit\":\"turn\",\"mode\":\"d\",\"est\":1,\"actual\":null}\n",
        "not json at all\n",
    );

    fn excluded_reason(report: &EstimateLedgerReport, ticket: &str) -> Option<String> {
        report
            .excluded
            .iter()
            .find(|(_, t, _)| t == ticket)
            .map(|(_, _, reason)| reason.clone())
    }

    #[test]
    fn estimate_ledger_check_flags_unit_less_rows_measured_or_not() {
        let report = check_ledger_text(CHECK_FIXTURE, None);
        assert_refused_with(&report.violations, "L1 ticket=NOUNIT: no unit");
        assert_refused_with(&report.violations, "L2 ticket=NOUNIT-NULL: no unit");
    }

    #[test]
    fn estimate_ledger_check_flags_a_non_json_line() {
        let report = check_ledger_text(CHECK_FIXTURE, None);
        assert_refused_with(&report.violations, "L11");
        assert_eq!(report.violations.len(), 4, "got: {:?}", report.violations);
    }

    #[test]
    fn estimate_ledger_check_flags_a_ledger_split_across_repo_keys() {
        let report = check_ledger_text(CHECK_FIXTURE, None);
        assert_eq!(report.keys, vec!["other".to_string(), "r".to_string()]);
        assert_refused_with(&report.violations, "ledger carries 2 repo keys (other,r)");
        let one_key = CHECK_FIXTURE.replace("\"repo\":\"other\"", "\"repo\":\"r\"");
        let report = check_ledger_text(&one_key, None);
        assert!(
            !report.violations.iter().any(|v| v.contains("repo keys")),
            "got: {:?}",
            report.violations
        );
    }

    #[test]
    fn estimate_ledger_check_classifies_the_pool() {
        let report = check_ledger_text(CHECK_FIXTURE, None);
        assert_eq!(
            excluded_reason(&report, "UNKNOWN").as_deref(),
            Some("unit-not-turn")
        );
        assert_eq!(
            excluded_reason(&report, "RANGE").as_deref(),
            Some("range-phase")
        );
        assert_eq!(
            excluded_reason(&report, "SESSION").as_deref(),
            Some("unit-not-turn")
        );
        for poolable in ["ALL", "NUM", "STR"] {
            assert_eq!(excluded_reason(&report, poolable), None, "{poolable}");
        }
        assert_eq!(report.poolable, 3);
        assert_eq!(
            report.unmeasured, 1,
            "only OTHER is schema-valid with null actual"
        );
        let range_line = report
            .excluded
            .iter()
            .find(|(_, t, _)| t == "RANGE")
            .unwrap()
            .0;
        assert_eq!(range_line, 8, "excluded rows carry their 1-based line");
    }

    #[test]
    fn estimate_ledger_check_repo_filter_counts_only_that_key() {
        let other = check_ledger_text(CHECK_FIXTURE, Some("other"));
        assert_eq!(other.rows, 1);
        assert_eq!(other.poolable, 0);
        assert_eq!(other.unmeasured, 1);
        assert!(other.excluded.is_empty());
        assert_eq!(other.violations.len(), 4, "violations span every repo");

        let r = check_ledger_text(CHECK_FIXTURE, Some("r"));
        assert_eq!(r.rows, 8);
        assert_eq!(r.poolable, 3);
    }

    #[test]
    fn estimate_ledger_check_flags_bad_values() {
        let text = concat!(
            "{\"repo\":\"r\",\"ticket\":\"T\",\"phase\":\"all\",\"unit\":\"banana\",\"actual\":1}\n",
            "{\"repo\":\"r\",\"ticket\":\"T\",\"phase\":\"all\",\"unit\":\"turn\",\"actual\":-1}\n",
            "{\"repo\":\"r\",\"ticket\":\"T\",\"phase\":\"all\",\"unit\":\"turn\",\"est\":\"4\",\"actual\":1}\n",
            "{\"repo\":7,\"ticket\":\"T\",\"phase\":\"all\",\"unit\":\"turn\",\"actual\":1}\n",
            "{\"repo\":\"r\",\"phase\":\"all\",\"unit\":\"turn\",\"actual\":1}\n",
            "[1,2]\n",
        );
        let report = check_ledger_text(text, None);
        for n in 1..=6 {
            assert_refused_with(&report.violations, &format!("L{n} "));
        }
        assert_eq!(report.poolable, 0);
    }

    #[test]
    fn estimate_ledger_a_written_row_passes_the_check() {
        let dir = tempfile::tempdir().unwrap();
        let ledger = dir.path().join("impl-estimates.jsonl");
        append_row(&ledger, &row("PMAT-1")).unwrap();
        let text = std::fs::read_to_string(&ledger).unwrap();
        let report = check_ledger_text(&text, Some("paiml-mcp-agent-toolkit"));
        assert!(report.violations.is_empty(), "got: {:?}", report.violations);
        assert_eq!(report.poolable, 1);
    }

    // ---- 7. the repo key ----------------------------------------------------

    #[test]
    fn estimate_ledger_repo_key_from_ssh_and_https_remotes() {
        for url in [
            "git@github.com:paiml/paiml-mcp-agent-toolkit.git",
            "https://github.com/paiml/paiml-mcp-agent-toolkit.git",
            "https://github.com/paiml/paiml-mcp-agent-toolkit",
            "https://github.com/paiml/paiml-mcp-agent-toolkit.git\n",
        ] {
            assert_eq!(
                repo_key_from_remote_url(url).as_deref(),
                Some("paiml-mcp-agent-toolkit"),
                "{url:?}"
            );
        }
        assert_eq!(repo_key_from_remote_url(""), None);
    }

    #[test]
    fn estimate_ledger_repo_key_resolution() {
        let origin = Some("paiml-mcp-agent-toolkit");
        assert_eq!(
            resolve_repo_key(None, origin).unwrap(),
            "paiml-mcp-agent-toolkit"
        );
        assert_eq!(
            resolve_repo_key(Some("paiml-mcp-agent-toolkit"), origin).unwrap(),
            "paiml-mcp-agent-toolkit"
        );
        assert_eq!(resolve_repo_key(Some("pmat"), None).unwrap(), "pmat");
        assert_eq!(
            resolve_repo_key(Some("x"), origin).unwrap(),
            "x",
            "--repo wins: a clone whose origin is a local path has a meaningless basename"
        );
        assert_eq!(
            resolve_repo_key(Some("  "), origin).unwrap(),
            "paiml-mcp-agent-toolkit"
        );

        let err = resolve_repo_key(None, None).unwrap_err().to_string();
        assert!(err.contains("no origin remote: pass --repo"), "got: {err}");
    }

    // ---- 8. the committed ledger --------------------------------------------

    #[test]
    fn the_committed_estimates_ledger_passes_the_write_time_gate() {
        let text = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/docs/audits/impl-estimates.jsonl"
        ))
        .unwrap();
        let report = check_ledger_text(&text, None);
        assert!(
            report.violations.is_empty(),
            "docs/audits/impl-estimates.jsonl carries {} row(s) the writer would refuse:\n{}",
            report.violations.len(),
            report.violations.join("\n")
        );
        assert_eq!(report.keys, vec!["paiml-mcp-agent-toolkit".to_string()]);
    }
}
