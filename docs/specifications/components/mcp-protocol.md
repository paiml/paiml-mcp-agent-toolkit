# MCP & Protocols

> Sub-spec of [pmat-spec.md](../pmat-spec.md) | Component 11

## MCP Server

### Architecture

PMAT exposes analysis tools via Model Context Protocol (MCP):
- stdio transport (primary)
- SSE transport (web integration)
- Tool registration with JSON Schema validation

### Tool Registration

Each tool declares:
- Name and description
- Input schema (JSON Schema)
- Output format (text or structured JSON)

### Acceptance Testing

Mock server validates:
- Tool discovery (list_tools)
- Schema validation on inputs
- Error handling for invalid requests
- Timeout behavior for long-running analyses

## Registry Publishing

### Package Distribution

```bash
pmat publish --registry mcp
```

Publishes to MCP registry with:
- Version from Cargo.toml
- Tool manifest from code annotations
- README and changelog

## Key Files

| File | Purpose |
|------|---------|
| `src/mcp_server/` | MCP server implementation |
| `src/mcp_pmcp/` | PMCP protocol handler |
| `src/models/mcp_types.rs` | MCP type definitions |

## References

- Consolidated from: mcp-specification, mcp-acceptance-testing, publish-mcp-registry
