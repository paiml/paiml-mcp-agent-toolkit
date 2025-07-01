# Refactor Auto Ignore Patterns Feature

## Overview

Added support for excluding/including files in `pmat refactor auto` to avoid refactoring test files, benchmarks, and other non-production code.

## New Command Options

```bash
pmat refactor auto \
  --exclude "tests/**,benches/**,**/fixtures/**" \
  --include "src/important/**" \
  --ignore-file .refactorignore
```

### Options:
- `--exclude`: Comma-separated glob patterns to exclude from refactoring
- `--include`: Comma-separated glob patterns to include (overrides exclude)
- `--ignore-file`: Path to a file containing exclude patterns (one per line)

## Default Exclusions

If no include patterns are specified, these patterns are excluded by default:
- `tests/**`
- `benches/**`
- `**/test_*.rs`
- `**/*_test.rs`
- `**/fixtures/**`

## .refactorignore File Format

```
# Comments start with #
tests/**
benches/**
**/fixtures/**
**/generated/**
build.rs
```

## Implementation Details

1. **Pattern Matching**: Uses glob patterns for flexible file matching
2. **Priority**: Include patterns override exclude patterns
3. **Applied To**: All file selection strategies (lint, complexity, SATD, coverage)
4. **Default Behavior**: Excludes common test/benchmark patterns unless explicitly included

## Example Usage

```bash
# Exclude all tests and benchmarks
pmat refactor auto --exclude "tests/**,benches/**"

# Only refactor specific modules
pmat refactor auto --include "src/core/**,src/api/**"

# Use ignore file
pmat refactor auto --ignore-file .refactorignore

# Override defaults to include tests
pmat refactor auto --include "tests/**" --exclude "benches/**"
```

This feature helps focus refactoring efforts on production code while avoiding test fixtures and benchmarks that often have intentionally complex code.