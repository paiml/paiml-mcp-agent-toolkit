# PMAT-7012: Go AST-Based Mutation Testing

**Status**: 🚀 TODO
**Priority**: P0 - Critical (Multi-language Mutation Testing)
**Complexity**: Medium
**Estimated Duration**: 1 day (following TypeScript/Python pattern)
**Sprint**: 24
**Created**: 2025-10-08
**Parent**: Multi-language Mutation Testing Initiative

---

## Objective

Implement **production-ready Go mutation testing** using tree-sitter AST transformation, following the proven architecture from TypeScript (PMAT-7010) and Python (PMAT-7011) mutation testing.

**Key Goal**: Leverage language-agnostic `TreeSitterMutationOperator` trait to achieve 80%+ mutation score on Go test suites.

---

## Background

### Current State
- ✅ **Language-agnostic architecture** exists ([tree_sitter_operators.rs](server/src/services/mutation/tree_sitter_operators.rs:1))
- ✅ **TypeScript mutation testing** complete (PMAT-7010) - 67 mutants in 14ms, 80% score
- ✅ **Python mutation testing** complete (PMAT-7011) - 56 mutants in 5.2ms, 80% score
- ✅ **Tree-sitter-go** parser available (v0.23)
- ✅ **Mutation operators** proven successful on 2 languages
- ⚠️ **Go adapter** exists but is a stub ([go_adapter.rs](server/src/services/mutation/go_adapter.rs:1))

### Gap
> "TypeScript and Python mutation testing work. Go is next in the multi-language mutation testing initiative."

### Opportunity
- **Reuse 90% of TypeScript/Python architecture** (operators, visitor pattern, test runner)
- **Go-specific features**: Bitwise operators, goroutines, defer, channels
- **Test framework**: Standard `go test` with `testing.T`
- **Performance**: Expected to be fastest yet (Go test startup is minimal)

---

## Scope

### In Scope

**1. Go Mutation Operators (6)**
- **Arithmetic Operator Replacement (AOR)**: `+`, `-`, `*`, `/`, `%`
- **Relational Operator Replacement (ROR)**: `<`, `>`, `<=`, `>=`, `==`, `!=`
- **Logical Operator Replacement (LOR)**: `&&`, `||`
- **Bitwise Operator Replacement (BOR)**: `&`, `|`, `^`, `<<`, `>>`
- **Unary Operator Mutation (UOR)**: `!`, `-`, `+`
- **Assignment Operator Mutation**: `=`, `+=`, `-=`, `*=`, `/=`

**2. Go Test Framework Integration**
- Execute tests via `go test`
- Parse test results (PASS/FAIL)
- Handle test output formatting
- Support subtests and table-driven tests

**3. Tree-sitter Go AST**
- Parse Go source using tree-sitter-go
- Byte-level source splicing (preserves formatting)
- AST visitor pattern for mutation generation

**4. End-to-End Workflow**
- Generate mutants from Go source
- Run baseline tests
- Test each mutant
- Calculate mutation score
- Identify surviving mutants

### Out of Scope (Future)
- Advanced Go features (goroutines, channels, select, defer)
- Interface mutation
- Multi-file mutation testing
- Go-specific optimizations (build caching)
- ML predictor integration (PMAT-7004)

---

## Success Criteria

| Criterion | Target | Measurement |
|-----------|--------|-------------|
| **Mutation Operators** | 6 operators | AOR, ROR, LOR, BOR, UOR, Assignment |
| **Generation Speed** | <5ms | For ~50 mutants (faster than Python) |
| **Mutation Score** | 80%+ | On test fixture |
| **Test Execution** | Working | `go test` integration |
| **Compilation** | Zero errors | `cargo build --features go-ast` |
| **Test Coverage** | 8+ tests | All operators tested |
| **Documentation** | Complete | User guide + examples |

---

## Implementation Plan

### EXTREME TDD Methodology

Following PMAT-7010/7011 proven process:

#### **RED Phase** (2-3 hours)
1. Create test fixtures:
   - `fixtures/go/calculator.go` (source code with package)
   - `fixtures/go/calculator_test.go` (Go tests)
   - `fixtures/go/go.mod` (module definition)
2. Write 8 failing tests (one per operator + integration)
3. Create stub implementations:
   - `go_tree_sitter_mutations.rs` (operators)
   - `go_mutation_generator.rs` (AST visitor)
4. Mark tests as `#[ignore]`
5. Document RED phase results

**Deliverables:**
- 8 failing tests
- Test fixtures
- Stub implementations
- RED phase documentation

#### **GREEN Phase** (3-4 hours)
1. Implement Go mutation operators:
   - `GoBinaryOpMutation` (AOR)
   - `GoRelationalOpMutation` (ROR)
   - `GoLogicalOpMutation` (LOR)
   - `GoBitwiseOpMutation` (BOR)
   - `GoUnaryOpMutation` (UOR)
   - `GoAssignmentOpMutation` (Assignment)
2. Implement Go test runner:
   - Execute `go test`
   - Parse output
   - Handle test errors
3. Implement AST visitor:
   - Parse Go source
   - Traverse AST
   - Apply operators
   - Generate mutants
4. Remove `#[ignore]` from tests
5. Verify all tests pass

**Deliverables:**
- ~700 LOC production code
- All tests passing
- Zero compilation errors
- GREEN phase documentation

#### **REFACTOR Phase** (2-3 hours)
1. Create end-to-end workflow example:
   - `examples/go_mutation_workflow.rs`
2. Run on real Go code (calculator.go)
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
├── go_tree_sitter_mutations.rs      # 6 mutation operators (~600 LOC)
└── go_mutation_generator.rs         # AST visitor (~200 LOC)

server/examples/
└── go_mutation_workflow.rs          # Sequential workflow (~250 LOC)

fixtures/go/
├── calculator.go                    # Test fixture (~100 LOC)
├── calculator_test.go               # Go tests (~150 LOC)
└── go.mod                           # Module definition
```

**Modified Files:**
```
server/src/services/mutation/mod.rs     # Export Go modules
server/src/services/mutation/go_adapter.rs  # Test runner implementation
server/Cargo.toml                       # Add tree-sitter-go dependency
```

### Go Mutation Operators

#### 1. Arithmetic Operator Replacement (AOR)

```go
// Original
func Add(a, b int) int {
    return a + b
}

// Mutants
return a - b  // + → -
return a * b  // + → *
return a / b  // + → /
return a % b  // + → %
```

**Tree-sitter Node Type:** `binary_expression` with operator `+`, `-`, `*`, `/`, `%`

**Implementation:**
```rust
pub struct GoBinaryOpMutation;

impl TreeSitterMutationOperator for GoBinaryOpMutation {
    fn name(&self) -> &str {
        "GoBinaryOp"
    }

    fn can_mutate(&self, node: &Node, _source: &[u8]) -> bool {
        node.kind() == "binary_expression"
    }

    fn mutate(&self, node: &Node, source: &[u8]) -> Vec<MutatedSource> {
        // Find operator child node (middle child in binary_expression)
        let mut cursor = node.walk();
        let mut operator_node = None;

        for child in node.children(&mut cursor) {
            let kind = child.kind();
            if matches!(kind, "+" | "-" | "*" | "/" | "%") {
                operator_node = Some(child);
                break;
            }
        }

        let operator_node = match operator_node {
            Some(n) => n,
            None => return vec![],
        };

        let op_bytes = &source[operator_node.byte_range()];
        let op_text = std::str::from_utf8(op_bytes).unwrap_or("");

        let replacements = match op_text {
            "+" => vec!["-", "*", "/", "%"],
            "-" => vec!["+", "*", "/", "%"],
            "*" => vec!["+", "-", "/", "%"],
            "/" => vec!["+", "-", "*", "%"],
            "%" => vec!["+", "-", "*", "/"],
            _ => return vec![],
        };

        replacements
            .into_iter()
            .map(|new_op| {
                let mut mutated = source.to_vec();
                mutated.splice(operator_node.byte_range(), new_op.bytes());

                MutatedSource {
                    source: String::from_utf8(mutated).unwrap(),
                    description: format!("{} → {}", op_text, new_op),
                    location: SourceLocation {
                        line: operator_node.start_position().row + 1,
                        column: operator_node.start_position().column + 1,
                        end_line: operator_node.end_position().row + 1,
                        end_column: operator_node.end_position().column + 1,
                    },
                }
            })
            .collect()
    }

    fn kill_probability(&self) -> f64 {
        0.85
    }
}
```

#### 2. Relational Operator Replacement (ROR)

```go
// Original
func IsPositive(value int) bool {
    return value > 0
}

// Mutants
return value < 0   // > → <
return value >= 0  // > → >=
return value <= 0  // > → <=
return value == 0  // > → ==
return value != 0  // > → !=
```

**Tree-sitter Node Type:** `binary_expression` with comparison operators

#### 3. Logical Operator Replacement (LOR)

```go
// Original
func BothPositive(a, b int) bool {
    return a > 0 && b > 0
}

// Mutants
return a > 0 || b > 0  // && → ||
```

**Tree-sitter Node Type:** `binary_expression` with operators `&&`, `||`

#### 4. Bitwise Operator Replacement (BOR)

```go
// Original
func BitwiseAnd(a, b int) int {
    return a & b
}

// Mutants
return a | b   // & → |
return a ^ b   // & → ^
return a << b  // & → <<
return a >> b  // & → >>
```

**Tree-sitter Node Type:** `binary_expression` with operators `&`, `|`, `^`, `<<`, `>>`

#### 5. Unary Operator Mutation (UOR)

```go
// Original
func Negate(value int) int {
    return -value
}

// Mutants
return +value  // - → +
return value   // - → (remove)

// Original
func Not(flag bool) bool {
    return !flag
}

// Mutants
return flag    // ! → (remove)
```

**Tree-sitter Node Type:** `unary_expression` with operators `!`, `-`, `+`

#### 6. Assignment Operator Mutation

```go
// Original
func Increment(value int) int {
    value += 5
    return value
}

// Mutants
value -= 5  // += → -=
value *= 5  // += → *=
value /= 5  // += → /=
```

**Tree-sitter Node Type:** Various assignment expression nodes

---

## Test Framework Integration

### Go Test Execution

```rust
async fn run_go_tests(project_root: &PathBuf) -> Result<bool> {
    // Check if Go is available
    let has_go = Command::new("go")
        .arg("version")
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_go {
        println!("   ⚠️  Go not installed, skipping test execution");
        return Ok(true); // Assume tests would pass
    }

    let output = Command::new("go")
        .arg("test")
        .arg("-v")
        .current_dir(project_root)
        .output()
        .await
        .context("Failed to run Go tests")?;

    Ok(output.status.success())
}
```

### Go Test Output Parsing

```
=== RUN   TestAdd
--- PASS: TestAdd (0.00s)
=== RUN   TestSubtract
--- PASS: TestSubtract (0.00s)
PASS
ok  	calculator	0.001s
```

**Success Indicators:**
- Exit code 0
- Output contains "PASS"
- No "FAIL" markers

---

## Go Test Fixtures

### fixtures/go/calculator.go

```go
package calculator

// Add returns the sum of two integers
func Add(a, b int) int {
	return a + b
}

// Subtract returns the difference of two integers
func Subtract(a, b int) int {
	return a - b
}

// Multiply returns the product of two integers
func Multiply(a, b int) int {
	return a * b
}

// Divide returns the quotient of two integers
func Divide(a, b int) int {
	if b == 0 {
		panic("division by zero")
	}
	return a / b
}

// IsPositive checks if a number is positive
func IsPositive(value int) bool {
	return value > 0
}

// BothPositive checks if both numbers are positive
func BothPositive(a, b int) bool {
	return a > 0 && b > 0
}

// BitwiseAnd performs bitwise AND
func BitwiseAnd(a, b int) int {
	return a & b
}

// BitwiseOr performs bitwise OR
func BitwiseOr(a, b int) int {
	return a | b
}

// Negate negates a number
func Negate(value int) int {
	return -value
}

// Not returns logical NOT
func Not(flag bool) bool {
	return !flag
}
```

### fixtures/go/calculator_test.go

```go
package calculator

import "testing"

func TestAdd(t *testing.T) {
	tests := []struct {
		name     string
		a, b     int
		expected int
	}{
		{"positive numbers", 2, 3, 5},
		{"negative numbers", -1, -1, -2},
		{"zero", 0, 5, 5},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := Add(tt.a, tt.b)
			if result != tt.expected {
				t.Errorf("Add(%d, %d) = %d; want %d", tt.a, tt.b, result, tt.expected)
			}
		})
	}
}

func TestSubtract(t *testing.T) {
	if result := Subtract(5, 3); result != 2 {
		t.Errorf("Subtract(5, 3) = %d; want 2", result)
	}
}

func TestMultiply(t *testing.T) {
	if result := Multiply(4, 5); result != 20 {
		t.Errorf("Multiply(4, 5) = %d; want 20", result)
	}
}

func TestDivide(t *testing.T) {
	if result := Divide(10, 2); result != 5 {
		t.Errorf("Divide(10, 2) = %d; want 5", result)
	}
}

func TestIsPositive(t *testing.T) {
	tests := []struct {
		value    int
		expected bool
	}{
		{5, true},
		{0, false},
		{-5, false},
	}

	for _, tt := range tests {
		if result := IsPositive(tt.value); result != tt.expected {
			t.Errorf("IsPositive(%d) = %v; want %v", tt.value, result, tt.expected)
		}
	}
}

func TestBothPositive(t *testing.T) {
	tests := []struct {
		a, b     int
		expected bool
	}{
		{5, 10, true},
		{5, -1, false},
		{-1, 5, false},
		{-1, -1, false},
	}

	for _, tt := range tests {
		if result := BothPositive(tt.a, tt.b); result != tt.expected {
			t.Errorf("BothPositive(%d, %d) = %v; want %v", tt.a, tt.b, result, tt.expected)
		}
	}
}

func TestBitwiseAnd(t *testing.T) {
	if result := BitwiseAnd(6, 3); result != 2 {
		t.Errorf("BitwiseAnd(6, 3) = %d; want 2", result)
	}
}

func TestBitwiseOr(t *testing.T) {
	if result := BitwiseOr(6, 3); result != 7 {
		t.Errorf("BitwiseOr(6, 3) = %d; want 7", result)
	}
}

func TestNegate(t *testing.T) {
	tests := []struct {
		value    int
		expected int
	}{
		{5, -5},
		{-5, 5},
		{0, 0},
	}

	for _, tt := range tests {
		if result := Negate(tt.value); result != tt.expected {
			t.Errorf("Negate(%d) = %d; want %d", tt.value, result, tt.expected)
		}
	}
}

func TestNot(t *testing.T) {
	if result := Not(true); result != false {
		t.Errorf("Not(true) = %v; want false", result)
	}
	if result := Not(false); result != true {
		t.Errorf("Not(false) = %v; want true", result)
	}
}
```

### fixtures/go/go.mod

```
module calculator

go 1.21
```

---

## Performance Expectations

| Language | Mutants | Generation Time | Test Time (per mutant) |
|----------|---------|-----------------|------------------------|
| TypeScript | 67 | 14ms | ~1,800ms |
| Python | 56 | 5.2ms | ~17ms |
| **Go** | **~60** | **<5ms (expected)** | **<10ms (expected)** |

**Why Go will be fastest:**
- `go test` has minimal startup overhead
- No interpreter/runtime startup (compiled)
- Fast test execution
- Efficient toolchain

---

## Dependencies

### Cargo.toml

```toml
[dependencies]
tree-sitter-go = { version = "0.23", optional = true }

[features]
go-ast = ["rust-ast", "tree-sitter", "tree-sitter-go"]
```

---

## Risks & Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| Go AST differences | Medium | Study tree-sitter-go grammar first |
| Test output parsing | Low | Use standard `go test` output format |
| Module system complexity | Low | Use simple single-module fixtures |
| Performance expectations | Low | May be slower than predicted, acceptable |

---

## Acceptance Criteria

✅ **Code Quality:**
- [ ] Zero compilation errors
- [ ] Zero compiler warnings
- [ ] All tests passing (8+ tests)
- [ ] Code follows PMAT patterns

✅ **Functionality:**
- [ ] 6 mutation operators implemented
- [ ] Go test execution working
- [ ] Mutation score calculation accurate
- [ ] End-to-end workflow complete

✅ **Performance:**
- [ ] Generation <5ms for ~60 mutants
- [ ] Test execution functional
- [ ] Overall workflow sub-second

✅ **Documentation:**
- [ ] User guide complete
- [ ] Code examples working
- [ ] Troubleshooting guide
- [ ] README.md updated

---

## Future Enhancements (Not in MVP)

1. **Goroutine Mutation**: Mutate concurrency patterns
2. **Channel Operation Mutation**: Mutate channel sends/receives
3. **Defer/Panic/Recover**: Mutate error handling
4. **Interface Mutation**: Mutate interface implementations
5. **Generics Mutation**: Mutate type parameters (Go 1.18+)
6. **Test Table Optimization**: Smart test case selection
7. **Build Cache Optimization**: Reuse Go build cache

---

## Related Work

- **PMAT-7010**: TypeScript Mutation Testing (Complete)
- **PMAT-7011**: Python Mutation Testing (Complete)
- **PMAT-7013**: C++ Mutation Testing 🔜 **AFTER GO**
- **PMAT-7014**: Rust Mutation Testing (Internal) 🔜
- **PMAT-7004**: ML Predictor Integration 🔜

---

## Definition of Done

- [x] Ticket created and reviewed
- [ ] RED phase complete (8 failing tests)
- [ ] GREEN phase complete (all tests passing)
- [ ] REFACTOR phase complete (workflow + docs)
- [ ] 80%+ mutation score achieved
- [ ] Documentation merged
- [ ] Code reviewed and merged
- [ ] Release notes updated (v2.153.0)
- [ ] Roadmap updated

---

**Created**: 2025-10-08
**Author**: PMAT Team
**Methodology**: EXTREME TDD (RED → GREEN → REFACTOR)
**Quality Standard**: Toyota Way (ALL DEFECTS FIXED)
