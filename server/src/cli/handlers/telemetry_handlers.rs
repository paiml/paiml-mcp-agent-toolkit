//! Telemetry command handlers for PMAT system monitoring
//! 
//! This module provides CLI handlers for interacting with the telemetry system,
//! enabling users to view system metrics, service performance data, and 
//! system health information.

use crate::services::telemetry_service::{telemetry, TelemetryInput, OperationMetrics};
use anyhow::Result;
use std::collections::HashMap;
use std::time::Instant;
use tracing::{info, debug};
use serde_json::json;

/// Handle telemetry command
pub async fn handle_telemetry(
    system: bool,
    service: Option<String>,
    reset: bool,
    test_event: bool,
) -> Result<()> {
    let start_time = Instant::now();

    if reset {
        #[cfg(test)]
        {
            telemetry().reset();
            println!("🔄 Telemetry data reset successfully");
            return Ok(());
        }
        
        #[cfg(not(test))]
        {
            println!("⚠️ Telemetry reset is only available in test builds");
            return Ok(());
        }
    }

    if test_event {
        record_test_telemetry_event().await?;
        println!("✅ Test telemetry event recorded successfully");
        return Ok(());
    }

    if let Some(service_name) = service {
        show_service_telemetry(&service_name).await?;
    } else if system {
        show_system_telemetry().await?;
    } else {
        // Default: show system overview
        show_system_overview().await?;
    }

    // Record this telemetry command execution
    let _ = record_telemetry_command_execution(start_time).await;

    Ok(())
}

/// Show comprehensive system telemetry data
async fn show_system_telemetry() -> Result<()> {
    info!("📊 Generating system telemetry report");
    
    let telemetry_service = telemetry();
    let system_data = telemetry_service.get_system_telemetry().await?;
    
    println!("🔍 PMAT System Telemetry Report");
    println!("{}", "=".repeat(50));
    println!();
    
    // System overview
    println!("📊 System Overview:");
    println!("  Uptime: {} seconds", system_data.uptime_seconds);
    println!("  Total Operations: {}", system_data.system_metrics.total_operations);
    println!("  Success Rate: {:.2}%", system_data.system_metrics.success_rate * 100.0);
    println!("  Average Duration: {} ms", system_data.system_metrics.avg_duration_ms);
    println!("  Total Items Processed: {}", system_data.system_metrics.total_items_processed);
    println!();

    // Service breakdown
    if !system_data.services.is_empty() {
        println!("🔧 Service Breakdown:");
        for (service_name, service_data) in system_data.services.iter() {
            println!("  📦 {}", service_name);
            println!("    Operations: {} (Success: {}, Failed: {})",
                service_data.total_operations,
                service_data.successful_operations,
                service_data.failed_operations
            );
            println!("    Success Rate: {:.2}%", service_data.success_rate * 100.0);
            println!("    Avg Duration: {} ms", service_data.avg_duration_ms);
            println!("    Items Processed: {}", service_data.total_items_processed);
            
            if !service_data.operation_counts.is_empty() {
                println!("    Top Operations:");
                let mut ops: Vec<_> = service_data.operation_counts.iter().collect();
                ops.sort_by(|a, b| b.1.cmp(a.1));
                for (op, count) in ops.iter().take(3) {
                    println!("      - {}: {} times", op, count);
                }
            }
            println!();
        }
    }

    // JSON output for programmatic access
    println!("📄 Raw Data (JSON):");
    println!("{}", serde_json::to_string_pretty(&system_data)?);

    Ok(())
}

/// Show telemetry data for a specific service
async fn show_service_telemetry(service_name: &str) -> Result<()> {
    info!(service = %service_name, "📊 Generating service telemetry report");
    
    let telemetry_service = telemetry();
    
    match telemetry_service.get_service_telemetry(service_name).await {
        Some(service_data) => {
            println!("🔍 Service Telemetry: {}", service_name);
            println!("{}", "=".repeat(50));
            println!();
            
            println!("📊 Performance Metrics:");
            println!("  Total Operations: {}", service_data.total_operations);
            println!("  Successful: {} ({:.2}%)", 
                service_data.successful_operations,
                service_data.success_rate * 100.0
            );
            println!("  Failed: {}", service_data.failed_operations);
            println!("  Average Duration: {} ms", service_data.avg_duration_ms);
            println!("  Total Duration: {} ms", service_data.total_duration_ms);
            println!("  Items Processed: {}", service_data.total_items_processed);
            
            if service_data.peak_memory_bytes > 0 {
                println!("  Peak Memory: {} bytes", service_data.peak_memory_bytes);
            }
            
            println!("  Last Operation: {} (timestamp)", service_data.last_operation_at);
            println!();
            
            if !service_data.operation_counts.is_empty() {
                println!("🔧 Operation Breakdown:");
                let mut operations: Vec<_> = service_data.operation_counts.iter().collect();
                operations.sort_by(|a, b| b.1.cmp(a.1));
                
                for (operation, count) in operations {
                    let percentage = (*count as f64 / service_data.total_operations as f64) * 100.0;
                    println!("  - {}: {} ({:.1}%)", operation, count, percentage);
                }
            }
            
            println!();
            println!("📄 Raw Data (JSON):");
            println!("{}", serde_json::to_string_pretty(&service_data)?);
        }
        None => {
            println!("❌ No telemetry data found for service: {}", service_name);
            println!("💡 Available services can be seen with: pmat telemetry --system");
        }
    }

    Ok(())
}

/// Show system overview (default command)
async fn show_system_overview() -> Result<()> {
    info!("📊 Generating system overview");
    
    let telemetry_service = telemetry();
    let system_data = telemetry_service.get_system_telemetry().await?;
    
    println!("🔍 PMAT System Overview");
    println!("{}", "=".repeat(30));
    println!();
    
    println!("⏱️ System Status:");
    println!("  Uptime: {} seconds", system_data.uptime_seconds);
    println!("  Total Operations: {}", system_data.system_metrics.total_operations);
    println!("  Success Rate: {:.1}%", system_data.system_metrics.success_rate * 100.0);
    println!();
    
    if !system_data.services.is_empty() {
        println!("🔧 Active Services: {}", system_data.services.len());
        for service_name in system_data.services.keys() {
            println!("  - {}", service_name);
        }
        println!();
        
        println!("💡 Use --system for detailed metrics or --service <name> for service details");
    } else {
        println!("📊 No service telemetry data available yet");
        println!("💡 Use --test-event to generate sample telemetry data");
    }
    
    Ok(())
}

/// Record a test telemetry event for demonstration
async fn record_test_telemetry_event() -> Result<()> {
    debug!("🧪 Recording test telemetry event");
    
    let test_input = TelemetryInput {
        event_type: "test_operation".to_string(),
        service_name: "telemetry_test_service".to_string(),
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
    info!(event_id = %output.event_id, "✅ Test telemetry event recorded");
    
    Ok(())
}

/// Record telemetry for this telemetry command execution  
async fn record_telemetry_command_execution(start_time: Instant) -> Result<()> {
    let duration = start_time.elapsed();
    
    let input = TelemetryInput {
        event_type: "cli_command".to_string(),
        service_name: "cli_telemetry_handler".to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
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
    async fn test_telemetry_command_service() {
        // Reset telemetry for clean test
        telemetry().reset();
        
        // Generate test data
        record_test_telemetry_event().await.unwrap();
        
        // Test service-specific telemetry
        let result = show_service_telemetry("telemetry_test_service").await;
        assert!(result.is_ok());
        
        // Test non-existent service
        let result = show_service_telemetry("non_existent_service").await;
        assert!(result.is_ok()); // Should handle gracefully
    }

    #[tokio::test]
    async fn test_telemetry_reset() {
        // Add some data
        record_test_telemetry_event().await.unwrap();
        
        // Reset should work in test mode
        let result = handle_telemetry(false, None, true, false).await;
        assert!(result.is_ok());
        
        // Verify data is reset
        let system_data = telemetry().get_system_telemetry().await.unwrap();
        assert_eq!(system_data.system_metrics.total_operations, 0);
    }

    #[tokio::test] 
    async fn test_test_event_generation() {
        telemetry().reset();
        
        let result = handle_telemetry(false, None, false, true).await;
        assert!(result.is_ok());
        
        // Verify event was recorded
        let system_data = telemetry().get_system_telemetry().await.unwrap();
        assert!(system_data.system_metrics.total_operations > 0);
    }
}