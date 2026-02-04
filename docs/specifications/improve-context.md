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

## Implementation Plan

### Phase 1: Core Infrastructure (2-3 days)

**Files to create:**
- `server/src/services/agent_context/mod.rs` - Module root
- `server/src/services/agent_context/index.rs` - RAG index builder
- `server/src/services/agent_context/query.rs` - Query engine
- `server/src/services/agent_context/function_chunker.rs` - AST-aware chunker

**Files to modify:**
- `server/src/cli/mod.rs` - Add `query` subcommand
- `server/src/cli/handlers/mod.rs` - Add query handler
- `server/src/services/mod.rs` - Export agent_context module

### Phase 2: CLI Commands (1-2 days)

**Implement:**
- `pmat query <query>` - Semantic search
- `pmat context --index` - Build index
- `pmat context --status` - Check index

**Files:**
- `server/src/cli/handlers/query_handler.rs` - Query command
- `server/src/cli/handlers/context_handler.rs` - Extend existing

### Phase 3: MCP Integration (2 days)

**Implement:**
- `pmat_query_code` tool
- `pmat_get_function` tool
- `pmat_find_similar` tool
- `pmat context --serve` mode

**Files:**
- `server/src/mcp_server/tools/query_code.rs`
- `server/src/mcp_server/tools/get_function.rs`
- `server/src/mcp_server/tools/find_similar.rs`

### Phase 4: Comply Integration (1 day)

**Implement:**
- CB-130 check
- `pmat comply setup agent-context` command
- CLAUDE.md auto-update

**Files:**
- `server/src/cli/handlers/comply_cb_detect.rs` - Add CB-130
- `server/src/cli/handlers/comply_setup.rs` - Setup command

### Phase 5: Testing & Documentation (1 day)

**Tests:**
- Unit tests for index builder
- Unit tests for query engine
- Integration tests for CLI
- MCP tool tests

**Documentation:**
- Update README.md
- Update CLAUDE.md
- Add to pmat-book Chapter 15

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

## References

- [trueno-rag documentation](https://docs.rs/trueno-rag)
- [TDG persistence](../server/src/tdg/storage_impl.rs)
- [Existing semantic infrastructure](../server/src/services/semantic/)
- [MCP specification](https://modelcontextprotocol.io/)
