# 🤖 PMAT Agent Scaffolding - Comprehensive Guide

## Table of Contents
1. [Overview](#overview)
2. [Agent Types](#agent-types)
3. [Scaffolding Workflows](#scaffolding-workflows)
4. [Template System](#template-system)
5. [Quality Standards](#quality-standards)
6. [MCP Integration](#mcp-integration)
7. [CI/CD Pipeline](#cicd-pipeline)
8. [Best Practices](#best-practices)

## Overview

PMAT's agent scaffolding system provides deterministic, high-quality code generation for creating MCP (Model Context Protocol) agents, state machines, and hybrid AI systems. Every generated agent follows extreme quality standards with built-in testing, documentation, and monitoring.

### Key Features
- **🎯 Deterministic Generation**: Reproducible agent creation with seed-based generation
- **🏗️ Multiple Templates**: MCP servers, state machines, hybrid agents, calculators
- **📊 Quality Enforcement**: TDD, 80% coverage, complexity ≤10 enforced
- **🔄 State Management**: Built-in state machine patterns for complex workflows
- **🔌 MCP Native**: First-class Model Context Protocol support
- **📈 Monitoring Built-in**: Telemetry, metrics, and observability from day one

## Agent Types

### 1. MCP Server Agent
**Purpose**: Standard MCP-compliant server for tool integration

```bash
pmat scaffold agent \
  --name my-tools \
  --template mcp-server \
  --features "file-ops,network,database" \
  --quality extreme
```

**Generated Structure:**
```
my-tools/
├── src/
│   ├── main.rs           # MCP server entry point
│   ├── tools/            # Tool implementations
│   │   ├── file_ops.rs   # File operation tools
│   │   ├── network.rs    # Network tools
│   │   └── database.rs   # Database tools
│   ├── handlers/         # Request handlers
│   ├── validation/       # Input validation
│   └── tests/           # Comprehensive tests
├── Cargo.toml           # Dependencies
├── mcp.json            # MCP manifest
└── README.md           # Documentation
```

### 2. State Machine Agent
**Purpose**: Complex workflow automation with deterministic states

```bash
pmat scaffold agent \
  --name workflow-engine \
  --template state-machine \
  --features "approval,retry,rollback" \
  --quality strict
```

**State Machine Features:**
- Deterministic state transitions
- Event-driven architecture
- Rollback capability
- Audit logging
- Visual state diagram generation

### 3. Hybrid Agent (Deterministic + Probabilistic)
**Purpose**: Combines deterministic core with AI capabilities

```bash
pmat scaffold agent \
  --name smart-analyzer \
  --template hybrid \
  --deterministic-core "ast-parser,rules-engine" \
  --probabilistic-wrapper "llm-interface,embedding-search" \
  --quality extreme
```

**Architecture:**
```
┌─────────────────────────────────────┐
│     Probabilistic Wrapper (AI)      │
│  • LLM Integration                  │
│  • Semantic Search                  │
│  • Pattern Learning                 │
├─────────────────────────────────────┤
│      Deterministic Core             │
│  • AST Parsing                      │
│  • Rules Engine                     │
│  • State Management                 │
└─────────────────────────────────────┘
```

### 4. Calculator Agent
**Purpose**: Mathematical and computational operations

```bash
pmat scaffold agent \
  --name math-engine \
  --template calculator \
  --features "statistics,linear-algebra,optimization" \
  --quality standard
```

### 5. Custom Agent
**Purpose**: User-defined template

```bash
pmat scaffold agent \
  --name custom-agent \
  --template custom:./templates/my-agent.yaml \
  --quality extreme
```

## Scaffolding Workflows

### Interactive Mode

```bash
# Guided agent creation with prompts
pmat scaffold agent --interactive

# Interactive prompts:
# 1. Agent name: my-agent
# 2. Template type: [mcp-server/state-machine/hybrid/calculator/custom]
# 3. Features to include: file-ops, network
# 4. Quality level: [standard/strict/extreme]
# 5. Output directory: ./agents/my-agent
# 6. Overwrite if exists? [y/n]
```

### Batch Generation

```bash
# Generate multiple agents from specification
cat > agents.yaml << EOF
agents:
  - name: file-manager
    template: mcp-server
    features: [file-ops, compression]
  - name: data-processor
    template: state-machine
    features: [validation, transformation]
  - name: ai-assistant
    template: hybrid
    deterministic: [parser]
    probabilistic: [llm]
EOF

pmat scaffold agent --batch agents.yaml
```

### Dry Run Mode

```bash
# Preview what will be generated without creating files
pmat scaffold agent \
  --name test-agent \
  --template mcp-server \
  --dry-run

# Output:
# Would create:
#   ./test-agent/src/main.rs (2.3 KB)
#   ./test-agent/src/tools/mod.rs (1.1 KB)
#   ./test-agent/Cargo.toml (0.8 KB)
#   ./test-agent/mcp.json (0.5 KB)
#   ./test-agent/tests/integration.rs (3.2 KB)
#   ./test-agent/README.md (4.5 KB)
# Total: 6 files, 12.4 KB
```

## Template System

### Built-in Templates

#### MCP Server Template
```yaml
# templates/mcp-server.yaml
name: mcp-server
version: 1.0.0
description: Standard MCP server implementation

structure:
  - path: src/main.rs
    template: mcp_server_main.rs.hbs
  - path: src/tools/mod.rs
    template: tools_module.rs.hbs
  - path: mcp.json
    template: mcp_manifest.json.hbs

features:
  file-ops:
    files:
      - src/tools/file_ops.rs
    dependencies:
      - tokio-fs = "1.0"
  network:
    files:
      - src/tools/network.rs
    dependencies:
      - reqwest = "0.11"

quality:
  standard:
    max_complexity: 20
    min_coverage: 60
  strict:
    max_complexity: 15
    min_coverage: 70
  extreme:
    max_complexity: 10
    min_coverage: 80
```

### Custom Templates

```yaml
# my-custom-template.yaml
name: custom-service
version: 1.0.0

structure:
  - path: src/main.rs
    content: |
      fn main() {
          println!("{{agent_name}} starting...");
      }

variables:
  - name: agent_name
    required: true
    default: "MyAgent"
  - name: port
    required: false
    default: 3000

validation:
  - rule: agent_name_valid
    pattern: "^[a-z][a-z0-9-]*$"
    message: "Agent name must be lowercase with hyphens"
```

### Template Validation

```bash
# Validate a custom template
pmat scaffold validate-template ./my-template.yaml

# Output:
# ✅ Template structure valid
# ✅ Variables defined correctly
# ✅ File paths valid
# ⚠️ Warning: No tests defined
# 
# Template can be used with:
# pmat scaffold agent --template custom:./my-template.yaml
```

## Quality Standards

### Enforced Standards

Every generated agent enforces:

1. **Complexity Limits**
   - Cyclomatic: ≤10 (extreme), ≤15 (strict), ≤20 (standard)
   - Cognitive: ≤10 (extreme), ≤15 (strict), ≤20 (standard)

2. **Test Coverage**
   - ≥80% (extreme), ≥70% (strict), ≥60% (standard)
   - Unit tests for all public functions
   - Integration tests for all tools
   - Property tests for state machines

3. **Documentation**
   - README with usage examples
   - Inline documentation for all public APIs
   - Architecture diagrams for complex agents

4. **Code Quality**
   - Zero SATD comments
   - No clippy warnings
   - Formatted with rustfmt

### Quality Verification

```bash
# Generated agents include quality checks
cd my-agent/

# Run quality gate
make quality-gate

# Run tests with coverage
make test-coverage

# Check complexity
pmat analyze complexity src/
```

## MCP Integration

### Generated MCP Manifest

```json
{
  "name": "my-agent",
  "version": "1.0.0",
  "description": "Generated MCP agent",
  "tools": [
    {
      "name": "read_file",
      "description": "Read file contents",
      "inputSchema": {
        "type": "object",
        "properties": {
          "path": {
            "type": "string",
            "description": "File path to read"
          }
        },
        "required": ["path"]
      }
    }
  ],
  "configuration": {
    "max_file_size": {
      "type": "integer",
      "default": 10485760,
      "description": "Maximum file size in bytes"
    }
  }
}
```

### MCP Tool Implementation

```rust
// Generated tool implementation
#[derive(Debug, Deserialize)]
struct ReadFileArgs {
    path: String,
}

async fn handle_read_file(args: ReadFileArgs) -> Result<String> {
    // Validation
    validate_path(&args.path)?;
    
    // Implementation with error handling
    let content = tokio::fs::read_to_string(&args.path)
        .await
        .context("Failed to read file")?;
    
    // Return with proper formatting
    Ok(content)
}

// Automatic test generation
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_read_file_success() {
        let args = ReadFileArgs {
            path: "test.txt".to_string(),
        };
        let result = handle_read_file(args).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_read_file_not_found() {
        let args = ReadFileArgs {
            path: "nonexistent.txt".to_string(),
        };
        let result = handle_read_file(args).await;
        assert!(result.is_err());
    }
}
```

## CI/CD Pipeline

### Generated GitHub Actions

```yaml
# .github/workflows/agent-ci.yml (generated)
name: Agent CI
on: [push, pull_request]

jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install PMAT
        run: cargo install pmat --locked
      
      - name: Quality Gate
        run: pmat quality-gate --file src/
      
      - name: Complexity Check
        run: |
          pmat analyze complexity src/
          # Fail if complexity > 10
          pmat analyze complexity src/ --max 10
      
      - name: Test Coverage
        run: |
          cargo llvm-cov --html
          # Fail if coverage < 80%
          cargo llvm-cov --fail-under 80
      
      - name: MCP Validation
        run: pmat validate mcp.json
```

### Docker Support

```dockerfile
# Generated Dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/my-agent /usr/local/bin/
COPY --from=builder /app/mcp.json /etc/my-agent/

EXPOSE 3000
CMD ["my-agent", "--config", "/etc/my-agent/mcp.json"]
```

## Best Practices

### 1. Start with Templates

```bash
# List available templates
pmat scaffold list-templates

# Show template details
pmat scaffold agent --template mcp-server --show-template

# Always start with a template
pmat scaffold agent \
  --name my-agent \
  --template mcp-server \
  --features "file-ops"
```

### 2. Use Interactive Mode for Learning

```bash
# Interactive mode provides guidance
pmat scaffold agent --interactive

# It will:
# - Explain each option
# - Suggest best practices
# - Validate inputs
# - Show preview before generation
```

### 3. Enforce Quality from Start

```bash
# Always use strict or extreme quality
pmat scaffold agent \
  --name production-agent \
  --template state-machine \
  --quality extreme

# This ensures:
# - Lower technical debt
# - Better maintainability
# - Easier debugging
# - Higher reliability
```

### 4. Version Control Integration

```bash
# Generate with git initialization
pmat scaffold agent \
  --name my-agent \
  --template mcp-server \
  --git-init

# Creates:
# - .gitignore with Rust patterns
# - Initial commit
# - Pre-commit hooks for quality
```

### 5. Testing Strategy

```bash
# Generated agents include test commands
cd my-agent/

# Unit tests
cargo test --lib

# Integration tests
cargo test --test integration

# Property tests (if state machine)
cargo test --features proptest

# Coverage report
cargo llvm-cov --html
```

## Advanced Features

### State Machine Visualization

```bash
# Generate state diagram for state machine agents
pmat scaffold agent \
  --name workflow \
  --template state-machine \
  --visualize

# Creates workflow.dot and workflow.svg
# Shows all states and transitions visually
```

### Multi-Agent Systems

```bash
# Generate coordinated agent system
cat > multi-agent.yaml << EOF
system:
  name: distributed-processor
  agents:
    - name: coordinator
      template: state-machine
      role: orchestrator
    - name: worker-1
      template: mcp-server
      role: processor
    - name: worker-2
      template: mcp-server
      role: processor
  communication:
    protocol: grpc
    service_discovery: consul
EOF

pmat scaffold agent --system multi-agent.yaml
```

### Agent Migration

```bash
# Migrate existing code to agent structure
pmat scaffold agent \
  --name migrated-agent \
  --template mcp-server \
  --migrate-from ./legacy-code/
```

## Integration with PMAT Ecosystem

### Quality Gate Integration

```toml
# Generated pmat.toml
[agent.quality]
complexity_limit = 10
coverage_minimum = 80
satd_tolerance = 0

[agent.monitoring]
telemetry = true
metrics = true
tracing = true
```

### MCP Tool Testing

```javascript
// Test generated MCP tools
const agent = await mcp.connect('localhost:3000');

// List available tools
const tools = await agent.listTools();
console.log(`Agent provides ${tools.length} tools`);

// Test each tool
for (const tool of tools) {
  const result = await agent.callTool(tool.name, tool.testData);
  assert(result.success, `Tool ${tool.name} failed`);
}
```

---

**Version**: 2.71.0 | **Last Updated**: 2025-09-09 | **Status**: Production Ready