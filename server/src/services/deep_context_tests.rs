#[cfg(test)]
mod deep_context_test_suite {
    use crate::services::complexity::ComplexitySummary;
    use crate::services::context::AstItem;
    use crate::services::deep_context::{
        AnalysisResults, AnnotatedNode, DefectFactor, DefectSummary, FileLocation, Impact,
        NodeAnnotations, NodeType, Priority, RefactoringEstimate,
    };
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// create_test_project
    ///
    /// # Panics
    ///
    /// May panic if internal assertions fail
    fn create_test_project() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        // Create a basic Rust project structure
        fs::create_dir_all(project_path.join("src")).unwrap();

        // Create main.rs
        fs::write(
            project_path.join("src/main.rs"),
            r#"
/// main
///
/// # Panics
///
/// May panic if internal assertions fail
fn main() {
    println!("Hello, world!");
    process_data(vec![1, 2, 3]);
}

fn process_data(data: Vec<i32>) -> i32 {
    data.iter().sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    /// test_process_data
    ///
    /// # Panics
    ///
    /// May panic if internal assertions fail
    async fn test_process_data() {
        assert_eq!(process_data(vec![1, 2, 3]), 6);
    }
}
"#,
        )
        .unwrap();

        // Create lib.rs with some complexity
        fs::write(
            project_path.join("src/lib.rs"),
            r#"
pub struct Calculator {
    value: i32,
}

impl Calculator {
    pub fn new(value: i32) -> Self {
        Calculator { value }
    }

    pub fn add(&mut self, other: i32) -> i32 {
        self.value += other;
        self.value
    }
}
"#,
        )
        .unwrap();

        (temp_dir, project_path)
    }

    #[tokio::test]
    /// Test language-specific analysis routing for analyze_file_by_language
    /// Toyota Way TDD: Test the extracted function to ensure correct routing
    async fn test_analyze_file_by_language_routing() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Test Rust file routing
        let rust_file = temp_path.join("test.rs");
        fs::write(&rust_file, "fn main() {}").unwrap();
        let result = crate::services::deep_context::analyze_file_by_language(&rust_file, "rust").await;
        assert!(result.is_ok(), "Rust file analysis should succeed");

        // Test JavaScript file routing
        let js_file = temp_path.join("test.js");
        fs::write(&js_file, "function main() {}").unwrap();
        let result = crate::services::deep_context::analyze_file_by_language(&js_file, "javascript").await;
        assert!(result.is_ok(), "JavaScript file analysis should succeed");

        // Test TypeScript file routing
        let ts_file = temp_path.join("test.ts");
        fs::write(&ts_file, "function main(): void {}").unwrap();
        let result = crate::services::deep_context::analyze_file_by_language(&ts_file, "typescript").await;
        assert!(result.is_ok(), "TypeScript file analysis should succeed");

        // Test Python file routing
        let py_file = temp_path.join("test.py");
        fs::write(&py_file, "def main(): pass").unwrap();
        let result = crate::services::deep_context::analyze_file_by_language(&py_file, "python").await;
        assert!(result.is_ok(), "Python file analysis should succeed");

        // Test unknown language returns empty vector
        let unknown_file = temp_path.join("test.unknown");
        fs::write(&unknown_file, "unknown content").unwrap();
        let result = crate::services::deep_context::analyze_file_by_language(&unknown_file, "unknown").await;
        assert!(result.is_ok(), "Unknown language should return Ok with empty vector");
        assert_eq!(result.unwrap().len(), 0, "Unknown language should return empty AST items");
    }

    #[tokio::test]
    /// Test individual language analysis functions
    /// Toyota Way Single Responsibility: Test each extracted function independently
    async fn test_rust_language_analysis() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("simple.rs");
        fs::write(&rust_file, "pub fn hello() -> String { String::from(\"Hello\") }").unwrap();
        
        let result = crate::services::deep_context::analyze_rust_language(&rust_file).await;
        assert!(result.is_ok(), "Rust language analysis should succeed");
        let items = result.unwrap();
        assert!(!items.is_empty(), "Rust file should produce AST items");
    }

    #[tokio::test]
    /// Test TypeScript/JavaScript language analysis
    /// Toyota Way Single Responsibility: Verify TypeScript-specific extraction
    async fn test_typescript_language_analysis() {
        let temp_dir = TempDir::new().unwrap();
        let ts_file = temp_dir.path().join("simple.ts");
        fs::write(&ts_file, "export function hello(): string { return 'Hello'; }").unwrap();
        
        let result = crate::services::deep_context::analyze_typescript_language(&ts_file).await;
        assert!(result.is_ok(), "TypeScript language analysis should succeed");
    }

    #[tokio::test]
    /// Test Python language analysis
    /// Toyota Way Single Responsibility: Verify Python-specific extraction
    async fn test_python_language_analysis() {
        let temp_dir = TempDir::new().unwrap();
        let py_file = temp_dir.path().join("simple.py");
        fs::write(&py_file, "def hello():\n    return 'Hello'").unwrap();
        
        let result = crate::services::deep_context::analyze_python_language(&py_file).await;
        assert!(result.is_ok(), "Python language analysis should succeed");
    }

    #[tokio::test]
    /// Test C/C++ language analysis
    /// Toyota Way Single Responsibility: Verify C/C++-specific extraction
    async fn test_c_language_analysis() {
        let temp_dir = TempDir::new().unwrap();
        let c_file = temp_dir.path().join("simple.c");
        fs::write(&c_file, "int hello() { return 42; }").unwrap();
        
        let result = crate::services::deep_context::analyze_c_language(&c_file).await;
        assert!(result.is_ok(), "C language analysis should succeed");
    }

    #[tokio::test]
    /// Test Kotlin language analysis with debug logging
    /// Toyota Way Single Responsibility: Verify Kotlin-specific extraction with logging
    async fn test_kotlin_language_analysis() {
        let temp_dir = TempDir::new().unwrap();
        let kt_file = temp_dir.path().join("simple.kt");
        fs::write(&kt_file, "fun hello(): String = \"Hello\"").unwrap();
        
        let result = crate::services::deep_context::analyze_kotlin_language(&kt_file).await;
        assert!(result.is_ok(), "Kotlin language analysis should succeed");
    }

    #[tokio::test]
    /// Test the refactored analyze_single_file function
    /// Toyota Way TDD: Verify the main function still works after refactoring
    async fn test_analyze_single_file_refactored() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("test.rs");
        fs::write(&rust_file, "fn main() { println!(\"Hello, world!\"); }").unwrap();
        
        let result = crate::services::deep_context::analyze_single_file(&rust_file).await;
        assert!(result.is_ok(), "analyze_single_file should succeed after refactoring");
        
        let file_context = result.unwrap();
        assert_eq!(file_context.language, "rust", "Should correctly detect Rust language");
        assert!(!file_context.path.is_empty(), "Should have valid file path");
    }

    #[tokio::test]
    /// Test complexity reduction verification
    /// Toyota Way Quality Assurance: Ensure refactoring maintains functionality
    async fn test_complexity_reduction_maintained_functionality() {
        let (temp_dir, project_path) = create_test_project();
        
        // Test various file types to ensure all language routing still works
        let test_files = vec![
            ("main.rs", "fn main() {}"),
            ("script.js", "function main() {}"),
            ("app.ts", "function main(): void {}"),
            ("program.py", "def main(): pass"),
            ("code.c", "int main() { return 0; }"),
            ("app.kt", "fun main() {}"),
        ];

        for (filename, content) in test_files {
            let file_path = project_path.join(filename);
            fs::write(&file_path, content).unwrap();
            
            let result = crate::services::deep_context::analyze_single_file(&file_path).await;
            assert!(result.is_ok(), "File {} should be analyzed successfully after refactoring", filename);
            
            let file_context = result.unwrap();
            assert!(!file_context.path.is_empty(), "File {} should have valid path", filename);
            assert!(!file_context.language.is_empty(), "File {} should have detected language", filename);
        }
    }

    #[tokio::test]
    /// Test error handling in language analysis functions
    /// Toyota Way Quality Assurance: Verify proper error handling
    async fn test_language_analysis_error_handling() {
        let temp_dir = TempDir::new().unwrap();
        
        // Test with non-existent file
        let non_existent = temp_dir.path().join("does_not_exist.rs");
        let result = crate::services::deep_context::analyze_rust_language(&non_existent).await;
        // Should handle file not found gracefully
        assert!(result.is_err() || result.unwrap().is_empty(), "Should handle non-existent files gracefully");
    }
}