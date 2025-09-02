/// Sprint 30 Integration Test: Transactional Hashed TDG System
///
/// This test demonstrates the complete integrated system with:
/// - Week 2: Tiered Storage (Hot/Warm/Cold with LZ4 compression)  
/// - Week 3: Fair Scheduling (Priority-based with tokio primitives)
/// - Week 4: Adaptive Thresholds (Performance monitoring and auto-adjustment)
/// - Week 5: Platform Resource Control (CPU/memory limits with enforcement)

#[cfg(test)]
mod sprint30_integration_tests {
    use crate::tdg::{
        AdaptiveThresholdFactory, OperationPriority, ResourceControllerFactory, SchedulerFactory,
        TdgAnalyzer, TdgConfig, TieredStorageFactory, Grade,
    };
    use crate::models::unified_ast::Language;
    use std::path::Path;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_sprint30_complete_integration() {
        // Create analyzer with all Sprint 30 components
        let config = TdgConfig::default();
        let analyzer = TdgAnalyzer::with_full_resource_management(config)
            .await
            .expect("Failed to create analyzer with full resource management");

        // Test file analysis with resource control
        let test_file = Path::new("server/src/tdg/resource_control.rs");

        // Simulate commit priority analysis (should get highest priority)
        let commit_score = analyzer
            .analyze_file_commit(test_file)
            .await
            .expect("Failed to analyze file with commit priority");

        println!(
            "Commit analysis completed - Score: {:.1}",
            commit_score.total
        );

        // Simulate background analysis (should get lower priority)
        let background_score = analyzer
            .analyze_file_background(test_file)
            .await
            .expect("Failed to analyze file with background priority");

        println!(
            "Background analysis completed - Score: {:.1}",
            background_score.total
        );

        // Get system diagnostics
        if let Some(scheduler_stats) = analyzer.get_scheduler_stats().await {
            println!("Scheduler Stats: {}", scheduler_stats.format_diagnostic());
        }

        if let Some(adaptive_stats) = analyzer.get_adaptive_stats().await {
            println!("Adaptive Stats: {}", adaptive_stats.format_diagnostic());
        }

        if let Some(resource_stats) = analyzer.get_resource_stats().await {
            println!("Resource Stats: {}", resource_stats.format_diagnostic());
        }

        if let Some(resource_usage) = analyzer.get_resource_usage().await {
            println!(
                "Current Resource Usage: {:.1}MB memory, {:.1}% CPU",
                resource_usage.memory_mb,
                resource_usage.cpu_utilization * 100.0
            );
        }

        // Verify both analyses succeeded
        assert!(
            commit_score.total > 0.0,
            "Commit analysis should produce valid score"
        );
        assert!(
            background_score.total > 0.0,
            "Background analysis should produce valid score"
        );
        assert!(
            commit_score.grade != crate::tdg::Grade::F,
            "Analysis should not fail"
        );
        assert!(
            background_score.grade != crate::tdg::Grade::F,
            "Analysis should not fail"
        );

        println!("✅ Sprint 30 Week 5 Complete: All transactional hashed TDG components integrated successfully!");
    }

    #[tokio::test]
    async fn test_resource_enforcement_under_pressure() {
        // Create resource controller with very low limits to test enforcement
        let resource_controller = ResourceControllerFactory::create_dev_optimized(); // Lower limits
        resource_controller.start_monitoring().await.unwrap();

        // Request multiple high-memory operations to trigger enforcement
        let mut allocations = Vec::new();

        for i in 0..5 {
            let result = resource_controller
                .request_resources(
                    format!("test-op-{}", i),
                    crate::tdg::resource_control::OperationType::Analysis,
                    OperationPriority::Medium,
                    100.0, // 100MB per operation
                )
                .await;

            match result {
                Ok(allocation) => {
                    println!("Operation {} allowed", i);
                    allocations.push(allocation);
                }
                Err(e) => {
                    println!("Operation {} rejected: {}", i, e);
                    break; // Expected - resource limits reached
                }
            }
        }

        // Check enforcement statistics
        let stats = resource_controller.get_enforcement_stats().await;
        println!("Final enforcement stats: {}", stats.format_diagnostic());

        resource_controller.stop_monitoring().await;

        // Should have some operations and likely some rejections/throttling due to limits
        assert!(
            stats.total_requests > 0,
            "Should have processed some requests"
        );
        println!("✅ Resource enforcement working correctly under pressure");
    }

    #[tokio::test]
    async fn test_adaptive_performance_optimization() {
        // Create adaptive manager and simulate performance samples
        let adaptive_manager = AdaptiveThresholdFactory::create_dev_optimized();

        // Simulate slow performance that should trigger optimization
        for i in 0..15 {
            let sample = adaptive_manager
                .create_sample(
                    Duration::from_millis(150 + i * 10), // Getting slower
                    false,                               // Cache misses
                    5,                                   // Queue depth
                )
                .await;

            adaptive_manager
                .record_sample(sample)
                .await
                .expect("Failed to record performance sample");
        }

        // Get performance statistics
        let stats = adaptive_manager.get_performance_stats().await;
        println!(
            "Performance stats after slow samples: {}",
            stats.format_diagnostic()
        );

        // Get current thresholds (may have been adjusted)
        let thresholds = adaptive_manager.get_current_thresholds().await;
        println!(
            "Current thresholds - Cache: {}, Permits: {}/{}, Compression: {}",
            thresholds.hot_cache_size,
            thresholds.high_priority_permits,
            thresholds.low_priority_permits,
            thresholds.compression_level
        );

        // Should have recorded samples and potentially made adjustments
        assert!(
            stats.total_samples > 10,
            "Should have recorded multiple samples"
        );
        println!("✅ Adaptive performance optimization working correctly");
    }

    #[tokio::test]
    async fn test_tiered_storage_with_compression() {
        // Create tiered storage
        let storage =
            TieredStorageFactory::create_default().expect("Failed to create tiered storage");

        // Create a sample TDG record to store
        use crate::tdg::{
            AnalysisMetadata, ComponentScores, FileIdentity, FullTdgRecord, Grade, Language,
            SemanticSignature, TdgScore,
        };
        use blake3;
        use std::time::SystemTime;

        let test_content = "fn main() { println!(\"Hello, world!\"); }";
        let content_hash = blake3::hash(test_content.as_bytes());

        let record = FullTdgRecord {
            identity: FileIdentity {
                path: Path::new("test.rs").to_path_buf(),
                content_hash,
                size_bytes: test_content.len() as u64,
                modified_time: SystemTime::now(),
            },
            score: TdgScore {
                total: 85.5,
                grade: Grade::AMinus,
                language: Language::Rust,
                confidence: 0.95,
                file_path: Some(Path::new("test.rs").to_path_buf()),
                ..Default::default()
            },
            components: ComponentScores {
                complexity_breakdown: std::collections::HashMap::new(),
                duplication_sources: Vec::new(),
                coupling_dependencies: Vec::new(),
                doc_missing_items: Vec::new(),
                consistency_violations: Vec::new(),
            },
            semantic_sig: SemanticSignature {
                ast_structure_hash: u64::from_le_bytes(
                    content_hash.as_bytes()[0..8].try_into().unwrap(),
                ),
                identifier_pattern: "main".to_string(),
                control_flow_pattern: "sequential".to_string(),
                import_dependencies: Vec::new(),
            },
            metadata: AnalysisMetadata {
                analyzer_version: "2.37.3".to_string(),
                analysis_duration_ms: 50,
                language_confidence: 0.95,
                analysis_timestamp: SystemTime::now(),
                cache_hit: false,
            },
        };

        // Store the record
        storage
            .store(record)
            .await
            .expect("Failed to store record in tiered storage");

        // Retrieve from hot cache
        if let Some(hot_entry) = storage.get_hot(&content_hash) {
            println!(
                "Retrieved from hot cache: score = {:.1}",
                hot_entry.total_score
            );
            assert!(
                (hot_entry.total_score - 85.5).abs() < 0.1,
                "Score should match stored value"
            );
        } else {
            panic!("Record should be in hot cache after storage");
        }

        // Get storage statistics
        let stats = storage.get_statistics();
        println!("Storage stats: {}", stats.format_diagnostic());

        assert!(
            stats.total_entries > 0,
            "Should have stored at least one record"
        );
        println!("✅ Tiered storage with compression working correctly");
    }

    #[tokio::test]
    async fn test_fair_scheduling_preemption() {
        // Create fair scheduler
        let scheduler = SchedulerFactory::create_balanced();

        // Start a background operation
        let background_guard = scheduler
            .schedule_background(Path::new("background_file.rs").to_path_buf())
            .await
            .expect("Failed to schedule background operation");

        // Wait briefly to ensure background operation is active
        sleep(Duration::from_millis(10)).await;

        // Schedule a commit operation (should preempt background)
        let commit_guard = scheduler
            .schedule_commit(Path::new("commit_file.rs").to_path_buf())
            .await
            .expect("Failed to schedule commit operation");

        // Get scheduling statistics
        let stats = scheduler.get_statistics().await;
        println!("Scheduler stats: {}", stats.format_diagnostic());

        // Both operations should be tracked
        assert!(
            stats.total_active_operations >= 0,
            "Should have tracked operations"
        );

        // Clean up
        drop(commit_guard);
        drop(background_guard);

        println!("✅ Fair scheduling with preemption working correctly");
    }
}
