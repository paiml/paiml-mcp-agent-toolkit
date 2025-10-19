# PMAT MCP Integration Guide

**Protocol Version**: MCP v2024-11-05
**Last Updated**: October 19, 2025

## Table of Contents

1. [Quick Start](#quick-start)
2. [Server Configuration](#server-configuration)
3. [Client Connection](#client-connection)
4. [Authentication](#authentication)
5. [Common Workflows](#common-workflows)
6. [Error Handling](#error-handling)
7. [Best Practices](#best-practices)

---

## Quick Start

### Start the PMAT MCP Server

```bash
# Start the server (default: localhost:3000)
pmat mcp-server

# Start with custom configuration
pmat mcp-server --bind 127.0.0.1:8080
```

### Connect a Client

```javascript
import { McpClient } from '@modelcontextprotocol/sdk';

const client = new McpClient({
  endpoint: 'http://localhost:3000',
  protocolVersion: '2024-11-05'
});

await client.connect();
```

---

## Server Configuration

### Default Configuration

The server uses the following defaults (from `server/src/mcp_integration/server.rs:31`):

```rust
ServerConfig {
    name: "PMAT MCP Server",
    version: env!("CARGO_PKG_VERSION"),
    bind_address: "127.0.0.1:3000",
    unix_socket: None,
    max_connections: 100,
    request_timeout: Duration::from_secs(30),
    enable_logging: true,

    // Semantic search (requires OPENAI_API_KEY)
    semantic_enabled: false,
    semantic_api_key: None,
    semantic_db_path: Some("~/.pmat/embeddings.db"),
    semantic_workspace: Some(cwd),
}
```

### Environment Variables

```bash
# Enable semantic search tools (optional)
export OPENAI_API_KEY="sk-..."
export PMAT_VECTOR_DB_PATH="~/.pmat/embeddings.db"
export PMAT_WORKSPACE="/path/to/workspace"
```

### Custom Configuration

```bash
# Custom bind address
pmat mcp-server --bind 0.0.0.0:8080

# Unix socket (for local IPC)
pmat mcp-server --unix-socket /tmp/pmat.sock

# Enable verbose logging
RUST_LOG=debug pmat mcp-server
```

---

## Client Connection

### TypeScript/JavaScript Client

```typescript
import { McpClient } from '@modelcontextprotocol/sdk';

async function connectToPMAT() {
  const client = new McpClient({
    endpoint: 'http://localhost:3000',
    protocolVersion: '2024-11-05',
    timeout: 30000 // 30 seconds
  });

  try {
    await client.connect();

    // Initialize the connection
    const initResponse = await client.initialize({
      clientInfo: {
        name: "my-ai-agent",
        version: "1.0.0"
      }
    });

    console.log('Connected to PMAT MCP Server:', initResponse.serverInfo);

    return client;
  } catch (error) {
    console.error('Connection failed:', error);
    throw error;
  }
}
```

### Python Client

```python
from mcp import Client

async def connect_to_pmat():
    client = Client(
        endpoint="http://localhost:3000",
        protocol_version="2024-11-05",
        timeout=30.0
    )

    await client.connect()

    # Initialize
    init_response = await client.initialize({
        "clientInfo": {
            "name": "my-ai-agent",
            "version": "1.0.0"
        }
    })

    print(f"Connected to: {init_response['serverInfo']['name']}")

    return client
```

---

## Authentication

**Current Status**: No authentication required for local connections.

**Future**: When deploying to production, consider:
- API key authentication
- OAuth 2.0
- mTLS for service-to-service

---

## Common Workflows

### Workflow 1: Validate Documentation Before Commit

```javascript
async function validateDocs(client) {
  // Step 1: Generate deep context
  await runCommand('pmat context --output deep_context.md');

  // Step 2: Validate documentation
  const result = await client.callTool('validate_documentation', {
    documentation_path: 'README.md',
    deep_context_path: 'deep_context.md',
    similarity_threshold: 0.7,
    fail_on_error: true
  });

  // Step 3: Check results
  if (!result.summary.pass) {
    console.error('Documentation validation failed!');
    console.error(`Contradictions: ${result.summary.contradictions}`);
    console.error(`Unverified claims: ${result.summary.unverified}`);
    process.exit(1);
  }

  console.log('✅ Documentation validated successfully');
}
```

### Workflow 2: Code Quality Check Before Merge

```javascript
async function qualityCheck(client, filePaths) {
  const issues = [];

  for (const path of filePaths) {
    // Analyze technical debt
    const analysis = await client.callTool('analyze_technical_debt', {
      path,
      include_penalties: true
    });

    // Get recommendations if score is low
    if (analysis.score.total < 70) {
      const recommendations = await client.callTool('get_quality_recommendations', {
        path,
        max_recommendations: 5,
        min_severity: 'high'
      });

      issues.push({
        file: path,
        score: analysis.score.total,
        grade: analysis.score.grade,
        recommendations: recommendations.recommendations
      });
    }
  }

  return issues;
}
```

### Workflow 3: AI-Assisted Code Review

```javascript
async function aiCodeReview(client, changedFiles) {
  const reviews = [];

  for (const file of changedFiles) {
    // Get quality recommendations
    const recommendations = await client.callTool('get_quality_recommendations', {
      path: file.path,
      max_recommendations: 10,
      min_severity: 'medium'
    });

    // Analyze technical debt
    const analysis = await client.callTool('analyze_technical_debt', {
      path: file.path,
      include_penalties: true
    });

    // Generate review comments
    const comments = recommendations.recommendations.map(rec => ({
      file: file.path,
      severity: rec.severity,
      message: `**${rec.category}**: ${rec.issue}\n\n` +
               `**Suggestion**: ${rec.suggestion}\n\n` +
               `**Impact**: ${rec.impact.toFixed(1)} points`,
      line: null // Could be enhanced with line numbers
    }));

    reviews.push({
      file: file.path,
      score: analysis.score.total,
      grade: analysis.score.grade,
      comments
    });
  }

  return reviews;
}
```

### Workflow 4: Documentation Accuracy CI/CD Gate

```javascript
async function documentationGate(client) {
  try {
    // Validate all documentation files
    const docFiles = ['README.md', 'CLAUDE.md', 'AGENT.md', 'GEMINI.md'];

    // Generate deep context once
    await runCommand('pmat context --output deep_context.md');

    let allPassed = true;
    const results = [];

    for (const docFile of docFiles) {
      const result = await client.callTool('validate_documentation', {
        documentation_path: docFile,
        deep_context_path: 'deep_context.md',
        similarity_threshold: 0.7,
        fail_on_error: false
      });

      results.push({
        file: docFile,
        passed: result.summary.pass,
        stats: result.summary
      });

      if (!result.summary.pass) {
        allPassed = false;
      }
    }

    // Generate report
    console.log('## Documentation Validation Report\n');
    for (const result of results) {
      const status = result.passed ? '✅ PASS' : '❌ FAIL';
      console.log(`### ${result.file}: ${status}`);
      console.log(`- Total Claims: ${result.stats.total_claims}`);
      console.log(`- Verified: ${result.stats.verified}`);
      console.log(`- Contradictions: ${result.stats.contradictions}`);
      console.log(`- Unverified: ${result.stats.unverified}\n`);
    }

    return allPassed;
  } catch (error) {
    console.error('Documentation gate failed:', error);
    return false;
  }
}
```

---

## Error Handling

### Error Code Reference

```javascript
const MCP_ERRORS = {
  PARSE_ERROR: -32700,
  INVALID_REQUEST: -32600,
  METHOD_NOT_FOUND: -32601,
  INVALID_PARAMS: -32602,
  INTERNAL_ERROR: -32603
};
```

### Handling Errors Gracefully

```javascript
async function robustToolCall(client, toolName, params) {
  try {
    return await client.callTool(toolName, params);
  } catch (error) {
    if (error.code === MCP_ERRORS.INVALID_PARAMS) {
      // Parameter validation error - check error.data.suggestion
      console.error(`Invalid parameters for ${toolName}:`, error.message);
      if (error.data?.suggestion) {
        console.error(`Suggestion: ${error.data.suggestion}`);
      }
      throw new Error(`Parameter error: ${error.message}`);
    } else if (error.code === MCP_ERRORS.INTERNAL_ERROR) {
      // Internal server error - retry with backoff
      console.warn(`Internal error in ${toolName}, retrying...`);
      await sleep(1000);
      return await client.callTool(toolName, params);
    } else {
      // Unknown error
      console.error(`Unexpected error calling ${toolName}:`, error);
      throw error;
    }
  }
}
```

### Timeout Handling

```javascript
async function callWithTimeout(client, toolName, params, timeoutMs = 30000) {
  return Promise.race([
    client.callTool(toolName, params),
    new Promise((_, reject) =>
      setTimeout(() => reject(new Error('Tool call timed out')), timeoutMs)
    )
  ]);
}
```

---

## Best Practices

### 1. Connection Management

```javascript
// Use connection pooling for multiple requests
class PMATClient {
  constructor(endpoint) {
    this.endpoint = endpoint;
    this.client = null;
  }

  async connect() {
    if (!this.client) {
      this.client = new McpClient({ endpoint: this.endpoint });
      await this.client.connect();
      await this.client.initialize({
        clientInfo: { name: "pmat-client", version: "1.0.0" }
      });
    }
    return this.client;
  }

  async disconnect() {
    if (this.client) {
      await this.client.disconnect();
      this.client = null;
    }
  }
}
```

### 2. Caching Deep Context

```javascript
// Generate deep context once, reuse for multiple validations
async function generateDeepContext() {
  const cacheFile = '.pmat-cache/deep_context.md';
  const cacheAge = await getFileAge(cacheFile);

  // Regenerate if cache is older than 1 hour
  if (cacheAge > 3600) {
    await runCommand('pmat context --output ' + cacheFile);
  }

  return cacheFile;
}
```

### 3. Batch Processing

```javascript
// Process files in batches to avoid overwhelming the server
async function analyzeBatch(client, files, batchSize = 10) {
  const results = [];

  for (let i = 0; i < files.length; i += batchSize) {
    const batch = files.slice(i, i + batchSize);
    const batchResults = await Promise.all(
      batch.map(file =>
        client.callTool('analyze_technical_debt', { path: file })
      )
    );
    results.push(...batchResults);
  }

  return results;
}
```

### 4. Logging and Monitoring

```javascript
// Wrap client calls with logging
async function loggedToolCall(client, toolName, params) {
  const startTime = Date.now();

  try {
    console.log(`[MCP] Calling ${toolName}...`);
    const result = await client.callTool(toolName, params);
    const duration = Date.now() - startTime;
    console.log(`[MCP] ${toolName} completed in ${duration}ms`);
    return result;
  } catch (error) {
    const duration = Date.now() - startTime;
    console.error(`[MCP] ${toolName} failed after ${duration}ms:`, error.message);
    throw error;
  }
}
```

### 5. Health Checks

```javascript
async function checkServerHealth(client) {
  try {
    // List tools as a health check
    const tools = await client.listTools();
    return {
      healthy: true,
      toolCount: tools.tools.length,
      timestamp: new Date().toISOString()
    };
  } catch (error) {
    return {
      healthy: false,
      error: error.message,
      timestamp: new Date().toISOString()
    };
  }
}
```

---

## Troubleshooting

### Server Won't Start

```bash
# Check if port is already in use
lsof -i :3000

# Check logs
RUST_LOG=debug pmat mcp-server

# Check firewall
sudo ufw status
```

### Connection Timeouts

```javascript
// Increase timeout for slow operations
const client = new McpClient({
  endpoint: 'http://localhost:3000',
  timeout: 120000 // 2 minutes for large projects
});
```

### Semantic Search Not Available

```bash
# Ensure OpenAI API key is set
echo $OPENAI_API_KEY

# Check server logs for semantic tool registration
RUST_LOG=info pmat mcp-server | grep semantic
```

---

## Next Steps

- [Tools Catalog](TOOLS.md) - Complete list of available tools
- [Examples](../../examples/mcp/) - Working code examples
- [Architecture](../architecture/) - System design documentation

---

**Maintained by**: PMAT Development Team
**Last Updated**: Sprint 40d (October 19, 2025)
