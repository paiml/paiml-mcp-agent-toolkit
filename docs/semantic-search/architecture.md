# Semantic Search Architecture

> **Design Philosophy**: Local-first, zero-config, production-ready

## System Overview

PMAT's semantic search system uses a layered architecture with clear separation of concerns:

```
┌─────────────────────────────────────────────────────────────┐
│                      CLIENT LAYER                           │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │   CLI    │  │   MCP    │  │   API    │  │   TUI    │  │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘  │
└─────────────────────────────────────────────────────────────┘
                           │
┌─────────────────────────────────────────────────────────────┐
│                     SERVICE LAYER                           │
│  ┌──────────────────────────────────────────────────────┐  │
│  │           Hybrid Search Engine                       │  │
│  │  • Query parsing & validation                        │  │
│  │  • Keyword search (ripgrep)                          │  │
│  │  • Vector search (cosine similarity)                 │  │
│  │  • Result fusion (RRF algorithm)                     │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                              │
│  ┌────────────────────┐    ┌─────────────────────────┐    │
│  │ Clustering Engine  │    │   Topic Engine          │    │
│  │ • K-means          │    │ • Simplified LDA        │    │
│  │ • Hierarchical     │    │ • Keyword extraction    │    │
│  │ • DBSCAN           │    │ • Coherence scoring     │    │
│  └────────────────────┘    └─────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
                           │
┌─────────────────────────────────────────────────────────────┐
│                      DATA LAYER                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │           Turso Vector Database (SQLite)             │  │
│  │  • Embedding storage (JSON arrays)                   │  │
│  │  • Metadata indexing                                 │  │
│  │  • Incremental updates (checksums)                   │  │
│  │  • UNIQUE constraints (upsert semantics)             │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                           │
┌─────────────────────────────────────────────────────────────┐
│                   INFRASTRUCTURE LAYER                      │
│  ┌───────────────────┐  ┌──────────────────────────────┐  │
│  │ OpenAI Client     │  │   AST Chunker               │  │
│  │ • Rate limiting   │  │ • Tree-sitter parsers        │  │
│  │ • Retry logic     │  │ • 5 languages supported      │  │
│  │ • Cost tracking   │  │ • SHA256 checksums           │  │
│  └───────────────────┘  └──────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Core Algorithms

### 1. AST-Aware Code Chunking

**Purpose**: Extract semantic units (functions, classes, modules) from source code

**Algorithm**:
```
1. Parse source code with tree-sitter
2. Traverse AST to find semantic nodes:
   - Functions (all languages)
   - Classes (OOP languages)
   - Modules (Rust, Python)
   - Interfaces (TypeScript)
3. Extract node text + context
4. Compute SHA256 checksum for change detection
5. Return CodeChunk with metadata
```

**Languages Supported**:
- Rust: `tree-sitter-rust`
- TypeScript: `tree-sitter-typescript`
- Python: `tree-sitter-python`
- C/C++: `tree-sitter-c`, `tree-sitter-cpp`
- Go: `tree-sitter-go`

**Complexity**: O(n) where n = source file size

### 2. Embedding Generation

**Purpose**: Convert code chunks to 1536-dimensional vectors

**Model**: OpenAI text-embedding-3-small
- **Dimensions**: 1536
- **Cost**: $0.00002 per 1K tokens
- **Max Input**: 8191 tokens per request

**Algorithm**:
```
1. Batch chunks (max 100 per request)
2. Send to OpenAI API
3. Retry with exponential backoff on failure
4. Parse response vectors
5. Store embeddings with metadata
6. Track cost (tokens * $0.00002/1K)
```

**Optimizations**:
- Batch processing (up to 100 chunks)
- Retry logic (max 3 attempts)
- Rate limiting (handle 429 errors)

**Complexity**: O(b) where b = batch size

### 3. Vector Similarity Search

**Purpose**: Find code chunks similar to query embedding

**Algorithm**: Cosine Similarity
```
similarity(v1, v2) = dot(v1, v2) / (||v1|| * ||v2||)

Where:
- dot(v1, v2) = Σ(v1[i] * v2[i])
- ||v|| = sqrt(Σ(v[i]²))
```

**Implementation**:
```
1. Generate query embedding
2. Fetch all embeddings from DB
3. Compute cosine similarity for each
4. Sort by similarity (descending)
5. Apply limit
6. Return top-k results
```

**Complexity**: O(n*d) where n = vectors, d = dimensions (1536)

**Future Optimization**: HNSW index for sub-linear search

### 4. Hybrid Search with RRF

**Purpose**: Combine keyword matching and vector search for best results

**Algorithm**: Reciprocal Rank Fusion (Cormack et al., 2009)
```
RRF(d) = Σ_{r ∈ R} 1 / (k + r(d))

Where:
- d = document
- R = set of rankings (keyword, vector)
- r(d) = rank of document d in ranking r
- k = constant (60)
```

**Implementation**:
```
1. Execute keyword search (ripgrep)
   - Fast exact matching
   - Rank by relevance score

2. Execute vector search
   - Semantic similarity
   - Rank by cosine similarity

3. Compute RRF scores:
   keyword_rrf = 1 / (60 + keyword_rank)
   vector_rrf = 1 / (60 + vector_rank)

4. Combine with weights:
   final_score = (w_k * keyword_rrf) + (w_v * vector_rrf)

5. Sort by final_score
6. Deduplicate by (file_path, chunk_name)
7. Return top-k results
```

**Complexity**: O(n log n) for sorting

### 5. K-means Clustering

**Purpose**: Group code by semantic similarity

**Algorithm**: Lloyd's algorithm with k-means++ initialization

```
Initialization (k-means++):
1. Choose first centroid randomly
2. For each remaining centroid:
   a. Compute D(x)² = squared distance to nearest centroid
   b. Choose next centroid with probability ∝ D(x)²
   c. Repeat until k centroids selected

Iteration:
1. Assignment: Assign each point to nearest centroid
2. Update: Recompute centroids as mean of assigned points
3. Convergence: Stop if centroids don't change (or max iterations)

Output: Cluster labels (0 to k-1)
```

**Distance Metric**: Euclidean distance
```
distance(v1, v2) = sqrt(Σ(v1[i] - v2[i])²)
```

**Quality Metric**: Silhouette score
```
silhouette(i) = (b(i) - a(i)) / max(a(i), b(i))

Where:
- a(i) = average distance to points in same cluster
- b(i) = average distance to nearest other cluster
```

**Complexity**: O(n*k*i*d)
- n = points
- k = clusters
- i = iterations (typically <100)
- d = dimensions

### 6. Topic Modeling (Simplified LDA)

**Purpose**: Extract semantic topics from codebase

**Algorithm**: K-means-based topic extraction

```
1. Cluster embeddings using K-means (num_topics clusters)
2. For each cluster:
   a. Extract chunk names
   b. Tokenize names (split on non-alphanumeric)
   c. Count word frequencies
   d. Take top-k words as keywords
3. Compute coherence score:
   coherence = 1 - (avg_keyword_overlap / max_overlap)
4. Return topics with keywords + representative chunks
```

**Keyword Extraction**:
```
1. Split chunk names on delimiters: _/-/camelCase
2. Lowercase all words
3. Filter words with length < 3
4. Count frequencies
5. Sort by frequency (descending)
6. Return top-k words
```

**Complexity**: O(n*k) + O(m log m) for sorting keywords

## Data Model

### CodeChunk
```rust
pub struct CodeChunk {
    pub file_path: String,      // Relative path
    pub chunk_type: ChunkType,  // Function, Class, Module, etc.
    pub chunk_name: String,      // Identifier name
    pub language: String,        // rust, typescript, python, etc.
    pub start_line: usize,       // Start line number
    pub end_line: usize,         // End line number
    pub content: String,         // Full source text
    pub content_checksum: String, // SHA256 for incremental updates
}
```

### EmbeddingEntry (Database)
```sql
CREATE TABLE code_embeddings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_path TEXT NOT NULL,
    chunk_name TEXT NOT NULL,
    chunk_type TEXT NOT NULL,
    language TEXT NOT NULL,
    start_line INTEGER NOT NULL,
    end_line INTEGER NOT NULL,
    content_checksum TEXT NOT NULL,
    embedding TEXT NOT NULL,     -- JSON: [0.123, -0.456, ...]
    model TEXT NOT NULL,          -- "text-embedding-3-small"
    created_at INTEGER NOT NULL,  -- Unix timestamp

    UNIQUE(file_path, chunk_name, content_checksum)
);

CREATE INDEX idx_file_path ON code_embeddings(file_path);
CREATE INDEX idx_language ON code_embeddings(language);
CREATE INDEX idx_checksum ON code_embeddings(content_checksum);
```

### SearchResult
```rust
pub struct HybridSearchResult {
    pub file_path: String,
    pub chunk_name: String,
    pub chunk_type: String,
    pub language: String,
    pub hybrid_score: f64,       // Combined RRF score
    pub keyword_score: f64,      // Ripgrep relevance
    pub vector_score: f64,       // Cosine similarity
    pub snippet: String,         // Context excerpt
    pub start_line: usize,
    pub end_line: usize,
}
```

## Scalability

### Current Limits
- **Max Embeddings**: 50K chunks
- **Max Query Time**: <150ms (typical)
- **Max Database Size**: ~100MB (50K embeddings)

### Optimization Strategies

#### 1. Embedding Storage
- **Current**: JSON arrays in TEXT column
- **Future**: Binary BLOB for 3x compression

#### 2. Vector Search
- **Current**: Brute-force O(n) scan
- **Future**: HNSW index for O(log n) search

#### 3. Incremental Updates
- **Current**: SHA256 checksums
- **Optimization**: Content-addressable storage

#### 4. Parallel Processing
- **Current**: Sequential embedding generation
- **Future**: Parallel batch processing

## Security

### API Key Management
- Stored in environment variables
- Never logged or persisted
- Validated before use

### Data Privacy
- All processing local (except OpenAI API)
- No telemetry or external tracking
- Database stored locally

### Input Validation
- Query sanitization
- Path traversal prevention
- SQL injection protection (parameterized queries)

## Error Handling

### Retry Logic
```rust
retry_with_backoff(operation, max_retries=3) {
    for attempt in 1..=max_retries {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) if retryable(e) => {
                delay = base_delay * 2^(attempt-1)
                sleep(delay)
            }
            Err(e) => return Err(e)
        }
    }
}
```

### Graceful Degradation
- Keyword-only fallback if embeddings fail
- Partial results if some chunks fail
- Clear error messages for user debugging

## Testing Strategy

### Unit Tests
- Every public function has tests
- Edge cases covered
- Error paths validated

### Integration Tests
- End-to-end workflows
- Real database operations
- API mocking for OpenAI

### Property Tests
- K-means convergence
- RRF score monotonicity
- Embedding normalization

## Performance Benchmarks

| Operation | Input Size | Time | Memory |
|-----------|-----------|------|--------|
| Chunk Extraction | 1K LOC | <50ms | <10MB |
| Embedding Generation | 100 chunks | <500ms | <20MB |
| Vector Search | 10K embeddings | <100ms | <50MB |
| Hybrid Search | 10K chunks | <150ms | <60MB |
| K-means Clustering | 1K vectors | <1s | <30MB |
| Topic Modeling | 1K chunks | <2s | <30MB |

*Measured on 2023 MacBook Pro M2*

## Future Enhancements

### Short-term
- [ ] Multi-threaded embedding generation
- [ ] Streaming search results
- [ ] Progress indicators for long operations

### Medium-term
- [ ] HNSW vector index
- [ ] GPU acceleration
- [ ] Distributed search across repos

### Long-term
- [ ] Fine-tuned code embeddings
- [ ] Multi-modal search (code + docs)
- [ ] Real-time collaborative search

---

**Design Principles**: Simple, Fast, Local-First, Production-Ready
