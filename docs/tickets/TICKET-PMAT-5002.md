# TICKET-PMAT-5002: Template System (pforge-based agents)

**Status**: GREEN ✅
**Priority**: P0
**Complexity**: 9
**Estimated Time**: 8 hours
**Dependencies**: TICKET-PMAT-5001 (Core ScaffoldEngine)
**Sprint**: Sprint 16 - Scaffolding Foundation

## Objective

Implement template rendering system for scaffolding pforge-based MCP agents. This builds on the Core ScaffoldEngine to generate complete agent projects with pforge.yaml, handlers, tests, and documentation.

## Success Criteria

- [ ] Template struct with variable substitution
- [ ] Template registry for pforge agent templates
- [ ] File generation from templates (Cargo.toml, pforge.yaml, src/, tests/)
- [ ] Variable interpolation ({{project_name}}, {{author}}, etc.)
- [ ] Template validation
- [ ] All quality gates pass (complexity <10, coverage >80%, no SATD)

## Test Strategy

### Unit Tests
- [ ] `test_template_creation` - Create template from string
- [ ] `test_template_render_simple` - Basic variable substitution
- [ ] `test_template_render_nested` - Nested variable substitution
- [ ] `test_template_registry_register` - Register template
- [ ] `test_template_registry_get` - Retrieve template by name
- [ ] `test_template_render_pforge_yaml` - Generate valid pforge.yaml
- [ ] `test_template_render_cargo_toml` - Generate valid Cargo.toml
- [ ] `test_template_render_handler` - Generate handler code
- [ ] `test_template_validation` - Validate template syntax

### Property Tests
- [ ] Property: Template rendering is deterministic (same input → same output)
- [ ] Property: All variables in template are substituted
- [ ] Property: Generated files have valid syntax
- [ ] Property: Template rendering never panics

### Integration Tests
- [ ] `integration_generate_pforge_agent` - Full agent scaffolding
- [ ] `integration_generated_agent_builds` - Generated project compiles
- [ ] `integration_generated_agent_tests_pass` - Generated tests pass

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
// tests/template_system_tests.rs
#[test]
fn test_template_render_simple() {
    let template = Template::from_string("Hello {{name}}!").unwrap();
    let mut vars = HashMap::new();
    vars.insert("name".to_string(), "World".to_string());

    let result = template.render(&vars).unwrap();
    assert_eq!(result, "Hello World!");
}

#[test]
fn test_template_render_pforge_yaml() {
    let template = Template::pforge_yaml();
    let vars = create_test_vars();

    let result = template.render(&vars).unwrap();

    // Should be valid YAML
    let parsed: serde_yaml::Value = serde_yaml::from_str(&result).unwrap();
    assert_eq!(parsed["forge"]["name"], "test-agent");
}
```

### Phase 2: GREEN (Minimal Implementation)

```rust
// server/src/scaffold/template.rs
use std::collections::HashMap;

pub struct Template {
    content: String,
    name: String,
}

impl Template {
    pub fn from_string(content: impl Into<String>) -> Result<Self> {
        Ok(Self {
            content: content.into(),
            name: String::new(),
        })
    }

    /// Render template with variable substitution
    ///
    /// # Complexity
    /// - Time: O(n*m) where n=template size, m=number of variables
    /// - Cyclomatic: 3 (iteration, substitution, error handling)
    pub fn render(&self, vars: &HashMap<String, String>) -> Result<String> {
        let mut result = self.content.clone();

        for (key, value) in vars {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        Ok(result)
    }

    pub fn pforge_yaml() -> Self {
        Self::from_string(include_str!("templates/pforge.yaml.tmpl")).unwrap()
    }
}

pub struct TemplateRegistry {
    templates: HashMap<String, Template>,
}

impl TemplateRegistry {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: String, template: Template) {
        self.templates.insert(name, template);
    }

    pub fn get(&self, name: &str) -> Result<&Template> {
        self.templates.get(name)
            .ok_or_else(|| TemplateError::NotFound(name.into()))
    }
}
```

### Phase 3: REFACTOR (Property Tests + Templates)

**Template Files** (embedded in binary):
```yaml
# templates/pforge.yaml.tmpl
forge:
  name: "{{project_name}}"
  version: "0.1.0"
  description: "{{description}}"
  author: "{{author}}"

tools:
  - name: "example"
    handler: "handlers::example::execute"
    params:
      type: "object"
      properties:
        message:
          type: "string"
          description: "Message to process"
    timeout_ms: 30000
```

```toml
# templates/Cargo.toml.tmpl
[package]
name = "{{project_name}}"
version = "0.1.0"
edition = "2021"
authors = ["{{author}}"]

[dependencies]
pforge-runtime = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
tokio = { version = "1.0", features = ["full"] }
```

```rust
// templates/handler.rs.tmpl
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct {{handler_name}}Input {
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct {{handler_name}}Output {
    pub result: String,
}

/// Handler: {{handler_description}}
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 1
pub async fn execute(input: {{handler_name}}Input) -> Result<{{handler_name}}Output> {
    Ok({{handler_name}}Output {
        result: format!("Processed: {}", input.message),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute() {
        let input = {{handler_name}}Input {
            message: "test".into(),
        };

        let result = execute(input).await.unwrap();
        assert!(result.result.contains("test"));
    }
}
```

**Property Tests**:
```rust
proptest! {
    #[test]
    fn prop_template_render_deterministic(
        name in "[a-z][a-z0-9-]{3,20}",
        author in "[A-Za-z ]{3,30}"
    ) {
        let template = Template::pforge_yaml();
        let mut vars = HashMap::new();
        vars.insert("project_name".into(), name.clone());
        vars.insert("author".into(), author.clone());

        let result1 = template.render(&vars).unwrap();
        let result2 = template.render(&vars).unwrap();

        // Property: Deterministic rendering
        prop_assert_eq!(result1, result2);

        // Property: Variables substituted
        prop_assert!(!result1.contains("{{project_name}}"));
        prop_assert!(!result1.contains("{{author}}"));

        // Property: Values present
        prop_assert!(result1.contains(&name));
        prop_assert!(result1.contains(&author));
    }

    #[test]
    fn prop_generated_yaml_valid(
        name in valid_project_name(),
        author in "[A-Za-z ]{3,30}"
    ) {
        let template = Template::pforge_yaml();
        let mut vars = HashMap::new();
        vars.insert("project_name".into(), name);
        vars.insert("author".into(), author);
        vars.insert("description".into(), "Test agent".into());

        let rendered = template.render(&vars).unwrap();

        // Property: Rendered output is valid YAML
        let parsed: serde_yaml::Value = serde_yaml::from_str(&rendered).unwrap();
        prop_assert!(parsed.is_mapping());
    }
}
```

## Complexity Analysis

Target functions with their estimated complexity:
- `Template::render`: CC=3 (iteration + substitution + error)
- `Template::from_string`: CC=1 (simple construction)
- `TemplateRegistry::register`: CC=1 (insert)
- `TemplateRegistry::get`: CC=2 (lookup + error)
- `validate_template`: CC=4 (syntax checks)

All functions well under CC=10 threshold.

## Verification Commands

```bash
# Run unit tests
cargo test --lib scaffold::template_tests

# Run property tests
cargo test --lib scaffold::template_property_tests

# Check coverage
cargo llvm-cov --lib --lcov --output-path target/lcov.info
cargo llvm-cov report --summary-only

# Check complexity
pmat analyze complexity --path server/src/scaffold/template.rs --max-cyclomatic 10

# Check SATD
pmat analyze satd --path server/src/scaffold/template.rs --strict

# Run clippy
cargo clippy --all-targets -- -D warnings

# Integration test: Generate and build agent
cd /tmp
pmat scaffold agent --name test-pforge-agent --template pforge
cd test-pforge-agent
cargo build
cargo test
```

## Files to Create/Modify

### New Files
- `server/src/scaffold/template.rs` - Template system
- `server/src/scaffold/template_tests.rs` - Unit tests
- `server/src/scaffold/template_property_tests.rs` - Property tests
- `server/src/scaffold/templates/pforge.yaml.tmpl` - pforge.yaml template
- `server/src/scaffold/templates/Cargo.toml.tmpl` - Cargo.toml template
- `server/src/scaffold/templates/handler.rs.tmpl` - Handler template
- `server/src/scaffold/templates/mod.rs.tmpl` - Module template
- `server/src/scaffold/templates/README.md.tmpl` - README template
- `server/src/tests/template_integration_tests.rs` - Integration tests

### Modified Files
- `server/src/scaffold/mod.rs` - Export template module
- `server/src/scaffold/config.rs` - Add template-related config
- `Cargo.toml` - Add handlebars or tera dependency (templating)

## Template Engine Choice

**Option 1: Simple String Replace** (chosen for MVP)
- Pros: Zero dependencies, fast, simple, predictable
- Cons: Limited features, no conditionals/loops

**Option 2: Handlebars**
- Pros: Industry standard, rich features, helpers
- Cons: Additional dependency, more complex

**Option 3: Tera**
- Pros: Jinja2-like syntax, powerful, Rust-native
- Cons: Additional dependency

**Decision**: Start with Option 1 (string replace) for simplicity. Can upgrade to Handlebars/Tera in later tickets if needed.

## Risk Assessment

**Medium Risk:**
- Template syntax errors could generate invalid files
- Variable escaping issues

**Mitigation:**
- Comprehensive validation
- Property tests for valid output
- Integration tests that compile generated code

## Notes

This ticket establishes the template rendering foundation. Subsequent tickets will add:
- WASM templates (TICKET-PMAT-5003)
- Pre-commit hook generation (TICKET-PMAT-5005)
- CI/CD workflow templates

**TDD Cycle Duration**: Estimated 3-4 hours for RED → GREEN → REFACTOR

## Reference Implementations

- `../pforge/examples/` - pforge agent examples
- `../pforge/pforge.yaml` - pforge configuration format
- Handlebars templates: https://handlebarsjs.com/guide/
