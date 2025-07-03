# PMAT API Guide

This guide covers using pmat as a Rust library in your own projects.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
pmat = "0.26.3"

# Optional: for async runtime
tokio = { version = "1", features = ["full"] }
```

## Core Services

### CodeAnalysisService

The main service for code analysis operations.

```rust
use pmat::{
    services::code_analysis::CodeAnalysisService,
    types::{ProjectPath, AnalysisFormat},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create service instance
    let service = CodeAnalysisService::new();
    
    // Set project path
    let path = ProjectPath::new("./my-project");
    
    // Generate context
    let context = service.generate_context(path.clone(), None).await?;
    println!("Context: {}", context);
    
    Ok(())
}
```

### Complexity Analysis

```rust
use pmat::{
    services::code_analysis::CodeAnalysisService,
    types::{ProjectPath, ComplexityOptions},
};

async fn analyze_complexity(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let service = CodeAnalysisService::new();
    let project_path = ProjectPath::new(path);
    
    // Basic complexity analysis
    let results = service.analyze_complexity(
        project_path.clone(),
        Some(10) // top 10 files
    ).await?;
    
    // With options
    let options = ComplexityOptions {
        max_cyclomatic: Some(20),
        max_cognitive: Some(15),
        include_tests: false,
        output_format: AnalysisFormat::Json,
    };
    
    let detailed = service.analyze_complexity_with_options(
        project_path,
        options
    ).await?;
    
    Ok(())
}
```

### SATD Detection

```rust
use pmat::{
    services::code_analysis::CodeAnalysisService,
    types::ProjectPath,
};

async fn find_technical_debt(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let service = CodeAnalysisService::new();
    let project_path = ProjectPath::new(path);
    
    // Find self-admitted technical debt
    let satd_results = service.analyze_satd(
        project_path,
        Some(10) // top 10 files
    ).await?;
    
    for item in satd_results.items {
        println!(
            "Found {} in {}:{}",
            item.debt_type,
            item.file_path,
            item.line_number
        );
    }
    
    Ok(())
}
```

### Deep Context Analysis

```rust
use pmat::{
    services::code_analysis::CodeAnalysisService,
    analysis::deep_context::{DeepContextOptions, AnalysisScope},
    types::ProjectPath,
};

async fn deep_analysis(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let service = CodeAnalysisService::new();
    let project_path = ProjectPath::new(path);
    
    let options = DeepContextOptions {
        include_ast: true,
        include_complexity: true,
        include_churn: true,
        include_coupling: true,
        include_tdg: true,
        output_format: AnalysisFormat::Json,
        scope: AnalysisScope::Full,
    };
    
    let results = service.analyze_deep_context_with_options(
        project_path,
        options
    ).await?;
    
    println!("Analysis complete: {} issues found", results.total_issues);
    
    Ok(())
}
```

## Quality Gates

### Running Quality Checks

```rust
use pmat::{
    services::quality::QualityGateService,
    types::{ProjectPath, QualityGateConfig},
};

async fn check_quality(path: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let service = QualityGateService::new();
    let project_path = ProjectPath::new(path);
    
    let config = QualityGateConfig {
        max_complexity: 20,
        max_coupling: 10,
        min_coverage: 90.0,
        allow_satd: false,
        strict_mode: true,
    };
    
    let results = service.run_quality_gate(project_path, config).await?;
    
    if results.passed {
        println!("Quality gate passed!");
    } else {
        println!("Quality gate failed:");
        for violation in results.violations {
            println!("  - {}: {}", violation.rule, violation.message);
        }
    }
    
    Ok(results.passed)
}
```

## Refactoring Engine

### Automated Refactoring

```rust
use pmat::{
    services::refactor::RefactorService,
    types::{ProjectPath, RefactorOptions},
};

async fn auto_refactor(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let service = RefactorService::new();
    let project_path = ProjectPath::new(path);
    
    let options = RefactorOptions {
        dry_run: false,
        max_iterations: 10,
        target_complexity: 5,
        preserve_behavior: true,
        create_backups: true,
    };
    
    let results = service.refactor_auto(project_path, options).await?;
    
    println!("Refactoring complete:");
    println!("  Files changed: {}", results.files_changed);
    println!("  Complexity reduced by: {}%", results.complexity_reduction);
    
    Ok(())
}
```

### Single File Refactoring

```rust
use pmat::{
    services::refactor::RefactorService,
    types::FilePath,
};

async fn refactor_file(file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let service = RefactorService::new();
    let file_path = FilePath::new(file);
    
    // Target specific file
    let results = service.refactor_single_file(
        file_path,
        Default::default() // Use default options
    ).await?;
    
    Ok(())
}
```

## Template Generation

### Scaffolding Projects

```rust
use pmat::{
    services::template::TemplateService,
    types::{Language, TemplateType},
};

async fn scaffold_project(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let service = TemplateService::new();
    
    // Create Rust project
    service.scaffold_project(
        name,
        Language::Rust,
        vec![
            TemplateType::Readme,
            TemplateType::Gitignore,
            TemplateType::Makefile,
            TemplateType::License,
        ]
    ).await?;
    
    Ok(())
}
```

## AST Analysis

### Language-Specific Analysis

```rust
use pmat::{
    analysis::ast::{AstAnalyzer, AstNode},
    types::{FilePath, Language},
};

fn analyze_ast(file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let analyzer = AstAnalyzer::new(Language::Rust);
    let ast = analyzer.parse_file(FilePath::new(file))?;
    
    // Walk AST
    ast.walk(|node: &AstNode| {
        match node.kind() {
            "function_item" => {
                println!("Found function: {}", node.text());
            }
            "struct_item" => {
                println!("Found struct: {}", node.text());
            }
            _ => {}
        }
    });
    
    Ok(())
}
```

## Error Handling

### Using Custom Error Types

```rust
use pmat::{
    error::{AnalysisError, ErrorKind},
    services::code_analysis::CodeAnalysisService,
    types::ProjectPath,
};

async fn handle_errors() {
    let service = CodeAnalysisService::new();
    let path = ProjectPath::new("./invalid-path");
    
    match service.analyze_complexity(path, None).await {
        Ok(results) => println!("Success: {:?}", results),
        Err(e) => match e.kind() {
            ErrorKind::PathNotFound => {
                eprintln!("Project path not found");
            }
            ErrorKind::LanguageNotSupported(lang) => {
                eprintln!("Language {} not supported", lang);
            }
            ErrorKind::ParseError(msg) => {
                eprintln!("Parse error: {}", msg);
            }
            _ => eprintln!("Analysis failed: {}", e),
        }
    }
}
```

## Configuration

### Using Configuration Files

```rust
use pmat::{
    config::{Config, load_config},
    services::code_analysis::CodeAnalysisService,
};

async fn with_config() -> Result<(), Box<dyn std::error::Error>> {
    // Load from file
    let config = load_config("pmat.toml")?;
    
    // Or create programmatically
    let config = Config {
        analysis: AnalysisConfig {
            ignore_patterns: vec!["target/".to_string()],
            max_file_size: 1_000_000,
            follow_symlinks: false,
        },
        quality: QualityConfig {
            max_complexity: 20,
            max_coupling: 10,
            allow_satd: false,
        },
        ..Default::default()
    };
    
    let service = CodeAnalysisService::with_config(config);
    
    Ok(())
}
```

## Async Patterns

### Concurrent Analysis

```rust
use pmat::{
    services::code_analysis::CodeAnalysisService,
    types::ProjectPath,
};
use futures::future::join_all;

async fn analyze_multiple_projects(paths: Vec<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let service = CodeAnalysisService::new();
    
    let tasks: Vec<_> = paths.into_iter()
        .map(|path| {
            let svc = service.clone();
            let project_path = ProjectPath::new(path);
            
            tokio::spawn(async move {
                svc.analyze_complexity(project_path, None).await
            })
        })
        .collect();
    
    let results = join_all(tasks).await;
    
    for result in results {
        match result {
            Ok(Ok(analysis)) => println!("Analysis complete"),
            Ok(Err(e)) => eprintln!("Analysis error: {}", e),
            Err(e) => eprintln!("Task error: {}", e),
        }
    }
    
    Ok(())
}
```

## Integration Examples

### CI/CD Integration

```rust
use pmat::{
    services::quality::QualityGateService,
    types::{ProjectPath, QualityGateConfig},
};
use std::process;

#[tokio::main]
async fn main() {
    let service = QualityGateService::new();
    let path = ProjectPath::new(".");
    
    let config = QualityGateConfig::strict();
    
    match service.run_quality_gate(path, config).await {
        Ok(results) => {
            if !results.passed {
                eprintln!("Quality gate failed!");
                for violation in results.violations {
                    eprintln!("  {}: {}", violation.rule, violation.message);
                }
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error running quality gate: {}", e);
            process::exit(2);
        }
    }
}
```

### Custom Analysis Tool

```rust
use pmat::{
    services::code_analysis::CodeAnalysisService,
    types::{ProjectPath, AnalysisFormat},
};
use serde_json::Value;

struct CustomAnalyzer {
    service: CodeAnalysisService,
}

impl CustomAnalyzer {
    pub fn new() -> Self {
        Self {
            service: CodeAnalysisService::new(),
        }
    }
    
    pub async fn analyze_project(&self, path: &str) -> Result<Value, Box<dyn std::error::Error>> {
        let project_path = ProjectPath::new(path);
        
        // Gather multiple analyses
        let context = self.service.generate_context(project_path.clone(), None).await?;
        let complexity = self.service.analyze_complexity(project_path.clone(), Some(20)).await?;
        let satd = self.service.analyze_satd(project_path.clone(), Some(10)).await?;
        
        // Combine results
        let combined = serde_json::json!({
            "project": path,
            "context": context,
            "complexity": complexity,
            "technical_debt": satd,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        
        Ok(combined)
    }
}
```

## Best Practices

1. **Error Handling**: Always handle errors appropriately
2. **Resource Management**: Services are lightweight and can be cloned
3. **Async Operations**: Use tokio runtime for async operations
4. **Configuration**: Use config files for complex setups
5. **Logging**: Enable RUST_LOG for debugging

## Performance Tips

1. **Reuse Services**: Create once, clone for concurrent use
2. **Batch Operations**: Process multiple files together
3. **Use Caching**: Enable caching for repeated analyses
4. **Limit Scope**: Use specific paths instead of entire repos
5. **Stream Results**: Use streaming for large datasets

## Further Resources

- [API Documentation on docs.rs](https://docs.rs/pmat)
- [Examples in Repository](https://github.com/paiml/paiml-mcp-agent-toolkit/tree/master/examples)
- [Integration Tests](https://github.com/paiml/paiml-mcp-agent-toolkit/tree/master/server/tests)