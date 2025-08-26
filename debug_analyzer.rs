use std::fs;
use std::path::Path;

// Simulate the analyzer logic
fn debug_analyzer() {
    let content = fs::read_to_string("test_project/test.rs").expect("Failed to read test file");
    let path = Path::new("test_project/test.rs");
    
    println!("File path: {:?}", path);
    println!("File extension: {:?}", path.extension());
    
    // Test language detection
    println!("Language detected: {:?}", crate::cli::language_analyzer::Language::from_path(path));
    
    println!("\nFile content:");
    println!("{}", content);
    
    println!("\nTesting line-by-line detection:");
    let lines: Vec<&str> = content.lines().collect();
    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        println!("Line {}: '{}' (trimmed: '{}')", line_num, line, trimmed);
        
        // Test function detection manually
        let is_func = trimmed.starts_with("fn ") ||
                     trimmed.starts_with("pub fn ") ||
                     trimmed.starts_with("async fn ") ||
                     trimmed.starts_with("pub async fn ");
        
        if is_func {
            println!("  -> DETECTED AS FUNCTION");
            
            // Test name extraction
            if let Some(fn_pos) = trimmed.find("fn ") {
                let after_fn = &trimmed[fn_pos + 3..];
                if let Some(paren_pos) = after_fn.find('(') {
                    let name = after_fn[..paren_pos].trim();
                    println!("  -> Function name: '{}'", name);
                }
            }
        }
    }
    
    // Test the actual analyzer
    let analyzer = crate::cli::language_analyzer::RustAnalyzer;
    let functions = analyzer.extract_functions(&content);
    println!("\nActual analyzer results:");
    println!("Functions found: {}", functions.len());
    for func in &functions {
        println!("  - {}: lines {}-{}", func.name, func.line_start, func.line_end);
    }
}

fn main() {
    debug_analyzer();
}