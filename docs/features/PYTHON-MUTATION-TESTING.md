# Python Mutation Testing

**Status:** ✅ Production Ready (v2.152.0+)
**Languages:** Python 3.6+
**Quality:** 80%+ mutation score achievable

---

## Overview

PMAT provides **AST-based mutation testing** for Python projects, helping you validate test suite quality by introducing controlled bugs (mutations) and checking if your tests catch them.

**Key Benefits:**
- 🎯 Quantify test quality with mutation scores
- 🔍 Identify gaps in test coverage
- ⚡ Fast generation (5ms for 56 mutants)
- 🧬 5 mutation operators covering common bug patterns
- 🔄 Works with pytest and unittest

---

## Quick Start

### 1. Install Dependencies

```bash
cd fixtures/python
pip3 install pytest  # Or use virtualenv
```

### 2. Run Mutation Testing

```bash
# From server directory
cargo run --example python_mutation_workflow --features all-languages

# Or build the example first
cargo build --example python_mutation_workflow --features all-languages
./target/debug/examples/python_mutation_workflow
```

### 3. View Results

```
🐍 Python Mutation Testing Workflow

📝 Reading source file: ../fixtures/python/calculator.py
   Size: 1,085 bytes

🔧 Generating mutants...
   Generated: 56 mutants
   Time: 5.2ms

✅ Running baseline tests...
   Baseline tests passed ✅

🧪 Testing mutants (56 total)...
   [Progress updates...]

📊 Mutation Testing Results
   Total Mutants:    56
   Killed:           45 (80%)
   Survived:         11 (19%)
   Timeout/Error:    0

🎯 Mutation Score: 80% ✅ EXCELLENT!
```

---

## Mutation Operators

### 1. Binary Operator Replacement (AOR)

**Replaces:** `+`, `-`, `*`, `/`, `//`, `%`, `**`

```python
# Original
def add(a, b):
    return a + b

# Mutants
return a - b   # + → -
return a * b   # + → *
return a / b   # + → /
return a // b  # + → // (floor division)
return a % b   # + → % (modulo)
return a ** b  # + → ** (power)
```

**Tests should fail** when arithmetic operators are changed.

### 2. Relational Operator Replacement (ROR)

**Replaces:** `<`, `>`, `<=`, `>=`, `==`, `!=`

```python
# Original
def is_positive(value):
    return value > 0

# Mutants
return value < 0   # > → <
return value >= 0  # > → >=
return value <= 0  # > → <=
return value == 0  # > → ==
return value != 0  # > → !=
```

**Tests should fail** when comparison logic changes.

### 3. Logical Operator Replacement (LOR)

**Replaces:** `and` ↔ `or`

```python
# Original
def both_true(a, b):
    return a and b

# Mutant
return a or b  # and → or

# Original
def either_true(a, b):
    return a or b

# Mutant
return a and b  # or → and
```

**Tests should fail** when logical operators are swapped.

### 4. Identity Operator Replacement

**Replaces:** `is` ↔ `is not`

```python
# Original
def is_none(value):
    return value is None

# Mutants
return value is not None  # is → is not

# Original
def is_not_none(value):
    return value is not None

# Mutants
return value is None  # is not → is
```

**Tests should fail** when identity checks are inverted.

### 5. Membership Operator Replacement

**Replaces:** `in` ↔ `not in`

```python
# Original
def contains(item, collection):
    return item in collection

# Mutant
return item not in collection  # in → not in

# Original
def not_contains(item, collection):
    return item not in collection

# Mutant
return item in collection  # not in → in
```

**Tests should fail** when membership checks are inverted.

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

```python
# Code
def is_positive(value):
    return value > 0

# Test (weak!)
assert is_positive(5) == True
assert is_positive(-5) == False
# Passes even with >= because 5 > 0 and -5 < 0

# Better test
assert is_positive(0) == False  # Would fail with >=
```

### Example: Logical Operator Gap

**Mutant:** `and` → `or` (survives)

```python
# Code
def both_conditions(a, b):
    return a > 0 and b > 0

# Test (weak!)
assert both_conditions(5, 10) == True
# Passes even with or because both are positive

# Better test
assert both_conditions(5, -1) == False  # Would fail with or
assert both_conditions(-1, 5) == False  # Would fail with or
```

### Example: Identity vs Equality Gap

**Mutant:** `is` → `==` (survives)

```python
# Code
def is_none_or_empty(value):
    return value is None or value == ""

# Test (weak!)
assert is_none_or_empty(None) == True
assert is_none_or_empty("") == True
# Might pass even with == instead of is

# Better test
# Use object identity to enforce is vs ==
obj = object()
assert (obj is None) == False  # Would fail with ==
```

### Example: Membership Inversion Gap

**Mutant:** `in` → `not in` (survives)

```python
# Code
def has_vowel(text):
    return any(char in 'aeiou' for char in text)

# Test (weak!)
assert has_vowel("hello") == True
# Doesn't test the actual membership logic thoroughly

# Better test
assert has_vowel("") == False  # Edge case
assert has_vowel("xyz") == False  # Would catch not in mutant
```

---

## Example Project Structure

```
my-python-project/
├── src/
│   ├── calculator.py          # Source code
│   └── test_calculator.py     # Tests
├── requirements.txt           # Dependencies
├── pytest.ini                 # pytest configuration
└── .venv/                     # Virtual environment
```

### Minimal requirements.txt

```
pytest>=7.0.0
```

### pytest.ini

```ini
[pytest]
testpaths = .
python_files = test_*.py
python_classes = Test*
python_functions = test_*
```

---

## Advanced Usage

### Programmatic Usage

```rust
use pmat::services::mutation::PythonMutationGenerator;

// Generate mutants
let generator = PythonMutationGenerator::with_default_operators();
let mutants = generator.generate_mutants(&source, "test.py")?;

// Process mutants
for mutant in mutants {
    println!("Mutant: {} at line {}", mutant.id, mutant.location.line);
}
```

### Custom Operator Selection

```rust
use pmat::services::mutation::python_tree_sitter_mutations::*;

let generator = PythonMutationGenerator {
    operators: vec![
        Box::new(PythonBinaryOpMutation),      // Only arithmetic
        Box::new(PythonRelationalOpMutation),  // Only comparisons
    ],
};
```

---

## Limitations & Known Issues

### Current Limitations

1. **Single-file testing** - Multi-file projects not yet supported
2. **pytest startup overhead** - Each mutant restarts pytest (~17ms per mutant)
3. **No test selection** - Runs all tests for each mutant
4. **Sequential execution** - No parallel testing yet

### Workarounds

**Speed up testing:**
```bash
# Use pytest-xdist for parallel test execution
pip install pytest-xdist
pytest -n auto
```

**Reduce mutants:**
- Focus on critical files
- Use smaller test suites during development
- Run full mutation testing in CI

### Future Enhancements

- [ ] Multi-file project support
- [ ] pytest keep-alive (3-4x speedup)
- [ ] Smart test selection (2-5x speedup)
- [ ] Parallel execution (8x speedup)
- [ ] HTML reports
- [ ] CI/CD integration
- [ ] Additional operators (unary, augmented assignment)

---

## Troubleshooting

### "pytest not installed"

**Problem:** pytest not found in environment

**Solution:**
```bash
# Install pytest
pip3 install pytest

# Or with virtualenv
python3 -m venv .venv
source .venv/bin/activate  # On Windows: .venv\Scripts\activate
pip install pytest
```

### "Baseline tests failed"

**Problem:** Tests fail on original code

**Solution:**
```bash
# Fix tests first
pytest

# Then run mutation testing
cargo run --example python_mutation_workflow
```

### "No mutants generated"

**Problem:** Code doesn't match mutation patterns

**Solution:**
- Ensure code has operators (+, -, *, /, and, or, etc.)
- Check for comparison operators (<, >, ==, etc.)
- Verify Python syntax is valid
- Check that file is valid Python 3 code

### "Import errors during testing"

**Problem:** Mutated code can't import modules

**Solution:**
```python
# Ensure PYTHONPATH is correct
# Or use relative imports
from . import module_name

# Or absolute imports with proper package structure
```

---

## Best Practices

### 1. Start with Small Files

```bash
# Good: Single file, focused tests
calculator.py + test_calculator.py (56 mutants, 1 minute)

# Avoid: Large files initially
entire_app.py (1000+ mutants, 30+ minutes)
```

### 2. Interpret Surviving Mutants

**Don't just aim for 100%** - some mutants are equivalent:

```python
# These might be equivalent:
return a > b if a > b else b
return a >= b if a >= b else b  # Same behavior in some cases
```

Focus on **meaningful survivors** that reveal test gaps.

### 3. Add Tests Iteratively

```python
# 1. Run mutation testing
# 2. Identify surviving mutants
# 3. Add tests to kill them
# 4. Re-run to verify

# Example: Add boundary test
def test_is_positive_boundary():
    assert is_positive(0) == False  # Kills > → >= mutant
    assert is_positive(1) == True
    assert is_positive(-1) == False
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
      - uses: actions/setup-python@v2
        with:
          python-version: '3.10'
      - run: pip install pytest
      - run: cargo run --example python_mutation_workflow
      - run: |
          # Fail if mutation score < 70%
          if [ $MUTATION_SCORE -lt 70 ]; then exit 1; fi
```

---

## Performance Expectations

### Typical Performance

| Mutants | Sequential | Optimized (Future) |
|---------|------------|-------------------|
| 10 | ~0.2s | ~0.05s |
| 50 | ~0.9s | ~0.2s |
| 100 | ~1.8s | ~0.4s |
| 500 | ~9s | ~2s |

**Note:** Times assume ~17ms per mutant (current) or ~4ms (optimized)

Python mutation testing is significantly faster than TypeScript because pytest startup is much lighter than npm test frameworks.

### Scaling Recommendations

**Small projects (<100 mutants):** Run on every commit
**Medium projects (100-500 mutants):** Run on PRs
**Large projects (500+ mutants):** Run nightly or on main branch

---

## Related Documentation

- [TypeScript Mutation Testing](TYPESCRIPT-MUTATION-TESTING.md) - Similar implementation for TypeScript/JavaScript
- [Original Specification](../tickets/TICKET-PMAT-7011.md)
- [Multi-Language Mutation Testing Architecture](../tickets/PMAT-7007-SUB-AGENT-SCAFFOLDING.md)

---

## Support & Contributing

### Getting Help

1. Check [Troubleshooting](#troubleshooting) section
2. Review [examples/](../../server/examples/) for working code
3. Open issue on GitHub with reproduction steps

### Contributing

Contributions welcome for:
- Additional mutation operators (augmented assignment, unary operators)
- Performance optimizations
- Test framework integrations (unittest, nose2)
- Documentation improvements

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

---

## Comparison with Other Tools

### vs mutmut

**PMAT Python:**
- ✅ AST-based mutations (preserves formatting)
- ✅ Tree-sitter parsing (fast, reliable)
- ✅ Rust performance
- ❌ Fewer operators currently

**mutmut:**
- ✅ More mutation operators
- ✅ Mature ecosystem
- ❌ String-based mutations (formatting issues)
- ❌ Slower Python implementation

### vs cosmic-ray

**PMAT Python:**
- ✅ Simpler setup
- ✅ Faster execution
- ✅ Multi-language support

**cosmic-ray:**
- ✅ More advanced operators
- ✅ Distributed execution
- ❌ Complex configuration
- ❌ Slower startup

---

## Technical Details

### AST Node Types

PMAT Python uses tree-sitter-python to parse and mutate these node types:

- `binary_operator` - Arithmetic operators (+, -, *, /, //, %, **)
- `comparison_operator` - Relational operators (<, >, <=, >=, ==, !=)
- `boolean_operator` - Logical operators (and, or)
- `identifier` - Identity operators (is, is not) - matched by text
- `identifier` - Membership operators (in, not in) - matched by text

### Source Splicing

Mutations use byte-level source splicing:

```rust
let op_range = operator_node.byte_range();
let mut mutated = source.to_vec();
mutated.splice(op_range, new_op.bytes());
```

This preserves:
- Original indentation
- Comments
- Whitespace
- String formatting

---

## License

MIT OR Apache-2.0 (same as PMAT)

---

**Last Updated:** 2025-10-08
**Version:** 2.152.0
**Status:** Production Ready
**Maintainer:** PMAT Team
