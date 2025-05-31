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

## Release Process

### Release Workflow (MANDATORY STEPS)

When the user requests to "prepare a new release", follow this exact sequence:

#### 1. Pre-Release Validation
```bash
# Verify all tests pass before release
make lint && make test

# Ensure clean git state
git status  # Should show clean working directory
```

#### 2. Documentation Updates

**Update README.md:**
- Add new features to the feature list sections
- Update MCP tools table with any new tools
- Add new CLI usage examples 
- Update performance metrics if applicable

**Update RELEASE_NOTES.md:**
- Add new version section at the top (e.g., `# Release Notes for v0.15.0`)
- Include comprehensive feature descriptions with technical details
- Add usage examples for new features
- Document breaking changes, bug fixes, and improvements
- Include performance characteristics and test coverage improvements

**Key Release Notes Sections:**
```markdown
# Release Notes for vX.Y.Z

## 🎯 Major Feature Release: [Feature Name]

## ✨ New Features
### [Feature Name] (`command-name`)
- **NEW**: [Feature description]
- **FIXED**: [Bug fixes] 
- **IMPROVED**: [Enhancements]

## 🔧 Technical Implementation
### [Module Name] (`path/to/file.rs`)
- [Implementation details]

## 📊 Usage Examples
```bash
# CLI Usage
command --param value
```

## 🧪 Test Coverage Improvements
- **NEW**: [Number] new tests for [feature]
- **VERIFIED**: All interface consistency checks passing

## 🚀 Performance Characteristics
- **Startup**: <Xms
- **Analysis**: [Performance details]
```

#### 3. Commit Changes
```bash
# Stage documentation files first
git add README.md RELEASE_NOTES.md

# Stage all implementation files
git add server/src/

# Create comprehensive commit message
git commit -m "$(cat <<'EOF'
feat: [feature description]

[Detailed implementation description]

Core Implementation:
- [Technical detail 1]
- [Technical detail 2]

Interface Support:
- CLI: [CLI details]
- MCP: [MCP details] 
- HTTP: [HTTP details]

Technical Fixes:
- [Fix 1]
- [Fix 2]

Documentation Updates:
- [Doc update 1]
- [Doc update 2]

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>
EOF
)"
```

#### 4. Push and Release
```bash
# Push changes to trigger automated version bumping
git push origin master

# Create GitHub release using gh CLI with version from RELEASE_NOTES.md
gh release create v[X.Y.Z] \
  --title "v[X.Y.Z]: [Feature Title]" \
  --notes "$(cat <<'EOF'
## 🎯 [Release Type]: [Feature Name]

[Copy the main sections from RELEASE_NOTES.md]

**Full release notes**: https://github.com/paiml/paiml-mcp-agent-toolkit/blob/master/RELEASE_NOTES.md
EOF
)"
```

### Version Numbering Policy

**DO NOT manually update Cargo.toml version numbers.** The GitHub Actions automatically handle version bumping when changes are pushed.

**Version Scheme:**
- **Major (X.0.0)**: Breaking changes, major architecture changes
- **Minor (0.X.0)**: New features, new analysis tools, significant enhancements  
- **Patch (0.0.X)**: Bug fixes, documentation updates, minor improvements

**Examples:**
- `v0.15.0`: Dead code analysis feature (new analysis tool = minor)
- `v0.14.1`: Bug fixes and documentation (patch)
- `v1.0.0`: Major API breaking changes (major)

### Release Notes Quality Standards

**MANDATORY elements for each release:**
1. **🎯 Clear feature title** - What the release accomplishes
2. **✨ New Features** - Bullet points with **NEW**/**FIXED**/**IMPROVED** tags
3. **📊 Usage Examples** - Copy-pasteable CLI, MCP, and HTTP examples
4. **🔧 Technical Implementation** - File paths and implementation details
5. **🧪 Test Coverage** - Number of new tests and validation improvements
6. **🚀 Performance** - Startup times, memory usage, scaling characteristics

**Release Notes Template:**
```markdown
# Release Notes for v[X.Y.Z]

## 🎯 [Major|Feature|Patch] Release: [Feature Name]

[1-2 sentence summary of what this release accomplishes]

## ✨ New Features

### [Feature Name] (`cli-command-name`)
- **NEW**: [Specific capability 1]
- **NEW**: [Specific capability 2]  
- **FIXED**: [Bug fix]
- **IMPROVED**: [Enhancement]

### [Integration Type] Integration
- **NEW**: [MCP/CLI/HTTP details]
- **UPDATED**: [Changes to existing functionality]

## 🔧 Technical Implementation

### [Module Name] (`server/src/path/file.rs`)
- [Implementation detail 1]
- [Implementation detail 2]

## 📊 Usage Examples

```bash
# CLI Usage
paiml-mcp-agent-toolkit command --param value

# MCP Tool Call  
{"method": "tool_name", "params": {"param": "value"}}

# HTTP API
GET /api/v1/endpoint?param=value
```

## 🧪 Test Coverage Improvements
- **NEW**: [X] new tests for [feature]
- **VERIFIED**: All interface consistency checks passing

## 🚀 Performance Characteristics
- **Startup**: <[X]ms
- **Analysis**: [Performance details]
- **Memory**: [Memory usage]
- **Scaling**: [Scaling behavior]
```

### Common Release Mistakes to Avoid

1. **❌ Don't manually edit Cargo.toml version** - Let GitHub Actions handle it
2. **❌ Don't commit without updating documentation** - Always update README.md and RELEASE_NOTES.md
3. **❌ Don't create releases without gh CLI** - Use the documented gh release create command
4. **❌ Don't skip usage examples** - Every feature needs CLI, MCP, and HTTP examples
5. **❌ Don't forget performance metrics** - Include startup times and scaling characteristics
6. **❌ Don't rush the commit message** - Use the comprehensive template with technical details

### Post-Release Verification

After creating a release:
1. **Verify release page**: Check https://github.com/paiml/paiml-mcp-agent-toolkit/releases/tag/v[X.Y.Z]
2. **Test installation**: Verify the install script works with the new version
3. **Update project metrics**: Re-run complexity analysis to update README dogfooding section
4. **Monitor CI/CD**: Ensure all automated builds complete successfully

Remember: **Releases are permanent and public. Take time to ensure quality and completeness.**

## Why Triple-Interface Testing Matters

1. **Protocol Bugs**: Interface-specific bugs are common (parameter parsing, serialization)
2. **Performance Gaps**: CLI might be 10x slower than HTTP due to startup overhead
3. **Feature Parity**: Easy to forget implementing a feature in all interfaces
4. **User Experience**: Different users prefer different interfaces
5. **Integration Issues**: MCP clients expect exact JSON-RPC compliance

Remember: **Every feature must work identically across CLI, MCP, and HTTP interfaces.**
Remember: **If we don't use our own tools constantly, we can't expect others to find them valuable.**