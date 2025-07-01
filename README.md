# PAIML MCP Agent Toolkit (pmat)

A unified protocol implementation supporting CLI, MCP, and HTTP interfaces through a single binary architecture. Built with extreme quality standards and zero tolerance for technical debt.

## Installation

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

## Quick Start

### Analyze Code Quality
```bash
# Generate comprehensive context analysis
pmat context

# Find complexity hotspots
pmat analyze complexity --max-cyclomatic 10

# Detect technical debt
pmat analyze satd

# Find the file with most lint violations (EXTREME quality mode)
pmat analyze lint-hotspot
```

### Refactor Code
```bash
# AI-powered automatic refactoring to achieve zero defects
pmat refactor auto

# Clean up documentation following Zero Tolerance standards
pmat refactor docs --dry-run

# Interactive refactoring mode
pmat refactor interactive
```

### Quality Gates
```bash
# Run all quality checks
pmat quality-gate --fail-on-violation

# Enforce extreme quality standards
pmat enforce extreme
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

## Documentation

- [Architecture](docs/architecture/ARCHITECTURE.md)
- [Feature Documentation](docs/features/)
- [API Reference](rust-docs/cli-reference.md)
- [Contributing Guidelines](CONTRIBUTING.md)

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.