# PMAT-SEARCH-006: MCP Tools Integration

**Sprint**: 30
**Status**: 🔴 RED PHASE
**Estimated**: 2.5 hours
**Actual**: TBD

## 🎯 Objective

Expose semantic search capabilities to AI assistants via MCP (Model Context Protocol) tools, enabling natural language code discovery and analysis.

## 📋 Requirements

**Must Support:**
- 4 new MCP tools for semantic search
- Integration with existing HybridSearchEngine
- Proper input validation and error handling
- JSON schema for all tool parameters
- Result formatting optimized for AI consumption

**MCP Tools:**

1. **semantic_search** - Search code by natural language query
   - Parameters: query (string), mode (keyword|vector|hybrid), language (optional), limit (optional)
   - Returns: Ranked search results with scores

2. **find_similar_code** - Find code similar to a reference file
   - Parameters: file_path (string), limit (optional)
   - Returns: Similar code chunks with similarity scores

3. **cluster_code** - Group code by semantic similarity
   - Parameters: method (kmeans|hierarchical|dbscan), k (clusters), language (optional)
   - Returns: Code clusters with centroids

4. **analyze_topics** - Extract topics from codebase
   - Parameters: num_topics (integer), language (optional)
   - Returns: Topics with representative code examples

## 🔴 RED Phase: Tests First

### Test Suite

```rust
// tests/unit_mcp_semantic_tools.rs

#[tokio::test]
async fn test_semantic_search_tool() {
    let tool = SemanticSearchTool::new(setup_engine().await);

    let params = json!({
        "query": "function that adds two numbers",
        "mode": "hybrid",
        "limit": 5
    });

    let result = tool.execute(params).await?;

    assert!(result["results"].is_array());
    assert!(result["results"].as_array().unwrap().len() <= 5);

    // Check result structure
    let first = &result["results"][0];
    assert!(first["file_path"].is_string());
    assert!(first["chunk_name"].is_string());
    assert!(first["score"].is_number());
}

#[tokio::test]
async fn test_semantic_search_validation() {
    let tool = SemanticSearchTool::new(setup_engine().await);

    // Empty query should fail
    let params = json!({
        "query": "",
        "mode": "hybrid"
    });

    let result = tool.execute(params).await;
    assert!(result.is_err());

    // Invalid mode should fail
    let params = json!({
        "query": "test",
        "mode": "invalid_mode"
    });

    let result = tool.execute(params).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_find_similar_code_tool() {
    let tool = FindSimilarCodeTool::new(setup_engine().await);

    let params = json!({
        "file_path": "src/main.rs",
        "limit": 3
    });

    let result = tool.execute(params).await?;

    assert!(result["results"].is_array());
    assert!(result["results"].as_array().unwrap().len() <= 3);
}

#[tokio::test]
async fn test_cluster_code_tool() {
    let tool = ClusterCodeTool::new(setup_engine().await);

    let params = json!({
        "method": "kmeans",
        "k": 3
    });

    let result = tool.execute(params).await?;

    assert!(result["clusters"].is_array());
    assert_eq!(result["clusters"].as_array().unwrap().len(), 3);

    // Each cluster should have code chunks
    let cluster = &result["clusters"][0];
    assert!(cluster["id"].is_number());
    assert!(cluster["chunks"].is_array());
}

#[tokio::test]
async fn test_analyze_topics_tool() {
    let tool = AnalyzeTopicsTool::new(setup_engine().await);

    let params = json!({
        "num_topics": 5
    });

    let result = tool.execute(params).await?;

    assert!(result["topics"].is_array());
    assert_eq!(result["topics"].as_array().unwrap().len(), 5);

    // Each topic should have keywords and examples
    let topic = &result["topics"][0];
    assert!(topic["id"].is_number());
    assert!(topic["keywords"].is_array());
    assert!(topic["examples"].is_array());
}

#[test]
fn test_tool_schemas() {
    let semantic_search = SemanticSearchTool::schema();
    assert_eq!(semantic_search["name"], "semantic_search");
    assert!(semantic_search["parameters"]["properties"]["query"].is_object());

    let find_similar = FindSimilarCodeTool::schema();
    assert_eq!(find_similar["name"], "find_similar_code");

    let cluster = ClusterCodeTool::schema();
    assert_eq!(cluster["name"], "cluster_code");

    let topics = AnalyzeTopicsTool::schema();
    assert_eq!(topics["name"], "analyze_topics");
}
```

**Total Tests**: 20
- semantic_search tool (5 tests)
- find_similar_code tool (4 tests)
- cluster_code tool (4 tests)
- analyze_topics tool (4 tests)
- Schema validation (3 tests)

## 🟢 GREEN Phase: Implementation

**File**: `server/src/mcp/tools/semantic_search_tools.rs`

**Key Structures:**

```rust
pub struct SemanticSearchTool {
    engine: Arc<HybridSearchEngine>,
}

pub struct FindSimilarCodeTool {
    engine: Arc<HybridSearchEngine>,
}

pub struct ClusterCodeTool {
    engine: Arc<HybridSearchEngine>,
}

pub struct AnalyzeTopicsTool {
    engine: Arc<HybridSearchEngine>,
}

// Tool trait
#[async_trait]
pub trait McpTool {
    fn name(&self) -> &str;
    fn schema(&self) -> serde_json::Value;
    async fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value, String>;
}
```

**Implementation Strategy:**
1. Implement McpTool trait for each tool
2. JSON schema for input validation
3. Call HybridSearchEngine methods
4. Format results as JSON for AI consumption
5. Add rich metadata (scores, explanations)

## 🔵 REFACTOR Phase: Quality

**Complexity Target**: ≤10 cyclomatic per function
**Coverage Target**: ≥95%
**SATD**: 0 violations

**Refactoring checklist:**
- Extract JSON formatting to helper functions
- Extract validation logic to separate module
- Add result caching for repeated queries
- Add usage tracking/telemetry
- Document JSON schemas with examples

## ✅ Exit Criteria

- [ ] 20 tests passing
- [ ] 4 MCP tools fully implemented
- [ ] JSON schemas defined for all tools
- [ ] Input validation working
- [ ] Results formatted for AI consumption
- [ ] Integration with HybridSearchEngine
- [ ] Cyclomatic ≤10 for all functions
- [ ] Zero clippy warnings

## 📊 MCP Tool Specifications

### 1. semantic_search

**Purpose**: Search code by natural language query

**Parameters**:
```json
{
  "query": "string (required) - Natural language search query",
  "mode": "string (optional) - Search mode: keyword|vector|hybrid (default: hybrid)",
  "language": "string (optional) - Filter by language: rust|typescript|python|c|cpp|go",
  "limit": "integer (optional) - Max results (default: 10, max: 100)"
}
```

**Response**:
```json
{
  "results": [
    {
      "file_path": "src/math.rs",
      "chunk_name": "add",
      "chunk_type": "function",
      "language": "rust",
      "score": 0.95,
      "keyword_score": 0.92,
      "vector_score": 0.98,
      "snippet": "fn add(a: i32, b: i32) -> i32 { a + b }",
      "start_line": 10,
      "end_line": 12
    }
  ],
  "total": 1,
  "mode": "hybrid",
  "query_time_ms": 245
}
```

### 2. find_similar_code

**Purpose**: Find code similar to a reference file/function

**Parameters**:
```json
{
  "file_path": "string (required) - Path to reference file",
  "limit": "integer (optional) - Max results (default: 5, max: 50)"
}
```

**Response**:
```json
{
  "results": [
    {
      "file_path": "src/multiply.rs",
      "chunk_name": "multiply",
      "similarity": 0.87,
      "snippet": "fn multiply(a: i32, b: i32) -> i32 { a * b }"
    }
  ],
  "reference_file": "src/add.rs",
  "total": 1
}
```

### 3. cluster_code

**Purpose**: Group code by semantic similarity

**Parameters**:
```json
{
  "method": "string (required) - Clustering method: kmeans|hierarchical|dbscan",
  "k": "integer (required for kmeans) - Number of clusters",
  "language": "string (optional) - Filter by language"
}
```

**Response**:
```json
{
  "clusters": [
    {
      "id": 0,
      "size": 15,
      "centroid": "Mathematical operations",
      "chunks": [
        {
          "file_path": "src/math.rs",
          "chunk_name": "add"
        }
      ]
    }
  ],
  "method": "kmeans",
  "total_chunks": 50,
  "total_clusters": 3
}
```

### 4. analyze_topics

**Purpose**: Extract semantic topics from codebase

**Parameters**:
```json
{
  "num_topics": "integer (required) - Number of topics to extract (1-20)",
  "language": "string (optional) - Filter by language"
}
```

**Response**:
```json
{
  "topics": [
    {
      "id": 0,
      "keywords": ["function", "calculate", "arithmetic"],
      "weight": 0.35,
      "examples": [
        {
          "file_path": "src/math.rs",
          "chunk_name": "add",
          "relevance": 0.92
        }
      ]
    }
  ],
  "num_topics": 5,
  "coverage": 0.87
}
```

## 🔗 Integration

Will be used by:
- AI assistants via MCP protocol
- Claude Code, Cursor, other MCP clients
- PMAT-SEARCH-009: CLI commands (may wrap MCP tools)

## 💡 Usage Examples

**From AI Assistant:**

```
User: "Find functions that handle error cases"

AI uses: semantic_search(query="error handling functions", mode="hybrid")

AI: "I found 12 functions handling errors:
1. src/parser.rs:parse_with_recovery (score: 0.95)
2. src/validator.rs:validate_input (score: 0.89)
..."
```

```
User: "What code is similar to my authentication module?"

AI uses: find_similar_code(file_path="src/auth.rs", limit=5)

AI: "Similar code found:
1. src/oauth.rs - OAuth authentication (87% similar)
2. src/jwt.rs - JWT token handling (79% similar)
..."
```

## 📚 References

- **MCP Specification**: https://spec.modelcontextprotocol.io/
- **PMAT MCP Integration**: `server/src/mcp/` (existing infrastructure)
- **pmcp SDK**: v1.4.2 with full tool support
