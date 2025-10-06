# What's Next: Sprint 21 Continuation Guide

**Last Updated:** October 6, 2025
**Current Status:** Sprint 21 in progress (1/4 tickets complete)
**Next Release:** v2.140.0 (Sprint 21 completion)

---

## Current State

### Completed ✅

**v2.139.0 Release:**
- ✅ Published to crates.io
- ✅ Git tag v2.139.0 pushed
- ✅ All 28 tickets (Sprints 16-20) complete
- ✅ 5,000+ lines of documentation

**Integration:**
- ✅ Quality gates configured
- ✅ Pre-commit hooks active
- ✅ Roadmap validated
- ✅ Health checks optimized (9.8s)

**Sprint 21:**
- ✅ Planning complete (1,000+ lines)
- ✅ PMAT-6011 implemented (hook verification fix)
- ✅ 1/4 P0+P1 tickets done

### In Progress 🔄

**Sprint 21 Remaining (3/4):**
- [ ] PMAT-6010: Parallel health checks (3-4h)
- [ ] PMAT-6012: Auto-generate tickets (3-4h)
- [ ] PMAT-6013: MCP scaffolding (4-6h)

**Estimated:** 10-15 hours remaining

---

## Quick Start: Resume Development

### 1. Verify Current State

```bash
# Check git status
git status
git log --oneline -5

# Verify PMAT installation
cargo install pmat --version 2.139.0

# Test current functionality
pmat maintain health --quick
pmat maintain roadmap --validate
pmat hooks verify
```

**Expected Results:**
- Health check: ~10s
- Roadmap: ✅ validation passed
- Hooks: ✅ verified successfully (PMAT-6011 fix working!)

### 2. Review Planning Documents

```bash
# Sprint 21 plan
cat docs/sprints/SPRINT-21-PLAN.md

# Priority matrix
cat docs/sprints/SPRINT-21-PRIORITIES.md

# Dogfooding findings
cat docs/dogfooding/v2.139.0-INTEGRATION-SHOWCASE.md
```

### 3. Start Next Ticket

**Recommended Order:**
1. PMAT-6010 (Parallel health checks) - Performance win
2. PMAT-6012 (Auto-generate tickets) - Automation win
3. PMAT-6013 (MCP scaffolding) - Integration win

---

## PMAT-6010: Parallel Health Checks

### Quick Reference

**Priority:** P0 - Critical
**Effort:** 3-4 hours
**Value:** 14-40% speed improvement

**Current Behavior:**
```bash
pmat maintain health --check-build --check-tests
# Build: 10s (sequential)
# Tests: 60s (sequential)
# Total: 70s
```

**Target Behavior:**
```bash
pmat maintain health --check-build --check-tests
# Build + Tests in parallel
# Total: 60s (limited by slowest)
# Improvement: 14% faster
```

### Implementation Steps

1. **Add tokio async support to health checks**
   - File: `server/src/cli/handlers/health_handler.rs`
   - Use `tokio::spawn` for concurrent execution
   - Target CC: <8

2. **Update progress indicators**
   - Show all active checks simultaneously
   - Use `indicatif::MultiProgress`
   - Handle completion order correctly

3. **Add tests**
   - Verify parallel execution
   - Confirm total time = max(check_times)
   - Property test for various check combinations

4. **Create ticket file**
   - Generate `docs/tickets/TICKET-PMAT-6010.md`
   - Document implementation
   - Update status to GREEN

### Code Skeleton

```rust
// server/src/cli/handlers/health_handler.rs

use tokio::task::JoinSet;

pub async fn run_health_checks_parallel(
    checks: Vec<CheckType>,
    project_dir: &Path,
) -> Result<Vec<CheckResult>> {
    let mut set = JoinSet::new();

    for check in checks {
        let dir = project_dir.to_path_buf();
        set.spawn(async move {
            run_single_check(check, &dir).await
        });
    }

    let mut results = Vec::new();
    while let Some(res) = set.join_next().await {
        results.push(res??);
    }

    Ok(results)
}
```

---

## PMAT-6012: Auto-Generate Ticket Files

### Quick Reference

**Priority:** P1 - High
**Effort:** 3-4 hours
**Value:** Eliminate 50+ minutes of manual work per sprint

**Current Behavior:**
- Manually create ticket files (10 min each)
- Copy/paste from template
- Inconsistent formatting

**Target Behavior:**
```bash
pmat maintain roadmap --generate-tickets

# Output:
# 📝 Generating ticket files...
#   Created TICKET-PMAT-6010.md
#   Created TICKET-PMAT-6012.md
#   Created TICKET-PMAT-6013.md
# ✅ Generated 3 ticket files
```

### Implementation Steps

1. **Parse roadmap for missing tickets**
   - File: `server/src/cli/handlers/roadmap_handler.rs`
   - Extract ticket IDs from roadmap
   - Check if ticket file exists
   - Detect sprint context

2. **Generate ticket template**
   - Include detected sprint
   - Use checkbox status for ticket status
   - Add standard sections (Problem, Solution, etc.)
   - Target CC: <7

3. **Add dry-run mode**
   - `--generate-tickets --dry-run`
   - Show what would be created
   - Confirm before writing

4. **Create tests**
   - Test ticket detection
   - Test template generation
   - Verify sprint context detection

### Template Structure

```markdown
# TICKET-PMAT-XXXX: [Title from roadmap]

**Sprint:** [Detected from roadmap]
**Priority:** [TBD]
**Estimated Effort:** [TBD]
**Status**: [GREEN if checked, PLANNED if unchecked]
**Created:** [Current date]

## Problem Statement

[TODO: Describe the problem]

## Solution

[TODO: Describe solution]

## Acceptance Criteria

- [ ] [TODO: Add criteria]
```

---

## PMAT-6013: MCP Scaffolding

### Quick Reference

**Priority:** P1 - High
**Effort:** 4-6 hours
**Value:** Enable agent ecosystem integration

**Current State:**
- Scaffolding only via CLI
- Agents can't use features

**Target State:**
- 5 MCP tools exposed
- Full scaffolding via protocol
- Agent integration enabled

### Implementation Steps

1. **Add MCP tools**
   - File: `server/src/mcp/tools.rs`
   - Tool: `scaffold_agent`
   - Tool: `scaffold_wasm`
   - Tool: `validate_roadmap`
   - Tool: `health_check`
   - Tool: `generate_tickets` (if PMAT-6012 done)

2. **Reuse existing handlers**
   - Call existing scaffolding functions
   - Add proper error handling for MCP
   - Convert results to MCP format

3. **Add MCP tests**
   - Test each tool via protocol
   - Verify input validation
   - Check output format

4. **Update documentation**
   - Add MCP examples to feature guide
   - Show agent integration patterns

### MCP Tool Example

```json
{
  "name": "scaffold_agent",
  "description": "Generate a new MCP agent project",
  "inputSchema": {
    "type": "object",
    "properties": {
      "name": {"type": "string"},
      "template": {
        "type": "string",
        "enum": ["basic", "stateful", "hybrid"]
      },
      "quality_level": {
        "type": "string",
        "enum": ["extreme", "high", "standard"]
      }
    },
    "required": ["name", "template"]
  }
}
```

---

## Testing Strategy

### For Each Ticket

1. **Write tests first** (TDD)
   - Unit tests for core logic
   - Integration tests for CLI
   - Property tests where applicable

2. **Implement feature**
   - Keep CC <8
   - Add comprehensive error handling
   - Use existing patterns

3. **Validate quality**
   - Run `cargo test`
   - Run `pmat maintain health --all`
   - Check complexity: `pmat analyze complexity`

4. **Dogfood immediately**
   - Use feature in PMAT development
   - Document any issues
   - Iterate if needed

### Quality Gates

Before committing:
```bash
# Run all tests
cargo test --lib

# Check complexity
pmat analyze complexity --max-cyclomatic 10

# Verify builds
cargo build --release

# Test CLI integration (if applicable)
cargo test --test cli_integration_tests
```

---

## Release Process (v2.140.0)

### When All Sprint 21 Tickets Complete

1. **Create Sprint 21 summary**
   - File: `docs/sprints/SPRINT-21-SUMMARY.md`
   - Document all 4 tickets
   - Include metrics and lessons learned

2. **Update ROADMAP**
   - Mark Sprint 21 complete
   - Move to completed releases section
   - Add v2.140.0 entry

3. **Create release notes**
   - File: `docs/release_notes/v2.140.0.md`
   - Highlight improvements
   - Include upgrade guide

4. **Version bump**
   - Update `Cargo.toml` to 2.140.0
   - Test build
   - Commit version change

5. **Tag and publish**
   ```bash
   git tag -a v2.140.0 -m "Release v2.140.0 - Sprint 21 Refinements"
   git push --tags
   cargo publish
   ```

6. **Dogfood and validate**
   - Test all new features in PMAT
   - Update integration showcase
   - Document findings

---

## Tips & Best Practices

### Development Workflow

1. **Small commits** - Commit every 1-2 hours
2. **Test continuously** - Don't wait until end
3. **Dogfood early** - Use features as you build them
4. **Document as you go** - Don't batch documentation

### Common Pitfalls

1. **Long build times** - Use `cargo check` frequently
2. **Scope creep** - Stick to ticket description
3. **Complex functions** - Extract helpers if CC >8
4. **Missing tests** - Write tests before code

### Quick Commands

```bash
# Fast feedback
cargo check

# Run specific test
cargo test test_name

# Watch mode
cargo watch -x check

# Format code
cargo fmt

# Lint
cargo clippy

# Health check
pmat maintain health --quick
```

---

## Success Criteria

### Sprint 21 Complete When:

- [ ] All 4 P0+P1 tickets implemented
- [ ] All tests passing (existing + new)
- [ ] All code CC <8
- [ ] Documentation updated
- [ ] Dogfooding validated
- [ ] v2.140.0 released

### Each Ticket Complete When:

- [ ] Implementation done
- [ ] Tests passing (>80% coverage)
- [ ] Complexity <8
- [ ] Ticket file created
- [ ] Documentation updated
- [ ] Dogfooded in PMAT
- [ ] Committed and pushed

---

## Resources

### Documentation
- Sprint 21 Plan: `docs/sprints/SPRINT-21-PLAN.md`
- Priority Matrix: `docs/sprints/SPRINT-21-PRIORITIES.md`
- Dogfooding Findings: `docs/dogfooding/v2.139.0-INTEGRATION-SHOWCASE.md`
- Feature Guide: `docs/features/SCAFFOLDING-AND-MAINTENANCE.md`

### Code References
- Health Handler: `server/src/cli/handlers/health_handler.rs`
- Roadmap Handler: `server/src/cli/handlers/roadmap_handler.rs`
- Hooks Handler: `server/src/cli/handlers/hooks_command_handlers.rs`
- MCP Tools: `server/src/mcp/tools.rs`

### Tests
- Integration Tests: `server/src/tests/cli_integration_tests.rs`
- Unit Tests: Throughout codebase
- Property Tests: `*_property_tests.rs` files

---

## Questions?

### If stuck on:

**Implementation:** Review existing similar code (e.g., PMAT-6011 for pattern)
**Testing:** Check `cli_integration_tests.rs` for examples
**MCP:** Review existing MCP tools in `server/src/mcp/tools.rs`
**Performance:** Profile with `cargo flamegraph`

### Need help?

1. Check documentation first
2. Review similar completed tickets
3. Look at dogfooding findings for context
4. Test in isolation before integrating

---

**Ready to continue:** Pick PMAT-6010 and start implementing! 🚀

**Estimated completion:** 2-3 days for full Sprint 21

**Target release:** v2.140.0

---

*Last session: Complete success - v2.139.0 released, integrated, Sprint 21 started*
*Next session: Continue Sprint 21 implementation*
