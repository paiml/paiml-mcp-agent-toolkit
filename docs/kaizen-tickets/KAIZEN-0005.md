# KAIZEN-0005: `pmat sub-agent delegate` primitive for isolated task contexts

**Source paper:** arxiv:2603.05344 — "Building AI Coding Agents for the Terminal: Scaffolding, Harness, Context Engineering, and Lessons Learned" (Mar 2026) + arxiv:2601.07577 — "Beyond Entangled Planning: Task-Decoupled Planning for Long-Horizon Agents"
**Category:** sub-agent-behavior
**Priority:** high
**Effort:** L

## Problem
Sub-agent delegation is today an *implicit* capability of the client (e.g., Claude Code's Task tool) — pmat has no first-class way to scope *which tools* a sub-agent may call, nor to persist/resume its contract state when invoked. Agents that use pmat as an orchestrator end up re-reading the same context.idx on every invocation.

## Proposed improvement
Add `pmat sub-agent delegate --contract-id <id> --tools <allowlist> --read-only` that:
1. Opens a scoped MCP session with only the listed tools available (least-privilege).
2. Loads the contract's *prior observations* from `.pmat-work/<id>/observations.jsonl` so the sub-agent resumes with warm context.
3. Streams new observations back and closes the session, appending a compact summary to the parent contract.

This mirrors the "Subagent Orchestration" pattern from the arxiv:2603.05344 paper (Claude Code scaffolding).

## Impact
- The paper shows isolated sub-agents with filtered tool access dramatically reduce context pollution in the parent.
- Enables pmat as a *delegation primitive* — a competitive alternative to bespoke agent frameworks.

## Implementation sketch
1. New CLI: `pmat sub-agent delegate` — spawns a constrained MCP server as a child process.
2. Tool allowlist enforced at the MCP transport layer (reject unlisted tool-call requests).
3. Observation log at `.pmat-work/<id>/observations.jsonl`, replayable on resume.
4. MCP tool `pmat_delegate_task` so parent agents can programmatically spawn.
5. Contract-file format (extend `.pmat-work/<id>/contract.json`) to record sub-agent lineage.

## Acceptance criteria
- `pmat sub-agent delegate --contract-id X --tools pmat_query_code,pmat_get_function` succeeds; calling an excluded tool returns `PermissionDenied`.
- Sub-agent observations are appended to the contract and visible in `pmat work show X`.
- `pmat_delegate_task` MCP tool works with Claude Code.
- **New MCP tool added (`pmat_delegate_task`) — additive schema change.**
