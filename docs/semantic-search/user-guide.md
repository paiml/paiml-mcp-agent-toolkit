# Semantic Search User Guide

> **Get started with AI-powered code discovery in 5 minutes**

## Table of Contents

1. [Installation](#installation)
2. [Quick Start](#quick-start)
3. [Embedding Management](#embedding-management)
4. [Semantic Search](#semantic-search)
5. [Code Analytics](#code-analytics)
6. [MCP Integration](#mcp-integration)
7. [Best Practices](#best-practices)
8. [Troubleshooting](#troubleshooting)

## Installation

### Prerequisites

```bash
# 1. Install PMAT
cargo install pmat

# 2. Set OpenAI API key
export OPENAI_API_KEY="sk-your-api-key-here"

# Optional: Add to ~/.bashrc or ~/.zshrc
echo 'export OPENAI_API_KEY="sk-your-key"' >> ~/.bashrc
```

### Verify Installation

```bash
pmat --version
# Output: pmat 2.158.0
```

## Quick Start

### 1. Sync Your Codebase

```bash
# Navigate to your project
cd my-rust-project

# Sync embeddings for all code
pmat embed sync ./src

# Output:
# 🔍 Analyzing codebase...
# 📦 Found 150 code chunks
# 🧠 Generating embeddings...
# ✅ Synced 150 chunks (150 created, 0 updated)
# 💰 Cost: $0.03
```

### 2. Search by Natural Language

```bash
# Find error handling code
pmat semantic search "error handling patterns"

# Output:
# 📊 Found 12 results for query: error handling patterns
#
# 1. src/error.rs:handle_error [score: 0.89]
#    Error handling function with Result types
#
# 2. src/main.rs:process_request [score: 0.76]
#    HTTP request processing with error propagation
# ...
```

### 3. Find Similar Code

```bash
# Find code similar to a specific file
pmat semantic similar src/auth/login.rs --limit 10

# Output:
# 🔍 Finding 10 similar files to: src/auth/login.rs
#
# 1. src/auth/signup.rs [similarity: 0.92]
# 2. src/auth/logout.rs [similarity: 0.88]
# 3. src/auth/session.rs [similarity: 0.81]
# ...
```

## Embedding Management

### Sync Command

Synchronize embeddings for your codebase:

```bash
# Basic sync
pmat embed sync ./src

# With language filter
pmat embed sync ./src --language rust

# Sync specific directory
pmat embed sync ./src/services --language typescript
```

**What happens**:
1. Scans directory for source files
2. Extracts semantic chunks (functions, classes, modules)
3. Generates embeddings via OpenAI API
4. Stores in local SQLite database
5. Skips unchanged files (SHA256 checksums)

**Incremental Updates**: Only changed files are re-embedded!

### Status Command

Check embedding database status:

```bash
pmat embed status

# Output:
# 📊 Embedding Database Status
# ├─ Total chunks: 1,250
# ├─ Languages:
# │  ├─ Rust: 800 chunks
# │  ├─ TypeScript: 350 chunks
# │  └─ Python: 100 chunks
# ├─ Database size: 25MB
# └─ Last updated: 2025-10-10 14:30:00
```

### Clear Command

Remove all embeddings (requires confirmation):

```bash
# This will fail (safety check)
pmat embed clear

# Use --confirm flag
pmat embed clear --confirm

# Output:
# ⚠️  This will delete ALL embeddings
# ✅ All embeddings cleared
```

## Semantic Search

### Basic Search

```bash
# Natural language query
pmat semantic search "database connection pooling"

# With mode specification
pmat semantic search "async functions" --mode hybrid

# Limit results
pmat semantic search "logging" --limit 5

# Language filter
pmat semantic search "HTTP handlers" --language rust
```

### Search Modes

**1. Hybrid Mode (Default)** - Best results
```bash
pmat semantic search "error handling" --mode hybrid
```
- Combines keyword matching + vector similarity
- Uses RRF algorithm for optimal ranking

**2. Vector Mode** - Semantic only
```bash
pmat semantic search "authentication logic" --mode vector
```
- Pure semantic similarity
- Finds conceptually similar code
- Good for discovering patterns

**3. Keyword Mode** - Fast exact matching
```bash
pmat semantic search "fn handle_error" --mode keyword
```
- Uses ripgrep for exact matching
- Fastest option
- Good for known identifiers

### Find Similar Code

```bash
# Find files similar to a reference file
pmat semantic similar src/main.rs

# Specify result limit
pmat semantic similar src/auth.rs --limit 20

# Output format
# 1. src/server.rs [similarity: 0.95]
#    HTTP server with similar structure
# 2. src/client.rs [similarity: 0.88]
#    Client implementation with shared patterns
```

**Use Cases**:
- Find duplicate code for refactoring
- Identify similar implementations
- Discover related functionality

## Code Analytics

### Clustering

Group code by semantic similarity:

```bash
# K-means clustering (must specify k)
pmat analyze cluster --method kmeans --k 5

# Hierarchical clustering
pmat analyze cluster --method hierarchical

# DBSCAN (density-based)
pmat analyze cluster --method dbscan
```

**Output**:
```
📊 Clustering Results
├─ Method: K-means
├─ Clusters: 5
├─ Total chunks: 1,250
├─ Silhouette score: 0.72 (good separation)
│
├─ Cluster 0: Database Layer (250 chunks)
│  ├─ Top keywords: query, transaction, connection
│  └─ Files: src/db/*.rs
│
├─ Cluster 1: API Handlers (300 chunks)
│  ├─ Top keywords: request, response, handler
│  └─ Files: src/api/*.rs
│
├─ Cluster 2: Business Logic (400 chunks)
│  ├─ Top keywords: process, validate, transform
│  └─ Files: src/services/*.rs
...
```

**Use Cases**:
- Understand codebase structure
- Identify architectural layers
- Find misplaced code

### Topic Modeling

Extract semantic topics from code:

```bash
# Extract 10 topics
pmat analyze topics --num-topics 10

# With language filter
pmat analyze topics --num-topics 5 --language rust
```

**Output**:
```
🧠 Topic Modeling Results
├─ Topics: 10
├─ Total chunks: 1,250
├─ Coherence score: 0.68
│
├─ Topic 0: Error Handling (150 chunks)
│  ├─ Keywords: error, result, handle, try, catch
│  ├─ Top files:
│  │  ├─ src/error.rs
│  │  ├─ src/validation.rs
│  │  └─ src/middleware/error.rs
│  └─ Strength: 0.82
│
├─ Topic 1: Database Operations (200 chunks)
│  ├─ Keywords: query, transaction, insert, select, update
│  ├─ Top files:
│  │  ├─ src/db/query.rs
│  │  ├─ src/db/transaction.rs
│  │  └─ src/models/*.rs
│  └─ Strength: 0.75
...
```

**Use Cases**:
- Discover code themes
- Identify cross-cutting concerns
- Plan refactoring efforts

## MCP Integration

Use semantic search in AI assistants like Claude Code:

### Available Tools

#### 1. semantic_search
```typescript
semantic_search(
    query: "error handling patterns",
    mode: "hybrid",
    language: "rust",
    limit: 10
)
```

#### 2. find_similar_code
```typescript
find_similar_code(
    file_path: "src/main.rs",
    limit: 5
)
```

#### 3. cluster_code
```typescript
cluster_code(
    method: "kmeans",
    k: 5
)
```

#### 4. analyze_topics
```typescript
analyze_topics(
    num_topics: 10
)
```

### Example Claude Code Session

```
You: Help me understand the error handling in this codebase

Claude: Let me search for error handling patterns.
[Uses semantic_search("error handling patterns")]

Found 15 error handling implementations. The main patterns are:
1. Result<T, Error> types (10 occurrences)
2. Custom error enums (3 occurrences)
3. Error propagation with ? operator (12 occurrences)

You: Find similar code to src/error.rs

Claude: [Uses find_similar_code("src/error.rs")]

The most similar files are:
1. src/validation.rs - Uses similar Result patterns
2. src/middleware/error.rs - Shared error types
3. src/api/handlers.rs - Error propagation patterns

Would you like me to suggest refactoring opportunities?
```

## Best Practices

### 1. Embedding Sync Strategy

**Initial Sync**:
```bash
# Sync entire codebase once
pmat embed sync ./src
```

**Regular Updates**:
```bash
# Re-sync after significant changes
git pull && pmat embed sync ./src

# Only changed files are re-embedded (fast!)
```

**CI/CD Integration**:
```yaml
# .github/workflows/embeddings.yml
name: Update Embeddings
on: [push]
jobs:
  embeddings:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - run: pmat embed sync ./src
        env:
          OPENAI_API_KEY: ${{ secrets.OPENAI_API_KEY }}
```

### 2. Search Query Tips

**Good Queries**:
- ✅ "error handling with retries"
- ✅ "async database transactions"
- ✅ "HTTP middleware patterns"
- ✅ "authentication logic"

**Bad Queries**:
- ❌ "fix bug" (too vague)
- ❌ "code" (not specific)
- ❌ "good practices" (subjective)

**Pro Tips**:
- Use domain-specific terms
- Include implementation details
- Try both broad and specific queries

### 3. Result Interpretation

**Similarity Scores**:
- **0.9-1.0**: Extremely similar (possible duplicates)
- **0.7-0.9**: Highly related (same concept)
- **0.5-0.7**: Moderately related (related domain)
- **<0.5**: Weakly related (false positives)

**Hybrid Scores**:
- Higher scores = better match
- Consider both keyword and vector components
- Review top 5-10 results for context

### 4. Cost Management

**Typical Costs** (text-embedding-3-small):
- Small project (1K chunks): ~$0.10
- Medium project (10K chunks): ~$1.00
- Large project (50K chunks): ~$5.00

**Cost Optimization**:
- Sync only changed files (automatic)
- Use language filters to reduce scope
- Batch sync large projects during off-hours

## Troubleshooting

### Issue: "API key not found"

**Solution**:
```bash
# Set environment variable
export OPENAI_API_KEY="sk-your-key"

# Verify
echo $OPENAI_API_KEY
```

### Issue: "No results found"

**Possible Causes**:
1. Database is empty (run `pmat embed sync`)
2. Query too specific (try broader terms)
3. Wrong language filter

**Solution**:
```bash
# Check database status
pmat embed status

# Try broader query
pmat semantic search "database" --mode hybrid

# Remove language filter
pmat semantic search "your query" # (no --language flag)
```

### Issue: "Slow search"

**Possible Causes**:
1. Large database (>10K embeddings)
2. Keyword search in large codebase

**Solution**:
```bash
# Use vector-only mode (faster)
pmat semantic search "query" --mode vector

# Reduce result limit
pmat semantic search "query" --limit 5
```

### Issue: "Out of memory"

**Possible Causes**:
1. Too many embeddings loaded at once
2. Large codebase

**Solution**:
```bash
# Use language filter to reduce scope
pmat semantic search "query" --language rust

# Increase system memory
# Or split database by module
```

## Examples

### Example 1: Find All Error Handlers

```bash
pmat semantic search "error handling functions" --mode hybrid --limit 20
```

### Example 2: Identify Duplicate Code

```bash
# Find code similar to a target file
pmat semantic similar src/auth/login.rs --limit 10

# Look for high similarity scores (>0.9)
# Review for refactoring opportunities
```

### Example 3: Understand Codebase Structure

```bash
# Cluster into architectural layers
pmat analyze cluster --method kmeans --k 10

# Extract semantic topics
pmat analyze topics --num-topics 15

# Review cluster/topic assignments to understand organization
```

### Example 4: Cross-Language Search

```bash
# Find authentication logic across languages
pmat semantic search "authentication and authorization"

# Results will include Rust, TypeScript, Python implementations
```

## Next Steps

- Read [Architecture](./architecture.md) for technical deep-dive
- See [API Reference](./api-reference.md) for programmatic usage
- Check [Integration Guide](./integration.md) for MCP setup

---

**Happy Semantic Searching!** 🚀
