# Comprehensive CLI Test Specification

## Executive Summary

This document specifies the complete testing strategy for the PAIML MCP Agent Toolkit CLI interface, encompassing unit tests, integration tests, property-based tests, and performance benchmarks. The specification leverages Rust's type system and testing infrastructure to ensure deterministic behavior across all CLI operations.

## Test Architecture

### Layered Testing Strategy

```rust
// Test hierarchy follows the principle of test isolation
mod unit {
    // Direct function testing with mocked dependencies
    use crate::cli::args::*;
}

mod integration {
    // Full CLI invocation through Command
    use assert_cmd::Command;
}

mod property {
    // Invariant validation with proptest
    use proptest::prelude::*;
}

mod performance {
    // Sub-millisecond operation guarantees
    use criterion::{black_box, criterion_group};
}
```

### Test Organization

```
server/
├── src/tests/              # Unit tests (81% coverage target)
│   ├── cli_tests.rs        # Core CLI functionality
│   ├── cli_simple_tests.rs # Basic parsing tests
│   └── analyze_cli_tests.rs # Analyze subcommand tests
├── tests/                  # Integration tests
│   ├── cli_integration_full.rs
│   └── e2e/               # Deno-based E2E tests
└── benches/               # Performance benchmarks
    └── critical_path.rs
```

## Command-Specific Test Specifications

### 1. Generate Command Tests

#### Unit Tests

```rust
#[test]
fn test_generate_command_parsing() {
    let args = vec![
        "paiml-mcp-agent-toolkit",
        "generate",
        "makefile",
        "rust/cli",
        "-p", "project_name=test",
        "-p", "has_tests=true",
        "-o", "Makefile",
        "--create-dirs"
    ];
    
    let cli = Cli::try_parse_from(&args).unwrap();
    match cli.command {
        Commands::Generate(cmd) => {
            assert_eq!(cmd.category, "makefile");
            assert_eq!(cmd.variant, "rust/cli");
            assert_eq!(cmd.output, Some(PathBuf::from("Makefile")));
            assert!(cmd.create_dirs);
            
            // Verify parameter parsing
            let params = params_to_json(cmd.params);
            assert_eq!(params["project_name"], "test");
            assert_eq!(params["has_tests"], true);
        }
        _ => panic!("Expected Generate command"),
    }
}
```

#### Integration Tests

```rust
#[test]
fn test_generate_makefile_e2e() {
    let temp_dir = TempDir::new().unwrap();
    
    let mut cmd = Command::cargo_bin("paiml-mcp-agent-toolkit").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&[
            "generate", "makefile", "rust/cli",
            "-p", "project_name=integration_test",
            "-o", "generated/Makefile",
            "--create-dirs"
        ])
        .assert()
        .success();
    
    // Verify file creation
    let makefile_path = temp_dir.path().join("generated/Makefile");
    assert!(makefile_path.exists());
    
    // Verify content
    let content = fs::read_to_string(makefile_path).unwrap();
    assert!(content.contains("integration_test"));
    assert!(content.contains("cargo build --release"));
}
```

#### Edge Cases

```rust
#[test]
fn test_generate_missing_required_params() {
    let mut cmd = Command::cargo_bin("paiml-mcp-agent-toolkit").unwrap();
    cmd.args(&["generate", "makefile", "rust/cli"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Missing required parameter: project_name"));
}

#[test]
fn test_generate_invalid_template_uri() {
    let mut cmd = Command::cargo_bin("paiml-mcp-agent-toolkit").unwrap();
    cmd.args(&[
            "generate", "invalid", "category",
            "-p", "project_name=test"
        ])
        .assert()
        .failure()
        .code(1);
}
```

### 2. Scaffold Command Tests

#### Parallel Execution Testing

```rust
#[tokio::test]
async fn test_scaffold_parallel_generation() {
    let temp_dir = TempDir::new().unwrap();
    let start = Instant::now();
    
    let mut cmd = Command::cargo_bin("paiml-mcp-agent-toolkit").unwrap();
    cmd.current_dir(&temp_dir)
        .args(&[
            "scaffold", "rust",
            "--templates", "makefile,readme,gitignore",
            "-p", "project_name=perf_test",
            "--parallel", "4"
        ])
        .assert()
        .success();
    
    let duration = start.elapsed();
    
    // Verify parallel execution performance
    assert!(duration.as_millis() < 100, "Scaffold took {}ms", duration.as_millis());
    
    // Verify all files created
    assert!(temp_dir.path().join("perf_test/Makefile").exists());
    assert!(temp_dir.path().join("perf_test/README.md").exists());
    assert!(temp_dir.path().join("perf_test/.gitignore").exists());
}
```

### 3. List Command Tests

#### Output Format Testing

```rust
#[test]
fn test_list_json_output_schema() {
    let mut cmd = Command::cargo_bin("paiml-mcp-agent-toolkit").unwrap();
    let output = cmd.args(&["list", "--format", "json"])
        .output()
        .unwrap();
    
    let templates: Vec<TemplateResource> = serde_json::from_slice(&output.stdout).unwrap();
    
    // Verify schema completeness
    for template in &templates {
        assert!(!template.uri.is_empty());
        assert!(!template.name.is_empty());
        assert!(!template.description.is_empty());
        assert!(template.toolchain.as_str().len() > 0);
        
        // Verify parameter specs
        for param in &template.parameters {
            assert!(!param.name.is_empty());
            assert!(matches!(param.param_type, 
                ParameterType::String | ParameterType::Boolean | ParameterType::Number));
        }
    }
    
    // Verify all 9 templates present
    assert_eq!(templates.len(), 9);
}
```

### 4. Context Generation Tests

#### AST Analysis Verification

```rust
#[test]
fn test_context_rust_ast_accuracy() {
    let test_code = r#"
pub struct Calculator {
    value: f64,
}

impl Calculator {
    pub fn new() -> Self {
        Self { value: 0.0 }
    }
    
    pub fn add(&mut self, n: f64) -> f64 {
        self.value += n;
        self.value
    }
}

pub trait Compute {
    fn compute(&self) -> f64;
}
"#;
    
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), test_code).unwrap();
    
    let context = analyze_rust_file(temp_file.path()).await.unwrap();
    
    // Verify AST extraction
    assert_eq!(context.items.len(), 5); // struct, 2 methods, trait, 1 trait method
    
    let struct_item = context.items.iter()
        .find(|i| matches!(i, AstItem::Struct { name, .. } if name == "Calculator"))
        .expect("Calculator struct not found");
    
    if let AstItem::Struct { fields, .. } = struct_item {
        assert_eq!(fields, &1);
    }
}
```

#### Cache Performance Testing

```rust
#[tokio::test]
async fn test_context_cache_performance() {
    let project_dir = setup_test_project();
    let cache_manager = PersistentCacheManager::with_default_dir().unwrap();
    
    // First run - cold cache
    let start = Instant::now();
    let context1 = analyze_project(&project_dir, &cache_manager).await.unwrap();
    let cold_duration = start.elapsed();
    
    // Second run - warm cache
    let start = Instant::now();
    let context2 = analyze_project(&project_dir, &cache_manager).await.unwrap();
    let warm_duration = start.elapsed();
    
    // Cache should provide >10x speedup
    assert!(warm_duration.as_micros() < cold_duration.as_micros() / 10);
    assert!(warm_duration.as_micros() < 10_000); // <10ms for cached
    
    // Verify identical results
    assert_eq!(context1.files.len(), context2.files.len());
}
```

### 5. Analyze Subcommand Tests

#### Churn Analysis Tests

```rust
#[test]
fn test_analyze_churn_git_integration() {
    let repo = setup_git_repo();
    
    // Create commit history
    create_test_commits(&repo, vec![
        ("file1.rs", 10, 5),  // 10 commits, 5 authors
        ("file2.rs", 50, 2),  // 50 commits, 2 authors - high churn
        ("stable.rs", 1, 1),  // 1 commit, 1 author - stable
    ]);
    
    let mut cmd = Command::cargo_bin("paiml-mcp-agent-toolkit").unwrap();
    let output = cmd.current_dir(repo.path())
        .args(&["analyze", "churn", "--format", "json", "--period-days", "30"])
        .output()
        .unwrap();
    
    let analysis: CodeChurnAnalysis = serde_json::from_slice(&output.stdout).unwrap();
    
    // Verify churn scoring algorithm
    let file2_metrics = &analysis.files["file2.rs"];
    assert!(file2_metrics.churn_score > 0.8); // High churn score
    
    let stable_metrics = &analysis.files["stable.rs"];
    assert!(stable_metrics.churn_score < 0.1); // Low churn score
}
```

#### Complexity Analysis Tests

```rust
#[test]
fn test_analyze_complexity_mccabe_calculation() {
    let test_code = r#"
fn complex_function(x: i32, y: i32) -> i32 {
    if x > 0 {                    // +1
        if y > 0 {                // +1
            return x + y;
        } else {
            return x - y;
        }
    }
    
    match x {
        0 => 0,                   // +1
        1..=10 => x * 2,         // +1
        _ => x * 3,              // +1
    }
}
"#;
    
    let complexity = calculate_cyclomatic_complexity(test_code);
    assert_eq!(complexity, 6); // 1 (base) + 5 (decision points)
}

#[test]
fn test_analyze_complexity_sarif_output() {
    let mut cmd = Command::cargo_bin("paiml-mcp-agent-toolkit").unwrap();
    let output = cmd.args(&[
            "analyze", "complexity",
            "--format", "sarif",
            "--max-cyclomatic", "10"
        ])
        .output()
        .unwrap();
    
    let sarif: Value = serde_json::from_slice(&output.stdout).unwrap();
    
    // Verify SARIF 2.1.0 schema compliance
    assert_eq!(sarif["version"], "2.1.0");
    assert_eq!(sarif["$schema"], "https://json.schemastore.org/sarif-2.1.0.json");
    
    let runs = sarif["runs"].as_array().unwrap();
    assert_eq!(runs.len(), 1);
    
    let tool = &runs[0]["tool"]["driver"];
    assert_eq!(tool["name"], "paiml-mcp-agent-toolkit");
}
```

### 6. DAG Generation Tests

#### Graph Correctness Tests

```rust
#[test]
fn test_dag_mermaid_generation() {
    let graph = DependencyGraph {
        nodes: hashmap! {
            "main".to_string() => NodeInfo {
                node_type: NodeType::Function,
                label: "main".to_string(),
                complexity: Some(5),
                file_path: "src/main.rs".to_string(),
                line_number: 10,
                metadata: HashMap::new(),
            },
            "helper".to_string() => NodeInfo {
                node_type: NodeType::Function,
                label: "helper".to_string(),
                complexity: Some(2),
                file_path: "src/lib.rs".to_string(),
                line_number: 20,
                metadata: HashMap::new(),
            },
        },
        edges: vec![
            Edge {
                from: "main".to_string(),
                to: "helper".to_string(),
                edge_type: EdgeType::Calls,
            },
        ],
    };
    
    let generator = MermaidGenerator::new(MermaidOptions {
        direction: "TB".to_string(),
        show_complexity: true,
        ..Default::default()
    });
    
    let mermaid = generator.generate(&graph);
    
    // Verify Mermaid syntax
    assert!(mermaid.contains("graph TB"));
    assert!(mermaid.contains("main[\"Function: main\"]"));
    assert!(mermaid.contains("helper[\"Function: helper\"]"));
    assert!(mermaid.contains("main --> helper"));
    
    // Verify complexity styling
    assert!(mermaid.contains("style main fill:#FFD700")); // Medium complexity
    assert!(mermaid.contains("style helper fill:#90EE90")); // Low complexity
}
```

## Property-Based Testing

### Parameter Parsing Properties

```rust
proptest! {
    #[test]
    fn prop_parameter_parsing_preserves_types(
        key in "[a-z_]+",
        str_val in ".*",
        bool_val in any::<bool>(),
        num_val in any::<f64>().prop_filter("finite", |x| x.is_finite()),
    ) {
        // String parameters
        let (k, v) = parse_key_val(&format!("{}={}", key, str_val)).unwrap();
        assert_eq!(k, key);
        assert_eq!(v.as_str().unwrap(), str_val);
        
        // Boolean parameters
        let (k, v) = parse_key_val(&format!("{}={}", key, bool_val)).unwrap();
        assert_eq!(v.as_bool().unwrap(), bool_val);
        
        // Number parameters
        let (k, v) = parse_key_val(&format!("{}={}", key, num_val)).unwrap();
        assert!((v.as_f64().unwrap() - num_val).abs() < f64::EPSILON);
    }
}
```

### Template URI Validation

```rust
proptest! {
    #[test]
    fn prop_template_uri_parsing(
        category in "(makefile|readme|gitignore)",
        toolchain in "(rust|deno|python-uv)",
        variant in "cli",
    ) {
        let uri = format!("template://{}/{}/{}", category, toolchain, variant);
        
        let (cat, tool, var) = parse_template_uri(&uri).unwrap();
        assert_eq!(cat, category);
        assert_eq!(tool, toolchain);
        assert_eq!(var, variant);
    }
}
```

## Performance Benchmarks

### Critical Path Benchmarks

```rust
fn benchmark_cli_parsing(c: &mut Criterion) {
    c.bench_function("cli_parse_complex_args", |b| {
        let args = vec![
            "paiml-mcp-agent-toolkit",
            "scaffold", "rust",
            "--templates", "makefile,readme,gitignore",
            "-p", "project_name=bench",
            "-p", "has_tests=true",
            "-p", "has_benchmarks=false",
            "-p", "author=Benchmark Test",
            "--parallel", "4",
        ];
        
        b.iter(|| {
            black_box(Cli::try_parse_from(&args).unwrap());
        });
    });
}

fn benchmark_template_generation(c: &mut Criterion) {
    let server = Arc::new(StatelessTemplateServer::new().unwrap());
    let params = json!({
        "project_name": "perf_test",
        "has_tests": true,
    });
    
    c.bench_function("generate_rust_makefile", |b| {
        b.iter(|| {
            let _ = black_box(generate_template(
                &server,
                "template://makefile/rust/cli",
                params.clone(),
            ));
        });
    });
}

criterion_group!(
    benches,
    benchmark_cli_parsing,
    benchmark_template_generation
);
```

### Performance Requirements

| Operation | Target | Measurement Method |
|-----------|--------|-------------------|
| CLI Parsing | <1ms | Criterion benchmark |
| Template Generation | <5ms | End-to-end timing |
| Context Generation (cold) | <500ms | Integration test |
| Context Generation (cached) | <10ms | Cache hit timing |
| Scaffold (3 templates) | <100ms | Parallel execution |
| DAG Generation (1000 nodes) | <50ms | Stress test |

## Error Handling Tests

### Comprehensive Error Scenarios

```rust
#[test]
fn test_error_propagation_and_codes() {
    struct ErrorCase {
        args: Vec<&'static str>,
        expected_code: i32,
        expected_message: &'static str,
    }
    
    let cases = vec![
        ErrorCase {
            args: vec!["generate", "nonexistent", "template"],
            expected_code: 1,
            expected_message: "Template not found",
        },
        ErrorCase {
            args: vec!["scaffold", "rust", "--templates", ""],
            expected_code: 1,
            expected_message: "No templates specified",
        },
        ErrorCase {
            args: vec!["analyze", "churn", "--period-days", "-1"],
            expected_code: 1,
            expected_message: "Invalid period",
        },
    ];
    
    for case in cases {
        let mut cmd = Command::cargo_bin("paiml-mcp-agent-toolkit").unwrap();
        cmd.args(&case.args)
            .assert()
            .failure()
            .code(case.expected_code)
            .stderr(predicate::str::contains(case.expected_message));
    }
}
```

## Test Execution Strategy

### Continuous Integration

```yaml
test:
  stage: test
  script:
    # Unit tests with coverage
    - cargo test --all-features --workspace
    - cargo llvm-cov --text --fail-under 60
    
    # Integration tests
    - cargo test --test '*' --workspace
    
    # Property tests (limited iterations for CI)
    - PROPTEST_CASES=100 cargo test prop_
    
    # E2E tests
    - deno test --allow-all server/tests/e2e/
```

### Local Development

```bash
# Fast unit tests
cargo test --lib

# Watch mode for TDD
cargo watch -x 'test --lib'

# Full test suite
make test

# Specific test focus
cargo test test_generate_ -- --nocapture
```

## Test Coverage Requirements

### Module Coverage Targets

| Module | Target | Rationale |
|--------|--------|-----------|
| cli/args.rs | 95% | Critical parsing logic |
| cli/mod.rs | 85% | Command dispatch |
| handlers/* | 90% | Core functionality |
| services/template_service.rs | 95% | Template generation |
| services/context.rs | 85% | AST analysis |
| services/cache/* | 80% | Cache operations |

### Integration Test Coverage

- Every CLI command must have at least 3 integration tests
- Every error path must have explicit test coverage
- Performance-critical paths require benchmark coverage

## Test Maintenance

### Test Naming Convention

```rust
// Unit tests
#[test]
fn test_{module}_{function}_{scenario}() { }

// Integration tests
#[test]
fn test_{command}_{feature}_e2e() { }

// Property tests
#[test]
fn prop_{property_name}() { }
```

### Test Documentation

Each test module must include:
- Purpose statement
- Test strategy explanation
- Coverage goals
- Performance expectations

## Conclusion

This specification ensures comprehensive validation of the PAIML MCP Agent Toolkit CLI interface through systematic testing at multiple levels. The combination of unit tests, integration tests, property-based tests, and performance benchmarks provides confidence in both correctness and performance characteristics of the CLI implementation.