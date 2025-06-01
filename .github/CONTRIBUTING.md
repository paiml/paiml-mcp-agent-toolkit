# Contributing to PAIML MCP Agent Toolkit

Thank you for your interest in contributing to the PAIML MCP Agent Toolkit! This document provides guidelines and best practices for contributing to the project.

## 🏗️ Project Structure

This is a **Cargo workspace** project with a centralized build system. Understanding this structure is critical for successful contributions.

```
paiml-mcp-agent-toolkit/
├── Makefile                 # ⚠️ PRIMARY control point - use this!
├── Cargo.toml              # Workspace definition
├── server/                 # Server project (workspace member)
│   ├── Makefile           # Project-specific targets only
│   └── Cargo.toml         # Server package
├── installer-macro/        # Procedural macro crate
│   └── Cargo.toml         # Macro package
└── .github/               # GitHub Actions workflows
    └── workflows/         # CI/CD pipelines
```

## ⚠️ Critical: Makefile Usage

### The Golden Rule

**Use the root Makefile for 80% of all operations!**

The root Makefile is designed to orchestrate the entire project and prevents workspace-related build issues.

### DO ✅

```bash
# From the root directory:
make server-test              # Run server tests
make server-build             # Build server
make server-lint              # Lint server code
make validate                 # Run all checks
make install                  # Install the binary
```

### DON'T ❌

```bash
# Avoid these patterns:
cd server && make test       # Wrong: Can cause workspace issues
cd server && cargo build     # Wrong: May not resolve dependencies correctly
```

### When to use project-specific Makefiles

Only use `cd server && make ...` when:
- You're actively developing within that specific directory
- You need a project-specific target not exposed at the root
- You're debugging an issue specific to that project

## 🔄 CI/CD Guidelines

### GitHub Actions Workflows

All workflows MUST follow these rules:

1. **Always run from repository root**
   ```yaml
   - name: Run tests
     run: make server-test  # ✅ Correct
   ```

2. **Never change directories in CI**
   ```yaml
   - name: Run tests
     run: |
       cd server         # ❌ Wrong
       make test
   ```

3. **Use --manifest-path for direct cargo commands**
   ```yaml
   - name: Build specific target
     run: cargo build --manifest-path server/Cargo.toml  # ✅ Correct
   ```

### Adding New Workflows

When creating new GitHub Actions workflows:

1. Add a comment at the top explaining the workspace structure:
   ```yaml
   # This workflow uses the root Makefile to orchestrate builds
   # DO NOT use 'cd server' - use 'make server-*' targets instead
   ```

2. Use the established patterns from existing workflows

3. Test locally with `act` before pushing:
   ```bash
   make test-actions
   ```

## 🧪 Testing Guidelines

### Running Tests

```bash
# Run all tests across the workspace
make test

# Fast tests (optimized for CI/quick feedback)
make test-fast

# Run specific project tests
make server-test

# Run with coverage
make coverage

# Run benchmarks
make server-bench
```

### Writing Tests

1. Place unit tests in the same file as the code
2. Place integration tests in `tests/` directory
3. Use descriptive test names that explain what's being tested
4. Aim for >60% code coverage (current: ~67%)

## 🎨 Code Style

### Formatting

Always format code before committing:

```bash
# Format everything
make format

# Or use the alias
make fix
```

### Linting

Ensure code passes all lints:

```bash
# Run all linters
make lint

# Check specific issues
make server-lint
```

### Type Checking

```bash
# Type check all projects
make check
```

## 📝 Documentation

### Where to Document

1. **README.md** - User-facing documentation
2. **CLAUDE.md** - AI assistant guidelines
3. **CONTRIBUTING.md** - This file, for contributors
4. **Code comments** - Implementation details
5. **Commit messages** - Change rationale

### Documentation Standards

- Use clear, concise language
- Include examples where helpful
- Keep documentation up-to-date with code changes
- Document workspace structure implications

## 🚀 Submitting Changes

### 1. Fork and Clone

```bash
git clone https://github.com/YOUR_USERNAME/paiml-mcp-agent-toolkit.git
cd paiml-mcp-agent-toolkit
```

### 2. Create a Feature Branch

```bash
git checkout -b feature/your-feature-name
```

### 3. Make Changes

- Follow the code style guidelines
- Write tests for new functionality
- Update documentation as needed

### 4. Validate Your Changes

```bash
# Run the full validation suite
make validate

# This runs:
# - Type checking (cargo check)
# - Linting (cargo clippy + deno lint)
# - Testing (cargo test)
# - Documentation validation
# - Naming convention checks
```

### 5. Commit Your Changes

Use conventional commit format:

```bash
git commit -m "feat: add new template parameter validation"
git commit -m "fix: correct workspace path resolution"
git commit -m "docs: update workspace structure documentation"
```

### 6. Push and Create PR

```bash
git push origin feature/your-feature-name
```

Then create a Pull Request on GitHub.

## 🐛 Reporting Issues

### Before Reporting

1. Check existing issues
2. Verify you're using the latest version
3. Try to reproduce with minimal example

### Issue Template

Include:
- Clear description of the problem
- Steps to reproduce
- Expected vs actual behavior
- System information (OS, Rust version)
- Relevant logs or error messages

## 🔧 Development Setup

### Prerequisites

- Rust (stable) - MSRV 1.80.0
- Deno 2.x
- Git

### Quick Setup

```bash
# Clone the repository
git clone https://github.com/paiml/paiml-mcp-agent-toolkit.git
cd paiml-mcp-agent-toolkit

# Setup development environment
make setup

# Run quick validation
make validate

# Start developing!
make quickstart
```

## 📦 Dependency Management

### Adding Dependencies

1. Add to the appropriate `Cargo.toml`
2. Run `make server-deps-update` to update lock file
3. Verify with `make server-deps-check`
4. Run tests to ensure compatibility

### Security

- Run `make audit` regularly
- Address security advisories promptly
- Keep dependencies up-to-date

## 💡 Tips for Success

1. **Always validate before pushing**: `make validate`
2. **Use root Makefile targets**: Prevents workspace issues
3. **Read CI logs carefully**: They often show the exact issue
4. **Ask questions**: Open an issue if you're stuck
5. **Small PRs are better**: Easier to review and merge

## 🙏 Thank You!

Your contributions help make PAIML MCP Agent Toolkit better for everyone. We appreciate your time and effort!

---

**Remember**: When in doubt, use the root Makefile! 🎯