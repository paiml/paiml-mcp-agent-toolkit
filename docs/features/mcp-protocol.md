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
│  │  • analyze_ast    • analyze_complexity  │  │
│  │  • analyze_churn  • generate_context    │  │
│  │  • analyze_dag    • analyze_deep        │  │
│  └──────────────────────────────────────────┘  │
└─────────────────────────────────────────────────┘
```

## Installation

### For Claude Desktop

1. Install PMAT:
```bash
curl -sSfL https://raw.githubusercontent.com/paiml/paiml-mcp-agent-toolkit/master/scripts/install.sh | sh
```

2. Configure Claude Desktop (`claude_desktop_config.json`):
```json
{
  "mcpServers": {
    "pmat": {
      "command": "pmat",
      "args": ["--mode", "mcp"]
    }
  }
}
```

### For Other MCP Clients

```bash
# Start MCP server
pmat --mode mcp

# Or with specific configuration
pmat --mode mcp --config mcp-config.toml
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

#### 1. `analyze_ast`
Performs Abstract Syntax Tree analysis on source code.

**Parameters:**
```typescript
{
  path: string;           // Project path
  language?: string;      // Force specific language
  include_metrics?: boolean;
}
```

**Response:**
```json
{
  "ast": {
    "total_nodes": 1523,
    "depth": 12,
    "functions": 87,
    "classes": 23,
    "complexity": {
      "average": 8.5,
      "max": 45
    }
  }
}
```

#### 2. `analyze_complexity`
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

#### 3. `analyze_churn`
Analyzes git history for code churn patterns.

**Parameters:**
```typescript
{
  path: string;
  days?: number;         // Default: 30
  threshold?: number;    // Minimum commits
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

#### 4. `generate_context`
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

#### 5. `analyze_dag`
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

## Security

### Authentication

```toml
[security]
require_auth = true
auth_token = "${MCP_AUTH_TOKEN}"
allowed_paths = ["/home/user/projects"]
```

### Rate Limiting

```toml
[rate_limit]
enabled = true
max_requests_per_minute = 100
burst_size = 20
```

## Monitoring

### Metrics

The MCP server exposes Prometheus metrics:

```
# HELP mcp_requests_total Total MCP requests
# TYPE mcp_requests_total counter
mcp_requests_total{method="tools/call",tool="analyze_complexity"} 1523

# HELP mcp_request_duration_seconds Request duration
# TYPE mcp_request_duration_seconds histogram
mcp_request_duration_seconds_bucket{le="0.1"} 1420
mcp_request_duration_seconds_bucket{le="0.5"} 1510
```

### Logging

```toml
[logging]
level = "info"
format = "json"
output = "stdout"

[logging.filters]
# Log all errors
error = "always"
# Sample 10% of successful requests
success = { sample_rate = 0.1 }
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

## Future Enhancements

- **WebSocket Transport**: Alternative to stdio
- **Batch Processing**: Multiple tools in one request
- **Subscription Support**: Real-time updates
- **Plugin System**: Custom tool development
- **Multi-Language Support**: Beyond current languages