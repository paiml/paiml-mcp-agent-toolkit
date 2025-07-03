# PMAT Interfaces Overview

PMAT provides multiple interfaces to accommodate different use cases, environments, and integration needs. This guide helps you choose the right interface for your specific requirements.

## Available Interfaces

| Interface | Best For | Access Method | Key Features |
|-----------|----------|---------------|--------------|
| **CLI** | Command-line users, scripts, CI/CD | `pmat` command | Direct, scriptable, pipes |
| **HTTP API** | Web services, remote access | REST endpoints | Standard HTTP, JSON responses |
| **MCP** | AI assistants, Claude integration | MCP protocol | Tool calling, streaming |
| **Rust API** | Native integration, custom tools | Library import | Full control, async |
| **Web Demo** | Visual exploration, presentations | Browser UI | Interactive, real-time |
| **TUI** | Terminal power users | Terminal UI | Keyboard-driven, efficient |

## Interface Comparison

### CLI (Command Line Interface)

**Strengths:**
- ✅ No setup required
- ✅ Perfect for scripts and automation
- ✅ Composable with Unix tools
- ✅ Fast for single operations
- ✅ Easy to integrate in CI/CD

**Limitations:**
- ❌ Limited interactivity
- ❌ Text-only output (unless using JSON)
- ❌ Separate process for each command

**Example:**
```bash
pmat analyze complexity --top-files 10 --format json | jq '.files[0]'
```

### HTTP REST API

**Strengths:**
- ✅ Language agnostic
- ✅ Remote access capability
- ✅ Standard REST patterns
- ✅ Stateless operations
- ✅ Easy caching

**Limitations:**
- ❌ Requires server running
- ❌ Network overhead
- ❌ Authentication needed for security

**Example:**
```bash
curl -X POST http://localhost:8080/api/analyze/complexity \
  -H "Content-Type: application/json" \
  -d '{"path": "./src", "top_files": 10}'
```

### MCP (Model Context Protocol)

**Strengths:**
- ✅ Native AI assistant integration
- ✅ Rich tool descriptions
- ✅ Streaming capabilities
- ✅ Context-aware responses
- ✅ Built for Claude Desktop/Code

**Limitations:**
- ❌ Requires MCP-compatible client
- ❌ More complex protocol
- ❌ Primarily for AI assistants

**Example:**
```json
{
  "jsonrpc": "2.0",
  "method": "analyze_complexity",
  "params": {
    "path": "./src",
    "top_files": 10
  },
  "id": 1
}
```

### Rust API (Library)

**Strengths:**
- ✅ Maximum performance
- ✅ Full type safety
- ✅ Direct integration
- ✅ Async/concurrent operations
- ✅ Custom workflows

**Limitations:**
- ❌ Rust knowledge required
- ❌ Compilation needed
- ❌ Version compatibility

**Example:**
```rust
let service = CodeAnalysisService::new();
let results = service.analyze_complexity(path, Some(10)).await?;
```

### Web Demo Interface

**Strengths:**
- ✅ Visual representations
- ✅ Interactive exploration
- ✅ No installation for users
- ✅ Great for presentations
- ✅ Real-time updates

**Limitations:**
- ❌ Requires browser
- ❌ Not scriptable
- ❌ Resource intensive

**Access:**
```bash
pmat demo --web
# Opens http://localhost:3000
```

### TUI (Terminal User Interface)

**Strengths:**
- ✅ Efficient navigation
- ✅ Works over SSH
- ✅ Keyboard-driven workflow
- ✅ Low resource usage
- ✅ Real-time updates

**Limitations:**
- ❌ Terminal only
- ❌ Learning curve for shortcuts
- ❌ Limited visualizations

**Access:**
```bash
pmat demo --mode tui
```

## Choosing the Right Interface

### By Use Case

**For CI/CD Pipelines:** CLI
```bash
pmat quality-gate --strict || exit 1
```

**For Web Services:** HTTP API
```python
response = requests.post("http://localhost:8080/api/analyze/context")
```

**For AI Assistants:** MCP
```javascript
const result = await mcp.call("analyze_complexity", { path: "./src" });
```

**For Custom Tools:** Rust API
```rust
let analyzer = CustomAnalyzer::new(CodeAnalysisService::new());
```

**For Code Reviews:** Web Demo
```bash
pmat demo --web --repo owner/repo
```

**For Interactive Analysis:** TUI
```bash
pmat demo --mode tui
```

### By Environment

| Environment | Recommended Interface | Alternative |
|-------------|----------------------|-------------|
| Local Development | CLI | TUI |
| CI/CD Pipeline | CLI | HTTP API |
| Cloud Services | HTTP API | - |
| IDE Integration | Rust API | MCP |
| AI Assistants | MCP | HTTP API |
| Remote Servers | CLI (SSH) | TUI (SSH) |
| Presentations | Web Demo | - |

### By Technical Requirements

**Need Speed?**
- 1st: Rust API (native)
- 2nd: CLI (direct)
- 3rd: MCP (efficient protocol)

**Need Flexibility?**
- 1st: HTTP API (any language)
- 2nd: CLI (shell scripts)
- 3rd: MCP (tool protocol)

**Need Visualization?**
- 1st: Web Demo (full graphics)
- 2nd: TUI (terminal graphics)
- 3rd: CLI with JSON + external tools

**Need Integration?**
- 1st: Rust API (deepest)
- 2nd: HTTP API (standard)
- 3rd: MCP (AI-focused)

## Interface Combinations

You can combine interfaces for powerful workflows:

### CLI + Web Demo
```bash
# Analyze with CLI, visualize with web
pmat analyze complexity --format json > results.json
pmat demo --web --data results.json
```

### HTTP API + CLI
```bash
# Start server
pmat serve --port 8080 &

# Use CLI for some operations, API for others
pmat analyze dag --output graph.mmd
curl http://localhost:8080/api/analyze/complexity
```

### MCP + TUI
```bash
# Use MCP for automation, TUI for exploration
echo '{"method": "analyze_project"}' | pmat --mode mcp
pmat demo --mode tui
```

## Performance Comparison

| Interface | Startup Time | Operation Overhead | Concurrency |
|-----------|--------------|-------------------|-------------|
| CLI | ~100ms | Low | Process-based |
| HTTP API | ~5ms* | Network latency | High |
| MCP | ~50ms | Protocol parsing | Medium |
| Rust API | ~0ms | None | Native |
| Web Demo | ~2s | Rendering | Limited |
| TUI | ~500ms | Terminal I/O | Single |

*After server startup

## Security Considerations

| Interface | Authentication | Authorization | Audit |
|-----------|----------------|---------------|-------|
| CLI | OS user | File permissions | Shell history |
| HTTP API | Token/OAuth | Role-based | Access logs |
| MCP | Client cert | Tool permissions | Protocol logs |
| Rust API | Compile-time | Code-based | Custom |
| Web Demo | Optional | Read-only | Browser |
| TUI | OS user | File permissions | Terminal |

## Migration Guide

### From CLI to HTTP API
```bash
# CLI
pmat analyze complexity --top-files 10

# HTTP API equivalent
curl -X GET "http://localhost:8080/api/analyze/complexity?top_files=10"
```

### From HTTP API to MCP
```python
# HTTP API
response = requests.post("/api/analyze/complexity", json={"top_files": 10})

# MCP equivalent
result = await mcp.call("analyze_complexity", {"top_files": 10})
```

### From CLI to Rust API
```bash
# CLI script
RESULT=$(pmat analyze complexity --format json)

# Rust API equivalent
let service = CodeAnalysisService::new();
let result = service.analyze_complexity(path, Some(10)).await?;
```

## Best Practices

1. **Start with CLI** for learning and prototyping
2. **Use HTTP API** for service integration
3. **Choose MCP** for AI assistant workflows
4. **Implement Rust API** for performance-critical applications
5. **Demo with Web** for stakeholder presentations
6. **Master TUI** for efficient daily use

## Quick Decision Tree

```
Need to integrate PMAT?
├─ With AI Assistant?
│  └─ Use MCP
├─ In a web service?
│  └─ Use HTTP API
├─ In a Rust application?
│  └─ Use Rust API
├─ For automation/scripts?
│  └─ Use CLI
├─ For interactive exploration?
│  ├─ Graphical? → Web Demo
│  └─ Terminal? → TUI
└─ Just want to try it?
   └─ Start with CLI
```

## See Also

- [CLI Reference](/docs/cli-reference.md)
- [HTTP API Documentation](/rust-docs/http-api.md)
- [MCP Protocol Guide](/docs/features/mcp-protocol.md)
- [Rust API Guide](/docs/api-guide.md)
- [Demo Interface](/docs/features/demo-interface.md)
- [TUI Documentation](/docs/features/tui-interface.md)