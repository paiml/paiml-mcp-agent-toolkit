use anyhow::Result;
use pmat::tdg::{AdaptiveThresholdFactory, ResourceLimits, SchedulerFactory, TieredStorageFactory};
use std::sync::Arc;
use tempfile;

#[tokio::main]
async fn main() -> Result<()> {
    println!("TDG System Demonstration (Simplified)");
    println!("=====================================\n");

    demo_storage_backends().await?;
    demo_scheduling().await?;
    demo_adaptive_thresholds().await?;
    demo_resource_config().await?;

    println!("All TDG system components demonstrated successfully!");
    Ok(())
}

/// Demonstrate storage backends
async fn demo_storage_backends() -> Result<()> {
    println!("1. Storage Backends");
    println!("------------------");

    let temp_dir = tempfile::TempDir::new()?;
    let storage = TieredStorageFactory::create_at_path(temp_dir.path())?;

    println!("✓ Tiered storage created successfully");
    println!("✓ Storage backend configured\n");

    Ok(())
}

/// Demonstrate scheduling
async fn demo_scheduling() -> Result<()> {
    println!("2. Fair Scheduling");
    println!("-----------------");

    let scheduler = Arc::new(SchedulerFactory::create_balanced());
    println!("✓ Fair scheduler created successfully");
    println!("✓ Scheduler ready for operations\n");

    Ok(())
}

/// Demonstrate adaptive thresholds
async fn demo_adaptive_thresholds() -> Result<()> {
    println!("3. Adaptive Thresholds");
    println!("---------------------");

    let _adaptive = AdaptiveThresholdFactory::create_default();
    println!("✓ Adaptive threshold manager created");
    println!("✓ Threshold management ready\n");

    Ok(())
}

/// Demonstrate resource configuration
async fn demo_resource_config() -> Result<()> {
    println!("4. Resource Configuration");
    println!("------------------------");

    let _limits = ResourceLimits {
        max_memory_mb: 500.0,
        max_cpu_utilization: 0.8,
        max_concurrent_ops: 10,
        memory_warning_threshold: 0.7,
        cpu_warning_threshold: 0.7,
        check_interval_secs: 5,
    };

    println!("✓ Resource limits configured");
    println!("✓ System ready for resource control\n");

    Ok(())
}
