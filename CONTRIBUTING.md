# Contributing to PMAT (PAIML MCP Agent Toolkit)

Thank you for your interest in contributing to the PAIML MCP Agent Toolkit! This project follows strict quality standards based on Toyota Way principles.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Quality Standards](#quality-standards)
- [Submitting Changes](#submitting-changes)
- [Testing](#testing)
- [Documentation](#documentation)
- [Release Process](#release-process)

## Code of Conduct

This project adheres to professional standards of respect and collaboration. We expect all contributors to:

- Be respectful and inclusive
- Focus on constructive feedback
- Help maintain our Zero Tolerance Quality Standards
- Follow the Toyota Way principles

## Getting Started

### Prerequisites

- Rust 1.80.0 or later
- Git
- Make (for build automation)
- Cargo (comes with Rust)

### Initial Setup

```bash
# Fork and clone the repository
git clone https://github.com/YOUR_USERNAME/paiml-mcp-agent-toolkit
cd paiml-mcp-agent-toolkit

# Add upstream remote
git remote add upstream https://github.com/paiml/paiml-mcp-agent-toolkit

# Install development dependencies
make install-deps

# Run initial build
cargo build

# Run tests to verify setup
make test-fast
```

## Development Setup

### Project Structure

The project is organized as follows:

```
paiml-mcp-agent-toolkit/
├── server/              # Main Rust application
│   ├── src/             # Source code
│   │   ├── analysis/    # Core analysis modules
│   │   ├── cli/         # Command-line interface
│   │   ├── model/       # Data models and types
│   │   ├── protocol/    # MCP and HTTP protocols
│   │   └── services/    # Application services
│   ├── tests/           # Integration and E2E tests
│   └── Cargo.toml       # Project dependencies
├── docs/                # Project documentation
├── scripts/             # Utility and automation scripts
└── Makefile             # Build automation entrypoint
```

### Development Tools

```bash
# Install recommended tools
cargo install cargo-watch    # Auto-rebuild on changes
cargo install cargo-edit     # Manage dependencies
cargo install cargo-outdated # Check for updates
```

## Quality Standards

### Zero Tolerance Policy

This project maintains Zero Tolerance Quality Standards:

1. **ZERO SATD**: No TODO, FIXME, HACK, or placeholder implementations
2. **ZERO High Complexity**: No function exceeds cyclomatic complexity of 20
3. **ZERO Known Defects**: All code must be fully functional
4. **ZERO Incomplete Features**: Only complete, tested features are merged

### Before Committing

Always run the following checks locally before submitting a pull request:

1.  **Run Extreme Quality Lints:**
    ```bash
    make lint
    ```

2.  **Run Fast Tests:**
    ```bash
    make test-fast
    ```

3.  **Run the Quality Gate:**
    ```bash
    pmat quality-gate --strict
    ```

These checks help ensure that your contributions meet our Zero Tolerance Quality Standards.

### Code Style

- Follow Rust idioms and best practices
- Use `rustfmt` for formatting (automatically run by `make lint`)
- Write clear, self-documenting code
- Add documentation comments for public APIs
- Keep functions small and focused (< 20 lines preferred)

## Submitting Changes

### Workflow

1. **Create a feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes**
   - Write tests for new functionality
   - Update documentation as needed
   - Follow quality standards

3. **Run quality checks**
   ```bash
   make lint
   make test-fast
   pmat quality-gate --strict
   ```

4. **Commit with clear messages**
   ```bash
   git commit -m "feat: Add new analysis feature for X

   - Implement algorithm Y
   - Add comprehensive tests
   - Update documentation"
   ```

5. **Push and create PR**
   ```bash
   git push origin feature/your-feature-name
   ```

### Pull Request Guidelines

- **Title**: Use conventional commit format (feat:, fix:, docs:, etc.)
- **Description**: Clearly explain what and why
- **Tests**: Include tests for all new code
- **Documentation**: Update relevant docs
- **Quality**: Ensure all quality gates pass

### PR Template

```markdown
## Summary
Brief description of changes

## Motivation
Why these changes are needed

## Changes
- Change 1
- Change 2

## Testing
How the changes were tested

## Checklist
- [ ] Tests pass (`make test-fast`)
- [ ] Lints pass (`make lint`)
- [ ] Quality gate passes (`pmat quality-gate --strict`)
- [ ] Documentation updated
- [ ] No SATD introduced
- [ ] Complexity limits maintained
```

## Testing

### Test Categories

```bash
# Unit tests (< 10s)
make test-unit

# Service tests (< 30s)
make test-services

# Protocol tests (< 45s)
make test-protocols

# End-to-end tests (< 120s)
make test-e2e

# All tests
make test-all

# With coverage
make coverage
```

### Writing Tests

- Use descriptive test names
- Test both success and failure cases
- Keep tests focused and independent
- Use property-based testing where appropriate
- Aim for > 90% coverage

Example:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_handles_empty_input() {
        let result = my_function("");
        assert!(result.is_err());
    }

    #[test]
    fn test_function_processes_valid_input() {
        let result = my_function("valid");
        assert_eq!(result.unwrap(), expected_value);
    }
}
```

## Documentation

### Types of Documentation

1. **Code Documentation**
   - Doc comments on public APIs
   - Examples in doc comments
   - Module-level documentation

2. **Feature Documentation**
   - New features need docs in `docs/features/`
   - Include examples and use cases

3. **API Documentation**
   - Keep `docs/api/` updated
   - Document all public endpoints

### Documentation Standards

- Write clear, concise explanations
- Include code examples
- Keep documentation up-to-date with code
- Use proper markdown formatting

## Release Process

### Version Numbering

We follow semantic versioning (MAJOR.MINOR.PATCH):
- MAJOR: Breaking changes
- MINOR: New features, backward compatible
- PATCH: Bug fixes

### Release Checklist

1. Update version in `Cargo.toml`
2. Update `RELEASE_NOTES.md`
3. Run full test suite
4. Build and test release binary
5. Create GitHub release
6. Publish to crates.io

See [Release Process](docs/release-process.md) for detailed steps.

## Questions?

- Check existing issues and discussions
- Join our community discussions
- Contact maintainers

Thank you for contributing to PMAT!