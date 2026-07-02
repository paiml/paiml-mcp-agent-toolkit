//! Tests for MACS-007/008/009 (structured CoT, chain checker, derivation).

use super::*;

fn step(id: &str, assumption: &str, evidence: &str) -> CotStepView {
    CotStepView {
        id: id.to_string(),
        assumption: assumption.to_string(),
        assumption_references: Vec::new(),
        implication: format!("implication of {id}"),
        evidence_method: evidence.to_string(),
        discharged_by: None,
        structured: true,
    }
}

#[test]
fn v2_roundtrip_prop() {
    // serde roundtrip across all DischargeRef wire forms.
    for (raw, want) in [
        ("\"CoT-3\"", DischargeRef::Step("CoT-3".to_string())),
        ("\"E5\"", DischargeRef::EnvFact("E5".to_string())),
        (
            "\"macs-cot-v1#chain_integrity\"",
            DischargeRef::Equation {
                contract: "macs-cot-v1".to_string(),
                equation: "chain_integrity".to_string(),
            },
        ),
        (
            "{\"Axiomatic\": {\"reason\": \"definitional\"}}",
            DischargeRef::Axiomatic {
                reason: "definitional".to_string(),
            },
        ),
    ] {
        let parsed: DischargeRef = serde_json::from_str(raw).expect("parses");
        assert_eq!(parsed, want);
        let re = serde_json::to_string(&parsed).expect("serializes");
        let back: DischargeRef = serde_json::from_str(&re).expect("re-parses");
        assert_eq!(back, want, "roundtrip for {raw}");
    }

    let view = CotStepView {
        id: "CoT-1".to_string(),
        assumption: "x (E3)".to_string(),
        assumption_references: vec!["E3".to_string()],
        implication: "y".to_string(),
        evidence_method: "cargo test z".to_string(),
        discharged_by: Some(DischargeRef::Step("CoT-0".to_string())),
        structured: true,
    };
    let json = serde_json::to_string(&view).expect("view serializes");
    let back: CotStepView = serde_json::from_str(&json).expect("view parses");
    assert_eq!(back, view);
}

#[test]
fn discharge_ref_parses_cot_equation_efact() {
    assert_eq!(
        DischargeRef::parse("CoT-2"),
        Some(DischargeRef::Step("CoT-2".to_string()))
    );
    assert_eq!(
        DischargeRef::parse("E9"),
        Some(DischargeRef::EnvFact("E9".to_string()))
    );
    assert!(matches!(
        DischargeRef::parse("macs-ladder-v1#gate_monotone"),
        Some(DischargeRef::Equation { .. })
    ));
    assert_eq!(DischargeRef::parse(""), None);
    assert_eq!(DischargeRef::parse("Elephant"), None);
    assert_eq!(DischargeRef::parse("CoT-"), None);
    assert_eq!(DischargeRef::parse("#eq"), None);
}

#[test]
fn legacy_prose_migrates_annotated_l0() {
    let legacy = serde_json::json!({
        "step": 1,
        "question": "Why is the sky blue?",
        "answer": "Rayleigh scattering."
    });
    let view = parse_step(0, &legacy);
    assert!(!view.structured, "legacy steps are annotated, not dropped");
    assert_eq!(view.id, "CoT-1");
    assert_eq!(view.assumption, "Why is the sky blue?");
    assert_eq!(view.implication, "Rayleigh scattering.");
    assert!(view.evidence_method.contains("MIGRATED-L0"));
}

#[test]
fn undischarged_assumption_violates_1640() {
    // A mid-chain step citing a nonexistent step is unresolved; a step with
    // no sources and no evidence_method is undischarged.
    let steps = vec![
        step("CoT-1", "root claim", "grep README"),
        CotStepView {
            assumption: "depends on (CoT-99)".to_string(),
            ..step("CoT-2", "", "cargo test x")
        },
        step("CoT-3", "floating claim with no citation", ""),
    ];
    let violations = check_chain(&steps);
    assert!(
        violations
            .iter()
            .any(|v| v.step_id == "CoT-2" && v.kind == "unresolved-ref"),
        "{violations:?}"
    );
    assert!(
        violations
            .iter()
            .any(|v| v.step_id == "CoT-3" && v.kind == "undischarged"),
        "{violations:?}"
    );
}

#[test]
fn cycle_detected_violates_1640() {
    let mut a = step("CoT-1", "a", "t");
    a.discharged_by = Some(DischargeRef::Step("CoT-2".to_string()));
    let mut b = step("CoT-2", "b", "t");
    b.discharged_by = Some(DischargeRef::Step("CoT-1".to_string()));
    let violations = check_chain(&[a, b]);
    assert!(
        violations.iter().any(|v| v.kind == "cycle"),
        "{violations:?}"
    );

    let mut selfref = step("CoT-1", "s", "t");
    selfref.discharged_by = Some(DischargeRef::Step("CoT-1".to_string()));
    let violations = check_chain(&[selfref]);
    assert!(
        violations.iter().any(|v| v.kind == "cycle"),
        "self-discharge is a cycle: {violations:?}"
    );
}

#[test]
fn spec_section_3_1_passes() {
    // Self-enforcement: the MACS spec's own §3.1 chain is the first fixture.
    let root = env!("CARGO_MANIFEST_DIR");
    let spec = std::fs::read_to_string(
        std::path::Path::new(root)
            .join("docs/specifications/components/modern-agentic-coding-support.md"),
    )
    .expect("MACS spec committed (MACS-000)");
    let section = spec
        .split("### 3.1")
        .nth(1)
        .expect("spec has §3.1")
        .split("```yaml")
        .nth(1)
        .expect("§3.1 has a yaml block")
        .split("```")
        .next()
        .expect("yaml block closes");
    let doc: serde_json::Value = serde_yaml_ng::from_str(section).expect("§3.1 yaml parses");
    let contract = serde_json::json!({ "chain_of_thought": doc["chain_of_thought"] });
    let steps = parse_steps(&contract);
    assert_eq!(steps.len(), 8, "§3.1 has CoT-1..CoT-8");
    assert!(steps.iter().all(|s| s.structured));
    let violations = check_chain(&steps);
    assert!(
        violations.is_empty(),
        "the spec is subject to the check it specifies: {violations:?}"
    );
}

#[test]
fn dag_property_prop() {
    use proptest::prelude::*;
    let mut runner = proptest::test_runner::TestRunner::new(ProptestConfig::with_cases(32));
    runner
        .run(
            &(
                2usize..12,
                proptest::collection::vec(0usize..100, 0..24),
                any::<bool>(),
            ),
            |(n, raw_edges, insert_cycle)| {
                // Build a chain of n steps; every step evidence-rooted.
                let mut steps: Vec<CotStepView> = (1..=n)
                    .map(|i| step(&format!("CoT-{i}"), "claim", "cargo test t"))
                    .collect();
                // Random back-edges (j discharges from earlier i): always a DAG.
                for (k, raw) in raw_edges.iter().enumerate() {
                    let j = 1 + (k % (n - 1)); // 1..n
                    let i = raw % j; // strictly earlier
                    steps[j].discharged_by = Some(DischargeRef::Step(format!("CoT-{}", i + 1)));
                }
                if insert_cycle && n >= 2 {
                    // Make step 1 depend on the last step: guaranteed cycle
                    // iff the last step (transitively) depends on step 1.
                    steps[0].discharged_by = Some(DischargeRef::Step(format!("CoT-{n}")));
                }
                let violations = check_chain(&steps);
                let has_cycle_violation = violations.iter().any(|v| v.kind == "cycle");
                if !insert_cycle {
                    prop_assert!(
                        violations.is_empty(),
                        "evidence-rooted DAG must be accepted: {violations:?}"
                    );
                } else {
                    // A forward edge from CoT-1 to CoT-n creates a cycle only
                    // when a back-path exists; verify the checker agrees with
                    // a reachability oracle.
                    let mut reach = vec![false; n];
                    reach[0] = true;
                    let mut changed = true;
                    while changed {
                        changed = false;
                        for j in 0..n {
                            if reach[j] {
                                continue;
                            }
                            if let Some(DischargeRef::Step(t)) = &steps[j].discharged_by {
                                let idx = t
                                    .strip_prefix("CoT-")
                                    .and_then(|d| d.parse::<usize>().ok())
                                    .map(|d| d - 1);
                                if idx.is_some_and(|i| reach[i]) {
                                    reach[j] = true;
                                    changed = true;
                                }
                            }
                        }
                    }
                    let oracle_cycle = reach[n - 1];
                    prop_assert_eq!(
                        has_cycle_violation,
                        oracle_cycle,
                        "checker must agree with reachability oracle"
                    );
                }
                Ok(())
            },
        )
        .expect("property holds");
}

#[test]
fn one_claim_per_step_verbatim_fields() {
    let steps = vec![
        step("CoT-1", "a", "cargo test alpha"),
        step("CoT-2", "b (CoT-1)", "grep -c beta src/"),
    ];
    for s in &steps {
        let d = derive(s);
        assert_eq!(d.statement, s.implication, "hypothesis verbatim");
        assert_eq!(d.method, s.evidence_method, "method verbatim");
        assert_eq!(d.obligation_id, format!("PO-{}", s.id));
    }
    // |claims| == |steps| by construction of the rendered artifact:
    let yaml = render_derivation("T-1", &steps, false);
    assert_eq!(yaml.matches("- hypothesis:").count(), steps.len());
    assert_eq!(yaml.matches("- id:").count(), steps.len());
}

#[test]
fn optional_clause_codegen_gated_by_flag() {
    let steps = vec![step("CoT-1", "a", "t")];
    let without = render_derivation("T-1", &steps, false);
    assert!(
        !without.contains("clauses:"),
        "clauses gated off by default"
    );
    let with = render_derivation("T-1", &steps, true);
    assert!(with.contains("clauses:"));
    assert!(with.contains("ensure.CoT-1"));
}

#[test]
fn stable_output_two_runs() {
    let steps = vec![step("CoT-2", "b", "t2"), step("CoT-1", "a", "t1")];
    let one = render_derivation("T-1", &steps, true);
    let two = render_derivation("T-1", &steps, true);
    assert_eq!(one, two, "derivation is deterministic");
}

#[test]
fn canonical_cot_sha_is_order_insensitive_for_keys() {
    let a: Value = serde_json::from_str(
        r#"{"chain_of_thought": [{"id": "CoT-1", "assumption": "x", "implication": "y"}]}"#,
    )
    .expect("a");
    let b: Value = serde_json::from_str(
        r#"{"chain_of_thought": [{"implication": "y", "assumption": "x", "id": "CoT-1"}]}"#,
    )
    .expect("b");
    assert_eq!(canonical_cot_sha(&a), canonical_cot_sha(&b));
    let c: Value = serde_json::from_str(r#"{"chain_of_thought": [{"id": "CoT-2"}]}"#).expect("c");
    assert_ne!(canonical_cot_sha(&a), canonical_cot_sha(&c));
}
