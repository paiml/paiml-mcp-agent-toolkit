# GitHub Actions Integration

## Overview

Integrate PMAT quality gates into GitHub Actions workflows.

## Example Workflow

```yaml
name: Quality Gates
on: [push, pull_request]
jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install PMAT
        run: cargo install pmat
      - name: Run Quality Gates
        run: pmat quality-gate
```

## Related Documentation

- [Quality Gates](../features/QUALITY_GATES.md)
- [CI/CD Integration](./pre-commit.md)
