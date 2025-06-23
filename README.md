# PAIML MCP Agent Toolkit

[![CI/CD](https://github.com/paiml/paiml-mcp-agent-toolkit/actions/workflows/main.yml/badge.svg?branch=master)](https://github.com/paiml/paiml-mcp-agent-toolkit/actions/workflows/main.yml) [![MCP Compatible](https://img.shields.io/badge/MCP-Compatible-green)](https://modelcontextprotocol.io) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Zero-configuration AI context generation system** that analyzes any codebase instantly through CLI, MCP, or HTTP interfaces. Built by [Pragmatic AI Labs](https://paiml.com).

## 🚀 Installation

```bash
curl -sSfL https://raw.githubusercontent.com/paiml/paiml-mcp-agent-toolkit/master/scripts/install.sh | sh
```

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

# Demo and visualization
pmat demo --format table                      # CLI demo
pmat demo --web --port 8080                   # Web interface
pmat demo --repo https://github.com/user/repo # Analyze GitHub repo
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

## 🔧 Supported Languages

- **Rust** - Complete AST analysis, complexity metrics
- **TypeScript/JavaScript** - Full parsing and analysis
- **Python** - AST analysis and code metrics  
- **C/C++** - Goto tracking, macro analysis, memory safety indicators
- **Cython** - Hybrid Python/C analysis
- **Kotlin** - Full AST parsing for classes, functions, interfaces, and more

## 📚 Documentation

### Feature Documentation

- **[Feature Overview](docs/features/README.md)** - Complete feature index
- **[Makefile Linter](docs/features/makefile-linter.md)** - 50+ rules for Makefile quality
- **[Emit-Refactor Engine](docs/features/emit-refactor-engine.md)** - Real-time defect detection & refactoring
- **[Excellence Tracker](docs/features/excellence-tracker.md)** - Code quality metrics tracking
- **[Technical Debt Gradient](docs/features/technical-debt-gradient.md)** - Quantitative debt measurement
- **[MCP Protocol](docs/features/mcp-protocol.md)** - AI agent integration guide

### Additional Features

- **Code Quality Tools**
  - `pmat analyze makefile-lint` - Lint Makefiles with 50+ quality rules
  - `pmat excellence-tracker` - Track code quality metrics over time
  - `pmat refactor serve` - Batch refactoring with checkpoints
  - `pmat refactor interactive` - Interactive refactoring with explanations

- **Advanced Analysis**
  - `pmat analyze tdg` - Calculate Technical Debt Gradient
  - `pmat analyze proof-annotations` - Lightweight formal verification
  - `pmat analyze defect-prediction` - ML-based defect prediction
  - `pmat analyze name-similarity` - Semantic name search with embeddings
  - `pmat analyze big-o` - Big-O complexity with confidence scores
  - `pmat analyze graph-metrics` - PageRank and centrality metrics
  - `pmat analyze incremental-coverage` - Coverage changes since base branch

## 📊 Output Formats

- **JSON** - Structured data for tools and APIs
- **Markdown** - Human-readable reports
- **SARIF** - Static analysis format for IDEs
- **Mermaid** - Dependency graphs and diagrams

## 🌐 Language Support

- **Rust** - Complete AST analysis with native syn parser
- **TypeScript/JavaScript** - Full parsing support via SWC
- **Python** - AST analysis with rustpython-parser
- **C/C++** - Tree-sitter based parsing with comprehensive coverage
- **Kotlin** - Full AST support via tree-sitter-kotlin (v0.26.0)
- **Go, Java, Ruby** - Basic support via unified AST

All parsers include memory safety protections and iterative processing to handle large codebases efficiently.

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

- [CLI Reference](rust-docs/cli-reference.md)
- [MCP Protocol](rust-docs/mcp-protocol.md) 
- [HTTP API](rust-docs/http-api.md)
- [Architecture](rust-docs/architecture.md)
- [Distributed Testing](docs/features/distributed-testing.md)

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

MIT License - see LICENSE file for details.

---

**Built with ❤️ by [Pragmatic AI Labs](https://paiml.com)**
