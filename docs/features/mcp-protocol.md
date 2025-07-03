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
│  │              Tool Registry               │  │
│  │  • Template Tools (6)  • Analysis (17) │  │
│  │  • Vectorized (7)      • Core (4)      │  │
│  │  • Total: 34 available MCP tools       │  │
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
      "args": ["--mode", "mcp"],
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
# Start MCP server
pmat --mode mcp

# Or with specific configuration
pmat --mode mcp --config mcp-config.toml

# With environment variables
RUST_LOG=debug pmat --mode mcp
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

**Total: 34 MCP Tools Available**

## Template Management Tools (6 tools)

#### 1. `generate_template`
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

#### 2. `list_templates`
List all available project templates.

**Parameters:**
```typescript
{
  filter?: string;  // Optional filter by type
}
```

#### 3. `validate_template`
Validate template parameters before generation.

**Parameters:**
```typescript
{
  template_type: string;
  parameters: Record<string, any>;
}
```

#### 4. `scaffold_project`
Create complete project structure with templates.

**Parameters:**
```typescript
{
  path: string;
  project_type: "rust" | "typescript" | "python" | "cpp";
  features?: string[];
}
```

#### 5. `search_templates`
Search available templates by keyword.

**Parameters:**
```typescript
{
  query: string;
  limit?: number;
}
```

#### 6. `get_server_info`
Get MCP server information and capabilities.

**Parameters:** None

## Core Analysis Tools (4 tools)

#### 7. `analyze_complexity`
Analyzes code complexity metrics.

**Parameters:**
```typescript
{
  path: string;
  max_cyclomatic?: number;
  max_cognitive?: number;
  format?: "summary" | "full" | "json";
}
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

#### 8. `analyze_code_churn`
Analyzes git history for code churn patterns.

**Parameters:**
```typescript
{
  path: string;
  days?: number;         // Default: 30
  threshold?: number;    // Minimum commits
  format?: "table" | "json" | "csv";
}
```

**Response:**
```json
{
  "high_churn_files": [
    {
      "file": "src/core/engine.rs",
      "commits": 45,
      "authors": 8,
      "added_lines": 1250,
      "deleted_lines": 890,
      "churn_score": 0.85
    }
  ],
  "churn_trends": {
    "increasing": ["src/api/"],
    "decreasing": ["src/utils/"],
    "stable": ["src/models/"]
  }
}
```

#### 9. `generate_context`
Generates comprehensive project context for AI understanding.

**Parameters:**
```typescript
{
  path: string;
  format?: "markdown" | "json";
  include_dependencies?: boolean;
  max_depth?: number;
}
```

**Response:**
```json
{
  "context": {
    "project_type": "rust",
    "structure": { /* ... */ },
    "dependencies": { /* ... */ },
    "key_files": [ /* ... */ ],
    "complexity_summary": { /* ... */ },
    "recent_changes": [ /* ... */ ]
  }
}
```

#### 10. `analyze_dag`
Generates dependency analysis graphs.

**Parameters:**
```typescript
{
  path: string;
  output_format?: "mermaid" | "dot" | "json";
  max_depth?: number;
  filter_external?: boolean;
}
```

**Response:**
```json
{
  "graph": {
    "nodes": 45,
    "edges": 123,
    "cycles": 0,
    "max_depth": 8,
    "visualization": "graph TD\n  A[main] --> B[lib]\n  ..."
  }
}
```

## Advanced Analysis Tools (17 tools)

#### 11. `analyze_system_architecture`
Analyze high-level system architecture and component relationships.

**Parameters:**
```typescript
{
  path: string;
  output_format?: "mermaid" | "json";
  include_metrics?: boolean;
}
```

#### 12. `analyze_dead_code`
Detect unused and unreachable code.

**Parameters:**
```typescript
{
  path: string;
  aggressive?: boolean;
  exclude_tests?: boolean;
}
```

#### 13. `analyze_deep_context`
Comprehensive analysis with defect detection.

**Parameters:**
```typescript
{
  path: string;
  include_ml_predictions?: boolean;
  max_depth?: number;
}
```

#### 14. `analyze_tdg`
Technical Debt Gradient analysis.

**Parameters:**
```typescript
{
  path: string;
  strict?: boolean;
  include_predictions?: boolean;
}
```

#### 15. `analyze_makefile_lint`
Makefile quality and best practices analysis.

**Parameters:**
```typescript
{
  path: string;
  strict_mode?: boolean;
}
```

#### 16. `analyze_provability`
Abstract interpretation and formal verification analysis.

**Parameters:**
```typescript
{
  path: string;
  verification_level?: "basic" | "advanced";
}
```

#### 17. `analyze_defect_prediction`
ML-based defect probability analysis.

**Parameters:**
```typescript
{
  path: string;
  model?: "default" | "advanced";
  confidence_threshold?: number;
}
```

#### 18. `analyze_comprehensive`
Multi-dimensional analysis combining all analysis types.

**Parameters:**
```typescript
{
  path: string;
  include_all?: boolean;
  output_format?: "json" | "report";
}
```

#### 19. `analyze_graph_metrics`
Graph centrality and network analysis metrics.

**Parameters:**
```typescript
{
  path: string;
  metrics?: ("pagerank" | "betweenness" | "closeness" | "degree")[];
}
```

#### 20. `analyze_name_similarity`
Name similarity analysis with embeddings.

**Parameters:**
```typescript
{
  path: string;
  similarity_threshold?: number;
  include_suggestions?: boolean;
}
```

#### 21. `analyze_proof_annotations`
Collect and analyze proof annotations in code.

**Parameters:**
```typescript
{
  path: string;
  annotation_types?: string[];
}
```

#### 22. `analyze_incremental_coverage`
Incremental coverage analysis with caching.

**Parameters:**
```typescript
{
  path: string;
  baseline_ref?: string;
  cache_enabled?: boolean;
}
```

#### 23. `analyze_symbol_table`
Symbol analysis with cross-references.

**Parameters:**
```typescript
{
  path: string;
  include_cross_refs?: boolean;
  export_format?: "json" | "csv";
}
```

#### 24. `analyze_big_o`
Algorithmic complexity analysis.

**Parameters:**
```typescript
{
  path: string;
  include_worst_case?: boolean;
  analysis_depth?: "shallow" | "deep";
}
```

#### 25. `analyze_assemblyscript`
AssemblyScript code analysis.

**Parameters:**
```typescript
{
  path: string;
  optimization_level?: "O0" | "O1" | "O2" | "O3";
}
```

#### 26. `analyze_webassembly`
WebAssembly binary and text format analysis.

**Parameters:**
```typescript
{
  path: string;
  format?: "binary" | "text" | "auto";
  include_imports?: boolean;
}
```

#### 27. `analyze_duplicates`
Duplicate code detection with multiple algorithms.

**Parameters:**
```typescript
{
  path: string;
  algorithm?: "exact" | "fuzzy" | "semantic" | "all";
  min_lines?: number;
}
```

## Vectorized/SIMD Tools (7 tools)

*High-performance parallel analysis tools using SIMD instructions*

#### 28. `analyze_duplicates_vectorized`
SIMD-accelerated duplicate detection.

#### 29. `analyze_graph_metrics_vectorized`
Vectorized graph analysis with parallel processing.

#### 30. `analyze_name_similarity_vectorized`
SIMD-based name similarity computation.

#### 31. `analyze_symbol_table_vectorized`
Parallel symbol table analysis.

#### 32. `analyze_incremental_coverage_vectorized`
Vectorized coverage analysis.

#### 33. `analyze_big_o_vectorized`
Parallel Big-O complexity analysis.

#### 34. `generate_enhanced_report`
Generate comprehensive enhanced analysis reports.

**Parameters:**
```typescript
{
  path: string;
  analysis_types?: string[];
  output_format?: "markdown" | "html" | "json";
  include_visualizations?: boolean;
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
        
        # Check for technical debt
        tdg = self.call_tool("analyze_tdg", {"path": path})
        
        return {
            "context": context,
            "complexity": complexity,
            "technical_debt": tdg
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
| `PMAT_REFACTOR_MCP` | Enable refactor MCP server | `false` |
| `RUST_LOG` | Logging level | `info` |
| `DOCS_RS` | Docs.rs build mode | `false` |

### MCP Server Modes

PMAT supports two MCP server implementations:

1. **Standard MCP Server** - Full analysis capabilities
2. **Refactor MCP Server** - Specialized for refactoring workflows

```bash
# Standard mode
pmat --mode mcp

# Refactor mode
PMAT_REFACTOR_MCP=1 pmat --mode mcp
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

## Additional Resources

- [MCP Specification](https://modelcontextprotocol.io)
- [PMAT on crates.io](https://crates.io/crates/pmat)
- [API Documentation](https://docs.rs/pmat)
- [GitHub Repository](https://github.com/paiml/paiml-mcp-agent-toolkit)