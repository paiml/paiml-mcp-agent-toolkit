# KAIZEN-0006: Graph-native navigation tool for agents (CSR + structural edges)

**Source paper:** arxiv:2602.20048 — "The Navigation Paradox in Large-Context Agentic Coding: Graph-Structured Dependency Navigation Outperforms Retrieval" (Feb 2026) + arxiv:2603.27277 — "Codebase-Memory: Tree-Sitter-Based Knowledge Graphs for LLM Code Exploration via MCP"
**Category:** sub-agent-behavior
**Priority:** high
**Effort:** M

## Problem
The "Navigation Paradox" paper empirically shows graph-navigation (IMPORTS / INHERITS / INSTANTIATES edges) beats dense-retrieval semantic search on architecture-heavy tasks. Codebase-Memory reports 10x fewer tokens and 2.1x fewer tool calls vs file-exploration agents. pmat already has a trueno-graph CSR graph backing `pmat query` but does **not expose structural-edge queries** to agents — they must fall back to semantic search for every navigation step.

## Proposed improvement
Expose three new MCP tools:
1. `pmat_graph_neighbors {node, edge_kind, direction, depth}` — O(1) lookup of IMPORTS / CALLS / INHERITS / DEFINES edges.
2. `pmat_graph_path {from, to, max_depth}` — shortest reachability path.
3. `pmat_graph_subgraph {seed, radius, edge_filter}` — return a focused subgraph as compact DOT/JSON.

These already exist internally via `trueno-graph`; this ticket surfaces them through MCP with a compact JSON schema.

## Impact
- Paper: 10x fewer tokens, 2.1x fewer tool calls on architecture tasks.
- pmat is uniquely positioned — we already build the graph every index cycle.
- Turns a latent capability into a competitive differentiator for agent workflows.

## Implementation sketch
1. Add `server/src/mcp_pmcp/tools/graph.rs` with three handlers.
2. Use existing `context_graph.rs` and `tdg_graph.rs` as backing stores.
3. Return compact shape: arrays of `{id, name, file:line}` tuples; no full function bodies.
4. Document the "when to use graph vs semantic search" guidance in book ch 14.

## Acceptance criteria
- Three new MCP tools callable from Claude Code.
- Benchmark: traversing a 50-node CALLS subgraph <50 ms.
- Book chapter updated with decision tree: graph navigation vs semantic search.
- **3 new MCP tools added — additive schema change.**
