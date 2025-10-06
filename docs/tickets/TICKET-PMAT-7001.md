# TICKET-PMAT-7001: CLI and MCP Documentation Enforcement

**Sprint:** Sprint 105 - Documentation Enforcement
**Priority:** P2 - Quality Infrastructure
**Estimated Effort:** 15 hours (3 phases over 1 day)
**Status**: ✅ **COMPLETE** (All 3 Phases: RED → GREEN → REFACTOR)
**Created:** 2025-10-06
**Completed:** 2025-10-06
**Release:** v2.142.0

---

## Problem Statement

PMAT has excellent code quality enforcement (complexity <8, coverage >85%, SATD detection), but **zero enforcement** for CLI and MCP interface documentation. This creates critical quality gaps:

### Current Issues

1. **CLI Flags Undocumented:**
   ```rust
   // Code has this flag
   #[arg(long)]
   generate_tickets: bool,

   // But `pmat maintain roadmap --help` doesn't mention it!
   ```

2. **MCP Parameters Poorly Documented:**
   ```json
   {
     "name": {
       "type": "string",
       "description": "Project name"  // Too generic!
     }
   }

   // Should be:
   {
     "name": {
       "type": "string",
       "description": "Agent project name (lowercase, alphanumeric, hyphens only)"
     }
   }
   ```

3. **Documentation Drift:**
   - Code changes but docs don't update
   - New flags added without documentation
   - Outdated examples in help text

4. **No Automated Verification:**
   - Can't verify CLI help completeness in CI
   - Can't catch missing MCP docs in pre-commit
   - Manual review is error-prone

### Real-World Impact

**Example from PMAT-6012:**
- Added `--generate-tickets` flag
- Forgot to update help text
- Users had to read source code to discover feature

**Example from PMAT-6017:**
- MCP tool has `quality_level` parameter
- Description says "Quality level" (generic!)
- Should explain: standard/high/extreme with tradeoffs

---

## Solution

Build a **Documentation Enforcement System** using EXTREME TDD:

### Phase 1: RED (Week 1) 🔴
Write all failing tests before any implementation.

**Deliverables:**
- `tests/cli_docs_enforcement.rs` - All tests RED
- `tests/mcp_docs_enforcement.rs` - All tests RED
- Test helper functions
- Documentation of expected failures

### Phase 2: GREEN (Week 2) 🟢
Implement enforcement system to make tests pass.

**Deliverables:**
- `server/src/quality/docs_enforcement/` module
- CLI documentation checker
- MCP documentation checker
- Generic description detector
- All tests passing

### Phase 3: REFACTOR (Week 3) 🔵
Optimize and integrate with quality gates.

**Deliverables:**
- Integrated with `pmat maintain health`
- Integrated with pre-commit hook
- User documentation
- CI/CD integration

---

## Requirements

### FR-1: CLI Help Text Validation

**Requirement:** Every CLI command must have complete help text.

**Success Criteria:**
- [ ] `--help` works for all commands
- [ ] All flags appear in help output
- [ ] Descriptions >10 characters
- [ ] Examples section exists
- [ ] Help matches clap definitions

**Test:**
```rust
#[test]
fn red_test_all_commands_have_help() {
    let commands = vec![
        "analyze complexity",
        "maintain health",
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
```

---

### FR-2: CLI Flag Documentation Completeness

**Requirement:** All CLI flags documented in help text.

**Success Criteria:**
- [ ] Every `#[arg(long)]` in help
- [ ] Every `#[arg(short)]` in help
- [ ] Default values shown
- [ ] Required vs optional marked

**Test:**
```rust
#[test]
fn red_test_maintain_roadmap_flags_complete() {
    let help = get_help("maintain roadmap");

    assert!(help.contains("--validate"));
    assert!(help.contains("--generate-tickets"));
    assert!(help.contains("--dry-run"));

    // Descriptions present
    assert!(help.contains("Validate roadmap"));
}
```

---

### FR-3: MCP Tool Description Validation

**Requirement:** All MCP tools have comprehensive descriptions.

**Success Criteria:**
- [ ] Tool description exists
- [ ] Description >20 characters
- [ ] Not generic (e.g., "Tool for X")
- [ ] Mentions key parameters

**Test:**
```rust
#[test]
fn red_test_mcp_tools_descriptive() {
    let tools = get_all_mcp_tools();

    for tool in tools {
        assert!(tool.description.len() > 20);
        assert!(!is_generic_description(&tool.description));
    }
}
```

---

### FR-4: Generic Description Detection

**Requirement:** Detect and reject generic descriptions.

**Generic (Forbidden):**
- "The name parameter"
- "Project name" (alone)
- "Input value"
- "Path to file"

**Good:**
- "Agent project name (lowercase, alphanumeric, hyphens only)"
- "Path to ROADMAP.md file (default: ./ROADMAP.md)"

**Test:**
```rust
#[test]
fn red_test_generic_detection() {
    assert!(is_generic_description("The name parameter"));
    assert!(is_generic_description("Project name"));

    assert!(!is_generic_description(
        "Agent project name (lowercase, alphanumeric, hyphens only)"
    ));
}
```

---

### FR-5: Documentation Drift Detection

**Requirement:** Detect when code changes but docs don't.

**Success Criteria:**
- [ ] New parameter without docs detected
- [ ] Removed parameter still documented detected
- [ ] Renamed parameter detected

**Test:**
```rust
#[test]
fn red_test_detect_drift() {
    let cli_params = extract_cli_parameters();
    let help_params = extract_help_parameters();

    for param in cli_params {
        assert!(help_params.contains(&param),
            "Parameter '{}' in code but not documented", param);
    }
}
```

---

### FR-6: Quality Gate Integration

**Requirement:** Run as quality gate in CI/CD.

**Success Criteria:**
- [ ] Runs in `pmat maintain health --check-docs`
- [ ] Runs in pre-commit hook
- [ ] Clear error messages
- [ ] Exit code 1 on failure

**Test:**
```rust
#[test]
fn red_test_quality_gate_integration() {
    let result = Command::cargo_bin("pmat")
        .unwrap()
        .args(["maintain", "health", "--check-docs"])
        .assert();

    // Should fail if docs incomplete
    assert!(result.get_output()
        .stderr
        .contains("Documentation check"));
}
```

---

## Architecture

### Module Structure

```
server/src/quality/docs_enforcement/
├── mod.rs                 # Module exports
├── cli_checker.rs         # CLI documentation validator
├── mcp_checker.rs         # MCP documentation validator
├── generic_detector.rs    # Generic description detector
├── drift_detector.rs      # Code/docs drift detector
└── reporter.rs            # Error reporting
```

### Data Flow

```
┌─────────────────────────────────────┐
│   Quality Gate                      │
│   pmat maintain health --check-docs │
└──────────────┬──────────────────────┘
               │
       ┌───────┴───────┐
       │               │
       ▼               ▼
┌─────────────┐  ┌──────────────┐
│ CLI Checker │  │ MCP Checker  │
└─────────────┘  └──────────────┘
       │               │
       ├───────┬───────┤
       │       │       │
       ▼       ▼       ▼
    Help   Clap    MCP
    Text   Defs   Schema
       │       │       │
       └───────┴───────┘
               │
               ▼
       ┌───────────────┐
       │ Generic       │
       │ Detector      │
       └───────────────┘
               │
               ▼
       ┌───────────────┐
       │ Drift         │
       │ Detector      │
       └───────────────┘
               │
               ▼
       ┌───────────────┐
       │ Reporter      │
       │ (Errors)      │
       └───────────────┘
```

---

## Implementation Plan

### Phase 1: RED - Write Failing Tests (Week 1)

**Days 1-2: Setup**
- [ ] Create test file structure
- [ ] Add assert_cmd dependency
- [ ] Create test helpers

**Days 3-4: CLI Tests**
- [ ] Write `red_test_all_commands_have_help`
- [ ] Write `red_test_maintain_roadmap_flags_complete`
- [ ] Write `red_test_scaffold_agent_flags_complete`
- [ ] Write `red_test_help_has_descriptions`
- [ ] Write `red_test_help_includes_examples`
- [ ] Write `red_test_no_generic_descriptions_cli`

**Days 5-7: MCP Tests**
- [ ] Write `red_test_all_mcp_tools_have_descriptions`
- [ ] Write `red_test_scaffold_agent_params_documented`
- [ ] Write `red_test_validate_roadmap_params_documented`
- [ ] Write `red_test_generic_description_detector`
- [ ] Write `red_test_detect_undocumented_params`
- [ ] Write `red_test_quality_gate_integration`

**Deliverables:**
- All tests compile
- All tests fail (RED)
- Test coverage matrix complete
- Documentation of expected failures

---

### Phase 2: GREEN - Implement System (Week 2)

**Days 1-3: Core Implementation**
- [ ] Implement `cli_checker.rs`
  - Parse `--help` output
  - Extract clap definitions
  - Compare and validate
- [ ] Implement `mcp_checker.rs`
  - Parse MCP tool definitions
  - Validate parameter descriptions
  - Check schema completeness
- [ ] Implement `generic_detector.rs`
  - Pattern matching for generic phrases
  - Length checks
  - Specific information validation

**Days 4-5: Integration**
- [ ] Implement `drift_detector.rs`
- [ ] Implement `reporter.rs`
- [ ] Create quality gate handler
- [ ] Make all tests pass

**Days 6-7: Verification**
- [ ] All tests GREEN
- [ ] Manual verification
- [ ] Fix edge cases
- [ ] Add integration tests

**Deliverables:**
- All tests passing (GREEN)
- Code complexity <8
- Documentation complete

---

### Phase 3: REFACTOR - Optimize & Integrate (Week 3)

**Days 1-2: Code Quality**
- [ ] Refactor for clarity
- [ ] Reduce duplication
- [ ] Add inline documentation
- [ ] Run complexity checks

**Days 3-4: Integration**
- [ ] Add to `pmat maintain health --all`
- [ ] Add `--check-docs` flag
- [ ] Add to pre-commit hook
- [ ] Add to CI/CD

**Days 5-7: Documentation**
- [ ] User guide
- [ ] Examples
- [ ] Troubleshooting
- [ ] Release notes

**Deliverables:**
- Clean, maintainable code
- Full integration
- Complete documentation
- Production ready

---

## Test Specifications

### Test File 1: `tests/cli_docs_enforcement.rs`

```rust
//! CLI documentation enforcement tests (EXTREME TDD)
//!
//! Phase: RED (All tests should fail)
//!
//! These tests verify that CLI documentation is complete,
//! accurate, and non-generic.

use assert_cmd::Command;
use predicates::prelude::*;

/// RED: All commands must have --help
#[test]
#[ignore] // Remove after implementation
fn red_test_all_commands_have_help() {
    let commands = vec![
        "analyze complexity",
        "analyze satd",
        "analyze dead-code",
        "analyze churn",
        "maintain health",
        "maintain roadmap",
        "scaffold agent",
        "hooks install",
        "hooks verify",
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

/// RED: maintain roadmap must document all flags
#[test]
#[ignore]
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

    // All flags from PMAT-6012
    assert!(help.contains("--validate"), "Missing --validate");
    assert!(help.contains("--health"), "Missing --health");
    assert!(help.contains("--fix"), "Missing --fix");
    assert!(help.contains("--generate-tickets"), "Missing --generate-tickets");
    assert!(help.contains("--dry-run"), "Missing --dry-run");
    assert!(help.contains("--format"), "Missing --format");
}

/// RED: scaffold agent must document all flags
#[test]
#[ignore]
fn red_test_scaffold_agent_flags_complete() {
    let output = Command::cargo_bin("pmat")
        .unwrap()
        .args(["scaffold", "agent", "--help"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let help = String::from_utf8(output).unwrap();

    assert!(help.contains("--template"), "Missing --template");
    assert!(help.contains("--quality-level"), "Missing --quality-level");
    assert!(help.contains("--output"), "Missing --output");
}

/// RED: Help text must have descriptions (not just flag names)
#[test]
#[ignore]
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

    // Should have actual descriptions
    assert!(help.contains("Validate roadmap structure") ||
            help.contains("Check roadmap consistency"),
        "Missing description for --validate");

    assert!(help.contains("Generate missing ticket files") ||
            help.contains("Create ticket files"),
        "Missing description for --generate-tickets");
}

/// RED: Help must include examples section
#[test]
#[ignore]
fn red_test_help_includes_examples() {
    let commands = vec![
        "scaffold agent",
        "maintain roadmap",
        "maintain health",
    ];

    for cmd in commands {
        let output = Command::cargo_bin("pmat")
            .unwrap()
            .args(cmd.split_whitespace())
            .arg("--help")
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();

        let help = String::from_utf8(output).unwrap();

        assert!(help.contains("EXAMPLE") ||
                help.contains("Example") ||
                help.contains("example") ||
                help.contains("EXAMPLES"),
            "Command '{}' missing examples section", cmd);
    }
}

/// RED: No generic descriptions allowed
#[test]
#[ignore]
fn red_test_no_generic_descriptions_cli() {
    let output = Command::cargo_bin("pmat")
        .unwrap()
        .args(["scaffold", "agent", "--help"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let help = String::from_utf8(output).unwrap();

    // Generic patterns forbidden
    let forbidden = vec![
        "The name parameter",
        "The template parameter",
        "Input value",
        "Output value",
    ];

    for pattern in forbidden {
        assert!(!help.contains(pattern),
            "Found forbidden generic pattern: '{}'", pattern);
    }
}
```

### Test File 2: `tests/mcp_docs_enforcement.rs`

```rust
//! MCP documentation enforcement tests (EXTREME TDD)
//!
//! Phase: RED (All tests should fail)

use serde_json::json;

/// RED: All MCP tools must have descriptions >20 chars
#[test]
#[ignore]
fn red_test_all_mcp_tools_have_descriptions() {
    let tools = vec![
        "scaffold_agent",
        "scaffold_wasm",
        "validate_roadmap",
        "health_check",
        "generate_tickets",
    ];

    for tool_name in tools {
        let tool = get_mcp_tool_definition(tool_name);

        assert!(!tool.description.is_empty(),
            "Tool '{}' has no description", tool_name);

        assert!(tool.description.len() > 20,
            "Tool '{}' description too short: '{}'",
            tool_name, tool.description);
    }
}

/// RED: scaffold_agent parameters must be documented
#[test]
#[ignore]
fn red_test_scaffold_agent_params_documented() {
    let tool = get_mcp_tool_definition("scaffold_agent");
    let schema = tool.input_schema;

    // Check 'name' parameter
    let name_desc = schema["properties"]["name"]["description"]
        .as_str()
        .expect("Missing description for 'name'");

    assert!(name_desc.len() > 15,
        "Parameter 'name' description too short: '{}'", name_desc);

    assert!(!is_generic_description(name_desc),
        "Parameter 'name' has generic description: '{}'", name_desc);

    // Should mention constraints
    assert!(name_desc.contains("lowercase") ||
            name_desc.contains("alphanumeric") ||
            name_desc.contains("hyphen"),
        "Parameter 'name' missing constraint info");
}

/// RED: Generic description detector
#[test]
#[ignore]
fn red_test_generic_description_detector() {
    // These should be detected as generic
    let generic = vec![
        "The name parameter",
        "Project name",
        "Input value",
        "Path to file",
        "The template",
        "Output directory",
    ];

    for desc in generic {
        assert!(is_generic_description(desc),
            "Failed to detect generic: '{}'", desc);
    }

    // These should NOT be detected as generic
    let good = vec![
        "Agent project name (lowercase, alphanumeric, hyphens only)",
        "Quality level: standard (fast), high (thorough), extreme (comprehensive with ML)",
        "Path to ROADMAP.md file for validation (default: ./ROADMAP.md)",
        "Output directory where the agent project will be created (default: current directory)",
    ];

    for desc in good {
        assert!(!is_generic_description(desc),
            "Incorrectly flagged as generic: '{}'", desc);
    }
}

// Helper functions (will be implemented)
fn get_mcp_tool_definition(name: &str) -> ToolDefinition {
    // TODO: Implement
    unimplemented!("get_mcp_tool_definition not yet implemented")
}

fn is_generic_description(desc: &str) -> bool {
    // TODO: Implement
    unimplemented!("is_generic_description not yet implemented")
}

struct ToolDefinition {
    name: String,
    description: String,
    input_schema: serde_json::Value,
}
```

---

## Acceptance Criteria

### Phase 1: RED Tests ✅ COMPLETE
- [x] All test files compile
- [x] All tests fail with clear messages
- [x] Test coverage: 100% of requirements
- [x] Helper functions defined
- [x] Expected failures documented
- **Results**: [PMAT-7001-RED-PHASE-RESULTS.md](./PMAT-7001-RED-PHASE-RESULTS.md)

### Phase 2: GREEN Implementation ✅ COMPLETE
- [x] All tests pass
- [x] CLI checker working
- [x] MCP checker working
- [x] Generic detector working
- [x] Code complexity <8
- [x] Documentation complete
- **Results**: [PMAT-7001-GREEN-PHASE-RESULTS.md](./PMAT-7001-GREEN-PHASE-RESULTS.md)

### Phase 3: REFACTOR & Integration ✅ COMPLETE
- [x] Integrated with quality gates (QualityGateService)
- [x] Integrated with pre-commit hook
- [x] JSON reporting for CI/CD
- [x] User documentation complete
- [x] Zero documentation debt (0 issues)
- **Results**: [PMAT-7001-REFACTOR-PHASE-RESULTS.md](./PMAT-7001-REFACTOR-PHASE-RESULTS.md)

### Production Ready ✅ COMPLETE
- [x] All tests GREEN (6/6 passing)
- [x] Quality gate working
- [x] Pre-commit hook working
- [x] <100ms check time (pre-commit)
- [x] 0% false positive rate (zero issues)
- **Release**: v2.142.0
- **Feature Docs**: [documentation-enforcement.md](../features/documentation-enforcement.md)

---

## Dependencies

**New Dependencies:**
```toml
[dev-dependencies]
assert_cmd = "2.0"      # CLI testing
predicates = "3.0"      # Assertion helpers
```

**Existing Code:**
- Clap command definitions (for CLI checking)
- MCP tool definitions (for MCP checking)
- Quality gate infrastructure

---

## Risks & Mitigations

### Risk 1: False Positives

**Risk:** Generic detector flags valid descriptions
**Mitigation:**
- Extensive test cases for edge cases
- Whitelist for domain-specific terms
- Manual review threshold

### Risk 2: Performance

**Risk:** Parsing help text is slow
**Mitigation:**
- Cache parsed results
- Run checks in parallel
- Target <2 second total time

### Risk 3: Maintenance Burden

**Risk:** Tests need updating when commands change
**Mitigation:**
- Auto-generate test cases from command structure
- Use property-based testing where possible
- Clear documentation on updating tests

---

## Success Metrics

### Coverage Targets
- **CLI Commands:** 100% documented
- **MCP Tools:** 100% documented
- **Test Coverage:** 100% of docs checks

### Performance Targets
- **CLI Check:** <1 second
- **MCP Check:** <500ms
- **Total:** <2 seconds

### Quality Targets
- **False Positives:** <5%
- **False Negatives:** 0%
- **Code Complexity:** <8

---

## Related Tickets

- PMAT-6012: Auto-generate tickets (added undocumented flag)
- PMAT-6017: scaffold_agent MCP (generic parameter descriptions)
- Sprint 22: MCP Phase 2 (highlighted documentation gaps)

---

## Conclusion

TICKET-PMAT-7001 will close a critical quality gap by enforcing documentation standards for CLI and MCP interfaces. Using EXTREME TDD ensures we build exactly what's needed with comprehensive test coverage.

**Status:** 🔴 RED (Phase 1 Starting)
**Next Step:** Create test files and write all failing tests
**Estimated Completion:** 3 weeks (12-16 hours total)

---

*Ticket Created: October 6, 2025*
*Sprint 23 - Documentation Quality*
*Methodology: EXTREME TDD*
