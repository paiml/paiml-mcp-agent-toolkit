//! Benchmarks for TDG storage backend performance
//! 
//! These benchmarks compare the performance of different storage backends
//! under various workload patterns.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use pmat::tdg::{
    StorageBackend, StorageBackendFactory, StorageBackendType, StorageConfig,
    InMemoryBackend, SledBackend, TieredStore, TieredStorageFactory,
    FullTdgRecord, FileIdentity, TdgScore, Grade, Language,
    ComponentScores, SemanticSignature, AnalysisMetadata,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;
use tempfile::TempDir;

/// Benchmark basic put operations
fn bench_put_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_put");
    
    let data_sizes = vec![100, 1000, 10000];
    
    for size in data_sizes {
        // Generate test data
        let data: Vec<(Vec<u8>, Vec<u8>)> = (0..size)
            .map(|i| {
                let key = format!("key_{:06}", i).into_bytes();
                let value = format!("value_{:06}_with_some_additional_data_to_make_it_realistic", i).into_bytes();
                (key, value)
            })
            .collect();
        
        // Benchmark InMemoryBackend
        group.bench_with_input(
            BenchmarkId::new("InMemory", size),
            &data,
            |b, data| {
                b.iter(|| {
                    let backend = InMemoryBackend::new();
                    for (key, value) in data {
                        black_box(backend.put(key, value).unwrap());
                    }
                });
            },
        );
        
        // Benchmark SledBackend
        group.bench_with_input(
            BenchmarkId::new("Sled", size),
            &data,
            |b, data| {
                b.iter_batched(
                    || {
                        let temp_dir = TempDir::new().unwrap();
                        (SledBackend::new(temp_dir.path()).unwrap(), temp_dir)
                    },
                    |(backend, _temp_dir)| {
                        for (key, value) in data {
                            black_box(backend.put(key, value).unwrap());
                        }
                        backend.flush().unwrap();
                    },
                    criterion::BatchSize::LargeInput,
                );
            },
        );
    }
    
    group.finish();
}

/// Benchmark get operations
fn bench_get_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_get");
    
    let data_sizes = vec![100, 1000, 10000];
    
    for size in data_sizes {
        let data: Vec<(Vec<u8>, Vec<u8>)> = (0..size)
            .map(|i| {
                let key = format!("key_{:06}", i).into_bytes();
                let value = format!("value_{:06}", i).into_bytes();
                (key, value)
            })
            .collect();
        
        // Benchmark InMemoryBackend
        group.bench_with_input(
            BenchmarkId::new("InMemory", size),
            &data,
            |b, data| {
                let backend = InMemoryBackend::new();
                // Pre-populate
                for (key, value) in data {
                    backend.put(key, value).unwrap();
                }
                
                let keys: Vec<_> = data.iter().map(|(k, _)| k).collect();
                b.iter(|| {
                    for key in &keys {
                        black_box(backend.get(key).unwrap());
                    }
                });
            },
        );
        
        // Benchmark SledBackend
        group.bench_with_input(
            BenchmarkId::new("Sled", size),
            &data,
            |b, data| {
                b.iter_batched(
                    || {
                        let temp_dir = TempDir::new().unwrap();
                        let backend = SledBackend::new(temp_dir.path()).unwrap();
                        
                        // Pre-populate
                        for (key, value) in data {
                            backend.put(key, value).unwrap();
                        }
                        backend.flush().unwrap();
                        
                        let keys: Vec<_> = data.iter().map(|(k, _)| k).collect();
                        (backend, keys, temp_dir)
                    },
                    |(backend, keys, _temp_dir)| {
                        for key in &keys {
                            black_box(backend.get(key).unwrap());
                        }
                    },
                    criterion::BatchSize::LargeInput,
                );
            },
        );
    }
    
    group.finish();
}

/// Benchmark mixed read/write workloads
fn bench_mixed_workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_mixed");
    
    let operations = 1000;
    
    // Generate mixed operations (70% reads, 30% writes)
    let mut ops = Vec::new();
    for i in 0..operations {
        if i % 10 < 7 {
            // Read operation
            let key = format!("key_{:06}", i % 100).into_bytes();
            ops.push(Operation::Get(key));
        } else {
            // Write operation
            let key = format!("key_{:06}", i).into_bytes();
            let value = format!("value_{:06}", i).into_bytes();
            ops.push(Operation::Put(key, value));
        }
    }
    
    // Benchmark InMemoryBackend
    group.bench_function("InMemory", |b| {
        b.iter(|| {
            let backend = InMemoryBackend::new();
            
            // Pre-populate with some data
            for i in 0..100 {
                let key = format!("key_{:06}", i).into_bytes();
                let value = format!("initial_value_{:06}", i).into_bytes();
                backend.put(&key, &value).unwrap();
            }
            
            // Execute mixed workload
            for op in &ops {
                match op {
                    Operation::Put(key, value) => {
                        black_box(backend.put(key, value).unwrap());
                    }
                    Operation::Get(key) => {
                        black_box(backend.get(key).unwrap());
                    }
                }
            }
        });
    });
    
    // Benchmark SledBackend
    group.bench_function("Sled", |b| {
        b.iter_batched(
            || {
                let temp_dir = TempDir::new().unwrap();
                let backend = SledBackend::new(temp_dir.path()).unwrap();
                
                // Pre-populate with some data
                for i in 0..100 {
                    let key = format!("key_{:06}", i).into_bytes();
                    let value = format!("initial_value_{:06}", i).into_bytes();
                    backend.put(&key, &value).unwrap();
                }
                backend.flush().unwrap();
                
                (backend, temp_dir)
            },
            |(backend, _temp_dir)| {
                // Execute mixed workload
                for op in &ops {
                    match op {
                        Operation::Put(key, value) => {
                            black_box(backend.put(key, value).unwrap());
                        }
                        Operation::Get(key) => {
                            black_box(backend.get(key).unwrap());
                        }
                    }
                }
            },
            criterion::BatchSize::LargeInput,
        );
    });
    
    group.finish();
}

/// Benchmark TieredStore operations
fn bench_tiered_store(c: &mut Criterion) {
    let mut group = c.benchmark_group("tiered_store");
    
    let record_count = 100;
    
    // Generate test TDG records
    let records: Vec<FullTdgRecord> = (0..record_count)
        .map(|i| create_test_record(&format!("file_{:03}.rs", i), 85.0 + (i as f32 % 15.0), Grade::B))
        .collect();
    
    // Benchmark in-memory tiered store
    group.bench_function("InMemory", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let storage = TieredStore::in_memory();
                
                // Store all records
                for record in &records {
                    black_box(storage.store(record.clone()).await.unwrap());
                }
                
                // Retrieve all records
                for record in &records {
                    black_box(storage.retrieve_full(&record.identity.content_hash).await.unwrap());
                }
            });
        });
    });
    
    // Benchmark sled-based tiered store
    group.bench_function("Sled", |b| {
        b.iter_batched(
            || TempDir::new().unwrap(),
            |temp_dir| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let storage = TieredStorageFactory::create_at_path(temp_dir.path()).unwrap();
                    
                    // Store all records
                    for record in &records {
                        black_box(storage.store(record.clone()).await.unwrap());
                    }
                    
                    // Retrieve all records
                    for record in &records {
                        black_box(storage.retrieve_full(&record.identity.content_hash).await.unwrap());
                    }
                });
            },
            criterion::BatchSize::LargeInput,
        );
    });
    
    group.finish();
}

/// Benchmark hot cache performance
fn bench_hot_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("hot_cache");
    
    let record_count = 1000;
    let records: Vec<FullTdgRecord> = (0..record_count)
        .map(|i| create_test_record(&format!("file_{:04}.rs", i), 80.0 + (i as f32 % 20.0), Grade::B))
        .collect();
    
    group.bench_function("hot_cache_hits", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let storage = TieredStore::in_memory();
                
                // Pre-populate hot cache
                for record in &records {
                    storage.store(record.clone()).await.unwrap();
                }
                
                // Benchmark hot cache lookups
                for record in &records {
                    black_box(storage.get_hot(&record.identity.content_hash));
                }
            });
        });
    });
    
    group.finish();
}

/// Benchmark compression ratios
fn bench_compression_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression");
    
    let records: Vec<FullTdgRecord> = (0..100)
        .map(|i| {
            // Create records with varying amounts of data to test compression
            let path = format!("src/module_{}/submodule_{}/file_{}.rs", i / 10, i / 5, i);
            create_test_record(&path, 75.0 + (i as f32 % 25.0), Grade::B)
        })
        .collect();
    
    group.bench_function("store_with_compression", |b| {
        b.iter_batched(
            || TempDir::new().unwrap(),
            |temp_dir| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let storage = TieredStorageFactory::create_at_path(temp_dir.path()).unwrap();
                    
                    for record in &records {
                        black_box(storage.store(record.clone()).await.unwrap());
                    }
                    
                    // Force flush to ensure compression is applied
                    storage.flush().unwrap();
                    
                    // Get statistics to verify compression
                    let stats = storage.get_statistics();
                    black_box(stats.compression_ratio);
                });
            },
            criterion::BatchSize::LargeInput,
        );
    });
    
    group.finish();
}

#[derive(Clone)]
enum Operation {
    Put(Vec<u8>, Vec<u8>),
    Get(Vec<u8>),
}

/// Helper function to create test TDG records
fn create_test_record(path: &str, score: f32, grade: Grade) -> FullTdgRecord {
    let content = format!("// File: {}\nfn main() {{\n    println!(\"Hello, world!\");\n}}\n", path);
    let content_bytes = content.as_bytes();
    let hash = blake3::hash(content_bytes);
    
    FullTdgRecord {
        identity: FileIdentity {
            path: PathBuf::from(path),
            content_hash: hash,
            size_bytes: content_bytes.len() as u64,
            modified_time: SystemTime::now(),
        },
        score: TdgScore {
            structural_complexity: score * 0.25,
            semantic_complexity: score * 0.20,
            duplication_ratio: score * 0.20,
            coupling_score: score * 0.15,
            doc_coverage: score * 0.10,
            consistency_score: score * 0.10,
            total: score,
            grade,
            confidence: 0.95,
            language: Language::Rust,
            file_path: Some(PathBuf::from(path)),
            penalties_applied: Vec::new(),
        },
        components: ComponentScores {
            complexity_breakdown: HashMap::new(),
            duplication_sources: Vec::new(),
            coupling_dependencies: Vec::new(),
            doc_missing_items: Vec::new(),
            consistency_violations: Vec::new(),
        },
        semantic_sig: SemanticSignature {
            ast_structure_hash: hash.as_bytes()[0..8].iter().fold(0u64, |acc, &b| acc.wrapping_mul(256) + b as u64),
            identifier_pattern: "main,println".to_string(),
            control_flow_pattern: "function,call".to_string(),
            import_dependencies: Vec::new(),
        },
        metadata: AnalysisMetadata {
            analyzer_version: "2.38.0".to_string(),
            analysis_duration_ms: 5 + (hash.as_bytes()[0] % 20) as u64,
            language_confidence: 0.95 + (hash.as_bytes()[1] % 5) as f32 * 0.01,
            analysis_timestamp: SystemTime::now(),
            cache_hit: hash.as_bytes()[2] % 3 == 0,
        },
    }
}

criterion_group!(
    benches,
    bench_put_operations,
    bench_get_operations,
    bench_mixed_workload,
    bench_tiered_store,
    bench_hot_cache,
    bench_compression_efficiency
);
criterion_main!(benches);