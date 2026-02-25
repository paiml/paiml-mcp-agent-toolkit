// memory_operations.rs - Async handler implementations for memory commands
// Included by memory.rs via include!() - shares parent module scope

async fn handle_memory_stats(detailed: bool, format: &str) -> Result<()> {
    let manager = global_memory_manager()?;
    let stats = manager.stats();

    let pool_stats_output = build_pool_stats_output(&stats.pool_stats);
    let recommendations = generate_memory_recommendations(&stats);

    let output = MemoryStatsOutput {
        total_allocated: stats.total_allocated,
        peak_usage: stats.peak_usage,
        allocation_pressure: stats.allocation_pressure,
        string_intern_size: stats.string_intern_size,
        pool_stats: pool_stats_output,
        recommendations,
    };

    output_memory_stats(&output, format, detailed)
}

async fn handle_memory_cleanup(target_pressure: f64, verbose: bool) -> Result<()> {
    let manager = global_memory_manager()?;

    if verbose {
        let stats_before = manager.stats();
        println!("Memory before cleanup:");
        println!(
            "  Allocated: {}",
            format_bytes(stats_before.total_allocated)
        );
        println!(
            "  Pressure:  {:.1}%",
            stats_before.allocation_pressure * 100.0
        );
        println!();
    }

    let cleaned = manager.cleanup()?;

    if verbose {
        let stats_after = manager.stats();
        println!("Memory after cleanup:");
        println!("  Allocated: {}", format_bytes(stats_after.total_allocated));
        println!(
            "  Pressure:  {:.1}%",
            stats_after.allocation_pressure * 100.0
        );
        println!("  Cleaned:   {}", format_bytes(cleaned));

        if stats_after.allocation_pressure <= target_pressure {
            println!("✓ Target pressure achieved");
        } else {
            println!("⚠ Target pressure not reached. Consider reducing workload.");
        }
    } else {
        println!("Cleaned {} of memory", format_bytes(cleaned));
    }

    Ok(())
}

async fn handle_memory_configure(
    max_memory_mb: &Option<usize>,
    pool_limits: &[String],
    enable_tracking: &Option<bool>,
) -> Result<()> {
    println!("Memory configuration:");

    if let Some(max_mb) = max_memory_mb {
        println!("  Maximum memory: {max_mb} MB");
        // Note: Current implementation doesn't support runtime reconfiguration
        println!("  Note: Runtime reconfiguration not yet supported");
    }

    if !pool_limits.is_empty() {
        println!("  Pool limits:");
        for limit_spec in pool_limits {
            println!("    {limit_spec}");
        }
        println!("  Note: Runtime pool reconfiguration not yet supported");
    }

    if let Some(tracking) = enable_tracking {
        println!(
            "  Memory tracking: {}",
            if *tracking { "enabled" } else { "disabled" }
        );
    }

    Ok(())
}

/// Handle memory pools command
async fn handle_memory_pools(pool: &Option<String>, efficiency: bool) -> Result<()> {
    let manager = global_memory_manager()?;
    let stats = manager.stats();

    print_pool_statistics_header();

    for (pool_type, pool_stats) in &stats.pool_stats {
        let pool_name = format!("{pool_type:?}");

        if should_skip_pool(&pool_name, pool) {
            continue;
        }

        print_pool_basic_stats(&pool_name, pool_stats);

        if efficiency {
            print_pool_efficiency_stats(pool_stats);
        }

        println!();
    }

    Ok(())
}

/// Print header for pool statistics
fn print_pool_statistics_header() {
    println!("Memory Pool Statistics");
    println!();
}

/// Check if pool should be skipped based on filter
fn should_skip_pool(pool_name: &str, target_pool: &Option<String>) -> bool {
    if let Some(target) = target_pool {
        !pool_name.to_lowercase().contains(&target.to_lowercase())
    } else {
        false
    }
}

/// Print basic pool statistics
fn print_pool_basic_stats(
    pool_name: &str,
    pool_stats: &crate::services::memory_manager::PoolStats,
) {
    println!("{}:", pool_name);
    println!("  Buffers:     {}", pool_stats.buffer_count);
    println!("  Total Size:  {}", format_bytes(pool_stats.total_size));
    println!("  Allocations: {}", pool_stats.allocation_count);
    println!("  Reuses:      {}", pool_stats.reuse_count);
}

/// Print pool efficiency statistics
fn print_pool_efficiency_stats(pool_stats: &crate::services::memory_manager::PoolStats) {
    println!("  Reuse Ratio: {:.1}%", pool_stats.reuse_ratio * 100.0);

    let avg_buffer_size = calculate_average_buffer_size(pool_stats);
    println!("  Avg Buffer:  {}", format_bytes(avg_buffer_size));

    let efficiency_rating = calculate_pool_efficiency_rating(pool_stats.reuse_ratio);
    println!("  Efficiency:  {efficiency_rating}");
}

/// Calculate average buffer size for pool
fn calculate_average_buffer_size(pool_stats: &crate::services::memory_manager::PoolStats) -> usize {
    if pool_stats.buffer_count > 0 {
        pool_stats.total_size / pool_stats.buffer_count
    } else {
        0
    }
}

async fn handle_memory_pressure(threshold: f64, watch: &Option<u64>) -> Result<()> {
    let manager = global_memory_manager()?;

    if let Some(interval) = watch {
        println!(
            "Monitoring memory pressure (threshold: {:.1}%, interval: {}s)",
            threshold * 100.0,
            interval
        );
        println!("Press Ctrl+C to stop");
        println!();

        loop {
            let stats = manager.stats();
            let timestamp = chrono::Utc::now().format("%H:%M:%S");

            let pressure_str = format!("{:.1}%", stats.allocation_pressure * 100.0);

            println!(
                "[{}] Pressure: {} | Allocated: {}",
                timestamp,
                pressure_str,
                format_bytes(stats.total_allocated)
            );

            if stats.allocation_pressure > threshold {
                println!("  ⚠ Warning: Memory pressure above threshold!");
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(*interval)).await;
        }
    } else {
        let stats = manager.stats();

        println!(
            "Current memory pressure: {:.1}%",
            stats.allocation_pressure * 100.0
        );
        println!("Threshold:               {:.1}%", threshold * 100.0);

        if stats.allocation_pressure > threshold {
            println!("Status: WARNING - Above threshold");
            println!("Recommendation: Consider running 'pmat memory cleanup'");
        } else {
            println!("Status: OK - Below threshold");
        }
    }

    Ok(())
}
