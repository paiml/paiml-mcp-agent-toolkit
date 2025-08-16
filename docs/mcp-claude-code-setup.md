# PMAT MCP Integration with Claude Code - Complete Setup Guide

## Table of Contents
- [Overview](#overview)
- [Installation](#installation)
- [Claude Code Configuration](#claude-code-configuration)
- [Available Tools](#available-tools)
- [PDMT Deterministic Todo Tool](#pdmt-deterministic-todo-tool)
- [Quality Gates Proxy](#quality-gates-proxy)
- [Running Examples](#running-examples)
- [Testing MCP Tools](#testing-mcp-tools)
- [Troubleshooting](#troubleshooting)

## Overview

PMAT (PAIML MCP Agent Toolkit) provides a comprehensive Model Context Protocol (MCP) server that integrates seamlessly with Claude Code. The MCP server offers 18+ tools for code analysis, refactoring, quality enforcement, and AI-assisted development.

### Key Features
- **Unified MCP Server**: Single implementation using high-performance pmcp SDK
- **Quality Enforcement**: Zero-tolerance quality gates with automated validation
- **PDMT Integration**: Deterministic todo generation with quality requirements
- **GitHub Integration**: Create and manage issues with quality enforcement
- **10x Performance**: All operations optimized with pmcp SDK

## Installation

### Step 1: Install PMAT

```bash
# Install from crates.io (recommended)
cargo install pmat

# Or build from source
git clone https://github.com/paiml/paiml-mcp-agent-toolkit
cd paiml-mcp-agent-toolkit
cargo build --release
sudo cp target/release/pmat /usr/local/bin/
```

### Step 2: Verify Installation

```bash
# Check version
pmat --version

# Test MCP server mode
echo '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"1.0.0","capabilities":{},"clientInfo":{"name":"test"}},"id":1}' | pmat

# Run a simple example
cargo run --example mcp_server_pmcp
```

## Claude Code Configuration

### Step 1: Locate Claude Code Settings

Claude Code stores MCP server configurations in:
- **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Linux**: `~/.config/Claude/claude_desktop_config.json`
- **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

### Step 2: Configure PMAT MCP Server

Edit the configuration file to add PMAT:

```json
{
  "mcpServers": {
    "pmat": {
      "command": "pmat",
      "args": [],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

### Step 3: Advanced Configuration Options

For more detailed logging and features:

```json
{
  "mcpServers": {
    "pmat": {
      "command": "pmat",
      "args": [],
      "env": {
        "RUST_LOG": "debug",
        "PMAT_CACHE_DIR": "/tmp/pmat-cache",
        "PMAT_QUALITY_STRICT": "true",
        "PMAT_PDMT_SEED": "42"
      }
    }
  }
}
```

### Step 4: Restart Claude Code

After updating the configuration:
1. Completely quit Claude Code (Cmd+Q on macOS, Alt+F4 on Windows/Linux)
2. Start Claude Code again
3. Open a new conversation
4. The MCP icon should appear in the conversation interface

### Step 5: Verify MCP Connection

In a new Claude Code conversation, you should see:
- MCP server icon indicating active connection
- Available tools when you click the tools menu
- Server info: "pmat - PAIML MCP Agent Toolkit"

## Available Tools

### Core Analysis Tools

#### analyze_complexity
Analyze code complexity with cyclomatic and cognitive metrics.

```bash
# Example via CLI
pmat analyze complexity --top-files 10

# Example via cargo
cargo run --example complexity_demo

# MCP usage in Claude Code
# Just ask: "Analyze the complexity of the codebase"
```

#### analyze_dead_code
Detect unused functions, structs, and modules.

```bash
# Example via CLI
pmat analyze dead-code

# Example via cargo
cargo run --example analyze_dead_code

# MCP usage in Claude Code
# Just ask: "Find dead code in this project"
```

#### analyze_satd
Find self-admitted technical debt in comments.

```bash
# Example via CLI
pmat analyze satd

# Example via cargo
cargo run --example analyze_satd

# MCP usage in Claude Code
# Just ask: "Check for technical debt comments"
```

#### analyze_lint_hotspot
Identify files with the most linting issues.

```bash
# Example via CLI
pmat analyze lint-hotspot --top-files 5

# Example via cargo
cargo run --example lint_hotspot_demo

# MCP usage in Claude Code
# Just ask: "Find lint hotspots in the codebase"
```

### Context Generation

#### generate_context
Generate comprehensive project context for AI assistants.

```bash
# Example via CLI
pmat context --format markdown

# Example via cargo
cargo run --example deep_context_complexity

# MCP usage in Claude Code
# Just ask: "Generate project context"
```

### Refactoring Tools

#### refactor_start
Begin an automated refactoring session.

```bash
# Example via CLI
pmat refactor auto --file src/main.rs

# Example via cargo
cargo run --example pmcp_refactor_session

# MCP usage in Claude Code
# Just ask: "Start refactoring src/main.rs"
```

## PDMT Deterministic Todo Tool

PDMT (Pragmatic Deterministic MCP Templating) generates high-quality, deterministic todo lists with validation commands and success criteria.

### How PDMT Works

1. **Deterministic Generation**: Uses fixed seed for reproducible outputs
2. **Quality Requirements**: Each todo includes test requirements and validation
3. **Implementation Specs**: Detailed architectural guidance
4. **Success Criteria**: Measurable outcomes for each task

### Using PDMT in Claude Code

```javascript
// MCP tool call structure
{
  "tool": "pdmt_deterministic_todos",
  "parameters": {
    "requirements": ["implement authentication", "add logging"],
    "granularity": "medium",
    "seed": 42
  }
}
```

### PDMT Examples

```bash
# Generate todos via CLI
pmat pdmt-todos "implement OAuth authentication" --granularity medium --seed 42

# Example via cargo
cargo run --example pmcp_analyze_workflow

# In Claude Code, just ask:
# "Create a PDMT todo list for implementing user authentication"
```

### PDMT Output Format

Each todo includes:
- **Title**: Clear, actionable task description
- **Implementation**: Specific code changes required
- **Validation**: Commands to verify completion
- **Success Criteria**: Measurable outcomes
- **Dependencies**: Related tasks and prerequisites

Example PDMT todo:

```yaml
- id: auth-001
  title: "Implement JWT token generation"
  implementation:
    - Create token service in src/auth/token.rs
    - Add RS256 signing with private key
    - Include user claims and expiration
  validation:
    - "cargo test auth::token"
    - "pmat quality-gate --file src/auth/token.rs"
  success_criteria:
    - All token tests pass
    - Complexity ≤ 20
    - 100% test coverage
  dependencies: ["auth-config"]
```

## Quality Gates Proxy

The Quality Gates Proxy intercepts and validates all code changes before they're applied, ensuring zero-tolerance quality standards.

### How Quality Proxy Works

1. **Intercepts Changes**: Captures all file modifications
2. **Validates Quality**: Runs comprehensive quality checks
3. **Enforces Standards**: Blocks changes that violate thresholds
4. **Provides Feedback**: Detailed reports on violations

### Quality Gate Configuration

```bash
# Run quality gate with strict mode
pmat quality-gate --strict

# Example via cargo
cargo run --example quality_gate
cargo run --example quality_gate_custom
cargo run --example quality_proxy_demo

# Check specific file
pmat quality-gate --file src/main.rs
```

### Quality Standards Enforced

| Metric | Threshold | Description |
|--------|-----------|-------------|
| Complexity | ≤ 20 | Cyclomatic complexity per function |
| SATD | 0 | Zero self-admitted technical debt |
| Test Coverage | ≥ 80% | Minimum code coverage |
| Lint Violations | 0 | No clippy warnings |
| Dead Code | 0 | No unused functions/structs |

### Using Quality Proxy in Claude Code

```javascript
// MCP tool call
{
  "tool": "quality_gate",
  "parameters": {
    "file_path": "src/main.rs",
    "strict": true
  }
}
```

In Claude Code conversations:
- "Check quality of src/main.rs"
- "Run quality gates on the entire project"
- "Validate this code meets quality standards"

## Running Examples

PMAT includes numerous examples demonstrating MCP features:

### Basic Examples

```bash
# Complexity analysis
cargo run --example complexity_demo
cargo run --example one_function_only
cargo run --example single_function_test

# Dead code detection
cargo run --example analyze_dead_code

# SATD analysis
cargo run --example analyze_satd
cargo run --example satd_lint_analysis

# Quality gates
cargo run --example quality_gate
cargo run --example quality_gate_custom
cargo run --example quality_gate_thresholds
cargo run --example quality_gate_shows_checks
```

### MCP Server Examples

```bash
# Run MCP server
cargo run --example mcp_server_pmcp

# Test MCP server
cargo run --example test_pmcp_server

# Unified MCP demo
cargo run --example unified_mcp_demo

# MCP workflow examples
cargo run --example pmcp_analyze_workflow
cargo run --example pmcp_refactor_session
```

### Advanced Examples

```bash
# Quality proxy demonstration
cargo run --example quality_proxy_demo

# GitHub integration
cargo run --example check_github_repo

# CI/CD integration
cargo run --example ci_integration

# Lint hotspot analysis
cargo run --example lint_hotspot_demo
cargo run --example lint_hotspot_enforce_flag

# Deep context analysis
cargo run --example deep_context_complexity
```

### Interactive Examples

```bash
# Scaffold agent examples
cargo run --example scaffold_agent_basics
cargo run --example scaffold_agent_interactive
cargo run --example scaffold_agent_hybrid
cargo run --example scaffold_agent_course_project
```

## Testing MCP Tools

### Unit Tests for MCP

```rust
/// Example doctest for PDMT tool
/// ```rust
/// use pmat::mcp_pmcp::pdmt_handler::generate_todos;
/// 
/// let requirements = vec!["implement auth".to_string()];
/// let todos = generate_todos(requirements, "medium", 42);
/// assert!(!todos.is_empty());
/// assert!(todos[0].contains("validation"));
/// ```
```

### Integration Tests

```bash
# Run all MCP tests
cargo test mcp

# Run specific MCP test suites
cargo test test_pmcp_server
cargo test mcp_protocol
cargo test mcp_server_integration

# Run property tests for MCP
cargo test mcp_property_tests
```

### Manual Testing with curl

```bash
# Test MCP initialization
echo '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"1.0.0","capabilities":{},"clientInfo":{"name":"test"}},"id":1}' | pmat

# List available tools
echo '{"jsonrpc":"2.0","method":"tools/list","params":{},"id":2}' | pmat

# Call analyze_complexity tool
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"analyze_complexity","arguments":{"top_files":5}},"id":3}' | pmat
```

## Troubleshooting

### Common Issues

#### MCP Server Not Appearing in Claude Code

1. **Check configuration path**:
   ```bash
   # macOS
   cat ~/Library/Application\ Support/Claude/claude_desktop_config.json
   
   # Linux
   cat ~/.config/Claude/claude_desktop_config.json
   ```

2. **Verify pmat is in PATH**:
   ```bash
   which pmat
   # Should output: /usr/local/bin/pmat or similar
   ```

3. **Test MCP server directly**:
   ```bash
   # This should output JSON responses
   echo '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"1.0.0","capabilities":{},"clientInfo":{"name":"test"}},"id":1}' | pmat
   ```

#### Debug Logging

Enable debug logging to troubleshoot issues:

```json
{
  "mcpServers": {
    "pmat": {
      "command": "pmat",
      "args": [],
      "env": {
        "RUST_LOG": "debug",
        "RUST_BACKTRACE": "1"
      }
    }
  }
}
```

Check logs in:
- **macOS/Linux**: `~/Library/Logs/Claude/` or check terminal output
- **Windows**: `%LOCALAPPDATA%\Claude\logs\`

#### Permission Issues

If you get permission denied errors:

```bash
# Make pmat executable
chmod +x /usr/local/bin/pmat

# Or use cargo run directly in config
{
  "mcpServers": {
    "pmat": {
      "command": "cargo",
      "args": ["run", "--release", "--", "mcp"],
      "cwd": "/path/to/paiml-mcp-agent-toolkit"
    }
  }
}
```

### Getting Help

- **Documentation**: [GitHub Wiki](https://github.com/paiml/paiml-mcp-agent-toolkit/wiki)
- **Issues**: [GitHub Issues](https://github.com/paiml/paiml-mcp-agent-toolkit/issues)
- **Examples**: Run `ls server/examples/*.rs` to see all available examples
- **MCP Spec**: [Model Context Protocol](https://modelcontextprotocol.io)

## Next Steps

1. **Explore Examples**: Run the various cargo examples to understand features
2. **Read CLAUDE.md**: Learn about quality standards and best practices
3. **Try PDMT**: Generate todos for your next feature
4. **Enable Quality Proxy**: Ensure all code meets standards
5. **Integrate with CI/CD**: Use GitHub Actions for automated quality checks

For more detailed information about specific tools and their parameters, see [docs/mcp-methods.md](./mcp-methods.md).