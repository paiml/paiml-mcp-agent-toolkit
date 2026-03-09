//! DHAT Memory Profiling for PMAT
//!
//! Profiles heap allocations across the 3 main hot paths:
//! 1. AgentContextIndex::build() — index construction from AST
//! 2. AgentContextIndex::load() + query() — cached index load + semantic search
//! 3. DeepContextAnalyzer::analyze_project() — full project analysis
//!
//! # Usage
//!
//! ```bash
//! # Profile index build (default)
//! cargo run --example dhat_memory_profile -- build
//!
//! # Profile index load + query
//! cargo run --example dhat_memory_profile -- query
//!
//! # Profile deep context analysis
//! cargo run --example dhat_memory_profile -- context
//!
//! # Profile all three
//! cargo run --example dhat_memory_profile -- all
//! ```
//!
//! Output: `dhat-heap.json` in the current directory.
//! View at: https://nnethercote.github.io/dh_view/dh_view.html

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

use pmat::services::agent_context::{AgentContextIndex, QueryOptions};
use std::path::{Path, PathBuf};

fn main() {
    let _profiler = dhat::Profiler::new_heap();

    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("all");
    let project_path = args
        .get(2)
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    eprintln!("=== DHAT Memory Profiling ===");
    eprintln!("Mode: {mode}");
    eprintln!("Project: {}", project_path.display());
    eprintln!();

    match mode {
        "build" => profile_index_build(&project_path),
        "query" => profile_index_query(&project_path),
        "context" => profile_deep_context(&project_path),
        "all" => {
            profile_index_build(&project_path);
            profile_index_query(&project_path);
            profile_deep_context(&project_path);
        }
        _ => {
            eprintln!("Unknown mode: {mode}");
            eprintln!("Usage: dhat_memory_profile [build|query|context|all] [project_path]");
            std::process::exit(1);
        }
    }

    eprintln!("\n=== Done. View dhat-heap.json at https://nnethercote.github.io/dh_view/dh_view.html ===");
}

fn profile_index_build(project_path: &Path) {
    eprintln!("--- Phase 1: Index Build ---");
    let start = std::time::Instant::now();

    match AgentContextIndex::build(project_path) {
        Ok(index) => {
            let elapsed = start.elapsed();
            let stats = index.stats();
            eprintln!(
                "  Built index: {} functions in {:.2}s",
                stats.total_functions,
                elapsed.as_secs_f64()
            );
        }
        Err(e) => {
            eprintln!("  Build failed: {e}");
        }
    }
}

fn profile_index_query(project_path: &Path) {
    eprintln!("--- Phase 2: Index Load + Query ---");

    // Try to load from cache first
    // load() expects the .idx directory path; it derives .db via with_extension("db")
    let idx_path = project_path.join(".pmat").join("context.idx");
    let db_path = idx_path.with_extension("db");

    let index = if db_path.exists() || idx_path.exists() {
        eprintln!("  Loading cached index from {}...", idx_path.display());
        let start = std::time::Instant::now();
        match AgentContextIndex::load(&idx_path) {
            Ok(idx) => {
                eprintln!("  Loaded in {:.2}s", start.elapsed().as_secs_f64());
                idx
            }
            Err(e) => {
                eprintln!("  Cache load failed ({e}), building fresh...");
                AgentContextIndex::build(project_path).expect("build failed")
            }
        }
    } else {
        eprintln!("  No cache found, building index...");
        AgentContextIndex::build(project_path).expect("build failed")
    };

    // Run a set of representative queries
    let queries = [
        ("error handling", 10),
        ("parse", 20),
        ("complexity analysis", 5),
        ("cache", 15),
        ("format output", 10),
    ];

    for (q, limit) in &queries {
        let start = std::time::Instant::now();
        let options = QueryOptions {
            limit: *limit,
            ..Default::default()
        };
        match index.query(q, options) {
            Ok(results) => {
                eprintln!(
                    "  query({q:?}, limit={limit}): {} results in {:.1}ms",
                    results.len(),
                    start.elapsed().as_secs_f64() * 1000.0
                );
            }
            Err(e) => {
                eprintln!("  query({q:?}) failed: {e}");
            }
        }
    }
}

fn profile_deep_context(project_path: &Path) {
    eprintln!("--- Phase 3: Deep Context Analysis ---");

    use pmat::services::deep_context::{DeepContextAnalyzer, DeepContextConfig};

    let config = DeepContextConfig::default();
    let analyzer = DeepContextAnalyzer::new(config);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");

    let start = std::time::Instant::now();
    let path_buf = project_path.to_path_buf();

    match rt.block_on(analyzer.analyze_project(&path_buf)) {
        Ok(context) => {
            let elapsed = start.elapsed();
            eprintln!(
                "  Deep context: {} files analyzed in {:.2}s",
                context.file_tree.total_files,
                elapsed.as_secs_f64()
            );
        }
        Err(e) => {
            eprintln!("  Deep context failed: {e:#}");
        }
    }
}
