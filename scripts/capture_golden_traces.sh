#!/bin/bash
# Golden Trace Capture Script for paiml-mcp-agent-toolkit
#
# Captures syscall traces for PMAT CLI operations using Renacer.
# Generates 3 formats: JSON, summary statistics, and source-correlated traces.
#
# Usage: ./scripts/capture_golden_traces.sh
#
# shellcheck disable=SC2032  # Variables are used within this script
# bashrs-disable: DET002 SEC010  # Timestamps intentional for trace capture

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
TRACES_DIR="golden_traces"
BINARY_PATH="./target/release/pmat"

# Ensure binary exists
if [ ! -f "$BINARY_PATH" ]; then
    echo -e "${YELLOW}Binary not found. Building release binary...${NC}"
    cargo build --release --bin pmat
fi

# Ensure renacer is installed
if ! command -v renacer &> /dev/null; then
    echo -e "${YELLOW}Renacer not found. Installing from crates.io...${NC}"
    cargo install renacer --version 0.6.2
fi

# Create traces directory
mkdir -p "$TRACES_DIR"

echo -e "${BLUE}=== Capturing Golden Traces for PMAT ===${NC}"
echo -e "Binary: $BINARY_PATH"
echo -e "Output: $TRACES_DIR/"
echo ""

# ==============================================================================
# Trace 1: pmat --version (minimal startup)
# ==============================================================================
echo -e "${GREEN}[1/5]${NC} Capturing: pmat --version"
renacer --format json -- "$BINARY_PATH" --version 2>&1 | \
    sed '/^pmat/d' > "$TRACES_DIR/pmat_version.json"

renacer --summary --timing -- "$BINARY_PATH" --version 2>&1 | \
    sed '/^pmat/d' | tail -n +2 > "$TRACES_DIR/pmat_version_summary.txt"

renacer -s --format json -- "$BINARY_PATH" --version 2>&1 | \
    sed '/^pmat/d' > "$TRACES_DIR/pmat_version_source.json"

# ==============================================================================
# Trace 2: pmat --help (help text generation)
# ==============================================================================
echo -e "${GREEN}[2/5]${NC} Capturing: pmat --help"
renacer --format json -- "$BINARY_PATH" --help 2>&1 | \
    grep -v "^PMAT\|^Pragmatic\|^Usage\|^Options\|^Commands\|^  " | \
    head -1 > "$TRACES_DIR/pmat_help.json"

renacer --summary --timing -- "$BINARY_PATH" --help 2>&1 | \
    tail -n +2 > "$TRACES_DIR/pmat_help_summary.txt"

# ==============================================================================
# Trace 3: pmat context (AST analysis)
# ==============================================================================
echo -e "${GREEN}[3/5]${NC} Capturing: pmat context"

# Create minimal test file for analysis
TEMP_DIR=$(mktemp -d)
TEST_FILE="$TEMP_DIR/test.rs"
cat > "$TEST_FILE" << 'EOF'
// Minimal test file for golden trace capture
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn main() {
    println!("Hello, world!");
}
EOF

# Capture trace (filter out analysis output, keep trace JSON)
renacer --format json -- "$BINARY_PATH" context "$TEST_FILE" 2>&1 | \
    grep -v "^Processing\|^Analyzing\|^Generated\|^##\|^-\|^Total\|^AST" | \
    head -1 > "$TRACES_DIR/pmat_context.json" 2>/dev/null || \
    echo '{"version":"0.6.2","format":"renacer-json-v1","syscalls":[]}' > "$TRACES_DIR/pmat_context.json"

renacer --summary --timing -- "$BINARY_PATH" context "$TEST_FILE" 2>&1 | \
    tail -n +2 > "$TRACES_DIR/pmat_context_summary.txt"

# Cleanup temp file
rm -rf "$TEMP_DIR"

# ==============================================================================
# Trace 4: pmat list (template listing - simple operation)
# ==============================================================================
echo -e "${GREEN}[4/5]${NC} Capturing: pmat list"

# Capture trace for list command
renacer --format json -- "$BINARY_PATH" list 2>&1 | \
    grep -v "^Available\|^  \|^Total" | \
    head -1 > "$TRACES_DIR/pmat_list.json" 2>/dev/null || \
    echo '{"version":"0.6.2","format":"renacer-json-v1","syscalls":[]}' > "$TRACES_DIR/pmat_list.json"

renacer --summary --timing -- "$BINARY_PATH" list 2>&1 | \
    tail -n +2 > "$TRACES_DIR/pmat_list_summary.txt"

# ==============================================================================
# Trace 5: pmat hooks status (hook management)
# ==============================================================================
echo -e "${GREEN}[5/5]${NC} Capturing: pmat hooks status"

renacer --format json -- "$BINARY_PATH" hooks status 2>&1 | \
    grep -v "^Git hooks\|^  " | \
    head -1 > "$TRACES_DIR/pmat_hooks_status.json" 2>/dev/null || \
    echo '{"version":"0.6.2","format":"renacer-json-v1","syscalls":[]}' > "$TRACES_DIR/pmat_hooks_status.json"

renacer --summary --timing -- "$BINARY_PATH" hooks status 2>&1 | \
    tail -n +2 > "$TRACES_DIR/pmat_hooks_status_summary.txt"

# ==============================================================================
# Generate Analysis Report
# ==============================================================================
echo ""
echo -e "${GREEN}Generating analysis report...${NC}"

cat > "$TRACES_DIR/ANALYSIS.md" << 'EOF'
# Golden Trace Analysis Report - PMAT

## Overview

This directory contains golden traces captured from paiml-mcp-agent-toolkit (PMAT) CLI operations.

## Trace Files

| File | Description | Format |
|------|-------------|--------|
| pmat_version.json | Minimal startup (--version) | JSON |
| pmat_version_summary.txt | Version syscall summary | Text |
| pmat_version_source.json | Version with source locations | JSON |
| pmat_help.json | Help text generation | JSON |
| pmat_help_summary.txt | Help syscall summary | Text |
| pmat_context.json | AST context generation | JSON |
| pmat_context_summary.txt | Context syscall summary | Text |
| pmat_list.json | Template listing | JSON |
| pmat_list_summary.txt | List syscall summary | Text |
| pmat_hooks_status.json | Hook status trace | JSON |
| pmat_hooks_status_summary.txt | Hooks syscall summary | Text |

## How to Use These Traces

### 1. Regression Testing

Compare new builds against golden traces:

```bash
# Capture new trace
renacer --format json -- ./target/release/pmat --version > new_trace.json

# Compare with golden
diff golden_traces/pmat_version.json new_trace.json

# Or use semantic equivalence validator (in test suite)
cargo test --test golden_trace_validation
```

### 2. Performance Budgeting

Check if new build meets performance requirements:

```bash
# Run with assertions
cargo test --test performance_assertions

# Or manually check against summary
cat golden_traces/pmat_version_summary.txt
```

### 3. CI/CD Integration

Add to .github/workflows/ci.yml:

```yaml
- name: Validate Performance
  run: |
    renacer --format json -- ./target/release/pmat --version > trace.json
    # Compare against golden trace or run assertions
    cargo test --test golden_trace_validation
```

## Trace Interpretation Guide

### JSON Trace Format

```json
{
  "version": "0.6.2",
  "format": "renacer-json-v1",
  "syscalls": [
    {
      "name": "write",
      "args": [["fd", "1"], ["buf", "pmat 2.202.0\n"], ["count", "12"]],
      "result": 12
    }
  ]
}
```

### Summary Statistics Format

```
% time     seconds  usecs/call     calls    errors syscall
------ ----------- ----------- --------- --------- ----------------
 19.27    0.000137          10        13           mmap
 14.35    0.000102          17         6           write
...
```

**Key metrics:**
- % time: Percentage of total runtime spent in this syscall
- usecs/call: Average latency per call (microseconds)
- calls: Total number of invocations
- errors: Number of failed calls

## Baseline Performance Metrics

From initial golden trace capture:

| Operation | Runtime | Syscalls | Notes |
|-----------|---------|----------|-------|
| pmat --version | TBD | TBD | Minimal startup path |
| pmat --help | TBD | TBD | Help text generation |
| pmat context file | TBD | TBD | AST context generation |
| pmat list | TBD | TBD | Template listing |
| pmat hooks status | TBD | TBD | Hook management |

## Next Steps

1. **Set performance baselines** using these golden traces
2. **Add assertions** in renacer.toml for automated checking
3. **Integrate with CI** to prevent regressions
4. **Compare across versions** to track performance improvements
5. **Monitor syscall patterns** for unexpected behavior changes

Generated: $(date)
Renacer Version: 0.6.2
EOF

# ==============================================================================
# Summary
# ==============================================================================
echo ""
echo -e "${BLUE}=== Golden Trace Capture Complete ===${NC}"
echo ""
echo "Traces saved to: $TRACES_DIR/"
echo ""
echo "Files generated:"
ls -lh "$TRACES_DIR"/*.json "$TRACES_DIR"/*.txt 2>/dev/null | awk '{print "  " $9 " (" $5 ")"}'
echo ""
echo -e "${GREEN}Next steps:${NC}"
echo "  1. Review traces: cat golden_traces/pmat_version_summary.txt"
echo "  2. View JSON: jq . golden_traces/pmat_version.json | less"
echo "  3. Run tests: cargo test --test golden_trace_validation"
echo "  4. Update baselines in ANALYSIS.md with actual metrics"
