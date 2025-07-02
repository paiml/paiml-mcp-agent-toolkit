# PAIML MCP Agent Toolkit (pmat)

[![CI/CD](https://github.com/paiml/paiml-mcp-agent-toolkit/actions/workflows/main.yml/badge.svg?branch=master)](https://github.com/paiml/paiml-mcp-agent-toolkit/actions/workflows/main.yml) [![MCP Compatible](https://img.shields.io/badge/MCP-Compatible-green)](https://modelcontextprotocol.io) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Zero-configuration AI context generation system** that analyzes any codebase instantly through CLI, MCP, or HTTP interfaces. Built by [Pragmatic AI Labs](https://paiml.com) with extreme quality standards and zero tolerance for technical debt.

## 🚀 Installation

### Quick Install (Linux/macOS)

```bash
curl -sSfL https://raw.githubusercontent.com/paiml/paiml-mcp-agent-toolkit/master/scripts/install.sh | sh
```

### Manual Installation

```bash
# Install from crates.io
cargo install paiml-mcp-agent-toolkit

# Or build from source
git clone https://github.com/paiml/paiml-mcp-agent-toolkit
cd paiml-mcp-agent-toolkit
cargo build --release
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
- **Documentation Cleanup** - `pmat refactor docs` maintains Zero Tolerance Quality Standards
- **Interactive Refactoring** - Step-by-step guided refactoring with explanations
- **Enforcement Mode** - Enforce extreme quality standards using state machines

### 📊 Quality Gates
- **Lint Hotspot Analysis** - Find files with highest defect density using EXTREME Clippy standards
- **Provability Analysis** - Lightweight formal verification with property analysis
- **Defect Prediction** - ML-based prediction of defect-prone code
- **Quality Enforcement** - Exit with error codes for CI/CD integration

### 🔧 Language Support
- **Rust** - Full support with cargo integration
- **TypeScript/JavaScript** - Modern AST-based analysis
- **Python** - Comprehensive Python 3 support
- **Kotlin** - Memory-safe parsing with full language support
- **C/C++** - Tree-sitter based analysis
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

# New in v0.26.0
pmat analyze graph-metrics                    # Graph centrality metrics (PageRank, betweenness, closeness)
pmat analyze name-similarity "function_name"  # Fuzzy name matching with phonetic support
pmat analyze symbol-table                     # Symbol extraction with cross-references
pmat analyze duplicates --min-lines 10        # Code duplication detection
pmat quality-gate --strict                    # Comprehensive quality enforcement
pmat diagnose --verbose                       # Self-diagnostics and health checks

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
claude mcp add paiml-toolkit ~/.local/bin/pmat
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

## New in v0.26.1

### 🧹 Documentation Cleanup (`pmat refactor docs`)
AI-assisted documentation cleanup that maintains Zero Tolerance Quality Standards:
- Identifies temporary files, outdated status reports, and build artifacts
- Interactive mode for reviewing files before removal
- Automatic backup before making changes
- Customizable patterns and preservation rules

Example:
```bash
# Dry run to see what would be removed
pmat refactor docs --dry-run

# Interactive mode
pmat refactor docs --format interactive

# Auto-remove with backup
pmat refactor docs --auto-remove --backup
```

### 🔥 EXTREME Quality Lint Analysis
The `lint-hotspot` command now uses the strictest possible quality standards by default:
```bash
# Runs with: -D warnings -D clippy::pedantic -D clippy::nursery -D clippy::cargo
pmat analyze lint-hotspot
```

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

### Feature Documentation

- **[Feature Overview](docs/features/README.md)** - Complete feature index
- **[Makefile Linter](docs/features/makefile-linter.md)** - 50+ rules for Makefile quality
- **[Emit-Refactor Engine](docs/features/emit-refactor-engine.md)** - Real-time defect detection & refactoring
- **[Excellence Tracker](docs/features/excellence-tracker.md)** - Code quality metrics tracking
- **[Technical Debt Gradient](docs/features/technical-debt-gradient.md)** - Quantitative debt measurement
- **[MCP Protocol](docs/features/mcp-protocol.md)** - AI agent integration guide
- **[Distributed Testing](docs/features/distributed-testing.md)** - Fast feedback test architecture

### API Documentation

- [Architecture](docs/architecture/ARCHITECTURE.md)
- [CLI Reference](rust-docs/cli-reference.md)
- [MCP Protocol](rust-docs/mcp-protocol.md) 
- [HTTP API](rust-docs/http-api.md)
- [Contributing Guidelines](CONTRIBUTING.md)

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

1. Fork the repository
2. Create a feature branch  
3. Run `make test-fast` for validation
4. Submit a pull request

## 📄 License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

---

**Built with ❤️ by [Pragmatic AI Labs](https://paiml.com)**