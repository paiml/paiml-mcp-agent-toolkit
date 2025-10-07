# PMAT-7009: Pattern Learning System

**Status**: 🚀 TODO
**Priority**: P1 - High
**Complexity**: Medium-High
**Estimated Duration**: 5-7 days
**Sprint**: 24
**Created**: 2025-10-07

---

## Objective

Implement a pattern learning system that stores and learns from historical analysis results, improving PMAT's predictions and suggestions over time.

**Key Insight**: Transform PMAT from stateless analyzer into a learning system that compounds intelligence with every analysis.

---

## Background

### Current State
- Analysis results generated per-run (stateless)
- ML mutation predictor uses static training data (75-95% accuracy)
- No persistence of insights across runs
- Memory manager exists (`server/src/services/memory_manager.rs`) but unused for learning

### Problem
Every analysis starts from scratch:
- No memory of past complexity patterns
- Can't learn which refactorings worked
- Can't track SATD evolution
- Can't correlate patterns across projects

### Desired Solution
Pattern learning enables:
- "Functions with similar structure had 85% mutation survival rate"
- "Teams using pattern X reduced complexity by refactoring Y"
- "TODO comments in async code persist 3x longer"
- "This pattern appears in 12 other analyzed projects"

---

## Scope

### Core Features

**1. Pattern Storage**
- Store analysis results with extracted features
- SQLite backend (using existing `rusqlite` dependency)
- Feature vectors for similarity matching
- Time-series tracking for evolution

**2. Pattern Types**
```rust
pub enum PatternType {
    Complexity {
        avg_cc: f64,
        avg_cognitive: f64,
        ast_features: Vec<f64>,
    },
    Mutation {
        operator: String,
        survival_rate: f64,
        context: MutationContext,
    },
    SATD {
        category: String,
        keywords: Vec<String>,
        resolution_time: Option<Duration>,
    },
    DeadCode {
        reason: String,
        dependencies: Vec<String>,
    },
    Refactoring {
        pattern_before: Vec<f64>,
        action: RefactoringAction,
        improvement: f64,
    },
}
```

**3. Similarity Matching**
- Cosine similarity on feature vectors
- Configurable similarity threshold (default: 0.85)
- Return top-K similar patterns

**4. Integration Points**
- ML mutation predictor (improve accuracy)
- Complexity analyzer (suggest refactorings)
- SATD detector (track evolution)
- RefactoringAdvisor sub-agent

---

## Implementation Plan

### Phase 1: Storage Layer (Day 1-2)

**1.1 Pattern Storage Schema**
```sql
CREATE TABLE patterns (
    id TEXT PRIMARY KEY,
    pattern_type TEXT NOT NULL,
    features BLOB NOT NULL,          -- Serialized feature vector
    context JSON NOT NULL,            -- Full pattern context
    project_id TEXT,
    file_path TEXT,
    function_name TEXT,
    confidence REAL DEFAULT 1.0,
    seen_count INTEGER DEFAULT 1,
    created_at INTEGER NOT NULL,
    last_seen_at INTEGER NOT NULL
);

CREATE TABLE pattern_outcomes (
    id TEXT PRIMARY KEY,
    pattern_id TEXT NOT NULL,
    action_taken TEXT,
    improvement REAL,
    timestamp INTEGER NOT NULL,
    FOREIGN KEY (pattern_id) REFERENCES patterns(id)
);

CREATE TABLE pattern_similarity (
    pattern_a TEXT NOT NULL,
    pattern_b TEXT NOT NULL,
    similarity REAL NOT NULL,
    computed_at INTEGER NOT NULL,
    PRIMARY KEY (pattern_a, pattern_b)
);

CREATE INDEX idx_pattern_type ON patterns(pattern_type);
CREATE INDEX idx_pattern_project ON patterns(project_id);
CREATE INDEX idx_pattern_confidence ON patterns(confidence);
CREATE INDEX idx_similarity_score ON pattern_similarity(similarity);
```

**1.2 Pattern Storage Service**
- New file: `server/src/services/learning/storage.rs`
```rust
pub struct PatternStorage {
    db: Arc<Mutex<rusqlite::Connection>>,
}

impl PatternStorage {
    pub fn new(db_path: &Path) -> Result<Self>;

    pub async fn store_pattern(&self, pattern: CodePattern) -> Result<Uuid>;

    pub async fn find_similar(
        &self,
        features: &[f64],
        pattern_type: PatternType,
        threshold: f64,
        limit: usize,
    ) -> Result<Vec<SimilarPattern>>;

    pub async fn get_pattern(&self, id: Uuid) -> Result<Option<CodePattern>>;

    pub async fn update_seen_count(&self, id: Uuid) -> Result<()>;

    pub async fn record_outcome(
        &self,
        pattern_id: Uuid,
        outcome: PatternOutcome,
    ) -> Result<()>;
}
```

### Phase 2: Feature Extraction (Day 3-4)

**2.1 Feature Extractor**
- New file: `server/src/services/learning/features.rs`
```rust
pub struct FeatureExtractor;

impl FeatureExtractor {
    /// Extract features from complexity analysis
    pub fn extract_complexity_features(
        ast: &NodeRef,
        metrics: &ComplexityMetrics,
    ) -> Vec<f64> {
        vec![
            metrics.cyclomatic as f64,
            metrics.cognitive as f64,
            count_branches(ast) as f64,
            count_loops(ast) as f64,
            nesting_depth(ast) as f64,
            lines_of_code(ast) as f64,
            parameter_count(ast) as f64,
            // ... 20+ features total
        ]
    }

    /// Extract features from mutation context
    pub fn extract_mutation_features(
        mutant: &Mutant,
        context: &MutationContext,
    ) -> Vec<f64> {
        vec![
            operator_type_to_numeric(&mutant.operator),
            context.complexity.cyclomatic as f64,
            context.nesting_depth as f64,
            context.has_loops as f64,
            context.has_error_handling as f64,
            // ... matches ML predictor features
        ]
    }

    /// Extract features from SATD
    pub fn extract_satd_features(
        satd: &SATDInstance,
        context: &CodeContext,
    ) -> Vec<f64> {
        vec![
            category_to_numeric(&satd.category),
            satd.priority as f64,
            context.function_complexity as f64,
            days_since_creation(satd) as f64,
            // ...
        ]
    }
}
```

**2.2 Similarity Calculator**
```rust
pub struct SimilarityCalculator;

impl SimilarityCalculator {
    /// Cosine similarity between feature vectors
    pub fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
        assert_eq!(a.len(), b.len());
        let dot_product: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let mag_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
        let mag_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
        dot_product / (mag_a * mag_b)
    }

    /// Find K nearest neighbors
    pub fn find_knn(
        query: &[f64],
        patterns: &[CodePattern],
        k: usize,
    ) -> Vec<(CodePattern, f64)> {
        let mut scores: Vec<_> = patterns
            .iter()
            .map(|p| {
                let sim = Self::cosine_similarity(query, &p.features);
                (p.clone(), sim)
            })
            .collect();

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scores.into_iter().take(k).collect()
    }
}
```

### Phase 3: Integration (Day 5-6)

**3.1 Pattern Learning Service**
- New file: `server/src/services/learning/mod.rs`
```rust
pub struct PatternLearningService {
    storage: Arc<PatternStorage>,
    extractor: FeatureExtractor,
    similarity_threshold: f64,
}

impl PatternLearningService {
    pub fn new(storage: Arc<PatternStorage>) -> Self;

    /// Learn from complexity analysis
    pub async fn learn_complexity_pattern(
        &self,
        ast: &NodeRef,
        metrics: &ComplexityMetrics,
        context: &CodeContext,
    ) -> Result<PatternInsights> {
        // Extract features
        let features = self.extractor.extract_complexity_features(ast, metrics);

        // Find similar patterns
        let similar = self.storage
            .find_similar(&features, PatternType::Complexity, self.similarity_threshold, 10)
            .await?;

        // Store new pattern
        let pattern = CodePattern {
            id: Uuid::new_v4(),
            pattern_type: PatternType::Complexity {
                avg_cc: metrics.cyclomatic as f64,
                avg_cognitive: metrics.cognitive as f64,
                ast_features: features.clone(),
            },
            features,
            context: json!(context),
            confidence: 1.0,
            seen_count: 1,
            created_at: Utc::now(),
            last_seen_at: Utc::now(),
        };

        self.storage.store_pattern(pattern).await?;

        // Generate insights from similar patterns
        Ok(PatternInsights {
            similar_patterns: similar,
            suggested_refactorings: self.extract_successful_refactorings(&similar),
            confidence: calculate_confidence(&similar),
        })
    }

    /// Learn from mutation testing
    pub async fn learn_mutation_pattern(
        &self,
        mutant: &Mutant,
        survived: bool,
        context: &MutationContext,
    ) -> Result<PatternInsights> {
        // Extract features (same as ML predictor)
        let features = self.extractor.extract_mutation_features(mutant, context);

        // Find similar mutations
        let similar = self.storage
            .find_similar(&features, PatternType::Mutation, self.similarity_threshold, 10)
            .await?;

        // Calculate survival rate from similar patterns
        let survival_rate = calculate_survival_rate(&similar);

        // Store pattern
        let pattern = CodePattern {
            pattern_type: PatternType::Mutation {
                operator: mutant.operator.clone(),
                survival_rate,
                context: context.clone(),
            },
            features,
            // ...
        };

        self.storage.store_pattern(pattern).await?;

        Ok(PatternInsights {
            similar_patterns: similar,
            predicted_survival_rate: survival_rate,
            confidence: calculate_confidence(&similar),
        })
    }

    /// Query patterns for insights
    pub async fn query_patterns(
        &self,
        query_type: PatternQueryType,
    ) -> Result<Vec<CodePattern>> {
        match query_type {
            PatternQueryType::HighComplexity { threshold } => {
                // Return patterns with CC > threshold
            }
            PatternQueryType::SurvivingMutations { operator } => {
                // Return mutations of this type that survived
            }
            PatternQueryType::UnresolvedSATD { days_old } => {
                // Return SATD older than X days
            }
            PatternQueryType::SuccessfulRefactorings { min_improvement } => {
                // Return refactorings with improvement > X
            }
        }
    }
}
```

**3.2 ML Mutation Predictor Integration**
- Modify: `server/src/services/mutation/ml_predictor.rs`
```rust
impl SurvivabilityPredictor {
    /// Enhanced prediction using pattern learning
    pub fn predict_with_patterns(
        &self,
        mutant: &Mutant,
        context: &MutationContext,
        pattern_service: &PatternLearningService,
    ) -> Result<MutationPrediction> {
        // Original ML prediction
        let ml_prediction = self.predict(mutant, context)?;

        // Query historical patterns
        let features = FeatureExtractor::extract_mutation_features(mutant, context);
        let similar = pattern_service.storage
            .find_similar(&features, PatternType::Mutation, 0.85, 5)
            .await?;

        // Combine ML + pattern-based predictions
        let pattern_survival_rate = calculate_survival_rate(&similar);
        let confidence = calculate_combined_confidence(
            ml_prediction.confidence,
            similar.len(),
        );

        // Weighted average
        let combined_probability = if similar.is_empty() {
            ml_prediction.survival_probability
        } else {
            0.7 * ml_prediction.survival_probability + 0.3 * pattern_survival_rate
        };

        Ok(MutationPrediction {
            survival_probability: combined_probability,
            confidence,
            reasoning: format!(
                "ML: {:.0}%, Patterns: {:.0}% (from {} similar cases)",
                ml_prediction.survival_probability * 100.0,
                pattern_survival_rate * 100.0,
                similar.len()
            ),
        })
    }
}
```

### Phase 4: CLI & Testing (Day 7)

**4.1 CLI Commands**
```bash
# Enable pattern learning
pmat learning enable --db ~/.pmat/patterns.db

# Query learned patterns
pmat learning query complexity --threshold 10
pmat learning query mutations --operator arithmetic
pmat learning query satd --unresolved-days 90

# Show pattern statistics
pmat learning stats

# Export patterns for sharing (anonymized)
pmat learning export --output patterns.json --anonymize

# Import patterns from another project
pmat learning import --input patterns.json
```

**4.2 Unit Tests (RED → GREEN)**
```rust
#[test]
fn test_store_and_retrieve_pattern() {
    let storage = PatternStorage::new_in_memory().unwrap();
    let pattern = CodePattern { /* ... */ };

    let id = storage.store_pattern(pattern.clone()).await.unwrap();
    let retrieved = storage.get_pattern(id).await.unwrap();

    assert_eq!(retrieved, Some(pattern));
}

#[test]
fn test_find_similar_patterns() {
    let storage = setup_storage_with_patterns();
    let query_features = vec![10.0, 5.0, 3.0]; // High complexity

    let similar = storage
        .find_similar(&query_features, PatternType::Complexity, 0.85, 5)
        .await
        .unwrap();

    assert!(!similar.is_empty());
    assert!(similar[0].similarity >= 0.85);
}

#[test]
fn test_mutation_predictor_with_patterns() {
    let predictor = SurvivabilityPredictor::new();
    let pattern_service = setup_pattern_service();

    let mutant = create_test_mutant();
    let context = create_test_context();

    let prediction = predictor
        .predict_with_patterns(&mutant, &context, &pattern_service)
        .unwrap();

    assert!(prediction.confidence > 0.0);
}

#[test]
fn test_cosine_similarity() {
    let a = vec![1.0, 2.0, 3.0];
    let b = vec![1.0, 2.0, 3.0];
    let sim = SimilarityCalculator::cosine_similarity(&a, &b);
    assert_eq!(sim, 1.0); // Identical vectors

    let c = vec![3.0, 2.0, 1.0];
    let sim2 = SimilarityCalculator::cosine_similarity(&a, &c);
    assert!(sim2 < 1.0); // Different vectors
}
```

**4.3 Property Tests**
```rust
#[proptest]
fn test_cosine_similarity_properties(
    #[strategy(arb_feature_vector())] a: Vec<f64>,
    #[strategy(arb_feature_vector())] b: Vec<f64>,
) {
    let sim = SimilarityCalculator::cosine_similarity(&a, &b);

    // Similarity is in [0, 1]
    prop_assert!(sim >= 0.0 && sim <= 1.0);

    // Similarity is symmetric
    let sim_rev = SimilarityCalculator::cosine_similarity(&b, &a);
    prop_assert!((sim - sim_rev).abs() < 1e-10);

    // Self-similarity is 1.0
    let self_sim = SimilarityCalculator::cosine_similarity(&a, &a);
    prop_assert!((self_sim - 1.0).abs() < 1e-10);
}
```

**4.4 Integration Tests**
```rust
#[tokio::test]
async fn test_end_to_end_pattern_learning() {
    // Analyze project A
    let result_a = analyze_project("project_a").await;

    // Learn patterns
    let pattern_service = PatternLearningService::new(...);
    pattern_service.learn_from_analysis(&result_a).await.unwrap();

    // Analyze similar project B
    let result_b = analyze_project("project_b").await;

    // Verify predictions improved using patterns from A
    let predictions = pattern_service
        .get_insights_for_project("project_b")
        .await
        .unwrap();

    assert!(!predictions.similar_patterns.is_empty());
}
```

---

## Files to Create

### New Files
```
server/src/services/learning/mod.rs             (400 lines)
server/src/services/learning/storage.rs         (500 lines)
server/src/services/learning/features.rs        (400 lines)
server/src/services/learning/similarity.rs      (200 lines)
server/src/services/learning/types.rs           (300 lines)
server/src/services/learning/tests.rs           (400 lines)
server/src/cli/handlers/learning_handlers.rs    (350 lines)
docs/features/PATTERN_LEARNING.md               (700 lines)
```

### Files to Modify
```
server/src/services/mod.rs                      (export learning module)
server/src/services/mutation/ml_predictor.rs    (+100 lines for integration)
server/src/cli/handlers/mod.rs                  (export learning handlers)
server/src/cli/commands.rs                      (add learning subcommand)
Cargo.toml                                       (already has rusqlite)
```

**Estimated Total**: ~2,750 new lines + 150 modified lines

---

## Success Criteria

### Functional Requirements ✅
- ✅ Pattern storage with SQLite backend
- ✅ Feature extraction for complexity, mutations, SATD
- ✅ Similarity matching (cosine similarity)
- ✅ ML mutation predictor integration
- ✅ CLI commands functional

### Quality Requirements ✅
- ✅ Test coverage ≥85%
- ✅ Property tests for similarity calculations
- ✅ Integration tests end-to-end
- ✅ Documentation with examples

### Performance Requirements ✅
- ✅ Pattern storage <10ms per pattern
- ✅ Similarity search <100ms for 10K patterns
- ✅ Feature extraction <5ms per code unit

### Accuracy Requirements ✅
- ✅ ML mutation predictor accuracy improves from 75-95% to 80-98% (with sufficient historical data)
- ✅ Pattern matching precision >90% at 0.85 similarity threshold

---

## Risks & Mitigation

### Risk 1: Storage Growth
**Impact**: Medium - Database grows unbounded
**Mitigation**:
- Implement pruning strategy (remove low-confidence old patterns)
- Configurable retention policy (default: 90 days)
- Aggregate similar patterns (merge duplicates)

### Risk 2: Feature Vector Stability
**Impact**: Medium - Feature extraction changes break similarity
**Mitigation**:
- Version feature extractors
- Store extractor version with each pattern
- Only compare patterns with same feature version

### Risk 3: Privacy Concerns
**Impact**: Low-Medium - Patterns may expose sensitive code
**Mitigation**:
- Anonymization layer for export
- Project-local storage by default
- Opt-in for cross-project learning

### Risk 4: Cold Start Problem
**Impact**: Medium - No benefit until sufficient patterns collected
**Mitigation**:
- Pre-seed with synthetic patterns from known good code
- Provide import functionality for community patterns
- Fall back to ML-only prediction when <5 similar patterns

---

## Dependencies

### Internal
- ML mutation predictor (`server/src/services/mutation/ml_predictor.rs`)
- Complexity analyzer (`server/src/services/complexity.rs`)
- SATD detector (`server/src/services/satd_detector.rs`)

### External
- rusqlite (already in Cargo.toml)
- serde_json (already in deps)

---

## Deliverables

1. **Code**
   - Pattern storage service
   - Feature extractors
   - Similarity calculator
   - ML predictor integration
   - CLI handlers
   - Tests (unit + property + integration)

2. **Documentation**
   - User guide for pattern learning
   - Examples of insights generation
   - Privacy and anonymization guide

3. **Validation**
   - All tests passing
   - Mutation predictor accuracy improvement demonstrated
   - Performance benchmarks

---

## Post-MVP Enhancements

### Phase 2: Advanced Features (Deferred)
- Cross-project pattern sharing (anonymized)
- Pattern visualization dashboard
- Automatic refactoring suggestions
- SATD evolution tracking
- Team-level pattern aggregation

### Phase 3: Integration (Deferred)
- RefactoringAdvisor sub-agent uses patterns
- MCP tool: `query_patterns`
- Web dashboard showing learned patterns

---

## Related Tickets

- PMAT-7007: RefactoringAdvisor sub-agent will use pattern learning
- PMAT-7004: ML mutation predictor (completed) - enhance with patterns
- PMAT-7008: Workflows can query patterns for optimization

---

## References

- [Learning System Ideas](../specifications/learning-system-ideas.md#12-pattern-learning)
- [Existing ML Predictor](../../server/src/services/mutation/ml_predictor.rs)
- [Memory Manager](../../server/src/services/memory_manager.rs)

---

**Created**: 2025-10-07
**Last Updated**: 2025-10-07
**Status**: Ready for implementation
