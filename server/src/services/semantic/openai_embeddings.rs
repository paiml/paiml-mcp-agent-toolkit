// OpenAI Embeddings Client
// PMAT-SEARCH-002: Generate embeddings using OpenAI API
//
// GREEN Phase: Full implementation with rate limiting and error handling

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;

/// OpenAI embeddings client for generating vector embeddings
pub struct OpenAIEmbeddingsClient {
    api_key: String,
    model: String,
    max_retries: u32,
    base_delay_ms: u64,
    client: Client,
}

impl std::fmt::Debug for OpenAIEmbeddingsClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenAIEmbeddingsClient")
            .field("model", &self.model)
            .field("max_retries", &self.max_retries)
            .field("base_delay_ms", &self.base_delay_ms)
            .finish_non_exhaustive()
    }
}

/// Result of embedding operation
#[derive(Debug, Clone)]
pub struct EmbeddingResult {
    /// Generated embeddings (1536 dimensions for text-embedding-3-small)
    pub embeddings: Vec<Vec<f32>>,
    /// Total tokens consumed by API
    pub tokens_used: usize,
    /// Estimated cost in USD
    pub cost: f64,
    /// Model name used
    pub model: String,
}

/// OpenAI API request structure
#[derive(Serialize)]
struct EmbeddingRequest {
    input: Vec<String>,
    model: String,
}

/// OpenAI API response structure
#[derive(Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
    usage: Usage,
    model: String,
}

#[derive(Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
    index: usize,
}

#[derive(Deserialize)]
struct Usage {
    total_tokens: usize,
}

impl OpenAIEmbeddingsClient {
    /// Create new OpenAI embeddings client
    ///
    /// # Arguments
    /// * `api_key` - OpenAI API key (must start with "sk-")
    ///
    /// # Returns
    /// Client instance or error if API key invalid
    pub fn new(api_key: &str) -> Result<Self, String> {
        // Validate API key
        if api_key.is_empty() {
            return Err("API key cannot be empty".to_string());
        }
        if !api_key.starts_with("sk-") {
            return Err("API key must start with 'sk-'".to_string());
        }
        if api_key.len() < 20 {
            return Err("API key too short".to_string());
        }

        Ok(Self {
            api_key: api_key.to_string(),
            model: "text-embedding-3-small".to_string(),
            max_retries: 3,
            base_delay_ms: 1000,
            client: Client::new(),
        })
    }

    /// Generate embedding for single text
    ///
    /// # Arguments
    /// * `text` - Text to embed (code chunk, documentation, etc.)
    ///
    /// # Returns
    /// Embedding result with 1536-dimensional vector
    pub async fn embed(&self, text: &str) -> Result<EmbeddingResult, String> {
        if text.trim().is_empty() {
            return Err("Empty text cannot be embedded".to_string());
        }

        self.embed_batch(&[text]).await
    }

    /// Generate embeddings for multiple texts in batch
    ///
    /// # Arguments
    /// * `texts` - Array of texts to embed (max 100)
    ///
    /// # Returns
    /// Embedding result with multiple vectors
    pub async fn embed_batch(&self, texts: &[&str]) -> Result<EmbeddingResult, String> {
        // Validate batch size
        if texts.is_empty() {
            return Err("Batch cannot be empty".to_string());
        }
        if texts.len() > 100 {
            return Err(format!(
                "Batch size exceeds maximum of 100 (got {})",
                texts.len()
            ));
        }

        // Validate all texts are non-empty
        for (i, text) in texts.iter().enumerate() {
            if text.trim().is_empty() {
                return Err(format!("Text at index {i} is empty"));
            }
        }

        // Prepare request
        let request = EmbeddingRequest {
            input: texts.iter().map(|t| t.to_string()).collect(),
            model: self.model.clone(),
        };

        // Execute with retries
        let response = self.execute_with_retry(&request).await?;

        // Sort embeddings by index (API may return out of order)
        let mut sorted_data = response.data;
        sorted_data.sort_by_key(|d| d.index);

        // Extract embeddings
        let embeddings: Vec<Vec<f32>> = sorted_data.into_iter().map(|d| d.embedding).collect();

        // Calculate cost
        let cost = Self::calculate_cost(response.usage.total_tokens, &self.model);

        Ok(EmbeddingResult {
            embeddings,
            tokens_used: response.usage.total_tokens,
            cost,
            model: response.model,
        })
    }

    /// Execute API request with exponential backoff retry
    async fn execute_with_retry(
        &self,
        request: &EmbeddingRequest,
    ) -> Result<EmbeddingResponse, String> {
        let mut attempts = 0;

        loop {
            let response = self
                .client
                .post("https://api.openai.com/v1/embeddings")
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(request)
                .send()
                .await
                .map_err(|e| format!("API request failed: {e}"))?;

            let status = response.status();

            // Success
            if status.is_success() {
                return response
                    .json::<EmbeddingResponse>()
                    .await
                    .map_err(|e| format!("Failed to parse response: {e}"));
            }

            // Rate limit - retry with exponential backoff
            if status.as_u16() == 429 {
                attempts += 1;
                if attempts > self.max_retries {
                    return Err("Rate limit exceeded, max retries reached".to_string());
                }

                let delay = self.base_delay_ms * 2_u64.pow(attempts - 1);
                sleep(Duration::from_millis(delay)).await;
                continue;
            }

            // Unauthorized
            if status.as_u16() == 401 {
                return Err("Unauthorized: Invalid API key".to_string());
            }

            // Other errors
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("API error ({}): {}", status, error_body));
        }
    }

    /// Calculate cost for given token count and model
    ///
    /// # Arguments
    /// * `tokens` - Number of tokens consumed
    /// * `model` - Model name
    ///
    /// # Returns
    /// Cost in USD
    fn calculate_cost(tokens: usize, model: &str) -> f64 {
        let cost_per_1k = match model {
            "text-embedding-3-small" => 0.00002,
            "text-embedding-3-large" => 0.00013,
            "text-embedding-ada-002" => 0.0001,
            _ => 0.00002, // Default to small model
        };

        (tokens as f64 / 1000.0) * cost_per_1k
    }

    /// Get model name
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Get max retries
    pub fn max_retries(&self) -> u32 {
        self.max_retries
    }

    /// Get base delay in milliseconds
    pub fn base_delay_ms(&self) -> u64 {
        self.base_delay_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_validation() {
        assert!(OpenAIEmbeddingsClient::new("").is_err());
        assert!(OpenAIEmbeddingsClient::new("invalid").is_err());
        assert!(OpenAIEmbeddingsClient::new("sk-").is_err());
        assert!(OpenAIEmbeddingsClient::new("sk-abc123def456ghi789jkl012mno345pqr678stu901vwx234").is_ok());
    }

    #[test]
    fn test_cost_calculation() {
        assert_eq!(OpenAIEmbeddingsClient::calculate_cost(1000, "text-embedding-3-small"), 0.00002);
        assert_eq!(OpenAIEmbeddingsClient::calculate_cost(5000, "text-embedding-3-small"), 0.0001);
        assert_eq!(OpenAIEmbeddingsClient::calculate_cost(1000, "text-embedding-3-large"), 0.00013);
    }

    #[test]
    fn test_client_configuration() {
        let client = OpenAIEmbeddingsClient::new("sk-test1234567890abcdefghijklmnopqrstuvwxyz").unwrap();
        assert_eq!(client.model(), "text-embedding-3-small");
        assert_eq!(client.max_retries(), 3);
        assert_eq!(client.base_delay_ms(), 1000);
    }
}
