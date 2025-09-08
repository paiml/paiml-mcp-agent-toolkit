# Pre-commit Hooks Specification

**Version**: 1.0  
**Status**: Draft  
**Priority**: P0 - Critical Quality Infrastructure  

## Overview

This specification defines PMAT's built-in pre-commit hook management system that enforces quality gates as a **core language feature**, eliminating configuration duplication and ensuring single-source-of-truth quality enforcement.

## Problem Statement

Currently, quality gate enforcement requires manual configuration in multiple locations:
- `pmat.toml` (project configuration)
- `server/pmat.toml` (server-specific config)
- `server/src/tdg/config.rs` (hardcoded defaults)
- `scripts/setup-quality.sh` (setup templates)
- `.git/hooks/pre-commit` (hook implementation)

This violates the DRY principle and creates maintenance overhead when updating quality standards.

## Solution Design

### Core Principle
**Quality gate enforcement is a fundamental language feature**, not an external add-on. PMAT should manage its own quality enforcement infrastructure automatically.

### Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────────┐
│   pmat.toml     │───▶│  PMAT Core       │───▶│  Quality Gates      │
│ (Single Source) │    │  Config Engine   │    │  Enforcement        │
└─────────────────┘    └──────────────────┘    └─────────────────────┘
                              │                          │
                              ▼                          ▼
                       ┌──────────────────┐    ┌─────────────────────┐
                       │  Hook Generator  │    │  Pre-commit Hook    │
                       │  (Dynamic)       │    │  (Auto-managed)     │
                       └──────────────────┘    └─────────────────────┘
```

## Requirements

### Functional Requirements

#### FR-1: Single Configuration Source
- **Requirement**: All quality gate thresholds must be defined in root `pmat.toml` only
- **Rationale**: Eliminates configuration duplication and ensures consistency
- **Acceptance Criteria**: 
  - No hardcoded thresholds in source code
  - No threshold duplication in multiple files
  - All enforcement uses same configuration values

#### FR-2: Dynamic Hook Management
- **Requirement**: PMAT must automatically install and manage pre-commit hooks
- **Commands**:
  - `pmat hooks install` - Install/update hooks based on current config
  - `pmat hooks uninstall` - Remove PMAT-managed hooks
  - `pmat hooks status` - Show hook installation status
  - `pmat hooks verify` - Test hooks against current configuration
- **Auto-installation**: Hooks automatically update when config changes

#### FR-3: Configuration Command Interface
- **Requirement**: Provide commands for external tools to access current configuration
- **Commands**:
  - `pmat config show` - Display current complete configuration
  - `pmat config get <key>` - Get specific configuration value
  - `pmat config validate` - Validate configuration file
- **Output Format**: JSON, TOML, or environment variable format

#### FR-4: Hook Template Generation
- **Requirement**: Generate hook scripts dynamically from current configuration
- **Template Engine**: Use current config to populate hook templates
- **Update Mechanism**: Hooks automatically reflect config changes
- **Rollback Support**: Ability to restore previous hook versions

#### FR-5: Documentation Synchronization
- **Requirement**: Hooks must enforce documentation synchronization
- **Integration**: Use existing Toyota Way documentation requirements
- **Flexibility**: Allow configuration of required documentation files

### Non-Functional Requirements

#### NFR-1: Performance
- **Hook Execution**: Pre-commit hooks must complete in <30 seconds for typical projects
- **Configuration Loading**: Config access must be <100ms
- **Memory Usage**: Hook processes must use <50MB RAM

#### NFR-2: Reliability
- **Failure Handling**: Graceful degradation when PMAT binary unavailable
- **Recovery**: Automatic recovery from corrupted hook state
- **Atomicity**: Hook installation/removal must be atomic operations

#### NFR-3: Compatibility
- **Git Integration**: Compatible with all Git versions ≥2.0
- **Platform Support**: Linux, macOS, Windows support
- **Shell Independence**: Works with bash, zsh, fish, cmd, powershell

## Implementation Specification

### Configuration Schema

```toml
[hooks]
# Hook management configuration
enabled = true                    # Enable PMAT hook management
auto_install = true              # Automatically install hooks on config changes
backup_existing = true           # Backup existing hooks before installation

# Quality gate enforcement
enforce_quality_gates = true     # Enable quality gate enforcement
enforce_documentation = true     # Enable documentation synchronization
allow_skip = false              # Allow --no-verify to skip hooks (security setting)

# Performance settings
timeout_seconds = 30            # Maximum hook execution time
parallel_checks = true          # Run quality checks in parallel

[hooks.quality_gates]
# All quality gate thresholds (single source of truth)
max_cyclomatic_complexity = 30  # Industry standard
max_cognitive_complexity = 25   # Maintainable threshold
max_satd_comments = 5           # Enterprise allowance
min_test_coverage = 80.0        # High standard
max_clippy_warnings = 100       # Reasonable limit

[hooks.documentation]
# Documentation requirements
required_files = [
    "docs/execution/roadmap.md",
    "docs/execution/quality-gates.md", 
    "CHANGELOG.md"
]
task_id_pattern = "PMAT-[0-9]{4}"  # Task ID validation pattern
```

### Command Line Interface

#### Hook Management Commands
```bash
# Install/update hooks
pmat hooks install [--force] [--backup]

# Remove hooks
pmat hooks uninstall [--restore-backup]

# Show status
pmat hooks status

# Verify hooks work with current config
pmat hooks verify [--fix]

# Regenerate hooks from current config
pmat hooks refresh
```

#### Configuration Commands
```bash
# Show complete configuration
pmat config show [--format json|toml|env]

# Get specific value
pmat config get hooks.quality_gates.max_cyclomatic_complexity

# Validate configuration
pmat config validate [--fix]

# Show configuration source hierarchy
pmat config sources
```

### Hook Implementation Architecture

#### Template-Based Generation
- **Hook Template**: Stored in PMAT resources, parameterized with config values
- **Generation Process**: Template + current config = installed hook script
- **Update Trigger**: Config file modification triggers hook regeneration

#### Dynamic Configuration Loading
```bash
#!/bin/bash
# Generated pre-commit hook (auto-managed by PMAT)
# DO NOT EDIT: This file is automatically generated

# Load current configuration dynamically
CONFIG=$(pmat config show --format env)
eval "$CONFIG"

# Use configuration values
if ! pmat analyze complexity --max-cyclomatic $PMAT_MAX_CYCLOMATIC_COMPLEXITY; then
    echo "❌ Complexity violation (threshold: $PMAT_MAX_CYCLOMATIC_COMPLEXITY)"
    exit 1
fi
```

### Error Handling

#### Graceful Degradation
- **PMAT Unavailable**: Hook warns but allows commit with degraded checking
- **Config Invalid**: Hook uses safe defaults and reports issue
- **Network Issues**: Skip external checks, continue with local validation

#### Recovery Mechanisms
- **Corrupted Hooks**: `pmat hooks verify --fix` repairs installation
- **Backup System**: All hook modifications create timestamped backups
- **Manual Override**: Emergency `--no-verify` bypass with audit logging

## Testing Strategy

### Test-Driven Development Requirements
1. **Red**: Write failing tests for each command before implementation
2. **Green**: Implement minimum code to pass tests
3. **Refactor**: Ensure complexity ≤30 cyclomatic, ≤25 cognitive

### Test Categories

#### Unit Tests
- Configuration loading and validation
- Template generation and parameterization
- Command parsing and validation
- Error handling and recovery

#### Integration Tests  
- Hook installation/removal processes
- Git integration and hook execution
- Cross-platform compatibility
- Performance and timeout handling

#### Property Tests
- Configuration value propagation correctness
- Hook script generation determinism
- Backup/restore operation integrity
- Concurrent operation safety

### Acceptance Tests
- Complete workflow: config change → hook update → enforcement
- Documentation synchronization enforcement
- Quality gate enforcement with dynamic thresholds
- Error recovery and graceful degradation

## Implementation Plan

### Phase 1: Core Infrastructure (Sprint 80)
1. `pmat config` command implementation with TDD
2. Configuration loading and validation system
3. Template engine for hook generation
4. Basic `pmat hooks` command structure

### Phase 2: Hook Management (Sprint 80)
1. Hook installation/removal system
2. Backup and recovery mechanisms
3. Status and verification commands
4. Integration tests and validation

### Phase 3: Quality Integration (Sprint 80)
1. Quality gate enforcement using dynamic config
2. Documentation synchronization integration
3. Performance optimization and parallel execution
4. Cross-platform testing and compatibility

### Phase 4: Dogfooding & Release (Sprint 80)
1. Self-hosting: Use PMAT hooks to manage PMAT development
2. Comprehensive testing on PMAT codebase itself
3. Performance validation and optimization
4. Documentation and release preparation

## Success Criteria

### Primary Goals
- ✅ **Single Source of Truth**: Only `pmat.toml` contains quality gate thresholds
- ✅ **Zero Configuration Duplication**: No hardcoded or duplicated thresholds
- ✅ **Automatic Management**: Hooks update automatically when config changes
- ✅ **Toyota Way Compliance**: All implementation ≤30 cyclomatic, ≤25 cognitive complexity

### Quality Metrics
- **Test Coverage**: ≥80% for all new code
- **Integration Tests**: Complete workflow coverage
- **Property Tests**: All configuration transformations verified
- **Performance**: Hook execution <30s, config loading <100ms

### Dogfooding Validation
- PMAT development uses PMAT-managed hooks exclusively
- All quality gate changes verified through dogfooding
- Development productivity maintained or improved
- Zero manual hook configuration required

## Migration Strategy

### Backward Compatibility
- Existing manual hooks preserved as backup
- Gradual migration with validation at each step
- Rollback capability for emergency situations
- Clear migration documentation and tooling

### Deployment Process
1. Implement core infrastructure with comprehensive tests
2. Deploy to PMAT development environment (dogfooding)
3. Validate through real development workflow
4. Update documentation and create migration guide
5. Release with clear upgrade instructions

---

**This specification establishes pre-commit hook management as a core PMAT feature, eliminating configuration duplication and ensuring quality gates are enforced consistently across all projects using industry-standard Toyota Way principles.**