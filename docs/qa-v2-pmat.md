# QA v2: Deep Context Analysis & Provability Verification Protocol

## Executive Summary

This protocol establishes rigorous verification procedures for the existing `deep_context.md` generation system, addressing critical deficiencies in dead code detection (showing 0% when non-zero is expected) and complexity reporting (lacking distribution metrics). Following Toyota Production System principles, we implement verification procedures using only existing tooling and infrastructure.

## 1. Pre-QA Verification Harness

### 1.1 Dead Code Analysis Verification

```rust
// Test harness to verify existing dead code analyzer behavior
#[cfg(test)]
mod dead_code_verification {
    use std::collections::HashSet;
    use crate::services::dead_code_analyzer::*;
    
    #[test]
    fn verify_entry_point_detection() {
        let analyzer = DeadCodeAnalyzer::new();
        
        // Verify binary entry points
        assert!(analyzer.is_entry_point("main", "src/bin/pmat.rs"));
        
        // Verify library public exports (lib.rs public items)
        let lib_exports = analyzer.get_public_exports("src/lib.rs");
        assert!(lib_exports.contains("run_mcp_server"));
        assert!(lib_exports.contains("TemplateServer"));
        
        // Verify test and bench detection
        assert!(analyzer.is_test_code("tests/", "any_test.rs"));
        assert!(analyzer.is_bench_code("benches/", "critical_path.rs"));
        
        // Edge case: legitimate zero cross-language refs
        // (e.g., early-stage Rust-only project)
        let cross_refs = analyzer.get_cross_language_references();
        if cross_refs.is_empty() {
            // Verify this is expected based on project structure
            let has_ts_files = std::fs::read_dir(".")
                .unwrap()
                .any(|e| e.unwrap().path().extension() == Some("ts".as_ref()));
            assert!(!has_ts_files, 
                "TypeScript files exist but no cross-language refs found");
        }
    }
    
    #[test] 
    fn verify_wasm_bindgen_detection() {
        // Test existing parser recognizes wasm_bindgen attributes
        let code = r#"
            #[wasm_bindgen]
            pub fn exported_function() {}
            
            fn internal_function() {} // Should be dead if not called
        "#;
        
        let ast = syn::parse_str::<syn::File>(code).unwrap();
        let analyzer = DeadCodeAnalyzer::new();
        let results = analyzer.analyze_ast(&ast);
        
        assert!(!results.is_dead("exported_function"));
        assert!(results.is_dead("internal_function"));
    }
}
```

### 1.2 Complexity Distribution Verification

```rust
#[derive(Debug, Clone)]
struct ComplexityDistributionConfig {
    /// Minimum expected entropy for healthy distribution
    min_entropy: f64,
    /// Warning threshold percentile (default: 5%)
    warning_threshold_percentile: f64,
    /// Minimum function count for distribution analysis
    min_function_count: usize,
}

impl Default for ComplexityDistributionConfig {
    fn default() -> Self {
        Self {
            min_entropy: 2.0,
            warning_threshold_percentile: 0.05,
            min_function_count: 100,
        }
    }
}

fn verify_complexity_distribution(
    metrics: &ComplexityMetrics,
    config: &ComplexityDistributionConfig,
) -> Result<(), String> {
    // Calculate Shannon entropy of complexity distribution
    let entropy = calculate_entropy(&metrics.functions);
    
    if entropy < config.min_entropy && metrics.functions.len() >= config.min_function_count {
        return Err(format!(
            "Low complexity entropy: {:.2} (expected >= {:.2}). \
             Possible parser issue or unnaturally uniform codebase.",
            entropy, config.min_entropy
        ));
    }
    
    // Verify reasonable distribution of complex functions
    let complex_count = metrics.functions.iter()
        .filter(|f| f.cyclomatic > 10) // McCabe threshold
        .count();
    
    let complex_ratio = complex_count as f64 / metrics.functions.len() as f64;
    
    if complex_ratio < config.warning_threshold_percentile 
        && metrics.functions.len() >= config.min_function_count {
        return Err(format!(
            "Suspiciously few complex functions: {:.1}% \
             (expected >= {:.1}% for codebase with {} functions)",
            complex_ratio * 100.0,
            config.warning_threshold_percentile * 100.0,
            metrics.functions.len()
        ));
    }
    
    Ok(())
}

fn calculate_entropy(functions: &[FunctionComplexity]) -> f64 {
    use std::collections::HashMap;
    
    let mut freq_map = HashMap::new();
    for func in functions {
        *freq_map.entry(func.cyclomatic).or_insert(0) += 1;
    }
    
    let total = functions.len() as f64;
    freq_map.values()
        .map(|&count| {
            let p = count as f64 / total;
            -p * p.log2()
        })
        .sum()
}
```

## 2. Verification Execution Protocol

### 2.1 Dead Code Detection Calibration

```bash
#!/bin/bash
# dead_code_calibration.sh - Run in isolated test environment

set -euo pipefail

# Create temporary test directory
TEST_DIR=$(mktemp -d)
trap "rm -rf $TEST_DIR" EXIT

cd "$TEST_DIR"

# Generate calibration fixture
cat > lib.rs << 'EOF'
pub fn used_function() {
    println!("This is used");
}

fn definitely_dead() {
    println!("Never called");
}

#[cfg(test)]
fn test_only_function() {
    // Should not be marked dead
}
EOF

# Run analyzer
OUTPUT=$(pmat analyze dead-code --path . --format json 2>&1)

# Verify detection
if ! echo "$OUTPUT" | jq -e '.summary.dead_functions >= 1' >/dev/null; then
    echo "FAIL: Dead code analyzer failed to detect known dead function"
    echo "Output: $OUTPUT"
    exit 1
fi

# Verify test code not marked as dead
if echo "$OUTPUT" | jq -e '.dead_items[] | select(.name == "test_only_function")' >/dev/null; then
    echo "FAIL: Test code incorrectly marked as dead"
    exit 1
fi

echo "PASS: Dead code detection calibration successful"
```

### 2.2 Complexity Distribution Analysis

```bash
#!/bin/bash
# complexity_distribution.sh - Analyze existing metrics

# Extract raw complexity data
pmat analyze complexity --path . --format json | \
  jq -r '.files[].functions[] | [.name, .cyclomatic, .cognitive] | @csv' > complexity_raw.csv

# Calculate distribution metrics using existing tools
python3 -c "
import csv
import math
from collections import Counter

with open('complexity_raw.csv', 'r') as f:
    reader = csv.reader(f)
    cyclomatic_values = [int(row[1]) for row in reader]

# Calculate entropy
counter = Counter(cyclomatic_values)
total = len(cyclomatic_values)
entropy = -sum((count/total) * math.log2(count/total) 
               for count in counter.values())

# Calculate percentiles
cyclomatic_values.sort()
p50 = cyclomatic_values[len(cyclomatic_values)//2]
p90 = cyclomatic_values[int(len(cyclomatic_values)*0.9)]
p99 = cyclomatic_values[int(len(cyclomatic_values)*0.99)]

print(f'Entropy: {entropy:.2f}')
print(f'P50: {p50}, P90: {p90}, P99: {p99}')
print(f'Functions > 10: {sum(1 for v in cyclomatic_values if v > 10)}')

# Verification
if entropy < 2.0 and len(cyclomatic_values) > 100:
    print('WARNING: Low complexity entropy detected')
"
```

### 2.3 Web Demo State Verification

```typescript
// state_verification.test.ts - Using existing test framework (assumed Jest/Mocha)
import { describe, it, expect } from '@jest/globals'; // or your test framework

// Verify existing reducer implementation
describe('Web Demo State Transitions', () => {
    const testCases = [
        {
            name: 'FETCH_START maintains invariants',
            initial: { isLoading: false, data: null, error: null },
            action: { type: 'FETCH_START' },
            expected: { isLoading: true, data: null, error: null },
            invariants: [
                (s: DemoState) => !(s.isLoading && s.error),
                (s: DemoState) => s.data === null || !s.isLoading,
            ]
        },
        {
            name: 'FETCH_SUCCESS clears loading',
            initial: { isLoading: true, data: null, error: null },
            action: { type: 'FETCH_SUCCESS', payload: { metrics: {} } },
            expected: { isLoading: false, data: { metrics: {} }, error: null },
            invariants: [
                (s: DemoState) => !s.isLoading,
                (s: DemoState) => s.data !== null,
            ]
        }
    ];
    
    testCases.forEach(tc => {
        it(tc.name, () => {
            const result = reducer(tc.initial, tc.action);
            expect(result).toEqual(tc.expected);
            
            // Verify invariants
            tc.invariants.forEach((inv, i) => {
                expect(inv(result)).toBe(true, 
                    `Invariant ${i} violated: ${inv.toString()}`);
            });
        });
    });
    
    it('reducer is pure (no mutations)', () => {
        const initial = { isLoading: false, data: null, error: null };
        const frozen = Object.freeze(initial);
        
        // Should not throw if pure
        expect(() => reducer(frozen, { type: 'FETCH_START' })).not.toThrow();
    });
});
```

## 3. Root Cause Analysis Protocol

### 3.1 Dead Code False Negative Analysis (5 Whys)

```rust
// Document findings in code comments for traceability
impl DeadCodeAnalyzer {
    fn analyze_file(&mut self, path: &Path) -> DeadCodeResult {
        // FINDING: Zero dead code reported
        // Why 1: Entry points not properly identified
        // Why 2: Cross-language boundaries not traced
        // Why 3: FFI/WASM exports not in call graph
        // Why 4: Parser doesn't recognize #[wasm_bindgen]
        // Why 5: Attribute handlers not registered for procedural macros
        
        // REMEDIATION: Check for common export attributes
        let content = std::fs::read_to_string(path)?;
        let has_wasm_bindgen = content.contains("#[wasm_bindgen]");
        let has_no_mangle = content.contains("#[no_mangle]");
        
        if has_wasm_bindgen || has_no_mangle {
            // Mark all pub items in this file as potential exports
            self.mark_file_as_export_module(path);
        }
        
        // Continue with standard analysis...
    }
}
```

### 3.2 Complexity Under-Reporting Analysis

```rust
// Document in existing complexity analyzer
impl ComplexityAnalyzer {
    fn analyze_function(&self, func: &Function) -> ComplexityMetrics {
        // FINDING: Complexity values suspiciously low
        // Why 1: Not all control flow paths counted
        // Why 2: Match expressions counted as single branch
        // Why 3: Closure complexity not included
        // Why 4: Async/await points not considered
        // Why 5: Macro-generated code skipped
        
        let mut complexity = 1; // Base complexity
        
        // Count all branching constructs
        complexity += self.count_if_statements(func);
        complexity += self.count_match_arms(func); // Each arm adds complexity
        complexity += self.count_loop_constructs(func);
        complexity += self.count_early_returns(func);
        complexity += self.count_error_propagation(func); // ? operator
        
        // Include closure complexity
        for closure in func.find_closures() {
            complexity += self.analyze_function(&closure);
        }
        
        ComplexityMetrics {
            cyclomatic: complexity,
            cognitive: self.calculate_cognitive_complexity(func),
        }
    }
}
```

## 4. Verification Quality Gates

```rust
// quality_gates.rs - Add to existing test suite
use std::collections::HashMap;

#[derive(Debug)]
struct QAVerification {
    checks: Vec<(&'static str, Box<dyn Fn(&AnalysisResult) -> Result<(), String>>)>,
}

impl QAVerification {
    fn new() -> Self {
        let mut checks: Vec<(&'static str, Box<dyn Fn(&AnalysisResult) -> Result<(), String>>)> = vec![];
        
        // Dead code sanity check
        checks.push(("dead_code_sanity", Box::new(|result| {
            let ratio = result.dead_code.total_dead_lines as f64 / 
                       result.total_lines as f64;
            
            if ratio == 0.0 && result.total_lines > 1000 {
                // Check if this is legitimate
                if result.files.iter().any(|f| f.path.contains("wasm") || 
                                              f.path.contains("ffi")) {
                    Err("Zero dead code with FFI/WASM code present - likely false negative".into())
                } else if result.language_stats.get("TypeScript").unwrap_or(&0) > &0 {
                    Err("Mixed language project with zero dead code - verify cross-language tracing".into())
                } else {
                    // Pure Rust project in early stages might legitimately have no dead code
                    Ok(())
                }
            } else if ratio > 0.15 {
                Err(format!("Excessive dead code: {:.1}%", ratio * 100.0))
            } else {
                Ok(())
            }
        })));
        
        // Complexity distribution check
        checks.push(("complexity_distribution", Box::new(|result| {
            let functions: Vec<_> = result.complexity.files
                .iter()
                .flat_map(|f| &f.functions)
                .collect();
                
            if functions.len() < 50 {
                return Ok(()); // Too small for distribution analysis
            }
            
            // Calculate coefficient of variation
            let mean = functions.iter().map(|f| f.cyclomatic as f64).sum::<f64>() 
                      / functions.len() as f64;
            let variance = functions.iter()
                .map(|f| (f.cyclomatic as f64 - mean).powi(2))
                .sum::<f64>() / functions.len() as f64;
            let cv = (variance.sqrt() / mean) * 100.0;
            
            if cv < 30.0 {
                Err(format!("Low complexity variation (CV={:.1}%) - possible parser issue", cv))
            } else {
                Ok(())
            }
        })));
        
        Self { checks }
    }
    
    fn verify(&self, result: &AnalysisResult) -> HashMap<&'static str, Result<(), String>> {
        self.checks.iter()
            .map(|(name, check)| (*name, check(result)))
            .collect()
    }
}
```

## 5. Configuration Validation

```rust
// Enhance existing configuration with validation
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct DeepContextConfig {
    #[serde(default)]
    pub entry_points: Vec<String>,
    
    #[serde(default)]
    pub dead_code_threshold: f64,
    
    #[serde(default)]
    pub complexity_thresholds: ComplexityThresholds,
}

impl DeepContextConfig {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        // Validate entry points
        if self.entry_points.is_empty() {
            // Auto-detect based on project structure
            let detected = self.detect_entry_points();
            if detected.is_empty() {
                errors.push("No entry points configured or detected".into());
            }
        } else {
            // Verify at least one standard entry point
            let has_standard = self.entry_points.iter().any(|ep| 
                ep == "main" || 
                ep.ends_with("::main") || 
                ep == "lib" ||
                ep.starts_with("bin/")
            );
            
            if !has_standard {
                errors.push(
                    "No standard entry point found (main, lib, bin/*). \
                     This may cause false dead code positives.".into()
                );
            }
        }
        
        // Validate thresholds
        if self.dead_code_threshold < 0.0 || self.dead_code_threshold > 1.0 {
            errors.push(format!(
                "Invalid dead_code_threshold: {} (must be 0.0-1.0)", 
                self.dead_code_threshold
            ));
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    fn detect_entry_points(&self) -> Vec<String> {
        let mut entry_points = Vec::new();
        
        // Check for binary targets
        if std::path::Path::new("src/main.rs").exists() {
            entry_points.push("main".into());
        }
        
        // Check for library
        if std::path::Path::new("src/lib.rs").exists() {
            entry_points.push("lib".into());
        }
        
        // Check for multiple binaries
        if let Ok(entries) = std::fs::read_dir("src/bin") {
            for entry in entries.flatten() {
                if let Some(name) = entry.path().file_stem() {
                    entry_points.push(format!("bin/{}", name.to_string_lossy()));
                }
            }
        }
        
        entry_points
    }
}
```

## 6. Final Verification Checklist

### 6.1 Dead Code Analysis ✓
- [ ] Run calibration script with known dead code - must detect
- [ ] Verify entry points include: src/bin/pmat.rs main, src/lib.rs exports
- [ ] Check cross-language references if mixed-language project
- [ ] Dead code ratio between 0.5%-15.0% (or documented exception)
- [ ] Manually verify 3 reportedly "live" functions are actually used
- [ ] Zero dead code triggers investigation, not automatic pass

### 6.2 Complexity Analysis ✓
- [ ] Distribution CV > 30% for projects with 50+ functions
- [ ] Entropy > 2.0 for projects with 100+ functions
- [ ] P99 complexity identifies actual complex functions (manual check)
- [ ] Match expressions counted per-arm, not as single branch
- [ ] Closures and async blocks included in parent complexity
- [ ] Correlation with file size calculated (document if < 0.3)

### 6.3 Web Demo State Management ✓
- [ ] All state modifications go through reducer (grep for direct assigns)
- [ ] Reducer passes purity test (frozen input doesn't throw)
- [ ] Key invariants documented and tested
- [ ] No localStorage/sessionStorage in artifact code
- [ ] State transitions complete in < 16ms (measure in tests)
- [ ] Coverage report shows > 80% branch coverage for reducers

### 6.4 Integration Verification ✓
- [ ] `deep_context.md` generates without errors
- [ ] Generated report includes all expected sections
- [ ] Timestamps and version numbers present and accurate
- [ ] File tree reflects actual project structure
- [ ] AST analysis includes at least 50% of source files
- [ ] No empty sections (all have data or explicit "None found")

### 6.5 Regression Prevention ✓
- [ ] Add test case for any fixed issue
- [ ] Document root cause in code comments
- [ ] Verification scripts added to existing test suite
- [ ] Manual verification steps documented in test README
- [ ] Configuration changes documented with rationale

## 7. Status Tracking

Create `.qa-verification.json` in project root after each verification:

```json
{
  "timestamp": "2025-06-04T16:35:57Z",
  "version": "0.21.0",
  "dead_code": {
    "status": "FAIL",
    "expected_range": [0.005, 0.15],
    "actual": 0.0,
    "notes": "Zero detection with TypeScript files present"
  },
  "complexity": {
    "status": "PASS",
    "entropy": 2.3,
    "cv": 45.2,
    "p99": 51
  },
  "provability": {
    "status": "PARTIAL",
    "pure_reducer_coverage": 0.82,
    "state_invariants_tested": 4,
    "notes": "localStorage usage found in app.js"
  },
  "overall": "FAIL"
}
```

This verification protocol leverages existing tooling while providing rigorous quality assurance aligned with Toyota Production System principles. Implementation requires no new infrastructure, only disciplined application of verification procedures using current capabilities.