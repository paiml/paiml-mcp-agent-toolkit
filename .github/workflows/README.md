# GitHub Actions Workflows

This directory contains the CI/CD workflows for the MCP Agent Toolkit project.

## Workflows

### 🔄 Continuous Integration (`ci.yml`)
- **Triggers**: Push to main branches, Pull requests
- **Jobs**:
  - **Lint**: Runs rustfmt and clippy
  - **Test**: Runs tests with coverage reporting
  - **Build**: Builds on multiple platforms (Linux, macOS, Windows)
  - **Docker**: Builds Docker image
  - **E2E Test**: Runs end-to-end tests
  - **Security**: Runs cargo audit

### 📦 Release (`release.yml`)
- **Triggers**: Git tags (v*)
- **Jobs**:
  - Creates GitHub release
  - Builds and uploads binaries for multiple platforms
  - Publishes Docker images to Docker Hub

### 🔐 Dependencies (`dependencies.yml`)
- **Triggers**: Weekly schedule, Manual dispatch
- **Jobs**:
  - Updates Rust dependencies
  - Runs security audit
  - Creates PRs for updates
  - Creates issues for vulnerabilities

### 📊 Code Quality (`code-quality.yml`)
- **Triggers**: Pull requests
- **Jobs**:
  - Checks code coverage (minimum 70%)
  - Analyzes code complexity
  - Checks documentation
  - Comments PR with metrics

### ✅ PR Checks (`pr-checks.yml`)
- **Triggers**: Pull request events
- **Jobs**:
  - Validates PR title format
  - Labels PR by size
  - Checks for merge conflicts

### 🚀 Benchmark (`benchmark.yml`)
- **Triggers**: Push to main, Pull requests
- **Jobs**:
  - Runs performance benchmarks
  - Measures startup time
  - Tracks memory usage
  - Comments results on PRs

## Required Secrets

No external secrets are required! The workflows use only the built-in `GITHUB_TOKEN` which is automatically provided by GitHub Actions.

## Branch Protection Rules

Recommended branch protection rules for `main`/`master`:

- Require pull request reviews
- Require status checks to pass:
  - `lint`
  - `test`
  - `build (ubuntu-latest, stable)`
  - `security`
- Require branches to be up to date
- Include administrators in restrictions

## Maintenance

- Dependabot will automatically create PRs for dependency updates
- Security vulnerabilities will create GitHub issues
- Workflow files are also monitored by Dependabot

## Local Testing

To test workflows locally, you can use [act](https://github.com/nektos/act):

```bash
# Test CI workflow
act -W .github/workflows/ci.yml

# Test specific job
act -W .github/workflows/ci.yml -j test
```