# Renacer Golden Trace Integration Report - PMAT

**Project**: paiml-mcp-agent-toolkit (PMAT)
**Renacer Version**: 0.6.2
**Date**: 2025-11-23
**Author**: Claude Code (Anthropic)

---

## Executive Summary

This document describes the integration of **Renacer** (pure Rust syscall tracer) with the **PMAT** (Pragmatic AI Labs Multi-language Agent Toolkit) CLI. The integration enables:

1. **Golden Trace Validation**: Capture canonical execution traces for regression testing
2. **Performance Baselines**: Enforce syscall count and latency budgets
3. **Build-Time Assertions**: TOML-based performance contracts that fail CI on violations
4. **Observability Integration**: Optional OpenTelemetry export to Jaeger/Grafana

---

## Quick Start

### 1. Install Renacer

```bash
cargo install renacer --version 0.6.2
```

### 2. Build PMAT

```bash
cd /path/to/paiml-mcp-agent-toolkit
cargo build --release --bin pmat
```

### 3. Capture Golden Traces

```bash
./scripts/capture_golden_traces.sh
```

**Output:**
- `golden_traces/pmat_version.json` - Minimal startup trace
- `golden_traces/pmat_analyze_complexity.json` - Analysis operation trace
- `golden_traces/pmat_repo_score.json` - Repository scoring trace
- Summary statistics for each operation

### 4. View Traces

```bash
# Summary statistics
cat golden_traces/pmat_version_summary.txt

# Full JSON trace (formatted)
jq . golden_traces/pmat_version.json | less

# Syscall timeline
renacer --timing -- ./target/release/pmat --version
```

---

## Integration Architecture

### Components

```
┌─────────────────────────────────────────────────────────────┐
│                    PMAT CLI Binary                          │
│              (pmat analyze, pmat repo-score, etc.)          │
└────────────────────┬────────────────────────────────────────┘
                     │
                     │ traced by
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                  Renacer (ptrace)                           │
│   - Syscall interception (read, write, openat, mmap)       │
│   - Lamport logical clocks (causal ordering)               │
│   - Source correlation (DWARF debug info)                  │
└────────────────────┬────────────────────────────────────────┘
                     │
                     │ exports to
                     ▼
┌─────────────────────────────────────────────────────────────┐
│              Golden Trace Storage                           │
│   - JSON format (machine-readable)                          │
│   - Summary statistics (human-readable)                     │
│   - Source-correlated traces (debugging)                    │
└────────────────────┬────────────────────────────────────────┘
                     │
                     │ validated by
                     ▼
┌─────────────────────────────────────────────────────────────┐
│           Performance Assertions (renacer.toml)             │
│   - CLI startup latency: <50ms                              │
│   - Syscall budget: <300 calls                              │
│   - Memory budget: <50MB                                    │
│   - Anti-pattern detection: GodProcess, TightLoop          │
└─────────────────────────────────────────────────────────────┘
```

### File Structure

```
paiml-mcp-agent-toolkit/
├── renacer.toml                          # Performance assertions
├── scripts/
│   └── capture_golden_traces.sh          # Trace capture automation
├── golden_traces/
│   ├── ANALYSIS.md                       # Trace analysis report
│   ├── pmat_version.json                 # Minimal startup trace
│   ├── pmat_version_summary.txt          # Statistics
│   ├── pmat_analyze_complexity.json      # Analysis operation
│   ├── pmat_repo_score.json              # Repo scoring trace
│   └── pmat_hooks_status.json            # Hook management trace
└── docs/
    └── integration-report-golden-trace.md  # This file
```

---

## Captured Operations

### 1. Minimal Startup: `pmat --version`

**Purpose**: Baseline for CLI startup latency
**Expected Behavior**:
- Print version string to stdout
- Minimal syscall overhead
- Fast exit (<10ms)

**Trace Capture**:
```bash
renacer --format json -- ./target/release/pmat --version > pmat_version.json
```

**Expected Syscalls**:
- `brk` / `mmap`: Memory allocation
- `openat`: Library loading
- `write`: Version output to stdout
- `exit_group`: Clean shutdown

**Performance Budget**:
- Runtime: <10ms
- Syscalls: <100
- Memory: <10MB

---

### 2. Help Generation: `pmat --help`

**Purpose**: Measure help text rendering overhead
**Expected Behavior**:
- Parse CLI structure
- Generate formatted help text
- Write to stdout

**Trace Capture**:
```bash
renacer --format json -- ./target/release/pmat --help > pmat_help.json
```

**Expected Syscalls**:
- `write`: Multiple calls for help text
- Similar memory profile to `--version`

**Performance Budget**:
- Runtime: <20ms
- Syscalls: <150
- Memory: <15MB

---

### 3. Code Analysis: `pmat analyze complexity <file>`

**Purpose**: Measure single-file analysis overhead
**Expected Behavior**:
- Read source file
- Parse AST (tree-sitter or syn)
- Calculate complexity metrics
- Output results

**Trace Capture**:
```bash
# Create minimal test file
echo 'fn add(a: i32, b: i32) -> i32 { a + b }' > test.rs
renacer --format json -- ./target/release/pmat analyze complexity test.rs > pmat_analyze.json
```

**Expected Syscalls**:
- `openat`: Read source file
- `read`: File contents
- `write`: Analysis results
- `mmap`: Parser memory allocation

**Performance Budget**:
- Runtime: <50ms (simple file)
- Syscalls: <200
- Memory: <30MB

---

### 4. Repository Scoring: `pmat repo-score .`

**Purpose**: Measure repository-wide analysis
**Expected Behavior**:
- Git repository scanning
- Multi-file analysis
- Aggregate metrics calculation

**Trace Capture**:
```bash
renacer --format json -- ./target/release/pmat repo-score . --format json > pmat_repo_score.json
```

**Expected Syscalls**:
- `openat`: Many (repository files)
- `read`: File reading
- `getdents64`: Directory traversal
- `stat`: File metadata
- `write`: JSON output

**Performance Budget**:
- Runtime: <500ms (small repo)
- Syscalls: <1000
- Memory: <100MB

---

### 5. Hook Management: `pmat hooks status`

**Purpose**: Measure hook introspection overhead
**Expected Behavior**:
- Check `.git/hooks` directory
- Read hook files
- Display status

**Trace Capture**:
```bash
renacer --format json -- ./target/release/pmat hooks status > pmat_hooks.json
```

**Expected Syscalls**:
- `openat`: `.git/hooks` directory
- `read`: Hook file contents
- `write`: Status output

**Performance Budget**:
- Runtime: <30ms
- Syscalls: <150
- Memory: <20MB

---

## Performance Assertions (renacer.toml)

### Critical Path Latency

```toml
[[assertion]]
name = "cli_startup_latency"
type = "critical_path"
max_duration_ms = 50
fail_on_violation = true
enabled = true
```

**Rationale**: CLI tools should feel instant. 50ms is perceptible but acceptable for complex operations. Violations indicate regression in startup path (dependency bloat, static initialization, etc.).

---

### Syscall Budget

```toml
[[assertion]]
name = "max_syscall_budget"
type = "span_count"
max_spans = 300
fail_on_violation = true
enabled = true
```

**Rationale**: Excessive syscalls indicate inefficient I/O patterns (e.g., reading files byte-by-byte, unnecessary stats). Budget prevents regressions.

---

### Memory Allocation Budget

```toml
[[assertion]]
name = "memory_allocation_budget"
type = "memory_usage"
max_bytes = 52428800  # 50MB
tracking_mode = "allocations"
fail_on_violation = true
enabled = true
```

**Rationale**: CLI tools should be memory-efficient. 50MB allows for AST parsing and analysis but prevents unbounded growth.

---

### Anti-Pattern Detection

```toml
[[assertion]]
name = "prevent_god_process"
type = "anti_pattern"
pattern = "GodProcess"
threshold = 0.8
fail_on_violation = false  # Warning only
enabled = true
```

**Patterns Detected**:
1. **GodProcess**: Single process doing too much (spawns many threads, opens many files)
2. **TightLoop**: Repeated syscalls in tight loops (e.g., `read()` in a loop without buffering)
3. **PCIeBottleneck**: Excessive GPU memory transfers (not applicable for PMAT CLI)

---

## CI/CD Integration

### GitHub Actions Workflow

Add to `.github/workflows/ci.yml`:

```yaml
name: Golden Trace Validation

on: [push, pull_request]

jobs:
  validate-traces:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install Renacer
        run: cargo install renacer --version 0.6.2

      - name: Build PMAT (Release)
        run: cargo build --release --bin pmat

      - name: Capture Golden Traces
        run: ./scripts/capture_golden_traces.sh

      - name: Run Performance Assertions
        run: |
          # Example: Validate pmat --version meets latency budget
          renacer --assert renacer.toml -- ./target/release/pmat --version

      - name: Upload Traces (Artifact)
        uses: actions/upload-artifact@v3
        with:
          name: golden-traces
          path: golden_traces/

      - name: Compare Against Baseline (Optional)
        run: |
          # Download baseline from previous run
          # diff baseline/pmat_version.json golden_traces/pmat_version.json
          echo "TODO: Implement baseline comparison"
```

---

## Advanced Usage

### 1. Source Code Correlation

Map syscalls back to Rust source code:

```bash
# Build with debug symbols
RUSTFLAGS="-C debuginfo=2" cargo build --release

# Trace with source locations
renacer -s -- ./target/release/pmat --version
```

**Output:**
```
write(1, "pmat 2.202.0\n", 12) = 12  [server/src/bin/pmat.rs:42]
```

---

### 2. OpenTelemetry Export

Export traces to Jaeger for visualization:

```bash
# Start Jaeger (Docker)
docker run -d --name jaeger \
  -e COLLECTOR_OTLP_ENABLED=true \
  -p 4317:4317 \
  -p 16686:16686 \
  jaegertracing/all-in-one:latest

# Export trace to OTLP
renacer --otlp http://localhost:4317 -- ./target/release/pmat --version

# View in Jaeger UI
open http://localhost:16686
```

---

### 3. Lamport Logical Clocks

Understand causal ordering across processes:

```bash
# Enable Lamport clock propagation
export RENACER_LAMPORT_CLOCK=1

# Trace with causal ordering
renacer --format json -- ./target/release/pmat analyze complexity test.rs
```

**Output** (excerpt):
```json
{
  "syscalls": [
    {"name": "fork", "lamport_clock": 42, "result": 1234},
    {"name": "execve", "lamport_clock": 43, "args": [["path", "/bin/sh"]]},
    {"name": "wait4", "lamport_clock": 44, "result": 1234}
  ]
}
```

**Use Case**: Track causality when PMAT spawns subprocesses (e.g., `cargo`, `git`).

---

### 4. Trace Compression

Reduce storage for large traces:

```bash
# Enable RLE compression (configured in renacer.toml)
renacer --format json -- ./target/release/pmat repo-score . > trace.json.rle
```

**Benefit**: 10× size reduction for repetitive syscall patterns (e.g., many `read()` calls).

---

## Troubleshooting

### Issue: Capture script fails with "Binary not found"

**Solution**:
```bash
cargo build --release --bin pmat
./scripts/capture_golden_traces.sh
```

---

### Issue: Trace contains program output mixed with JSON

**Symptoms**: JSON parsing fails due to stdout contamination.

**Solution**: Capture script uses filters to separate program output from trace:
```bash
renacer --format json -- pmat --version 2>&1 | sed '/^pmat/d' > trace.json
```

---

### Issue: Performance regression detected

**Diagnosis**:
```bash
# Compare current vs golden
renacer --summary --timing -- ./target/release/pmat --version
cat golden_traces/pmat_version_summary.txt
```

**Common causes**:
- New dependencies (increase startup overhead)
- Debug build instead of release
- CI environment variance (increase tolerance)
- Unnecessary file operations (check syscall diff)

---

### Issue: Syscall count regression

**Diagnosis**:
```bash
# Detailed syscall comparison
renacer -- ./target/release/pmat --version > current_trace.txt
diff current_trace.txt golden_traces/pmat_version_summary.txt
```

**Common causes**:
- New library initialization (e.g., logging setup)
- Environment differences (locale, timezone queries)
- Tracing overhead variance (ptrace adds overhead)

---

## Testing Integration

### Unit Tests (Optional)

Create `tests/golden_trace_validation.rs`:

```rust
//! Golden Trace Validation Tests
//!
//! Validates that PMAT CLI operations produce expected syscall patterns
//! and meet performance budgets defined in renacer.toml.

use std::process::Command;
use std::fs;

#[test]
fn test_cli_version_completes() {
    let output = Command::new("./target/release/pmat")
        .arg("--version")
        .output()
        .expect("Failed to execute pmat --version");

    assert!(output.status.success(), "CLI should exit with success");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("pmat"), "Output should contain 'pmat'");
}

#[test]
fn test_golden_trace_exists() {
    let golden_trace_path = "golden_traces/pmat_version.json";
    assert!(
        std::path::Path::new(golden_trace_path).exists(),
        "Golden trace should exist. Run: ./scripts/capture_golden_traces.sh"
    );
}

#[test]
fn test_golden_trace_format() {
    let golden_trace_path = "golden_traces/pmat_version.json";
    let contents = fs::read_to_string(golden_trace_path)
        .expect("Golden trace file should be readable");

    let json: serde_json::Value = serde_json::from_str(&contents)
        .expect("Golden trace should be valid JSON");

    // Check version
    assert_eq!(json["version"], "0.6.2", "Trace version should match");
    assert_eq!(json["format"], "renacer-json-v1", "Format should be renacer-json-v1");

    // Check syscalls array exists
    assert!(json["syscalls"].is_array(), "Should have syscalls array");
}

#[test]
fn test_performance_baseline() {
    let summary_path = "golden_traces/pmat_version_summary.txt";
    let summary = fs::read_to_string(summary_path)
        .expect("Summary file should exist");

    // Parse total runtime from last line
    let last_line = summary.lines().last().unwrap();
    let parts: Vec<&str> = last_line.split_whitespace().collect();

    // Extract total time (column 2)
    let total_time_str = parts[1];
    let total_time_secs: f64 = total_time_str.parse().unwrap();
    let total_time_ms = total_time_secs * 1000.0;

    println!("Golden trace total runtime: {:.3}ms", total_time_ms);

    // Baseline: CLI should complete in <10ms
    assert!(
        total_time_ms < 10.0,
        "CLI --version should complete in <10ms (actual: {:.3}ms)",
        total_time_ms
    );
}

#[test]
fn test_syscall_count_budget() {
    let summary_path = "golden_traces/pmat_version_summary.txt";
    let summary = fs::read_to_string(summary_path)
        .expect("Summary file should exist");

    // Parse total syscalls from last line
    let last_line = summary.lines().last().unwrap();
    let parts: Vec<&str> = last_line.split_whitespace().collect();

    // Extract total calls (column 4)
    let total_calls: usize = parts[3].parse().unwrap();

    println!("Golden trace total syscalls: {}", total_calls);

    // Budget: CLI should use <100 syscalls for --version
    assert!(
        total_calls < 100,
        "CLI --version should use <100 syscalls (actual: {})",
        total_calls
    );
}

#[test]
fn test_expected_syscall_patterns() {
    let golden_trace_path = "golden_traces/pmat_version.json";
    let contents = fs::read_to_string(golden_trace_path)
        .expect("Golden trace file should be readable");

    let json: serde_json::Value = serde_json::from_str(&contents)
        .expect("Golden trace should be valid JSON");

    let syscalls = json["syscalls"].as_array().unwrap();

    // Find write syscall (for version output)
    let has_write = syscalls.iter().any(|sc| {
        sc["name"].as_str() == Some("write")
    });

    assert!(has_write, "CLI should perform write syscall for version output");

    // Find memory allocation
    let has_memory_alloc = syscalls.iter().any(|sc| {
        matches!(sc["name"].as_str(), Some("brk") | Some("mmap"))
    });

    assert!(has_memory_alloc, "CLI should perform memory allocation");
}

#[test]
#[ignore] // Run manually: cargo test --test golden_trace_validation test_regression_check -- --ignored
fn test_regression_check() {
    use std::process::Command;

    // Run CLI with renacer to capture new trace
    let output = Command::new("renacer")
        .args(&["--format", "json", "--", "./target/release/pmat", "--version"])
        .output()
        .expect("Failed to run renacer");

    assert!(output.status.success(), "Renacer should execute successfully");

    let new_trace = String::from_utf8_lossy(&output.stdout);

    // Filter out version output
    let new_trace_json: String = new_trace
        .lines()
        .filter(|line| !line.starts_with("pmat"))
        .collect::<Vec<_>>()
        .join("\n");

    let new_json: serde_json::Value = serde_json::from_str(&new_trace_json)
        .expect("New trace should be valid JSON");

    // Load golden trace
    let golden_trace_path = "golden_traces/pmat_version.json";
    let golden_contents = fs::read_to_string(golden_trace_path)
        .expect("Golden trace should exist");
    let golden_json: serde_json::Value = serde_json::from_str(&golden_contents)
        .expect("Golden trace should be valid JSON");

    // Compare syscall counts
    let new_count = new_json["syscalls"].as_array().unwrap().len();
    let golden_count = golden_json["syscalls"].as_array().unwrap().len();

    // Allow some variance (±10 syscalls) due to environment differences
    let diff = (new_count as i32 - golden_count as i32).abs();

    assert!(
        diff <= 10,
        "Syscall count regression detected. Golden: {}, New: {}, Diff: {}",
        golden_count, new_count, diff
    );

    println!("✓ No significant regression detected");
    println!("  Golden syscalls: {}", golden_count);
    println!("  New syscalls: {}", new_count);
    println!("  Difference: {}", diff);
}
```

**Add to `Cargo.toml`:**
```toml
[[test]]
name = "golden_trace_validation"
path = "tests/golden_trace_validation.rs"

[dev-dependencies]
serde_json = "1.0"
```

**Run tests:**
```bash
cargo test --test golden_trace_validation
```

---

## Renacer Features Used

| Feature | Description | PMAT Use Case |
|---------|-------------|---------------|
| **JSON Export** | Machine-readable trace format | CI/CD integration, automated validation |
| **Summary Statistics** | Human-readable syscall summary | Performance baselines, quick diagnostics |
| **Source Correlation** | Map syscalls to Rust source | Debug performance bottlenecks |
| **Low Overhead** | <1% runtime impact | Production tracing without distortion |
| **Lamport Clocks** | Causal ordering guarantees | Track subprocess spawning (cargo, git) |
| **Build-Time Assertions** | TOML-based performance contracts | Fail CI on regression (Andon principle) |
| **Anti-Pattern Detection** | GodProcess, TightLoop, PCIe | Prevent architectural decay |
| **OTLP Export** | OpenTelemetry integration | Jaeger/Grafana visualization |
| **Trace Compression** | RLE encoding | 10× size reduction for large traces |

---

## Toyota Way Principles

### Andon (Stop the Line)

**Build-time assertions** embody Andon: When performance regressions are detected, CI fails immediately. No defects pass to the next stage.

```toml
[[assertion]]
name = "cli_startup_latency"
type = "critical_path"
max_duration_ms = 50
fail_on_violation = true  # ← Andon: Stop the line
enabled = true
```

---

### Poka-Yoke (Error-Proofing)

**Golden traces** prevent regressions by making expected behavior explicit. Deviations are automatically detected.

```bash
# Automated comparison (poka-yoke)
diff golden_traces/pmat_version.json new_trace.json
```

---

### Jidoka (Autonomation)

**Renacer runs automatically in CI**, validating every commit without manual intervention. Quality is built-in, not inspected-in.

```yaml
# Automated quality enforcement
- name: Run Performance Assertions
  run: renacer --assert renacer.toml -- ./target/release/pmat --version
```

---

## Next Steps

1. **Capture Baselines**: Run `./scripts/capture_golden_traces.sh` and update `ANALYSIS.md` with actual metrics
2. **Integrate with CI**: Add GitHub Actions workflow for automated validation
3. **Tune Assertions**: Adjust `renacer.toml` budgets based on observed performance
4. **Enable OTLP** (Optional): Export traces to observability stack (Jaeger, Grafana)
5. **Monitor Production**: Use Renacer to trace production workloads and detect anomalies

---

## References

- [Renacer GitHub](https://github.com/paiml/renacer)
- [Renacer Documentation](https://docs.rs/renacer)
- [OpenTelemetry Specification](https://opentelemetry.io/docs/specs/otel/)
- [Toyota Way Principles](https://en.wikipedia.org/wiki/The_Toyota_Way)
- [PMAT Documentation](https://paiml.github.io/pmat-book/)

---

**Generated**: 2025-11-23
**Renacer Version**: 0.6.2
**PMAT Version**: 2.202.0
