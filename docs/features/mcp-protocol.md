# MCP Protocol Implementation

## Overview

The Model Context Protocol (MCP) implementation in PMAT provides a standardized interface for AI agents to interact with development tools. It enables seamless integration with AI assistants like Claude, providing them with powerful code analysis capabilities.

## Architecture

```
┌─────────────────────────────────────────────────┐
│              AI Assistant (Claude)              │
└─────────────────────┬───────────────────────────┘
                      │ MCP Protocol
                      │ (JSON-RPC 2.0)
┌─────────────────────┴───────────────────────────┐
│                 MCP Server (PMAT)               │
├─────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌────────────────────────┐  │
│  │   Transport  │  │    Request Handler     │  │
│  │    (stdio)   │  │   (JSON-RPC Router)    │  │
│  └──────────────┘  └────────────────────────┘  │
│  ┌──────────────────────────────────────────┐  │
│  │           Unified Tool Registry          │  │
│  │    Single pmcp SDK-based server with    │  │
│  │      17 core tools consolidated         │  │
│  │                                        │  │
│  └──────────────────────────────────────────┘  │
└─────────────────────────────────────────────────┘
```

## Installation

### For Claude Desktop

1. Install PMAT using one of these methods:

**Option A: Install from crates.io (Recommended)**
```bash
cargo install pmat
```

**Option B: Quick install script**
```bash
curl -sSfL https://raw.githubusercontent.com/paiml/paiml-mcp-agent-toolkit/master/scripts/install.sh | sh
```

2. Configure Claude Desktop:

Find your Claude Desktop configuration file:
- **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`
- **Linux**: `~/.config/Claude/claude_desktop_config.json`

Add the PMAT MCP server:
```json
{
  "mcpServers": {
    "paiml-toolkit": {
      "command": "pmat",
      "args": [],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

3. Restart Claude Desktop to load the configuration.

### For Claude Code

```bash
# Add to Claude Code
claude mcp add paiml-toolkit ~/.cargo/bin/pmat

# Or if installed elsewhere
claude mcp add paiml-toolkit /usr/local/bin/pmat
```

### For Other MCP Clients

```bash
# Start unified MCP server (auto-detected)
pmat

# With debug logging
RUST_LOG=debug pmat

# The server automatically detects MCP mode when:
# - stdin is not a terminal
# - MCP_VERSION environment variable is set
```

## Protocol Specification

### Message Format

All messages follow JSON-RPC 2.0 specification:

```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "analyze_complexity",
    "arguments": {
      "path": "/path/to/project",
      "format": "json"
    }
  },
  "id": "123"
}
```

### Available Tools

**Unified Server: 17 Core Tools**

The unified MCP server consolidates all functionality into 17 core tools using the pmcp SDK:

## Core Tools (17 tools)

### 1. Analysis Tools

#### `analyze_complexity`
Analyze code complexity metrics including cyclomatic and cognitive complexity.

#### `analyze_churn` 
Analyze code change patterns over time.

#### `analyze_code_churn`
Analyze code change patterns over time (alias for analyze_churn).

#### `analyze_dag`
Generate and analyze dependency graphs.

#### `analyze_dead_code`
Detect unused code in the project.

#### `analyze_deep_context`
Perform deep contextual analysis of the codebase with defect detection.

#### `analyze_satd`
Analyze self-admitted technical debt in code comments.

### 2. Context and Template Tools

#### `generate_context`
Generate comprehensive project context for AI assistants.

#### `generate_template`
Generate project templates (Makefile, README, .gitignore).

**Parameters:**
```typescript
{
  template_type: "makefile" | "readme" | "gitignore" | "all";
  path: string;
  project_name?: string;
  language?: string;
}
```

#### `list_templates`
List all available project templates.

**Parameters:**
```typescript
{
  filter?: string;  // Optional filter by type
}
```

#### `validate_template`
Validate template parameters before generation.

**Parameters:**
```typescript
{
  template_type: string;
  parameters: Record<string, any>;
}
```

#### `scaffold_project`
Create complete project structure with templates.

**Parameters:**
```typescript
{
  path: string;
  project_type: "rust" | "typescript" | "python" | "cpp";
  features?: string[];
}
```

#### `search_templates`
Search available templates by keyword.

**Parameters:**
```typescript
{
  query: string;
  limit?: number;
}
```

#### `get_server_info`
Get MCP server information and capabilities.

**Parameters:** None

### 3. Quality and Refactoring Tools

#### `quality_gate`
Run comprehensive quality checks and validation.

#### `quality_proxy`
**NEW in v2.2** - Intercept and validate AI-generated code before it's written.

Acts as a quality gatekeeper between AI agents and your codebase, enforcing standards on all generated code.
Analyzes code complexity metrics with tool composition support.

**Parameters:**
```typescript
{
  path: string;
  file?: string;           // Single file analysis (conflicts with files)
  files?: string[];        // Multi-file analysis for MCP composition
  max_cyclomatic?: number;
  max_cognitive?: number;
  format?: "summary" | "full" | "json";
}
```

**MCP Tool Composition:**
```javascript
// AI agent discovers complexity hotspots
const hotspots = await callTool("analyze_complexity", {
  path: "/project",
  top_files: 5,
  format: "json"
});

// Agent performs targeted analysis on specific files
const targeted = await callTool("analyze_complexity", {
  path: "/project",
  files: hotspots.files.map(f => f.path)
});
```

**Response:**
```json
{
  "summary": {
    "total_files": 125,
    "total_functions": 1850,
    "median_complexity": 5,
    "p90_complexity": 15,
    "hotspots": [
      {
        "file": "src/analyzer.rs",
        "function": "process_ast",
        "cyclomatic": 32,
        "cognitive": 45
      }
    ]
  }
}
```

### 4. Refactoring Tools

#### `refactor.start`
Start an interactive refactoring session.

#### `refactor.nextIteration`
Continue to the next step in an active refactoring session.

#### `refactor.getState`
Get the current state of an active refactoring session.

#### `refactor.stop`
Stop and finalize an active refactoring session.

## Unified Server Benefits

The unified server architecture provides:
- **Single Implementation**: All tools consolidated into one server
- **10x Performance**: pmcp SDK optimization for all operations
- **Type Safety**: Compile-time validation of tool interfaces
- **Quality Integration**: Built-in quality proxy for all operations
- **Consistent Behavior**: One code path for all MCP tools

**Parameters:**
```typescript
{
  operation: "write" | "edit" | "append";
  file_path: string;
  content?: string;          // For write/append operations
  old_content?: string;      // For edit operations
  new_content?: string;      // For edit operations
  mode?: "strict" | "advisory" | "auto_fix";  // Default: "strict"
  quality_config?: {
    max_complexity?: number;    // Default: 20
    allow_satd?: boolean;       // Default: false
    require_docs?: boolean;     // Default: true
    auto_format?: boolean;      // Default: true
  }
}
```

**Response:**
```json
{
  "status": "accepted" | "rejected" | "modified",
  "quality_report": {
    "passed": boolean,
    "metrics": {
      "max_complexity": 15,
      "satd_count": 0,
      "lint_violations": 0
    },
    "violations": [
      {
        "type": "complexity" | "satd" | "docs" | "lint",
        "severity": "error" | "warning",
        "location": "file.rs:45",
        "message": "Function complexity exceeds threshold",
        "suggestion": "Split into smaller functions"
      }
    ]
  },
  "final_content": "// Validated or auto-fixed content",
  "refactoring_applied": boolean,
  "refactoring_plan": []  // Steps taken in auto-fix mode
}
```

**Usage Example:**
```javascript
// AI agent validates code before writing
const result = await callTool("quality_proxy", {
  operation: "write",
  file_path: "src/new_feature.rs",
  content: generatedCode,
  mode: "auto_fix",  // Automatically fix issues
  quality_config: {
    max_complexity: 15,  // Stricter than default
    require_docs: true
  }
});

if (result.status === "accepted" || result.status === "modified") {
  // Code meets standards, safe to write
  await writeFile(result.final_content);
} else {
  // Code rejected, show violations to user
  console.error("Quality violations:", result.quality_report.violations);
}
```

### Resources

The MCP server exposes project resources:

```json
{
  "method": "resources/list",
  "result": {
    "resources": [
      {
        "uri": "project://current",
        "name": "Current Project",
        "mimeType": "application/x-project"
      },
      {
        "uri": "analysis://complexity",
        "name": "Complexity Report",
        "mimeType": "application/json"
      }
    ]
  }
}
```

### Prompts

Pre-configured analysis prompts:

```json
{
  "method": "prompts/list",
  "result": {
    "prompts": [
      {
        "name": "code_review",
        "description": "Comprehensive code review analysis",
        "arguments": [
          {
            "name": "path",
            "description": "Path to review",
            "required": true
          }
        ]
      }
    ]
  }
}
```

## Integration Examples

### Claude Desktop Integration

When PMAT is configured in Claude Desktop, you can use natural language:

```
Claude: "Analyze the complexity of the src/services directory"

PMAT (via MCP): {
  "tool": "analyze_complexity",
  "result": {
    "files_analyzed": 23,
    "avg_complexity": 12.5,
    "hotspots": [...]
  }
}

Claude: "Based on the analysis, the services directory has moderate complexity 
with 3 hotspots that should be refactored..."
```

### Custom MCP Client

```python
import json
import subprocess
from typing import Any, Dict

class PMATMCPClient:
    def __init__(self):
        self.process = subprocess.Popen(
            ['pmat', '--mode', 'mcp'],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True
        )
        self.message_id = 0
    
    def call_tool(self, tool_name: str, arguments: Dict[str, Any]) -> Dict[str, Any]:
        self.message_id += 1
        request = {
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {
                "name": tool_name,
                "arguments": arguments
            },
            "id": str(self.message_id)
        }
        
        # Send request
        self.process.stdin.write(json.dumps(request) + '\n')
        self.process.stdin.flush()
        
        # Read response
        response_line = self.process.stdout.readline()
        response = json.loads(response_line)
        
        if "error" in response:
            raise Exception(response["error"])
        
        return response["result"]
    
    def analyze_project(self, path: str):
        # Generate context
        context = self.call_tool("generate_context", {"path": path})
        
        # Analyze complexity
        complexity = self.call_tool("analyze_complexity", {
            "path": path,
            "format": "json"
        })
        
        return {
            "context": context,
            "complexity": complexity
        }
```

### Node.js Integration

```javascript
const { spawn } = require('child_process');
const readline = require('readline');

class PMATMCPClient {
  constructor() {
    this.process = spawn('pmat', ['--mode', 'mcp']);
    this.rl = readline.createInterface({
      input: this.process.stdout,
      output: this.process.stdin
    });
    this.messageId = 0;
    this.pendingRequests = new Map();
    
    this.rl.on('line', (line) => {
      const response = JSON.parse(line);
      const resolver = this.pendingRequests.get(response.id);
      if (resolver) {
        resolver(response);
        this.pendingRequests.delete(response.id);
      }
    });
  }
  
  async callTool(name, args) {
    const id = String(++this.messageId);
    const request = {
      jsonrpc: "2.0",
      method: "tools/call",
      params: { name, arguments: args },
      id
    };
    
    return new Promise((resolve, reject) => {
      this.pendingRequests.set(id, (response) => {
        if (response.error) {
          reject(new Error(response.error.message));
        } else {
          resolve(response.result);
        }
      });
      
      this.process.stdin.write(JSON.stringify(request) + '\n');
    });
  }
}

// Usage
const client = new PMATMCPClient();
const result = await client.callTool('analyze_complexity', {
  path: './src',
  format: 'json'
});
```

## Error Handling

### Error Response Format

```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32602,
    "message": "Invalid params",
    "data": {
      "field": "path",
      "reason": "Path does not exist"
    }
  },
  "id": "123"
}
```

### Error Codes

| Code | Meaning | Description |
|------|---------|-------------|
| -32700 | Parse error | Invalid JSON |
| -32600 | Invalid request | Not a valid request object |
| -32601 | Method not found | Unknown method |
| -32602 | Invalid params | Invalid method parameters |
| -32603 | Internal error | Server error |
| -32000 | Tool error | Tool-specific error |

## Performance Considerations

### Streaming Large Results

For large analysis results, use streaming:

```json
{
  "method": "tools/call",
  "params": {
    "name": "analyze_deep_context",
    "arguments": {
      "path": "/large/project",
      "stream": true
    }
  }
}
```

Responses are chunked:
```json
{"chunk": 1, "total": 5, "data": "..."}
{"chunk": 2, "total": 5, "data": "..."}
```

### Caching

The MCP server implements intelligent caching:

```toml
# mcp-config.toml
[cache]
enabled = true
max_size = "1GB"
ttl = "1h"
strategy = "lru"

[cache.rules]
# Cache analysis results for 1 hour
"analyze_*" = { ttl = "1h" }
# Don't cache context generation
"generate_context" = { enabled = false }
```

## Configuration

### Environment Variables

| Variable | Purpose | Default |
|----------|---------|----------|
| `MCP_VERSION` | Force MCP mode | `false` |
| `RUST_LOG` | Logging level | `info` |
| `DOCS_RS` | Docs.rs build mode | `false` |

### Unified MCP Server

As of v2.2.0, PMAT uses a single unified MCP server implementation based on the pmcp SDK:

```bash
# Run unified server (auto-detected)
pmat

# With debug logging
RUST_LOG=debug pmat
```

### Cache Configuration

```toml
# .pmat.toml
[cache]
strategy = "normal"  # normal, force-refresh, offline
enabled = true
max_size = "1GB"
ttl = "1h"
```

## Performance Features

### SIMD/Vectorized Analysis

PMAT includes high-performance vectorized tools that use SIMD instructions for parallel processing:

- **analyze_duplicates_vectorized** - Up to 8x faster duplicate detection
- **analyze_graph_metrics_vectorized** - Parallel graph analysis
- **analyze_big_o_vectorized** - Concurrent complexity analysis

### Parallel Processing

Most analysis tools support parallel execution:

```typescript
{
  path: string;
  parallel_workers?: number;  // Default: CPU cores
  chunk_size?: number;        // Files per worker
}
```

### GPU Acceleration

Some tools support GPU acceleration when available:

```bash
# Enable GPU acceleration
PMAT_GPU_ENABLED=1 pmat --mode mcp
```

## Best Practices

1. **Batch Operations**: Combine multiple analyses in one request
2. **Use Caching**: Enable caching for repeated analyses
3. **Stream Large Results**: Use streaming for large codebases
4. **Handle Errors Gracefully**: Implement proper error handling
5. **Monitor Performance**: Track request latencies and errors

## Troubleshooting

### Common Issues

**Q: MCP server not starting**
A: Check that PMAT is in PATH: `which pmat`

**Q: Timeout errors**
A: Increase timeout in client configuration

**Q: Authentication failures**
A: Ensure MCP_AUTH_TOKEN environment variable is set

### Debug Mode

```bash
# Enable debug logging
RUST_LOG=debug pmat --mode mcp

# Trace all MCP messages
RUST_LOG=paiml_mcp_agent_toolkit::handlers=trace pmat --mode mcp
```

## Version Compatibility

### Minimum Requirements
- **PMAT**: v0.26.0 or later
- **MCP Protocol**: v1.0
- **Claude Desktop**: Latest version
- **Claude Code**: v0.5.0 or later

### Feature Availability by Version

| Feature | Version | Notes |
|---------|---------|-------|
| Template tools | v0.26.0+ | 6 template management tools |
| Core analysis | v0.26.0+ | 4 fundamental analysis tools |
| Advanced analysis | v0.26.1+ | 17 specialized analysis tools |
| Vectorized tools | v0.27.0+ | 7 SIMD-accelerated tools |
| WebAssembly analysis | v0.26.2+ | WASM/AssemblyScript |
| Graph metrics | v0.26.1+ | PageRank, centrality |
| Refactor MCP mode | v0.27.2+ | Specialized refactoring server |
| Enhanced reports | v0.27.3+ | Multi-format comprehensive reports |

## Future Enhancements

- **WebSocket Transport**: Alternative to stdio transport
- **Batch Processing**: Multiple tools in one request
- **Subscription Support**: Real-time file system updates
- **Plugin System**: Custom tool development framework
- **Additional Languages**: Go, Java, C#, Swift support
- **Distributed Analysis**: Multi-node processing
- **Real-time Collaboration**: Live analysis sharing

## Using the pmcp Rust SDK

PMAT now supports running the MCP server using the [pmcp](https://github.com/paiml/pmcp) Rust SDK, which provides a more idiomatic Rust interface with improved type safety and async support.

### Benefits of pmcp SDK

- **Type Safety**: Strongly typed tool definitions and handlers
- **Async First**: Built on tokio for efficient async I/O
- **Better Error Handling**: Rust's Result type throughout
- **Connection Management**: Automatic lifecycle handling
- **Extensibility**: Easy to add custom tools

### Running with pmcp

```bash
# Run the example MCP server
cargo run --example mcp_server_pmcp

# Connect with an MCP client
npx @modelcontextprotocol/inspector tcp://127.0.0.1:3000
```

### Integration Example

```rust
use pmat::mcp_pmcp::{handlers::*, PmcpServer};
use pmcp::{Server, ServerBuilder, ToolHandler};

// Create server with all pmat tools
let server = ServerBuilder::new("pmat-mcp", "1.0.0")
    .with_tool("analyze_complexity", 
               "Analyze code complexity metrics", 
               Box::new(AnalyzeComplexityTool))
    .with_tool("analyze_satd", 
               "Detect self-admitted technical debt",
               Box::new(AnalyzeSatdTool))
    .with_tool("quality_gate",
               "Run comprehensive quality checks",
               Box::new(QualityGateTool))
    // Add more tools...
    .build();

// Handle connections
let listener = TcpListener::bind("127.0.0.1:3000").await?;
loop {
    let (stream, addr) = listener.accept().await?;
    let server = server.clone();
    tokio::spawn(async move {
        server.handle_connection(stream).await
    });
}
```

### Custom Tool Implementation

```rust
use pmcp::{ToolHandler, PmcpResult};
use async_trait::async_trait;
use serde_json::Value;

struct MyCustomTool;

#[async_trait]
impl ToolHandler for MyCustomTool {
    async fn handle(&self, args: Value) -> PmcpResult<Value> {
        // Parse arguments
        let path = args["path"].as_str()
            .ok_or_else(|| pmcp::Error::InvalidParams("path required"))?;
        
        // Perform analysis
        let result = analyze_something(path).await?;
        
        // Return JSON result
        Ok(serde_json::to_value(result)?)
    }
}
```

### Migration from stdio to pmcp

The pmcp SDK maintains compatibility with existing MCP clients while providing a cleaner implementation:

| Feature | stdio Implementation | pmcp SDK |
|---------|---------------------|----------|
| Protocol | Manual JSON-RPC | Built-in |
| Transport | stdio only | TCP, stdio, WebSocket (planned) |
| Error Handling | Custom enums | Standard Result<T, E> |
| Async | Tokio channels | Native async/await |
| Type Safety | Runtime validation | Compile-time types |

## Additional Resources

- [MCP Specification](https://modelcontextprotocol.io)
- [PMAT on crates.io](https://crates.io/crates/pmat)
- [pmcp SDK](https://github.com/paiml/pmcp)
- [API Documentation](https://docs.rs/pmat)
- [GitHub Repository](https://github.com/paiml/paiml-mcp-agent-toolkit)