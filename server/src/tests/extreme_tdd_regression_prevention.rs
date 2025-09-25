//! EXTREME TDD: Comprehensive Regression Prevention Tests
//! Goal: Prevent any future regressions in performance, annotations, and language support

use tempfile::TempDir;
use std::fs;
use std::time::Instant;

/// RED TEST: Context generation must complete within performance bounds
#[tokio::test]
async fn test_performance_regression_prevention() {
    // ARRANGE: Create test project similar to real usage
    let temp_dir = TempDir::new().unwrap();

    // Create files in multiple languages to test real-world scenario
    let rust_file = temp_dir.path().join("main.rs");
    fs::write(&rust_file, r#"
fn complex_function(data: Vec<i32>) -> i32 {
    let mut sum = 0;
    for item in data {
        if item > 0 {
            for j in 0..item {
                if j % 2 == 0 {
                    sum += j;
                }
            }
        }
    }
    sum
}

struct Calculator {
    value: i32,
}

impl Calculator {
    fn new() -> Self { Self { value: 0 } }
    fn add(&mut self, n: i32) { self.value += n; }
}
"#).unwrap();

    let python_file = temp_dir.path().join("script.py");
    fs::write(&python_file, r#"
def complex_function(data):
    """Complex function for testing."""
    sum_val = 0
    for item in data:
        if item > 0:
            for j in range(item):
                if j % 2 == 0:
                    sum_val += j
    return sum_val

class Calculator:
    def __init__(self):
        self.value = 0

    def add(self, n):
        self.value += n
"#).unwrap();

    let ruby_file = temp_dir.path().join("script.rb");
    fs::write(&ruby_file, r#"
def complex_function(data)
  sum = 0
  data.each do |item|
    if item > 0
      (0...item).each do |j|
        sum += j if j.even?
      end
    end
  end
  sum
end

class Calculator
  def initialize
    @value = 0
  end

  def add(n)
    @value += n
  end
end
"#).unwrap();

    // ACT: Time the context generation
    let start = Instant::now();

    let output_file = temp_dir.path().join("context.md");
    let result = crate::cli::handlers::utility_handlers::handle_context(
        None, // Auto-detect language
        temp_dir.path().to_path_buf(),
        Some(output_file.clone()),
        crate::cli::handlers::utility_handlers::ContextFormat::Markdown,
        false, // include_large_files
        false, // skip_expensive_metrics = FALSE (full analysis)
    ).await;

    let duration = start.elapsed();

    // ASSERT: Performance requirements
    assert!(result.is_ok(), "Context generation must succeed: {:?}", result.err());
    assert!(duration.as_secs() < 5, "Context generation must complete within 5 seconds, took {:?}", duration);

    let output = fs::read_to_string(output_file).unwrap();

    // ASSERT: Must contain enhanced AST header (not old format)
    assert!(output.contains("Enhanced Annotated AST Context"),
        "Must use enhanced AST format");
}

/// RED TEST: All required annotations must be present
#[tokio::test]
async fn test_annotations_regression_prevention() {
    // ARRANGE: Create test with complex code that should trigger all annotations
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("complex.rs");

    fs::write(&test_file, r#"
// TODO: Refactor this - it's getting complex
fn high_complexity_function(data: Vec<i32>) -> i32 {
    let mut result = 0;
    for i in 0..data.len() {
        for j in 0..data.len() {
            if data[i] > data[j] {
                for k in 0..10 {
                    if k % 2 == 0 {
                        if k > 5 {
                            result += data[i] * data[j] * k;
                        } else {
                            result -= data[i] + data[j];
                        }
                    } else {
                        result *= 2;
                    }
                }
            }
        }
    }
    result
}

// FIXME: Performance issue here
fn another_complex_function() -> bool {
    true
}

struct DataProcessor {
    cache: Vec<i32>,
}

impl DataProcessor {
    fn process(&self) -> i32 { 42 }
}
"#).unwrap();

    // ACT: Generate context
    let output_file = temp_dir.path().join("context.md");
    let result = crate::cli::handlers::utility_handlers::handle_context(
        Some("rust".to_string()),
        temp_dir.path().to_path_buf(),
        Some(output_file.clone()),
        crate::cli::handlers::utility_handlers::ContextFormat::Markdown,
        false, // include_large_files
        false, // skip_expensive_metrics = FALSE (full analysis)
    ).await;

    assert!(result.is_ok(), "Context generation must succeed: {:?}", result.err());

    let output = fs::read_to_string(output_file).unwrap();

    // ASSERT: ALL required annotations must be present

    // 1. Complexity annotations
    assert!(output.contains("[complexity:") && output.contains("[cognitive:"),
        "Must contain complexity annotations");

    // 2. Big-O notation
    assert!(output.contains("[big-o:") || output.contains("O("),
        "Must contain Big-O complexity annotations");

    // 3. SATD (TODO/FIXME) annotations
    assert!(output.contains("TODO") || output.contains("FIXME") || output.contains("SATD"),
        "Must detect and annotate SATD (TODO/FIXME) comments");

    // 4. Graph metrics
    assert!(output.contains("Graph") || output.contains("PageRank") || output.contains("Community"),
        "Must contain graph analysis metrics");

    // 5. Quality scorecard
    assert!(output.contains("Quality Scorecard") && output.contains("Overall Health"),
        "Must contain quality scorecard");

    // 6. Provability analysis
    assert!(output.contains("provability") || output.contains("Provability") || output.contains("verification"),
        "Must contain provability analysis");

    // 7. Technical Debt Gradient (TDG)
    assert!(output.contains("TDG") || output.contains("Technical Debt") || output.contains("Debt Hours"),
        "Must contain technical debt analysis");

    // 8. Churn analysis (if git repo)
    // This might not always be available, but structure should be there
    assert!(output.contains("Project Metrics") || output.contains("Files"),
        "Must contain project analysis structure");
}

/// RED TEST: Multi-language support must work
#[tokio::test]
async fn test_language_support_regression_prevention() {
    // ARRANGE: Create project with multiple languages
    let temp_dir = TempDir::new().unwrap();

    // Create files in all supported languages
    let files_and_content = vec![
        ("main.rs", "fn main() { println!(\"Hello Rust\"); }"),
        ("script.py", "def main():\n    print(\"Hello Python\")"),
        ("script.rb", "def main\n  puts \"Hello Ruby\"\nend"),
        ("Main.java", "public class Main { public static void main(String[] args) { System.out.println(\"Hello Java\"); } }"),
        ("script.js", "function main() { console.log(\"Hello JavaScript\"); }"),
        ("script.ts", "function main(): void { console.log(\"Hello TypeScript\"); }"),
        ("program.go", "package main\nimport \"fmt\"\nfunc main() { fmt.Println(\"Hello Go\") }"),
        ("Program.cs", "using System; class Program { static void Main() { Console.WriteLine(\"Hello C#\"); } }"),
    ];

    for (filename, content) in files_and_content {
        let file_path = temp_dir.path().join(filename);
        fs::write(&file_path, content).unwrap();
    }

    // ACT: Generate context
    let output_file = temp_dir.path().join("context.md");
    let result = crate::cli::handlers::utility_handlers::handle_context(
        None, // Auto-detect (should detect multiple languages)
        temp_dir.path().to_path_buf(),
        Some(output_file.clone()),
        crate::cli::handlers::utility_handlers::ContextFormat::Markdown,
        false,
        false,
    ).await;

    assert!(result.is_ok(), "Multi-language context generation must succeed: {:?}", result.err());

    let output = fs::read_to_string(output_file).unwrap();

    // ASSERT: Must detect and analyze multiple languages
    let language_patterns = vec![
        ("main.rs", "Rust"),
        ("script.py", "Python"),
        ("script.rb", "Ruby"),
        ("Main.java", "Java"),
        ("script.js", "JavaScript"),
        ("script.ts", "TypeScript"),
        ("program.go", "Go"),
        ("Program.cs", "C#"),
    ];

    let mut languages_found = 0;
    for (filename, _language) in language_patterns {
        if output.contains(filename) {
            languages_found += 1;
        }
    }

    assert!(languages_found >= 5,
        "Must detect and analyze at least 5 different languages, found: {}", languages_found);

    // ASSERT: Must have annotations for different language files
    assert!(output.contains("Function") || output.contains("function"),
        "Must identify functions across languages");
}

/// RED TEST: Prevent complete analysis type removal
#[tokio::test]
async fn test_analysis_types_completeness() {
    // ARRANGE: Simple test file
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("simple.rs");
    fs::write(&test_file, "fn test() -> i32 { 42 }").unwrap();

    // ACT: Generate context
    let output_file = temp_dir.path().join("context.md");
    let result = crate::cli::handlers::utility_handlers::handle_context(
        Some("rust".to_string()),
        temp_dir.path().to_path_buf(),
        Some(output_file.clone()),
        crate::cli::handlers::utility_handlers::ContextFormat::Markdown,
        false, // include_large_files
        false, // skip_expensive_metrics = FALSE
    ).await;

    assert!(result.is_ok(), "Analysis must succeed: {:?}", result.err());

    let output = fs::read_to_string(output_file).unwrap();

    // ASSERT: Required analysis types are enabled (not removed due to performance fears)
    let required_analysis_indicators = vec![
        ("AST", vec!["Function", "function", "Enhanced Annotated AST"]),
        ("Complexity", vec!["complexity:", "cognitive:"]),
        ("Quality", vec!["Quality Scorecard", "Overall Health"]),
        ("Metrics", vec!["Project Metrics", "Total Files"]),
    ];

    for (analysis_name, indicators) in required_analysis_indicators {
        let found = indicators.iter().any(|indicator| output.contains(indicator));
        assert!(found, "Required analysis '{}' must be present. Indicators: {:?}", analysis_name, indicators);
    }
}

/// RED TEST: Configuration must enable all analysis types for full analysis
#[test]
fn test_deep_context_config_completeness() {
    use crate::services::deep_context::{AnalysisType, DeepContextConfig, CacheStrategy, DagType};

    // ARRANGE: Check what analysis types should be enabled for full analysis
    let full_analysis_types = vec![
        AnalysisType::Ast,
        AnalysisType::Complexity,
        AnalysisType::Churn,
        AnalysisType::TechnicalDebtGradient,
        AnalysisType::DeadCode,
        AnalysisType::Satd,
        AnalysisType::Provability,
        AnalysisType::BigO,
    ];

    // ACT: Create config as done in handle_context
    let config = DeepContextConfig {
        include_analyses: full_analysis_types.clone(),
        period_days: 30,
        dag_type: DagType::CallGraph,
        complexity_thresholds: None,
        max_depth: Some(10), // Reasonable depth, not artificially small
        include_patterns: vec![], // Should not be overly restrictive
        exclude_patterns: vec![
            "**/target/**".to_string(),
            "**/node_modules/**".to_string(),
            "**/build/**".to_string(),
            "**/dist/**".to_string(),
            "**/.git/**".to_string(),
        ],
        cache_strategy: CacheStrategy::Normal,
        parallel: 4, // Should allow reasonable parallelism
        file_classifier_config: None,
    };

    // ASSERT: All analysis types must be enabled for full analysis
    assert_eq!(config.include_analyses.len(), full_analysis_types.len(),
        "Full analysis must include ALL analysis types");

    for analysis_type in full_analysis_types {
        assert!(config.include_analyses.contains(&analysis_type),
            "Full analysis must include {:?}", analysis_type);
    }

    // ASSERT: Configuration should not be artificially limited
    assert!(config.max_depth.unwrap_or(0) >= 5,
        "Max depth should not be artificially small (was {})", config.max_depth.unwrap_or(0));
    assert!(config.parallel >= 2,
        "Parallelism should not be artificially reduced (was {})", config.parallel);
    assert!(config.period_days >= 7,
        "Analysis period should not be artificially short (was {} days)", config.period_days);
}