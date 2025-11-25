# PMAT - Pragmatic AI Labs Multi-language Agent Toolkit


<!-- PMAT-REPO-SCORE:START -->
![Repository Health](https://img.shields.io/badge/repo%20health-104%2F125%20(A%2B)-brightgreen?style=flat-square)
<!-- PMAT-REPO-SCORE:END -->

[![Documentation](https://img.shields.io/badge/docs-pmat--book-blue)](https://paiml.github.io/pmat-book/)
[![Crates.io](https://img.shields.io/crates/v/pmat.svg)](https://crates.io/crates/pmat)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Version](https://img.shields.io/badge/version-2.195.0-green)](https://github.com/paiml/paiml-mcp-agent-toolkit/releases/tag/v2.195.0)

**Zero-configuration AI context generation** for any codebase. Analyze code quality, complexity, and technical debt across 17+ programming languages with extreme quality enforcement and Toyota Way standards.

📖 **[https://paiml.github.io/pmat-book/](https://paiml.github.io/pmat-book/)** - Complete documentation, tutorials, and guides

---

## Quick Start

### Installation

```bash
# Rust (recommended)
cargo install pmat

# macOS/Linux
brew install pmat

# Windows
choco install pmat

# npm (global)
npm install -g pmat-agent
```

### Basic Usage

```bash
# Analyze codebase and generate AI-ready context
pmat context

# Analyze complexity
pmat analyze complexity

# Grade technical debt (A+ through F)
pmat analyze tdg

# Score repository health (0-110 scale)
pmat repo-score .              # Fast: scans HEAD only
pmat repo-score . --deep       # Thorough: scans entire git history

# Find Self-Admitted Technical Debt
pmat analyze satd

# Test suite quality (mutation testing)
pmat mutate --target src/
```

### Git Hooks Setup

Install pre-commit hooks for automatic quality enforcement:

```bash
# Install git hooks (bashrs quality, pmat-book validation)
pmat hooks install

# Check hook status
pmat hooks status

# Dry-run to see what would be checked
pmat hooks install --dry-run
```

**Hooks enforce:**
- Bash/Makefile safety (bashrs linting)
- pmat-book validation (multi-language examples)
- Documentation accuracy (zero hallucinations)

---

## Features

- **17+ Languages**: Rust, TypeScript, Python, Go, Java, C/C++, Ruby, PHP, Swift, Kotlin, and more
- **AI-Ready Context**: Generate deep context for Claude, GPT, and other LLMs
- **Technical Debt Grading (TDG)**: A+ through F scoring with 6 orthogonal metrics
- **Repository Health Scoring** ✨NEW: Quantitative assessment (0-110 scale) across 6 categories + bonus features
- **Rust Project Score (v2.0)** ✨NEW: Evidence-based Rust quality scoring (0-211 scale) with CI/CD integration
- **Workflow Prompts** ✨NEW: 11 pre-configured AI prompts enforcing EXTREME TDD and Toyota Way principles
- **Git-Commit Correlation**: Track TDG scores at specific commits for quality archaeology
- **Semantic Code Search**: Natural language code discovery with hybrid search
- **Quality Gates**: Pre-commit hooks, CI/CD integration, mutation testing
- **MCP Integration**: 19 tools for Claude Code, Cline, and other MCP clients
- **Organizational Intelligence** ✨NEW: Context-aware AI prompts from historical defect patterns (`--features org-intelligence`)
- **Zero Configuration**: Works out of the box on any codebase

---

## Documentation

📖 **[PMAT Book](https://paiml.github.io/pmat-book/)** - Complete guide with tutorials

**Key chapters:**
- [Installation and Setup](https://paiml.github.io/pmat-book/ch01-00-installation.html)
- [Getting Started](https://paiml.github.io/pmat-book/ch02-00-getting-started.html)
- [MCP Protocol](https://paiml.github.io/pmat-book/ch03-00-mcp-protocol.html)
- [Technical Debt Grading](https://paiml.github.io/pmat-book/ch04-01-tdg.html)
- [Workflow Prompts](https://paiml.github.io/pmat-book/ch09-01-prompt-command.html) ✨NEW
- [Repository Health Scoring](https://paiml.github.io/pmat-book/ch31-00-repo-score.html) ✨NEW
- [Multi-Language Examples](https://paiml.github.io/pmat-book/ch13-00-language-examples.html)

---

## Examples

```bash
# Generate context for Claude/GPT
pmat context --output context.md --format llm-optimized

# Analyze TypeScript project
pmat analyze complexity --language typescript

# Technical debt grading with components
pmat analyze tdg --include-components

# Semantic search (natural language)
pmat embed sync ./src
pmat semantic search "error handling patterns"

# Validate documentation for hallucinations
pmat validate-readme --targets README.md
```

---

## Workflow Prompts (v2.194.0+)

Pre-configured AI workflow prompts that enforce **EXTREME TDD** and **Toyota Way** quality principles. Perfect for piping to Claude Code, ChatGPT, or other AI assistants.

```bash
# List all available prompts
pmat prompt --list

# Show specific prompt (YAML format)
pmat prompt code-coverage

# Get prompt as text for AI assistants
pmat prompt debug --format text | pbcopy

# JSON format for programmatic use
pmat prompt quality-enforcement --format json

# Customize for non-Rust projects
pmat prompt code-coverage \
  --set TEST_CMD="pytest" \
  --set COVERAGE_CMD="pytest --cov"

# Save to file
pmat prompt continue -o workflow.yaml
```

**Available Prompts (11 total):**

**CRITICAL Priority:**
- `code-coverage` - Enforce 85%+ coverage using EXTREME TDD
- `debug` - Five Whys root cause analysis
- `quality-enforcement` - Run all quality gates (12 gates)
- `security-audit` - Security analysis and fixes

**HIGH Priority:**
- `continue` - Continue next best step with EXTREME TDD
- `assert-cmd-testing` - Verify CLI test coverage
- `mutation-testing` - Run mutation testing on weak code
- `performance-optimization` - Speed up compilation and tests
- `refactor-hotspots` - Refactor high-TDG code

**MEDIUM Priority:**
- `clean-repo-cruft` - Remove temporary files
- `documentation` - Update and validate docs

**Key Features:**
- Toyota Way principles (Jidoka, Andon Cord, Five Whys)
- Variable substitution for multi-language support
- Quality gates and time constraints
- Zero-tolerance policies
- Short alias: `pmat p --list`

**Documentation:** [Workflow Prompts Guide](https://paiml.github.io/pmat-book/ch09-01-prompt-command.html)

---

## Mutation Testing

Evaluate test suite quality by introducing code mutations and checking if tests detect them.

```bash
# Basic mutation testing
pmat mutate --target src/lib.rs

# With quality gate (fail if score < 85%)
pmat mutate --target src/ --threshold 85

# Failures only (CI/CD optimization)
pmat mutate --target src/ --failures-only

# JSON output for integration
pmat mutate --target src/ --output-format json > results.json
```

**Mutation Score** = (Killed Mutants / Total Valid Mutants) × 100%

**Supported Languages:** Rust, Python, TypeScript, JavaScript, Go, C++

**Key Features:**
- Multi-language mutation operators (arithmetic, comparison, logical, boundary)
- CI/CD integration (GitHub Actions, GitLab CI, Jenkins)
- Performance optimization (parallel execution, differential testing)
- Quality gates with configurable thresholds

**Documentation:**
- [User Guide](docs/guides/mutation-testing.md) - Complete guide with examples
- [API Reference](docs/guides/mutation-testing-api-reference.md) - All CLI flags and options
- [Best Practices](docs/guides/mutation-testing-best-practices.md) - Team adoption strategies
- [CI/CD Integration](docs/ci-cd/) - GitHub Actions, GitLab CI, Jenkins

**Example Projects:**
- [Rust Example](examples/rust-mutation-testing/) - 8 functions, 8 tests, ~90% mutation score
- [Python Example](examples/python-mutation-testing/) - 8 functions, 24 tests, comprehensive coverage
- [TypeScript Example](examples/typescript-mutation-testing/) - 8 functions, 24 tests, Jest integration

---

## TDG Enforcement System (v2.180.0+)

**Zero-regression quality enforcement** across local development, git workflows, and CI/CD pipelines.

### Quick Start

```bash
# 1. Create quality baseline
pmat tdg baseline create --output .pmat/tdg-baseline.json --path .

# 2. Install git hooks (optional)
pmat hooks install --tdg-enforcement

# 3. Check for regressions
pmat tdg check-regression \
  --baseline .pmat/tdg-baseline.json \
  --path . \
  --max-score-drop 5.0 \
  --fail-on-regression

# 4. Enforce quality standards for new code
pmat tdg check-quality \
  --path . \
  --min-grade B+ \
  --new-files-only \
  --fail-on-violation
```

### Features

**Baseline System**:
- Blake3 content-hash based deduplication
- Project-wide quality snapshots
- Delta detection (improved, regressed, unchanged files)

**Quality Gates**:
- Regression detection (prevents quality degradation)
- Minimum grade enforcement (ensures new code quality)
- Language-specific thresholds

**Git Hooks**:
- Pre-commit quality checks
- Post-commit baseline updates
- Configurable enforcement modes (strict, warning, disabled)

**CI/CD Templates**:
- GitHub Actions workflow
- GitLab CI pipeline
- Jenkins declarative pipeline

### Configuration

Create `.pmat/tdg-rules.toml`:

```toml
[quality_gates]
rust_min_grade = "A"
python_min_grade = "B+"
max_score_drop = 5.0
mode = "strict"  # strict, warning, or disabled

[baseline]
baseline_path = ".pmat/tdg-baseline.json"
auto_update_on_main = true
```

### CI/CD Integration

**GitHub Actions**:
```bash
cp templates/ci/github-actions-tdg.yml .github/workflows/tdg-quality.yml
```

**GitLab CI**:
```bash
cp templates/ci/gitlab-ci-tdg.yml .gitlab-ci.yml
```

**Jenkins**:
```bash
cp templates/ci/Jenkinsfile-tdg Jenkinsfile
```

**See Complete Guide**: `docs/guides/ci-cd-tdg-integration.md`

---

## Rust Project Score v2.0 (NEW)

**Evidence-based Rust project quality scoring** (0-211 points) inspired by elite projects (tokio, serde, clap, syn, regex) with academic foundation from 15+ peer-reviewed papers.

### Quick Start

```bash
# Fast mode analysis (~3 minutes)
pmat rust-project-score

# Full mode with comprehensive checks (~10-15 minutes)
pmat rust-project-score --full

# Specific project path with JSON output
pmat rust-project-score --path /path/to/rust/project --format json

# Markdown report for documentation
pmat rust-project-score --verbose --format markdown --output SCORE.md
```

### Scoring Categories (211 points total)

1. **Rust Tooling & CI/CD** (130pts)
   - Clippy (tiered: correctness > suspicious > pedantic): 10pts
   - Rustfmt compliance: 5pts
   - cargo-audit (risk-based scoring): 7pts
   - cargo-deny policy enforcement: 3pts
   - **Workspace-level lints**: 12pts
   - **CI/CD Integration** (multi-platform, workflows, build automation): 37pts
   - **Advanced Metadata** (docs.rs, workspace, release automation): 35pts
   - **MSRV Tracking**: 10pts
   - **Release Profile Optimization**: 11pts

2. **Code Quality** (26pts)
   - Cyclomatic Complexity (≤20): 3pts
   - Unsafe Code Documentation: 9pts
   - Mutation Testing (≥80%): 8pts
   - Build Time (<5min): 4pts
   - Dead Code Detection: 2pts

3. **Testing Excellence** (20pts)
   - Coverage (≥85%): 8pts
   - Integration Tests: 4pts
   - Doc Tests: 3pts
   - Mutation Coverage: 5pts

4. **Documentation** (15pts)
   - Rustdoc Coverage: 7pts
   - README Quality: 5pts
   - Changelog Maintenance: 3pts

5. **Performance & Benchmarking** (10pts)
   - Criterion Benchmarks: 5pts
   - CI Benchmark Workflows: 3pts
   - Custom Harness: 2pts

6. **Dependency Health** (12pts)
   - Dependency Count: 5pts
   - Feature Flags: 4pts
   - Tree Pruning: 3pts

7. **Formal Verification** (8pts - bonus category)

### Example Output

```
🦀  Rust Project Score v2.0

📌  Summary
  Score: 100.5/211
  Percentage: 47.6%
  Grade: B

📂  Categories
  ✅ Rust Tooling & CI/CD: 56.0/130 (43.1%)
  ⚠️  Code Quality: 20.0/26 (76.9%)
  ❌ Testing Excellence: 5.5/20 (27.5%)
  ...

💡  Recommendations
  • Run 'cargo clippy --fix' to automatically fix warnings
  • Add workspace-level lints to Cargo.toml
  • Create .github/workflows for multi-platform CI
  • Configure docs.rs metadata for better documentation
  ...
```

### Academic Foundation

**15+ Peer-Reviewed References** (IEEE, ACM, arXiv 2013-2025):
- **Complexity Weight Reduced** (8→3pts): No bug correlation (arXiv 2024)
- **Unsafe Code Emphasis** (6→9pts): Memory safety critical in Rust
- **Mutation Testing** (5→8pts): Test quality validation (ICST 2024)
- **CI/CD Integration**: Faster releases (Hilton 2016 ASE)
- **Build Time Impact**: Developer productivity (Beller 2017 MSR)
- **Workspace Benefits**: Fewer conflicts (ICSE 2024)

### CI/CD Integration

```yaml
# .github/workflows/rust-quality.yml
name: Rust Quality Score
on: [push, pull_request]

jobs:
  rust-score:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo install pmat
      - run: pmat rust-project-score --format json --output score.json
      - name: Check minimum score
        run: |
          SCORE=$(jq '.total_earned' score.json)
          if (( $(echo "$SCORE < 80" | bc -l) )); then
            echo "Score $SCORE below threshold"
            exit 1
          fi
```

### Fast vs Full Mode

**Fast Mode** (default, ~3 minutes):
- Skips: clippy, mutation testing, build time measurement
- Provides: Moderate credit for skipped checks
- Use case: Quick CI checks, development feedback

**Full Mode** (--full, ~10-15 minutes):
- Runs: All checks comprehensively
- Provides: Evidence-based, peer-reviewed scoring
- Use case: Release validation, comprehensive audits

### Documentation

- **Specification**: `docs/specifications/learn-from-rust-giants-spec.md`
- **Implementation Status**: `docs/implementation-status-rust-project-score.md`
- **CLAUDE.md**: Comprehensive usage guide with 200+ lines

---

## Git-Commit Correlation (v2.179.0+)

Track Technical Debt Grading (TDG) scores at specific git commits for "quality archaeology" workflows.

```bash
# Analyze with git context
pmat tdg server/src/lib.rs --with-git-context

# Query specific commit
pmat tdg history --commit v2.178.0

# History since reference
pmat tdg history --since HEAD~10

# Commit range
pmat tdg history --range v2.177.0..v2.178.0

# Filter by file
pmat tdg history --path server/src/lib.rs --since HEAD~5

# JSON output for scripting
pmat tdg history --commit 60125a0 --format json
```

**Use Cases:**
- **Quality Archaeology**: Find which commit broke quality
- **Release Tracking**: Compare quality between releases
- **Regression Detection**: Identify quality drops over time
- **Developer Metrics**: Track quality attribution

**Example Workflows:**

```bash
# Find when quality dropped below B+
pmat tdg history --since HEAD~50 --format json | \
  jq '.history[] | select(.score.grade | test("C|D|F"))'

# Quality delta between releases
pmat tdg history --range v2.177.0..v2.178.0

# Per-file quality trend
pmat tdg history --path src/lib.rs --since HEAD~20
```

**Features:**
- Tag resolution support (e.g., `--commit v2.178.0`)
- MCP integration (`with_git_context: true` parameter)
- Zero performance overhead (<1% analysis time)
- Backward compatible (git context is optional)

---

## MCP Integration

PMAT provides 19 MCP tools for AI agents:

```bash
# Start MCP server
pmat mcp

# Use with Claude Code, Cline, or other MCP clients
```

**Tools include:** context generation, complexity analysis, TDG scoring, semantic search, code clustering, documentation validation, and more.

See [MCP Tools Documentation](docs/mcp/TOOLS.md) for details.

---

## Project Info

**Built by:** [Pragmatic AI Labs](https://paiml.com)  
**License:** MIT  
**Repository:** [github.com/paiml/paiml-mcp-agent-toolkit](https://github.com/paiml/paiml-mcp-agent-toolkit)  
**Issues:** [GitHub Issues](https://github.com/paiml/paiml-mcp-agent-toolkit/issues)

**Current Version:** v2.167.0 (Sprint 44 - Coverage Remediation)

---

## Contributing

See [ROADMAP.md](ROADMAP.md) for project status and future plans.

**Quality Standards:**
- EXTREME TDD (RED → GREEN → REFACTOR)
- 85%+ code coverage
- Five Whys root cause analysis
- Toyota Way principles (Jidoka, Genchi Genbutsu, Kaizen)
- Zero tolerance for defects