# Contributing to PMAT (Polyglot Mutation Analysis Tool)

Thank you for your interest in contributing to PMAT! This document provides guidelines for contributing to the project.

## Development Setup

### Prerequisites

- Rust 1.70+ (stable)
- Cargo
- Git

### Building from Source

```bash
# Clone the repository
git clone https://github.com/paiml/paiml-mcp-agent-toolkit.git
cd paiml-mcp-agent-toolkit

# Build the project
cargo build --release

# Run tests
cargo test
```

## Quality Standards

PMAT follows **Toyota Way principles** and maintains strict quality standards:

- **Zero Tolerance for Defects**: All code must pass quality gates before merging
- **85%+ Test Coverage**: Measured via `cargo llvm-cov`
- **Mutation Testing**: Critical features must have mutation tests
- **Property-Based Testing**: Complex logic requires property tests

### Quality Gates

Before submitting a PR, ensure all quality gates pass:

```bash
# Run all quality checks
make validate

# Individual checks
make lint          # Clippy linting
make test          # Unit tests
make coverage      # Test coverage report
make validate-book # Book validation (critical)
```

### Pre-commit Hooks

We use pre-commit hooks to enforce quality:

```bash
# Install hooks (automatic on first commit)
git commit  # Hooks auto-install

# Manual hook installation
cp .git/hooks/pre-commit.sample .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

Hooks validate:
- **bashrs linting**: All bash scripts and Makefiles
- **pmat-book validation**: Documentation accuracy (critical)
- **Documentation accuracy**: Zero hallucinations in docs

## Git Workflow

We use a **single-branch workflow** (master only):

```bash
# Always work on master
git checkout master

# Make changes
# ... edit files ...

# Commit with descriptive message
git commit -m "feat: Add new feature"

# Push directly to master (after quality gates pass)
git push origin master
```

## pmat-book Synchronization

**CRITICAL**: Documentation must stay in sync with code.

### Book Validation (Mandatory)

```bash
# Fast, parallel validation (recommended)
make validate-book

# This runs critical book tests:
# - Chapter 5: Core functionality
# - Chapter 7: Advanced features
# - Chapter 13: Multi-language support (CRITICAL)
# - Chapter 14: Integration
```

### Book Push Enforcement

Before pushing code changes:

1. **Update pmat-book** if documentation needs changes
2. **Push pmat-book first**: `cd ../pmat-book && git push origin main`
3. **Then push code**: `git push origin master`

A **pre-push hook** enforces this synchronization.

## Commit Message Format

Follow conventional commit format:

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `refactor`: Code refactoring
- `test`: Test additions/changes
- `chore`: Maintenance tasks

**Examples:**

```
feat(cli): Add mutation testing for TypeScript

Implements mutant generation and analysis for TypeScript files
using tree-sitter-typescript parser.

Closes #123
```

```
fix(parser): Handle empty Rust files correctly

Fixes crash when parsing empty .rs files by adding null check
in RustVisitor::visit_source_file.

Fixes #456
```

## Pull Request Guidelines

**Note**: We don't use branches or PRs for internal development. External contributors should:

1. Fork the repository
2. Create a feature branch
3. Make changes and commit
4. Ensure all quality gates pass
5. Submit PR with descriptive title and description

## Code Style

### Rust Code

- Follow `rustfmt` formatting (automatic via `cargo fmt`)
- Use `clippy` for linting (zero warnings policy)
- Document public APIs with `///` doc comments
- Add unit tests for new functionality

### Bash Scripts

- Use `bashrs` for linting (enforced via pre-commit hook)
- Quote all variable expansions: `"${VAR}"` not `$VAR`
- Check exit codes: `command || handle_error`

## Testing

### Test Categories

- **Unit tests**: `#[cfg(test)]` modules in source files
- **Integration tests**: `tests/` directory
- **Property tests**: `#[cfg(feature = "proptest")]` tests
- **Mutation tests**: Critical logic must have mutation tests

### Running Tests

```bash
# All tests
cargo test

# Specific test
cargo test test_name

# Property tests (requires feature)
cargo test --features proptest

# Mutation tests
cargo mutants --test-tool=nextest
```

### Test Coverage

```bash
# Generate coverage report
cargo llvm-cov --html

# View report
open target/llvm-cov/html/index.html
```

## Documentation

### Code Documentation

- Public APIs require doc comments
- Include examples in doc comments
- Document panics, errors, and safety invariants

```rust
/// Analyzes Rust code for mutation testing opportunities.
///
/// # Arguments
///
/// * `file_path` - Path to the Rust source file
/// * `source` - Source code content
///
/// # Returns
///
/// Returns `Vec<Mutant>` containing all identified mutants.
///
/// # Errors
///
/// Returns `Err` if parsing fails or file is invalid.
///
/// # Examples
///
/// ```
/// let mutants = analyze_rust_file("src/main.rs", source)?;
/// println!("Found {} mutants", mutants.len());
/// ```
pub fn analyze_rust_file(file_path: &str, source: &str) -> Result<Vec<Mutant>> {
    // ...
}
```

### pmat-book Updates

When adding new features, update the corresponding book chapter:

```bash
cd ../pmat-book

# Edit relevant chapter
vim src/SUMMARY.md
vim src/chapter_XX/feature.md

# Test locally
mdbook serve

# Commit and push (BEFORE pushing code)
git add .
git commit -m "docs: Add feature documentation"
git push origin main
```

## Issue Reporting

### Bug Reports

Include:
- PMAT version: `pmat --version`
- Operating system
- Rust version: `rustc --version`
- Steps to reproduce
- Expected vs actual behavior
- Error messages (full output)

### Feature Requests

Include:
- Use case description
- Proposed solution
- Alternatives considered
- Impact on existing functionality

## Performance Considerations

- Profile changes with `cargo-flamegraph`
- Avoid allocations in hot paths
- Use `cargo-bloat` to check binary size
- Benchmark performance-critical code

## Security

- Never commit secrets or credentials
- Use `cargo-audit` to check dependencies
- Report security issues privately via email

## License

By contributing, you agree that your contributions will be licensed under the same license as the project.

## Getting Help

- Read the [pmat-book](https://paiml.github.io/pmat-book/)
- Check existing issues
- Ask questions in GitHub issues

## Recognition

Contributors are recognized in:
- CHANGELOG.md
- Release notes
- Project documentation

Thank you for contributing to PMAT!
