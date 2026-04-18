# KAIZEN-0002: Pointer-based large-result returns from pmat MCP tools

**Source paper:** arxiv:2511.22729 — "Solving Context Window Overflow in AI Agents" (Labate, Nov 2025)
**Category:** sub-agent-behavior
**Priority:** high
**Effort:** M

## Problem
`pmat query --include-source --limit 30` or `pmat context --format llm-optimized` can return 50–300 KB of text per call. When a sub-agent triggers this from its parent's conversation, the result is either truncated (lossy) or blows the context window (OOM). Today pmat's MCP tools return raw blobs; agents must then stream them through their own context.

## Proposed improvement
Adopt the paper's *pointer-to-value* pattern. Any MCP tool whose result exceeds a configurable threshold (e.g., 8 KB) should:
1. Persist the full payload to `.pmat/mcp-artifacts/<uuid>.json` (or `.md`).
2. Return a compact summary `{"artifact_id": "...", "preview": "first 40 lines", "size_bytes": N, "row_count": N, "fetch_tool": "pmat_read_artifact"}`.
3. Add companion `pmat_read_artifact` and `pmat_slice_artifact(start, end)` MCP tools so agents can request just what they need.

## Impact
- The paper demonstrates lossless handling of arbitrary-size tool responses.
- Projected: 30–50% context-window savings on deep-context / large-query workloads. Enables agents to `pmat context` on monorepos (currently unusable because output >1 MB).

## Implementation sketch
1. New crate module `server/src/mcp_pmcp/artifacts.rs` with artifact store (file-backed, LRU eviction at 500 MB).
2. Wrap every `Tool::call()` result in `maybe_spill_to_artifact(result, threshold)`.
3. Add `pmat_read_artifact {id, offset?, limit?}` and `pmat_slice_artifact {id, jsonpath}` tools.
4. Configure threshold via `.pmat/mcp-config.toml`.

## Acceptance criteria
- Any tool result >8 KB is spilled; preview+id returned.
- `pmat_read_artifact` round-trip passes integration test.
- Benchmark: running `pmat context` via MCP no longer truncates or fails on a 50 KLOC repo.
- **Requires MCP tool schema changes — two new tools added (additive, non-breaking).**
