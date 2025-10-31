# Bug Report: `pmat embed status` Shows Wrong Error Message

**Date**: 2025-10-31
**Reporter**: User feedback
**Severity**: Medium → ✅ FIXED
**Component**: CLI - embed subcommand
**Status**: GREEN phase complete (default value fixed)

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

## Fix Applied

**Root Cause**: `default_value = "summary"` but OutputFormat enum only has `Table`, `Json`, `Yaml` (no `Summary` variant)

**Solution**: Changed default value from "summary" to "table"

**Files Modified**:
- `server/src/cli/commands.rs:4002` - Changed Status default from "summary" to "table"
- `server/tests/bug_001_002_003_embed_tests.rs` - 7 comprehensive tests (BUG-001, BUG-002, BUG-003 combined)
- `bug-reports/001-embed-status-wrong-error.md` - Updated to FIXED

**TDD Approach**:
1. ✅ RED: 3 tests for embed status command (test 1, 3, part of 6)
2. ✅ GREEN: Changed `default_value = "table"`
3. ✅ Verification: Code compiles without errors

**Implementation Details**:
```rust
// Before (broken):
Status {
    #[arg(long, value_enum, default_value = "summary")]  // ❌ "summary" doesn't exist!
    format: OutputFormat,
},

// After (fixed):
Status {
    #[arg(long, value_enum, default_value = "table")]  // ✅ Valid variant
    format: OutputFormat,
},
```

**Impact**: `pmat embed status` now works with default arguments, no more confusing "invalid value 'summary'" error.
