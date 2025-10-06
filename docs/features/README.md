# PMAT Features Documentation

*Feature implementation details aligned with [SPECIFICATION.md](../SPECIFICATION.md)*  
*Version 2.5.0 - Toyota Way Quality Standards*

This directory contains comprehensive documentation for all PMAT features, cross-referenced with the complete system specification.

## 📚 Feature Categories

### 🔧 Build & Quality Tools
- [**Makefile Linter**](./makefile-linter.md) - Automated Makefile quality analysis with 50+ rules
- [**Excellence Tracker**](./excellence-tracker.md) - Code quality metrics and tracking system
- [**Documentation Enforcement**](./documentation-enforcement.md) - ✨ NEW v2.142.0 - Automated CLI/MCP documentation quality validation ⭐

### 🚀 Refactoring & Optimization
- [**Refactor Auto**](./refactor-auto.md) - AI-powered automated refactoring with extreme quality standards ⭐
- [**Single File Mode**](./single-file-mode.md) - Targeted quality improvements following Toyota Way principles ⭐
- [**Emit-Refactor Engine**](./emit-refactor-engine.md) - Real-time defect emission and interactive refactoring
### 📊 Analysis Tools
- [**Deep Context Analysis**](./deep-context-analysis.md) - Comprehensive codebase analysis with AST
- [**Technical Debt Gradient (TDG)**](./technical-debt-gradient.md) - Quantitative technical debt measurement
- [**SATD Detection**](./satd-detection.md) - Self-Admitted Technical Debt identification
<!-- TODO: Add dedicated documentation files for Complexity Analysis -->


### 🌐 Protocol Support
- [**MCP Protocol**](./mcp-protocol.md) - Model Context Protocol implementation
- [**HTTP API**](../../rust-docs/http-api.md) - RESTful API interface
- [**CLI Interface**](../../rust-docs/cli-reference.md) - Command-line interface reference

### 📈 Visualization & Reporting
<!-- TODO: Add dedicated documentation for Mermaid Diagram Generation -->
<!-- TODO: Add dedicated documentation for DAG Visualization -->
<!-- TODO: Add dedicated documentation for Demo Mode -->

### 🏗️ Project Management
<!-- TODO: Add dedicated documentation for Scaffolding -->
<!-- TODO: Add dedicated documentation for Git Integration -->

### 🌐 Language Support
- **Rust** - Complete AST analysis with syn
- **TypeScript/JavaScript** - Full parsing via SWC
- **Python** - AST analysis with rustpython-parser
- **C/C++** - Tree-sitter based parsing with goto tracking
- **Kotlin** - Full AST support via tree-sitter-kotlin (with memory safety guarantees)
- **WebAssembly** - Binary (.wasm) and text format (.wat) analysis (v0.26.2)
- **AssemblyScript** - TypeScript-like syntax for WebAssembly (v0.26.2)
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
| Refactor Auto | ✅ Stable | 0.26.3 | AI-powered automated refactoring |
| Single File Mode | ✅ Stable | 0.26.3 | Targeted incremental improvements |
| Makefile Linter | ✅ Stable | 0.26.1 | 50+ rules for Makefile quality |
| Emit-Refactor Engine | ✅ Stable | 0.26.1 | Dual-mode refactoring system |
| Deep Context | ✅ Stable | 0.26.0 | AST-based analysis |
| TDG Calculator | ✅ Stable | 0.26.0 | Technical debt metrics |
| Provability Analysis | ✅ Stable | 0.26.1 | Formal verification |
| MCP Protocol | ✅ Stable | 0.26.0 | AI agent integration |
| Kotlin AST Parser | ✅ Stable | 0.26.0 | Full AST with memory safety |
| Memory Safety | ✅ Stable | 0.26.0 | System stability guarantees |
| WebAssembly Support | ✅ Stable | 0.26.2 | WASM and AssemblyScript analysis |

## 🔗 Related Documentation

- [**Architecture Overview**](../architecture/ARCHITECTURE.md)
- [**Contributing Guide**](../../CONTRIBUTING.md)
- [**API Reference**](../api-guide.md)
- [Examples](../examples/)