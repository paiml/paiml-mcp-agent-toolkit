# Specification: Improve Context Generation & Agent Integration

**Status**: Draft
**Version**: 2.0.0
**Created**: 2025-02-04
**Updated**: 2025-02-04
**Author**: PMAT Team

## The Real Problem: Agents Don't Use Context

### Core Issue

`pmat context` generates rich AST with quality annotations:
- Function signatures with complexity scores
- TDG grades per file
- SATD markers
- Big-O estimates
- Provability scores

**But agents like Claude Code NEVER use it.** They grep/glob the codebase instead.

This is wasteful:
1. Grepping is slow and context-inefficient
2. No quality awareness (agents don't know what's complex/risky)
3. No semantic understanding (just text matching)
4. Repeated work every session

### The Vision: RAG-Powered Agent Context

```
┌─────────────────────────────────────────────────────────────────────┐
│                    CURRENT (Broken)                                  │
├─────────────────────────────────────────────────────────────────────┤
│  Agent: "Find error handling code"                                   │
│    ↓                                                                 │
│  grep -r "error" src/ | head -50                                     │
│    ↓                                                                 │
│  [500 irrelevant matches, no context, no quality info]              │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│                    PROPOSED (RAG-Powered)                            │
├─────────────────────────────────────────────────────────────────────┤
│  Agent: "Find error handling code"                                   │
│    ↓                                                                 │
│  pmat query "error handling"                                         │
│    ↓                                                                 │
│  [Top 5 functions with error handling, ranked by:                    │
│   - Semantic relevance                                               │
│   - TDG score (quality)                                              │
│   - Complexity (maintainability)                                     │
│   - Full function signatures + doc comments]                         │
└─────────────────────────────────────────────────────────────────────┘
```

### What Agents Need

| Need | Current | Proposed |
|------|---------|----------|
| Find code by intent | grep (text match) | Semantic search |
| Know code quality | Nothing | TDG/complexity scores |
| Understand structure | Read files | AST-aware chunks |
| Find related code | Manual | Graph traversal |
| Avoid bad code | Nothing | Quality filtering |

## Problem Statement (Original)

### Hallucination Detection Limitations

1. **Claim Extraction is Too Narrow**
   - Only matches 3 regex patterns:
     - `PMAT can [verb] [object]`
     - `PMAT cannot [verb] [object]`
     - `PMAT supports [object]`
   - Common documentation patterns are NOT detected:
     - Bullet points: `- **Feature** - description`
     - Badges: `![label](url)` with counts
     - Count claims: `17+ languages`, `4600+ tests`
     - Capability verbs: `provides`, `includes`, `offers`, `enables`

2. **Result**: `pmat validate-readme` finds 0 claims in real documentation

3. **Context is Generated but Underutilized**
   - `pmat context` generates ~50K lines of rich function/file metadata
   - `CodeFactDatabase` parses it correctly
   - But claim extraction fails, so validation never happens

### Evidence

```bash
$ pmat validate-readme --targets README.md --deep-context /tmp/deep_context.md --verbose
📖 Validating README.md...
✅ Verified claims:  0
❌ Contradictions:   0
⚠️  Unverified:       0
```

README.md contains 300+ lines with claims like:
- "17+ Languages"
- "19 tools for Claude Code"
- "4600+ passing" tests
- "coverage >85%"

None are detected because they don't match `PMAT can/supports` patterns.

## Proposed Solution

### Option A: Enhance ClaimExtractor (Incremental)

Extend `src/services/hallucination_detector.rs` with additional patterns:

```rust
let capability_patterns = vec![
    // Existing
    Regex::new(r"(?i)PMAT can ([a-z]+)\s+(.+?)(?:\.|$)"),
    Regex::new(r"(?i)PMAT cannot ([a-z]+)\s+(.+?)(?:\.|$)"),
    Regex::new(r"(?i)PMAT supports? (.+?)(?:\.|$)"),

    // NEW: Common documentation patterns
    Regex::new(r"(?i)(?:pmat\s+)?provides\s+(.+?)(?:\.|$)"),
    Regex::new(r"(?i)(?:pmat\s+)?includes\s+(.+?)(?:\.|$)"),
    Regex::new(r"(?i)(?:pmat\s+)?offers\s+(.+?)(?:\.|$)"),
    Regex::new(r"(?i)(?:pmat\s+)?enables\s+(.+?)(?:\.|$)"),

    // NEW: Count claims
    Regex::new(r"(\d+)\+?\s+(languages?|tools?|tests?|features?)"),

    // NEW: Bullet point features
    Regex::new(r"-\s+\*\*([^*]+)\*\*\s*[-–]\s*(.+)"),

    // NEW: Badge claims (coverage, tests)
    Regex::new(r"(?i)coverage[^\d]*(\d+)%"),
    Regex::new(r"(?i)(\d+)[^\d]*(?:tests?|passing)"),
];
```

**Pros**: Minimal changes, no new dependencies
**Cons**: Regex-based, brittle, won't handle semantic variations

### Option B: Integrate trueno-rag (Recommended)

Replace regex-based claim extraction with semantic search using `trueno-rag`:

#### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     pmat context pipeline                        │
├─────────────────────────────────────────────────────────────────┤
│  1. Generate context.md (existing)                              │
│  2. Index context.md into trueno-rag                            │
│     - StructuralChunker for markdown sections                   │
│     - FastEmbedder for semantic embeddings                      │
│     - BM25Index for keyword search                              │
│  3. Extract claims from README.md                               │
│     - Use LLM-style claim extraction (not regex)                │
│     - Semantic similarity to find evidence                      │
│  4. Validate claims against indexed context                     │
│     - Hybrid search (dense + sparse)                            │
│     - RRF fusion for best results                               │
│     - Confidence scoring based on similarity                    │
└─────────────────────────────────────────────────────────────────┘
```

#### Implementation

```rust
use trueno_rag::{
    pipeline::RagPipelineBuilder,
    chunk::StructuralChunker,  // Markdown-aware
    embed::FastEmbedder,
    rerank::LexicalReranker,
    fusion::FusionStrategy,
    Document,
};

pub struct SemanticClaimValidator {
    pipeline: RagPipeline,
}

impl SemanticClaimValidator {
    pub fn from_context(context_md: &str) -> Result<Self> {
        let mut pipeline = RagPipelineBuilder::new()
            .chunker(StructuralChunker::new())  // Header-aware
            .embedder(FastEmbedder::new(EmbeddingModelType::AllMiniLmL6V2)?)
            .reranker(LexicalReranker::new())
            .fusion(FusionStrategy::RRF { k: 60.0 })
            .build()?;

        // Index the deep context
        let doc = Document::new(context_md).with_title("Project Context");
        pipeline.index_document(&doc)?;

        Ok(Self { pipeline })
    }

    pub fn validate_claim(&self, claim: &str) -> ValidationResult {
        // Semantic search for evidence
        let (results, _) = self.pipeline.query_with_context(claim, 5)?;

        if results.is_empty() {
            return ValidationResult::Unverified;
        }

        let top_score = results[0].score;

        if top_score > 0.85 {
            ValidationResult::Verified { confidence: top_score }
        } else if top_score > 0.5 {
            ValidationResult::PartialMatch { confidence: top_score }
        } else {
            ValidationResult::Unverified
        }
    }
}
```

#### Claim Extraction with Semantic Understanding

Instead of regex, use semantic patterns:

```rust
/// Extract claims using semantic pattern matching
pub fn extract_claims_semantic(documentation: &str) -> Vec<Claim> {
    let mut claims = Vec::new();

    for line in documentation.lines() {
        // Skip code blocks, headers as before

        // Use semantic indicators instead of exact regex
        if contains_capability_indicator(line) {
            claims.push(extract_capability(line));
        }
        if contains_count_claim(line) {
            claims.push(extract_count_claim(line));
        }
        if contains_feature_bullet(line) {
            claims.push(extract_feature_claim(line));
        }
    }

    claims
}

fn contains_capability_indicator(line: &str) -> bool {
    let indicators = [
        "provides", "includes", "offers", "enables", "supports",
        "can", "allows", "features", "built-in", "integrated",
    ];
    indicators.iter().any(|i| line.to_lowercase().contains(i))
}
```

**Pros**:
- Semantic understanding, not just regex
- Hybrid search (keywords + embeddings)
- Uses sovereign stack (trueno-rag)
- Battle-tested chunking strategies
- Confidence scores from similarity

**Cons**:
- Adds dependency (~90MB model download on first run)
- Slightly slower (embedding computation)

### Option C: Hybrid Approach

Use Option A (enhanced regex) for fast extraction, then Option B (trueno-rag) for validation:

1. Extract claims with enhanced regex (fast, catches most patterns)
2. Index context.md with trueno-rag
3. Validate each claim with semantic search
4. Return confidence scores

This gives the best of both worlds: fast extraction + semantic validation.

## Dependency Analysis

### Current pmat Dependencies

```toml
# Already in Cargo.toml
trueno-rag = "0.1.10"  # ✅ Already a dependency!
```

### trueno-rag Already Integrated!

trueno-rag is already used in pmat for semantic search:

| File | Usage |
|------|-------|
| `src/services/semantic/turso_vector_db.rs` | VectorStore for SIMD-accelerated search |
| `src/services/semantic/chunker.rs` | RecursiveChunker for RAG pipelines |
| `src/services/semantic/hybrid_search.rs` | BM25Index for keyword search |

**Key insight**: The infrastructure exists but is NOT connected to hallucination detection.

```rust
// turso_vector_db.rs:10-11
use trueno_rag::index::{VectorStore, VectorStoreConfig};
use trueno_rag::{Chunk, ChunkId, DocumentId};

// chunker.rs:735-736
use trueno_rag::chunk::{Chunker, RecursiveChunker};
use trueno_rag::Document;

// hybrid_search.rs:411-412
use trueno_rag::index::BM25Index;
use trueno_rag::{Chunk, ChunkId, DocumentId, SparseIndex};
```

**The solution is to wire up the existing trueno-rag infrastructure to the claim validation pipeline.**

### trueno-rag Capabilities

| Feature | Description | Useful For |
|---------|-------------|------------|
| StructuralChunker | Header/section-aware | Markdown context |
| FastEmbedder | 384-dim vectors | Semantic similarity |
| BM25Index | Keyword search | Exact term matching |
| RRF Fusion | Hybrid retrieval | Best of both |
| LexicalReranker | Term overlap | Claim verification |

## Implementation Plan

### Phase 1: Enhance Claim Extraction (1-2 days)

1. Add new regex patterns to `ClaimExtractor` in `src/services/hallucination_detector.rs`
2. Add tests for common documentation patterns
3. Verify claims are now extracted from README.md

**Files to modify:**
- `src/services/hallucination_detector.rs` - Add patterns
- `src/tests/hallucination_detection_tests.rs` - Add tests

### Phase 2: Wire trueno-rag to Validation (1-2 days)

Since trueno-rag is already integrated, we just need to connect it:

1. Create `SemanticClaimValidator` in `src/services/semantic_claim_validator.rs`
2. Use existing `TursoVectorDb` and `HybridSearchEngine`
3. Index deep context at validation time
4. Add confidence scoring from similarity scores

**Files to create/modify:**
- `src/services/semantic_claim_validator.rs` - NEW
- `src/services/hallucination_detector.rs` - Wire up
- `src/cli/handlers/readme_validate_handlers.rs` - Use new validator

**Concrete implementation:**

```rust
// src/services/semantic_claim_validator.rs
use crate::services::semantic::hybrid_search::HybridSearchEngine;
use crate::services::semantic::turso_vector_db::TursoVectorDb;
use trueno_rag::chunk::{Chunker, StructuralChunker};

pub struct SemanticClaimValidator {
    hybrid_engine: HybridSearchEngine,
    vector_db: TursoVectorDb,
}

impl SemanticClaimValidator {
    pub async fn from_context(context_md: &str) -> Result<Self> {
        // 1. Chunk the context with StructuralChunker
        let chunker = StructuralChunker::new();
        let doc = trueno_rag::Document::new(context_md);
        let chunks = chunker.chunk(&doc)?;

        // 2. Create vector DB and index chunks
        let vector_db = TursoVectorDb::new()?;
        for chunk in &chunks {
            vector_db.insert_chunk(chunk)?;
        }

        // 3. Create hybrid search engine
        let hybrid_engine = HybridSearchEngine::new(vector_db.clone())?;

        Ok(Self { hybrid_engine, vector_db })
    }

    pub async fn validate_claim(&self, claim: &str) -> ValidationResult {
        // Hybrid search for evidence
        let results = self.hybrid_engine.search(claim, 5).await?;

        if results.is_empty() {
            return ValidationResult::Unverified { confidence: 0.0 };
        }

        let top_score = results[0].score;
        let evidence = results[0].content.clone();

        if top_score > 0.85 {
            ValidationResult::Verified { confidence: top_score, evidence }
        } else if top_score > 0.5 {
            ValidationResult::PartialMatch { confidence: top_score, evidence }
        } else {
            ValidationResult::Unverified { confidence: top_score }
        }
    }
}
```

### Phase 3: Improve Context Output (1 day)

1. Ensure context.md is optimized for chunking
2. Add section headers for better structural parsing
3. Include function signatures for precise matching

### Phase 4: Testing & Validation (1 day)

1. Create test suite with real README claims
2. Verify true positives (valid claims verified)
3. Verify true negatives (invalid claims caught)
4. Benchmark performance

**Test cases:**
```rust
#[test]
fn test_language_count_claim() {
    let validator = SemanticClaimValidator::from_context(SAMPLE_CONTEXT).await?;
    let result = validator.validate_claim("PMAT supports 17+ languages").await?;
    assert!(matches!(result, ValidationResult::Verified { .. }));
}

#[test]
fn test_false_claim_detected() {
    let validator = SemanticClaimValidator::from_context(SAMPLE_CONTEXT).await?;
    let result = validator.validate_claim("PMAT supports 100 languages").await?;
    assert!(matches!(result, ValidationResult::Contradiction { .. }));
}
```

## Success Criteria

1. `pmat validate-readme` extracts >80% of claims from README.md
2. Verified claims have confidence scores
3. False positive rate <5%
4. Performance: <5s for typical README validation

## Open Questions

1. Should we cache the trueno-rag index for faster subsequent runs?
2. Should claim extraction be LLM-assisted for better semantic understanding?
3. Should we add a `--semantic` flag for optional deep validation?

## Proposed: RAG-Powered Agent Context

### Architecture

```
┌──────────────────────────────────────────────────────────────────────────┐
│                         pmat context --serve                              │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─────────────┐    ┌──────────────┐    ┌─────────────────────────────┐ │
│  │ AST Parser  │───▶│ Annotator    │───▶│ trueno-rag Index            │ │
│  │ (tree-sitter│    │ - TDG scores │    │ - VectorStore (embeddings)  │ │
│  │  + custom)  │    │ - Complexity │    │ - BM25Index (keywords)      │ │
│  │             │    │ - SATD       │    │ - StructuralChunker         │ │
│  │             │    │ - Big-O      │    │                             │ │
│  └─────────────┘    └──────────────┘    └─────────────────────────────┘ │
│                                                    │                     │
│                                                    ▼                     │
│                                          ┌─────────────────┐            │
│                                          │ MCP Server      │            │
│                                          │ - query_code    │            │
│                                          │ - get_function  │            │
│                                          │ - find_similar  │            │
│                                          │ - quality_filter│            │
│                                          └─────────────────┘            │
│                                                    │                     │
└────────────────────────────────────────────────────│─────────────────────┘
                                                     │
                                                     ▼
                                          ┌─────────────────┐
                                          │ Claude Code     │
                                          │ Cline           │
                                          │ Other Agents    │
                                          └─────────────────┘
```

### New MCP Tools

```json
{
  "tools": [
    {
      "name": "pmat_query_code",
      "description": "Semantic search for code by intent, returns annotated functions",
      "input": {
        "query": "error handling in API layer",
        "limit": 5,
        "min_quality": "B",
        "max_complexity": 15
      },
      "output": {
        "results": [
          {
            "function": "handle_api_error",
            "file": "src/api/error.rs",
            "line": 42,
            "signature": "pub fn handle_api_error(err: ApiError) -> Response",
            "doc": "Converts API errors to HTTP responses",
            "tdg_score": 2.1,
            "tdg_grade": "A",
            "complexity": 8,
            "cognitive": 5,
            "big_o": "O(1)",
            "satd_count": 0,
            "relevance_score": 0.92
          }
        ]
      }
    },
    {
      "name": "pmat_get_function",
      "description": "Get full function with context and quality metrics",
      "input": {
        "file": "src/api/error.rs",
        "function": "handle_api_error"
      }
    },
    {
      "name": "pmat_find_similar",
      "description": "Find functions similar to a given one (for refactoring)",
      "input": {
        "file": "src/api/error.rs",
        "function": "handle_api_error",
        "limit": 5
      }
    },
    {
      "name": "pmat_quality_report",
      "description": "Get quality summary for a file or module",
      "input": {
        "path": "src/api/"
      }
    }
  ]
}
```

### Why This Stops Grepping

1. **Semantic Search**: Agents ask "find error handling" not `grep -r "error"`
2. **Quality Awareness**: Results sorted by TDG score, not file order
3. **Context Included**: Full signatures, docs, metrics - no need to read files
4. **Pre-indexed**: O(1) lookup vs O(n) grep
5. **Structured Output**: JSON with all metadata, not raw text

### Integration with `pmat comply`

Add compliance check for agent behavior:

```yaml
# .pmat-gates.toml
[agent-context]
# Require agents to use RAG instead of grep
require_semantic_search = true
max_grep_calls = 5  # Allow some for edge cases
warn_on_file_read_without_query = true
```

```bash
$ pmat comply check
✓ Agent Context: 95% queries via RAG (target: >80%)
⚠ Agent Context: 3 grep calls detected (max: 5)
```

### Implementation with trueno-rag

```rust
// src/services/agent_context.rs
use trueno_rag::{
    pipeline::RagPipelineBuilder,
    chunk::StructuralChunker,
    embed::FastEmbedder,
    fusion::FusionStrategy,
};

pub struct AgentContextServer {
    pipeline: RagPipeline,
    quality_index: HashMap<String, QualityMetrics>,
}

impl AgentContextServer {
    /// Build from pmat context output
    pub async fn from_project(project_path: &Path) -> Result<Self> {
        // 1. Generate annotated context
        let context = pmat_context::analyze_project(project_path).await?;

        // 2. Build quality index
        let mut quality_index = HashMap::new();
        for file in &context.files {
            for func in &file.functions {
                quality_index.insert(
                    format!("{}::{}", file.path, func.name),
                    QualityMetrics {
                        tdg_score: func.tdg_score,
                        complexity: func.complexity,
                        cognitive: func.cognitive,
                        big_o: func.big_o.clone(),
                        satd_count: func.satd_markers.len(),
                    },
                );
            }
        }

        // 3. Build RAG pipeline with function-aware chunking
        let mut pipeline = RagPipelineBuilder::new()
            .chunker(FunctionAwareChunker::new())  // Custom: one chunk per function
            .embedder(FastEmbedder::new(EmbeddingModelType::AllMiniLmL6V2)?)
            .fusion(FusionStrategy::RRF { k: 60.0 })
            .build()?;

        // 4. Index all functions with metadata
        for file in &context.files {
            for func in &file.functions {
                let doc = Document::new(&func.to_markdown())
                    .with_metadata("file", &file.path)
                    .with_metadata("function", &func.name)
                    .with_metadata("tdg_score", func.tdg_score)
                    .with_metadata("complexity", func.complexity);
                pipeline.index_document(&doc)?;
            }
        }

        Ok(Self { pipeline, quality_index })
    }

    /// Semantic search with quality filtering
    pub async fn query(
        &self,
        query: &str,
        limit: usize,
        min_quality: Option<Grade>,
        max_complexity: Option<u32>,
    ) -> Result<Vec<AnnotatedResult>> {
        let results = self.pipeline.query(query, limit * 2)?;  // Over-fetch for filtering

        let mut annotated: Vec<_> = results
            .into_iter()
            .filter_map(|r| {
                let key = format!("{}::{}", r.metadata["file"], r.metadata["function"]);
                let quality = self.quality_index.get(&key)?;

                // Apply quality filters
                if let Some(min) = min_quality {
                    if quality.grade() < min { return None; }
                }
                if let Some(max) = max_complexity {
                    if quality.complexity > max { return None; }
                }

                Some(AnnotatedResult {
                    content: r.content,
                    file: r.metadata["file"].clone(),
                    function: r.metadata["function"].clone(),
                    relevance: r.score,
                    quality: quality.clone(),
                })
            })
            .take(limit)
            .collect();

        // Sort by relevance * quality
        annotated.sort_by(|a, b| {
            let score_a = a.relevance * (1.0 - a.quality.tdg_score / 10.0);
            let score_b = b.relevance * (1.0 - b.quality.tdg_score / 10.0);
            score_b.partial_cmp(&score_a).unwrap()
        });

        Ok(annotated)
    }
}
```

### CLI Command

```bash
# Start context server (indexes project, starts MCP)
pmat context --serve

# One-shot query (for scripting)
pmat query "error handling in API" --limit 5 --min-quality B

# Check if context is indexed
pmat context --status
```

### Benefits

| Metric | Grep Approach | RAG Approach |
|--------|---------------|--------------|
| Time to find code | O(n) scan | O(1) lookup |
| Context tokens | ~500 per match | ~100 per match (structured) |
| Quality awareness | None | Full metrics |
| Semantic match | No | Yes |
| Repeated work | Every query | Indexed once |

### Migration Path

1. **Phase 1**: Add `pmat query` command using existing trueno-rag
2. **Phase 2**: Add MCP tools for agent integration
3. **Phase 3**: Add `pmat context --serve` for persistent index
4. **Phase 4**: Add compliance checks for grep vs query

## References

- [trueno-rag documentation](https://docs.rs/trueno-rag)
- [Semantic Entropy paper (Farquhar et al., Nature 2024)](https://www.nature.com/articles/s41586-024-07421-0)
- [PMAT hallucination detection](../CLAUDE.md#documentation-accuracy-enforcement)
- [Claude Code MCP integration](https://docs.anthropic.com/claude-code/mcp)

## Appendix: Current ClaimExtractor Code

Location: `src/services/hallucination_detector.rs:122-132`

```rust
let capability_patterns = vec![
    // Positive capabilities: "PMAT can analyze X"
    Regex::new(r"(?i)PMAT can ([a-z]+)\s+(.+?)(?:\.|$)").expect("internal error"),
    // Negative capabilities: "PMAT cannot compile"
    Regex::new(r"(?i)PMAT cannot ([a-z]+)\s+(.+?)(?:\.|$)").expect("internal error"),
    // Alternative patterns: "PMAT supports X"
    Regex::new(r"(?i)PMAT supports? (.+?)(?:\.|$)").expect("internal error"),
];
```

This is the root cause of 0 claims being extracted from typical documentation.
