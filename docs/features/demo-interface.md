# Demo Interface Documentation

The `pmat demo` command provides an interactive demonstration of PMAT's capabilities across different protocol interfaces (CLI, HTTP, MCP, and TUI). It showcases how the same analysis capabilities can be accessed through various interfaces.

## Table of Contents

1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [Demo Modes](#demo-modes)
4. [Protocol Demonstrations](#protocol-demonstrations)
5. [Web Demo Interface](#web-demo-interface)
6. [TUI Demo Interface](#tui-demo-interface)
7. [Configuration Options](#configuration-options)
8. [Examples](#examples)
9. [Troubleshooting](#troubleshooting)

## Overview

The demo interface serves multiple purposes:

- **Educational**: Learn how different PMAT interfaces work
- **Comparative**: See the same analysis through different protocols
- **Interactive**: Explore analysis results in web or terminal UI
- **Validation**: Test PMAT capabilities on your codebase

## Quick Start

```bash
# Run web demo on current directory
pmat demo --web

# Run TUI demo
pmat demo --mode tui

# Compare all protocols
pmat demo --protocol all

# Demo specific repository
pmat demo --repo owner/repo
```

## Demo Modes

### 1. Protocol Demonstration Mode (Default)

Shows how to access PMAT functionality through different interfaces:

```bash
# Demo CLI protocol
pmat demo --protocol cli

# Demo HTTP protocol  
pmat demo --protocol http

# Demo MCP protocol
pmat demo --protocol mcp

# Demo all protocols
pmat demo --protocol all
```

### 2. Web Demo Mode

Interactive web interface with visualizations:

```bash
# Start web demo
pmat demo --web

# Custom port
pmat demo --web --port 8080

# Don't open browser automatically
pmat demo --web --no-browser
```

### 3. TUI Demo Mode

Terminal-based interactive interface:

```bash
# Start TUI demo (requires 'tui' feature)
pmat demo --mode tui
```

## Protocol Demonstrations

### CLI Protocol Demo

Shows direct command-line usage:

```bash
$ pmat demo --protocol cli
🚀 CLI Protocol Demo

Analyzing /path/to/project...

Results:
- Files analyzed: 150
- Average complexity: 3.2
- Technical debt: 12 hours
- Top complexity hotspots:
  1. src/parser.rs::parse_expression (complexity: 15)
  2. src/analyzer.rs::analyze_node (complexity: 12)
  ...
```

### HTTP Protocol Demo

Demonstrates REST API calls:

```bash
$ pmat demo --protocol http
🌐 HTTP Protocol Demo

Request:
  GET /demo/analyze?path=/path/to/project

Response:
  {
    "files_analyzed": 150,
    "average_complexity": 3.2,
    "technical_debt_hours": 12,
    "hotspots": [...]
  }
```

### MCP Protocol Demo

Shows Model Context Protocol usage:

```bash
$ pmat demo --protocol mcp
🔌 MCP Protocol Demo

Request:
  {
    "jsonrpc": "2.0",
    "method": "demo.analyze",
    "params": {
      "path": "/path/to/project"
    },
    "id": 1
  }

Response:
  {
    "jsonrpc": "2.0",
    "result": {
      "files_analyzed": 150,
      ...
    },
    "id": 1
  }
```

## Web Demo Interface

The web demo provides an interactive dashboard with:

### Features

1. **Real-time Analysis**
   - AST parsing with timing metrics
   - Complexity analysis visualization
   - Dependency graph rendering
   - Code churn heatmap

2. **Interactive Visualizations**
   - System architecture diagram (Mermaid)
   - Dependency graph (D3.js)
   - Complexity distribution charts
   - Hotspot grid view

3. **Navigation**
   - File browser
   - Function-level drill-down
   - Search and filter capabilities
   - Export options

### Web Demo Layout

```
┌─────────────────────────────────────────┐
│  PMAT Demo - Project Analysis Dashboard │
├─────────────────────────────────────────┤
│ Summary Stats    │ System Architecture  │
│ - Files: 150     │ [Mermaid Diagram]    │
│ - Complexity: 3.2│                      │
│ - Debt: 12h      │                      │
├──────────────────┴──────────────────────┤
│ Complexity Hotspots                     │
│ [Interactive Grid View]                 │
├─────────────────────────────────────────┤
│ Dependency Graph │ Code Churn Analysis │
│ [D3.js Graph]    │ [Heatmap]           │
└─────────────────────────────────────────┘
```

## TUI Demo Interface

Terminal-based interactive interface for exploring analysis results:

### Features

1. **Keyboard Navigation**
   - Arrow keys: Navigate
   - Enter: Select/Drill down
   - Tab: Switch panels
   - q: Quit
   - h: Help

2. **Interactive Panels**
   - File tree browser
   - Function list
   - Complexity metrics
   - Analysis details

3. **Real-time Updates**
   - Live analysis progress
   - Dynamic filtering
   - Search functionality

### TUI Layout

```
┌─ PMAT TUI Demo ─────────────────────────┐
│ Files │ Functions │ Metrics │ Details   │
├───────┴───────────┴─────────┴───────────┤
│ src/                                    │
│ ├── main.rs        analyze_file()   15  │
│ ├── parser.rs      parse_expr()     12  │
│ └── analyzer.rs    check_node()     8   │
├─────────────────────────────────────────┤
│ Complexity: 15  Cognitive: 20  LOC: 150 │
│ Technical Debt: 2.5 hours               │
│ Last Modified: 2 days ago               │
└─────────────────────────────────────────┘
[q]uit [h]elp [/]search [f]ilter
```

## Configuration Options

### Common Options

```bash
--path <PATH>           # Analyze specific directory
--url <URL>             # Clone and analyze from URL
--repo <OWNER/REPO>     # Analyze GitHub repository
--format <FORMAT>       # Output format (json|yaml|table)
--debug                 # Enable debug output
--debug-output <PATH>   # Save debug info to file
--skip-vendor           # Skip vendor directories
--max-line-length <N>   # Maximum line length for display
```

### Web Demo Options

```bash
--web                   # Enable web interface
--port <PORT>           # Web server port (default: 3000)
--no-browser            # Don't open browser automatically
```

### Protocol Demo Options

```bash
--protocol <PROTOCOL>   # Protocol to demo (cli|http|mcp|tui|all)
--show-api              # Show API introspection details
```

### Analysis Options

```bash
--target-nodes <N>      # Target nodes for graph (default: 20)
--centrality-threshold  # Centrality threshold (default: 0.5)
--merge-threshold <N>   # Merge threshold (default: 5)
```

## Examples

### Example 1: Quick Web Demo

```bash
# Analyze current project with web interface
pmat demo --web

# Opens browser to http://localhost:3000
# Shows interactive dashboard with all analysis results
```

### Example 2: Compare Protocols

```bash
# See how different interfaces handle the same analysis
pmat demo --protocol all --format json

# Output shows CLI, HTTP, and MCP responses
# Useful for understanding API differences
```

### Example 3: Analyze Remote Repository

```bash
# Demo with a GitHub repository
pmat demo --repo rust-lang/cargo --web

# Clones repository to temp directory
# Runs full analysis
# Opens web interface with results
```

### Example 4: TUI Exploration

```bash
# Interactive terminal interface
pmat demo --mode tui --path ./src

# Use keyboard to navigate
# Press 'h' for help
# Real-time filtering and search
```

### Example 5: Debug Protocol Details

```bash
# Show detailed protocol information
pmat demo --protocol mcp --show-api --debug

# Includes request/response details
# API introspection information
# Timing and performance metrics
```

## Troubleshooting

### Common Issues

1. **Web demo won't start**
   - Check if port is already in use
   - Try different port: `--port 8080`
   - Verify demo feature is enabled

2. **TUI demo not available**
   - Requires 'tui' feature to be compiled
   - Check terminal compatibility
   - Try different terminal emulator

3. **Browser doesn't open**
   - Use `--no-browser` and open manually
   - Check default browser settings
   - Copy URL from console output

4. **Analysis takes too long**
   - Use `--skip-vendor` to skip dependencies
   - Limit scope with specific path
   - Adjust thresholds for large projects

### Debug Mode

Enable debug output for troubleshooting:

```bash
# Console debug output
pmat demo --web --debug

# Save debug info to file
pmat demo --web --debug --debug-output debug.log
```

## Best Practices

1. **Start Simple**: Begin with web demo for visual exploration
2. **Compare Protocols**: Use `--protocol all` to understand differences
3. **Use Appropriate Interface**: 
   - Web for visual analysis
   - TUI for terminal environments
   - Protocol demos for API learning
4. **Performance**: Skip vendor directories for faster analysis
5. **Repository Analysis**: Use `--repo` for quick GitHub analysis

## See Also

- [CLI Reference](/docs/cli-reference.md) - Full CLI documentation
- [HTTP API](/rust-docs/http-api.md) - REST API details
- [MCP Protocol](/docs/features/mcp-protocol.md) - MCP integration
- [TUI Interface](/docs/features/tui-interface.md) - Terminal UI guide