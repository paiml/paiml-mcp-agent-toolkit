#[cfg(test)]
mod deep_context_test_suite {
    use super::super::*;
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
    /// new
    ///
    /// # Panics
    ///
    /// This function will panic if:
    /// - Panics if the value is `None` or Err
    /// - May panic on out-of-bounds array/slice access
    /// - Panics if assertions fail
#[cfg(test)]
mod deep_context_test_suite {
    use super::super::*;
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
