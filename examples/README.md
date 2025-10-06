# PMAT Examples

This directory contains examples and guides for using PMAT's scaffolding commands.

## Quick Links

- [Agent Scaffolding Guide](./agent-scaffolding.md) - Create MCP agents
- [WASM Scaffolding Guide](./wasm-scaffolding.md) - Create WASM projects
- [Quick Start Guide](./scaffolding-quickstart.md) - Get started in 5 minutes

## Available Scaffolding Commands

### 1. Agent Scaffolding

Create Model Context Protocol (MCP) agents with built-in quality gates.

```bash
pmat scaffold agent --name my_agent --template basic
```

**Features:**
- MCP server setup
- State machine patterns
- Quality gates pre-configured
- Testing infrastructure
- Multiple templates available

**Templates:**
- `basic` - Simple MCP server
- `stateful` - Server with state management
- `hybrid` - Deterministic core + LLM wrapper

### 2. WASM Scaffolding

Create WebAssembly projects optimized for local development.

```bash
pmat scaffold wasm --name my_wasm --framework wasm-labs
```

**Features:**
- WasmLabs or PureWasm frameworks
- Local development setup
- Quality gates (85%+ coverage, mutation testing)
- Makefile automation
- localhost:8000 testing

**Frameworks:**
- `wasm-labs` - Full-featured WASM development
- `pure-wasm` - Minimal WASM setup

## Project Structure

All scaffolded projects follow PMAT quality standards:
- ✅ Extreme TDD (85%+ coverage)
- ✅ Complexity <10
- ✅ Mutation testing
- ✅ Property-based testing
- ✅ Quality gate enforcement

## Next Steps

1. Choose your project type (agent or WASM)
2. Read the relevant guide
3. Run the scaffold command
4. Follow the generated README
5. Start building!

## Support

- [PMAT Documentation](../README.md)
- [Quality Gates](../docs/quality-gates.md)
- [Dogfooding Report](../docs/dogfooding/SPRINT-19-FINDINGS.md)
