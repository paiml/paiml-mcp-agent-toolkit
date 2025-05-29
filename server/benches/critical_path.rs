use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use paiml_mcp_agent_toolkit::services::renderer::{render_template, TemplateRenderer};
use serde_json::json;

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
        let template = r#"
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
"#;
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

criterion_group!(benches, benchmark_template_generation);
criterion_main!(benches);
