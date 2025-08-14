# Context and Deep-Context Documentation Update

## Date: 2025-07-13

## Overview

This document summarizes the documentation updates made to reflect the current implementation of `context` and `deep-context` commands after fixing GitHub issue #33.

## Changes Made

### 1. **CLI Reference Documentation** (`docs/cli-reference.md`)

#### Updated `context` command
- Already had correct documentation with all current flags:
  - `-t, --toolchain` - Target toolchain (auto-detected if not specified)
  - `-p, --project-path` - Project path to analyze (default: .)
  - `-o, --output` - Output file path
  - `--format` - Output format (markdown, json, yaml)
  - `--include-large-files` - Include large files >500KB
  - `--skip-expensive-metrics` - Skip expensive metrics for faster execution

#### Updated `analyze deep-context` command
- **Added missing options**:
  - `--include` - Comma-separated list of analyses to include
  - `--exclude` - Comma-separated list of analyses to exclude
  - `--parallel` - Enable parallel processing (boolean flag, not number)
  - `--verbose` - Enable verbose logging
  - `--top-files` - Number of top files to show in summary (default: 10)
  
- **Updated option values**:
  - `--format`: Changed from (markdown, json, yaml) to (markdown, json, sarif)
  - `--dag-type`: Updated values to (call-graph, import-graph, inheritance, full-dependency)
  - `--cache-strategy`: Updated values to (normal, force-refresh, offline)
  
- **Removed outdated options**:
  - `--max-top-k` - Not in current implementation
  - `--defect-threshold` - Not in current implementation
  - `--parallel <NUM>` - Changed to boolean flag

- **Enhanced examples** with practical use cases including pattern filtering

### 2. **Main README.md**

- Added `--skip-expensive-metrics` example for context command
- Added `--full` and `--include-pattern` examples for deep-context command
- Kept existing examples intact while adding new relevant ones

### 3. **Deep Context Analysis Feature Documentation** (`docs/features/deep-context-analysis.md`)

- **Updated command examples** to reflect current CLI options
- **Replaced configuration file section** with command-line options
- **Added new examples**:
  - SARIF format output
  - Include/exclude specific analyses
  - Cache strategies (normal, force-refresh, offline)
  - Pattern matching with simpler syntax
  - Top files control
  - Verbose output

- **Updated configuration section** to accurately list all available options
- **Added comprehensive multi-option example** showing real-world usage

### 4. **MCP Methods Documentation** (`docs/mcp-methods.md`)

- **Enhanced `generate_context` tool documentation**:
  - Added detailed parameter descriptions
  - Added return value description
  
- **Enhanced `analyze_deep_context` tool documentation**:
  - Added detailed parameter descriptions
  - Added return value description
  - Removed parameters not in MCP interface

### 5. **API Guide** (`docs/api-guide.md`)

- No changes needed - existing examples were already correct

## Key Differences Between Commands

### `context` Command
- **Purpose**: Generate AST-based project context for AI/LLM consumption
- **Focus**: Code structure and organization
- **Performance**: Fast, optimized for AI input
- **Key Options**: Toolchain selection, large file handling, metric skipping

### `deep-context` Command  
- **Purpose**: Comprehensive code analysis with quality metrics
- **Focus**: Complexity analysis, quality assessment, refactoring insights
- **Performance**: More thorough, includes complexity calculations
- **Key Options**: Pattern filtering, analysis selection, cache control, verbosity

## Implementation Details Fixed

1. **Deep context now uses proper AST analysis** instead of stub implementation
2. **Include patterns are fully functional** for file filtering
3. **JSON output includes file-level details** with complexity metrics
4. **Consistent implementation** between context and deep-context commands

## Testing

All documentation updates were verified against:
- Current command definitions in `server/src/cli/commands.rs`
- Working implementation in `server/src/services/simple_deep_context.rs`
- Passing integration tests in `server/tests/deep_context_cli_integration.rs`

## Status

✅ All documentation is now accurate and up-to-date with the current implementation.