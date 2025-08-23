//! Fuzz testing for AST parsers
//! 
//! This fuzz test ensures that our AST parsers handle malformed and
//! unexpected input gracefully without panicking.

#![no_main]

use libfuzzer_sys::fuzz_target;
use std::path::Path;
use std::fs;
use tempfile::TempDir;

// Import the AST parsers
#[cfg(feature = "python-ast")]
use pmat::services::ast_python;
use pmat::services::ast_rust;
use pmat::services::ast_typescript;

fuzz_target!(|data: &[u8]| {
    // Try to interpret the data as UTF-8
    if let Ok(content) = std::str::from_utf8(data) {
        // Create a temporary directory and file
        let temp_dir = TempDir::new().unwrap();
        
        // Fuzz Python parser
        #[cfg(feature = "python-ast")]
        {
            let py_file = temp_dir.path().join("fuzz.py");
            fs::write(&py_file, content).unwrap();
            
            // Should not panic on any input
            let _ = tokio::runtime::Runtime::new().unwrap().block_on(
                ast_python::analyze_python_file(&py_file)
            );
        }
        
        // Fuzz Rust parser
        {
            let rs_file = temp_dir.path().join("fuzz.rs");
            fs::write(&rs_file, content).unwrap();
            
            // Should not panic on any input
            let _ = tokio::runtime::Runtime::new().unwrap().block_on(
                ast_rust::analyze_rust_file(&rs_file)
            );
        }
        
        // Fuzz TypeScript/JavaScript parser
        {
            let ts_file = temp_dir.path().join("fuzz.ts");
            fs::write(&ts_file, content).unwrap();
            
            // Should not panic on any input
            let _ = tokio::runtime::Runtime::new().unwrap().block_on(
                ast_typescript::analyze_typescript_file(&ts_file)
            );
            
            let js_file = temp_dir.path().join("fuzz.js");
            fs::write(&js_file, content).unwrap();
            
            // Should not panic on any input
            let _ = tokio::runtime::Runtime::new().unwrap().block_on(
                ast_typescript::analyze_javascript_file(&js_file)
            );
        }
    }
    
    // Also test with raw binary data (non-UTF8)
    {
        let temp_dir = TempDir::new().unwrap();
        
        // Test binary data handling
        let bin_file = temp_dir.path().join("fuzz.bin");
        fs::write(&bin_file, data).unwrap();
        
        // Parsers should handle binary files gracefully
        #[cfg(feature = "python-ast")]
        {
            let _ = tokio::runtime::Runtime::new().unwrap().block_on(
                ast_python::analyze_python_file(&bin_file)
            );
        }
        
        let _ = tokio::runtime::Runtime::new().unwrap().block_on(
            ast_rust::analyze_rust_file(&bin_file)
        );
        
        let _ = tokio::runtime::Runtime::new().unwrap().block_on(
            ast_typescript::analyze_typescript_file(&bin_file)
        );
    }
});