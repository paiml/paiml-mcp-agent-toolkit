# 📚 PMAT Features Documentation Index

## Overview
This index provides quick access to all PMAT feature documentation, including both established and recently documented capabilities.

## Core Features

### ✅ Quality Analysis & Enforcement
- [**TDG (Technical Debt Grading)**](./specifications/tdg-specification.md) - Comprehensive code quality scoring system
- [**Quality Gates**](./specifications/quality-gate-specification.md) - Enforced quality standards
- [**QDD (Quality-Driven Development)**](./specifications/qdd-tool-specification.md) - TDD with quality guarantees
- [**Complexity Analysis**](./specifications/complexity-specification.md) - Cyclomatic and cognitive complexity

### ✅ Automated Code Improvement
- [**🔧 Clippy Automation Guide**](./clippy-automatic-fixes-guide.md) - Comprehensive guide to automatic clippy fixes
- [**📊 Clippy Automation Summary**](./clippy-automation-summary.md) - Executive summary with metrics and ROI
- [**Refactoring Engine**](./specifications/refactoring-specification.md) - AI-powered code refactoring

### ✅ Build System Support
- [**🔨 Makefile Linting**](./makefile-linting-guide.md) - Comprehensive Makefile analysis and auto-fix
  - Rule-based analysis with 20+ configurable rules
  - Auto-fix capabilities for common issues
  - GNU Make compatibility checking
  - CI/CD integration with SARIF output

### ✅ Infrastructure & Operations
- [**💾 Memory Management**](./memory-management-guide.md) - Advanced memory optimization and monitoring
  - Real-time memory statistics and pressure detection
  - Pool management and optimization
  - Automatic cleanup and garbage collection
  - CI/CD memory profile configuration

### ✅ Code Generation & Scaffolding
- [**🤖 Agent Scaffolding**](./agent-scaffolding-guide.md) - Deterministic MCP agent generation
  - Multiple agent templates (MCP server, state machine, hybrid)
  - Quality enforcement from generation
  - Built-in testing and monitoring
  - Interactive and batch generation modes

### ✅ Integration Interfaces
- [**MCP Server**](./specifications/mcp-specification.md) - Model Context Protocol integration
- [**HTTP API**](./specifications/http-api-specification.md) - RESTful API with WebSocket support
- [**CLI Interface**](./specifications/cli-specification.md) - Command-line interface

## Language Support

### First-Class Languages
- **Rust** - Full AST analysis, all features supported
- **Ruchy** - Native support with TDG scoring and entropy analysis
- **TypeScript/JavaScript** - Comprehensive analysis via tree-sitter
- **Python** - Full support including type hints
- **Go** - Complete analysis including interfaces

### Experimental Support
- C/C++ - Basic analysis
- Java - Pattern detection
- Ruby - Syntax analysis

## Upcoming Features (Roadmap)

### Sprint 87: Enhanced Clippy Automation
- ML-based confidence scoring
- Pattern library with 100+ fixes
- Cross-project learning

### Sprint 90: Enhanced Build System Linting
- CMake and Bazel support
- Cross-build-system consistency
- AI-powered optimization suggestions

### Sprint 91: Markdown Documentation Linting
- Link checking and validation
- Code block compilation checks
- Style enforcement and spell check

### Sprint 92: Agent & AI Code Linting
- Claude Code specific patterns
- GitHub Copilot pattern detection
- Prompt engineering quality checks

## Quick Start Guides

### For New Users
1. [Installation Guide](../README.md#installation)
2. [Quick Start Tutorial](./quick-start.md)
3. [Configuration Guide](./configuration.md)

### For Developers
1. [Contributing Guide](../CONTRIBUTING.md)
2. [Architecture Overview](./architecture.md)
3. [API Reference](./api/)

### For CI/CD Integration
1. [GitHub Actions Examples](./ci-cd/github-actions.md)
2. [Pre-commit Hooks](./ci-cd/pre-commit.md)
3. [Docker Integration](./ci-cd/docker.md)

## Documentation Standards

All feature documentation follows these standards:
- **Comprehensive**: Cover all aspects of the feature
- **Examples**: Real-world usage examples
- **Integration**: Show MCP, CLI, and HTTP usage
- **Metrics**: Include performance and success metrics
- **Best Practices**: Provide guidance for optimal usage
- **Troubleshooting**: Common issues and solutions

## Recent Updates (2025-09-09)

### Newly Documented Features
- ✅ **Makefile Linting** - Full documentation created
- ✅ **Memory Management** - Comprehensive guide added
- ✅ **Agent Scaffolding** - Complete documentation
- ✅ **Clippy Automation** - Rich documentation with examples

### Roadmap Updates
- Added Sprint 90: Enhanced Build System Linting
- Added Sprint 91: Markdown Documentation Linting
- Added Sprint 92: Agent & AI Code Linting

## Support

- **GitHub Issues**: [Report bugs or request features](https://github.com/paiml/pmat/issues)
- **Documentation Issues**: [Report documentation problems](https://github.com/paiml/pmat/issues?label=documentation)
- **Community**: [Discord Server](https://discord.gg/pmat)

---

**Version**: 2.71.0 | **Last Updated**: 2025-09-09 | **Maintained by**: PMAT Team