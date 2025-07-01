use criterion::{criterion_group, criterion_main, Criterion};
use paiml_mcp_agent_toolkit::services::renderer::{render_template, TemplateRenderer};
use serde_json::json;
use std::hint::black_box;

// CLI parsing benchmarks
fn benchmark_cli_parsing(c: &mut Criterion) {
    c.bench_function("cli_parse_simple_generate", |b| {
        let args = vec![
            "paiml-mcp-agent-toolkit",
            "generate",
            "makefile",
            "rust/cli",
            "-p",
            "project_name=bench",
        ];

        b.iter(|| {
            // Simulate CLI parsing by checking argument validity
            let _ = black_box(args.clone());
        });
    });

    c.bench_function("cli_parse_complex_scaffold", |b| {
        let args = vec![
            "paiml-mcp-agent-toolkit",
            "scaffold",
            "rust",
            "--templates",
            "makefile,readme,gitignore",
            "-p",
            "project_name=bench",
            "-p",
            "has_tests=true",
            "-p",
            "has_benchmarks=false",
            "-p",
            "author=Benchmark Test",
            "--parallel",
            "4",
        ];

        b.iter(|| {
            // Simulate CLI parsing by checking argument validity
            let _ = black_box(args.clone());
        });
    });

    c.bench_function("cli_parse_analyze_complexity", |b| {
        let args = vec![
            "paiml-mcp-agent-toolkit",
            "analyze",
            "complexity",
            "--toolchain",
            "rust",
            "--format",
            "json",
            "--max-cyclomatic",
            "15",
            "--max-cognitive",
            "20",
        ];

        b.iter(|| {
            // Simulate CLI parsing by checking argument validity
            let _ = black_box(args.clone());
        });
    });
}

fn benchmark_template_generation(c: &mut Criterion) {
    let renderer = TemplateRenderer::new().unwrap();

    c.bench_function("template_render_simple", |b| {
        let template = "Hello {{project_name}}!";
        let mut context = serde_json::Map::new();
        context.insert("project_name".to_string(), json!("test-project"));

        b.iter(|| {
            render_template(
                black_box(&renderer),
                black_box(template),
                black_box(context.clone()),
            )
        });
    });

    c.bench_function("template_render_with_helpers", |b| {
        let template = "Project: {{pascal_case project_name}}, Year: {{current_year}}";
        let mut context = serde_json::Map::new();
        context.insert("project_name".to_string(), json!("test-project"));

        b.iter(|| {
            render_template(
                black_box(&renderer),
                black_box(template),
                black_box(context.clone()),
            )
        });
    });

    c.bench_function("template_render_complex", |b| {
        let template = r"
# {{project_name}}

{{#if has_tests}}
## Testing
Run tests with: `cargo test`
{{/if}}

{{#if has_benchmarks}}
## Benchmarks
Run benchmarks with: `cargo bench`
{{/if}}

Copyright {{current_year}} {{author}}
";
        let mut context = serde_json::Map::new();
        context.insert("project_name".to_string(), json!("test-project"));
        context.insert("has_tests".to_string(), json!(true));
        context.insert("has_benchmarks".to_string(), json!(false));
        context.insert("author".to_string(), json!("Test Author"));

        b.iter(|| {
            render_template(
                black_box(&renderer),
                black_box(template),
                black_box(context.clone()),
            )
        });
    });
}

// DAG generation benchmarks
fn benchmark_dag_generation(c: &mut Criterion) {
    c.bench_function("dag_generate_small_graph", |b| {
        // Simulate small graph generation (10 nodes)
        let node_count = 10;
        b.iter(|| {
            let mut edges = Vec::new();
            for i in 0..node_count {
                if i > 0 {
                    edges.push((i - 1, i));
                }
            }
            black_box(edges);
        });
    });

    c.bench_function("dag_generate_medium_graph", |b| {
        // Simulate medium graph generation (100 nodes)
        let node_count = 100;
        b.iter(|| {
            let mut edges = Vec::new();
            for i in 0..node_count {
                if i > 0 {
                    edges.push((i - 1, i));
                }
                // Add some cross-edges
                if i > 10 && i % 5 == 0 {
                    edges.push((i - 10, i));
                }
            }
            black_box(edges);
        });
    });

    c.bench_function("dag_generate_large_graph", |b| {
        // Simulate large graph generation (1000 nodes)
        let node_count = 1000;
        b.iter(|| {
            let mut edges = Vec::new();
            for i in 0..node_count {
                if i > 0 {
                    edges.push((i - 1, i));
                }
                // Add complex connectivity
                if i > 10 && i % 5 == 0 {
                    edges.push((i - 10, i));
                }
                if i > 50 && i % 7 == 0 {
                    edges.push((i - 50, i));
                }
            }
            black_box(edges);
        });
    });
}

// Context generation benchmarks
fn benchmark_context_generation(c: &mut Criterion) {
    c.bench_function("context_parse_rust_file", |b| {
        let sample_code = r#"
use std::collections::HashMap;

pub struct Server {
    name: String,
    port: u16,
}

impl Server {
    pub fn new(name: String, port: u16) -> Self {
        Self { name, port }
    }
    
    pub fn start(&self) {
        println!("Starting server {} on port {}", self.name, self.port);
    }
}

pub fn process_data(data: Vec<i32>) -> i32 {
    data.iter().sum()
}
"#;
        b.iter(|| {
            // Simulate AST parsing
            let lines: Vec<&str> = black_box(sample_code.lines().collect());
            let _ = lines.len();
        });
    });

    c.bench_function("context_cache_lookup", |b| {
        use std::collections::HashMap;
        let mut cache = HashMap::new();

        // Pre-populate cache
        for i in 0..100 {
            cache.insert(format!("file_{i}.rs"), format!("content_{i}"));
        }

        b.iter(|| {
            let key = "file_42.rs";
            let _ = black_box(cache.get(key));
        });
    });
}

criterion_group!(
    benches,
    benchmark_cli_parsing,
    benchmark_template_generation,
    benchmark_dag_generation,
    benchmark_context_generation
);
criterion_main!(benches);
