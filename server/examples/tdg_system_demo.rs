//! Comprehensive example demonstrating the Transactional Hashed TDG System
//! 
//! This example shows how to use all Sprint 30 features including:
//! - Tiered storage with different backends
//! - Fair scheduling for transactional operations
//! - Adaptive threshold management
//! - Platform resource control
//! - System diagnostics

use pmat::tdg::{
    // Storage components
    TieredStore, TieredStorageFactory, StorageConfig, StorageBackendType,
    StorageBackendFactory, FileIdentity, FullTdgRecord, ComponentScores,
    SemanticSignature, AnalysisMetadata,
    
    // Scheduling components
    SimpleFairScheduler, SchedulerFactory,
    
    // Adaptive thresholds
    AdaptiveThresholdManager, AdaptiveThresholdFactory, AdaptiveConfig,
    PerformanceSample,
    
    // Resource control
    PlatformResourceController, ResourceControllerFactory, ResourceLimits,
    OperationPriority,
    
    // Core TDG
    TdgScore, Grade, Language, TdgAnalyzer,
};

use std::path::PathBuf;
use std::time::{Duration, SystemTime, Instant};
use std::collections::HashMap;
use tokio::time::sleep;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Transactional Hashed TDG System Demo ===\n");
    
    // 1. Initialize storage with different backends
    demo_storage_backends().await?;
    
    // 2. Demonstrate fair scheduling
    demo_fair_scheduling().await?;
    
    // 3. Show adaptive threshold management
    demo_adaptive_thresholds().await?;
    
    // 4. Demonstrate resource control
    demo_resource_control().await?;
    
    // 5. Full integrated example
    demo_integrated_system().await?;
    
    Ok(())
}

/// Demonstrate different storage backends
async fn demo_storage_backends() -> Result<()> {
    println!("1. Storage Backend Demonstration");
    println!("---------------------------------");
    
    // Create in-memory storage for testing
    println!("Creating in-memory storage...");
    let memory_storage = TieredStore::in_memory();
    
    // Create sled-based storage for production
    let temp_dir = tempfile::TempDir::new()?;
    println!("Creating Sled storage at {:?}...", temp_dir.path());
    let sled_storage = TieredStorageFactory::create_at_path(temp_dir.path())?;
    
    // Create a test record
    let record = create_test_record("example.rs", 85.0, Grade::B);
    
    // Store in both backends
    println!("Storing record in both backends...");
    memory_storage.store(record.clone()).await?;
    sled_storage.store(record.clone()).await?;
    
    // Retrieve and verify
    let hash = record.identity.content_hash;
    
    if let Some(hot_entry) = memory_storage.get_hot(&hash) {
        println!("✓ Retrieved from memory hot cache: score={:.1}, grade={}", 
            hot_entry.total_score, hot_entry.grade);
    }
    
    if let Some(retrieved) = sled_storage.retrieve_full(&hash).await? {
        println!("✓ Retrieved from Sled storage: {}", retrieved.identity.path.display());
    }
    
    // Show statistics
    let mem_stats = memory_storage.get_statistics();
    let sled_stats = sled_storage.get_statistics();
    
    println!("\nMemory Storage Stats:");
    println!("  - Total entries: {}", mem_stats.total_entries);
    println!("  - Backend: {}", mem_stats.warm_backend);
    
    println!("\nSled Storage Stats:");
    println!("  - Total entries: {}", sled_stats.total_entries);
    println!("  - Backend: {}", sled_stats.warm_backend);
    println!("  - Compression ratio: {:.1}%", sled_stats.compression_ratio * 100.0);
    
    println!();
    Ok(())
}

/// Demonstrate fair scheduling with priority preemption
async fn demo_fair_scheduling() -> Result<()> {
    println!("2. Fair Scheduling Demonstration");
    println!("--------------------------------");
    
    let scheduler = SchedulerFactory::create_balanced();
    
    // Simulate background analysis
    let bg_path = PathBuf::from("background.rs");
    println!("Starting background analysis on {:?}...", bg_path);
    
    let bg_task = tokio::spawn({
        let scheduler = scheduler.clone();
        let path = bg_path.clone();
        async move {
            match scheduler.schedule_background(path).await {
                Ok(_guard) => {
                    println!("  → Background task acquired permit");
                    sleep(Duration::from_millis(500)).await;
                    println!("  → Background task completed");
                    Ok(())
                }
                Err(e) => {
                    println!("  → Background task preempted: {}", e);
                    Err(e)
                }
            }
        }
    });
    
    // Give background task time to start
    sleep(Duration::from_millis(100)).await;
    
    // High-priority commit arrives
    println!("High-priority commit requested on same file...");
    let commit_guard = scheduler.schedule_commit(bg_path.clone()).await?;
    println!("  → Commit acquired priority permit immediately");
    
    // Simulate commit work
    sleep(Duration::from_millis(200)).await;
    println!("  → Commit completed");
    drop(commit_guard);
    
    // Wait for background task
    let _ = bg_task.await;
    
    // Show scheduler statistics
    let stats = scheduler.get_statistics().await;
    println!("\nScheduler Statistics:");
    println!("  - High priority permits available: {}", stats.high_permits_available);
    println!("  - Low priority permits available: {}", stats.low_permits_available);
    println!("  - Active operations: {}", stats.total_active_operations);
    
    println!();
    Ok(())
}

/// Demonstrate adaptive threshold management
async fn demo_adaptive_thresholds() -> Result<()> {
    println!("3. Adaptive Threshold Management");
    println!("---------------------------------");
    
    let config = AdaptiveConfig {
        initial_hot_cache_size: 100,
        initial_compression_level: 3,
        initial_high_priority_permits: 10,
        initial_low_priority_permits: 2,
        learning_rate: 0.1,
        adjustment_interval: Duration::from_secs(5),
        performance_window: 10,
        auto_tune: true,
    };
    
    let adaptive = AdaptiveThresholdFactory::create_with_config(config)?;
    
    // Simulate performance samples
    println!("Recording performance samples...");
    
    for i in 0..5 {
        let sample = PerformanceSample {
            timestamp: Instant::now(),
            analysis_duration_ms: (50.0 + (i as f32 * 10.0)) as u64,
            cache_hit_ratio: if i % 2 == 0 { 0.8 } else { 0.6 },
            memory_usage_mb: 100.0 + (i as f32 * 5.0),
            cpu_utilization: 0.3 + (i as f32 * 0.1),
            queue_depth: i % 3,
        };
        
        adaptive.record_performance(sample).await;
        println!("  Sample {}: duration={}ms, cache_hit_ratio={}, memory={}MB", 
            i + 1, sample.analysis_duration_ms, sample.cache_hit_ratio, sample.memory_usage_mb);
    }
    
    // Trigger adjustment
    println!("\nTriggering threshold adjustment...");
    let adjustment = adaptive.adjust_thresholds().await?;
    
    if let Some(adj) = adjustment {
        println!("  Adjustments made:");
        if adj.cache_size_delta != 0 {
            println!("    - Cache size: {:+}", adj.cache_size_delta);
        }
        if adj.compression_delta != 0 {
            println!("    - Compression level: {:+}", adj.compression_delta);
        }
        if adj.high_priority_delta != 0 {
            println!("    - High priority permits: {:+}", adj.high_priority_delta);
        }
        if adj.low_priority_delta != 0 {
            println!("    - Low priority permits: {:+}", adj.low_priority_delta);
        }
        println!("    - Reason: {}", adj.reason);
    } else {
        println!("  No adjustments needed (system performing well)");
    }
    
    // Show current thresholds
    let thresholds = adaptive.get_current_thresholds().await;
    println!("\nCurrent Thresholds:");
    println!("  - Hot cache size: {}", thresholds.hot_cache_size);
    println!("  - Compression level: {}", thresholds.compression_level);
    println!("  - High priority permits: {}", thresholds.high_priority_permits);
    println!("  - Low priority permits: {}", thresholds.low_priority_permits);
    
    // Show performance statistics
    let stats = adaptive.get_performance_stats().await;
    println!("\nPerformance Statistics:");
    println!("  - Avg analysis time: {:.1}ms", stats.avg_analysis_duration_ms);
    println!("  - Cache hit ratio: {:.1}%", stats.avg_cache_hit_ratio * 100.0);
    println!("  - Avg memory usage: {:.1}MB", stats.avg_memory_usage_mb);
    println!("  - Avg CPU utilization: {:.1}%", stats.avg_cpu_utilization * 100.0);
    
    println!();
    Ok(())
}

/// Demonstrate platform resource control
async fn demo_resource_control() -> Result<()> {
    println!("4. Platform Resource Control");
    println!("-----------------------------");
    
    let limits = ResourceLimits {
        max_memory_mb: 500.0,
        max_cpu_percent: 80.0,
        max_concurrent_operations: 10,
        max_cache_size_mb: 100.0,
        throttle_threshold: 0.7,
        queue_threshold: 0.9,
    };
    
    let controller = ResourceControllerFactory::create_with_limits(limits)?;
    
    // Request resources for different priority operations
    println!("Requesting resources for operations...");
    
    // High priority operation
    let high_alloc = controller.request_resources(
        "critical_analysis",
        50.0,  // memory
        30.0,  // cpu
        OperationPriority::Critical
    ).await?;
    
    match high_alloc.action {
        pmat::tdg::ResourceAction::Allow => {
            println!("✓ Critical operation: ALLOWED");
            println!("  - Allocated memory: {:.1}MB", high_alloc.allocated_memory_mb);
            println!("  - Allocated CPU: {:.1}%", high_alloc.allocated_cpu_percent);
        }
        _ => println!("✗ Critical operation: {:?}", high_alloc.action),
    }
    
    // Normal priority operation
    let normal_alloc = controller.request_resources(
        "routine_scan",
        100.0,
        20.0,
        OperationPriority::Medium
    ).await?;
    
    match normal_alloc.action {
        pmat::tdg::ResourceAction::Allow => {
            println!("✓ Normal operation: ALLOWED");
        }
        pmat::tdg::ResourceAction::Throttle(factor) => {
            println!("⚠ Normal operation: THROTTLED (factor: {:.1})", factor);
        }
        _ => println!("✗ Normal operation: {:?}", normal_alloc.action),
    }
    
    // Show resource statistics
    let stats = controller.get_statistics().await;
    println!("\nResource Usage Statistics:");
    println!("  - Current memory: {:.1}MB / {:.1}MB", 
        stats.current_memory_mb, stats.max_memory_mb);
    println!("  - Current CPU: {:.1}% / {:.1}%", 
        stats.current_cpu_percent, stats.max_cpu_percent);
    println!("  - Active operations: {}", stats.active_operations);
    println!("  - Throttled operations: {}", stats.throttled_operations);
    println!("  - Rejected operations: {}", stats.rejected_operations);
    
    // Release resources
    controller.release_resources("critical_analysis").await;
    controller.release_resources("routine_scan").await;
    
    println!();
    Ok(())
}

/// Demonstrate integrated system with all components
async fn demo_integrated_system() -> Result<()> {
    println!("5. Integrated System Demonstration");
    println!("-----------------------------------");
    
    // Initialize all components
    let temp_dir = tempfile::TempDir::new()?;
    let storage = TieredStorageFactory::create_at_path(temp_dir.path())?;
    let scheduler = SchedulerFactory::create_balanced();
    let adaptive = AdaptiveThresholdFactory::create_default()?;
    let resources = ResourceControllerFactory::create_default()?;
    
    println!("System initialized with all components");
    
    // Simulate a transactional TDG analysis workflow
    println!("\nSimulating code analysis workflow...");
    
    // Step 1: Request resources
    let allocation = resources.request_resources(
        "tdg_analysis",
        50.0,
        25.0,
        OperationPriority::Medium
    ).await?;
    
    if allocation.action != pmat::tdg::ResourceAction::Allow {
        println!("⚠ Resource allocation limited: {:?}", allocation.action);
    }
    
    // Step 2: Schedule the analysis
    let path = PathBuf::from("src/main.rs");
    let _guard = scheduler.schedule_background(path.clone()).await?;
    println!("✓ Analysis scheduled");
    
    // Step 3: Perform analysis (simulated)
    let start = Instant::now();
    let record = create_test_record("src/main.rs", 92.0, Grade::A);
    
    // Step 4: Store results transactionally
    storage.store(record.clone()).await?;
    println!("✓ Results stored (hash: {}...)", 
        hex::encode(&record.identity.content_hash.as_bytes()[0..8]));
    
    // Step 5: Record performance
    let sample = PerformanceSample {
        timestamp: Instant::now(),
        analysis_duration_ms: start.elapsed().as_millis() as u64,
        cache_hit_ratio: 0.0, // Cold cache
        memory_usage_mb: allocation.allocated_memory_mb,
        cpu_utilization: allocation.allocated_cpu_percent / 100.0,
        queue_depth: 0,
    };
    adaptive.record_performance(sample).await;
    
    // Step 6: Check if thresholds need adjustment
    if let Some(adjustment) = adaptive.adjust_thresholds().await? {
        println!("✓ System auto-tuned: {}", adjustment.reason);
    }
    
    // Show final statistics
    println!("\nSystem Statistics:");
    let storage_stats = storage.get_statistics();
    println!("  Storage: {} entries, {:.1}% compression", 
        storage_stats.total_entries,
        storage_stats.compression_ratio * 100.0);
    
    let scheduler_stats = scheduler.get_statistics().await;
    println!("  Scheduler: {} active ops, {}ms avg wait", 
        scheduler_stats.total_active_operations,
        scheduler_stats.avg_wait_time_ms);
    
    let perf_stats = adaptive.get_performance_stats().await;
    println!("  Performance: {:.1}ms avg, {:.1}% cache hits",
        perf_stats.avg_analysis_duration_ms,
        perf_stats.avg_cache_hit_ratio * 100.0);
    
    println!("\n✅ Integrated system demonstration complete!");
    
    Ok(())
}

/// Helper function to create a test TDG record
fn create_test_record(path: &str, score: f32, grade: Grade) -> FullTdgRecord {
    let content = b"fn example() { println!(\"test\"); }";
    let hash = blake3::hash(content);
    
    FullTdgRecord {
        identity: FileIdentity {
            path: PathBuf::from(path),
            content_hash: hash,
            size_bytes: content.len() as u64,
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
            ast_structure_hash: 987654321,
            identifier_pattern: "example,println".to_string(),
            control_flow_pattern: "function,call".to_string(),
            import_dependencies: Vec::new(),
        },
        metadata: AnalysisMetadata {
            analyzer_version: "2.38.0".to_string(),
            analysis_duration_ms: 10,
            language_confidence: 0.98,
            analysis_timestamp: SystemTime::now(),
            cache_hit: false,
        },
    }
}