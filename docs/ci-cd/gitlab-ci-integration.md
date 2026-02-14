# GitLab CI/CD Integration for PMAT Mutation Testing

Complete guide for integrating PMAT mutation testing into GitLab CI/CD pipelines.

## Table of Contents

- [Quick Start](#quick-start)
- [Basic Setup](#basic-setup)
- [Multi-Language Support](#multi-language-support)
- [Quality Gates](#quality-gates)
- [Advanced Patterns](#advanced-patterns)
- [Caching Strategies](#caching-strategies)
- [Artifacts and Reports](#artifacts-and-reports)
- [Merge Request Widgets](#merge-request-widgets)
- [Troubleshooting](#troubleshooting)
- [Best Practices](#best-practices)
- [Complete Production Example](#complete-production-example)

## Quick Start

Add to `.gitlab-ci.yml`:

```yaml
mutation-test:
  stage: test
  image: rust:latest
  before_script:
    - cargo install pmat
  script:
    - pmat mutate --target src/ --failures-only
  only:
    - merge_requests
    - main
```

## Basic Setup

### Minimal Configuration

`.gitlab-ci.yml`:

```yaml
stages:
  - build
  - test
  - mutation

build:
  stage: build
  image: rust:latest
  script:
    - cargo build --release
  artifacts:
    paths:
      - target/release/
    expire_in: 1 hour

mutation-test:
  stage: mutation
  image: rust:latest
  needs:
    - build
  before_script:
    - cargo install pmat
  script:
    - pmat mutate --target src/ --output-format text
  artifacts:
    reports:
      junit: mutation-report.xml
    paths:
      - mutation-report.xml
      - mutation-report.md
    expire_in: 30 days
  only:
    - merge_requests
    - main
```

### With Unit Tests First

```yaml
stages:
  - test
  - mutation

unit-tests:
  stage: test
  image: rust:latest
  script:
    - cargo test --all-features
  only:
    - merge_requests
    - main

mutation-test:
  stage: mutation
  needs:
    - unit-tests
  image: rust:latest
  before_script:
    - cargo install pmat
  script:
    - pmat mutate --target src/ --failures-only
  allow_failure: false
  only:
    - merge_requests
    - main
```

## Multi-Language Support

### Rust Projects

```yaml
mutation-test-rust:
  stage: mutation
  image: rust:latest
  cache:
    key: rust-pmat
    paths:
      - ~/.cargo/bin/pmat
      - ~/.cargo/registry/
      - target/
  before_script:
    - |
      if [ ! -f ~/.cargo/bin/pmat ]; then
        cargo install pmat
      fi
  script:
    - pmat mutate --target src/ --threshold 85 --output-format json > mutation-results.json
  artifacts:
    reports:
      junit: mutation-junit.xml
    paths:
      - mutation-results.json
      - mutation-report.md
  only:
    - merge_requests
    - main
```

### Python Projects

```yaml
mutation-test-python:
  stage: mutation
  image: python:3.11
  cache:
    key: python-pmat
    paths:
      - ~/.cargo/bin/pmat
      - venv/
  before_script:
    # Install Rust and PMAT
    - apt-get update && apt-get install -y curl build-essential
    - curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    - source $HOME/.cargo/env
    - cargo install pmat
    # Install Python dependencies
    - python -m venv venv
    - source venv/bin/activate
    - pip install -r requirements.txt
  script:
    - source venv/bin/activate
    - pytest  # Run tests first
    - pmat mutate --target src/ --threshold 80 --output-format markdown > mutation-report.md
  coverage: '/Mutation Score: (\d+\.\d+)%/'
  artifacts:
    paths:
      - mutation-report.md
      - htmlcov/
    reports:
      coverage_report:
        coverage_format: cobertura
        path: coverage.xml
  only:
    - merge_requests
    - main
```

### TypeScript/JavaScript Projects

```yaml
mutation-test-typescript:
  stage: mutation
  image: node:18
  cache:
    key: node-pmat
    paths:
      - ~/.cargo/bin/pmat
      - node_modules/
  before_script:
    # Install Rust and PMAT
    - apt-get update && apt-get install -y curl build-essential
    - curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    - source $HOME/.cargo/env
    - cargo install pmat
    # Install Node dependencies
    - npm ci
  script:
    - npm test  # Run Jest tests first
    - pmat mutate --target src/ --threshold 85 --output-format json > mutation-results.json
  coverage: '/Mutation Score: (\d+\.\d+)%/'
  artifacts:
    paths:
      - mutation-results.json
      - mutation-report.md
      - coverage/
    reports:
      junit: junit.xml
      coverage_report:
        coverage_format: cobertura
        path: coverage/cobertura-coverage.xml
  only:
    - merge_requests
    - main
```

### Multi-Language Matrix

```yaml
.mutation-test-template: &mutation-test-template
  stage: mutation
  cache:
    key: $CI_JOB_NAME
    paths:
      - ~/.cargo/bin/pmat
      - target/
      - node_modules/
      - venv/
  before_script:
    - apt-get update && apt-get install -y curl build-essential
    - curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    - source $HOME/.cargo/env
    - |
      if [ ! -f ~/.cargo/bin/pmat ]; then
        cargo install pmat
      fi
  artifacts:
    paths:
      - mutation-report-$LANGUAGE.md
      - mutation-results-$LANGUAGE.json
    expire_in: 30 days
  only:
    - merge_requests
    - main

mutation-rust:
  <<: *mutation-test-template
  image: rust:latest
  variables:
    LANGUAGE: rust
    TARGET: src/
    THRESHOLD: 90
  script:
    - cargo test
    - pmat mutate --target $TARGET --threshold $THRESHOLD --output-format markdown > mutation-report-$LANGUAGE.md

mutation-python:
  <<: *mutation-test-template
  image: python:3.11
  variables:
    LANGUAGE: python
    TARGET: python/src/
    THRESHOLD: 85
  script:
    - python -m venv venv && source venv/bin/activate
    - pip install -r python/requirements.txt
    - pytest python/tests
    - pmat mutate --target $TARGET --threshold $THRESHOLD --output-format markdown > mutation-report-$LANGUAGE.md

mutation-typescript:
  <<: *mutation-test-template
  image: node:18
  variables:
    LANGUAGE: typescript
    TARGET: typescript/src/
    THRESHOLD: 85
  script:
    - npm ci
    - npm test
    - pmat mutate --target $TARGET --threshold $THRESHOLD --output-format markdown > mutation-report-$LANGUAGE.md
```

## Quality Gates

### Fail Pipeline on Low Mutation Score

```yaml
mutation-quality-gate:
  stage: mutation
  image: rust:latest
  before_script:
    - cargo install pmat
  script:
    - |
      pmat mutate --target src/ --threshold 85 --output-format json > mutation-results.json
      SCORE=$(jq -r '.mutation_score' mutation-results.json)
      echo "Mutation Score: $SCORE%"

      if (( $(echo "$SCORE < 85.0" | bc -l) )); then
        echo "❌ Mutation score $SCORE% is below threshold 85%"
        exit 1
      fi

      echo "✅ Mutation score $SCORE% meets quality gate"
  artifacts:
    paths:
      - mutation-results.json
      - mutation-report.md
  only:
    - merge_requests
    - main
```

### Multi-Threshold Strategy

```yaml
mutation-with-thresholds:
  stage: mutation
  image: rust:latest
  before_script:
    - cargo install pmat
  script:
    - |
      pmat mutate --target src/ --output-format json > mutation-results.json

      SCORE=$(jq -r '.mutation_score' mutation-results.json)
      KILLED=$(jq -r '.killed' mutation-results.json)
      SURVIVED=$(jq -r '.survived' mutation-results.json)

      echo "📊 Mutation Testing Results:"
      echo "  Score: $SCORE%"
      echo "  Killed: $KILLED"
      echo "  Survived: $SURVIVED"

      # Critical: Must pass 80%
      if (( $(echo "$SCORE < 80.0" | bc -l) )); then
        echo "❌ CRITICAL: Score below 80% (hard fail)"
        exit 1
      fi

      # Warning: Should be above 85%
      if (( $(echo "$SCORE < 85.0" | bc -l) )); then
        echo "⚠️  WARNING: Score below recommended 85%"
      fi

      # Excellent: Above 90%
      if (( $(echo "$SCORE >= 90.0" | bc -l) )); then
        echo "🌟 EXCELLENT: Score above 90%"
      fi
  artifacts:
    paths:
      - mutation-results.json
  only:
    - merge_requests
    - main
```

## Advanced Patterns

### Scheduled Mutation Testing

```yaml
mutation-nightly:
  stage: mutation
  image: rust:latest
  before_script:
    - cargo install pmat
  script:
    - pmat mutate --target src/ --output-format json > mutation-results-nightly.json
    - pmat mutate --target src/ --output-format markdown > mutation-report-nightly.md
  artifacts:
    paths:
      - mutation-results-nightly.json
      - mutation-report-nightly.md
    expire_in: 90 days
  only:
    - schedules
```

**Setup**: Go to **CI/CD > Schedules** in GitLab UI:
- Interval: `0 2 * * *` (2 AM daily)
- Target branch: `main`
- Variables: `SCHEDULED_PIPELINE=true`

### Differential Mutation Testing

```yaml
mutation-diff:
  stage: mutation
  image: rust:latest
  before_script:
    - cargo install pmat
    - apt-get update && apt-get install -y git
  script:
    - |
      # Get changed files
      git fetch origin $CI_MERGE_REQUEST_TARGET_BRANCH_NAME
      CHANGED_FILES=$(git diff --name-only origin/$CI_MERGE_REQUEST_TARGET_BRANCH_NAME...HEAD | grep '\.rs$' | tr '\n' ' ')

      if [ -z "$CHANGED_FILES" ]; then
        echo "No Rust files changed, skipping mutation testing"
        exit 0
      fi

      echo "Running mutation testing on changed files:"
      echo "$CHANGED_FILES"

      for FILE in $CHANGED_FILES; do
        echo "Testing $FILE..."
        pmat mutate --target "$FILE" --failures-only --output-format markdown >> mutation-report-diff.md
      done
  artifacts:
    paths:
      - mutation-report-diff.md
  only:
    - merge_requests
```

### Parallel Jobs by Module

```yaml
mutation-auth:
  stage: mutation
  image: rust:latest
  before_script:
    - cargo install pmat
  script:
    - pmat mutate --target src/auth/ --threshold 90 --output-format json > mutation-auth.json
  artifacts:
    paths:
      - mutation-auth.json
  only:
    - merge_requests
    - main

mutation-api:
  stage: mutation
  image: rust:latest
  before_script:
    - cargo install pmat
  script:
    - pmat mutate --target src/api/ --threshold 85 --output-format json > mutation-api.json
  artifacts:
    paths:
      - mutation-api.json
  only:
    - merge_requests
    - main

mutation-database:
  stage: mutation
  image: rust:latest
  before_script:
    - cargo install pmat
  script:
    - pmat mutate --target src/database/ --threshold 80 --output-format json > mutation-database.json
  artifacts:
    paths:
      - mutation-database.json
  only:
    - merge_requests
    - main
```

## Caching Strategies

### Aggressive Caching

```yaml
variables:
  CARGO_HOME: $CI_PROJECT_DIR/.cargo

cache:
  key: "$CI_JOB_NAME-$CI_COMMIT_REF_SLUG"
  paths:
    - .cargo/
    - target/
    - ~/.cargo/bin/pmat

mutation-test-cached:
  stage: mutation
  image: rust:latest
  cache:
    key: pmat-cache-$CI_PIPELINE_ID
    paths:
      - ~/.cargo/bin/pmat
      - target/
      - .cargo/
    policy: pull-push
  before_script:
    - |
      if [ ! -f ~/.cargo/bin/pmat ]; then
        echo "Installing PMAT..."
        cargo install pmat
      else
        echo "Using cached PMAT binary"
      fi
  script:
    - pmat mutate --target src/ --failures-only
  only:
    - merge_requests
    - main
```

### Language-Specific Caching

```yaml
.rust-cache: &rust-cache
  cache:
    key: rust-$CI_COMMIT_REF_SLUG
    paths:
      - ~/.cargo/bin/pmat
      - target/
      - .cargo/

.python-cache: &python-cache
  cache:
    key: python-$CI_COMMIT_REF_SLUG
    paths:
      - venv/
      - ~/.cargo/bin/pmat

.node-cache: &node-cache
  cache:
    key: node-$CI_COMMIT_REF_SLUG
    paths:
      - node_modules/
      - ~/.cargo/bin/pmat

mutation-rust:
  <<: *rust-cache
  stage: mutation
  image: rust:latest
  script:
    - pmat mutate --target src/ --threshold 90

mutation-python:
  <<: *python-cache
  stage: mutation
  image: python:3.11
  script:
    - source venv/bin/activate
    - pmat mutate --target python/src/ --threshold 85

mutation-node:
  <<: *node-cache
  stage: mutation
  image: node:18
  script:
    - pmat mutate --target typescript/src/ --threshold 85
```

## Artifacts and Reports

### Multiple Output Formats

```yaml
mutation-test-all-formats:
  stage: mutation
  image: rust:latest
  before_script:
    - cargo install pmat
  script:
    - pmat mutate --target src/ --output-format text > mutation-report.txt
    - pmat mutate --target src/ --output-format json > mutation-results.json
    - pmat mutate --target src/ --output-format markdown > mutation-report.md
  artifacts:
    name: "mutation-reports-$CI_COMMIT_SHORT_SHA"
    paths:
      - mutation-report.txt
      - mutation-results.json
      - mutation-report.md
    reports:
      junit: mutation-junit.xml
    expire_in: 30 days
  only:
    - merge_requests
    - main
```

### JUnit XML for Test Reports

```yaml
mutation-junit:
  stage: mutation
  image: rust:latest
  before_script:
    - cargo install pmat
  script:
    - |
      pmat mutate --target src/ --output-format json > mutation-results.json

      # Convert JSON to JUnit XML (example using jq and xmlstarlet)
      cat mutation-results.json | jq -r '.mutants[] | "\(.status) \(.location) \(.mutation)"' | \
        awk 'BEGIN {print "<?xml version=\"1.0\" encoding=\"UTF-8\"?><testsuite>"}
             {print "<testcase classname=\"mutation\" name=\""$1"\">"}
             {if ($1 == "Survived") print "<failure message=\"Mutation survived\"/>"}
             {print "</testcase>"}
             END {print "</testsuite>"}' > mutation-junit.xml
  artifacts:
    reports:
      junit: mutation-junit.xml
    paths:
      - mutation-results.json
      - mutation-junit.xml
  only:
    - merge_requests
    - main
```

### Code Quality Reports

```yaml
mutation-code-quality:
  stage: mutation
  image: rust:latest
  before_script:
    - cargo install pmat
  script:
    - |
      pmat mutate --target src/ --output-format json > mutation-results.json

      # Generate Code Quality report (GitLab format)
      jq '[.mutants[] | select(.status == "Survived") | {
        type: "issue",
        check_name: "mutation-survived",
        description: ("Mutation survived: " + .mutation),
        categories: ["Bug Risk"],
        location: {
          path: .location | split(":")[0],
          lines: {
            begin: (.location | split(":")[1] | tonumber)
          }
        },
        severity: "major",
        fingerprint: (.location + .mutation | @base64)
      }]' mutation-results.json > code-quality.json
  artifacts:
    reports:
      codequality: code-quality.json
    paths:
      - mutation-results.json
      - code-quality.json
  only:
    - merge_requests
    - main
```

## Merge Request Widgets

### Comment on MR with Results

```yaml
mutation-mr-comment:
  stage: mutation
  image: rust:latest
  before_script:
    - cargo install pmat
    - apt-get update && apt-get install -y jq curl
  script:
    - |
      pmat mutate --target src/ --output-format json > mutation-results.json
      pmat mutate --target src/ --output-format markdown > mutation-report.md

      SCORE=$(jq -r '.mutation_score' mutation-results.json)
      KILLED=$(jq -r '.killed' mutation-results.json)
      SURVIVED=$(jq -r '.survived' mutation-results.json)
      TOTAL=$(jq -r '.total_mutants' mutation-results.json)

      # Determine emoji based on score
      if (( $(echo "$SCORE >= 90.0" | bc -l) )); then
        EMOJI="🟢"
        STATUS="Excellent"
      elif (( $(echo "$SCORE >= 80.0" | bc -l) )); then
        EMOJI="🟡"
        STATUS="Good"
      else
        EMOJI="🔴"
        STATUS="Needs Improvement"
      fi

      # Create MR comment
      COMMENT="## ${EMOJI} Mutation Testing - ${SCORE}% (${STATUS})

      **Mutation Testing Results:**
      - **Score:** ${SCORE}%
      - **Killed:** ${KILLED} / ${TOTAL}
      - **Survived:** ${SURVIVED}

      <details>
      <summary>📋 Full Report</summary>

      $(cat mutation-report.md)

      </details>
      "

      # Post comment to MR (requires GITLAB_TOKEN with api scope)
      curl --request POST \
        --header "PRIVATE-TOKEN: $GITLAB_TOKEN" \
        --data-urlencode "body=$COMMENT" \
        "$CI_API_V4_URL/projects/$CI_PROJECT_ID/merge_requests/$CI_MERGE_REQUEST_IID/notes"
  artifacts:
    paths:
      - mutation-results.json
      - mutation-report.md
  only:
    - merge_requests
```

**Setup**:
1. Create project access token: **Settings > Access Tokens**
2. Scopes: `api`, `read_repository`, `write_repository`
3. Add as CI/CD variable: **Settings > CI/CD > Variables**
   - Key: `GITLAB_TOKEN`
   - Value: `<your-token>`
   - Protected: Yes
   - Masked: Yes

### Update Merge Request Description

<!-- pmat:ignore-link -->
````yaml
mutation-update-mr:
  stage: mutation
  image: rust:latest
  before_script:
    - cargo install pmat
    - apt-get update && apt-get install -y jq curl
  script:
    - |
      pmat mutate --target src/ --output-format json > mutation-results.json

      SCORE=$(jq -r '.mutation_score' mutation-results.json)

      # Get current MR description
      CURRENT_DESC=$(curl --silent --header "PRIVATE-TOKEN: $GITLAB_TOKEN" \
        "$CI_API_V4_URL/projects/$CI_PROJECT_ID/merge_requests/$CI_MERGE_REQUEST_IID" | \
        jq -r '.description')

      # Append mutation results
      NEW_DESC="${CURRENT_DESC}

---

## 🧬 Mutation Testing Results

**Score:** ${SCORE}%

See [artifacts](${CI_PROJECT_URL}/-/jobs/${CI_JOB_ID}/artifacts/browse) for detailed report.
"

      # Update MR description
      curl --request PUT \
        --header "PRIVATE-TOKEN: $GITLAB_TOKEN" \
        --header "Content-Type: application/json" \
        --data "{\"description\": $(echo "$NEW_DESC" | jq -Rs .)}" \
        "$CI_API_V4_URL/projects/$CI_PROJECT_ID/merge_requests/$CI_MERGE_REQUEST_IID"
  only:
    - merge_requests
````

## Troubleshooting

### Long-Running Tests

```yaml
mutation-test-timeout:
  stage: mutation
  image: rust:latest
  timeout: 2h  # Increase job timeout
  before_script:
    - cargo install pmat
  script:
    - pmat mutate --target src/ --timeout 60 --jobs 2  # 60s per mutant, 2 parallel jobs
  artifacts:
    paths:
      - mutation-report.md
    when: always  # Save artifacts even on failure
  only:
    - merge_requests
    - main
```

### Memory Issues

```yaml
mutation-memory-optimized:
  stage: mutation
  image: rust:latest
  variables:
    CARGO_BUILD_JOBS: 2
    RUST_BACKTRACE: 1
  before_script:
    - cargo install pmat
  script:
    - pmat mutate --target src/ --jobs 1  # Reduce parallelism
  only:
    - merge_requests
    - main
```

### Debug Mode

```yaml
mutation-debug:
  stage: mutation
  image: rust:latest
  variables:
    RUST_LOG: debug
    RUST_BACKTRACE: full
  before_script:
    - cargo install pmat
  script:
    - pmat mutate --target src/ --failures-only --verbose 2>&1 | tee mutation-debug.log
  artifacts:
    paths:
      - mutation-debug.log
    when: always
  only:
    - merge_requests
```

### Conditional Execution

```yaml
mutation-conditional:
  stage: mutation
  image: rust:latest
  before_script:
    - cargo install pmat
  script:
    - |
      if [ "$CI_MERGE_REQUEST_TITLE" =~ "WIP" ]; then
        echo "Skipping mutation testing for WIP merge request"
        exit 0
      fi

      pmat mutate --target src/ --threshold 85
  only:
    - merge_requests
  except:
    variables:
      - $CI_MERGE_REQUEST_TITLE =~ /^Draft:/
```

## Best Practices

1. **Run mutation testing after unit tests pass** - Don't waste time on broken code
2. **Use `--failures-only` for faster feedback** - Focus on survived mutants
3. **Cache PMAT binary** - Avoid reinstalling on every pipeline run
4. **Set reasonable timeouts** - Prevent infinite loops from blocking pipelines
5. **Use parallel jobs for large codebases** - Mutation test different modules concurrently
6. **Store artifacts for 30 days** - Keep historical mutation reports
7. **Schedule nightly comprehensive runs** - Run full mutation testing off critical path
8. **Use differential testing for MRs** - Only test changed files in merge requests
9. **Integrate with merge request widgets** - Surface results directly in GitLab UI
10. **Set appropriate thresholds per module** - Critical modules may need 90%+, utilities 80%+

## Complete Production Example

`.gitlab-ci.yml` for production-ready mutation testing pipeline:

```yaml
stages:
  - build
  - test
  - mutation
  - report

variables:
  CARGO_HOME: $CI_PROJECT_DIR/.cargo
  RUST_VERSION: "1.75.0"
  MUTATION_THRESHOLD: 85

# Build stage
build-debug:
  stage: build
  image: rust:${RUST_VERSION}
  cache:
    key: rust-debug-$CI_COMMIT_REF_SLUG
    paths:
      - .cargo/
      - target/
  script:
    - cargo build --all-features
  artifacts:
    paths:
      - target/debug/
    expire_in: 1 hour
  only:
    - merge_requests
    - main

build-release:
  stage: build
  image: rust:${RUST_VERSION}
  cache:
    key: rust-release-$CI_COMMIT_REF_SLUG
    paths:
      - .cargo/
      - target/
  script:
    - cargo build --release --all-features
  artifacts:
    paths:
      - target/release/
    expire_in: 1 day
  only:
    - main
    - tags

# Test stage
unit-tests:
  stage: test
  image: rust:${RUST_VERSION}
  needs:
    - build-debug
  cache:
    key: rust-debug-$CI_COMMIT_REF_SLUG
    paths:
      - .cargo/
      - target/
    policy: pull
  script:
    - cargo test --all-features
  coverage: '/^TOTAL\s+\d+\s+\d+\s+(\d+)%/'
  artifacts:
    reports:
      junit: test-results.xml
      coverage_report:
        coverage_format: cobertura
        path: coverage.xml
  only:
    - merge_requests
    - main

integration-tests:
  stage: test
  image: rust:${RUST_VERSION}
  needs:
    - build-debug
  services:
    - postgres:14
  variables:
    POSTGRES_DB: testdb
    POSTGRES_USER: testuser
    POSTGRES_PASSWORD: testpass
  cache:
    key: rust-debug-$CI_COMMIT_REF_SLUG
    paths:
      - .cargo/
      - target/
    policy: pull
  script:
    - cargo test --test integration_tests
  only:
    - merge_requests
    - main

# Mutation testing stage
.mutation-template: &mutation-template
  stage: mutation
  image: rust:${RUST_VERSION}
  cache:
    key: pmat-$CI_COMMIT_REF_SLUG
    paths:
      - ~/.cargo/bin/pmat
      - .cargo/
      - target/
    policy: pull-push
  before_script:
    - apt-get update && apt-get install -y jq curl bc
    - |
      if [ ! -f ~/.cargo/bin/pmat ]; then
        echo "Installing PMAT..."
        cargo install pmat
      else
        echo "Using cached PMAT binary"
        pmat --version
      fi
  artifacts:
    name: "mutation-reports-$CI_COMMIT_SHORT_SHA"
    paths:
      - mutation-results-*.json
      - mutation-report-*.md
    expire_in: 30 days
    when: always
  only:
    - merge_requests
    - main

mutation-auth-module:
  <<: *mutation-template
  needs:
    - unit-tests
  script:
    - |
      echo "Running mutation testing on authentication module..."
      pmat mutate --target src/auth/ --threshold 90 --output-format json > mutation-results-auth.json
      pmat mutate --target src/auth/ --output-format markdown > mutation-report-auth.md

      SCORE=$(jq -r '.mutation_score' mutation-results-auth.json)
      echo "Authentication Module Mutation Score: $SCORE%"

      if (( $(echo "$SCORE < 90.0" | bc -l) )); then
        echo "❌ Authentication module mutation score below 90%"
        exit 1
      fi

mutation-api-module:
  <<: *mutation-template
  needs:
    - unit-tests
  script:
    - |
      echo "Running mutation testing on API module..."
      pmat mutate --target src/api/ --threshold 85 --output-format json > mutation-results-api.json
      pmat mutate --target src/api/ --output-format markdown > mutation-report-api.md

      SCORE=$(jq -r '.mutation_score' mutation-results-api.json)
      echo "API Module Mutation Score: $SCORE%"

mutation-database-module:
  <<: *mutation-template
  needs:
    - integration-tests
  script:
    - |
      echo "Running mutation testing on database module..."
      pmat mutate --target src/database/ --threshold 80 --output-format json > mutation-results-database.json
      pmat mutate --target src/database/ --output-format markdown > mutation-report-database.md

      SCORE=$(jq -r '.mutation_score' mutation-results-database.json)
      echo "Database Module Mutation Score: $SCORE%"

mutation-differential:
  <<: *mutation-template
  needs:
    - unit-tests
  script:
    - |
      # Only run on merge requests
      if [ -z "$CI_MERGE_REQUEST_IID" ]; then
        echo "Not a merge request, skipping differential testing"
        exit 0
      fi

      # Get changed Rust files
      git fetch origin $CI_MERGE_REQUEST_TARGET_BRANCH_NAME
      CHANGED_FILES=$(git diff --name-only origin/$CI_MERGE_REQUEST_TARGET_BRANCH_NAME...HEAD | grep '\.rs$' | tr '\n' ' ')

      if [ -z "$CHANGED_FILES" ]; then
        echo "No Rust files changed"
        exit 0
      fi

      echo "Changed files: $CHANGED_FILES"

      # Run mutation testing on changed files
      for FILE in $CHANGED_FILES; do
        if [ -f "$FILE" ]; then
          echo "Testing $FILE..."
          pmat mutate --target "$FILE" --failures-only --output-format markdown >> mutation-report-diff.md
        fi
      done
  only:
    - merge_requests

# Report stage
mutation-summary:
  stage: report
  image: rust:${RUST_VERSION}
  needs:
    - mutation-auth-module
    - mutation-api-module
    - mutation-database-module
  before_script:
    - apt-get update && apt-get install -y jq curl bc
  script:
    - |
      # Aggregate mutation results
      TOTAL_MUTANTS=0
      TOTAL_KILLED=0
      TOTAL_SURVIVED=0

      for RESULT in mutation-results-*.json; do
        MUTANTS=$(jq -r '.total_mutants' "$RESULT")
        KILLED=$(jq -r '.killed' "$RESULT")
        SURVIVED=$(jq -r '.survived' "$RESULT")

        TOTAL_MUTANTS=$((TOTAL_MUTANTS + MUTANTS))
        TOTAL_KILLED=$((TOTAL_KILLED + KILLED))
        TOTAL_SURVIVED=$((TOTAL_SURVIVED + SURVIVED))
      done

      OVERALL_SCORE=$(echo "scale=2; ($TOTAL_KILLED / $TOTAL_MUTANTS) * 100" | bc)

      echo "📊 Overall Mutation Testing Summary"
      echo "===================================="
      echo "Total Mutants: $TOTAL_MUTANTS"
      echo "Killed: $TOTAL_KILLED"
      echo "Survived: $TOTAL_SURVIVED"
      echo "Overall Score: $OVERALL_SCORE%"

      # Create summary report
      cat > mutation-summary.md <<EOF
      # 🧬 Mutation Testing Summary

      **Overall Score:** ${OVERALL_SCORE}%

      ## Module Breakdown

      | Module | Score | Killed | Survived | Total |
      |--------|-------|--------|----------|-------|
      $(for RESULT in mutation-results-*.json; do
        MODULE=$(basename "$RESULT" .json | sed 's/mutation-results-//')
        SCORE=$(jq -r '.mutation_score' "$RESULT")
        KILLED=$(jq -r '.killed' "$RESULT")
        SURVIVED=$(jq -r '.survived' "$RESULT")
        TOTAL=$(jq -r '.total_mutants' "$RESULT")
        echo "| $MODULE | ${SCORE}% | $KILLED | $SURVIVED | $TOTAL |"
      done)

      ## Quality Gate

      $(if (( $(echo "$OVERALL_SCORE >= $MUTATION_THRESHOLD" | bc -l) )); then
        echo "✅ **PASSED** - Overall score ${OVERALL_SCORE}% meets threshold ${MUTATION_THRESHOLD}%"
      else
        echo "❌ **FAILED** - Overall score ${OVERALL_SCORE}% below threshold ${MUTATION_THRESHOLD}%"
      fi)
      EOF

      cat mutation-summary.md

      # Post to merge request if applicable
      if [ -n "$CI_MERGE_REQUEST_IID" ] && [ -n "$GITLAB_TOKEN" ]; then
        curl --request POST \
          --header "PRIVATE-TOKEN: $GITLAB_TOKEN" \
          --data-urlencode "body=$(cat mutation-summary.md)" \
          "$CI_API_V4_URL/projects/$CI_PROJECT_ID/merge_requests/$CI_MERGE_REQUEST_IID/notes"
      fi

      # Fail if below threshold
      if (( $(echo "$OVERALL_SCORE < $MUTATION_THRESHOLD" | bc -l) )); then
        exit 1
      fi
  artifacts:
    paths:
      - mutation-summary.md
  only:
    - merge_requests
    - main

# Scheduled comprehensive mutation testing
mutation-nightly:
  stage: mutation
  image: rust:${RUST_VERSION}
  cache:
    key: pmat-nightly
    paths:
      - ~/.cargo/bin/pmat
      - .cargo/
      - target/
  before_script:
    - cargo install pmat
  script:
    - |
      echo "Running comprehensive nightly mutation testing..."
      pmat mutate --target src/ --output-format json > mutation-results-nightly.json
      pmat mutate --target src/ --output-format markdown > mutation-report-nightly.md

      SCORE=$(jq -r '.mutation_score' mutation-results-nightly.json)
      echo "Nightly Mutation Score: $SCORE%"
  artifacts:
    paths:
      - mutation-results-nightly.json
      - mutation-report-nightly.md
    expire_in: 90 days
  only:
    - schedules
```

## Additional Resources

- **PMAT Documentation**: `server/README.md`
- **GitLab CI/CD Docs**: https://docs.gitlab.com/ee/ci/
- **Mutation Testing Concepts**: `examples/*/README.md`
- **GitHub Actions Guide**: `docs/ci-cd/github-actions-integration.md`

## Summary

GitLab CI/CD provides powerful features for mutation testing integration:

✅ **Parallel jobs** - Test multiple modules concurrently
✅ **Merge request widgets** - Surface results in MR UI
✅ **Scheduled pipelines** - Run comprehensive tests nightly
✅ **Code quality reports** - Integrate with GitLab's native quality features
✅ **Flexible caching** - Speed up pipelines with smart caching
✅ **JUnit integration** - Standard test reporting format
✅ **Artifact management** - Store and browse historical reports

For production deployments, combine unit tests, integration tests, and mutation testing in a multi-stage pipeline with appropriate quality gates.
