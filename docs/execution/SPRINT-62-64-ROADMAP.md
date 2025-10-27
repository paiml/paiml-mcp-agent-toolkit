# Sprint 62-64 Roadmap - Mutation Testing Feature Completion

**Timeline**: 9 days total (3 days per sprint)

**Goal**: Complete mutation testing CLI feature with output refinement, multi-language support, and comprehensive testing

---

## Sprint 62 - Output Refinement (v2.175.0)

**Duration**: 3 days
**Target Release**: v2.175.0

### Objectives
- Enhanced mutant details in all output formats (text, JSON, markdown)
- `--failures-only` flag for CI/CD optimization
- Color-coded terminal output
- Multi-file size validation

### Deliverables
- ✅ Code snippets in text/JSON/markdown outputs
- ✅ Diff blocks for survived mutants in markdown
- ✅ `--failures-only` flag implementation
- ✅ Color coding with `console` crate
- ✅ Testing with small (37 mutants), medium (239 mutants), and large (500+ mutants) files
- ✅ Updated documentation and examples

### Key Features
```bash
# Enhanced output with mutant details
pmat mutate --target src/file.rs

# Show only failures (CI/CD optimization)
pmat mutate --target src/ --failures-only --output-format json

# Colored output (default) or disable colors
NO_COLOR=1 pmat mutate --target src/
```

### Files Modified
- `server/src/cli/commands.rs` - Add failures_only flag
- `server/src/cli/handlers/mutate.rs` - Enhanced outputs and color coding
- `server/src/services/mutation/types.rs` - Add code snippet fields
- `server/README.md` - Updated documentation
- `CHANGELOG.md` - v2.175.0 entry
- `examples/mutation_testing_workflow.md` - Workflow guide (NEW)

---

## Sprint 63 - Multi-Language Support (v2.176.0)

**Duration**: 3 days
**Target Release**: v2.176.0

### Objectives
- Extend mutation testing beyond Rust to Python, TypeScript, Go, and C++
- Automatic language detection based on file extension
- Language-specific mutation operators

### Day 1: Language Detection and Python Support
**Tasks**:
1. Implement language auto-detection in `MutationEngine`
   - File extension mapping: `.py` → Python, `.ts` → TypeScript, etc.
   - `--language` flag override for explicit language selection
2. Add Python mutation operators:
   - Binary operators: `+` ↔ `-`, `*` ↔ `/`, `==` ↔ `!=`
   - Boolean operators: `and` ↔ `or`, `True` ↔ `False`
   - Comparison operators: `<` ↔ `<=`, `>` ↔ `>=`
3. Test with Python file (create `test_sample.py` with basic functions)

### Day 2: TypeScript/JavaScript and Go Support
**Tasks**:
1. Add TypeScript/JavaScript mutation operators:
   - Leverage existing tree-sitter-typescript integration
   - Binary operators: `+` ↔ `-`, `&&` ↔ `||`
   - String operators: `===` ↔ `!==`
2. Add Go mutation operators:
   - Binary operators: `+` ↔ `-`, `*` ↔ `/`
   - Boolean operators: `&&` ↔ `||`, `true` ↔ `false`
3. Test with TypeScript and Go files

### Day 3: C++ Support and Integration Testing
**Tasks**:
1. Add C++ mutation operators (extend existing C support):
   - Leverage existing tree-sitter-cpp integration
   - Binary operators, pointer operators
2. Create integration tests for all 5 languages:
   - Rust, Python, TypeScript, Go, C++
3. Update documentation with language support matrix

### Deliverables
- ✅ Language auto-detection
- ✅ Python mutation operators and testing
- ✅ TypeScript/JavaScript mutation operators and testing
- ✅ Go mutation operators and testing
- ✅ C++ mutation operators and testing
- ✅ Integration tests for all languages
- ✅ Documentation update

### Key Features
```bash
# Auto-detect language from file extension
pmat mutate --target src/module.py

# Explicit language specification
pmat mutate --target script.txt --language python

# Multi-language directory scanning
pmat mutate --target src/ --output-format markdown
```

### Files Modified
- `server/src/services/mutation/engine.rs` - Language detection
- `server/src/services/mutation/python_mutations.rs` - Python operators (NEW)
- `server/src/services/mutation/typescript_mutations.rs` - TypeScript operators (NEW)
- `server/src/services/mutation/go_mutations.rs` - Go operators (NEW)
- `server/src/services/mutation/cpp_mutations.rs` - C++ operators (NEW)
- `server/tests/mutation_multi_language.rs` - Integration tests (NEW)
- `server/README.md` - Language support matrix
- `CHANGELOG.md` - v2.176.0 entry

---

## Sprint 64 - Testing, Examples, and Documentation (v2.177.0)

**Duration**: 3 days
**Target Release**: v2.177.0

### Objectives
- Comprehensive testing suite for mutation testing feature
- Real-world examples and use cases
- CI/CD integration guides
- Performance benchmarking

### Day 1: Unit and Integration Tests
**Tasks**:
1. Write unit tests for `handlers/mutate.rs`:
   - Test argument parsing
   - Test output formatting (text, JSON, markdown)
   - Test failures-only filtering
   - Test color coding
2. Write integration tests for full workflow:
   - End-to-end test: file → mutants → execution → output
   - Test with all 5 languages (Rust, Python, TypeScript, Go, C++)
   - Test threshold enforcement (pass/fail scenarios)
3. Property-based tests with `proptest`:
   - Mutant generation consistency
   - Output format validity (JSON schema, markdown syntax)

### Day 2: Examples and CI/CD Integration
**Tasks**:
1. Create example projects:
   - `examples/rust_mutation_testing/` - Rust project with tests
   - `examples/python_mutation_testing/` - Python project with pytest
   - `examples/typescript_mutation_testing/` - TypeScript project with Jest
2. Create CI/CD integration guides:
   - **GitHub Actions**: `.github/workflows/mutation-testing.yml` template
   - **GitLab CI**: `.gitlab-ci.yml` example
   - **Jenkins**: Jenkinsfile example
3. Add badge generation for mutation score:
   - Generate shields.io badge URL from mutation score
   - Example: `![Mutation Score](https://img.shields.io/badge/mutation-85%25-brightgreen)`

### Day 3: Performance Benchmarking and Documentation
**Tasks**:
1. Performance benchmarking:
   - Benchmark mutant generation speed (mutants/second)
   - Benchmark execution speed (mutants/minute)
   - Memory usage profiling
   - Compare with cargo-mutants for Rust files
2. Create comprehensive documentation:
   - **User Guide**: `docs/guides/mutation-testing.md`
   - **API Reference**: Document all CLI flags and options
   - **Best Practices**: When to use mutation testing, optimal thresholds
3. Update main README with mutation testing section

### Deliverables
- ✅ Unit tests for handler (~50 tests)
- ✅ Integration tests for full workflow (~20 tests)
- ✅ Property-based tests (~10 tests)
- ✅ Example projects for Rust, Python, TypeScript
- ✅ CI/CD integration guides (GitHub Actions, GitLab CI, Jenkins)
- ✅ Performance benchmarks
- ✅ Comprehensive documentation
- ✅ Badge generation

### Key Examples
```bash
# Run tests with threshold enforcement (CI/CD)
pmat mutate --target src/ --threshold 80.0 --failures-only --output-format json

# Generate markdown report for PR
pmat mutate --target src/ --output-format markdown > MUTATION_REPORT.md

# Performance mode (fast mutation testing)
pmat mutate --target src/ --jobs 8 --timeout 10
```

### Files Created
- `server/src/cli/handlers/mutate_tests.rs` - Unit tests (NEW)
- `server/tests/mutation_integration.rs` - Integration tests (NEW)
- `server/tests/mutation_property_tests.rs` - Property-based tests (NEW)
- `examples/rust_mutation_testing/` - Rust example project (NEW)
- `examples/python_mutation_testing/` - Python example project (NEW)
- `examples/typescript_mutation_testing/` - TypeScript example project (NEW)
- `examples/ci_cd/github_actions.yml` - GitHub Actions template (NEW)
- `examples/ci_cd/gitlab_ci.yml` - GitLab CI template (NEW)
- `examples/ci_cd/Jenkinsfile` - Jenkins example (NEW)
- `docs/guides/mutation-testing.md` - User guide (NEW)
- `docs/guides/mutation-testing-best-practices.md` - Best practices (NEW)
- `benchmarks/mutation_performance.rs` - Performance benchmarks (NEW)

---

## Post-Sprint 64: Future Enhancements (Backlog)

### v2.178.0+: Advanced Features
- **Incremental Mutation Testing**: Only mutate changed code (git diff integration)
- **Mutation Equivalence Detection**: Automatically detect equivalent mutants
- **Smart Mutant Selection**: Machine learning-based mutant prioritization
- **Distributed Execution**: Run mutation testing across multiple machines
- **IDE Integration**: VS Code extension for inline mutation scores
- **Coverage-Guided Mutation**: Only mutate covered code

### v2.180.0+: Language Expansion
- **Ruby**: Ruby mutation operators
- **Java**: Java mutation operators (leverage existing java-ast support)
- **Kotlin**: Kotlin mutation operators
- **C#**: C# mutation operators
- **Scala**: Scala mutation operators (leverage existing scala-ast support)
- **Swift**: Swift mutation operators

---

## Success Metrics

### Sprint 62 (Output Refinement)
- [ ] All output formats include code snippets
- [ ] `--failures-only` reduces output size by 70-90% for passing mutation scores
- [ ] Color coding improves terminal readability
- [ ] Large file testing (500+ mutants) completes successfully

### Sprint 63 (Multi-Language)
- [ ] 5 languages supported: Rust, Python, TypeScript, Go, C++
- [ ] Auto-detection works for all 5 languages
- [ ] Each language has 5-10 mutation operators
- [ ] Integration tests pass for all languages

### Sprint 64 (Testing & Documentation)
- [ ] Test coverage for mutation feature: >85%
- [ ] 3 example projects created
- [ ] 3 CI/CD integration guides written
- [ ] Performance benchmarks show competitive speed with cargo-mutants

---

## Risk Mitigation

### Technical Risks
- **Risk**: Python/TypeScript/Go/C++ mutation operators may be complex
  - **Mitigation**: Start with basic operators (binary, boolean), expand incrementally
- **Risk**: Large file performance may degrade with 500+ mutants
  - **Mitigation**: Implement batching, optimize memory usage, add progress caching

### Timeline Risks
- **Risk**: 3-day sprints may be too aggressive
  - **Mitigation**: Prioritize core features, defer advanced features to backlog
- **Risk**: Testing may uncover bugs requiring fixes
  - **Mitigation**: Allocate Day 3 buffer time in each sprint

---

## Dependencies

### Existing Infrastructure
- ✅ tree-sitter-python (Python AST parsing)
- ✅ tree-sitter-typescript (TypeScript AST parsing)
- ✅ tree-sitter-go (Go AST parsing)
- ✅ tree-sitter-cpp (C++ AST parsing)
- ✅ MutationEngine (Rust mutation engine)
- ✅ Progress indicators (implemented in Sprint 61)

### No New Dependencies Required
All features can be implemented with existing dependencies.

---

## Timeline Summary

| Sprint | Duration | Focus | Target Version |
|--------|----------|-------|----------------|
| Sprint 61 | 4 days (Days 1-4) | Command skeleton, output formats, progress | v2.174.0 ✅ |
| Sprint 62 | 3 days | Output refinement, failures-only, color coding | v2.175.0 |
| Sprint 63 | 3 days | Multi-language support (Python, TypeScript, Go, C++) | v2.176.0 |
| Sprint 64 | 3 days | Testing, examples, documentation, benchmarking | v2.177.0 |
| **Total** | **13 days** | **Complete mutation testing feature** | v2.177.0 |

---

## References

- **Sprint 61 Completion**: `docs/execution/SPRINT-61-PROGRESS.md`
- **Sprint 62 Kickoff**: `docs/execution/SPRINT-62-KICKOFF.md`
- **Mutation Engine**: `server/src/services/mutation/engine.rs`
- **Rust Mutations**: `server/src/services/mutation/rust_tree_sitter_mutations.rs`
- **CLI Handler**: `server/src/cli/handlers/mutate.rs`
