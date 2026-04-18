# KAIZEN-0007: Experiential heuristics memory for pmat work contracts

**Source paper:** arxiv:2603.24639 — "Experiential Reflective Learning" (ICLR 2026 MemAgents Workshop) + arxiv:2512.12818 — "Hindsight is 20/20: Building Agent Memory that Retains, Recalls, and Reflects"
**Category:** sub-agent-behavior
**Priority:** medium
**Effort:** M

## Problem
Each `pmat work` contract starts fresh. When an agent (or human) closes a CB-xxxx with a root-cause-and-fix, the learning is lost — the next similar bug re-runs the same Five-Whys from scratch. The ERL paper shows agents that reflect into a *structured heuristic bank* (trigger conditions + recommended action) self-improve on repeat tasks.

## Proposed improvement
On contract closure, `pmat work close` automatically:
1. Runs a reflection template over the contract's Five-Whys output + fix diff.
2. Emits a `Heuristic {trigger_condition, recommended_action, evidence_contract_ids[], confidence}`.
3. Appends it to `.pmat/heuristics.jsonl`.
4. On new contract creation, `pmat work new` runs `pmat heuristics query --topic <title>` and surfaces the top-3 relevant heuristics as a *starter prompt* for the agent.

## Impact
- The ERL paper reports measurable compounding improvement on repeat tasks.
- pmat already captures Five-Whys + falsification claims; heuristics is the natural next artifact.
- Closes the loop between Toyota Way root-cause-analysis and future prevention.

## Implementation sketch
1. `pmat/heuristics/` module: schema + JSONL storage + embed-index (reuse aprender embeddings).
2. Hook into `work.close()` to trigger reflection.
3. New CLI: `pmat heuristics list | query | prune`.
4. Integrate with `pmat work new` to pre-seed relevant heuristics.
5. MCP tool `pmat_heuristics_query` so sub-agents can self-retrieve.

## Acceptance criteria
- Closing a contract emits one heuristic JSONL line.
- `pmat heuristics query "stack overflow"` returns ranked prior heuristics.
- `pmat work new` surfaces top-3 related heuristics in the contract scaffold.
- **New MCP tool `pmat_heuristics_query` — additive.**
