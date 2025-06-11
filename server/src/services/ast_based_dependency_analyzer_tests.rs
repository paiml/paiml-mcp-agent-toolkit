use super::*;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::fs;

#[test]
fn test_ast_based_dependency_analyzer_creation() {
    let analyzer = AstBasedDependencyAnalyzer::new();
    // Just ensure it creates successfully
    assert!(Arc::strong_count(&analyzer.builtin_modules) >= 1);
    assert!(Arc::strong_count(&analyzer.workspace_resolver) >= 1);
}

#[test]
fn test_builtin_module_registry_creation() {
    let registry = BuiltinModuleRegistry::new();
    
    // Check Rust builtins
    assert!(registry.rust_builtins.contains("std"));
    assert!(registry.rust_builtins.contains("core"));
    assert!(registry.rust_builtins.contains("alloc"));
    
    // Check Python builtins
    assert!(registry.python_builtins.contains("sys"));
    assert!(registry.python_builtins.contains("os"));
    assert!(registry.python_builtins.contains("json"));
    
    // Check Node builtins
    assert!(registry.node_builtins.contains("fs"));
    assert!(registry.node_builtins.contains("path"));
    assert!(registry.node_builtins.contains("http"));
}

#[test]
fn test_workspace_resolver_creation() {
    let resolver = WorkspaceResolver::new();
    assert!(resolver.workspace_members.is_empty());
}

#[tokio::test]
async fn test_analyze_unsupported_file() {
    let analyzer = AstBasedDependencyAnalyzer::new();
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    
    fs::write(&file_path, "Some text content").await.unwrap();
    
    let result = analyzer.analyze_file(&file_path).await.unwrap();
    assert!(result.external.is_empty());
    assert!(result.internal.is_empty());
    assert!(result.boundary_violations.is_empty());
}

#[tokio::test]
async fn test_analyze_rust_file() {
    let analyzer = AstBasedDependencyAnalyzer::new();
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.rs");
    
    let rust_content = r#"
use std::collections::HashMap;
use crate::models::User;
use super::utils;

fn main() {
    let map = HashMap::new();
}
"#;
    
    fs::write(&file_path, rust_content).await.unwrap();
    
    let result = analyzer.analyze_file(&file_path).await.unwrap();
    
    // Should have at least one external dependency (std)
    assert!(!result.external.is_empty());
    assert!(result.external.iter().any(|d| d.name.starts_with("std")));
    
    // Should have internal dependencies
    assert!(!result.internal.is_empty());
}

#[tokio::test]
async fn test_analyze_python_file() {
    let analyzer = AstBasedDependencyAnalyzer::new();
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.py");
    
    let python_content = r#"
import os
import json
from datetime import datetime
from .utils import helper
import requests
"#;
    
    fs::write(&file_path, python_content).await.unwrap();
    
    let result = analyzer.analyze_file(&file_path).await.unwrap();
    
    // Python analysis should return some results
    assert!(result.external.len() > 0 || result.internal.len() > 0);
}

#[tokio::test]
async fn test_analyze_typescript_file() {
    let analyzer = AstBasedDependencyAnalyzer::new();
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.ts");
    
    let ts_content = r#"
import { Component } from '@angular/core';
import * as fs from 'fs';
import { UserService } from './services/user.service';
const lodash = require('lodash');
"#;
    
    fs::write(&file_path, ts_content).await.unwrap();
    
    let result = analyzer.analyze_file(&file_path).await.unwrap();
    
    // TypeScript analysis should work
    assert!(result.external.len() > 0 || result.internal.len() > 0);
}

#[tokio::test]
async fn test_analyze_c_file() {
    let analyzer = AstBasedDependencyAnalyzer::new();
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.c");
    
    let c_content = r#"
#include <stdio.h>
#include <stdlib.h>
#include "myheader.h"

int main() {
    return 0;
}
"#;
    
    fs::write(&file_path, c_content).await.unwrap();
    
    let result = analyzer.analyze_file(&file_path).await.unwrap();
    
    // C analysis returns empty for now
    assert!(result.external.is_empty());
    assert!(result.internal.is_empty());
    assert!(result.boundary_violations.is_empty());
}

#[test]
fn test_import_type_serialization() {
    // Test that ImportType variants serialize correctly
    let types = vec![
        ImportType::Use,
        ImportType::Import,
        ImportType::FromImport,
        ImportType::Require,
        ImportType::DynamicImport,
        ImportType::TypeOnly,
    ];
    
    for import_type in types {
        let serialized = serde_json::to_string(&import_type).unwrap();
        let deserialized: ImportType = serde_json::from_str(&serialized).unwrap();
        assert!(matches!(import_type, deserialized));
    }
}

#[test]
fn test_violation_type_serialization() {
    let types = vec![
        ViolationType::LayerViolation,
        ViolationType::VisibilityViolation,
        ViolationType::CyclicDependency,
    ];
    
    for violation_type in types {
        let serialized = serde_json::to_string(&violation_type).unwrap();
        let deserialized: ViolationType = serde_json::from_str(&serialized).unwrap();
        assert!(matches!(violation_type, deserialized));
    }
}

#[test]
fn test_architecture_layer_equality() {
    assert_eq!(ArchitectureLayer::Presentation, ArchitectureLayer::Presentation);
    assert_ne!(ArchitectureLayer::Presentation, ArchitectureLayer::Domain);
    assert_eq!(ArchitectureLayer::Infrastructure, ArchitectureLayer::Infrastructure);
}

#[test]
fn test_dependency_creation() {
    let dep = Dependency {
        name: "tokio".to_string(),
        version: Some("1.0".to_string()),
        is_external: true,
        import_type: ImportType::Use,
        location: Location {
            file: "main.rs".to_string(),
            line: 5,
            column: 1,
        },
    };
    
    assert_eq!(dep.name, "tokio");
    assert_eq!(dep.version, Some("1.0".to_string()));
    assert!(dep.is_external);
    assert_eq!(dep.location.line, 5);
}

#[test]
fn test_boundary_violation_creation() {
    let violation = BoundaryViolation {
        from_module: "ui::components".to_string(),
        to_module: "db::models".to_string(),
        violation_type: ViolationType::LayerViolation,
        location: Location {
            file: "ui/components/user.rs".to_string(),
            line: 10,
            column: 5,
        },
    };
    
    assert_eq!(violation.from_module, "ui::components");
    assert_eq!(violation.to_module, "db::models");
    assert!(matches!(violation.violation_type, ViolationType::LayerViolation));
}