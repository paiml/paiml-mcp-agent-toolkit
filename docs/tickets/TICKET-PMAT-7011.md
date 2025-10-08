# PMAT-7011: Python AST-Based Mutation Testing

**Status**: 🚀 TODO
**Priority**: P0 - Critical (Multi-language Mutation Testing)
**Complexity**: Medium
**Estimated Duration**: 1 day (following TypeScript pattern)
**Sprint**: 24
**Created**: 2025-10-08
**Parent**: Multi-language Mutation Testing Initiative

---

## Objective

Implement **production-ready Python mutation testing** using tree-sitter AST transformation, following the proven architecture from TypeScript/JavaScript mutation testing (PMAT-7010).

**Key Goal**: Leverage language-agnostic `TreeSitterMutationOperator` trait to achieve 80%+ mutation score on Python test suites.

---

## Background

### Current State
- ✅ **Language-agnostic architecture** exists ([tree_sitter_operators.rs](server/src/services/mutation/tree_sitter_operators.rs:1))
- ✅ **TypeScript mutation testing** complete (PMAT-7010) - 67 mutants in 14ms, 80% score
- ✅ **Tree-sitter-python** parser available (v0.23)
- ✅ **Mutation operators** proven successful on TypeScript
- ⚠️ **Python adapter** exists but is a stub ([python_adapter.rs](server/src/services/mutation/python_adapter.rs:1))

### Gap
> "TypeScript mutation testing works, but Python/Go/C++ need implementations using the same architecture."

### Opportunity
- **Reuse 90% of TypeScript architecture** (operators, visitor pattern, test runner)
- **Python-specific features**: List comprehensions, decorators, `is/is not`, `in/not in`
- **Test framework support**: pytest, unittest, nose2 (auto-detection from requirements.txt)

---

## Scope

### In Scope

**1. Python Mutation Operators (5)**
- **Arithmetic Operator Replacement (AOR)**: `+`, `-`, `*`, `/`, `//`, `%`, `**`
- **Relational Operator Replacement (ROR)**: `<`, `>`, `<=`, `>=`, `==`, `!=`
- **Logical Operator Replacement (LOR)**: `and`, `or`
- **Identity Operator Mutation**: `is` ↔ `is not`, `==`
- **Membership Operator Mutation**: `in` ↔ `not in`

**2. Python Test Framework Integration**
- Auto-detect pytest, unittest, nose2 from `requirements.txt`
- Execute tests via `python -m pytest`, `python -m unittest`, etc.
- Parse test results (pass/fail)
- Handle virtual environments

**3. Tree-sitter Python AST**
- Parse Python source using tree-sitter-python
- Byte-level source splicing (preserves formatting)
- AST visitor pattern for mutation generation

**4. End-to-End Workflow**
- Generate mutants from Python source
- Run baseline tests
- Test each mutant
- Calculate mutation score
- Identify surviving mutants

### Out of Scope (Future)
- Advanced Python features (type hints, async/await, walrus operator)
- Multi-file mutation testing
- Python-specific optimizations (AST caching)
- ML predictor integration (PMAT-7004)

---

## Success Criteria

| Criterion | Target | Measurement |
|-----------|--------|-------------|
| **Mutation Operators** | 5 operators | AOR, ROR, LOR, Identity, Membership |
| **Generation Speed** | <100ms | For ~50 mutants |
| **Mutation Score** | 80%+ | On test fixture |
| **Test Execution** | Working | pytest/unittest auto-detection |
| **Compilation** | Zero errors | `cargo build --features python-ast` |
| **Test Coverage** | 7+ tests | All operators tested |
| **Documentation** | Complete | User guide + examples |

---

## Implementation Plan

### EXTREME TDD Methodology

Following PMAT-7010 proven process:

#### **RED Phase** (2-3 hours)
1. Create test fixtures:
   - `fixtures/python/calculator.py` (source code)
   - `fixtures/python/test_calculator.py` (pytest tests)
   - `fixtures/python/requirements.txt` (dependencies)
2. Write 7 failing tests (one per operator + integration)
3. Create stub implementations:
   - `python_tree_sitter_mutations.rs` (operators)
   - `python_mutation_generator.rs` (AST visitor)
4. Mark tests as `#[ignore]`
5. Document RED phase results

**Deliverables:**
- 7 failing tests
- Test fixtures
- Stub implementations
- RED phase documentation

#### **GREEN Phase** (3-4 hours)
1. Implement Python mutation operators:
   - `PythonBinaryOpMutation` (AOR, ROR)
   - `PythonLogicalOpMutation` (LOR)
   - `PythonIdentityOpMutation` (is/is not)
   - `PythonMembershipOpMutation` (in/not in)
2. Implement Python test runner:
   - Auto-detect test framework
   - Execute tests
   - Parse results
3. Implement AST visitor:
   - Parse Python source
   - Traverse AST
   - Apply operators
   - Generate mutants
4. Remove `#[ignore]` from tests
5. Verify all tests pass

**Deliverables:**
- ~600 LOC production code
- All tests passing
- Zero compilation errors
- GREEN phase documentation

#### **REFACTOR Phase** (2-3 hours)
1. Create end-to-end workflow example:
   - `examples/python_mutation_workflow.rs`
2. Run on real Python code (calculator.py)
3. Achieve 80%+ mutation score
4. Identify and document surviving mutants
5. Create comprehensive documentation:
   - User guide
   - Troubleshooting
   - Best practices

**Deliverables:**
- End-to-end workflow
- 80%+ mutation score
- Complete documentation
- Release-ready code

---

## Technical Architecture

### File Structure

**New Files:**
```
server/src/services/mutation/
├── python_tree_sitter_mutations.rs  # 5 mutation operators (~400 LOC)
└── python_mutation_generator.rs     # AST visitor (~180 LOC)

server/examples/
├── python_mutation_workflow.rs      # Sequential workflow (~230 LOC)
└── python_mutation_workflow_parallel.rs  # Parallel version (~270 LOC)

fixtures/python/
├── calculator.py                    # Test fixture (~80 LOC)
├── test_calculator.py               # pytest tests (~100 LOC)
└── requirements.txt                 # pytest dependency
```

**Modified Files:**
```
server/src/services/mutation/mod.rs     # Export Python modules
server/src/services/mutation/python_adapter.rs  # Test runner implementation
server/Cargo.toml                       # Enable python-ast feature
```

### Python Mutation Operators

#### 1. Arithmetic Operator Replacement (AOR)

```python
# Original
def add(a, b):
    return a + b

# Mutants
return a - b  # + → -
return a * b  # + → *
return a / b  # + → /
return a // b  # + → //
return a % b  # + → %
return a ** b  # + → **
```

**Tree-sitter Node Type:** `binary_operator` with operator `+`, `-`, `*`, `/`, `//`, `%`, `**`

**Implementation:**
```rust
pub struct PythonBinaryOpMutation;

impl TreeSitterMutationOperator for PythonBinaryOpMutation {
    fn can_mutate(&self, node: &Node, _source: &[u8]) -> bool {
        node.kind() == "binary_operator"
    }

    fn mutate(&self, node: &Node, source: &[u8]) -> Vec<MutatedSource> {
        // Find operator child node
        let operator_node = node.child_by_field_name("operator").unwrap();
        let op_text = &source[operator_node.byte_range()];

        let replacements = match op_text {
            b"+" => vec!["-", "*", "/", "//", "%", "**"],
            b"-" => vec!["+", "*", "/", "//", "%", "**"],
            b"*" => vec!["+", "-", "/", "//", "%", "**"],
            b"/" => vec!["+", "-", "*", "//", "%", "**"],
            b"//" => vec!["+", "-", "*", "/", "%", "**"],
            b"%" => vec!["+", "-", "*", "/", "//", "**"],
            b"**" => vec!["+", "-", "*", "/", "//", "%"],
            _ => return vec![],
        };

        replacements.into_iter().map(|new_op| {
            let mut mutated = source.to_vec();
            mutated.splice(operator_node.byte_range(), new_op.bytes());

            MutatedSource {
                source: String::from_utf8(mutated).unwrap(),
                description: format!("{} → {}",
                    String::from_utf8_lossy(op_text), new_op),
                location: SourceLocation::from_node(&operator_node),
            }
        }).collect()
    }
}
```

#### 2. Relational Operator Replacement (ROR)

```python
# Original
def is_greater(a, b):
    return a > b

# Mutants
return a < b   # > → <
return a >= b  # > → >=
return a <= b  # > → <=
return a == b  # > → ==
return a != b  # > → !=
```

**Tree-sitter Node Type:** `comparison_operator` with operators `<`, `>`, `<=`, `>=`, `==`, `!=`

#### 3. Logical Operator Replacement (LOR)

```python
# Original
def is_valid(x, y):
    return x > 0 and y > 0

# Mutants
return x > 0 or y > 0   # and → or
return x > 0            # and → left only
return y > 0            # and → right only
```

**Tree-sitter Node Type:** `boolean_operator` with operators `and`, `or`

#### 4. Identity Operator Mutation

```python
# Original
def is_none(value):
    return value is None

# Mutants
return value is not None  # is → is not
return value == None       # is → ==
```

**Tree-sitter Node Type:** `comparison_operator` with operators `is`, `is not`

#### 5. Membership Operator Mutation

```python
# Original
def contains(item, collection):
    return item in collection

# Mutants
return item not in collection  # in → not in
```

**Tree-sitter Node Type:** `comparison_operator` with operators `in`, `not in`

### Python Test Runner

```rust
impl MutationAdapter for PythonMutationAdapter {
    async fn run_tests(&self, source_file: &Path) -> Result<TestRunResult> {
        // 1. Detect test framework
        let project_root = find_python_project_root(source_file)?;
        let test_framework = detect_python_test_framework(&project_root)?;

        // 2. Execute tests
        let output = match test_framework {
            "pytest" => Command::new("python")
                .arg("-m")
                .arg("pytest")
                .arg("--tb=short")
                .current_dir(&project_root)
                .output()
                .await?,
            "unittest" => Command::new("python")
                .arg("-m")
                .arg("unittest")
                .arg("discover")
                .current_dir(&project_root)
                .output()
                .await?,
            _ => return Err(anyhow::anyhow!("Unsupported test framework")),
        };

        // 3. Parse results
        Ok(TestRunResult {
            success: output.status.success(),
            duration: std::time::Duration::from_secs(0),
            failures: parse_python_test_failures(&output.stdout, &output.stderr),
        })
    }
}

fn detect_python_test_framework(project_root: &Path) -> Result<String> {
    let requirements = project_root.join("requirements.txt");
    if requirements.exists() {
        let content = std::fs::read_to_string(&requirements)?;
        if content.contains("pytest") {
            return Ok("pytest".to_string());
        }
    }

    // Check for pytest.ini, setup.py, etc.
    if project_root.join("pytest.ini").exists() {
        return Ok("pytest".to_string());
    }

    Ok("unittest".to_string()) // Default
}
```

### AST Visitor Pattern

```rust
pub struct PythonMutationGenerator {
    operators: Vec<Box<dyn TreeSitterMutationOperator>>,
}

impl PythonMutationGenerator {
    pub fn with_default_operators() -> Self {
        Self {
            operators: vec![
                Box::new(PythonBinaryOpMutation),
                Box::new(PythonLogicalOpMutation),
                Box::new(PythonIdentityOpMutation),
                Box::new(PythonMembershipOpMutation),
            ],
        }
    }

    pub fn generate_mutants(&self, source: &str, file_path: &str) -> Result<Vec<Mutant>> {
        let tree = self.parse_python(source)?;
        let mut mutants = Vec::new();
        self.visit_node(&tree.root_node(), source.as_bytes(), &mut mutants, file_path);
        Ok(mutants)
    }

    fn parse_python(&self, source: &str) -> Result<Tree> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .map_err(|e| anyhow::anyhow!("Failed to set Python language: {}", e))?;
        parser.parse(source, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse Python source"))
    }

    fn visit_node(&self, node: &Node, source: &[u8], mutants: &mut Vec<Mutant>, file_path: &str) {
        // Apply all operators to current node
        for operator in &self.operators {
            if operator.can_mutate(node, source) {
                let mutations = operator.mutate(node, source);
                for mutation in mutations {
                    mutants.push(Mutant {
                        id: format!("{}_{}_{}:{}",
                            operator.name(),
                            file_path,
                            mutation.location.line,
                            mutation.location.column),
                        original_file: PathBuf::from(file_path),
                        mutated_source: mutation.source,
                        operator: map_operator_name_to_type(operator.name()),
                        location: mutation.location,
                        hash: format!("{:x}", Sha256::digest(&mutation.source)),
                        status: MutantStatus::Pending,
                    });
                }
            }
        }

        // Recursively visit children
        for child in node.children(&mut node.walk()) {
            self.visit_node(&child, source, mutants, file_path);
        }
    }
}
```

---

## Test Fixtures

### calculator.py
```python
"""Simple calculator for mutation testing."""

def add(a: int, b: int) -> int:
    """Add two numbers."""
    return a + b

def subtract(a: int, b: int) -> int:
    """Subtract b from a."""
    return a - b

def multiply(a: int, b: int) -> int:
    """Multiply two numbers."""
    return a * b

def divide(a: int, b: int) -> float:
    """Divide a by b."""
    if b == 0:
        raise ValueError("Division by zero")
    return a / b

def power(a: int, b: int) -> int:
    """Raise a to the power of b."""
    return a ** b

def is_positive(value: int) -> bool:
    """Check if value is positive."""
    return value > 0

def is_even(value: int) -> bool:
    """Check if value is even."""
    return value % 2 == 0

def logical_and(a: bool, b: bool) -> bool:
    """Logical AND."""
    return a and b

def logical_or(a: bool, b: bool) -> bool:
    """Logical OR."""
    return a or b

def is_none(value) -> bool:
    """Check if value is None."""
    return value is None

def contains(item, collection) -> bool:
    """Check if item is in collection."""
    return item in collection
```

### test_calculator.py
```python
"""Pytest tests for calculator."""
import pytest
from calculator import (
    add, subtract, multiply, divide, power,
    is_positive, is_even, logical_and, logical_or,
    is_none, contains
)

def test_add():
    assert add(2, 3) == 5
    assert add(-1, 1) == 0
    assert add(0, 0) == 0

def test_subtract():
    assert subtract(5, 3) == 2
    assert subtract(0, 5) == -5

def test_multiply():
    assert multiply(3, 4) == 12
    assert multiply(0, 5) == 0

def test_divide():
    assert divide(10, 2) == 5.0
    with pytest.raises(ValueError):
        divide(10, 0)

def test_power():
    assert power(2, 3) == 8
    assert power(5, 0) == 1

def test_is_positive():
    assert is_positive(5) == True
    assert is_positive(-5) == False
    assert is_positive(0) == False

def test_is_even():
    assert is_even(4) == True
    assert is_even(5) == False

def test_logical_and():
    assert logical_and(True, True) == True
    assert logical_and(True, False) == False
    assert logical_and(False, False) == False

def test_logical_or():
    assert logical_or(True, False) == True
    assert logical_or(False, False) == False

def test_is_none():
    assert is_none(None) == True
    assert is_none(0) == False
    assert is_none("") == False

def test_contains():
    assert contains(1, [1, 2, 3]) == True
    assert contains(4, [1, 2, 3]) == False
```

### requirements.txt
```
pytest>=7.0.0
```

---

## Testing Strategy

### Unit Tests (7 tests)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Remove after GREEN phase
    fn test_python_arithmetic_mutation() {
        let source = "result = a + b";
        let operator = PythonBinaryOpMutation;
        // ... test mutation generation
    }

    #[test]
    #[ignore]
    fn test_python_relational_mutation() {
        let source = "return a > b";
        let operator = PythonRelationalOpMutation;
        // ... test mutation generation
    }

    #[test]
    #[ignore]
    fn test_python_logical_mutation() {
        let source = "return a and b";
        let operator = PythonLogicalOpMutation;
        // ... test mutation generation
    }

    #[test]
    #[ignore]
    fn test_python_identity_mutation() {
        let source = "return value is None";
        let operator = PythonIdentityOpMutation;
        // ... test mutation generation
    }

    #[test]
    #[ignore]
    fn test_python_membership_mutation() {
        let source = "return item in collection";
        let operator = PythonMembershipOpMutation;
        // ... test mutation generation
    }

    #[test]
    #[ignore]
    fn test_python_test_runner() {
        // Test pytest execution
    }

    #[test]
    #[ignore]
    fn test_python_mutation_generator() {
        // Test end-to-end generation
    }
}
```

### Integration Test

```rust
// examples/python_mutation_workflow.rs
#[tokio::main]
async fn main() -> Result<()> {
    println!("🐍 Python Mutation Testing Workflow\n");

    // 1. Generate mutants
    let generator = PythonMutationGenerator::with_default_operators();
    let mutants = generator.generate_mutants(&source, "calculator.py")?;
    println!("   Generated: {} mutants\n", mutants.len());

    // 2. Run baseline tests
    let baseline_passed = run_tests(&project_root, None).await?;
    if !baseline_passed {
        println!("❌ Baseline tests failed!");
        return Ok(());
    }

    // 3. Test each mutant
    for mutant in mutants.iter_mut() {
        match test_mutant(&source_file, &project_root, &mutant.mutated_source).await {
            Ok(false) => mutant.status = MutantStatus::Killed,
            Ok(true) => mutant.status = MutantStatus::Survived,
            Err(_) => mutant.status = MutantStatus::Timeout,
        }
    }

    // 4. Calculate mutation score
    let killed = mutants.iter().filter(|m| m.status == MutantStatus::Killed).count();
    let total = mutants.len();
    let mutation_score = (killed * 100) / total;

    println!("🎯 Mutation Score: {}%", mutation_score);
    if mutation_score >= 80 {
        println!("✅ EXCELLENT! Test suite quality is high.");
    }

    Ok(())
}
```

---

## Documentation

### User Guide

**File:** `docs/features/PYTHON-MUTATION-TESTING.md`

**Sections:**
1. **Quick Start** - Installation and first run
2. **Mutation Operators** - Detailed explanation of each operator
3. **Understanding Mutation Scores** - Interpretation guide
4. **Surviving Mutants** - How to identify test weaknesses
5. **Example Project Structure** - Directory layout
6. **Advanced Usage** - Parallel execution, programmatic API
7. **Limitations & Known Issues** - Current constraints
8. **Troubleshooting** - Common problems and solutions
9. **Best Practices** - Usage recommendations
10. **Performance Expectations** - Benchmarks

---

## Success Metrics

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| **Compilation** | 0 errors | `cargo build --features python-ast` |
| **Tests** | 7/7 passing | `cargo test python_mutation` |
| **Mutants Generated** | 50+ | Run on calculator.py |
| **Generation Time** | <100ms | Benchmark |
| **Mutation Score** | 80%+ | Integration test |
| **Code Quality** | <7 CC avg | Complexity analysis |
| **Documentation** | Complete | User guide + examples |

---

## Risk Assessment

### Low Risk
- ✅ **Architecture proven** - TypeScript implementation successful
- ✅ **Tree-sitter-python mature** - v0.23 stable
- ✅ **Test frameworks standard** - pytest, unittest well-documented

### Medium Risk
- ⚠️ **Python syntax complexity** - More operators than TypeScript
- ⚠️ **Virtual environment handling** - May need venv detection
- ⚠️ **Test framework variations** - pytest vs unittest output differs

### Mitigation
- Start with simple operators (AOR, ROR)
- Use subprocess for test execution (isolates environment)
- Parse test output defensively (regex patterns)

---

## Timeline

**Total Estimated Duration:** 1 day (8 hours)

| Phase | Duration | Deliverables |
|-------|----------|--------------|
| **RED** | 2-3 hours | 7 failing tests, stubs, fixtures |
| **GREEN** | 3-4 hours | Passing tests, working operators |
| **REFACTOR** | 2-3 hours | End-to-end workflow, docs, 80% score |

---

## Dependencies

### Required
- ✅ tree-sitter 0.23
- ✅ tree-sitter-python 0.23
- ✅ `TreeSitterMutationOperator` trait (exists)
- ✅ Python 3.8+ (for testing)
- ✅ pytest (for test fixtures)

### Optional
- ML predictor integration (PMAT-7004) - Future
- Parallel execution (rayon) - Already available

---

## Related Tickets

- **PMAT-7010**: TypeScript/JavaScript Mutation Testing ✅ **COMPLETE**
- **PMAT-7004**: ML Mutation Predictor ✅ **COMPLETE** (ready for integration)
- **PMAT-7012**: Go Mutation Testing 🔜 **NEXT**
- **PMAT-7013**: C++ Mutation Testing 🔜 **FUTURE**
- **PMAT-7009**: Pattern Learning System ⏳ **IN PROGRESS**

---

## Acceptance Criteria

### Functional
- [  ] 5 Python mutation operators implemented
- [  ] pytest/unittest test execution working
- [  ] 50+ mutants generated from calculator.py
- [  ] 80%+ mutation score achieved
- [  ] Surviving mutants identified
- [  ] End-to-end workflow example working

### Technical
- [  ] Zero compilation errors
- [  ] 7/7 tests passing
- [  ] <100ms generation time
- [  ] <7 average cyclomatic complexity
- [  ] Tree-sitter 0.23 compatible

### Documentation
- [  ] User guide created (PYTHON-MUTATION-TESTING.md)
- [  ] README.md updated
- [  ] Roadmap updated
- [  ] Examples documented
- [  ] Troubleshooting guide included

### Release
- [  ] Code committed to git
- [  ] Version bumped (v2.152.0)
- [  ] Tag created and pushed
- [  ] Published to crates.io
- [  ] GitHub release created

---

## Notes

- **Follow TypeScript pattern exactly** - Proven successful
- **EXTREME TDD non-negotiable** - RED → GREEN → REFACTOR
- **80% mutation score target** - Same as TypeScript
- **Documentation critical** - Production release requires complete docs
- **Reuse existing code** - Don't reinvent, adapt TypeScript approach

---

**Created:** 2025-10-08
**Author:** PMAT Team
**Status:** Ready to start RED phase
**Estimated Completion:** 2025-10-08 EOD
