# Cargo-Mutants Integration: Practical Examples

**Feature**: `pmat mutate --use-cargo-mutants`
**Audience**: Rust developers
**Level**: Beginner to Advanced

---

## Table of Contents

1. [Basic Examples](#basic-examples)
2. [Timeout Configuration](#timeout-configuration)
3. [Parallel Execution](#parallel-execution)
4. [Feature Selection](#feature-selection)
5. [Output Customization](#output-customization)
6. [CI/CD Integration](#cicd-integration)
7. [Real-World Workflows](#real-world-workflows)

---

## Basic Examples

### Example 1: Simplest Usage

**Scenario**: First time running mutation testing on a Rust project

```bash
# Navigate to project
cd ~/my-rust-project

# Run mutation testing
pmat mutate --target . --use-cargo-mutants
```

**Expected Output**:
```
🧪 cargo-mutants Backend
✅ Detected: cargo-mutants 25.3.1
⏳ Running mutation testing...
✅ Mutation testing complete

📊 Mutation Testing Results:
   Total mutants: 15
   Caught: 12 (80.0%)
   Missed: 3 (20.0%)
   Timeout: 0 (0.0%)
   Unviable: 0 (0.0%)
📈 Mutation Score: 80.0%
```

**Interpretation**: 80% of mutants were caught, indicating good test coverage. 3 mutants survived, suggesting potential test gaps.

---

### Example 2: Test a Specific Module

**Scenario**: Focus mutation testing on a critical module

```bash
# Test only the core module
pmat mutate --target src/core/ --use-cargo-mutants
```

**Why**: Faster iteration when working on specific code areas.

---

### Example 3: Explicit Target Path

**Scenario**: Test from outside the project directory

```bash
# From parent directory
pmat mutate --target ~/projects/my-rust-app --use-cargo-mutants
```

---

## Timeout Configuration

### Example 4: Default Timeout

**Scenario**: Fast test suite (< 5 seconds)

```bash
# Use default timeout (300s = 5 minutes)
pmat mutate --use-cargo-mutants
```

**When to Use**: For small projects with fast tests.

---

### Example 5: Increased Timeout

**Scenario**: Slow test suite (30+ seconds)

```bash
# Increase timeout to 10 minutes
pmat mutate --use-cargo-mutants --timeout 600
```

**Expected Outcome**: Fewer timeout failures.

**Before** (timeout too low):
```
📊 Mutation Testing Results:
   Total mutants: 20
   Caught: 8 (40.0%)
   Timeout: 10 (50.0%)  ← Too many timeouts!
```

**After** (timeout increased):
```
📊 Mutation Testing Results:
   Total mutants: 20
   Caught: 16 (80.0%)
   Timeout: 0 (0.0%)    ← Much better!
```

---

### Example 6: Very Long Timeout

**Scenario**: Integration tests or end-to-end tests

```bash
# Allow up to 20 minutes per mutant
pmat mutate --use-cargo-mutants --timeout 1200
```

**When to Use**: Projects with slow integration tests.

---

## Parallel Execution

### Example 7: Specify Job Count

**Scenario**: Speed up mutation testing with parallelism

```bash
# Use 4 parallel jobs
pmat mutate --use-cargo-mutants --jobs 4
```

**Performance Comparison**:
- **Sequential** (`--jobs 1`): 50 mutants × 10s = 500s (~8 minutes)
- **Parallel** (`--jobs 4`): 50 mutants × 10s / 4 = 125s (~2 minutes)

**Performance Gain**: 4x speedup!

---

### Example 8: Auto-Detect CPU Cores

**Scenario**: Let cargo-mutants choose optimal parallelism

```bash
# Omit --jobs flag (auto-detect)
pmat mutate --use-cargo-mutants
```

**Default Behavior**: Uses number of CPU cores.

---

### Example 9: Conservative Parallelism

**Scenario**: Memory-limited environment or CI/CD

```bash
# Use only 2 jobs to conserve resources
pmat mutate --use-cargo-mutants --jobs 2
```

**When to Use**: Shared CI runners, low-memory machines.

---

## Feature Selection

### Example 10: Enable Specific Features

**Scenario**: Test code behind feature flags

```bash
# Test with "serde" and "logging" features enabled
pmat mutate --use-cargo-mutants --features "serde,logging"
```

**Project Context**:
```toml
# Cargo.toml
[features]
serde = ["dep:serde"]
logging = ["dep:log"]
```

---

### Example 11: Test All Features

**Scenario**: Comprehensive testing across all feature combinations

```bash
# Enable all features
pmat mutate --use-cargo-mutants --all-features
```

**When to Use**: Before releases to ensure all features are tested.

---

### Example 12: No Default Features

**Scenario**: Test minimal build (no default features)

```bash
# Disable default features
pmat mutate --use-cargo-mutants --no-default-features
```

**Project Context**:
```toml
# Cargo.toml
[features]
default = ["std", "alloc"]
std = []
alloc = []
```

---

### Example 13: Combine Feature Flags

**Scenario**: Test specific feature combination

```bash
# No defaults, but enable specific features
pmat mutate --use-cargo-mutants \
  --no-default-features \
  --features "minimal,logging"
```

---

## Output Customization

### Example 14: Custom Output Directory

**Scenario**: Save results to specific location

```bash
# Save to custom directory
pmat mutate --use-cargo-mutants --output results/mutation-$(date +%Y%m%d)
```

**Result**:
```
results/mutation-20251029/
├── outcomes.json
├── mutants.json
└── lock.json
```

---

### Example 15: Preserve Multiple Runs

**Scenario**: Compare results across runs

```bash
# Run 1: Before changes
pmat mutate --use-cargo-mutants --output baseline-results/

# Make code changes...

# Run 2: After changes
pmat mutate --use-cargo-mutants --output updated-results/

# Compare
diff baseline-results/outcomes.json updated-results/outcomes.json
```

---

## CI/CD Integration

### Example 16: GitHub Actions Workflow

**File**: `.github/workflows/mutation-testing.yml`

```yaml
name: Mutation Testing

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  mutation-testing:
    runs-on: ubuntu-latest

    steps:
      - name: Checkout code
        uses: actions/checkout@v3

      - name: Install Rust toolchain
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true

      - name: Cache cargo registry
        uses: actions/cache@v3
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}

      - name: Install PMAT and cargo-mutants
        run: |
          cargo install pmat cargo-mutants

      - name: Run mutation testing
        run: |
          pmat mutate \
            --target . \
            --use-cargo-mutants \
            --timeout 600 \
            --jobs 2

      - name: Upload results
        uses: actions/upload-artifact@v3
        if: always()
        with:
          name: mutation-results
          path: mutants.out/
```

**What This Does**:
1. Installs Rust toolchain
2. Caches cargo dependencies (faster builds)
3. Installs PMAT and cargo-mutants
4. Runs mutation testing with 10-minute timeout and 2 parallel jobs
5. Uploads results as artifact (available for download)

---

### Example 17: GitHub Actions with Quality Gate

**File**: `.github/workflows/mutation-gate.yml`

```yaml
name: Mutation Quality Gate

on:
  pull_request:
    branches: [main]

jobs:
  mutation-gate:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Install tools
        run: |
          cargo install pmat cargo-mutants jq

      - name: Run mutation testing
        id: mutants
        run: |
          pmat mutate \
            --use-cargo-mutants \
            --timeout 600 \
            --output results/ > output.txt

          cat output.txt

      - name: Parse mutation score
        id: score
        run: |
          # Extract score from output (example parsing)
          SCORE=$(grep "Mutation Score:" output.txt | awk '{print $NF}' | tr -d '%')
          echo "score=$SCORE" >> $GITHUB_OUTPUT
          echo "Mutation Score: $SCORE%"

      - name: Check quality gate
        run: |
          SCORE=${{ steps.score.outputs.score }}
          MIN_SCORE=70

          if (( $(echo "$SCORE < $MIN_SCORE" | bc -l) )); then
            echo "❌ Mutation score $SCORE% is below minimum $MIN_SCORE%"
            exit 1
          else
            echo "✅ Mutation score $SCORE% meets minimum $MIN_SCORE%"
          fi
```

**Quality Gate**: Fails PR if mutation score < 70%

---

### Example 18: GitLab CI Pipeline

**File**: `.gitlab-ci.yml`

```yaml
stages:
  - test
  - quality

mutation-testing:
  stage: quality
  image: rust:latest

  before_script:
    - cargo install pmat cargo-mutants

  script:
    - |
      pmat mutate \
        --use-cargo-mutants \
        --timeout 600 \
        --jobs 2 \
        --output $CI_PROJECT_DIR/mutation-results/

  artifacts:
    paths:
      - mutation-results/
    expire_in: 1 week

  allow_failure: true  # Don't block pipeline on low score

  only:
    - main
    - merge_requests
```

---

### Example 19: Jenkins Pipeline

**File**: `Jenkinsfile`

```groovy
pipeline {
    agent any

    stages {
        stage('Setup') {
            steps {
                sh 'cargo install pmat cargo-mutants'
            }
        }

        stage('Mutation Testing') {
            steps {
                sh '''
                    pmat mutate \
                        --use-cargo-mutants \
                        --timeout 600 \
                        --jobs ${env.BUILD_NUMBER % 4 + 1} \
                        --output results/
                '''
            }
        }

        stage('Archive Results') {
            steps {
                archiveArtifacts artifacts: 'results/**/*', fingerprint: true
            }
        }
    }

    post {
        always {
            cleanWs()
        }
    }
}
```

---

## Real-World Workflows

### Example 20: Pre-Release Checklist

**Scenario**: Comprehensive quality check before release

```bash
#!/bin/bash
# File: scripts/pre-release-quality.sh

set -e

echo "🚀 Pre-Release Quality Checks"
echo "=============================="

# Step 1: Run tests
echo "📋 Step 1: Running unit tests..."
cargo test --all-features

# Step 2: TDG baseline
echo "📊 Step 2: Creating TDG baseline..."
pmat tdg baseline create --output .pmat/release-baseline.json

# Step 3: Mutation testing
echo "🧪 Step 3: Running mutation testing..."
pmat mutate \
  --use-cargo-mutants \
  --all-features \
  --timeout 900 \
  --jobs 4 \
  --output mutation-results/

# Step 4: Quality gates
echo "✅ Step 4: Checking quality gates..."
pmat tdg check-quality --min-grade B

echo "🎉 All quality checks passed!"
```

**Usage**:
```bash
chmod +x scripts/pre-release-quality.sh
./scripts/pre-release-quality.sh
```

---

### Example 21: Weekly Quality Report

**Scenario**: Track mutation score over time

```bash
#!/bin/bash
# File: scripts/weekly-mutation-report.sh

DATE=$(date +%Y-%m-%d)
RESULTS_DIR="reports/mutation-$DATE"

echo "📈 Weekly Mutation Testing Report"
echo "=================================="
echo "Date: $DATE"

# Run mutation testing
pmat mutate \
  --use-cargo-mutants \
  --timeout 600 \
  --output "$RESULTS_DIR/"

# Parse results
SCORE=$(cat "$RESULTS_DIR/outcomes.json" | jq -r '.mutation_score // "N/A"')
echo "Mutation Score: $SCORE%"

# Append to history
echo "$DATE,$SCORE" >> reports/mutation-history.csv

# Generate trend chart (requires gnuplot or similar)
# gnuplot -e "set terminal png; set output 'reports/trend.png'; plot 'reports/mutation-history.csv' using 1:2 with lines"

echo "✅ Report saved to $RESULTS_DIR"
```

**Usage**:
```bash
# Run weekly (via cron)
0 9 * * 1 /path/to/weekly-mutation-report.sh
```

---

### Example 22: PR Comment Bot

**Scenario**: Post mutation results as PR comment

**File**: `.github/workflows/pr-comment.yml`

```yaml
name: PR Mutation Comment

on:
  pull_request:
    branches: [main]

jobs:
  mutation-comment:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Install tools
        run: cargo install pmat cargo-mutants

      - name: Run mutation testing
        id: mutants
        run: |
          pmat mutate --use-cargo-mutants --timeout 600 > results.txt
          cat results.txt

      - name: Post comment
        uses: actions/github-script@v6
        with:
          github-token: ${{ secrets.GITHUB_TOKEN }}
          script: |
            const fs = require('fs');
            const results = fs.readFileSync('results.txt', 'utf8');

            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: '## 🧪 Mutation Testing Results\n\n```\n' + results + '\n```'
            });
```

---

### Example 23: Incremental Module Testing

**Scenario**: Test modules one at a time for large projects

```bash
#!/bin/bash
# File: scripts/test-all-modules.sh

MODULES=(
  "src/core"
  "src/api"
  "src/utils"
  "src/models"
)

for module in "${MODULES[@]}"; do
  echo "🧪 Testing module: $module"

  pmat mutate \
    --target "$module" \
    --use-cargo-mutants \
    --timeout 300 \
    --output "results/$module/"

  echo "✅ $module complete"
  echo ""
done

echo "🎉 All modules tested!"
```

---

### Example 24: Benchmark Mode (No Shuffle)

**Scenario**: Consistent benchmarking across runs

```bash
#!/bin/bash
# File: scripts/benchmark-mutations.sh

# Run 3 times with same mutant order
for i in {1..3}; do
  echo "📊 Run $i/3"

  time pmat mutate \
    --use-cargo-mutants \
    --no-shuffle \
    --timeout 300 \
    --output "benchmark-run-$i/"
done

# Compare execution times
echo "Execution times:"
ls -lh benchmark-run-*/
```

---

### Example 25: Combined Quality Workflow

**Scenario**: Full quality analysis (TDG + Mutation + Coverage)

```bash
#!/bin/bash
# File: scripts/full-quality-check.sh

set -e

echo "🔍 Comprehensive Quality Analysis"
echo "================================="

# Step 1: TDG Analysis
echo "1️⃣ TDG Quality Scoring..."
pmat tdg baseline create \
  --output quality-results/tdg-baseline.json \
  --path .

# Step 2: Mutation Testing
echo "2️⃣ Mutation Testing..."
pmat mutate \
  --use-cargo-mutants \
  --timeout 600 \
  --jobs 4 \
  --output quality-results/mutation/

# Step 3: Code Coverage (using cargo-llvm-cov)
echo "3️⃣ Code Coverage..."
cargo llvm-cov --all-features --workspace --html

# Step 4: Generate report
echo "4️⃣ Generating Combined Report..."
cat << EOF > quality-results/REPORT.md
# Quality Analysis Report
**Date**: $(date)

## TDG Scores
$(pmat tdg baseline list | head -20)

## Mutation Testing
$(cat quality-results/mutation/outcomes.json | jq -r '.summary')

## Code Coverage
See: target/llvm-cov/html/index.html

EOF

echo "✅ Quality report: quality-results/REPORT.md"
```

---

## Summary

These examples demonstrate:

✅ **Basic Usage**: Simple commands for getting started
✅ **Performance**: Timeout and parallelism tuning
✅ **Features**: Testing different feature combinations
✅ **CI/CD**: GitHub Actions, GitLab CI, Jenkins integration
✅ **Workflows**: Real-world automation scripts
✅ **Quality**: Combined analysis with TDG and coverage

---

## Next Steps

1. Try examples 1-3 on your project
2. Tune timeouts and parallelism (examples 4-9)
3. Set up CI/CD integration (examples 16-19)
4. Create quality workflows (examples 20-25)

---

## Additional Resources

- **User Guide**: `docs/user-guides/cargo-mutants-integration.md`
- **cargo-mutants Docs**: https://mutants.rs/
- **PMAT Book**: https://paiml.github.io/pmat-book/

---

**Last Updated**: October 29, 2025
