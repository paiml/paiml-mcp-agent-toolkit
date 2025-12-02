# Semantic Search Feature Specification

**Version**: 2.0.0
**Status**: Production
**Author**: PAIML Team
**Last Updated**: 2025-12-02
**Toyota Way Alignment**: Jidoka (Built-in Quality), Genchi Genbutsu (Go and See)

---

## Executive Summary

This specification defines PMAT's semantic search feature using **pure Rust implementations only**:

- **trueno-rag** (path dependency): RAG pipeline with chunking, hybrid retrieval, reranking
- **trueno-graph** (path dependency): Graph database with PageRank, BFS, pattern detection
- **aprender 0.14.0** (crates.io): TF-IDF, LDA, clustering algorithms

**Zero external API dependencies.** No OpenAI, no cloud services, no API keys required.

---

## Table of Contents

1. [Architecture Overview](#1-architecture-overview)
2. [Core Dependencies](#2-core-dependencies)
3. [Feature Components](#3-feature-components)
4. [Toyota Way Implementation](#4-toyota-way-implementation)
5. [Dog Food Sprint](#5-dog-food-sprint)
6. [Organizational Intelligence Integration](#6-organizational-intelligence-integration)
7. [Performance Targets](#7-performance-targets)
8. [Academic Foundation](#8-academic-foundation)
9. [Implementation Plan](#9-implementation-plan)
10. [Quality Gates](#10-quality-gates)

---

## 1. Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    PMAT Semantic Search Architecture                         │
│                        (Pure Rust - Zero API Keys)                           │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────────────────┐   │
│  │   CLI/MCP    │───▶│   Indexer    │───▶│       trueno-rag             │   │
│  │   Interface  │    │              │    │  ┌────────────────────────┐  │   │
│  └──────────────┘    └──────────────┘    │  │ Chunking Strategies:   │  │   │
│                                          │  │  - Recursive (code)    │  │   │
│  ┌──────────────┐    ┌──────────────┐    │  │  - Structural (AST)    │  │   │
│  │   Query      │───▶│   Retriever  │───▶│  │  - Semantic (topics)   │  │   │
│  │   Engine     │    │              │    │  └────────────────────────┘  │   │
│  └──────────────┘    └──────────────┘    │  ┌────────────────────────┐  │   │
│                                          │  │ Hybrid Retrieval:      │  │   │
│  ┌──────────────┐    ┌──────────────┐    │  │  - BM25 (sparse)       │  │   │
│  │   Cluster    │───▶│   aprender   │    │  │  - TF-IDF (dense)      │  │   │
│  │   Topics     │    │   (0.14.0)   │    │  │  - RRF Fusion          │  │   │
│  └──────────────┘    └──────────────┘    │  └────────────────────────┘  │   │
│                                          └──────────────────────────────┘   │
│                                                                              │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │                        trueno-graph                                   │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │   │
│  │  │ CSR Storage │  │  PageRank   │  │   Louvain   │  │  Patterns   │  │   │
│  │  │   O(1)      │  │  Scoring    │  │  Clustering │  │  Detection  │  │   │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘  │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Core Dependencies

### 2.1 trueno-rag (Path Dependency)

```toml
[dependencies]
trueno-rag = { path = "../trueno-rag" }
```

**Capabilities**:
| Component | Description | Use Case |
|-----------|-------------|----------|
| `RecursiveChunker` | Hierarchical splitting | Code files [10] |
| `StructuralChunker` | Header/section-aware | Markdown docs |
| `SemanticChunker` | Topic-based grouping | Large files |
| `HybridRetriever` | Dense + Sparse search | Query engine [1] |
| `RRF Fusion` | Reciprocal Rank Fusion | Result merging [2] |
| `LexicalReranker` | Post-retrieval ranking | Precision boost |

### 2.2 trueno-graph (Path Dependency)

```toml
[dependencies]
trueno-graph = { path = "../trueno-graph" }
```

**Capabilities**:
| Component | Description | Use Case |
|-----------|-------------|----------|
| `CsrGraph` | CSR storage | O(1) neighbor queries |
| `pagerank()` | PageRank algorithm | Code importance [8] |
| `bfs()` | Breadth-first search | Dependency traversal |
| `louvain()` | Community detection | Module clustering [9] |
| `find_patterns()` | Anti-pattern detection | Code smells |

### 2.3 aprender (crates.io)

```toml
[dependencies]
aprender = "0.14.0"
```

**Capabilities**:
| Component | Description | Use Case |
|-----------|-------------|----------|
| `TfidfVectorizer` | TF-IDF vectors | Document similarity [3] |
| `LatentDirichletAllocation` | LDA topics | Topic extraction [4] |
| `KMeans` | K-means clustering | Code clustering [6] |
| `DBSCAN` | Density clustering | Outlier detection [7] |
| `AgglomerativeClustering` | Hierarchical | Dendrogram analysis |

---

## 3. Feature Components

### 3.1 Semantic Search (`pmat semantic search`)

**Implementation**: Hybrid BM25 + TF-IDF with RRF fusion

```rust
use trueno_rag::{
    pipeline::RagPipelineBuilder,
    chunk::StructuralChunker,
    fusion::FusionStrategy,
    retrieve::HybridRetriever,
};
use aprender::text::TfidfVectorizer;

pub struct LocalSemanticSearch {
    pipeline: RagPipeline,
    vectorizer: TfidfVectorizer,
}

impl LocalSemanticSearch {
    pub fn search(&self, query: &str, limit: usize) -> Vec<SearchResult> {
        // 1. TF-IDF query vector
        let query_vec = self.vectorizer.transform_single(query);

        // 2. Hybrid retrieval (BM25 + TF-IDF)
        let candidates = self.pipeline.query(query, limit * 3)?;

        // 3. RRF fusion
        let fused = FusionStrategy::RRF { k: 60.0 }.fuse(&candidates);

        // 4. Return top-k
        fused.into_iter().take(limit).collect()
    }
}
```

### 3.2 Code Clustering (`pmat analyze cluster`)

**Implementation**: aprender clustering with trueno-graph importance

```rust
use aprender::cluster::{KMeans, DBSCAN, AgglomerativeClustering};
use trueno_graph::{CsrGraph, pagerank};

pub fn cluster_with_importance(
    documents: &[CodeDocument],
    method: &str,
    k: Option<usize>,
) -> ClusterResult {
    // 1. Build TF-IDF matrix
    let vectorizer = TfidfVectorizer::new()
        .with_max_features(1000)
        .with_min_df(2);
    let dtm = vectorizer.fit_transform(&documents);

    // 2. Build code graph for PageRank importance
    let mut graph = CsrGraph::new();
    for (i, doc) in documents.iter().enumerate() {
        for dep in &doc.dependencies {
            graph.add_edge(NodeId(i as u32), *dep, 1.0)?;
        }
    }
    let importance = pagerank(&graph, 20, 1e-6)?;

    // 3. Cluster with method
    let labels = match method {
        "kmeans" => {
            let k = k.unwrap_or(5);
            KMeans::new(k).fit_predict(&dtm)
        }
        "dbscan" => {
            DBSCAN::new(0.5, 5).fit_predict(&dtm)
        }
        "hierarchical" => {
            let k = k.unwrap_or(5);
            AgglomerativeClustering::new(k).fit_predict(&dtm)
        }
    };

    // 4. Return with importance-weighted centroids
    ClusterResult::new(labels, importance)
}
```

### 3.3 Topic Modeling (`pmat analyze topics`)

**Implementation**: LDA with trueno-graph topic relationships

```rust
use aprender::decomposition::LatentDirichletAllocation;
use trueno_graph::{CsrGraph, louvain};

pub fn extract_topics_with_graph(
    documents: &[CodeDocument],
    num_topics: usize,
) -> TopicResult {
    // 1. TF-IDF for LDA
    let vectorizer = TfidfVectorizer::new()
        .with_max_features(1000);
    let dtm = vectorizer.fit_transform(&documents);

    // 2. LDA topic extraction
    let mut lda = LatentDirichletAllocation::new(num_topics)
        .with_max_iter(50)
        .with_random_state(42);
    lda.fit(&dtm)?;

    // 3. Build topic co-occurrence graph
    let topic_words = lda.topic_words()?;
    let mut topic_graph = CsrGraph::new();
    for i in 0..num_topics {
        for j in (i+1)..num_topics {
            let similarity = cosine_similarity(
                &topic_words.row(i),
                &topic_words.row(j),
            );
            if similarity > 0.1 {
                topic_graph.add_edge(NodeId(i as u32), NodeId(j as u32), similarity)?;
            }
        }
    }

    // 4. Louvain clustering on topics
    let communities = louvain(&topic_graph)?;

    TopicResult {
        topics: lda.topics(),
        communities: communities.assignments,
        vocabulary: vectorizer.vocabulary(),
    }
}
```

---

## 4. Toyota Way Implementation

### 4.1 Jidoka (Built-in Quality)

**Principle**: Stop and fix problems immediately, build quality in

| Practice | Implementation |
|----------|----------------|
| **Andon Cord** | Quality gates block releases if tests fail |
| **Poka-Yoke** | Type system prevents invalid states |
| **In-Station Quality** | Each module has self-contained tests |

```rust
// Poka-Yoke: Type-safe pipeline construction
pub struct RagPipelineBuilder<C, E, R> {
    chunker: Option<C>,
    embedder: Option<E>,  // Now uses TF-IDF, not API
    reranker: Option<R>,
}

impl RagPipelineBuilder<(), (), ()> {
    pub fn new() -> Self { ... }
}

// Cannot build without all components (compile-time guarantee)
impl<C: Chunker, E: Embedder, R: Reranker> RagPipelineBuilder<C, E, R> {
    pub fn build(self) -> Result<RagPipeline, Error> { ... }
}
```

### 4.2 Genchi Genbutsu (Go and See)

**Principle**: Base decisions on firsthand observation

| Practice | Implementation |
|----------|----------------|
| **Dog Food Sprint** | Test on pmat-book, paiml-mcp-agent-toolkit |
| **Real Metrics** | Measure actual search quality, not synthetic |
| **Gemba Walks** | Analyze real user queries from MCP logs |

### 4.3 Kaizen (Continuous Improvement)

**Principle**: Small, incremental improvements

| Sprint | Focus | Metric |
|--------|-------|--------|
| Dog Food | Self-testing | Precision@10 on own codebase |
| Alpha | Internal users | Query latency < 100ms |
| Beta | External users | User satisfaction > 4.0/5.0 |

### 4.4 Heijunka (Level Loading)

**Principle**: Smooth workflow, avoid batching

| Practice | Implementation |
|----------|----------------|
| **Incremental Indexing** | SHA256 hash-based updates |
| **Background Processing** | tokio async indexing |
| **Cache Warming** | Pre-compute popular queries |

---

## 5. Dog Food Sprint

### 5.1 Overview

**Duration**: 1 week
**Goal**: Validate semantic search on PAIML's own codebases
**Principle**: Genchi Genbutsu - test on real systems, not synthetic data

### 5.2 Test Repositories

| Repository | Size | Purpose |
|------------|------|---------|
| `paiml-mcp-agent-toolkit` | 4,243 files | Primary test (this repo) |
| `pmat-book` | ~100 files | Documentation search |
| `trueno-rag` | ~50 files | RAG component search |
| `trueno-graph` | ~50 files | Graph component search |
| `aprender` | ~100 files | ML library search |

### 5.3 Test Scenarios

#### Scenario 1: Code Discovery
```bash
# Find error handling patterns
pmat semantic search "error handling patterns" --language rust

# Expected: Find anyhow, thiserror usage, Result<T, E> patterns
# Metric: Precision@10 >= 0.7
```

#### Scenario 2: Topic Extraction
```bash
# Extract main topics from codebase
pmat analyze topics --num-topics 10

# Expected Topics:
# - CLI parsing and argument handling
# - AST analysis and tree-sitter
# - MCP protocol integration
# - Graph algorithms (PageRank, BFS)
# - Semantic search and embeddings
# Metric: Topic coherence >= 0.6 (PMI-based) [5]
```

#### Scenario 3: Code Clustering
```bash
# Cluster by semantic similarity
pmat analyze cluster --method kmeans --k 8

# Expected Clusters:
# - CLI handlers
# - Service layer
# - MCP integration
# - Test utilities
# - Graph algorithms
# Metric: Silhouette score >= 0.3
```

#### Scenario 4: Cross-Repository Search
```bash
# Search across all PAIML repositories
pmat semantic search "SIMD acceleration" --repos ../trueno,../trueno-graph,../aprender

# Expected: Find trueno SIMD primitives, aprender vectorization
# Metric: Cross-repo recall >= 0.8
```

### 5.4 Success Criteria

| Metric | Target | Measurement |
|--------|--------|-------------|
| Precision@10 | >= 0.70 | Manual relevance judgment |
| Topic Coherence | >= 0.60 | PMI-based coherence |
| Silhouette Score | >= 0.30 | sklearn.metrics |
| Query Latency | < 100ms | p95 latency |
| Indexing Speed | >= 100 files/s | Files per second |
| Memory Usage | < 500MB | Peak RSS |

### 5.5 Dog Food Sprint Schedule

| Day | Activity | Deliverable |
|-----|----------|-------------|
| 1 | Index all repositories | Baseline metrics |
| 2 | Run all test scenarios | Raw results |
| 3 | Analyze failures | Root cause report |
| 4 | Fix critical issues | Patch release |
| 5 | Re-run validation | Final metrics |

### 5.6 Failure Response (Andon Cord)

If any metric fails:

1. **STOP** - Do not proceed to next phase
2. **ANALYZE** - Five Whys root cause analysis
3. **FIX** - Address root cause, not symptoms
4. **VERIFY** - Re-run full test suite
5. **DOCUMENT** - Update specification with learnings

---

## 6. Organizational Intelligence Integration

### 6.1 Overview

Integration with `organizational-intelligence-plugin` provides:

- **Defect Pattern Context**: Search weighted by defect likelihood
- **Team Knowledge Graph**: Find experts for code areas
- **Historical Quality**: Prioritize stable code in search results

### 6.2 Integration Points

```rust
use organizational_intelligence_plugin::{
    DefectPatternAnalyzer,
    TeamKnowledgeGraph,
    QualityHistory,
};

pub struct OrgAwareSemanticSearch {
    semantic: LocalSemanticSearch,
    defect_analyzer: DefectPatternAnalyzer,
    team_graph: TeamKnowledgeGraph,
}

impl OrgAwareSemanticSearch {
    pub fn search_with_context(
        &self,
        query: &str,
        limit: usize,
    ) -> Vec<EnrichedResult> {
        // 1. Basic semantic search
        let results = self.semantic.search(query, limit * 2);

        // 2. Enrich with defect risk
        let enriched: Vec<_> = results.into_iter().map(|r| {
            let defect_risk = self.defect_analyzer.predict_risk(&r.path);
            let experts = self.team_graph.find_experts(&r.path);
            EnrichedResult {
                result: r,
                defect_risk,
                experts,
            }
        }).collect();

        // 3. Re-rank by quality (lower defect risk = higher rank)
        enriched.sort_by(|a, b| {
            a.defect_risk.partial_cmp(&b.defect_risk).unwrap()
        });

        enriched.into_iter().take(limit).collect()
    }
}
```

### 6.3 Team Knowledge Graph

Using `trueno-graph` for team expertise mapping:

```rust
use trueno_graph::{CsrGraph, pagerank, NodeId};

pub struct TeamKnowledgeGraph {
    graph: CsrGraph,
    developer_map: HashMap<String, NodeId>,
    file_map: HashMap<PathBuf, NodeId>,
}

impl TeamKnowledgeGraph {
    /// Find developers who are experts on a file
    pub fn find_experts(&self, file: &Path) -> Vec<Developer> {
        let file_node = self.file_map.get(file)?;

        // Get all developers who contributed
        let contributors = self.graph.incoming_neighbors(*file_node)?;

        // Rank by PageRank (contribution importance)
        let scores = pagerank(&self.graph, 20, 1e-6)?;

        contributors
            .iter()
            .map(|n| (n, scores[n.0 as usize]))
            .sorted_by(|a, b| b.1.partial_cmp(&a.1).unwrap())
            .take(3)
            .map(|(n, _)| self.developer_map.get_by_right(n).unwrap().clone())
            .collect()
    }
}
```

---

## 7. Performance Targets

### 7.1 Latency Requirements

| Operation | Target | Method |
|-----------|--------|--------|
| Semantic search | < 100ms | TF-IDF + BM25 hybrid |
| Topic extraction | < 5s | aprender LDA |
| Clustering | < 3s | aprender K-means |
| Indexing (per file) | < 10ms | Incremental hashing |

### 7.2 Memory Requirements

| Operation | Target | Method |
|-----------|--------|--------|
| Index (per file) | < 100KB | Compressed TF-IDF |
| Search (working set) | < 200MB | LRU cache |
| Clustering | < 500MB | Sparse matrices |

### 7.3 Scalability

| Codebase Size | Index Time | Search Time |
|---------------|------------|-------------|
| 1K files | < 10s | < 50ms |
| 10K files | < 100s | < 100ms |
| 100K files | < 1000s | < 200ms |

---

## 8. Academic Foundation

### 8.1 Peer-Reviewed Citations

This implementation is grounded in 10 peer-reviewed papers:

#### Information Retrieval & Search

[1] **Robertson, S., & Zaragoza, H. (2009).** "The Probabilistic Relevance Framework: BM25 and Beyond." *Foundations and Trends in Information Retrieval*, 3(4), 333-389.
   - **Contribution**: BM25 algorithm for sparse retrieval
   - **Application**: `trueno-rag` hybrid retrieval

[2] **Cormack, G. V., Clarke, C. L., & Buettcher, S. (2009).** "Reciprocal Rank Fusion Outperforms Condorcet and Individual Rank Learning Methods." *SIGIR '09*, 758-759.
   - **Contribution**: RRF algorithm for result fusion
   - **Application**: Combining BM25 + TF-IDF results

[3] **Manning, C. D., Raghavan, P., & Schütze, H. (2008).** "Introduction to Information Retrieval." *Cambridge University Press*.
   - **Contribution**: TF-IDF vectorization foundations
   - **Application**: `aprender::TfidfVectorizer`

#### Topic Modeling

[4] **Blei, D. M., Ng, A. Y., & Jordan, M. I. (2003).** "Latent Dirichlet Allocation." *Journal of Machine Learning Research*, 3, 993-1022.
   - **Contribution**: LDA algorithm
   - **Application**: `aprender::LatentDirichletAllocation`

[5] **Mimno, D., Wallach, H. M., Talley, E., Leenders, M., & McCallum, A. (2011).** "Optimizing Semantic Coherence in Topic Models." *EMNLP '11*, 262-272.
   - **Contribution**: Topic coherence metrics
   - **Application**: Dog food sprint validation

#### Clustering

[6] **MacQueen, J. (1967).** "Some Methods for Classification and Analysis of Multivariate Observations." *Proceedings of the 5th Berkeley Symposium*, 281-297.
   - **Contribution**: K-means algorithm
   - **Application**: `aprender::KMeans`

[7] **Ester, M., Kriegel, H. P., Sander, J., & Xu, X. (1996).** "A Density-Based Algorithm for Discovering Clusters in Large Spatial Databases with Noise." *KDD '96*, 226-231.
   - **Contribution**: DBSCAN algorithm
   - **Application**: `aprender::DBSCAN`

#### Graph Algorithms

[8] **Page, L., Brin, S., Motwani, R., & Winograd, T. (1999).** "The PageRank Citation Ranking: Bringing Order to the Web." *Stanford InfoLab Technical Report*.
   - **Contribution**: PageRank algorithm
   - **Application**: `trueno-graph::pagerank`

[9] **Blondel, V. D., Guillaume, J. L., Lambiotte, R., & Lefebvre, E. (2008).** "Fast Unfolding of Communities in Large Networks." *Journal of Statistical Mechanics*, P10008.
   - **Contribution**: Louvain algorithm
   - **Application**: `trueno-graph::louvain`

#### Code Analysis

[10] **Allamanis, M., Barr, E. T., Devanbu, P., & Sutton, C. (2018).** "A Survey of Machine Learning for Big Code and Naturalness." *ACM Computing Surveys*, 51(4), 1-37.
    - **Contribution**: ML for code analysis patterns
    - **Application**: Code-specific chunking strategies

### 8.2 Why Local Models Over API-Based

| Factor | Local (trueno-rag + aprender) | API-Based (OpenAI) |
|--------|-------------------------------|-------------------|
| **Latency** | 10-50ms | 200-500ms |
| **Cost** | $0 | $0.10-$1/project |
| **Privacy** | Code never leaves machine | Code sent to cloud |
| **Offline** | Works without internet | Requires connectivity |
| **Reproducibility** | Deterministic | Model versions change |

Research by Reimers & Gurevych (2019) shows that hybrid BM25 + dense retrieval achieves competitive results with pure embedding approaches, especially for code search where lexical matching is crucial.

---

## 9. Implementation Plan

### 9.1 Phase 1: Core Integration (Week 1)

| Task | Component | Status |
|------|-----------|--------|
| Add trueno-rag dependency | Cargo.toml | Pending |
| Add trueno-graph dependency | Cargo.toml | Pending |
| Update aprender to 0.14.0 | Cargo.toml | Complete |
| Remove OpenAI dependencies | Multiple | Pending |
| Create LocalSemanticEngine | local_semantic.rs | Complete |

### 9.2 Phase 2: Feature Implementation (Week 2)

| Task | Component | Status |
|------|-----------|--------|
| Hybrid search (BM25 + TF-IDF) | search.rs | Pending |
| trueno-graph PageRank scoring | graph_scoring.rs | Pending |
| Code-specific chunking | chunking.rs | Pending |
| Incremental indexing | indexer.rs | Pending |

### 9.3 Phase 3: Dog Food Sprint (Week 3)

| Task | Metric | Target |
|------|--------|--------|
| Index paiml-mcp-agent-toolkit | Indexing speed | >= 100 files/s |
| Run semantic search tests | Precision@10 | >= 0.70 |
| Run topic extraction tests | Coherence | >= 0.60 |
| Run clustering tests | Silhouette | >= 0.30 |

### 9.4 Phase 4: OIP Integration (Week 4)

| Task | Component | Status |
|------|-----------|--------|
| Defect-aware search ranking | oip_integration.rs | Pending |
| Team knowledge graph | team_graph.rs | Pending |
| Quality history weighting | quality_weight.rs | Pending |

---

## 10. Quality Gates

### 10.1 Pre-Commit Gates (O(1))

```bash
# Cached metric validation
make pre-commit

# Checks:
# - lint duration <= 30s
# - test-fast duration <= 5min
# - binary size <= 50MB
```

### 10.2 CI/CD Gates

```bash
# Full test suite
cargo test --release

# Coverage gate
cargo llvm-cov --release --fail-under 85

# Clippy gate
cargo clippy -- -D warnings

# Benchmark gate
cargo bench --bench semantic_search -- --noplot
```

### 10.3 Release Gates

| Gate | Requirement |
|------|-------------|
| Dog Food Sprint | All metrics pass |
| pmat-book validation | All code examples work |
| Zero SATD | No TODO/FIXME in semantic code |
| Documentation | All public APIs documented |

---

## Appendix A: Removed Dependencies

The following OpenAI-related dependencies are **completely removed**:

```toml
# REMOVED - No longer used
# openai-api = "X.X"
# async-openai = "X.X"
# tiktoken-rs = "X.X"
```

Environment variables no longer required:
- ~~`OPENAI_API_KEY`~~ - Not needed
- ~~`OPENAI_ORG_ID`~~ - Not needed

---

## Appendix B: Migration Guide

### From OpenAI-Based Search

```rust
// OLD (OpenAI-based)
let embeddings = openai.embed(texts).await?;
let results = vector_db.search(query_embedding, limit)?;

// NEW (Local)
let vectorizer = TfidfVectorizer::new().fit(&texts);
let query_vec = vectorizer.transform_single(query);
let results = hybrid_search(query_vec, bm25_index, limit)?;
```

### From External API Clustering

```rust
// OLD (External API)
let clusters = api.cluster(embeddings, k).await?;

// NEW (Local aprender)
let kmeans = KMeans::new(k).fit(&matrix);
let clusters = kmeans.predict(&matrix);
```

---

## Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 2.0.0 | 2025-12-02 | PAIML | Complete OpenAI removal, trueno integration |
| 1.0.0 | 2025-10-01 | PAIML | Initial specification (OpenAI-based) |
