# Agent Scaffolding Guide

Create Model Context Protocol (MCP) agents with PMAT scaffolding.

## Quick Start

```bash
# Basic agent
pmat scaffold agent --name my_agent --template basic

# Stateful agent with features
pmat scaffold agent --name smart_agent \
  --template stateful \
  --features logging,metrics \
  --quality extreme

# Dry run to preview
pmat scaffold agent --name test_agent --template basic --dry-run
```

## Command Options

**Required:**
- `--name <NAME>`: Agent name (kebab-case recommended)
- `--template <TEMPLATE>`: Template type (basic, stateful, hybrid)

**Optional:**
- `--features <FEATURES>`: Comma-separated (logging, metrics, tracing)
- `--quality <LEVEL>`: standard, strict, extreme (default: strict)
- `--output <PATH>`: Output directory
- `--force`: Overwrite existing
- `--dry-run`: Preview only

## Templates

### Basic Agent

Simple MCP server for straightforward use cases.

**Structure:**
```
my_agent/
├── src/
│   ├── main.rs
│   ├── server.rs
│   ├── handlers.rs
│   └── lib.rs
├── tests/
├── Cargo.toml
├── README.md
└── .pmat-gates.toml
```

**Next Steps:**
```bash
cd my_agent
cargo build && cargo test
pmat quality-gates
```

### Stateful Agent

Agent with state management.

**Additional:**
- State machine patterns
- Persistence layer
- Event sourcing
- Transaction support

### Hybrid Agent

Deterministic core + LLM wrapper.

**Architecture:**
- Deterministic core (property-tested)
- LLM wrapper (probabilistic)
- Fallback to deterministic
- Confidence thresholds

## Features

### Logging
Structured logging with `tracing` crate.

### Metrics
Metrics collection with Prometheus exporter.

### Tracing
Distributed tracing with OpenTelemetry.

## Quality Levels

- **Standard**: Coverage ≥60%, Complexity ≤15
- **Strict** (default): Coverage ≥80%, Complexity ≤10
- **Extreme**: Coverage ≥85%, Mutation testing, Property tests

## Examples

```bash
# Simple echo agent
pmat scaffold agent --name echo_agent --template basic

# File system agent
pmat scaffold agent --name fs_agent --template stateful \
  --features logging,metrics

# Code assistant
pmat scaffold agent --name code_assistant --template hybrid \
  --features logging,metrics,tracing --quality extreme
```

## Workflow

1. **Scaffold**: `pmat scaffold agent --name my_agent --template basic`
2. **Build**: `cd my_agent && cargo build`
3. **Test**: `cargo test`
4. **Quality**: `pmat quality-gates`
5. **Develop**: Add your agent logic
6. **Iterate**: Test, refactor, improve

## Troubleshooting

**Directory exists**: Use `--force` or different `--output`
**Unknown template**: Run `pmat scaffold list-templates`

## More Information

- [Quick Start Guide](./scaffolding-quickstart.md)
- [Examples README](./README.md)
