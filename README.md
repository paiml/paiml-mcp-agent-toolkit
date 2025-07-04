# PAIML MCP Agent Toolkit (pmat)

[![Crates.io](https://img.shields.io/crates/v/pmat.svg)](https://crates.io/crates/pmat)
[![Documentation](https://docs.rs/pmat/badge.svg)](https://docs.rs/pmat)
[![CI/CD](https://github.com/paiml/paiml-mcp-agent-toolkit/actions/workflows/main.yml/badge.svg?branch=master)](https://github.com/paiml/paiml-mcp-agent-toolkit/actions/workflows/main.yml)
[![codecov](https://codecov.io/gh/paiml/paiml-mcp-agent-toolkit/branch/master/graph/badge.svg)](https://codecov.io/gh/paiml/paiml-mcp-agent-toolkit)
[![MCP Compatible](https://img.shields.io/badge/MCP-Compatible-green)](https://modelcontextprotocol.io)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Downloads](https://img.shields.io/crates/d/pmat)](https://crates.io/crates/pmat)
[![Rust 1.80+](https://img.shields.io/badge/rust-1.80+-orange.svg)](https://www.rust-lang.org)

**Zero-configuration AI context generation system** that analyzes any codebase instantly through CLI, MCP, or HTTP interfaces. Built by [Pragmatic AI Labs](https://paiml.com) with extreme quality standards and zero tolerance for technical debt.

## 🚀 Installation

Install `pmat` using one of the following methods:

- **From Crates.io (Recommended):**
  ```bash
  cargo install pmat
  ```

- **With the Quick Install Script (Linux/macOS):**
  ```bash
  curl -sSfL https://raw.githubusercontent.com/paiml/paiml-mcp-agent-toolkit/master/scripts/install.sh | sh
  ```

- **From Source:**
  ```bash
  git clone https://github.com/paiml/paiml-mcp-agent-toolkit
  cd paiml-mcp-agent-toolkit
  cargo build --release
  ```

- **From GitHub Releases:**
  Pre-built binaries for Linux, macOS, and Windows are available on the [releases page](https://github.com/paiml/paiml-mcp-agent-toolkit/releases).

### Requirements
- **Rust:** 1.80.0 or later
- **Git:** For repository analysis

## 🚀 Getting Started

### Quick Start

```bash
# Analyze current directory
pmat context

# Get complexity metrics for top 10 files
pmat analyze complexity --top-files 10

# Find technical debt
pmat analyze satd

# Run comprehensive quality checks
pmat quality-gate --strict
```

### Using as a Library

Add to your `Cargo.toml`:

```toml
[dependencies]
pmat = "0.28.0"
```

Basic usage:

```rust
use pmat::{
    services::code_analysis::CodeAnalysisService,
    types::ProjectPath,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = CodeAnalysisService::new();
    let path = ProjectPath::new(".");
    
    // Generate context
    let context = service.generate_context(path, None).await?;
    println!("Project context: {}", context);
    
    // Analyze complexity
    let complexity = service.analyze_complexity(path, Some(10)).await?;
    println!("Complexity results: {:?}", complexity);
    
    Ok(())
}
```

## Key Features

### 🔍 Code Analysis
- **Deep Context Analysis** - Comprehensive AST-based code analysis with defect prediction
- **Complexity Analysis** - Cyclomatic and cognitive complexity metrics
- **Dead Code Detection** - Find unused code across your project
- **Technical Debt Gradient (TDG)** - Quantify and prioritize technical debt
- **SATD Detection** - Find Self-Admitted Technical Debt in comments
- **Code Duplication** - Detect exact, renamed, gapped, and semantic clones

### 🛠️ Refactoring Tools
- **AI-Powered Auto Refactoring** - `pmat refactor auto` achieves extreme quality standards
  - **Single File Mode** - `pmat refactor auto --single-file-mode --file path/to/file.rs` for targeted refactoring
- **Documentation Cleanup** - `pmat refactor docs` maintains Zero Tolerance Quality Standards
- **Interactive Refactoring** - Step-by-step guided refactoring with explanations
- **Enforcement Mode** - Enforce extreme quality standards using state machines
  - **Single File Mode** - `pmat enforce extreme --file path/to/file.rs` for file-specific enforcement

### 📊 Quality Gates
- **Lint Hotspot Analysis** - Find files with highest defect density using EXTREME Clippy standards
  - **Single File Mode** - `pmat lint-hotspot --file path/to/file.rs` for targeted analysis
- **Provability Analysis** - Lightweight formal verification with property analysis
- **Defect Prediction** - ML-based prediction of defect-prone code
- **Quality Enforcement** - Exit with error codes for CI/CD integration

### 🔧 Language Support
- **Rust** - Full support with cargo integration
- **TypeScript/JavaScript** - Modern AST-based analysis
- **Python** - Comprehensive Python 3 support
- **Kotlin** - Memory-safe parsing with full language support
- **C/C++** - Tree-sitter based analysis
- **WebAssembly** - WASM binary and text format analysis
- **AssemblyScript** - TypeScript-like syntax for WebAssembly
- **Makefiles** - Specialized linting and analysis

## 📋 Tool Usage

### CLI Interface

```bash
# Zero-configuration context generation
pmat context                                    # Auto-detects language
pmat context --format json                     # JSON output
pmat context rust                              # Force language

# Code analysis
pmat analyze complexity --top-files 5         # Complexity analysis
pmat analyze churn --days 30                  # Git history analysis  
pmat analyze dag --target-nodes 25            # Dependency graph
pmat analyze dead-code --format json          # Dead code detection
pmat analyze satd --top-files 10              # Technical debt
pmat analyze deep-context --format json       # Comprehensive analysis
pmat analyze big-o                            # Big-O complexity analysis
pmat analyze makefile-lint                    # Makefile quality linting
pmat analyze proof-annotations                # Provability analysis

# Analysis commands
pmat analyze graph-metrics                    # Graph centrality metrics (PageRank, betweenness, closeness)
pmat analyze name-similarity "function_name"  # Fuzzy name matching with phonetic support
pmat analyze symbol-table                     # Symbol extraction with cross-references
pmat analyze duplicates --min-lines 10        # Code duplication detection
pmat quality-gate --strict                    # Comprehensive quality enforcement
pmat diagnose --verbose                       # Self-diagnostics and health checks

# WebAssembly Support
pmat analyze assemblyscript --wasm-complexity  # AssemblyScript analysis with WASM metrics
pmat analyze webassembly --include-binary      # WebAssembly binary and text format analysis

# Project scaffolding
pmat scaffold rust --templates makefile,readme,gitignore
pmat list                                      # Available templates

# Refactoring engine
pmat refactor interactive                      # Interactive refactoring
pmat refactor serve --config refactor.json     # Batch refactoring
pmat refactor status                          # Check refactor progress
pmat refactor resume                          # Resume from checkpoint
pmat refactor auto                            # AI-powered automatic refactoring
pmat refactor docs --dry-run                  # Clean up documentation

# Demo and visualization
pmat demo --format table                      # CLI demo
pmat demo --web --port 8080                   # Web interface
pmat demo --repo https://github.com/user/repo # Analyze GitHub repo

# Quality enforcement
pmat quality-gate --fail-on-violation         # Run all quality checks
pmat enforce extreme                          # Enforce extreme quality standards
```

<details>
<summary><i>💫 See CLI usage in action</i></summary>
<br>
<b>Context and code analysis:</b>
<img src="https://github.com/paiml/paiml-mcp-agent-toolkit/blob/master/assets/demo2.gif" width=875>
<br><br>
<b>Running demos/visualization:</b>
<img src="https://github.com/paiml/paiml-mcp-agent-toolkit/blob/master/assets/demo3.gif" width=875>
</details>

### MCP Integration (Claude Code)

```bash
# Add to Claude Code
claude mcp add pmat ~/.local/bin/pmat
```
<details>
<summary><i>💫 See Claude Code usage in action</i></summary>
<br>
<img src="https://github.com/paiml/paiml-mcp-agent-toolkit/blob/master/assets/demo1.gif" width=875>
</details>

Available MCP tools:
- `generate_template` - Generate project files from templates
- `scaffold_project` - Generate complete project structure  
- `analyze_complexity` - Code complexity metrics
- `analyze_code_churn` - Git history analysis
- `analyze_dag` - Dependency graph generation
- `analyze_dead_code` - Dead code detection
- `analyze_deep_context` - Comprehensive analysis
- `generate_context` - Zero-config context generation
- `analyze_big_o` - Big-O complexity analysis with confidence scores
- `analyze_makefile_lint` - Lint Makefiles with 50+ quality rules
- `analyze_proof_annotations` - Lightweight formal verification
- `analyze_graph_metrics` - Graph centrality and PageRank analysis
- `refactor_interactive` - Interactive refactoring with explanations

### HTTP API

```bash
# Start server
pmat serve --port 8080 --cors

# API endpoints
curl "http://localhost:8080/health"
curl "http://localhost:8080/api/v1/analyze/complexity?top_files=5"
curl "http://localhost:8080/api/v1/templates"

# POST analysis
curl -X POST "http://localhost:8080/api/v1/analyze/deep-context" \
  -H "Content-Type: application/json" \
  -d '{"project_path":"./","include":["ast","complexity","churn"]}'
```

## Recent Updates

### 🏆 v0.26.3 - Quality Uplift
- **SATD Elimination**: Removed all TODO/FIXME/HACK comments from implementation.
- **Complexity Reduction**: All functions now below a cyclomatic complexity of 20.
- **Extreme Linting**: `make lint` passes with pedantic and nursery standards.
- **Single File Mode**: Enhanced support for targeted quality improvements.

### 🧹 v0.26.1 - Documentation Cleanup (`pmat refactor docs`)
- AI-assisted documentation cleanup to maintain Zero Tolerance Quality Standards.
- Identifies and removes temporary files, outdated reports, and build artifacts.
- Interactive mode for reviewing files before removal with automatic backups.

### 🔥 v0.26.0 - New Analysis Commands
- **Graph Metrics**: `pmat analyze graph-metrics` for centrality analysis.
- **Name Similarity**: `pmat analyze name-similarity` for fuzzy name matching.
- **Symbol Table**: `pmat analyze symbol-table` for symbol extraction.
- **Code Duplication**: `pmat analyze duplicates` for detecting duplicate code.

## Zero Tolerance Quality Standards

This project follows strict quality standards:
- **ZERO SATD**: No TODO, FIXME, HACK, or placeholder implementations
- **ZERO High Complexity**: No function exceeds cyclomatic complexity of 20
- **ZERO Known Defects**: All code must be fully functional
- **ZERO Incomplete Features**: Only complete, tested features are merged

## 📊 Output Formats

- **JSON** - Structured data for tools and APIs
- **Markdown** - Human-readable reports
- **SARIF** - Static analysis format for IDEs
- **Mermaid** - Dependency graphs and diagrams

## 🎯 Use Cases

### For AI Agents
- **Context Generation**: Give AI perfect project understanding
- **Code Analysis**: Deterministic metrics and facts
- **Template Generation**: Scaffolding with best practices

### For Developers  
- **Code Reviews**: Automated complexity and quality analysis
- **Technical Debt**: SATD detection and prioritization
- **Onboarding**: Quick project understanding
- **CI/CD**: Integrate quality gates and analysis

### For Teams
- **Documentation**: Auto-generated project overviews
- **Quality Gates**: Automated quality scoring
- **Dependency Analysis**: Visual dependency graphs
- **Project Health**: Comprehensive health metrics

## 📚 Documentation

Explore our comprehensive documentation to get the most out of `pmat`.

### Getting Started
- **[Architecture](https://github.com/paiml/paiml-mcp-agent-toolkit/blob/master/docs/architecture/ARCHITECTURE.md)**: Understand the system design and principles.
- **[CLI Reference](https://github.com/paiml/paiml-mcp-agent-toolkit/blob/master/docs/cli-reference.md)**: View the full command-line interface guide.
- **[API Documentation](https://docs.rs/pmat)**: Browse the complete Rust API documentation on docs.rs.

### Usage Guides
- **[Feature Overview](https://github.com/paiml/paiml-mcp-agent-toolkit/blob/master/docs/features/README.md)**: Discover all available features.
- **[MCP Integration](https://github.com/paiml/paiml-mcp-agent-toolkit/blob/master/docs/features/mcp-protocol.md)**: Learn how to integrate `pmat` with AI agents.
- **[CI/CD Integration](https://github.com/paiml/paiml-mcp-agent-toolkit/blob/master/docs/integrations/ci-cd-integration.md)**: Set up quality gates in your CI/CD pipeline.

### Development
- **[Contributing Guide](https://github.com/paiml/paiml-mcp-agent-toolkit/blob/master/CONTRIBUTING.md)**: Read our guidelines for contributing to the project.
- **[Release Process](https://github.com/paiml/paiml-mcp-agent-toolkit/blob/master/docs/release-process.md)**: Follow our step-by-step release workflow.

## 🛠️ System Operations

### Memory Management

For systems with low swap space, we provide a configuration tool:

```bash
make config-swap      # Configure 8GB swap (requires sudo)
make clear-swap       # Clear swap memory between heavy operations
```

## 🧪 Testing

The project uses a distributed test architecture for fast feedback:

```bash
# Run specific test suites
make test-unit        # <10s - Core logic tests
make test-services    # <30s - Service integration
make test-protocols   # <45s - Protocol validation
make test-e2e         # <120s - Full system tests
make test-performance # Performance regression

# Run all tests in parallel
make test-all

# Coverage analysis
make coverage-stratified
```

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](https://github.com/paiml/paiml-mcp-agent-toolkit/blob/master/CONTRIBUTING.md) for details.

### Quick Start for Contributors

```bash
# Clone and setup
git clone https://github.com/paiml/paiml-mcp-agent-toolkit
cd paiml-mcp-agent-toolkit

# Install dependencies
make install-deps

# Run tests
make test-fast      # Quick validation
make test-all       # Complete test suite

# Check code quality
make lint           # Run extreme quality lints
make coverage       # Generate coverage report
```

### Development Workflow

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes following our Zero Tolerance Quality Standards
4. Run `make lint` and `make test-fast` before committing
5. Submit a pull request with a clear description of changes

See [CONTRIBUTING.md](https://github.com/paiml/paiml-mcp-agent-toolkit/blob/master/CONTRIBUTING.md) for detailed guidelines.

## 📄 License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/paiml/paiml-mcp-agent-toolkit/blob/master/LICENSE-APACHE))
- MIT license ([LICENSE-MIT](https://github.com/paiml/paiml-mcp-agent-toolkit/blob/master/LICENSE-MIT))

at your option.

---

**Built with ❤️ by [Pragmatic AI Labs](https://paiml.com)**