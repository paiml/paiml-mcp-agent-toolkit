# CLI & HTTP API

> Sub-spec of [pmat-spec.md](../pmat-spec.md) | Component 12

## CLI Structure

### Command Hierarchy

```
pmat
├── query           # Semantic code search
├── context         # Generate project context
├── analyze         # Run analysis (complexity, tdg, churn, etc.)
├── comply          # Compliance checks (33+ checks)
├── work            # Work management (tickets, contracts)
├── five-whys       # Root cause analysis
├── rust-score      # Rust project scoring
├── validate-readme # Documentation accuracy
├── scaffold        # Project scaffolding
└── publish         # Registry publishing
```

### Implementation

- Parser: clap derive macros
- Stack size: 8 MB minimum (clap tests SIGABRT without it)
- `RUST_MIN_STACK=8388608` required for `make test-lib`

## Unified --help Generation

Dynamic documentation across CLI, MCP, and HTTP:
- Single source of truth for command descriptions
- Auto-generated from code annotations
- Includes examples and default values

## HTTP API

### Actix-web Server

```
GET  /api/v1/health         # Health check
POST /api/v1/query           # Semantic search
POST /api/v1/analyze         # Run analysis
GET  /api/v1/comply          # Compliance report
GET  /api/v1/metrics         # Project metrics
```

### Schema Validation

- Request/response schemas via serde + JSON Schema
- OpenAPI specification auto-generated

## Acceptance Testing

### CLI Tests

```bash
# Validate command exists and --help works
pmat query --help
# Validate output format
pmat query "test" --format json | jq .
# Validate exit codes
pmat comply check; echo $?
```

### HTTP API Tests

```bash
# Validate endpoint availability
curl -s http://localhost:8080/api/v1/health | jq .status
# Validate schema
curl -s -X POST http://localhost:8080/api/v1/query -d '{"query":"test"}' | jq .
```

## Key Files

| File | Purpose |
|------|---------|
| `src/cli/` | CLI command definitions |
| `src/cli/handlers/` | Command handler implementations |
| `src/http/` | HTTP API server |

## References

- Consolidated from: cli-specification, http-api-specification, cli-acceptance-testing,
  http-api-acceptance-testing, unified-cli-mcp-help-integration
