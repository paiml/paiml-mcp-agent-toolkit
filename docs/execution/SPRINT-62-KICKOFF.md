# Sprint 62 Kickoff - Mutation Testing Output Refinement

**Sprint Goal**: Enhance mutation testing CLI output with detailed reporting and filtering options

**Timeline**: 3-day sprint (following Sprint 61 v2.174.0 release)

**Target Version**: v2.175.0

---

## Sprint Context

**Previous Sprint (Sprint 61 - v2.174.0)**:
- ✅ Completed Days 1-4: Command skeleton, real file testing, output formats, progress indicators
- ✅ Published to crates.io: https://crates.io/crates/pmat
- ✅ GitHub release: https://github.com/paiml/paiml-mcp-agent-toolkit/releases/tag/v2.174.0

**Current State**:
- Mutation testing CLI is functional with basic text, JSON, and Markdown outputs
- Progress indicators work correctly
- 239 mutants generated and tested on path_validator.rs
- 37 mutants generated and tested on test_sample.rs

**What's Missing** (deferred from Sprint 61):
- Detailed mutant information in outputs
- `--failures-only` flag for CI/CD optimization
- Color coding for terminal output
- Multi-file size testing validation

---

## Sprint 62 Goals

### Day 1: Enhanced Mutant Details (Output Refinement)

**Goal**: Add detailed information about each mutant to all output formats

**Tasks**:
1. **Enhance Text Output** (2 hours)
   - Add mutant details section showing:
     - File location (path:line:column)
     - Mutation operator applied (e.g., "BinaryOp(+ → -)")
     - Original vs mutated code snippet
     - Execution time per mutant
   - Example format:
     ```
     Survived Mutants (needs test coverage):
     1. src/utils/path_validator.rs:150:20
        Operator: BinaryOp(+ → -)
        Original: `if count + 1 > MAX_DEPTH`
        Mutated:  `if count - 1 > MAX_DEPTH`
        Time: 0.42s
     ```

2. **Enhance JSON Output** (1 hour)
   - Add code snippets to JSON output:
     ```json
     {
       "mutants": [{
         "id": 1,
         "file": "src/utils/path_validator.rs",
         "line": 150,
         "column": 20,
         "operator": "BinaryOp",
         "original_code": "if count + 1 > MAX_DEPTH",
         "mutated_code": "if count - 1 > MAX_DEPTH",
         "status": "survived",
         "execution_time_ms": 420
       }]
     }
     ```

3. **Enhance Markdown Output** (1 hour)
   - Add code diff blocks for survived mutants:
     ```markdown
     ### Survived Mutant #1
     **Location**: `src/utils/path_validator.rs:150:20`
     **Operator**: BinaryOp(+ → -)
     **Execution Time**: 0.42s

     ```diff
     - if count + 1 > MAX_DEPTH
     + if count - 1 > MAX_DEPTH
     ```
     ```

4. **Testing** (1 hour)
   - Test with path_validator.rs (239 mutants)
   - Test with test_sample.rs (37 mutants)
   - Verify all three output formats display correctly

---

### Day 2: Failures-Only Flag and Color Coding

**Goal**: Add `--failures-only` flag and color-coded terminal output

**Tasks**:
1. **Implement `--failures-only` Flag** (2 hours)
   - Add to MutateArgs in `commands.rs`:
     ```rust
     #[arg(long, help = "Show only survived mutants (failures) in output")]
     pub failures_only: bool,
     ```
   - Modify `output_text()`, `output_json()`, `output_markdown()` in `handlers/mutate.rs`
   - Filter results to show only:
     - Survived mutants (test gaps)
     - Compile errors (if any)
     - Timeouts (if any)
   - Hide killed mutants from detailed output
   - Keep summary statistics intact

2. **Add Color Coding to Text Output** (2 hours)
   - Use `console` crate (already a dependency):
     ```rust
     use console::Style;

     let green = Style::new().green().bold();
     let red = Style::new().red().bold();
     let yellow = Style::new().yellow();
     let cyan = Style::new().cyan();
     ```
   - Color scheme:
     - **Green**: Killed mutants, mutation score if passing threshold
     - **Red**: Survived mutants, mutation score if below threshold
     - **Yellow**: Compile errors, timeouts
     - **Cyan**: File paths, operator names
   - Respect `NO_COLOR` environment variable

3. **Testing** (1 hour)
   - Test `--failures-only` with both passing and failing mutation scores
   - Test color output in different terminals
   - Verify CI/CD compatibility (NO_COLOR=1)

---

### Day 3: Multi-File Testing and Documentation

**Goal**: Test with varying file sizes and update documentation

**Tasks**:
1. **Multi-File Size Testing** (2 hours)
   - Test with small file (test_sample.rs, 52 lines, 37 mutants) ✅ Already done
   - Test with medium file (path_validator.rs, 352 lines, 239 mutants) ✅ Already done
   - Test with large file (>1000 lines):
     - Candidate: `server/src/services/deep_context.rs` (~2000 lines)
     - Expected: 500-1000+ mutants
     - Timeout: 60s per mutant, `--jobs 4`
   - Measure performance:
     - Mutant generation time
     - Execution time
     - Memory usage
     - Output file sizes

2. **Update Documentation** (1 hour)
   - Update `server/README.md` with new flags:
     ```bash
     # Show only failures (CI/CD optimization)
     pmat mutate --target src/ --failures-only --output-format json

     # Colored output (default)
     pmat mutate --target src/file.rs

     # Disable colors (CI/CD)
     NO_COLOR=1 pmat mutate --target src/file.rs
     ```
   - Update `CHANGELOG.md` with v2.175.0 entry

3. **Create Examples** (1 hour)
   - Create `examples/mutation_testing_workflow.md` showing:
     - Basic usage
     - CI/CD integration
     - GitHub Actions example
     - Markdown report in PR comments

4. **Sprint Completion** (1 hour)
   - Update `SPRINT-62-PROGRESS.md` with completion status
   - Run clippy, tests, and quality gates
   - Commit all changes with proper messages

---

## Implementation Plan

### File Modifications

**Day 1** (5 files):
- `server/src/cli/handlers/mutate.rs` - Add code snippets to outputs
- `server/src/services/mutation/types.rs` - Add original_code/mutated_code fields to MutationResult

**Day 2** (2 files):
- `server/src/cli/commands.rs` - Add failures_only flag
- `server/src/cli/handlers/mutate.rs` - Implement filtering and color coding

**Day 3** (4 files):
- `server/README.md` - Update documentation
- `CHANGELOG.md` - Add v2.175.0 entry
- `docs/execution/SPRINT-62-PROGRESS.md` - Track progress
- `examples/mutation_testing_workflow.md` - Create workflow guide (NEW FILE)

---

## Success Criteria

### Day 1 - Enhanced Mutant Details
- [ ] Text output shows code snippets for survived mutants
- [ ] JSON output includes original/mutated code fields
- [ ] Markdown output includes diff blocks
- [ ] All three formats tested with path_validator.rs and test_sample.rs

### Day 2 - Failures-Only Flag and Color Coding
- [ ] `--failures-only` flag implemented and tested
- [ ] Color-coded terminal output functional
- [ ] `NO_COLOR` environment variable respected
- [ ] CI/CD compatibility verified

### Day 3 - Multi-File Testing and Documentation
- [ ] Large file tested (>1000 lines, 500+ mutants)
- [ ] Performance metrics documented
- [ ] README.md updated with new features
- [ ] CHANGELOG.md updated with v2.175.0
- [ ] Example workflow guide created

---

## Risk Assessment

### Low Risk ✅
- Color coding implementation (console crate already a dependency)
- Failures-only filtering (straightforward conditional logic)

### Medium Risk ⚠️
- Code snippet extraction (need to parse original source files)
- Large file performance (may need optimization for 500+ mutants)

### Mitigation Strategies
- **Code snippet extraction**: Use existing file reading infrastructure, cache source files
- **Large file performance**: Implement batching if needed, optimize memory usage

---

## Dependencies

**Existing Dependencies** (no additions needed):
- `console = "0.15"` - Color coding and styling
- `serde_json = "1.0"` - JSON serialization

**Existing Infrastructure**:
- `MutationEngine` - Already generates mutants
- `MutationResult` - Will be extended with code snippet fields
- `MutationScore` - Already calculates statistics

---

## Technical Notes

### Code Snippet Extraction Strategy

1. **Store original code during mutant generation** (in `MutationEngine`)
   - Capture source lines when creating mutants
   - Store in `MutationResult` struct

2. **Format for display** (in output functions)
   - Text: Multi-line format with indentation
   - JSON: String fields with newlines escaped
   - Markdown: Diff blocks with syntax highlighting

### Performance Considerations

- **Memory**: Code snippets add ~50-200 bytes per mutant
  - 239 mutants × 100 bytes = ~24 KB (negligible)
  - 1000 mutants × 100 bytes = ~100 KB (still acceptable)
- **I/O**: One additional file read per target file (cached)
- **Rendering**: Color codes add minimal overhead (<1ms)

---

## Sprint Retrospective Template

**What Went Well**:
-

**What Could Be Improved**:
-

**Action Items**:
-

---

## References

- **Sprint 61 Progress**: `docs/execution/SPRINT-61-PROGRESS.md`
- **Mutation Testing Handler**: `server/src/cli/handlers/mutate.rs`
- **Mutation Types**: `server/src/services/mutation/types.rs`
- **Console Crate Docs**: https://docs.rs/console/latest/console/
