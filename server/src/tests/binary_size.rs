// Binary size regression tests
// Ensures binary size optimizations don't regress

use std::fs;

#[test]
fn binary_size_regression() {
    // Check if release binary exists (path relative to workspace root)
    let binary_path = "../target/release/paiml-mcp-agent-toolkit";
    let metadata = fs::metadata(binary_path)
        .expect("Release binary not found. Run `cargo build --release` first");

    let size_bytes = metadata.len();
    let size_mb = size_bytes as f64 / (1024.0 * 1024.0);

    println!("Binary size: {} bytes ({:.2} MB)", size_bytes, size_mb);

    // Define size thresholds
    const MAX_SIZE_BYTES: u64 = 20 * 1024 * 1024; // 20MB
    const EXPECTED_SIZE_BYTES: u64 = 17 * 1024 * 1024; // ~17MB expected

    // Hard limit: fail if binary exceeds 20MB
    assert!(
        size_bytes < MAX_SIZE_BYTES,
        "Binary size {} bytes exceeds maximum limit of {} bytes (20MB). \
         Binary size optimizations may have regressed.",
        size_bytes,
        MAX_SIZE_BYTES
    );

    // Warning if binary is significantly larger than expected
    if size_bytes > EXPECTED_SIZE_BYTES {
        println!(
            "WARNING: Binary size {} bytes is larger than expected {} bytes. \
             This may indicate a regression in size optimizations.",
            size_bytes, EXPECTED_SIZE_BYTES
        );
    }
}

#[test]
fn feature_size_impact() {
    // This test measures the impact of features on binary size
    // It requires manual measurement as we can't build different configs in same test

    // Record expected sizes for different feature combinations
    let feature_sizes = vec![
        ("all-languages", 16_900_000, 17_500_000), // min, max bytes
        ("rust-only", 14_000_000, 16_000_000),     // should be smaller
    ];

    for (feature_name, min_size, max_size) in feature_sizes {
        println!(
            "Expected size for '{}' features: {} - {} bytes",
            feature_name, min_size, max_size
        );
    }

    // The actual size measurement would be done via Makefile `size-compare` target
    println!("Run `make size-compare` to measure actual feature impact");
}

#[test]
fn template_compression_works() {
    // Verify that template compression is functioning
    // This checks that the build.rs compression is working

    // Check that compressed templates constant exists
    // This is a compile-time check that compression happened
    let compressed_content = include_str!(concat!(env!("OUT_DIR"), "/compressed_templates.rs"));
    let compressed_size = compressed_content.len();

    // The compressed template file should exist and be non-empty
    assert!(
        compressed_size > 100,
        "Template compression appears to have failed: size was {} bytes",
        compressed_size
    );

    // Should contain the expected compressed template structure
    assert!(compressed_content.contains("COMPRESSED_TEMPLATES"));
    assert!(compressed_content.contains("hex::decode"));

    println!("Compressed templates file size: {} bytes", compressed_size);
}

#[cfg(test)]
mod benchmarks {

    #[test]
    fn startup_time_regression() {
        use std::process::Command;
        use std::time::Instant;

        let binary_path = "../target/release/paiml-mcp-agent-toolkit";

        // Measure cold startup time
        let start = Instant::now();
        let output = Command::new(binary_path)
            .arg("--version")
            .output()
            .expect("Failed to execute binary");
        let duration = start.elapsed();

        assert!(output.status.success(), "Binary failed to execute");

        // Startup should be under 100ms for good UX
        let startup_ms = duration.as_millis();
        println!("Cold startup time: {}ms", startup_ms);

        // Warning threshold
        if startup_ms > 100 {
            println!(
                "WARNING: Startup time {}ms exceeds recommended 100ms threshold",
                startup_ms
            );
        }

        // Hard limit: fail if startup exceeds 1 second
        assert!(
            startup_ms < 1000,
            "Startup time {}ms exceeds maximum limit of 1000ms",
            startup_ms
        );
    }

    #[test]
    fn memory_usage_baseline() {
        // This test establishes a baseline for memory usage
        // Actual measurement would require external tools like valgrind or system monitoring

        println!("Memory usage baseline test");
        println!(
            "Run with: valgrind --tool=massif ./target/release/paiml-mcp-agent-toolkit --version"
        );
        println!("Expected peak memory: <50MB for CLI operations");

        // For now, this is just documentation
        // Future implementation could use system APIs to measure actual memory
    }
}
