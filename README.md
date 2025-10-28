# PMAT - Pragmatic AI Labs Multi-language Agent Toolkit

[![Documentation](https://img.shields.io/badge/docs-pmat--book-blue)](https://paiml.github.io/pmat-book/)
[![Crates.io](https://img.shields.io/crates/v/pmat.svg)](https://crates.io/crates/pmat)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Zero-configuration AI context generation** for any codebase. Analyze code quality, complexity, and technical debt across 17+ programming languages with extreme quality enforcement and Toyota Way standards.

📖 **[https://paiml.github.io/pmat-book/](https://paiml.github.io/pmat-book/)** - Complete documentation, tutorials, and guides

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

# Test suite quality (mutation testing)
pmat mutate --target src/
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
- **Git-Commit Correlation** ✨NEW: Track TDG scores at specific commits for quality archaeology
- **Semantic Code Search**: Natural language code discovery with hybrid search
- **Quality Gates**: Pre-commit hooks, CI/CD integration, mutation testing
- **MCP Integration**: 19 tools for Claude Code, Cline, and other MCP clients
- **Zero Configuration**: Works out of the box on any codebase

---

## Documentation

📖 **[PMAT Book](https://paiml.github.io/pmat-book/)** - Complete guide with tutorials

**Key chapters:**
- [Installation and Setup](https://paiml.github.io/pmat-book/ch01-00-installation.html)
- [Getting Started](https://paiml.github.io/pmat-book/ch02-00-getting-started.html)
- [MCP Protocol](https://paiml.github.io/pmat-book/ch03-00-mcp-protocol.html)
- [Technical Debt Grading](https://paiml.github.io/pmat-book/ch04-01-tdg.html)
- [Multi-Language Examples](https://paiml.github.io/pmat-book/ch13-00-language-examples.html)

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

## Mutation Testing

Evaluate test suite quality by introducing code mutations and checking if tests detect them.

```bash
# Basic mutation testing
pmat mutate --target src/lib.rs

# With quality gate (fail if score < 85%)
pmat mutate --target src/ --threshold 85

# Failures only (CI/CD optimization)
pmat mutate --target src/ --failures-only

# JSON output for integration
pmat mutate --target src/ --output-format json > results.json
```

**Mutation Score** = (Killed Mutants / Total Valid Mutants) × 100%

**Supported Languages:** Rust, Python, TypeScript, JavaScript, Go, C++

**Key Features:**
- Multi-language mutation operators (arithmetic, comparison, logical, boundary)
- CI/CD integration (GitHub Actions, GitLab CI, Jenkins)
- Performance optimization (parallel execution, differential testing)
- Quality gates with configurable thresholds

**Documentation:**
- [User Guide](docs/guides/mutation-testing.md) - Complete guide with examples
- [API Reference](docs/guides/mutation-testing-api-reference.md) - All CLI flags and options
- [Best Practices](docs/guides/mutation-testing-best-practices.md) - Team adoption strategies
- [CI/CD Integration](docs/ci-cd/) - GitHub Actions, GitLab CI, Jenkins

**Example Projects:**
- [Rust Example](examples/rust-mutation-testing/) - 8 functions, 8 tests, ~90% mutation score
- [Python Example](examples/python-mutation-testing/) - 8 functions, 24 tests, comprehensive coverage
- [TypeScript Example](examples/typescript-mutation-testing/) - 8 functions, 24 tests, Jest integration

---

## Git-Commit Correlation (v2.179.0+)

Track Technical Debt Grading (TDG) scores at specific git commits for "quality archaeology" workflows.

```bash
# Analyze with git context
pmat tdg server/src/lib.rs --with-git-context

# Query specific commit
pmat tdg history --commit v2.178.0

# History since reference
pmat tdg history --since HEAD~10

# Commit range
pmat tdg history --range v2.177.0..v2.178.0

# Filter by file
pmat tdg history --path server/src/lib.rs --since HEAD~5

# JSON output for scripting
pmat tdg history --commit 60125a0 --format json
```

**Use Cases:**
- **Quality Archaeology**: Find which commit broke quality
- **Release Tracking**: Compare quality between releases
- **Regression Detection**: Identify quality drops over time
- **Developer Metrics**: Track quality attribution

**Example Workflows:**

```bash
# Find when quality dropped below B+
pmat tdg history --since HEAD~50 --format json | \
  jq '.history[] | select(.score.grade | test("C|D|F"))'

# Quality delta between releases
pmat tdg history --range v2.177.0..v2.178.0

# Per-file quality trend
pmat tdg history --path src/lib.rs --since HEAD~20
```

**Features:**
- Tag resolution support (e.g., `--commit v2.178.0`)
- MCP integration (`with_git_context: true` parameter)
- Zero performance overhead (<1% analysis time)
- Backward compatible (git context is optional)

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
