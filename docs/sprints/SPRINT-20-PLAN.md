# Sprint 20 Planning: UX Improvements & Optimizations

**Sprint**: Sprint 20 - UX Improvements & Optimizations
**Duration**: 2-3 days
**Status**: 📋 **PLANNED**
**Start Date**: 2025-10-06
**Based On**: Sprint 19 dogfooding findings

## Executive Summary

Sprint 20 focuses on addressing the UX issues and performance problems discovered during Sprint 19 dogfooding. This sprint prioritizes developer experience improvements that make PMAT CLI faster and more intuitive to use.

## Background

Sprint 19 successfully delivered all CLI integration features, but dogfooding revealed several areas for improvement:

**Critical Issues:**
1. Health command times out (300s) when running all checks
2. Documentation examples use inconsistent naming conventions
3. No progress feedback for long-running operations

**Based On:** `docs/dogfooding/SPRINT-19-DOGFOODING-RESULTS.md`

## Sprint Goals

1. **Make health checks fast and usable** - Add quick mode and progress indicators
2. **Fix documentation inconsistencies** - Update all examples to use correct naming
3. **Improve error messages** - Add helpful suggestions for common mistakes
4. **Add testing** - Integration tests for CLI commands
5. **Polish UX** - Better feedback and clearer output

## Tickets

### High Priority (Must Have)

#### TICKET-PMAT-6001: Health Command Optimization
**Priority:** P0 - Critical
**Estimated Effort:** 4-6 hours

**Problem:**
- Current: All health checks run by default (build, test, coverage, complexity, SATD)
- Full test suite takes 606s, coverage adds more time
- Commands time out after 300s
- No way to run quick health check

**Solution:**
Add `--quick` flag and make checks opt-in:

```bash
# Quick health check (build + basic tests only, <30s)
pmat maintain health --quick

# Full health check (opt-in to slow checks)
pmat maintain health --check-coverage --check-complexity --check-satd

# Individual checks
pmat maintain health --check-build
pmat maintain health --check-tests --check-coverage
```

**Implementation:**
1. Change defaults: only build + basic tests enabled by default
2. Add `--quick` flag as alias for minimal checks
3. Make coverage, complexity, SATD opt-in with explicit flags
4. Update help text with performance guidance

**Success Criteria:**
- Default health check completes in <30s
- --quick mode completes in <10s
- Full checks still available via explicit flags
- Documentation explains performance implications

**Complexity Target:** CC <8
**Tests:** Unit tests for flag combinations, integration test for performance

---

#### TICKET-PMAT-6002: Progress Indicators for Long Operations
**Priority:** P0 - Critical
**Estimated Effort:** 3-4 hours

**Problem:**
- No feedback during long-running operations
- Users don't know if command is frozen or working
- Especially problematic for health checks and scaffolding

**Solution:**
Add progress indicators using `indicatif` crate:

```bash
pmat maintain health
⠋ Running build check...
✓ Build check passed (2.3s)
⠋ Running tests...
✓ Tests passed: 4074/4088 (606s)
⠋ Running coverage check...
✓ Coverage: 84.2% (120s)
```

**Implementation:**
1. Add `indicatif` dependency
2. Create progress bar wrapper for long operations
3. Add to health checks (build, test, coverage)
4. Add to scaffolding operations (file generation, git init)
5. Support `--quiet` flag to disable progress bars

**Success Criteria:**
- Clear progress indication for operations >5s
- Spinners show current operation
- Completion shows duration and status
- Works in CI environments (non-TTY)

**Complexity Target:** CC <6
**Tests:** Unit tests for progress wrapper, visual testing

---

#### TICKET-PMAT-6003: Documentation Naming Convention Fixes
**Priority:** P1 - High
**Estimated Effort:** 1-2 hours

**Problem:**
- Examples use `test-agent` (kebab-case)
- Command requires `test_agent` (snake_case)
- Confusing for new users
- Error messages don't explain the issue

**Solution:**
1. Update all documentation examples to use snake_case
2. Add validation error with helpful suggestion
3. Consider auto-converting kebab-case to snake_case

**Files to Update:**
- `examples/agent-scaffolding.md`
- `examples/wasm-scaffolding.md`
- `examples/scaffolding-quickstart.md`
- `examples/README.md`

**Enhanced Error Message:**
```
Error: Agent name must be alphanumeric with underscores only
  You provided: "test-agent"
  Did you mean: "test_agent"?
```

**Success Criteria:**
- All examples use consistent snake_case naming
- Error messages provide helpful suggestions
- Users understand the naming requirement immediately

**Complexity Target:** CC <5
**Tests:** Update example validation tests

---

### Medium Priority (Should Have)

#### TICKET-PMAT-6004: Enhanced Error Messages
**Priority:** P1 - High
**Estimated Effort:** 2-3 hours

**Problem:**
- Generic "No such file or directory" errors
- No context about what file was expected
- No suggestions for resolution

**Solution:**
Improve error messages throughout CLI:

**Before:**
```
ERROR Error: No such file or directory (os error 2)
```

**After:**
```
ERROR Failed to read ROADMAP.md
  Location: /home/user/project/ROADMAP.md
  Reason: File not found

  Suggestions:
  - Run 'pmat maintain roadmap' from project root
  - Create ROADMAP.md using 'pmat init roadmap'
  - Specify custom path: --roadmap /path/to/ROADMAP.md
```

**Implementation:**
1. Create error context wrapper
2. Add file path to all file operations
3. Add suggestions for common error scenarios
4. Test with various failure modes

**Success Criteria:**
- All file errors include full path
- Common errors have actionable suggestions
- Error messages are beginner-friendly

**Complexity Target:** CC <7
**Tests:** Unit tests for error formatting, integration tests for error scenarios

---

#### TICKET-PMAT-6005: CLI Integration Tests
**Priority:** P1 - High
**Estimated Effort:** 3-4 hours

**Problem:**
- No integration tests for CLI commands
- Manual testing only
- Risk of regressions

**Solution:**
Add integration tests for all Sprint 19 commands:

```rust
#[test]
fn test_scaffold_agent_dry_run() {
    let output = Command::new("pmat")
        .args(&["scaffold", "agent", "--name", "test_agent", "--template", "basic", "--dry-run"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Dry run mode"));
    assert!(stdout.contains("test_agent"));
}
```

**Tests to Add:**
1. Scaffold agent (dry-run, validation)
2. Scaffold wasm (dry-run, frameworks)
3. Maintain roadmap (health, validate)
4. Maintain health (quick mode, individual checks)
5. Hooks (status, install dry-run)

**Success Criteria:**
- 20+ integration tests covering all Sprint 19 commands
- Tests run in <60s
- CI integration working
- Coverage >90% for CLI handlers

**Complexity Target:** CC <5 per test
**Tests:** Integration test suite

---

### Low Priority (Nice to Have)

#### TICKET-PMAT-6006: UX Polish
**Priority:** P2 - Medium
**Estimated Effort:** 2-3 hours

**Improvements:**
1. **Color Configuration**
   - `--color auto|always|never` flag
   - Respect `NO_COLOR` environment variable
   - Smart TTY detection

2. **Verbose/Quiet Modes**
   - `--verbose` for detailed output
   - `--quiet` for minimal output
   - Affects progress bars and logging

3. **Shell Completion**
   - Bash completion script
   - Zsh completion script
   - Fish completion script
   - Installation instructions

**Success Criteria:**
- Consistent color behavior across platforms
- Verbose mode shows debug information
- Quiet mode shows errors only
- Shell completion available for major shells

**Complexity Target:** CC <6
**Tests:** Unit tests for output modes

---

## Success Criteria

### Performance
- ✅ Default health check: <30s
- ✅ Quick health check: <10s
- ✅ Scaffold dry-run: <1s
- ✅ Roadmap health: <2s

### Quality
- ✅ All functions CC <10
- ✅ Test coverage >80%
- ✅ Integration test coverage >90%
- ✅ No SATD violations

### User Experience
- ✅ Clear progress indication for long operations
- ✅ Helpful error messages with suggestions
- ✅ Consistent documentation examples
- ✅ Fast default commands

### Testing
- ✅ 20+ CLI integration tests
- ✅ Performance benchmarks for health checks
- ✅ Validation tests for all inputs
- ✅ Error scenario coverage

## Implementation Strategy

### Phase 1: Critical Performance (Day 1)
1. TICKET-PMAT-6001: Health command optimization
2. TICKET-PMAT-6002: Progress indicators
3. Test and validate performance improvements

### Phase 2: Documentation & Errors (Day 2)
4. TICKET-PMAT-6003: Documentation fixes
5. TICKET-PMAT-6004: Enhanced error messages
6. Update all examples and docs

### Phase 3: Testing & Polish (Day 3)
7. TICKET-PMAT-6005: Integration tests
8. TICKET-PMAT-6006: UX polish (if time permits)
9. Final dogfooding and validation

## Dependencies

**Depends On:**
- Sprint 19: CLI Integration & Dogfooding (COMPLETE)

**Enables:**
- Better developer experience
- Faster feedback loops
- Reduced support burden
- Higher adoption rate

## Risks and Mitigation

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Progress bars break in CI | High | Medium | Add TTY detection, fallback to simple logging |
| Health check changes break existing usage | Medium | Low | Keep old flags working, add deprecation warnings |
| Integration tests slow down CI | Medium | Medium | Run in parallel, optimize test setup |
| Scope creep on UX polish | Low | High | Defer P2 items if timeline slips |

## Metrics

**Code Changes (Estimated):**
- New code: ~800 lines
- Tests: ~600 lines
- Documentation: ~200 lines
- Modified files: ~15

**Time Breakdown:**
- Implementation: 15-20 hours
- Testing: 5-7 hours
- Documentation: 2-3 hours
- Total: 22-30 hours (2-3 days)

## Definition of Done

- [ ] All P0/P1 tickets complete
- [ ] Health check <30s by default
- [ ] Progress bars working in all environments
- [ ] Documentation examples corrected
- [ ] 20+ integration tests passing
- [ ] All quality gates passing
- [ ] Dogfooding complete
- [ ] Sprint summary document created

## References

- Sprint 19 Summary: `docs/sprints/SPRINT-19-SUMMARY.md`
- Dogfooding Results: `docs/dogfooding/SPRINT-19-DOGFOODING-RESULTS.md`
- Original Spec: `docs/specifications/scaffold-maintain-spec.md`

---

**Created:** 2025-10-06
**Status:** Planning → Ready for Implementation
