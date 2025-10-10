// RED Phase: Write failing tests first
// PMAT-SEARCH-002: OpenAI Embeddings Client
// Test count: 15 tests

use pmat::services::semantic::openai_embeddings::*;

// ============================================================================
// Single Embedding Tests (1 test)
// ============================================================================

#[tokio::test]
async fn test_embed_single_text() {
    let client = OpenAIEmbeddingsClient::new("test-api-key").unwrap();
    let text = "fn add(a: i32, b: i32) -> i32 { a + b }";

    let result = client.embed(text).await.unwrap();
    assert_eq!(result.embeddings.len(), 1);
    assert_eq!(result.embeddings[0].len(), 1536); // text-embedding-3-small dimensions
    assert!(result.tokens_used > 0);
    assert_eq!(result.model, "text-embedding-3-small");
}

// ============================================================================
// Batch Embedding Tests (3 tests)
// ============================================================================

#[tokio::test]
async fn test_embed_batch() {
    let client = OpenAIEmbeddingsClient::new("test-api-key").unwrap();
    let texts = vec![
        "fn add(a: i32, b: i32) -> i32 { a + b }",
        "fn multiply(a: i32, b: i32) -> i32 { a * b }",
        "fn divide(a: i32, b: i32) -> i32 { a / b }",
    ];

    let result = client.embed_batch(&texts).await.unwrap();
    assert_eq!(result.embeddings.len(), 3);
    assert_eq!(result.embeddings[0].len(), 1536);
    assert!(result.tokens_used > 0);
}

#[tokio::test]
async fn test_batch_size_limit() {
    let client = OpenAIEmbeddingsClient::new("test-api-key").unwrap();
    let texts: Vec<String> = (0..101).map(|i| format!("fn test_{i}() {{}}")).collect();
    let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();

    // Should error on >100 batch size
    let result = client.embed_batch(&text_refs).await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Batch size exceeds"));
}

#[tokio::test]
async fn test_batch_empty() {
    let client = OpenAIEmbeddingsClient::new("test-api-key").unwrap();
    let texts: Vec<&str> = vec![];

    let result = client.embed_batch(&texts).await;
    assert!(result.is_err());
}

// ============================================================================
// Input Validation Tests (2 tests)
// ============================================================================

#[tokio::test]
async fn test_empty_input() {
    let client = OpenAIEmbeddingsClient::new("test-api-key").unwrap();
    let result = client.embed("").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Empty text"));
}

#[tokio::test]
async fn test_whitespace_only_input() {
    let client = OpenAIEmbeddingsClient::new("test-api-key").unwrap();
    let result = client.embed("   \n\t   ").await;
    assert!(result.is_err());
}

// ============================================================================
// Cost Calculation Tests (1 test)
// ============================================================================

#[tokio::test]
async fn test_cost_calculation() {
    let client = OpenAIEmbeddingsClient::new("test-api-key").unwrap();
    let text = "fn add(a: i32, b: i32) -> i32 { a + b }";

    let result = client.embed(text).await.unwrap();
    // text-embedding-3-small: $0.00002 per 1K tokens
    let expected_cost = (result.tokens_used as f64 / 1000.0) * 0.00002;
    assert!((result.cost - expected_cost).abs() < 0.000001); // Floating point tolerance
}

// ============================================================================
// Normalization Tests (1 test)
// ============================================================================

#[tokio::test]
async fn test_embedding_normalization() {
    let client = OpenAIEmbeddingsClient::new("test-api-key").unwrap();
    let text = "fn test() {}";

    let result = client.embed(text).await.unwrap();
    let embedding = &result.embeddings[0];

    // Verify L2 norm is approximately 1.0 (OpenAI returns normalized embeddings)
    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 0.01);
}

// ============================================================================
// Error Handling Tests (3 tests)
// ============================================================================

#[tokio::test]
#[ignore] // Requires actual API call to test
async fn test_invalid_api_key() {
    let client = OpenAIEmbeddingsClient::new("sk-invalid-key").unwrap();
    let result = client.embed("fn test() {}").await;
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("Unauthorized") || error_msg.contains("Invalid API key"),
        "Unexpected error: {error_msg}"
    );
}

#[tokio::test]
async fn test_api_key_validation() {
    // Empty API key should fail at construction
    let result = OpenAIEmbeddingsClient::new("");
    assert!(result.is_err());
}

#[tokio::test]
async fn test_model_name_validation() {
    let client = OpenAIEmbeddingsClient::new("test-key").unwrap();
    assert_eq!(client.model(), "text-embedding-3-small");
}

// ============================================================================
// Rate Limiting Tests (2 tests)
// ============================================================================

#[tokio::test]
async fn test_retry_configuration() {
    let client = OpenAIEmbeddingsClient::new("test-api-key").unwrap();
    // Verify exponential backoff config exists
    assert!(client.max_retries() > 0);
    assert!(client.base_delay_ms() > 0);
}

#[tokio::test]
#[ignore] // Requires mocking to test retry behavior
async fn test_retry_on_rate_limit() {
    let client = OpenAIEmbeddingsClient::new("test-api-key").unwrap();
    // Would need to mock 429 response to test this properly
    // For now, just verify the configuration
    assert_eq!(client.max_retries(), 3);
}

// ============================================================================
// API Key Validation Tests (2 tests)
// ============================================================================

#[test]
fn test_api_key_format_validation() {
    // OpenAI API keys start with "sk-"
    let result = OpenAIEmbeddingsClient::new("invalid-format");
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("API key must start with 'sk-'"));
}

#[test]
fn test_api_key_length_validation() {
    // OpenAI API keys are typically 51+ characters
    let result = OpenAIEmbeddingsClient::new("sk-short");
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("API key too short"));
}
