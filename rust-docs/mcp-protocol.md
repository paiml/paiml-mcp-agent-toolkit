# MCP Wire Protocol Specification

## Transport Layer

- **Protocol**: JSON-RPC 2.0 over STDIO
- **Framing**: Line-delimited JSON (LF = 0x0A)
- **Encoding**: UTF-8 without BOM
- **Buffer Size**: 64KB read buffer, 8KB write buffer
- **Backpressure**: Token bucket rate limiting (1000 req/s)

## Connection Flow

```
Client                                  Server
  |                                       |
  |-------- initialize request -------->  |
  |<------- initialize response -------   |
  |                                       |
  |-------- tools/list request -------->  |
  |<------- tools/list response -------   |
  |                                       |
  |-------- resources/list req -------->  |
  |<------- resources/list resp -------   |
  |                                       |
  |-------- prompts/list request ------>  |
  |<------- prompts/list response -----   |
  |                                       |
  |-------- tools/call request -------->  |
  |<------- tools/call response -------   |
  |                                       |
```

## State Machine

```rust
#[derive(Debug, Clone, Copy)]
enum ConnectionState {
    Uninitialized,
    Initializing { capabilities: bool },
    Ready,
    Shutting { graceful: bool },
    Closed,
}

impl ConnectionState {
    fn transition(&mut self, event: Event) -> Result<(), ProtocolError> {
        use ConnectionState::*;
        *self = match (*self, event) {
            (Uninitialized, Event::Initialize) => Initializing { capabilities: false },
            (Initializing { .. }, Event::Initialized) => Ready,
            (Ready, Event::Shutdown) => Shutting { graceful: true },
            (_, Event::Error) => Shutting { graceful: false },
            _ => return Err(ProtocolError::InvalidTransition),
        };
        Ok(())
    }
}
```

## Message Format

### Request Structure

```json
{
    "jsonrpc": "2.0",
    "id": "unique-request-id",
    "method": "method-name",
    "params": { /* method-specific parameters */ }
}
```

### Response Structure

Success:
```json
{
    "jsonrpc": "2.0",
    "id": "unique-request-id",
    "result": { /* method-specific result */ }
}
```

Error:
```json
{
    "jsonrpc": "2.0",
    "id": "unique-request-id",
    "error": {
        "code": -32601,
        "message": "Method not found",
        "data": { /* optional error details */ }
    }
}
```

## Core Methods

### `initialize`

Establishes connection and negotiates capabilities.

**Request:**
```json
{
    "jsonrpc": "2.0",
    "id": "1",
    "method": "initialize",
    "params": {
        "protocolVersion": "2024-11-05",
        "capabilities": {},
        "clientInfo": {
            "name": "claude-code",
            "version": "1.0.0"
        }
    }
}
```

**Response:**
```json
{
    "jsonrpc": "2.0",
    "id": "1",
    "result": {
        "protocolVersion": "2024-11-05",
        "capabilities": {},
        "serverInfo": {
            "name": "paiml-mcp-agent-toolkit",
            "version": "0.5.3"
        }
    }
}
```

### `tools/list`

Lists available tools with their schemas.

**Request:**
```json
{
    "jsonrpc": "2.0",
    "id": "2",
    "method": "tools/list",
    "params": {}
}
```

**Response:**
```json
{
    "jsonrpc": "2.0",
    "id": "2",
    "result": {
        "tools": [
            {
                "name": "generate_template",
                "description": "Generate a template from a URI",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "resource_uri": {
                            "type": "string",
                            "description": "Template URI (e.g., template://makefile/rust/cli)"
                        },
                        "parameters": {
                            "type": "object",
                            "description": "Template parameters"
                        }
                    },
                    "required": ["resource_uri", "parameters"]
                }
            }
        ]
    }
}
```

### `resources/list`

Lists available resources (templates).

**Request:**
```json
{
    "jsonrpc": "2.0",
    "id": "3",
    "method": "resources/list",
    "params": {}
}
```

**Response:**
```json
{
    "jsonrpc": "2.0",
    "id": "3",
    "result": {
        "resources": [
            {
                "uri": "template://makefile/rust/cli",
                "name": "Rust CLI Makefile",
                "description": "Professional Makefile template for Rust CLI projects",
                "mimeType": "text/x-makefile"
            }
        ]
    }
}
```

### `tools/call`

Executes a tool with provided arguments.

**Request:**
```json
{
    "jsonrpc": "2.0",
    "id": "4",
    "method": "tools/call",
    "params": {
        "name": "generate_template",
        "arguments": {
            "resource_uri": "template://makefile/rust/cli",
            "parameters": {
                "project_name": "my-project",
                "has_tests": true
            }
        }
    }
}
```

**Response:**
```json
{
    "jsonrpc": "2.0",
    "id": "4",
    "result": {
        "content": [
            {
                "type": "text",
                "text": "# Generated Makefile content..."
            }
        ]
    }
}
```

## Performance Optimizations

### 1. **Zero-Copy JSON Parsing**

Using optimized JSON parsing to minimize allocations:

```rust
// Standard parsing (with allocations)
let value: Value = serde_json::from_str(&input)?;

// Optimized parsing (zero-copy where possible)
let mut buffer = input.as_bytes();
let value = serde_json::from_slice(&mut buffer)?;
```

### 2. **Request Pipelining**

Supports up to 128 in-flight requests with tagged responses:

```rust
struct RequestTracker {
    pending: DashMap<RequestId, oneshot::Sender<Response>>,
    next_id: AtomicU64,
}

impl RequestTracker {
    fn register(&self) -> (RequestId, oneshot::Receiver<Response>) {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = oneshot::channel();
        self.pending.insert(id, tx);
        (id, rx)
    }
    
    fn complete(&self, id: RequestId, response: Response) {
        if let Some((_, tx)) = self.pending.remove(&id) {
            let _ = tx.send(response);
        }
    }
}
```

### 3. **Streaming Large Responses**

For large template outputs, streaming is used to reduce memory pressure:

```rust
async fn stream_large_response(content: &str, writer: &mut impl AsyncWrite) {
    const CHUNK_SIZE: usize = 8192;
    
    for chunk in content.as_bytes().chunks(CHUNK_SIZE) {
        writer.write_all(chunk).await?;
        writer.flush().await?;
    }
}
```

## Error Codes

Standard JSON-RPC 2.0 error codes:

| Code | Message | Description |
|------|---------|-------------|
| -32700 | Parse error | Invalid JSON |
| -32600 | Invalid request | Missing required fields |
| -32601 | Method not found | Unknown method |
| -32602 | Invalid params | Invalid method parameters |
| -32603 | Internal error | Server error |

Custom error codes:

| Code | Message | Description |
|------|---------|-------------|
| -32000 | Template not found | Requested template doesn't exist |
| -32001 | Invalid template URI | Malformed template URI |
| -32002 | Parameter validation failed | Template parameter validation error |
| -32003 | Rate limit exceeded | Too many requests |

## Available Tools

### `generate_template`

Generate a single template file.

**Arguments:**
- `resource_uri` (string, required): Template URI
- `parameters` (object, required): Template parameters

**Returns:**
- `content`: Generated template content

### `list_templates`

List available templates with optional filtering.

**Arguments:**
- `toolchain` (string, optional): Filter by toolchain
- `category` (string, optional): Filter by category

**Returns:**
- Array of template resources

### `validate_template`

Validate template parameters without generation.

**Arguments:**
- `resource_uri` (string, required): Template URI
- `parameters` (object, required): Parameters to validate

**Returns:**
- `valid` (boolean): Whether parameters are valid
- `errors` (array): Validation errors if any

### `scaffold_project`

Generate multiple templates for project scaffolding.

**Arguments:**
- `toolchain` (string, required): Target toolchain
- `templates` (array, required): Template names to generate
- `parameters` (object, required): Shared parameters

**Returns:**
- `files`: Array of generated files with paths and content

### `search_templates`

Search templates by query string.

**Arguments:**
- `query` (string, required): Search query
- `toolchain` (string, optional): Filter by toolchain

**Returns:**
- Array of search results with relevance scores

### `analyze_complexity`

Analyze code complexity metrics with optional file ranking.

**Arguments:**
- `project_path` (string, required): Path to analyze
- `toolchain` (string, required): Project toolchain
- `format` (string, optional): Output format
- `max_cyclomatic` (number, optional): Cyclomatic threshold
- `max_cognitive` (number, optional): Cognitive threshold
- `top_files` (number, optional): Number of top complex files to show (0 = all violations)

**Returns:**
- Complexity analysis report with optional file rankings

**Example with file ranking:**
```json
{
    "jsonrpc": "2.0",
    "method": "analyze_complexity",
    "params": {
        "project_path": "./",
        "toolchain": "rust",
        "format": "json",
        "top_files": 5
    },
    "id": 1
}
```

**Response includes `top_files` section when requested:**
```json
{
    "jsonrpc": "2.0",
    "id": 1,
    "result": {
        "summary": {},
        "violations": [],
        "top_files": {
            "requested": 5,
            "returned": 5,
            "rankings": [
                {
                    "rank": 1,
                    "file": "./server/src/services/context.rs",
                    "function_count": 30,
                    "max_cyclomatic": 32,
                    "avg_cognitive": 5.8,
                    "halstead_effort": 1950.0,
                    "total_score": 30.92
                }
            ]
        }
    }
}
```

### `analyze_code_churn`

Analyze git history for code churn metrics.

**Arguments:**
- `project_path` (string, required): Path to analyze
- `period_days` (number, optional): Days to analyze (default: 30)
- `format` (string, optional): Output format

**Returns:**
- Code churn analysis report

### `analyze_deep_context` ✨ **NEW**

**NEW**: Generate comprehensive deep context analysis combining multiple analysis types into unified quality assessment.

**Arguments:**
- `project_path` (string, required): Path to analyze (default: current directory)
- `include_analyses` (array, optional): List of analyses to include. Available options:
  - `"ast"`: Abstract syntax tree parsing and symbol extraction
  - `"complexity"`: McCabe Cyclomatic and Cognitive complexity metrics
  - `"churn"`: Git history and change frequency tracking
  - `"dag"`: Dependency graph generation and visualization
  - `"dead-code"`: Unused code detection with confidence scoring
  - `"satd"`: Self-Admitted Technical Debt detection from comments
  - `"defect-probability"`: ML-based defect prediction and hotspot identification
- `exclude_analyses` (array, optional): List of analyses to exclude
- `period_days` (number, optional): Period for churn analysis (default: 30 days)
- `dag_type` (string, optional): DAG type for dependency analysis ("call-graph", "import-graph", "inheritance", "full-dependency")
- `max_depth` (number, optional): Maximum directory traversal depth
- `include_patterns` (array, optional): Include file patterns (e.g., ["src/**/*.rs", "tests/**/*.rs"])
- `exclude_patterns` (array, optional): Exclude file patterns (e.g., ["**/target/**", "**/node_modules/**"])
- `cache_strategy` (string, optional): Cache usage strategy ("normal", "force-refresh", "offline")
- `parallel` (number, optional): Parallelism level for analysis
- `format` (string, optional): Output format ("markdown", "json", "sarif")

**Returns:**
Comprehensive deep context analysis with:
- **Quality Scorecard**: Overall health score (0-100), maintainability index, technical debt estimation
- **Multi-Analysis Pipeline**: Combined results from all requested analysis types
- **Defect Correlation**: Cross-analysis insights and risk prediction  
- **Prioritized Recommendations**: AI-generated actionable improvement suggestions
- **Enhanced File Tree**: Annotated project structure with defect scores and metrics
- **Template Provenance**: Project scaffolding drift analysis (if applicable)
- **Cross-Language References**: FFI bindings, WASM exports, inter-language dependencies

**Example Request:**
```json
{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
        "name": "analyze_deep_context",
        "arguments": {
            "project_path": "./",
            "include_analyses": ["ast", "complexity", "churn", "satd"],
            "period_days": 30,
            "cache_strategy": "normal",
            "format": "json",
            "include_patterns": ["src/**/*.rs", "tests/**/*.rs"],
            "exclude_patterns": ["**/target/**"],
            "parallel": 8
        }
    },
    "id": 1
}
```

**Example Response Structure:**
```json
{
    "jsonrpc": "2.0",
    "id": 1,
    "result": {
        "content": [
            {
                "type": "text",
                "text": "{\"metadata\":{\"generated_at\":\"2025-06-01T...\",\"tool_version\":\"0.18.5\",\"analysis_duration\":{\"secs\":8,\"nanos\":245000000}},\"quality_scorecard\":{\"overall_health\":78.5,\"complexity_score\":65.2,\"maintainability_index\":82.1,\"technical_debt_hours\":45.2},\"file_tree\":{\"root\":{\"name\":\"project\",\"children\":[...]},\"total_files\":146,\"total_size_bytes\":1250000},\"analyses\":{\"ast_contexts\":[...],\"complexity_report\":{...},\"churn_analysis\":{...},\"satd_results\":{...}},\"hotspots\":[{\"location\":{\"file\":\"src/complex.rs\",\"line\":42},\"composite_score\":0.85,\"refactoring_effort\":{\"estimated_hours\":4.5,\"priority\":\"High\"}}],\"recommendations\":[{\"title\":\"Reduce Code Complexity\",\"description\":\"Several functions exceed complexity thresholds...\",\"priority\":\"High\",\"estimated_effort\":{\"secs\":28800},\"impact\":\"High\"}]}"
            }
        ]
    }
}
```

**Quality Scorecard Features:**
- **Overall Health Score** (0-100): Composite quality assessment
- **Maintainability Index**: Code maintainability metrics based on complexity and churn
- **Technical Debt Hours**: Estimated effort to address identified debt items
- **Defect Correlation**: Cross-analysis insights for risk prediction

**Performance Characteristics:**
- **Parallel Execution**: Tokio-based concurrent analysis using JoinSet
- **Cache Integration**: Smart caching strategies for incremental analysis
- **Memory Efficiency**: Optimized data structures with streaming output
- **Analysis Time**: ~2.5ms for focused analysis, ~8 seconds for full project

**Output Format Support:**
- **JSON**: Structured data for API consumption and tool integration
- **Markdown**: Human-readable comprehensive reports with annotated file trees
- **SARIF**: Static Analysis Results Interchange Format for IDE integration and CI/CD pipelines

### `analyze_duplicates` ✨ **NEW**

**NEW**: Detect code duplicates using SIMD-accelerated MinHash algorithms with four detection types.

**Arguments:**
- `project_path` (string, optional): Path to analyze (default: current directory)
- `detection_type` (string, optional): Type of detection ("exact", "renamed", "gapped", "semantic", "all")
- `threshold` (number, optional): Similarity threshold for semantic clones (0.0-1.0, default: 0.85)
- `gpu` (boolean, optional): Use GPU acceleration if available
- `perf` (boolean, optional): Output performance metrics
- `format` (string, optional): Output format ("summary", "detailed", "json", "sarif")
- `min_lines` (number, optional): Minimum lines of code for duplicate detection (default: 5)

**Detection Types:**
1. **Exact**: Identical code blocks
2. **Renamed**: Code with renamed variables/functions  
3. **Gapped**: Code with inserted/deleted lines
4. **Semantic**: Functionally equivalent code with different syntax
5. **All**: Comprehensive detection using all methods

**Returns:**
- Duplicate detection results with performance metrics (if requested)

**Example Request:**
```json
{
    "jsonrpc": "2.0",
    "method": "tools/call", 
    "params": {
        "name": "analyze_duplicates",
        "arguments": {
            "detection_type": "all",
            "threshold": 0.8,
            "perf": true,
            "format": "json"
        }
    },
    "id": 1
}
```

### `analyze_defect_probability` ✨ **NEW**

**NEW**: ML-based defect prediction using feature vectors and confidence scoring.

**Arguments:**
- `project_path` (string, optional): Path to analyze (default: current directory)
- `min_confidence` (number, optional): Minimum confidence threshold (0.0-1.0, default: 0.7)
- `explain` (boolean, optional): Include feature importance breakdown
- `sarif` (boolean, optional): Output SARIF format for IDE integration
- `format` (string, optional): Output format ("summary", "detailed", "json", "sarif")

**Feature Vectors:**
The ML model uses 6 primary feature vectors:
- **Complexity**: Cyclomatic and cognitive complexity metrics
- **Churn**: Code change frequency and recency
- **Duplication**: Code clone density and distribution  
- **Coupling**: Dependency coupling and cohesion metrics
- **Name Quality**: Identifier semantic quality scoring
- **Test Coverage**: Test coverage and quality metrics

**Returns:**
- Defect predictions with confidence scores and optional feature explanations

**Example Request:**
```json
{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
        "name": "analyze_defect_probability", 
        "arguments": {
            "min_confidence": 0.8,
            "explain": true,
            "format": "detailed"
        }
    },
    "id": 1
}
```

### `analyze_comprehensive` ✨ **NEW**

**NEW**: Multi-dimensional analysis combining all analysis types with parallel execution and performance metrics.

**Arguments:**
- `project_path` (string, optional): Path to analyze (default: current directory)
- `format` (string, optional): Output format ("summary", "detailed", "json", "markdown", "sarif")
- `include_duplicates` (boolean, optional): Enable duplicate detection analysis
- `include_dead_code` (boolean, optional): Enable dead code analysis  
- `include_defects` (boolean, optional): Enable defect prediction analysis
- `include_complexity` (boolean, optional): Enable complexity analysis
- `include_tdg` (boolean, optional): Enable TDG (Technical Debt Gradient) analysis
- `confidence_threshold` (number, optional): Minimum confidence threshold for predictions (default: 0.5)
- `min_lines` (number, optional): Minimum lines of code for analysis (default: 10)
- `include_patterns` (array, optional): Include file patterns (e.g., ["**/*.rs"])
- `exclude_patterns` (array, optional): Exclude file patterns (e.g., ["**/target/**"])
- `perf` (boolean, optional): Show performance metrics for each analysis component
- `executive_summary` (boolean, optional): Generate executive summary only (faster analysis)

**Performance Characteristics:**
- **Duplicate detection**: ~84ms for 200-file codebase
- **Dead code analysis**: ~7ms analysis time
- **Defect prediction**: ~45ms ML inference
- **Complexity analysis**: ~36ms processing
- **Total comprehensive**: ~143ms for full analysis

**Returns:**
- Comprehensive analysis results with optional performance breakdown

**Example Request:**
```json
{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
        "name": "analyze_comprehensive",
        "arguments": {
            "include_duplicates": true,
            "include_dead_code": true,
            "include_defects": true,
            "include_complexity": true,
            "include_tdg": true,
            "perf": true,
            "format": "detailed"
        }
    },
    "id": 1
}
```

### `analyze_graph_metrics` ✨ **NEW**

**NEW**: Vectorized graph analytics with PageRank and centrality computation.

**Arguments:**
- `project_path` (string, optional): Path to analyze (default: current directory)
- `metrics` (array, optional): Metrics to compute (["centrality", "pagerank", "clustering", "components", "all"])
- `pagerank_seeds` (array, optional): Personalized PageRank seed nodes
- `graphml` (boolean, optional): Export as GraphML format
- `format` (string, optional): Output format ("summary", "detailed", "json")

**Graph Metrics:**
- **Centrality**: Betweenness, closeness, and degree centrality
- **PageRank**: Authority scoring with personalization options
- **Clustering**: Clustering coefficient and modularity
- **Components**: Connected component analysis

**Returns:**
- Graph analytics results with optional GraphML export

**Example Request:**
```json
{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
        "name": "analyze_graph_metrics",
        "arguments": {
            "metrics": ["pagerank", "centrality"],
            "pagerank_seeds": ["main.rs", "lib.rs"],
            "format": "json"
        }
    },
    "id": 1
}
```

### `analyze_name_similarity` ✨ **NEW**

**NEW**: Semantic name similarity using embeddings and phonetic matching.

**Arguments:**
- `query` (string, required): Name to search for
- `project_path` (string, optional): Path to analyze (default: current directory)
- `top_k` (number, optional): Number of results (default: 10)
- `phonetic` (boolean, optional): Include phonetic matches
- `scope` (string, optional): Search scope ("functions", "types", "variables", "all")
- `format` (string, optional): Output format ("summary", "detailed", "json")

**Returns:**
- Similar names ranked by semantic similarity with optional phonetic matches

**Example Request:**
```json
{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
        "name": "analyze_name_similarity",
        "arguments": {
            "query": "calculateTotal",
            "scope": "functions",
            "top_k": 5,
            "phonetic": true
        }
    },
    "id": 1
}
```

### `quality_gate` ✨ **NEW**

**NEW**: Comprehensive quality checks with configurable thresholds for CI/CD integration.

**Arguments:**
- `project_path` (string, optional): Path to analyze (default: current directory)
- `complexity_threshold` (number, optional): Maximum allowed complexity (default: 10)
- `duplication_threshold` (number, optional): Maximum duplication percentage (default: 5.0)
- `coverage_threshold` (number, optional): Minimum test coverage (default: 80.0)
- `defect_threshold` (number, optional): Maximum defect probability (default: 0.3)
- `format` (string, optional): Output format ("summary", "detailed", "json", "sarif", "junit")
- `fail_on_violation` (boolean, optional): Return error code on quality gate failures

**Returns:**
- Quality gate results with pass/fail status and detailed metrics

**Example Request:**
```json
{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
        "name": "quality_gate",
        "arguments": {
            "complexity_threshold": 8,
            "duplication_threshold": 3.0,
            "coverage_threshold": 90.0,
            "fail_on_violation": true,
            "format": "junit"
        }
    },
    "id": 1
}
```

## Connection Management

### Heartbeat

No explicit heartbeat mechanism. Connection health determined by successful request/response cycles.

### Graceful Shutdown

Server supports graceful shutdown with in-flight request completion:

```rust
async fn shutdown_handler(tracker: Arc<RequestTracker>) {
    // Wait for pending requests with timeout
    let timeout = Duration::from_secs(30);
    let start = Instant::now();
    
    while !tracker.pending.is_empty() && start.elapsed() < timeout {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    // Force close remaining
    tracker.pending.clear();
}
```

## Security Considerations

1. **Input Validation**: All inputs validated before processing
2. **Path Traversal**: Template URIs sanitized to prevent directory traversal
3. **Resource Limits**: 
   - Max request size: 1MB
   - Max response size: 10MB
   - Max concurrent requests: 128
4. **Rate Limiting**: Token bucket with 1000 req/s burst, 100 req/s sustained