# CI/CD Integration Guide

This guide shows how to integrate pmat's quality analysis and refactoring commands into your CI/CD pipelines to maintain extreme quality standards automatically.

## Overview

All `pmat analyze` commands now support the `--fail-on-violation` flag, which makes them exit with code 1 when violations exceed configured thresholds. This enables seamless CI/CD integration for quality gates.

## Exit Codes

- **0**: Success - no violations found or within thresholds
- **1**: Failure - violations exceed configured thresholds

## Supported Commands

### Analyze Commands with CI/CD Support
- `pmat analyze complexity --fail-on-violation` - Exit if complexity exceeds thresholds
- `pmat analyze dead-code --fail-on-violation` - Exit if dead code percentage too high
- `pmat analyze satd --fail-on-violation` - Exit if ANY technical debt found
- `pmat quality-gate --fail-on-violation` - Comprehensive quality check

### Configurable Thresholds
- **Complexity**: `--max-cyclomatic` (default: 20), `--max-cognitive` (default: 15)
- **Dead Code**: `--max-percentage` (default: 15.0%)
- **SATD**: Zero tolerance when using `--fail-on-violation`

## GitHub Actions Integration

### Comprehensive Quality Gate

```yaml
name: Code Quality Check
on: [push, pull_request]

jobs:
  quality-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install pmat
        run: |
          cargo install pmat
          # Or use quick install:
          # curl -sSfL https://raw.githubusercontent.com/paiml/paiml-mcp-agent-toolkit/master/scripts/install.sh | sh
          # echo "$HOME/.local/bin" >> $GITHUB_PATH
          
      - name: Check Code Complexity
        run: |
          pmat analyze complexity \
            --max-cyclomatic 15 \
            --max-cognitive 10 \
            --fail-on-violation \
            --format json > complexity-report.json
            
      - name: Check Technical Debt
        run: |
          pmat analyze satd \
            --strict \
            --fail-on-violation \
            --format json > satd-report.json
            
      - name: Check Dead Code
        run: |
          pmat analyze dead-code \
            --max-percentage 10.0 \
            --fail-on-violation \
            --format json > deadcode-report.json
            
      - name: Run Comprehensive Quality Gate
        run: |
          pmat quality-gate \
            --fail-on-violation \
            --format json > quality-report.json
          
      - name: Upload quality report
        uses: actions/upload-artifact@v3
        with:
          name: quality-report
          path: quality-report.json
          
      - name: Comment PR with quality report
        if: github.event_name == 'pull_request'
        uses: actions/github-script@v6
        with:
          script: |
            const fs = require('fs');
            const report = JSON.parse(fs.readFileSync('quality-report.json', 'utf8'));
            
            const comment = `## 🤖 AI Refactoring Quality Report
            
            **Quality Metrics:**
            - Violations: ${report.quality_metrics.total_violations}
            - Max Complexity: ${report.quality_metrics.max_complexity}
            - Coverage: ${report.quality_metrics.coverage_percent}%
            - SATD Items: ${report.quality_metrics.satd_count}
            
            ${report.quality_metrics.total_violations === 0 ? '✅ All quality standards met!' : '❌ Quality standards not met'}`;
            
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: comment
            });
```

### Matrix Strategy for Multiple Checks

```yaml
name: Quality Matrix
on: [push, pull_request]

jobs:
  quality-matrix:
    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        check:
          - name: complexity
            cmd: analyze complexity --max-cyclomatic 15 --max-cognitive 10 --fail-on-violation
          - name: dead-code
            cmd: analyze dead-code --max-percentage 10.0 --fail-on-violation
          - name: technical-debt
            cmd: analyze satd --strict --fail-on-violation
    steps:
      - uses: actions/checkout@v3
      - name: Install pmat
        run: cargo install pmat
      - name: Run ${{ matrix.check.name }} check
        run: pmat ${{ matrix.check.cmd }}
```

## GitLab CI Integration

### .gitlab-ci.yml

```yaml
stages:
  - quality
  - test
  - deploy

variables:
  PMAT_VERSION: "latest"

quality-check:
  stage: quality
  image: rust:latest
  script:
    - cargo install pmat
    # Run all quality checks
    - pmat analyze complexity --max-cyclomatic 15 --fail-on-violation
    - pmat analyze satd --strict --fail-on-violation  
    - pmat analyze dead-code --max-percentage 10.0 --fail-on-violation
    - pmat quality-gate --fail-on-violation --format json --output quality-report.json
  artifacts:
    reports:
      codequality: quality-report.json
    paths:
      - quality-report.json
    expire_in: 1 week
  allow_failure: false

# Separate jobs for detailed reporting
complexity-check:
  stage: quality
  image: rust:latest
  script:
    - cargo install pmat
    - pmat analyze complexity --max-cyclomatic 15 --fail-on-violation --format json > complexity.json
  artifacts:
    paths:
      - complexity.json
```

## Jenkins Pipeline Integration

### Jenkinsfile

```groovy
pipeline {
    agent any
    
    environment {
        PMAT_VERSION = 'latest'
    }
    
    stages {
        stage('Install Tools') {
            steps {
                sh '''
                    curl -sSfL https://raw.githubusercontent.com/paiml/paiml-mcp-agent-toolkit/master/scripts/install.sh | sh
                    export PATH="$HOME/.local/bin:$PATH"
                '''
            }
        }
        
        stage('Quality Checks') {
            parallel {
                stage('Complexity') {
                    steps {
                        sh 'pmat analyze complexity --max-cyclomatic 15 --fail-on-violation'
                    }
                }
                stage('Technical Debt') {
                    steps {
                        sh 'pmat analyze satd --strict --fail-on-violation'
                    }
                }
                stage('Dead Code') {
                    steps {
                        sh 'pmat analyze dead-code --max-percentage 10.0 --fail-on-violation'
                    }
                }
            }
        }
        
        stage('Generate Reports') {
            steps {
                sh '''
                    pmat quality-gate --format json > quality-report.json
                    pmat analyze complexity --format sarif > complexity.sarif
                '''
            }
            post {
                always {
                    archiveArtifacts artifacts: '*.json,*.sarif', fingerprint: true
                    publishHTML([
                        allowMissing: false,
                        alwaysLinkToLastBuild: true,
                        keepAll: true,
                        reportDir: '.',
                        reportFiles: 'quality-report.json',
                        reportName: 'Quality Report'
                    ])
                }
            }
        }
    }
    
    post {
        failure {
            emailext(
                subject: "Quality Gate Failed: ${env.JOB_NAME} - ${env.BUILD_NUMBER}",
                body: "Quality checks failed. Please check the build logs.",
                to: "${env.CHANGE_AUTHOR_EMAIL}"
            )
        }
    }
}
```

## CircleCI Integration

### .circleci/config.yml

```yaml
version: 2.1

executors:
  rust-executor:
    docker:
      - image: rust:latest

jobs:
  quality-checks:
    executor: rust-executor
    steps:
      - checkout
      - run:
          name: Install pmat
          command: cargo install pmat
      - run:
          name: Check Complexity
          command: |
            pmat analyze complexity \
              --max-cyclomatic 15 \
              --max-cognitive 10 \
              --fail-on-violation
      - run:
          name: Check Technical Debt
          command: pmat analyze satd --strict --fail-on-violation
      - run:
          name: Check Dead Code  
          command: pmat analyze dead-code --max-percentage 10.0 --fail-on-violation
      - run:
          name: Generate Reports
          command: |
            pmat quality-gate --format json > quality-report.json
          when: always
      - store_artifacts:
          path: quality-report.json
          destination: quality-reports

workflows:
  quality-workflow:
    jobs:
      - quality-checks
```

## Pre-commit Hook Integration

### .pre-commit-config.yaml

```yaml
repos:
  - repo: local
    hooks:
      - id: pmat-complexity
        name: Check code complexity
        entry: pmat analyze complexity --max-cyclomatic 15 --fail-on-violation
        language: system
        pass_filenames: false
        
      - id: pmat-satd
        name: Check for technical debt
        entry: pmat analyze satd --strict --fail-on-violation
        language: system
        pass_filenames: false
        
      - id: pmat-dead-code
        name: Check for dead code
        entry: pmat analyze dead-code --max-percentage 10.0 --fail-on-violation
        language: system
        pass_filenames: false
```

### Git Hook Script

```bash
#!/bin/bash
# .git/hooks/pre-commit

echo "Running quality checks..."

# Check complexity
if ! pmat analyze complexity --max-cyclomatic 15 --fail-on-violation; then
    echo "❌ Complexity check failed. Please refactor complex functions."
    exit 1
fi

# Check for technical debt
if ! pmat analyze satd --strict --fail-on-violation; then
    echo "❌ Technical debt found. Please remove TODO/FIXME comments."
    exit 1
fi

# Check dead code percentage
if ! pmat analyze dead-code --max-percentage 10.0 --fail-on-violation; then
    echo "❌ Too much dead code. Please remove unused code."
    exit 1
fi

echo "✅ All quality checks passed!"
```

### Automated Refactoring Workflow

```yaml
name: Automated Refactoring
on:
  schedule:
    - cron: '0 2 * * 1'  # Weekly on Monday at 2 AM
  workflow_dispatch:     # Manual trigger

jobs:
  auto-refactor:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
          
      - name: Install pmat
        run: |
          curl -sSfL https://raw.githubusercontent.com/paiml/paiml-mcp-agent-toolkit/master/scripts/install.sh | sh
          echo "$HOME/.local/bin" >> $GITHUB_PATH
          
      - name: Run automated refactoring
        run: |
          pmat refactor auto --max-iterations 5 --format json > refactor-results.json
          
      - name: Check for changes
        id: changes
        run: |
          if [[ -n $(git status --porcelain) ]]; then
            echo "changes=true" >> $GITHUB_OUTPUT
          else
            echo "changes=false" >> $GITHUB_OUTPUT
          fi
          
      - name: Create Pull Request
        if: steps.changes.outputs.changes == 'true'
        uses: peter-evans/create-pull-request@v5
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
          commit-message: |
            🤖 Automated refactoring for quality standards
            
            - Functions refactored to complexity ≤10
            - Test coverage improved to ≥80%
            - SATD items eliminated
            - All lint violations fixed
            
            Generated with pmat refactor auto
          title: "🤖 Automated Quality Refactoring"
          body: |
            ## AI-Powered Automated Refactoring
            
            This PR contains automated refactoring to meet EXTREME quality standards:
            
            ### Quality Standards Enforced
            - ✅ Function complexity ≤ 10
            - ✅ Test coverage ≥ 80%
            - ✅ Zero SATD comments
            - ✅ All lints fixed
            
            ### Generated by
            `pmat refactor auto` - AI-powered automated refactoring
            
            Please review and merge if the changes look correct.
          branch: automated-refactoring
          delete-branch: true
```

## GitLab CI Integration

### .gitlab-ci.yml

```yaml
stages:
  - quality-check
  - refactor

variables:
  PMAT_VERSION: "latest"

quality-gate:
  stage: quality-check
  image: rust:latest
  before_script:
    - curl -sSfL https://raw.githubusercontent.com/paiml/paiml-mcp-agent-toolkit/master/scripts/install.sh | sh
    - export PATH="$HOME/.local/bin:$PATH"
  script:
    - pmat refactor auto --ci-mode --format json
  artifacts:
    reports:
      junit: quality-report.xml
    paths:
      - quality-report.json
    expire_in: 1 week
  rules:
    - if: $CI_PIPELINE_SOURCE == "merge_request_event"
    - if: $CI_COMMIT_BRANCH == $CI_DEFAULT_BRANCH

automated-refactor:
  stage: refactor
  image: rust:latest
  before_script:
    - curl -sSfL https://raw.githubusercontent.com/paiml/paiml-mcp-agent-toolkit/master/scripts/install.sh | sh
    - export PATH="$HOME/.local/bin:$PATH"
  script:
    - pmat refactor auto --max-iterations 3 --format detailed
    - |
      if [[ -n $(git status --porcelain) ]]; then
        git config --global user.email "ci@company.com"
        git config --global user.name "CI Automated Refactoring"
        git add .
        git commit -m "🤖 Automated refactoring for quality standards"
        git push origin HEAD:automated-refactoring-$CI_COMMIT_SHORT_SHA
        echo "Created branch: automated-refactoring-$CI_COMMIT_SHORT_SHA"
      fi
  rules:
    - if: $CI_PIPELINE_SOURCE == "schedule"
    - when: manual
```

## Jenkins Integration

### Jenkinsfile

```groovy
pipeline {
    agent any
    
    environment {
        PMAT_HOME = "${env.HOME}/.local/bin"
    }
    
    stages {
        stage('Setup') {
            steps {
                sh '''
                    curl -sSfL https://raw.githubusercontent.com/paiml/paiml-mcp-agent-toolkit/master/scripts/install.sh | sh
                    export PATH="$PMAT_HOME:$PATH"
                    pmat --version
                '''
            }
        }
        
        stage('Quality Check') {
            steps {
                sh '''
                    export PATH="$PMAT_HOME:$PATH"
                    pmat refactor auto --dry-run --format json > quality-report.json
                '''
            }
            post {
                always {
                    archiveArtifacts artifacts: 'quality-report.json', fingerprint: true
                    publishHTML([
                        allowMissing: false,
                        alwaysLinkToLastBuild: true,
                        keepAll: true,
                        reportDir: '.',
                        reportFiles: 'quality-report.json',
                        reportName: 'Quality Report'
                    ])
                }
            }
        }
        
        stage('Automated Refactoring') {
            when {
                anyOf {
                    triggeredBy 'TimerTrigger'
                    triggeredBy cause: 'UserIdCause'
                }
            }
            steps {
                sh '''
                    export PATH="$PMAT_HOME:$PATH"
                    pmat refactor auto --max-iterations 5 --format detailed > refactor-log.txt
                '''
            }
            post {
                always {
                    archiveArtifacts artifacts: 'refactor-log.txt', fingerprint: true
                }
            }
        }
    }
    
    post {
        failure {
            emailext (
                subject: "Quality Gate Failed: ${env.JOB_NAME} - ${env.BUILD_NUMBER}",
                body: "Quality standards not met. Please run 'pmat refactor auto' to fix issues.",
                to: "${env.CHANGE_AUTHOR_EMAIL}"
            )
        }
    }
}
```

## Azure DevOps Integration

### azure-pipelines.yml

```yaml
trigger:
  branches:
    include:
      - main
      - develop

pr:
  branches:
    include:
      - main

schedules:
  - cron: "0 2 * * 1"
    displayName: Weekly automated refactoring
    branches:
      include:
        - main

pool:
  vmImage: 'ubuntu-latest'

stages:
  - stage: QualityCheck
    displayName: 'Quality Gate'
    jobs:
      - job: QualityGate
        displayName: 'Run Quality Gate'
        steps:
          - script: |
              curl -sSfL https://raw.githubusercontent.com/paiml/paiml-mcp-agent-toolkit/master/scripts/install.sh | sh
              echo "##vso[task.prependpath]$HOME/.local/bin"
            displayName: 'Install pmat'
            
          - script: |
              pmat refactor auto --ci-mode --format json > quality-report.json
            displayName: 'Run Quality Check'
            
          - task: PublishBuildArtifacts@1
            inputs:
              pathToPublish: 'quality-report.json'
              artifactName: 'quality-report'
            displayName: 'Publish Quality Report'

  - stage: AutomatedRefactoring
    displayName: 'Automated Refactoring'
    condition: eq(variables['Build.Reason'], 'Schedule')
    jobs:
      - job: Refactor
        displayName: 'Auto Refactor'
        steps:
          - script: |
              curl -sSfL https://raw.githubusercontent.com/paiml/paiml-mcp-agent-toolkit/master/scripts/install.sh | sh
              echo "##vso[task.prependpath]$HOME/.local/bin"
            displayName: 'Install pmat'
            
          - script: |
              pmat refactor auto --max-iterations 3 --format json > refactor-results.json
            displayName: 'Run Automated Refactoring'
            
          - script: |
              if [[ -n $(git status --porcelain) ]]; then
                git config user.email "azure-devops@company.com"
                git config user.name "Azure DevOps Automated Refactoring"
                git checkout -b automated-refactoring-$(Build.BuildNumber)
                git add .
                git commit -m "🤖 Automated refactoring for quality standards"
                git push origin automated-refactoring-$(Build.BuildNumber)
                echo "##vso[task.logissue type=warning]Created branch: automated-refactoring-$(Build.BuildNumber)"
              fi
            displayName: 'Create Refactoring Branch'
            condition: succeeded()
```

## Docker Integration

### Dockerfile for CI

```dockerfile
FROM rust:1.75-slim

# Install pmat
RUN curl -sSfL https://raw.githubusercontent.com/paiml/paiml-mcp-agent-toolkit/master/scripts/install.sh | sh
ENV PATH="/root/.local/bin:${PATH}"

# Set working directory
WORKDIR /app

# Copy source code
COPY . .

# Run quality check
RUN pmat refactor auto --ci-mode --format json > quality-report.json

# Build if quality gates pass
RUN cargo build --release
```

### Docker Compose for Development

```yaml
version: '3.8'

services:
  quality-gate:
    build:
      context: .
      dockerfile: Dockerfile.quality
    volumes:
      - .:/app
      - quality-reports:/app/reports
    command: |
      sh -c "
        pmat refactor auto --dry-run --format json > /app/reports/quality-report.json
        pmat refactor auto --ci-mode
      "
    
  automated-refactor:
    build:
      context: .
      dockerfile: Dockerfile.quality
    volumes:
      - .:/app
    command: |
      sh -c "
        pmat refactor auto --max-iterations 5 --format detailed > refactor-log.txt
        echo 'Refactoring completed. Check refactor-log.txt for details.'
      "

volumes:
  quality-reports:
```

## Pre-commit Hooks

### .pre-commit-config.yaml

```yaml
repos:
  - repo: local
    hooks:
      - id: pmat-quality-check
        name: PMAT Quality Check
        entry: bash -c 'pmat refactor auto --dry-run --ci-mode'
        language: system
        files: '\.(rs|js|ts|py|java|kt|go|cpp|c)$'
        fail_fast: true
        verbose: true
```

### Git Hook Script

```bash
#!/bin/bash
# .git/hooks/pre-commit

echo "🤖 Running AI-powered quality check..."

# Check if pmat is installed
if ! command -v pmat &> /dev/null; then
    echo "❌ pmat not found. Installing..."
    curl -sSfL https://raw.githubusercontent.com/paiml/paiml-mcp-agent-toolkit/master/scripts/install.sh | sh
    export PATH="$HOME/.local/bin:$PATH"
fi

# Run quality check
if ! pmat refactor auto --dry-run --ci-mode --format json > /tmp/quality-check.json; then
    echo "❌ Quality standards not met!"
    echo ""
    echo "Run the following command to fix issues automatically:"
    echo "  pmat refactor auto"
    echo ""
    echo "Or to see what would be changed:"
    echo "  pmat refactor auto --dry-run --format detailed"
    exit 1
fi

echo "✅ Quality check passed! All standards met."
```

## Monitoring and Metrics

### Quality Metrics Dashboard

```bash
#!/bin/bash
# scripts/collect-quality-metrics.sh

# Collect daily quality metrics
pmat refactor auto --dry-run --format json | \
  jq '.quality_metrics + {"timestamp": now}' >> metrics/quality-history.jsonl

# Generate weekly quality report
if [[ $(date +%u) -eq 1 ]]; then
    echo "## Weekly Quality Report" > reports/weekly-$(date +%Y-%W).md
    echo "" >> reports/weekly-$(date +%Y-%W).md
    
    # Calculate trend
    cat metrics/quality-history.jsonl | \
      jq -s 'sort_by(.timestamp) | [first, last] | 
             "Complexity trend: " + (.[1].max_complexity - .[0].max_complexity | tostring)' \
      >> reports/weekly-$(date +%Y-%W).md
fi
```

## Best Practices

### 1. Incremental Integration
Start with dry-run mode to understand what the tool would change:
```bash
pmat refactor auto --dry-run --format detailed
```

### 2. Quality Gate Thresholds
Configure appropriate thresholds for your team:
- Start with current complexity levels and gradually decrease
- Set coverage targets based on existing coverage
- Plan SATD elimination timeline

### 3. Branch Strategy
- Use separate branches for automated refactoring
- Require human review for AI-generated changes
- Test thoroughly before merging

### 4. Monitoring
- Track quality metrics over time
- Monitor refactoring frequency and success rate
- Alert on quality degradation

### 5. Team Training
- Educate team on quality standards enforced
- Provide guidelines for reviewing AI-generated refactorings
- Establish processes for handling refactoring conflicts

This integration guide enables teams to maintain EXTREME quality standards automatically while minimizing manual overhead and ensuring consistent code quality across all projects.