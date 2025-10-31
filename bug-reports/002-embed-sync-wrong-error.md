# Bug Report: `pmat embed sync` Shows Wrong Error Message

**Date**: 2025-10-31
**Reporter**: User feedback
**Severity**: Medium
**Component**: CLI - embed subcommand

## Description

When running `pmat embed sync`, the command shows the same incorrect error message as `pmat embed status`. The error mentions `--format <FORMAT>` with value 'summary' and shows generic PMAT examples unrelated to the embed sync functionality.

## Steps to Reproduce

```bash
pmat embed sync
```

## Actual Output

```
error: invalid value 'summary' for '--format <FORMAT>'
  [possible values: table, json, yaml]

For more information, try '--help'.

EXAMPLES:
# Analyze code complexity
pmat analyze complexity --project-path .

# Find technical debt
pmat analyze satd --path .

# Find dead code
pmat analyze dead-code --path .

# Generate project context
pmat context

# Run quality gates
pmat quality-gate --strict

# Start agent daemon
pmat agent start
```

## Expected Behavior

The command should either:
1. Sync embeddings for the current codebase
2. Show a relevant error message specific to `pmat embed sync`
3. Show correct examples related to the `embed sync` command

## Analysis

- Same error as `pmat embed status` (see bug report #001)
- Suggests shared code path or initialization issue affecting all embed subcommands
- Error message and examples are completely unrelated to embed functionality

## Impact

- Users cannot sync embeddings for codebase
- Blocks semantic search functionality (PMAT-SEARCH-011)
- Confusing error message provides no actionable guidance

## Related Issues

- Bug #001: `pmat embed status` shows wrong error message

## Files to Investigate

- `server/src/cli/mod.rs` - Main CLI definition
- `server/src/cli/handlers/embed.rs` or similar - Embed command handler
- Shared argument parsing logic for embed subcommands
