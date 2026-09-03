# PMAT MCP Tools Catalog

**Protocol Version**: MCP v2024-11-05
**Transport**: stdio (JSON-RPC 2.0)
**Server**: `MCP_VERSION=1 pmat` (equivalently `pmat --mode mcp`) — `src/mcp_pmcp/simple_unified_server.rs`
**Total Tools**: 20
**Last Updated**: 2026-08-28

> Exact, authoritative input schemas for every tool are published by the server
> itself — call `tools/list` after `initialize`. This catalog gives the tool name,
> category, and purpose.
>
> **The counts in this file are checked.** `tools_doc_states_the_live_tool_count`
> (`src/mcp_pmcp/tool_manifest.rs`) reads this document and compares every number
> it states — the `Total Tools` header, every "N tools" sentence, and the
> per-section counts in both the table of contents and the section headings —
> against `LIVE_MCP_TOOLS.len()`. Update one and not the others and the test
> fails. That guard exists because this catalog had, in a single revision,
> asserted 16 in one paragraph and 20 in another while the server advertised 19,
> with a table of contents that reached 20 by counting four tools that had been
> unregistered and omitting three that had been added.
>
> **`pmat agent mcp-server` does not run this server.** That subcommand starts
> `ClaudeCodeAgentMcpServer`, a different surface of four agent-monitoring tools,
> and it is compiled out entirely unless `--features agent-daemon` is set — which
> is not in `default`. This document described it as the entry point for years.

## Table of Contents

1. [Core Analysis Tools (6)](#core-analysis-tools-6)
2. [Forensic Analyzers (3)](#forensic-analyzers-3)
3. [Quality Tools (4)](#quality-tools-4)
4. [Git & Context Tools (3)](#git--context-tools-3)
5. [Agent-Context Tools (4)](#agent-context-tools-4)
6. [Unregistered: the refactor.* tools](#unregistered-the-refactor-tools)
7. [Error Handling](#error-handling)
8. [Tool Discovery](#tool-discovery)

---

## Core Analysis Tools (6)

### 1. `analyze_complexity`
Computes cyclomatic and cognitive complexity per function/file and flags functions
over configured thresholds. Typical args: `paths`, `top_files`, `threshold`.

### 2. `analyze_satd`
Detects self-admitted technical debt — `TODO` / `FIXME` / `HACK` style comments —
and classifies them. Typical args: `paths`, `include_resolved`, `include_tests`.

### 3. `analyze_dead_code`
Finds unreachable or unused code (functions, types, modules). For Rust this is
cargo/rustc-backed for accuracy; the walk is `.gitignore`/hidden-dir aware (it does
not descend into hidden git worktrees). Typical args: `paths`, `include_tests`.

### 4. `analyze_dag`
Builds the project dependency graph — call graph, import graph, inheritance, or the
full dependency DAG. Typical args: `paths`, `dag_type`. Advertises a non-empty
schema requiring `paths` (fixed in v3.18.2).

### 5. `analyze_deep_context`
Runs the full deep-context pipeline (AST, complexity, churn, dead code) over the
given paths — the data behind `pmat context`. Typical args: `paths`. Advertises a
non-empty schema requiring `paths` (fixed in v3.18.2).

### 6. `analyze_big_o`
Classifies the Big-O time complexity of functions. Typical args: `paths`,
`top_files`. Advertises a non-empty schema requiring `paths` (fixed in v3.18.2).

---

## Forensic Analyzers (3)

Registered in **#1029**. These three shipped CLI-only in 3.32.0 because the MCP
tool list was hand-curated beside `AnalyzeCommands` rather than derived from it;
which `analyze` subcommands belong on MCP is now decided by a total match in
`cli::analyze_mcp_exposure`, so the next variant cannot reach a release
undeclared. Each takes one `project_path`, not a `paths` array.

### 7. `analyze_reachability`
Reports tracked `.rs` files that no compilation unit reaches — orphaned modules
that compile to nothing and whose tests never run.

### 8. `analyze_hardcoded_paths`
Finds machine-specific absolute paths baked into source (a user's home, a nix store
hash, a build root) — correct where they were written, inert everywhere else.

### 9. `analyze_vacuous_tests`
Finds `#[test]` functions that cannot fail: no assertion, an assertion over
constants, or a body that silently returns when a fixture is missing.

---

## Quality Tools (4)

### 10. `quality_gate`
Runs the `pmat quality-gate --checks all` suite (complexity, dead code, SATD,
entropy, security, duplicates, coverage, documentation sections, provability) plus a
TDG score, and returns a **pass/fail verdict**. Any check a path could not answer is
named in `not_measured` and, with its reason, in `checks.not_run`. The verdict
comparison was fixed in v3.18.2 (`Grade`'s derived `Ord` is reversed; a single
`Grade::meets_threshold()` now drives the decision) — earlier versions could return
an inverted `passed`.

### 11. `quality_check_content`
**Never writes.** Grades proposed `content` for `file_path` against the project's
quality gate and returns it with a verdict. Args: `file_path`, `content`, `mode`
(`strict` / `advisory` / `auto_fix`), `quality_config`. The response carries
`written: false` — always: the tool has no writer, and the only layer that can gate
a client's own writes is the harness `PreToolUse` hook. `advisory` returns the
content with `status: rejected` when the gate failed (it no longer launders a
failing verdict as `accepted`); a client `quality_config` may only tighten the
project's `[quality]` in `pmat.toml`; and `metrics.satd_count` always equals the
number of `violations[]` of type `satd` in the same response (CRUX-10, #1151).
Until 3.36.0 this tool was `quality_proxy`, took an `operation` of
`write`/`edit`/`append`, and this catalog claimed the tool wrote to disk — nine live calls
returned `accepted` and not one created a file. A request still carrying
`operation` gets `-32602`.

### 12. `quality_proxy`

One-release alias of `quality_check_content` (CRUX-10, #1151): the same handler,
the same schema, the same verdict, under the name the tool carried before
3.36.0. It is a separate entry in `tools/list` so a client pinned to the old
name keeps working for one release; it is removed the release after.

### 13. `pdmt_deterministic_todos`
Generates deterministic, quality-enforced todo lists from a list of requirements.
IDs are deterministic UUIDv8s derived from seed/index/requirement (v3.18.2) —
byte-identical output for identical input, so results can be cached, diffed, and
reproduced across agents.

---

## Git & Context Tools (3)

Two of these three are named for a mutation they do not perform. The names are
historical aliases held for wire compatibility; the behaviour below is what the
handlers actually do, and it is what `tools/list` says.

### 14. `git_operation`
**Read-only.** Despite the name, this is `GitStatusTool`: it queries git
working-tree status for the given repository path and performs no git operation of
any kind. Args: `path`.

### 15. `generate_context`
Generates project context (file tree plus an optional dependency graph) for LLM/agent
consumption — the MCP equivalent of `pmat context`. Args: `paths`, `format` (`json`),
`max_depth`, `include_dependencies`.

### 16. `scaffold_project`
**Writes nothing.** Despite the name, this is `ContextSummaryTool`: it produces a
high-level project summary for the given paths. It does not scaffold a project and
does not create files. Args: `paths`, `level` (`brief` / `normal` / `detailed`).

---

## Agent-Context Tools (4)

Added in **KAIZEN-0165**. Backed by the SQLite + FTS5 code index, these are the
primary code-intelligence surface for autonomous agents. Their schemas and
descriptions are generated from `mcp_tool_schemas/*.json` by `build.rs`
(KAIZEN-0178), so they cannot drift from what the handler advertises.

### 17. `pmat_query_code`
Searches code functions by natural-language query with TDG quality filtering.
Returns semantically ranked results with complexity, fault patterns, and call-graph
context. The MCP analogue of `pmat query`.

### 18. `pmat_get_function`
Returns detailed information about a function by its ID: full metadata including
source code, quality metrics, and SATD markers. (Source retrieval was restored in
v3.18.2 after an incremental-save bug that wiped the `source` column.)

### 19. `pmat_find_similar`
Finds functions similar to a reference function — related code, potential
duplicates, or other implementations of the same pattern.

### 20. `pmat_index_stats`
Reports code-index statistics: function counts, quality distribution, index health.

---

## Unregistered: the refactor.* tools

`refactor.start` / `refactor.nextIteration` / `refactor.getState` / `refactor.stop`
were **removed from the advertised surface in EV-0 (#999)**. They are absent from
`tools/list` and from `mcp.json`, and they are not counted above.

They were four of the twenty tools then advertised, and the engine behind them is
not an analyzer. `find_violations` (`src/models/refactor_impls.rs:140-145`) returns a
hardcoded `HighComplexity` violation at *"line 100, column 1"* for any file whose
**path contains the substring `"complex"`**, and nothing otherwise. It reads no
source and builds no AST.

This document previously disclosed that in prose, and a test asserted the disclosure
was present in each tool's description. A disclosure is not a guard: an agent calls
`tools/list`, sees four tools, and acts on fabricated line numbers and a
`suggested_fix`. The test now asserts **absence** instead.

The state machine, handlers and tests remain in the tree. Re-registering is four
lines in `SimpleUnifiedServer::run()` — do it when the engine analyses something.

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

or, in one shot from a shell:

```bash
printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' | pmat --mode mcp
```

The response lists all 20 tools with their names, descriptions, and input schemas
(every tool advertises non-empty metadata — pinned by tests since v3.18.2).

The packaged `mcp.json` at the repository root advertises the same 20 tools with the
same descriptions. It is **generated**, never hand-edited: regenerate it with
`cargo test --lib regenerate_mcp_json -- --ignored` (or `pmat mcp manifest --write`)
after changing `LIVE_MCP_TOOLS`.

---

**Maintained by**: PAIML
**Server source**: `src/mcp_pmcp/simple_unified_server.rs`
**Tool list source of truth**: `src/mcp_pmcp/tool_manifest.rs` (`LIVE_MCP_TOOLS`)
