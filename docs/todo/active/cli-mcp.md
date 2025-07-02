# PAIML MCP Agent Toolkit CLI & MCP Protocol Reference

This document provides comprehensive documentation for the PAIML MCP Agent Toolkit's dual-mode binary that supports both command-line interface (CLI) usage and Model Context Protocol (MCP) integration.

## Table of Contents

- [Binary Architecture](#binary-architecture)
- [Installation](#installation)
- [Usage Modes](#usage-modes)
- [CLI Command Reference](#cli-command-reference)
- [MCP Protocol Implementation](#mcp-protocol-implementation)
- [Performance Characteristics](#performance-characteristics)
- [Caching Architecture](#caching-architecture)
- [Template System](#template-system)
- [Integration Examples](#integration-examples)
- [Troubleshooting](#troubleshooting)

## Binary Architecture

The `paiml-mcp-agent-toolkit` binary implements a dual-mode execution model:

- **CLI Mode**: Direct command execution with POSIX-compliant argument parsing
- **MCP Mode**: JSON-RPC 2.0 over STDIO for Model Context Protocol integration

Mode detection occurs at runtime via terminal detection:

```rust
fn detect_execution_mode() -> ExecutionMode {
    let is_mcp = !std::io::stdin().is_terminal() 
        && std::env::args().len() == 1
        || std::env::var("MCP_VERSION").is_ok();
    
    if is_mcp {
        ExecutionMode::Mcp
    } else {
        ExecutionMode::Cli
    }
}
```

## Installation

### Quick Install (Linux/macOS)

```bash
curl -sSfL https://raw.githubusercontent.com/paiml/paiml-mcp-agent-toolkit/master/scripts/install.sh | sh
```

### Manual Installation

Download the appropriate binary for your platform from the [releases page](https://github.com/paiml/paiml-mcp-agent-toolkit/releases) and add it to your PATH.

### Build from Source

```bash
git clone https://github.com/paiml/paiml-mcp-agent-toolkit.git
cd paiml-mcp-agent-toolkit
make install
```

## Usage Modes

### CLI Mode

Direct command execution from terminal:

```bash
pmat generate makefile rust/cli -p project_name=my-project
```

### MCP Mode

Automatic activation when used as MCP server (e.g., with Claude Code):

```bash
# Add to Claude Code
claude mcp add paiml-toolkit ~/.local/bin/paiml-mcp-agent-toolkit
```

### Force Mode

Override auto-detection:

```bash
# Force CLI mode
pmat --mode cli list

# Force MCP mode (waits for JSON-RPC input)
pmat --mode mcp
```

## CLI Command Reference

### Command: `generate`

Generate a single template with parameter substitution.

#### Synopsis

```bash
pmat generate <CATEGORY> <PATH> [OPTIONS]
```

#### Arguments

- `<CATEGORY>` - Template category: `makefile`, `readme`, `gitignore`
- `<PATH>` - Toolchain path: `rust/cli`, `deno/cli`, `python-uv/cli`

#### Options

- `-p, --param <KEY=VALUE>` - Template parameters (repeatable)
- `-o, --output <PATH>` - Output file path (default: stdout)
- `--create-dirs` - Create parent directories if needed

#### Parameter Type Inference

The CLI implements automatic type inference for parameters:

- `"true"` / `"false"` → Boolean
- Valid integers → Number
- Valid floats → Number
- Everything else → String
- Empty value → `true` (for flags)

#### Examples

```bash
# Basic generation with typed parameters
paiml-mcp-agent-toolkit generate makefile rust/cli \
  -p project_name=servo \
  -p has_tests=true \
  -p max_jobs=8

# Output to specific path with directory creation
paiml-mcp-agent-toolkit generate readme deno/cli \
  -p project_name=fresh \
  -o docs/README.md \
  --create-dirs
```

#### MCP Equivalent

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "generate_template",
    "arguments": {
      "resource_uri": "template://makefile/rust/cli",
      "parameters": {
        "project_name": "servo",
        "has_tests": true,
        "max_jobs": 8
      }
    }
  }
}
```

### Command: `scaffold`

Generate multiple templates at once for a complete project setup.

#### Synopsis

```bash
pmat scaffold <TOOLCHAIN> [OPTIONS]
```

#### Arguments

- `<TOOLCHAIN>` - Target toolchain: `rust`, `deno`, `python-uv`

#### Options

- `-t, --templates <TEMPLATES>` - Comma-separated list of templates
- `-p, --param <KEY=VALUE>` - Template parameters (repeatable)
- `--parallel <N>` - Number of parallel file writes (default: CPU count)

#### Examples

```bash
# Scaffold a complete Rust project
paiml-mcp-agent-toolkit scaffold rust \
  --templates makefile,readme,gitignore \
  -p project_name=my-project \
  -p author="John Doe"

# Scaffold with custom parallelism
paiml-mcp-agent-toolkit scaffold deno \
  --templates makefile,readme \
  -p project_name=my-app \
  --parallel 4
```

### Command: `list`

Display all available templates with filtering options.

#### Synopsis

```bash
pmat list [OPTIONS]
```

#### Options

- `--toolchain <TOOLCHAIN>` - Filter by toolchain
- `--category <CATEGORY>` - Filter by category
- `--format <FORMAT>` - Output format: `table`, `json`, `yaml` (default: table)

#### Examples

```bash
# List all templates
paiml-mcp-agent-toolkit list

# Filter by toolchain with JSON output
paiml-mcp-agent-toolkit list --toolchain rust --format json
```

### Command: `search`

Find templates by searching in names, descriptions, and parameters.

#### Synopsis

```bash
pmat search <QUERY> [OPTIONS]
```

#### Arguments

- `<QUERY>` - Search query string

#### Options

- `--toolchain <TOOLCHAIN>` - Filter results by toolchain
- `--limit <N>` - Maximum number of results (default: 20)

### Command: `validate`

Validate template parameters before generation.

#### Synopsis

```bash
pmat validate <URI> [OPTIONS]
```

#### Arguments

- `<URI>` - Template URI (e.g., `template://makefile/rust/cli`)

#### Options

- `-p, --param <KEY=VALUE>` - Parameters to validate

### Command: `context`

Generate project context using Abstract Syntax Tree (AST) analysis.

#### Synopsis

```bash
pmat context <TOOLCHAIN> [OPTIONS]
```

#### Arguments

- `<TOOLCHAIN>` - Target toolchain: `rust`, `deno`, `python-uv`

#### Options

- `-p, --project-path <PATH>` - Project path to analyze (default: .)
- `-o, --output <PATH>` - Output file path
- `--format <FORMAT>` - Output format: `markdown`, `json` (default: markdown)

#### Features

- **Language Support**:
  - Rust: Functions, structs, enums, traits, implementations
  - TypeScript/JavaScript: Functions, classes, interfaces, types
  - Python: Functions, classes, imports

- **Performance**:
  - Persistent cache with 5-minute TTL
  - Cross-session caching in `~/.cache/paiml-mcp-agent-toolkit/`
  - Cache hit rates typically exceed 70%

### Command: `report`

Generate enhanced analysis reports combining multiple analysis types into comprehensive documentation.

#### Synopsis

```bash
pmat report [OPTIONS]
```

#### Options

- `-p, --project-path <PATH>` - Project path to analyze (default: .)
- `-f, --output-format <FORMAT>` - Output format: `html`, `markdown`, `json`, `pdf`, `dashboard` (default: markdown)
- `--include-visualizations` - Include charts and graphs in the report
- `--include-executive-summary` - Include executive summary (default: true)
- `--include-recommendations` - Include actionable recommendations (default: true)
- `--analyses <TYPES>` - Analysis types to include: `all`, `complexity`, `churn`, `dead-code`, `satd` (default: all)
- `--confidence-threshold <VALUE>` - Confidence threshold for findings 0-100 (default: 50)

#### Examples

```bash
# Generate comprehensive markdown report
pmat report

# Generate HTML report with visualizations
pmat report --output-format html --include-visualizations

# Generate focused complexity report
pmat report --analyses complexity --confidence-threshold 80
```

### Command: `analyze complexity`

Performs static complexity analysis using dual algorithms:
- **McCabe Cyclomatic Complexity**: M = E - N + 2P (graph-theoretic)
- **Sonar Cognitive Complexity**: Context-aware nesting analysis

#### Synopsis

```bash
pmat analyze complexity [OPTIONS]
```

#### Options

- `-p, --project-path <PATH>` - Project path to analyze (default: .)
- `--toolchain <TOOLCHAIN>` - Force specific toolchain (auto-detected by default)
- `--format <FORMAT>` - Output format: `summary`, `full`, `json`, `sarif`
- `-o, --output <PATH>` - Output file path
- `--max-cyclomatic <N>` - Custom cyclomatic complexity threshold
- `--max-cognitive <N>` - Custom cognitive complexity threshold
- `--include <PATTERN>` - Include file patterns (e.g., "**/*.rs")

#### Performance Profile

- **Parsing**: Reuses AST cache (5-minute TTL)
- **Analysis**: <1ms per KLOC
- **Memory**: O(n) where n = AST nodes
- **Cache Key**: SHA-256(file_path + mtime + size)

### Command: `analyze churn`

Analyze code change frequency and patterns to identify maintenance hotspots.

#### Synopsis

```bash
pmat analyze churn [OPTIONS]
```

#### Options

- `-p, --project-path <PATH>` - Project path to analyze (default: .)
- `-d, --days <N>` - Number of days to analyze (default: 30)
- `--format <FORMAT>` - Output format: `summary`, `markdown`, `json`, `csv`
- `-o, --output <PATH>` - Output file path

### Command: `analyze dag`

Generate dependency graphs in Mermaid format for visualizing code structure.

#### Synopsis

```bash
pmat analyze dag [OPTIONS]
```

#### Options

- `--dag-type <TYPE>` - Type of graph to generate:
  - `call-graph` - Function call relationships (default)
  - `import-graph` - Module import dependencies
  - `inheritance` - Class inheritance hierarchies
  - `full-dependency` - Complete dependency analysis
- `-p, --project-path <PATH>` - Project path to analyze (default: .)
- `-o, --output <PATH>` - Output file path
- `--max-depth <N>` - Maximum depth for graph traversal
- `--filter-external` - Filter out external dependencies
- `--show-complexity` - Include complexity metrics in the graph

### Command: `demo`

Run an interactive demonstration of all PAIML MCP Agent Toolkit capabilities.

#### Synopsis

```bash
pmat demo [OPTIONS]
```

#### Options

- `-p, --path <PATH>` - Repository path to analyze (defaults to current directory)
- `-f, --format <FORMAT>` - Output format: `table`, `json`, `yaml` (default: table)

#### Description

The demo command provides a comprehensive showcase of all toolkit capabilities in a single execution:

1. **AST Context Analysis** - Analyzes project structure and generates context
2. **Code Complexity Analysis** - Measures cyclomatic and cognitive complexity
3. **DAG Visualization** - Generates dependency graphs
4. **Code Churn Analysis** - Analyzes git history for change patterns
5. **Template Generation** - Demonstrates project scaffolding

Each capability is executed with timing information, demonstrating real-world performance. The command automatically detects git repositories and provides execution metrics for each step.

#### Examples

```bash
# Run demo in current directory
paiml-mcp-agent-toolkit demo

# Run demo on specific repository
paiml-mcp-agent-toolkit demo --path /path/to/repo

# Get JSON output for integration
paiml-mcp-agent-toolkit demo --format json
```

### Command: `diagnose`

Run self-diagnostics to verify all features are working correctly.

#### Synopsis

```bash
pmat diagnose [OPTIONS]
```

#### Options

- `--format <FORMAT>` - Output format for diagnostic report: `pretty`, `json`, `compact` (default: pretty)
- `--only <ONLY>` - Only run specific feature tests (can be repeated)
- `--skip <SKIP>` - Skip specific feature tests (can be repeated)
- `--timeout <TIMEOUT>` - Maximum time to run diagnostics in seconds (default: 60)
- `-v, --verbose` - Enable verbose output (info level)
- `--debug` - Enable debug output (debug level)
- `--trace` - Enable trace output (trace level)
- `--trace-filter <TRACE_FILTER>` - Custom trace filter (overrides other flags)

#### Description

The diagnose command performs comprehensive self-diagnostics to verify that all toolkit features are functioning correctly. This is useful for:

- Troubleshooting installation issues
- Verifying feature availability in different environments
- Generating diagnostic reports for support
- Automated health checks in CI/CD pipelines

The diagnostics include tests for:
- Template rendering capabilities
- AST parsing for all supported languages
- Cache system functionality
- File system access
- Git integration
- Performance benchmarks

#### Examples

```bash
# Run full diagnostics with pretty output
pmat diagnose

# Generate JSON diagnostic report
pmat diagnose --format json

# Run only specific feature tests  
pmat diagnose --only template-rendering --only ast-parsing

# Skip slow tests with timeout
pmat diagnose --skip performance --timeout 30

# Verbose diagnostic output
pmat diagnose --verbose
```

### Command: `enforce`

Enforce extreme quality standards using automated code quality enforcement.

#### Synopsis

```bash
pmat enforce <SUBCOMMAND> [OPTIONS]
```

#### Description

The enforce command provides automated enforcement of extreme quality standards through state machine-driven code improvements. It can automatically apply fixes for code quality issues including complexity reduction, SATD elimination, and linting violations.

#### Subcommands

- `extreme` - Enforce extreme quality standards with zero tolerance

### Command: `enforce extreme`

Enforce extreme quality standards with zero tolerance for code quality issues.

#### Synopsis

```bash
pmat enforce extreme [OPTIONS]
```

#### Options

- `-p, --project-path <PATH>` - Project path to enforce quality on (default: .)
- `--single-file-mode` - Enforce on one file at a time
- `--dry-run` - Show what would be changed without making changes
- `--profile <PROFILE>` - Quality profile to use: `extreme` (default: extreme)
- `--show-progress` - Show progress during enforcement (default: true)
- `-f, --format <FORMAT>` - Output format: `summary`, `detailed`, `json` (default: summary)
- `-o, --output <PATH>` - Output file path
- `--max-iterations <N>` - Maximum iterations before giving up (default: 100)
- `--target-improvement <PERCENT>` - Target improvement percentage
- `--max-time <SECONDS>` - Maximum time in seconds
- `--auto-apply` - Apply suggestions automatically

#### Description

The enforce extreme command applies the highest quality standards to your codebase:

- **Zero SATD Policy**: Eliminates all TODO, FIXME, HACK, and XXX comments
- **Complexity Reduction**: Refactors functions exceeding cyclomatic complexity of 20
- **Lint Enforcement**: Applies pedantic and nursery clippy lints
- **Dead Code Removal**: Identifies and removes unused code
- **Documentation**: Ensures all public items are documented

The enforcement process uses a state machine to iteratively improve code quality until all standards are met or limits are reached.

#### Examples

```bash
# Enforce extreme quality standards on current directory
pmat enforce extreme

# Dry run to see what would be changed
pmat enforce extreme --dry-run

# Enforce with automatic fixes applied
pmat enforce extreme --auto-apply

# Enforce on a specific project with progress
pmat enforce extreme -p ./my-project --show-progress

# Enforce with time limit and target improvement
pmat enforce extreme --max-time 300 --target-improvement 50

# Generate detailed JSON report
pmat enforce extreme -f json -o quality-report.json
```

### Command: `serve`

Start the HTTP REST API server for programmatic access to all toolkit capabilities.

#### Synopsis

```bash
pmat serve [OPTIONS]
```

#### Options

- `--host <HOST>` - Host address to bind to (default: 127.0.0.1)
- `--port <PORT>` - Port to bind the HTTP server to (default: 8080)
- `--cors` - Enable CORS for cross-origin requests

#### Description

The serve command starts an HTTP REST API server that provides programmatic access to all toolkit capabilities through standard HTTP endpoints. This enables integration with web applications, CI/CD pipelines, and other tools that can make HTTP requests.

The server provides endpoints for:
- Template generation and listing
- Code analysis (complexity, churn, DAG, dead code, SATD)
- Deep context analysis
- Health checks and metrics

All endpoints support both JSON request bodies and query parameters where appropriate.

#### Examples

```bash
# Start server on default settings (localhost:8080)
paiml-mcp-agent-toolkit serve

# Start server with custom host and port
paiml-mcp-agent-toolkit serve --host 0.0.0.0 --port 3000

# Start server with CORS enabled for web apps
paiml-mcp-agent-toolkit serve --port 8080 --cors

# Use the REST API
curl "http://localhost:8080/health"
curl "http://localhost:8080/api/v1/analyze/complexity?top_files=5"
```

### Command: `refactor`

Automated refactoring with real-time analysis and interactive mode support.

#### Synopsis

```bash
pmat refactor <SUBCOMMAND> [OPTIONS]
```

#### Description

The refactor command provides automated refactoring capabilities to reduce code complexity and improve maintainability. It supports both batch processing and interactive modes, with checkpoint support for resuming operations.

#### Subcommands

- `serve` - Run refactor server mode for batch processing
- `interactive` - Run interactive refactoring mode
- `status` - Show current refactoring status
- `resume` - Resume refactoring from checkpoint

#### Examples

```bash
# Run batch refactoring with configuration
pmat refactor serve --config refactor-config.json

# Start interactive refactoring session
pmat refactor interactive

# Check refactoring status
pmat refactor status

# Resume from checkpoint
pmat refactor resume
```

### Command: `refactor serve`

Run refactor server mode for batch processing of refactoring operations.

#### Synopsis

```bash
pmat refactor serve [OPTIONS]
```

#### Options

- `--refactor-mode <MODE>` - Refactor mode: `batch`, `interactive` (default: batch)
- `-c, --config <PATH>` - JSON configuration file for batch mode
- `-p, --project <PATH>` - Project directory to refactor (default: .)
- `--parallel <N>` - Number of parallel workers (default: 4)
- `--memory-limit <MB>` - Memory limit in MB (default: 512)
- `--batch-size <N>` - Files per batch (default: 10)
- `--priority <EXPR>` - Priority sorting expression (e.g., "complexity * defect_probability")
- `--checkpoint-dir <PATH>` - Checkpoint directory for resuming
- `--resume` - Resume from previous checkpoint
- `--auto-commit <MSG>` - Auto-commit with message template
- `--max-runtime <SECS>` - Maximum runtime in seconds

#### Examples

```bash
# Run batch refactoring with configuration file
pmat refactor serve --config refactor-config.json

# Run with custom parallelism and memory limits
pmat refactor serve --parallel 8 --memory-limit 1024 --batch-size 20

# Resume from checkpoint with auto-commit
pmat refactor serve --resume --checkpoint-dir ./checkpoints --auto-commit "refactor: reduce complexity in {file}"
```

### Command: `refactor interactive`

Run interactive refactoring mode with real-time feedback and explanations.

#### Synopsis

```bash
pmat refactor interactive [OPTIONS]
```

#### Options

- `-p, --project-path <PATH>` - Project path to analyze (default: .)
- `--explain <LEVEL>` - Explanation level: `minimal`, `normal`, `detailed` (default: detailed)
- `--checkpoint <PATH>` - Checkpoint file for state persistence (default: refactor_state.json)
- `--target-complexity <N>` - Target complexity threshold (default: 20)
- `--steps <N>` - Maximum steps to execute
- `--config <PATH>` - Configuration file path

#### Examples

```bash
# Start interactive refactoring session
pmat refactor interactive

# Interactive mode with custom target complexity
pmat refactor interactive --target-complexity 15 --explain minimal

# Limited steps with checkpoint
pmat refactor interactive --steps 5 --checkpoint my-refactor.json
```

### Command: `refactor status`

Show current refactoring status from checkpoint file.

#### Synopsis

```bash
pmat refactor status [OPTIONS]
```

#### Options

- `--checkpoint <PATH>` - Checkpoint file to read state from (default: refactor_state.json)
- `--format <FORMAT>` - Output format: `json`, `yaml`, `summary` (default: json)

#### Examples

```bash
# Show refactoring status
pmat refactor status

# Show status from specific checkpoint
pmat refactor status --checkpoint ./checkpoints/refactor-2024-01-01.json --format summary
```

### Command: `refactor resume`

Resume refactoring from a checkpoint file.

#### Synopsis

```bash
pmat refactor resume [OPTIONS]
```

#### Options

- `--checkpoint <PATH>` - Checkpoint file to resume from (default: refactor_state.json)
- `--steps <N>` - Maximum steps to execute (default: 10)
- `--explain <LEVEL>` - Override explanation level: `minimal`, `normal`, `detailed`

#### Examples

```bash
# Resume refactoring with default settings
pmat refactor resume

# Resume with limited steps
pmat refactor resume --steps 5 --explain minimal

# Resume from specific checkpoint
pmat refactor resume --checkpoint ./backups/refactor-backup.json
```

## MCP Protocol Implementation

### Transport Layer

- Protocol: JSON-RPC 2.0
- Transport: STDIO (stdin/stdout)
- Encoding: UTF-8
- Message Framing: Newline-delimited

### Message Flow

```
Client                    Server
|                         |
|-- initialize -->        |
|                         |
|<-- capabilities --      |
|                         |
|-- tools/call -->        |
|                         |
|<-- result/error --      |
```

### Available MCP Methods

- `initialize` - Initialize connection and get capabilities
- `tools/list` - List available tools
- `tools/call` - Execute a tool
- `resources/list` - List available templates
- `resources/read` - Read template metadata
- `prompts/list` - List available prompts

### Error Codes

| Code    | Constant            | Description                      |
|---------|---------------------|----------------------------------|
| -32700  | PARSE_ERROR         | Invalid JSON                     |
| -32600  | INVALID_REQUEST     | Invalid method                   |
| -32601  | METHOD_NOT_FOUND    | Unknown method                   |
| -32602  | INVALID_PARAMS      | Invalid parameters               |
| -32001  | TEMPLATE_NOT_FOUND  | Template URI not found           |
| -32002  | VALIDATION_ERROR    | Parameter validation failed      |
| -32003  | RENDER_ERROR        | Template rendering failed        |

### Available MCP Tools

The MCP server exposes the following tools via the `tools/call` method:

#### `generate_template`
Generate templates with parameter substitution for project files.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "generate_template",
    "arguments": {
      "resource_uri": "template://makefile/rust/cli",
      "parameters": {
        "project_name": "my-project",
        "has_tests": true
      }
    }
  }
}
```

#### `analyze_complexity`
Analyze code complexity using McCabe Cyclomatic and Sonar Cognitive algorithms.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "analyze_complexity",
    "arguments": {
      "project_path": "/path/to/project",
      "toolchain": "rust",
      "format": "summary"
    }
  }
}
```

#### `analyze_code_churn`
Analyze git history for code churn patterns and maintenance hotspots.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "analyze_code_churn",
    "arguments": {
      "project_path": "/path/to/project",
      "period_days": 30,
      "format": "summary"
    }
  }
}
```

#### `analyze_dag`
Generate dependency graphs in Mermaid format for code visualization.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "analyze_dag",
    "arguments": {
      "project_path": "/path/to/project",
      "dag_type": "call-graph",
      "show_complexity": true
    }
  }
}
```

#### `generate_context`
Generate project context using AST analysis with persistent caching.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "generate_context",
    "arguments": {
      "toolchain": "rust",
      "project_path": "/path/to/project",
      "format": "markdown"
    }
  }
}
```

#### `get_server_info`
Get information about the PAIML MCP Agent Toolkit server.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "get_server_info",
    "arguments": {}
  }
}
```

#### `list_templates`
List available templates with optional filtering.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "list_templates",
    "arguments": {
      "toolchain": "rust",
      "category": "makefile"
    }
  }
}
```

#### `scaffold_project`
Scaffold a complete project with multiple templates.

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
        "project_name": "my-project"
      }
    }
  }
}
```

#### `search_templates`
Search for templates by query string.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "search_templates",
    "arguments": {
      "query": "rust",
      "toolchain": "rust"
    }
  }
}
```

#### `validate_template`
Validate template parameters before generation.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "validate_template",
    "arguments": {
      "resource_uri": "template://makefile/rust/cli",
      "parameters": {
        "project_name": "my-project"
      }
    }
  }
}
```

#### `analyze_dead_code`
Analyze dead code in the project to identify unused functions, classes, and modules.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "analyze_dead_code",
    "arguments": {
      "project_path": "/path/to/project",
      "format": "summary",
      "top_files": 10,
      "include_unreachable": false
    }
  }
}
```

#### `analyze_deep_context`
Perform deep context analysis combining multiple analysis types for comprehensive insights.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "analyze_deep_context",
    "arguments": {
      "project_path": "/path/to/project",
      "format": "markdown",
      "include_analyses": ["ast", "complexity", "churn"]
    }
  }
}
```

#### `analyze_duplicates_vectorized`
Analyze code duplicates using high-performance SIMD vectorization.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "analyze_duplicates_vectorized",
    "arguments": {
      "project_path": "/path/to/project",
      "min_lines": 10,
      "format": "summary"
    }
  }
}
```

#### `analyze_graph_metrics_vectorized`
Analyze graph metrics using vectorized operations for performance.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "analyze_graph_metrics_vectorized",
    "arguments": {
      "project_path": "/path/to/project",
      "metrics": ["centrality", "clustering"],
      "format": "json"
    }
  }
}
```

#### `analyze_name_similarity_vectorized`
Analyze name similarity using SIMD-accelerated string comparison.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "analyze_name_similarity_vectorized",
    "arguments": {
      "project_path": "/path/to/project",
      "threshold": 0.8,
      "format": "summary"
    }
  }
}
```

#### `analyze_symbol_table_vectorized`
Analyze symbol tables with vectorized processing for large codebases.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "analyze_symbol_table_vectorized",
    "arguments": {
      "project_path": "/path/to/project",
      "include_private": false,
      "format": "json"
    }
  }
}
```

#### `analyze_incremental_coverage_vectorized`
Analyze incremental code coverage using SIMD operations.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "analyze_incremental_coverage_vectorized",
    "arguments": {
      "project_path": "/path/to/project",
      "baseline_commit": "main",
      "format": "summary"
    }
  }
}
```

#### `analyze_big_o_vectorized`
Analyze Big O complexity using vectorized algorithm analysis.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "analyze_big_o_vectorized",
    "arguments": {
      "project_path": "/path/to/project",
      "functions": ["sort", "search"],
      "format": "summary"
    }
  }
}
```

#### `generate_enhanced_report`
Generate comprehensive enhanced analysis report combining multiple analyses.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "generate_enhanced_report",
    "arguments": {
      "project_path": "/path/to/project",
      "output_format": "html",
      "analyses": ["complexity", "coverage", "duplicates"],
      "include_visualizations": true
    }
  }
}
```

### Batch Request Support

```json
[
  {"jsonrpc": "2.0", "id": 1, "method": "tools/list"},
  {"jsonrpc": "2.0", "id": 2, "method": "resources/list"}
]
```

## Performance Characteristics

| Operation                | Latency          | Memory      | Cache Hit      |
|--------------------------|------------------|-------------|----------------|
| Template Rendering       | <3ms             | 512KB       | N/A            |
| AST Analysis (cold)      | 50-200ms/file    | 2MB/KLOC    | 0%             |
| AST Analysis (warm)      | <10ms            | 128KB       | >70%           |
| Complexity Analysis      | <1ms/KLOC        | 1MB         | Inherits AST   |
| DAG Generation           | 5-50ms           | 4MB         | Inherits AST   |
| Mode Detection           | 13ns             | 0           | N/A            |
| Startup Time             | 7-8ms            | 15MB        | N/A            |

## Caching Architecture

### Cache Hierarchy

1. **Session Cache** (in-memory LRU)
   - Capacity: 100 entries per type
   - TTL: Configurable per strategy
   - Eviction: LRU with memory pressure threshold

2. **Persistent Cache** (disk-based)
   - Location: `~/.cache/paiml-mcp-agent-toolkit/`
   - Format: MessagePack serialization
   - Compression: LZ4 for entries >4KB

### Cache Strategies

| Strategy  | TTL     | Max Size | Key Components               |
|-----------|---------|----------|------------------------------|
| AST       | 5 min   | 100      | path + mtime + size          |
| Template  | 30 min  | 50       | URI + version                |
| DAG       | 2 min   | 20       | project + type + commit      |
| Churn     | 10 min  | 10       | path + branch + HEAD         |

## Template System

### Embedded Template System

Templates are compiled into the binary using `include_str!` at build time, ensuring:
- Zero runtime dependencies
- Fast startup (<10ms)
- Single binary distribution

### Template URI Scheme

```
template://[category]/[toolchain]/[variant]
         │      │           │          │
         │      │           │          └─> Currently always "cli"
         │      │           └─> rust, deno, python-uv
         │      └─> makefile, readme, gitignore  
         └─> URI scheme identifier
```

### Available Templates

- **Makefile Templates**: Build automation for all toolchains
- **README Templates**: Professional documentation
- **Gitignore Templates**: Language-specific ignore patterns

Each template supports customizable parameters with validation.

## Integration Examples

### Shell Pipeline Integration

```bash
# Generate multiple files with parameter reuse
PARAMS="-p project_name=tokio -p has_tests=true"
pmat generate makefile rust/cli $PARAMS > Makefile
pmat generate readme rust/cli $PARAMS > README.md

# Complexity analysis with jq processing
pmat analyze complexity --format json | \
  jq '.files[] | select(.complexity.cyclomatic > 10)'
```

### CI/CD Integration

```yaml
# GitHub Actions
- name: Analyze Complexity
  run: |
    paiml-mcp-agent-toolkit analyze complexity \
      --format sarif \
      --output complexity.sarif
    
- name: Upload SARIF
  uses: github/codeql-action/upload-sarif@v2
  with:
    sarif_file: complexity.sarif
```

### IDE Integration

```jsonc
// VS Code tasks.json
{
  "label": "Generate Project Context",
  "type": "shell",
  "command": "paiml-mcp-agent-toolkit",
  "args": ["context", "rust", "--format", "json"],
  "problemMatcher": []
}
```

### Claude Code Integration

```bash
# Add MCP server
claude mcp add paiml-toolkit ~/.local/bin/paiml-mcp-agent-toolkit

# Use in Claude Code
# "Generate a Makefile for my Rust project"
# "Analyze complexity of this codebase"
# "Show me code hotspots from the last month"
```

## Troubleshooting

### Debug Mode

Enable detailed logging with trace-level output:

```bash
RUST_LOG=paiml_mcp_agent_toolkit=trace paiml-mcp-agent-toolkit generate makefile rust/cli
```

### Cache Diagnostics

```bash
# View cache statistics (shown during context generation)
paiml-mcp-agent-toolkit context rust

# Clear cache manually
rm -rf ~/.cache/paiml-mcp-agent-toolkit/

# Inspect cache entries
ls -la ~/.cache/paiml-mcp-agent-toolkit/
```

### Common Issues

#### Mode Detection Problems

If the tool runs in the wrong mode:

```bash
# Force CLI mode
pmat --mode cli list

# Check detection
echo "test" | pmat  # Should wait for JSON-RPC
pmat list           # Should show CLI output
```

#### Performance Issues

For slow performance:

1. Check cache effectiveness during context generation
2. Use `--include` patterns to limit file analysis
3. Ensure sufficient disk space for cache

#### Template Not Found

```bash
# List all available templates
paiml-mcp-agent-toolkit list

# Verify template URI format
paiml-mcp-agent-toolkit validate template://makefile/rust/cli
```

### Performance Profiling

```bash
# Time command execution
time paiml-mcp-agent-toolkit analyze complexity

# Memory usage
/usr/bin/time -v paiml-mcp-agent-toolkit context rust

# CPU profiling with perf (Linux)
perf record -g paiml-mcp-agent-toolkit analyze complexity
perf report
```

## Environment Variables

- `RUST_LOG` - Set logging level (e.g., `RUST_LOG=debug`)
- `MCP_VERSION` - Forces MCP mode when set
- `NO_COLOR` - Disable colored output

## Documentation Synchronization

This documentation is kept in sync with the implementation through integration tests that verify all documented features exist and work as described.

### Implementation Details

The project uses integration tests to ensure this documentation stays accurate:

1. **CLI Documentation Verification** (`server/tests/cli_documentation_sync.rs`)
   - Parses this file to extract all documented CLI commands
   - Runs `paiml-mcp-agent-toolkit --help` and verifies all commands are present
   - Runs each command with `--help` to verify subcommand documentation
   - Compares documented parameters with actual CLI argument parsing

2. **MCP Tools Documentation Verification** (`server/tests/mcp_documentation_sync.rs`)
   - Parses this file to extract all documented MCP tools
   - Starts MCP server and sends `tools/list` request
   - Verifies all documented tools exist in the response
   - Checks tool descriptions and parameter schemas match

3. **Example Verification** (`server/tests/documentation_examples.rs`)
   - Extracts code examples from this documentation
   - Validates CLI command structure
   - Verifies JSON-RPC examples are well-formed
   - Checks template URIs follow the correct format

### Test Data Structures

The tests use these structures to parse documentation:

```rust
#[derive(Debug, PartialEq)]
struct DocumentedCommand {
    name: String,
    description: String,
    subcommands: Vec<String>,
    arguments: Vec<String>,
    options: Vec<String>,
}

#[derive(Debug, PartialEq)]
struct DocumentedTool {
    name: String,
    description: String,
    required_params: Vec<String>,
    optional_params: Vec<String>,
}
```

### Running Documentation Sync Tests

```bash
# Run only documentation sync tests (with nextest for speed)
cargo nextest run --test cli_documentation_sync || cargo test --test cli_documentation_sync
cargo nextest run --test mcp_documentation_sync || cargo test --test mcp_documentation_sync
cargo nextest run --test documentation_examples || cargo test --test documentation_examples

# Run all documentation tests
cargo nextest run doc_sync || cargo test doc_sync
```

### CI Integration

The CI workflow fails if documentation is out of sync:

```yaml
# .github/workflows/ci.yml
- name: Verify Documentation Sync
  run: |
    (cargo nextest run doc_sync || cargo test doc_sync)
    if [ $? -ne 0 ]; then
      echo "Documentation is out of sync with implementation!"
      echo "Please update docs/cli-mcp.md to match the current implementation"
      exit 1
    fi
```

### Updating Documentation

When tests fail due to documentation drift:

1. **Review the test output** - It will show exactly what's missing or incorrect
2. **Update this file** - Add/modify the documentation to match implementation
3. **Run tests again** - Verify the documentation now matches
4. **Commit both changes** - Include implementation and documentation updates together

### Documentation Parsing Rules

The tests parse this markdown file with these rules:

1. **CLI Commands**: Extracted from the "CLI Command Reference" section
   - Command names from `### Command: \`command-name\`` headers
   - Descriptions from the first paragraph after the header
   - Parameters from code blocks after "Options:" or "Arguments:"

2. **MCP Tools**: Extracted from documentation and verified against actual implementation
   - Tool names from MCP-related sections
   - The tests run the MCP server and compare with documented tools

3. **Code Examples**: Extracted from fenced code blocks
   - Blocks marked with ` ```bash` are validated for command structure
   - Blocks containing `jsonrpc` are validated as proper JSON-RPC
   - Template URIs are validated for correct format

## See Also

- [Main README](../README.md) - Project overview and quick start
- [CLAUDE.md](../CLAUDE.md) - Development guidelines
- [GitHub Issues](https://github.com/paiml/paiml-mcp-agent-toolkit/issues) - Report bugs or request features