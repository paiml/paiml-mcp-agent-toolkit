// Included from check_cot_proof.rs — do NOT add `use` imports or `#!` attributes here.
// MACS-009 guard: the digest emitted by `pmat work cot derive`
// (crate::models::work_cot::canonical_cot_sha) and the digest recomputed by
// CB-1646 (this module's canonical_cot_sha) must never diverge.

#[cfg(all(test, not(coverage_nightly)))]
mod tests_macs_digest_equality {
    use super::*;

    #[test]
    fn derive_digest_matches_cb1646_recomputation() {
        let contract: Value = serde_json::from_str(
            r#"{"chain_of_thought": [
                {"id": "CoT-1", "assumption": {"text": "x", "references": ["E1"]},
                 "implication": "y", "evidence_method": "cargo test z",
                 "discharged_by": "E1"},
                {"step": 2, "question": "legacy?", "answer": "yes"}
            ]}"#,
        )
        .expect("fixture parses");
        assert_eq!(
            canonical_cot_sha(&contract),
            crate::models::work_cot::canonical_cot_sha(&contract),
            "cot derive and CB-1646 must share one canonical digest"
        );
    }
}
