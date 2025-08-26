use crate::cli::language_analyzer::{RustAnalyzer, LanguageAnalyzer};

fn main() {
    let content = r#"fn first_function() {
    println!("Hello");
}

fn second_function() {
    println!("World");
    if true {
        println!("nested");
    }
}

fn third_function() {
    println!("Third");
}
"#;
    
    let analyzer = RustAnalyzer;
    let functions = analyzer.extract_functions(content);
    
    for func in &functions {
        println!("Function: {}, Start: {}, End: {}", func.name, func.line_start, func.line_end);
    }
}