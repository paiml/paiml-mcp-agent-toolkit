# Semantic Code Search for PMAT

> **Status**: ✅ Production Ready (v2.158.0)
> **Sprint**: 29-31 Complete
> **Tests**: 102+ passing
> **Coverage**: 95%+

## Overview

PMAT's semantic search system enables AI-powered code discovery using natural language queries. Built on OpenAI embeddings, vector similarity, and hybrid search algorithms, it provides concept-based code navigation beyond traditional keyword search.

## Quick Start

```bash
# 1. Sync embeddings for your codebase
pmat embed sync ./src

# 2. Search by natural language
pmat semantic search "error handling patterns" --mode hybrid

# 3. Find similar code
pmat semantic similar src/main.rs --limit 10

# 4. Cluster code by similarity
pmat analyze cluster --method kmeans --k 5

# 5. Extract semantic topics
pmat analyze topics --num-topics 10
```

## Features

### 🔍 Semantic Search
- **Natural Language Queries**: Search code using plain English
- **Hybrid Search**: Combines keyword matching (ripgrep) with vector similarity
- **Multi-Language Support**: Rust, TypeScript, Python, C/C++, Go
- **Smart Ranking**: Reciprocal Rank Fusion (RRF) algorithm

### 🧠 Code Intelligence
- **AST-Aware Chunking**: Extracts semantic units (functions, classes, modules)
- **Vector Embeddings**: OpenAI text-embedding-3-small (1536 dimensions)
- **Incremental Updates**: SHA256-based change detection
- **Context Preservation**: Maintains code structure and relationships

### 📊 Analytics
- **Clustering**: K-means, Hierarchical, DBSCAN algorithms
- **Topic Modeling**: LDA-inspired topic extraction
- **Quality Metrics**: Silhouette score, coherence score
- **Architecture Discovery**: Identify code patterns and themes

### 🤖 MCP Integration
- **Claude Code**: 4 MCP tools for AI assistants
- **Cursor**: Works with MCP-compatible editors
- **Programmatic**: JSON-based tool interface

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    PMAT Semantic Search                  │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ┌──────────────┐     ┌──────────────┐                 │
│  │  CLI Layer   │────▶│  MCP Tools   │                 │
│  └──────────────┘     └──────────────┘                 │
│         │                     │                          │
│         ▼                     ▼                          │
│  ┌─────────────────────────────────────────┐           │
│  │       Hybrid Search Engine              │           │
│  │  ┌──────────┐        ┌──────────────┐  │           │
│  │  │ Keyword  │  +RRF  │   Vector     │  │           │
│  │  │ (ripgrep)│ ─────▶ │  Similarity  │  │           │
│  │  └──────────┘        └──────────────┘  │           │
│  └─────────────────────────────────────────┘           │
│         │                     │                          │
│         ▼                     ▼                          │
│  ┌──────────────┐     ┌──────────────┐                 │
│  │  Clustering  │     │    Topics    │                 │
│  │   Engine     │     │   Engine     │                 │
│  └──────────────┘     └──────────────┘                 │
│         │                     │                          │
│         ▼                     ▼                          │
│  ┌─────────────────────────────────────────┐           │
│  │        Turso Vector Database            │           │
│  │         (SQLite + Embeddings)           │           │
│  └─────────────────────────────────────────┘           │
│                      ▲                                   │
│                      │                                   │
│  ┌─────────────────────────────────────────┐           │
│  │      OpenAI Embeddings Client           │           │
│  │    (text-embedding-3-small, 1536-d)     │           │
│  └─────────────────────────────────────────┘           │
│                      ▲                                   │
│                      │                                   │
│  ┌─────────────────────────────────────────┐           │
│  │         AST-Aware Code Chunker          │           │
│  │  (Tree-sitter: Rust, TS, Python, C, Go) │           │
│  └─────────────────────────────────────────┘           │
└─────────────────────────────────────────────────────────┘
```

## Components

| Component | File | Lines | Tests | Purpose |
|-----------|------|-------|-------|---------|
| **Code Chunker** | `chunker.rs` | 637 | 20 | AST-aware code extraction |
| **OpenAI Client** | `openai_embeddings.rs` | 274 | 15 | Embedding generation |
| **Vector DB** | `turso_vector_db.rs` | 402 | 12 | SQLite vector storage |
| **Search Engine** | `search_engine.rs` | 377 | 18 | Semantic search orchestration |
| **Hybrid Search** | `hybrid_search.rs` | 457 | 25 | Keyword + vector fusion |
| **Clustering** | `clustering.rs` | 555 | 15 | K-means, hierarchical, DBSCAN |
| **Topic Modeling** | `topic_modeling.rs` | 307 | 10 | LDA topic extraction |
| **MCP Tools** | `semantic_search_tools.rs` | 459 | 20 | AI assistant integration |
| **CLI Commands** | `semantic_commands.rs` | 268 | 14 | Command-line interface |
| **Total** | 9 files | 3,736 | 149 | Full system |

## Performance

| Operation | Typical Time | Max Codebase |
|-----------|-------------|--------------|
| Embedding Generation | 50ms/chunk | 50K chunks |
| Vector Search | <100ms | 10K vectors |
| Hybrid Search | <150ms | 10K chunks |
| Clustering (K-means) | <5s | 10K vectors |
| Topic Modeling | <10s | 10K chunks |

## Cost

### OpenAI Embeddings
- **Model**: text-embedding-3-small
- **Cost**: $0.00002 per 1K tokens
- **Typical Codebase**: 10K chunks ≈ $0.50-$2.00 (one-time)
- **Incremental**: Only changed files re-embedded

### Storage
- **Vector DB**: SQLite file (~2MB per 1K embeddings)
- **Typical Codebase**: 10K chunks ≈ 20MB

## Use Cases

### 1. **Code Discovery**
Find relevant code without knowing exact file names:
```bash
pmat semantic search "authentication logic" --mode hybrid
```

### 2. **Refactoring**
Identify similar code for deduplication:
```bash
pmat semantic similar src/auth/login.rs --limit 20
```

### 3. **Architecture Understanding**
Discover code organization patterns:
```bash
pmat analyze cluster --method kmeans --k 10
pmat analyze topics --num-topics 15
```

### 4. **AI-Assisted Development**
Use MCP tools in Claude Code/Cursor:
- `semantic_search("error handling")`
- `find_similar_code("src/main.rs")`
- `cluster_code(method="kmeans", k=5)`
- `analyze_topics(num_topics=10)`

## Technology Stack

| Layer | Technology | Rationale |
|-------|-----------|-----------|
| **Embeddings** | OpenAI text-embedding-3-small | Best cost/performance ($0.00002/1K tokens) |
| **Vector DB** | Turso (SQLite) | Local-first, zero config, battle-tested |
| **Hybrid Search** | Reciprocal Rank Fusion | Scientifically validated (Cormack et al., 2009) |
| **Chunking** | Tree-sitter AST parsers | Language-aware semantic units |
| **Clustering** | K-means++, Hierarchical, DBSCAN | Industry-standard algorithms |
| **Topic Modeling** | Simplified LDA (K-means based) | Fast, interpretable results |
| **MCP** | pmcp SDK v1.4.2 | AI assistant integration |

## Documentation

- [Architecture](./architecture.md) - System design and algorithms
- [API Reference](./api-reference.md) - Public API documentation
- [User Guide](./user-guide.md) - Getting started and examples
- [Integration Guide](./integration.md) - MCP and programmatic usage
- [Algorithms](./algorithms.md) - Technical deep-dive

## Development

### Testing
```bash
# Run all semantic search tests
cargo test --lib semantic

# Run specific test suites
cargo test unit_code_chunker
cargo test unit_openai_embeddings
cargo test unit_hybrid_search
cargo test unit_kmeans_clustering
cargo test unit_topic_modeling
cargo test semantic_commands
```

### Methodology
All code developed using **EXTREME TDD**:
1. **RED**: Write failing tests first
2. **GREEN**: Implement minimal code to pass
3. **REFACTOR**: Improve code quality

### Quality Metrics
- **Test Coverage**: 95%+
- **Tests**: 149 unit tests
- **Cyclomatic Complexity**: ≤10 per function
- **Clippy Warnings**: 0 (except 4 expected dead_code)

## Roadmap

### ✅ Sprint 29: Foundation (COMPLETE)
- AST-aware code chunker
- OpenAI embeddings client
- Turso vector database

### ✅ Sprint 30: Search Engine (COMPLETE)
- Vector similarity search
- Hybrid search with RRF
- MCP tools integration

### ✅ Sprint 31: Analytics (COMPLETE)
- K-means clustering
- Topic modeling
- CLI commands
- Documentation

### 🔮 Future Enhancements
- [ ] GPU-accelerated vector search
- [ ] Multi-modal embeddings (code + comments)
- [ ] Cross-repo semantic search
- [ ] Real-time embedding sync (watch mode)
- [ ] Interactive TUI
- [ ] Semantic code diff

## Contributing

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for development guidelines.

## License

See [LICENSE](../../LICENSE) for details.

## References

### Academic Papers
- Cormack et al. (2009) - "Reciprocal Rank Fusion"
- Blei et al. (2003) - "Latent Dirichlet Allocation"
- Arthur & Vassilvitskii (2007) - "k-means++: The advantages of careful seeding"

### Technologies
- [OpenAI Embeddings](https://platform.openai.com/docs/guides/embeddings)
- [Turso Database](https://turso.tech/)
- [Tree-sitter](https://tree-sitter.github.io/)
- [MCP Protocol](https://github.com/anthropics/mcp)

---

**Built with EXTREME TDD** | **v2.158.0** | **October 2025**
