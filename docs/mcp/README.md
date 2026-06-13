# PMAT MCP Documentation

**Model Context Protocol** (MCP) integration for PMAT — the PAIML MCP Agent Toolkit.

## Quick Links

- **[Tools Catalog](TOOLS.md)** - Complete list of 20 MCP tools with purposes and inputs
- **[Examples](../../examples/)** - Working code examples

## What is MCP?

Model Context Protocol (MCP) is a standardized protocol for AI agents to interact
with tools and services. PMAT exposes its code-analysis capabilities over MCP so
agents (Claude Code, Cline, and any MCP client) can analyze code, grade technical
debt, search a semantic index, and drive refactoring sessions — all over a single
stdio JSON-RPC connection.

## Available Tools (20 total)

The unified server (`pmat agent mcp-server`) registers **20 tools — 16 core + 4
agent-context** (the exact set validated end-to-end over stdio JSON-RPC, including
8-way concurrent sessions, in v3.18.2):

### Core Analysis (6 tools)
- `analyze_complexity` - Cyclomatic & cognitive complexity per function/file
- `analyze_satd` - Self-admitted technical debt (TODO/FIXME/HACK) detection
- `analyze_dead_code` - Dead / unreachable code detection (cargo-backed for Rust)
- `analyze_dag` - Dependency / call-graph (DAG) generation
- `analyze_deep_context` - Full AST-based deep-context generation for a project
- `analyze_big_o` - Big-O algorithmic-complexity estimation

### Refactoring (4 tools) — stateful session
- `refactor.start` - Begin an interactive refactoring session
- `refactor.nextIteration` - Advance the refactoring state machine one step
- `refactor.getState` - Inspect the current refactoring session state
- `refactor.stop` - End the refactoring session

### Quality (3 tools)
- `quality_gate` - Run the quality gate and return a pass/fail verdict
- `quality_proxy` - Quality-proxy metrics
- `pdmt_deterministic_todos` - Deterministic todo/task generation (reproducible IDs)

### Git & Context (3 tools)
- `git_operation` - Git context operations (history, churn, blame)
- `generate_context` - Generate AI-ready project context (markdown/json/llm-optimized)
- `scaffold_project` - Scaffold a new project / files from templates

### Agent Context (4 tools) — KAIZEN-0165
- `pmat_query_code` - Semantic code search by intent (quality-annotated results)
- `pmat_get_function` - Fetch a function's source with quality metrics
- `pmat_find_similar` - Find functions similar to a given one (refactoring/dedup)
- `pmat_index_stats` - Code-index health and statistics

## Quick Start

### 1. Start the Server (stdio)

```bash
pmat agent mcp-server
```

The server speaks JSON-RPC 2.0 over **stdio** and shuts down cleanly on stdin EOF.
MCP clients launch it as a child process rather than connecting to a network port.

### 2. Register with an MCP client

For **Claude Code**, add it as an MCP server (stdio command `pmat agent mcp-server`).
For other clients, configure the same command. On `initialize` the server advertises
the 20 tools above with full schemas via `tools/list`.

### 3. Call a tool

```jsonc
// tools/call request
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "analyze_complexity",
    "arguments": { "path": "src/", "top_files": 5 }
  }
}
```

```jsonc
// Semantic code search
{ "name": "pmat_query_code", "arguments": { "query": "error handling", "limit": 5 } }
```

## Common Use Cases

### Agent-driven code intelligence
An agent calls `pmat_query_code` to find functions by intent, `pmat_get_function`
to pull the full source + metrics, and `analyze_complexity` / `analyze_dead_code`
to triage quality — all without leaving the MCP session.

### CI quality gating
Call `quality_gate` from an agent or script and branch on the pass/fail verdict.

### Refactoring loop
Drive `refactor.start` → `refactor.nextIteration` (repeat) → `refactor.getState`
→ `refactor.stop` as a stateful, resumable refactoring session.

## Protocol Compliance

- **Version**: MCP v2024-11-05
- **Transport**: stdio (JSON-RPC 2.0)
- **Capabilities**: Tools (the server is `tools_only`)
- **Concurrency**: validated for multi-agent use (per-tool calls + 8-way concurrent
  sessions + framing purity), v3.18.2

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

Error codes:
- `-32700`: Parse error
- `-32600`: Invalid request
- `-32601`: Method not found
- `-32602`: Invalid parameters
- `-32603`: Internal error

## Implementation

The unified server lives in `src/mcp_pmcp/simple_unified_server.rs` and is built on
the [`pmcp`](https://crates.io/crates/pmcp) MCP SDK (2.9). The CLI entry point that
launches it is `pmat agent mcp-server` (see `src/cli/mod.rs`).

## Support

- **Issues**: https://github.com/paiml/paiml-mcp-agent-toolkit/issues
- **API docs**: https://docs.rs/pmat
- **Book**: https://paiml.github.io/pmat-book/

---

**Maintained by**: PAIML
**Protocol Version**: MCP v2024-11-05
**Last Updated**: 2026-06-13 (v3.19.2)
