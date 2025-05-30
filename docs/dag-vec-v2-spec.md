# DAG+Vectorized Architecture Specification v2

## Abstract

This specification defines a unified code intelligence platform combining hierarchical DAG representation with vectorized computation for comprehensive defect detection, code quality analysis, and architectural insights. The design leverages SIMD acceleration, language-agnostic AST analysis, and graph analytics to provide sub-millisecond query response while maintaining binary size under 15MB.

## Core Architecture

### 1. Unified AST Representation

```rust
#[repr(C, align(32))]
pub struct UnifiedAstNode {
    // Core node data - 32 bytes aligned
    kind: AstKind,              // 2 bytes - language-agnostic
    lang: Language,             // 1 byte
    flags: NodeFlags,           // 1 byte
    parent: NodeKey,            // 4 bytes
    first_child: NodeKey,       // 4 bytes
    next_sibling: NodeKey,      // 4 bytes
    source_range: Range<u32>,   // 8 bytes
    
    // Semantic data - 32 bytes
    semantic_hash: u64,         // 8 bytes - content hash
    structural_hash: u64,       // 8 bytes - structure hash
    name_vector: u64,           // 8 bytes - packed name embedding
    metadata: NodeMetadata,     // 8 bytes - union type
} // Total: 64 bytes per node (cache line aligned)

#[repr(C)]
pub enum AstKind {
    // Universal constructs
    Function(FunctionKind),
    Class(ClassKind),
    Variable(VarKind),
    Import(ImportKind),
    Expression(ExprKind),
    Statement(StmtKind),
    Type(TypeKind),
    Module(ModuleKind),
}

pub struct AstDag {
    // Columnar storage for SIMD operations
    nodes: ColumnStore<UnifiedAstNode>,
    
    // Language-specific parsers
    parsers: LanguageParsers,
    
    // Incremental update tracking
    dirty_nodes: RoaringBitmap,
    generation: AtomicU32,
}
```

### 2. Vectorized Duplicate Detection Engine

```rust
pub struct DuplicateDetector {
    // Type-1: Exact clones (Rabin fingerprints)
    exact_hashes: RadixMap<u64, Vec<NodeKey>>,
    
    // Type-2: Renamed clones (α-normalized)
    alpha_hashes: RadixMap<u64, Vec<NodeKey>>,
    
    // Type-3: Gapped clones (MinHash signatures)
    minhash_index: VectorizedLSH<128>,
    
    // Type-4: Semantic clones (AST embeddings)
    semantic_index: ANNIndex<384>, // 384-dim embeddings
    
    // Cross-language clone detection
    universal_features: UniversalFeatureExtractor,
}

impl DuplicateDetector {
    pub fn detect_all_clones(&self) -> CloneReport {
        // Parallel detection across all types
        let (exact, renamed, gapped, semantic) = rayon::join4(
            || self.detect_exact_clones(),
            || self.detect_renamed_clones(),
            || self.detect_gapped_clones(),
            || self.detect_semantic_clones()
        );
        
        // Merge and rank by confidence
        self.merge_clone_groups(exact, renamed, gapped, semantic)
    }
    
    #[inline]
    pub fn compute_ast_embedding(&self, nodes: &[UnifiedAstNode]) -> [f32; 384] {
        let mut embedding = [0.0f32; 384];
        
        // Extract structural features (128 dims)
        self.extract_structural_features(&mut embedding[0..128], nodes);
        
        // Extract semantic features (128 dims)
        self.extract_semantic_features(&mut embedding[128..256], nodes);
        
        // Extract contextual features (128 dims)
        self.extract_contextual_features(&mut embedding[256..384], nodes);
        
        // L2 normalize for cosine similarity
        let norm = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        embedding.iter_mut().for_each(|x| *x /= norm);
        
        embedding
    }
}
```

### 3. Dead Code Detection with Cross-Reference Analysis

```rust
pub struct DeadCodeAnalyzer {
    // Multi-level reachability
    reachability: HierarchicalBitSet,
    
    // Cross-language reference tracking
    references: CrossLangReferenceGraph,
    
    // Dynamic dispatch resolution
    vtable_analysis: VTableResolver,
    
    // Test coverage integration
    coverage_map: Option<CoverageData>,
}

impl DeadCodeAnalyzer {
    pub fn analyze(&mut self) -> DeadCodeReport {
        // Phase 1: Build reference graph from AST
        self.build_reference_graph();
        
        // Phase 2: Resolve dynamic dispatch
        self.resolve_dynamic_calls();
        
        // Phase 3: Mark reachable from entry points
        self.mark_reachable_vectorized();
        
        // Phase 4: Classify dead code by type
        self.classify_dead_code()
    }
    
    #[inline]
    unsafe fn mark_reachable_vectorized(&mut self) {
        let mut changed = true;
        let reachable = self.reachability.as_mut_slice();
        
        while changed {
            changed = false;
            
            // Process 256 nodes at a time with AVX2
            for chunk in reachable.chunks_mut(32) { // 32 * 8 = 256 bits
                let old = _mm256_loadu_si256(chunk.as_ptr() as *const __m256i);
                
                // Propagate reachability through edges
                let mut new = old;
                for &edge in &self.references.edges_for_chunk(chunk) {
                    let target = _mm256_loadu_si256(
                        reachable.as_ptr().add(edge.target_chunk) as *const __m256i
                    );
                    new = _mm256_or_si256(new, target);
                }
                
                if _mm256_movemask_epi8(_mm256_cmpeq_epi8(old, new)) != -1 {
                    _mm256_storeu_si256(chunk.as_mut_ptr() as *mut __m256i, new);
                    changed = true;
                }
            }
        }
    }
}
```

### 4. Name Similarity with Vector Embeddings

```rust
pub struct NameSimilarityEngine {
    // Character-level embeddings
    char_embeddings: CharEmbedding,
    
    // Subword tokenization
    bpe_tokenizer: BPETokenizer,
    
    // Learned name embeddings (from code2vec style training)
    name_embeddings: EmbeddingMatrix<256>,
    
    // Approximate nearest neighbor index
    ann_index: HNSWIndex<256>,
    
    // Phonetic encoding for typo detection
    soundex_index: DoubleMetaphone,
}

impl NameSimilarityEngine {
    pub fn find_similar_names(&self, query: &str, k: usize) -> Vec<SimilarName> {
        // Compute query embedding
        let query_emb = self.embed_name(query);
        
        // Parallel search strategies
        let (vector_similar, edit_similar, phonetic_similar) = rayon::join3(
            || self.ann_index.search(&query_emb, k * 2),
            || self.edit_distance_search(query, k),
            || self.soundex_index.find_similar(query)
        );
        
        // Ensemble ranking with learned weights
        self.ensemble_rank(vector_similar, edit_similar, phonetic_similar, k)
    }
    
    fn embed_name(&self, name: &str) -> [f32; 256] {
        let mut embedding = [0.0f32; 256];
        
        // Character trigrams (64 dims)
        self.char_trigram_features(&mut embedding[0..64], name);
        
        // Subword tokens (64 dims)
        self.bpe_features(&mut embedding[64..128], name);
        
        // Semantic features from learned embeddings (128 dims)
        self.semantic_features(&mut embedding[128..256], name);
        
        embedding
    }
}
```

### 5. Composite Defect Scoring

```rust
pub struct DefectPredictor {
    // Individual analyzers
    complexity: ComplexityAnalyzer,
    churn: ChurnAnalyzer,
    duplication: DuplicateDetector,
    coupling: CouplingAnalyzer,
    
    // Learned weights from historical defect data
    weights: DefectWeights,
    
    // Feature normalization parameters
    normalizer: FeatureNormalizer,
}

#[derive(Debug, Clone)]
pub struct DefectScore {
    pub entity: EntityRef,
    pub total_score: f32,
    pub components: ScoreComponents,
    pub confidence: f32,
}

#[derive(Debug, Clone)]
pub struct ScoreComponents {
    pub complexity: f32,      // McCabe + Cognitive
    pub churn: f32,          // Commit frequency * change size
    pub duplication: f32,    // Clone coverage percentage
    pub coupling: f32,       // Afferent + Efferent coupling
    pub name_quality: f32,   // Naming convention adherence
    pub test_coverage: f32,  // Inverse correlation with defects
}

impl DefectPredictor {
    pub fn predict_defects(&self) -> Vec<DefectScore> {
        let entities = self.collect_all_entities();
        
        // Vectorized feature extraction
        let features = self.extract_features_vectorized(&entities);
        
        // Apply learned model
        let scores = self.apply_model_vectorized(&features);
        
        // Sort by defect probability
        let mut results: Vec<DefectScore> = entities.iter()
            .zip(scores.iter())
            .map(|(entity, &score)| DefectScore {
                entity: entity.clone(),
                total_score: score,
                components: self.decompose_score(entity, score),
                confidence: self.calculate_confidence(entity),
            })
            .collect();
        
        results.sort_by(|a, b| b.total_score.partial_cmp(&a.total_score).unwrap());
        results
    }
    
    #[inline]
    fn apply_model_vectorized(&self, features: &FeatureMatrix) -> Vec<f32> {
        // SIMD-accelerated matrix multiplication
        let mut scores = vec![0.0f32; features.rows()];
        
        for i in (0..features.rows()).step_by(8) {
            let feat = _mm256_loadu_ps(features.row(i));
            let weights = _mm256_loadu_ps(&self.weights.as_slice());
            
            // Weighted sum with learned coefficients
            let score = _mm256_dp_ps(feat, weights, 0xFF);
            _mm256_storeu_ps(&mut scores[i], score);
        }
        
        scores
    }
}
```

### 6. Graph Analytics Engine

```rust
pub struct GraphAnalytics {
    // Adjacency representation for graph algorithms
    adjacency: CompressedSparseRow<NodeKey>,
    
    // Pre-computed centrality measures
    centrality_cache: CentralityCache,
    
    // PageRank with personalization
    pagerank: PersonalizedPageRank,
}

#[derive(Debug, Clone)]
pub struct GraphMetrics {
    pub degree_centrality: f32,
    pub betweenness_centrality: f32,
    pub closeness_centrality: f32,
    pub eigenvector_centrality: f32,
    pub pagerank: f32,
    pub clustering_coefficient: f32,
    pub strongly_connected_component: ComponentId,
}

impl GraphAnalytics {
    pub fn compute_metrics(&mut self, graph: &DependencyGraph) -> HashMap<NodeKey, GraphMetrics> {
        // Parallel computation of different metrics
        let (degree, between, close, eigen) = rayon::join4(
            || self.degree_centrality_vectorized(graph),
            || self.betweenness_centrality_parallel(graph),
            || self.closeness_centrality_gpu(graph),
            || self.eigenvector_centrality_lanczos(graph)
        );
        
        // PageRank with damping factor 0.85
        let pagerank = self.pagerank.compute(graph, 0.85, 1e-6);
        
        // Local clustering coefficients
        let clustering = self.clustering_coefficients_vectorized(graph);
        
        // Tarjan's SCC algorithm
        let components = self.find_sccs(graph);
        
        self.merge_metrics(degree, between, close, eigen, pagerank, clustering, components)
    }
    
    #[inline]
    fn degree_centrality_vectorized(&self, graph: &DependencyGraph) -> Vec<f32> {
        let n = graph.nodes.len();
        let mut centrality = vec![0.0f32; n];
        
        // Vectorized degree computation
        for i in (0..n).step_by(8) {
            let degrees = unsafe {
                _mm256_set_ps(
                    self.adjacency.degree(i + 7) as f32,
                    self.adjacency.degree(i + 6) as f32,
                    self.adjacency.degree(i + 5) as f32,
                    self.adjacency.degree(i + 4) as f32,
                    self.adjacency.degree(i + 3) as f32,
                    self.adjacency.degree(i + 2) as f32,
                    self.adjacency.degree(i + 1) as f32,
                    self.adjacency.degree(i) as f32,
                )
            };
            
            let normalized = _mm256_div_ps(degrees, _mm256_set1_ps((n - 1) as f32));
            unsafe {
                _mm256_storeu_ps(&mut centrality[i], normalized);
            }
        }
        
        centrality
    }
}
```

### 7. Unified Query Interface

```rust
pub struct CodeIntelligence {
    dag: Arc<AstDag>,
    duplicate: Arc<DuplicateDetector>,
    deadcode: Arc<DeadCodeAnalyzer>,
    names: Arc<NameSimilarityEngine>,
    defects: Arc<DefectPredictor>,
    graph: Arc<GraphAnalytics>,
    cache: Arc<UnifiedCache>,
}

impl CodeIntelligence {
    pub async fn analyze_comprehensive(&self, req: AnalysisRequest) -> AnalysisReport {
        let cache_key = req.cache_key();
        
        if let Some(cached) = self.cache.get(&cache_key).await {
            return cached;
        }
        
        // Parallel analysis pipeline
        let futures = vec![
            tokio::spawn({
                let dup = self.duplicate.clone();
                async move { dup.detect_all_clones() }
            }),
            tokio::spawn({
                let dead = self.deadcode.clone();
                async move { dead.analyze() }
            }),
            tokio::spawn({
                let defect = self.defects.clone();
                async move { defect.predict_defects() }
            }),
            tokio::spawn({
                let graph = self.graph.clone();
                let dag = self.dag.clone();
                async move { graph.compute_metrics(&dag.to_dependency_graph()) }
            }),
        ];
        
        let results = futures::future::join_all(futures).await;
        
        let report = AnalysisReport {
            duplicates: results[0].as_ref().unwrap().clone(),
            dead_code: results[1].as_ref().unwrap().clone(),
            defect_scores: results[2].as_ref().unwrap().clone(),
            graph_metrics: results[3].as_ref().unwrap().clone(),
            timestamp: Utc::now(),
        };
        
        self.cache.put(cache_key, report.clone()).await;
        report
    }
}
```

## Performance Characteristics

### Memory Layout

```
Total memory usage for 100K files, 10M AST nodes:
- Unified AST DAG: 10M * 64 bytes = 640 MB
- Duplicate index: 100K * 2KB = 200 MB
- Name embeddings: 500K * 1KB = 500 MB
- Graph analytics: 100K * 128 bytes = 12.8 MB
- Caches and indexes: ~200 MB
- Total: ~1.5 GB resident (with mmap: 300MB working set)
```

### Query Latencies

| Analysis Type | Cold (ms) | Warm (μs) | Complexity |
|--------------|-----------|-----------|------------|
| Exact duplicate detection | 5 | 50 | O(n) hash |
| Semantic clone detection | 100 | 500 | O(log n) ANN |
| Dead code analysis | 50 | 200 | O(V + E) |
| Name similarity (top-10) | 2 | 20 | O(log n) |
| Defect prediction | 20 | 100 | O(n) vectorized |
| PageRank computation | 200 | 1000 | O(k(V + E)) |
| Full analysis report | 300 | 2000 | O(n log n) |

### Binary Size Impact

```
Component sizes (release build with LTO):
- Core analyzer: 2.8 MB
- SIMD kernels: 0.6 MB
- Duplicate detection: 1.8 MB
- Graph analytics: 1.2 MB
- Name similarity: 1.4 MB
- ML models: 0.8 MB
- Total: ~8.6 MB
```

## Example Usage

```bash
paiml-mcp-agent-toolkit analyze comprehensive --format json

{
  "defect_predictions": [
    {
      "entity": "src/parser/expression.rs::parse_complex_expression",
      "score": 0.89,
      "components": {
        "complexity": 0.82,
        "churn": 0.91,
        "duplication": 0.76,
        "coupling": 0.88,
        "name_quality": 0.45,
        "test_coverage": 0.12
      },
      "recommendation": "High defect risk - consider refactoring"
    }
  ],
  "duplicates": {
    "type_3_clones": [
      {
        "similarity": 0.87,
        "locations": [
          "src/analyzer/type_check.rs:142-203",
          "src/validator/semantic.rs:89-147"
        ],
        "refactoring_effort": "3.2 hours"
      }
    ]
  },
  "graph_insights": {
    "architectural_hotspots": [
      {
        "module": "src/core/engine.rs",
        "pagerank": 0.124,
        "betweenness": 0.891,
        "recommendation": "Critical architectural component - high coupling"
      }
    ]
  }
}
```

## Implementation Milestones

1. **Phase 1**: Unified AST representation + language parsers
2. **Phase 2**: Vectorized duplicate detection
3. **Phase 3**: Dead code analysis with cross-references
4. **Phase 4**: Name similarity with embeddings
5. **Phase 5**: Composite defect scoring
6. **Phase 6**: Graph analytics integration

## Conclusion

This architecture achieves comprehensive code intelligence through:
- Unified AST representation enabling cross-language analysis
- SIMD-accelerated algorithms for all compute-intensive operations
- Machine learning integration for defect prediction
- Graph-theoretic insights into code architecture
- Sub-second analysis of million-line codebases

The system provides actionable insights by combining multiple signals (complexity, churn, duplication, coupling) into a unified defect prediction model, while maintaining the performance characteristics necessary for interactive use.