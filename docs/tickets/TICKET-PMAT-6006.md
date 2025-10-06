# TICKET-PMAT-6006: UX Polish

**Sprint:** Sprint 20 - UX Improvements & Optimizations
**Priority:** P1 - High
**Estimated Effort:** 2-3 hours
**Status**: GREEN ✅
**Created:** 2025-10-06
**Completed:** 2025-10-06
**Commit:** 99fc664

## Problem Statement

Missing UX polish features for controlling output verbosity and color, making the tool less flexible for different environments (CI, piped output, etc.).

## Solution

Added comprehensive output control features:

### Quiet Mode
- `--quiet/-q` flag for errors-only output
- Global flag available on all commands
- Conflicts with `--verbose` flag
- Sets `PMAT_QUIET` environment variable

### Color Control
- `--color auto|always|never` flag
- Auto-detect based on TTY and environment
- Respects `NO_COLOR` and `CLICOLOR_FORCE`
- Sets appropriate environment variables

### Implementation

**File:** `server/src/cli/commands.rs`

**Changes:**
```rust
pub struct Cli {
    /// Enable quiet mode (errors only)
    #[arg(short, long, global = true, conflicts_with = "verbose")]
    pub quiet: bool,

    /// Control color output
    #[arg(long, global = true, value_enum, default_value = "auto")]
    pub color: ColorMode,
}

#[derive(Clone, Debug, clap::ValueEnum, PartialEq, Default)]
pub enum ColorMode {
    #[default]
    Auto,
    Always,
    Never,
}
```

**File:** `server/src/cli/mod.rs`

```rust
fn apply_ux_settings(cli: &commands::Cli) {
    if cli.quiet {
        std::env::set_var("PMAT_QUIET", "1");
    }

    match cli.color {
        commands::ColorMode::Never => {
            std::env::set_var("NO_COLOR", "1");
        }
        commands::ColorMode::Always => {
            std::env::set_var("CLICOLOR_FORCE", "1");
        }
        commands::ColorMode::Auto => {
            // Auto mode - respect existing environment
        }
    }
}
```

**File:** `server/src/cli/progress.rs`

Updated to respect `PMAT_QUIET`:
```rust
fn should_show_progress() -> bool {
    // Don't show in quiet mode (TICKET-PMAT-6006)
    if std::env::var("PMAT_QUIET").is_ok() {
        return false;
    }
    // ... other checks
}
```

## Usage Examples

```bash
# Quiet mode (errors only)
pmat scaffold agent --name test --quiet

# No color output
pmat maintain health --color never

# Force color even when piped
pmat maintain health --color always

# CI-friendly
CI=1 pmat quality-gates run
```

## Acceptance Criteria

- [x] `--quiet/-q` flag added
- [x] `--color auto|always|never` flag added
- [x] ColorMode enum created
- [x] Global flags work on all commands
- [x] Environment variables set correctly
- [x] Progress indicators respect quiet mode
- [x] Color respects NO_COLOR
- [x] Cyclomatic complexity <8
- [x] Test coverage >80%

## Quality Metrics

- **CC:** All functions <8
- **Coverage:** >80%
- **UX:** Flexible output control for all environments

---

**Status:** ✅ Complete
**Delivered:** v2.139.0
