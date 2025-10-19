# PMAT MCP Documentation

**Model Context Protocol** (MCP) integration for PMAT - Polyglot Multi-Language Analysis Toolkit

## Quick Links

- **[Tools Catalog](TOOLS.md)** - Complete list of 19 MCP tools with schemas and examples
- **[Integration Guide](INTEGRATION.md)** - How to connect clients and build workflows
- **[Examples](../../examples/mcp/)** - Working code examples

## What is MCP?

Model Context Protocol (MCP) is a standardized protocol for AI agents to interact with tools and services. PMAT exposes its code analysis capabilities via MCP, enabling:

- AI-powered code review
- Automated documentation validation
- Quality gate integration
- Technical debt analysis
- WebAssembly deep analysis

## Available Tools (19 total)

### Documentation Quality (2 tools)
- `validate_documentation` - Validate docs against codebase
- `check_claim` - Verify individual claims

### Code Quality (2 tools)
- `analyze_technical_debt` - TDG quality analysis
- `get_quality_recommendations` - Actionable refactoring suggestions

### Agent-Based Analysis (5 tools)
- `analyze` - Code analysis
- `transform` - Code transformation
- `validate` - Code validation
- `orchestrate` - Multi-agent workflows
- `quality_gate` - Comprehensive quality checks

### Deep WASM Analysis (5 tools)
- `deep_wasm_analyze` - Bytecode analysis
- `deep_wasm_query_mapping` - Source mappings
- `deep_wasm_trace_execution` - Execution tracing
- `deep_wasm_compare_optimizations` - Optimization comparison
- `deep_wasm_detect_issues` - Issue detection

### Semantic Search (4 tools)
- `semantic_search` - Semantic code search
- `find_similar_code` - Find similar patterns
- `cluster_code` - Cluster by similarity
- `analyze_topics` - Topic analysis

### Testing (1 tool)
- `mutation_test` - Mutation testing

## Quick Start

### 1. Start the Server

```bash
pmat mcp-server
```

### 2. Connect a Client

```javascript
import { McpClient } from '@modelcontextprotocol/sdk';

const client = new McpClient({
  endpoint: 'http://localhost:3000'
});

await client.connect();
```

### 3. Call a Tool

```javascript
// Validate documentation
const result = await client.callTool('validate_documentation', {
  documentation_path: 'README.md',
  deep_context_path: 'deep_context.md'
});

// Analyze code quality
const analysis = await client.callTool('analyze_technical_debt', {
  path: 'src/main.rs'
});

// Get refactoring recommendations
const recommendations = await client.callTool('get_quality_recommendations', {
  path: 'src/complex_module.rs',
  min_severity: 'high'
});
```

## Common Use Cases

### Pre-Commit Hook: Validate Documentation

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Generate deep context
pmat context --output deep_context.md

# Validate documentation via MCP
node scripts/validate-docs.js || exit 1
```

### CI/CD: Quality Gate

```yaml
# .github/workflows/quality.yml
- name: Quality Gate
  run: |
    pmat mcp-server &
    sleep 2
    node scripts/quality-gate.js
```

### AI Code Review Bot

```javascript
// Automatically review pull requests
const files = await getChangedFiles(pr);
const reviews = await aiCodeReview(client, files);
await postReviewComments(pr, reviews);
```

## Architecture

```
┌─────────────┐
│  AI Agent   │
└──────┬──────┘
       │ MCP Protocol
       │ (JSON-RPC over HTTP)
       ▼
┌─────────────┐
│ MCP Server  │ ← server/src/mcp_integration/server.rs
├─────────────┤
│   Tools     │
├─────────────┤
│ - validate_ │ ← hallucination_detection_tools.rs
│   documenta │
│   tion      │
│ - check_    │
│   claim     │
├─────────────┤
│ - analyze_  │ ← tdg_tools.rs
│   technical │
│   _debt     │
│ - get_      │
│   quality_  │
│   recommend │
│   ations    │
├─────────────┤
│ - analyze   │ ← tools.rs
│ - transform │
│ - validate  │
│ - orchestr  │
│   ate       │
├─────────────┤
│ - deep_wasm │ ← deep_wasm_tools.rs
│   _*        │
├─────────────┤
│ - semantic_ │ ← tools.rs (adapters)
│   search    │
├─────────────┤
│ - mutation_ │ ← mutation_tools.rs
│   test      │
└─────────────┘
       │
       ▼
┌─────────────┐
│  Services   │
├─────────────┤
│ - Hallucin  │
│   ation     │
│   Detector  │
│ - TDG       │
│   Analyzer  │
│ - Agent     │
│   Registry  │
│ - Deep WASM │
│ - Semantic  │
│   Search    │
└─────────────┘
```

## Protocol Compliance

- **Version**: MCP v2024-11-05
- **Transport**: HTTP/1.1 (JSON-RPC 2.0)
- **Capabilities**:
  - ✅ Tools
  - ✅ Resources (planned)
  - ✅ Prompts (planned)
  - ✅ Logging
  - ❌ Sampling (not applicable)

## Configuration

See [Integration Guide](INTEGRATION.md#server-configuration) for detailed configuration options.

## Error Handling

All tools follow consistent error patterns:

```json
{
  "code": -32602,
  "message": "Path does not exist: /invalid/path",
  "data": {
    "path": "/invalid/path",
    "suggestion": "Please provide a valid file or directory path"
  }
}
```

Error codes:
- `-32700`: Parse error
- `-32600`: Invalid request
- `-32601`: Method not found
- `-32602`: Invalid parameters
- `-32603`: Internal error

## Development History

- **Sprint 40a** (Oct 19, 2025): Hallucination Detection MCP Tools
- **Sprint 40c** (Oct 19, 2025): TDG Analysis MCP Tools
- **Sprint 40d** (Oct 19, 2025): MCP Documentation Enhancement

## Contributing

When adding new MCP tools:

1. Create tool in `server/src/mcp_integration/`
2. Implement `McpTool` trait
3. Register in `server.rs::register_defaults()`
4. Add tests (8+ tests following EXTREME TDD)
5. Document in `TOOLS.md`
6. Add usage examples

## Support

- **Issues**: https://github.com/pmat/pmat/issues
- **Documentation**: https://docs.pmat.dev
- **Slack**: #pmat-mcp channel

---

**Maintained by**: PMAT Development Team
**Protocol Version**: MCP v2024-11-05
**Last Updated**: Sprint 40d (October 19, 2025)
