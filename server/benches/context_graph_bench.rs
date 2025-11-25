use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pmat::services::context::AstItem;
use pmat::services::context_graph::ProjectContextGraph;
use std::hint::black_box as hint_black_box;

fn bench_context_graph_operations(c: &mut Criterion) {
    c.bench_function("context_graph_add_1000_symbols", |b| {
        b.iter(|| {
            let mut graph = ProjectContextGraph::new();
            for i in 0..1000 {
                let item = AstItem::Function {
                    name: format!("func_{}", i),
                    visibility: "pub".to_string(),
                    is_async: false,
                    line: i,
                };
                graph
                    .add_item(format!("func_{}", i), item)
                    .expect("Failed to add item");
            }
            hint_black_box(graph)
        });
    });

    c.bench_function("context_graph_o1_lookup", |b| {
        // Setup: Build graph with 1000 symbols
        let mut graph = ProjectContextGraph::new();
        for i in 0..1000 {
            let item = AstItem::Function {
                name: format!("func_{}", i),
                visibility: "pub".to_string(),
                is_async: false,
                line: i,
            };
            graph
                .add_item(format!("func_{}", i), item)
                .expect("Failed to add item");
        }

        b.iter(|| {
            // Benchmark O(1) lookup
            let result = graph.get_item(black_box("func_500"));
            hint_black_box(result)
        });
    });

    c.bench_function("context_graph_pagerank_1000_nodes", |b| {
        // Setup: Build graph with 1000 symbols and edges
        let mut graph = ProjectContextGraph::new();
        for i in 0..1000 {
            let item = AstItem::Function {
                name: format!("func_{}", i),
                visibility: "pub".to_string(),
                is_async: false,
                line: i,
            };
            graph
                .add_item(format!("func_{}", i), item)
                .expect("Failed to add item");
        }

        // Add edges to create call graph
        for i in 0..999 {
            graph
                .add_edge(&format!("func_{}", i), &format!("func_{}", i + 1))
                .ok();
        }

        b.iter(|| {
            let mut graph_clone = graph.clone();
            graph_clone.update_hotness().expect("PageRank failed");
            hint_black_box(graph_clone.hot_symbols())
        });
    });

    c.bench_function("context_graph_full_build_10_files", |b| {
        use pmat::services::context::{analyze_project_with_cache, FileContext};
        use std::path::PathBuf;

        let rt = tokio::runtime::Runtime::new().unwrap();

        b.iter(|| {
            rt.block_on(async {
                // Benchmark building context graph from 10 small files
                let path = PathBuf::from("src/services");
                let result = analyze_project_with_cache(&path, "rust", None).await;

                if let Ok(context) = result {
                    // Verify graph was built
                    if let Some(graph) = context.graph {
                        hint_black_box((graph.num_nodes(), graph.num_edges()))
                    } else {
                        hint_black_box((0, 0))
                    }
                } else {
                    hint_black_box((0, 0))
                }
            })
        });
    });
}

criterion_group!(benches, bench_context_graph_operations);
criterion_main!(benches);
