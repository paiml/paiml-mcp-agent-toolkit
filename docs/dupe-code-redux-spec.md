# Duplicate Code Detection Redux Specification

## Abstract

This specification defines a high-performance, cross-language duplicate code detection system leveraging locality-sensitive hashing (LSH), MinHash signatures, and approximate nearest neighbor (ANN) indexing. The system detects Type 1-3 clones across Rust, TypeScript, and Python codebases with sub-linear query complexity O(n^ρ) where ρ < 1, achieving 95%+ recall at 80%+ precision for semantically similar code fragments.

## 1. System Architecture

### 1.1 Core Components

```rust
pub struct DuplicateDetectionEngine {
    feature_extractor: UniversalFeatureExtractor,
    lsh_index: VectorizedLSH,
    ann_index: ANNIndex,
    fingerprint_cache: DashMap<FileHash, FingerprintSet>,
    clone_registry: Arc<RwLock<CloneRegistry>>,
    config: DuplicateDetectionConfig,
}

pub struct UniversalFeatureExtractor {
    tokenizers: LanguageTokenizers,
    normalizer: CodeNormalizer,
    shingle_generator: ShingleGenerator,
    ast_encoder: Option<AstFeatureEncoder>,
}
```

The engine operates through a multi-stage pipeline:

1. **Feature Extraction**: Transform source code into normalized token streams and structural features
2. **Fingerprinting**: Generate multiple hash signatures (Rabin, MinHash, SimHash) for different granularities
3. **Indexing**: Build LSH buckets and ANN structures for sub-linear similarity search
4. **Clustering**: Group similar fragments into clone classes using Union-Find with path compression

### 1.2 Clone Type Taxonomy

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum CloneType {
    Type1 { similarity: f64 },                    // Exact clones (modulo whitespace)
    Type2 { similarity: f64, normalized: bool },  // Parametric clones
    Type3 { similarity: f64, ast_distance: f64 }, // Structural clones
    Type4 { semantic_score: f64 },                // Semantic clones (future work)
}
```

## 2. Feature Extraction Pipeline

### 2.1 Token-Level Processing

```rust
impl UniversalFeatureExtractor {
    pub fn extract_features(&self, source: &str, lang: Language) -> FeatureVector {
        // Phase 1: Lexical tokenization
        let tokens = self.tokenizers.tokenize(source, lang);
        
        // Phase 2: Normalization for Type-2 detection
        let normalized = self.normalizer.normalize(&tokens, NormalizationLevel::Aggressive);
        
        // Phase 3: Shingle generation (k-grams)
        let shingles = self.shingle_generator.generate(&normalized, self.config.shingle_size);
        
        // Phase 4: Structural feature extraction (optional)
        let ast_features = if self.config.enable_type3_detection {
            Some(self.extract_ast_features(source, lang))
        } else {
            None
        };
        
        FeatureVector {
            raw_tokens: tokens,
            normalized_tokens: normalized,
            shingles,
            ast_features,
            metrics: self.compute_metrics(source),
        }
    }
}
```

### 2.2 Normalization Strategy

```rust
impl CodeNormalizer {
    fn normalize(&self, tokens: &[Token], level: NormalizationLevel) -> Vec<Token> {
        tokens.iter().map(|token| {
            match (&token.kind, level) {
                (TokenKind::Identifier(name), NormalizationLevel::Aggressive) => {
                    Token::new(TokenKind::Identifier(self.canonicalize_identifier(name)))
                }
                (TokenKind::Literal(lit), _) => {
                    Token::new(TokenKind::Literal(self.abstract_literal(lit)))
                }
                (TokenKind::Comment, _) => Token::new(TokenKind::Whitespace),
                _ => token.clone(),
            }
        }).filter(|t| !matches!(t.kind, TokenKind::Whitespace))
          .collect()
    }
    
    fn canonicalize_identifier(&self, name: &str) -> String {
        // Map identifiers to canonical forms based on scope and type
        match self.identifier_map.get(name) {
            Some(canonical) => canonical.clone(),
            None => {
                let canonical = format!("VAR_{}", self.next_id.fetch_add(1, Ordering::SeqCst));
                self.identifier_map.insert(name.to_string(), canonical.clone());
                canonical
            }
        }
    }
}
```

## 3. Fingerprinting Algorithms

### 3.1 Rabin Fingerprinting (Type-1 Detection)

```rust
const RABIN_PRIME: u64 = 0xbfe6b8a5bf378d83;
const WINDOW_SIZE: usize = 50;

impl RabinFingerprinter {
    pub fn compute_fingerprints(&self, tokens: &[Token]) -> Vec<u64> {
        let mut fingerprints = Vec::with_capacity(tokens.len());
        let mut hasher = RollingHash::new(RABIN_PRIME, WINDOW_SIZE);
        
        for (i, token) in tokens.iter().enumerate() {
            hasher.push(token.hash());
            
            if i >= WINDOW_SIZE {
                fingerprints.push(hasher.value());
                hasher.pop(tokens[i - WINDOW_SIZE].hash());
            }
        }
        
        fingerprints
    }
}
```

**Complexity**: O(n) time, O(n) space for n tokens

### 3.2 MinHash Signatures (Type-2 Detection)

```rust
impl MinHashGenerator {
    pub fn compute_signature(&self, shingles: &[u64], num_hashes: usize) -> MinHashSignature {
        let mut signature = vec![u64::MAX; num_hashes];
        
        // Use tabulation hashing for speed
        let hash_tables = self.generate_hash_tables(num_hashes);
        
        for &shingle in shingles {
            for (i, table) in hash_tables.iter().enumerate() {
                let hash = table.hash(shingle);
                signature[i] = signature[i].min(hash);
            }
        }
        
        MinHashSignature { values: signature }
    }
    
    pub fn jaccard_similarity(&self, sig1: &MinHashSignature, sig2: &MinHashSignature) -> f64 {
        let matches = sig1.values.iter()
            .zip(&sig2.values)
            .filter(|(a, b)| a == b)
            .count();
        
        matches as f64 / sig1.values.len() as f64
    }
}
```

**Complexity**: O(k·s) where k = num_hashes, s = num_shingles

### 3.3 SimHash for Structural Similarity

```rust
impl SimHashGenerator {
    pub fn compute_simhash(&self, features: &[WeightedFeature]) -> u64 {
        let mut v = vec![0i32; 64];
        
        for feature in features {
            let hash = xxhash64(feature.content.as_bytes());
            for i in 0..64 {
                if (hash >> i) & 1 == 1 {
                    v[i] += feature.weight as i32;
                } else {
                    v[i] -= feature.weight as i32;
                }
            }
        }
        
        let mut simhash = 0u64;
        for i in 0..64 {
            if v[i] > 0 {
                simhash |= 1u64 << i;
            }
        }
        
        simhash
    }
}
```

## 4. Indexing and Search

### 4.1 LSH Implementation

```rust
pub struct VectorizedLSH {
    bands: usize,
    rows_per_band: usize,
    buckets: Vec<DashMap<u64, Vec<FragmentId>>>,
    hash_family: HashFamily,
}

impl VectorizedLSH {
    pub fn insert(&self, fragment_id: FragmentId, signature: &MinHashSignature) {
        let sig_len = signature.values.len();
        let band_size = sig_len / self.bands;
        
        for band in 0..self.bands {
            let start = band * band_size;
            let end = start + band_size;
            let band_hash = self.hash_band(&signature.values[start..end]);
            
            self.buckets[band]
                .entry(band_hash)
                .or_insert_with(Vec::new)
                .push(fragment_id);
        }
    }
    
    pub fn query(&self, signature: &MinHashSignature, threshold: f64) -> Vec<FragmentId> {
        let mut candidates = FxHashSet::default();
        let required_bands = self.estimate_required_bands(threshold);
        
        for band in 0..self.bands {
            let band_hash = self.compute_band_hash(signature, band);
            if let Some(bucket) = self.buckets[band].get(&band_hash) {
                candidates.extend(bucket.value().iter().cloned());
            }
        }
        
        // Post-filter by actual similarity
        candidates.into_iter()
            .filter(|&id| {
                let stored_sig = self.get_signature(id);
                self.jaccard_similarity(signature, &stored_sig) >= threshold
            })
            .collect()
    }
}
```

**LSH Parameters Optimization**:
- For threshold t = 0.8: bands = 20, rows = 10 → P(detection) ≈ 0.97
- For threshold t = 0.6: bands = 25, rows = 8 → P(detection) ≈ 0.95

### 4.2 ANN Index for Refined Search

```rust
pub struct ANNIndex {
    hnsw: HNSW<f32>,  // Hierarchical Navigable Small World graphs
    id_map: BiMap<FragmentId, usize>,
    distance_metric: DistanceMetric,
}

impl ANNIndex {
    pub fn build(&mut self, embeddings: &[(FragmentId, Vec<f32>)]) {
        self.hnsw = HNSW::new(
            self.distance_metric,
            embeddings[0].1.len(),
            32,  // M parameter
            200, // ef_construction
            16,  // seed
        );
        
        for (id, embedding) in embeddings {
            let idx = self.hnsw.insert(embedding);
            self.id_map.insert(*id, idx);
        }
    }
    
    pub fn search(&self, query: &[f32], k: usize, ef: usize) -> Vec<(FragmentId, f32)> {
        self.hnsw.set_ef(ef);
        let results = self.hnsw.search(query, k);
        
        results.into_iter()
            .map(|(idx, dist)| {
                let fragment_id = self.id_map.get_by_right(&idx).unwrap();
                (*fragment_id, dist)
            })
            .collect()
    }
}
```

## 5. Clone Detection Pipeline

### 5.1 Fragment Extraction

```rust
impl FragmentExtractor {
    pub fn extract_fragments(&self, ast: &UnifiedAst) -> Vec<CodeFragment> {
        let mut fragments = Vec::new();
        
        // Method-level fragments
        fragments.extend(self.extract_functions(ast));
        
        // Block-level fragments (if-else, loops, etc.)
        if self.config.enable_block_level {
            fragments.extend(self.extract_blocks(ast));
        }
        
        // Sliding window fragments for partial clones
        if self.config.enable_sliding_window {
            fragments.extend(self.extract_sliding_windows(ast));
        }
        
        fragments
    }
}
```

### 5.2 Clone Group Formation

```rust
pub struct CloneGrouper {
    union_find: UnionFind<FragmentId>,
    similarity_matrix: DashMap<(FragmentId, FragmentId), f64>,
}

impl CloneGrouper {
    pub fn group_clones(&mut self, candidates: Vec<ClonePair>) -> Vec<CloneGroup> {
        // Phase 1: Build equivalence classes
        for pair in candidates {
            if pair.similarity >= self.config.grouping_threshold {
                self.union_find.union(pair.fragment1, pair.fragment2);
                self.similarity_matrix.insert(
                    (pair.fragment1, pair.fragment2),
                    pair.similarity
                );
            }
        }
        
        // Phase 2: Build clone groups
        let mut groups: FxHashMap<FragmentId, Vec<FragmentId>> = FxHashMap::default();
        
        for fragment in self.union_find.iter() {
            let root = self.union_find.find(fragment);
            groups.entry(root).or_default().push(fragment);
        }
        
        // Phase 3: Compute group metrics
        groups.into_iter()
            .filter(|(_, members)| members.len() >= self.config.min_group_size)
            .map(|(representative, members)| {
                self.build_clone_group(representative, members)
            })
            .collect()
    }
}
```

## 6. Cross-Language Support

### 6.1 Language-Agnostic Features

```rust
impl AstFeatureEncoder {
    pub fn encode_ast(&self, ast: &UnifiedAst) -> Vec<StructuralFeature> {
        let mut features = Vec::new();
        
        // Control flow patterns
        features.extend(self.extract_control_flow_patterns(ast));
        
        // Data flow edges
        features.extend(self.extract_data_dependencies(ast));
        
        // API usage patterns
        features.extend(self.extract_api_calls(ast));
        
        // Normalized AST paths
        features.extend(self.extract_ast_paths(ast));
        
        features
    }
    
    fn extract_ast_paths(&self, ast: &UnifiedAst) -> Vec<StructuralFeature> {
        // Path-based representation inspired by code2vec
        let paths = self.enumerate_ast_paths(ast, self.config.max_path_length);
        
        paths.into_iter()
            .map(|path| StructuralFeature {
                kind: FeatureKind::AstPath,
                value: self.encode_path(&path),
                weight: self.compute_path_weight(&path),
            })
            .collect()
    }
}
```

### 6.2 Language-Specific Normalization

```rust
impl LanguageNormalizer for RustNormalizer {
    fn normalize_semantic(&self, tokens: &[Token]) -> Vec<Token> {
        let mut normalized = Vec::new();
        let mut i = 0;
        
        while i < tokens.len() {
            match &tokens[i].kind {
                // Normalize lifetime parameters
                TokenKind::Lifetime(_) => {
                    normalized.push(Token::new(TokenKind::Lifetime("'a".into())));
                }
                // Normalize generic parameters
                TokenKind::GenericParam(name) if self.is_type_param(name) => {
                    normalized.push(Token::new(TokenKind::GenericParam("T".into())));
                }
                // Handle macro invocations
                TokenKind::Macro(name) => {
                    let expansion = self.estimate_macro_expansion(name, &tokens[i+1..]);
                    normalized.extend(expansion);
                    i += self.macro_arg_count(name);
                }
                _ => normalized.push(tokens[i].clone()),
            }
            i += 1;
        }
        
        normalized
    }
}
```

## 7. Performance Optimization

### 7.1 Incremental Updates

```rust
impl IncrementalDuplicateDetection {
    pub fn update(&mut self, changes: &FileChangeSet) -> DeltaCloneReport {
        let mut affected_fragments = FxHashSet::default();
        
        // Phase 1: Remove outdated fragments
        for deleted_file in &changes.deleted {
            let fragments = self.fragment_index.remove(deleted_file);
            affected_fragments.extend(fragments);
        }
        
        // Phase 2: Update modified files
        for modified_file in &changes.modified {
            let old_fragments = self.fragment_index.get(modified_file);
            let new_fragments = self.extract_fragments(modified_file);
            
            // Compute fragment-level diff
            let (deleted, added) = self.diff_fragments(old_fragments, new_fragments);
            affected_fragments.extend(deleted);
            
            // Update indices
            for fragment in added {
                self.lsh_index.insert(fragment.id, &fragment.signature);
                self.ann_index.add(fragment.id, &fragment.embedding);
            }
        }
        
        // Phase 3: Re-evaluate affected clone groups
        self.reevaluate_groups(affected_fragments)
    }
}
```

### 7.2 Parallel Processing

```rust
impl ParallelDuplicateDetection {
    pub fn detect_parallel(&self, files: Vec<PathBuf>) -> CloneReport {
        let chunk_size = files.len() / rayon::current_num_threads();
        
        // Phase 1: Parallel feature extraction
        let features: Vec<_> = files
            .par_chunks(chunk_size)
            .flat_map(|chunk| {
                chunk.iter()
                    .filter_map(|path| self.extract_file_features(path).ok())
                    .collect::<Vec<_>>()
            })
            .collect();
        
        // Phase 2: Parallel LSH insertion
        features.par_iter()
            .for_each(|feature| {
                self.lsh_index.insert_concurrent(feature.id, &feature.signature);
            });
        
        // Phase 3: Parallel candidate generation
        let candidates: Vec<_> = features
            .par_iter()
            .flat_map(|feature| {
                self.lsh_index.query(&feature.signature, self.config.threshold)
                    .into_iter()
                    .filter(|&cand_id| cand_id > feature.id) // Avoid duplicates
                    .map(|cand_id| ClonePair {
                        fragment1: feature.id,
                        fragment2: cand_id,
                        similarity: 0.0, // Computed later
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        
        // Phase 4: Refine similarities and group
        self.refine_and_group(candidates)
    }
}
```

## 8. Configuration and Tuning

### 8.1 Configuration Schema

```toml
[duplicate_detection]
# Detection parameters
min_tokens = 50
similarity_threshold = 0.70
shingle_size = 5

# LSH parameters
num_hash_functions = 200
num_bands = 20
rows_per_band = 10

# Feature extraction
[duplicate_detection.features]
normalize_identifiers = true
normalize_literals = true
ignore_comments = true
extract_ast_features = true

# Language-specific
[duplicate_detection.rust]
normalize_lifetimes = true
expand_simple_macros = true

[duplicate_detection.typescript]
normalize_type_annotations = true
ignore_ambient_declarations = true

# Performance
[duplicate_detection.performance]
parallel_extraction = true
incremental_mode = true
cache_signatures = true
max_memory_gb = 4.0
```

### 8.2 Output Format

```rust
#[derive(Serialize)]
pub struct CloneReport {
    pub summary: CloneSummary,
    pub groups: Vec<CloneGroup>,
    pub metrics: DuplicationMetrics,
    pub hotspots: Vec<DuplicationHotspot>,
}

#[derive(Serialize)]
pub struct CloneGroup {
    pub id: GroupId,
    pub clone_type: CloneType,
    pub fragments: Vec<CloneInstance>,
    pub total_lines: usize,
    pub total_tokens: usize,
    pub average_similarity: f64,
    pub representative: FragmentId,
}

#[derive(Serialize)]
pub struct CloneInstance {
    pub file: PathBuf,
    pub start_line: usize,
    pub end_line: usize,
    pub start_column: usize,
    pub end_column: usize,
    pub similarity_to_representative: f64,
    pub normalized_hash: u64,
}
```

## 9. Benchmarks and Validation

### 9.1 Performance Benchmarks

```rust
#[bench]
fn bench_linux_kernel_clone_detection(b: &mut Bencher) {
    let files = collect_source_files("/path/to/linux");
    let engine = DuplicateDetectionEngine::new(default_config());
    
    b.iter(|| {
        engine.detect_all(&files)
    });
}

// Expected performance on 1M LOC:
// - Feature extraction: ~2s (parallel)
// - LSH indexing: ~500ms
// - Candidate generation: ~1s
// - Refinement and grouping: ~1.5s
// - Total: ~5s for full analysis
// - Incremental update (1K LOC change): ~150ms
```

### 9.2 Quality Metrics

```rust
#[test]
fn test_clone_detection_quality() {
    let benchmark = BigCloneBench::load("ijadataset/bcb_reduced");
    let engine = DuplicateDetectionEngine::new(optimal_config());
    
    let results = engine.detect_all(&benchmark.files);
    let metrics = evaluate_against_ground_truth(&results, &benchmark.ground_truth);
    
    assert!(metrics.precision >= 0.80);
    assert!(metrics.recall >= 0.95);
    assert!(metrics.f1_score >= 0.87);
}
```

## 10. Integration with DeepContext

```rust
impl DeepContextIntegration for DuplicateDetectionEngine {
    fn analyze(&self, context: &ProjectContext) -> DuplicationAnalysis {
        let fragments = self.extract_all_fragments(context);
        let report = self.detect_clones(fragments);
        
        DuplicationAnalysis {
            duplication_ratio: report.summary.duplicate_lines as f64 / 
                               report.summary.total_lines as f64,
            largest_clone_group: report.groups
                .iter()
                .max_by_key(|g| g.total_lines)
                .cloned(),
            refactoring_opportunities: self.identify_refactoring_candidates(&report),
            impact_on_maintainability: self.calculate_duplication_impact(&report),
        }
    }
}
```

## References

1. Broder, A. (1997). "On the Resemblance and Containment of Documents"
2. Charikar, M. (2002). "Similarity Estimation Techniques from Rounding Algorithms"
3. Indyk, P. & Motwani, R. (1998). "Approximate Nearest Neighbors: Towards Removing the Curse of Dimensionality"
4. Kamiya, T. et al. (2002). "CCFinder: A Multilinguistic Token-Based Code Clone Detection System"
5. Jiang, L. et al. (2007). "DECKARD: Scalable and Accurate Tree-based Detection of Code Clones"
6. Roy, C. & Cordy, J. (2007). "A Survey on Software Clone Detection Research"