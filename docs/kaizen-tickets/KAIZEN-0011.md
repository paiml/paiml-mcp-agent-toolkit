# KAIZEN-0011: Mutation-prioritized coverage-gap targeting

**Source paper:** arxiv:2505.05584 — "PRIMG: Efficient LLM-driven Test Generation Using Mutant Prioritization" (May 2025) + arxiv:2403.16218 — "CoverUp: Coverage-Guided LLM-Based Test Generation"
**Category:** other
**Priority:** medium
**Effort:** M

## Problem
`pmat query --coverage-gaps --rank-by impact` ranks by `missed_lines * pagerank / complexity`. That's a static proxy for "how important is this to test"; it doesn't distinguish between "uncovered trivial getter" and "uncovered branch that, if mutated, survives". PRIMG shows mutant-based prioritization produces smaller, higher-impact test suites.

## Proposed improvement
Add `--rank-by mutation-impact` that scores uncovered functions by:
1. How many *surviving mutants* they'd produce if tested (estimated from line count × operator kinds).
2. How many downstream callers would be protected (PageRank on CALLS graph).

Integrate with a new `pmat test-plan` that emits a ranked to-do list for LLM-driven test-generation tools (e.g., "write tests for fn X covering branch B; kill mutants M1, M2").

## Impact
- PRIMG reports significantly smaller test suites at equivalent mutation coverage.
- Converts pmat from "here are gaps" into "here's the optimal test-writing plan for an agent".

## Implementation sketch
1. Lightweight mutation-operator estimator in aprender (no need for actual mutation-testing execution — estimate survivor count).
2. New ranking function in `query_handler` behind `--rank-by mutation-impact`.
3. New `pmat test-plan` command + MCP tool `pmat_test_plan {target, budget}`.
4. Integration guide: Claude Code → pmat_test_plan → test-writing sub-agent.

## Acceptance criteria
- `--rank-by mutation-impact` works and differs from `impact`.
- `pmat test-plan --budget 20` returns a ranked plan of ≤20 items with reasoning.
- Book chapter: "Generating tests with sub-agents".
- **One new MCP tool `pmat_test_plan` — additive.**
