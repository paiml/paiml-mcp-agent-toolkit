//! Stub module for backward compatibility during AST migration
//! 
//! This module provides minimal stubs to prevent compilation errors.
//! All functionality has been moved to server/src/ast/

use std::path::Path;
use anyhow::Result;

// Stub types for backward compatibility
pub struct AnalyzerPool;

impl Default for AnalyzerPool {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalyzerPool {
    pub fn new() -> Self {
        Self
    }
}

pub struct RustAnalyzer;

impl Default for RustAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl RustAnalyzer {
    pub fn new() -> Self {
        Self
    }
    
    pub fn analyze_file(&self, _path: &Path) -> Result<()> {
        Ok(())
    }
}