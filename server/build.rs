use std::env;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=src/installer/mod.rs");
    println!("cargo:rerun-if-changed=../scripts/install.sh");
    println!("cargo:rerun-if-changed=assets/vendor/");
    println!("cargo:rerun-if-changed=assets/demo/");

    // Verify critical dependencies at build time
    verify_dependency_versions();

    // Download and compress assets for demo mode
    if env::var("CARGO_FEATURE_NO_DEMO").is_err() {
        download_and_compress_assets();
    }
}

fn verify_dependency_versions() {
    // In a workspace, Cargo.lock is in the parent directory
    let lock_path = if Path::new("../Cargo.lock").exists() {
        "../Cargo.lock"
    } else {
        "Cargo.lock"
    };
    let lock_content = fs::read_to_string(lock_path).expect("Cargo.lock must exist");

    // Critical dependencies for your MCP server
    let critical_deps = [
        "tokio",      // Async runtime
        "serde",      // Serialization
        "handlebars", // Template engine
    ];

    for dep in &critical_deps {
        if !lock_content.contains(&format!("name = \"{}\"", dep)) {
            panic!("Critical dependency {} not found", dep);
        }
    }
}

fn download_and_compress_assets() {
    use flate2::write::GzEncoder;
    use flate2::Compression;

    // Create asset directories
    let vendor_dir = Path::new("assets/vendor");
    let demo_dir = Path::new("assets/demo");
    let _ = fs::create_dir_all(vendor_dir);
    let _ = fs::create_dir_all(demo_dir);

    // Define assets to fetch
    let assets = [
        (
            "https://unpkg.com/gridjs@6.0.6/dist/gridjs.umd.js",
            "gridjs.min.js",
        ),
        (
            "https://unpkg.com/gridjs@6.0.6/dist/theme/mermaid.min.css",
            "gridjs-mermaid.min.css",
        ),
        // Mermaid.js already exists from previous implementation
    ];

    // Download and compress vendor assets
    for (url, filename) in &assets {
        let path = vendor_dir.join(filename);
        let gz_path = vendor_dir.join(format!("{}.gz", filename));

        // Skip if already compressed
        if gz_path.exists() {
            continue;
        }

        // Check if file already exists uncompressed
        if !path.exists() {
            println!("cargo:warning=Downloading {} from {}", filename, url);

            match ureq::get(url).call() {
                Ok(response) => {
                    let mut content = Vec::new();
                    if let Err(e) = response
                        .into_reader()
                        .take(10_000_000) // 10MB limit
                        .read_to_end(&mut content)
                    {
                        println!("cargo:warning=Failed to read {}: {}", filename, e);
                        continue;
                    }
                    if let Err(e) = fs::write(&path, &content) {
                        println!("cargo:warning=Failed to write {}: {}", filename, e);
                        continue;
                    }
                }
                Err(e) => {
                    println!(
                        "cargo:warning=Failed to download {}: {}. Using placeholder.",
                        filename, e
                    );
                    // Create a placeholder file
                    let _ = fs::write(&path, b"/* Asset download failed during build */");
                }
            }
        }

        // Compress the file
        if path.exists() {
            if let Ok(input) = fs::read(&path) {
                let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
                if let Ok(()) = encoder.write_all(&input) {
                    if let Ok(compressed) = encoder.finish() {
                        if let Ok(()) = fs::write(&gz_path, compressed) {
                            if let Ok(metadata) = fs::metadata(&gz_path) {
                                println!(
                                    "cargo:warning=Compressed {} ({} -> {} bytes)",
                                    filename,
                                    input.len(),
                                    metadata.len()
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    // Calculate hash for cache busting
    let hash = calculate_asset_hash();
    println!("cargo:rustc-env=ASSET_HASH={}", hash);
}

fn calculate_asset_hash() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();

    // Hash all asset files
    if let Ok(entries) = fs::read_dir("assets/vendor") {
        for entry in entries.filter_map(Result::ok) {
            if let Ok(content) = fs::read(entry.path()) {
                content.hash(&mut hasher);
            }
        }
    }

    format!("{:x}", hasher.finish())
}
