# TICKET-PMAT-6004: Enhanced Error Messages

**Sprint:** Sprint 20 - UX Improvements & Optimizations
**Priority:** P1 - High
**Estimated Effort:** 2-3 hours
**Status**: GREEN ✅
**Created:** 2025-10-06
**Completed:** 2025-10-06
**Commit:** 6eda28a

## Problem Statement

Generic errors with no context or actionable suggestions make debugging difficult for users.

## Solution

Created comprehensive error context module with rich error formatting:

**File:** `server/src/cli/error_context.rs`

**Features:**
- 4 error types: `FileNotFound`, `FileWriteError`, `ParseError`, `ConfigError`
- Helper functions: `roadmap_not_found()`, `cargo_toml_not_found()`, `file_not_found()`
- Rich formatting with file paths and actionable suggestions
- All functions CC <7

## Error Format Example

```
ERROR: Failed to read ROADMAP.md
  Location: /home/user/project/ROADMAP.md
  Reason: File not found

  Suggestions:
  - Run 'pmat maintain roadmap' from project root
  - Ensure you're in the correct directory
  - Check if ROADMAP.md exists in your project
```

## Updated Handlers

- `roadmap_handler.rs`: ROADMAP.md errors
- `generation_handlers.rs`: Scaffold errors
- All errors include full paths and suggestions

## Enhanced Error Examples

1. **Missing ROADMAP**: Shows path, suggests running from root
2. **Directory exists**: Shows location, suggests `--force` or different name
3. **Invalid framework**: Lists valid options with descriptions

## Acceptance Criteria

- [x] Error context module created
- [x] 4 error types with rich formatting
- [x] Helper functions for common scenarios
- [x] Full file paths in all errors
- [x] Actionable suggestions provided
- [x] All handlers updated
- [x] Cyclomatic complexity <7
- [x] Test coverage >80%

## Quality Metrics

- **CC:** All functions <7
- **Coverage:** >80%
- **Improved UX:** Clear, actionable error messages

---

**Status:** ✅ Complete
**Delivered:** v2.139.0
