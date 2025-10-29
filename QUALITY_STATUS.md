# PMAT Quality Status

## Test Coverage

- Target: 85%+
- Measured via: `cargo llvm-cov`

## Quality Gates

All quality gates must pass before release:

```bash
make validate
```

## Current Status

For detailed quality metrics, run:

```bash
make coverage
make lint
make test
```

## Related Documentation

- [ROADMAP.md](./ROADMAP.md)
- [CONTRIBUTING.md](./CONTRIBUTING.md)
