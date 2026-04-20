# KAIZEN-0001: Sanitize pmat MCP tool descriptions via the "smell" taxonomy

**Source paper:** arxiv:2602.14878 — "Model Context Protocol (MCP) Tool Descriptions Are Smelly! Towards Improving AI Agent Efficiency with Augmented MCP Tool Descriptions" (Hasan et al., 2026)
**Category:** sub-agent-behavior
**Priority:** high
**Effort:** S

## Problem
Hasan et al. empirically found 97.1% of public MCP tool descriptions have quality "smells" (ambiguous parameters, missing examples, inconsistent units, unbounded return shape). pmat's MCP server exposes ~20 tools (`pmat_query_code`, `pmat_get_function`, `pmat_find_similar`, `pmat_index_stats`, etc.) whose descriptions were written iteratively and have not been audited against a canonical checklist. Sub-agents that see these schemas pay a token tax and sometimes mis-parameterize calls (e.g., passing globs where regexes are expected on `--exclude-file` vs `--exclude`).

## Proposed improvement
Apply the paper's augmentation pattern to every `#[tool]` description in `server/src/mcp_pmcp/`:
1. Enforce a standard preamble: *purpose*, *when-to-use*, *when-not-to-use*, *output-shape*, *failure-mode*.
2. Add one canonical JSON example per tool (paper shows +18% first-call success).
3. Explicit parameter units (e.g., `limit: number of functions, 1..200, default 10` not just `limit: number`).
4. Add a `cost_hint` field ("cheap" / "expensive") so orchestrators can budget.

## Impact
- Paper reports 6x token reduction on initial tool overhead for capable agents and measurable increase in tool-call correctness.
- pmat agents today spend 2–4k tokens just on tool discovery; projected savings ~1.5k tokens per session.

## Implementation sketch
1. Audit every tool in `server/src/mcp_pmcp/tools/*.rs` — produce a smell report.
2. Update doc comments (these become MCP descriptions via pmcp macros).
3. Add a `mcp-lint` subcommand that re-checks descriptions against the smell taxonomy on CI.
4. Add golden test: snapshot the JSON schema emitted to MCP clients and regress on diff.

## Acceptance criteria
- `pmat mcp-lint` reports 0 smells across all exposed tools.
- Each tool has ≥1 in-description example and explicit parameter units/bounds.
- MCP schema snapshot test passes; diff reviewed in PR.
- **Requires MCP tool schema changes — flagged for client compatibility review.**
