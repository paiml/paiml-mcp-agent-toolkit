# Bug Report: Incorrect Parallel Analysis Count and Typo

**Date**: 2025-10-31
**Reporter**: User feedback
**Severity**: Low
**Component**: CLI - context command parallel analysis

## Description

When running `pmat context`, two issues occur:
1. **Typo**: Shows "parallel analyses" instead of "parallel analysis" (or should be plural "analyses" everywhere)
2. **Wrong count**: Shows 8 parallel analyses but only 4 actually run

## Steps to Reproduce

```bash
pmat context
```

## Actual Output

```
⠙ Running parallel analyses...
  Running analyses [███████████▎                  ] 3/8
```

But only 4 analyses actually execute.

## Expected Behavior

Should show:
```
⠙ Running parallel analyses...
  Running analyses [████████████████████████████] 4/4
```

Or if it's meant to show 8, then all 8 should actually run.

## Analysis

Two separate issues:

### 1. Typo Issue
- "analyses" is technically correct (plural of "analysis")
- But message may say "parallel analyses" in one place and "parallel analysis" in another
- Should be consistent throughout

### 2. Count Mismatch
Possible causes:
- Hardcoded total count (8) doesn't match actual analyses spawned (4)
- Some analyses are skipped but count not updated
- Planned analyses reduced but UI not updated
- Race condition in count tracking

## Impact

- Confusing user experience (where are the other 4 analyses?)
- Minor typo affects polish
- Makes it seem like analyses are incomplete or stuck

## Files to Investigate

- `server/src/cli/handlers/context.rs` - Context command implementation
- Parallel analysis spawning logic
- Progress tracking for analyses

## Suggested Fix

1. Count actual analyses dynamically instead of hardcoding
2. Ensure progress bar total matches spawned analyses
3. Fix typo to use consistent plural form ("analyses")

```rust
let analyses = vec![
    spawn_complexity_analysis(),
    spawn_satd_analysis(),
    spawn_dead_code_analysis(),
    spawn_entropy_analysis(),
];

let pb = ProgressBar::new(analyses.len() as u64);
pb.set_message("Running parallel analyses...");
```
