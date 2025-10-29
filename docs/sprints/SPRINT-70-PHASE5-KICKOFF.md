# Sprint 70 Phase 5: Documentation - KICKOFF

**Phase**: 5/7
**Status**: Starting
**Date**: October 29, 2025
**Estimated Duration**: 2-3 hours

---

## Overview

Phase 5 focuses on creating comprehensive user-facing documentation for the cargo-mutants integration. After successfully implementing and testing the feature (Phases 1-4), we now need to ensure users can discover, understand, and effectively use `pmat mutate --use-cargo-mutants`.

---

## Prerequisites (All Complete ✅)

- ✅ Phase 1: CargoMutantsWrapper implementation
- ✅ Phase 2: JSON parser for cargo-mutants output
- ✅ Phase 3: CLI integration (`--use-cargo-mutants` flag)
- ✅ Phase 4: Comprehensive testing (10/10 tests passing)

---

## Goals

1. **User Guide**: Clear instructions for using `pmat mutate --use-cargo-mutants`
2. **Troubleshooting**: Common issues and solutions
3. **pmat-book Integration**: Add chapter to official documentation
4. **CLI Help**: Improve `--help` output for discoverability
5. **Examples**: Real-world usage examples

---

## Tasks Breakdown

### Task 1: User Guide (30-45 min)

**Create**: `docs/user-guides/cargo-mutants-integration.md`

**Sections**:
1. Introduction
   - What is cargo-mutants?
   - Why use cargo-mutants with PMAT?
   - When to use `--use-cargo-mutants` vs built-in mutation testing

2. Installation
   - Installing cargo-mutants (v24.7.0+)
   - Version verification
   - Platform compatibility

3. Quick Start
   - Basic usage: `pmat mutate --use-cargo-mutants`
   - Understanding output
   - Interpreting mutation scores

4. Advanced Usage
   - All flag options (`--timeout`, `--jobs`, `--features`, etc.)
   - Output customization
   - Integration with CI/CD

5. Best Practices
   - Timeout recommendations
   - Feature selection strategies
   - Performance optimization

**Deliverable**: Comprehensive user guide (500-700 lines)

---

### Task 2: Troubleshooting Section (20-30 min)

**Add to**: `docs/user-guides/cargo-mutants-integration.md`

**Common Issues**:
1. **cargo-mutants not found**
   - Symptom: "cargo-mutants not found in PATH"
   - Solution: Installation instructions
   - Verification: `cargo mutants --version`

2. **Version too old**
   - Symptom: "cargo-mutants version X.Y.Z is too old"
   - Solution: Upgrade instructions
   - Required: v24.7.0+

3. **Timeout errors**
   - Symptom: Tests timing out frequently
   - Solution: Increase `--timeout` value
   - Recommendation: Start with 300s

4. **No mutants found**
   - Symptom: "Total mutants: 0"
   - Possible causes: No test coverage, excluded files
   - Solution: Check test suite, review exclusions

5. **Parse errors**
   - Symptom: "Failed to parse outcomes.json"
   - Cause: cargo-mutants version mismatch
   - Solution: Ensure v24.7.0+ is installed

6. **Permission errors**
   - Symptom: "Failed to write output"
   - Solution: Check directory permissions

**Deliverable**: Troubleshooting section (150-200 lines)

---

### Task 3: pmat-book Chapter (45-60 min)

**Create**: `../pmat-book/src/mutation-testing-cargo-mutants.md`

**Chapter Structure**:
1. Chapter Title: "Mutation Testing with cargo-mutants"
2. Chapter Number: Insert after current mutation testing chapter

**Content**:
1. Introduction
   - Overview of cargo-mutants integration
   - Comparison with built-in mutation testing
   - Use cases for each approach

2. Getting Started
   - Prerequisites
   - Installation
   - First mutation test

3. Configuration
   - Command-line flags
   - Configuration files (future)
   - Environment variables

4. Understanding Results
   - Mutation score calculation
   - Outcome types (Caught, Missed, Timeout, Unviable)
   - Quality thresholds

5. CI/CD Integration
   - GitHub Actions example
   - GitLab CI example
   - Quality gates

6. Real-World Examples
   - Small project example
   - Large project example
   - Monorepo considerations

7. Performance Tips
   - Parallel execution (`--jobs`)
   - Feature selection (`--features`)
   - Timeout tuning

**Deliverable**: pmat-book chapter (600-800 lines)

---

### Task 4: CLI Help Improvements (15-20 min)

**Files to Update**:
1. `server/src/cli/handlers/mutate.rs` - Command help text
2. `server/src/cli/mod.rs` - Flag descriptions

**Improvements**:
1. Enhanced `--use-cargo-mutants` description
2. Add examples to help text
3. Clarify flag interactions
4. Link to documentation

**Before**:
```
--use-cargo-mutants    Use cargo-mutants for mutation testing
```

**After**:
```
--use-cargo-mutants    Use cargo-mutants backend for mutation testing (requires cargo-mutants v24.7.0+)
                       Provides comprehensive Rust mutation testing with industry-standard tool.
                       Example: pmat mutate --use-cargo-mutants --timeout 300
```

**Deliverable**: Improved CLI help (50-100 lines changed)

---

### Task 5: Examples and Use Cases (20-30 min)

**Create**: `docs/examples/cargo-mutants-examples.md`

**Examples**:
1. **Basic Usage**
   ```bash
   # Simple mutation test
   pmat mutate --target . --use-cargo-mutants
   ```

2. **With Timeout**
   ```bash
   # Increase timeout for slower tests
   pmat mutate --target . --use-cargo-mutants --timeout 600
   ```

3. **Parallel Execution**
   ```bash
   # Use 4 parallel jobs
   pmat mutate --target . --use-cargo-mutants --jobs 4
   ```

4. **Feature Selection**
   ```bash
   # Test specific features
   pmat mutate --target . --use-cargo-mutants --features "feat1,feat2"
   ```

5. **CI/CD Integration**
   ```yaml
   # GitHub Actions example
   - name: Mutation Testing
     run: |
       cargo install pmat cargo-mutants
       pmat mutate --use-cargo-mutants --timeout 600
   ```

6. **With Output File**
   ```bash
   # Save results for later analysis
   pmat mutate --target . --use-cargo-mutants --output results.json
   ```

**Deliverable**: Examples document (200-300 lines)

---

## Documentation Structure

```
docs/
├── user-guides/
│   └── cargo-mutants-integration.md (NEW - Tasks 1-2)
├── examples/
│   └── cargo-mutants-examples.md (NEW - Task 5)
└── sprints/
    └── SPRINT-70-PHASE5-KICKOFF.md (this file)

../pmat-book/src/
└── mutation-testing-cargo-mutants.md (NEW - Task 3)

server/src/cli/
├── handlers/mutate.rs (UPDATED - Task 4)
└── mod.rs (UPDATED - Task 4)
```

---

## Success Criteria

### Documentation Quality
- [ ] All sections complete and comprehensive
- [ ] Code examples tested and working
- [ ] No broken links
- [ ] Clear, concise language
- [ ] Proper formatting (Markdown)

### User Experience
- [ ] Users can install and use feature independently
- [ ] Common problems addressed in troubleshooting
- [ ] Examples cover typical use cases
- [ ] CLI help is clear and actionable

### Technical Accuracy
- [ ] All commands verified to work
- [ ] Version requirements correct
- [ ] Flag descriptions accurate
- [ ] Output examples match actual output

### Completeness
- [ ] User guide covers all features
- [ ] pmat-book chapter integrated properly
- [ ] CLI help reflects all options
- [ ] Examples demonstrate best practices

---

## Timeline

**Total Estimated Time**: 2-3 hours

| Task | Duration | Status |
|------|----------|--------|
| Task 1: User Guide | 30-45 min | Pending |
| Task 2: Troubleshooting | 20-30 min | Pending |
| Task 3: pmat-book Chapter | 45-60 min | Pending |
| Task 4: CLI Help | 15-20 min | Pending |
| Task 5: Examples | 20-30 min | Pending |

**Order of Execution**:
1. Task 1 (User Guide) - Foundation
2. Task 2 (Troubleshooting) - Extends user guide
3. Task 5 (Examples) - Practical demonstrations
4. Task 4 (CLI Help) - Quick wins
5. Task 3 (pmat-book) - Comprehensive integration

---

## Key Decisions

### Documentation Tone
- **Audience**: Rust developers familiar with testing
- **Style**: Clear, concise, example-driven
- **Assumption**: Users know cargo and Rust basics
- **Focus**: Practical usage over implementation details

### Coverage Scope
- **In Scope**: Installation, usage, troubleshooting, examples
- **Out of Scope**: cargo-mutants internals, PMAT implementation details
- **Future Work**: Configuration files, advanced filtering

### pmat-book Integration
- **Location**: New chapter after existing mutation testing chapter
- **Format**: Follow existing pmat-book style
- **Validation**: Test all examples with mdbook-test (if available)

---

## Quality Checks

### Before Committing
1. **Spelling & Grammar**: Run through spell checker
2. **Links**: Verify all internal/external links work
3. **Code Examples**: Test all command examples
4. **Formatting**: Ensure consistent Markdown formatting
5. **Completeness**: All sections filled in

### Testing Documentation
1. **User Guide**: Have someone unfamiliar try to follow it
2. **Examples**: Run every command example
3. **Troubleshooting**: Verify solutions work for listed problems
4. **CLI Help**: Check `pmat mutate --help` output

---

## Notes

- Keep examples realistic and practical
- Use actual output from cargo-mutants v25.3.1
- Link to cargo-mutants documentation where appropriate
- Emphasize that this is an alternative backend, not a replacement
- Highlight when to use built-in vs cargo-mutants

---

## References

**Cargo-mutants Documentation**:
- https://mutants.rs/
- https://github.com/sourcefrog/cargo-mutants

**PMAT Documentation**:
- `docs/PROJECT-STATE-SUMMARY.md`
- `docs/sprints/SPRINT-70-*`

**Related Issues**:
- Sprint 70 tracking
- Phase 1-4 completion docs

---

## Next Steps After Phase 5

1. **Phase 6**: Performance Validation (2-3 hours)
   - Benchmark with large projects
   - Profile memory usage
   - Optimize if needed

2. **Phase 7**: Release Preparation (1-2 hours)
   - Update CHANGELOG
   - Version bump
   - Release notes

---

**Status**: Ready to begin Task 1 (User Guide)
