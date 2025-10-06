# TICKET-PMAT-5034: Wire Up `pmat hooks` CLI Command

**Status**: GREEN
**Priority**: P1
**Complexity**: 2
**Estimated Time**: 20 minutes
**Dependencies**: Existing hooks infrastructure, TICKET-PMAT-5033
**Sprint**: Sprint 19 - CLI Integration & Dogfooding

## Objective

Wire up the existing `pmat hooks` command infrastructure to make it accessible via CLI. The hooks command handlers and subcommands already exist (install, uninstall, status, verify, refresh), they just need to be connected to the command dispatcher and structure.

## Success Criteria

- [ ] `pmat hooks install` installs pre-commit hooks
- [ ] `pmat hooks uninstall` removes hooks
- [ ] `pmat hooks status` shows hook status
- [ ] `pmat hooks verify` verifies hooks work
- [ ] `pmat hooks refresh` regenerates hooks
- [ ] All quality gates pass (complexity <10, coverage >80%, no SATD)

## Current State

**Already Exists:**
- `Commands::Hooks(HooksCommands)` variant in commands.rs:780
- `HooksCommands` enum with all subcommands (commands.rs:3528)
- `HooksCommand` implementation in hooks_command_handlers.rs
- `handle_hooks_command()` function already exported

**Missing:**
- Wire-up in command_structure.rs
- Wire-up in command_dispatcher.rs
- Add to unified_protocol/adapters/cli.rs

## Test Strategy

### Manual Testing
- [ ] `pmat hooks install` - Installs pre-commit hook
- [ ] `pmat hooks status` - Shows installed status
- [ ] `pmat hooks verify` - Verifies hook works
- [ ] `pmat hooks uninstall` - Removes hook
- [ ] `pmat hooks refresh` - Regenerates hook

## Quality Gates

- [ ] Cyclomatic complexity <10 for all functions
- [ ] 0 clippy warnings
- [ ] All tests pass

## Implementation Plan

### Phase 1: Wire Up in command_structure.rs

```rust
// server/src/cli/command_structure.rs

Commands::Hooks(hooks_cmd) => {
    super::handlers::handle_hooks_command(hooks_cmd).await
}
```

### Phase 2: Wire Up in command_dispatcher.rs

```rust
// server/src/cli/command_dispatcher.rs

Commands::Hooks(hooks_cmd) => {
    handlers::handle_hooks_command(hooks_cmd).await
}
```

### Phase 3: Add to unified_protocol/adapters/cli.rs

Add to the CLI-only commands list:

```rust
Commands::Hooks(_) => Self::cli_only_command_error(),
```

And add to command category:

```rust
Commands::Hooks(_) => CommandCategory::Workflow,
```

## Complexity Analysis

This is a simple wire-up task with no new functions, just routing existing handlers:
- No new complexity introduced
- All existing functions already under CC=10

All functions under CC=10 threshold ✓

## Verification Commands

```bash
# Install hooks
pmat hooks install

# Check status
pmat hooks status

# Verify hooks work
pmat hooks verify

# Refresh hooks
pmat hooks refresh

# Uninstall
pmat hooks uninstall
```

## Files to Modify

### Modified Files
- `server/src/cli/command_structure.rs` - Add Hooks routing
- `server/src/cli/command_dispatcher.rs` - Add Hooks dispatcher
- `server/src/unified_protocol/adapters/cli.rs` - Add Hooks to CLI-only commands

## Risk Assessment

**Very Low Risk:**
- No new code, just routing
- Handlers already implemented and tested
- Infrastructure mature and stable

**Mitigation:**
- Simple pattern match additions
- Follows same pattern as other commands
- Existing tests verify handler functionality

## Notes

This ticket is straightforward because the hooks infrastructure was already built in Sprint 80 (Pre-commit Hook Management). We're just exposing it via the top-level CLI.

**Existing Infrastructure:**
- `HooksCommand` struct with install/uninstall/status/verify/refresh
- Hook generation from templates
- Backup/restore functionality
- PMAT marker detection
- Force overwrite support

**Value:**
- Developers can install Git hooks easily
- Pre-commit quality gates enforcement
- Consistent hook management across team
- Integration with PMAT quality system

**Integration:**
- Works with `pmat quality-gates` for enforcement
- Complements `pmat maintain health` for checks
- Part of complete quality workflow

**TDD Cycle Duration**: Estimated 20 minutes for wire-up and verification
