//! Cache management handlers for O(1) hooks cache (PMAT-453)

#![cfg_attr(coverage_nightly, coverage(off))]

use crate::cli::colors as c;
use crate::cli::commands::HooksCacheAction;
use crate::cli::OutputFormat;
use crate::tdg::hooks_cache::{CacheCheckResult, HooksCacheManager};
use anyhow::Result;

/// Handle hooks cache subcommand
pub(super) async fn handle_cache(action: &HooksCacheAction) -> Result<()> {
    let project_root = std::env::current_dir()?;
    let manager = HooksCacheManager::new(&project_root);

    match action {
        HooksCacheAction::Init => handle_cache_init(&manager).await,
        HooksCacheAction::Status { format } => handle_cache_status(&manager, format).await,
        HooksCacheAction::Clear { gate } => handle_cache_clear(&manager, gate.as_deref()).await,
        HooksCacheAction::Metrics { format } => handle_cache_metrics(&manager, format).await,
    }
}

/// Initialize cache directory structure
async fn handle_cache_init(manager: &HooksCacheManager) -> Result<()> {
    println!("{}", c::label("Initializing hooks cache..."));

    manager.init()?;

    println!("{}", c::pass("Cache directory structure created:"));
    println!("   {}", c::path(".pmat/hooks-cache/"));
    println!(
        "   {} tree-hash.json    {}",
        c::dim("├──"),
        c::dim("(Level 0: repo-wide cache)")
    );
    println!(
        "   {} gates/            {}",
        c::dim("├──"),
        c::dim("(Level 1: per-gate cache)")
    );
    println!(
        "   {} files/            {}",
        c::dim("├──"),
        c::dim("(Level 2: per-file cache)")
    );
    println!(
        "   {} metrics.json      {}",
        c::dim("└──"),
        c::dim("(CB-021: health monitoring)")
    );

    Ok(())
}

/// Show cache status and check result
async fn handle_cache_status(manager: &HooksCacheManager, format: &OutputFormat) -> Result<()> {
    let check_result = manager.check()?;
    let metrics = manager.get_metrics().unwrap_or_default();
    let hit_rate = manager.hit_rate().unwrap_or(0.0);

    match format {
        OutputFormat::Json | OutputFormat::Yaml => {
            let status = serde_json::json!({
                "cache_status": match &check_result {
                    CacheCheckResult::Hit { result, cached_at } => serde_json::json!({
                        "type": "hit",
                        "result": format!("{:?}", result),
                        "cached_at": cached_at.to_rfc3339()
                    }),
                    CacheCheckResult::Miss { reason } => serde_json::json!({
                        "type": "miss",
                        "reason": reason.to_string()
                    }),
                    CacheCheckResult::Partial { cached_gates, uncached_gates } => serde_json::json!({
                        "type": "partial",
                        "cached_gates": cached_gates,
                        "uncached_gates": uncached_gates
                    }),
                },
                "metrics": {
                    "total_runs": metrics.total_runs,
                    "cache_hits": metrics.cache_hits,
                    "cache_misses": metrics.cache_misses,
                    "hit_rate": hit_rate,
                    "avg_hit_time_ms": metrics.avg_cache_hit_time_ms,
                    "avg_miss_time_ms": metrics.avg_cache_miss_time_ms,
                    "cache_size_bytes": metrics.cache_size_bytes
                }
            });
            println!("{}", serde_json::to_string_pretty(&status)?);
        }
        _ => {
            println!("{}", c::header("Hooks Cache Status"));
            println!("{}", c::rule());
            println!();

            match &check_result {
                CacheCheckResult::Hit { result, cached_at } => {
                    println!(
                        "{} Cache Status: {}HIT{}",
                        c::pass(""),
                        c::BOLD_GREEN,
                        c::RESET
                    );
                    println!("   {}: {:?}", c::dim("Result"), result);
                    println!(
                        "   {}: {}",
                        c::dim("Cached at"),
                        cached_at.format("%Y-%m-%d %H:%M:%S UTC")
                    );
                    println!();
                    println!(
                        "   {}",
                        c::pass("O(1) skip available - no full analysis needed")
                    );
                }
                CacheCheckResult::Miss { reason } => {
                    println!(
                        "{} Cache Status: {}MISS{}",
                        c::fail(""),
                        c::BOLD_RED,
                        c::RESET
                    );
                    println!("   {}: {}", c::dim("Reason"), reason);
                    println!();
                    println!("   {}", c::dim("Full analysis required on next hook run"));
                }
                CacheCheckResult::Partial {
                    cached_gates,
                    uncached_gates,
                } => {
                    println!(
                        "{} Cache Status: {}PARTIAL{}",
                        c::warn(""),
                        c::BOLD_YELLOW,
                        c::RESET
                    );
                    println!("   {}: {}", c::dim("Cached gates"), cached_gates.join(", "));
                    println!(
                        "   {}: {}",
                        c::dim("Uncached gates"),
                        uncached_gates.join(", ")
                    );
                }
            }

            println!();
            println!("{}", c::subheader("Metrics:"));
            println!(
                "   {}: {}",
                c::dim("Total runs"),
                c::number(&metrics.total_runs.to_string())
            );
            println!(
                "   {}: {}",
                c::dim("Hit rate"),
                c::pct(hit_rate * 100.0, 60.0, 30.0)
            );
            println!(
                "   {}: {:.1}ms",
                c::dim("Avg hit time"),
                metrics.avg_cache_hit_time_ms
            );
            println!(
                "   {}: {:.1}ms",
                c::dim("Avg miss time"),
                metrics.avg_cache_miss_time_ms
            );
            println!(
                "   {}: {} bytes",
                c::dim("Cache size"),
                c::number(&metrics.cache_size_bytes.to_string())
            );
        }
    }

    Ok(())
}

/// Clear cache
async fn handle_cache_clear(manager: &HooksCacheManager, gate: Option<&str>) -> Result<()> {
    if let Some(gate_name) = gate {
        println!(
            "{} Clearing cache for gate: {}",
            c::label(""),
            c::label(gate_name)
        );
        manager.clear_gate(gate_name)?;
        println!("{}", c::pass("Gate cache cleared"));
    } else {
        println!("{}", c::label("Clearing all hooks cache..."));
        manager.clear()?;
        println!(
            "{}",
            c::pass("All cache cleared - next commit will run full analysis")
        );
    }

    Ok(())
}

/// Show detailed metrics
async fn handle_cache_metrics(manager: &HooksCacheManager, format: &OutputFormat) -> Result<()> {
    let metrics = manager.get_metrics()?;
    let hit_rate = manager.hit_rate()?;
    let is_healthy = manager.is_healthy()?;

    match format {
        OutputFormat::Json | OutputFormat::Yaml => {
            let output = serde_json::json!({
                "total_runs": metrics.total_runs,
                "cache_hits": metrics.cache_hits,
                "cache_misses": metrics.cache_misses,
                "hit_rate": hit_rate,
                "avg_cache_hit_time_ms": metrics.avg_cache_hit_time_ms,
                "avg_cache_miss_time_ms": metrics.avg_cache_miss_time_ms,
                "cache_size_bytes": metrics.cache_size_bytes,
                "last_full_rebuild": metrics.last_full_rebuild.map(|t| t.to_rfc3339()),
                "health_status": if is_healthy { "healthy" } else { "degraded" }
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            println!("{}", c::header("Hooks Cache Metrics (CB-021)"));
            println!("{}", c::rule());
            println!();
            println!(
                "{}: {}",
                c::label("Health Status"),
                if is_healthy {
                    c::pass("Healthy")
                } else {
                    c::warn("Degraded")
                }
            );
            println!();
            println!("{}", c::subheader("Performance:"));
            println!(
                "   {}: {}",
                c::dim("Total runs"),
                c::number(&metrics.total_runs.to_string())
            );
            println!(
                "   {}: {}",
                c::dim("Cache hits"),
                c::number(&metrics.cache_hits.to_string())
            );
            println!(
                "   {}: {}",
                c::dim("Cache misses"),
                c::number(&metrics.cache_misses.to_string())
            );
            println!(
                "   {}: {}",
                c::dim("Hit rate"),
                c::pct(hit_rate * 100.0, 60.0, 30.0)
            );
            println!();
            println!("{}", c::subheader("Timing:"));
            println!(
                "   {}: {:.2}ms {}",
                c::dim("Avg cache hit"),
                metrics.avg_cache_hit_time_ms,
                c::dim("(target: <5ms)")
            );
            println!(
                "   {}: {:.2}ms",
                c::dim("Avg cache miss"),
                metrics.avg_cache_miss_time_ms
            );
            if let Some(last_rebuild) = metrics.last_full_rebuild {
                println!(
                    "   {}: {}",
                    c::dim("Last full rebuild"),
                    last_rebuild.format("%Y-%m-%d %H:%M:%S UTC")
                );
            }
            println!();
            println!("{}", c::subheader("Storage:"));
            println!(
                "   {}: {} bytes",
                c::dim("Cache size"),
                c::number(&metrics.cache_size_bytes.to_string())
            );

            // Show health recommendation if degraded
            if !is_healthy {
                println!();
                println!("{}", c::warn("Cache health is degraded (hit rate < 60%)"));
                println!(
                    "   {}",
                    c::dim("Consider running 'pmat hooks cache clear' to reset")
                );
            }
        }
    }

    Ok(())
}
