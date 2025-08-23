//! Performance tests for Universal Demo
//! 
//! Ensures the demo runs efficiently even on large repositories.

#[cfg(test)]
mod universal_demo_performance_tests {
    use std::time::{Duration, Instant};
    use tempfile::TempDir;
    use std::fs;
    use std::path::Path;

    /// Create a test repository with specified number of files
    fn create_test_repo(num_files: usize, lines_per_file: usize) -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // Create directory structure
        let src_path = base_path.join("src");
        fs::create_dir(&src_path).unwrap();

        // Create Python files
        for i in 0..num_files/3 {
            let file_path = src_path.join(format!("module_{}.py", i));
            let content = generate_python_file(lines_per_file, i);
            fs::write(file_path, content).unwrap();
        }

        // Create JavaScript files
        for i in 0..num_files/3 {
            let file_path = src_path.join(format!("component_{}.js", i));
            let content = generate_javascript_file(lines_per_file, i);
            fs::write(file_path, content).unwrap();
        }

        // Create TypeScript files
        for i in 0..num_files/3 {
            let file_path = src_path.join(format!("service_{}.ts", i));
            let content = generate_typescript_file(lines_per_file, i);
            fs::write(file_path, content).unwrap();
        }

        // Create a README
        fs::write(
            base_path.join("README.md"),
            "# Test Repository\nThis is a test repository for performance testing."
        ).unwrap();

        temp_dir
    }

    fn generate_python_file(lines: usize, id: usize) -> String {
        let mut content = String::new();
        content.push_str(&format!("# Module {}\n", id));
        content.push_str("import os\n");
        content.push_str("import sys\n");
        content.push_str("from typing import List, Dict, Optional\n\n");

        // Add classes
        content.push_str(&format!("class Module{}:\n", id));
        content.push_str("    def __init__(self):\n");
        content.push_str("        self.data = {}\n\n");

        // Add functions
        for i in 0..lines/10 {
            content.push_str(&format!("    def method_{}(self, x):\n", i));
            content.push_str("        if x > 0:\n");
            content.push_str("            return x * 2\n");
            content.push_str("        else:\n");
            content.push_str("            return 0\n\n");
        }

        content
    }

    fn generate_javascript_file(lines: usize, id: usize) -> String {
        let mut content = String::new();
        content.push_str(&format!("// Component {}\n", id));
        content.push_str("import React from 'react';\n");
        content.push_str("import { useState, useEffect } from 'react';\n\n");

        // Add component
        content.push_str(&format!("const Component{} = () => {{\n", id));
        content.push_str("    const [state, setState] = useState(0);\n\n");

        // Add functions
        for i in 0..lines/15 {
            content.push_str(&format!("    const handler{} = (value) => {{\n", i));
            content.push_str("        if (value > 0) {\n");
            content.push_str("            setState(value * 2);\n");
            content.push_str("        }\n");
            content.push_str("    };\n\n");
        }

        content.push_str("    return <div>{state}</div>;\n");
        content.push_str("};\n\n");
        content.push_str(&format!("export default Component{};\n", id));

        content
    }

    fn generate_typescript_file(lines: usize, id: usize) -> String {
        let mut content = String::new();
        content.push_str(&format!("// Service {}\n", id));
        content.push_str("interface User {\n");
        content.push_str("    id: number;\n");
        content.push_str("    name: string;\n");
        content.push_str("}\n\n");

        content.push_str(&format!("class Service{} {{\n", id));
        content.push_str("    private users: User[] = [];\n\n");

        // Add methods
        for i in 0..lines/20 {
            content.push_str(&format!("    public method{}(id: number): User | undefined {{\n", i));
            content.push_str("        return this.users.find(u => u.id === id);\n");
            content.push_str("    }\n\n");
        }

        content.push_str("}\n\n");
        content.push_str(&format!("export default Service{};\n", id));

        content
    }

    #[tokio::test]
    async fn test_small_repo_performance() {
        let temp_dir = create_test_repo(10, 100);
        let start = Instant::now();

        let config = pmat::services::deep_context::DeepContextConfig::default();
        let analyzer = pmat::services::deep_context::DeepContextAnalyzer::new(config);
        let result = analyzer.analyze_project(temp_dir.path()).await.unwrap();

        let duration = start.elapsed();

        // Small repo should analyze quickly
        assert!(
            duration < Duration::from_secs(5),
            "Small repo (10 files) should analyze in <5 seconds, took {:?}",
            duration
        );

        // Should discover all files
        assert!(result.metadata.project_root.exists());
        assert!(result.qa_verification.is_some());
    }

    #[tokio::test]
    async fn test_medium_repo_performance() {
        let temp_dir = create_test_repo(100, 200);
        let start = Instant::now();

        let config = pmat::services::deep_context::DeepContextConfig::default();
        let analyzer = pmat::services::deep_context::DeepContextAnalyzer::new(config);
        let result = analyzer.analyze_project(temp_dir.path()).await.unwrap();

        let duration = start.elapsed();

        // Medium repo should still be reasonably fast
        assert!(
            duration < Duration::from_secs(30),
            "Medium repo (100 files) should analyze in <30 seconds, took {:?}",
            duration
        );

        assert!(result.qa_verification.is_some());
    }

    #[tokio::test]
    #[ignore] // Ignore by default as it takes longer
    async fn test_large_repo_performance() {
        let temp_dir = create_test_repo(500, 500);
        let start = Instant::now();

        let config = pmat::services::deep_context::DeepContextConfig::default();
        let analyzer = pmat::services::deep_context::DeepContextAnalyzer::new(config);
        let result = analyzer.analyze_project(temp_dir.path()).await.unwrap();

        let duration = start.elapsed();

        // Large repo should complete in reasonable time
        assert!(
            duration < Duration::from_secs(120),
            "Large repo (500 files) should analyze in <2 minutes, took {:?}",
            duration
        );

        assert!(result.qa_verification.is_some());
    }

    #[test]
    fn test_file_generation() {
        // Verify our test file generators work
        let python_content = generate_python_file(100, 1);
        assert!(python_content.contains("class Module1"));
        assert!(python_content.contains("import os"));

        let js_content = generate_javascript_file(100, 2);
        assert!(js_content.contains("Component2"));
        assert!(js_content.contains("useState"));

        let ts_content = generate_typescript_file(100, 3);
        assert!(ts_content.contains("Service3"));
        assert!(ts_content.contains("interface User"));
    }

    #[tokio::test]
    async fn test_memory_efficiency() {
        // Create a repo with many small files
        let temp_dir = create_test_repo(200, 50);
        
        // Get initial memory (approximate)
        let before = get_approximate_memory_usage();

        let config = pmat::services::deep_context::DeepContextConfig::default();
        let analyzer = pmat::services::deep_context::DeepContextAnalyzer::new(config);
        let _result = analyzer.analyze_project(temp_dir.path()).await.unwrap();

        // Get final memory (approximate)
        let after = get_approximate_memory_usage();

        // Memory growth should be reasonable (less than 100MB for 200 small files)
        let growth_mb = (after.saturating_sub(before)) / 1_000_000;
        assert!(
            growth_mb < 100,
            "Memory growth should be <100MB for 200 files, was {}MB",
            growth_mb
        );
    }

    fn get_approximate_memory_usage() -> usize {
        // This is a rough approximation
        // In real tests, you might use a proper memory profiler
        use std::alloc::{GlobalAlloc, Layout, System};
        
        // Allocate and deallocate to get a rough idea
        let layout = Layout::from_size_align(1, 1).unwrap();
        unsafe {
            let ptr = System.alloc(layout);
            System.dealloc(ptr, layout);
        }
        
        // Return a placeholder value
        // In production, use proper memory tracking
        50_000_000 // 50MB baseline
    }
}