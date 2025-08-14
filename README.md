# PAIML MCP Agent Toolkit (pmat)

[![Crates.io](https://img.shields.io/crates/v/pmat.svg)](https://crates.io/crates/pmat)
[![Documentation](https://docs.rs/pmat/badge.svg)](https://docs.rs/pmat)
[![CI/CD](https://github.com/paiml/paiml-mcp-agent-toolkit/actions/workflows/main.yml/badge.svg?branch=master)](https://github.com/paiml/paiml-mcp-agent-toolkit/actions/workflows/main.yml)
[![Quality Gate](https://github.com/paiml/paiml-mcp-agent-toolkit/actions/workflows/quality-gate-test.yml/badge.svg?branch=master)](https://github.com/paiml/paiml-mcp-agent-toolkit/actions/workflows/quality-gate-test.yml)
[![MCP Compatible](https://img.shields.io/badge/MCP-Compatible-green)](https://modelcontextprotocol.io)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Downloads](https://img.shields.io/crates/d/pmat)](https://crates.io/crates/pmat)
[![Rust 1.80+](https://img.shields.io/badge/rust-1.80+-orange.svg)](https://www.rust-lang.org)

**Zero-configuration AI context generation system** that analyzes any codebase instantly through CLI, MCP, or HTTP interfaces. Built by [Pragmatic AI Labs](https://paiml.com) with extreme quality standards and zero tolerance for technical debt.

> **🎉 v2.3.1 Release**: **Dependency Updates & PDMT Integration!** All MCP implementations consolidated into ONE high-performance pmcp SDK-based server. Features include:
> - **Quality Proxy**: Intercept and validate AI-generated code with strict quality enforcement
> - **PDMT Integration**: Deterministic todo generation with comprehensive quality gates
> - **Single Implementation**: Eliminated duplicate MCP servers, now ONE unified pmcp-based server
> - **10x Performance**: All MCP operations use high-performance pmcp SDK exclusively  
> - **18 Core Tools**: Analysis, refactoring, quality gates, PDMT todos, and context generation
> - **Zero Tolerance**: 0 SATD comments, 0 failing tests, comprehensive property-based testing

## 🚀 Installation

Install `pmat` using one of the following methods:

- **From Crates.io (Recommended):**
  ```bash
  cargo install pmat
  ```

- **With the Quick Install Script (Linux only):**
  ```bash
  curl -sSfL https://raw.githubusercontent.com/paiml/paiml-mcp-agent-toolkit/master/scripts/install.sh | sh
  ```
  
  **macOS users:** Please use `cargo install pmat` instead. Pre-built binaries are only available for Linux.

- **From Source:**
  ```bash
  git clone https://github.com/paiml/paiml-mcp-agent-toolkit
  cd paiml-mcp-agent-toolkit
  cargo build --release
  ```

- **From GitHub Releases:**
  Pre-built binaries for Linux are available on the [releases page](https://github.com/paiml/paiml-mcp-agent-toolkit/releases). macOS and Windows users should use `cargo install pmat`.

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

# Analyze specific files with include patterns
pmat analyze complexity --include "src/*.rs" --format json

# Test with validated examples (try these!)
cargo run --example complexity_demo
pmat analyze complexity --include "server/examples/complexity_*.rs"

# Find technical debt
pmat analyze satd

# Run comprehensive quality checks
pmat quality-gate --strict

# NEW: Quality Proxy - Enforce quality standards on AI-generated code
cargo run --example quality_proxy_demo

# NEW: PDMT Integration - Generate deterministic todos with quality enforcement
pmat mcp-call pdmt_deterministic_todos --requirements '["implement auth", "add logging"]'
```

### Using as a Library

Add to your `Cargo.toml`:

```toml
[dependencies]
pmat = "2.3"
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
- **Complexity Analysis** - Accurate McCabe cyclomatic and cognitive complexity metrics with AST-based precision
  - ✅ **v0.28.6+**: Fixed algorithm accuracy - now provides 100% correct complexity calculations
  - Supports nested control flow, match statements, async functions, and complex branching
  - Validated against manual calculations with comprehensive test examples
- **Dead Code Detection** - Find unused code across your project
- **Technical Debt Gradient (TDG)** - Quantify and prioritize technical debt
- **SATD Detection** - Find Self-Admitted Technical Debt in comments
- **Code Duplication** - Detect exact, renamed, gapped, and semantic clones

### 🛠️ Refactoring & Quality Tools
- **AI-Powered Auto Refactoring** - `pmat refactor auto` achieves extreme quality standards
  - **Single File Mode** - `pmat refactor auto --single-file-mode --file path/to/file.rs` for targeted refactoring
- **Documentation Cleanup** - `pmat refactor docs` maintains Zero Tolerance Quality Standards
- **Interactive Refactoring** - Step-by-step guided refactoring with explanations
- **Enforcement Mode** - Enforce extreme quality standards using state machines
  - **Single File Mode** - `pmat enforce extreme --file path/to/file.rs` for file-specific enforcement
- **Quality Proxy** (v2.3) - Intercept and validate AI-generated code before it's written
  - Enforces complexity thresholds, SATD detection, and documentation requirements
  - Three modes: Strict (reject), Advisory (warn), and Auto-Fix (refactor automatically)
  - Integrated into unified MCP server for all operations
  - Perfect for CI/CD pipelines and AI agent integrations
- **PDMT Integration** (v2.3) - Deterministic todo generation with quality enforcement
  - Generates reproducible, high-quality todo lists from requirements
  - Enforces 80%+ coverage, zero SATD, complexity limits
  - Includes validation commands and success criteria for each todo
  - Full integration with quality proxy for code validation

### 📊 Quality Gates & CI/CD Integration
- **Lint Hotspot Analysis** - Find files with highest defect density using EXTREME Clippy standards
  - **Single File Mode** - `pmat lint-hotspot --file path/to/file.rs` for targeted analysis
- **Provability Analysis** - Lightweight formal verification with property analysis
- **Defect Prediction** - ML-based prediction of defect-prone code
- **Quality Enforcement** - Exit with error codes for CI/CD integration
  - **NEW**: All analyze commands now support `--fail-on-violation` for CI/CD pipelines
  - Exit code 0 on success, 1 when violations exceed thresholds
  - Perfect for GitHub Actions, GitLab CI, Jenkins, and other CI/CD systems

### 🤖 Agent Scaffolding (NEW)
- **MCP Agent Templates** - Generate deterministic MCP-compatible agents
  - MCP Tool Server - Standard async MCP server with tool handlers
  - State Machine Workflow - Agents with state transitions and invariants
  - Deterministic Calculator - Pure computation agents
  - Hybrid Analyzer - Deterministic core with AI wrapper
- **Interactive Mode** - Guided agent creation wizard
- **Quality Enforcement** - All generated agents meet Toyota Way standards
  - Zero SATD comments
  - Maximum complexity 10
  - Comprehensive property tests
- **Feature-Rich Templates** - Support for monitoring, tracing, health checks

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
pmat context -t rust                           # Force toolchain
pmat context --skip-expensive-metrics          # Fast mode

# Code analysis
pmat analyze complexity --top-files 5         # Complexity analysis
pmat analyze complexity --fail-on-violation   # CI/CD mode - exit(1) if violations
pmat analyze churn --days 30                  # Git history analysis  
pmat analyze dag --target-nodes 25            # Dependency graph
pmat analyze dead-code --format json          # Dead code detection
pmat analyze dead-code --fail-on-violation --max-percentage 10  # CI/CD mode
pmat analyze satd --top-files 10              # Technical debt
pmat analyze satd --strict --fail-on-violation  # Zero tolerance for debt
pmat analyze deep-context --format json       # Comprehensive analysis
pmat analyze deep-context --full               # Full detailed report
pmat analyze deep-context --include-pattern "*.rs" # Filter by file pattern
pmat analyze big-o                            # Big-O complexity analysis
pmat analyze makefile-lint                    # Makefile quality linting
pmat analyze proof-annotations                # Provability analysis

# Analysis commands
pmat analyze graph-metrics                    # Graph centrality metrics (PageRank, betweenness, closeness)
pmat analyze name-similarity "function_name"  # Fuzzy name matching with phonetic support
pmat analyze symbol-table                     # Symbol extraction with cross-references
pmat analyze duplicates --min-lines 10        # Code duplication detection
pmat quality-gate --fail-on-violation         # Comprehensive quality enforcement
pmat diagnose --verbose                       # Self-diagnostics and health checks

# WebAssembly Support
pmat analyze assemblyscript --wasm-complexity  # AssemblyScript analysis with WASM metrics
pmat analyze webassembly --include-binary      # WebAssembly binary and text format analysis

# Project scaffolding
pmat scaffold project rust --templates makefile,readme,gitignore  # Scaffold a Rust project
pmat scaffold agent --name my_agent --template mcp-server         # Create MCP agent
pmat scaffold agent --interactive                                 # Interactive agent creation
pmat list                                      # Available templates
pmat list-agents                                # Available agent templates

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
pmat quality-gate --fail-on-violation         # CI/CD quality gate
pmat quality-gate --checks complexity,satd,security  # Specific checks
pmat quality-gate --format human              # Human-readable output
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

### MCP Integration

#### Using with Claude Code

```bash
# Add to Claude Code
claude mcp add pmat ~/.local/bin/pmat
```
<details>
<summary><i>💫 See Claude Code usage in action</i></summary>
<br>
<img src="https://github.com/paiml/paiml-mcp-agent-toolkit/blob/master/assets/demo1.gif" width=875>
</details>

#### Enhanced MCP Server Performance (v2.0+)

**New in v2.0**: pmat now uses the [pmcp 1.2.0](https://github.com/paiml/pmcp) Rust MCP SDK for dramatically improved performance:

- **10x faster response times** (50ms → 5ms average)
- **Native WebSocket support** for browser-based clients
- **HTTP/SSE transport** for web integration
- **Type-safe tool handlers** with compile-time validation
- **Production-grade reliability** with comprehensive testing

The unified MCP server uses the high-performance [pmcp](https://github.com/paiml/pmcp) Rust SDK exclusively:

```bash
# Run the unified MCP server (auto-detected)
pmat

# With debug logging
pmat --debug
```

Supported transports:
- **stdio** (default): For CLI tools like Claude Desktop
- **WebSocket**: For browser-based clients (`ws://localhost:8080`)
- **HTTP/SSE**: For web applications with Server-Sent Events

#### Architecture

All MCP operations now flow through a single, unified server implementation:

```rust
// The unified server provides all tools in one place
use pmat::mcp_pmcp::UnifiedServer;

let server = UnifiedServer::new()?;
server.run().await?;

// Includes 17 core tools:
// - Analysis: complexity, SATD, dead code, DAG, deep context, Big-O
// - Refactoring: start, nextIteration, getState, stop
// - Quality: quality_gate, quality_proxy
// - Context: git_operation, generate_context, scaffold_project
```

The unified architecture provides:
- **Single Implementation**: No duplicate code or multiple server variants
- **10x Performance**: All operations use pmcp SDK's optimized runtime
- **Type Safety**: Compile-time validation of all tool interfaces
- **Quality Integration**: Built-in quality proxy for all operations
- **Consistent Behavior**: One code path for all MCP tools

Available MCP tools:
- `generate_template` - Generate project files from templates
- `scaffold_project` - Generate complete project structure  
- `scaffold_agent` - Create MCP-compatible agents with templates
- `analyze_complexity` - Code complexity metrics **with tool composition**
- `analyze_code_churn` - Git history analysis
- `analyze_dag` - Dependency graph generation
- `analyze_dead_code` - Dead code detection
- `analyze_deep_context` - Comprehensive analysis **with tool composition**
- `generate_context` - Zero-config context generation
- `analyze_big_o` - Big-O complexity analysis with confidence scores
- `analyze_makefile_lint` - Lint Makefiles with 50+ quality rules
- `analyze_proof_annotations` - Lightweight formal verification
- `analyze_graph_metrics` - Graph centrality and PageRank analysis
- `refactor_interactive` - Interactive refactoring with explanations
- `quality_proxy` - **NEW** Proxy AI-generated code through quality gates

#### 🔗 MCP Tool Composition (NEW)

AI agents can now chain analysis tools using the `--files` parameter:

```bash
# Step 1: Find complexity hotspots
pmat analyze complexity --top-files 5 --format json

# Step 2: Deep analyze those specific files (MCP composition)
pmat analyze comprehensive --files src/complex.rs,src/legacy.rs

# Step 3: Target refactoring on problematic files
pmat refactor auto --files src/complex.rs
```

**MCP Agent Workflow Example:**
```javascript
// AI agent discovers hotspots
const hotspots = await callTool("pmat_analyze_complexity", {
  top_files: 5, 
  format: "json"
});

// Agent extracts file paths and performs deep analysis
const detailed = await callTool("pmat_analyze_comprehensive", {
  files: hotspots.files.map(f => f.path)
});

// Agent generates targeted refactoring plan
const refactor = await callTool("pmat_refactor_auto", {
  files: detailed.high_risk_files
});
```

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

## 🚀 CI/CD Integration

All analyze commands now support `--fail-on-violation` for seamless CI/CD integration:

### GitHub Actions Example
```yaml
name: Code Quality
on: [push, pull_request]

jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Install pmat
        run: cargo install pmat
        
      - name: Check Complexity
        run: |
          pmat analyze complexity \
            --max-cyclomatic 15 \
            --max-cognitive 10 \
            --fail-on-violation
            
      - name: Check Technical Debt
        run: pmat analyze satd --strict --fail-on-violation
        
      - name: Check Dead Code
        run: |
          pmat analyze dead-code \
            --max-percentage 10.0 \
            --fail-on-violation
            
      - name: Run Quality Gate
        run: pmat quality-gate --fail-on-violation
```

### Configurable Thresholds
- **Complexity**: `--max-cyclomatic` (default: 20), `--max-cognitive` (default: 15)
- **Dead Code**: `--max-percentage` (default: 15.0%)
- **SATD**: Fails on ANY technical debt when using `--fail-on-violation`

### Exit Codes
- **0**: Success - no violations found or within thresholds
- **1**: Failure - violations exceed configured thresholds

See `examples/ci_integration.rs` for more CI/CD patterns including GitLab CI, Jenkins, and pre-commit hooks.

## Recent Updates

### 🐛 v0.29.6 - Critical Bug Fixes
- **Fixed Quality Gate Bug**: Quality gate dead code detection now correctly analyzes code instead of always reporting violations
- **Fixed Include Patterns**: `--include` patterns now properly work with test directories (e.g., `--include "tests/**/*.rs"`)
- **Fixed Clippy Warning**: Replaced deprecated `map_or` with `is_some_and` for cleaner code

### 🏆 v0.28.14 - Toyota Way Kaizen Success
- **Massive Complexity Reduction**: Core `handle_refactor_auto` function complexity reduced from 136 → 21 (84% reduction)
- **Zero Quality Violations**: Project now maintains 0 lint violations, 0 max complexity, 0 SATD comments
- **Toyota Way Implementation**: Applied Kaizen (continuous improvement), Genchi Genbutsu (go and see), Jidoka (quality at source), and Poka-Yoke (error-proofing) principles
- **Code Size Optimization**: Net reduction of 3,401 lines while improving functionality and maintainability
- **Comprehensive Testing**: Added extensive property tests, doctests, and unit tests for all refactored components
- **Zero Compromise**: No hacks, shortcuts, or technical debt introduced during refactoring

### 🚦 v0.28.9 - CI/CD Integration & Exit Codes
- **CI/CD Support**: All analyze commands now support `--fail-on-violation` flag
- **Exit Codes**: Commands exit with code 1 when violations exceed thresholds
- **Configurable Thresholds**: Added `--max-percentage` for dead-code analysis
- **Examples**: Added comprehensive CI/CD examples for GitHub Actions, GitLab CI, Jenkins
- **Documentation**: Updated all documentation with CI/CD integration patterns

### 🔍 v0.28.3 - Enhanced Single-File Analysis
- **Single File Analysis**: Added `--file` flag to `pmat analyze comprehensive` for analyzing individual files.
- **Bug Fix**: Fixed "No such file or directory" error in single-file refactor mode by using dynamic executable path detection.
- **Test Improvements**: Fixed stack overflow issues in CLI tests by using wildcard pattern matching.
- **Documentation**: Updated CLI reference with new single-file analysis capabilities.

### 🚦 v0.28.2 - Quality Gate Improvements
- **Quality Gate Tests**: Added comprehensive integration tests for CI/CD quality gates.
- **Public API**: Made quality gate structs public for better testing support.
- **Doctests**: Added doctests for quality gate and DAG generation functions.
- **Bug Fixes**: Fixed mermaid test failures and unused import warnings.

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

## Toyota Way Quality Standards

This project exemplifies the Toyota Way philosophy through disciplined quality practices:

### **Kaizen (改善) - Continuous Improvement**
- **84% Complexity Reduction**: Achieved through systematic refactoring
- **Zero Quality Violations**: Maintained through iterative improvement
- **Net -3,401 Lines**: Simplified while enhancing functionality

### **Genchi Genbutsu (現地現物) - Go and See**
- **Data-Driven Decisions**: Used actual metrics to identify complexity hotspots
- **Root Cause Analysis**: Fixed underlying architectural issues, not symptoms
- **Measurement-Based**: Every improvement verified through quantitative analysis

### **Jidoka (自働化) - Quality at Source**
- **ZERO SATD**: No TODO, FIXME, HACK, or placeholder implementations
- **ZERO High Complexity**: No function exceeds cyclomatic complexity of 20
- **ZERO Known Defects**: All code must be fully functional
- **ZERO Incomplete Features**: Only complete, tested features are merged

### **Poka-Yoke (ポカヨケ) - Error Prevention**
- **Comprehensive Testing**: Property tests, doctests, and unit tests prevent regressions
- **Automated Quality Gates**: CI/CD integration prevents quality degradation
- **Type Safety**: Rust's type system eliminates entire categories of errors

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

## 🚦 CI/CD Quality Gates

### GitHub Actions Integration
```yaml
# .github/workflows/quality.yml
- name: Run Quality Gate
  run: |
    pmat quality-gate \
      --checks complexity,satd,security,dead-code \
      --max-complexity-p99 20 \
      --fail-on-violation
```

### Available Quality Checks
- **complexity** - Cyclomatic complexity thresholds
- **satd** - Self-admitted technical debt (TODO/FIXME/HACK)
- **security** - Hardcoded passwords and API keys
- **dead-code** - Unused functions and variables
- **entropy** - Code diversity metrics
- **duplicates** - Code duplication detection
- **coverage** - Test coverage thresholds
- **sections** - Required documentation sections
- **provability** - Formal verification readiness

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
4. Run quality checks before committing:
   ```bash
   make lint  # Check code quality
   make test  # Run all tests (fast, doctests, property tests, examples)
   ```
5. Submit a pull request with a clear description of changes

**Note**: The `make test` command runs comprehensive testing including:
- ⚡ Fast unit and integration tests
- 📚 Documentation tests (doctests)
- 🎲 Property-based tests
- 📘 All cargo examples

See [CONTRIBUTING.md](https://github.com/paiml/paiml-mcp-agent-toolkit/blob/master/CONTRIBUTING.md) for detailed guidelines.

## 📄 License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/paiml/paiml-mcp-agent-toolkit/blob/master/LICENSE-APACHE))

## References

* [Real-world AI needs eventual provability](https://papers.ssrn.com/sol3/papers.cfm?abstract_id=5289884)
- MIT license ([LICENSE-MIT](https://github.com/paiml/paiml-mcp-agent-toolkit/blob/master/LICENSE-MIT))

at your option.

---

**Built with ❤️ by [Pragmatic AI Labs](https://paiml.com)**
