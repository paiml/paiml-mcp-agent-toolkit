# Python Mutation Testing Example

This example demonstrates how to use PMAT for mutation testing on a Python project.

## Overview

Mutation testing is a technique to evaluate the quality of your test suite by introducing small changes (mutations) to your code and checking if your tests detect them.

**Mutation Score** = (Killed Mutants / Total Valid Mutants) × 100%

- **Killed**: Test suite detected the mutation (good!)
- **Survived**: Test suite did not detect the mutation (test gap!)
- **CompileError**: Mutation caused invalid syntax
- **Timeout**: Mutation caused infinite loop

## Project Structure

```
python-mutation-testing/
├── src/
│   └── calculator.py      # Calculator module with 8 functions
├── tests/
│   └── test_calculator.py # Comprehensive test suite (24 tests)
├── requirements.txt        # Python dependencies
└── README.md               # This file
```

## Getting Started

### 1. Install Dependencies

First, install the required Python packages:

```bash
cd examples/python-mutation-testing
pip install -r requirements.txt
```

### 2. Run Tests

Verify that all tests pass:

```bash
pytest tests/test_calculator.py -v
```

You should see:

```
===================== test session starts =====================
collected 24 tests

tests/test_calculator.py::TestArithmeticOperations::test_add PASSED
tests/test_calculator.py::TestArithmeticOperations::test_subtract PASSED
tests/test_calculator.py::TestArithmeticOperations::test_multiply PASSED
tests/test_calculator.py::TestArithmeticOperations::test_divide PASSED
tests/test_calculator.py::TestLogicalOperations::test_is_even PASSED
tests/test_calculator.py::TestLogicalOperations::test_max_value PASSED
tests/test_calculator.py::TestComplexOperations::test_factorial PASSED
tests/test_calculator.py::TestComplexOperations::test_factorial_negative PASSED
tests/test_calculator.py::TestComplexOperations::test_is_prime PASSED
... (15 more tests)

===================== 24 passed in 0.05s =====================
```

### 3. Run Mutation Testing

Use PMAT to perform mutation testing on the calculator module:

```bash
# Install pmat if not already installed
cargo install pmat

# Run mutation testing on the calculator module
pmat mutate --target src/calculator.py
```

### 4. Analyze Results

PMAT will generate mutants and run your tests against each one. Example output:

```
Mutation Testing Results
========================
Total Mutants: 52
Killed: 47 (90.4%)
Survived: 3 (5.8%)
Compile Errors: 2 (3.8%)
Timeouts: 0 (0.0%)

Mutation Score: 90.4%
```

### 5. Review Survived Mutants (Test Gaps)

Use `--failures-only` to see only survived mutants:

```bash
pmat mutate --target src/calculator.py --failures-only
```

This shows which mutations your tests failed to catch, indicating gaps in test coverage.

## Example: Detecting Test Gaps

### Original Code
```python
def max_value(a: int, b: int) -> int:
    """Calculate the maximum of two numbers."""
    if a > b:  # Mutation: change > to >=
        return a
    else:
        return b
```

### Mutation
```python
def max_value(a: int, b: int) -> int:
    """Calculate the maximum of two numbers."""
    if a >= b:  # Mutated: > changed to >=
        return a
    else:
        return b
```

### Test That Catches It
```python
def test_max_value(self):
    """Test maximum value calculation."""
    assert max_value(5, 3) == 5  # Would pass with either > or >=
    assert max_value(2, 8) == 8  # Would pass with either > or >=
    assert max_value(4, 4) == 4  # CRITICAL: Catches the >= mutation!
```

Without the `max_value(4, 4)` test case, the `> to >=` mutation would **survive**, indicating a test gap.

## Output Formats

### Text (Default)
```bash
pmat mutate --target src/calculator.py
```

Color-coded terminal output with summary and individual mutant details.

### JSON
```bash
pmat mutate --target src/calculator.py --output-format json > results.json
```

Machine-readable JSON for CI/CD integration.

### Markdown
```bash
pmat mutate --target src/calculator.py --output-format markdown > results.md
```

Human-readable report with tables and statistics.

## Advanced Usage

### Control Concurrency
```bash
# Use 4 parallel jobs
pmat mutate --target src/calculator.py --jobs 4
```

### Set Timeout
```bash
# Timeout individual tests after 10 seconds
pmat mutate --target src/calculator.py --timeout 10
```

### Enforce Mutation Score Threshold
```bash
# Fail if mutation score is below 85%
pmat mutate --target src/calculator.py --threshold 85
```

This will exit with code 1 if the mutation score is below 85%, useful for CI/CD quality gates.

## CI/CD Integration

### GitHub Actions

Create `.github/workflows/mutation-testing.yml`:

```yaml
name: Mutation Testing

on: [push, pull_request]

jobs:
  mutation-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Set up Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.11'

      - name: Install dependencies
        run: |
          pip install -r examples/python-mutation-testing/requirements.txt

      - name: Install Rust and pmat
        run: |
          curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
          source $HOME/.cargo/env
          cargo install pmat

      - name: Run mutation tests
        run: |
          cd examples/python-mutation-testing
          pmat mutate --target src/calculator.py --failures-only --threshold 80
```

### GitLab CI

Add to `.gitlab-ci.yml`:

```yaml
mutation-testing:
  image: python:3.11
  stage: test
  before_script:
    - pip install -r examples/python-mutation-testing/requirements.txt
    - curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    - source $HOME/.cargo/env
    - cargo install pmat
  script:
    - cd examples/python-mutation-testing
    - pmat mutate --target src/calculator.py --output-format json > mutation_results.json
  artifacts:
    reports:
      junit: mutation_results.json
```

## Best Practices

1. **Run mutation testing regularly** - Integrate into CI/CD pipeline
2. **Set reasonable thresholds** - 80-90% mutation score is excellent
3. **Focus on critical code** - Test business logic thoroughly
4. **Review survived mutants** - Each one represents a potential bug
5. **Use `--failures-only`** - Reduces noise in large codebases
6. **Test edge cases** - Boundary values catch more mutations
7. **Combine with coverage** - High code coverage + high mutation score = robust tests

## Understanding Mutation Operators

PMAT applies various mutation operators to Python code:

### Arithmetic Operators
- `+` → `-`, `*`, `/`, `//`, `%`
- `-` → `+`, `*`, `/`, `//`, `%`
- `*` → `+`, `-`, `/`, `//`, `%`
- `/` → `+`, `-`, `*`, `//`, `%`
- `//` → `+`, `-`, `*`, `/`, `%`
- `%` → `+`, `-`, `*`, `/`, `//`

### Comparison Operators
- `==` → `!=`, `<`, `>`, `<=`, `>=`
- `<` → `<=`, `>`, `>=`, `==`, `!=`
- `>` → `>=`, `<`, `<=`, `==`, `!=`
- `<=` → `<`, `>=`, `!=`
- `>=` → `>`, `<=`, `!=`
- `!=` → `==`, `<`, `>`, `<=`, `>=`

### Boolean Operators
- `and` → `or`
- `or` → `and`
- `not` → (removed)

### Boundary Values
- `<` → `<=`
- `>` → `>=`
- `0` → `1`, `-1`
- `range(n)` → `range(n+1)`, `range(n-1)`

### Return Values
- `return x` → `return None`, `return 0`, `return -1`
- `return True` → `return False`
- `return False` → `return True`

## Example Results

```
╔═══════════════════════════════════════════╗
║     Mutation Testing Summary              ║
╠═══════════════════════════════════════════╣
║ Total Mutants:        52                  ║
║ Killed:               47 (90.4%)          ║
║ Survived:             3 (5.8%)            ║
║ Compile Errors:       2 (3.8%)            ║
║ Timeouts:             0 (0.0%)            ║
╠═══════════════════════════════════════════╣
║ Mutation Score:       90.4% ✓             ║
╚═══════════════════════════════════════════╝
```

## Running Tests Manually

### With pytest
```bash
# Run all tests
pytest tests/

# Run with verbose output
pytest tests/ -v

# Run with coverage report
pytest tests/ --cov=src --cov-report=html
```

### With unittest (alternative)
```bash
python -m pytest tests/test_calculator.py
```

## Troubleshooting

### Import Errors

If you get `ModuleNotFoundError`, ensure you're running tests from the project root:

```bash
cd examples/python-mutation-testing
python -m pytest tests/
```

Or use the `PYTHONPATH` environment variable:

```bash
export PYTHONPATH="${PYTHONPATH}:$(pwd)/src"
pytest tests/
```

### Mutation Testing Takes Too Long

Use the `--timeout` flag to limit test execution time:

```bash
pmat mutate --target src/calculator.py --timeout 5
```

Or reduce concurrency if tests are CPU-bound:

```bash
pmat mutate --target src/calculator.py --jobs 1
```

## Resources

- **PMAT Repository**: https://github.com/paiml/paiml-mcp-agent-toolkit
- **Crates.io**: https://crates.io/crates/pmat
- **Documentation**: `server/README.md`
- **Mutation Testing Paper**: [Mutation Testing: An Empirical Evaluation](https://dl.acm.org/doi/10.1145/3183440)

## License

This example is part of the PMAT project and is provided as-is for demonstration purposes.
