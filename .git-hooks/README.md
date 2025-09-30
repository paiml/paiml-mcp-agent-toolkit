# Git Hooks for Claude Integration

Quality enforcement hooks for the Claude Agent SDK integration.

## Installation

To install the pre-commit hook:

```bash
# Copy the hook to your .git/hooks directory
cp .git-hooks/pre-commit-claude-integration.sh .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

Or use git config to set up the hooks directory:

```bash
git config core.hooksPath .git-hooks
```

## Pre-Commit Hook

The `pre-commit-claude-integration.sh` hook enforces EXTREME TDD quality gates:

### Quality Checks

1. **SATD Detection** - Zero tolerance for TODO/FIXME/HACK/XXX
2. **Rust Formatting** - Enforces `cargo fmt` standards
3. **Clippy Lints** - All warnings treated as errors
4. **Complexity** - Ensures code compiles (proxy for complexity)
5. **TypeScript Build** - Verifies bridge compilation
6. **Unit Tests** - All tests must pass
7. **File Size** - Warns about files >500 lines

### Bypassing Hooks (Emergency Only)

If you need to bypass hooks in an emergency:

```bash
git commit --no-verify
```

**Warning**: This should only be used in exceptional circumstances. Bypassing quality gates may introduce technical debt.

### Troubleshooting

#### Hook not running
```bash
# Check if hook is executable
ls -l .git/hooks/pre-commit

# Make it executable
chmod +x .git/hooks/pre-commit
```

#### TypeScript checks failing
```bash
# Install dependencies
cd bridge
npm install
npm run build
```

#### Rust checks failing
```bash
# Fix formatting
cd server
cargo fmt

# Fix clippy warnings
cargo clippy --fix
```

## Quality Metrics

The hooks enforce these standards:

| Metric | Threshold | Action |
|--------|-----------|--------|
| SATD Count | 0 | Block commit |
| Clippy Warnings | 0 | Block commit |
| Formatting | 100% | Block commit |
| Test Pass Rate | 100% | Block commit |
| File Size | 500 lines | Warning only |

## Philosophy

These hooks implement the EXTREME TDD methodology:

- **Red**: Tests must fail first
- **Green**: Minimal implementation
- **Refactor**: Quality gates prevent degradation

Quality is enforced at commit time, not PR time, ensuring:
- Faster feedback loops
- Cleaner git history
- Reduced review burden
- Higher code quality