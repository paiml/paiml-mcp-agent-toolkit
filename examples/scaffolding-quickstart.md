# Scaffolding Quick Start

Get started with PMAT scaffolding in 5 minutes.

## Agent in 60 Seconds

```bash
# Create agent
pmat scaffold agent --name quick_agent --template basic

# Build and test
cd quick_agent
cargo build
cargo test

# Quality check
pmat quality-gates

# Done! 🎉
```

## WASM in 60 Seconds

```bash
# Create WASM project
pmat scaffold wasm --name quick_wasm --framework wasm-labs

# Build
cd quick_wasm
make wasm-full

# Serve
python3 -m http.server 8000

# Open browser: http://localhost:8000
# Done! 🎉
```

## Choose Your Adventure

### I want to build an MCP agent

```bash
pmat scaffold agent --name my_agent --template basic
```

→ [Full Agent Guide](./agent-scaffolding.md)

### I want to build a WASM app

```bash
pmat scaffold wasm --name my_wasm --framework wasm-labs
```

→ [Full WASM Guide](./wasm-scaffolding.md)

### I want to see what would be created

```bash
# Agent dry run
pmat scaffold agent --name test --template basic --dry-run

# WASM dry run
pmat scaffold wasm --name test --framework wasm-labs --dry-run
```

## Common Workflows

### Prototype Fast (Standard Quality)

```bash
pmat scaffold agent --name prototype --template basic --quality standard
cd prototype
cargo run
```

### Production Ready (Strict Quality)

```bash
pmat scaffold agent --name production --template stateful --quality strict
cd production
cargo build --release
pmat quality-gates
```

### Mission Critical (Extreme Quality)

```bash
pmat scaffold agent \
  --name critical \
  --template hybrid \
  --features logging,metrics,tracing \
  --quality extreme

cd critical
cargo test
pmat quality-gates
```

## Next Steps

1. **Scaffold your project** using commands above
2. **Read the generated README.md** in your project
3. **Start coding** in src/
4. **Run tests frequently** with `cargo test`
5. **Check quality gates** with `pmat quality-gates`
6. **Install hooks** with `pmat hooks install` for automatic checks

## Tips

- Use `--dry-run` to preview before creating
- Use `--force` to overwrite existing projects
- Start with `basic` template, upgrade later
- Run `pmat quality-gates` often
- Install hooks early: `pmat hooks install`
- Follow generated README.md for specifics

## Help

```bash
# Agent help
pmat scaffold agent --help

# WASM help
pmat scaffold wasm --help

# List templates
pmat scaffold list-templates
```

## Full Guides

- [Agent Scaffolding Guide](./agent-scaffolding.md)
- [WASM Scaffolding Guide](./wasm-scaffolding.md)
- [Main Examples README](./README.md)
