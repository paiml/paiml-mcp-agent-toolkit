# Git Hooks Management

## Overview

PMAT uses git hooks to enforce quality standards automatically.

## Pre-commit Hook

Enforces:
- bashrs linting for shell scripts
- pmat-book validation
- Documentation accuracy

## Pre-push Hook

Enforces:
- pmat-book synchronization (book must be pushed before code)

## Installation

Hooks are automatically installed on first commit.

## Manual Installation

```bash
cp .git/hooks/pre-commit.sample .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

## Related Documentation

- [CI/CD Pre-commit](./ci-cd/pre-commit.md)
- [Operations Hooks](./operations/hooks.md)
- [CLAUDE.md Hook Policies](../CLAUDE.md)
