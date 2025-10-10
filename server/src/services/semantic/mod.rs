// Semantic Search Services
// PMAT-SEARCH-001: AST-Aware Code Chunker
// PMAT-SEARCH-002: OpenAI Embeddings Client
// PMAT-SEARCH-003: Turso Vector Database
// PMAT-SEARCH-004: Vector Similarity Search Engine
// PMAT-SEARCH-005: Hybrid Search Engine with RRF
// PMAT-SEARCH-007: K-means Clustering
// PMAT-SEARCH-008: Topic Modeling with LDA

pub mod chunker;
pub mod clustering;
pub mod hybrid_search;
pub mod openai_embeddings;
pub mod search_engine;
pub mod topic_modeling;
pub mod turso_vector_db;

pub use chunker::{chunk_code, CodeChunk, ChunkType, Language};
pub use clustering::{
    Cluster, ClusterFilters, ClusteringEngine, ClusteringMethod, ClusterMember, ClusterResult,
    Dendrogram, DendrogramMerge, Linkage, OutlierPoint,
};
pub use hybrid_search::{HybridSearchEngine, HybridSearchMode, HybridSearchQuery, HybridSearchResult};
pub use openai_embeddings::{EmbeddingResult, OpenAIEmbeddingsClient};
pub use search_engine::{IndexStats, SearchMode, SearchQuery, SemanticSearchEngine};
pub use search_engine::SearchResult; // Primary search result type
pub use topic_modeling::{Topic, TopicChunk, TopicEngine, TopicFilters, TopicResult};
pub use turso_vector_db::{EmbeddingEntry, TursoVectorDB};
