# GitHub Actions Quality Gate Guide

This guide shows how to integrate pmat quality gates into your GitHub Actions workflows for automated code quality enforcement.

## Table of Contents
- [Quick Start](#quick-start)
- [Basic Quality Gate](#basic-quality-gate)
- [Advanced Multi-File Analysis](#advanced-multi-file-analysis)
- [PR-Specific Checks](#pr-specific-checks)
- [Matrix Testing](#matrix-testing)
- [Custom Thresholds](#custom-thresholds)
- [SARIF Integration](#sarif-integration)
- [Caching Strategies](#caching-strategies)
- [Best Practices](#best-practices)

## Quick Start

Add this to `.github/workflows/quality-gate.yml`:

```yaml
name: Quality Gate
on: [push, pull_request]

jobs:
  quality-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install pmat
        run: |
          curl -fsSL https://github.com/paiml/paiml-mcp-agent-toolkit/releases/latest/download/install.sh | bash
          echo "$HOME/.local/bin" >> $GITHUB_PATH
      
      - name: Run Quality Gate
        run: pmat quality-gate . --profile strict --fail-on-violations
```

## Basic Quality Gate

### Whole Project Analysis

```yaml
name: Code Quality Gate
on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  quality-gate:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Install pmat
        run: |
          curl -fsSL https://github.com/paiml/paiml-mcp-agent-toolkit/releases/latest/download/install.sh | bash
          echo "$HOME/.local/bin" >> $GITHUB_PATH
      
      - name: Check pmat version
        run: pmat --version
      
      - name: Run Quality Gate
        run: |
          pmat quality-gate . \
            --profile extreme \
            --format sarif \
            --output quality-report.sarif \
            --fail-on-violations
      
      - name: Upload SARIF results
        uses: github/codeql-action/upload-sarif@v3
        if: always()
        with:
          sarif_file: quality-report.sarif
          category: pmat-quality
```

### Single File Quality Gate

```yaml
name: Single File Quality Check
on:
  pull_request:
    paths:
      - '**/*.rs'
      - '**/*.ts'
      - '**/*.py'

jobs:
  check-modified-files:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      
      - name: Get changed files
        id: changed-files
        uses: tj-actions/changed-files@v41
        with:
          files: |
            **/*.rs
            **/*.ts
            **/*.py
      
      - name: Install pmat
        if: steps.changed-files.outputs.any_changed == 'true'
        run: |
          curl -fsSL https://github.com/paiml/paiml-mcp-agent-toolkit/releases/latest/download/install.sh | bash
          echo "$HOME/.local/bin" >> $GITHUB_PATH
      
      - name: Check each modified file
        if: steps.changed-files.outputs.any_changed == 'true'
        run: |
          for file in ${{ steps.changed-files.outputs.all_changed_files }}; do
            echo "Checking $file..."
            
            # Run complexity analysis
            pmat analyze complexity --file "$file" --format json > "${file}.complexity.json"
            
            # Run lint analysis
            pmat analyze lint-hotspot --file "$file" --max-density 5.0 --enforce || {
              echo "::error file=$file::File exceeds quality thresholds"
              exit 1
            }
          done
```

## Advanced Multi-File Analysis

### Parallel Quality Checks

```yaml
name: Parallel Quality Analysis
on: [push, pull_request]

jobs:
  prepare:
    runs-on: ubuntu-latest
    outputs:
      matrix: ${{ steps.set-matrix.outputs.matrix }}
    steps:
      - uses: actions/checkout@v4
      
      - name: Find all source files
        id: set-matrix
        run: |
          # Find all Rust files and create matrix
          FILES=$(find . -name "*.rs" -type f | jq -R -s -c 'split("\n")[:-1]')
          echo "matrix={\"file\":$FILES}" >> $GITHUB_OUTPUT

  quality-check:
    needs: prepare
    runs-on: ubuntu-latest
    strategy:
      matrix: ${{fromJson(needs.prepare.outputs.matrix)}}
      fail-fast: false
      max-parallel: 10
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Install pmat
        run: |
          curl -fsSL https://github.com/paiml/paiml-mcp-agent-toolkit/releases/latest/download/install.sh | bash
          echo "$HOME/.local/bin" >> $GITHUB_PATH
      
      - name: Analyze file
        run: |
          echo "Analyzing ${{ matrix.file }}..."
          
          # Complexity check
          pmat analyze complexity --file "${{ matrix.file }}" \
            --max-cyclomatic 10 \
            --max-cognitive 15 \
            --format json > complexity.json
          
          # Extract metrics
          CYCLOMATIC=$(jq -r '.files[0].total_complexity.cyclomatic' complexity.json)
          COGNITIVE=$(jq -r '.files[0].total_complexity.cognitive' complexity.json)
          
          echo "::notice file=${{ matrix.file }}::Complexity - Cyclomatic: $CYCLOMATIC, Cognitive: $COGNITIVE"
          
          # Lint check
          pmat analyze lint-hotspot --file "${{ matrix.file }}" \
            --max-density 5.0 \
            --format json > lint.json || true
          
          VIOLATIONS=$(jq -r '.hotspot.total_violations // 0' lint.json)
          DENSITY=$(jq -r '.hotspot.defect_density // 0' lint.json)
          
          echo "::notice file=${{ matrix.file }}::Lint - Violations: $VIOLATIONS, Density: $DENSITY"
          
          # Fail if thresholds exceeded
          if [ "$CYCLOMATIC" -gt 20 ] || [ "$COGNITIVE" -gt 30 ]; then
            echo "::error file=${{ matrix.file }}::Complexity exceeds thresholds"
            exit 1
          fi
          
          if (( $(echo "$DENSITY > 5.0" | bc -l) )); then
            echo "::error file=${{ matrix.file }}::Defect density exceeds threshold"
            exit 1
          fi
      
      - name: Upload results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: quality-results-${{ strategy.job-index }}
          path: |
            complexity.json
            lint.json
```

## PR-Specific Checks

### Incremental Analysis

```yaml
name: PR Quality Gate
on:
  pull_request:
    types: [opened, synchronize, reopened]

jobs:
  incremental-quality:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      
      - name: Fetch base branch
        run: |
          git fetch origin ${{ github.base_ref }}:${{ github.base_ref }}
      
      - name: Install pmat
        run: |
          curl -fsSL https://github.com/paiml/paiml-mcp-agent-toolkit/releases/latest/download/install.sh | bash
          echo "$HOME/.local/bin" >> $GITHUB_PATH
      
      - name: Get changed files
        id: changed-files
        run: |
          # Get list of modified files
          CHANGED_FILES=$(git diff --name-only ${{ github.base_ref }}...HEAD | grep -E '\.(rs|ts|py)$' || true)
          echo "files<<EOF" >> $GITHUB_OUTPUT
          echo "$CHANGED_FILES" >> $GITHUB_OUTPUT
          echo "EOF" >> $GITHUB_OUTPUT
          
          # Count changes
          FILE_COUNT=$(echo "$CHANGED_FILES" | wc -l)
          echo "count=$FILE_COUNT" >> $GITHUB_OUTPUT
      
      - name: Run targeted analysis
        if: steps.changed-files.outputs.count > 0
        run: |
          # Initialize summary
          echo "# Quality Gate Report" >> $GITHUB_STEP_SUMMARY
          echo "" >> $GITHUB_STEP_SUMMARY
          echo "| File | Complexity | Violations | Status |" >> $GITHUB_STEP_SUMMARY
          echo "|------|------------|------------|--------|" >> $GITHUB_STEP_SUMMARY
          
          FAILED=0
          
          while IFS= read -r file; do
            [ -z "$file" ] && continue
            
            # Skip if file doesn't exist (was deleted)
            [ -f "$file" ] || continue
            
            echo "Analyzing $file..."
            
            # Run complexity analysis
            COMPLEXITY_OUTPUT=$(pmat analyze complexity --file "$file" --format json 2>&1) || true
            
            if [ $? -eq 0 ]; then
              CYCLOMATIC=$(echo "$COMPLEXITY_OUTPUT" | jq -r '.files[0].total_complexity.cyclomatic // 0')
              COGNITIVE=$(echo "$COMPLEXITY_OUTPUT" | jq -r '.files[0].total_complexity.cognitive // 0')
              COMPLEXITY="${CYCLOMATIC}/${COGNITIVE}"
            else
              COMPLEXITY="N/A"
            fi
            
            # Run lint analysis
            LINT_OUTPUT=$(pmat analyze lint-hotspot --file "$file" --format json 2>&1) || true
            
            if [ $? -eq 0 ]; then
              VIOLATIONS=$(echo "$LINT_OUTPUT" | jq -r '.hotspot.total_violations // 0')
              DENSITY=$(echo "$LINT_OUTPUT" | jq -r '.hotspot.defect_density // 0')
              LINT_INFO="${VIOLATIONS} (${DENSITY}/100 LOC)"
            else
              LINT_INFO="N/A"
              VIOLATIONS=0
              DENSITY=0
            fi
            
            # Determine status
            STATUS="✅ Pass"
            if [ "$CYCLOMATIC" -gt 20 ] || [ "$COGNITIVE" -gt 30 ] || (( $(echo "$DENSITY > 5.0" | bc -l) )); then
              STATUS="❌ Fail"
              FAILED=$((FAILED + 1))
            fi
            
            # Add to summary
            echo "| $file | $COMPLEXITY | $LINT_INFO | $STATUS |" >> $GITHUB_STEP_SUMMARY
            
          done <<< "${{ steps.changed-files.outputs.files }}"
          
          # Add summary statistics
          echo "" >> $GITHUB_STEP_SUMMARY
          echo "**Total files analyzed**: ${{ steps.changed-files.outputs.count }}" >> $GITHUB_STEP_SUMMARY
          echo "**Failed files**: $FAILED" >> $GITHUB_STEP_SUMMARY
          
          # Fail if any file failed
          if [ $FAILED -gt 0 ]; then
            echo "::error::$FAILED files failed quality gate"
            exit 1
          fi
      
      - name: Comment PR
        if: always() && github.event_name == 'pull_request'
        uses: actions/github-script@v7
        with:
          script: |
            const fs = require('fs');
            const summary = fs.readFileSync(process.env.GITHUB_STEP_SUMMARY, 'utf8');
            
            // Find existing comment
            const { data: comments } = await github.rest.issues.listComments({
              owner: context.repo.owner,
              repo: context.repo.repo,
              issue_number: context.issue.number,
            });
            
            const botComment = comments.find(comment => 
              comment.user.type === 'Bot' && 
              comment.body.includes('Quality Gate Report')
            );
            
            const body = `## 🔍 Quality Gate Report\n\n${summary}`;
            
            if (botComment) {
              await github.rest.issues.updateComment({
                owner: context.repo.owner,
                repo: context.repo.repo,
                comment_id: botComment.id,
                body: body
              });
            } else {
              await github.rest.issues.createComment({
                owner: context.repo.owner,
                repo: context.repo.repo,
                issue_number: context.issue.number,
                body: body
              });
            }
```

## Matrix Testing

### Multi-Language Support

```yaml
name: Multi-Language Quality Gate
on: [push, pull_request]

jobs:
  quality-matrix:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        language: [rust, typescript, python]
        profile: [standard, strict, extreme]
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Install pmat
        run: |
          curl -fsSL https://github.com/paiml/paiml-mcp-agent-toolkit/releases/latest/download/install.sh | bash
          echo "$HOME/.local/bin" >> $GITHUB_PATH
      
      - name: Set file extension
        id: ext
        run: |
          case "${{ matrix.language }}" in
            rust) echo "ext=rs" >> $GITHUB_OUTPUT ;;
            typescript) echo "ext=ts" >> $GITHUB_OUTPUT ;;
            python) echo "ext=py" >> $GITHUB_OUTPUT ;;
          esac
      
      - name: Run quality gate
        run: |
          echo "Running ${{ matrix.profile }} profile for ${{ matrix.language }} files..."
          
          # Find files of this language
          FILES=$(find . -name "*.${{ steps.ext.outputs.ext }}" -type f)
          
          if [ -n "$FILES" ]; then
            pmat quality-gate . \
              --profile ${{ matrix.profile }} \
              --include "**/*.${{ steps.ext.outputs.ext }}" \
              --format json \
              --output "quality-${{ matrix.language }}-${{ matrix.profile }}.json"
          else
            echo "No ${{ matrix.language }} files found"
          fi
      
      - name: Upload results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: quality-results-${{ matrix.language }}-${{ matrix.profile }}
          path: quality-*.json
```

## Custom Thresholds

### Project-Specific Configuration

```yaml
name: Custom Quality Thresholds
on: [push, pull_request]

jobs:
  custom-quality:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Install pmat
        run: |
          curl -fsSL https://github.com/paiml/paiml-mcp-agent-toolkit/releases/latest/download/install.sh | bash
          echo "$HOME/.local/bin" >> $GITHUB_PATH
      
      - name: Create quality config
        run: |
          cat > .pmat-quality.json << 'EOF'
          {
            "complexity": {
              "cyclomatic_warn": 10,
              "cyclomatic_error": 20,
              "cognitive_warn": 15,
              "cognitive_error": 30,
              "nesting_max": 5,
              "method_length": 50
            },
            "lint": {
              "max_density": 3.0,
              "max_violations_per_file": 25,
              "allowed_lints": [
                "clippy::module_name_repetitions",
                "clippy::must_use_candidate"
              ]
            },
            "coverage": {
              "minimum": 80,
              "warn_at": 85,
              "target": 90
            }
          }
          EOF
      
      - name: Run with custom config
        run: |
          # Complexity check with custom thresholds
          pmat analyze complexity . \
            --max-cyclomatic 20 \
            --max-cognitive 30 \
            --format json > complexity.json
          
          # Lint check with custom density
          pmat analyze lint-hotspot . \
            --max-density 3.0 \
            --top-files 0 \
            --format json > lint.json
          
          # Extract and validate metrics
          TOTAL_FILES=$(jq -r '.summary.total_files // 0' complexity.json)
          VIOLATIONS=$(jq -r '.summary.total_violations // 0' complexity.json)
          
          echo "Total files analyzed: $TOTAL_FILES"
          echo "Total violations: $VIOLATIONS"
          
          # Generate badge data
          if [ "$VIOLATIONS" -eq 0 ]; then
            BADGE_COLOR="brightgreen"
            BADGE_TEXT="passing"
          elif [ "$VIOLATIONS" -lt 10 ]; then
            BADGE_COLOR="yellow"
            BADGE_TEXT="warnings"
          else
            BADGE_COLOR="red"
            BADGE_TEXT="failing"
          fi
          
          # Create badge JSON
          cat > badge.json << EOF
          {
            "schemaVersion": 1,
            "label": "quality",
            "message": "$BADGE_TEXT",
            "color": "$BADGE_COLOR"
          }
          EOF
      
      - name: Upload badge data
        uses: actions/upload-artifact@v4
        with:
          name: quality-badge
          path: badge.json
```

## SARIF Integration

### Advanced SARIF Reporting

```yaml
name: SARIF Quality Reporting
on:
  push:
    branches: [main]
  pull_request:

jobs:
  sarif-analysis:
    runs-on: ubuntu-latest
    permissions:
      security-events: write
      actions: read
      contents: read
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Install pmat
        run: |
          curl -fsSL https://github.com/paiml/paiml-mcp-agent-toolkit/releases/latest/download/install.sh | bash
          echo "$HOME/.local/bin" >> $GITHUB_PATH
      
      - name: Run comprehensive analysis
        run: |
          # Complexity analysis
          pmat analyze complexity . \
            --format sarif \
            --output complexity.sarif
          
          # Dead code analysis
          pmat analyze dead-code . \
            --format sarif \
            --output deadcode.sarif
          
          # Technical debt
          pmat analyze satd . \
            --format sarif \
            --output satd.sarif
          
          # Merge SARIF files
          jq -s '.[0] * {runs: [.[] | .runs[]] | group_by(.tool.driver.name) | map(.[0])}' \
            complexity.sarif deadcode.sarif satd.sarif > combined.sarif
      
      - name: Upload to GitHub Security
        uses: github/codeql-action/upload-sarif@v3
        with:
          sarif_file: combined.sarif
          category: pmat-comprehensive
      
      - name: Upload individual reports
        uses: actions/upload-artifact@v4
        with:
          name: sarif-reports
          path: |
            *.sarif
```

## Caching Strategies

### Efficient Caching

```yaml
name: Cached Quality Analysis
on: [push, pull_request]

jobs:
  cached-quality:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Cache pmat binary
        id: cache-pmat
        uses: actions/cache@v4
        with:
          path: ~/.local/bin/pmat
          key: ${{ runner.os }}-pmat-${{ hashFiles('**/quality-gate.yml') }}
      
      - name: Install pmat
        if: steps.cache-pmat.outputs.cache-hit != 'true'
        run: |
          curl -fsSL https://github.com/paiml/paiml-mcp-agent-toolkit/releases/latest/download/install.sh | bash
          echo "$HOME/.local/bin" >> $GITHUB_PATH
      
      - name: Cache analysis results
        uses: actions/cache@v4
        with:
          path: |
            .pmat-cache/
            **/*.pmat.cache
          key: ${{ runner.os }}-pmat-analysis-${{ github.sha }}
          restore-keys: |
            ${{ runner.os }}-pmat-analysis-
      
      - name: Run incremental analysis
        run: |
          # Create cache directory
          mkdir -p .pmat-cache
          
          # Run analysis with caching
          pmat quality-gate . \
            --profile strict \
            --cache-dir .pmat-cache \
            --incremental \
            --format json \
            --output quality-report.json
      
      - name: Compare with baseline
        if: github.event_name == 'pull_request'
        run: |
          # Fetch baseline from main branch
          git fetch origin main:main
          
          # Run baseline analysis
          git checkout main
          pmat quality-gate . \
            --profile strict \
            --format json \
            --output baseline.json || true
          
          # Switch back to PR branch
          git checkout -
          
          # Compare results
          if [ -f baseline.json ]; then
            # Extract metrics
            BASELINE_VIOLATIONS=$(jq -r '.summary.total_violations // 0' baseline.json)
            CURRENT_VIOLATIONS=$(jq -r '.summary.total_violations // 0' quality-report.json)
            
            DIFF=$((CURRENT_VIOLATIONS - BASELINE_VIOLATIONS))
            
            if [ $DIFF -gt 0 ]; then
              echo "::warning::Quality degraded by $DIFF violations compared to main branch"
            elif [ $DIFF -lt 0 ]; then
              echo "::notice::Quality improved by $((-DIFF)) violations compared to main branch"
            else
              echo "::notice::Quality unchanged from main branch"
            fi
          fi
```

## Best Practices

### Complete Workflow Example

```yaml
name: Comprehensive Quality Pipeline
on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]
  schedule:
    - cron: '0 0 * * 0'  # Weekly on Sunday

env:
  PMAT_VERSION: latest
  QUALITY_PROFILE: strict

jobs:
  # First job: Quick checks on changed files
  quick-check:
    if: github.event_name == 'pull_request'
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      
      - name: Install pmat
        run: |
          curl -fsSL https://github.com/paiml/paiml-mcp-agent-toolkit/releases/latest/download/install.sh | bash
          echo "$HOME/.local/bin" >> $GITHUB_PATH
      
      - name: Quick quality check
        run: |
          # Get changed files
          CHANGED=$(git diff --name-only origin/${{ github.base_ref }}...HEAD | grep -E '\.(rs|ts|py)$' || true)
          
          if [ -n "$CHANGED" ]; then
            echo "$CHANGED" | while read file; do
              [ -f "$file" ] || continue
              echo "Quick check: $file"
              
              # Basic complexity check
              pmat analyze complexity --file "$file" --format json | \
                jq -r '"Complexity: \(.files[0].total_complexity.cyclomatic // 0)"'
            done
          fi

  # Second job: Full analysis
  full-analysis:
    runs-on: ubuntu-latest
    needs: [quick-check]
    if: always() && (needs.quick-check.result == 'success' || needs.quick-check.result == 'skipped')
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Install pmat
        run: |
          if [ "${{ env.PMAT_VERSION }}" = "latest" ]; then
            curl -fsSL https://github.com/paiml/paiml-mcp-agent-toolkit/releases/latest/download/install.sh | bash
          else
            curl -fsSL "https://github.com/paiml/paiml-mcp-agent-toolkit/releases/download/v${{ env.PMAT_VERSION }}/install.sh" | bash
          fi
          echo "$HOME/.local/bin" >> $GITHUB_PATH
      
      - name: Full quality gate
        run: |
          pmat quality-gate . \
            --profile ${{ env.QUALITY_PROFILE }} \
            --format sarif \
            --output quality.sarif \
            --fail-on-violations
      
      - name: Generate reports
        if: always()
        run: |
          # Generate multiple format reports
          pmat quality-gate . --format json > quality.json
          pmat quality-gate . --format markdown > quality.md
          
          # Extract key metrics
          TOTAL_FILES=$(jq -r '.summary.total_files // 0' quality.json)
          TOTAL_VIOLATIONS=$(jq -r '.summary.total_violations // 0' quality.json)
          COMPLEXITY_ISSUES=$(jq -r '.summary.complexity_violations // 0' quality.json)
          LINT_ISSUES=$(jq -r '.summary.lint_violations // 0' quality.json)
          
          # Create summary
          cat > summary.md << EOF
          # Quality Gate Summary
          
          - **Total Files**: $TOTAL_FILES
          - **Total Violations**: $TOTAL_VIOLATIONS
          - **Complexity Issues**: $COMPLEXITY_ISSUES
          - **Lint Issues**: $LINT_ISSUES
          
          ## Details
          
          $(cat quality.md)
          EOF
          
          # Set outputs for badge
          echo "violations=$TOTAL_VIOLATIONS" >> $GITHUB_OUTPUT
      
      - name: Upload SARIF
        if: always()
        uses: github/codeql-action/upload-sarif@v3
        with:
          sarif_file: quality.sarif
      
      - name: Create badge
        if: always()
        uses: schneegans/dynamic-badges-action@v1.7.0
        with:
          auth: ${{ secrets.GIST_SECRET }}
          gistID: your-gist-id
          filename: quality-badge.json
          label: Quality
          message: ${{ steps.metrics.outputs.violations }} violations
          color: ${{ steps.metrics.outputs.violations == '0' && 'green' || 'red' }}

  # Third job: Trend analysis (scheduled)
  trend-analysis:
    if: github.event_name == 'schedule'
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      
      - name: Install dependencies
        run: |
          curl -fsSL https://github.com/paiml/paiml-mcp-agent-toolkit/releases/latest/download/install.sh | bash
          echo "$HOME/.local/bin" >> $GITHUB_PATH
          pip install matplotlib pandas
      
      - name: Analyze trends
        run: |
          # Analyze last 30 days of commits
          mkdir -p trends
          
          for i in {0..30}; do
            DATE=$(date -d "$i days ago" +%Y-%m-%d)
            COMMIT=$(git rev-list -n 1 --before="$DATE" HEAD 2>/dev/null || true)
            
            if [ -n "$COMMIT" ]; then
              git checkout $COMMIT
              
              pmat quality-gate . \
                --profile strict \
                --format json \
                --output "trends/$DATE.json" || true
            fi
          done
          
          # Generate trend report
          python << 'EOF'
          import json
          import glob
          import matplotlib.pyplot as plt
          import pandas as pd
          from datetime import datetime
          
          # Load all trend files
          data = []
          for file in sorted(glob.glob('trends/*.json')):
              try:
                  with open(file) as f:
                      content = json.load(f)
                      date = file.split('/')[-1].replace('.json', '')
                      data.append({
                          'date': datetime.strptime(date, '%Y-%m-%d'),
                          'violations': content.get('summary', {}).get('total_violations', 0),
                          'complexity': content.get('summary', {}).get('avg_complexity', 0)
                      })
              except:
                  pass
          
          if data:
              df = pd.DataFrame(data)
              
              # Create plots
              fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(10, 8))
              
              # Violations trend
              ax1.plot(df['date'], df['violations'], 'b-')
              ax1.set_title('Code Violations Trend')
              ax1.set_ylabel('Total Violations')
              ax1.grid(True)
              
              # Complexity trend
              ax2.plot(df['date'], df['complexity'], 'r-')
              ax2.set_title('Average Complexity Trend')
              ax2.set_ylabel('Complexity')
              ax2.set_xlabel('Date')
              ax2.grid(True)
              
              plt.tight_layout()
              plt.savefig('quality-trends.png')
          EOF
      
      - name: Upload trend report
        uses: actions/upload-artifact@v4
        with:
          name: quality-trends
          path: |
            trends/
            quality-trends.png

  # Final job: Notify on failure
  notify:
    if: failure()
    needs: [quick-check, full-analysis]
    runs-on: ubuntu-latest
    
    steps:
      - name: Send notification
        run: |
          echo "Quality gate failed! Check the workflow run for details."
          # Add your notification logic here (Slack, email, etc.)
```

## Configuration Files

### .github/pmat-config.yml

```yaml
# pmat configuration for GitHub Actions
quality-profiles:
  standard:
    complexity:
      cyclomatic_max: 20
      cognitive_max: 30
    lint:
      max_density: 5.0
      max_violations: 50
  
  strict:
    complexity:
      cyclomatic_max: 15
      cognitive_max: 20
    lint:
      max_density: 3.0
      max_violations: 25
  
  extreme:
    complexity:
      cyclomatic_max: 10
      cognitive_max: 15
    lint:
      max_density: 1.0
      max_violations: 10

# File patterns to analyze
include:
  - "**/*.rs"
  - "**/*.ts"
  - "**/*.tsx"
  - "**/*.py"

# Patterns to exclude
exclude:
  - "**/target/**"
  - "**/node_modules/**"
  - "**/build/**"
  - "**/*.test.*"
  - "**/*.spec.*"

# Enforcement rules
enforcement:
  fail_on_violations: true
  block_pr_on_failure: true
  require_improvement: false  # Require metrics to improve vs baseline
```

### Badge Generation

Add this to your README.md:

```markdown
![Quality Gate](https://img.shields.io/endpoint?url=https://gist.githubusercontent.com/YOUR_USERNAME/GIST_ID/raw/quality-badge.json)
![Complexity](https://img.shields.io/badge/complexity-low-green)
![Technical Debt](https://img.shields.io/badge/tech%20debt-0-brightgreen)
```

## Troubleshooting

### Common Issues

1. **pmat not found**
   ```yaml
   - name: Debug PATH
     run: |
       echo "PATH: $PATH"
       which pmat || echo "pmat not in PATH"
       ls -la ~/.local/bin/
   ```

2. **Out of memory on large projects**
   ```yaml
   - name: Run with memory limits
     run: |
       # Increase Node.js memory for TypeScript analysis
       export NODE_OPTIONS="--max-old-space-size=4096"
       
       # Run with limited parallelism
       pmat quality-gate . --parallel 2
   ```

3. **Timeout on analysis**
   ```yaml
   - name: Run with timeout
     timeout-minutes: 30
     run: |
       # Run with progress reporting
       pmat quality-gate . --verbose --progress
   ```