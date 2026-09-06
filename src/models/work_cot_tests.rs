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

/// #1200 (PMAT-685): the exact v5.0 step aprender's `.pmat-work/GH-663/contract.json`
/// carries — `falsifiable_claim`, no `assumption`, no `implication`. 3.38.0 parsed it
/// as a structured step with an empty implication and rendered `statement: ""`.
const V5_CLAIM: &str =
    "apr compare-hf --offline hangs instead of rejecting network operation — must not occur after fix";

fn v5_contract(step: serde_json::Value, top_level_claims: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "version": "5.0",
        "work_item_id": "GH-663",
        "chain_of_thought": [step],
        "falsifiable_claims": top_level_claims
    })
}

/// Quorum lane 3 on PMAT-685: a step carrying nothing but `falsifiable_claim`
/// used to parse as legacy prose (structured=false) and be refused as hollow.
#[test]
fn a_step_with_only_a_falsifiable_claim_is_structured_and_derives_it() {
    let contract = v5_contract(
        serde_json::json!({"step": 1, "falsifiable_claim": V5_CLAIM}),
        serde_json::json!([]),
    );
    let steps = parse_steps(&contract);
    assert!(steps[0].structured, "a claim is a structured field");
    assert_eq!(steps[0].implication, V5_CLAIM);
    assert!(hollow_steps(&steps).is_empty());
}

/// Quorum lane 1 on PMAT-685: the digest must cover every text the derivation
/// reads. A contract that never borrows a top-level claim keeps its 3.38.0
/// digest byte-for-byte; one that does moves when — and only when — the
/// borrowed claim changes.
#[test]
fn digest_covers_borrowed_top_level_claims_and_nothing_else() {
    // (a) a pre-#1200 contract: top-level claims are not read, so they are not hashed.
    let v2 = serde_json::json!({
        "chain_of_thought": [{"id": "CoT-1", "assumption": "root (E1)", "implication": "alpha holds",
                              "evidence_method": "cargo test alpha"}],
        "falsifiable_claims": [{"id": "FC-1", "claim": "unused"}]
    });
    let mut v2_edited = v2.clone();
    v2_edited["falsifiable_claims"][0]["claim"] = serde_json::json!("unused, edited");
    assert_eq!(
        canonical_cot_sha(&v2),
        canonical_cot_sha(&v2_edited),
        "unread text is not hashed"
    );
    let mut v2_without = v2.clone();
    v2_without
        .as_object_mut()
        .expect("object")
        .remove("falsifiable_claims");
    assert_eq!(
        canonical_cot_sha(&v2),
        canonical_cot_sha(&v2_without),
        "3.38.0 digests are unchanged"
    );

    // (b) a v5.0 step that borrows FC-1's text: editing FC-1 moves the digest, editing FC-2 does not.
    let borrowed = v5_contract(
        serde_json::json!({"step": 1, "evidence_method": "Falsification", "discharged_by": ["FC-1"]}),
        serde_json::json!([{"id": "FC-1", "claim": "exits 2 within 1s"}, {"id": "FC-2", "claim": "other"}]),
    );
    let mut fc1_edited = borrowed.clone();
    fc1_edited["falsifiable_claims"][0]["claim"] = serde_json::json!("exits 2 within 10s");
    let mut fc2_edited = borrowed.clone();
    fc2_edited["falsifiable_claims"][1]["claim"] = serde_json::json!("other, edited");
    assert_ne!(
        canonical_cot_sha(&borrowed),
        canonical_cot_sha(&fc1_edited),
        "the borrowed text is hashed"
    );
    assert_eq!(
        canonical_cot_sha(&borrowed),
        canonical_cot_sha(&fc2_edited),
        "an unborrowed sibling is not"
    );
    assert_eq!(
        canonical_cot_sha(&borrowed),
        canonical_cot_sha(&borrowed),
        "deterministic"
    );
}

#[test]
fn v5_step_falsifiable_claim_becomes_the_implication() {
    let contract = v5_contract(
        serde_json::json!({"step": 1, "evidence_method": "Falsification",
                           "falsifiable_claim": V5_CLAIM, "discharged_by": ["FC-1"]}),
        serde_json::json!([]),
    );
    let steps = parse_steps(&contract);
    assert_eq!(steps.len(), 1);
    assert_eq!(steps[0].id, "CoT-1");
    assert_eq!(
        steps[0].implication, V5_CLAIM,
        "the claim IS the obligation text"
    );
    assert_eq!(steps[0].evidence_method, "Falsification");
    assert!(hollow_steps(&steps).is_empty(), "nothing hollow here");
    let yaml = render_derivation("GH-663", &steps, false);
    assert!(yaml.contains(V5_CLAIM), "rendered verbatim: {yaml}");
    assert!(
        !yaml.contains("statement: \"\""),
        "no hollow obligation: {yaml}"
    );
    assert!(
        !yaml.contains("hypothesis: \"\""),
        "no hollow claim: {yaml}"
    );
}

#[test]
fn v5_step_without_its_own_claim_reads_the_top_level_claim_it_discharges() {
    let contract = v5_contract(
        serde_json::json!({"step": 1, "evidence_method": "Falsification", "discharged_by": ["FC-1"]}),
        serde_json::json!([{"id": "FC-1", "claim": "apr compare-hf --offline exits 2 within 1s",
                            "method": "cargo test offline_refuses"}]),
    );
    let steps = parse_steps(&contract);
    assert_eq!(
        steps[0].implication, "apr compare-hf --offline exits 2 within 1s",
        "the discharged top-level claim fills an absent implication"
    );
    // A claim object may spell its text as `text` or `hypothesis` too.
    let contract = v5_contract(
        serde_json::json!({"step": 1, "evidence_method": "Falsification",
                           "falsifiable_claim": {"text": "object form", "references": ["E1"]}}),
        serde_json::json!([]),
    );
    assert_eq!(parse_steps(&contract)[0].implication, "object form");
}

#[test]
fn hollow_step_is_named_not_rendered() {
    let contract = v5_contract(
        serde_json::json!({"step": 1, "evidence_method": "Falsification", "discharged_by": ["FC-9"]}),
        serde_json::json!([{"id": "FC-1", "claim": "unrelated"}]),
    );
    let steps = parse_steps(&contract);
    assert_eq!(
        steps[0].implication, "",
        "nothing to read: FC-9 does not exist"
    );
    assert_eq!(hollow_steps(&steps), vec!["CoT-1".to_string()]);
}

/// The CLI refuses a hollow derivation: exit non-zero, the step and the missing
/// fields named, and NOTHING written — neither the artifact nor the digest.
#[tokio::test]
async fn cot_derive_refuses_a_hollow_step_and_writes_nothing() {
    let project = tempfile::tempdir().expect("temp project");
    let dir = project.path().join(".pmat-work").join("GH-663");
    std::fs::create_dir_all(&dir).expect("work dir");
    let contract = v5_contract(
        serde_json::json!({"step": 1, "evidence_method": "Falsification", "discharged_by": ["FC-9"]}),
        serde_json::json!([]),
    );
    std::fs::write(dir.join("contract.json"), contract.to_string()).expect("contract");

    let result = crate::cli::handlers::work_handlers::core_handlers::handle_work_cot_derive(
        "GH-663".to_string(),
        false,
        Some(project.path().to_path_buf()),
    )
    .await;

    let err = result.err().map_or_else(String::new, |e| format!("{e:#}"));
    assert!(!err.is_empty(), "a hollow step must not derive");
    assert!(err.contains("CoT-1"), "names the step: {err}");
    assert!(
        err.contains("falsifiable_claim") && err.contains("implication"),
        "names the fields that could have filled it: {err}"
    );
    assert!(
        !project
            .path()
            .join("contracts/work/GH-663.cot.yaml")
            .exists(),
        "no hollow artifact on disk"
    );
    assert!(
        !dir.join("cot-digest.json").exists(),
        "no digest for a derivation that did not happen"
    );
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

/// Independent reachability oracle: is CoT-n transitively reachable from
/// CoT-1 via `discharged_by` step edges? (Extracted so the property test
/// body stays under the cognitive-complexity gate.)
fn oracle_reaches_last(steps: &[CotStepView]) -> bool {
    let n = steps.len();
    let mut reach = vec![false; n];
    reach[0] = true;
    let mut changed = true;
    while changed {
        changed = false;
        for j in 0..n {
            if reach[j] {
                continue;
            }
            let Some(DischargeRef::Step(t)) = &steps[j].discharged_by else {
                continue;
            };
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
    reach[n - 1]
}

/// Build an n-step evidence-rooted chain with random back-edges; optionally
/// add a forward edge CoT-1 -> CoT-n (which is a cycle iff a back-path exists).
fn build_dag_chain(n: usize, raw_edges: &[usize], insert_cycle: bool) -> Vec<CotStepView> {
    let mut steps: Vec<CotStepView> = (1..=n)
        .map(|i| step(&format!("CoT-{i}"), "claim", "cargo test t"))
        .collect();
    for (k, raw) in raw_edges.iter().enumerate() {
        let j = 1 + (k % (n - 1)); // 1..n
        let i = raw % j; // strictly earlier -> always a DAG
        steps[j].discharged_by = Some(DischargeRef::Step(format!("CoT-{}", i + 1)));
    }
    if insert_cycle && n >= 2 {
        steps[0].discharged_by = Some(DischargeRef::Step(format!("CoT-{n}")));
    }
    steps
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
                let steps = build_dag_chain(n, &raw_edges, insert_cycle);
                let has_cycle = check_chain(&steps).iter().any(|v| v.kind == "cycle");
                if insert_cycle {
                    // Checker must agree with the independent reachability oracle.
                    prop_assert_eq!(has_cycle, oracle_reaches_last(&steps));
                } else {
                    prop_assert!(!has_cycle, "evidence-rooted DAG must be accepted");
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

#[test]
fn inline_prose_efact_does_not_discharge() {
    // ADVERSARIAL-REVIEW regression: an incidental "E5" in free assumption
    // prose must NOT silently discharge an otherwise floating assumption.
    let steps = vec![CotStepView {
        id: "CoT-1".to_string(),
        assumption: "p99 latency stays under 5ms at E5 traffic".to_string(),
        assumption_references: Vec::new(),
        implication: "SLA holds".to_string(),
        evidence_method: String::new(),
        discharged_by: None,
        structured: true,
    }];
    let violations = check_chain(&steps);
    assert!(
        violations.iter().any(|v| v.kind == "undischarged"),
        "floating assumption with an incidental E5 token must still be flagged: {violations:?}"
    );
    // But a DECLARED E-fact reference does discharge it.
    let mut declared = steps;
    declared[0].assumption_references = vec!["E5".to_string()];
    assert!(
        check_chain(&declared).is_empty(),
        "declared E-fact discharges"
    );
}

/// #1200 as filed: aprender's ten `GH-663..672` contracts (`version: "5.0"`,
/// steps carrying `falsifiable_claim` + `discharged_by: ["FC-n"]`), copied
/// verbatim into `src/tests/fixtures/aprender-cot/`. On 3.38.0 every one of
/// them derived hollow. Now every step derives a non-empty obligation, and the
/// rendered artifacts carry no `statement: ""` / `hypothesis: ""` at all.
#[test]
fn every_aprender_gh_663_to_672_contract_derives_zero_empty_statements() {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/tests/fixtures/aprender-cot");
    let mut seen = 0usize;
    for id in 663..=672 {
        let path = dir.join(format!("GH-{id}.contract.json"));
        let text = std::fs::read_to_string(&path);
        assert!(
            text.is_ok(),
            "{} must be readable: {:?}",
            path.display(),
            text.as_ref().err()
        );
        let contract: serde_json::Value =
            serde_json::from_str(&text.unwrap_or_default()).expect("fixture is JSON");
        let steps = parse_steps(&contract);
        assert!(!steps.is_empty(), "GH-{id}: at least one step");
        assert_eq!(
            hollow_steps(&steps),
            Vec::<String>::new(),
            "GH-{id}: no hollow step"
        );
        let yaml = render_derivation(&format!("GH-{id}"), &steps, false);
        assert_eq!(
            yaml.matches("- id:").count(),
            steps.len(),
            "GH-{id}: one obligation per step"
        );
        assert_eq!(
            yaml.matches("- hypothesis:").count(),
            steps.len(),
            "GH-{id}: one claim per step"
        );
        assert!(
            !yaml.contains("statement: \"\""),
            "GH-{id}: hollow obligation:\n{yaml}"
        );
        assert!(
            !yaml.contains("hypothesis: \"\""),
            "GH-{id}: hollow claim:\n{yaml}"
        );
        for step in &steps {
            assert!(
                yaml.contains(&yaml_quote(&step.implication)),
                "GH-{id}/{}: the claim text is rendered verbatim",
                step.id
            );
        }
        seen += 1;
    }
    assert_eq!(seen, 10, "all ten aprender contracts were exercised");
}
