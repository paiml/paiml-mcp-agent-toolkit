# KAIZEN-0009: Attention-probe-style scalable fault localization signal

**Source paper:** arxiv:2502.13966 — "Where's the Bug? Attention Probing for Scalable Fault Localization" (Feb 2025) + arxiv:2501.18005 — "Fault Localization via Fine-tuning LLMs with Mutation Generated Stack Traces"
**Category:** other
**Priority:** medium
**Effort:** L

## Problem
pmat's current fault-annotation (`--faults`) is rule-based: unwrap / panic / unsafe counts. This catches *potential* fault sites but not *likely* fault sites. The Bug-Attention-Probe paper learns scalable fault-localization without direct labels and outperforms prompting of large LLMs. The mutation-stack-trace paper achieves 66.9% localization accuracy on HANA crashes via fine-tuning on 4.1M mutants.

## Proposed improvement
Add a `--probe-faults` flag that, on top of rule-based faults, emits a *learned* fault-likelihood score per function using a lightweight attention-probe trained on pmat's own `.pmat-work/*/contract.json` bug-history + fix-diffs. Because we already track "which function was changed to close CB-xxxx", we have free training data.

## Impact
- Paper reports SOTA fault localization without labels, outperforming LLM prompting.
- pmat's contract history is an untapped supervised signal — this converts it into a first-class prediction.

## Implementation sketch
1. Mine fix-commits from `.pmat-work/*/contract.json` → `(function_id, was_root_cause: bool)` pairs.
2. Train a simple probe (aprender linear classifier) over pmat's existing function-level features: complexity, churn, duplicates, entropy, centrality, existing faults.
3. Serve predictions via `pmat query --probe-faults` and the existing MCP tools.
4. Nightly retrain when ≥10 new contracts closed.

## Impact (quantified)
- If probe reaches paper's 66–74% top-1 accuracy, the current rule-based unwrap count (which has ~20% precision for real bugs per informal review) becomes a minor contributor.

## Acceptance criteria
- `pmat query "X" --probe-faults` adds `probe_fault_score ∈ [0,1]` per result.
- Precision-at-10 on held-out contracts ≥50% on pmat itself (dogfood baseline).
- Probe artifact <1 MB, loadable in <100 ms.
- **No MCP schema change** — `probe_fault_score` is an additive optional field on existing query results.
