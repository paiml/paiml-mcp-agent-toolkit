# PMAT-SEARCH-002: OpenAI Embeddings Client

**Sprint**: 29
**Status**: 🔴 RED PHASE
**Estimated**: 2 hours
**Actual**: TBD

## 🎯 Objective

Implement OpenAI embeddings client to generate 1536-dimensional vectors from code chunks using `text-embedding-3-small` model.

## 📋 Requirements

**Must Support:**
- Generate embeddings for single text inputs
- Batch processing (1-100 texts per API call)
- Rate limit handling with exponential backoff
- Cost tracking (tokens used, API calls)
- Model: `text-embedding-3-small` (1536 dimensions)

**Response Metadata:**
- `embeddings`: Vec<Vec<f32>> (1536 dimensions each)
- `tokens_used`: Total tokens consumed
- `cost`: Estimated cost in USD
- `model`: Model name used

## 🔴 RED Phase: Tests First

### Test Suite

```rust
// tests/unit_openai_embeddings.rs

#[test]
fn test_embed_single_text() {
    let client = OpenAIEmbeddingsClient::new("test-api-key")?;
    let text = "fn add(a: i32, b: i32) -> i32 { a + b }";

    let result = client.embed(text).await?;
    assert_eq!(result.embeddings.len(), 1);
    assert_eq!(result.embeddings[0].len(), 1536); // text-embedding-3-small dimensions
    assert!(result.tokens_used > 0);
    assert_eq!(result.model, "text-embedding-3-small");
}

#[test]
fn test_embed_batch() {
    let client = OpenAIEmbeddingsClient::new("test-api-key")?;
    let texts = vec![
        "fn add(a: i32, b: i32) -> i32 { a + b }",
        "fn multiply(a: i32, b: i32) -> i32 { a * b }",
        "fn divide(a: i32, b: i32) -> i32 { a / b }",
    ];

    let result = client.embed_batch(&texts).await?;
    assert_eq!(result.embeddings.len(), 3);
    assert_eq!(result.embeddings[0].len(), 1536);
    assert!(result.tokens_used > 0);
}

#[test]
fn test_batch_size_limit() {
    let client = OpenAIEmbeddingsClient::new("test-api-key")?;
    let texts: Vec<String> = (0..101).map(|i| format!("fn test_{i}() {{}}")).collect();

    // Should error on >100 batch size
    let result = client.embed_batch(&texts).await;
    assert!(result.is_err());
}

#[test]
fn test_empty_input() {
    let client = OpenAIEmbeddingsClient::new("test-api-key")?;
    let result = client.embed("").await;
    assert!(result.is_err());
}

#[test]
fn test_cost_calculation() {
    let client = OpenAIEmbeddingsClient::new("test-api-key")?;
    let text = "fn add(a: i32, b: i32) -> i32 { a + b }";

    let result = client.embed(text).await?;
    // text-embedding-3-small: $0.00002 per 1K tokens
    let expected_cost = (result.tokens_used as f64 / 1000.0) * 0.00002;
    assert!((result.cost - expected_cost).abs() < 0.000001); // Floating point tolerance
}

#[test]
fn test_embedding_normalization() {
    let client = OpenAIEmbeddingsClient::new("test-api-key")?;
    let text = "fn test() {}";

    let result = client.embed(text).await?;
    let embedding = &result.embeddings[0];

    // Verify L2 norm is approximately 1.0 (OpenAI returns normalized embeddings)
    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 0.01);
}

#[test]
fn test_retry_on_rate_limit() {
    let client = OpenAIEmbeddingsClient::new("test-api-key")?;
    // This test requires mocking the OpenAI API
    // For now, verify exponential backoff config exists
    assert!(client.max_retries() > 0);
}

#[test]
fn test_invalid_api_key() {
    let client = OpenAIEmbeddingsClient::new("invalid-key")?;
    let result = client.embed("fn test() {}").await;
    assert!(result.is_err());
}
```

**Total Tests**: 15
- Single embedding (1 test)
- Batch embeddings (3 tests)
- Input validation (2 tests)
- Cost calculation (1 test)
- Normalization (1 test)
- Error handling (3 tests)
- Rate limiting (2 tests)
- API key validation (2 tests)

## 🟢 GREEN Phase: Implementation

**File**: `server/src/services/semantic/openai_embeddings.rs`

**Key Structures:**

```rust
pub struct OpenAIEmbeddingsClient {
    api_key: String,
    model: String, // "text-embedding-3-small"
    max_retries: u32,
    base_delay_ms: u64,
}

pub struct EmbeddingResult {
    pub embeddings: Vec<Vec<f32>>,
    pub tokens_used: usize,
    pub cost: f64,
    pub model: String,
}
```

**Key Functions:**
- `new(api_key: &str) -> Result<Self>`
- `embed(&self, text: &str) -> Result<EmbeddingResult>`
- `embed_batch(&self, texts: &[&str]) -> Result<EmbeddingResult>`
- `calculate_cost(tokens: usize, model: &str) -> f64`

**Dependencies:**
- `reqwest` for HTTP client
- `serde_json` for JSON parsing
- `tokio` for async runtime

## 🔵 REFACTOR Phase: Quality

**Complexity Target**: ≤10 cyclomatic per function
**Coverage Target**: ≥95%
**SATD**: 0 violations

**Refactoring checklist:**
- Extract retry logic to separate function
- Extract cost calculation to separate module
- Add comprehensive error types
- Add debug logging
- Document all public APIs

## ✅ Exit Criteria

- [ ] 15 tests passing
- [ ] Supports single and batch embeddings
- [ ] Rate limit handling with exponential backoff
- [ ] Cost tracking accurate to 6 decimal places
- [ ] Embeddings are L2-normalized
- [ ] Batch size limited to 100
- [ ] Invalid API key returns clear error
- [ ] Cyclomatic ≤10 for all functions
- [ ] Zero clippy warnings

## 📊 Cost Analysis

**Pricing (text-embedding-3-small)**:
- $0.00002 per 1K tokens
- Average code chunk: ~500 tokens
- Cost per chunk: $0.00001

**Example costs:**
```
1,000 chunks × 500 tokens × $0.00002 = $0.01 (1¢)
10,000 chunks × 500 tokens × $0.00002 = $0.10 (10¢)
100,000 chunks × 500 tokens × $0.00002 = $1.00
```

## 🔗 Integration

Will be used by:
- PMAT-SEARCH-003: Turso Vector Database (store embeddings)
- PMAT-SEARCH-004: Vector Similarity Search (query embeddings)
- PMAT-SEARCH-009: CLI Commands (generate embeddings on demand)
