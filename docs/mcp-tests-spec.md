# MCP Protocol Test Specification Extension

## MCP Testing Architecture

### Protocol Layer Testing

```rust
// MCP protocol test harness with bidirectional STDIO simulation
pub struct McpTestHarness {
    server_process: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    request_counter: AtomicU64,
    pending_requests: Arc<Mutex<HashMap<u64, oneshot::Sender<Value>>>>,
}

impl McpTestHarness {
    pub async fn send_request(&mut self, method: &str, params: Value) -> Result<Value> {
        let id = self.request_counter.fetch_add(1, Ordering::SeqCst);
        let request = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        });
        
        // Write with proper framing
        let serialized = serde_json::to_vec(&request)?;
        self.stdin.write_all(&serialized).await?;
        self.stdin.write_all(b"\n").await?;
        self.stdin.flush().await?;
        
        // Correlate response
        let (tx, rx) = oneshot::channel();
        self.pending_requests.lock().unwrap().insert(id, tx);
        
        tokio::time::timeout(Duration::from_secs(5), rx).await?
    }
}
```

### MCP Conformance Test Suite

```rust
mod mcp_conformance {
    use super::*;
    
    #[tokio::test]
    async fn test_jsonrpc_version_enforcement() {
        let mut harness = McpTestHarness::spawn().await;
        
        // Missing version
        let response = harness.send_raw(json!({
            "id": 1,
            "method": "initialize"
        })).await;
        
        assert_error_code(&response, -32600); // Invalid Request
        
        // Wrong version
        let response = harness.send_raw(json!({
            "jsonrpc": "1.0",
            "id": 2,
            "method": "initialize"
        })).await;
        
        assert_error_code(&response, -32600);
    }
    
    #[tokio::test]
    async fn test_request_id_correlation() {
        let mut harness = McpTestHarness::spawn().await;
        
        // Send multiple concurrent requests
        let futures: Vec<_> = (0..100).map(|i| {
            let mut h = harness.clone();
            tokio::spawn(async move {
                let response = h.send_request("resources/list", json!({})).await.unwrap();
                (i, response["id"].as_u64().unwrap())
            })
        }).collect();
        
        let results = futures::future::join_all(futures).await;
        
        // Verify each request got its matching response
        for (sent_id, received_id) in results {
            assert_eq!(sent_id as u64, received_id);
        }
    }
}
```

## MCP Method-Specific Tests

### Initialize Handshake Testing

```rust
#[tokio::test]
async fn test_initialize_capability_negotiation() {
    let mut harness = McpTestHarness::spawn().await;
    
    let response = harness.send_request("initialize", json!({
        "protocolVersion": "1.0",
        "capabilities": {
            "roots": true,
            "experimental": {
                "streaming": true
            }
        },
        "clientInfo": {
            "name": "test-client",
            "version": "1.0.0"
        }
    })).await.unwrap();
    
    // Verify server capabilities
    let result = &response["result"];
    assert_eq!(result["protocolVersion"], "1.0");
    
    let capabilities = &result["capabilities"];
    assert!(capabilities["tools"].is_object());
    assert!(capabilities["resources"].is_object());
    assert!(capabilities["prompts"].is_object());
    
    // Verify all 10 tools exposed
    let tools = harness.send_request("tools/list", json!({})).await.unwrap();
    let tool_count = tools["result"]["tools"].as_array().unwrap().len();
    assert_eq!(tool_count, 10);
}

#[tokio::test] 
async fn test_initialize_idempotency() {
    let mut harness = McpTestHarness::spawn().await;
    
    // First initialization
    let response1 = harness.send_request("initialize", json!({
        "protocolVersion": "1.0"
    })).await.unwrap();
    
    // Second initialization should fail
    let response2 = harness.send_request("initialize", json!({
        "protocolVersion": "1.0"
    })).await;
    
    assert!(response2.is_err() || response2.unwrap()["error"].is_object());
}
```

### Tool Invocation Testing

```rust
#[tokio::test]
async fn test_tool_call_parameter_validation() {
    let mut harness = McpTestHarness::spawn_initialized().await;
    
    // Test each validation path
    let test_cases = vec![
        // Missing required parameter
        (
            json!({
                "name": "generate_template",
                "arguments": {
                    "resource_uri": "template://makefile/rust/cli"
                    // missing parameters
                }
            }),
            -32602, // Invalid params
            "Missing required parameter: parameters"
        ),
        // Invalid parameter type
        (
            json!({
                "name": "scaffold_project",
                "arguments": {
                    "toolchain": "rust",
                    "templates": "makefile", // Should be array
                    "parameters": {}
                }
            }),
            -32602,
            "templates must be an array"
        ),
        // Unknown tool
        (
            json!({
                "name": "nonexistent_tool",
                "arguments": {}
            }),
            -32601, // Method not found
            "Unknown tool"
        ),
    ];
    
    for (params, expected_code, expected_msg) in test_cases {
        let response = harness.send_request("tools/call", params).await.unwrap();
        let error = &response["error"];
        assert_eq!(error["code"], expected_code);
        assert!(error["message"].as_str().unwrap().contains(expected_msg));
    }
}
```

### Resource Handling Tests

```rust
#[tokio::test]
async fn test_resource_list_filtering() {
    let mut harness = McpTestHarness::spawn_initialized().await;
    
    // Unfiltered list
    let all_resources = harness.send_request("resources/list", json!({}))
        .await.unwrap();
    let all_count = all_resources["result"]["resources"].as_array().unwrap().len();
    assert_eq!(all_count, 9); // All 9 templates
    
    // Filtered by toolchain
    let rust_resources = harness.send_request("resources/list", json!({
        "toolchain": "rust"
    })).await.unwrap();
    let rust_count = rust_resources["result"]["resources"].as_array().unwrap().len();
    assert_eq!(rust_count, 3); // makefile, readme, gitignore
    
    // Verify URI structure
    for resource in rust_resources["result"]["resources"].as_array().unwrap() {
        let uri = resource["uri"].as_str().unwrap();
        assert!(uri.starts_with("template://"));
        assert!(uri.contains("/rust/"));
    }
}

#[tokio::test]
async fn test_resource_read_content_integrity() {
    let mut harness = McpTestHarness::spawn_initialized().await;
    
    let response = harness.send_request("resources/read", json!({
        "uri": "template://makefile/rust/cli"
    })).await.unwrap();
    
    let contents = response["result"]["contents"][0]["text"].as_str().unwrap();
    
    // Verify template markers present
    assert!(contents.contains("{{project_name}}"));
    assert!(contents.contains("cargo build --release"));
    
    // Verify no rendering occurred
    assert!(!contents.contains("{{#if"));
    assert!(contents.len() > 1000); // Non-trivial template
}
```

## STDIO Communication Edge Cases

### Message Framing Tests

```rust
#[tokio::test]
async fn test_stdio_message_framing() {
    let server = Command::cargo_bin("paiml-mcp-agent-toolkit")
        .unwrap()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
        
    let mut stdin = server.stdin.unwrap();
    let stdout = BufReader::new(server.stdout.unwrap());
    
    // Test multiple messages in single write
    let batch = format!(
        "{}\n{}\n{}\n",
        json!({"jsonrpc": "2.0", "id": 1, "method": "tools/list"}),
        json!({"jsonrpc": "2.0", "id": 2, "method": "resources/list"}),
        json!({"jsonrpc": "2.0", "id": 3, "method": "prompts/list"})
    );
    
    stdin.write_all(batch.as_bytes()).unwrap();
    stdin.flush().unwrap();
    
    // Verify all three responses received
    let mut responses = Vec::new();
    for _ in 0..3 {
        let mut line = String::new();
        stdout.read_line(&mut line).unwrap();
        let response: Value = serde_json::from_str(&line).unwrap();
        responses.push(response["id"].as_u64().unwrap());
    }
    
    responses.sort();
    assert_eq!(responses, vec![1, 2, 3]);
}

#[tokio::test]
async fn test_large_payload_handling() {
    let mut harness = McpTestHarness::spawn_initialized().await;
    
    // Generate large parameter set
    let mut large_params = Map::new();
    for i in 0..1000 {
        large_params.insert(
            format!("param_{}", i),
            Value::String("x".repeat(1024)) // 1KB per param
        );
    }
    
    // Should handle ~1MB payload
    let response = harness.send_request("tools/call", json!({
        "name": "validate_template",
        "arguments": {
            "resource_uri": "template://makefile/rust/cli",
            "parameters": large_params
        }
    })).await.unwrap();
    
    // Verify graceful handling
    assert!(response["result"].is_object() || response["error"].is_object());
}
```

### Concurrent Request Handling

```rust
#[tokio::test]
async fn test_concurrent_request_processing() {
    let mut harness = McpTestHarness::spawn_initialized().await;
    
    // Spawn 100 concurrent tool calls
    let start = Instant::now();
    let futures: Vec<_> = (0..100).map(|i| {
        let h = harness.clone();
        tokio::spawn(async move {
            let response = h.send_request("tools/call", json!({
                "name": "generate_template",
                "arguments": {
                    "resource_uri": "template://makefile/rust/cli",
                    "parameters": {
                        "project_name": format!("concurrent_test_{}", i)
                    }
                }
            })).await.unwrap();
            
            response["result"]["content"].as_str().unwrap().len()
        })
    }).collect();
    
    let results = futures::future::join_all(futures).await;
    let duration = start.elapsed();
    
    // All should succeed
    assert!(results.iter().all(|r| r.is_ok()));
    
    // Should complete in reasonable time (not serialized)
    assert!(duration.as_secs() < 5, "Took {} seconds", duration.as_secs());
}
```

## Error Injection Testing

### Protocol Error Simulation

```rust
#[tokio::test]
async fn test_malformed_json_handling() {
    let mut harness = McpTestHarness::spawn().await;
    
    // Various malformed inputs
    let malformed_inputs = vec![
        b"not json at all\n",
        b"{\"jsonrpc\": \"2.0\", \"method\": \"test\", invalid\n",
        b"\x00\x01\x02\x03\n", // Binary data
        b"{}\n", // Empty object
        b"null\n",
    ];
    
    for input in malformed_inputs {
        harness.stdin.write_all(input).await.unwrap();
        harness.stdin.flush().await.unwrap();
        
        let response = harness.read_response().await;
        assert_error_code(&response, -32700); // Parse error
    }
}

#[tokio::test]
async fn test_error_response_format() {
    let mut harness = McpTestHarness::spawn_initialized().await;
    
    let response = harness.send_request("tools/call", json!({
        "name": "generate_template",
        "arguments": {
            "resource_uri": "template://invalid/uri",
            "parameters": {}
        }
    })).await.unwrap();
    
    // Verify error response structure per JSON-RPC spec
    assert!(response["result"].is_null());
    let error = &response["error"];
    assert!(error["code"].is_i64());
    assert!(error["message"].is_string());
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["id"].is_u64());
}
```

## Performance Benchmarks for MCP

### Protocol Overhead Measurement

```rust
fn benchmark_mcp_overhead(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("mcp_request_response_cycle", |b| {
        b.to_async(&rt).iter(|| async {
            let mut harness = McpTestHarness::cached().await;
            
            let response = harness.send_request("tools/list", json!({})).await.unwrap();
            black_box(response);
        });
    });
}

fn benchmark_concurrent_mcp_load(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("mcp_100_concurrent_requests", |b| {
        b.to_async(&rt).iter(|| async {
            let harness = Arc::new(McpTestHarness::spawn_initialized().await);
            
            let futures: Vec<_> = (0..100).map(|_| {
                let h = harness.clone();
                tokio::spawn(async move {
                    h.send_request("resources/list", json!({})).await
                })
            }).collect();
            
            futures::future::join_all(futures).await;
        });
    });
}
```

### MCP Performance Requirements

| Metric | Target | Measurement |
|--------|--------|-------------|
| Single request RTT | <10ms | End-to-end timing |
| Concurrent requests (100) | <100ms total | Parallel execution test |
| Large payload (1MB) | <50ms | Serialization + transport |
| Initialize handshake | <20ms | Cold start timing |
| Memory per connection | <1MB | RSS monitoring |
| CPU usage (idle) | <1% | Process monitoring |

## Integration Testing with Real MCP Clients

### Claude Code Simulation

```rust
pub struct ClaudeSimulator {
    process: Child,
    capabilities: Value,
}

impl ClaudeSimulator {
    pub async fn new() -> Self {
        let mut sim = Self::spawn().await;
        sim.initialize().await;
        sim
    }
    
    pub async fn test_tool_workflow(&mut self, tool_name: &str) -> Result<()> {
        // 1. List tools
        let tools = self.request("tools/list", json!({})).await?;
        
        // 2. Find specific tool
        let tool = tools["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["name"] == tool_name)
            .expect("Tool not found");
            
        // 3. Validate schema
        let input_schema = &tool["inputSchema"];
        assert_eq!(input_schema["type"], "object");
        
        // 4. Call with valid parameters
        let response = self.request("tools/call", json!({
            "name": tool_name,
            "arguments": self.generate_valid_args(input_schema)
        })).await?;
        
        assert!(response["result"].is_object());
        Ok(())
    }
}

#[tokio::test]
async fn test_claude_code_workflow_simulation() {
    let mut simulator = ClaudeSimulator::new().await;
    
    // Test each tool workflow
    for tool in ["generate_template", "scaffold_project", "analyze_complexity"] {
        simulator.test_tool_workflow(tool).await.unwrap();
    }
}
```

## State Machine Testing

### MCP Session State Verification

```rust
#[derive(Debug, Clone, PartialEq)]
enum McpState {
    Uninitialized,
    Initialized,
    Error,
}

#[tokio::test]
async fn test_mcp_state_transitions() {
    let mut harness = McpTestHarness::spawn().await;
    let mut state = McpState::Uninitialized;
    
    // Uninitialized -> Error (calling method before init)
    let response = harness.send_request("tools/list", json!({})).await.unwrap();
    if response["error"].is_object() {
        state = McpState::Error;
    }
    assert_eq!(state, McpState::Error);
    
    // Error -> Initialized (successful init)
    let response = harness.send_request("initialize", json!({
        "protocolVersion": "1.0"
    })).await.unwrap();
    if response["result"].is_object() {
        state = McpState::Initialized;
    }
    assert_eq!(state, McpState::Initialized);
    
    // Initialized -> Initialized (methods work)
    let response = harness.send_request("tools/list", json!({})).await.unwrap();
    assert!(response["result"].is_object());
    assert_eq!(state, McpState::Initialized);
}
```

## Fuzzing MCP Protocol

```rust
// fuzz/fuzz_targets/fuzz_mcp_protocol.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use arbitrary::{Arbitrary, Unstructured};

#[derive(Arbitrary, Debug)]
struct FuzzMcpRequest {
    jsonrpc: Option<String>,
    id: Option<FuzzId>,
    method: String,
    params: Option<FuzzParams>,
}

#[derive(Arbitrary, Debug)]
enum FuzzId {
    Number(u64),
    String(String),
    Null,
}

#[derive(Arbitrary, Debug)]
enum FuzzParams {
    Object(HashMap<String, Value>),
    Array(Vec<Value>),
    Null,
}

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);
    if let Ok(request) = FuzzMcpRequest::arbitrary(&mut u) {
        let json = serde_json::to_string(&request).unwrap();
        
        // Feed to MCP server
        let output = Command::new("target/release/paiml-mcp-agent-toolkit")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
            
        output.stdin.unwrap().write_all(json.as_bytes()).unwrap();
        
        // Server must not crash
        let status = output.wait().unwrap();
        assert!(!status.core_dumped());
    }
});
```

## MCP Compliance Matrix

| Feature | Specification | Test Coverage | Status |
|---------|--------------|---------------|--------|
| JSON-RPC 2.0 | Required | ✓ Full | Complete |
| Request correlation | Required | ✓ Full | Complete |
| Error codes | -32700 to -32603 | ✓ Full | Complete |
| Batch requests | Optional | ✓ Partial | In progress |
| Notifications | Optional | ✗ | Not implemented |
| Initialize handshake | Required | ✓ Full | Complete |
| Capability negotiation | Required | ✓ Full | Complete |
| Schema validation | Required | ✓ Full | Complete |
| STDIO transport | Required | ✓ Full | Complete |
| Concurrent requests | Required | ✓ Full | Complete |

## Test Execution Strategy for MCP

```bash
# MCP-specific test suite
cargo test --test mcp_conformance -- --test-threads=1  # Serial for STDIO
cargo test --test mcp_integration
PROPTEST_CASES=1000 cargo test mcp_prop_

# Fuzzing
cd fuzz && cargo +nightly fuzz run fuzz_mcp_protocol -- -max_total_time=300

# Performance
cargo bench --bench mcp_benchmarks

# Full MCP validation
make test-mcp-compliance
```

This extension ensures the MCP protocol implementation maintains the same rigor as the CLI interface, with particular attention to protocol conformance, concurrent operation safety, and integration with real-world MCP clients like Claude Code.