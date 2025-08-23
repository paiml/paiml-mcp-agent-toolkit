# PMAT - Pragmatic AI MCP Agent Toolkit

[![npm version](https://badge.fury.io/js/pmat.svg)](https://badge.fury.io/js/pmat)
[![Downloads](https://img.shields.io/npm/dm/pmat.svg)](https://www.npmjs.com/package/pmat)

**Zero-configuration AI context generation system** with Claude Code Agent Mode for continuous quality monitoring.

## 🚀 Quick Start

```bash
# Install via npm
npm install -g pmat

# Or using npx (no installation)  
npx pmat --version

# Start Claude Code Agent Mode
pmat agent mcp-server
```

## 🤖 Claude Code Integration

Configure in your Claude Code settings:

```json
{
  "mcpServers": {
    "pmat": {
      "command": "pmat",
      "args": ["agent", "mcp-server"],
      "env": {}
    }
  }
}
```

## 📊 Core Features

- **Claude Code Agent Mode**: Continuous quality monitoring
- **AI Context Generation**: Optimized for LLM workflows  
- **Code Complexity Analysis**: McCabe cyclomatic & cognitive complexity
- **Technical Debt Detection**: SATD analysis with severity classification
- **Quality Gates**: Toyota Way ≤20 complexity enforcement
- **Multi-Language Support**: 30+ languages via tree-sitter

## 🛠️ Usage

```bash
# Generate AI context
pmat context

# Analyze complexity
pmat analyze complexity --top-files 10

# Run quality gates
pmat quality-gate --strict

# Start background agent
pmat agent start --project-path .

# MCP server for Claude Code
pmat agent mcp-server
```

## 📚 Documentation

- **[Complete Guide](https://github.com/paiml/paiml-mcp-agent-toolkit/blob/master/docs/CLAUDE_CODE_AGENT.md)**: Claude Code Agent setup
- **[GitHub Repository](https://github.com/paiml/paiml-mcp-agent-toolkit)**: Full source code and documentation
- **[Examples](https://github.com/paiml/paiml-mcp-agent-toolkit/tree/master/examples)**: Usage examples and integrations

## 🆘 Support

- **Issues**: [GitHub Issues](https://github.com/paiml/paiml-mcp-agent-toolkit/issues)
- **Discussions**: [GitHub Discussions](https://github.com/paiml/paiml-mcp-agent-toolkit/discussions)

## 📄 License

MIT License - See [LICENSE](https://github.com/paiml/paiml-mcp-agent-toolkit/blob/master/LICENSE) for details.

Built with ❤️ by [Pragmatic AI Labs](https://paiml.com)