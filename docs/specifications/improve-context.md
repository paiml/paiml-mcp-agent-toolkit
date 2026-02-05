# Specification: RAG-Powered Agent Context

**Status**: Approved
**Version**: 3.0.0
**Created**: 2025-02-04
**Updated**: 2025-02-04
**Author**: PMAT Team
**Work Item**: `rag-agent-context`

## Executive Summary

Agents like Claude Code **NEVER** use `pmat context`. They grep/glob instead, wasting tokens and missing quality signals. This spec defines a RAG-powered context system that:

1. **Indexes functions** with quality metadata (TDG, complexity, SATD)
2. **Enables semantic search** via `pmat query` command
3. **Exposes MCP tools** for agent integration
4. **Enforces adoption** via `pmat comply` checks

## The Problem

### Current Reality

```
Agent: "Find error handling code"
  ↓
grep -r "error" src/ | head -50
  ↓
[500 irrelevant matches, no context, no quality info]
```

### What We Built (Unused)

`pmat context` generates rich AST with:
- Function signatures with complexity scores
- TDG grades per file
- SATD markers
- Big-O estimates
- Provability scores

**But agents never use it.** They don't know it exists.

## TDG Persistence vs RAG Context

These are **complementary**, not duplicative:

| Aspect | TDG Persistence | RAG Context |
|--------|-----------------|-------------|
| **Granularity** | File-level | Function-level |
| **Data stored** | Scores, grades, metrics | Embeddings + metadata |
| **Access pattern** | Hash/path/commit lookup | Semantic/keyword query |
| **Query type** | "Get TDG for file X" | "Find code that does X" |
| **Purpose** | Quality tracking over time | Intelligent code retrieval |
| **Indexing** | Content hash → FullTdgRecord | Embedding vector → Chunk |

### What TDG Persistence Stores

```
FullTdgRecord (per file):
├── FileIdentity (path, content_hash, size)
├── TdgScore (complexity, duplication, coupling, grade)
├── ComponentScores (breakdown by function)
├── SemanticSignature (ast_structure_hash, patterns)
├── AnalysisMetadata (version, timestamp)
└── GitContext (commit, author, message)
```

### What RAG Context Stores

```
FunctionChunk (per function):
├── Embedding vector (384-dim semantic)
├── Content (signature, docs, body)
├── TDG metadata (score, grade, complexity)  ← FROM TDG
├── File location (path, line)
└── Semantic neighbors (similar functions)
```

### Integration

RAG context **uses TDG data** as ranking signals:

```rust
// Results sorted by: relevance × quality_factor
// Higher TDG grade = better quality = ranked higher
annotated.sort_by(|a, b| {
    let score_a = a.relevance * quality_factor(a.tdg_grade);
    let score_b = b.relevance * quality_factor(b.tdg_grade);
    score_b.total_cmp(&score_a)
});
```

## Solution Architecture

```
┌──────────────────────────────────────────────────────────────────────────┐
│                         pmat context --serve                              │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─────────────┐    ┌──────────────┐    ┌─────────────────────────────┐ │
│  │ AST Parser  │───▶│ Annotator    │───▶│ trueno-rag Index            │ │
│  │ (tree-sitter│    │ - TDG scores │    │ - VectorStore (embeddings)  │ │
│  │  + custom)  │    │ - Complexity │    │ - BM25Index (keywords)      │ │
│  │             │    │ - SATD       │    │ - FunctionChunker           │ │
│  │             │    │ - Big-O      │    │                             │ │
│  └─────────────┘    └──────────────┘    └─────────────────────────────┘ │
│                                                    │                     │
│                                                    ▼                     │
│                                          ┌─────────────────┐            │
│                                          │ Query Engine    │            │
│                                          │ - pmat query    │            │
│                                          │ - MCP tools     │            │
│                                          │ - quality filter│            │
│                                          └─────────────────┘            │
│                                                    │                     │
└────────────────────────────────────────────────────│─────────────────────┘
                                                     │
                                                     ▼
                                          ┌─────────────────┐
                                          │ Claude Code     │
                                          │ Cline           │
                                          │ Other Agents    │
                                          └─────────────────┘
```

## CLI Commands

### `pmat query` - Semantic Code Search

```bash
# Basic semantic search
pmat query "error handling in API layer"

# With quality filters
pmat query "authentication logic" --min-grade B --max-complexity 15 --limit 10

# JSON output for scripting
pmat query "database connection" --format json

# Search in specific path
pmat query "validation" --path src/api/
```

**Output:**
```
Found 5 functions matching "error handling in API layer":

1. src/api/error.rs:42 - handle_api_error
   Signature: pub fn handle_api_error(err: ApiError) -> Response
   TDG: A (2.1) | Complexity: 8 | Big-O: O(1)
   Doc: Converts API errors to HTTP responses
   Relevance: 0.92

2. src/api/middleware.rs:128 - error_middleware
   Signature: pub async fn error_middleware(req: Request, next: Next) -> Response
   TDG: B (3.4) | Complexity: 12 | Big-O: O(1)
   Doc: Catches and formats errors in request pipeline
   Relevance: 0.87

...
```

### `pmat context --index` - Build RAG Index

```bash
# Build index for current project
pmat context --index

# Build with specific output
pmat context --index --output .pmat/context.idx

# Rebuild (clear cache)
pmat context --index --rebuild

# Check index status
pmat context --status
```

**Output:**
```
Building RAG index for /home/noah/src/project...
  Parsing AST: 1,234 files
  Extracting functions: 8,456 functions
  Computing TDG scores: 8,456 functions
  Generating embeddings: 8,456 chunks
  Building BM25 index: 8,456 documents

Index built in 12.3s
  Storage: .pmat/context.idx (24.5 MB)
  Functions: 8,456
  Average TDG: B (3.2)
  Languages: Rust, TypeScript, Python
```

### `pmat context --serve` - MCP Server Mode

```bash
# Start MCP server with context index
pmat context --serve

# With custom port
pmat context --serve --port 3000

# Background mode
pmat context --serve --daemon
```

## MCP Tools

### `pmat_query_code`

```json
{
  "name": "pmat_query_code",
  "description": "Semantic search for code by intent, returns annotated functions",
  "inputSchema": {
    "type": "object",
    "properties": {
      "query": {
        "type": "string",
        "description": "Natural language description of what you're looking for"
      },
      "limit": {
        "type": "integer",
        "default": 5,
        "description": "Maximum number of results"
      },
      "min_grade": {
        "type": "string",
        "enum": ["A", "B", "C", "D", "F"],
        "description": "Minimum TDG grade filter"
      },
      "max_complexity": {
        "type": "integer",
        "description": "Maximum cyclomatic complexity filter"
      },
      "path": {
        "type": "string",
        "description": "Restrict search to this path"
      }
    },
    "required": ["query"]
  }
}
```

### `pmat_get_function`

```json
{
  "name": "pmat_get_function",
  "description": "Get full function source with quality metrics",
  "inputSchema": {
    "type": "object",
    "properties": {
      "file": {
        "type": "string",
        "description": "File path"
      },
      "function": {
        "type": "string",
        "description": "Function name"
      },
      "include_callers": {
        "type": "boolean",
        "default": false,
        "description": "Include functions that call this one"
      },
      "include_callees": {
        "type": "boolean",
        "default": false,
        "description": "Include functions this one calls"
      }
    },
    "required": ["file", "function"]
  }
}
```

### `pmat_find_similar`

```json
{
  "name": "pmat_find_similar",
  "description": "Find functions similar to a given one (for refactoring, deduplication)",
  "inputSchema": {
    "type": "object",
    "properties": {
      "file": {
        "type": "string",
        "description": "File path of reference function"
      },
      "function": {
        "type": "string",
        "description": "Reference function name"
      },
      "limit": {
        "type": "integer",
        "default": 5
      },
      "min_similarity": {
        "type": "number",
        "default": 0.7,
        "description": "Minimum similarity threshold (0-1)"
      }
    },
    "required": ["file", "function"]
  }
}
```

## `pmat comply` Integration

### New Compliance Check: CB-130 (Agent Context Adoption)

```bash
$ pmat comply check
...
CB-130 Agent Context Adoption
  ✓ RAG index exists: .pmat/context.idx
  ✓ Index is fresh (updated 2 hours ago)
  ✗ MCP tools not configured in claude_desktop_config.json
  ⚠ No pmat_query_code calls detected in last session

  Recommendation: Add PMAT MCP server to your agent configuration
```

### Configuration: `.pmat-gates.toml`

```toml
[comply.agent-context]
# CB-130: Agent context adoption
enabled = true
severity = "warning"  # warning | error

# Require RAG index to exist
require_index = true

# Maximum index age before warning
max_index_age_hours = 24

# Require MCP tools to be configured
require_mcp_config = true

# Track grep vs query ratio (future: analyze agent logs)
# warn_on_grep_ratio = 0.5  # Warn if >50% searches are grep
```

### Setup Command

```bash
# Configure MCP for Claude Code
pmat comply setup agent-context

# Output:
Setting up agent context integration...

1. Building RAG index...
   ✓ Index created: .pmat/context.idx (24.5 MB)

2. Configuring MCP server...
   ✓ Added to ~/.config/claude-code/mcp_servers.json:
   {
     "pmat-context": {
       "command": "pmat",
       "args": ["context", "--serve"],
       "cwd": "/home/noah/src/project"
     }
   }

3. Updating CLAUDE.md...
   ✓ Added agent context instructions

Setup complete! Restart Claude Code to use semantic search.

Usage:
  - In Claude Code: Use pmat_query_code tool instead of grep
  - CLI: pmat query "your search"
  - Status: pmat context --status
```

### CLAUDE.md Addition

```markdown
## Agent Context (RAG-Powered Search)

**PREFER `pmat_query_code` over grep/glob for code search.**

This project has a RAG-indexed context with quality annotations.

### Available MCP Tools

| Tool | Use Case |
|------|----------|
| `pmat_query_code` | Find code by intent ("error handling", "auth logic") |
| `pmat_get_function` | Get full function with metrics |
| `pmat_find_similar` | Find similar functions (refactoring) |

### Example

Instead of:
```bash
grep -r "error" src/ | head -50  # ❌ No quality info, too many results
```

Use:
```
pmat_query_code(query="error handling", min_grade="B", limit=5)  # ✓ Quality-filtered
```

### Why?

- **Semantic search**: Understands intent, not just keywords
- **Quality-aware**: Results ranked by TDG score, complexity
- **Efficient**: Pre-indexed, O(1) lookup vs O(n) grep
- **Context-rich**: Full signatures, docs, metrics included
```

## Implementation Status

### Phase 1: Core Infrastructure - COMPLETE

**Files created:**
- `src/services/agent_context/mod.rs` - Module root
- `src/services/agent_context/function_index.rs` - RAG index builder (AgentContextIndex)
- `src/services/agent_context/query.rs` - Query engine with term-based scoring

**Files modified:**
- `src/services/mod.rs` - Export agent_context module
- `src/cli/enums.rs` - QueryOutputFormat enum

**Tests:** 29 unit tests (function_index: 6, query: 6, integration: 17)

### Phase 2: CLI Commands - COMPLETE

**Implemented:**
- `pmat query <query>` - Semantic search with quality filtering
- Text, JSON, markdown output formats
- --min-grade, --max-complexity, --language, --path filters
- Index auto-build and persistence to .pmat/context.idx

**Files created:**
- `src/cli/handlers/query_handler.rs` - Query command handler

**Files modified:**
- `src/cli/commands/mod.rs` - Query command definition
- `src/cli/command_dispatcher/mod.rs` - Command routing
- `src/cli/command_structure.rs` - Command structure
- `src/cli/handlers/mod.rs` - Handler exports

**Tests:** 2 integration tests (empty project, with functions)

### Phase 3: MCP Integration - COMPLETE

**Implemented:**
- `pmat_query_code` tool - Semantic search via MCP
- `pmat_get_function` tool - Function lookup by ID
- `pmat_find_similar` tool - Similar function discovery
- `pmat_index_stats` tool - Index health and statistics
- IndexManager for shared index lifecycle with async caching
- MCP integration adapters (QueryCodeToolAdapter, etc.)

**Files created:**
- `src/mcp/tools/agent_context_tools.rs` - MCP tool implementations

**Files modified:**
- `src/mcp/tools/mod.rs` - Module exports
- `src/mcp/mod.rs` - Re-exports
- `src/mcp_integration/tools.rs` - Adapter tools

**Tests:** 10 unit tests (schema validation, ID parsing, index manager)

### Phase 4: Comply Integration - COMPLETE

**Implemented:**
- CB-130 compliance check for agent context adoption
- detect_cb130_agent_context_adoption detector function
- check_agent_context_adoption aggregator in handle_check
- Validates: index existence, freshness, function count, CLAUDE.md config
- Configurable via .pmat.yaml (cb-130 key)

**Files modified:**
- `src/cli/handlers/comply_cb_detect.rs` - CB-130 detector + AgentContextReport
- `src/cli/handlers/comply_handlers/check_handlers.rs` - Check registration

**Tests:** 5 unit tests (no index, with/without CLAUDE.md, index file)

### Phase 5: Testing & Documentation - COMPLETE

**Total tests:** 46 passing tests across all phases
- 29 core infrastructure tests
- 2 CLI integration tests
- 10 MCP tool tests
- 5 CB-130 compliance tests

## Success Criteria

1. **`pmat query` works**: Returns relevant, quality-ranked results
2. **Index builds quickly**: <30s for medium projects (5K functions)
3. **MCP tools accessible**: Claude Code can use semantic search
4. **Comply check passes**: CB-130 validates setup
5. **Measurable improvement**: Agents use query >50% vs grep

## Technical Notes

### Using Existing Infrastructure

trueno-rag is already integrated:
- `src/services/semantic/turso_vector_db.rs` - VectorStore
- `src/services/semantic/chunker.rs` - RecursiveChunker
- `src/services/semantic/hybrid_search.rs` - BM25Index

We just need to:
1. Create function-aware chunker (one chunk per function)
2. Add TDG metadata to chunks
3. Wire up CLI and MCP interfaces

### Index Storage

```
.pmat/context.idx/
├── vectors.bin       # Embedding vectors (mmap)
├── bm25.idx          # BM25 inverted index
├── metadata.json     # Function metadata (TDG, complexity)
├── manifest.json     # Index version, timestamp, stats
└── chunks/           # Raw chunk content (LZ4 compressed)
```

### Embedding Model

Use `all-MiniLM-L6-v2` via trueno-rag's FastEmbedder:
- 384 dimensions
- ~90MB model (downloaded on first use)
- ~1ms per embedding

## Phase 6: Graph Integration - IMPLEMENTED ✅

**Goal**: Integrate call graph metrics (PageRank, centrality) into `pmat query` results.
**Status**: Implemented in v2.216.0 (PMAT-471)
**Index Version**: 1.2.0 (includes graph_metrics)

### The Problem (Solved)

Previously `pmat query` and `pmat analyze graph-metrics` were separate. Users couldn't combine semantic search with PageRank ranking.

### Solution: Unified Query + Graph

#### CLI Options (Implemented)

```bash
# Rank results by PageRank (most important functions first)
pmat query "error handling" --rank-by pagerank

# Rank by degree centrality (most connected functions)
pmat query "validation" --rank-by centrality

# Rank by in-degree (most called functions)
pmat query "mcp" --rank-by indegree

# Filter by minimum PageRank score
pmat query "parser" --min-pagerank 0.0001

# Default: rank by relevance (semantic similarity)
pmat query "error handling"
```

#### Enhanced Output (Implemented)

```
Found 3 functions matching "mcp server":

1. src/contracts/mcp_impl.rs:40 - error
   Signature: /// Create an error result
   TDG: A (0.1) | Complexity: 1 | Big-O: O(1)
   Doc: Create an error result
   Calls: error, result, message, success, data
   Called by: main, categorize_error, initialize_agents, serve_mcp, ...
   Graph: PageRank 0.000426 | In-Degree: 4649 | Out-Degree: 46
   Relevance: 0.40
```

#### RankBy Options

| Option | Description | Use Case |
|--------|-------------|----------|
| `relevance` | Default. Semantic similarity to query | Finding specific functionality |
| `pagerank` | Function importance (called by important callers) | Finding critical code paths |
| `centrality` | Total connections (in + out degree) | Finding hub functions |
| `indegree` | Most called functions | Finding utility functions |

#### JSON Output

```json
{
  "function_name": "error",
  "file_path": "src/contracts/mcp_impl.rs",
  "pagerank": 0.000426,
  "in_degree": 4649,
  "out_degree": 46,
  "relevance_score": 0.40
}
```

#### MCP Tool Parameters (Implemented)

```json
{
  "name": "pmat_query_code",
  "inputSchema": {
    "properties": {
      "query": { "type": "string" },
      "rank_by": {
        "type": "string",
        "enum": ["relevance", "pagerank", "centrality", "indegree"],
        "default": "relevance"
      },
      "min_pagerank": {
        "type": "number",
        "description": "Minimum PageRank score (0-1)"
      },
      "caller_depth": {
        "type": "integer",
        "default": 1,
        "description": "Depth of caller graph to include"
      },
      "callee_depth": {
        "type": "integer",
        "default": 1,
        "description": "Depth of callee graph to include"
      }
    }
  }
}
```

### Implementation Plan

1. **Extend AgentContextIndex** with graph data:
   - Store call graph edges in index
   - Pre-compute PageRank scores at index build time
   - Cache centrality metrics

2. **Add graph ranking to QueryEngine**:
   - `--rank-by pagerank|betweenness|centrality`
   - Hybrid scoring: `final_score = relevance * (1 + pagerank_boost)`

3. **Expand caller/callee traversal**:
   - `--caller-depth N` - traverse callers N levels
   - `--callee-depth N` - traverse callees N levels
   - Include call counts in output

4. **Update MCP tools** with new parameters

### Files to Modify

- `src/services/agent_context/function_index.rs` - Add graph storage
- `src/services/agent_context/query.rs` - Add rank_by logic
- `src/cli/handlers/query_handler.rs` - Add CLI args
- `src/mcp/tools/agent_context_tools.rs` - Add MCP params

### Success Criteria

1. `pmat query "X" --rank-by pagerank` returns PageRank-sorted results
2. Graph metrics visible in query output
3. Call graph depth traversal works (--caller-depth, --callee-depth)
4. MCP tools support new parameters
5. Index build time increases <20% with graph data

## References

- [trueno-rag documentation](https://docs.rs/trueno-rag)
- [TDG persistence](../server/src/tdg/storage_impl.rs)
- [Existing semantic infrastructure](../server/src/services/semantic/)
- [MCP specification](https://modelcontextprotocol.io/)
- [trueno-graph PageRank](https://docs.rs/trueno-graph) - CSR graph with PageRank
