# TICKET-PMAT-5003: Template System (wasm-labs-based WASM)

**Status**: RED
**Priority**: P0
**Complexity**: 8
**Estimated Time**: 6 hours
**Dependencies**: TICKET-PMAT-5002 (Template System)
**Sprint**: Sprint 16 - Scaffolding Foundation

## Objective

Implement template rendering system for scaffolding wasm-labs-based WASM projects. This extends the Template system to support pure WASM projects with extreme quality gates, property-based testing, and deterministic execution patterns.

## Success Criteria

- [ ] WASM-specific templates (Cargo.toml, Makefile, lib.rs, benches/, tests/)
- [ ] Template registry extended with WASM templates
- [ ] Templates follow wasm-labs best practices (pure functions, im-rs, deterministic)
- [ ] Generate valid WASM projects that compile and pass tests
- [ ] All quality gates pass (complexity <10, coverage >80%, no SATD)

## Test Strategy

### Unit Tests
- [ ] `test_wasm_cargo_toml` - Generate valid Cargo.toml for WASM
- [ ] `test_wasm_makefile` - Generate Makefile with quality gates
- [ ] `test_wasm_lib_rs` - Generate lib.rs with WASM exports
- [ ] `test_wasm_vfs_template` - Generate VFS module
- [ ] `test_wasm_context_template` - Generate context module
- [ ] `test_wasm_benchmark_template` - Generate benchmark file
- [ ] `test_wasm_property_test_template` - Generate property test file

### Property Tests
- [ ] Property: Generated Cargo.toml has wasm32-unknown-unknown target
- [ ] Property: Generated Makefile contains wasm-full target
- [ ] Property: Generated lib.rs exports valid WASM functions
- [ ] Property: All templates render without unreplaced variables

### Integration Tests
- [ ] `integration_generate_wasm_project` - Full WASM project scaffolding
- [ ] `integration_wasm_project_builds` - Generated project compiles
- [ ] `integration_wasm_tests_pass` - Generated tests pass

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

### Phase 1: Create WASM Templates

Reference: `../wasm-labs` best practices

**Cargo.toml Template**:
```toml
# templates/wasm_Cargo.toml.tmpl
[package]
name = "{{project_name}}"
version = "0.1.0"
edition = "2021"
authors = ["{{author}}"]

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
im = "15.1"  # Persistent data structures

[dev-dependencies]
wasm-bindgen-test = "0.3"
proptest = "1.0"
criterion = "0.5"

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = "symbols"
```

**Makefile Template**:
```makefile
# templates/wasm_Makefile.tmpl
.PHONY: help build test coverage quality wasm-full

help:
	@echo "WASM Project: {{project_name}}"
	@echo "make build          - Build WASM binary"
	@echo "make test           - Run tests"
	@echo "make coverage       - Generate coverage report"
	@echo "make quality        - Run quality gates"
	@echo "make wasm-full      - Complete WASM pipeline"

build:
	cargo build --target wasm32-unknown-unknown --release

test:
	cargo test --all-features

coverage:
	cargo llvm-cov --lib --lcov --output-path target/lcov.info
	cargo llvm-cov report --summary-only

quality: test
	cargo fmt --check
	cargo clippy --all-targets -- -D warnings
	pmat analyze complexity --max-cyclomatic 10 --path $(CURDIR)
	pmat analyze satd --strict --path $(CURDIR)

wasm-full: build
	wasm-bindgen target/wasm32-unknown-unknown/release/{{project_name}}.wasm \
		--out-dir pkg --target web
	@echo "✨ WASM build complete: pkg/{{project_name}}_bg.wasm"
```

**lib.rs Template**:
```rust
// templates/wasm_lib.rs.tmpl
//! {{project_name}}
//!
//! {{description}}

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

/// Main state for {{project_name}}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    // Add state fields
}

impl State {
    /// Create new state
    ///
    /// # Complexity
    /// - Time: O(1)
    /// - Cyclomatic: 1
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

/// WASM entry point
#[wasm_bindgen]
pub fn process(input: &str) -> Result<String, JsValue> {
    let _state = State::new();

    // Process input
    Ok(format!("Processed: {}", input))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_creation() {
        let state = State::new();
        assert!(true); // Replace with actual assertion
    }

    #[test]
    fn test_process() {
        let result = process("test").unwrap();
        assert!(result.contains("test"));
    }
}
```

**VFS Template**:
```rust
// templates/wasm_vfs.rs.tmpl
//! Virtual filesystem using persistent data structures

use im::HashMap;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Virtual filesystem with O(1) cloning
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct VirtualFileSystem {
    files: HashMap<PathBuf, Vec<u8>>,
    cwd: PathBuf,
}

impl VirtualFileSystem {
    /// Create new VFS
    ///
    /// # Complexity
    /// - Time: O(1)
    /// - Cyclomatic: 1
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            cwd: PathBuf::from("/"),
        }
    }

    /// Write file (returns new VFS)
    ///
    /// # Complexity
    /// - Time: O(log n) where n is number of files
    /// - Cyclomatic: 1
    pub fn write(&self, path: PathBuf, content: Vec<u8>) -> Self {
        let mut new_vfs = self.clone();
        new_vfs.files.insert(path, content);
        new_vfs
    }

    /// Read file
    ///
    /// # Complexity
    /// - Time: O(log n)
    /// - Cyclomatic: 2
    pub fn read(&self, path: &PathBuf) -> Option<&Vec<u8>> {
        self.files.get(path)
    }
}

impl Default for VirtualFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vfs_write_read() {
        let vfs = VirtualFileSystem::new();
        let path = PathBuf::from("/test.txt");
        let content = b"hello".to_vec();

        let vfs2 = vfs.write(path.clone(), content.clone());

        assert_eq!(vfs2.read(&path), Some(&content));
        assert_eq!(vfs.read(&path), None); // Original unchanged
    }
}
```

### Phase 2: Extend TemplateRegistry

```rust
// server/src/scaffold/template.rs
impl Template {
    /// WASM Cargo.toml template
    pub fn wasm_cargo_toml() -> Self {
        Self::new(
            "Cargo.toml",
            include_str!("templates/wasm_Cargo.toml.tmpl")
        ).expect("wasm Cargo.toml template should be valid")
    }

    /// WASM Makefile template
    pub fn wasm_makefile() -> Self {
        Self::new(
            "Makefile",
            include_str!("templates/wasm_Makefile.tmpl")
        ).expect("wasm Makefile template should be valid")
    }

    /// WASM lib.rs template
    pub fn wasm_lib_rs() -> Self {
        Self::new(
            "lib.rs",
            include_str!("templates/wasm_lib.rs.tmpl")
        ).expect("wasm lib.rs template should be valid")
    }

    /// WASM VFS template
    pub fn wasm_vfs_rs() -> Self {
        Self::new(
            "vfs.rs",
            include_str!("templates/wasm_vfs.rs.tmpl")
        ).expect("wasm vfs.rs template should be valid")
    }
}

impl TemplateRegistry {
    /// Create registry with WASM templates
    pub fn with_wasm_templates() -> Self {
        let mut registry = Self::new();
        registry.register("Cargo.toml".into(), Template::wasm_cargo_toml());
        registry.register("Makefile".into(), Template::wasm_makefile());
        registry.register("lib.rs".into(), Template::wasm_lib_rs());
        registry.register("vfs.rs".into(), Template::wasm_vfs_rs());
        registry
    }
}
```

### Phase 3: Tests

```rust
// tests
#[test]
fn test_wasm_cargo_toml() {
    let template = Template::wasm_cargo_toml();
    let mut vars = HashMap::new();
    vars.insert("project_name".into(), "my-wasm".into());
    vars.insert("author".into(), "Test Author".into());

    let result = template.render(&vars).unwrap();

    assert!(result.contains("name = \"my-wasm\""));
    assert!(result.contains("crate-type = [\"cdylib\", \"rlib\"]"));
    assert!(result.contains("wasm-bindgen"));
}

proptest! {
    #[test]
    fn prop_wasm_cargo_has_wasm_target(name in valid_project_name()) {
        let template = Template::wasm_cargo_toml();
        let mut vars = HashMap::new();
        vars.insert("project_name".into(), name);
        vars.insert("author".into(), "Test".into());

        let rendered = template.render(&vars).unwrap();

        // Property: WASM projects have cdylib
        prop_assert!(rendered.contains("cdylib"));
    }
}
```

## Complexity Analysis

New functions with complexity:
- `Template::wasm_cargo_toml`: CC=1
- `Template::wasm_makefile`: CC=1
- `Template::wasm_lib_rs`: CC=1
- `Template::wasm_vfs_rs`: CC=1
- `TemplateRegistry::with_wasm_templates`: CC=1

All functions CC=1 (well under threshold of 10)

## Verification Commands

```bash
# Run tests
cargo test --lib scaffold::template::wasm_tests

# Generate and test a WASM project
cd /tmp
pmat scaffold wasm --name test-wasm --template wasm-labs
cd test-wasm
make build
make test
make quality
```

## Files to Create/Modify

### New Files
- `server/src/scaffold/templates/wasm_Cargo.toml.tmpl`
- `server/src/scaffold/templates/wasm_Makefile.tmpl`
- `server/src/scaffold/templates/wasm_lib.rs.tmpl`
- `server/src/scaffold/templates/wasm_vfs.rs.tmpl`
- `server/src/scaffold/templates/wasm_context.rs.tmpl`
- `server/src/scaffold/templates/wasm_benchmark.rs.tmpl`
- `server/src/scaffold/template/wasm_tests.rs` - WASM-specific tests

### Modified Files
- `server/src/scaffold/template.rs` - Add WASM template methods
- `server/src/scaffold/template/tests.rs` - Add WASM template tests

## Risk Assessment

**Low Risk:**
- Building on proven Template system from TICKET-PMAT-5002
- Templates based on tested wasm-labs patterns

**Mitigation:**
- Property tests ensure valid WASM configuration
- Integration tests verify projects compile

## Notes

This ticket extends the template system with WASM-specific patterns from wasm-labs:
- Pure functional design (explicit state threading)
- Persistent data structures (im-rs)
- Quality gates in Makefile
- Property-based testing included by default

**TDD Cycle Duration**: Estimated 2-3 hours for RED → GREEN → REFACTOR
