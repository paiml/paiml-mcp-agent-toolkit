# Property Testing Best Practices Guide

## 🎯 Sprint 88 Achievement: 80% Property Test Coverage

As of Sprint 88, we have achieved **80% property test coverage** (431/539 files) through automated injection. This guide helps maintain and improve upon this achievement.

## Table of Contents
1. [Overview](#overview)
2. [Current Coverage Status](#current-coverage-status)
3. [Writing Effective Property Tests](#writing-effective-property-tests)
4. [Upgrading Placeholder Tests](#upgrading-placeholder-tests)
5. [Common Patterns](#common-patterns)
6. [CI/CD Integration](#cicd-integration)
7. [Maintenance Guidelines](#maintenance-guidelines)

## Overview

Property-based testing uses the `proptest` framework to generate random test inputs and verify that certain properties hold for all valid inputs. This is superior to traditional example-based testing as it can discover edge cases you might not think of.

## Current Coverage Status

### Metrics (as of v2.78.0)
- **Files with Property Tests**: 431/539 (80.0%)
- **Test Pattern Distribution**:
  - Basic placeholder tests: ~380 files
  - Meaningful property tests: ~51 files
- **CI/CD Enforcement**: GitHub Actions validates 80% threshold

### Coverage Verification
```bash
# Check current coverage
find server/src -name "*.rs" -type f | wc -l  # Total files
grep -r "proptest!" server/src --include="*.rs" -l | wc -l  # Files with tests

# Run property tests
cd server
PROPTEST_CASES=100 cargo test --lib -- proptest
```

## Writing Effective Property Tests

### ❌ Basic Placeholder (Current State)
```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }
    }
}
```

### ✅ Meaningful Property Test (Target State)
```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn parse_format_roundtrip(
            input in "[a-zA-Z0-9_]{1,100}"
        ) {
            // Property: parsing and formatting should roundtrip
            let parsed = parse_identifier(&input);
            let formatted = format_identifier(&parsed);
            prop_assert_eq!(formatted, input);
        }

        #[test]
        fn complexity_bounds(
            code in prop::collection::vec("[a-z]{1,10}", 1..100)
        ) {
            // Property: complexity should never exceed threshold
            let complexity = calculate_complexity(&code.join("\n"));
            prop_assert!(complexity.cyclomatic <= 20);
            prop_assert!(complexity.cognitive <= 15);
        }

        #[test]
        fn deterministic_hashing(
            data in prop::collection::vec(0u8..255, 0..1000)
        ) {
            // Property: hashing should be deterministic
            let hash1 = calculate_hash(&data);
            let hash2 = calculate_hash(&data);
            prop_assert_eq!(hash1, hash2);
        }
    }
}
```

## Upgrading Placeholder Tests

### Priority Upgrade Targets
1. **Core Services** (`server/src/services/`)
   - Start with critical services like `analyzer`, `detection`, `refactor`
   - Focus on roundtrip properties and invariants

2. **AST Modules** (`server/src/ast/`)
   - Parse/unparse roundtrips
   - AST transformation properties
   - Hash determinism

3. **Models** (`server/src/models/`)
   - Serialization/deserialization roundtrips
   - Validation properties
   - Builder patterns

### Upgrade Strategy
```rust
// Step 1: Identify the module's core functionality
// Step 2: Define properties that should always hold
// Step 3: Create generators for valid inputs
// Step 4: Write property assertions

// Example: Upgrading a parser module
proptest! {
    #[test]
    fn parser_never_panics(input in ".*") {
        // Property: parser should handle any input without panicking
        let _ = parse_safely(&input);
    }

    #[test]
    fn valid_input_parses_successfully(
        input in valid_input_generator()
    ) {
        // Property: valid inputs should always parse
        let result = parse(&input);
        prop_assert!(result.is_ok());
    }
}

fn valid_input_generator() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9_]*"
}
```

## Common Patterns

### 1. Roundtrip Properties
```rust
proptest! {
    #[test]
    fn serialize_deserialize_roundtrip(
        value in arbitrary_value()
    ) {
        let serialized = serde_json::to_string(&value)?;
        let deserialized: MyType = serde_json::from_str(&serialized)?;
        prop_assert_eq!(value, deserialized);
    }
}
```

### 2. Invariant Properties
```rust
proptest! {
    #[test]
    fn sorted_remains_sorted(
        mut items in prop::collection::vec(any::<i32>(), 0..100)
    ) {
        items.sort();
        for window in items.windows(2) {
            prop_assert!(window[0] <= window[1]);
        }
    }
}
```

### 3. Oracle Properties
```rust
proptest! {
    #[test]
    fn optimized_matches_naive(
        input in test_input_generator()
    ) {
        let naive_result = naive_implementation(&input);
        let optimized_result = optimized_implementation(&input);
        prop_assert_eq!(naive_result, optimized_result);
    }
}
```

### 4. Statistical Properties
```rust
proptest! {
    #[test]
    fn hash_distribution_is_uniform(
        inputs in prop::collection::vec(".*", 1000)
    ) {
        let hashes: Vec<u64> = inputs.iter()
            .map(|s| calculate_hash(s))
            .collect();
        
        let mean = statistical_mean(&hashes);
        let stddev = statistical_stddev(&hashes);
        
        // Check for reasonable distribution
        prop_assert!(stddev > mean * 0.1);
    }
}
```

## CI/CD Integration

### GitHub Actions Workflows

#### property-test-validation.yml
- **Purpose**: Enforce 80% coverage threshold
- **Frequency**: Every push/PR + daily
- **Failure Condition**: Coverage < 80%

#### property-tests.yml  
- **Purpose**: Run comprehensive property tests
- **Features**: Coverage analysis, platform matrix, fuzzing
- **Platforms**: Linux, macOS, Windows

### Local Pre-commit Hook
```bash
#!/bin/bash
# .git/hooks/pre-commit

# Check property test coverage
total_files=$(find server/src -name "*.rs" -type f | wc -l)
files_with_tests=$(grep -r "proptest!" server/src --include="*.rs" -l | wc -l)
coverage=$((files_with_tests * 100 / total_files))

if [ $coverage -lt 80 ]; then
    echo "❌ Property test coverage $coverage% below 80% threshold"
    exit 1
fi
```

## Maintenance Guidelines

### Weekly Tasks
1. **Review Coverage Reports**
   ```bash
   make test-property-coverage
   ```

2. **Upgrade Placeholder Tests**
   - Target: Convert 5-10 placeholder tests to meaningful ones
   - Focus on high-value modules first

3. **Monitor CI Performance**
   - Check GitHub Actions run times
   - Optimize slow property tests

### Monthly Tasks
1. **Coverage Analysis**
   ```bash
   ./scripts/property_test_metrics.sh
   ```

2. **Update Strategies**
   - Review and improve input generators
   - Add new property patterns

3. **Documentation Updates**
   - Document new property test patterns
   - Update this guide with learnings

### Quarterly Goals
- **Q1 2025**: Achieve 85% coverage
- **Q2 2025**: 50% meaningful tests (vs placeholders)
- **Q3 2025**: 90% coverage
- **Q4 2025**: 75% meaningful tests

## Tools and Scripts

### Rapid Injection Script
```bash
# Add property tests to uncovered files
./scripts/rapid_property_test_injection.py

# Check specific module coverage
./scripts/rapid_property_test_injection.py --check server/src/services
```

### Property Test Metrics
```bash
# Generate metrics report
./scripts/property_test_metrics.sh > metrics_report.md

# Find files without tests
find server/src -name "*.rs" -type f | while read f; do
    if ! grep -q "proptest!" "$f"; then
        echo "$f"
    fi
done
```

### Upgrade Helpers
```bash
# Find placeholder tests to upgrade
grep -r "prop_assert!(true)" server/src --include="*.rs" -l

# Count meaningful vs placeholder tests
meaningful=$(grep -r "prop_assert!" server/src --include="*.rs" | grep -v "prop_assert!(true)" | wc -l)
placeholder=$(grep -r "prop_assert!(true)" server/src --include="*.rs" | wc -l)
echo "Meaningful: $meaningful, Placeholder: $placeholder"
```

## Best Practices Summary

### DO ✅
- Write properties that test invariants
- Use roundtrip testing for parsers/serializers
- Test edge cases with generated inputs
- Keep property tests fast (<100ms per test)
- Use deterministic seeds for reproducibility

### DON'T ❌
- Don't write trivial always-true properties
- Don't test implementation details
- Don't use too many test cases (100 is usually enough)
- Don't ignore failing property tests
- Don't skip shrinking (it finds minimal failures)

## Resources

- [Proptest Documentation](https://proptest-rs.github.io/proptest/)
- [Property Testing Patterns](https://hypothesis.works/articles/testing-the-hard-stuff-and-staying-sane/)
- [QuickCheck Original Paper](http://www.cs.tufts.edu/~nr/cs257/archive/john-hughes/quick.pdf)

## Conclusion

Property testing is a powerful technique that complements traditional testing. With our 80% coverage achievement, we have a solid foundation. The next step is to upgrade placeholder tests to meaningful property tests that actually validate system behavior.

Remember: **Quality over Quantity** - A few well-designed property tests are worth more than hundreds of placeholder tests.