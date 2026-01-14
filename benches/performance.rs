use criterion::{criterion_group, criterion_main, Criterion};
use pmat::services::{ast_rust::analyze_rust_file, context::analyze_project};
use std::hint::black_box;
use std::path::Path;

fn bench_core_operations(c: &mut Criterion) {
    // Create runtime once and reuse
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("analyze_rust_file", |b| {
        b.iter(|| {
            rt.block_on(async { analyze_rust_file(black_box(Path::new("src/lib.rs"))).await })
        });
    });

    c.bench_function("analyze_project_small", |b| {
        b.iter(|| {
            rt.block_on(async {
                analyze_project(black_box(Path::new("src/models")), black_box("rust")).await
            })
        });
    });
}

criterion_group!(benches, bench_core_operations);
criterion_main!(benches);
