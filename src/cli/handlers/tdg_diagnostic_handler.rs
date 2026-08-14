use crate::cli::colors as c;
use crate::cli::commands::{DiagnosticOutputFormat, StorageCommand, TdgCommand};
use crate::tdg::{StorageBackendType, StorageConfig, TieredStorageFactory, TieredStore};
use anyhow::Result;
use serde_json::json;
use std::path::PathBuf;

#[cfg(feature = "http-server")]
/// Open URL in default browser using platform-specific command
/// Replaces webbrowser crate to reduce transitive dependencies
fn open_browser(url: &str) -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(url).spawn()?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(url).spawn()?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/c", "start", url])
            .spawn()?;
    }
    Ok(())
}

/// Handle TDG diagnostic commands
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_tdg_diagnostics(command: &TdgCommand, base_path: &PathBuf) -> Result<()> {
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
        TdgCommand::Compare { .. }
        | TdgCommand::History { .. }
        | TdgCommand::Baseline { .. }
        | TdgCommand::CheckRegression { .. }
        | TdgCommand::CheckQuality { .. } => {
            // These are handled elsewhere in the existing TDG handler
            Ok(())
        }
        #[cfg(feature = "http-server")]
        TdgCommand::Dashboard {
            port,
            host,
            open,
            update_interval,
        } => handle_dashboard_command(*port, host.clone(), *open, *update_interval).await,
        #[cfg(not(feature = "http-server"))]
        TdgCommand::Dashboard { .. } => {
            anyhow::bail!("Dashboard requires the 'http-server' feature. Rebuild with: cargo build --features http-server")
        }
        TdgCommand::Config(config_cmd) => {
            super::config_command_handlers::handle_config_command(config_cmd).await
        }
    }
}

/// Whether the storage section should be rendered for this flag combination.
///
/// A bare `pmat tdg diagnostics` used to `return Ok(())` before printing
/// anything — exit 0 with zero bytes on stdout and stderr, from a command whose
/// help promises "TDG system diagnostics and health status". With no section
/// selected the caller means "whatever you have", and storage is the only
/// section that exists.
fn should_show_storage(storage: bool, scheduler: bool, adaptive: bool, resources: bool) -> bool {
    storage || !scheduler && !adaptive && !resources
}

/// Sections `tdg diagnostics` advertises but has no implementation for.
///
/// `--scheduler`, `--adaptive` and `--resources` were underscore-prefixed
/// unused parameters: three documented flags that were compiled-in no-ops.
/// Naming them as unavailable is the only honest answer while that is true.
fn unavailable_sections(scheduler: bool, adaptive: bool, resources: bool) -> Vec<&'static str> {
    let mut out = Vec::new();
    if scheduler {
        out.push("scheduler");
    }
    if adaptive {
        out.push("adaptive-thresholds");
    }
    if resources {
        out.push("resources");
    }
    out
}

/// Show TDG system diagnostics
async fn show_diagnostics(
    base_path: &PathBuf,
    detailed: bool,
    show_storage: bool,
    show_scheduler: bool,
    show_adaptive: bool,
    show_resources: bool,
    format: DiagnosticOutputFormat,
) -> Result<()> {
    let show_storage =
        should_show_storage(show_storage, show_scheduler, show_adaptive, show_resources);
    let unavailable = unavailable_sections(show_scheduler, show_adaptive, show_resources);

    let structured = matches!(
        format,
        DiagnosticOutputFormat::Json | DiagnosticOutputFormat::Yaml
    );

    // Opening the tiered store creates .pmat/tdg-*.db, so only do it when the
    // storage section is actually going to be reported.
    if show_storage || structured {
        let storage = TieredStorageFactory::create_at_path(base_path)?;
        let stats = storage.get_statistics();

        match format {
            DiagnosticOutputFormat::Plain => show_plain_diagnostics(&stats, detailed),
            DiagnosticOutputFormat::Human => show_human_diagnostics(&stats, detailed),
            DiagnosticOutputFormat::Json => {
                show_json_diagnostics(&stats, show_storage, &unavailable)?;
            }
            DiagnosticOutputFormat::Yaml => {
                show_yaml_diagnostics(&stats, show_storage, &unavailable)?;
            }
            DiagnosticOutputFormat::Table => show_table_diagnostics(&stats),
        }
    }

    if !unavailable.is_empty()
        && matches!(
            format,
            DiagnosticOutputFormat::Plain
                | DiagnosticOutputFormat::Human
                | DiagnosticOutputFormat::Table
        )
    {
        println!(
            "{}",
            c::warn(&format!(
                "Requested but not implemented: {}. These flags have no diagnostic behind them yet.",
                unavailable.join(", ")
            ))
        );
    }

    Ok(())
}

fn show_plain_diagnostics(stats: &crate::tdg::storage::StorageStatistics, detailed: bool) {
    println!("{}\n", c::header("TDG System Diagnostics"));
    print_basic_storage_info(stats);

    if detailed {
        show_backend_details(&stats.backend_stats);
    }
}

fn show_human_diagnostics(stats: &crate::tdg::storage::StorageStatistics, detailed: bool) {
    println!("{}\n", c::header("TDG System Diagnostics"));
    println!("{}", c::subheader("Storage Diagnostics:"));
    println!("{}", stats.format_diagnostic());

    if detailed {
        show_human_backend_details(&stats.backend_stats);
    }

    println!();
    println!(
        "{}",
        c::dim("Note: Full diagnostic infrastructure is in development.")
    );
    println!("{}", c::dim("Currently showing storage statistics only."));
}

/// The storage section, with every field that carries `null` named.
///
/// `tdg storage stats` says "Compression ratio: not measured (nothing stored)"
/// while this section reported `"compression_ratio": 0.0` for the same store.
/// The `null` alone is not enough to close that gap — a reader must not have to
/// guess why it is null, so the unmeasured fields are listed the way `analyze
/// tdg` lists `average_score`/`average_grade`.
fn storage_section(stats: &crate::tdg::storage::StorageStatistics) -> Result<serde_json::Value> {
    let mut value = serde_json::to_value(stats)?;
    if let Some(object) = value.as_object_mut() {
        object.insert("not_measured".to_string(), json!(stats.not_measured()));
    }
    Ok(value)
}

fn show_json_diagnostics(
    stats: &crate::tdg::storage::StorageStatistics,
    show_storage: bool,
    unavailable: &[&str],
) -> Result<()> {
    let storage = if show_storage {
        Some(storage_section(stats)?)
    } else {
        None
    };
    let json_output = json!({
        "storage": storage,
        "unavailable_sections": unavailable,
        "note": "Full diagnostic infrastructure in development"
    });
    println!("{}", serde_json::to_string_pretty(&json_output)?);
    Ok(())
}

fn show_yaml_diagnostics(
    stats: &crate::tdg::storage::StorageStatistics,
    show_storage: bool,
    unavailable: &[&str],
) -> Result<()> {
    let storage = if show_storage {
        Some(storage_section(stats)?)
    } else {
        None
    };
    let yaml_output = json!({
        "storage": storage,
        "unavailable_sections": unavailable,
        "note": "Full diagnostic infrastructure in development"
    });
    println!("{}", serde_yaml_ng::to_string(&yaml_output)?);
    Ok(())
}

fn show_table_diagnostics(stats: &crate::tdg::storage::StorageStatistics) {
    println!("┌─────────────┬─────────────────────┬─────────────────────────────────┐");
    println!("│ Component   │ Status              │ Details                         │");
    println!("├─────────────┼─────────────────────┼─────────────────────────────────┤");
    println!(
        "│ Storage     │ {:>17} │ Hot: {}, Warm: {}, Cold: {} │",
        format!("{} entries", stats.total_entries),
        stats.hot_entries,
        stats.warm_entries,
        stats.cold_entries
    );
    println!(
        "│ Backends    │ Warm: {:>12} │ Cold: {:>24} │",
        stats.warm_backend, stats.cold_backend
    );
    println!(
        "│ Compression │ {:>17} │ Memory: {:>21} KB │",
        stats.compression_ratio_display(),
        stats.hot_memory_kb
    );
    println!("└─────────────┴─────────────────────┴─────────────────────────────────┘");
}

fn print_basic_storage_info(stats: &crate::tdg::storage::StorageStatistics) {
    println!(
        "{} {} entries",
        c::label("Storage:"),
        c::number(&format!("{}", stats.total_entries))
    );
    println!(
        "{} {}, {} {}, {} {}",
        c::label("Hot:"),
        c::number(&format!("{}", stats.hot_entries)),
        c::label("Warm:"),
        c::number(&format!("{}", stats.warm_entries)),
        c::label("Cold:"),
        c::number(&format!("{}", stats.cold_entries))
    );
    println!(
        "{} Warm: {}, Cold: {}",
        c::label("Backends -"),
        stats.warm_backend,
        stats.cold_backend
    );
    println!(
        "{} {}, {} {} KB",
        c::label("Compression:"),
        c::number(&stats.compression_ratio_display()),
        c::label("Memory:"),
        c::number(&format!("{}", stats.hot_memory_kb))
    );
}

fn show_backend_details(
    backend_stats: &std::collections::HashMap<String, std::collections::HashMap<String, String>>,
) {
    println!("\n{}", c::subheader("Backend Details:"));
    for (tier, stats) in backend_stats {
        println!("{}: {stats:?}", c::label(tier));
    }
}

fn show_human_backend_details(
    backend_stats: &std::collections::HashMap<String, std::collections::HashMap<String, String>>,
) {
    println!("\n{}", c::subheader("Backend Details:"));
    for (tier, stats) in backend_stats {
        println!("  {}:", c::label(tier));
        for (key, value) in stats {
            println!("    {}: {}", c::label(key), value);
        }
    }
}

/// Handle storage management commands
async fn handle_storage_command(command: &StorageCommand, base_path: &PathBuf) -> Result<()> {
    // Create storage instance
    let storage = TieredStorageFactory::create_at_path(base_path)?;

    match command {
        StorageCommand::Stats { detailed } => handle_stats(&storage, *detailed),
        StorageCommand::Cleanup { max_age } => handle_cleanup(&storage, *max_age),
        StorageCommand::Migrate { backend, path } => handle_migrate(backend, path.as_ref()),
        StorageCommand::Flush => handle_flush(&storage),
    }
}

/// Handle stats command
fn handle_stats(storage: &TieredStore, detailed: bool) -> Result<()> {
    let stats = storage.get_statistics();
    println!("{}\n", c::header("TDG Storage Statistics"));
    println!("{}", c::rule());
    println!();

    println!("{}", c::subheader("Storage Tiers:"));
    println!(
        "   {}: {} entries, {} KB",
        c::dim("Hot (memory)"),
        c::number(&stats.hot_entries.to_string()),
        c::number(&stats.hot_memory_kb.to_string())
    );
    println!(
        "   {}: {} entries",
        c::dim(&format!("Warm ({} backend)", stats.warm_backend)),
        c::number(&stats.warm_entries.to_string())
    );
    println!(
        "   {}: {} entries",
        c::dim(&format!("Cold ({} backend)", stats.cold_backend)),
        c::number(&stats.cold_entries.to_string())
    );
    println!(
        "   {}: {}",
        c::dim("Total"),
        c::number(&stats.total_entries.to_string())
    );
    // `tdg storage stats` printed "Compression ratio: 33.0%" over a store with
    // zero entries, byte-identically before and after a full `pmat tdg .` run.
    // 0.0 means the ratio was never measured, and must not render as a number.
    println!(
        "   {}: {}",
        c::dim("Compression ratio"),
        if stats.compression_ratio > 0.0 {
            c::pct(f64::from(stats.compression_ratio) * 100.0, 50.0, 20.0)
        } else {
            c::dim(&stats.compression_ratio_display())
        }
    );
    println!();

    if detailed {
        print_backend_statistics(&stats.backend_stats);
    }
    Ok(())
}

/// Print backend statistics
fn print_backend_statistics(
    backend_stats: &std::collections::HashMap<String, std::collections::HashMap<String, String>>,
) {
    println!("\n{}", c::subheader("Backend Statistics:"));
    for (tier, backend_stats) in backend_stats {
        println!("\n{}:", c::label(tier));
        for (key, value) in backend_stats {
            println!("  {}: {}", c::label(key), value);
        }
    }
}

/// Handle cleanup command
fn handle_cleanup(storage: &TieredStore, max_age: u64) -> Result<()> {
    let removed = storage.cleanup_hot_cache(max_age);
    println!(
        "{}",
        c::pass(&format!(
            "Cleaned up {} expired hot cache entries",
            c::number(&format!("{removed}"))
        ))
    );
    Ok(())
}

/// Handle migrate command
fn handle_migrate(backend: &str, path: Option<&PathBuf>) -> Result<()> {
    let backend_type = parse_backend_type(backend)?;
    let (warm_config, cold_config) = create_migration_configs(backend_type, path);

    println!(
        "{}",
        c::dim(&format!(
            "Migrating storage to {} backend...",
            c::label(backend)
        ))
    );
    println!("{}", c::warn("Migration requires restart to take effect"));
    println!("{}", c::subheader("New configuration:"));
    println!("  {} {warm_config:?}", c::label("Warm storage:"));
    println!("  {} {cold_config:?}", c::label("Cold storage:"));

    Ok(())
}

/// Parse backend type from string
fn parse_backend_type(backend: &str) -> Result<StorageBackendType> {
    match backend {
        "libsql" => Ok(StorageBackendType::Libsql),
        "inmemory" => Ok(StorageBackendType::InMemory),
        _ => Err(anyhow::anyhow!(
            "Unknown backend type: {backend}. Valid options: libsql, inmemory"
        )),
    }
}

/// Create migration configurations
fn create_migration_configs(
    backend_type: StorageBackendType,
    path: Option<&PathBuf>,
) -> (StorageConfig, StorageConfig) {
    let warm_config = StorageConfig {
        backend_type,
        path: path.map(|p| p.join("tdg-warm")),
        cache_size_mb: Some(128),
        compression: true,
    };

    let cold_config = StorageConfig {
        backend_type,
        path: path.map(|p| p.join("tdg-cold")),
        cache_size_mb: Some(64),
        compression: false,
    };

    (warm_config, cold_config)
}

/// Handle flush command
fn handle_flush(storage: &TieredStore) -> Result<()> {
    storage.flush()?;
    println!("{}", c::pass("All pending writes flushed to storage"));
    Ok(())
}

#[cfg(feature = "http-server")]
/// Handle dashboard command - start web dashboard server
async fn handle_dashboard_command(
    port: u16,
    host: String,
    open: bool,
    _update_interval: u64,
) -> Result<()> {
    use crate::tdg::web_dashboard::start_dashboard_server;
    use std::net::{IpAddr, SocketAddr};

    println!("{}", c::header("Starting TDG Dashboard server..."));

    let addr: IpAddr = host.parse()?;
    let socket_addr = SocketAddr::new(addr, port);

    println!(
        "{} {}",
        c::label("Dashboard:"),
        c::path(&format!("http://{host}:{port}"))
    );
    println!("{}", c::pass("Real-time metrics updates enabled"));

    // Open browser if requested
    if open {
        if let Err(e) = open_browser(&format!("http://{host}:{port}")) {
            eprintln!("{}", c::warn(&format!("Could not open browser: {e}")));
        } else {
            println!("{}", c::dim("Opening dashboard in browser..."));
        }
    }

    println!("{}", c::dim("Press Ctrl+C to stop the server"));

    // Start the dashboard server (this will block)
    start_dashboard_server(socket_addr)
        .await
        .map_err(|e| anyhow::anyhow!("Dashboard server error: {e}"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn empty_stats() -> crate::tdg::storage::StorageStatistics {
        crate::tdg::storage::StorageStatistics {
            hot_entries: 0,
            warm_entries: 0,
            cold_entries: 0,
            total_entries: 0,
            hot_memory_kb: 0,
            compression_ratio: 0.0,
            warm_backend: "memory".into(),
            cold_backend: "libsql".into(),
            backend_stats: HashMap::new(),
        }
    }

    fn populated_stats() -> crate::tdg::storage::StorageStatistics {
        let mut backend = HashMap::new();
        let mut warm = HashMap::new();
        warm.insert("disk_kb".to_string(), "256".to_string());
        warm.insert("entries".to_string(), "12".to_string());
        backend.insert("warm".to_string(), warm);
        crate::tdg::storage::StorageStatistics {
            hot_entries: 50,
            warm_entries: 100,
            cold_entries: 200,
            total_entries: 350,
            hot_memory_kb: 1024,
            compression_ratio: 0.65,
            warm_backend: "libsql".into(),
            cold_backend: "libsql".into(),
            backend_stats: backend,
        }
    }

    // ── parse_backend_type ──────────────────────────────────────────────────

    #[test]
    fn test_parse_backend_type_libsql() {
        let result = parse_backend_type("libsql").unwrap();
        assert!(matches!(result, StorageBackendType::Libsql));
    }

    #[test]
    fn test_parse_backend_type_inmemory() {
        let result = parse_backend_type("inmemory").unwrap();
        assert!(matches!(result, StorageBackendType::InMemory));
    }

    #[test]
    fn test_parse_backend_type_unknown_errors() {
        let err = parse_backend_type("redis").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("Unknown backend type"));
        assert!(msg.contains("redis"));
        assert!(msg.contains("libsql"));
        assert!(msg.contains("inmemory"));
    }

    #[test]
    fn test_parse_backend_type_empty_errors() {
        assert!(parse_backend_type("").is_err());
    }

    // ── create_migration_configs ────────────────────────────────────────────

    #[test]
    fn test_create_migration_configs_with_path() {
        let path = PathBuf::from("/tmp/tdg");
        let (warm, cold) = create_migration_configs(StorageBackendType::Libsql, Some(&path));

        // Warm config: 128 MB cache, compression on, path = base/tdg-warm
        assert_eq!(warm.cache_size_mb, Some(128));
        assert!(warm.compression);
        assert_eq!(warm.path, Some(PathBuf::from("/tmp/tdg/tdg-warm")));
        assert!(matches!(warm.backend_type, StorageBackendType::Libsql));

        // Cold config: 64 MB cache, compression off, path = base/tdg-cold
        assert_eq!(cold.cache_size_mb, Some(64));
        assert!(!cold.compression);
        assert_eq!(cold.path, Some(PathBuf::from("/tmp/tdg/tdg-cold")));
        assert!(matches!(cold.backend_type, StorageBackendType::Libsql));
    }

    #[test]
    fn test_create_migration_configs_no_path() {
        let (warm, cold) = create_migration_configs(StorageBackendType::InMemory, None);
        // No path → both configs have None path
        assert!(warm.path.is_none());
        assert!(cold.path.is_none());
        // Backend type propagates
        assert!(matches!(warm.backend_type, StorageBackendType::InMemory));
    }

    #[test]
    fn test_create_migration_configs_warm_vs_cold_differs() {
        let (warm, cold) = create_migration_configs(StorageBackendType::Libsql, None);
        // Warm and cold have different cache sizes + compression policies
        assert_ne!(warm.cache_size_mb, cold.cache_size_mb);
        assert_ne!(warm.compression, cold.compression);
    }

    // ── show_*_diagnostics (println side-effect — exercise without panic) ───

    #[test]
    fn test_show_plain_diagnostics_no_panic() {
        show_plain_diagnostics(&populated_stats(), false);
        show_plain_diagnostics(&populated_stats(), true);
    }

    #[test]
    fn test_show_human_diagnostics_no_panic() {
        show_human_diagnostics(&populated_stats(), false);
        show_human_diagnostics(&populated_stats(), true);
    }

    #[test]
    fn test_show_json_diagnostics_with_storage_includes_field() {
        // The function prints — we can't intercept, but it returns Ok and
        // the json! macro internally serializes. Just exercise both branches.
        assert!(show_json_diagnostics(&populated_stats(), true, &[]).is_ok());
        assert!(show_json_diagnostics(&populated_stats(), false, &["scheduler"]).is_ok());
    }

    #[test]
    fn test_show_yaml_diagnostics_no_panic_both_branches() {
        assert!(show_yaml_diagnostics(&populated_stats(), true, &[]).is_ok());
        assert!(show_yaml_diagnostics(&populated_stats(), false, &["resources"]).is_ok());
    }

    // ── silent-success diagnostics + fabricated compression ratio ───────────

    /// `pmat tdg diagnostics` with no flags produced zero bytes and exit 0.
    /// With nothing selected, storage — the only implemented section — must be
    /// shown.
    #[test]
    fn test_bare_diagnostics_selects_storage_section() {
        // Bare invocation: no flags at all must still render something.
        assert!(should_show_storage(false, false, false, false));
        assert!(should_show_storage(true, false, false, false));
        // An explicit unrelated section does not silently pull storage in.
        assert!(!should_show_storage(false, true, false, false));
        assert!(!should_show_storage(false, false, true, true));
    }

    /// The three flags with no implementation must be named, not ignored.
    #[test]
    fn test_unavailable_sections_names_unimplemented_flags() {
        assert!(unavailable_sections(false, false, false).is_empty());
        assert_eq!(unavailable_sections(true, false, false), vec!["scheduler"]);
        assert_eq!(
            unavailable_sections(true, true, true),
            vec!["scheduler", "adaptive-thresholds", "resources"]
        );
    }

    /// A ratio cannot be derived from zero stored entries; it used to print
    /// "Compression ratio: 33.0%" regardless.
    #[test]
    fn test_compression_ratio_not_reported_for_empty_store() {
        let rendered = empty_stats().compression_ratio_display();
        assert!(
            rendered.contains("not measured"),
            "empty store must not report a compression percentage, got {rendered}"
        );
        assert!(!rendered.contains('%'), "got {rendered}");
        assert_eq!(populated_stats().compression_ratio_display(), "65.0%");
    }

    #[test]
    fn test_show_table_diagnostics_no_panic() {
        show_table_diagnostics(&populated_stats());
        show_table_diagnostics(&empty_stats());
    }

    // ── print_basic_storage_info / show_backend_details ─────────────────────

    #[test]
    fn test_print_basic_storage_info_no_panic() {
        print_basic_storage_info(&populated_stats());
        print_basic_storage_info(&empty_stats());
    }

    #[test]
    fn test_show_backend_details_iterates_all_tiers() {
        let mut backend = HashMap::new();
        let mut warm = HashMap::new();
        warm.insert("k".to_string(), "v".to_string());
        let mut cold = HashMap::new();
        cold.insert("k2".to_string(), "v2".to_string());
        backend.insert("warm".to_string(), warm);
        backend.insert("cold".to_string(), cold);
        show_backend_details(&backend);
    }

    #[test]
    fn test_show_backend_details_empty_no_panic() {
        let backend = HashMap::new();
        show_backend_details(&backend);
    }

    #[test]
    fn test_show_human_backend_details_iterates_keys() {
        let mut backend = HashMap::new();
        let mut warm = HashMap::new();
        warm.insert("a".to_string(), "1".to_string());
        warm.insert("b".to_string(), "2".to_string());
        backend.insert("warm".to_string(), warm);
        show_human_backend_details(&backend);
    }

    #[test]
    fn test_print_backend_statistics_no_panic() {
        let mut backend = HashMap::new();
        let mut tier = HashMap::new();
        tier.insert("entries".to_string(), "42".to_string());
        backend.insert("tier1".to_string(), tier);
        print_backend_statistics(&backend);
    }
}
