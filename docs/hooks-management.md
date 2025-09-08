# PMAT Pre-commit Hooks Management

**Version**: 2.66.0  
**Status**: Production Ready  
**Priority**: Core Feature  

## 🎯 Overview

PMAT's pre-commit hook management system provides **single source of truth** configuration for quality gate enforcement. This core feature eliminates configuration duplication and ensures consistent quality standards across all development workflows.

## 🚀 Quick Start

```bash
# Install PMAT-managed hooks
pmat tdg hooks install --backup

# Check installation status
pmat tdg hooks status

# View current configuration
pmat tdg config show --format json
```

## 📋 Features

### Single Source of Truth

All quality gate thresholds are defined in `pmat.toml`:

```toml
[quality]
max_complexity = 20           # Cyclomatic complexity limit
max_cognitive_complexity = 15 # Cognitive complexity limit  
min_coverage = 80.0          # Minimum test coverage
allow_satd = false           # Zero SATD tolerance
```

### Dynamic Hook Generation

Hooks are generated dynamically from your configuration:
- **No hardcoded values** - All thresholds from pmat.toml
- **Auto-updates** - Hooks refresh when configuration changes
- **Template-based** - Consistent, maintainable hook scripts

### Quality Gate Enforcement

Pre-commit hooks enforce:
- ✅ **Complexity Analysis** - Max cyclomatic/cognitive complexity
- ✅ **SATD Detection** - Zero tolerance for technical debt comments
- ✅ **Test Coverage** - Minimum 80% coverage enforcement
- ✅ **Documentation Sync** - Ensures docs are updated with code
- ✅ **Task ID Validation** - Commit messages match PMAT-XXXX pattern

## 🔧 Commands

### Hook Management

```bash
# Install hooks (creates backup of existing)
pmat tdg hooks install --backup

# Force install (overwrite without prompt)
pmat tdg hooks install --force --backup

# Uninstall hooks
pmat tdg hooks uninstall

# Restore original hooks
pmat tdg hooks uninstall --restore-backup

# Check hook status
pmat tdg hooks status

# Verify and fix issues
pmat tdg hooks verify --fix

# Refresh from configuration
pmat tdg hooks refresh
```

### Configuration Access

```bash
# Show complete configuration
pmat tdg config show --format json
pmat tdg config show --format toml
pmat tdg config show --format env

# Get specific value
pmat tdg config get hooks.quality_gates.max_cyclomatic_complexity

# Validate configuration
pmat tdg config validate

# Show configuration sources
pmat tdg config sources
```

## 📝 Configuration Schema

Add to your `pmat.toml`:

```toml
[hooks]
enabled = true                    # Enable hook management
auto_install = true              # Auto-install on config changes
backup_existing = true           # Backup existing hooks

[hooks.quality_gates]
max_cyclomatic_complexity = 30  # Industry standard
max_cognitive_complexity = 25   # Maintainable threshold
max_satd_comments = 5           # Enterprise allowance
min_test_coverage = 80.0        # High standard
max_clippy_warnings = 100       # Reasonable limit

[hooks.documentation]
required_files = [
    "docs/execution/roadmap.md",
    "CHANGELOG.md"
]
task_id_pattern = "PMAT-[0-9]{4}"
```

## 🔍 Hook Script Details

The generated pre-commit hook:

1. **Loads configuration dynamically**:
   ```bash
   export PMAT_MAX_CYCLOMATIC_COMPLEXITY=20
   export PMAT_MAX_COGNITIVE_COMPLEXITY=15
   export PMAT_MIN_TEST_COVERAGE=80
   ```

2. **Runs quality checks**:
   - Complexity analysis via `pmat analyze complexity`
   - SATD detection via `pmat analyze satd`
   - Documentation synchronization verification
   - Task ID pattern matching

3. **Provides actionable feedback**:
   ```
   ❌ Complexity violation (threshold: 20)
   🔧 Fix: pmat refactor auto --file <file>
   ```

## 🎯 Toyota Way Principles

This implementation follows Toyota Way principles:

### Kaizen (改善) - Continuous Improvement
- Configuration changes automatically update hooks
- Single source of truth eliminates duplication
- Incremental quality gate enforcement

### Jidoka (自働化) - Automation with Human Touch
- Automated quality checks at commit time
- Human-readable error messages
- Suggested fixes for violations

### Genchi Genbutsu (現地現物) - Go and See
- Direct integration with development workflow
- Real-time quality feedback
- Actual metrics, not estimates

## 🔒 Security & Recovery

### Backup System
- Automatic backup before installation
- Timestamped backup files
- One-command restore capability

### Graceful Degradation
- Warns if pmat binary unavailable
- Falls back to basic checks
- Never blocks emergency commits

### Manual Override
```bash
# Emergency bypass (logged for audit)
git commit --no-verify -m "EMERGENCY: <reason>"
```

## 📊 Integration Examples

### CI/CD Pipeline

```yaml
# GitHub Actions example
- name: Verify hooks installed
  run: pmat tdg hooks verify
  
- name: Enforce quality gates
  run: pmat tdg config get hooks.quality_gates.max_cyclomatic_complexity
```

### Docker Integration

```dockerfile
# Install hooks in container
RUN cargo install pmat && \
    pmat tdg hooks install --force
```

### Team Onboarding

```bash
# Setup script for new developers
#!/bin/bash
cargo install pmat
pmat tdg hooks install --backup
pmat tdg hooks verify
echo "✅ Quality gates configured!"
```

## 🚦 Status Indicators

When running `pmat tdg hooks status`:

- ✅ **Installed** - Hook file exists
- ✅ **PMAT-managed** - Hook generated by PMAT
- ✅ **Config up-to-date** - Matches current configuration
- 📅 **Last updated** - Timestamp of last modification

## 🐛 Troubleshooting

### Hook Not Working

```bash
# Verify installation
pmat tdg hooks verify --fix

# Check permissions (Unix)
ls -la .git/hooks/pre-commit

# Reinstall if needed
pmat tdg hooks install --force --backup
```

### Configuration Issues

```bash
# Validate configuration
pmat tdg config validate

# Show current values
pmat tdg config show --format env

# Test with explicit values
PMAT_MAX_CYCLOMATIC_COMPLEXITY=25 git commit
```

### Recovery

```bash
# Restore original hooks
pmat tdg hooks uninstall --restore-backup

# Remove all PMAT hooks
rm .git/hooks/pre-commit
rm .git/hooks/pre-commit.pmat-backup
```

## 📈 Benefits

### For Developers
- ⚡ **Instant feedback** - Quality issues caught before commit
- 🔧 **Actionable fixes** - Clear remediation steps
- 📊 **Consistent standards** - Same rules for everyone

### For Teams
- 🎯 **Single configuration** - One pmat.toml for all settings
- 🔄 **Auto-synchronization** - Config changes propagate automatically
- 📝 **Documentation enforcement** - Never forget to update docs

### For Organizations
- 🏭 **Quality at source** - Issues prevented, not discovered later
- 📊 **Measurable standards** - Concrete thresholds, not guidelines
- 🔒 **Audit trail** - All quality decisions tracked

## 🎓 Best Practices

1. **Start with moderate thresholds**, tighten gradually:
   ```toml
   max_complexity = 30         # Start here
   max_complexity = 25         # After 1 sprint
   max_complexity = 20         # Target state
   ```

2. **Use verify before committing**:
   ```bash
   pmat tdg hooks verify && git commit
   ```

3. **Keep configuration in version control**:
   ```bash
   git add pmat.toml
   git commit -m "PMAT-0080: Update quality thresholds"
   ```

4. **Document exceptions**:
   ```toml
   # Temporary increase for legacy refactoring
   max_complexity = 35  # TODO: Reduce to 20 after PMAT-0081
   ```

## 🔗 Related Documentation

- [Pre-commit Hooks Specification](specifications/pre-commit-hooks-spec.md)
- [Quality Gates Documentation](execution/quality-gates.md)
- [Configuration Service](../server/src/services/configuration_service.rs)
- [Toyota Way Principles](execution/toyota-way.md)

## 📚 Further Reading

- [Git Hooks Documentation](https://git-scm.com/book/en/v2/Customizing-Git-Git-Hooks)
- [Pre-commit Framework](https://pre-commit.com/)
- [Toyota Way in Software](https://www.lean.org/lexicon/toyota-way)

---

**Sprint 80 Achievement**: Pre-commit hook management is now a core PMAT feature, eliminating configuration duplication and ensuring quality gates are enforced consistently from a single source of truth.