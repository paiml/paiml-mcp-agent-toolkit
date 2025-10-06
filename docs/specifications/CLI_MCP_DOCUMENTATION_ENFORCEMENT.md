# Specification: CLI and MCP Documentation Enforcement

**Type:** Quality Gate Specification
**Priority:** P0 - Critical
**Status:** 🔴 RED (Specification Phase)
**Created:** 2025-10-06

---

## Problem Statement

PMAT has excellent code quality enforcement (complexity, coverage, SATD), but **zero enforcement** for CLI and MCP interface documentation. This creates critical gaps:

### Current Issues

1. **CLI Commands Undocumented:**
   - Flags exist in code but not in help text
   - Examples missing or outdated
   - No enforcement that `--help` matches reality

2. **MCP Tools Undocumented:**
   - JSON schema exists but no human-readable docs
   - Parameter descriptions missing or generic
   - No examples showing actual usage

3. **Documentation Drift:**
   - Code changes but docs don't update
   - New flags added without documentation
   - Renamed parameters break examples

4. **No Automated Verification:**
   - Can't verify CLI help is complete
   - Can't verify MCP schemas match reality
   - Can't catch missing documentation in CI

### Real-World Impact

**Example 1: Missing Flag Documentation**
```rust
// Code has this flag
#[arg(long)]
generate_tickets: bool,

// But `pmat maintain roadmap --help` doesn't mention it!
```

**Example 2: MCP Schema vs Reality**
```rust
// MCP schema says:
"name": {
    "type": "string",
    "description": "Project name"  // Generic!
}

// Should say:
"name": {
    "type": "string",
    "description": "Agent project name (lowercase, alphanumeric, hyphens only)"
}
```

**Example 3: Outdated Examples**
```bash
# Documentation shows:
pmat scaffold agent my-agent --quality extreme

# But flag is now:
pmat scaffold agent my-agent --quality-level extreme
```

---

## Proposed Solution

Create a **Documentation Enforcement System** that:

1. **Validates CLI Documentation** using `assert_cmd`
2. **Validates MCP Documentation** using schema analysis
3. **Runs in CI/CD** as a quality gate
4. **Fails builds** on missing/incorrect documentation
5. **Provides clear error messages** on what's missing

---

## Scope

### In Scope

1. **CLI Command Verification:**
   - All commands have `--help` text
   - All flags documented in help
   - Help text matches clap definitions
   - Examples exist and are valid
   - Description is non-empty

2. **MCP Tool Verification:**
   - All tools have descriptions
   - All parameters documented
   - Parameter descriptions are descriptive (not generic)
   - JSON schema matches actual parameters
   - Examples provided in docs

3. **Automated Testing:**
   - `assert_cmd` tests for CLI help
   - Schema validation tests for MCP
   - Runs in pre-commit hook
   - Runs in CI/CD

4. **Quality Gate Integration:**
   - Add to `pmat maintain health`
   - Add to pre-commit hook
   - Clear error reporting

### Out of Scope

1. API documentation enforcement (REST/HTTP)
2. Code comment enforcement (covered by existing tools)
3. README/user guide enforcement (manual review)
4. Internationalization/localization

---

## Requirements

### FR-1: CLI Help Text Validation

**Requirement:** Every CLI command must have complete, accurate help text.

**Success Criteria:**
- `--help` flag works for all commands
- All flags appear in help output
- Descriptions are non-empty (>10 chars)
- Examples section exists
- Help matches clap definitions

**Test Specification:**
```rust
#[test]
fn test_all_commands_have_help() {
    for command in ALL_COMMANDS {
        Command::cargo_bin("pmat")
            .unwrap()
            .args(command.split_whitespace())
            .arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("Usage:"));
    }
}
```

---

### FR-2: CLI Flag Documentation Completeness

**Requirement:** All CLI flags must be documented in help text.

**Success Criteria:**
- Every `#[arg(long)]` appears in help
- Every `#[arg(short)]` appears in help
- Default values shown for optional flags
- Required vs optional clearly marked

**Test Specification:**
```rust
#[test]
fn test_maintain_roadmap_flags_documented() {
    let output = Command::cargo_bin("pmat")
        .unwrap()
        .args(["maintain", "roadmap", "--help"])
        .output()
        .unwrap();

    let help = String::from_utf8(output.stdout).unwrap();

    // All flags must appear
    assert!(help.contains("--validate"), "Missing --validate flag");
    assert!(help.contains("--health"), "Missing --health flag");
    assert!(help.contains("--fix"), "Missing --fix flag");
    assert!(help.contains("--generate-tickets"), "Missing --generate-tickets flag");
    assert!(help.contains("--dry-run"), "Missing --dry-run flag");

    // Descriptions must be present
    assert!(help.contains("Validate roadmap"), "Missing description for --validate");
}
```

---

### FR-3: CLI Example Validation

**Requirement:** CLI help must include valid, runnable examples.

**Success Criteria:**
- Examples section exists
- Examples use correct syntax
- Examples can actually run (dry-run mode)
- Examples cover common use cases

**Test Specification:**
```rust
#[test]
fn test_help_examples_are_valid() {
    // Extract examples from help text
    let help = get_command_help("pmat maintain roadmap");
    let examples = extract_examples(&help);

    // Each example should parse and validate
    for example in examples {
        let args = parse_example_args(example);
        // This should not error (even in dry-run)
        assert!(validate_args(&args).is_ok());
    }
}
```

---

### FR-4: MCP Tool Description Validation

**Requirement:** All MCP tools must have comprehensive descriptions.

**Success Criteria:**
- Tool description exists
- Description is >20 characters
- Description explains what the tool does
- Description mentions key parameters

**Test Specification:**
```rust
#[test]
fn test_mcp_tools_have_descriptions() {
    let tools = get_all_mcp_tools();

    for tool in tools {
        assert!(!tool.description.is_empty(),
            "Tool {} has no description", tool.name);

        assert!(tool.description.len() > 20,
            "Tool {} description too short: '{}'",
            tool.name, tool.description);

        // Description should not be generic
        assert!(!is_generic_description(&tool.description),
            "Tool {} has generic description", tool.name);
    }
}
```

---

### FR-5: MCP Parameter Documentation

**Requirement:** All MCP parameters must have descriptive documentation.

**Success Criteria:**
- Every parameter has description
- Description is >15 characters
- Description explains parameter purpose
- Type information is accurate
- Required vs optional is clear

**Test Specification:**
```rust
#[test]
fn test_scaffold_agent_parameters_documented() {
    let tool = get_mcp_tool("scaffold_agent");

    // Required parameter: name
    let name_param = tool.input_schema
        .properties
        .get("name")
        .expect("Missing 'name' parameter");

    assert!(name_param.description.len() > 15,
        "Parameter 'name' description too short");

    assert!(!is_generic_description(&name_param.description),
        "Parameter 'name' has generic description");

    // Should explain constraints
    assert!(name_param.description.contains("lowercase") ||
            name_param.description.contains("alphanumeric"),
        "Parameter 'name' missing constraint information");
}
```

---

### FR-6: Generic Description Detection

**Requirement:** System must detect and reject generic/placeholder descriptions.

**Generic Descriptions (FORBIDDEN):**
- "The X parameter"
- "X value"
- "Input for X"
- "Project name" (too vague)
- "Path to file" (too vague)

**Good Descriptions:**
- "Agent project name (lowercase, alphanumeric, hyphens only)"
- "Path to ROADMAP.md file (default: ./ROADMAP.md)"
- "Quality level: standard (fast), high (thorough), extreme (comprehensive with ML)"

**Test Specification:**
```rust
#[test]
fn test_reject_generic_descriptions() {
    let generic = vec![
        "The name parameter",
        "Project name",
        "Input value",
        "Path to file",
    ];

    for desc in generic {
        assert!(is_generic_description(desc),
            "Failed to detect generic: '{}'", desc);
    }

    let good = vec![
        "Agent project name (lowercase, alphanumeric, hyphens only)",
        "Quality level: standard (fast), high (thorough), extreme (comprehensive)",
    ];

    for desc in good {
        assert!(!is_generic_description(desc),
            "Incorrectly flagged as generic: '{}'", desc);
    }
}
```

---

### FR-7: Documentation Drift Detection

**Requirement:** System must detect when code changes but docs don't update.

**Success Criteria:**
- Parameter rename detected
- New parameter without docs detected
- Removed parameter still documented detected

**Test Specification:**
```rust
#[test]
fn test_detect_undocumented_new_parameters() {
    // Get actual clap definitions
    let cli_params = extract_cli_parameters();

    // Get help text parameters
    let help_params = extract_help_parameters();

    // Every CLI param must be in help
    for param in cli_params {
        assert!(help_params.contains(&param),
            "Parameter '{}' exists in code but not documented in help",
            param);
    }
}
```

---

### FR-8: Quality Gate Integration

**Requirement:** Documentation checks must run as quality gate.

**Success Criteria:**
- Runs in `pmat maintain health --all`
- Runs in pre-commit hook
- Fails with clear error messages
- Lists all missing documentation
- Exit code 1 on failure

**Test Specification:**
```rust
#[test]
fn test_quality_gate_fails_on_missing_docs() {
    // Temporarily remove a flag description
    let result = Command::cargo_bin("pmat")
        .unwrap()
        .args(["maintain", "health", "--check-docs"])
        .assert()
        .failure(); // Should fail

    let output = result.get_output();
    assert!(output.stderr.contains("Missing documentation"),
        "Should report missing documentation");
}
```

---

## Architecture

### Component Design

```
┌─────────────────────────────────────┐
│   Documentation Enforcement         │
│   (New Module)                      │
└──────────────┬──────────────────────┘
               │
       ┌───────┴───────┐
       │               │
       ▼               ▼
┌─────────────┐  ┌──────────────┐
│ CLI Checker │  │ MCP Checker  │
└─────────────┘  └──────────────┘
       │               │
       ▼               ▼
┌─────────────┐  ┌──────────────┐
│ assert_cmd  │  │ Schema Parse │
│ tests       │  │ tests        │
└─────────────┘  └──────────────┘
       │               │
       └───────┬───────┘
               ▼
┌─────────────────────────────────────┐
│   Quality Gate                      │
│   pmat maintain health --check-docs │
└─────────────────────────────────────┘
```

### File Structure

```
server/
├── src/
│   ├── quality/
│   │   ├── docs_enforcement/
│   │   │   ├── mod.rs              # Main module
│   │   │   ├── cli_checker.rs      # CLI documentation checker
│   │   │   ├── mcp_checker.rs      # MCP documentation checker
│   │   │   ├── generic_detector.rs # Generic description detection
│   │   │   └── reporter.rs         # Error reporting
│   │   └── gates.rs                # Integration point
│   └── cli/
│       └── handlers/
│           └── docs_enforcement_handler.rs  # CLI handler
└── tests/
    ├── cli_docs_enforcement.rs     # CLI tests (assert_cmd)
    └── mcp_docs_enforcement.rs     # MCP tests
```

---

## Implementation Plan (EXTREME TDD)

### Phase 1: RED - Write Failing Tests

**Week 1:**
1. Create test structure
2. Write all `#[test]` functions (they will fail)
3. Define test helpers and utilities
4. Document expected failures

**Deliverables:**
- `tests/cli_docs_enforcement.rs` (all RED)
- `tests/mcp_docs_enforcement.rs` (all RED)
- Test helper functions

**Success Criteria:**
- All tests compile
- All tests fail with clear messages
- Test coverage plan at 100%

---

### Phase 2: GREEN - Implement Enforcement

**Week 2:**
1. Implement `cli_checker.rs`
2. Implement `mcp_checker.rs`
3. Implement `generic_detector.rs`
4. Make tests pass one by one

**Deliverables:**
- Working CLI documentation checker
- Working MCP documentation checker
- Generic description detector
- All tests passing

**Success Criteria:**
- All tests GREEN
- Code complexity <8
- Documentation complete

---

### Phase 3: REFACTOR - Optimize & Integrate

**Week 3:**
1. Refactor for clarity
2. Integrate with quality gates
3. Add to pre-commit hook
4. Update documentation

**Deliverables:**
- Clean, maintainable code
- Quality gate integration
- Pre-commit hook update
- User documentation

---

## Test Specifications (EXTREME TDD)

### Test File 1: `tests/cli_docs_enforcement.rs`

```rust
//! CLI documentation enforcement tests
//!
//! Phase: RED
//! All tests should FAIL until implementation complete

use assert_cmd::Command;
use predicates::prelude::*;

/// Test: All commands have --help
#[test]
fn red_test_all_commands_have_help() {
    let commands = vec![
        "analyze complexity",
        "analyze satd",
        "analyze dead-code",
        "maintain health",
        "maintain roadmap",
        "scaffold agent",
    ];

    for cmd in commands {
        Command::cargo_bin("pmat")
            .unwrap()
            .args(cmd.split_whitespace())
            .arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("Usage:"));
    }
}

/// Test: maintain roadmap flags all documented
#[test]
fn red_test_maintain_roadmap_flags_complete() {
    let output = Command::cargo_bin("pmat")
        .unwrap()
        .args(["maintain", "roadmap", "--help"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let help = String::from_utf8(output).unwrap();

    assert!(help.contains("--validate"));
    assert!(help.contains("--health"));
    assert!(help.contains("--fix"));
    assert!(help.contains("--generate-tickets"));
    assert!(help.contains("--dry-run"));
    assert!(help.contains("--format"));
}

/// Test: Help text has descriptions (not just flag names)
#[test]
fn red_test_help_has_descriptions() {
    let output = Command::cargo_bin("pmat")
        .unwrap()
        .args(["maintain", "roadmap", "--help"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let help = String::from_utf8(output).unwrap();

    // Should have actual descriptions, not just "--validate"
    assert!(help.contains("Validate roadmap") ||
            help.contains("Check roadmap"),
        "Missing description for --validate");
}

/// Test: Help includes examples
#[test]
fn red_test_help_includes_examples() {
    let output = Command::cargo_bin("pmat")
        .unwrap()
        .args(["scaffold", "agent", "--help"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let help = String::from_utf8(output).unwrap();

    assert!(help.contains("EXAMPLE") ||
            help.contains("Example") ||
            help.contains("example"),
        "Help should include examples section");
}

/// Test: No generic descriptions like "The X parameter"
#[test]
fn red_test_no_generic_descriptions() {
    let output = Command::cargo_bin("pmat")
        .unwrap()
        .args(["scaffold", "agent", "--help"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let help = String::from_utf8(output).unwrap();

    // Should NOT contain generic patterns
    assert!(!help.contains("The name parameter"),
        "Found generic description");
    assert!(!help.contains("Input value"),
        "Found generic description");
}
```

### Test File 2: `tests/mcp_docs_enforcement.rs`

```rust
//! MCP documentation enforcement tests
//!
//! Phase: RED
//! All tests should FAIL until implementation complete

use pmat::contracts::mcp_impl::*;
use serde_json::Value;

/// Test: All MCP tools have descriptions
#[test]
fn red_test_all_mcp_tools_have_descriptions() {
    let tools = get_all_mcp_tool_definitions();

    for tool in tools {
        assert!(!tool.description.is_empty(),
            "Tool '{}' has no description", tool.name);

        assert!(tool.description.len() > 20,
            "Tool '{}' description too short: '{}'",
            tool.name, tool.description);
    }
}

/// Test: MCP parameters have descriptive docs
#[test]
fn red_test_mcp_parameters_descriptive() {
    let tool = get_mcp_tool_definition("scaffold_agent");

    let schema = tool.input_schema;
    let props = schema["properties"].as_object()
        .expect("Schema should have properties");

    for (param_name, param_spec) in props {
        let desc = param_spec["description"].as_str()
            .expect(&format!("Parameter '{}' missing description", param_name));

        assert!(desc.len() > 15,
            "Parameter '{}' description too short: '{}'",
            param_name, desc);

        assert!(!is_generic_description(desc),
            "Parameter '{}' has generic description: '{}'",
            param_name, desc);
    }
}

/// Test: Detect generic descriptions
#[test]
fn red_test_generic_description_detector() {
    let generic = vec![
        "The name parameter",
        "Project name",
        "Input value",
        "Path to file",
        "The template",
    ];

    for desc in generic {
        assert!(is_generic_description(desc),
            "Failed to detect generic: '{}'", desc);
    }

    let good = vec![
        "Agent project name (lowercase, alphanumeric, hyphens only)",
        "Quality level: standard (fast), high (thorough), extreme (comprehensive)",
        "Path to ROADMAP.md file (default: ./ROADMAP.md)",
    ];

    for desc in good {
        assert!(!is_generic_description(desc),
            "Incorrectly flagged as generic: '{}'", desc);
    }
}

// Helper function to detect generic descriptions
fn is_generic_description(desc: &str) -> bool {
    let generic_patterns = vec![
        "The ",
        " parameter",
        " value",
        "Input for",
        "Output for",
    ];

    // Too short is generic
    if desc.len() < 15 {
        return true;
    }

    // Contains generic patterns
    for pattern in generic_patterns {
        if desc.starts_with(pattern) || desc.contains(pattern) {
            return true;
        }
    }

    // No specific information (just noun)
    let words = desc.split_whitespace().count();
    if words < 5 {
        return true;
    }

    false
}

// Helper to get MCP tool definitions
fn get_all_mcp_tool_definitions() -> Vec<ToolDefinition> {
    // This will be implemented to extract tool definitions
    vec![]
}

fn get_mcp_tool_definition(name: &str) -> ToolDefinition {
    // This will be implemented
    unimplemented!()
}
```

---

## Acceptance Criteria

### Phase 1: RED Tests (Week 1)

- [ ] All test files compile
- [ ] All tests fail (RED)
- [ ] Test coverage: CLI commands
- [ ] Test coverage: MCP tools
- [ ] Test coverage: Generic detection
- [ ] Documentation of expected failures

### Phase 2: GREEN Implementation (Week 2)

- [ ] All tests pass (GREEN)
- [ ] CLI checker implemented
- [ ] MCP checker implemented
- [ ] Generic detector implemented
- [ ] Code complexity <8
- [ ] Documentation complete

### Phase 3: REFACTOR & Integration (Week 3)

- [ ] Code refactored for clarity
- [ ] Integrated with `pmat maintain health`
- [ ] Integrated with pre-commit hook
- [ ] User documentation updated
- [ ] Examples provided

### Production Ready

- [ ] All tests GREEN
- [ ] Quality gate working
- [ ] Pre-commit hook working
- [ ] CI/CD integration
- [ ] Zero false positives

---

## Success Metrics

### Coverage Targets

- **CLI Commands:** 100% documented
- **MCP Tools:** 100% documented
- **Test Coverage:** 100% of documentation checks
- **Generic Detection:** <5% false positive rate

### Performance Targets

- **CLI Check Time:** <1 second
- **MCP Check Time:** <500ms
- **Total Check Time:** <2 seconds

### Quality Targets

- **False Positives:** <5%
- **False Negatives:** 0%
- **Code Complexity:** <8 for all functions

---

## Future Enhancements

### Phase 4: Advanced Features (Future)

1. **Auto-Generation:**
   - Generate help text from docstrings
   - Generate MCP schemas from types
   - Update docs automatically

2. **Documentation Quality:**
   - Check for typos
   - Check for broken links
   - Check for outdated examples

3. **Cross-Reference:**
   - Verify examples in README match CLI
   - Verify MCP examples match actual usage
   - Verify changelog mentions new flags

---

## Conclusion

This specification defines a comprehensive documentation enforcement system for PMAT's CLI and MCP interfaces. Using EXTREME TDD, we will:

1. **RED Phase:** Write all failing tests
2. **GREEN Phase:** Implement enforcement system
3. **REFACTOR Phase:** Optimize and integrate

**Status:** 🔴 RED (Specification Complete, Ready for Implementation)
**Next Step:** Begin RED phase - write all failing tests

---

*Specification Created: October 6, 2025*
*Type: Quality Gate Enhancement*
*Methodology: EXTREME TDD*
