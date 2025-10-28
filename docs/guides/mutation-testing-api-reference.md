# Mutation Testing API Reference

Complete reference for PMAT mutation testing command-line interface.

## Table of Contents

- [Command Overview](#command-overview)
- [Flags and Options](#flags-and-options)
- [Exit Codes](#exit-codes)
- [Output Formats](#output-formats)
- [Environment Variables](#environment-variables)
- [Examples](#examples)
- [Language Support](#language-support)
- [Mutation Operators](#mutation-operators)

---

## Command Overview

### Basic Syntax

```bash
pmat mutate --target <FILE_OR_DIR> [OPTIONS]
```

### Description

Performs mutation testing on source code by introducing small, deliberate changes (mutations) and verifying that the test suite detects them. Returns a mutation score indicating test quality.

**Mutation Score** = (Killed Mutants / Total Valid Mutants) × 100%

### Quick Examples

```bash
# Single file
pmat mutate --target src/lib.rs

# Directory with threshold
pmat mutate --target src/ --threshold 85

# JSON output with failures only
pmat mutate --target src/ --output-format json --failures-only
```

---

## Flags and Options

### Required Flags

#### `--target <PATH>`

**Type**: String (file or directory path)

**Description**: Target source file or directory for mutation testing. If directory, all supported language files will be tested recursively.

**Examples**:
```bash
# Single file
pmat mutate --target src/calculator.rs

# Directory (recursive)
pmat mutate --target src/

# Specific module
pmat mutate --target src/auth/session.ts
```

**Notes**:
- Path must exist (command fails with error if not found)
- Directory testing is recursive
- Hidden files (starting with `.`) are skipped
- Test files are automatically excluded from mutation

---

### Output Control

#### `--output-format <FORMAT>`

**Type**: String

**Default**: `text`

**Values**: `text`, `json`, `markdown`

**Description**: Controls output format for mutation testing results.

**Examples**:
```bash
# Default text output (color-coded)
pmat mutate --target src/ --output-format text

# JSON for CI/CD integration
pmat mutate --target src/ --output-format json > results.json

# Markdown for pull request comments
pmat mutate --target src/ --output-format markdown > REPORT.md
```

**Output Schemas**:

**Text Format** (default):
```
Mutation Testing Results
========================
Total Mutants: 50
Killed: 45 (90.0%)
Survived: 3 (6.0%)
Compile Errors: 2 (4.0%)

Mutation Score: 90.0%

✓ Killed: src/lib.rs:10:5 - Changed + to -
✗ Survived: src/lib.rs:15:9 - Changed > to >=
```

**JSON Format**:
```json
{
  "mutation_score": 90.0,
  "total_mutants": 50,
  "killed": 45,
  "survived": 3,
  "compile_errors": 2,
  "timeouts": 0,
  "mutants": [
    {
      "location": "src/lib.rs:10:5",
      "mutation": "Changed + to -",
      "status": "Killed",
      "original_code": "a + b",
      "mutated_code": "a - b"
    }
  ]
}
```

**Markdown Format**:
```markdown
# Mutation Testing Report

**Mutation Score**: 90.0%

## Summary

| Metric | Count | Percentage |
|--------|-------|------------|
| Total Mutants | 50 | 100% |
| Killed | 45 | 90.0% |
| Survived | 3 | 6.0% |

## Survived Mutants

| Location | Mutation | Code |
|----------|----------|------|
| src/lib.rs:15:9 | Changed > to >= | `if a > b` |
```

---

#### `--failures-only`

**Type**: Flag (boolean)

**Default**: `false`

**Description**: Show only survived mutants (failures). Omits killed mutants from output to reduce noise.

**Examples**:
```bash
# Show only survived mutants
pmat mutate --target src/ --failures-only

# Combine with JSON for CI/CD
pmat mutate --target src/ --output-format json --failures-only > failures.json
```

**Use Cases**:
- CI/CD pipelines (reduce log size)
- Large codebases (focus on problems)
- Pull request reviews (highlight gaps)

**Output Comparison**:

**Without `--failures-only`** (all mutants):
```
Total: 50 mutants
Killed: 45 (shown)
Survived: 3 (shown)
Compile Errors: 2 (shown)
```

**With `--failures-only`** (survivors only):
```
Total: 50 mutants
Killed: 45 (omitted)
Survived: 3 (shown)
Compile Errors: 2 (omitted)
```

---

### Quality Gates

#### `--threshold <PERCENTAGE>`

**Type**: Float (0.0 - 100.0)

**Default**: None (no threshold enforcement)

**Description**: Minimum mutation score required for command to succeed. If score is below threshold, command exits with error code 1.

**Examples**:
```bash
# Require 85% mutation score
pmat mutate --target src/ --threshold 85

# Require 90% for critical code
pmat mutate --target src/auth/ --threshold 90

# Combined with failures-only for CI/CD
pmat mutate --target src/ --threshold 80 --failures-only
```

**Exit Behavior**:
```bash
# Mutation score: 92% (above threshold 85%)
pmat mutate --target src/ --threshold 85
echo $?  # Output: 0 (success)

# Mutation score: 78% (below threshold 85%)
pmat mutate --target src/ --threshold 85
echo $?  # Output: 1 (failure)
```

**CI/CD Integration**:
```yaml
# GitHub Actions
- name: Mutation testing quality gate
  run: pmat mutate --target src/ --threshold 85
  # Workflow fails if mutation score < 85%
```

**Recommended Thresholds**:
- **Authentication/Security**: 95-100%
- **Business Logic**: 85-95%
- **Utilities**: 80-85%
- **UI Components**: 70-80%

---

### Performance Control

#### `--jobs <NUMBER>`

**Type**: Integer (1 to CPU core count)

**Default**: Number of CPU cores

**Description**: Number of parallel threads for mutation testing. Higher values speed up testing but increase CPU/memory usage.

**Examples**:
```bash
# Use 4 parallel jobs
pmat mutate --target src/ --jobs 4

# Single-threaded (no parallelism)
pmat mutate --target src/ --jobs 1

# Use all available cores (default)
pmat mutate --target src/
```

**Performance Impact**:

| Jobs | Test Time | CPU Usage | Memory Usage |
|------|-----------|-----------|--------------|
| 1    | 10 minutes | 100% (1 core) | 500 MB |
| 4    | 3 minutes | 400% (4 cores) | 1.2 GB |
| 8    | 2 minutes | 800% (8 cores) | 2.0 GB |

**Recommendations**:
- **Small projects** (<1000 LOC): `--jobs 2`
- **Medium projects** (1000-10000 LOC): `--jobs 4`
- **Large projects** (>10000 LOC): `--jobs 8`
- **Memory-constrained** (CI/CD): `--jobs 1` or `--jobs 2`

**CI/CD Example**:
```bash
# GitHub Actions runners have 2 cores
pmat mutate --target src/ --jobs 2
```

---

#### `--timeout <SECONDS>`

**Type**: Integer (seconds)

**Default**: `60`

**Description**: Maximum time (in seconds) for each mutant's test run. If tests exceed timeout, mutant is marked as "Timeout".

**Examples**:
```bash
# 10-second timeout per mutant
pmat mutate --target src/ --timeout 10

# 2-minute timeout for slow tests
pmat mutate --target src/ --timeout 120

# No timeout (not recommended)
pmat mutate --target src/ --timeout 0
```

**Use Cases**:

**Fast Test Suites** (<5 seconds):
```bash
pmat mutate --target src/ --timeout 10
```

**Slow Test Suites** (30+ seconds):
```bash
pmat mutate --target src/ --timeout 90
```

**Preventing Infinite Loops**:
```rust
// Original code
while condition {
    do_work();
}

// Mutation: condition -> true (infinite loop)
while true {  // Mutation causes infinite loop
    do_work();
}
```
Timeout prevents infinite loops from hanging mutation testing.

**Recommendations**:
- Set timeout to **2-3× average test runtime**
- Example: If tests take 10 seconds → `--timeout 30`
- CI/CD: Use shorter timeouts to avoid hanging builds

---

### Language Control

#### `--language <LANGUAGE>`

**Type**: String

**Default**: Auto-detect from file extension

**Values**: `rust`, `python`, `typescript`, `javascript`, `go`, `cpp`

**Description**: Override automatic language detection. Useful for files with non-standard extensions.

**Examples**:
```bash
# Auto-detect (default)
pmat mutate --target src/lib.rs  # Detected as Rust

# Force Python for .txt file
pmat mutate --target script.txt --language python

# Force TypeScript for .mjs file
pmat mutate --target module.mjs --language typescript
```

**Auto-Detection Table**:

| Language   | Extensions | Auto-Detected |
|------------|------------|---------------|
| Rust       | `.rs`      | ✅ |
| Python     | `.py`      | ✅ |
| TypeScript | `.ts`, `.tsx` | ✅ |
| JavaScript | `.js`, `.jsx` | ✅ |
| Go         | `.go`      | ✅ |
| C++        | `.cpp`, `.cc`, `.cxx`, `.hpp` | ✅ |

**When to Override**:
- Non-standard file extensions
- Polyglot files with mixed syntax
- Testing specific language parsers

---

## Exit Codes

PMAT mutation testing command returns specific exit codes for automation.

### Exit Code Reference

| Code | Status | Description |
|------|--------|-------------|
| `0` | Success | Mutation testing completed, score meets threshold (if set) |
| `1` | Failure | Mutation score below threshold, or command error |
| `2` | Invalid Arguments | Missing required flags or invalid options |

### Examples

#### Exit Code 0 (Success)
```bash
pmat mutate --target src/ --threshold 85
# Mutation score: 90% (above 85%)
echo $?  # Output: 0
```

#### Exit Code 1 (Below Threshold)
```bash
pmat mutate --target src/ --threshold 85
# Mutation score: 75% (below 85%)
echo $?  # Output: 1
```

#### Exit Code 2 (Invalid Arguments)
```bash
pmat mutate --target nonexistent.rs
# Error: File not found
echo $?  # Output: 2
```

### CI/CD Usage

```bash
#!/bin/bash
pmat mutate --target src/ --threshold 85
EXIT_CODE=$?

if [ $EXIT_CODE -eq 0 ]; then
    echo "✓ Mutation testing passed"
    exit 0
elif [ $EXIT_CODE -eq 1 ]; then
    echo "✗ Mutation score below threshold"
    exit 1
else
    echo "✗ Command error"
    exit 2
fi
```

---

## Output Formats

Detailed specifications for each output format.

### Text Format (Default)

**Characteristics**:
- Human-readable
- Color-coded (green: killed, red: survived, yellow: errors)
- Suitable for terminal output

**Example**:
```
Mutation Testing Results
========================
Total Mutants: 50
Killed: 45 (90.0%)
Survived: 3 (6.0%)
Compile Errors: 2 (4.0%)
Timeouts: 0 (0.0%)

Mutation Score: 90.0%

Survived Mutants:
-----------------
1. src/lib.rs:15:9 - Changed > to >=
   Original: if a > b
   Mutated:  if a >= b

2. src/lib.rs:22:5 - Changed + to -
   Original: return a + b
   Mutated:  return a - b

3. src/lib.rs:30:12 - Changed && to ||
   Original: if x > 0 && y > 0
   Mutated:  if x > 0 || y > 0
```

---

### JSON Format

**Characteristics**:
- Machine-readable
- Suitable for CI/CD integration
- Structured data for programmatic analysis

**Schema**:
```json
{
  "mutation_score": 90.0,
  "total_mutants": 50,
  "killed": 45,
  "survived": 3,
  "compile_errors": 2,
  "timeouts": 0,
  "timestamp": "2025-10-28T14:30:00Z",
  "target": "src/",
  "mutants": [
    {
      "id": 1,
      "location": {
        "file": "src/lib.rs",
        "line": 15,
        "column": 9
      },
      "mutation": {
        "type": "RelationalOperator",
        "from": ">",
        "to": ">="
      },
      "status": "Survived",
      "original_code": "if a > b",
      "mutated_code": "if a >= b",
      "execution_time_ms": 125
    }
  ]
}
```

**Field Descriptions**:

| Field | Type | Description |
|-------|------|-------------|
| `mutation_score` | Float | Overall mutation score (0.0-100.0) |
| `total_mutants` | Integer | Total number of mutants generated |
| `killed` | Integer | Number of mutants detected by tests |
| `survived` | Integer | Number of mutants not detected |
| `compile_errors` | Integer | Mutants causing compilation errors |
| `timeouts` | Integer | Mutants exceeding timeout |
| `timestamp` | String (ISO 8601) | When mutation testing started |
| `target` | String | Target file/directory path |
| `mutants[]` | Array | Detailed information per mutant |
| `mutants[].id` | Integer | Sequential mutant identifier |
| `mutants[].location` | Object | File, line, column of mutation |
| `mutants[].mutation` | Object | Mutation type and transformation |
| `mutants[].status` | String | "Killed", "Survived", "CompileError", "Timeout" |
| `mutants[].original_code` | String | Original source code |
| `mutants[].mutated_code` | String | Mutated source code |
| `mutants[].execution_time_ms` | Integer | Test execution time in milliseconds |

**CI/CD Parsing Example**:
```bash
# Extract mutation score
pmat mutate --target src/ --output-format json > results.json
SCORE=$(jq '.mutation_score' results.json)
echo "Mutation Score: $SCORE%"

# Count survived mutants
SURVIVED=$(jq '.survived' results.json)
echo "Survived Mutants: $SURVIVED"

# List survived mutant locations
jq -r '.mutants[] | select(.status == "Survived") | .location.file + ":" + (.location.line | tostring)' results.json
```

---

### Markdown Format

**Characteristics**:
- Human-readable documentation
- Suitable for pull request comments
- GitHub/GitLab compatible tables

**Example**:
```markdown
# Mutation Testing Report

**Mutation Score**: 90.0%

**Generated**: 2025-10-28 14:30:00 UTC

**Target**: `src/`

---

## Summary

| Metric | Count | Percentage |
|--------|-------|------------|
| Total Mutants | 50 | 100.0% |
| Killed | 45 | 90.0% |
| Survived | 3 | 6.0% |
| Compile Errors | 2 | 4.0% |
| Timeouts | 0 | 0.0% |

---

## Survived Mutants

### 1. `src/lib.rs:15:9`

**Mutation**: Changed `>` to `>=`

**Original Code**:
```rust
if a > b
```

**Mutated Code**:
```rust
if a >= b
```

**Status**: Survived

---

### 2. `src/lib.rs:22:5`

**Mutation**: Changed `+` to `-`

**Original Code**:
```rust
return a + b
```

**Mutated Code**:
```rust
return a - b
```

**Status**: Survived
```

**GitHub PR Comment Example**:
```bash
# Generate markdown report
pmat mutate --target src/ --output-format markdown > mutation_report.md

# Comment on PR via GitHub API
gh pr comment $PR_NUMBER --body-file mutation_report.md
```

---

## Environment Variables

PMAT mutation testing respects the following environment variables.

### `PMAT_MUTATION_TIMEOUT`

**Type**: Integer (seconds)

**Description**: Default timeout for mutation testing (overridden by `--timeout` flag).

**Example**:
```bash
export PMAT_MUTATION_TIMEOUT=120
pmat mutate --target src/  # Uses 120-second timeout
```

---

### `PMAT_MUTATION_JOBS`

**Type**: Integer

**Description**: Default number of parallel jobs (overridden by `--jobs` flag).

**Example**:
```bash
export PMAT_MUTATION_JOBS=4
pmat mutate --target src/  # Uses 4 parallel jobs
```

---

### `PMAT_LOG_LEVEL`

**Type**: String

**Values**: `error`, `warn`, `info`, `debug`, `trace`

**Description**: Controls verbosity of PMAT logging.

**Example**:
```bash
export PMAT_LOG_LEVEL=debug
pmat mutate --target src/  # Shows detailed debug logs
```

---

## Examples

### Basic Usage

#### Single File Mutation Testing
```bash
pmat mutate --target src/calculator.rs
```

#### Directory Mutation Testing
```bash
pmat mutate --target src/
```

#### With Threshold
```bash
pmat mutate --target src/ --threshold 85
```

---

### CI/CD Integration

#### GitHub Actions Workflow
```yaml
name: Mutation Testing
on: [push, pull_request]

jobs:
  mutation-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install PMAT
        run: cargo install pmat

      - name: Run mutation tests
        run: |
          pmat mutate \
            --target src/ \
            --threshold 85 \
            --failures-only \
            --output-format json \
            > mutation-results.json

      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: mutation-results
          path: mutation-results.json
```

#### GitLab CI Pipeline
```yaml
mutation-test:
  stage: test
  image: rust:latest
  script:
    - cargo install pmat
    - pmat mutate --target src/ --threshold 85 --output-format json > mutation.json
  artifacts:
    paths:
      - mutation.json
    reports:
      junit: mutation.json
```

#### Jenkins Declarative Pipeline
```groovy
pipeline {
    agent { docker { image 'rust:latest' } }

    stages {
        stage('Mutation Testing') {
            steps {
                sh 'cargo install pmat'
                sh '''
                    pmat mutate \
                      --target src/ \
                      --threshold 85 \
                      --output-format json \
                      > mutation-results.json
                '''
            }
        }
    }

    post {
        always {
            archiveArtifacts artifacts: 'mutation-results.json'
        }
    }
}
```

---

### Advanced Patterns

#### Differential Mutation Testing (PR Changes Only)
```bash
# Get list of changed files in pull request
CHANGED_FILES=$(git diff --name-only origin/main...HEAD | grep '\.rs$')

# Run mutation testing only on changed files
for file in $CHANGED_FILES; do
    pmat mutate --target "$file" --failures-only
done
```

#### Parallel Testing with Custom Job Count
```bash
# Use 4 parallel jobs for faster testing
pmat mutate --target src/ --jobs 4 --timeout 30
```

#### JSON Report with Post-Processing
```bash
# Generate JSON report
pmat mutate --target src/ --output-format json > results.json

# Extract key metrics
SCORE=$(jq '.mutation_score' results.json)
SURVIVED=$(jq '.survived' results.json)

# Send to monitoring system
curl -X POST https://metrics.example.com/mutation \
  -H "Content-Type: application/json" \
  -d "{\"score\": $SCORE, \"survived\": $SURVIVED}"
```

#### Pre-Commit Hook
```bash
#!/bin/bash
# .git/hooks/pre-commit

echo "Running mutation testing on staged files..."

# Get staged Rust files
STAGED_FILES=$(git diff --cached --name-only --diff-filter=ACM | grep '\.rs$')

if [ -z "$STAGED_FILES" ]; then
    echo "No Rust files staged, skipping mutation testing"
    exit 0
fi

# Run mutation testing
for file in $STAGED_FILES; do
    echo "Testing $file..."
    pmat mutate --target "$file" --threshold 80 --failures-only

    if [ $? -ne 0 ]; then
        echo "❌ Mutation testing failed for $file"
        exit 1
    fi
done

echo "✅ All mutation tests passed"
exit 0
```

---

## Language Support

PMAT automatically detects language from file extensions.

### Supported Languages

| Language | Extensions | Mutation Operators | Auto-Detect |
|----------|------------|-------------------|-------------|
| **Rust** | `.rs` | Arithmetic, Comparison, Logical, Boundary | ✅ |
| **Python** | `.py` | Arithmetic, Comparison, Logical, Return | ✅ |
| **TypeScript** | `.ts`, `.tsx` | Arithmetic, Comparison, Logical, Optional | ✅ |
| **JavaScript** | `.js`, `.jsx` | Arithmetic, Comparison, Logical, Nullish | ✅ |
| **Go** | `.go` | Arithmetic, Comparison, Logical, Error | ✅ |
| **C++** | `.cpp`, `.cc`, `.cxx`, `.hpp` | Arithmetic, Comparison, Logical, Pointer | ✅ |

### Language-Specific Examples

#### Rust
```bash
pmat mutate --target src/lib.rs --threshold 90
```

#### Python
```bash
pmat mutate --target src/calculator.py --threshold 85
```

#### TypeScript
```bash
pmat mutate --target src/app.ts --threshold 80
```

#### JavaScript
```bash
pmat mutate --target src/utils.js --threshold 80
```

#### Go
```bash
pmat mutate --target main.go --threshold 85
```

#### C++
```bash
pmat mutate --target src/main.cpp --threshold 85
```

---

## Mutation Operators

PMAT applies various mutation operators based on language.

### Arithmetic Operators

| Original | Mutations |
|----------|-----------|
| `+` | `-`, `*`, `/`, `%` |
| `-` | `+`, `*`, `/`, `%` |
| `*` | `+`, `-`, `/`, `%` |
| `/` | `+`, `-`, `*`, `%` |
| `%` | `+`, `-`, `*`, `/` |

**Example**:
```rust
// Original
let sum = a + b;

// Mutations
let sum = a - b;  // Mutation 1
let sum = a * b;  // Mutation 2
let sum = a / b;  // Mutation 3
```

---

### Comparison Operators

| Original | Mutations |
|----------|-----------|
| `>` | `>=`, `<`, `<=`, `==`, `!=` |
| `>=` | `>`, `<`, `<=`, `==`, `!=` |
| `<` | `<=`, `>`, `>=`, `==`, `!=` |
| `<=` | `<`, `>`, `>=`, `==`, `!=` |
| `==` | `!=`, `<`, `>`, `<=`, `>=` |
| `!=` | `==`, `<`, `>`, `<=`, `>=` |

**Example**:
```rust
// Original
if a > b {
    return a;
}

// Mutations
if a >= b { return a; }  // Mutation 1
if a < b { return a; }   // Mutation 2
if a == b { return a; }  // Mutation 3
```

---

### Logical Operators

| Original | Mutations |
|----------|-----------|
| `&&` | `\|\|` |
| `\|\|` | `&&` |
| `!` | (removed) |

**Example**:
```rust
// Original
if x > 0 && y > 0 {
    return true;
}

// Mutation
if x > 0 || y > 0 {  // Changed && to ||
    return true;
}
```

---

### Boundary Mutations

| Original | Mutations |
|----------|-----------|
| `0` | `1`, `-1` |
| `i++` | `i--`, `++i`, `--i` |
| `i--` | `i++`, `++i`, `--i` |

**Example**:
```rust
// Original
for i in 0..n {
    sum += i;
}

// Mutation
for i in 1..n {  // Changed 0 to 1
    sum += i;
}
```

---

### Return Value Mutations

| Original | Mutations |
|----------|-----------|
| `return x` | `return 0`, `return null`, `return -1` |
| `return true` | `return false` |
| `return false` | `return true` |

**Example**:
```rust
// Original
fn is_positive(x: i32) -> bool {
    return x > 0;
}

// Mutation
fn is_positive(x: i32) -> bool {
    return false;  // Changed return value
}
```

---

## Additional Resources

- **User Guide**: [Mutation Testing with PMAT](./mutation-testing.md)
- **Best Practices**: [Mutation Testing Best Practices](./mutation-testing-best-practices.md)
- **CI/CD Integration**:
  - [GitHub Actions Integration](../ci-cd/github-actions-integration.md)
  - [GitLab CI Integration](../ci-cd/gitlab-ci-integration.md)
  - [Jenkins Integration](../ci-cd/jenkins-integration.md)
- **Example Projects**:
  - [Rust Mutation Testing Example](../../examples/rust-mutation-testing/)
  - [Python Mutation Testing Example](../../examples/python-mutation-testing/)
  - [TypeScript Mutation Testing Example](../../examples/typescript-mutation-testing/)

---

**Version**: v2.177.0
**Last Updated**: October 28, 2025
**Sprint**: Sprint 64 Day 3
