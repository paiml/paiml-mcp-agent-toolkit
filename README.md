# PMAT - Pragmatic AI Labs Multi-language Agent Toolkit

[![Documentation](https://img.shields.io/badge/docs-pmat--book-blue)](https://paiml.com/docs/pmat-book/)
[![Crates.io](https://img.shields.io/crates/v/pmat.svg)](https://crates.io/crates/pmat)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Zero-configuration AI context generation** for any codebase. Analyze code quality, complexity, and technical debt across 17+ programming languages with extreme quality enforcement and Toyota Way standards.

📖 **[Read the PMAT Book](https://paiml.com/docs/pmat-book/)** - Complete documentation, tutorials, and guides

---

## Quick Start

### Installation

```bash
# Rust (recommended)
cargo install pmat

# macOS/Linux
brew install pmat

# Windows
choco install pmat

# npm (global)
npm install -g pmat-agent
```

### Basic Usage

```bash
# Analyze codebase and generate AI-ready context
pmat context

# Analyze complexity
pmat analyze complexity

# Grade technical debt (A+ through F)
pmat analyze tdg

# Find Self-Admitted Technical Debt
pmat analyze satd
```

### Git Hooks Setup

Install pre-commit hooks for automatic quality enforcement:

```bash
# Install git hooks (bashrs quality, pmat-book validation)
pmat hooks install

# Check hook status
pmat hooks status

# Dry-run to see what would be checked
pmat hooks install --dry-run
```

**Hooks enforce:**
- Bash/Makefile safety (bashrs linting)
- pmat-book validation (multi-language examples)
- Documentation accuracy (zero hallucinations)

---

## Features

- **17+ Languages**: Rust, TypeScript, Python, Go, Java, C/C++, Ruby, PHP, Swift, Kotlin, and more
- **AI-Ready Context**: Generate deep context for Claude, GPT, and other LLMs
- **Technical Debt Grading (TDG)**: A+ through F scoring with 6 orthogonal metrics
- **Semantic Code Search**: Natural language code discovery with hybrid search
- **Quality Gates**: Pre-commit hooks, CI/CD integration, mutation testing
- **MCP Integration**: 19 tools for Claude Code, Cline, and other MCP clients
- **Zero Configuration**: Works out of the box on any codebase

---

## Documentation

📖 **[PMAT Book](https://paiml.com/docs/pmat-book/)** - Complete guide with tutorials

**Key chapters:**
- [Getting Started](https://paiml.com/docs/pmat-book/ch01-getting-started.html)
- [Multi-Language Support](https://paiml.com/docs/pmat-book/ch13-multi-language.html)
- [Technical Debt Grading](https://paiml.com/docs/pmat-book/ch07-tdg.html)
- [MCP Integration](https://paiml.com/docs/pmat-book/ch10-mcp.html)
- [Git Hooks](https://paiml.com/docs/pmat-book/ch14-git-hooks.html)

---

## Examples

```bash
# Generate context for Claude/GPT
pmat context --output context.md --format llm-optimized

# Analyze TypeScript project
pmat analyze complexity --language typescript

# Technical debt grading with components
pmat analyze tdg --include-components

# Semantic search (natural language)
pmat embed sync ./src
pmat semantic search "error handling patterns"

# Validate documentation for hallucinations
pmat validate-readme --targets README.md
```

---

## MCP Integration

PMAT provides 19 MCP tools for AI agents:

```bash
# Start MCP server
pmat mcp

# Use with Claude Code, Cline, or other MCP clients
```

**Tools include:** context generation, complexity analysis, TDG scoring, semantic search, code clustering, documentation validation, and more.

See [MCP Tools Documentation](docs/mcp/TOOLS.md) for details.

---

## Project Info

**Built by:** [Pragmatic AI Labs](https://paiml.com)  
**License:** MIT  
**Repository:** [github.com/paiml/paiml-mcp-agent-toolkit](https://github.com/paiml/paiml-mcp-agent-toolkit)  
**Issues:** [GitHub Issues](https://github.com/paiml/paiml-mcp-agent-toolkit/issues)

**Current Version:** v2.167.0 (Sprint 44 - Coverage Remediation)

---

## Contributing

See [ROADMAP.md](ROADMAP.md) for project status and future plans.

**Quality Standards:**
- EXTREME TDD (RED → GREEN → REFACTOR)
- 85%+ code coverage
- Five Whys root cause analysis
- Toyota Way principles (Jidoka, Genchi Genbutsu, Kaizen)
- Zero tolerance for defects
