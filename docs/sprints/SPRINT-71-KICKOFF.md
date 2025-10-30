# Sprint 71: DAP Server Foundation - KICKOFF

**Sprint**: 71
**Status**: Starting
**Date**: October 29, 2025
**Duration**: 1-2 weeks (estimated)
**Methodology**: EXTREME TDD (RED → GREEN → REFACTOR)

---

## Overview

Sprint 71 implements the **Debug Adapter Protocol (DAP) Server Foundation**, the first phase of PMAT's Interactive Tracing & Debugging capabilities (Spec Part 1).

**Parent Specification**: `docs/specifications/tracing-bug-discovery-tdg-git-expansion-spec.md`

---

## Sprint Goals

Implement the foundational DAP server infrastructure that enables:
1. ✅ Standard DAP protocol communication
2. ✅ Multi-language debugging support (17+ languages)
3. ✅ Breakpoint management system
4. ✅ Variable inspection with AST awareness
5. ✅ Integration with existing PMAT analyzers

**Success Criteria**:
- DAP server responds to standard protocol messages
- Can set/remove breakpoints in any supported language
- Variable inspection works with tree-sitter AST
- All tests passing with EXTREME TDD methodology
- <100ms average response latency

---

## Tickets (EXTREME TDD)

### TRACE-001: DAP Protocol Server Implementation

**Goal**: Implement core DAP server that handles standard protocol messages

**Phase**: RED (Write failing tests first)

**Test Requirements**:
```rust
#[test]
fn test_dap_server_initialization() {
    let server = DapServer::new();
    let init_request = json!({
        "seq": 1,
        "type": "request",
        "command": "initialize",
        "arguments": {
            "clientID": "vscode",
            "adapterID": "pmat-debug"
        }
    });

    let response = server.handle_request(init_request);
    assert_eq!(response["type"], "response");
    assert_eq!(response["command"], "initialize");
    assert_eq!(response["success"], true);
}

#[test]
fn test_dap_server_launch_request() {
    let mut server = DapServer::new();
    server.initialize();

    let launch_request = json!({
        "seq": 2,
        "type": "request",
        "command": "launch",
        "arguments": {
            "program": "test.py",
            "stopOnEntry": false
        }
    });

    let response = server.handle_request(launch_request);
    assert_eq!(response["success"], true);
}

#[test]
fn test_dap_server_handles_invalid_request() {
    let server = DapServer::new();
    let invalid = json!({"invalid": "request"});

    let response = server.handle_request(invalid);
    assert_eq!(response["success"], false);
    assert!(response["message"].as_str().unwrap().contains("invalid"));
}
```

**Implementation Skeleton**:
```rust
pub struct DapServer {
    state: ServerState,
    debugger: Option<Box<dyn Debugger>>,
    breakpoints: BreakpointManager,
}

impl DapServer {
    pub fn new() -> Self {
        Self {
            state: ServerState::Uninitialized,
            debugger: None,
            breakpoints: BreakpointManager::new(),
        }
    }

    pub fn handle_request(&mut self, request: Value) -> Value {
        // TODO: Parse request, route to handler, return response
        todo!()
    }
}
```

**Acceptance Criteria**:
- [ ] Handles `initialize` request
- [ ] Handles `launch` request
- [ ] Handles `setBreakpoints` request
- [ ] Handles `configurationDone` request
- [ ] Returns proper error responses
- [ ] All tests passing

**Time Estimate**: 4-6 hours

---

### TRACE-002: Breakpoint Management System

**Goal**: Implement breakpoint setting, removal, and validation across languages

**Phase**: RED (Write failing tests first)

**Test Requirements**:
```rust
#[test]
fn test_set_breakpoint_in_rust_file() {
    let mut mgr = BreakpointManager::new();
    let bp = Breakpoint {
        source: "src/main.rs".into(),
        line: 10,
        column: None,
        condition: None,
    };

    let result = mgr.set_breakpoint(bp.clone());
    assert!(result.is_ok());
    assert_eq!(mgr.count(), 1);
    assert!(mgr.has_breakpoint(&bp.source, bp.line));
}

#[test]
fn test_set_conditional_breakpoint() {
    let mut mgr = BreakpointManager::new();
    let bp = Breakpoint {
        source: "test.py".into(),
        line: 25,
        column: None,
        condition: Some("x > 10".to_string()),
    };

    mgr.set_breakpoint(bp.clone()).unwrap();
    let retrieved = mgr.get_breakpoint(&bp.source, bp.line).unwrap();
    assert_eq!(retrieved.condition, Some("x > 10".to_string()));
}

#[test]
fn test_remove_breakpoint() {
    let mut mgr = BreakpointManager::new();
    let bp = Breakpoint {
        source: "src/lib.rs".into(),
        line: 42,
        column: None,
        condition: None,
    };

    mgr.set_breakpoint(bp.clone()).unwrap();
    assert_eq!(mgr.count(), 1);

    mgr.remove_breakpoint(&bp.source, bp.line).unwrap();
    assert_eq!(mgr.count(), 0);
}

#[test]
fn test_validate_breakpoint_line_exists() {
    let mut mgr = BreakpointManager::new();
    let bp = Breakpoint {
        source: "tests/fixtures/sample.rs".into(),
        line: 5,
        column: None,
        condition: None,
    };

    let result = mgr.set_breakpoint(bp.clone());
    assert!(result.is_ok());

    // Invalid line (beyond file length)
    let invalid_bp = Breakpoint {
        source: "tests/fixtures/sample.rs".into(),
        line: 99999,
        column: None,
        condition: None,
    };

    let result = mgr.set_breakpoint(invalid_bp);
    assert!(result.is_err());
}

#[test]
fn test_list_breakpoints_for_file() {
    let mut mgr = BreakpointManager::new();

    mgr.set_breakpoint(Breakpoint {
        source: "a.rs".into(),
        line: 10,
        column: None,
        condition: None,
    }).unwrap();

    mgr.set_breakpoint(Breakpoint {
        source: "a.rs".into(),
        line: 20,
        column: None,
        condition: None,
    }).unwrap();

    mgr.set_breakpoint(Breakpoint {
        source: "b.rs".into(),
        line: 15,
        column: None,
        condition: None,
    }).unwrap();

    let breakpoints_a = mgr.list_for_file("a.rs");
    assert_eq!(breakpoints_a.len(), 2);

    let breakpoints_b = mgr.list_for_file("b.rs");
    assert_eq!(breakpoints_b.len(), 1);
}
```

**Implementation Skeleton**:
```rust
#[derive(Debug, Clone)]
pub struct Breakpoint {
    pub source: PathBuf,
    pub line: u32,
    pub column: Option<u32>,
    pub condition: Option<String>,
}

pub struct BreakpointManager {
    breakpoints: HashMap<PathBuf, Vec<Breakpoint>>,
}

impl BreakpointManager {
    pub fn new() -> Self {
        Self {
            breakpoints: HashMap::new(),
        }
    }

    pub fn set_breakpoint(&mut self, bp: Breakpoint) -> Result<(), String> {
        // TODO: Validate line exists in file
        // TODO: Store breakpoint
        todo!()
    }

    pub fn remove_breakpoint(&mut self, source: &Path, line: u32) -> Result<(), String> {
        todo!()
    }

    pub fn has_breakpoint(&self, source: &Path, line: u32) -> bool {
        todo!()
    }

    pub fn count(&self) -> usize {
        todo!()
    }
}
```

**Acceptance Criteria**:
- [ ] Set breakpoints in any supported language
- [ ] Remove breakpoints
- [ ] Conditional breakpoints work
- [ ] Validates line numbers exist
- [ ] Lists breakpoints per file
- [ ] All tests passing

**Time Estimate**: 3-4 hours

---

### TRACE-003: Variable Inspection with AST

**Goal**: Inspect variables with AST-aware formatting using tree-sitter

**Phase**: RED (Write failing tests first)

**Test Requirements**:
```rust
#[test]
fn test_inspect_variable_simple_type() {
    let inspector = VariableInspector::new();
    let source = r#"
        fn main() {
            let x = 42;
            let name = "Alice";
        }
    "#;

    let ast = parse_rust(source).unwrap();
    let scope = Scope::at_line(&ast, 3); // Line with variables

    let x_var = inspector.inspect("x", &scope).unwrap();
    assert_eq!(x_var.name, "x");
    assert_eq!(x_var.type_info, "i32");
    assert_eq!(x_var.value, "42");

    let name_var = inspector.inspect("name", &scope).unwrap();
    assert_eq!(name_var.name, "name");
    assert_eq!(name_var.type_info, "&str");
    assert_eq!(name_var.value, "\"Alice\"");
}

#[test]
fn test_inspect_variable_complex_type() {
    let inspector = VariableInspector::new();
    let source = r#"
        struct Person {
            name: String,
            age: u32,
        }

        fn main() {
            let person = Person {
                name: "Bob".to_string(),
                age: 30,
            };
        }
    "#;

    let ast = parse_rust(source).unwrap();
    let scope = Scope::at_line(&ast, 9);

    let person_var = inspector.inspect("person", &scope).unwrap();
    assert_eq!(person_var.name, "person");
    assert_eq!(person_var.type_info, "Person");
    assert_eq!(person_var.children.len(), 2);

    assert_eq!(person_var.children[0].name, "name");
    assert_eq!(person_var.children[0].value, "\"Bob\"");

    assert_eq!(person_var.children[1].name, "age");
    assert_eq!(person_var.children[1].value, "30");
}

#[test]
fn test_inspect_variable_not_in_scope() {
    let inspector = VariableInspector::new();
    let source = r#"
        fn foo() {
            let x = 42;
        }

        fn bar() {
            // x not in scope here
        }
    "#;

    let ast = parse_rust(source).unwrap();
    let scope = Scope::at_line(&ast, 6); // Inside bar()

    let result = inspector.inspect("x", &scope);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not in scope"));
}

#[test]
fn test_inspect_variable_python() {
    let inspector = VariableInspector::new();
    let source = r#"
def main():
    x = 42
    names = ["Alice", "Bob"]
    data = {"key": "value"}
"#;

    let ast = parse_python(source).unwrap();
    let scope = Scope::at_line(&ast, 3);

    let x_var = inspector.inspect("x", &scope).unwrap();
    assert_eq!(x_var.type_info, "int");

    let names_var = inspector.inspect("names", &scope).unwrap();
    assert_eq!(names_var.type_info, "list");
    assert_eq!(names_var.children.len(), 2);

    let data_var = inspector.inspect("data", &scope).unwrap();
    assert_eq!(data_var.type_info, "dict");
}

#[test]
fn test_format_variable_for_display() {
    let var = Variable {
        name: "person".to_string(),
        type_info: "Person".to_string(),
        value: String::new(),
        children: vec![
            Variable {
                name: "name".to_string(),
                type_info: "String".to_string(),
                value: "\"Alice\"".to_string(),
                children: vec![],
            },
            Variable {
                name: "age".to_string(),
                type_info: "u32".to_string(),
                value: "30".to_string(),
                children: vec![],
            },
        ],
    };

    let formatted = format_variable(&var, 0);
    let expected = "person: Person\n  name: String = \"Alice\"\n  age: u32 = 30";
    assert_eq!(formatted, expected);
}
```

**Implementation Skeleton**:
```rust
#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub type_info: String,
    pub value: String,
    pub children: Vec<Variable>,
}

pub struct VariableInspector {
    analyzers: HashMap<Language, Box<dyn LanguageAnalyzer>>,
}

impl VariableInspector {
    pub fn new() -> Self {
        // TODO: Register language analyzers
        todo!()
    }

    pub fn inspect(&self, name: &str, scope: &Scope) -> Result<Variable, String> {
        // TODO: Find variable in scope
        // TODO: Analyze type using tree-sitter AST
        // TODO: Format value
        // TODO: Handle complex types recursively
        todo!()
    }
}

pub struct Scope {
    variables: HashMap<String, Variable>,
    parent: Option<Box<Scope>>,
}

impl Scope {
    pub fn at_line(ast: &Tree, line: u32) -> Self {
        // TODO: Build scope from AST at given line
        todo!()
    }
}
```

**Acceptance Criteria**:
- [ ] Inspect simple types (int, str, bool)
- [ ] Inspect complex types (structs, classes)
- [ ] Handles nested structures
- [ ] Multi-language support (Rust, Python, TypeScript)
- [ ] Scope awareness (variables not in scope = error)
- [ ] Pretty formatting for display
- [ ] All tests passing

**Time Estimate**: 6-8 hours

---

### TRACE-004: DAP-PMAT Integration

**Goal**: Integrate DAP server with existing PMAT infrastructure

**Phase**: RED (Write failing tests first)

**Test Requirements**:
```rust
#[test]
fn test_dap_uses_pmat_language_analyzer() {
    let mut server = DapServer::new();

    // Launch with a Rust file
    let launch_request = json!({
        "seq": 2,
        "type": "request",
        "command": "launch",
        "arguments": {
            "program": "tests/fixtures/sample.rs"
        }
    });

    server.handle_request(launch_request);

    // Should detect Rust and use PMAT's Rust analyzer
    assert_eq!(server.current_language(), Some(Language::Rust));
}

#[test]
fn test_dap_breakpoint_uses_tree_sitter() {
    let mut server = DapServer::new();
    server.launch("tests/fixtures/sample.py");

    // Set breakpoint
    let bp_request = json!({
        "seq": 3,
        "type": "request",
        "command": "setBreakpoints",
        "arguments": {
            "source": { "path": "tests/fixtures/sample.py" },
            "breakpoints": [{ "line": 5 }]
        }
    });

    let response = server.handle_request(bp_request);
    assert_eq!(response["success"], true);

    // Should have parsed file with tree-sitter
    assert!(server.has_ast_for("tests/fixtures/sample.py"));
}

#[test]
fn test_dap_variable_inspection_uses_pmat_context() {
    let mut server = DapServer::new();
    server.launch_and_stop_at_line("tests/fixtures/complex.rs", 10);

    // Request variable scopes
    let scopes_request = json!({
        "seq": 5,
        "type": "request",
        "command": "scopes",
        "arguments": { "frameId": 0 }
    });

    let response = server.handle_request(scopes_request);
    let scopes = &response["body"]["scopes"];

    assert!(scopes.as_array().unwrap().len() > 0);

    // Request variables (should use PMAT's deep context)
    let vars_request = json!({
        "seq": 6,
        "type": "request",
        "command": "variables",
        "arguments": { "variablesReference": 1 }
    });

    let response = server.handle_request(vars_request);
    let variables = &response["body"]["variables"];

    // Should have rich type information from PMAT
    assert!(variables.as_array().unwrap().len() > 0);
}

#[test]
fn test_end_to_end_debugging_session() {
    let mut server = DapServer::new();

    // 1. Initialize
    server.handle_request(json!({
        "seq": 1,
        "command": "initialize",
        "arguments": { "clientID": "test" }
    }));

    // 2. Launch program
    let launch_response = server.handle_request(json!({
        "seq": 2,
        "command": "launch",
        "arguments": { "program": "tests/fixtures/sample.rs" }
    }));
    assert_eq!(launch_response["success"], true);

    // 3. Set breakpoint
    let bp_response = server.handle_request(json!({
        "seq": 3,
        "command": "setBreakpoints",
        "arguments": {
            "source": { "path": "tests/fixtures/sample.rs" },
            "breakpoints": [{ "line": 5 }]
        }
    }));
    assert_eq!(bp_response["success"], true);

    // 4. Configuration done
    server.handle_request(json!({
        "seq": 4,
        "command": "configurationDone"
    }));

    // 5. Continue (simulate hitting breakpoint)
    server.handle_request(json!({
        "seq": 5,
        "command": "continue",
        "arguments": { "threadId": 1 }
    }));

    // Should emit stopped event at breakpoint
    let events = server.get_events();
    assert!(events.iter().any(|e| e["event"] == "stopped"));
}
```

**Implementation Requirements**:
- Integrate with existing `LanguageAnalyzer` trait
- Use tree-sitter parsers already in PMAT
- Leverage deep context generation for rich variable info
- Connect to TDG analyzer for code quality annotations

**Acceptance Criteria**:
- [ ] Uses PMAT language detection
- [ ] Uses tree-sitter for AST parsing
- [ ] Variable inspection shows rich type info
- [ ] End-to-end debugging session works
- [ ] All tests passing

**Time Estimate**: 4-6 hours

---

## Development Workflow

### EXTREME TDD Process

**For each ticket**:

1. **RED Phase** (Write Failing Tests)
   ```bash
   # Create test file
   touch server/tests/dap_server_tests.rs

   # Write failing tests (all marked #[test])
   cargo test dap_server_tests
   # Should see: X tests failed
   ```

2. **GREEN Phase** (Minimal Implementation)
   ```bash
   # Implement minimal code to pass tests
   # Focus on making tests pass, not perfection
   cargo test dap_server_tests
   # Should see: X tests passed
   ```

3. **REFACTOR Phase** (Improve Code Quality)
   ```bash
   # Clean up implementation
   # Extract helpers, improve names, add docs
   cargo clippy
   cargo fmt
   cargo test  # Ensure still passing
   ```

4. **COMMIT Phase** (Document Progress)
   ```bash
   git add .
   git commit -m "feat: TRACE-001 GREEN phase - DAP server initialization"
   ```

### Quality Gates

**Before moving to next ticket**:
- [ ] All tests passing (100%)
- [ ] No clippy warnings
- [ ] Code formatted (cargo fmt)
- [ ] Documentation added
- [ ] Performance validated (<100ms target)

---

## File Structure

```
server/
├── src/
│   ├── debug/              # New module
│   │   ├── mod.rs          # Module definition
│   │   ├── dap_server.rs   # TRACE-001
│   │   ├── breakpoints.rs  # TRACE-002
│   │   ├── inspector.rs    # TRACE-003
│   │   └── integration.rs  # TRACE-004
│   └── lib.rs              # Add debug module
├── tests/
│   ├── dap_server_tests.rs      # TRACE-001 tests
│   ├── breakpoint_tests.rs      # TRACE-002 tests
│   ├── variable_inspector_tests.rs  # TRACE-003 tests
│   └── dap_integration_tests.rs # TRACE-004 tests
└── tests/fixtures/
    ├── sample.rs           # Test fixture
    ├── sample.py           # Test fixture
    └── complex.rs          # Complex test case
```

---

## Success Metrics

**Code Quality**:
- Test coverage: >85%
- All tests passing (RED → GREEN → REFACTOR)
- Zero clippy warnings
- Performance: <100ms average latency

**Functionality**:
- DAP protocol compatibility (VS Code, vim)
- Multi-language support (Rust, Python, TypeScript)
- Breakpoint management working
- Variable inspection accurate

**Documentation**:
- All public APIs documented
- Test cases self-documenting
- Integration guide written

---

## Timeline

**Estimated Duration**: 1-2 weeks

| Ticket | Estimate | Status |
|--------|----------|--------|
| TRACE-001 | 4-6 hours | Pending |
| TRACE-002 | 3-4 hours | Pending |
| TRACE-003 | 6-8 hours | Pending |
| TRACE-004 | 4-6 hours | Pending |
| **Total** | **17-24 hours** | **0% Complete** |

**Daily Progress Target**: 3-4 hours focused work

---

## Next Steps

1. ✅ Review kickoff document
2. ⏳ Create test fixtures
3. ⏳ Begin TRACE-001 (RED phase)
4. ⏳ Iterate through tickets with EXTREME TDD

---

**Status**: Ready to begin TRACE-001
**Methodology**: EXTREME TDD (RED → GREEN → REFACTOR → COMMIT)
**Quality Standard**: Toyota Way - Zero defects, continuous improvement
