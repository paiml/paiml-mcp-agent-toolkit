# Bug Report: Broken Progress Output in `pmat context`

**Date**: 2025-10-31
**Reporter**: User feedback
**Severity**: Medium
**Component**: CLI - context command progress display

## Description

When running `pmat context`, the progress output is broken and doesn't properly rewrite the same line. Instead of updating a single progress line, it creates multiple lines that overlap or display incorrectly.

## Steps to Reproduce

```bash
pmat context
```

## Actual Output

Based on screenshot description: Progress lines are not being properly overwritten, causing visual corruption in the terminal output.

## Expected Behavior

Progress output should:
1. Use ANSI escape codes to overwrite the same line
2. Display clean, single-line progress indicators
3. Properly clear previous content before updating

Example expected output:
```
🔍 Auto-detecting project language...
✅ Detected: rust (confidence: 95.0%)
⠋ Discovering project structure... [===>    ] 50%
```

Each line should update in place, not create new lines.

## Analysis

Possible causes:
- Not using `\r` (carriage return) to reset to line start
- Not clearing line with ANSI escape codes before update
- Progress indicator library (like `indicatif`) not configured correctly
- Terminal capabilities not detected properly

## Impact

- Poor user experience with cluttered output
- Difficult to read progress during long operations
- Makes debugging harder

## Files to Investigate

- `server/src/cli/handlers/context.rs` - Context command handler
- Progress bar/indicator implementation
- Terminal output formatting utilities

## Suggested Fix

Use proper progress indicator library:

```rust
use indicatif::{ProgressBar, ProgressStyle};

let pb = ProgressBar::new(total_files);
pb.set_style(ProgressStyle::default_bar()
    .template("{spinner:.green} {msg} [{bar:40.cyan/blue}] {pos}/{len}")
    .unwrap());

// Update progress
pb.set_message("Processing files...");
pb.inc(1);
```

Or use ANSI escape codes directly:

```rust
print!("\r\x1b[K🔍 Processing file {}/{}", current, total);
std::io::stdout().flush()?;
```
