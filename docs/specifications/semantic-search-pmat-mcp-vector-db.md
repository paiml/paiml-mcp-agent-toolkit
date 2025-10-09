# PMAT Semantic Code Search - MCP Vector Database Integration

**Version**: 1.0
**Created**: October 9, 2025
**Status**: 🟡 SPECIFICATION (Ready for Implementation)
**Methodology**: EXTREME TDD
**Estimated Effort**: 2-3 sprints (Sprint 29-31)

---

## 🎯 Executive Summary

Add semantic code search capabilities to PMAT using OpenAI embeddings and vector similarity, enabling AI assistants to find code by meaning rather than just keywords. Integrates with existing MCP server to provide powerful code discovery tools for Claude Code, Cursor, and other AI assistants.

**Key Benefits:**
- 🧠 **Find code by concept**: "memory safety patterns" finds relevant code even without exact keyword matches
- 🔀 **Hybrid search**: Combines grep-like keyword search with semantic similarity (Reciprocal Rank Fusion)
- 🤖 **MCP integration**: Exposes as AI assistant tools (already have MCP infrastructure)
- 💰 **Cost-effective**: ~$0.04 per 1,000 code files using OpenAI text-embedding-3-small
- ⚡ **Incremental**: Only embeds new/modified files (checksum-based)
- 📊 **Analytics**: Clustering, topic modeling, similarity analysis for codebase insights

---

## 📋 Background & Motivation

### Current State (PMAT v2.156.0)

**What PMAT Can Do:**
- ✅ Deep context generation (AST-based)
- ✅ Complexity analysis (cyclomatic, cognitive)
- ✅ Quality gates (SATD, dead code)
- ✅ MCP server integration (pmcp SDK v1.4.2)
- ✅ Multi-language support (Rust, TypeScript, Python, C/C++, Go, Java, C#, Kotlin, Swift, Elixir)

**What's Missing:**
- ❌ Semantic code search (find by meaning)
- ❌ Code similarity detection
- ❌ Cluster analysis (discover code patterns)
- ❌ Topic modeling (identify architectural themes)
- ❌ Cross-file relationship discovery

### Proven Solution: AssetsSearch

AssetsSearch (our sibling project) has successfully implemented:
- **Phase 5 Complete**: Semantic search with hybrid FTS5+vector search
- **26 tests passing**: Embedding pipeline with GREEN phase stubs
- **Turso/SQLite**: Vector storage with JSON arrays
- **OpenAI Integration**: text-embedding-3-small (1536 dimensions)
- **RRF Algorithm**: Reciprocal Rank Fusion for hybrid search

**Architecture to Adopt:**
```
AssetsSearch Pattern:
  Transcripts → OpenAI Embeddings → Turso Vector DB → Hybrid Search (FTS5 + Vector)

PMAT Adaptation:
  Code Files → OpenAI Embeddings → Turso Vector DB → Hybrid Search (ripgrep + Vector)
```

---

## 🏗️ Architecture Overview

### High-Level Flow

```
┌─────────────────────┐
│ Code Files          │
│ (Rust, TS, Py, etc) │
└──────────┬──────────┘
           │
           ↓
┌──────────────────────────────────────┐
│ Embedding Pipeline                   │
│ - Chunking (by function/class/module)│
│ - Checksum tracking                  │
│ - Incremental sync                   │
└──────────┬───────────────────────────┘
           │
           ↓
┌──────────────────────────────────────┐
│ OpenAI Embeddings API                │
│ Model: text-embedding-3-small        │
│ Dimensions: 1536                     │
│ Cost: $0.00002/1K tokens             │
└──────────┬───────────────────────────┘
           │
           ↓
┌──────────────────────────────────────┐
│ Turso Vector Database (SQLite)       │
│ - Local: .pmat-cache/embeddings.db   │
│ - Schema: code_embeddings table      │
│ - Format: JSON array (1536 floats)   │
└──────────┬───────────────────────────┘
           │
           ↓
┌──────────────────────────────────────┐
│ Hybrid Search Engine                 │
│ - ripgrep (keyword search)           │
│ - Vector similarity (cosine)         │
│ - Reciprocal Rank Fusion (RRF)      │
└──────────┬───────────────────────────┘
           │
           ↓
┌──────────────────────────────────────┐
│ MCP Tools (Claude Code, Cursor)      │
│ - semantic_search                    │
│ - find_similar_code                  │
│ - cluster_code                       │
│ - analyze_topics                     │
└──────────────────────────────────────┘
```

### Component Design

#### 1. Code Chunking Strategy

**Challenge**: Code files can be large (>100K tokens). Must chunk intelligently.

**Solution**: AST-Aware Chunking
- Use existing PMAT AST infrastructure
- Chunk by **semantic units**: functions, classes, modules
- Each chunk = 1 embedding

**Chunking Rules:**
```rust
// Function-level chunking (preferred)
fn calculate_total(items: &[Item]) -> f64 {
    // Chunk: Full function with signature + docstring
}

// Class-level chunking (for OOP languages)
class UserManager {
    // Chunk: Class with all methods
}

// Module-level chunking (for small modules)
mod utils {
    // Chunk: Entire module if <500 lines
}
```

**Metadata per Chunk:**
- `file_path`: Full path to source file
- `chunk_type`: "function" | "class" | "module" | "file"
- `chunk_name`: Identifier (function name, class name)
- `language`: Detected language
- `start_line`, `end_line`: Location in file
- `checksum`: SHA256 of chunk content (for incremental updates)

#### 2. Embedding Storage Schema

**Database**: Turso (SQLite with vector support)
**Location**: `.pmat-cache/embeddings.db`

```sql
CREATE TABLE code_embeddings (
    id INTEGER PRIMARY KEY,
    file_path TEXT NOT NULL,
    chunk_type TEXT NOT NULL,  -- 'function', 'class', 'module', 'file'
    chunk_name TEXT NOT NULL,  -- identifier
    language TEXT NOT NULL,    -- 'rust', 'typescript', 'python', etc
    start_line INTEGER NOT NULL,
    end_line INTEGER NOT NULL,
    content_checksum TEXT NOT NULL,  -- SHA256 for incremental updates
    embedding TEXT NOT NULL,   -- JSON array: [0.123, -0.456, ...] (1536 floats)
    model TEXT NOT NULL,       -- 'text-embedding-3-small'
    dimensions INTEGER NOT NULL,  -- 1536
    token_count INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE(file_path, chunk_type, chunk_name)
);

CREATE INDEX idx_file_path ON code_embeddings(file_path);
CREATE INDEX idx_language ON code_embeddings(language);
CREATE INDEX idx_chunk_type ON code_embeddings(chunk_type);
CREATE INDEX idx_checksum ON code_embeddings(content_checksum);
```

#### 3. Hybrid Search Algorithm

**Reciprocal Rank Fusion (RRF)** - Proven by AssetsSearch

```rust
fn hybrid_search(
    query: &str,
    mode: SearchMode,
    ripgrep_weight: f32,
    vector_weight: f32,
) -> Vec<SearchResult> {
    // Step 1: Run both searches in parallel
    let (ripgrep_results, vector_results) = tokio::join!(
        ripgrep_search(query),
        vector_search(query_embedding)
    );

    // Step 2: Apply Reciprocal Rank Fusion
    // RRF_score = Σ (weight / (k + rank))
    // k = 60 (constant from Cormack et al.)

    let k = 60;
    for (rank, result) in ripgrep_results.iter().enumerate() {
        scores[result.id] += ripgrep_weight / (k + rank + 1);
    }
    for (rank, result) in vector_results.iter().enumerate() {
        scores[result.id] += vector_weight / (k + rank + 1);
    }

    // Step 3: Sort by combined score
    results.sort_by(|a, b| scores[b.id].cmp(&scores[a.id]));
    results
}
```

**Search Modes:**
- `ripgrep-only`: Fast keyword/regex matching
- `vector-only`: Pure semantic similarity
- `hybrid`: Combined (default, 50/50 weights)

---

## 🚀 Implementation Plan

### Sprint 29: Foundation & Embedding Pipeline (Week 1)

**Goal**: Implement core embedding generation pipeline

#### Ticket 1: Code Chunker (PMAT-SEARCH-001)

**RED Phase:**
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_chunk_rust_file_by_functions() {
        let source = r#"
            fn foo() { }
            fn bar() { }
        "#;
        let chunks = chunk_code(source, "rust");
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].chunk_type, ChunkType::Function);
        assert_eq!(chunks[0].chunk_name, "foo");
    }
}
```

**GREEN Phase:**
- Use existing PMAT AST parsers (already have for 14+ languages!)
- Extract functions, classes, modules
- Include docstrings/comments in chunks

**Deliverables:**
- `src/services/semantic/chunker.rs` (new module)
- Support: Rust, TypeScript, Python, C/C++, Go (top 5)
- 20 unit tests (1 test per language + edge cases)

**Complexity Target**: ≤10 cyclomatic per function

---

#### Ticket 2: OpenAI Embeddings Client (PMAT-SEARCH-002)

**RED Phase:**
```rust
#[tokio::test]
async fn test_generate_embedding() {
    let client = OpenAIClient::new("sk-test");
    let embedding = client.embed("fn main() { }").await?;
    assert_eq!(embedding.len(), 1536);
}
```

**GREEN Phase:**
- Use `reqwest` for HTTP client
- Batch processing (50 chunks per API call)
- Retry logic with exponential backoff
- Rate limit handling

**Deliverables:**
- `src/services/semantic/embeddings.rs`
- 15 unit tests (batching, errors, retry)
- Mock OpenAI API in tests

**Dependencies:**
```toml
[dependencies]
reqwest = { version = "0.12", features = ["json"] }
tokio = { version = "1.47", features = ["full"] }
```

---

#### Ticket 3: Turso Vector Database (PMAT-SEARCH-003)

**RED Phase:**
```rust
#[test]
fn test_store_embedding() {
    let db = TursoDB::new(".pmat-cache/test.db")?;
    let embedding = vec![0.1; 1536];
    db.store_embedding("src/main.rs", "function", "main", &embedding)?;

    let retrieved = db.get_embedding("src/main.rs", "function", "main")?;
    assert_eq!(retrieved.len(), 1536);
}
```

**GREEN Phase:**
- SQLite with Turso extensions
- Store embeddings as JSON arrays
- Checksum-based incremental updates

**Deliverables:**
- `src/services/semantic/turso.rs`
- 12 unit tests (CRUD operations)
- Migration script for schema creation

**Dependencies:**
```toml
[dependencies]
rusqlite = { version = "0.32", features = ["bundled"] }
serde_json = "1.0"
```

---

### Sprint 30: Search Engine & MCP Tools (Week 2)

#### Ticket 4: Vector Similarity Search (PMAT-SEARCH-004)

**RED Phase:**
```rust
#[test]
fn test_cosine_similarity() {
    let query = vec![1.0; 1536];
    let results = search_similar(&query, limit=10)?;
    assert!(results[0].score >= results[1].score); // Sorted by score
}
```

**GREEN Phase:**
- Cosine similarity calculation
- K-nearest neighbors search
- Filtering by language, file path

**Deliverables:**
- `src/services/semantic/vector_search.rs`
- 18 unit tests (similarity, filtering, edge cases)

---

#### Ticket 5: Hybrid Search Engine (PMAT-SEARCH-005)

**RED Phase:**
```rust
#[tokio::test]
async fn test_hybrid_search() {
    let results = hybrid_search(
        "ownership and borrowing",
        SearchMode::Hybrid,
        0.5, 0.5
    ).await?;

    assert!(results.len() > 0);
    assert!(results[0].score > 0.0);
}
```

**GREEN Phase:**
- Integrate ripgrep for keyword search
- Implement Reciprocal Rank Fusion (RRF)
- Weight tuning (default 50/50)

**Deliverables:**
- `src/services/semantic/hybrid.rs`
- 25 unit tests (modes, weights, fusion algorithm)

---

#### Ticket 6: MCP Tools Integration (PMAT-SEARCH-006)

**RED Phase:**
```rust
#[test]
fn test_mcp_semantic_search_tool() {
    let tool = SemanticSearchTool::new();
    let result = tool.execute(json!({
        "query": "error handling patterns",
        "mode": "hybrid",
        "limit": 10
    }))?;

    assert!(result["results"].is_array());
}
```

**GREEN Phase:**
- Add 4 new MCP tools:
  1. `semantic_search` - Main search tool
  2. `find_similar_code` - Find similar files/functions
  3. `cluster_code` - K-means clustering
  4. `analyze_topics` - Topic extraction

**Deliverables:**
- Update `src/bin/pmat-agent.rs` with new tools
- 20 MCP integration tests
- JSON schemas for each tool

**Tool Schemas:**
```typescript
// semantic_search
{
  "name": "semantic_search",
  "description": "Search codebase by semantic meaning",
  "inputSchema": {
    "type": "object",
    "properties": {
      "query": { "type": "string", "description": "Search query" },
      "mode": {
        "type": "string",
        "enum": ["ripgrep-only", "vector-only", "hybrid"],
        "default": "hybrid"
      },
      "language": { "type": "string", "optional": true },
      "limit": { "type": "number", "default": 10 }
    },
    "required": ["query"]
  }
}
```

---

### Sprint 31: Analytics & Polish (Week 3)

#### Ticket 7: Code Clustering (PMAT-SEARCH-007)

**Implementation**: K-means clustering on embeddings

**Use Cases:**
- Discover architectural patterns
- Find duplicate/similar functionality
- Identify refactoring opportunities

**Deliverables:**
- `src/services/semantic/clustering.rs`
- Support K-means, hierarchical, DBSCAN
- 15 unit tests

---

#### Ticket 8: Topic Modeling (PMAT-SEARCH-008)

**Implementation**: LDA (Latent Dirichlet Allocation) on embeddings

**Use Cases:**
- Identify code themes (e.g., "error handling", "networking", "data structures")
- Documentation generation
- Onboarding assistance

**Deliverables:**
- `src/services/semantic/topics.rs`
- 10 unit tests

---

#### Ticket 9: CLI Commands (PMAT-SEARCH-009)

**Commands to Add:**
```bash
# Embedding management
pmat embed sync ./src --all
pmat embed status
pmat embed regenerate --model text-embedding-3-large

# Semantic search
pmat semantic search "ownership patterns" --mode hybrid
pmat semantic similar src/main.rs --limit 20

# Analytics
pmat analyze cluster --method kmeans --k 10
pmat analyze topics --num-topics 15
```

**Deliverables:**
- `src/cli/semantic.rs` (new CLI module)
- 30 CLI integration tests

---

#### Ticket 10: Documentation (PMAT-SEARCH-010)

**Deliverables:**
- `docs/features/semantic-search.md`
- `docs/features/code-clustering.md`
- `docs/features/embedding-pipeline.md`
- Updated README with examples
- ROADMAP update for Sprint 29-31

---

## 📊 Cost Analysis

### OpenAI Pricing (text-embedding-3-small)

| Codebase Size | Chunks (~500 tokens each) | Cost Estimate |
|---------------|---------------------------|---------------|
| Small (1K files) | ~5K chunks | $0.05 (5¢) |
| Medium (10K files) | ~50K chunks | $0.50 (50¢) |
| Large (100K files) | ~500K chunks | $5.00 |

**Assumptions:**
- Average: 5 functions per file
- Average: 500 tokens per function (includes context)
- Model: text-embedding-3-small ($0.00002/1K tokens)

**Incremental Cost** (after initial sync):
- Only re-embed changed files (checksum-based)
- Typical: 1-5% of codebase per day
- Daily cost for 10K file project: $0.005-$0.025 (0.5¢ - 2.5¢)

---

## 🎯 Success Metrics

### Must-Have (MVP)

- [ ] Embedding pipeline generates embeddings for Rust/TypeScript/Python
- [ ] Vector similarity search works with cosine distance
- [ ] Hybrid search combines ripgrep + vector with RRF
- [ ] 4 MCP tools working in Claude Code
- [ ] CLI commands for embed sync and semantic search
- [ ] Incremental updates based on checksums
- [ ] 100+ tests passing (unit + integration)

### Should-Have

- [ ] Support for all 14 PMAT languages
- [ ] K-means clustering for code discovery
- [ ] Topic modeling for architecture insights
- [ ] Batch processing with progress bars
- [ ] Cost tracking and reporting
- [ ] Local caching to minimize API calls

### Nice-to-Have

- [ ] Alternative embedding models (Cohere, Voyage AI)
- [ ] Multi-modal embeddings (code + comments + docs)
- [ ] Graph-based code relationship analysis
- [ ] Similarity-based code review suggestions
- [ ] Auto-detect refactoring opportunities via clustering

---

## 🔧 Technical Decisions

### Why OpenAI Embeddings?

**✅ Chosen**: OpenAI text-embedding-3-small

**Alternatives Considered:**
- Cohere embed-v3: $0.0001/1K tokens (5x more expensive)
- Voyage AI code-2: $0.00012/1K tokens (6x more expensive)
- Local models (sentence-transformers): Free but slower, less accurate

**Rationale:**
- Proven in AssetsSearch
- Best cost/performance ratio
- 1536 dimensions sufficient for code
- Stable API with good docs

---

### Why Turso/SQLite for Vector Storage?

**✅ Chosen**: Turso (SQLite-based)

**Alternatives Considered:**
- Pinecone: Requires cloud, $70/month minimum
- Weaviate: Complex setup, overkill for local search
- pgvector: Requires PostgreSQL, heavyweight

**Rationale:**
- Local-first (no cloud dependency)
- Zero configuration (SQLite file)
- AssetsSearch proven success
- Can migrate to Turso cloud later if needed

---

### Why Reciprocal Rank Fusion?

**✅ Chosen**: RRF for hybrid search

**Alternatives Considered:**
- Weighted averaging: Requires score normalization (complex)
- CombSUM: Sensitive to score magnitudes
- Linear combination: Less robust than RRF

**Rationale:**
- Scientifically validated (Cormack et al., 2009)
- Works well without score normalization
- Proven in AssetsSearch hybrid search
- Simple to implement and tune

---

## 🚧 Risks & Mitigations

### Risk 1: Large Codebase Embedding Costs

**Risk**: Embedding 100K files could cost $5-$10
**Probability**: Medium
**Impact**: Low (one-time cost)

**Mitigation:**
- Implement progressive embedding (critical files first)
- Add cost estimation before batch operations
- Cache aggressively (checksum-based)
- Offer local embedding models as alternative

---

### Risk 2: Vector Search Performance

**Risk**: Slow similarity search on large embedding sets
**Probability**: Medium
**Impact**: Medium (user experience)

**Mitigation:**
- Use approximate nearest neighbors (ANN) if >10K embeddings
- Add language/path filtering to reduce search space
- Implement result caching for common queries
- Consider HNSW index (Turso supports this)

---

### Risk 3: Chunking Quality

**Risk**: Poor chunk boundaries reduce search quality
**Probability**: Low (PMAT has mature AST parsers)
**Impact**: High (core feature quality)

**Mitigation:**
- Leverage existing PMAT AST infrastructure
- Test chunking extensively per language
- Include surrounding context in chunks
- Property-based testing for edge cases

---

## 📚 References & Prior Art

### AssetsSearch Implementation
- **Semantic Search**: `docs/features/semantic-search.md`
- **Embedding Pipeline**: `docs/features/embedding-pipeline.md`
- **Vector Analytics**: `docs/features/vector-analytics.md`
- **Tests**: 65 tests across 3 test files

### Academic References
- **RRF**: Cormack et al. (2009) - "Reciprocal Rank Fusion outperforms the best known automatic evaluation"
- **Embeddings**: Mikolov et al. (2013) - "Efficient Estimation of Word Representations in Vector Space"
- **Code Search**: Husain et al. (2019) - "CodeSearchNet Challenge"

### Industry Examples
- GitHub Copilot: Uses Codex embeddings for code search
- Sourcegraph: Hybrid search with keyword + embeddings
- Tabnine: Local code embeddings for context

---

## 🎓 Learning Opportunities

This feature provides excellent learning for:
- Vector databases and similarity search
- Hybrid search algorithms (RRF)
- AST-aware code analysis
- Cost-effective ML API integration
- Incremental data pipelines
- MCP protocol extensions

---

## ✅ Definition of Done

**Sprint 29 (Foundation):**
- [ ] Code chunker works for top 5 languages
- [ ] OpenAI client generates embeddings
- [ ] Turso DB stores/retrieves embeddings
- [ ] 45+ tests passing
- [ ] Documentation: architecture.md

**Sprint 30 (Search):**
- [ ] Vector similarity search implemented
- [ ] Hybrid search with RRF working
- [ ] 4 MCP tools integrated
- [ ] 65+ tests passing (total)
- [ ] Documentation: semantic-search.md

**Sprint 31 (Analytics):**
- [ ] Clustering and topic modeling working
- [ ] CLI commands fully implemented
- [ ] 100+ tests passing (total)
- [ ] All documentation complete
- [ ] ROADMAP updated

---

**Created**: October 9, 2025
**Next Steps**: Review → Approve → Start Sprint 29 EXTREME TDD implementation
**Assigned**: TBD
**Blocked By**: None (all dependencies available)

🦀 **Ready for semantic code intelligence in PMAT!** 🧠
