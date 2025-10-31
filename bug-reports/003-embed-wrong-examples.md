# Bug Report: `pmat embed` Shows Wrong Examples

**Date**: 2025-10-31
**Reporter**: User feedback
**Severity**: Low
**Component**: CLI - embed subcommand help text

## Description

When running `pmat embed` (without subcommand), the help text shows correct usage information but includes generic PMAT examples that have nothing to do with the embed functionality.

## Steps to Reproduce

```bash
pmat embed
```

## Actual Output

```
Manage semantic search embeddings (PMAT-SEARCH-011)

Usage: pmat embed [OPTIONS] <COMMAND>

Commands:
  sync    Sync embeddings for codebase
  status  Show embedding database status
  clear   Clear all embeddings (requires --confirm)
  help    Print this message or the help of the given subcommand(s)

Options:
      --mode <MODE>                  Force specific mode (auto-detected by default) [possible values: cli, mcp]
  -v, --verbose                      Enable verbose output (info level)
  -q, --quiet                        Enable quiet mode (errors only)
      --debug                        Enable debug output (debug level)
      --trace                        Enable trace output (trace level)
      --trace-filter <TRACE_FILTER>  Custom trace filter (overrides other flags) Example: --trace-filter="paiml=debug,cache=trace" [env: RUST_LOG=]
      --color <COLOR>                Control color output [default: auto] [possible values: auto, always, never]
  -h, --help                         Print help (see more with '--help')

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

## Expected Examples

Should show examples relevant to embedding functionality:

```
EXAMPLES:
# Sync embeddings for current codebase
pmat embed sync

# Check embedding database status
pmat embed status

# Clear all embeddings (requires confirmation)
pmat embed clear --confirm

# Sync with verbose output
pmat embed sync --verbose
```

## Analysis

- Help text correctly describes the embed subcommand and its options
- EXAMPLES section shows generic PMAT examples instead of embed-specific examples
- Likely using wrong template or shared examples constant

## Impact

- Low severity - doesn't break functionality
- Confusing for users trying to learn embed command usage
- Reduces discoverability of embed features

## Files to Investigate

- `server/src/cli/mod.rs` - CLI examples definition
- Clap derive macros for embed subcommand
- Shared examples constant or template
