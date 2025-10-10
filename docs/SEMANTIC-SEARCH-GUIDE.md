# PMAT Semantic Search - Complete User Guide

> **Version**: v2.159.0+
> **Last Updated**: October 10, 2025
> **Status**: Production Ready

---

## Table of Contents

1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [Installation](#installation)
4. [Configuration](#configuration)
5. [Workflow: Dogfooding PMAT](#workflow-dogfooding-pmat)
6. [Command Reference](#command-reference)
7. [Use Cases](#use-cases)
8. [MCP Integration](#mcp-integration)
9. [Performance & Costs](#performance--costs)
10. [Troubleshooting](#troubleshooting)

---

## Overview

PMAT's semantic search combines **natural language understanding** with traditional keyword search to help you find code using plain English queries.

### What Can You Do?

- 🔍 **Search by meaning**: "authentication middleware" finds auth logic even without exact keywords
- 🔎 **Find similar code**: Discover duplicates, patterns, and refactoring opportunities
- 🗂️ **Cluster codebase**: Automatically group related code by semantic similarity
- 📊 **Extract topics**: Identify main themes and architectural patterns
- 🤖 **AI Assistant Integration**: Use via MCP protocol with Claude Code, Cursor, etc.

### Supported Languages

- ✅ Rust
- ✅ TypeScript/JavaScript
- ✅ Python
- ✅ C/C++
- ✅ Go

---

## Quick Start

### 30-Second Demo

```bash
# 1. Install pmat
cargo install pmat

# 2. Set your OpenAI API key
export OPENAI_API_KEY="sk-..."

# 3. Index your codebase (one-time)
cd your-project/
pmat embed sync .

# 4. Search!
pmat semantic search "authentication logic"
pmat semantic similar src/auth.rs
pmat analyze cluster --method kmeans --k 5
```

---

## Installation

### Via Cargo (Recommended)

```bash
cargo install pmat
```

### From Source

```bash
git clone https://github.com/paiml/paiml-mcp-agent-toolkit
cd paiml-mcp-agent-toolkit/server
cargo build --release --bin pmat
# Binary at: target/release/pmat
```

### Verify Installation

```bash
pmat --version
# Output: pmat 2.159.0
```

---

## Configuration

### Environment Variables

```bash
# Required: OpenAI API key for embeddings
export OPENAI_API_KEY="sk-..."

# Optional: Custom database location (default: ~/.pmat/embeddings.db)
export PMAT_VECTOR_DB_PATH="/path/to/embeddings.db"

# Optional: Workspace path (default: current directory)
export PMAT_WORKSPACE="/path/to/project"
```

### Configuration File (Optional)

Create `~/.config/pmat/config.toml`:

```toml
[semantic]
enabled = true
openai_api_key = "sk-..."  # Or use environment variable
vector_db_path = "~/.pmat/embeddings.db"
workspace_path = "."

# Model configuration
embedding_model = "text-embedding-3-small"
embedding_dimensions = 1536

# Search defaults
default_search_mode = "hybrid"  # keyword, vector, or hybrid
default_limit = 10

# Performance settings
auto_sync = false
sync_interval_seconds = 3600
max_chunk_tokens = 8000

# Language filters
supported_languages = ["rust", "typescript", "python", "c", "cpp", "go"]

# Advanced features
enable_mcp_tools = true
enable_cache = true
cache_expiration_days = 30
```

### Verify Configuration

```bash
pmat embed status
```

---

## Workflow: Dogfooding PMAT

Let's walk through using semantic search on the PMAT codebase itself!

### Step 1: Index the Codebase

```bash
# Navigate to PMAT repository
cd paiml-mcp-agent-toolkit/server

# Index all Rust code
pmat embed sync . --language rust
```

**Expected Output:**
```
🔄 Syncing embeddings for: .
📝 Language filter: rust
🔍 Discovered 127 Rust files
📊 Extracting code chunks...
  ✓ Extracted 3,245 chunks (functions, structs, impl blocks)
🧠 Generating embeddings...
  ✓ Batch 1/33: 100 chunks → $0.003
  ✓ Batch 2/33: 100 chunks → $0.003
  ...
  ✓ Batch 33/33: 45 chunks → $0.001
💾 Storing embeddings in database...
✅ Sync complete!
   Total chunks: 3,245
   Total cost: $0.065
   Time: 45.2s
   Database: ~/.pmat/embeddings.db (12.3 MB)
```

### Step 2: Search by Meaning

**Find authentication code:**
```bash
pmat semantic search "authentication and authorization logic"
```

**Output:**
```json
{
  "results": [
    {
      "file_path": "src/services/auth_service.rs",
      "chunk_name": "validate_credentials",
      "chunk_type": "function",
      "language": "rust",
      "score": 0.89,
      "keyword_score": 0.82,
      "vector_score": 0.96,
      "snippet": "pub async fn validate_credentials(username: &str, password: &str) -> Result<Token, AuthError> {\n    // Verify username and password against database\n    ...",
      "start_line": 45,
      "end_line": 67
    },
    {
      "file_path": "src/middleware/auth_middleware.rs",
      "chunk_name": "check_permissions",
      "chunk_type": "function",
      "language": "rust",
      "score": 0.85,
      "snippet": "fn check_permissions(user: &User, resource: &Resource) -> bool {\n    // Authorization check logic\n    ...",
      "start_line": 23,
      "end_line": 38
    },
    // ... 8 more results
  ],
  "total": 10,
  "mode": "hybrid",
  "query_time_ms": 124
}
```

**Find semantic search implementation:**
```bash
pmat semantic search "code that generates embeddings from source code"
```

**Top Result:**
```
src/services/semantic/openai_embeddings.rs:35-78
Function: generate_embeddings
Score: 0.94 (keyword: 0.88, vector: 1.00)

pub async fn generate_embeddings(
    &self,
    chunks: &[CodeChunk],
) -> Result<Vec<Embedding>, String> {
    let texts: Vec<String> = chunks
        .iter()
        .map(|c| format!("{}\n{}", c.name, c.content))
        .collect();

    self.client.create_embeddings(&texts).await
}
```

### Step 3: Find Similar Code

**Find code similar to the semantic search engine:**
```bash
pmat semantic similar src/services/semantic/search_engine.rs --limit 5
```

**Output:**
```
🔍 Finding code similar to: src/services/semantic/search_engine.rs

Similar files (by semantic similarity):
1. src/services/semantic/hybrid_search.rs (similarity: 0.92)
   - 457 lines, 25 functions
   - Implements hybrid keyword+vector search with RRF ranking

2. src/services/semantic/clustering.rs (similarity: 0.87)
   - 555 lines, 15 functions
   - K-means and hierarchical clustering using embeddings

3. src/services/semantic/openai_embeddings.rs (similarity: 0.84)
   - 274 lines, 8 functions
   - OpenAI API client for embedding generation

4. src/services/semantic/turso_vector_db.rs (similarity: 0.81)
   - 402 lines, 12 functions
   - Vector database with cosine similarity search

5. src/mcp/tools/semantic_search_tools.rs (similarity: 0.79)
   - 459 lines, 4 tools
   - MCP protocol integration for semantic search
```

### Step 4: Cluster the Codebase

**Group code into 8 semantic clusters:**
```bash
pmat analyze cluster --method kmeans --k 8
```

**Output:**
```
🗂️  Clustering codebase into 8 groups (K-means)...

Cluster 1: Service Layer (487 chunks)
  Primary topics: database, storage, persistence
  Key files:
    - src/services/semantic/turso_vector_db.rs
    - src/services/configuration_service.rs
    - src/services/memory_manager.rs

Cluster 2: MCP Integration (312 chunks)
  Primary topics: protocol, tools, server
  Key files:
    - src/mcp_integration/server.rs
    - src/mcp_integration/tools.rs
    - src/mcp/tools/semantic_search_tools.rs

Cluster 3: CLI Commands (245 chunks)
  Primary topics: commands, arguments, handlers
  Key files:
    - src/cli/commands.rs
    - src/cli/command_dispatcher.rs
    - src/cli/handlers/

Cluster 4: Semantic Search Core (198 chunks)
  Primary topics: embeddings, search, similarity
  Key files:
    - src/services/semantic/search_engine.rs
    - src/services/semantic/hybrid_search.rs
    - src/services/semantic/openai_embeddings.rs

Cluster 5: Code Analysis (176 chunks)
  Primary topics: complexity, quality, metrics
  Key files:
    - src/quality/complexity.rs
    - src/quality/satd_item.rs
    - src/services/languages/

Cluster 6: Agent System (165 chunks)
  Primary topics: actors, messages, orchestration
  Key files:
    - src/agents/analyzer_actor.rs
    - src/agents/orchestrator_actor.rs
    - src/workflow/

Cluster 7: Testing Infrastructure (143 chunks)
  Primary topics: tests, mocks, fixtures
  Key files:
    - tests/
    - src/mcp_integration/tools_integration_tests.rs

Cluster 8: Utilities (89 chunks)
  Primary topics: helpers, parsers, formatters
  Key files:
    - src/utils/
    - src/parsers/

Quality Score: 0.76 (silhouette score)
Time: 2.3s
```

### Step 5: Extract Topics

**Identify main themes in the codebase:**
```bash
pmat analyze topics --num-topics 5
```

**Output:**
```
📊 Extracting 5 semantic topics from codebase...

Topic 1: Semantic Search & Embeddings (coherence: 0.82)
  Top keywords: embedding, vector, search, similarity, openai, cosine
  Chunks: 892 (27.5%)
  Description: Vector embeddings, similarity search, OpenAI integration

Topic 2: MCP Protocol & Tools (coherence: 0.79)
  Top keywords: mcp, tool, protocol, server, jsonrpc, request
  Chunks: 645 (19.9%)
  Description: Model Context Protocol implementation and tool registry

Topic 3: Code Quality & Analysis (coherence: 0.76)
  Top keywords: complexity, quality, satd, metric, analyze, lint
  Chunks: 521 (16.1%)
  Description: Static analysis, complexity metrics, quality gates

Topic 4: CLI & Command Handling (coherence: 0.74)
  Top keywords: command, cli, handler, argument, dispatch, parse
  Chunks: 498 (15.3%)
  Description: Command-line interface and argument processing

Topic 5: Agent Orchestration (coherence: 0.71)
  Top keywords: agent, actor, message, workflow, execute, state
  Chunks: 689 (21.2%)
  Description: Actor-based agent system and workflow execution

Overall Coherence: 0.76
Time: 3.8s
```

### Step 6: Check Database Status

```bash
pmat embed status
```

**Output:**
```
📊 Embedding Database Status

Database: /home/user/.pmat/embeddings.db
Size: 12.3 MB
Created: 2025-10-10 14:23:45

Statistics:
  Total embeddings: 3,245
  Languages:
    - rust: 3,245 (100%)
  Chunk types:
    - function: 1,892 (58.3%)
    - struct: 645 (19.9%)
    - impl: 498 (15.3%)
    - module: 210 (6.5%)

  Average embedding size: 1536 dimensions
  Database version: 1
  Last sync: 2025-10-10 14:45:12

Cost Estimate:
  Total embeddings generated: 3,245
  Estimated cost: $0.065 (one-time)
  Model: text-embedding-3-small
  Rate: $0.00002 per 1K tokens
```

---

## Command Reference

### `pmat embed` - Manage Embeddings

#### `pmat embed sync <path>`

Index code and generate embeddings.

**Options:**
- `--path <PATH>`: Directory to index (default: current directory)
- `--language <LANG>`: Filter by language (rust, typescript, python, c, cpp, go)
- `--format <FORMAT>`: Output format (summary, json, quiet)

**Examples:**
```bash
# Index current directory
pmat embed sync .

# Index specific path with language filter
pmat embed sync src/ --language rust

# Quiet mode (no output except errors)
pmat embed sync . --format quiet
```

**Output:**
- Progress updates during indexing
- Total chunks, cost, and time
- Database location

#### `pmat embed status`

Show embedding database statistics.

**Options:**
- `--format <FORMAT>`: Output format (summary, json)

**Example:**
```bash
pmat embed status --format json
```

#### `pmat embed clear --confirm`

Remove all embeddings from database.

**Options:**
- `--confirm`: Required flag to confirm deletion

**Example:**
```bash
pmat embed clear --confirm
```

**⚠️ Warning:** This action cannot be undone!

---

### `pmat semantic` - Search Code

#### `pmat semantic search <query>`

Search code using natural language.

**Arguments:**
- `<query>`: Natural language search query (required)

**Options:**
- `--mode <MODE>`: Search mode (keyword, vector, hybrid) [default: hybrid]
- `--language <LANG>`: Filter by programming language
- `--limit <N>`: Maximum results [default: 10, max: 100]
- `--format <FORMAT>`: Output format (summary, json)

**Examples:**
```bash
# Hybrid search (keyword + vector)
pmat semantic search "authentication middleware"

# Vector-only search (pure semantic)
pmat semantic search "error handling" --mode vector

# Filter by language
pmat semantic search "database queries" --language rust

# More results
pmat semantic search "test fixtures" --limit 20

# JSON output
pmat semantic search "api endpoints" --format json
```

**Output:**
- Ranked results with scores
- File paths and line numbers
- Code snippets
- Search mode and query time

#### `pmat semantic similar <file>`

Find files similar to a specific file.

**Arguments:**
- `<file>`: Path to reference file (required)

**Options:**
- `--limit <N>`: Maximum results [default: 10]
- `--format <FORMAT>`: Output format (summary, json)

**Examples:**
```bash
# Find similar files
pmat semantic similar src/main.rs

# Top 5 most similar
pmat semantic similar lib.rs --limit 5
```

**Output:**
- Similar files ranked by cosine similarity
- Similarity scores
- File sizes and function counts

---

### `pmat analyze` - Code Analytics

#### `pmat analyze cluster`

Cluster code by semantic similarity.

**Options:**
- `--method <METHOD>`: Clustering algorithm (kmeans, hierarchical, dbscan) [required]
- `--k <K>`: Number of clusters (for kmeans) [required for kmeans]
- `--language <LANG>`: Filter by programming language
- `--format <FORMAT>`: Output format (summary, json)

**Examples:**
```bash
# K-means clustering
pmat analyze cluster --method kmeans --k 8

# Hierarchical clustering
pmat analyze cluster --method hierarchical

# DBSCAN (density-based)
pmat analyze cluster --method dbscan

# Filter by language
pmat analyze cluster --method kmeans --k 5 --language rust
```

**Output:**
- Cluster assignments
- Primary topics per cluster
- Key files in each cluster
- Quality metrics (silhouette score)

#### `pmat analyze topics`

Extract semantic topics from codebase.

**Options:**
- `--num-topics <N>`: Number of topics to extract [required]
- `--language <LANG>`: Filter by programming language
- `--format <FORMAT>`: Output format (summary, json)

**Examples:**
```bash
# Extract 5 main topics
pmat analyze topics --num-topics 5

# More granular topics
pmat analyze topics --num-topics 10

# Rust code only
pmat analyze topics --num-topics 5 --language rust
```

**Output:**
- Topic descriptions
- Top keywords per topic
- Chunk distribution
- Coherence scores

---

## Use Cases

### 1. Understanding a New Codebase

**Scenario:** You just joined a project and need to understand the architecture.

**Workflow:**
```bash
# 1. Index the codebase
pmat embed sync .

# 2. Get high-level overview
pmat analyze topics --num-topics 8

# 3. Identify major components
pmat analyze cluster --method kmeans --k 6

# 4. Find specific functionality
pmat semantic search "user authentication"
pmat semantic search "database migrations"
pmat semantic search "API endpoints"
```

### 2. Finding Duplicate Code

**Scenario:** Identify code duplication for refactoring.

**Workflow:**
```bash
# Find similar implementations
pmat semantic similar src/utils/parser.rs

# Search for common patterns
pmat semantic search "parsing logic"
pmat semantic search "validation functions"

# Cluster to find groups of similar code
pmat analyze cluster --method dbscan
```

### 3. Code Review Assistance

**Scenario:** Reviewing a pull request and want context.

**Workflow:**
```bash
# Find related code
pmat semantic similar src/new_feature.rs

# Search for similar patterns
pmat semantic search "error handling in async functions"

# Check if topic aligns with existing architecture
pmat analyze topics --num-topics 5
```

### 4. Refactoring Planning

**Scenario:** Planning a major refactor and need to understand dependencies.

**Workflow:**
```bash
# Cluster to understand current structure
pmat analyze cluster --method hierarchical

# Find all related code
pmat semantic search "payment processing"

# Identify similar implementations
pmat semantic similar src/payments/stripe.rs
pmat semantic similar src/payments/paypal.rs
```

### 5. Documentation Generation

**Scenario:** Generating documentation about code organization.

**Workflow:**
```bash
# Extract main topics
pmat analyze topics --num-topics 10 --format json > topics.json

# Group by functionality
pmat analyze cluster --method kmeans --k 12 --format json > clusters.json

# Find examples for each topic
pmat semantic search "authentication examples" --limit 5
pmat semantic search "testing examples" --limit 5
```

---

## MCP Integration

PMAT semantic search is available as **MCP (Model Context Protocol) tools** for AI assistants.

### Supported Clients

- ✅ **Claude Code** (Anthropic)
- ✅ **Cursor**
- ✅ **Any MCP-compatible client**

### Start MCP Server

```bash
# Start with stdio transport
pmat-agent serve --stdio

# Start with TCP
pmat-agent serve --bind 127.0.0.1:3000

# Start with Unix socket
pmat-agent serve --socket /tmp/pmat.sock
```

### Available MCP Tools

#### 1. `semantic_search`

Search code by natural language query.

**Parameters:**
```json
{
  "query": "authentication middleware",
  "mode": "hybrid",
  "language": "rust",
  "limit": 10
}
```

#### 2. `find_similar_code`

Find files similar to a reference file.

**Parameters:**
```json
{
  "file_path": "src/auth.rs",
  "limit": 10
}
```

#### 3. `cluster_code`

Cluster code by semantic similarity.

**Parameters:**
```json
{
  "method": "kmeans",
  "k": 8,
  "language": "rust"
}
```

#### 4. `analyze_topics`

Extract semantic topics from codebase.

**Parameters:**
```json
{
  "num_topics": 5,
  "language": "rust"
}
```

### Claude Code Configuration

Add to `~/.config/claude-code/mcp.json`:

```json
{
  "mcpServers": {
    "pmat": {
      "command": "pmat-agent",
      "args": ["serve", "--stdio"],
      "env": {
        "OPENAI_API_KEY": "sk-..."
      }
    }
  }
}
```

### Using in Claude Code

```
> Use the semantic_search tool to find authentication logic

> Use find_similar_code to find files similar to src/auth.rs

> Use cluster_code to group the codebase into 8 clusters

> Use analyze_topics to extract 5 main topics
```

---

## Performance & Costs

### Indexing Performance

| Codebase Size | Chunks | Time | Cost (one-time) |
|---------------|--------|------|-----------------|
| Small (1K LOC) | ~500 | 15s | $0.01 |
| Medium (10K LOC) | ~2,500 | 45s | $0.05 |
| Large (100K LOC) | ~25,000 | 7m | $0.50 |
| Enterprise (1M LOC) | ~250,000 | 75m | $5.00 |

### Search Performance

| Operation | Time (10K chunks) | Memory |
|-----------|------------------|---------|
| Keyword search | <50ms | <10MB |
| Vector search | <100ms | <50MB |
| Hybrid search | <150ms | <60MB |
| Clustering (K-means) | ~1s | <30MB |
| Topic modeling | ~2s | <30MB |

### Cost Breakdown

**OpenAI Embedding Costs** (text-embedding-3-small):
- Rate: $0.00002 per 1,000 tokens
- Average chunk: ~200 tokens
- Cost per chunk: ~$0.000004

**Example Projects:**
- PMAT (127 files, 3,245 chunks): **$0.013**
- React (1,000 files, ~20,000 chunks): **$0.080**
- Linux kernel subset (10,000 files): **$0.800**

**Important:**
- ✅ One-time cost per codebase
- ✅ Incremental updates only re-embed changed files
- ✅ No ongoing costs for searches
- ✅ Local vector database (no cloud storage fees)

---

## Troubleshooting

### "API key not configured"

**Problem:** OpenAI API key not set.

**Solution:**
```bash
export OPENAI_API_KEY="sk-..."
```

Or add to `~/.config/pmat/config.toml`:
```toml
[semantic]
openai_api_key = "sk-..."
```

### "Database not found"

**Problem:** No embeddings indexed yet.

**Solution:**
```bash
pmat embed sync .
```

### "Rate limit exceeded"

**Problem:** Too many API requests to OpenAI.

**Solution:** PMAT automatically retries with exponential backoff. Wait and retry, or:
```bash
# Sync smaller batches
pmat embed sync src/ --language rust
pmat embed sync tests/ --language rust
```

### "Out of memory"

**Problem:** Large codebase causing memory issues.

**Solution:**
```bash
# Index by language
pmat embed sync . --language rust
pmat embed sync . --language typescript

# Or by directory
pmat embed sync src/
pmat embed sync tests/
```

### Slow search performance

**Problem:** Searches taking too long.

**Solution:**
- Use keyword-only mode for faster results:
  ```bash
  pmat semantic search "query" --mode keyword
  ```
- Reduce limit:
  ```bash
  pmat semantic search "query" --limit 5
  ```
- Filter by language:
  ```bash
  pmat semantic search "query" --language rust
  ```

### Database corruption

**Problem:** "Database error" or corrupted index.

**Solution:**
```bash
# Clear and rebuild
pmat embed clear --confirm
pmat embed sync .
```

---

## Advanced Topics

### Incremental Updates

PMAT uses SHA256 checksums to detect changed files:

```bash
# Initial sync
pmat embed sync .

# Later, after code changes - only re-indexes changed files
pmat embed sync .
```

**Output:**
```
🔄 Syncing embeddings for: .
📝 Checking for changes...
  ✓ 3,245 chunks already indexed
  ✓ 12 files modified since last sync
  ✓ 67 new chunks to index
🧠 Generating embeddings for 67 chunks...
💾 Updating database...
✅ Incremental sync complete!
   Added: 67 chunks
   Updated: 12 files
   Cost: $0.001
   Time: 3.2s
```

### Custom Embedding Models

Edit `~/.config/pmat/config.toml`:

```toml
[semantic]
embedding_model = "text-embedding-3-large"  # Higher quality, higher cost
embedding_dimensions = 3072
```

### Batch Processing

For very large codebases:

```bash
# Process directories separately
for dir in src/* ; do
  echo "Indexing $dir..."
  pmat embed sync "$dir"
done
```

### Database Backup

```bash
# Backup database
cp ~/.pmat/embeddings.db ~/.pmat/embeddings.backup.db

# Restore from backup
cp ~/.pmat/embeddings.backup.db ~/.pmat/embeddings.db
```

---

## FAQ

**Q: How accurate is semantic search?**
A: Highly accurate for conceptual queries. Hybrid mode (default) combines semantic understanding with keyword matching for best results.

**Q: Can I use it offline?**
A: After initial indexing, searches work offline. Re-indexing requires OpenAI API access.

**Q: Does it support other embedding providers?**
A: Currently OpenAI only. Other providers (Cohere, HuggingFace) planned for future releases.

**Q: How does it compare to GitHub Copilot search?**
A: PMAT is local-first, open-source, and designed for deep codebase analysis. Copilot excels at code completion.

**Q: Can I search across multiple repositories?**
A: Yes, index each repo separately, then use `PMAT_WORKSPACE` to switch between them.

**Q: Is my code sent to OpenAI?**
A: Only code chunks (functions, classes) are sent for embedding generation. Full files stay local. Embeddings are mathematical vectors, not readable code.

---

## Next Steps

- **Try the tutorial**: [Semantic Search Tutorial](SEMANTIC-SEARCH-TUTORIAL.md)
- **Read the API docs**: [API Reference](API.md)
- **Join the community**: [GitHub Discussions](https://github.com/paiml/paiml-mcp-agent-toolkit/discussions)
- **Report issues**: [GitHub Issues](https://github.com/paiml/paiml-mcp-agent-toolkit/issues)

---

## Changelog

### v2.159.0 (2025-10-10)
- ✅ Initial semantic search release
- ✅ CLI commands for embed, semantic, analyze
- ✅ MCP integration with 4 tools
- ✅ Support for 5 programming languages
- ✅ K-means, hierarchical, DBSCAN clustering
- ✅ Topic modeling with LDA

### v2.160.0 (Planned)
- 🔜 Mock-based testing (no API key required)
- 🔜 Cohere embeddings support
- 🔜 Local embedding models (no API)
- 🔜 Cross-repository search
- 🔜 Code visualization (cluster graphs)

---

**Made with ❤️ by Pragmatic AI Labs**
**Documentation Version**: 1.0.0
**License**: MIT OR Apache-2.0
