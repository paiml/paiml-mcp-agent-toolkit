# KAIZEN-0004: Sub-agent "budget hint" headers on pmat tool responses

**Source paper:** arxiv:2511.03728 — "Efficient On-Device Agents via Adaptive Context Management" (Nov 2025) + arxiv:2508.21433 — "The Complexity Trap: Simple Observation Masking Is as Efficient as LLM Summarization"
**Category:** sub-agent-behavior
**Priority:** medium
**Effort:** S

## Problem
Orchestrator agents (Claude Code main loop, CAID dispatcher) need to decide whether to keep or discard a tool observation to stay under context budget. Today pmat results have no metadata hinting at their "value density" — a 20 KB `pmat query --coverage-gaps` is full of signal, while a 20 KB `pmat scaffold-agent` is mostly boilerplate. Agents default to keeping everything, then hit the 200k window.

## Proposed improvement
Every MCP tool response emits a `meta` envelope:
```json
{
  "meta": {
    "tokens_estimate": 4821,
    "value_density": 0.82,        // signal:noise heuristic
    "safe_to_mask_after": 5,      // turns — paper's observation-masking cue
    "summary_one_line": "Found 12 uncovered functions; top impact: tokenize() 42 lines",
    "persistent_key": "coverage-gaps-2026-04-18"
  }
}
```
Orchestrators can then drop or summarize results once `safe_to_mask_after` turns have passed.

## Impact
- The complexity-trap paper shows simple observation-masking matches LLM-summarization at a fraction of cost. Giving agents the mask hint converts a reactive strategy into a proactive one.
- Projected 15–25% context savings in long sessions (>50 turns) observed in the cited paper.

## Implementation sketch
1. Compute `tokens_estimate` via tiktoken-rs (already a transitive dep).
2. Emit `summary_one_line` from the existing result formatter (pick top-1 finding).
3. Populate `safe_to_mask_after` per tool (e.g., 2 for `query`, ∞ for `verify-citation`).
4. Document semantics in `docs/claude-integration-final.md`.

## Acceptance criteria
- Every MCP tool response has `meta` envelope.
- Integration test: orchestrator can mask past results based on `safe_to_mask_after` and still resume work via `persistent_key`.
- **MCP schema additive change — new optional `meta` envelope on every response.**
