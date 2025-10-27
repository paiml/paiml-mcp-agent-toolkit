# Sprint 64 Kickoff: Mutation Testing - Testing, Examples, and Documentation

**Sprint**: Sprint 64
**Target Version**: v2.177.0
**Duration**: 3 days
**Start Date**: October 27, 2025
**Focus**: Comprehensive testing infrastructure, example projects, CI/CD integration, and documentation

---

## Context

### Previous Sprint Completion
- **Sprint 63**: ✅ Complete - Multi-Language Mutation Testing Support (v2.176.0)
  - Centralized language detection (Language enum)
  - 6 languages supported (Rust, Python, TypeScript, JavaScript, Go, C++)
  - 19 comprehensive tests (100% passing)
  - Published to crates.io on October 27, 2025

### Current State
- **Mutation Testing Feature**: Core functionality implemented and released
- **Language Support**: 6 languages with centralized detection
- **Test Coverage**: Basic tests exist, but comprehensive test suite needed
- **Documentation**: Feature documented in CHANGELOG, needs user guides
- **Examples**: No example projects exist yet
- **CI/CD**: No integration guides available

---

## Sprint 64 Objectives

### Primary Goals
1. **Build Comprehensive Test Suite** for mutation testing feature
2. **Create Example Projects** demonstrating mutation testing in 3 languages
3. **Develop CI/CD Integration Guides** for popular platforms
4. **Establish Performance Benchmarks** and optimization targets
5. **Write User Documentation** and best practices guides

### Success Criteria
- ✅ Test coverage >85% for mutation feature
- ✅ 3 example projects created (Rust, Python, TypeScript)
- ✅ 3 CI/CD integration guides written (GitHub Actions, GitLab CI, Jenkins)
- ✅ Performance benchmarks established
- ✅ Mutation score badge generation implemented
- ✅ User guide and best practices documentation complete

---

## Day 1: Testing Infrastructure

### Objectives
- Implement comprehensive test suite for mutation testing
- Achieve >85% test coverage for mutation feature
- Establish testing patterns for future development

### Tasks

#### 1. Unit Tests for Mutation Handler (~50 tests)
**File**: `server/src/cli/handlers/mutate.rs`

**Test Categories**:
- Command argument parsing
- Output format selection (text, JSON, markdown)
- Failures-only filtering
- Color coding logic
- Error handling
- Progress indicator functionality

**Example Tests**:
```rust
#[test]
fn test_mutate_handler_text_output_format() { }

#[test]
fn test_mutate_handler_json_output_format() { }

#[test]
fn test_mutate_handler_markdown_output_format() { }

#[test]
fn test_mutate_handler_failures_only_filter() { }

#[test]
fn test_mutate_handler_invalid_target_error() { }

#[test]
fn test_mutate_handler_timeout_configuration() { }
```

#### 2. Integration Tests for Full Workflow (~20 tests)
**File**: `server/tests/mutation_integration_tests.rs`

**Test Scenarios**:
- End-to-end mutation testing for each supported language
- Multi-file mutation testing
- Large file handling (>1000 lines)
- Concurrent mutation execution
- Error recovery and resilience

**Example Tests**:
```rust
#[test]
fn test_rust_mutation_full_workflow() { }

#[test]
fn test_python_mutation_full_workflow() { }

#[test]
fn test_typescript_mutation_full_workflow() { }

#[test]
fn test_multi_file_mutation_testing() { }

#[test]
fn test_large_file_mutation_performance() { }
```

#### 3. Property-Based Tests with proptest (~10 tests)
**File**: `server/tests/mutation_property_tests.rs`

**Properties to Test**:
- Mutant generation is deterministic for same input
- All generated mutants are syntactically valid
- Mutation score is always between 0.0 and 1.0
- Killed mutant count ≤ total mutant count
- Output format consistency across languages

**Example Tests**:
```rust
proptest! {
    #[test]
    fn test_mutant_generation_deterministic(code: String) { }

    #[test]
    fn test_mutation_score_bounded(mutants: Vec<Mutant>) { }

    #[test]
    fn test_output_format_consistency(format: OutputFormat) { }
}
```

### Deliverables
- [ ] 50+ unit tests for mutation handler
- [ ] 20+ integration tests for full workflow
- [ ] 10+ property-based tests
- [ ] Test coverage report showing >85% coverage
- [ ] CI integration for automated test execution

---

## Day 2: Example Projects and CI/CD Integration

### Objectives
- Create working example projects for 3 languages
- Develop CI/CD integration guides for 3 platforms
- Demonstrate real-world mutation testing usage

### Tasks

#### 1. Rust Example Project
**Directory**: `examples/rust-mutation-testing/`

**Structure**:
```
examples/rust-mutation-testing/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs (calculator library with tests)
│   └── validator.rs (input validation)
├── tests/
│   └── integration_tests.rs
└── .github/
    └── workflows/
        └── mutation-testing.yml
```

**Features**:
- Simple library with 5-10 functions
- Comprehensive unit tests
- Integration tests
- GitHub Actions workflow for mutation testing
- Mutation score badge in README

#### 2. Python Example Project
**Directory**: `examples/python-mutation-testing/`

**Structure**:
```
examples/python-mutation-testing/
├── pyproject.toml
├── README.md
├── src/
│   ├── calculator.py
│   └── validator.py
├── tests/
│   ├── test_calculator.py
│   └── test_validator.py
└── .github/
    └── workflows/
        └── mutation-testing.yml
```

**Features**:
- Python package with pytest
- Type hints
- GitHub Actions workflow
- Mutation score tracking

#### 3. TypeScript Example Project
**Directory**: `examples/typescript-mutation-testing/`

**Structure**:
```
examples/typescript-mutation-testing/
├── package.json
├── tsconfig.json
├── README.md
├── src/
│   ├── calculator.ts
│   └── validator.ts
├── tests/
│   ├── calculator.test.ts
│   └── validator.test.ts
└── .github/
    └── workflows/
        └── mutation-testing.yml
```

**Features**:
- TypeScript project with Jest
- Type-safe implementation
- GitHub Actions workflow
- npm package structure

#### 4. CI/CD Integration Guides

**Guide 1: GitHub Actions** (`docs/guides/mutation-testing-github-actions.md`)
```yaml
name: Mutation Testing

on: [push, pull_request]

jobs:
  mutation-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install pmat
        run: cargo install pmat
      - name: Run mutation tests
        run: pmat mutate --target src/ --failures-only
      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: mutation-results
          path: mutation_results.json
```

**Guide 2: GitLab CI** (`docs/guides/mutation-testing-gitlab-ci.md`)
```yaml
mutation-testing:
  image: rust:latest
  stage: test
  script:
    - cargo install pmat
    - pmat mutate --target src/ --output-format json > mutation_results.json
  artifacts:
    reports:
      junit: mutation_results.json
```

**Guide 3: Jenkins** (`docs/guides/mutation-testing-jenkins.md`)
```groovy
pipeline {
    agent any
    stages {
        stage('Mutation Testing') {
            steps {
                sh 'cargo install pmat'
                sh 'pmat mutate --target src/ --failures-only'
            }
        }
    }
}
```

### Deliverables
- [ ] 3 complete example projects (Rust, Python, TypeScript)
- [ ] Each example project has README with setup instructions
- [ ] 3 CI/CD integration guides (GitHub Actions, GitLab CI, Jenkins)
- [ ] Each guide includes badge generation
- [ ] Examples demonstrate best practices

---

## Day 3: Performance Benchmarking and Documentation

### Objectives
- Establish performance benchmarks
- Implement mutation score badge generation
- Write comprehensive user documentation

### Tasks

#### 1. Performance Benchmarking
**File**: `server/benches/mutation_benchmarks.rs`

**Benchmarks**:
- Mutant generation speed (mutants/second)
- Large file processing (>1000 lines)
- Multi-file project analysis
- Language-specific performance comparisons
- Memory usage profiling

**Target Metrics**:
- Rust: >100 mutants/second
- Python: >50 mutants/second
- TypeScript: >50 mutants/second
- Memory: <500MB for 1000+ mutants

**Implementation**:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_rust_mutation(c: &mut Criterion) {
    c.bench_function("rust mutant generation", |b| {
        b.iter(|| {
            // Benchmark mutant generation for Rust code
        });
    });
}

criterion_group!(benches, benchmark_rust_mutation);
criterion_main!(benches);
```

#### 2. Mutation Score Badge Generation
**Feature**: Generate SVG badges for mutation scores

**Implementation**:
```rust
// server/src/services/mutation/badge_generator.rs

pub struct BadgeGenerator;

impl BadgeGenerator {
    pub fn generate_svg(score: f64) -> String {
        let color = match score {
            s if s >= 0.8 => "brightgreen",  // ≥80% = green
            s if s >= 0.6 => "yellow",        // 60-80% = yellow
            _ => "red",                        // <60% = red
        };

        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="140" height="20">
                <text x="10" y="14">Mutation Score: {:.1}%</text>
            </svg>"#,
            score * 100.0
        )
    }
}
```

**CLI Integration**:
```bash
# Generate badge SVG
pmat mutate --target src/ --output-badge mutation-score.svg

# Badge in README.md
![Mutation Score](./mutation-score.svg)
```

#### 3. User Documentation

**Guide 1**: `docs/guides/mutation-testing.md`
- Introduction to mutation testing
- How PMAT mutation testing works
- Command reference
- Output formats
- Best practices

**Guide 2**: `docs/guides/mutation-testing-best-practices.md`
- Writing testable code
- Interpreting mutation scores
- Common pitfalls
- Performance optimization
- CI/CD integration strategies

**Guide 3**: `examples/mutation_testing_workflow.md`
- Step-by-step workflow
- Real-world scenarios
- Troubleshooting guide
- FAQ

### Deliverables
- [ ] Performance benchmark suite
- [ ] Baseline performance metrics documented
- [ ] Badge generation feature implemented
- [ ] User guide (mutation-testing.md)
- [ ] Best practices guide
- [ ] Workflow examples

---

## Testing Strategy

### Unit Testing
- Focus on individual components
- Mock external dependencies
- Test edge cases and error conditions
- Aim for >90% code coverage

### Integration Testing
- Test end-to-end workflows
- Use real file system (temporary directories)
- Test all supported languages
- Validate output formats

### Property-Based Testing
- Use proptest for invariant testing
- Generate random test inputs
- Verify mathematical properties
- Ensure deterministic behavior

### Performance Testing
- Use criterion for benchmarking
- Establish baseline metrics
- Track performance regressions
- Profile memory usage

---

## Quality Gates

### Before Each Commit
1. ✅ Run clippy: `cargo clippy --all-targets --all-features`
2. ✅ Run tests: `cargo test --all-features`
3. ✅ Run benchmarks: `cargo bench` (Day 3)
4. ✅ Check test coverage: `cargo llvm-cov --all-features`

### Before Release (v2.177.0)
1. ✅ All tests passing (unit, integration, property-based)
2. ✅ Test coverage >85%
3. ✅ All example projects working
4. ✅ All CI/CD guides tested
5. ✅ Documentation reviewed and complete
6. ✅ Performance benchmarks meet targets
7. ✅ Update CHANGELOG.md
8. ✅ Update version in Cargo.toml

---

## Success Metrics

### Testing
- [ ] >50 unit tests for mutation handler
- [ ] >20 integration tests for workflows
- [ ] >10 property-based tests
- [ ] >85% test coverage

### Examples
- [ ] 3 example projects (Rust, Python, TypeScript)
- [ ] Each example has working CI/CD workflow
- [ ] Each example includes mutation score badge

### Documentation
- [ ] 3 CI/CD integration guides
- [ ] User guide and best practices
- [ ] API documentation complete

### Performance
- [ ] Benchmarks establish baseline metrics
- [ ] Performance competitive with existing tools
- [ ] Memory usage <500MB for large projects

---

## Risk Mitigation

### Risk 1: Test Implementation Complexity
**Mitigation**: Start with simple unit tests, progressively add complexity

### Risk 2: Example Project Scope Creep
**Mitigation**: Keep examples simple (5-10 functions each), focus on clarity

### Risk 3: Performance Benchmarking Variability
**Mitigation**: Run benchmarks multiple times, use statistical analysis

### Risk 4: Documentation Completeness
**Mitigation**: Follow template structure, peer review before completion

---

## Dependencies

### External Dependencies
- `proptest` - Property-based testing framework
- `criterion` - Benchmarking framework
- `tempfile` - Temporary file/directory creation for tests

### Internal Dependencies
- Mutation engine (existing)
- Language detection (v2.176.0)
- Output formatters (existing)

---

## Resources

### Code References
- **Mutation Handler**: `server/src/cli/handlers/mutate.rs`
- **Mutation Engine**: `server/src/services/mutation/engine.rs`
- **Language Detector**: `server/src/services/mutation/language_detector.rs`
- **Mutation Types**: `server/src/services/mutation/types.rs`

### Documentation
- **Sprint 62-64 Roadmap**: `docs/execution/SPRINT-62-64-ROADMAP.md`
- **Sprint 63 Kickoff**: `docs/execution/SPRINT-63-KICKOFF.md`
- **NEXT-STEPS**: `NEXT-STEPS.md`

---

## Daily Checklist

### Day 1
- [ ] Create test files for unit tests
- [ ] Implement 50+ unit tests
- [ ] Create integration test suite
- [ ] Implement 20+ integration tests
- [ ] Create property test suite
- [ ] Implement 10+ property tests
- [ ] Run coverage analysis
- [ ] Document test patterns

### Day 2
- [ ] Create Rust example project
- [ ] Create Python example project
- [ ] Create TypeScript example project
- [ ] Write GitHub Actions guide
- [ ] Write GitLab CI guide
- [ ] Write Jenkins guide
- [ ] Test all examples with CI/CD workflows

### Day 3
- [ ] Implement benchmark suite
- [ ] Run baseline benchmarks
- [ ] Implement badge generation
- [ ] Write user guide
- [ ] Write best practices guide
- [ ] Write workflow examples
- [ ] Final quality gate checks

---

## Contact

**Project Maintainer**: Noah Gift (@noahgift)
**Repository**: https://github.com/paiml/paiml-mcp-agent-toolkit
**Issues**: https://github.com/paiml/paiml-mcp-agent-toolkit/issues

---

**Created**: October 27, 2025
**Sprint Duration**: 3 days
**Target Version**: v2.177.0
**Previous Sprint**: Sprint 63 (v2.176.0 - Multi-Language Mutation Testing Support)
