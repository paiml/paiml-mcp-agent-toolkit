// Tests for CB-1604 postcondition weakening —
// included from check_binding_scope.rs.
// Do NOT add `use` imports or `#!` attributes here.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_postcondition {
    use super::*;
    use tempfile::tempdir;

    // ── CB-1604 postcondition weakening tests ────────────────────────────
    //
    // Shape: each test writes ticket contract(s) with parent-side
    // `inherited_postconditions` + child-side `ensure`, then expects
    // Pass / Skip / Fail.

    fn pw_clause(
        id: &str,
        threshold: Option<crate::cli::handlers::work_contract::ClauseThreshold>,
    ) -> crate::cli::handlers::work_contract::ContractClause {
        use crate::cli::handlers::work_contract::{
            ClauseKind, ClauseSource, ContractClause, FalsificationMethod,
        };
        ContractClause {
            id: id.into(),
            kind: ClauseKind::Ensure,
            description: id.into(),
            falsification_method: FalsificationMethod::ManifestIntegrity,
            threshold,
            blocking: false,
            source: ClauseSource::Manual,
        }
    }

    fn pw_threshold_gte(
        metric: &str,
        value: f64,
    ) -> crate::cli::handlers::work_contract::ClauseThreshold {
        use crate::cli::handlers::work_contract::{ClauseThreshold, ThresholdOp};
        ClauseThreshold::Numeric {
            metric: metric.into(),
            op: ThresholdOp::Gte,
            value,
        }
    }

    fn pw_save(
        project: &Path,
        ticket: &str,
        inherited: Vec<crate::cli::handlers::work_contract::ContractClause>,
        ensure: Vec<crate::cli::handlers::work_contract::ContractClause>,
    ) {
        use crate::cli::handlers::work_contract::WorkContract;
        let mut c = WorkContract::new(ticket.into(), "deadbeef".into());
        c.inherited_postconditions = inherited;
        c.ensure = ensure;
        c.save(project).unwrap();
    }

    #[test]
    fn pw_skips_when_no_tickets() {
        let tmp = tempdir().unwrap();
        let r = check_binding_postcondition_weakening(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No `.pmat-work/*/contract.json`"));
    }

    #[test]
    fn pw_skips_when_no_ticket_carries_inherited() {
        let tmp = tempdir().unwrap();
        // Iteration 1 ticket — no parent, empty inherited_postconditions
        pw_save(tmp.path(), "T-1", vec![], vec![pw_clause("e.c", None)]);
        let r = check_binding_postcondition_weakening(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip, "{}", r.message);
        assert!(r.message.contains("iteration 1"));
    }

    #[test]
    fn pw_passes_when_child_equal_to_parent() {
        let tmp = tempdir().unwrap();
        let inherited = vec![pw_clause("e.cov", Some(pw_threshold_gte("coverage", 95.0)))];
        let ensure = vec![pw_clause("e.cov", Some(pw_threshold_gte("coverage", 95.0)))];
        pw_save(tmp.path(), "T-1", inherited, ensure);
        let r = check_binding_postcondition_weakening(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
        assert!(r.message.contains("1 ticket"));
    }

    #[test]
    fn pw_passes_when_child_strengthens() {
        let tmp = tempdir().unwrap();
        let inherited = vec![pw_clause("e.cov", Some(pw_threshold_gte("coverage", 90.0)))];
        let ensure = vec![pw_clause("e.cov", Some(pw_threshold_gte("coverage", 95.0)))];
        pw_save(tmp.path(), "T-1", inherited, ensure);
        let r = check_binding_postcondition_weakening(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn pw_fails_when_child_weakens() {
        let tmp = tempdir().unwrap();
        let inherited = vec![pw_clause("e.cov", Some(pw_threshold_gte("coverage", 95.0)))];
        let ensure = vec![pw_clause("e.cov", Some(pw_threshold_gte("coverage", 80.0)))];
        pw_save(tmp.path(), "T-1", inherited, ensure);
        let r = check_binding_postcondition_weakening(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("T-1"));
    }

    #[test]
    fn pw_fails_when_child_drops_clause() {
        let tmp = tempdir().unwrap();
        let inherited = vec![pw_clause("e.cov", Some(pw_threshold_gte("coverage", 95.0)))];
        let ensure: Vec<_> = Vec::new(); // dropped
        pw_save(tmp.path(), "T-1", inherited, ensure);
        let r = check_binding_postcondition_weakening(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("T-1"));
    }

    #[test]
    fn pw_fails_when_child_incompatible_threshold_op() {
        use crate::cli::handlers::work_contract::{ClauseThreshold, ThresholdOp};
        let tmp = tempdir().unwrap();
        let inherited = vec![pw_clause(
            "e.cov",
            Some(ClauseThreshold::Numeric {
                metric: "coverage".into(),
                op: ThresholdOp::Gte,
                value: 95.0,
            }),
        )];
        let ensure = vec![pw_clause(
            "e.cov",
            Some(ClauseThreshold::Numeric {
                metric: "coverage".into(),
                op: ThresholdOp::Lte,
                value: 95.0,
            }),
        )];
        pw_save(tmp.path(), "T-1", inherited, ensure);
        let r = check_binding_postcondition_weakening(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
    }

    #[test]
    fn pw_aggregates_violations_across_tickets() {
        let tmp = tempdir().unwrap();
        // T-1 violates; T-2 preserves
        let inherited_bad = vec![pw_clause("e.cov", Some(pw_threshold_gte("coverage", 95.0)))];
        let ensure_bad = vec![pw_clause("e.cov", Some(pw_threshold_gte("coverage", 50.0)))];
        pw_save(tmp.path(), "T-1", inherited_bad, ensure_bad);

        let inherited_ok = vec![pw_clause("e.cov", Some(pw_threshold_gte("coverage", 95.0)))];
        let ensure_ok = vec![pw_clause("e.cov", Some(pw_threshold_gte("coverage", 99.0)))];
        pw_save(tmp.path(), "T-2", inherited_ok, ensure_ok);

        let r = check_binding_postcondition_weakening(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("T-1"));
        assert!(
            !r.message.contains("T-2"),
            "non-violating ticket should not be listed: {}",
            r.message
        );
    }

    #[test]
    fn pw_passes_when_no_threshold_on_either_side() {
        let tmp = tempdir().unwrap();
        // No threshold → None vs None → Equal
        pw_save(
            tmp.path(),
            "T-1",
            vec![pw_clause("e.flag", None)],
            vec![pw_clause("e.flag", None)],
        );
        let r = check_binding_postcondition_weakening(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn pw_passes_counting_only_tickets_with_inherited() {
        let tmp = tempdir().unwrap();
        // T-1: iteration 1, no inherited (skip-silent)
        pw_save(tmp.path(), "T-1", vec![], vec![pw_clause("e.c", None)]);
        // T-2: iteration 2, preserves
        let inh = vec![pw_clause("e.cov", Some(pw_threshold_gte("cov", 90.0)))];
        let ens = vec![pw_clause("e.cov", Some(pw_threshold_gte("cov", 95.0)))];
        pw_save(tmp.path(), "T-2", inh, ens);
        let r = check_binding_postcondition_weakening(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
        assert!(r.message.contains("1 ticket"), "{}", r.message);
    }
}
