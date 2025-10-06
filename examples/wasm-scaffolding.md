# WASM Scaffolding Guide

Create WebAssembly projects with PMAT scaffolding.

## Quick Start

```bash
# WasmLabs project
pmat scaffold wasm --name my-wasm --framework wasm-labs

# PureWasm project
pmat scaffold wasm --name minimal-wasm --framework pure-wasm

# With features
pmat scaffold wasm --name quality-wasm \
  --framework wasm-labs \
  --features logging,metrics \
  --quality extreme

# Dry run
pmat scaffold wasm --name test --framework wasm-labs --dry-run
```

## Command Options

**Required:**
- `--name <NAME>`: Project name (kebab-case)
- `--framework <FRAMEWORK>`: wasm-labs or pure-wasm

**Optional:**
- `--features <FEATURES>`: logging, metrics, tracing
- `--quality <LEVEL>`: standard, strict, extreme
- `--output <PATH>`: Output directory
- `--force`: Overwrite existing
- `--dry-run`: Preview only

## Frameworks

### WasmLabs

Full-featured WASM for local development.

**Structure:**
```
my-wasm/
├── src/lib.rs
├── tests/
├── www/
│   ├── index.html
│   ├── index.js
│   └── styles.css
├── Cargo.toml
├── Makefile
└── .pmat-gates.toml
```

**Build:**
```bash
cd my-wasm
make wasm-full          # Build
python3 -m http.server 8000  # Serve
# Open: http://localhost:8000
```

### PureWasm

Minimal WASM for learning.

**Build:**
```bash
cd minimal-wasm
cargo build --target wasm32-unknown-unknown --release
cargo test
```

## Features

- **Logging**: Console logging for browser
- **Metrics**: Performance metrics
- **Tracing**: Execution tracing for debugging

## Quality Levels

- **Standard**: Coverage ≥60%, Basic tests
- **Strict**: Coverage ≥80%, Browser + headless tests
- **Extreme**: Coverage ≥85%, Mutation + property tests

## Local Development

```bash
# 1. Scaffold
pmat scaffold wasm --name my-app --framework wasm-labs
cd my-app

# 2. Build
make wasm-full

# 3. Serve
python3 -m http.server 8000

# 4. Develop
# Edit src/lib.rs → make wasm-full → refresh browser

# 5. Test
make wasm-test
make wasm-quality
```

## Examples

```bash
# Hello WASM
pmat scaffold wasm --name hello-wasm --framework wasm-labs

# Calculator
pmat scaffold wasm --name calc-wasm --framework wasm-labs \
  --features logging --quality strict

# Game engine
pmat scaffold wasm --name game-wasm --framework wasm-labs \
  --features logging,metrics,tracing --quality extreme
```

## Troubleshooting

**wasm-pack not found**: `cargo install wasm-pack`
**Build fails**: `rustup target add wasm32-unknown-unknown`
**Tests fail**: `wasm-pack test --headless --firefox`

## Deployment Note

Deployment (S3, CloudFront, CI/CD) is out of MVP scope.
Focus: Perfect local development with extreme quality.

## More Information

- [Quick Start Guide](./scaffolding-quickstart.md)
- [Examples README](./README.md)
