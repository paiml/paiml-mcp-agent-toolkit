# Multi-Language Repository Analysis Requirements

**Version**: 2.9.0  
**Priority**: P0 - Universal Demo Support  
**Date**: 2025-08-22  

## Executive Summary

This document defines the technical requirements for enabling PMAT to analyze GitHub repositories in any of the 30+ supported languages with "just works" functionality. The goal is to transform PMAT from a Rust-centric tool into a universal code analysis platform.

## Language Support Matrix

### Tier 1: Full AST + Complexity + Dependencies (Target: v2.9.0)
These languages require complete function-level analysis with complexity metrics.

#### Rust ✅ COMPLETE
- **Parser**: syn crate (fully implemented)
- **Features**: Complete AST, complexity analysis, dependency graph
- **Status**: Production ready

#### TypeScript/JavaScript 🔄 NEEDS ENHANCEMENT
- **Parser**: swc_ecma_parser (partially implemented)
- **Current Issue**: No function extraction
- **Required**: Function parsing, complexity calculation, import/export analysis
- **Files to Fix**: `server/src/services/ast_typescript.rs`

#### Python 🔄 NEEDS ENHANCEMENT  
- **Parser**: rustpython-parser (partially implemented)
- **Current Issue**: Functions not detected (`"functions": Array []`)
- **Required**: Function/class parsing, complexity calculation, import analysis
- **Files to Fix**: `server/src/services/ast_python.rs`

#### C/C++ 🔄 NEEDS IMPLEMENTATION
- **Parser**: tree-sitter-c, tree-sitter-cpp (basic integration exists)
- **Current Status**: Language detection only
- **Required**: Function parsing, complexity analysis, include dependency tracking
- **Files to Create**: Enhanced AST analysis in `server/src/services/ast_c.rs`, `ast_cpp.rs`

#### Java ❌ NOT IMPLEMENTED
- **Parser**: tree-sitter-java (not integrated)
- **Required**: Class/method parsing, import analysis, package dependency graph
- **Files to Create**: `server/src/services/ast_java.rs`

#### Go ❌ NOT IMPLEMENTED  
- **Parser**: tree-sitter-go (not integrated)
- **Required**: Function/package parsing, import analysis, module dependency graph
- **Files to Create**: `server/src/services/ast_go.rs`

### Tier 2: Basic Analysis + Dependencies (Target: v2.9.1)

#### Kotlin 🔄 EXTEND EXISTING
- **Parser**: tree-sitter-kotlin (basic support exists)
- **Required**: Class/function parsing, import analysis
- **Files to Fix**: `server/src/services/ast_kotlin.rs`

#### C# ❌ NOT IMPLEMENTED
- **Parser**: tree-sitter-c-sharp (not integrated)  
- **Required**: Class/method parsing, using statement analysis
- **Files to Create**: `server/src/services/ast_csharp.rs`

#### PHP ❌ NOT IMPLEMENTED
- **Parser**: tree-sitter-php (not integrated)
- **Required**: Function/class parsing, include/require analysis
- **Files to Create**: `server/src/services/ast_php.rs`

#### Ruby ❌ NOT IMPLEMENTED
- **Parser**: tree-sitter-ruby (not integrated)
- **Required**: Method/class parsing, require/load analysis
- **Files to Create**: `server/src/services/ast_ruby.rs`

### Tier 3: File Analysis + Language Detection (Target: v2.9.2)
Basic file counting, language detection, and simple metrics for remaining 20+ languages.

## Core Technical Requirements

### 1. Universal Function Extraction Interface

All language parsers must implement a common interface:

```rust
pub trait LanguageAnalyzer {
    fn analyze_file(&self, file_path: &Path, source: &str) -> Result<FileAnalysis>;
    fn supported_extensions(&self) -> &[&str];
    fn language_name(&self) -> &str;
}

pub struct FileAnalysis {
    pub functions: Vec<FunctionInfo>,
    pub classes: Vec<ClassInfo>, 
    pub imports: Vec<ImportInfo>,
    pub exports: Vec<ExportInfo>,
    pub complexity_summary: ComplexityMetrics,
}

pub struct FunctionInfo {
    pub name: String,
    pub line_start: u32,
    pub line_end: u32,
    pub cyclomatic_complexity: u32,
    pub cognitive_complexity: u32,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<String>,
    pub is_async: bool,
    pub is_method: bool,
    pub visibility: Visibility,
}
```

### 2. Language-Specific Complexity Calculation

Each language has different complexity considerations:

#### JavaScript/TypeScript
- **Functions**: Function declarations, expressions, arrow functions, methods
- **Complexity Factors**: if/else, loops, switch, try/catch, ternary operators, logical operators
- **Special Cases**: Async/await, Promise chains, callback nesting

#### Python  
- **Functions**: def, async def, class methods, lambda functions
- **Complexity Factors**: if/elif/else, for/while, try/except, with statements, list comprehensions
- **Special Cases**: Decorators, context managers, generator functions

#### Java
- **Functions**: Methods in classes, static methods, constructors
- **Complexity Factors**: if/else, loops, switch, try/catch, lambda expressions
- **Special Cases**: Stream operations, optional chaining

#### C/C++
- **Functions**: Function definitions, class methods, template functions
- **Complexity Factors**: if/else, loops, switch, goto statements, macro conditions
- **Special Cases**: Template specializations, operator overloading

### 3. Dependency Analysis Requirements

#### Import/Export Pattern Detection

**JavaScript/TypeScript**:
```typescript
// Patterns to detect:
import { Component } from 'react';           // Named import
import React from 'react';                  // Default import  
const fs = require('fs');                   // CommonJS require
export { MyComponent };                     // Named export
export default MyComponent;                 // Default export
```

**Python**:
```python
# Patterns to detect:
import os                                   # Module import
from pathlib import Path                    # From import
import numpy as np                          # Aliased import
from . import sibling_module               # Relative import
```

**Java**:
```java
// Patterns to detect:
import java.util.List;                      // Single class import
import java.util.*;                         // Wildcard import
import static java.lang.Math.PI;            // Static import
package com.example.project;                // Package declaration
```

**C/C++**:
```cpp
// Patterns to detect:
#include <iostream>                         // System header
#include "myheader.h"                       // Local header
#include "../utils/helper.h"                // Relative path
```

#### Dependency Graph Construction

```rust
pub struct UniversalDependencyBuilder {
    analyzers: HashMap<Language, Box<dyn LanguageAnalyzer>>,
}

impl UniversalDependencyBuilder {
    pub fn build_project_graph(&self, project_root: &Path) -> Result<DependencyGraph> {
        let mut graph = DependencyGraph::new();
        
        // 1. Discover all source files
        let source_files = self.discover_source_files(project_root)?;
        
        // 2. Parse each file with appropriate analyzer
        for file in source_files {
            let language = self.detect_language(&file)?;
            let analyzer = self.analyzers.get(&language)
                .ok_or_else(|| anyhow!("No analyzer for language: {:?}", language))?;
            
            let analysis = analyzer.analyze_file(&file, &read_file(&file)?)?;
            
            // 3. Add file node to graph
            graph.add_file_node(&file, analysis.complexity_summary);
            
            // 4. Add dependency edges
            for import in analysis.imports {
                if let Some(target_file) = self.resolve_import(&import, &file, project_root)? {
                    graph.add_dependency_edge(&file, &target_file, import.import_type);
                }
            }
        }
        
        Ok(graph)
    }
}
```

### 4. Quality Gate Adaptations

Different languages have different complexity norms:

```rust
pub struct LanguageQualityThresholds {
    pub max_function_complexity: u32,
    pub max_file_complexity: u32,
    pub typical_function_size: u32,
}

impl LanguageQualityThresholds {
    pub fn for_language(language: Language) -> Self {
        match language {
            Language::Rust => Self {
                max_function_complexity: 15,  // Rust encourages small functions
                max_file_complexity: 100,
                typical_function_size: 20,
            },
            Language::Python => Self {
                max_function_complexity: 20,  // Python allows slightly more complexity
                max_file_complexity: 150, 
                typical_function_size: 30,
            },
            Language::Javascript => Self {
                max_function_complexity: 25,  // JS often has higher complexity
                max_file_complexity: 200,
                typical_function_size: 40,
            },
            Language::Java => Self {
                max_function_complexity: 20,  // Java methods can be complex
                max_file_complexity: 300,     // Large classes common
                typical_function_size: 25,
            },
            // ... other languages
        }
    }
}
```

## Implementation Priority

### Phase 1: Core Language Enhancement (v2.9.0)

**Week 1: Fix JavaScript/TypeScript Analysis**
- Priority: P0 (most common demo scenario)
- Files: `server/src/services/ast_typescript.rs`
- Target: Function extraction working for React, Vue, Angular projects

**Week 2: Fix Python Analysis**  
- Priority: P0 (second most common)
- Files: `server/src/services/ast_python.rs`
- Target: Function extraction working for Django, Flask, FastAPI projects

**Week 3: C/C++ Implementation**
- Priority: P1 (system programming showcase)
- Files: `server/src/services/ast_c.rs`, `ast_cpp.rs`
- Target: Function extraction working for Linux, embedded projects

### Phase 2: JVM Ecosystem (v2.9.1)

**Java Implementation**:
- Most requested enterprise language
- Spring Boot, Android projects
- Maven/Gradle dependency analysis

**Kotlin Enhancement**:
- Android development focus
- Interop with Java analysis
- Build system integration

### Phase 3: Ecosystem Expansion (v2.9.2)

**Additional Languages**:
- Go: Kubernetes, Docker, cloud projects
- C#: .NET Core, ASP.NET projects  
- PHP: WordPress, Laravel, Symfony projects
- Ruby: Rails, Jekyll projects

## Demo Validation Requirements

### Universal Repository Test Suite

```bash
#!/bin/bash
# test-universal-demo.sh

# JavaScript/TypeScript
test_repo "https://github.com/microsoft/vscode" "TypeScript"
test_repo "https://github.com/facebook/react" "JavaScript" 
test_repo "https://github.com/angular/angular" "TypeScript"
test_repo "https://github.com/vuejs/core" "TypeScript"

# Python
test_repo "https://github.com/django/django" "Python"
test_repo "https://github.com/pallets/flask" "Python"
test_repo "https://github.com/tiangolo/fastapi" "Python"
test_repo "https://github.com/python/cpython" "Python"

# Java  
test_repo "https://github.com/spring-projects/spring-boot" "Java"
test_repo "https://github.com/elastic/elasticsearch" "Java"
test_repo "https://github.com/apache/kafka" "Java"

# C/C++
test_repo "https://github.com/torvalds/linux" "C"
test_repo "https://github.com/microsoft/calculator" "C++"
test_repo "https://github.com/google/leveldb" "C++"

# Go
test_repo "https://github.com/kubernetes/kubernetes" "Go"
test_repo "https://github.com/docker/cli" "Go"
test_repo "https://github.com/prometheus/prometheus" "Go"

function test_repo() {
    local url=$1
    local language=$2
    
    echo "Testing $language repository: $url"
    
    # Run demo and capture results
    result=$(pmat demo --repo "$url" --cli --format json 2>&1)
    
    # Validate results
    if echo "$result" | jq -e '.result.analyses.complexity_report.summary.total_functions > 0' > /dev/null; then
        echo "✅ $language: Functions detected"
    else
        echo "❌ $language: No functions detected"
        return 1
    fi
    
    if echo "$result" | jq -e '.result.analyses.dependency_graph.nodes | length > 0' > /dev/null; then
        echo "✅ $language: Dependencies analyzed"  
    else
        echo "❌ $language: No dependencies found"
        return 1
    fi
    
    echo "✅ $language repository analysis complete"
}
```

### Success Criteria

For each Tier 1 language, the demo must:

1. **Clone Successfully**: No "path does not exist" errors
2. **Detect Functions**: `total_functions > 0` for typical projects
3. **Calculate Complexity**: Non-zero cyclomatic complexity values
4. **Map Dependencies**: Non-empty dependency graph
5. **Pass Quality Gates**: Appropriate thresholds for language
6. **Complete Quickly**: < 60 seconds end-to-end

### Error Handling Requirements

```rust
pub enum AnalysisError {
    // Network/IO errors
    RepositoryCloneFailure { url: String, reason: String },
    FileAccessError { path: PathBuf, io_error: std::io::Error },
    
    // Language parsing errors  
    UnsupportedLanguage { language: String },
    ParseError { file: PathBuf, language: Language, error: String },
    
    // Analysis errors
    ComplexityCalculationFailed { function_name: String, error: String },
    DependencyResolutionFailed { import_path: String, from_file: PathBuf },
    
    // Resource limits
    RepositoryTooLarge { size_mb: u64, limit_mb: u64 },
    AnalysisTimeout { elapsed_seconds: u64, limit_seconds: u64 },
}

impl AnalysisError {
    pub fn user_friendly_message(&self) -> String {
        match self {
            Self::RepositoryCloneFailure { url, reason } => {
                format!("Unable to clone repository {}: {}. Please check the URL and your internet connection.", url, reason)
            },
            Self::UnsupportedLanguage { language } => {
                format!("Language '{}' is not yet supported for deep analysis. Basic file analysis will be provided.", language)
            },
            Self::RepositoryTooLarge { size_mb, limit_mb } => {
                format!("Repository is {} MB, which exceeds the {} MB limit for demo analysis. Please try with a smaller repository.", size_mb, limit_mb)
            },
            // ... other user-friendly messages
        }
    }
}
```

## Performance Requirements

### Analysis Speed Targets

| Repository Size | Analysis Time | Memory Usage |
|----------------|---------------|--------------|
| Small (< 1K LOC) | < 5 seconds | < 50 MB |
| Medium (1K-10K LOC) | < 15 seconds | < 200 MB |
| Large (10K-100K LOC) | < 45 seconds | < 1 GB |
| Enterprise (> 100K LOC) | < 60 seconds | < 2 GB |

### Optimization Strategies

1. **Parallel Processing**: Parse files concurrently where possible
2. **Intelligent Sampling**: For huge repositories, analyze representative subset
3. **Progressive Results**: Show partial results as analysis progresses  
4. **Caching**: Cache parsed results for repeated analysis
5. **Language Prioritization**: Analyze primary language files first

## Conclusion

This specification defines the technical foundation for universal multi-language repository analysis. By implementing these requirements, PMAT will transform from a Rust-focused tool into a universal code analysis platform that works seamlessly with any GitHub repository.

The phased approach ensures steady progress while maintaining Toyota Way quality standards. The comprehensive test suite ensures reliability across diverse language ecosystems, creating confidence for users trying PMAT on their own projects.

Success in this initiative directly supports PMAT's mission of democratizing code quality analysis across all programming languages and development teams.