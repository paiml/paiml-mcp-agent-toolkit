# KAIZEN-0012: Refinement-type contract synthesis for pmat-verified functions

**Source paper:** arxiv:2510.25015 — "VeriStruct: AI-assisted Automated Verification of Data-Structure Modules in Verus" (Mar 2026) + arxiv:2602.02881 — "Learning-Infused Formal Reasoning: From Contract Synthesis to Artifact Reuse"
**Category:** provable-contracts
**Priority:** low
**Effort:** L

## Problem
pmat's `pmat work` contract system tracks *work items* (CB-xxxx) but not *function-level specifications*. VeriStruct and the learning-infused-formal-reasoning vision paper argue that lightweight AI-assisted spec synthesis (pre/post-conditions, invariants) is now tractable for real Rust code.

## Proposed improvement
Add `pmat spec synthesize --function <path>::<name>` that:
1. Extracts the function's signature + body + callers + tests.
2. Runs a constrained LLM prompt to propose Verus-compatible `requires` / `ensures` / invariant clauses.
3. Optionally invokes Verus (if present on PATH) to verify; records results in `.pmat/specs/<function_id>.verus`.
4. Surfaces in `pmat query` as `spec_coverage: 0..1` — fraction of call sites with a verified contract.

## Impact
- Paper demonstrates practical Verus contract synthesis on real data structures.
- Gives pmat a novel metric (`spec_coverage`) orthogonal to line coverage — measures *semantic* coverage.
- Aligns with pmat's existing "contract-first enforcement" principle.

## Implementation sketch
1. `pmat/specs/` module: storage + render to Verus syntax.
2. External process integration with `verus` binary (graceful fallback if absent).
3. Aggregation: `pmat analyze spec-coverage` reports spec_coverage per file/module.
4. Intentionally scope to Rust only in v1.

## Acceptance criteria
- `pmat spec synthesize` produces a syntactically valid Verus stub in ≥50% of trivial pure functions on pmat itself.
- `pmat analyze spec-coverage` reports aggregate.
- Documented limitation: non-Rust, non-pure, IO-heavy functions unsupported.
- **No MCP schema change in v1** — CLI only.
