//! EXTREME TDD: Language Support Integration for Deep Context
//! RED-GREEN-REFACTOR implementation of TICKET-2001 through TICKET-2005

use std::fs;
use tempfile::TempDir;

/// TICKET-2001: RED TEST - C# support in deep_context pipeline
#[tokio::test]
async fn test_csharp_deep_context_analysis() {
    // ARRANGE: Create C# file with known complexity
    let temp_dir = TempDir::new().unwrap();
    let csharp_file = temp_dir.path().join("Program.cs");
    fs::write(
        &csharp_file,
        r#"
using System;

namespace TestApp
{
    class Program
    {
        // Simple function - complexity 1
        static void Main(string[] args)
        {
            Console.WriteLine("Hello World!");
        }

        // Complex function with conditionals - complexity 4
        static int CalculateScore(int value, bool isBonus)
        {
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
        include_patterns: vec!["**/*.cs".to_string()],
        exclude_patterns: vec![],
        enable_verbose: false,
    };

    let report = analyzer.analyze(config).await.unwrap();

    // ASSERT: Should detect C# functions and provide complexity analysis
    assert!(report.file_count > 0, "Should find C# files");
    assert!(
        report.complexity_metrics.total_functions >= 2,
        "Should detect at least 2 C# functions (Main and CalculateScore)"
    );
    assert!(
        report.complexity_metrics.avg_complexity > 1.0,
        "Should calculate meaningful complexity for C# code"
    );

    // Verify specific file analysis
    let csharp_details = report
        .file_complexity_details
        .iter()
        .find(|d| d.file_path.file_name().unwrap() == "Program.cs")
        .expect("Should find Program.cs in analysis results");

    // Debug output
    eprintln!(
        "C# details: function_count={}, function_names={:?}",
        csharp_details.function_count, csharp_details.function_names
    );

    assert!(
        csharp_details.function_count >= 2,
        "Should detect C# methods"
    );

    // Check if function names contain Main and CalculateScore (with or without namespace)
    let has_main = csharp_details
        .function_names
        .iter()
        .any(|n| n.contains("Main"));
    let has_calculate = csharp_details
        .function_names
        .iter()
        .any(|n| n.contains("CalculateScore"));

    assert!(
        has_main && has_calculate,
        "Should extract C# function names Main and CalculateScore: found {:?}",
        csharp_details.function_names
    );
}

/// TICKET-2002: RED TEST - Go support in deep_context pipeline
#[tokio::test]
async fn test_go_deep_context_analysis() {
    // ARRANGE: Create Go file with known complexity
    let temp_dir = TempDir::new().unwrap();
    let go_file = temp_dir.path().join("main.go");
    fs::write(
        &go_file,
        r#"
package main

import "fmt"

// Simple function - complexity 1
func main() {
    fmt.Println("Hello, World!")
}

// Complex function with switch and loops - complexity 5
func ProcessData(data []int, operation string) int {
    result := 0

    switch operation {
    case "sum":
        for _, val := range data {
            result += val
        }
    case "max":
        for _, val := range data {
            if val > result {
                result = val
            }
        }
    default:
        return -1
    }

    return result
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
        include_patterns: vec!["**/*.go".to_string()],
        exclude_patterns: vec![],
        enable_verbose: false,
    };

    let report = analyzer.analyze(config).await.unwrap();

    // ASSERT: Should detect Go functions and provide complexity analysis
    assert!(report.file_count > 0, "Should find Go files");
    assert!(
        report.complexity_metrics.total_functions >= 2,
        "Should detect at least 2 Go functions"
    );
    assert!(
        report.complexity_metrics.avg_complexity > 1.0,
        "Should calculate meaningful complexity for Go code"
    );

    // Verify function name extraction
    let go_details = report
        .file_complexity_details
        .iter()
        .find(|d| d.file_path.file_name().unwrap() == "main.go")
        .expect("Should find main.go in analysis results");

    assert!(
        go_details.function_names.len() >= 2,
        "Should extract Go function names"
    );
}

/// TICKET-2003: RED TEST - Java support in deep_context pipeline
#[tokio::test]
async fn test_java_deep_context_analysis() {
    // ARRANGE: Create Java file with known complexity
    let temp_dir = TempDir::new().unwrap();
    let java_file = temp_dir.path().join("Calculator.java");
    fs::write(
        &java_file,
        r#"
public class Calculator {

    // Simple method - complexity 1
    public static void main(String[] args) {
        System.out.println("Calculator ready");
    }

    // Complex method with nested conditions - complexity 6
    public int calculate(int a, int b, String operation) {
        if (operation == null) {
            throw new IllegalArgumentException("Operation cannot be null");
        }

        try {
            switch (operation.toLowerCase()) {
                case "add":
                    return a + b;
                case "subtract":
                    return a - b;
                case "multiply":
                    if (b == 0) {
                        return 0;
                    }
                    return a * b;
                case "divide":
                    if (b == 0) {
                        throw new ArithmeticException("Division by zero");
                    }
                    return a / b;
                default:
                    throw new UnsupportedOperationException("Unknown operation: " + operation);
            }
        } catch (Exception e) {
            System.err.println("Error: " + e.getMessage());
            return 0;
        }
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
        include_patterns: vec!["**/*.java".to_string()],
        exclude_patterns: vec![],
        enable_verbose: false,
    };

    let report = analyzer.analyze(config).await.unwrap();

    // ASSERT: Should detect Java methods and provide complexity analysis
    assert!(report.file_count > 0, "Should find Java files");
    assert!(
        report.complexity_metrics.total_functions >= 2,
        "Should detect at least 2 Java methods"
    );
    assert!(
        report.complexity_metrics.avg_complexity > 1.0,
        "Should calculate meaningful complexity for Java code"
    );
}

/// TICKET-2004: RED TEST - Kotlin support in deep_context pipeline
#[tokio::test]
async fn test_kotlin_deep_context_analysis() {
    // ARRANGE: Create Kotlin file with known complexity
    let temp_dir = TempDir::new().unwrap();
    let kotlin_file = temp_dir.path().join("Main.kt");
    fs::write(
        &kotlin_file,
        r#"
fun main() {
    println("Hello, Kotlin!")
}

// Complex function with when expression and loops - complexity 4
fun processNumbers(numbers: List<Int>, operation: String): Int {
    return when (operation) {
        "sum" -> {
            var total = 0
            for (num in numbers) {
                total += num
            }
            total
        }
        "product" -> {
            var result = 1
            for (num in numbers) {
                if (num == 0) {
                    return 0
                }
                result *= num
            }
            result
        }
        else -> -1
    }
}

data class Result(val value: Int, val valid: Boolean)
"#,
    )
    .unwrap();

    // ACT: Analyze using simple deep context
    use crate::services::simple_deep_context::{SimpleAnalysisConfig, SimpleDeepContext};

    let analyzer = SimpleDeepContext::new();
    let config = SimpleAnalysisConfig {
        project_path: temp_dir.path().to_path_buf(),
        include_features: vec![],
        include_patterns: vec!["**/*.kt".to_string()],
        exclude_patterns: vec![],
        enable_verbose: false,
    };

    let report = analyzer.analyze(config).await.unwrap();

    // ASSERT: Should detect Kotlin functions and provide complexity analysis
    assert!(report.file_count > 0, "Should find Kotlin files");
    assert!(
        report.complexity_metrics.total_functions >= 2,
        "Should detect at least 2 Kotlin functions"
    );
    assert!(
        report.complexity_metrics.avg_complexity > 1.0,
        "Should calculate meaningful complexity for Kotlin code"
    );
}

/// TICKET-2005: RED TEST - Ruby support in deep_context pipeline
#[tokio::test]
async fn test_ruby_deep_context_analysis() {
    // ARRANGE: Create Ruby file with known complexity
    let temp_dir = TempDir::new().unwrap();
    let ruby_file = temp_dir.path().join("calculator.rb");
    fs::write(
        &ruby_file,
        r#"
# Simple method - complexity 1
def greet(name)
  puts "Hello, " + name + "!"
end

# Complex method with nested conditionals - complexity 5
def calculate_grade(score, bonus = false)
  return "F" if score < 0 || score > 100

  adjusted_score = score
  if bonus
    adjusted_score += 10 if score < 90
  end

  case adjusted_score
  when 90..100
    "A"
  when 80..89
    "B"
  when 70..79
    "C"
  when 60..69
    "D"
  else
    "F"
  end
end

class Student
  def initialize(name)
    @name = name
  end

  def study
    puts @name + " is studying"
  end
end
"#,
    )
    .unwrap();

    // ACT: Analyze using simple deep context
    use crate::services::simple_deep_context::{SimpleAnalysisConfig, SimpleDeepContext};

    let analyzer = SimpleDeepContext::new();
    let config = SimpleAnalysisConfig {
        project_path: temp_dir.path().to_path_buf(),
        include_features: vec![],
        include_patterns: vec!["**/*.rb".to_string()],
        exclude_patterns: vec![],
        enable_verbose: false,
    };

    let report = analyzer.analyze(config).await.unwrap();

    // ASSERT: Should detect Ruby methods and provide complexity analysis
    assert!(report.file_count > 0, "Should find Ruby files");
    assert!(
        report.complexity_metrics.total_functions >= 3,
        "Should detect at least 3 Ruby methods"
    );
    assert!(
        report.complexity_metrics.avg_complexity > 1.0,
        "Should calculate meaningful complexity for Ruby code"
    );
}
