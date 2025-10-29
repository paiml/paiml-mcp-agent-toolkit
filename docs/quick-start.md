# PMAT Quick Start

## Installation

### From Source

```bash
git clone https://github.com/paiml/paiml-mcp-agent-toolkit.git
cd paiml-mcp-agent-toolkit
cargo build --release
```

### Add to PATH

```bash
export PATH="$PATH:$(pwd)/target/release"
```

## Basic Usage

### Generate Deep Context

```bash
pmat context
```

### Analyze Complexity

```bash
pmat context --complexity-threshold 10
```

### Run Quality Gates

```bash
pmat quality-gate
```

## Next Steps

- See [CLI Reference](./cli-reference.md) for full command documentation
- Read [Features](./features/) for detailed feature documentation
- Check [Examples](../examples/) for practical examples

## Related Documentation

- [README](../README.md)
- [Documentation Index](./README.md)
