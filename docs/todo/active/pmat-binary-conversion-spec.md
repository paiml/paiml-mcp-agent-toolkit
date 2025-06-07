# PMAT Binary Conversion Specification

Version: 1.0.0  
Date: 2025-06-04  
Status: Draft

## Executive Summary

This specification defines the systematic conversion of the `paiml-mcp-agent-toolkit` binary to `pmat` while maintaining internal module structure. The conversion preserves package distribution naming for release continuity while updating user-facing interfaces and documentation.

## Technical Scope

### Binary Renaming

**Primary Target**: `server/src/bin/paiml-mcp-agent-toolkit.rs` → `server/src/bin/pmat.rs`

**Cargo.toml Modifications**:
```toml
# server/Cargo.toml
[[bin]]
name = "pmat"
path = "src/bin/pmat.rs"
```

### MCP Protocol Identity

**File**: `server/mcp.json`
```json
{
  "name": "pmat",
  "description": "Project Analysis and Intelligence Modeling Toolkit",
  // Other fields remain unchanged
}
```

### Docker Configuration Updates

**Dockerfile Entrypoint**:
```dockerfile
# server/Dockerfile
ENTRYPOINT ["pmat"]
```

**Docker Compose References** (if present):
- Update service names from `paiml-mcp-agent-toolkit` to `pmat`
- Maintain volume mounts and network configurations

## File Modification Matrix

### Critical Path Files

| File | Modification Type | Test Coverage Impact |
|------|------------------|---------------------|
| `server/src/bin/paiml-mcp-agent-toolkit.rs` | Rename to `pmat.rs` | Binary execution tests |
| `server/Cargo.toml` | Update `[[bin]]` section | Build system tests |
| `server/mcp.json` | Update `name` field | MCP protocol tests |
| `README.md` | Update all binary references | Documentation tests |
| `CLAUDE.md` | Update usage examples | - |
| `installer.sh` | Update binary name references | Installation tests |
| `server/installer.sh` | Update binary name references | Installation tests |

### Documentation Updates

**Pattern Matching for Updates**:
```regex
# Find all references
paiml-mcp-agent-toolkit(?!.*\.tar\.gz)
```

**Excluded Patterns** (preserve package names):
```regex
paiml-mcp-agent-toolkit.*\.tar\.gz
paiml-mcp-agent-toolkit-v\d+\.\d+\.\d+
```

### Script Modifications

| Script | Update Required | Validation Method |
|--------|----------------|-------------------|
| `scripts/install.sh` | Binary path references | Integration test |
| `scripts/install.ts` | Binary name in checks | Deno test suite |
| `scripts/test-curl-install.ts` | Expected binary name | E2E test |
| `scripts/validate-naming.ts` | Add pmat to valid names | Unit test |

## Self-Verification Strategy

### Phase 1: Pre-Conversion Baseline

```bash
# Generate baseline metrics using current binary
./paiml-mcp-agent-toolkit analyze --deep-context . > baseline-metrics.json
./paiml-mcp-agent-toolkit mermaid --output baseline-dag.mmd
./paiml-mcp-agent-toolkit complexity --json > baseline-complexity.json
```

### Phase 2: Post-Conversion Validation

```bash
# Verify identical functionality with new binary
./pmat analyze --deep-context . > post-metrics.json
./pmat mermaid --output post-dag.mmd
./pmat complexity --json > post-complexity.json

# Differential analysis
diff baseline-metrics.json post-metrics.json
diff baseline-dag.mmd post-dag.mmd
diff baseline-complexity.json post-complexity.json
```

### Verification Criteria

1. **Functional Equivalence**: Zero diff between baseline and post-conversion outputs
2. **Performance Regression**: Execution time within 5% tolerance
3. **Memory Usage**: Peak RSS within 10% tolerance

## Test Coverage Strategy

### Unit Test Modifications

```rust
// server/src/tests/binary_size.rs
#[test]
fn test_binary_name() {
    let binary_path = env!("CARGO_BIN_EXE_pmat");
    assert!(binary_path.contains("pmat"));
}
```

### Integration Test Suite

```rust
// server/tests/bin_integration.rs
#[test]
fn test_pmat_execution() {
    let output = Command::new("pmat")
        .arg("--version")
        .output()
        .expect("Failed to execute pmat");
    
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("pmat"));
}
```

### Coverage Enforcement

```toml
# .config/nextest.toml
[profile.ci.junit]
path = "junit.xml"

[profile.coverage]
failure-output = "immediate-final"
success-output = "final"
test-coverage-threshold = 80
```

## Implementation Phases

### Phase 1: Core Binary Conversion (Day 1)
1. Rename binary file
2. Update Cargo.toml
3. Verify build success
4. Run existing test suite

### Phase 2: Documentation Update (Day 1-2)
1. Update README.md
2. Update CLAUDE.md
3. Update inline documentation
4. Update CLI help text

### Phase 3: Infrastructure Updates (Day 2)
1. Update Docker configurations
2. Update MCP protocol definition
3. Update installation scripts
4. Update CI/CD references

### Phase 4: Verification (Day 3)
1. Execute pre-conversion baseline
2. Complete conversion
3. Execute post-conversion validation
4. Generate comparison report

## Risk Mitigation

### Binary Discovery Issues
- **Risk**: Scripts failing to find renamed binary
- **Mitigation**: Comprehensive grep search for hardcoded paths
- **Validation**: Full E2E test suite execution

### MCP Protocol Breaking Changes
- **Risk**: MCP clients unable to discover renamed tool
- **Mitigation**: Staged rollout with compatibility testing
- **Validation**: MCP protocol conformance tests

### Installation Script Failures
- **Risk**: curl installation pipeline breakage
- **Mitigation**: Test installation on all supported platforms
- **Validation**: Automated installation tests in CI

## Validation Checklist

- [ ] Binary builds successfully as `pmat`
- [ ] All unit tests pass (>80% coverage maintained)
- [ ] All integration tests pass
- [ ] MCP protocol tests pass with new identity
- [ ] Docker container starts with correct entrypoint
- [ ] Installation scripts successfully install `pmat`
- [ ] Self-verification shows functional equivalence
- [ ] Documentation accurately reflects new binary name
- [ ] No references to old binary name in user-facing content
- [ ] Performance benchmarks remain within tolerance

## Appendix: Affected File Inventory

### Primary Modifications Required
```
server/src/bin/paiml-mcp-agent-toolkit.rs → server/src/bin/pmat.rs
server/Cargo.toml
server/mcp.json
server/Dockerfile
README.md
CLAUDE.md
installer.sh
server/installer.sh
```

### Secondary Documentation Updates
```
docs/cli-spec.md
docs/cli-mcp.md
rust-docs/cli-reference.md
scripts/install.sh
scripts/install.ts
scripts/mcp-install.ts
```

### Test File Updates
```
server/src/tests/binary_size.rs
server/tests/bin_integration.rs
server/tests/cli_comprehensive_integration.rs
server/tests/e2e/installation.test.ts
scripts/install.test.ts
scripts/install.integration.test.ts
```

## Success Metrics

1. **Zero Functional Regression**: Bit-identical output for all commands
2. **Coverage Maintenance**: ≥80% test coverage post-conversion
3. **Performance Stability**: <5% variance in execution time
4. **Installation Success**: 100% success rate across all platforms
5. **MCP Protocol Compliance**: Full protocol test suite passage