# Bug Report: Warnings Displayed as Errors in File Processing

**Date**: 2025-10-31
**Reporter**: User feedback
**Severity**: Low
**Component**: CLI - error/warning display

## Description

When running `pmat context` on large projects (tested on Ceph), warnings about file processing errors are displayed in a way that appears as errors. The messages start with "Warning:" but are formatted or colored like errors, causing confusion.

## Steps to Reproduce

```bash
cd /path/to/ceph
pmat context
```

## Actual Output

```
⠙ Running parallel analyses...
  Running analyses [███████████▎                  ] 3/8

Warning: Error processing file ./pybind/mgr/dashboard/services/proto/gateway_pb2.py: Parameter validation failed: l
⠙ Running parallel analyses...

Warning: Error processing file ./rgw/driver/posix/zpp_bits.h: Parameter validation failed: line - Line too long for
✅ Context written to: output.txt
```

## Issues

1. **Message says "Warning:" but contains "Error processing file"** - contradictory
2. **Truncated messages** - "Parameter validation failed: l" is incomplete
3. **Poor formatting** - warnings mixed with progress indicators
4. **No context** - what does "line too long for" mean? Too long for what?

## Expected Behavior

Warnings should be:

1. **Properly categorized**:
   - Either it's a warning (file skipped, continue processing)
   - Or it's an error (processing failed)

2. **Complete messages**:
```
⚠️  Skipping file: ./pybind/mgr/dashboard/services/proto/gateway_pb2.py
    Reason: Parameter validation failed - line too long (>10000 characters)

⚠️  Skipping file: ./rgw/driver/posix/zpp_bits.h
    Reason: Line exceeds maximum length (10000 characters at line 42)
```

3. **Grouped at end**:
```
✅ Context written to: output.txt

⚠️  2 files skipped due to parsing errors:
    - ./pybind/mgr/dashboard/services/proto/gateway_pb2.py (line too long)
    - ./rgw/driver/posix/zpp_bits.h (line too long)
```

4. **Not interleaved with progress**

## Analysis

Issues:
- Terminal line width not detected, causing truncation
- Warnings printed to stdout instead of stderr or being buffered
- No message length limits or wrapping
- Mixed concerns (progress + warnings in same output stream)

## Impact

- Confusing user experience
- Unclear whether processing succeeded or failed
- Truncated messages provide no actionable information
- Makes output look sloppy/unprofessional

## Files to Investigate

- `server/src/cli/handlers/context.rs` - Warning/error handling
- File processing error handling
- Progress indicator + logging interaction

## Suggested Fix

1. **Buffer warnings** during processing, display at end
2. **Complete error messages** with full context
3. **Proper formatting** with terminal width detection
4. **Use stderr** for warnings/errors, stdout for results

```rust
use terminal_size::{Width, terminal_size};

fn format_warning(file: &str, error: &str) -> String {
    let width = terminal_size()
        .map(|(Width(w), _)| w as usize)
        .unwrap_or(80);

    let msg = format!("⚠️  Skipping file: {}\n    Reason: {}", file, error);

    // Wrap long lines
    textwrap::wrap(&msg, width - 4)
        .join("\n    ")
}
```
