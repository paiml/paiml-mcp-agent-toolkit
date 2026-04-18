# KAIZEN-0015: `pmat_index_stats_incremental` streaming tool for responsive agents

**Source paper:** arxiv:2604.11462 — "Escaping the Context Bottleneck: Active Context Curation for LLM Agents via Reinforcement Learning" + arxiv:2604.08224 — "Externalization in LLM Agents: A Unified Review of Memory, Skills, Protocols and Harness Engineering"
**Category:** sub-agent-behavior
**Priority:** low
**Effort:** S

## Problem
When an agent runs `pmat index rebuild` on a large monorepo, the tool blocks for 10–30 s with no feedback. Modern harnesses (per the "Externalization" survey) reward *streaming progress updates* — the agent can plan next steps while indexing runs. Today pmat returns one big reply at the end.

## Proposed improvement
Add MCP streaming protocol support for long-running tools. Specifically:
1. `pmat_index_rebuild` emits progress events: `{phase: "parsing", files_done: 120, total: 5000, eta_seconds: 18}`.
2. `pmat_query` for very large corpora emits partial result batches.
3. Leverage MCP's optional `progress` notification channel.

Per the "Active Context Curation" paper, agents that observe progress can decide to kill long-running calls and re-plan — critical for sub-agent delegation.

## Impact
- Better agent ergonomics on monorepos: fewer timeouts, actionable progress.
- Unblocks sub-agent patterns where the parent wants to dispatch other work in parallel.

## Implementation sketch
1. Add `ProgressReporter` trait; wire into `save()` / `load()` / `rebuild()` in agent-context-index.
2. Use pmcp SDK's notification API (already supported; not currently wired).
3. Document required client support in MCP integration docs.

## Acceptance criteria
- `pmat index rebuild` via MCP emits ≥3 progress events on a typical session.
- Cancellable mid-flight (agent-side timeout → clean cancel).
- **MCP protocol usage change — additive notifications, but requires client to handle them (graceful degradation for non-supporting clients).**
