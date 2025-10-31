# Bug Report: `pmat embed status` Shows Wrong Error Message

**Date**: 2025-10-31
**Reporter**: User feedback
**Severity**: Medium
**Component**: CLI - embed subcommand

## Description

When running `pmat embed status`, the command shows an error message that appears to belong to a different command. The error mentions `--format <FORMAT>` with value 'summary', but this doesn't match the embed status command's expected arguments.

## Steps to Reproduce

```bash
pmat embed status
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
1. Show the current embedding database status
2. Show a relevant error message specific to `pmat embed status`
3. Show correct examples related to the `embed` subcommand

## Analysis

- The error message references `--format <FORMAT>` with value 'summary', suggesting wrong default or argument parsing issue
- Examples shown are generic PMAT examples, not specific to `embed` subcommand
- Error appears to be from a different command's context

## Impact

- Users cannot check embedding database status
- Confusing error message makes debugging difficult
- Wrong examples provide no guidance for the embed command

## Files to Investigate

- `server/src/cli/mod.rs` - Main CLI definition
- `server/src/cli/handlers/embed.rs` or similar - Embed command handler
- Clap argument parsing for embed subcommand
