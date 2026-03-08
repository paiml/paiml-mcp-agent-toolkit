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
    use crate::cli::colors as c;

    let manager = global_memory_manager()?;

    if verbose {
        let stats_before = manager.stats();
        println!("{}", c::subheader("Memory before cleanup:"));
        println!(
            "  {}: {}",
            c::label("Allocated"),
            c::number(&format_bytes(stats_before.total_allocated))
        );
        println!(
            "  {}: {}",
            c::label("Pressure"),
            c::pct_inverse(stats_before.allocation_pressure * 100.0, 30.0, 60.0)
        );
        println!();
    }

    let cleaned = manager.cleanup()?;

    if verbose {
        let stats_after = manager.stats();
        println!("{}", c::subheader("Memory after cleanup:"));
        println!("  {}: {}", c::label("Allocated"), c::number(&format_bytes(stats_after.total_allocated)));
        println!(
            "  {}: {}",
            c::label("Pressure"),
            c::pct_inverse(stats_after.allocation_pressure * 100.0, 30.0, 60.0)
        );
        println!("  {}: {}", c::label("Cleaned"), c::number(&format_bytes(cleaned)));

        if stats_after.allocation_pressure <= target_pressure {
            println!("{}", c::pass("Target pressure achieved"));
        } else {
            println!("{}", c::warn("Target pressure not reached. Consider reducing workload."));
        }
    } else {
        println!("Cleaned {} of memory", c::number(&format_bytes(cleaned)));
    }

    Ok(())
}

async fn handle_memory_configure(
    max_memory_mb: &Option<usize>,
    pool_limits: &[String],
    enable_tracking: &Option<bool>,
) -> Result<()> {
    use crate::cli::colors as c;

    println!("{}", c::subheader("Memory configuration:"));

    if let Some(max_mb) = max_memory_mb {
        println!("  {}: {} MB", c::label("Maximum memory"), c::number(&max_mb.to_string()));
        println!("  {}", c::dim("Note: Runtime reconfiguration not yet supported"));
    }

    if !pool_limits.is_empty() {
        println!("  {}:", c::label("Pool limits"));
        for limit_spec in pool_limits {
            println!("    {limit_spec}");
        }
        println!("  {}", c::dim("Note: Runtime pool reconfiguration not yet supported"));
    }

    if let Some(tracking) = enable_tracking {
        println!(
            "  {}: {}",
            c::label("Memory tracking"),
            if *tracking {
                format!("{}enabled{}", c::GREEN, c::RESET)
            } else {
                format!("{}disabled{}", c::RED, c::RESET)
            }
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
    use crate::cli::colors as c;

    println!("{}", c::header("Memory Pool Statistics"));
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
    use crate::cli::colors as c;

    println!("{}:", c::label(pool_name));
    println!("  {}: {}", c::label("Buffers"), c::number(&pool_stats.buffer_count.to_string()));
    println!("  {}: {}", c::label("Total Size"), c::number(&format_bytes(pool_stats.total_size)));
    println!("  {}: {}", c::label("Allocations"), c::number(&pool_stats.allocation_count.to_string()));
    println!("  {}: {}", c::label("Reuses"), c::number(&pool_stats.reuse_count.to_string()));
}

/// Print pool efficiency statistics
fn print_pool_efficiency_stats(pool_stats: &crate::services::memory_manager::PoolStats) {
    use crate::cli::colors as c;

    println!("  {}: {}", c::label("Reuse Ratio"), c::pct(pool_stats.reuse_ratio * 100.0, 80.0, 60.0));

    let avg_buffer_size = calculate_average_buffer_size(pool_stats);
    println!("  {}: {}", c::label("Avg Buffer"), c::number(&format_bytes(avg_buffer_size)));

    let efficiency_rating = calculate_pool_efficiency_rating(pool_stats.reuse_ratio);
    let colored_rating = match efficiency_rating {
        "Excellent" | "Good" => format!("{}{}{}", c::GREEN, efficiency_rating, c::RESET),
        "Fair" => format!("{}{}{}", c::YELLOW, efficiency_rating, c::RESET),
        _ => format!("{}{}{}", c::RED, efficiency_rating, c::RESET),
    };
    println!("  {}: {}", c::label("Efficiency"), colored_rating);
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
    use crate::cli::colors as c;

    let manager = global_memory_manager()?;

    if let Some(interval) = watch {
        println!(
            "Monitoring memory pressure ({}: {}, {}: {}s)",
            c::label("threshold"),
            c::pct_inverse(threshold * 100.0, 30.0, 60.0),
            c::label("interval"),
            c::number(&interval.to_string())
        );
        println!("{}", c::dim("Press Ctrl+C to stop"));
        println!();

        loop {
            let stats = manager.stats();
            let timestamp = chrono::Utc::now().format("%H:%M:%S");

            let pressure_pct = stats.allocation_pressure * 100.0;

            println!(
                "[{}] {}: {} | {}: {}",
                c::dim(&timestamp.to_string()),
                c::label("Pressure"),
                c::pct_inverse(pressure_pct, 30.0, 60.0),
                c::label("Allocated"),
                c::number(&format_bytes(stats.total_allocated))
            );

            if stats.allocation_pressure > threshold {
                println!("  {}", c::warn("Memory pressure above threshold!"));
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(*interval)).await;
        }
    } else {
        let stats = manager.stats();

        println!(
            "{}: {}",
            c::label("Current memory pressure"),
            c::pct_inverse(stats.allocation_pressure * 100.0, 30.0, 60.0)
        );
        println!(
            "{}: {}",
            c::label("Threshold"),
            c::pct_inverse(threshold * 100.0, 30.0, 60.0)
        );

        if stats.allocation_pressure > threshold {
            println!("{}: {}", c::label("Status"), format!("{}WARNING - Above threshold{}", c::YELLOW, c::RESET));
            println!("{}", c::dim("Recommendation: Consider running 'pmat memory cleanup'"));
        } else {
            println!("{}: {}", c::label("Status"), format!("{}OK - Below threshold{}", c::GREEN, c::RESET));
        }
    }

    Ok(())
}
