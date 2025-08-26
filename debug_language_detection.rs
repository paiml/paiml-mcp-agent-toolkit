// Debug the exact line numbers being returned
use std::fs;

fn main() {
    let content = fs::read_to_string("test_project/main.rs").expect("Failed to read test file");
    
    println!("Content lines:");
    let lines: Vec<&str> = content.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        println!("Line {}: '{}'", i, line);
    }
    
    println!("\nTotal lines: {}", lines.len());
    println!("lines.len() - 1 = {}", lines.len() - 1);
    
    // Test the function detection manually
    let analyzer = crate::cli::language_analyzer::RustAnalyzer;
    let functions = analyzer.extract_functions(&content);
    
    for func in &functions {
        println!("Function: {}, Start: {}, End: {}", func.name, func.line_start, func.line_end);
        
        let actual_lines = &lines[func.line_start..=func.line_end.min(lines.len()-1)];
        println!("Actual function content:");
        for (i, line) in actual_lines.iter().enumerate() {
            println!("  {}: {}", func.line_start + i, line);
        }
    }
}