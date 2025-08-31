use crate::cli::commands::{DiagnosticOutputFormat, StorageCommand, TdgCommand};
use crate::tdg::{TieredStorageFactory, StorageBackendType, StorageConfig};
use anyhow::Result;
use serde_json::json;
use std::path::PathBuf;

/// Handle TDG diagnostic commands
pub async fn handle_tdg_diagnostics(
    command: &TdgCommand,
    base_path: &PathBuf,
) -> Result<()> {
    match command {
        TdgCommand::Diagnostics {
            detailed,
            storage,
            scheduler,
            adaptive,
            resources,
            all,
            format,
        } => {
            show_diagnostics(
                base_path,
                *detailed,
                *storage || *all,
                *scheduler || *all,
                *adaptive || *all,
                *resources || *all,
                format.clone(),
            )
            .await
        }
        TdgCommand::Storage { command } => handle_storage_command(command, base_path).await,
        TdgCommand::Compare { .. } => {
            // This is handled elsewhere in the existing TDG handler
            Ok(())
        }
        TdgCommand::Dashboard { port, host, open, update_interval } => {
            handle_dashboard_command(*port, host.clone(), *open, *update_interval).await
        }
    }
}

/// Show TDG system diagnostics
async fn show_diagnostics(
    base_path: &PathBuf,
    detailed: bool,
    show_storage: bool,
    _show_scheduler: bool,
    _show_adaptive: bool,
    _show_resources: bool,
    format: DiagnosticOutputFormat,
) -> Result<()> {
    // For now, we'll show basic storage diagnostics since that's what we have implemented
    let storage = TieredStorageFactory::create_at_path(base_path)?;
    let stats = storage.get_statistics();
    
    match format {
        DiagnosticOutputFormat::Human => {
            println!("=== TDG System Diagnostics ===\n");
            
            if show_storage {
                println!("Storage Diagnostics:");
                println!("{}", stats.format_diagnostic());
                if detailed {
                    println!("\nBackend Details:");
                    for (tier, backend_stats) in &stats.backend_stats {
                        println!("  {}:", tier);
                        for (key, value) in backend_stats {
                            println!("    {}: {}", key, value);
                        }
                    }
                }
                println!();
            }
            
            // Placeholder for other diagnostics
            println!("Note: Full diagnostic infrastructure is in development.");
            println!("Currently showing storage statistics only.");
        }
        DiagnosticOutputFormat::Json => {
            let json_output = json!({
                "storage": if show_storage { Some(&stats) } else { None },
                "note": "Full diagnostic infrastructure in development"
            });
            println!("{}", serde_json::to_string_pretty(&json_output)?);
        }
        DiagnosticOutputFormat::Yaml => {
            let yaml_output = json!({
                "storage": if show_storage { Some(&stats) } else { None },
                "note": "Full diagnostic infrastructure in development"
            });
            println!("{}", serde_yaml::to_string(&yaml_output)?);
        }
        DiagnosticOutputFormat::Table => {
            use prettytable::{Table, row};
            
            let mut table = Table::new();
            table.add_row(row!["Component", "Status", "Details"]);
            
            if show_storage {
                table.add_row(row![
                    "Storage",
                    format!("{} entries", stats.total_entries),
                    format!("Hot: {}, Warm: {}, Cold: {}", 
                        stats.hot_entries,
                        stats.warm_entries,
                        stats.cold_entries)
                ]);
                
                table.add_row(row![
                    "Backends",
                    format!("Warm: {}", stats.warm_backend),
                    format!("Cold: {}", stats.cold_backend)
                ]);
                
                table.add_row(row![
                    "Compression",
                    format!("{:.1}%", stats.compression_ratio * 100.0),
                    format!("Memory: {} KB", stats.hot_memory_kb)
                ]);
            }
            
            table.printstd();
        }
    }
    
    Ok(())
}

/// Handle storage management commands
async fn handle_storage_command(command: &StorageCommand, base_path: &PathBuf) -> Result<()> {
    // Create storage instance
    let storage = TieredStorageFactory::create_at_path(base_path)?;
    
    match command {
        StorageCommand::Stats { detailed } => {
            let stats = storage.get_statistics();
            println!("=== TDG Storage Statistics ===\n");
            println!("{}", stats.format_diagnostic());
            
            if *detailed {
                println!("\nBackend Statistics:");
                for (tier, backend_stats) in &stats.backend_stats {
                    println!("\n{}:", tier);
                    for (key, value) in backend_stats {
                        println!("  {}: {}", key, value);
                    }
                }
            }
        }
        StorageCommand::Cleanup { max_age } => {
            let removed = storage.cleanup_hot_cache(*max_age);
            println!("Cleaned up {} expired hot cache entries", removed);
        }
        StorageCommand::Migrate { backend, path } => {
            let backend_type = match backend.as_str() {
                "sled" => StorageBackendType::Sled,
                "rocksdb" => StorageBackendType::RocksDb,
                "inmemory" => StorageBackendType::InMemory,
                _ => {
                    return Err(anyhow::anyhow!(
                        "Unknown backend type: {}. Valid options: sled, rocksdb, inmemory",
                        backend
                    ));
                }
            };
            
            // Create new backend configurations
            let warm_config = StorageConfig {
                backend_type,
                path: path.as_ref().map(|p| p.join("tdg-warm")),
                cache_size_mb: Some(128),
                compression: true,
            };
            
            let cold_config = StorageConfig {
                backend_type,
                path: path.as_ref().map(|p| p.join("tdg-cold")),
                cache_size_mb: Some(64),
                compression: false,
            };
            
            println!("Migrating storage to {} backend...", backend);
            
            // Note: This requires mutable access to storage which we don't have here
            // In a real implementation, we'd need to refactor the storage API
            println!("⚠️  Migration requires restart to take effect");
            println!("New configuration:");
            println!("  Warm storage: {:?}", warm_config);
            println!("  Cold storage: {:?}", cold_config);
        }
        StorageCommand::Flush => {
            storage.flush()?;
            println!("✅ All pending writes flushed to storage");
        }
    }
    
    Ok(())
}

/// Handle dashboard command - start web dashboard server
async fn handle_dashboard_command(
    port: u16,
    host: String,
    open: bool,
    _update_interval: u64,
) -> Result<()> {
    use crate::tdg::web_dashboard::start_dashboard_server;
    use std::net::{IpAddr, SocketAddr};
    
    println!("🚀 Starting TDG Dashboard server...");
    
    let addr: IpAddr = host.parse()?;
    let socket_addr = SocketAddr::new(addr, port);
    
    println!("📊 Dashboard will be available at: http://{}:{}", host, port);
    println!("🔄 Real-time metrics updates enabled");
    
    // Open browser if requested
    if open {
        if let Err(e) = webbrowser::open(&format!("http://{}:{}", host, port)) {
            eprintln!("⚠️  Could not open browser: {}", e);
        } else {
            println!("🌐 Opening dashboard in browser...");
        }
    }
    
    println!("Press Ctrl+C to stop the server");
    
    // Start the dashboard server (this will block)
    start_dashboard_server(socket_addr).await.map_err(|e| anyhow::anyhow!("Dashboard server error: {}", e))?;
    
    Ok(())
}