# TICKET-PMAT-5004: Project Structure Generation

**Status**: RED
**Priority**: P0
**Complexity**: 9
**Estimated Time**: 6 hours
**Dependencies**: TICKET-PMAT-5001, TICKET-PMAT-5002, TICKET-PMAT-5003
**Sprint**: Sprint 16 - Scaffolding Foundation

## Objective

Integrate ScaffoldEngine with TemplateRegistry to generate complete project structures. This includes rendering all templates, creating directory structures, and writing files to disk with proper permissions.

## Success Criteria

- [ ] ScaffoldEngine uses TemplateRegistry to render templates
- [ ] Generate complete directory structures (src/, tests/, benches/, docs/)
- [ ] Write all files to disk with correct permissions
- [ ] Support both pforge and WASM project types
- [ ] Integration tests verify generated projects compile
- [ ] All quality gates pass (complexity <10, coverage >80%, no SATD)

## Test Strategy

### Unit Tests
- [ ] `test_generate_files_from_registry` - Generate files from templates
- [ ] `test_create_directory_structure` - Create nested directories
- [ ] `test_write_file_with_content` - Write file to disk
- [ ] `test_scaffold_pforge_project` - Full pforge project generation
- [ ] `test_scaffold_wasm_project` - Full WASM project generation

### Property Tests
- [ ] Property: All generated files exist on disk
- [ ] Property: Generated files have correct permissions
- [ ] Property: No template variables unreplaced in generated files

### Integration Tests
- [ ] `integration_pforge_project_builds` - Generated pforge project compiles
- [ ] `integration_wasm_project_builds` - Generated WASM project compiles
- [ ] `integration_generated_tests_pass` - Tests in generated projects pass

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

### Phase 1: File Generation

```rust
// server/src/scaffold/mod.rs
impl ScaffoldEngine {
    /// Generate project from template type
    ///
    /// # Complexity
    /// - Time: O(n) where n is number of template files
    /// - Cyclomatic: 5
    pub fn generate_project(&self, config: ScaffoldConfig) -> Result<PathBuf> {
        // 1. Validate config
        self.validate_config(&config)?;

        // 2. Create project directory
        let project_dir = self.create_directory(&config.project_name)?;

        // 3. Initialize git
        self.init_git(&project_dir)?;

        // 4. Get appropriate template registry
        let registry = self.get_template_registry(&config.template_type);

        // 5. Create directory structure
        self.create_project_structure(&project_dir, &config.template_type)?;

        // 6. Render and write all templates
        self.generate_files(&project_dir, &registry, &config)?;

        Ok(project_dir)
    }

    /// Get template registry based on project type
    ///
    /// # Complexity
    /// - Time: O(1)
    /// - Cyclomatic: 3
    fn get_template_registry(&self, template_type: &TemplateType) -> TemplateRegistry {
        match template_type {
            TemplateType::Agent { .. } => TemplateRegistry::with_pforge_templates(),
            TemplateType::Wasm { .. } => TemplateRegistry::with_wasm_templates(),
            _ => TemplateRegistry::new(),
        }
    }

    /// Create project directory structure
    ///
    /// # Complexity
    /// - Time: O(1) - fixed number of directories
    /// - Cyclomatic: 3
    fn create_project_structure(
        &self,
        project_dir: &Path,
        template_type: &TemplateType,
    ) -> Result<()> {
        match template_type {
            TemplateType::Agent { .. } => {
                fs::create_dir_all(project_dir.join("src/handlers"))?;
                fs::create_dir_all(project_dir.join("tests"))?;
                fs::create_dir_all(project_dir.join("docs"))?;
            }
            TemplateType::Wasm { .. } => {
                fs::create_dir_all(project_dir.join("src"))?;
                fs::create_dir_all(project_dir.join("tests"))?;
                fs::create_dir_all(project_dir.join("benches"))?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Generate all files from templates
    ///
    /// # Complexity
    /// - Time: O(n*m) where n=templates, m=avg template size
    /// - Cyclomatic: 4
    fn generate_files(
        &self,
        project_dir: &Path,
        registry: &TemplateRegistry,
        config: &ScaffoldConfig,
    ) -> Result<()> {
        // Prepare variables for template rendering
        let vars = self.prepare_template_vars(config);

        // Render and write each template
        for template_name in registry.list() {
            let template = registry.get(&template_name)?;
            let rendered = template.render(&vars)?;

            let file_path = self.get_file_path(
                project_dir,
                &template_name,
                &config.template_type,
            );

            self.write_file(&file_path, &rendered)?;
        }

        Ok(())
    }

    /// Prepare template variables from config
    ///
    /// # Complexity
    /// - Time: O(1)
    /// - Cyclomatic: 1
    fn prepare_template_vars(&self, config: &ScaffoldConfig) -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert("project_name".into(), config.project_name.clone());
        vars.insert("author".into(), "Developer".into()); // TODO: from config
        vars.insert("description".into(), format!("{} project", config.project_name));

        // Add handler-specific vars if needed
        vars.insert("handler_name".into(), "Example".into());
        vars.insert("handler_description".into(), "Example handler".into());

        vars
    }

    /// Get file path for template
    ///
    /// # Complexity
    /// - Time: O(1)
    /// - Cyclomatic: 6
    fn get_file_path(
        &self,
        project_dir: &Path,
        template_name: &str,
        template_type: &TemplateType,
    ) -> PathBuf {
        let path = match template_name {
            "pforge.yaml" => project_dir.join("pforge.yaml"),
            "Cargo.toml" => project_dir.join("Cargo.toml"),
            "Makefile" => project_dir.join("Makefile"),
            "README.md" => project_dir.join("README.md"),
            "handler.rs" => project_dir.join("src/handlers/example.rs"),
            "lib.rs" => project_dir.join("src/lib.rs"),
            "vfs.rs" => project_dir.join("src/vfs.rs"),
            _ => project_dir.join(template_name),
        };
        path
    }

    /// Write file to disk
    ///
    /// # Complexity
    /// - Time: O(n) where n is content length
    /// - Cyclomatic: 2
    fn write_file(&self, path: &Path, content: &str) -> Result<()> {
        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, content)
            .map_err(ScaffoldError::IoError)?;

        Ok(())
    }
}
```

### Phase 2: Directory Structure

**pforge Project Structure**:
```
project-name/
├── Cargo.toml
├── pforge.yaml
├── Makefile (optional)
├── README.md
├── .git/
├── src/
│   └── handlers/
│       └── example.rs
├── tests/
│   └── integration_tests.rs (generated later)
└── docs/
    └── tickets/ (generated later)
```

**WASM Project Structure**:
```
project-name/
├── Cargo.toml
├── Makefile
├── README.md
├── .git/
├── src/
│   ├── lib.rs
│   └── vfs.rs
├── tests/
│   └── property_tests.rs (generated later)
└── benches/
    └── benchmarks.rs (generated later)
```

### Phase 3: Tests

```rust
// tests
#[test]
fn test_generate_files_from_registry() {
    let temp_dir = TempDir::new().unwrap();
    let engine = ScaffoldEngine::new().unwrap();
    let registry = TemplateRegistry::with_pforge_templates();

    let config = ScaffoldConfig {
        project_name: "test-project".into(),
        template_type: TemplateType::Agent { based_on: AgentFramework::Pforge },
        features: vec![],
        quality_gates: QualityGateConfig::default(),
    };

    let vars = engine.prepare_template_vars(&config);
    engine.generate_files(temp_dir.path(), &registry, &config).unwrap();

    // Verify files exist
    assert!(temp_dir.path().join("pforge.yaml").exists());
    assert!(temp_dir.path().join("Cargo.toml").exists());
}

#[test]
fn test_scaffold_pforge_project() {
    let temp_dir = TempDir::new().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let config = ScaffoldConfig {
        project_name: "my-agent".into(),
        template_type: TemplateType::Agent { based_on: AgentFramework::Pforge },
        features: vec![],
        quality_gates: QualityGateConfig::extreme_tdd(),
    };

    let engine = ScaffoldEngine::new().unwrap();
    let project_dir = engine.generate_project(config).unwrap();

    // Verify structure
    assert!(project_dir.join("pforge.yaml").exists());
    assert!(project_dir.join("Cargo.toml").exists());
    assert!(project_dir.join("src/handlers").exists());
    assert!(project_dir.join(".git").exists());
}

// Integration test
#[test]
#[ignore] // Slow test - run with --ignored
fn integration_pforge_project_builds() {
    let temp_dir = TempDir::new().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let config = ScaffoldConfig {
        project_name: "build-test-agent".into(),
        template_type: TemplateType::Agent { based_on: AgentFramework::Pforge },
        features: vec![],
        quality_gates: QualityGateConfig::default(),
    };

    let engine = ScaffoldEngine::new().unwrap();
    let project_dir = engine.generate_project(config).unwrap();

    // Try to build the generated project
    let output = Command::new("cargo")
        .current_dir(&project_dir)
        .args(&["build"])
        .output()
        .unwrap();

    assert!(output.status.success(), "Generated project should compile");
}
```

## Complexity Analysis

New functions with complexity:
- `generate_project`: CC=5 (validation, directory creation, git, registry selection, file generation)
- `get_template_registry`: CC=3 (match with 3 arms)
- `create_project_structure`: CC=3 (match with 3 arms)
- `generate_files`: CC=4 (loop + error handling)
- `prepare_template_vars`: CC=1
- `get_file_path`: CC=6 (match with 7 cases) - **NEEDS REFACTORING** if >10
- `write_file`: CC=2

**Note**: `get_file_path` at CC=6 is acceptable but close to threshold.

## Verification Commands

```bash
# Run tests
cargo test --lib scaffold::tests::test_scaffold_pforge_project
cargo test --lib scaffold::tests::test_scaffold_wasm_project

# Run integration tests
cargo test --lib --ignored integration_pforge_project_builds
cargo test --lib --ignored integration_wasm_project_builds

# Manual test: Generate and build a project
cd /tmp
cargo run --bin pmat -- scaffold agent --name test-agent
cd test-agent
cargo build
cargo test
```

## Files to Create/Modify

### Modified Files
- `server/src/scaffold/mod.rs` - Add generation methods
- `server/src/scaffold/tests.rs` - Add generation tests
- `server/src/scaffold/property_tests.rs` - Add generation property tests

### New Files
- `server/src/tests/scaffold_integration_tests.rs` - Integration tests

## Risk Assessment

**Medium Risk:**
- File I/O operations may fail
- Generated projects may not compile if templates are malformed

**Mitigation:**
- Comprehensive error handling
- Integration tests that actually compile generated projects
- Property tests verify no unreplaced variables

## Notes

This ticket brings together all previous work:
- TICKET-PMAT-5001: ScaffoldEngine foundation
- TICKET-PMAT-5002: pforge templates
- TICKET-PMAT-5003: WASM templates

The result is a fully functional project generator!

**TDD Cycle Duration**: Estimated 2-3 hours for RED → GREEN → REFACTOR
