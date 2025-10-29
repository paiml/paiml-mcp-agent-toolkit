# CI/CD Integration Guide for PMAT TDG Quality Enforcement

**Version**: Sprint 66 Phase 4
**Date**: October 29, 2025
**Status**: Complete

## Table of Contents

1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [GitHub Actions Integration](#github-actions-integration)
4. [GitLab CI Integration](#gitlab-ci-integration)
5. [Jenkins Integration](#jenkins-integration)
6. [Configuration](#configuration)
7. [Enforcement Modes](#enforcement-modes)
8. [Workflows and Strategies](#workflows-and-strategies)
9. [Troubleshooting](#troubleshooting)
10. [Best Practices](#best-practices)

---

## Overview

PMAT's TDG (Technical Debt Grade) Enforcement System provides automated quality gates for CI/CD pipelines. This guide covers integrating TDG quality checks into your continuous integration workflows.

### What TDG Enforcement Provides

- **Regression Detection**: Automatically detect quality degradation in pull requests
- **Quality Gates**: Enforce minimum quality standards for new/modified code
- **Baseline Tracking**: Maintain project-wide quality baselines with content-hash deduplication
- **Automated Reporting**: Generate comprehensive quality reports for every build
- **Flexible Enforcement**: Choose between strict blocking, warning-only, or disabled modes

### Supported CI/CD Platforms

✅ **GitHub Actions** - Full support with PR comments and artifact uploads
✅ **GitLab CI** - Full support with merge request pipelines and artifact reports
✅ **Jenkins** - Full support with pipeline stages and build artifacts
🔄 **CircleCI, Travis CI, Azure Pipelines** - Use GitHub Actions template as reference

---

## Quick Start

### 1. Install TDG Enforcement Hooks

```bash
# Navigate to your project
cd /path/to/your/project

# Install PMAT
cargo install pmat

# Install TDG enforcement (includes CI templates)
pmat hooks install --tdg-enforcement

# This creates:
# - .pmat/tdg-rules.toml (configuration)
# - .git/hooks/pre-commit (local git hook)
# - .git/hooks/post-commit (baseline auto-update)
```

### 2. Configure Quality Thresholds

Edit `.pmat/tdg-rules.toml`:

```toml
[quality_gates]
rust_min_grade = "B+"
python_min_grade = "B"
max_score_drop = 5.0
allow_grade_drop = false
mode = "strict"  # strict, warning, or disabled
block_on_regression = true
block_on_new_files_below_threshold = true

[baseline]
baseline_path = ".pmat/tdg-baseline.json"
auto_update_on_main = true
store_in_git = true

[ci_cd]
fail_on_regression = true
fail_on_new_file_violation = true
generate_reports = true
comment_on_pr = true  # GitHub Actions only
```

### 3. Copy CI Template to Your Repository

**GitHub Actions**:
```bash
mkdir -p .github/workflows
cp templates/ci/github-actions-tdg.yml .github/workflows/tdg-quality.yml
```

**GitLab CI**:
```bash
cp templates/ci/gitlab-ci-tdg.yml .gitlab-ci.yml
# Or append to existing .gitlab-ci.yml
```

**Jenkins**:
```bash
cp templates/ci/Jenkinsfile-tdg Jenkinsfile
# Or create as separate pipeline
```

### 4. Create Initial Baseline

```bash
# Create project-wide quality baseline
pmat tdg baseline create --output .pmat/tdg-baseline.json --path .

# Review baseline
pmat tdg baseline list

# Commit baseline
git add .pmat/tdg-baseline.json .pmat/tdg-rules.toml
git commit -m "chore: Add TDG quality enforcement baseline"
git push
```

### 5. Verify CI Integration

Open a pull request with a code change. The CI pipeline should:
- ✅ Install PMAT
- ✅ Load existing baseline
- ✅ Run regression check
- ✅ Run quality check on new/modified files
- ✅ Generate TDG report
- ✅ Post comment on PR (GitHub Actions)
- ✅ Pass or fail based on enforcement mode

---

## GitHub Actions Integration

### Full Workflow Template

The GitHub Actions template (`templates/ci/github-actions-tdg.yml`) provides:

**Triggers**:
- Pull requests to main/master/develop branches
- Pushes to main/master/develop branches
- Manual workflow dispatch

**Jobs**:
1. **Checkout** - Full git history for context
2. **Install PMAT** - Cache cargo dependencies
3. **Baseline Check** - Load or create baseline
4. **Regression Check** - Detect quality degradation (PR only)
5. **Quality Check** - Enforce minimum grades for new files (PR only)
6. **Generate Report** - Create JSON/Markdown reports
7. **PR Comment** - Post quality results as PR comment
8. **Baseline Update** - Auto-update baseline on main branch commits

### Template Variables

The template includes placeholder variables that are substituted when you run `pmat hooks install --tdg-enforcement`:

```yaml
env:
  PMAT_VERSION: "{{PMAT_VERSION}}"           # Current PMAT version
  BASELINE_PATH: "{{BASELINE_PATH}}"         # From .pmat/tdg-rules.toml
  MIN_GRADE: "{{MIN_GRADE}}"                 # Minimum acceptable grade
  MAX_SCORE_DROP: "{{MAX_SCORE_DROP}}"       # Maximum allowed score drop
  MODE: "{{MODE}}"                           # strict, warning, disabled
```

### Customization Examples

**Example 1: Multi-Language Project**

```yaml
# Add language-specific quality gates
- name: Check Rust code quality
  run: |
    pmat tdg check-quality \
      --path . \
      --format table \
      --filter "*.rs" \
      --min-grade A \
      --fail-on-violation

- name: Check Python code quality
  run: |
    pmat tdg check-quality \
      --path . \
      --format table \
      --filter "*.py" \
      --min-grade B+ \
      --fail-on-violation
```

**Example 2: Nightly Quality Reports**

```yaml
on:
  schedule:
    - cron: '0 0 * * *'  # Daily at midnight

jobs:
  nightly_quality_report:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Generate comprehensive TDG report
        run: |
          pmat tdg --path . --format markdown --output nightly-report.md
          # Post to Slack, email, etc.
```

**Example 3: Quality Trend Analysis**

```yaml
- name: Compare against previous baseline
  run: |
    # Store daily baselines for trend analysis
    pmat tdg baseline create \
      --output .pmat/baselines/baseline-$(date +%Y%m%d).json \
      --path .

    # Compare with 7 days ago
    PREV_BASELINE=".pmat/baselines/baseline-$(date -d '7 days ago' +%Y%m%d).json"
    if [ -f "$PREV_BASELINE" ]; then
      pmat tdg baseline compare \
        --baseline "$PREV_BASELINE" \
        --current .pmat/baselines/baseline-$(date +%Y%m%d).json \
        --format table
    fi
```

### GitHub Actions Secrets

For baseline auto-update on main branch:

```yaml
# No additional secrets needed!
# Uses built-in GITHUB_TOKEN with write permissions
```

Ensure your workflow has write permissions:

```yaml
permissions:
  contents: write     # For committing baseline updates
  pull-requests: write # For PR comments
```

---

## GitLab CI Integration

### Pipeline Structure

The GitLab CI template (`templates/ci/gitlab-ci-tdg.yml`) provides:

**Stages**:
1. **install** - Install PMAT and check/create baseline
2. **analyze** - Run regression and quality checks (MR only)
3. **report** - Generate comprehensive TDG reports
4. **update** - Update baseline on main branch commits

**Caching**:
- Cargo dependencies cached across jobs
- PMAT binary cached between stages

### Template Variables

```yaml
variables:
  PMAT_VERSION: "{{PMAT_VERSION}}"
  BASELINE_PATH: "{{BASELINE_PATH}}"
  MIN_GRADE: "{{MIN_GRADE}}"
  MAX_SCORE_DROP: "{{MAX_SCORE_DROP}}"
  MODE: "{{MODE}}"
  GIT_DEPTH: "0"  # Full history for git context
```

### Customization Examples

**Example 1: Parallel Language Analysis**

```yaml
stage: analyze
parallel:
  matrix:
    - LANGUAGE: rust
      FILE_PATTERN: "*.rs"
      MIN_GRADE: "A"
    - LANGUAGE: python
      FILE_PATTERN: "*.py"
      MIN_GRADE: "B+"
    - LANGUAGE: typescript
      FILE_PATTERN: "*.ts"
      MIN_GRADE: "B"
script:
  - |
    pmat tdg check-quality \
      --path . \
      --filter "${FILE_PATTERN}" \
      --min-grade "${MIN_GRADE}" \
      --fail-on-violation
```

**Example 2: Merge Request Quality Report**

```yaml
tdg_mr_report:
  stage: report
  only:
    - merge_requests
  script:
    - pmat tdg --path . --format markdown --output mr-report.md
    - |
      # Post report to merge request discussion
      curl -X POST \
        -H "PRIVATE-TOKEN: ${CI_JOB_TOKEN}" \
        -d "body=$(cat mr-report.md)" \
        "${CI_API_V4_URL}/projects/${CI_PROJECT_ID}/merge_requests/${CI_MERGE_REQUEST_IID}/notes"
```

**Example 3: Quality Badge Generation**

```yaml
tdg_badge:
  stage: report
  script:
    - |
      # Generate quality badge based on average score
      AVG_SCORE=$(pmat tdg --path . --format json | jq '.summary.avg_score')
      GRADE=$(pmat tdg --path . --format json | jq -r '.summary.avg_grade')

      # Create badge JSON
      echo "{\"schemaVersion\": 1, \"label\": \"quality\", \"message\": \"${GRADE}\", \"color\": \"green\"}" > badge.json
  artifacts:
    paths:
      - badge.json
```

### GitLab Secrets

For baseline auto-update:

```yaml
# Uses built-in CI_JOB_TOKEN - no additional setup needed
git push "https://oauth2:${CI_JOB_TOKEN}@${CI_SERVER_HOST}/${CI_PROJECT_PATH}.git"
```

---

## Jenkins Integration

### Pipeline Structure

The Jenkins pipeline (`templates/ci/Jenkinsfile-tdg`) provides:

**Stages**:
1. **Setup** - Checkout with full git history
2. **Install PMAT** - Install or verify PMAT installation
3. **Baseline Check** - Load or create baseline
4. **Quality Analysis** - Parallel regression + quality checks
5. **Generate Report** - Create and archive TDG reports
6. **Update Baseline** - Auto-update on main/master branch

**Parallel Execution**:
- Regression check and quality check run in parallel (PR only)
- Reduces pipeline execution time

### Template Variables

```groovy
environment {
    PMAT_VERSION = '{{PMAT_VERSION}}'
    BASELINE_PATH = '{{BASELINE_PATH}}'
    MIN_GRADE = '{{MIN_GRADE}}'
    MAX_SCORE_DROP = '{{MAX_SCORE_DROP}}'
    MODE = '{{MODE}}'
}
```

### Customization Examples

**Example 1: Multi-Branch Quality Gates**

```groovy
stage('Quality Analysis') {
    when {
        anyOf {
            branch 'develop'
            branch pattern: "feature/.*", comparator: "REGEXP"
        }
    }
    steps {
        script {
            def minGrade = (env.BRANCH_NAME == 'develop') ? 'A' : 'B+'
            sh """
                pmat tdg check-quality \
                    --path . \
                    --min-grade ${minGrade} \
                    --fail-on-violation
            """
        }
    }
}
```

**Example 2: Slack Notifications**

```groovy
post {
    always {
        script {
            def qualityReport = readFile('tdg-report.md')
            slackSend(
                channel: '#code-quality',
                color: currentBuild.result == 'SUCCESS' ? 'good' : 'danger',
                message: """
                    Quality Gate: ${currentBuild.result}
                    Branch: ${env.BRANCH_NAME}
                    Build: ${env.BUILD_URL}

                    ${qualityReport}
                """
            )
        }
    }
}
```

**Example 3: Quality Metrics Dashboard**

```groovy
stage('Publish Metrics') {
    steps {
        script {
            // Parse JSON report and publish to InfluxDB/Prometheus
            sh '''
                AVG_SCORE=$(jq '.summary.avg_score' tdg-report.json)
                TOTAL_FILES=$(jq '.summary.total_files' tdg-report.json)

                # Publish to metrics backend
                curl -X POST "http://metrics.example.com/api/metrics" \
                    -d "pmat_avg_score,project=${JOB_NAME} value=${AVG_SCORE}"
                curl -X POST "http://metrics.example.com/api/metrics" \
                    -d "pmat_total_files,project=${JOB_NAME} value=${TOTAL_FILES}"
            '''
        }
    }
}
```

### Jenkins Credentials

For baseline auto-update:

1. Create Jenkins credentials (type: Username with password)
   - ID: `jenkins-git-credentials`
   - Username: Your git username
   - Password: Git token with push access

2. Use in pipeline:
```groovy
withCredentials([usernamePassword(
    credentialsId: 'jenkins-git-credentials',
    usernameVariable: 'GIT_USERNAME',
    passwordVariable: 'GIT_PASSWORD'
)]) {
    sh """
        git push https://${GIT_USERNAME}:${GIT_PASSWORD}@github.com/your-org/your-repo.git
    """
}
```

---

## Configuration

### TDG Rules Configuration (`.pmat/tdg-rules.toml`)

Complete configuration reference:

```toml
# Quality Gates Configuration
[quality_gates]
# Language-specific minimum grades
rust_min_grade = "A"
python_min_grade = "B+"
typescript_min_grade = "B"
javascript_min_grade = "B"
go_min_grade = "B+"
java_min_grade = "B"
cpp_min_grade = "B"

# Global thresholds
max_score_drop = 5.0           # Maximum allowed score drop (0-100)
allow_grade_drop = false       # Allow letter grade drops (e.g., A → B+)

# Enforcement mode
mode = "strict"                # strict, warning, or disabled

# Blocking behavior
block_on_regression = true     # Block commits with regressions
block_on_new_files_below_threshold = true  # Block low-quality new files

# Baseline Configuration
[baseline]
baseline_path = ".pmat/tdg-baseline.json"  # Baseline file location
auto_update_on_main = true     # Auto-update baseline on main branch commits
store_in_git = true            # Commit baseline to git repository

# CI/CD Integration
[ci_cd]
fail_on_regression = true      # CI fails if regression detected
fail_on_new_file_violation = true  # CI fails if new files below threshold
generate_reports = true        # Generate JSON/Markdown reports
comment_on_pr = true           # Comment on pull requests (GitHub only)
upload_artifacts = true        # Upload reports as CI artifacts
```

### Grade System

PMAT uses letter grades based on TDG scores:

| Grade | Score Range | Description |
|-------|-------------|-------------|
| A+    | 95-100      | Exceptional quality |
| A     | 90-94       | Excellent quality |
| A-    | 85-89       | Very good quality |
| B+    | 80-84       | Good quality |
| B     | 75-79       | Above average quality |
| B-    | 70-74       | Average quality |
| C+    | 65-69       | Below average quality |
| C     | 60-64       | Needs improvement |
| C-    | 55-59       | Significant issues |
| D     | 50-54       | Poor quality |
| F     | 0-49        | Unacceptable quality |

---

## Enforcement Modes

### Strict Mode

**Configuration**: `mode = "strict"`

**Behavior**:
- ❌ **Blocks** commits/merges on quality regressions
- ❌ **Blocks** commits/merges on new files below threshold
- ❌ **Fails** CI pipeline if quality gates not met
- ✅ Prevents quality degradation
- ✅ Enforces quality standards for all new code

**Use Case**: Production codebases, mature projects, zero-regression policy

**Example**:
```bash
# Pre-commit hook output (strict mode)
🔍 PMAT TDG Quality Enforcement
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

❌ Quality regression detected - commit blocked

File: server/src/services/parser.rs
Old Score: 85.2 (A-)
New Score: 78.4 (B)
Delta: -6.8 (exceeds threshold of 5.0)

To fix:
  1. Review quality issues above
  2. Improve code quality to meet standards
  3. Or update baseline if changes are intentional:
     pmat tdg baseline update --output ".pmat/tdg-baseline.json"
```

### Warning Mode

**Configuration**: `mode = "warning"`

**Behavior**:
- ⚠️  **Allows** commits/merges with quality warnings
- ⚠️  **Displays** quality issues but doesn't block
- ✅ CI pipeline passes but shows warnings
- ✅ Good for gradual quality improvement
- ✅ Raises awareness without blocking development

**Use Case**: Legacy codebases, gradual adoption, learning phase

**Example**:
```bash
# Pre-commit hook output (warning mode)
🔍 PMAT TDG Quality Enforcement
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

⚠️  Quality regression detected (warning mode)
   Commit allowed but please review quality issues

File: server/src/services/parser.rs
Old Score: 85.2 (A-)
New Score: 78.4 (B)
Delta: -6.8 (exceeds threshold of 5.0)

✅ Commit proceeding with warnings
```

### Disabled Mode

**Configuration**: `mode = "disabled"`

**Behavior**:
- ⏸️  **Skips** all quality checks
- ⏸️  **Skips** regression detection
- ⏸️  **Skips** quality gates
- ✅ Useful for temporary bypass during refactoring
- ⚠️  Not recommended for production use

**Use Case**: Emergency hotfixes, large-scale refactoring, temporary bypass

---

## Workflows and Strategies

### Strategy 1: Progressive Quality Adoption

**Goal**: Gradually improve codebase quality without blocking development

**Steps**:

1. **Phase 1: Baseline Creation (Week 1)**
   ```bash
   # Create initial baseline
   pmat tdg baseline create --output .pmat/tdg-baseline.json --path .

   # Review current state
   pmat tdg --path . --format table

   # Set mode to warning
   echo 'mode = "warning"' >> .pmat/tdg-rules.toml
   ```

2. **Phase 2: Awareness (Weeks 2-4)**
   - Mode: `warning`
   - CI runs but doesn't block
   - Team sees quality issues in PR comments
   - Focus on improving new code

3. **Phase 3: Selective Enforcement (Weeks 5-8)**
   ```toml
   mode = "strict"
   block_on_regression = false          # Allow regressions temporarily
   block_on_new_files_below_threshold = true  # Enforce for new code only
   ```

4. **Phase 4: Full Enforcement (Week 9+)**
   ```toml
   mode = "strict"
   block_on_regression = true           # Block all regressions
   block_on_new_files_below_threshold = true
   max_score_drop = 5.0
   ```

### Strategy 2: Zero-Regression Policy

**Goal**: Maintain or improve quality at all times

**Configuration**:
```toml
[quality_gates]
mode = "strict"
max_score_drop = 0.0                    # No score drops allowed
allow_grade_drop = false                 # No grade drops allowed
block_on_regression = true
block_on_new_files_below_threshold = true

rust_min_grade = "A-"
python_min_grade = "B+"
```

**Workflow**:
1. All PRs must maintain or improve quality
2. Regressions trigger CI failure
3. Team must fix quality issues before merge
4. Baseline auto-updates on main branch

### Strategy 3: Language-Specific Standards

**Goal**: Different quality standards for different languages

**Configuration**:
```toml
[quality_gates]
mode = "strict"

# Strict standards for core services (Rust)
rust_min_grade = "A"
max_score_drop = 3.0

# Moderate standards for scripts (Python)
python_min_grade = "B+"

# Lenient standards for frontend (TypeScript/JavaScript)
typescript_min_grade = "B"
javascript_min_grade = "B-"
```

**CI Workflow**:
```yaml
# Separate jobs for each language
jobs:
  rust_quality:
    steps:
      - run: pmat tdg check-quality --filter "*.rs" --min-grade A

  python_quality:
    steps:
      - run: pmat tdg check-quality --filter "*.py" --min-grade B+

  frontend_quality:
    steps:
      - run: pmat tdg check-quality --filter "*.ts" --min-grade B
```

---

## Troubleshooting

### Issue 1: Baseline Not Found

**Symptom**:
```
⚠️  No baseline found at .pmat/tdg-baseline.json
```

**Solutions**:

1. Create baseline manually:
   ```bash
   pmat tdg baseline create --output .pmat/tdg-baseline.json --path .
   ```

2. Check baseline path in config:
   ```toml
   [baseline]
   baseline_path = ".pmat/tdg-baseline.json"  # Verify this path
   ```

3. Ensure baseline is committed to git:
   ```bash
   git add .pmat/tdg-baseline.json
   git commit -m "chore: Add TDG baseline"
   ```

### Issue 2: CI Fails with "pmat: command not found"

**Symptom**:
```
❌ Error: pmat binary not found in PATH
```

**Solutions**:

1. **GitHub Actions**: Add Rust toolchain setup:
   ```yaml
   - uses: dtolnay/rust-toolchain@stable
   - run: cargo install pmat
   ```

2. **GitLab CI**: Ensure `rust:latest` image is used:
   ```yaml
   image: rust:latest
   ```

3. **Jenkins**: Install Rust in setup stage:
   ```groovy
   sh 'curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y'
   ```

### Issue 3: Excessive Regression Failures

**Symptom**:
```
❌ Quality regression detected - commit blocked
Delta: -10.2 (exceeds threshold of 5.0)
```

**Solutions**:

1. **Adjust threshold**:
   ```toml
   max_score_drop = 10.0  # Increase threshold
   ```

2. **Allow grade drops** (temporarily):
   ```toml
   allow_grade_drop = true
   ```

3. **Use warning mode during refactoring**:
   ```toml
   mode = "warning"
   ```

4. **Update baseline after intentional changes**:
   ```bash
   pmat tdg baseline update --output .pmat/tdg-baseline.json --path .
   git add .pmat/tdg-baseline.json
   git commit -m "chore: Update baseline after refactoring"
   ```

### Issue 4: CI Too Slow

**Symptom**: TDG analysis takes 10+ minutes in CI

**Solutions**:

1. **Enable caching** (GitHub Actions):
   ```yaml
   - uses: actions/cache@v4
     with:
       path: ~/.cargo
       key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
   ```

2. **Analyze only changed files**:
   ```bash
   # Get list of changed files
   CHANGED_FILES=$(git diff --name-only origin/main...HEAD)

   # Analyze only changed files
   for file in $CHANGED_FILES; do
     pmat tdg --path "$file" --format table
   done
   ```

3. **Use faster analysis mode** (if available):
   ```bash
   pmat tdg --path . --fast-mode
   ```

### Issue 5: Baseline Auto-Update Not Working

**Symptom**: Baseline not updating on main branch commits

**Solutions**:

1. **Check CI permissions** (GitHub Actions):
   ```yaml
   permissions:
     contents: write  # Required for git push
   ```

2. **Verify git configuration**:
   ```bash
   git config user.name "ci-bot"
   git config user.email "ci-bot@example.com"
   ```

3. **Check credentials** (Jenkins):
   ```groovy
   withCredentials([usernamePassword(
       credentialsId: 'jenkins-git-credentials',  // Verify this exists
       usernameVariable: 'GIT_USERNAME',
       passwordVariable: 'GIT_PASSWORD'
   )]) { ... }
   ```

---

## Best Practices

### 1. Baseline Management

✅ **DO**:
- Create baseline on stable main branch
- Commit baseline to git repository
- Update baseline after major refactoring
- Store historical baselines for trend analysis

❌ **DON'T**:
- Create baseline on feature branches
- Manually edit baseline files
- Update baseline to bypass quality gates
- Delete baselines without team review

### 2. Configuration

✅ **DO**:
- Start with `mode = "warning"` for legacy codebases
- Set language-specific thresholds based on team expertise
- Document rationale for threshold choices
- Review and adjust thresholds quarterly

❌ **DON'T**:
- Set `mode = "disabled"` in production
- Use identical thresholds for all languages
- Set unrealistic standards (e.g., `rust_min_grade = "A+"` for legacy code)
- Change thresholds without team consensus

### 3. CI Pipeline

✅ **DO**:
- Run quality checks on every pull request
- Auto-update baseline on main branch commits
- Generate and archive quality reports
- Use parallel jobs for multi-language projects

❌ **DON'T**:
- Skip quality checks on hotfix branches (use warning mode instead)
- Run quality checks only on nightly builds
- Ignore quality warnings in CI logs
- Block all development with overly strict gates

### 4. Team Workflow

✅ **DO**:
- Review quality reports in PR discussions
- Celebrate quality improvements
- Share best practices for improving scores
- Schedule regular quality retrospectives

❌ **DON'T**:
- Blame individuals for quality regressions
- Bypass quality gates with `--no-verify`
- Ignore quality warnings until they become errors
- Set quality goals without team input

### 5. Monitoring and Reporting

✅ **DO**:
- Track quality trends over time
- Create dashboards for quality metrics
- Send weekly quality summaries to team
- Celebrate milestones (e.g., "All files now B+ or higher!")

❌ **DON'T**:
- Focus only on average scores (distribution matters)
- Ignore outliers with very low scores
- Compare quality scores across different languages directly
- Use quality scores for performance reviews (focus on trends, not individuals)

---

## Summary

PMAT's TDG Enforcement System provides robust, automated quality gates for CI/CD pipelines across GitHub Actions, GitLab CI, and Jenkins. By following this guide, you can:

✅ Integrate quality checks into your existing CI/CD workflows
✅ Enforce zero-regression policies or gradual quality improvement
✅ Generate comprehensive quality reports for every build
✅ Automatically track and update quality baselines
✅ Customize enforcement modes and thresholds for your team's needs

**Next Steps**:
1. Install TDG enforcement: `pmat hooks install --tdg-enforcement`
2. Configure thresholds in `.pmat/tdg-rules.toml`
3. Copy CI template for your platform
4. Create initial baseline
5. Open a test PR and verify CI integration

**Support**:
- Documentation: https://paiml.github.io/pmat-book/
- Issues: https://github.com/paiml/paiml-mcp-agent-toolkit/issues
- Examples: See `templates/ci/` directory

---

**Version**: Sprint 66 Phase 4
**Date**: October 29, 2025
**Maintained by**: PAIML Team
