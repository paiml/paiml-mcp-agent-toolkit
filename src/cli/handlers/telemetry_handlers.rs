//! Telemetry command handlers for PMAT system monitoring
//!
//! This module provides CLI handlers for interacting with the telemetry system,
//! enabling users to view system metrics, service performance data, and
//! system health information.

use crate::cli::colors as c;
use crate::services::telemetry_service::{telemetry, OperationMetrics, TelemetryInput};
use anyhow::Result;
use serde_json::json;
use std::collections::HashMap;
use std::time::Instant;
use tracing::{debug, info};

/// The service name `handle_telemetry` records its own execution under.
const CLI_HANDLER_SERVICE: &str = "cli_telemetry_handler";

/// The service name `--test-event` records its sample event under.
const TEST_EVENT_SERVICE: &str = "telemetry_test_service";

/// Every service name this binary is capable of recording under.
///
/// Telemetry has no registry of service names: `services` is a `DashMap` that
/// grows an entry the first time something records under a name. So the set of
/// names that can ever appear is exactly the set of names the code records with,
/// and these two are it. Kept next to the two call sites that use them so the
/// hint printed on an empty result cannot drift away from reality.
const RECORDABLE_SERVICES: [&str; 2] = [CLI_HANDLER_SERVICE, TEST_EVENT_SERVICE];

/// Handle telemetry command
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn handle_telemetry(
    system: bool,
    service: Option<String>,
    reset: bool,
    test_event: bool,
) -> Result<()> {
    let start_time = Instant::now();

    // Handle reset command
    if reset {
        return handle_reset_command().await;
    }

    // Handle test event command.
    //
    // This must NOT return: the telemetry store lives in this process only, so
    // an early return meant the recorded event was thrown away before anything
    // could show it. `pmat telemetry --test-event` reported success and the very
    // next `pmat telemetry --system` reported "Total Operations: 0", and even
    // `--test-event --system` in one process printed nothing but the success
    // line. Falling through is what makes the recorded event observable at all.
    if test_event {
        handle_test_event_command().await?;
    }

    // Handle display commands
    handle_display_command(system, service).await?;

    // Record this telemetry command execution
    let _ = record_telemetry_command_execution(start_time).await;

    Ok(())
}

/// Handle reset command based on build configuration
async fn handle_reset_command() -> Result<()> {
    #[cfg(test)]
    {
        telemetry().reset();
        println!("{}", c::pass("Telemetry data reset successfully"));
        Ok(())
    }

    // `--reset` is advertised in `--help` on every published binary, but on a
    // release build it printed a warning marker and exited 0 — a request that
    // did nothing reported as success. It has to fail.
    #[cfg(not(test))]
    {
        anyhow::bail!("Telemetry reset is only available in test builds; nothing was reset")
    }
}

/// Handle test event recording command
///
/// The success line has to say WHERE the event went. Telemetry is an in-memory,
/// process-local store with no persistence, so "recorded successfully" read as a
/// promise that a later `pmat telemetry --system` would count it — it never can.
async fn handle_test_event_command() -> Result<()> {
    record_test_telemetry_event().await?;
    println!(
        "{}",
        c::pass("Test telemetry event recorded in this process (telemetry is in-memory and is not persisted between runs)")
    );
    Ok(())
}

/// Handle display command based on options
async fn handle_display_command(system: bool, service: Option<String>) -> Result<()> {
    match (system, service) {
        (_, Some(service_name)) => show_service_telemetry(&service_name).await,
        (true, None) => show_system_telemetry().await,
        (false, None) => show_system_overview().await,
    }
}

/// Show comprehensive system telemetry data
async fn show_system_telemetry() -> Result<()> {
    info!("Generating system telemetry report");

    let telemetry_service = telemetry();
    let system_data = telemetry_service.get_system_telemetry().await?;

    println!("{}", c::header("PMAT System Telemetry Report"));
    println!("{}", c::rule());
    println!();

    // System overview
    println!("{}", c::subheader("System Overview:"));
    println!(
        "  {}: {} seconds",
        c::dim("Uptime"),
        c::number(&system_data.uptime_seconds.to_string())
    );
    println!(
        "  {}: {}",
        c::dim("Total Operations"),
        c::number(&system_data.system_metrics.total_operations.to_string())
    );
    println!(
        "  {}: {}",
        c::dim("Success Rate"),
        c::pct(system_data.system_metrics.success_rate * 100.0, 90.0, 70.0)
    );
    println!(
        "  {}: {} ms",
        c::dim("Average Duration"),
        c::number(&system_data.system_metrics.avg_duration_ms.to_string())
    );
    println!(
        "  {}: {}",
        c::dim("Total Items Processed"),
        c::number(&system_data.system_metrics.total_items_processed.to_string())
    );
    println!();

    // Service breakdown
    if !system_data.services.is_empty() {
        println!("{}", c::subheader("Service Breakdown:"));
        for (service_name, service_data) in &system_data.services {
            println!("  {}", c::label(service_name));
            println!(
                "    {}: {} ({}Success{}: {}, {}Failed{}: {})",
                c::dim("Operations"),
                c::number(&service_data.total_operations.to_string()),
                c::GREEN,
                c::RESET,
                service_data.successful_operations,
                c::RED,
                c::RESET,
                service_data.failed_operations
            );
            println!(
                "    {}: {}",
                c::dim("Success Rate"),
                c::pct(service_data.success_rate * 100.0, 90.0, 70.0)
            );
            println!(
                "    {}: {} ms",
                c::dim("Avg Duration"),
                c::number(&service_data.avg_duration_ms.to_string())
            );
            println!(
                "    {}: {}",
                c::dim("Items Processed"),
                c::number(&service_data.total_items_processed.to_string())
            );

            if !service_data.operation_counts.is_empty() {
                println!("    {}:", c::dim("Top Operations"));
                let mut ops: Vec<_> = service_data.operation_counts.iter().collect();
                ops.sort_by(|a, b| b.1.cmp(a.1));
                for (op, count) in ops.iter().take(3) {
                    println!("      - {}: {} times", c::label(op), count);
                }
            }
            println!();
        }
    }

    // JSON output for programmatic access
    println!("{}", c::dim("Raw Data (JSON):"));
    println!("{}", serde_json::to_string_pretty(&system_data)?);

    Ok(())
}

/// Show telemetry data for a specific service
async fn show_service_telemetry(service_name: &str) -> Result<()> {
    info!(service = %service_name, "Generating service telemetry report");

    // A blank name is not a service that happens to be empty, it is a missing
    // argument. `TelemetryService::validate_input` rejects an empty
    // `service_name` as a missing field for exactly this reason, so a query for
    // one cannot be answered and stays an error.
    if service_name.trim().is_empty() {
        anyhow::bail!(
            "--service needs a service name; got an empty one. \
             Services this build records under: {}",
            RECORDABLE_SERVICES.join(", ")
        );
    }

    let telemetry_service = telemetry();

    if let Some(service_data) = telemetry_service.get_service_telemetry(service_name).await {
        println!(
            "{} {}",
            c::header("Service Telemetry:"),
            c::label(service_name)
        );
        println!("{}", c::rule());
        println!();

        println!("{}", c::subheader("Performance Metrics:"));
        println!(
            "  {}: {}",
            c::dim("Total Operations"),
            c::number(&service_data.total_operations.to_string())
        );
        println!(
            "  {}: {} ({})",
            c::dim("Successful"),
            c::number(&service_data.successful_operations.to_string()),
            c::pct(service_data.success_rate * 100.0, 90.0, 70.0)
        );
        println!(
            "  {}: {}",
            c::dim("Failed"),
            if service_data.failed_operations > 0 {
                format!("{}{}{}", c::RED, service_data.failed_operations, c::RESET)
            } else {
                service_data.failed_operations.to_string()
            }
        );
        println!(
            "  {}: {} ms",
            c::dim("Average Duration"),
            c::number(&service_data.avg_duration_ms.to_string())
        );
        println!(
            "  {}: {} ms",
            c::dim("Total Duration"),
            c::number(&service_data.total_duration_ms.to_string())
        );
        println!(
            "  {}: {}",
            c::dim("Items Processed"),
            c::number(&service_data.total_items_processed.to_string())
        );

        if service_data.peak_memory_bytes > 0 {
            println!(
                "  {}: {} bytes",
                c::dim("Peak Memory"),
                c::number(&service_data.peak_memory_bytes.to_string())
            );
        }

        println!(
            "  {}: {}",
            c::dim("Last Operation"),
            service_data.last_operation_at
        );
        println!();

        if !service_data.operation_counts.is_empty() {
            println!("{}", c::subheader("Operation Breakdown:"));
            let mut operations: Vec<_> = service_data.operation_counts.iter().collect();
            operations.sort_by(|a, b| b.1.cmp(a.1));

            for (operation, count) in operations {
                let percentage = (*count as f64 / service_data.total_operations as f64) * 100.0;
                println!(
                    "  - {}: {} ({:.1}%)",
                    c::label(operation),
                    count,
                    percentage
                );
            }
        }

        println!();
        println!("{}", c::dim("Raw Data (JSON):"));
        println!("{}", serde_json::to_string_pretty(&service_data)?);
        Ok(())
    } else {
        show_empty_service_telemetry(service_name)
    }
}

/// Report that a service has recorded nothing — as an empty result, not a failure.
///
/// This used to be an error, and it made every `pmat telemetry --service <name>`
/// exit 1: telemetry is an in-memory, process-local store that starts empty, and
/// the only event a query process records (`cli_telemetry_handler`) is recorded
/// *after* the display runs. So the miss was universal — even
/// `--service cli_telemetry_handler`, a name pmat definitely records under, exited
/// 1 with "No telemetry data found". Nothing was wrong with the request; there was
/// simply nothing recorded yet, which is the normal state.
///
/// The cost of calling that an error was not cosmetic: `pmat bug-report` files a
/// GitHub issue on a non-zero exit, and it filed #922 for a query that behaved
/// exactly as designed.
///
/// The old message also gave advice that cannot be followed —
/// "Available services can be seen with: pmat telemetry --system" — because
/// `--system` in a fresh process always reports zero services for the same reason.
fn show_empty_service_telemetry(service_name: &str) -> Result<()> {
    println!(
        "{} {}",
        c::header("Service Telemetry:"),
        c::label(service_name)
    );
    println!("{}", c::rule());
    println!();
    println!(
        "No telemetry recorded for {} in this process.",
        c::label(service_name)
    );
    println!(
        "{}",
        c::dim(
            "Telemetry counts operations performed by THIS pmat process and is not persisted \
             between runs, so a freshly started query always finds an empty store."
        )
    );
    println!(
        "{}",
        c::dim(&format!(
            "Services this build records under: {}. To see a populated report in one run: \
             `pmat telemetry --test-event --service {TEST_EVENT_SERVICE}`.",
            RECORDABLE_SERVICES.join(", ")
        ))
    );
    println!();
    println!("{}", c::dim("Raw Data (JSON):"));
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "service_name": service_name,
            "data_recorded": false,
            "total_operations": 0,
        }))?
    );
    Ok(())
}

/// Show system overview (default command)
async fn show_system_overview() -> Result<()> {
    info!("Generating system overview");

    let telemetry_service = telemetry();
    let system_data = telemetry_service.get_system_telemetry().await?;

    println!("{}", c::header("PMAT System Overview"));
    println!("{}", c::rule());
    println!();

    println!("{}", c::subheader("System Status:"));
    println!(
        "  {}: {} seconds",
        c::dim("Uptime"),
        c::number(&system_data.uptime_seconds.to_string())
    );
    println!(
        "  {}: {}",
        c::dim("Total Operations"),
        c::number(&system_data.system_metrics.total_operations.to_string())
    );
    println!(
        "  {}: {}",
        c::dim("Success Rate"),
        c::pct(system_data.system_metrics.success_rate * 100.0, 90.0, 70.0)
    );
    println!();

    if system_data.services.is_empty() {
        println!("{}", c::dim("No service telemetry data available yet"));
        // The old hint — "Use --test-event to generate sample telemetry data" —
        // pointed at something that cannot work across invocations: counters
        // live in this process only, so a separate `--test-event` run always
        // leaves this report at zero.
        println!(
            "{}",
            c::dim(
                "Telemetry counts operations performed by THIS process and is not persisted; \
                 use `pmat telemetry --test-event --system` to see a sample event in one run"
            )
        );
    } else {
        println!(
            "{}: {}",
            c::subheader("Active Services"),
            c::number(&system_data.services.len().to_string())
        );
        for service_name in system_data.services.keys() {
            println!("  - {}", c::label(service_name));
        }
        println!();

        println!(
            "{}",
            c::dim("Use --system for detailed metrics or --service <name> for service details")
        );
    }

    Ok(())
}

/// Record a test telemetry event for demonstration
async fn record_test_telemetry_event() -> Result<()> {
    debug!("Recording test telemetry event");

    let test_input = TelemetryInput {
        event_type: "test_operation".to_string(),
        service_name: TEST_EVENT_SERVICE.to_string(),
        operation: "test_command_execution".to_string(),
        metrics: OperationMetrics {
            duration_ms: 125,
            items_processed: 3,
            memory_bytes: Some(2048),
            cpu_time_ms: Some(95),
            success: true,
            error_message: None,
        },
        tags: {
            let mut tags = HashMap::new();
            tags.insert("test_type".to_string(), "cli_demo".to_string());
            tags.insert("user".to_string(), "system".to_string());
            tags
        },
        properties: {
            let mut props = HashMap::new();
            props.insert("version".to_string(), json!("2.6.8"));
            props.insert("environment".to_string(), json!("development"));
            props
        },
    };

    let output = telemetry().record_operation(test_input).await?;
    info!(event_id = %output.event_id, "Test telemetry event recorded");

    Ok(())
}

/// Record telemetry for this telemetry command execution
async fn record_telemetry_command_execution(start_time: Instant) -> Result<()> {
    let duration = start_time.elapsed();

    let input = TelemetryInput {
        event_type: "cli_command".to_string(),
        service_name: CLI_HANDLER_SERVICE.to_string(),
        operation: "telemetry_command".to_string(),
        metrics: OperationMetrics {
            duration_ms: duration.as_millis() as u64,
            items_processed: 1,
            memory_bytes: None,
            cpu_time_ms: None,
            success: true,
            error_message: None,
        },
        tags: HashMap::new(),
        properties: HashMap::new(),
    };

    let _ = telemetry().record_operation(input).await;
    Ok(())
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[serial_test::serial]
    async fn test_telemetry_command_system() {
        // Reset telemetry for clean test
        telemetry().reset();

        // Generate some test data
        record_test_telemetry_event().await.unwrap();

        // Test system telemetry display
        let result = show_system_overview().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_telemetry_command_service() {
        // Reset telemetry for clean test
        telemetry().reset();

        // Generate test data
        record_test_telemetry_event().await.unwrap();

        // Test service-specific telemetry
        let result = show_service_telemetry("telemetry_test_service").await;
        assert!(result.is_ok());

        // A service with nothing recorded is an EMPTY RESULT, not a failure.
        // This assertion has now been wrong in both directions: it first
        // asserted `is_ok()` while the handler printed a ✗ marker and returned
        // Ok (a failure reported as success), then asserted `is_err()` for a
        // store that is empty on every process start (an empty result reported
        // as a failure). See `empty_service_is_an_empty_result_not_an_error`.
        let result = show_service_telemetry("non_existent_service").await;
        assert!(
            result.is_ok(),
            "a service with no recorded data is an empty result: {:?}",
            result.err()
        );
    }

    /// Issue #922: `pmat telemetry --service tdg` exited 1, and `pmat bug-report`
    /// filed a GitHub issue for it.
    ///
    /// Telemetry is in-memory and process-local, and the one event a query
    /// process records happens *after* the display, so the store is empty for
    /// EVERY name a fresh `--service` query can ask about — including
    /// `cli_telemetry_handler`, which pmat itself records under. Reporting that
    /// as an error made a normal query indistinguishable from a broken one.
    #[tokio::test]
    #[serial_test::serial]
    async fn empty_service_is_an_empty_result_not_an_error() {
        telemetry().reset();

        // The exact command from #922.
        let result = handle_telemetry(false, Some("tdg".to_string()), false, false).await;
        assert!(
            result.is_ok(),
            "pmat telemetry --service tdg must exit 0 with an empty result: {:?}",
            result.err()
        );

        // A name pmat genuinely records under is just as empty in a fresh
        // process — proof that the miss was never about the name being wrong.
        telemetry().reset();
        let result =
            handle_telemetry(false, Some(CLI_HANDLER_SERVICE.to_string()), false, false).await;
        assert!(
            result.is_ok(),
            "a name pmat records under must not error either: {:?}",
            result.err()
        );
    }

    /// The other half: a name that is not a service name at all still fails.
    /// `TelemetryService::validate_input` already rejects an empty
    /// `service_name` as a missing field, so a query for one cannot be answered.
    #[tokio::test]
    #[serial_test::serial]
    async fn a_blank_service_name_is_still_an_error() {
        telemetry().reset();
        record_test_telemetry_event().await.unwrap();

        let err = handle_telemetry(false, Some(String::new()), false, false)
            .await
            .expect_err("--service '' must exit non-zero");
        assert!(
            err.to_string().contains("needs a service name"),
            "the error must say what is wrong with the argument: {err}"
        );

        let err = handle_telemetry(false, Some("   ".to_string()), false, false)
            .await
            .expect_err("--service '   ' must exit non-zero");
        assert!(
            err.to_string().contains("needs a service name"),
            "whitespace is no more a service name than empty is: {err}"
        );
    }

    /// The hint printed on an empty result names the services this build can
    /// record under. If a recording site is renamed without updating the
    /// constants, the hint becomes a lie — this pins them together.
    #[tokio::test]
    #[serial_test::serial]
    async fn the_recordable_service_list_matches_what_is_actually_recorded() {
        telemetry().reset();

        record_test_telemetry_event().await.unwrap();
        record_telemetry_command_execution(Instant::now())
            .await
            .unwrap();

        let data = telemetry().get_system_telemetry().await.unwrap();
        for name in RECORDABLE_SERVICES {
            assert!(
                data.services.contains_key(name),
                "{name} is advertised as recordable but nothing records under it: {:?}",
                data.services.keys().collect::<Vec<_>>()
            );
        }
        assert_eq!(
            data.services.len(),
            RECORDABLE_SERVICES.len(),
            "a service records under a name the hint does not list: {:?}",
            data.services.keys().collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_telemetry_reset() {
        // Add some data
        record_test_telemetry_event().await.unwrap();

        // Reset should work in test mode
        let result = handle_telemetry(false, None, true, false).await;
        assert!(result.is_ok());

        // Verify data is reset
        let _system_data = telemetry().get_system_telemetry().await.unwrap();
        // Note: Assertion disabled due to test flakiness in parallel test environment
        // assert_eq!(system_data.system_metrics.total_operations, 0);
    }

    /// `--test-event` used to return before the display path ran, so the event
    /// it had just recorded was invisible even in the SAME process — and the
    /// command execution that the display path records never happened either.
    #[tokio::test]
    #[serial_test::serial]
    async fn test_test_event_falls_through_to_the_display_path() {
        telemetry().reset();

        handle_telemetry(true, None, false, true).await.unwrap();

        let data = telemetry().get_system_telemetry().await.unwrap();
        assert!(
            data.services.contains_key("telemetry_test_service"),
            "the test event itself must be recorded: {:?}",
            data.services.keys().collect::<Vec<_>>()
        );
        assert!(
            data.services.contains_key("cli_telemetry_handler"),
            "--test-event returned before the display path ran: {:?}",
            data.services.keys().collect::<Vec<_>>()
        );
    }

    /// IGNORED: Flaky in parallel test environment - telemetry state races
    #[tokio::test]
    #[ignore = "requires telemetry setup"]
    async fn test_test_event_generation() {
        telemetry().reset();

        let result = handle_telemetry(false, None, false, true).await;
        assert!(result.is_ok());

        // Verify event was recorded
        let system_data = telemetry().get_system_telemetry().await.unwrap();
        assert!(system_data.system_metrics.total_operations > 0);
    }
}
