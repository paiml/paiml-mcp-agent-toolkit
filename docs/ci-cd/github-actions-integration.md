# GitHub Actions CI/CD Integration for PMAT Mutation Testing

This guide demonstrates how to integrate PMAT mutation testing into GitHub Actions workflows for continuous quality assurance.

## Overview

GitHub Actions provides automated CI/CD pipelines that can run mutation testing on every push, pull request, or scheduled basis. This ensures code quality is continuously monitored and mutation scores are tracked over time.

## Table of Contents

1. [Basic Setup](#basic-setup)
2. [Multi-Language Support](#multi-language-support)
3. [Quality Gates](#quality-gates)
4. [Advanced Patterns](#advanced-patterns)
5. [Caching Strategies](#caching-strategies)
6. [Artifacts and Reporting](#artifacts-and-reporting)
7. [Troubleshooting](#troubleshooting)

## Basic Setup

### Minimal Workflow

Create `.github/workflows/mutation-testing.yml`:

```yaml
name: Mutation Testing

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  mutation-test:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          profile: minimal

      - name: Install PMAT
        run: cargo install pmat

      - name: Run mutation tests
        run: pmat mutate --target src/ --failures-only
```

### With Threshold Enforcement

Add quality gates with mutation score thresholds:

```yaml
      - name: Run mutation tests with threshold
        run: |
          pmat mutate \
            --target src/ \
            --threshold 80 \
            --failures-only \
            --output-format json > mutation-results.json

      - name: Check mutation score
        run: |
          SCORE=$(jq '.mutation_score' mutation-results.json)
          echo "Mutation Score: ${SCORE}%"
          if (( $(echo "$SCORE < 80" | bc -l) )); then
            echo "❌ Mutation score ${SCORE}% is below threshold 80%"
            exit 1
          fi
          echo "✅ Mutation score ${SCORE}% meets threshold"
```

## Multi-Language Support

### Rust Project

```yaml
name: Rust Mutation Testing

on: [push, pull_request]

jobs:
  test-and-mutate:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: rustfmt, clippy

      - name: Cache cargo registry
        uses: actions/cache@v3
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo index
        uses: actions/cache@v3
        with:
          path: ~/.cargo/git
          key: ${{ runner.os }}-cargo-index-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo build
        uses: actions/cache@v3
        with:
          path: target
          key: ${{ runner.os }}-cargo-build-target-${{ hashFiles('**/Cargo.lock') }}

      - name: Run tests
        run: cargo test --all-features

      - name: Install PMAT
        run: cargo install pmat

      - name: Run mutation tests
        run: |
          pmat mutate \
            --target src/ \
            --threshold 85 \
            --jobs 4 \
            --timeout 10 \
            --output-format markdown > mutation-report.md

      - name: Upload mutation report
        uses: actions/upload-artifact@v3
        if: always()
        with:
          name: mutation-report
          path: mutation-report.md
```

### Python Project

```yaml
name: Python Mutation Testing

on: [push, pull_request]

jobs:
  test-and-mutate:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        python-version: ['3.9', '3.10', '3.11']

    steps:
      - uses: actions/checkout@v4

      - name: Set up Python ${{ matrix.python-version }}
        uses: actions/setup-python@v4
        with:
          python-version: ${{ matrix.python-version }}

      - name: Cache pip packages
        uses: actions/cache@v3
        with:
          path: ~/.cache/pip
          key: ${{ runner.os }}-pip-${{ hashFiles('**/requirements.txt') }}
          restore-keys: |
            ${{ runner.os }}-pip-

      - name: Install dependencies
        run: |
          python -m pip install --upgrade pip
          pip install -r requirements.txt
          pip install pytest pytest-cov

      - name: Run tests with coverage
        run: pytest tests/ --cov=src --cov-report=xml

      - name: Install Rust and PMAT
        run: |
          curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
          source $HOME/.cargo/env
          cargo install pmat

      - name: Run mutation tests
        run: |
          source $HOME/.cargo/env
          pmat mutate \
            --target src/ \
            --threshold 80 \
            --failures-only \
            --output-format json > mutation-results.json

      - name: Upload results
        uses: actions/upload-artifact@v3
        if: always()
        with:
          name: mutation-results-py${{ matrix.python-version }}
          path: mutation-results.json
```

### TypeScript/Node.js Project

```yaml
name: TypeScript Mutation Testing

on: [push, pull_request]

jobs:
  test-and-mutate:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        node-version: [16, 18, 20]

    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js ${{ matrix.node-version }}
        uses: actions/setup-node@v3
        with:
          node-version: ${{ matrix.node-version }}
          cache: 'npm'

      - name: Install dependencies
        run: npm ci

      - name: Run linter
        run: npm run lint

      - name: Run tests
        run: npm test -- --coverage

      - name: Install Rust and PMAT
        run: |
          curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
          source $HOME/.cargo/env
          cargo install pmat

      - name: Run mutation tests
        run: |
          source $HOME/.cargo/env
          pmat mutate \
            --target src/ \
            --threshold 85 \
            --jobs 4 \
            --output-format markdown > mutation-report.md

      - name: Comment PR with results
        if: github.event_name == 'pull_request'
        uses: actions/github-script@v6
        with:
          script: |
            const fs = require('fs');
            const report = fs.readFileSync('mutation-report.md', 'utf8');
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.name,
              body: report
            });
```

## Quality Gates

### Fail on Low Mutation Score

```yaml
      - name: Enforce mutation score threshold
        run: |
          SCORE=$(pmat mutate --target src/ --output-format json | jq '.mutation_score')
          echo "::notice ::Mutation Score: ${SCORE}%"

          if (( $(echo "$SCORE < 80" | bc -l) )); then
            echo "::error ::Mutation score ${SCORE}% is below required 80%"
            exit 1
          fi
```

### Progressive Thresholds

Different thresholds for different branches:

```yaml
      - name: Run mutation tests with branch-specific thresholds
        run: |
          if [[ "${{ github.ref }}" == "refs/heads/main" ]]; then
            THRESHOLD=90
          elif [[ "${{ github.ref }}" == "refs/heads/develop" ]]; then
            THRESHOLD=85
          else
            THRESHOLD=80
          fi

          echo "Using mutation score threshold: ${THRESHOLD}%"
          pmat mutate --target src/ --threshold $THRESHOLD
```

### Comment on PR with Results

```yaml
      - name: Run mutation tests
        id: mutation
        run: |
          pmat mutate \
            --target src/ \
            --output-format markdown > mutation-report.md

          SCORE=$(pmat mutate --target src/ --output-format json | jq '.mutation_score')
          echo "score=$SCORE" >> $GITHUB_OUTPUT

      - name: Comment PR
        if: github.event_name == 'pull_request'
        uses: actions/github-script@v6
        with:
          github-token: ${{ secrets.GITHUB_TOKEN }}
          script: |
            const fs = require('fs');
            const report = fs.readFileSync('mutation-report.md', 'utf8');
            const score = '${{ steps.mutation.outputs.score }}';

            const emoji = score >= 90 ? '🟢' : score >= 80 ? '🟡' : '🔴';
            const body = `## ${emoji} Mutation Testing Report\n\n**Score: ${score}%**\n\n${report}`;

            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.name,
              body: body
            });
```

## Advanced Patterns

### Matrix Testing (Multiple Languages)

```yaml
name: Multi-Language Mutation Testing

on: [push, pull_request]

jobs:
  mutation-test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        language: [rust, python, typescript]
        include:
          - language: rust
            target: src/
            threshold: 90
          - language: python
            target: python/src/
            threshold: 85
          - language: typescript
            target: ts/src/
            threshold: 85

    steps:
      - uses: actions/checkout@v4

      - name: Setup environment for ${{ matrix.language }}
        run: |
          case "${{ matrix.language }}" in
            rust)
              curl https://sh.rustup.rs -sSf | sh -s -- -y
              ;;
            python)
              sudo apt-get install -y python3 python3-pip
              ;;
            typescript)
              curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
              sudo apt-get install -y nodejs
              ;;
          esac

      - name: Install PMAT
        run: cargo install pmat

      - name: Run mutation tests for ${{ matrix.language }}
        run: |
          pmat mutate \
            --target ${{ matrix.target }} \
            --threshold ${{ matrix.threshold }} \
            --output-format json > mutation-${{ matrix.language }}.json

      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: mutation-results-${{ matrix.language }}
          path: mutation-${{ matrix.language }}.json
```

### Scheduled Mutation Testing

Run comprehensive mutation testing on a schedule:

```yaml
name: Scheduled Mutation Testing

on:
  schedule:
    # Run every night at 2 AM UTC
    - cron: '0 2 * * *'
  workflow_dispatch:  # Allow manual triggers

jobs:
  comprehensive-mutation-test:
    runs-on: ubuntu-latest
    timeout-minutes: 120  # 2 hours max

    steps:
      - uses: actions/checkout@v4

      - name: Install PMAT
        run: cargo install pmat

      - name: Run comprehensive mutation tests
        run: |
          pmat mutate \
            --target src/ \
            --jobs 8 \
            --timeout 30 \
            --output-format json > full-mutation-report.json

      - name: Generate historical trend
        run: |
          mkdir -p mutation-history
          cp full-mutation-report.json mutation-history/$(date +%Y-%m-%d).json

      - name: Commit history
        run: |
          git config user.name "Mutation Bot"
          git config user.email "bot@example.com"
          git add mutation-history/
          git commit -m "chore: Add mutation testing report for $(date +%Y-%m-%d)" || true
          git push
```

### Differential Mutation Testing

Test only changed files:

```yaml
      - name: Get changed files
        id: changed-files
        uses: tj-actions/changed-files@v39
        with:
          files: |
            src/**/*.rs
            src/**/*.py
            src/**/*.ts

      - name: Run mutation tests on changed files only
        if: steps.changed-files.outputs.any_changed == 'true'
        run: |
          for file in ${{ steps.changed-files.outputs.all_changed_files }}; do
            echo "Testing mutations in: $file"
            pmat mutate --target "$file" --failures-only
          done
```

## Caching Strategies

### Cache PMAT Binary

```yaml
      - name: Cache PMAT binary
        id: cache-pmat
        uses: actions/cache@v3
        with:
          path: ~/.cargo/bin/pmat
          key: ${{ runner.os }}-pmat-${{ hashFiles('**/Cargo.lock') }}

      - name: Install PMAT
        if: steps.cache-pmat.outputs.cache-hit != 'true'
        run: cargo install pmat
```

### Cache Build Artifacts

```yaml
      - name: Cache build artifacts
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-
```

## Artifacts and Reporting

### Upload Multiple Reports

```yaml
      - name: Generate multiple report formats
        run: |
          pmat mutate --target src/ --output-format text > mutation-report.txt
          pmat mutate --target src/ --output-format json > mutation-report.json
          pmat mutate --target src/ --output-format markdown > mutation-report.md

      - name: Upload all reports
        uses: actions/upload-artifact@v3
        if: always()
        with:
          name: mutation-reports
          path: |
            mutation-report.txt
            mutation-report.json
            mutation-report.md
```

### Create GitHub Release with Report

```yaml
      - name: Create release with mutation report
        if: startsWith(github.ref, 'refs/tags/')
        uses: softprops/action-gh-release@v1
        with:
          files: |
            mutation-report.md
            mutation-report.json
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

## Troubleshooting

### Timeout Issues

If mutation testing takes too long:

```yaml
      - name: Run mutation tests with timeout
        timeout-minutes: 30
        run: |
          pmat mutate \
            --target src/ \
            --timeout 10 \
            --jobs 4
```

### Memory Issues

For large projects:

```yaml
      - name: Run mutation tests in batches
        run: |
          find src -name "*.rs" | xargs -n 10 -P 2 pmat mutate --target
```

### Debug Output

Enable verbose logging:

```yaml
      - name: Run mutation tests with debug output
        run: |
          RUST_LOG=debug pmat mutate --target src/ --verbose
```

## Best Practices

1. **Start with `--failures-only`** to reduce noise
2. **Use caching** to speed up builds
3. **Set reasonable timeouts** (5-10 seconds per mutant)
4. **Run comprehensive tests nightly**, lightweight on PRs
5. **Use matrix strategy** for multi-language projects
6. **Cache PMAT binary** to avoid rebuilding
7. **Set mutation score thresholds** as quality gates
8. **Upload artifacts** for historical tracking
9. **Comment on PRs** with mutation scores
10. **Use differential testing** on PRs to save time

## Example: Complete Production Workflow

```yaml
name: Production Mutation Testing

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]
  schedule:
    - cron: '0 2 * * 0'  # Weekly on Sunday

jobs:
  mutation-test:
    runs-on: ubuntu-latest
    timeout-minutes: 60

    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0  # Full history for trend analysis

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          profile: minimal

      - name: Cache dependencies
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
            ~/.cargo/bin/pmat
          key: ${{ runner.os }}-mutation-${{ hashFiles('**/Cargo.lock') }}

      - name: Install PMAT
        run: cargo install pmat || true

      - name: Run tests first
        run: cargo test --all-features

      - name: Run mutation tests
        id: mutation
        run: |
          if [[ "${{ github.event_name }}" == "pull_request" ]]; then
            MODE="--failures-only"
            THRESHOLD=80
          else
            MODE=""
            THRESHOLD=85
          fi

          pmat mutate \
            --target src/ \
            --threshold $THRESHOLD \
            $MODE \
            --jobs 4 \
            --timeout 10 \
            --output-format json > mutation-results.json

          SCORE=$(jq '.mutation_score' mutation-results.json)
          echo "score=$SCORE" >> $GITHUB_OUTPUT

      - name: Generate markdown report
        run: |
          pmat mutate \
            --target src/ \
            --output-format markdown > mutation-report.md

      - name: Upload artifacts
        uses: actions/upload-artifact@v3
        if: always()
        with:
          name: mutation-reports-${{ github.sha }}
          path: |
            mutation-results.json
            mutation-report.md

      - name: Comment on PR
        if: github.event_name == 'pull_request'
        uses: actions/github-script@v6
        with:
          script: |
            const fs = require('fs');
            const report = fs.readFileSync('mutation-report.md', 'utf8');
            const score = '${{ steps.mutation.outputs.score }}';
            const threshold = 80;

            const emoji = score >= 90 ? '🟢' : score >= threshold ? '🟡' : '🔴';
            const status = score >= threshold ? 'PASSED' : 'FAILED';

            const body = `## ${emoji} Mutation Testing Report - ${status}

**Mutation Score: ${score}%** (Threshold: ${threshold}%)

${report}

---
*Mutation testing powered by PMAT*
            `;

            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.name,
              body: body
            });
```

## Resources

- [PMAT GitHub Repository](https://github.com/paiml/paiml-mcp-agent-toolkit)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust Example Project](../../examples/rust-mutation-testing/)
- [Python Example Project](../../examples/python-mutation-testing/)
- [TypeScript Example Project](../../examples/typescript-mutation-testing/)
