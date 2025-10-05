# TICKET-PMAT-5001: Core ScaffoldEngine Implementation

**Status**: RED
**Priority**: P0
**Complexity**: 8
**Estimated Time**: 6 hours
**Dependencies**: None
**Sprint**: Sprint 16 - Scaffolding Foundation

## Objective

Implement the core `ScaffoldEngine` that orchestrates project scaffolding from templates. This is the foundation for all scaffolding operations.

## Success Criteria

- [ ] `ScaffoldEngine` struct with core methods implemented
- [ ] Configuration validation (complexity <10)
- [ ] Directory creation with error handling
- [ ] Git initialization
- [ ] Basic template rendering
- [ ] All quality gates pass (complexity <10, coverage >80%, no SATD)

## Test Strategy

### Unit Tests
- [ ] `test_scaffold_engine_creation` - Verify engine instantiation
- [ ] `test_validate_config_valid` - Valid configuration accepted
- [ ] `test_validate_config_invalid_name` - Invalid names rejected
- [ ] `test_create_directory_success` - Directory creation works
- [ ] `test_create_directory_exists` - Handle existing directory
- [ ] `test_init_git_success` - Git initialization works
- [ ] `test_init_git_fails_gracefully` - Handle git errors

### Property Tests
- [ ] Property: Project names are valid filesystem paths
- [ ] Property: Created directories always have expected structure
- [ ] Property: Git initialization is idempotent

### Integration Tests
- [ ] `integration_scaffold_minimal_project` - End-to-end minimal scaffold
- [ ] `integration_scaffold_cleanup_on_error` - Rollback on failure

## Quality Gates

- [ ] Cyclomatic complexity <10 for all functions
- [ ] Cognitive complexity <15 for all functions
- [ ] Line coverage >80%
- [ ] Branch coverage >80%
- [ ] 0 SATD violations
- [ ] 0 clippy warnings
- [ ] All tests pass
- [ ] Mutation score >85% (target)

## Implementation Plan

### Phase 1: RED (Write Failing Tests)
```rust
// tests/scaffold_engine_tests.rs
#[test]
fn test_scaffold_engine_creation() {
    let engine = ScaffoldEngine::new();
    assert!(engine.is_ok());
}

#[test]
fn test_validate_config_valid() {
    let config = ScaffoldConfig {
        project_name: "valid-project".into(),
        template: Template::Agent { based_on: AgentFramework::Pforge },
        features: vec![],
        quality_gates: QualityGateConfig::default(),
    };

    let engine = ScaffoldEngine::new().unwrap();
    assert!(engine.validate_config(&config).is_ok());
}
```

### Phase 2: GREEN (Minimal Implementation)
```rust
// server/src/scaffold/mod.rs
pub struct ScaffoldEngine {
    template_dir: PathBuf,
}

impl ScaffoldEngine {
    pub fn new() -> Result<Self> {
        Ok(Self {
            template_dir: PathBuf::from("templates"),
        })
    }

    pub fn validate_config(&self, config: &ScaffoldConfig) -> Result<()> {
        validate_project_name(&config.project_name)?;
        Ok(())
    }

    pub fn create_directory(&self, name: &str) -> Result<PathBuf> {
        let path = PathBuf::from(name);
        fs::create_dir_all(&path)?;
        Ok(path)
    }

    pub fn init_git(&self, project_dir: &Path) -> Result<()> {
        Command::new("git")
            .args(&["init"])
            .current_dir(project_dir)
            .output()?;
        Ok(())
    }
}

fn validate_project_name(name: &str) -> Result<()> {
    if is_valid_name(name) {
        Ok(())
    } else {
        Err(Error::InvalidProjectName(name.into()))
    }
}

fn is_valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() < 256
        && !name.contains(['/', '\\', '\0'])
}
```

### Phase 3: REFACTOR (Property Tests + Optimization)
```rust
// Property tests
proptest! {
    #[test]
    fn prop_valid_names_accepted(name in valid_project_name()) {
        let config = ScaffoldConfig {
            project_name: name,
            template: Template::Agent { based_on: AgentFramework::Pforge },
            features: vec![],
            quality_gates: QualityGateConfig::default(),
        };

        let engine = ScaffoldEngine::new().unwrap();
        assert!(engine.validate_config(&config).is_ok());
    }

    #[test]
    fn prop_directory_creation_idempotent(name in valid_project_name()) {
        let engine = ScaffoldEngine::new().unwrap();

        let path1 = engine.create_directory(&name).unwrap();
        let path2 = engine.create_directory(&name).unwrap();

        assert_eq!(path1, path2);
        assert!(path1.exists());
    }
}
```

## Complexity Analysis

Target functions with their estimated complexity:
- `validate_config`: CC=3 (input validation branches)
- `create_directory`: CC=2 (error handling)
- `init_git`: CC=2 (error handling)
- `validate_project_name`: CC=1 (delegates to is_valid_name)
- `is_valid_name`: CC=4 (multiple conditions)

All functions well under CC=10 threshold.

## Verification Commands

```bash
# Run unit tests
cargo test --lib scaffold::tests

# Run property tests
cargo test --lib scaffold::property_tests

# Check coverage
cargo llvm-cov --lib --lcov --output-path target/lcov.info
cargo llvm-cov report --summary-only

# Check complexity
pmat analyze complexity --path server/src/scaffold/mod.rs --max-cyclomatic 10

# Check SATD
pmat analyze satd --path server/src/scaffold/ --strict

# Run clippy
cargo clippy --all-targets -- -D warnings

# Mutation testing (after GREEN phase)
pmat analyze mutate --path server/src/scaffold/mod.rs --min-score 85
```

## Files to Create/Modify

### New Files
- `server/src/scaffold/mod.rs` - Main module
- `server/src/scaffold/config.rs` - Configuration types
- `server/src/scaffold/errors.rs` - Error types
- `server/src/scaffold/tests.rs` - Unit tests
- `server/src/scaffold/property_tests.rs` - Property tests
- `server/src/tests/scaffold_integration_tests.rs` - Integration tests

### Modified Files
- `server/src/lib.rs` - Add scaffold module
- `Cargo.toml` - Add proptest dependency (dev)

## Risk Assessment

**Low Risk:**
- Standard file I/O operations
- Well-understood git initialization
- Clear validation rules

**Mitigation:**
- Comprehensive error handling
- Property tests for edge cases
- Integration tests for rollback

## Notes

This ticket establishes the foundation. Subsequent tickets (5002-5005) will build on this to add template rendering, hook installation, etc.

**TDD Cycle Duration**: Estimated 2-3 hours for RED → GREEN → REFACTOR
