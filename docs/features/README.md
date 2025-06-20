# PAIML MCP Agent Toolkit - Feature Documentation

This directory contains comprehensive documentation for all features of the PAIML MCP Agent Toolkit.

## 📚 Feature Categories

### 🔧 Build & Quality Tools
- [**Makefile Linter**](./makefile-linter.md) - Automated Makefile quality analysis with 50+ rules
- [**Excellence Tracker**](./excellence-tracker.md) - Code quality metrics and tracking system

### 🚀 Refactoring & Optimization
- [**Emit-Refactor Engine**](./emit-refactor-engine.md) - Real-time defect emission and interactive refactoring
- [**Complexity Analysis**](./complexity-analysis.md) - Cyclomatic and cognitive complexity metrics

### 📊 Analysis Tools
- [**Deep Context Analysis**](./deep-context-analysis.md) - Comprehensive codebase analysis with AST
- [**Technical Debt Gradient (TDG)**](./technical-debt-gradient.md) - Quantitative technical debt measurement
- [**SATD Detection**](./satd-detection.md) - Self-Admitted Technical Debt identification
- [**Dead Code Analysis**](./dead-code-analysis.md) - Unused code detection and ranking
- [**Provability Analysis**](./provability-analysis.md) - Lightweight formal verification

### 🌐 Protocol Support
- [**MCP Protocol**](./mcp-protocol.md) - Model Context Protocol implementation
- [**HTTP API**](./http-api.md) - RESTful API interface
- [**CLI Interface**](./cli-interface.md) - Command-line interface reference

### 📈 Visualization & Reporting
- [**Mermaid Diagram Generation**](./mermaid-generation.md) - Automatic diagram creation
- [**DAG Visualization**](./dag-visualization.md) - Dependency graph analysis
- [**Demo Mode**](./demo-mode.md) - Interactive demonstrations

### 🏗️ Project Management
- [**Scaffolding**](./scaffolding.md) - Project template generation
- [**Git Integration**](./git-integration.md) - Repository analysis and cloning

### 🌐 Language Support
- **Rust** - Complete AST analysis with syn
- **TypeScript/JavaScript** - Full parsing via SWC
- **Python** - AST analysis with rustpython-parser
- **C/C++** - Tree-sitter based parsing with goto tracking
- **Kotlin** - Full AST support via tree-sitter-kotlin (with memory safety guarantees)
- **Cython** - Hybrid Python/C analysis

#### 🛡️ Memory Safety (v0.26.0)
All language parsers now include comprehensive memory safety protections:
- **Bounded parsing**: Maximum nodes, time limits, and file size restrictions
- **Iterative processing**: Prevents stack overflow in large codebases
- **Toyota Way methodology**: Five Whys root cause analysis for reliability

## 🚀 Quick Start

Each feature document includes:
- Overview and purpose
- Installation/setup (if needed)
- Usage examples
- Configuration options
- API reference
- Best practices
- Troubleshooting

## 📋 Index

| Feature | Status | Version | Description |
|---------|--------|---------|-------------|
| Makefile Linter | ✅ Stable | 0.25.0 | 50+ rules for Makefile quality |
| Emit-Refactor Engine | ✅ Stable | 0.25.0 | Dual-mode refactoring system |
| Deep Context | ✅ Stable | 0.25.0 | AST-based analysis |
| TDG Calculator | ✅ Stable | 0.25.0 | Technical debt metrics |
| Provability Analysis | ✅ Stable | 0.25.0 | Formal verification |
| MCP Protocol | ✅ Stable | 0.25.0 | AI agent integration |
| Kotlin AST Parser | ✅ Stable | 0.26.0 | Full AST with memory safety |
| Memory Safety | ✅ Stable | 0.26.0 | System stability guarantees |

## 🔗 Related Documentation

- [Architecture Overview](../ARCHITECTURE.md)
- [Contributing Guide](../CONTRIBUTING.md)
- [API Reference](../API.md)
- [Examples](../examples/)