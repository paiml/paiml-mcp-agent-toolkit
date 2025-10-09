# Go Mutation Testing

**Status:** ✅ Production Ready (v2.153.0+)
**Languages:** Go 1.21+
**Quality:** 80%+ mutation score achievable

---

## Overview

PMAT provides **AST-based mutation testing** for Go projects, helping you validate test suite quality by introducing controlled bugs (mutations) and checking if your tests catch them.

**Key Benefits:**
- 🎯 Quantify test quality with mutation scores
- 🔍 Identify gaps in test coverage
- ⚡ Ultra-fast generation (<3ms for ~60 mutants) - **Fastest yet!**
- 🧬 6 mutation operators covering common bug patterns
- 🔄 Works with standard `go test`
- 🔷 Go-specific operators: bitwise, assignment, unary

---

## Quick Start

### 1. Create Go Module

```bash
# In your project
go mod init myproject
```

### 2. Run Mutation Testing

```bash
# From server directory
cargo run --example go_mutation_workflow --features go-ast

# Or build the example first
cargo build --example go_mutation_workflow --features go-ast
./target/debug/examples/go_mutation_workflow
```

### 3. View Results

```
🔷 Go Mutation Testing Workflow

📝 Reading source file: ../fixtures/go/calculator.go
   Size: 2,847 bytes

🔧 Generating mutants...
   Generated: 62 mutants
   Time: 2.8ms

✅ Running baseline tests...
   Baseline tests passed ✅

🧪 Testing mutants (62 total)...
   [Progress updates...]

📊 Mutation Testing Results
   Total Mutants:    62
   Killed:           50 (80%)
   Survived:         12 (19%)
   Timeout/Error:    0

🎯 Mutation Score: 80% ✅ EXCELLENT!
```

---

## Mutation Operators

### 1. Arithmetic Operator Replacement (AOR)

**Replaces:** `+`, `-`, `*`, `/`, `%`

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

**Tests should fail** when arithmetic operators are changed.

### 2. Relational Operator Replacement (ROR)

**Replaces:** `<`, `>`, `<=`, `>=`, `==`, `!=`

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

**Tests should fail** when comparison logic changes.

### 3. Logical Operator Replacement (LOR)

**Replaces:** `&&` ↔ `||`

```go
// Original
func BothPositive(a, b int) bool {
    return a > 0 && b > 0
}

// Mutant
return a > 0 || b > 0  // && → ||

// Original
func EitherPositive(a, b int) bool {
    return a > 0 || b > 0
}

// Mutant
return a > 0 && b > 0  // || → &&
```

**Tests should fail** when logical operators are swapped.

### 4. Bitwise Operator Replacement (BOR) 🆕

**Replaces:** `&`, `|`, `^`, `<<`, `>>`

```go
// Original
func BitwiseAnd(a, b int) int {
    return a & b
}

// Mutants
return a | b   // & → |  (OR)
return a ^ b   // & → ^  (XOR)
return a << b  // & → << (left shift)
return a >> b  // & → >> (right shift)

// Original
func LeftShift(a, shift int) int {
    return a << shift
}

// Mutant
return a >> shift  // << → >>
```

**Tests should fail** when bitwise operations change.

**Use Cases:**
- Low-level bit manipulation
- Flag operations
- Performance-critical code
- Cryptographic operations

### 5. Unary Operator Replacement (UOR) 🆕

**Replaces:** `-` ↔ `+`, `!` (limited)

```go
// Original
func Negate(value int) int {
    return -value
}

// Mutant
return +value  // - → +

// Original
func Positive(value int) int {
    return +value
}

// Mutant
return -value  // + → -

// Note: ! operator not mutated (type safety)
func Not(flag bool) bool {
    return !flag  // No mutations (bool ≠ int)
}
```

**Tests should fail** when sign changes.

### 6. Assignment Operator Replacement 🆕

**Replaces:** `+=`, `-=`, `*=`, `/=`, `%=`, `&=`, `|=`, `^=`, `<<=`, `>>=`

```go
// Original
func AddAssign(value, delta int) int {
    value += delta
    return value
}

// Mutants
value -= delta  // += → -=
value *= delta  // += → *=
value /= delta  // += → /=

// Original
func BitwiseOrAssign(value, mask int) int {
    value |= mask
    return value
}

// Mutants
value &= mask  // |= → &=
value ^= mask  // |= → ^=
```

**Tests should fail** when compound assignment changes.

---

## Understanding Mutation Scores

### Score Interpretation

| Score | Quality | Recommendation |
|-------|---------|----------------|
| **90-100%** | Excellent | Maintain current quality |
| **80-89%** | Good | Minor improvements needed |
| **70-79%** | Acceptable | Add targeted tests |
| **60-69%** | Weak | Significant gaps exist |
| **< 60%** | Poor | Major test suite overhaul needed |

### What Mutation Scores Tell You

**High Score (80%+):**
- ✅ Tests catch most bugs
- ✅ Good coverage of edge cases
- ✅ Operator behavior validated
- ✅ Error conditions tested

**Low Score (<70%):**
- ❌ Tests miss common bug patterns
- ❌ Weak edge case coverage
- ❌ Type checks not validated
- ❌ Happy path bias

---

## Surviving Mutants (Test Weaknesses)

Surviving mutants indicate **real gaps** in your test suite:

### Example: Boundary Condition Gap

**Mutant:** `>` → `>=` (survives)

```go
// Code
func IsPositive(value int) bool {
    return value > 0
}

// Test (weak!)
func TestIsPositive(t *testing.T) {
    if !IsPositive(5) {
        t.Error("Expected true for 5")
    }
    if IsPositive(-5) {
        t.Error("Expected false for -5")
    }
}
// Passes even with >= because 5 > 0 and -5 < 0

// Better test
func TestIsPositiveBoundary(t *testing.T) {
    if IsPositive(0) {  // Would fail with >=
        t.Error("Expected false for 0")
    }
}
```

### Example: Logical Operator Gap

**Mutant:** `&&` → `||` (survives)

```go
// Code
func BothPositive(a, b int) bool {
    return a > 0 && b > 0
}

// Test (weak!)
func TestBothPositive(t *testing.T) {
    if !BothPositive(5, 10) {
        t.Error("Expected true for (5, 10)")
    }
}
// Passes even with || because both are positive

// Better test
func TestBothPositiveOneNegative(t *testing.T) {
    tests := []struct {
        a, b     int
        expected bool
    }{
        {5, -1, false},  // Would fail with ||
        {-1, 5, false},  // Would fail with ||
        {5, 10, true},
    }
    for _, tt := range tests {
        if got := BothPositive(tt.a, tt.b); got != tt.expected {
            t.Errorf("BothPositive(%d, %d) = %v; want %v",
                tt.a, tt.b, got, tt.expected)
        }
    }
}
```

### Example: Bitwise Operation Gap

**Mutant:** `&` → `|` (survives)

```go
// Code
func BitwiseAnd(a, b int) int {
    return a & b
}

// Test (weak!)
func TestBitwiseAnd(t *testing.T) {
    result := BitwiseAnd(15, 15)  // 1111 & 1111 = 1111
    if result != 15 {
        t.Errorf("Expected 15, got %d", result)
    }
}
// Passes even with | because 15 & 15 == 15 | 15

// Better test
func TestBitwiseAndVsOr(t *testing.T) {
    result := BitwiseAnd(6, 3)  // 110 & 011 = 010 (2)
    if result != 2 {
        t.Errorf("Expected 2, got %d", result)  // Would fail with | (gives 7)
    }
}
```

### Example: Assignment Operator Gap

**Mutant:** `+=` → `*=` (survives)

```go
// Code
func AddAssign(value, delta int) int {
    value += delta
    return value
}

// Test (weak!)
func TestAddAssign(t *testing.T) {
    result := AddAssign(0, 5)
    if result != 5 {
        t.Errorf("Expected 5, got %d", result)
    }
}
// Passes even with *= because 0 + 5 == 0 * 5 when value starts at 0

// Better test
func TestAddAssignNonZero(t *testing.T) {
    result := AddAssign(10, 5)  // 10 + 5 = 15
    if result != 15 {
        t.Errorf("Expected 15, got %d", result)  // Would fail with *= (gives 50)
    }
}
```

---

## Example Project Structure

```
my-go-project/
├── calculator.go          # Source code
├── calculator_test.go     # Tests
└── go.mod                 # Module definition
```

### Minimal go.mod

```
module myproject

go 1.21
```

### Test File Pattern

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
                t.Errorf("Add(%d, %d) = %d; want %d",
                    tt.a, tt.b, result, tt.expected)
            }
        })
    }
}
```

---

## Advanced Usage

### Programmatic Usage

```rust
use pmat::services::mutation::GoMutationGenerator;

// Generate mutants
let generator = GoMutationGenerator::with_default_operators();
let mutants = generator.generate_mutants(&source, "test.go")?;

// Process mutants
for mutant in mutants {
    println!("Mutant: {} at line {}", mutant.id, mutant.location.line);
}
```

### Custom Operator Selection

```rust
use pmat::services::mutation::go_tree_sitter_mutations::*;

let generator = GoMutationGenerator {
    operators: vec![
        Box::new(GoBinaryOpMutation),      // Only arithmetic
        Box::new(GoRelationalOpMutation),  // Only comparisons
        Box::new(GoBitwiseOpMutation),     // Only bitwise
    ],
};
```

---

## Limitations & Known Issues

### Current Limitations

1. **Single-file testing** - Multi-file projects not yet supported
2. **`go test` startup overhead** - Each mutant restarts test framework (~5ms per mutant)
3. **No test selection** - Runs all tests for each mutant
4. **Sequential execution** - No parallel testing yet

### Workarounds

**Speed up testing:**
```bash
# Use -short flag for faster tests
go test -short

# Or create a separate test suite for mutation testing
go test -run TestMutation
```

**Reduce mutants:**
- Focus on critical files
- Use smaller test suites during development
- Run full mutation testing in CI

### Future Enhancements

- [ ] Multi-file project support
- [ ] `go test` keep-alive (3-4x speedup)
- [ ] Smart test selection (2-5x speedup)
- [ ] Parallel execution (8x speedup)
- [ ] HTML reports
- [ ] CI/CD integration
- [ ] Additional operators (channel operations, goroutines, defer)

---

## Troubleshooting

### "Go not installed"

**Problem:** Go not found in environment

**Solution:**
```bash
# Install Go
# https://go.dev/dl/

# Verify installation
go version
```

### "Baseline tests failed"

**Problem:** Tests fail on original code

**Solution:**
```bash
# Fix tests first
go test -v

# Then run mutation testing
cargo run --example go_mutation_workflow
```

### "No mutants generated"

**Problem:** Code doesn't match mutation patterns

**Solution:**
- Ensure code has operators (+, -, *, /, &&, ||, etc.)
- Check for comparison operators (<, >, ==, etc.)
- Verify Go syntax is valid
- Check that file is valid Go 1.21+ code

### "Module not found" errors

**Problem:** go.mod missing or incorrect

**Solution:**
```bash
# Initialize module
go mod init myproject

# Download dependencies
go mod tidy
```

---

## Best Practices

### 1. Start with Small Files

```bash
# Good: Single file, focused tests
calculator.go + calculator_test.go (62 mutants, 30 seconds)

# Avoid: Large files initially
entire_app.go (1000+ mutants, 5+ minutes)
```

### 2. Use Table-Driven Tests

```go
// Good: Table-driven tests catch more mutants
func TestAdd(t *testing.T) {
    tests := []struct {
        a, b, expected int
    }{
        {2, 3, 5},
        {0, 0, 0},
        {-1, 1, 0},
    }

    for _, tt := range tests {
        if got := Add(tt.a, tt.b); got != tt.expected {
            t.Errorf("Add(%d, %d) = %d; want %d",
                tt.a, tt.b, got, tt.expected)
        }
    }
}

// Weak: Single test case
func TestAdd(t *testing.T) {
    if Add(2, 3) != 5 {
        t.Error("Addition failed")
    }
}
```

### 3. Test Boundary Conditions

```go
// Always test boundaries
func TestIsPositive(t *testing.T) {
    tests := []struct {
        value    int
        expected bool
    }{
        {1, true},   // Positive
        {0, false},  // Boundary - kills > → >= mutant
        {-1, false}, // Negative
    }
    // ...
}
```

### 4. Use in CI/CD

```yaml
# .github/workflows/mutation-testing.yml
name: Mutation Testing
on: [pull_request]

jobs:
  mutation:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions/setup-go@v2
        with:
          go-version: '1.21'
      - run: cargo run --example go_mutation_workflow
      - run: |
          # Fail if mutation score < 70%
          if [ $MUTATION_SCORE -lt 70 ]; then exit 1; fi
```

---

## Performance Expectations

### Typical Performance

| Mutants | Sequential | Optimized (Future) |
|---------|------------|-------------------|
| 10 | ~0.05s | ~0.01s |
| 50 | ~0.25s | ~0.05s |
| 100 | ~0.5s | ~0.1s |
| 500 | ~2.5s | ~0.5s |

**Note:** Times assume ~5ms per mutant (current) or ~1ms (optimized)

Go mutation testing is **THE FASTEST** because:
- No interpreter startup (compiled language)
- `go test` has minimal overhead
- No npm/node.js delays
- Efficient binary execution
- Sub-3ms mutant generation

### Scaling Recommendations

**Small projects (<100 mutants):** Run on every commit
**Medium projects (100-500 mutants):** Run on PRs
**Large projects (500+ mutants):** Run nightly or on main branch

---

## Go-Specific Features

### 1. Short Variable Declaration

```go
// Supported
result := a + b  // := mutation works

// Traditional declaration
var result int = a + b
```

### 2. Bitwise Operators

```go
// Bitwise AND, OR, XOR
flags := enable | disable
mask := flag & 0xFF

// Bit shifts
shifted := value << 8
```

### 3. Compound Assignment

```go
// All compound assignments supported
count += 1
value *= factor
bits &= mask
position <<= 1
```

### 4. Table-Driven Tests

```go
// Go idiom - perfect for mutation testing
tests := []struct {
    name     string
    input    int
    expected int
}{
    {"zero", 0, 0},
    {"positive", 5, 5},
    {"negative", -5, -5},
}
```

---

## Related Documentation

- [TypeScript Mutation Testing](TYPESCRIPT-MUTATION-TESTING.md) - JavaScript/TypeScript implementation
- [Python Mutation Testing](PYTHON-MUTATION-TESTING.md) - Python implementation
- [Original Specification](../tickets/TICKET-PMAT-7012.md)
- [Multi-Language Mutation Testing Architecture](../tickets/PMAT-7007-SUB-AGENT-SCAFFOLDING.md)

---

## Support & Contributing

### Getting Help

1. Check [Troubleshooting](#troubleshooting) section
2. Review [examples/](../../server/examples/) for working code
3. Open issue on GitHub with reproduction steps

### Contributing

Contributions welcome for:
- Additional mutation operators (channel ops, goroutines, defer)
- Performance optimizations
- Test framework integrations
- Documentation improvements

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

---

## Comparison with Other Tools

### vs go-mutesting

**PMAT Go:**
- ✅ AST-based mutations (preserves formatting)
- ✅ Tree-sitter parsing (fast, reliable)
- ✅ Rust performance
- ✅ 6 operators (including bitwise, assignment, unary)
- ✅ Multi-language support (TypeScript, Python, Go)

**go-mutesting:**
- ✅ Pure Go implementation
- ✅ Mature ecosystem
- ❌ Fewer operators
- ❌ Text-based mutations

### vs go-carpet

**PMAT Go:**
- ✅ Production-ready
- ✅ Fast generation (<3ms)
- ✅ Comprehensive operators

**go-carpet:**
- ✅ Visual coverage reports
- ❌ Limited mutation support
- ❌ Slower execution

---

## Technical Details

### AST Node Types

PMAT Go uses tree-sitter-go to parse and mutate these node types:

- `binary_expression` - Arithmetic, relational, logical, bitwise operators
- `unary_expression` - Unary operators (!, -, +)
- `assignment_statement` - Compound assignment operators

### Source Splicing

Mutations use byte-level source splicing:

```rust
let op_range = operator_node.byte_range();
let mut mutated = source.to_vec();
mutated.splice(op_range, new_op.bytes());
```

This preserves:
- Original formatting
- Comments
- Whitespace
- Indentation

---

## License

MIT OR Apache-2.0 (same as PMAT)

---

**Last Updated:** 2025-10-08
**Version:** 2.153.0
**Status:** Production Ready
**Maintainer:** PMAT Team
