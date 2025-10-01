// Example usage of Claude Agent SDK Integration
// Demonstrates basic usage, caching, feature flags, and observability

use pmat::claude_integration::{
    BridgeConfig, ClaudeBridge, FeatureFlagsBuilder, MetricsCollector, RolloutStrategy,
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("=== Claude Agent SDK Integration Examples ===\n");

    // Example 1: Basic Usage
    basic_usage().await?;

    // Example 2: Feature Flags
    feature_flags_example().await?;

    // Example 3: Caching
    caching_example().await?;

    // Example 4: Observability
    observability_example().await?;

    // Example 5: Progressive Rollout
    progressive_rollout_example().await?;

    println!("\n=== All examples completed successfully ===");

    Ok(())
}

/// Example 1: Basic usage of Claude bridge
async fn basic_usage() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Example 1: Basic Usage\n");

    // Create bridge with default configuration
    let config = BridgeConfig::default();
    let bridge = ClaudeBridge::new(config).await?;

    println!("✓ Bridge initialized in {:?}", bridge.init_time());

    // Analyze some code
    let code = r#"
    fn complex_function() {
        if condition1 {
            for item in items {
                while processing {
                    match state {
                        State::A => {},
                        State::B => {},
                    }
                }
            }
        }
    }
    "#;

    let result = bridge.analyze_code(code).await?;

    println!("Analysis Results:");
    println!("  Complexity: {}", result.complexity);
    println!("  Cognitive Complexity: {}", result.cognitive_complexity);
    println!("  SATD Count: {}", result.satd_count);

    // Check health
    let healthy = bridge.health_check().await;
    println!("  Health: {}\n", if healthy { "✓" } else { "✗" });

    Ok(())
}

/// Example 2: Feature flags for controlled rollout
async fn feature_flags_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚩 Example 2: Feature Flags\n");

    // Create feature flags with allowlist strategy
    let flags = FeatureFlagsBuilder::new()
        .enabled(true)
        .strategy(RolloutStrategy::Allowlist)
        .add_to_allowlist("user_123")
        .add_to_allowlist("user_456")
        .build();

    // Test different users
    let test_users = vec!["user_123", "user_456", "user_789", "user_000"];

    for user in test_users {
        let should_use = flags.should_use_claude(user);
        println!(
            "  {} {} Claude: {}",
            user,
            if should_use { "✓ uses" } else { "✗ skips" },
            if should_use { "enabled" } else { "disabled" }
        );
    }

    // Switch to percentage rollout
    println!("\n  Switching to 50% rollout...");
    flags.set_strategy(RolloutStrategy::Percentage(50));

    // Test with many users
    let mut enabled_count = 0;
    for i in 0..100 {
        if flags.should_use_claude(&format!("user_{}", i)) {
            enabled_count += 1;
        }
    }

    println!("  50% rollout: {}/100 users enabled", enabled_count);

    // Test kill switch
    println!("\n  Testing kill switch...");
    flags.disable();
    println!("  After disable: {}", flags.should_use_claude("any_user"));

    println!();
    Ok(())
}

/// Example 3: Caching performance
async fn caching_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚡ Example 3: Caching Performance\n");

    let config = BridgeConfig {
        enable_cache: true,
        ..Default::default()
    };

    let bridge = ClaudeBridge::new(config).await?;

    let test_code = "fn test() { if x { for y in z {} } }";

    // First call - cache miss
    let start = std::time::Instant::now();
    let _ = bridge.analyze_code(test_code).await?;
    let first_duration = start.elapsed();

    // Second call - cache hit
    let start = std::time::Instant::now();
    let _ = bridge.analyze_code(test_code).await?;
    let second_duration = start.elapsed();

    println!("  First call (cache miss):  {:?}", first_duration);
    println!("  Second call (cache hit):  {:?}", second_duration);

    let speedup = first_duration.as_micros() as f64 / second_duration.as_micros() as f64;
    println!("  Speedup: {:.2}x faster", speedup);

    // Get cache statistics
    let stats = bridge.cache_stats();
    println!("\n  Cache Statistics:");
    println!("    L1 hit rate: {:.1}%", stats.l1_hit_rate * 100.0);
    println!("    L2 hit rate: {:.1}%", stats.l2_hit_rate * 100.0);
    println!(
        "    Effective hit rate: {:.1}%\n",
        stats.effective_hit_rate * 100.0
    );

    Ok(())
}

/// Example 4: Observability and metrics
async fn observability_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Example 4: Observability\n");

    let metrics = MetricsCollector::new();

    // Simulate some requests
    for i in 0..10 {
        let latency = Duration::from_micros(1000 + (i * 100));
        if i < 8 {
            metrics.record_success(latency);
        } else {
            metrics.record_error(
                pmat::claude_integration::observability::ErrorType::Application,
                latency,
            );
        }
    }

    // Simulate cache activity
    for _ in 0..7 {
        metrics.record_cache_hit();
    }
    for _ in 0..3 {
        metrics.record_cache_miss();
    }

    // Get metrics snapshot
    let snapshot = metrics.snapshot();

    println!("  Metrics Summary:");
    println!("    Total requests: {}", snapshot.requests_total);
    println!("    Success rate: {:.1}%", snapshot.success_rate * 100.0);
    println!("    Avg latency: {}μs", snapshot.avg_latency_us);
    println!("    Min latency: {}μs", snapshot.min_latency_us);
    println!("    Max latency: {}μs", snapshot.max_latency_us);
    println!(
        "    Cache hit rate: {:.1}%\n",
        snapshot.cache_hit_rate * 100.0
    );

    println!("  Formatted: {}\n", snapshot.format_summary());

    Ok(())
}

/// Example 5: Progressive rollout with automatic rollback
async fn progressive_rollout_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Example 5: Progressive Rollout\n");

    let flags = FeatureFlagsBuilder::new()
        .enabled(true)
        .strategy(RolloutStrategy::Percentage(10))
        .max_latency(1000) // 1 second max
        .build();

    println!("  Starting with 10% rollout...");
    println!("  Current percentage: {}%", flags.get_percentage());

    // Simulate good performance
    println!("\n  Simulating good performance (500ms)...");
    let rolled_back = flags.auto_rollback_on_degradation(500);
    println!("  Rolled back: {}", rolled_back);
    println!("  Still enabled: {}", flags.is_enabled());

    // Simulate bad performance - triggers rollback
    println!("\n  Simulating bad performance (2000ms)...");
    let rolled_back = flags.auto_rollback_on_degradation(2000);
    println!("  Rolled back: {}", rolled_back);
    println!("  Still enabled: {}", flags.is_enabled());

    // Try to use after rollback
    println!("\n  Attempting to use after rollback:");
    println!("  User allowed: {}\n", flags.should_use_claude("test_user"));

    Ok(())
}

/// Example 6: Complete workflow
#[allow(dead_code)]
async fn complete_workflow_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Example 6: Complete Workflow\n");

    // 1. Setup feature flags
    let flags = FeatureFlagsBuilder::new()
        .enabled(true)
        .strategy(RolloutStrategy::Percentage(25))
        .max_latency(3000)
        .build();

    // 2. Setup metrics
    let metrics = MetricsCollector::new();

    // 3. Create bridge
    let config = BridgeConfig {
        pool_size: 5,
        enable_cache: true,
        ..Default::default()
    };
    let bridge = ClaudeBridge::new(config).await?;

    println!("  ✓ Bridge initialized");

    // 4. Process requests with feature flags
    for i in 0..100 {
        let user_id = format!("user_{}", i);

        if flags.should_use_claude(&user_id) {
            let timer = pmat::claude_integration::observability::Timer::new();

            match bridge.analyze_code("fn test() {}").await {
                Ok(_result) => {
                    let latency = timer.elapsed();
                    metrics.record_success(latency);

                    // Check for performance degradation
                    if flags.auto_rollback_on_degradation(latency.as_millis() as u32) {
                        println!("  ⚠️  Automatic rollback triggered!");
                        break;
                    }
                }
                Err(_e) => {
                    let latency = timer.elapsed();
                    metrics.record_error(
                        pmat::claude_integration::observability::ErrorType::Application,
                        latency,
                    );
                }
            }
        }
    }

    // 5. Report results
    let snapshot = metrics.snapshot();
    println!("\n  Final Results:");
    println!("  {}", snapshot.format_summary());

    let pool_stats = bridge.pool_stats();
    println!(
        "  Pool: {} successes, {} failures",
        pool_stats.successes, pool_stats.failures
    );

    let cache_stats = bridge.cache_stats();
    println!(
        "  Cache: {:.1}% effective hit rate\n",
        cache_stats.effective_hit_rate * 100.0
    );

    Ok(())
}
