# Cargo-Mutants Integration User Guide

**Feature**: `pmat mutate --use-cargo-mutants`
**Version**: PMAT v2.181.0+
**Status**: Production Ready

---

## Table of Contents

1. [Introduction](#introduction)
2. [Installation](#installation)
3. [Quick Start](#quick-start)
4. [Advanced Usage](#advanced-usage)
5. [Best Practices](#best-practices)
6. [Troubleshooting](#troubleshooting)
7. [FAQ](#faq)

---

## Introduction

### What is cargo-mutants?

[cargo-mutants](https://mutants.rs/) is an industry-standard mutation testing tool for Rust that:
- Generates mutants by modifying your source code
- Runs your test suite against each mutant
- Reports which mutants were caught (killed) vs missed (survived)
- Helps identify gaps in your test coverage

### Why use cargo-mutants with PMAT?

PMAT's cargo-mutants integration provides:

✅ **Seamless Integration**: Run cargo-mutants through PMAT's unified CLI
✅ **Consistent Output**: PMAT formats and displays results consistently
✅ **Workflow Integration**: Combine with TDG scoring, quality gates, and CI/CD
✅ **Best Practices**: Built-in timeout handling, parallel execution, and error reporting
✅ **Production Ready**: Thoroughly tested with cargo-mutants v25.3.1

### When to use `--use-cargo-mutants` vs Built-in Mutation Testing

**Use `--use-cargo-mutants` when**:
- You need comprehensive Rust mutation testing
- You want industry-standard mutation operators
- You're working on production Rust code
- You need detailed mutation reports

**Use PMAT's built-in mutation testing when**:
- You need multi-language support (Python, TypeScript, Go, etc.)
- You want lighter-weight mutation analysis
- You're prototyping or experimenting
- You need custom mutation operators

**Use both when**:
- You want comprehensive coverage analysis
- You're establishing quality baselines
- You're comparing mutation testing approaches

---

## Installation

### Prerequisites

- **PMAT**: v2.181.0 or later
- **Rust**: 1.70.0 or later
- **cargo-mutants**: v24.7.0 or later (v25.3.1 recommended)

### Step 1: Install PMAT

```bash
# From crates.io (recommended)
cargo install pmat

# Or from source
git clone https://github.com/paiml/paiml-mcp-agent-toolkit.git
cd paiml-mcp-agent-toolkit/server
cargo install --path .
```

Verify installation:
```bash
pmat --version
# Expected: pmat 2.181.0 (or later)
```

### Step 2: Install cargo-mutants

```bash
cargo install cargo-mutants
```

Verify installation:
```bash
cargo mutants --version
# Expected: cargo-mutants 25.3.1 (or v24.7.0+)
```

### Step 3: Verify Integration

```bash
# This should detect cargo-mutants
pmat mutate --use-cargo-mutants --help
```

If cargo-mutants is not found, ensure it's in your `PATH`:
```bash
which cargo-mutants
# Expected: /home/user/.cargo/bin/cargo-mutants
```

---

## Quick Start

### Your First Mutation Test

1. **Navigate to your Rust project**:
   ```bash
   cd ~/my-rust-project
   ```

2. **Run mutation testing**:
   ```bash
   pmat mutate --target . --use-cargo-mutants
   ```

3. **Interpret the output**:
   ```
   🧪 cargo-mutants Backend
   ✅ Detected: cargo-mutants 25.3.1
   ✅ Mutation testing complete

   📊 Mutation Testing Results:
      Total mutants: 12
      Caught: 10 (83.3%)
      Missed: 2 (16.7%)
   📈 Mutation Score: 83.3%
   ```

### Understanding the Output

**Mutation Score**: Percentage of mutants caught by your test suite
- **100%**: Perfect test coverage (all mutants caught)
- **80-99%**: Excellent coverage
- **60-79%**: Good coverage
- **< 60%**: Needs improvement

**Mutant Outcomes**:
- **Caught (Killed)**: ✅ Test suite detected the mutant (good!)
- **Missed (Survived)**: ❌ Test suite missed the mutant (test gap!)
- **Timeout**: ⏱️ Tests took too long (increase `--timeout`)
- **Unviable**: 🔧 Mutant didn't compile (not a problem)

### Example: Understanding Results

Given this output:
```
Total mutants: 20
Caught: 16 (80.0%)
Missed: 4 (20.0%)
Timeout: 0 (0.0%)
Unviable: 0 (0.0%)
```

**Interpretation**:
- 80% mutation score is good
- 4 mutants survived, indicating potential test gaps
- No timeouts or compilation issues

**Next Steps**:
1. Review which mutants survived (check cargo-mutants output)
2. Add tests to catch missed mutants
3. Re-run to verify improvement

---

## Advanced Usage

### All Command-Line Flags

```bash
pmat mutate --use-cargo-mutants [OPTIONS]
```

**Available Options**:

| Flag | Description | Default | Example |
|------|-------------|---------|---------|
| `--target <PATH>` | Path to analyze | `.` | `--target src/` |
| `--timeout <SECONDS>` | Test timeout per mutant | 300 | `--timeout 600` |
| `--jobs <N>` | Parallel jobs | Auto | `--jobs 4` |
| `--features <LIST>` | Cargo features to enable | None | `--features "feat1,feat2"` |
| `--all-features` | Enable all features | false | `--all-features` |
| `--no-default-features` | Disable default features | false | `--no-default-features` |
| `--no-shuffle` | Don't shuffle mutants | false | `--no-shuffle` |
| `--output <FILE>` | Save output directory | `mutants.out` | `--output results/` |

### Timeout Configuration

**Why Timeouts Matter**:
Mutants can cause infinite loops or very slow execution. The `--timeout` flag prevents tests from running indefinitely.

**Recommendations**:
- **Fast test suites** (< 5s): `--timeout 60` (1 minute)
- **Medium test suites** (5-30s): `--timeout 300` (5 minutes)
- **Slow test suites** (> 30s): `--timeout 600` (10 minutes)

**Example**:
```bash
# For a slow test suite
pmat mutate --use-cargo-mutants --timeout 900
```

**Timeout Too Low** (bad):
```
Timeout: 8 (40.0%)  # Too many timeouts!
```
→ Increase `--timeout` value

**Timeout Just Right** (good):
```
Timeout: 0 (0.0%)  # No timeouts
```

### Parallel Execution

**Speed Up Mutation Testing with `--jobs`**:

```bash
# Use 4 parallel jobs
pmat mutate --use-cargo-mutants --jobs 4
```

**Recommendations**:
- **Default** (auto-detect): Let cargo-mutants choose based on CPU cores
- **CPU-bound projects**: `--jobs <CPU_CORES>`
- **Memory-limited**: `--jobs 2` (reduce parallelism)
- **CI/CD**: `--jobs 2` (conservative for shared runners)

**Example**:
```bash
# On 8-core machine, use 6 jobs
pmat mutate --use-cargo-mutants --jobs 6
```

### Feature Selection

**Test Specific Features with `--features`**:

```bash
# Test with specific features enabled
pmat mutate --use-cargo-mutants --features "serde,logging"

# Test with all features
pmat mutate --use-cargo-mutants --all-features

# Test with no default features
pmat mutate --use-cargo-mutants --no-default-features
```

**When to Use**:
- **Feature-gated code**: Test each feature combination
- **Optional dependencies**: Validate behavior with/without deps
- **Platform-specific code**: Test platform features

**Example Workflow**:
```bash
# Test default features
pmat mutate --use-cargo-mutants

# Test with "production" feature
pmat mutate --use-cargo-mutants --features "production"

# Test all features
pmat mutate --use-cargo-mutants --all-features
```

### Output Customization

**Save Results for Later Analysis**:

```bash
# Save to custom directory
pmat mutate --use-cargo-mutants --output my-results/

# Results saved to: my-results/outcomes.json
```

**Output Contains**:
- `outcomes.json`: Detailed mutant results
- `mutants.json`: List of generated mutants
- `lock.json`: Execution metadata

**Analyzing Saved Results**:
```bash
# Parse results later
cat my-results/outcomes.json | jq '.outcomes[] | select(.summary == "MissedMutant")'
```

### No Shuffle Mode

**Disable Mutant Shuffling**:

```bash
pmat mutate --use-cargo-mutants --no-shuffle
```

**When to Use**:
- **Reproducible results**: Get same mutant order each run
- **Debugging**: Easier to track specific mutants
- **CI/CD**: Consistent results across runs

**Default Behavior** (shuffled):
```
Mutants tested: 4, 12, 7, 1, 9, ...  # Random order
```

**With `--no-shuffle`**:
```
Mutants tested: 1, 2, 3, 4, 5, ...  # Sequential order
```

---

## Best Practices

### 1. Start Small, Scale Up

**Recommended Workflow**:

```bash
# Step 1: Test a single module first
pmat mutate --target src/core/ --use-cargo-mutants --timeout 300

# Step 2: If successful, test entire project
pmat mutate --target . --use-cargo-mutants --timeout 600

# Step 3: Optimize with parallel execution
pmat mutate --target . --use-cargo-mutants --timeout 600 --jobs 4
```

### 2. Tune Timeout Based on Test Suite Speed

**Measure Your Test Suite**:
```bash
# Time your test suite
time cargo test

# Use 5-10x that duration as timeout
# Example: If tests take 30s, use --timeout 300
```

**Progressive Timeout Tuning**:
1. Start conservative: `--timeout 600`
2. If no timeouts: Reduce to `--timeout 300`
3. If many timeouts: Increase to `--timeout 900`

### 3. Use Feature Flags Strategically

**Test Critical Features First**:
```bash
# Test core functionality
pmat mutate --use-cargo-mutants --features "core"

# Then test optional features
pmat mutate --use-cargo-mutants --features "optional-feature"
```

### 4. Interpret Results in Context

**Good Mutation Scores Vary by Project**:
- **Libraries**: Aim for 90%+ (high test coverage critical)
- **Applications**: 70-80% is good (some code hard to test)
- **Prototypes**: 50%+ acceptable (rapid iteration)

**Focus on Trends**:
```bash
# Week 1: 60% score → Add tests
# Week 2: 75% score → Good progress!
# Week 3: 85% score → Excellent!
```

### 5. Integrate with CI/CD

**GitHub Actions Example**:
```yaml
name: Mutation Testing

on: [push, pull_request]

jobs:
  mutants:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install Tools
        run: |
          cargo install pmat cargo-mutants

      - name: Run Mutation Testing
        run: |
          pmat mutate --use-cargo-mutants --timeout 600 --jobs 2

      - name: Check Score
        run: |
          # Add quality gate logic here
          echo "Mutation testing complete"
```

**GitLab CI Example**:
```yaml
mutation-testing:
  stage: test
  script:
    - cargo install pmat cargo-mutants
    - pmat mutate --use-cargo-mutants --timeout 600 --jobs 2
  allow_failure: true  # Optional: don't fail build on low score
```

### 6. Combine with TDG Scoring

**Comprehensive Quality Analysis**:
```bash
# Step 1: TDG quality baseline
pmat tdg baseline create --output baseline.json

# Step 2: Mutation testing
pmat mutate --use-cargo-mutants

# Step 3: Check for regressions
pmat tdg check-regression --baseline baseline.json
```

### 7. Regular Mutation Testing

**Establish a Cadence**:
- **Daily**: For critical projects
- **Weekly**: For most projects
- **Per-PR**: For high-quality codebases
- **Pre-release**: Mandatory quality gate

---

## Troubleshooting

### Issue 1: cargo-mutants not found

**Symptom**:
```
Error: cargo-mutants not found in PATH
```

**Solution**:
```bash
# Install cargo-mutants
cargo install cargo-mutants

# Verify installation
which cargo-mutants
cargo mutants --version
```

**Still not working?**
```bash
# Add cargo bin to PATH
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

---

### Issue 2: Version too old

**Symptom**:
```
Error: cargo-mutants version 24.5.0 is too old (required: 24.7.0+)
```

**Solution**:
```bash
# Upgrade cargo-mutants
cargo install cargo-mutants --force

# Verify new version
cargo mutants --version
# Expected: cargo-mutants 25.3.1
```

---

### Issue 3: Timeout errors

**Symptom**:
```
📊 Mutation Testing Results:
   Total mutants: 20
   Caught: 8 (40.0%)
   Timeout: 10 (50.0%)  # Too many timeouts!
```

**Solution**:
```bash
# Increase timeout (from 300s to 600s)
pmat mutate --use-cargo-mutants --timeout 600
```

**Still timing out?**
```bash
# Try even longer timeout
pmat mutate --use-cargo-mutants --timeout 1200

# Or reduce parallelism
pmat mutate --use-cargo-mutants --timeout 600 --jobs 1
```

---

### Issue 4: No mutants found

**Symptom**:
```
📊 Mutation Testing Results:
   Total mutants: 0
```

**Possible Causes**:
1. **No test coverage**: cargo-mutants needs tests to run
2. **Excluded files**: Files may be in `.cargo-mutants.toml` exclude list
3. **Wrong directory**: Not in a Rust project root

**Solutions**:
```bash
# Verify you have tests
cargo test

# Check you're in project root (has Cargo.toml)
ls Cargo.toml

# Specify correct target
pmat mutate --target path/to/src --use-cargo-mutants
```

---

### Issue 5: Parse errors

**Symptom**:
```
Error: Failed to parse outcomes.json: unexpected format
```

**Cause**: cargo-mutants version mismatch

**Solution**:
```bash
# Upgrade to supported version (v24.7.0+)
cargo install cargo-mutants --force

# Verify version
cargo mutants --version
```

---

### Issue 6: Permission errors

**Symptom**:
```
Error: Failed to write output: Permission denied
```

**Solution**:
```bash
# Check directory permissions
ls -la mutants.out/

# Change output location
pmat mutate --use-cargo-mutants --output ~/my-results/

# Or fix permissions
chmod 755 mutants.out/
```

---

### Issue 7: Out of memory

**Symptom**:
```
Error: Cannot allocate memory
# Or: Killed (process terminated)
```

**Solution**:
```bash
# Reduce parallelism
pmat mutate --use-cargo-mutants --jobs 1

# Or run in batches (future feature)
```

---

## FAQ

### Q: What's the difference between cargo-mutants and PMAT's built-in mutation testing?

**A**:
- **cargo-mutants**: Industry-standard Rust mutation testing, comprehensive operators
- **PMAT built-in**: Multi-language support, lighter-weight, experimental
- **Use cargo-mutants** for production Rust code
- **Use built-in** for multi-language projects

---

### Q: How long does mutation testing take?

**A**: Depends on:
- Number of mutants generated (usually 1-100 per file)
- Test suite execution time
- Parallelism (`--jobs`)

**Estimate**: `(num_mutants × test_suite_time) / jobs`

Example:
- 50 mutants
- 10s test suite
- 4 parallel jobs
- **Time**: `(50 × 10s) / 4 = 125s (~2 minutes)`

---

### Q: What's a good mutation score?

**A**:
- **90%+**: Excellent
- **70-89%**: Good
- **50-69%**: Acceptable
- **< 50%**: Needs improvement

**Context matters**: A library needs higher scores than an application.

---

### Q: Can I exclude files from mutation testing?

**A**: Yes, via cargo-mutants configuration:

Create `.cargo-mutants.toml`:
```toml
exclude = [
    "tests/**",
    "benches/**",
    "examples/**"
]
```

---

### Q: Does mutation testing modify my source code?

**A**:
- **During testing**: Yes, temporarily (cargo-mutants creates copies)
- **After testing**: No, original code is untouched
- **Version control**: No changes committed

---

### Q: Can I run mutation testing in CI/CD?

**A**: Yes! See [Best Practices → Integrate with CI/CD](#5-integrate-with-cicd)

---

### Q: What if my project is too large?

**A**: Strategies:
1. Test modules incrementally: `--target src/core/`
2. Reduce parallelism: `--jobs 1`
3. Split into multiple runs (manual batching)
4. Use cargo-mutants exclusions (`.cargo-mutants.toml`)

---

### Q: How do I see which mutants survived?

**A**: Check cargo-mutants output directory:

```bash
# View all outcomes
cat mutants.out/outcomes.json | jq

# Show only missed mutants
cat mutants.out/outcomes.json | jq '.outcomes[] | select(.summary == "MissedMutant")'
```

---

## Additional Resources

- **cargo-mutants Documentation**: https://mutants.rs/
- **PMAT Documentation**: https://paiml.github.io/pmat-book/
- **Mutation Testing Concepts**: https://en.wikipedia.org/wiki/Mutation_testing
- **PMAT GitHub**: https://github.com/paiml/paiml-mcp-agent-toolkit

---

## Getting Help

- **GitHub Issues**: https://github.com/paiml/paiml-mcp-agent-toolkit/issues
- **Discussions**: https://github.com/paiml/paiml-mcp-agent-toolkit/discussions
- **Discord**: [PAIML Community](https://discord.gg/paiml) (coming soon)

---

**Last Updated**: October 29, 2025
**Version**: PMAT v2.181.0
**cargo-mutants Version**: v25.3.1
