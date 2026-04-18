# KAIZEN-0003: Citation-grounded query results to prevent sub-agent hallucination

**Source paper:** arxiv:2512.12117 — "Citation-Grounded Code Comprehension: Preventing LLM Hallucination Through Hybrid Retrieval and Graph-Augmented Context" (Dec 2025)
**Category:** sub-agent-behavior
**Priority:** high
**Effort:** M

## Problem
The paper measures that 100% of fabricated "cite non-existent file" or "invalid line range" errors can be prevented by mechanical verification. Sub-agents using pmat today sometimes synthesize plausible-but-wrong claims ("this function calls X") after reading pmat output, because results lack a machine-checkable, structured citation block. `pmat query --include-source` gives file/line but agents paraphrase and drift.

## Proposed improvement
Augment every pmat query result with a structured `citations` block:
```json
{
  "citations": [
    {"file": "server/src/foo.rs", "start_line": 123, "end_line": 160, "sha": "b6d7be...", "content_hash": "sha256:..."}
  ]
}
```
Add `pmat verify-citation --file F --start S --end E --sha H` that returns the exact bytes or a cryptographic mismatch, so downstream agents (or a second verifier agent) can mechanically check that a synthesized claim refers to real code at a known revision.

## Impact
- Paper reports 92% citation accuracy vs. 74–78% baseline, and prevents 100% of "hallucinated file" errors.
- Enables a two-phase agent workflow: (1) query pmat → receive cited snippets, (2) verify cited ranges before committing to a plan.

## Implementation sketch
1. Add `Citation` struct to `QueryResult`; populate via existing AST index offsets (we already have these).
2. Add `--with-citations` flag (on by default for MCP callers).
3. Implement `pmat verify-citation` using git blob lookup when `sha` is set.
4. Extend `pmat-book` chapter on "Agent Workflows" to show citation-verify pattern.

## Acceptance criteria
- Every `pmat query` result with `--include-source` emits a `citations[]` array.
- `pmat verify-citation` returns byte-exact match or specific failure (`file_missing | sha_mismatch | range_outside_file | content_hash_mismatch`).
- Book chapter 15 (agent workflows) shows end-to-end example.
- **MCP schema change — new optional `citations` field on query-result tools (additive).**
