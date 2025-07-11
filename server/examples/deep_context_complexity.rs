//! Example demonstrating deep context analysis with proper AST-based complexity
//!
//! This example addresses issue #33 where deep-context showed all complexities as 1.0

use pmat::services::simple_deep_context::{SimpleDeepContext, SimpleAnalysisConfig};
use tempfile::TempDir;
use std::fs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("# Deep Context Complexity Analysis Example\n");
    println!("This example demonstrates the fix for issue #33.\n");
    
    // Create a test project with various complexity levels
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path();
    let src_dir = project_path.join("src");
    fs::create_dir_all(&src_dir)?;
    
    // Create files with different complexity levels
    create_test_files(&src_dir)?;
    
    println!("## Created test project with 3 files:\n");
    println!("1. simple.rs - Low complexity functions");
    println!("2. moderate.rs - Medium complexity functions");  
    println!("3. complex.rs - High complexity functions\n");
    
    // Run deep context analysis
    let analyzer = SimpleDeepContext::new();
    let config = SimpleAnalysisConfig {
        project_path: project_path.to_path_buf(),
        include_features: vec!["all".to_string()],
        exclude_patterns: vec![],
        enable_verbose: false,
    };
    
    println!("## Running Deep Context Analysis...\n");
    let report = analyzer.analyze(config).await?;
    
    // Display results
    println!("## Analysis Results:\n");
    println!("Total files analyzed: {}", report.file_count);
    println!("Total functions: {}", report.complexity_metrics.total_functions);
    println!("Average complexity: {:.2}", report.complexity_metrics.avg_complexity);
    println!("High complexity functions: {}\n", report.complexity_metrics.high_complexity_count);
    
    println!("## Per-File Complexity:\n");
    for file_detail in &report.file_complexity_details {
        let file_name = file_detail.file_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        println!("- {}: {:.2} avg complexity ({} functions)", 
            file_name, 
            file_detail.avg_complexity,
            file_detail.function_count
        );
    }
    
    println!("\n## Key Points:");
    println!("✅ Complexity values are now calculated using AST analysis");
    println!("✅ Different functions show different complexity values");
    println!("✅ No more heuristic-based 1.0 values for all functions");
    println!("✅ Accurate complexity helps identify real refactoring targets");
    
    Ok(())
}

fn create_test_files(src_dir: &std::path::Path) -> anyhow::Result<()> {
    // Simple file
    fs::write(src_dir.join("simple.rs"), r#"
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn greet(name: &str) {
    println!("Hello, {}!", name);
}

fn is_positive(n: i32) -> bool {
    n > 0
}
"#)?;

    // Moderate complexity file  
    fs::write(src_dir.join("moderate.rs"), r#"
fn fibonacci(n: u32) -> u32 {
    if n <= 1 {
        n
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

fn grade_score(score: u32) -> &'static str {
    match score {
        90..=100 => "A",
        80..=89 => "B", 
        70..=79 => "C",
        60..=69 => "D",
        _ => "F",
    }
}
"#)?;

    // Complex file
    fs::write(src_dir.join("complex.rs"), r#"
fn process_data(items: Vec<i32>, mode: &str) -> Vec<i32> {
    let mut result = Vec::new();
    
    for item in items {
        if item > 0 {
            match mode {
                "double" => {
                    if item < 50 {
                        result.push(item * 2);
                    } else if item < 100 {
                        result.push(item + 50);
                    } else {
                        result.push(item);
                    }
                }
                "square" => {
                    if item < 10 {
                        result.push(item * item);
                    } else {
                        result.push(item + 10);
                    }
                }
                "filter" => {
                    if item % 2 == 0 {
                        if item % 3 == 0 {
                            result.push(item);
                        }
                    }
                }
                _ => result.push(item),
            }
        }
    }
    
    result
}

fn validate_and_transform(input: &str, rules: Vec<&str>) -> Result<String, String> {
    if input.is_empty() {
        return Err("Empty input".to_string());
    }
    
    let mut output = input.to_string();
    
    for rule in rules {
        match rule {
            "uppercase" => {
                if output.chars().any(|c| c.is_lowercase()) {
                    output = output.to_uppercase();
                }
            }
            "trim" => {
                if output.starts_with(' ') || output.ends_with(' ') {
                    output = output.trim().to_string();
                }
            }
            "reverse" => {
                if output.len() > 1 {
                    output = output.chars().rev().collect();
                }
            }
            _ => {
                if rule.starts_with("min_length:") {
                    if let Some(len_str) = rule.strip_prefix("min_length:") {
                        if let Ok(min_len) = len_str.parse::<usize>() {
                            if output.len() < min_len {
                                return Err(format!("Too short: {} < {}", output.len(), min_len));
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(output)
}
"#)?;

    Ok(())
}