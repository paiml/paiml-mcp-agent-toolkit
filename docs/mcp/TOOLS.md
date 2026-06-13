# PMAT MCP Tools Catalog

**Protocol Version**: MCP v2024-11-05
**Transport**: stdio (JSON-RPC 2.0)
**Server**: `pmat agent mcp-server` (`src/mcp_pmcp/simple_unified_server.rs`)
**Total Tools**: 20 (16 core + 4 agent-context)
**Last Updated**: 2026-06-13 (v3.19.2)

> Exact, authoritative input schemas for every tool are published by the server
> itself — call `tools/list` after `initialize`. This catalog gives the tool name,
> category, and purpose. The 20-tool set is pinned by tests and was validated
> end-to-end (per-tool calls + 8-way concurrent sessions + framing purity) in v3.18.2.

## Table of Contents

1. [Core Analysis Tools (6)](#core-analysis-tools-6)
2. [Refactoring Tools (4)](#refactoring-tools-4)
3. [Quality Tools (3)](#quality-tools-3)
4. [Git & Context Tools (3)](#git--context-tools-3)
5. [Agent-Context Tools (4)](#agent-context-tools-4)
6. [Error Handling](#error-handling)
7. [Tool Discovery](#tool-discovery)

---

## Core Analysis Tools (6)

### 1. `analyze_complexity`
Computes cyclomatic and cognitive complexity per function/file and flags functions
over configured thresholds. Typical args: `path`, `top_files`, `format`.

### 2. `analyze_satd`
Detects self-admitted technical debt — `TODO` / `FIXME` / `HACK` / `XXX` style
comments — and classifies them. Typical args: `path`, `format`.

### 3. `analyze_dead_code`
Detects dead / unreachable code. For Rust this is cargo/rustc-backed for accuracy;
the walk is `.gitignore`/hidden-dir aware (it does not descend into hidden git
worktrees). Typical args: `path`, `top_files`, `format`.

### 4. `analyze_dag`
Builds the project dependency / call-graph (DAG). Typical args: `path`,
`target_nodes`, `enhanced`, `format` (Mermaid or JSON). Advertises a non-empty
schema requiring `paths` (fixed in v3.18.2).

### 5. `analyze_deep_context`
Generates full AST-based deep context for a project (the data behind
`pmat context`). Typical args: `paths`, `format` (`markdown` / `json` /
`llm-optimized`). Advertises a non-empty schema requiring `paths` (fixed in v3.18.2).

### 6. `analyze_big_o`
Estimates Big-O algorithmic complexity of functions. Typical args: `paths`.
Advertises a non-empty schema requiring `paths` (fixed in v3.18.2).

---

## Refactoring Tools (4)

These drive a single **stateful, resumable refactoring session** (state held by the
server's state manager).

### 7. `refactor.start`
Begins an interactive refactoring session for a target.

### 8. `refactor.nextIteration`
Advances the refactoring state machine by one step.

### 9. `refactor.getState`
Returns the current refactoring session state.

### 10. `refactor.stop`
Ends the refactoring session.

> Note: the `refactor.*` analysis engine is currently **simulated** — violations are
> synthesized from filename patterns rather than real AST analysis. The tool
> descriptions disclose this (v3.18.2) so agents don't act on the output as if it
> were a real refactoring engine.

---

## Quality Tools (3)

### 11. `quality_gate`
Runs the quality gate (complexity / SATD / lint / etc.) and returns a **pass/fail
verdict**. The verdict comparison was fixed in v3.18.2 (`Grade`'s derived `Ord` is
reversed; a single `Grade::meets_threshold()` now drives the decision) — earlier
versions could return an inverted `passed`.

### 12. `quality_proxy`
Quality-proxy metrics for fast gating.

### 13. `pdmt_deterministic_todos`
Deterministic todo / task generation (PDMT). IDs are deterministic UUIDv8s derived
from seed/index/requirement (v3.18.2) — byte-identical output for identical input,
so results can be cached, diffed, and reproduced across agents.

---

## Git & Context Tools (3)

### 14. `git_operation`
Git context operations (history, churn, blame) used to enrich analysis.

### 15. `generate_context`
Generates AI-ready project context — the MCP equivalent of `pmat context`. Typical
args: `path`, `format` (`markdown` / `json` / `llm-optimized`).

### 16. `scaffold_project`
Scaffolds a new project or files from PMAT templates.

---

## Agent-Context Tools (4)

Added in **KAIZEN-0165**. Backed by the SQLite + FTS5 code index, these are the
primary code-intelligence surface for autonomous agents.

### 17. `pmat_query_code`
Semantic code search by intent. Returns quality-annotated functions (TDG grade,
complexity, fault patterns). The MCP analogue of `pmat query`.

### 18. `pmat_get_function`
Fetches a function's full source plus its quality metrics. (Source retrieval was
restored in v3.18.2 after an incremental-save bug that wiped the `source` column.)

### 19. `pmat_find_similar`
Finds functions similar to a given one — useful for refactoring and de-duplication.

### 20. `pmat_index_stats`
Reports code-index health and statistics (function count, index freshness, etc.).

---

## Error Handling

All tools follow consistent JSON-RPC error patterns:

```json
{
  "code": -32602,
  "message": "Path does not exist: /invalid/path",
  "data": {
    "path": "/invalid/path",
    "suggestion": "Please provide a valid file or directory path"
  }
}
```

**Error codes**:
- `-32700`: Parse error
- `-32600`: Invalid request
- `-32601`: Method not found
- `-32602`: Invalid parameters
- `-32603`: Internal error

**Best practices**:
- Always check the `status` field in responses.
- Use `data.suggestion` for actionable error recovery.

---

## Tool Discovery

Discover the tools (with full, authoritative schemas) at runtime:

```jsonc
// after `initialize`
{ "jsonrpc": "2.0", "id": 1, "method": "tools/list" }
```

The response lists all 20 tools with their names, descriptions, and input schemas
(every tool advertises non-empty metadata — pinned by tests since v3.18.2).

---

**Maintained by**: PAIML
**Server source**: `src/mcp_pmcp/simple_unified_server.rs`
**Last Updated**: 2026-06-13 (v3.19.2)
