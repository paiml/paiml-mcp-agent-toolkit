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
