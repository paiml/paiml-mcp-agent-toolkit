//! Language Regression Tests - Priority 3
//! Comprehensive integration tests for all 15 supported languages
//! Prevents future multi-language bugs by validating each language independently

use std::fs;
use tempfile::TempDir;

/// Regression Test: C language support in deep_context pipeline
#[tokio::test]
async fn test_c_deep_context_analysis() {
    // ARRANGE: Create C file with known complexity
    let temp_dir = TempDir::new().unwrap();
    let c_file = temp_dir.path().join("main.c");
    fs::write(
        &c_file,
        r#"
#include <stdio.h>
#include <stdlib.h>

// Simple function - complexity 1
void print_hello() {
    printf("Hello World!\n");
}

// Complex function with conditionals - complexity 4
int calculate_score(int value, int is_bonus) {
    if (value < 0) {
        return 0;
    }

    if (is_bonus) {
        if (value > 100) {
            return value * 2;
        } else {
            return value + 50;
        }
    }

    return value;
}

// Function with loop - complexity 3
int sum_array(int *arr, int size) {
    int sum = 0;
    for (int i = 0; i < size; i++) {
        if (arr[i] > 0) {
            sum += arr[i];
        }
    }
    return sum;
}
"#,
    )
    .unwrap();

    // ACT: Analyze using simple deep context
    use crate::services::simple_deep_context::{SimpleAnalysisConfig, SimpleDeepContext};

    let analyzer = SimpleDeepContext::new();
    let config = SimpleAnalysisConfig {
        project_path: temp_dir.path().to_path_buf(),
        include_features: vec![],
        include_patterns: vec!["**/*.c".to_string()],
        exclude_patterns: vec![],
        enable_verbose: false,
    };

    let report = analyzer.analyze(config).await.unwrap();

    // ASSERT: Should detect C functions and provide complexity analysis
    assert!(report.file_count > 0, "Should find C files");
    assert!(
        report.complexity_metrics.total_functions >= 3,
        "Should detect at least 3 C functions (print_hello, calculate_score, sum_array)"
    );
    assert!(
        report.complexity_metrics.avg_complexity > 1.0,
        "Should calculate meaningful complexity for C code"
    );

    // Verify specific file analysis
    let c_details = report
        .file_complexity_details
        .iter()
        .find(|d| d.file_path.file_name().unwrap() == "main.c")
        .expect("Should find main.c in analysis results");

    eprintln!(
        "C details: function_count={}, function_names={:?}",
        c_details.function_count, c_details.function_names
    );

    assert!(
        c_details.function_count >= 3,
        "Should detect C functions"
    );
}

/// Regression Test: C++ language support in deep_context pipeline
/// C++ function extraction improved to detect both top-level functions and class methods
#[tokio::test]
async fn test_cpp_deep_context_analysis() {
    // ARRANGE: Create C++ file with known complexity
    let temp_dir = TempDir::new().unwrap();
    let cpp_file = temp_dir.path().join("main.cpp");
    fs::write(
        &cpp_file,
        r#"
#include <iostream>
#include <vector>
#include <string>

// Simple function - complexity 1
void printHello() {
    std::cout << "Hello World!" << std::endl;
}

// Complex function with conditionals - complexity 4
int calculateScore(int value, bool isBonus) {
    if (value < 0) {
        return 0;
    }

    if (isBonus) {
        if (value > 100) {
            return value * 2;
        } else {
            return value + 50;
        }
    }

    return value;
}

// Template function - complexity 2
template<typename T>
T max(T a, T b) {
    return (a > b) ? a : b;
}

// Class with methods
class Calculator {
public:
    int add(int a, int b) {
        return a + b;
    }

    int multiply(int a, int b) {
        return a * b;
    }

    // Complex method - complexity 3
    int complexOperation(int x) {
        if (x < 0) {
            return -1;
        } else if (x == 0) {
            return 0;
        }
        return x * x;
    }
};
"#,
    )
    .unwrap();

    // ACT: Analyze using simple deep context
    use crate::services::simple_deep_context::{SimpleAnalysisConfig, SimpleDeepContext};

    let analyzer = SimpleDeepContext::new();
    let config = SimpleAnalysisConfig {
        project_path: temp_dir.path().to_path_buf(),
        include_features: vec![],
        include_patterns: vec!["**/*.cpp".to_string()],
        exclude_patterns: vec![],
        enable_verbose: false,
    };

    let report = analyzer.analyze(config).await.unwrap();

    // ASSERT: Should detect C++ functions and provide complexity analysis
    assert!(report.file_count > 0, "Should find C++ files");
    assert!(
        report.complexity_metrics.total_functions >= 4,
        "Should detect at least 4 C++ functions/methods"
    );
    assert!(
        report.complexity_metrics.avg_complexity > 1.0,
        "Should calculate meaningful complexity for C++ code"
    );

    // Verify specific file analysis
    let cpp_details = report
        .file_complexity_details
        .iter()
        .find(|d| d.file_path.file_name().unwrap() == "main.cpp")
        .expect("Should find main.cpp in analysis results");

    eprintln!(
        "C++ details: function_count={}, function_names={:?}",
        cpp_details.function_count, cpp_details.function_names
    );

    assert!(
        cpp_details.function_count >= 4,
        "Should detect C++ functions and methods"
    );
}

/// Regression Test: Bash language support in deep_context pipeline
/// Uses BashScriptAnalyzer to extract function definitions
#[tokio::test]
async fn test_bash_deep_context_analysis() {
    // ARRANGE: Create Bash file with known complexity
    let temp_dir = TempDir::new().unwrap();
    let bash_file = temp_dir.path().join("script.sh");
    fs::write(
        &bash_file,
        r#"#!/bin/bash

# Simple function - complexity 1
print_hello() {
    echo "Hello World!"
}

# Complex function with conditionals - complexity 4
calculate_score() {
    local value=$1
    local is_bonus=$2

    if [ "$value" -lt 0 ]; then
        echo 0
        return
    fi

    if [ "$is_bonus" = "true" ]; then
        if [ "$value" -gt 100 ]; then
            echo $((value * 2))
        else
            echo $((value + 50))
        fi
    else
        echo "$value"
    fi
}

# Function with loop - complexity 3
sum_array() {
    local sum=0
    for num in "$@"; do
        if [ "$num" -gt 0 ]; then
            sum=$((sum + num))
        fi
    done
    echo $sum
}

# Main execution
main() {
    print_hello
    score=$(calculate_score 150 true)
    echo "Score: $score"
}

main "$@"
"#,
    )
    .unwrap();

    // ACT: Analyze using simple deep context
    use crate::services::simple_deep_context::{SimpleAnalysisConfig, SimpleDeepContext};

    let analyzer = SimpleDeepContext::new();
    let config = SimpleAnalysisConfig {
        project_path: temp_dir.path().to_path_buf(),
        include_features: vec![],
        include_patterns: vec!["**/*.sh".to_string()],
        exclude_patterns: vec![],
        enable_verbose: false,
    };

    let report = analyzer.analyze(config).await.unwrap();

    // ASSERT: Should detect Bash functions and provide complexity analysis
    assert!(report.file_count > 0, "Should find Bash files");
    assert!(
        report.complexity_metrics.total_functions >= 3,
        "Should detect at least 3 Bash functions (print_hello, calculate_score, sum_array, main)"
    );

    // Verify specific file analysis
    let bash_details = report
        .file_complexity_details
        .iter()
        .find(|d| d.file_path.file_name().unwrap() == "script.sh")
        .expect("Should find script.sh in analysis results");

    eprintln!(
        "Bash details: function_count={}, function_names={:?}",
        bash_details.function_count, bash_details.function_names
    );

    assert!(
        bash_details.function_count >= 3,
        "Should detect Bash functions"
    );
}

/// Regression Test: PHP language support in deep_context pipeline
/// PHP AST parser is implemented and integrated into the deep_context pipeline
#[tokio::test]
async fn test_php_deep_context_analysis() {
    // ARRANGE: Create PHP file with known complexity
    let temp_dir = TempDir::new().unwrap();
    let php_file = temp_dir.path().join("index.php");
    fs::write(
        &php_file,
        r#"<?php

// Simple function - complexity 1
function printHello() {
    echo "Hello World!\n";
}

// Complex function with conditionals - complexity 4
function calculateScore($value, $isBonus) {
    if ($value < 0) {
        return 0;
    }

    if ($isBonus) {
        if ($value > 100) {
            return $value * 2;
        } else {
            return $value + 50;
        }
    }

    return $value;
}

// Function with loop - complexity 3
function sumArray($arr) {
    $sum = 0;
    foreach ($arr as $num) {
        if ($num > 0) {
            $sum += $num;
        }
    }
    return $sum;
}

// Class with methods
class Calculator {
    public function add($a, $b) {
        return $a + $b;
    }

    public function multiply($a, $b) {
        return $a * $b;
    }

    // Complex method - complexity 3
    public function complexOperation($x) {
        if ($x < 0) {
            return -1;
        } elseif ($x == 0) {
            return 0;
        }
        return $x * $x;
    }
}

?>"#,
    )
    .unwrap();

    // ACT: Analyze using simple deep context
    use crate::services::simple_deep_context::{SimpleAnalysisConfig, SimpleDeepContext};

    let analyzer = SimpleDeepContext::new();
    let config = SimpleAnalysisConfig {
        project_path: temp_dir.path().to_path_buf(),
        include_features: vec![],
        include_patterns: vec!["**/*.php".to_string()],
        exclude_patterns: vec![],
        enable_verbose: false,
    };

    let report = analyzer.analyze(config).await.unwrap();

    // ASSERT: Should detect PHP functions and provide complexity analysis
    assert!(report.file_count > 0, "Should find PHP files");
    assert!(
        report.complexity_metrics.total_functions >= 3,
        "Should detect at least 3 PHP functions"
    );
    assert!(
        report.complexity_metrics.avg_complexity > 1.0,
        "Should calculate meaningful complexity for PHP code"
    );

    // Verify specific file analysis
    let php_details = report
        .file_complexity_details
        .iter()
        .find(|d| d.file_path.file_name().unwrap() == "index.php")
        .expect("Should find index.php in analysis results");

    eprintln!(
        "PHP details: function_count={}, function_names={:?}",
        php_details.function_count, php_details.function_names
    );

    assert!(
        php_details.function_count >= 3,
        "Should detect PHP functions"
    );
}

/// Regression Test: Swift language support in deep_context pipeline
/// Swift AST parser is implemented and integrated into the deep_context pipeline
#[tokio::test]
async fn test_swift_deep_context_analysis() {
    // ARRANGE: Create Swift file with known complexity
    let temp_dir = TempDir::new().unwrap();
    let swift_file = temp_dir.path().join("main.swift");
    fs::write(
        &swift_file,
        r#"
import Foundation

// Simple function - complexity 1
func printHello() {
    print("Hello World!")
}

// Complex function with conditionals - complexity 4
func calculateScore(value: Int, isBonus: Bool) -> Int {
    if value < 0 {
        return 0
    }

    if isBonus {
        if value > 100 {
            return value * 2
        } else {
            return value + 50
        }
    }

    return value
}

// Function with loop - complexity 3
func sumArray(arr: [Int]) -> Int {
    var sum = 0
    for num in arr {
        if num > 0 {
            sum += num
        }
    }
    return sum
}

// Class with methods
class Calculator {
    func add(_ a: Int, _ b: Int) -> Int {
        return a + b
    }

    func multiply(_ a: Int, _ b: Int) -> Int {
        return a * b
    }

    // Complex method - complexity 3
    func complexOperation(_ x: Int) -> Int {
        if x < 0 {
            return -1
        } else if x == 0 {
            return 0
        }
        return x * x
    }
}
"#,
    )
    .unwrap();

    // ACT: Analyze using simple deep context
    use crate::services::simple_deep_context::{SimpleAnalysisConfig, SimpleDeepContext};

    let analyzer = SimpleDeepContext::new();
    let config = SimpleAnalysisConfig {
        project_path: temp_dir.path().to_path_buf(),
        include_features: vec![],
        include_patterns: vec!["**/*.swift".to_string()],
        exclude_patterns: vec![],
        enable_verbose: false,
    };

    let report = analyzer.analyze(config).await.unwrap();

    // ASSERT: Should detect Swift functions and provide complexity analysis
    assert!(report.file_count > 0, "Should find Swift files");
    assert!(
        report.complexity_metrics.total_functions >= 3,
        "Should detect at least 3 Swift functions"
    );
    assert!(
        report.complexity_metrics.avg_complexity > 1.0,
        "Should calculate meaningful complexity for Swift code"
    );

    // Verify specific file analysis
    let swift_details = report
        .file_complexity_details
        .iter()
        .find(|d| d.file_path.file_name().unwrap() == "main.swift")
        .expect("Should find main.swift in analysis results");

    eprintln!(
        "Swift details: function_count={}, function_names={:?}",
        swift_details.function_count, swift_details.function_names
    );

    assert!(
        swift_details.function_count >= 3,
        "Should detect Swift functions"
    );
}

/// Regression Test: WASM language support in deep_context pipeline
#[tokio::test]
async fn test_wasm_deep_context_analysis() {
    // ARRANGE: Create WAT (WebAssembly Text) file with known complexity
    let temp_dir = TempDir::new().unwrap();
    let wat_file = temp_dir.path().join("module.wat");
    fs::write(
        &wat_file,
        r#"(module
  ;; Simple function - complexity 1
  (func $add (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.add
  )

  ;; Complex function with conditionals - complexity 3
  (func $max (param $a i32) (param $b i32) (result i32)
    (if (result i32)
      (i32.gt_s (local.get $a) (local.get $b))
      (then (local.get $a))
      (else (local.get $b))
    )
  )

  ;; Function with loop - complexity 4
  (func $sum_to_n (param $n i32) (result i32)
    (local $sum i32)
    (local $i i32)
    (local.set $sum (i32.const 0))
    (local.set $i (i32.const 1))
    (block $break
      (loop $continue
        (br_if $break (i32.gt_s (local.get $i) (local.get $n)))
        (local.set $sum (i32.add (local.get $sum) (local.get $i)))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $continue)
      )
    )
    (local.get $sum)
  )

  (export "add" (func $add))
  (export "max" (func $max))
  (export "sum_to_n" (func $sum_to_n))
)"#,
    )
    .unwrap();

    // ACT: Analyze using simple deep context
    use crate::services::simple_deep_context::{SimpleAnalysisConfig, SimpleDeepContext};

    let analyzer = SimpleDeepContext::new();
    let config = SimpleAnalysisConfig {
        project_path: temp_dir.path().to_path_buf(),
        include_features: vec![],
        include_patterns: vec!["**/*.wat".to_string()],
        exclude_patterns: vec![],
        enable_verbose: false,
    };

    let report = analyzer.analyze(config).await.unwrap();

    // ASSERT: Should detect WASM functions and provide complexity analysis
    assert!(report.file_count > 0, "Should find WASM files");
    assert!(
        report.complexity_metrics.total_functions >= 3,
        "Should detect at least 3 WASM functions (add, max, sum_to_n)"
    );

    // Verify specific file analysis
    let wasm_details = report
        .file_complexity_details
        .iter()
        .find(|d| d.file_path.file_name().unwrap() == "module.wat")
        .expect("Should find module.wat in analysis results");

    eprintln!(
        "WASM details: function_count={}, function_names={:?}",
        wasm_details.function_count, wasm_details.function_names
    );

    assert!(
        wasm_details.function_count >= 3,
        "Should detect WASM functions"
    );
}
