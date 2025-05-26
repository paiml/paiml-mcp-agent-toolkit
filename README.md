# PAIML MCP Agent Toolkit

> **Professional project scaffolding toolkit for Claude Code - Generate Makefiles, READMEs, and .gitignore files with AI**

[![CI](https://github.com/paiml/paiml-mcp-agent-toolkit/actions/workflows/ci.yml/badge.svg)](https://github.com/paiml/paiml-mcp-agent-toolkit/actions/workflows/ci.yml)
[![Code Quality](https://github.com/paiml/paiml-mcp-agent-toolkit/actions/workflows/code-quality.yml/badge.svg)](https://github.com/paiml/paiml-mcp-agent-toolkit/actions/workflows/code-quality.yml)
[![PR Checks](https://github.com/paiml/paiml-mcp-agent-toolkit/actions/workflows/pr-checks.yml/badge.svg)](https://github.com/paiml/paiml-mcp-agent-toolkit/actions/workflows/pr-checks.yml)
[![Dependencies](https://github.com/paiml/paiml-mcp-agent-toolkit/actions/workflows/dependencies.yml/badge.svg)](https://github.com/paiml/paiml-mcp-agent-toolkit/actions/workflows/dependencies.yml)
[![Coverage](https://img.shields.io/badge/coverage-78%25-green)](https://github.com/paiml/paiml-mcp-agent-toolkit/actions)
[![Built by Pragmatic AI Labs](https://img.shields.io/badge/Built%20by-Pragmatic%20AI%20Labs-blue)](https://paiml.com)
[![MCP Compatible](https://img.shields.io/badge/MCP-Compatible-green)](https://modelcontextprotocol.io)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

The PAIML MCP Agent Toolkit is a stateless Model Context Protocol (MCP) server created by [Pragmatic AI Labs](https://paiml.com) that provides intelligent project scaffolding through Claude Code and other MCP-compatible clients. It generates production-ready Makefiles, README files, and .gitignore configurations optimized for Rust, Deno, and Python development.

![PAIML MCP Agent Toolkit Demo](assets/demo.gif)

## 🚀 Quick Start

### For Claude Code Users

1. **Install the MCP server**:
```bash
# Option A: Use pre-built binary (recommended)
curl -L https://github.com/paiml/paiml-mcp-agent-toolkit/releases/latest/download/paiml-mcp-agent-toolkit-$(uname -s | tr '[:upper:]' '[:lower:]')-amd64.tar.gz -o paiml-mcp-agent-toolkit.tar.gz
tar xzf paiml-mcp-agent-toolkit.tar.gz
chmod +x paiml-mcp-agent-toolkit
sudo mv paiml-mcp-agent-toolkit /usr/local/bin/

# Option B: Build from source
git clone https://github.com/paiml/paiml-mcp-agent-toolkit.git
cd paiml-mcp-agent-toolkit
make install  # Automatically bumps version, builds, and installs

# Add to Claude Code (with a name)
claude mcp add paiml-toolkit /usr/local/bin/paiml-mcp-agent-toolkit
# Or if installed to ~/.local/bin:
claude mcp add paiml-toolkit ~/.local/bin/paiml-mcp-agent-toolkit
```

2. **Verify installation**:
```bash
# Check MCP status
claude mcp status

# Or use the /mcp command in Claude Code
/mcp
```

3. **Ask Claude to generate project files**:
- "Generate a Makefile for my Rust project"
- "Create a professional README for this TypeScript library"
- "Set up a .gitignore for Python development"
- "Validate my template parameters"
- "Search for Docker-related templates"
- "Scaffold a complete Rust project"
- "What does the paiml-mcp-agent-toolkit do?"
- "Who made this MCP server?"

### For Developers

```bash
# Using pre-built binary
curl -L https://github.com/paiml/paiml-mcp-agent-toolkit/releases/latest/download/paiml-mcp-agent-toolkit-linux-x64 -o paiml-mcp-agent-toolkit
chmod +x paiml-mcp-agent-toolkit
./paiml-mcp-agent-toolkit

# Building from source
cd server
make build
./target/release/paiml-mcp-agent-toolkit
```

## 📋 Table of Contents

- [Overview](#overview)
- [Key Features](#key-features)
- [Installation](#installation)
- [Usage Guide](#usage-guide)
- [Architecture](#architecture)
- [Available Templates](#available-templates)
- [Development](#development)
- [API Reference](#api-reference)
- [Performance](#performance)
- [Troubleshooting](#troubleshooting)
- [Contributing](#contributing)

## Overview

The PAIML MCP Agent Toolkit implements a production-grade template server using a stateless Rust architecture with embedded templates. All templates are compiled directly into the binary, requiring no external dependencies or cloud storage.

### Key Features

- 🏃 **Zero Dependencies**: Single binary with embedded templates
- ⚡ **Instant Generation**: Sub-5ms template rendering
- 🔧 **Three Toolchains**: Rust CLI, Deno/TypeScript, Python UV
- 📦 **MCP Native**: Full Model Context Protocol compliance
- 🔍 **Smart Search**: Find templates by keywords and content
- 🎯 **Type Safe**: Comprehensive parameter validation
- 🚀 **Batch Operations**: Scaffold entire projects at once
- 📋 **Interactive Prompts**: Guided project setup workflows
- 📁 **Smart Directory Creation**: Files are created in project subdirectories
- ℹ️ **Discoverable**: Built-in server info tool for metadata access

### Supported Toolchains

1. **Rust CLI** (cargo + clippy + rustfmt)
   - Binary applications
   - Library crates
   - Embedded systems

2. **Deno/TypeScript** (native runtime)
   - CLI applications
   - Web services
   - TypeScript libraries

3. **Python UV** (Rust-based package management)
   - CLI applications
   - Library packages
   - Data science projects

## Installation

### Method 1: Pre-built Binaries (Recommended)

Download binaries for your platform:

- [Linux x64](https://github.com/paiml/paiml-mcp-agent-toolkit/releases/latest/download/paiml-mcp-agent-toolkit-linux-x64)
- [macOS ARM64](https://github.com/paiml/paiml-mcp-agent-toolkit/releases/latest/download/paiml-mcp-agent-toolkit-macos-arm64)
- [macOS x64](https://github.com/paiml/paiml-mcp-agent-toolkit/releases/latest/download/paiml-mcp-agent-toolkit-macos-x64)

```bash
# Download and install
curl -L https://github.com/paiml/paiml-mcp-agent-toolkit/releases/latest/download/paiml-mcp-agent-toolkit-$(uname -s)-$(uname -m) -o paiml-mcp-agent-toolkit
chmod +x paiml-mcp-agent-toolkit
sudo mv paiml-mcp-agent-toolkit /usr/local/bin/
```

### Method 2: Build from Source

```bash
# Clone the repository
git clone https://github.com/paiml/paiml-mcp-agent-toolkit.git
cd paiml-mcp-agent-toolkit

# Install (automatically bumps version, builds, and installs)
make install

# Or build without installing
make build
```

### Claude Code Integration

Add to your Claude Code configuration (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

```json
{
  "mcpServers": {
    "paiml-toolkit": {
      "command": "/usr/local/bin/paiml-mcp-agent-toolkit",
      "args": [],
      "env": {}
    }
  }
}
```

## Usage Guide

### For Claude Code Users

The PAIML MCP Agent Toolkit integrates seamlessly with Claude Code. Simply ask Claude to generate project files using natural language:

#### Examples:

**Generate a Makefile:**
```
"Create a Makefile for my Rust CLI project"
"I need a Makefile for a Deno web service"
"Generate a Python UV Makefile with testing and linting"
```

**Create a README:**
```
"Generate a professional README for my Rust library"
"Create a README for my TypeScript CLI tool"
"I need documentation for my Python package"
```

**Setup .gitignore:**
```
"Create a .gitignore for Rust development"
"Generate a gitignore for my Deno project"
"Setup Python gitignore with UV and pytest"
```

### For Developers

#### List Available Templates

```bash
# Using the tool directly
echo '{"jsonrpc":"2.0","id":1,"method":"resources/list"}' | paiml-mcp-agent-toolkit

# Filter by category
echo '{"jsonrpc":"2.0","id":1,"method":"resources/list","params":{"category":"makefile"}}' | paiml-mcp-agent-toolkit
```

#### Generate a Template

```bash
# Generate a Rust CLI Makefile
echo '{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "generate_template",
    "arguments": {
      "resource_uri": "template://makefile/rust/cli-binary",
      "parameters": {
        "project_name": "my-awesome-cli",
        "has_tests": true,
        "has_benchmarks": false
      }
    }
  }
}' | paiml-mcp-agent-toolkit
```

### Template Parameters

Each template accepts specific parameters:

#### Makefile Templates
- `project_name` (required): Your project's name
- `has_tests`: Whether to include test targets
- `has_benchmarks`: Include benchmark targets
- `target_triple`: Build target architecture

#### README Templates
- `project_name` (required): Project name
- `description`: Brief project description
- `author`: Your name or organization
- `license`: License type (MIT, Apache-2.0, etc.)

#### Gitignore Templates
- `project_name` (required): Project name
- `deployment_target`: Target environment
- `include_ide`: Include IDE-specific patterns
- `include_os`: Include OS-specific patterns

## Architecture

### System Overview

```
┌─────────────────┐     JSON-RPC 2.0      ┌──────────────────┐
│  Claude Code    │◄─────────────────────►│  MCP Server      │
│  (MCP Client)   │        STDIO           │  (Rust Binary)   │
└─────────────────┘                        └──────────────────┘
                                                    │
                                          ┌─────────┴──────────┐
                                          │ Embedded Templates │
                                          └────────────────────┘
```

### Technical Architecture

The PAIML MCP Agent Toolkit uses a stateless architecture with several key components:

1. **MCP Protocol Handler**: Implements JSON-RPC 2.0 over STDIO
2. **Template Engine**: Handlebars-based rendering with custom helpers
3. **Resource Manager**: URI-based template resolution
4. **Cache Layer**: LRU cache for template content

### Template URI Schema

Templates follow a hierarchical URI structure:
```
template://[category]/[toolchain]/[variant]
```

Examples:
- `template://makefile/rust/cli-binary`
- `template://readme/deno/web-service`
- `template://gitignore/python-uv/library-package`

## Available Templates

### Makefile Templates

All Makefiles implement a standardized interface with these targets:

```makefile
all      # Complete build pipeline
format   # Code formatting
lint     # Static analysis
check    # Type checking
test     # Run tests with coverage
build    # Create optimized artifacts
install  # System installation
clean    # Remove artifacts
validate # Project validation checklist
help     # Show all targets
```

#### Rust Templates
- `template://makefile/rust/cli-binary` - CLI applications
- `template://makefile/rust/library-crate` - Rust libraries

#### Deno Templates
- `template://makefile/deno/cli-application` - CLI tools
- `template://makefile/deno/web-service` - Web services

#### Python UV Templates
- `template://makefile/python-uv/cli-application` - CLI tools
- `template://makefile/python-uv/library-package` - Python packages

### README Templates
- `template://readme/rust/cli-application`
- `template://readme/deno/typescript-library`
- `template://readme/python-uv/data-science`

### Gitignore Templates
- `template://gitignore/rust/embedded-target`
- `template://gitignore/deno/web-deployment`
- `template://gitignore/python-uv/data-science`

## Development

### CI/CD Pipeline

This project uses GitHub Actions for continuous integration and deployment:

- **Continuous Integration**: Runs on every push and pull request
  - Linting with rustfmt and clippy
  - Testing with 77% code coverage
  - Multi-platform builds (Linux, macOS, Windows)
  - Security audits
  - E2E testing

- **Release Process**: Automated binary releases
  - Triggered by version tags (e.g., `v1.0.0`)
  - Builds for all platforms
  - Creates GitHub releases with attached binaries

- **Code Quality**: Enforced standards
  - Minimum 70% test coverage
  - No clippy warnings
  - Proper formatting
  - Documentation checks

### Testing

The project maintains comprehensive test coverage:

```bash
# Run all tests
make test

# Run tests with coverage report
cargo llvm-cov --all-features --workspace --html --output-dir coverage

# Run specific test modules
cargo test --test mcp_protocol
cargo test --test template_rendering
```

Test categories:
- **Unit Tests**: Core functionality (78% coverage)
- **Integration Tests**: MCP protocol handling
- **E2E Tests**: Full server functionality
- **Template Tests**: All template rendering paths

### Project Structure

```
paiml-mcp-agent-toolkit/
├── server/                 # Rust server implementation
│   ├── src/
│   │   ├── bin/           # Binary entry points
│   │   ├── handlers/      # MCP protocol handlers
│   │   ├── models/        # Data models
│   │   ├── services/      # Core services
│   │   └── main.rs        # Main server
│   ├── templates/         # Embedded templates
│   └── Cargo.toml
├── client/                # Future client implementation
└── scripts/              # Build and deployment scripts
```

### Building and Installing

```bash
# From root directory
make install     # Bumps version, builds, and installs (recommended)
make build       # Just builds without installing
make test        # Run all tests
make validate    # Run all validation checks

# Or from server directory
cd server
make build-binary   # Build binary only
make install        # Install with version bump
make test           # Run tests
make test-mcp       # Run E2E MCP tests
```

### Adding New Templates

1. Create template file in `server/templates/`
2. Update `embedded_templates.rs` to include it
3. Add metadata to the template registry
4. Write tests for the new template

### Development Commands

```bash
# Run MCP server locally
make run-mcp

# Run with test templates
make run-mcp-test

# Check code quality
make lint

# Format code
make format

# Run benchmarks
make bench
```

## API Reference

### MCP Methods

#### `initialize`

Initialize the MCP connection and get server capabilities.

**Response includes:**
- Server metadata with name, version, and description
- Supported templates and toolchains
- Available capabilities (tools, resources, prompts)

#### `resources/list`

List available templates with optional filtering.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "resources/list"
}
```

#### `resources/read`

Read the raw template content before rendering.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "resources/read",
  "params": {
    "uri": "template://makefile/rust/cli-binary"
  }
}
```

#### `prompts/list`

Get available interactive prompts for project scaffolding.

**Response includes:**
- scaffold-rust-project
- scaffold-deno-project
- scaffold-python-project

#### `tools/list`

List all available tools.

**Available tools:**
- `generate_template` - Generate a single template
- `list_templates` - List templates with filtering
- `validate_template` - Validate template parameters
- `scaffold_project` - Generate multiple templates at once
- `search_templates` - Search templates by keyword
- `get_server_info` - Get server metadata and capabilities

### Available Tools

#### `generate_template`

Generate a template with parameters.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "generate_template",
    "arguments": {
      "resource_uri": "template://makefile/rust/cli-binary",
      "parameters": {
        "project_name": "my-project",
        "has_tests": true
      }
    }
  }
}
```

#### `validate_template`

Validate template parameters before generation.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "validate_template",
    "arguments": {
      "resource_uri": "template://makefile/rust/cli-binary",
      "parameters": {
        "project_name": "my-project"
      }
    }
  }
}
```

#### `scaffold_project`

Generate multiple templates for a complete project setup.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "scaffold_project",
    "arguments": {
      "toolchain": "rust",
      "templates": ["makefile", "readme", "gitignore"],
      "parameters": {
        "project_name": "my-project",
        "has_tests": true
      }
    }
  }
}
```

#### `search_templates`

Search templates by keyword in names, descriptions, and parameters.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "search_templates",
    "arguments": {
      "query": "docker",
      "toolchain": "rust"
    }
  }
}
```

## Performance

### Server Performance Metrics

| Metric | Target | Actual |
|--------|--------|--------|
| Startup Time | <10ms | 7ms |
| Template Generation | <5ms | 3ms |
| Memory Usage | <20MB | 15MB |
| Concurrent Requests | 1000+ | 1200 |

### Client Performance

| Operation | Target | Strategy |
|-----------|--------|----------|
| Project Analysis | <500ms | Parallel file scanning |
| MCP Transport | <50ms RTT | Connection pooling |
| Template Generation | <200ms | Predictive caching |

## Troubleshooting

### Common Issues

#### MCP Server Not Starting
```bash
# Check if the binary is executable
ls -la /usr/local/bin/paiml-mcp-agent-toolkit

# Test the server directly
echo '{"jsonrpc":"2.0","id":1,"method":"resources/list"}' | paiml-mcp-agent-toolkit

# Check the installed version
grep '^version' /path/to/paiml-mcp-agent-toolkit/server/Cargo.toml
```

#### Claude Code Integration Issues

If you see "failed" status in `claude mcp status`:

**For existing installations with old paths:**
If you previously had the MCP server registered with a different path, you'll need to update it:

```bash
# Remove the old server (if it exists)
claude mcp remove paiml-mcp-agent-toolkit 2>/dev/null || true

# Add the new server with correct path
claude mcp add paiml-toolkit ~/.local/bin/paiml-mcp-agent-toolkit

# Restart Claude Code to pick up the changes
```

**Important**: If you have multiple Claude instances running, close ALL of them before restarting to ensure they all pick up the new binary version.

**For new installations:**

1. **Make sure the binary is executable:**
   ```bash
   chmod +x /usr/local/bin/paiml-mcp-agent-toolkit
   # Or for local install:
   chmod +x ~/.local/bin/paiml-mcp-agent-toolkit
   ```

2. **Ensure the binary is in your PATH:**
   ```bash
   which paiml-mcp-agent-toolkit
   ```

3. **Check Claude Code logs:**
   ```bash
   tail -f ~/Library/Logs/Claude/mcp.log
   ```

4. **Run with debug mode to see errors:**
   ```bash
   claude --mcp-debug
   ```

5. **Verify MCP is working:**
   ```bash
   # Use the /mcp command in Claude Code
   /mcp
   
   # During installation, check the version displayed:
   # 📌 Version: 0.1.x
   ```

#### Template Generation Errors
- Ensure all required parameters are provided
- Check parameter types match expected values
- Verify the template URI is correct

### Debug Mode

Run the server with debug logging:
```bash
RUST_LOG=debug paiml-mcp-agent-toolkit
```

## Demo Commands

Showcase PAIML branding and capabilities:

```bash
# Display server info with branding
echo '{"jsonrpc":"2.0","id":1,"method":"server/info"}' | paiml-mcp-agent-toolkit

# Create a demo project
mkdir demo-project && cd demo-project
echo "name = \"demo-project\"" > Cargo.toml

# Generate branded Makefile
echo '{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "generate_template",
    "arguments": {
      "resource_uri": "template://makefile/rust/cli-binary",
      "parameters": {
        "project_name": "demo-project"
      }
    }
  }
}' | paiml-mcp-agent-toolkit > Makefile
```

## What's New

### Recent Improvements
- ✅ **All 9 Templates Available**: Fixed template embedding to include all Deno and Python-uv templates
- 🚀 **Smart Installation**: Automatic rebuild detection based on source file changes
- 📁 **Proper Subdirectories**: Templates now create files in project-named subdirectories
- ℹ️ **Enhanced Discoverability**: New `get_server_info` tool provides metadata about the server
- 🧪 **E2E Testing**: Comprehensive end-to-end tests simulating Claude Code operations
- 📊 **Improved Coverage**: Test coverage increased from 77% to 78%
- 🔧 **Consolidated Tooling**: Unified installation scripts and centralized Makefile commands
- 🔢 **Auto-Versioning**: Installation automatically increments version for easy tracking

## Contributing

We welcome contributions!

### Development Setup

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `make test`
5. Submit a pull request

### Code Style

- Follow Rust standard formatting (`rustfmt`)
- Write tests for new features
- Update documentation as needed
- Include PAIML attribution in generated files

## License

This project is licensed under the MIT License.

## Support

- **Issues**: [GitHub Issues](https://github.com/paiml/paiml-mcp-agent-toolkit/issues)
- **Discussions**: [GitHub Discussions](https://github.com/paiml/paiml-mcp-agent-toolkit/discussions)
- **Email**: contact@paiml.com
- **Website**: [paiml.com](https://paiml.com)

---

<div style="text-align: center">
  <strong>Built with ❤️ by PAIML</strong><br>
  <sub>Empowering developers with AI-powered tools</sub>
</div>