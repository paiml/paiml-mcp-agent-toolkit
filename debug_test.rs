// Quick debug test
fn main() {
    println!("Testing function detection");
    
    let content = r#"fn test_function() {
    println!("hello");
}

pub fn another_function() {
    if true {
        println!("world");
    }
}
"#;
    
    println!("Content to analyze:\n{}", content);
    println!("\nLine by line:");
    
    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        println!("{}: '{}' -> trimmed: '{}'", i, line, trimmed);
        
        let is_func = trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ");
        if is_func {
            println!("  *** FUNCTION DETECTED");
        }
    }
}