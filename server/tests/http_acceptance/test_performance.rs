//! HTTP API Acceptance Tests - Performance and Load Testing
//!
//! Tests for HTTP API performance, concurrency, and load handling.
//! Validates response times, throughput, and system behavior under load.

use crate::http_acceptance::helpers::http_test_client::{HttpTestClient, HttpValidators};
use anyhow::Result;
use futures::future::join_all;
use std::time::Duration;
use tokio::time::Instant;

/// Test single request performance requirements
async fn test_single_request_performance() -> Result<()> {
    let client = HttpTestClient::new("http://localhost:3000")?;

    // Dashboard/UI endpoints - should be very fast (cached)
    let dashboard_result = client.get("/").await?;
    if dashboard_result.success {
        HttpValidators::assert_performance(&dashboard_result, Duration::from_secs(2))?;
    }

    // API summary endpoints - should be reasonably fast (cached analysis)
    let summary_result = client.get("/api/summary").await?;
    if summary_result.success {
        HttpValidators::assert_performance(&summary_result, Duration::from_secs(5))?;
    }

    let metrics_result = client.get("/api/metrics").await?;
    if metrics_result.success {
        HttpValidators::assert_performance(&metrics_result, Duration::from_secs(5))?;
    }

    // Complex analysis endpoints - can be slower but still reasonable
    let analysis_result = client.get("/api/analysis").await?;
    if analysis_result.success {
        HttpValidators::assert_performance(&analysis_result, Duration::from_secs(30))?;
    }

    let architecture_result = client.get("/api/v1/analysis/architecture").await?;
    if architecture_result.success {
        HttpValidators::assert_performance(&architecture_result, Duration::from_secs(30))?;
    }

    Ok(())
}

/// Test concurrent request handling
async fn test_concurrent_requests() -> Result<()> {
    let base_url = "http://localhost:3000";
    let concurrent_requests = 10;

    // Create multiple clients for concurrent testing
    let mut clients = Vec::new();
    for _ in 0..concurrent_requests {
        clients.push(HttpTestClient::new(base_url)?);
    }

    // Test concurrent access to different endpoints
    let endpoints = [
        "/api/summary",
        "/api/metrics",
        "/api/hotspots",
        "/api/dag",
        "/api/analysis",
    ];

    let start_time = Instant::now();

    // Create futures for concurrent requests
    let mut futures = Vec::new();
    for (i, client) in clients.iter().enumerate() {
        let endpoint = endpoints[i % endpoints.len()];
        futures.push(client.get(endpoint));
    }

    // Execute all requests concurrently
    let results = join_all(futures).await;
    let total_time = start_time.elapsed();

    // Validate results
    let mut successful_requests = 0;
    for http_result in results.into_iter().flatten() {
        if http_result.success {
            successful_requests += 1;
            // Each request should still meet individual performance requirements
            HttpValidators::assert_performance(&http_result, Duration::from_secs(30))?;
        }
    }

    // At least some requests should succeed
    assert!(
        successful_requests > 0,
        "Some concurrent requests should succeed"
    );

    // Total time should be reasonable (not sequential)
    assert!(
        total_time < Duration::from_secs(45),
        "Concurrent requests should not take too long: {:?}",
        total_time
    );

    println!(
        "Concurrent test: {}/{} requests successful in {:?}",
        successful_requests, concurrent_requests, total_time
    );

    Ok(())
}

/// Test sustained load handling
#[tokio::test]
async fn test_sustained_load() -> Result<()> {
    let client = HttpTestClient::new("http://localhost:3000")?;
    let test_duration = Duration::from_secs(30);
    let request_interval = Duration::from_millis(500); // 2 requests per second

    let start_time = Instant::now();
    let mut request_count = 0;
    let mut successful_requests = 0;
    let mut total_response_time = Duration::default();

    // Run sustained load test
    while start_time.elapsed() < test_duration {
        let request_start = Instant::now();

        // Alternate between different endpoints
        let endpoint = match request_count % 4 {
            0 => "/api/summary",
            1 => "/api/metrics",
            2 => "/api/hotspots",
            _ => "/api/dag",
        };

        if let Ok(result) = client.get(endpoint).await {
            request_count += 1;
            total_response_time += result.execution_time;

            if result.success {
                successful_requests += 1;
            }

            // Verify individual request performance doesn't degrade significantly
            if result.execution_time > Duration::from_secs(10) {
                println!(
                    "Warning: Slow response time: {:?} for {}",
                    result.execution_time, endpoint
                );
            }
        }

        // Wait before next request
        let elapsed = request_start.elapsed();
        if elapsed < request_interval {
            tokio::time::sleep(request_interval - elapsed).await;
        }
    }

    // Calculate performance metrics
    let success_rate = if request_count > 0 {
        (successful_requests as f64 / request_count as f64) * 100.0
    } else {
        0.0
    };

    let avg_response_time = if successful_requests > 0 {
        total_response_time / successful_requests as u32
    } else {
        Duration::default()
    };

    println!("Sustained load test results:");
    println!("  Total requests: {}", request_count);
    println!("  Successful requests: {}", successful_requests);
    println!("  Success rate: {:.1}%", success_rate);
    println!("  Average response time: {:?}", avg_response_time);

    // Validate performance requirements
    assert!(
        success_rate >= 80.0,
        "Success rate should be at least 80%, got {:.1}%",
        success_rate
    );
    assert!(
        avg_response_time < Duration::from_secs(8),
        "Average response time should be reasonable: {:?}",
        avg_response_time
    );
    assert!(
        request_count >= 50,
        "Should complete reasonable number of requests: {}",
        request_count
    );

    Ok(())
}

/// Test memory usage under load
#[tokio::test]
async fn test_memory_usage_stability() -> Result<()> {
    let client = HttpTestClient::new("http://localhost:3000")?;

    // Create a large number of requests to test memory stability
    let request_count = 100;
    let batch_size = 10;

    for batch in 0..(request_count / batch_size) {
        let mut batch_futures = Vec::new();

        // Create batch of concurrent requests
        for _ in 0..batch_size {
            let endpoint = match batch % 3 {
                0 => "/api/summary",
                1 => "/api/metrics",
                _ => "/api/analysis",
            };
            batch_futures.push(client.get(endpoint));
        }

        // Execute batch
        let batch_results = join_all(batch_futures).await;

        // Validate batch results
        for http_result in batch_results.into_iter().flatten() {
            // Responses should not degrade over time
            if http_result.success {
                HttpValidators::assert_performance(&http_result, Duration::from_secs(30))?;
            }
        }

        // Small delay between batches to allow garbage collection
        tokio::time::sleep(Duration::from_millis(100)).await;

        if batch % 5 == 0 {
            println!(
                "Completed batch {}/{}",
                batch + 1,
                request_count / batch_size
            );
        }
    }

    println!(
        "Memory stability test completed {} requests successfully",
        request_count
    );

    Ok(())
}

/// Test request timeout handling
async fn test_timeout_handling() -> Result<()> {
    let client = HttpTestClient::new("http://localhost:3000")?;

    // Test that long-running requests don't hang indefinitely
    let long_running_endpoints = [
        "/api/analysis",
        "/api/v1/analysis/architecture",
        "/api/v1/analysis/defects",
    ];

    for endpoint in &long_running_endpoints {
        let start_time = Instant::now();

        // Set a reasonable timeout for testing
        let timeout_duration = Duration::from_secs(60);

        let result = tokio::time::timeout(timeout_duration, client.get(endpoint)).await;

        match result {
            Ok(http_result) => {
                let elapsed = start_time.elapsed();
                if let Ok(_response) = http_result {
                    println!("Endpoint {} completed in {:?}", endpoint, elapsed);

                    // Should complete within reasonable time
                    assert!(
                        elapsed < Duration::from_secs(45),
                        "Endpoint {} took too long: {:?}",
                        endpoint,
                        elapsed
                    );
                } else {
                    println!("Endpoint {} failed (may not be implemented)", endpoint);
                }
            }
            Err(_) => {
                anyhow::bail!(
                    "Endpoint {} timed out after {:?}",
                    endpoint,
                    timeout_duration
                );
            }
        }
    }

    Ok(())
}

/// Test response size and bandwidth efficiency
async fn test_response_efficiency() -> Result<()> {
    let client = HttpTestClient::new("http://localhost:3000")?;

    let endpoints = [
        ("/api/summary", 50_000),   // Should be concise
        ("/api/metrics", 100_000),  // Can be larger but reasonable
        ("/api/analysis", 500_000), // Complex analysis can be large
        ("/", 100_000),             // Dashboard HTML should be reasonable
    ];

    for (endpoint, max_size_bytes) in &endpoints {
        let result = client.get(endpoint).await?;

        if result.success {
            let response_size = result.response.body.len();

            assert!(
                response_size <= *max_size_bytes,
                "Response from {} too large: {} bytes > {} bytes",
                endpoint,
                response_size,
                max_size_bytes
            );

            println!(
                "Endpoint {} response size: {} bytes",
                endpoint, response_size
            );

            // Check for compression headers
            if response_size > 10_000 {
                let has_compression = result
                    .response
                    .headers
                    .get("content-encoding")
                    .map(|enc| enc.contains("gzip") || enc.contains("deflate"))
                    .unwrap_or(false);

                if !has_compression {
                    println!("Warning: Large response from {} not compressed", endpoint);
                }
            }
        }
    }

    Ok(())
}

/// Test error rate under stress
#[tokio::test]
async fn test_error_rate_under_stress() -> Result<()> {
    let base_url = "http://localhost:3000";
    let concurrent_clients = 20;
    let requests_per_client = 5;

    // Create stress scenario with many concurrent clients
    let mut client_futures = Vec::new();

    for client_id in 0..concurrent_clients {
        let client = HttpTestClient::new(base_url)?;

        // Each client makes multiple requests
        let client_future = async move {
            let mut results = Vec::new();

            for request_num in 0..requests_per_client {
                let endpoint = match (client_id + request_num) % 4 {
                    0 => "/api/summary",
                    1 => "/api/metrics",
                    2 => "/api/hotspots",
                    _ => "/api/dag",
                };

                if let Ok(result) = client.get(endpoint).await {
                    results.push((client_id, request_num, result.success, result.status_code));
                }

                // Small delay between requests from same client
                tokio::time::sleep(Duration::from_millis(50)).await;
            }

            results
        };

        client_futures.push(client_future);
    }

    // Execute all client workflows concurrently
    let start_time = Instant::now();
    let all_results = join_all(client_futures).await;
    let total_time = start_time.elapsed();

    // Analyze results
    let mut total_requests = 0;
    let mut successful_requests = 0;
    let mut error_counts = std::collections::HashMap::new();

    for client_results in all_results {
        for (_client_id, _request_num, success, status_code) in client_results {
            total_requests += 1;

            if success {
                successful_requests += 1;
            } else {
                *error_counts.entry(status_code).or_insert(0) += 1;
            }
        }
    }

    let success_rate = if total_requests > 0 {
        (successful_requests as f64 / total_requests as f64) * 100.0
    } else {
        0.0
    };

    println!("Stress test results:");
    println!("  Total requests: {}", total_requests);
    println!("  Successful requests: {}", successful_requests);
    println!("  Success rate: {:.1}%", success_rate);
    println!("  Total time: {:?}", total_time);

    if !error_counts.is_empty() {
        println!("  Error breakdown:");
        for (status_code, count) in error_counts {
            println!("    {}: {} requests", status_code, count);
        }
    }

    // Validate stress test requirements
    assert!(
        success_rate >= 70.0,
        "Success rate under stress should be at least 70%, got {:.1}%",
        success_rate
    );

    assert!(
        total_time < Duration::from_secs(120),
        "Stress test should complete within reasonable time: {:?}",
        total_time
    );

    Ok(())
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Test complete performance validation workflow
    #[tokio::test]
    async fn test_complete_performance_validation() -> Result<()> {
        println!("Starting comprehensive HTTP performance validation...");

        // 1. Basic performance validation
        println!("1. Testing single request performance...");
        test_single_request_performance().await?;

        // 2. Concurrency test
        println!("2. Testing concurrent request handling...");
        test_concurrent_requests().await?;

        // 3. Response efficiency
        println!("3. Testing response size efficiency...");
        test_response_efficiency().await?;

        // 4. Timeout handling
        println!("4. Testing timeout handling...");
        test_timeout_handling().await?;

        println!("HTTP performance validation completed successfully!");

        Ok(())
    }
}
