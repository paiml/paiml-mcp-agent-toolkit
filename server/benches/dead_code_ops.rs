use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pmat::models::unified_ast::{AstDag, AstNode, Language, NodeFlags, NodeKind};
use pmat::services::dead_code_analyzer::DeadCodeAnalyzer;
use std::hint::black_box as hint_black_box;

/// Create a synthetic AST DAG for benchmarking
fn create_synthetic_dag(num_nodes: usize) -> AstDag {
    let mut dag = AstDag::new();

    for i in 0..num_nodes {
        let node = AstNode {
            key: i as u32,
            kind: NodeKind::FunctionDef,
            lang: Language::Rust,
            flags: if i == 0 {
                NodeFlags::EXPORTED // First node is entry point
            } else {
                NodeFlags::empty()
            },
            first_child: if i * 2 + 1 < num_nodes {
                (i * 2 + 1) as u32 // Tree structure
            } else {
                0
            },
            next_sibling: if i * 2 + 2 < num_nodes {
                (i * 2 + 2) as u32
            } else {
                0
            },
            parent: if i > 0 { ((i - 1) / 2) as u32 } else { 0 },
        };

        dag.nodes.push(node);
    }

    dag
}

/// Benchmark dead code analysis (10K nodes)
fn bench_dead_code_10k(c: &mut Criterion) {
    let dag = create_synthetic_dag(10_000);

    c.bench_function("dead_code_analysis_10k", |b| {
        b.iter(|| {
            let mut analyzer = DeadCodeAnalyzer::new(10_000);
            let _report = hint_black_box(analyzer.analyze(black_box(&dag)));
        });
    });
}

/// Benchmark dead code analysis (50K nodes) - realistic large project
fn bench_dead_code_50k(c: &mut Criterion) {
    let dag = create_synthetic_dag(50_000);

    c.bench_function("dead_code_analysis_50k", |b| {
        b.iter(|| {
            let mut analyzer = DeadCodeAnalyzer::new(50_000);
            let _report = hint_black_box(analyzer.analyze(black_box(&dag)));
        });
    });
}

/// Benchmark dead code analysis (100K nodes) - stress test
#[cfg(feature = "simd")]
fn bench_dead_code_100k_simd(c: &mut Criterion) {
    let dag = create_synthetic_dag(100_000);

    c.bench_function("dead_code_analysis_100k_simd", |b| {
        b.iter(|| {
            let mut analyzer = DeadCodeAnalyzer::new(100_000);
            let _report = hint_black_box(analyzer.analyze(black_box(&dag)));
        });
    });
}

#[cfg(feature = "simd")]
criterion_group!(
    benches,
    bench_dead_code_10k,
    bench_dead_code_50k,
    bench_dead_code_100k_simd
);

#[cfg(not(feature = "simd"))]
criterion_group!(benches, bench_dead_code_10k, bench_dead_code_50k);

criterion_main!(benches);
