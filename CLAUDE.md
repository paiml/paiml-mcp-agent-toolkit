# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Important Context

**IMPORTANT**: Always check `docs/bugs/` directory for active bugs before making changes. Archived bugs are in `docs/bugs/archived/`. Current active bugs may affect your work.

**This is a frequently accessed project** - assume familiarity with the codebase structure, development patterns, and ongoing work. This is the MCP Agent Toolkit project that provides template generation services for project scaffolding.

**MANDATORY TRIPLE-INTERFACE TESTING**: This project MUST test ALL THREE interfaces (CLI, MCP, HTTP) continuously throughout development. Every coding session MUST demonstrate comprehensive interface coverage to ensure protocol consistency and identify interface-specific bugs.

## Project Overview

MCP Agent Toolkit is a production-grade unified protocol server that provides:
1. **Template Generation** - Project scaffolding for Makefile, README.md, and .gitignore files
2. **AST-Based Code Analysis** - Full AST parsing and analysis for Rust, TypeScript/JavaScript, and Python
3. **Code Complexity Metrics** - Cyclomatic complexity, cognitive complexity, file ranking system
4. **Code Churn Tracking** - Git-based code change analysis and hotspot detection
5. **Dependency Graph Generation** - Visual code structure analysis with Mermaid
6. **Unified Protocol Architecture** - Single binary serving CLI, MCP JSON-RPC, and HTTP REST interfaces

The system is built in Rust with a unified protocol layer that ensures consistent behavior across all interfaces.

## Architecture

**Unified Protocol Layer**:
```rust
pub trait ProtocolAdapter: Send + Sync {
    type Input;
    type Output;
    
    async fn decode(&self, input: Self::Input) -> Result<UnifiedRequest, ProtocolError>;
    async fn encode(&self, response: UnifiedResponse) -> Result<Self::Output, ProtocolError>;
}
```

**Three Interface Adapters**:
- **CLI Adapter**: Direct command parsing to UnifiedRequest
- **MCP Adapter**: JSON-RPC 2.0 to UnifiedRequest translation
- **HTTP Adapter**: REST endpoints to UnifiedRequest mapping

## Mandatory Triple-Interface Testing Protocol

### Session Start Ritual (ALL INTERFACES REQUIRED)

```bash
# 1. Build the binary with interface validation
make server-build-binary
export BINARY_PATH="./target/release/paiml-mcp-agent-toolkit"

# 2. Start HTTP server in background
$BINARY_PATH serve --port 8080 &
HTTP_PID=$!
sleep 2  # Wait for startup

# 3. Test complexity analysis through ALL interfaces
echo "=== Testing Complexity Analysis ==="

# CLI Interface
time $BINARY_PATH analyze complexity --top-files 5 --format json > cli-complexity.json
echo "CLI Response size: $(wc -c < cli-complexity.json) bytes"

# MCP Interface
echo '{"jsonrpc":"2.0","method":"analyze_complexity","params":{"project_path":"./","top_files":5,"format":"json"},"id":1}' | \
  $BINARY_PATH --mode mcp > mcp-complexity.json
echo "MCP Response size: $(wc -c < mcp-complexity.json) bytes"

# HTTP Interface
time curl -X GET "http://localhost:8080/api/v1/analyze/complexity?top_files=5&format=json" > http-complexity.json
echo "HTTP Response size: $(wc -c < http-complexity.json) bytes"

# 4. Verify consistency across interfaces
./scripts/verify-interface-consistency.ts \
  cli-complexity.json \
  mcp-complexity.json \
  http-complexity.json

# 5. Performance comparison
hyperfine --warmup 10 \
  "$BINARY_PATH analyze complexity --top-files 5 --format json" \
  "echo '{\"jsonrpc\":\"2.0\",\"method\":\"analyze_complexity\",\"params\":{\"project_path\":\"./\",\"top_files\":5},\"id\":1}' | $BINARY_PATH --mode mcp" \
  "curl -s http://localhost:8080/api/v1/analyze/complexity?top_files=5"
```

### During Development (CONTINUOUS TRIPLE TESTING)

#### After Implementing New Features

```bash
# Example: Adding --top-files ranking feature

# 1. Test through CLI
$BINARY_PATH analyze churn --top-files 10 --format table
$BINARY_PATH analyze complexity --top-files 5 --format json
$BINARY_PATH analyze dag --top-files 10 --enhanced

# 2. Test through MCP (using test harness)
cat <<EOF | $BINARY_PATH --mode mcp
{"jsonrpc":"2.0","method":"analyze_churn","params":{"project_path":"./","top_files":10},"id":1}
{"jsonrpc":"2.0","method":"analyze_complexity","params":{"project_path":"./","top_files":5},"id":2}
{"jsonrpc":"2.0","method":"analyze_dag","params":{"project_path":"./","top_files":10,"enhanced":true},"id":3}
EOF

# 3. Test through HTTP
curl "http://localhost:8080/api/v1/analyze/churn?top_files=10"
curl "http://localhost:8080/api/v1/analyze/complexity?top_files=5"
curl "http://localhost:8080/api/v1/analyze/dag?top_files=10&enhanced=true"

# 4. Load test all interfaces
artillery quick \
  --count 100 \
  --num 10 \
  "http://localhost:8080/api/v1/analyze/complexity?top_files=5"
```

#### Interface-Specific Test Patterns

```bash
# CLI-specific: Test argument parsing edge cases
$BINARY_PATH analyze complexity --top-files 0  # Should show all
$BINARY_PATH analyze complexity --top-files -1  # Should error
$BINARY_PATH analyze complexity --top-files 999999  # Should handle gracefully

# MCP-specific: Test JSON-RPC compliance
# Test batch requests
echo '[
  {"jsonrpc":"2.0","method":"analyze_complexity","params":{"top_files":5},"id":1},
  {"jsonrpc":"2.0","method":"analyze_churn","params":{"top_files":5},"id":2}
]' | $BINARY_PATH --mode mcp

# Test notifications (no id)
echo '{"jsonrpc":"2.0","method":"analyze_complexity","params":{"top_files":5}}' | \
  $BINARY_PATH --mode mcp

# HTTP-specific: Test REST semantics
# Test HEAD requests
curl -I "http://localhost:8080/api/v1/analyze/complexity?top_files=5"

# Test content negotiation
curl -H "Accept: application/json" "http://localhost:8080/api/v1/analyze/complexity?top_files=5"
curl -H "Accept: application/x-sarif+json" "http://localhost:8080/api/v1/analyze/complexity?top_files=5"
```

### Session End Ritual (MANDATORY INTERFACE VALIDATION)

```bash
# 1. Generate interface compatibility report
./scripts/generate-interface-report.ts \
  --test-all-endpoints \
  --measure-latency \
  --check-consistency \
  > interface-report-$(date +%Y%m%d).json

# 2. Run interface-specific benchmarks
# CLI benchmark
hyperfine --warmup 5 --min-runs 50 \
  "$BINARY_PATH analyze complexity --top-files 10" \
  --export-json cli-bench.json

# MCP benchmark (with connection reuse)
./scripts/bench-mcp-interface.ts --runs 50 --top-files 10

# HTTP benchmark (with keep-alive)
ab -n 1000 -c 10 -k "http://localhost:8080/api/v1/analyze/complexity?top_files=10"

# 3. Verify memory usage across interfaces
./scripts/measure-interface-memory.ts

# 4. Kill HTTP server
kill $HTTP_PID
```

## Interface Testing Matrix

| Feature | CLI Test | MCP Test | HTTP Test | Consistency Check |
|---------|----------|----------|-----------|-------------------|
| `--top-files` | `analyze complexity --top-files 5` | `{"method":"analyze_complexity","params":{"top_files":5}}` | `GET /api/v1/analyze/complexity?top_files=5` | Compare JSON outputs |
| Composite ranking | `analyze composite --weights complexity=0.3,churn=0.7` | `{"method":"analyze_composite","params":{"weights":{"complexity":0.3,"churn":0.7}}}` | `GET /api/v1/analyze/composite?weights=complexity:0.3,churn:0.7` | Verify same file order |
| Error handling | `analyze complexity --top-files -1` | `{"params":{"top_files":-1}}` | `GET /api/v1/analyze/complexity?top_files=-1` | Same error code |
| Streaming | `analyze dag --stream` | `{"params":{"stream":true}}` with chunked responses | `GET /api/v1/analyze/dag?stream=true` with SSE | Chunk boundaries |

## Protocol-Specific Monitoring

### CLI Interface Monitoring
```bash
# Trace syscalls to understand CLI overhead
strace -c $BINARY_PATH analyze complexity --top-files 5

# Profile with perf
perf record -g $BINARY_PATH analyze complexity --top-files 5
perf report
```

### MCP Interface Monitoring
```bash
# Monitor JSON-RPC message flow
$BINARY_PATH --mode mcp --trace-protocol < requests.jsonl

# Measure message parsing overhead
./scripts/profile-json-parsing.ts
```

### HTTP Interface Monitoring
```bash
# Monitor with tcpdump
sudo tcpdump -i lo -A 'tcp port 8080' -w http-trace.pcap

# Analyze with Wireshark or tshark
tshark -r http-trace.pcap -Y "http.request.method == GET"

# Profile with async-profiler
async-profiler -d 30 -f profile.html $HTTP_PID
```

## Common Interface Pitfalls

### 1. Inconsistent Parameter Names
```rust
// ❌ WRONG: Different parameter names per interface
impl CliArgs {
    top_files: Option<usize>,  // CLI uses snake_case
}

impl McpParams {
    topFiles: Option<usize>,   // MCP uses camelCase
}

// ✅ CORRECT: Unified parameter handling
impl UnifiedRequest {
    pub fn top_files(&self) -> Option<usize> {
        // Handle both snake_case and camelCase
    }
}
```

### 2. Format Negotiation Differences
```rust
// Each interface handles format differently:
// CLI: --format json
// MCP: params.format = "json"  
// HTTP: Accept: application/json

// Test all variations:
test_format_negotiation(&["json", "sarif", "markdown", "csv"])
```

### 3. Error Response Variations
```rust
// Ensure consistent error codes across interfaces:
// CLI: Exit code 1-255
// MCP: JSON-RPC error codes
// HTTP: HTTP status codes

#[test]
fn test_error_consistency() {
    assert_eq!(cli_error_code(FileNotFound), 2);
    assert_eq!(mcp_error_code(FileNotFound), -32000);
    assert_eq!(http_status_code(FileNotFound), 404);
}
```

## Performance Targets by Interface

| Interface | Startup | First Response | Throughput | Memory |
|-----------|---------|----------------|------------|---------|
| CLI | <10ms | <50ms | N/A | <20MB |
| MCP | <5ms | <20ms | 1000 req/s | <30MB |
| HTTP | <2ms | <10ms | 5000 req/s | <50MB |

## Development Workflow Commands

### Quick Interface Tests
```bash
# Test all interfaces with one command
make test-all-interfaces

# Test specific feature across interfaces
./scripts/test-feature-all-interfaces.ts --feature top-files

# Regression test interface consistency
make test-interface-regression
```

### Debugging Interface Issues
```bash
# Debug CLI parsing
RUST_LOG=paiml_mcp=trace $BINARY_PATH analyze complexity --top-files 5

# Debug MCP protocol
$BINARY_PATH --mode mcp --debug-protocol

# Debug HTTP routing
RUST_LOG=tower_http=debug,paiml_mcp=trace $BINARY_PATH serve
```

## Integration Test Examples

### Testing New Ranking Features
```rust
#[tokio::test]
async fn test_top_files_all_interfaces() {
    let test_harness = TestHarness::new();
    
    // Test CLI
    let cli_result = test_harness.cli()
        .args(&["analyze", "complexity", "--top-files", "5"])
        .output()
        .await?;
    
    // Test MCP
    let mcp_result = test_harness.mcp()
        .call("analyze_complexity", json!({
            "top_files": 5
        }))
        .await?;
    
    // Test HTTP
    let http_result = test_harness.http()
        .get("/api/v1/analyze/complexity?top_files=5")
        .await?;
    
    // Verify consistency
    assert_interface_consistency![cli_result, mcp_result, http_result];
}
```

### Load Testing Pattern
```rust
#[tokio::test]
async fn bench_top_files_interfaces() {
    let mut group = c.benchmark_group("top_files");
    
    group.bench_function("cli", |b| {
        b.iter(|| {
            Command::new(&binary)
                .args(&["analyze", "complexity", "--top-files", "10"])
                .output()
        })
    });
    
    group.bench_function("mcp", |b| {
        let client = MpcClient::new(&binary);
        b.iter(|| {
            client.call("analyze_complexity", json!({"top_files": 10}))
        })
    });
    
    group.bench_function("http", |b| {
        let client = HttpClient::new("localhost:8080");
        b.iter(|| {
            client.get("/api/v1/analyze/complexity?top_files=10")
        })
    });
}
```

## Git Commit Policy

**NEVER commit changes unless explicitly asked by the user.** The user will commit when they are ready. This ensures:
- User maintains control over git history
- Interface tests can be run before committing
- Performance regressions can be caught
- All three interfaces are validated

## Why Triple-Interface Testing Matters

1. **Protocol Bugs**: Interface-specific bugs are common (parameter parsing, serialization)
2. **Performance Gaps**: CLI might be 10x slower than HTTP due to startup overhead
3. **Feature Parity**: Easy to forget implementing a feature in all interfaces
4. **User Experience**: Different users prefer different interfaces
5. **Integration Issues**: MCP clients expect exact JSON-RPC compliance

Remember: **Every feature must work identically across CLI, MCP, and HTTP interfaces.**
Remember: **If we don't use our own tools constantly, we can't expect others to find them valuable.**