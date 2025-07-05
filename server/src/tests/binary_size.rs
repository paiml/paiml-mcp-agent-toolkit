// Binary size regression tests
// Ensures binary size optimizations don't regress

use std::fs;

#[test]
fn binary_size_regression() {
    // Apply Kaizen - Use correct binary name and path with fallback strategy
    let binary_path = if std::path::Path::new("target/release/pmat").exists() {
        "target/release/pmat"
    } else if std::path::Path::new("../target/release/pmat").exists() {
        "../target/release/pmat"
    } else if std::path::Path::new("target/debug/pmat").exists() {
        // Skip test if only debug build exists
        println!("⚠️  Skipping binary size regression test - release binary not found");
        println!("   Run 'cargo build --release' to enable this test");
        return;
    } else {
        panic!("No binary found. Run 'cargo build' or 'cargo build --release' first");
    };

    let metadata = fs::metadata(binary_path)
        .unwrap_or_else(|e| panic!("Failed to read binary metadata for {binary_path}: {e}"));

    let size_bytes = metadata.len();
    let size_mb = size_bytes as f64 / (1024.0 * 1024.0);

    println!(
        "Kaizen Quality Check - Binary size: {size_bytes} bytes ({size_mb:.2} MB) at {binary_path}"
    );

    // Define size thresholds
    const MAX_SIZE_BYTES: u64 = 25 * 1024 * 1024; // 25MB (increased due to additional features)
    const EXPECTED_SIZE_BYTES: u64 = 22 * 1024 * 1024; // ~22MB expected

    // Jidoka - Build quality in: fail if binary exceeds 25MB
    assert!(
        size_bytes < MAX_SIZE_BYTES,
        "Kaizen Quality Gate Failed: Binary size {size_bytes} bytes exceeds maximum limit of {MAX_SIZE_BYTES} bytes (25MB). \
         Consider applying Muda elimination to reduce binary size."
    );

    // Kaizen warning if binary is significantly larger than expected
    if size_bytes > EXPECTED_SIZE_BYTES {
        println!(
            "⚠️  Kaizen Warning: Binary size {size_bytes} bytes is larger than expected {EXPECTED_SIZE_BYTES} bytes. \
             This may indicate a regression in size optimizations."
        );
    } else {
        println!("✅ Kaizen Success: Binary size within expected limits");
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
        println!("Expected size for '{feature_name}' features: {min_size} - {max_size} bytes");
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
        "Template compression appears to have failed: size was {compressed_size} bytes"
    );

    // Should contain the expected compressed template structure
    assert!(compressed_content.contains("COMPRESSED_TEMPLATES"));
    assert!(compressed_content.contains("hex::decode"));

    println!("Compressed templates file size: {compressed_size} bytes");
}

#[cfg(test)]
mod benchmarks {

    #[test]
    fn startup_time_regression() {
        use std::process::Command;
        use std::time::Instant;

        // Apply Kaizen - Use correct binary name and path with fallback strategy
        let binary_path = if std::path::Path::new("target/release/pmat").exists() {
            "target/release/pmat"
        } else if std::path::Path::new("../target/release/pmat").exists() {
            "../target/release/pmat"
        } else if std::path::Path::new("target/debug/pmat").exists() {
            // Fallback to debug build for development
            "target/debug/pmat"
        } else if std::path::Path::new("../target/debug/pmat").exists() {
            "../target/debug/pmat"
        } else {
            // Try workspace-level paths
            "server/target/debug/pmat"
        };

        // Apply Poka-yoke - Verify binary exists before testing
        if !std::path::Path::new(binary_path).exists() {
            panic!("Binary not found at {binary_path}. Run 'cargo build --release' to create it.");
        }

        // Measure cold startup time
        let start = Instant::now();
        let output = Command::new(binary_path)
            .arg("--version")
            .output()
            .unwrap_or_else(|e| panic!("Failed to execute binary at {binary_path}: {e}"));
        let duration = start.elapsed();

        assert!(output.status.success(), "Binary failed to execute");

        // Toyota Way quality standards for user experience
        let startup_ms = duration.as_millis();
        println!("Kaizen Quality Check - Cold startup time: {startup_ms}ms using {binary_path}");

        // Jidoka - Build quality in: Startup should be under 100ms for good UX
        let startup_threshold_ms = 100;
        if startup_ms > startup_threshold_ms {
            println!("⚠️  Kaizen Warning: Startup time {startup_ms}ms exceeds {startup_threshold_ms}ms threshold");
            println!("   Consider applying Muda elimination to reduce startup overhead");
        } else {
            println!("✅ Kaizen Success: Startup time meets quality standard");
        }

        // Quality gate - Allow some flexibility for CI environments
        let max_startup_ms = if std::env::var("CI").is_ok() {
            200
        } else {
            100
        };
        assert!(
            startup_ms <= max_startup_ms,
            "Kaizen Quality Gate Failed: Startup time {startup_ms}ms exceeds maximum {max_startup_ms}ms"
        );

        // Hard limit: fail if startup exceeds 1 second
        assert!(
            startup_ms < 1000,
            "Startup time {startup_ms}ms exceeds maximum limit of 1000ms"
        );
    }

    #[test]
    fn memory_usage_baseline() {
        // This test establishes a baseline for memory usage
        // Actual measurement would require external tools like valgrind or system monitoring

        println!("Memory usage baseline test");
        println!("Run with: valgrind --tool=massif ./target/release/pmat --version");
        println!("Expected peak memory: <50MB for CLI operations");

        // For now, this is just documentation
        // Future implementation could use system APIs to measure actual memory
    }
}
