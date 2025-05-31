use std::env;
use std::fs;
use std::io::Write;
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
    setup_asset_directories();
    let assets = get_asset_definitions();
    process_assets(&assets);
    set_asset_hash_env();
}

fn setup_asset_directories() {
    let vendor_dir = Path::new("assets/vendor");
    let demo_dir = Path::new("assets/demo");
    let _ = fs::create_dir_all(vendor_dir);
    let _ = fs::create_dir_all(demo_dir);
}

fn get_asset_definitions() -> [(&'static str, &'static str); 2] {
    [
        (
            "https://unpkg.com/gridjs@6.0.6/dist/gridjs.umd.js",
            "gridjs.min.js",
        ),
        (
            "https://unpkg.com/gridjs@6.0.6/dist/theme/mermaid.min.css",
            "gridjs-mermaid.min.css",
        ),
        // Mermaid.js already exists from previous implementation
    ]
}

fn process_assets(assets: &[(&str, &str)]) {
    let vendor_dir = Path::new("assets/vendor");

    for (url, filename) in assets {
        let path = vendor_dir.join(filename);
        let gz_path = vendor_dir.join(format!("{}.gz", filename));

        if should_skip_asset(&gz_path) {
            continue;
        }

        ensure_asset_downloaded(&path, url, filename);
        compress_asset(&path, &gz_path, filename);
    }
}

fn should_skip_asset(gz_path: &Path) -> bool {
    gz_path.exists()
}

fn ensure_asset_downloaded(path: &Path, url: &str, filename: &str) {
    if !path.exists() {
        download_asset(url, path, filename);
    }
}

fn download_asset(url: &str, path: &Path, filename: &str) {
    println!("cargo:warning=Downloading {} from {}", filename, url);

    match ureq::get(url).call() {
        Ok(mut response) => match response.body_mut().read_to_vec() {
            Ok(content) => {
                if let Err(e) = fs::write(path, &content) {
                    println!("cargo:warning=Failed to write {}: {}", filename, e);
                }
            }
            Err(e) => {
                println!("cargo:warning=Failed to read {}: {}", filename, e);
                let _ = fs::write(path, b"/* Asset download failed during build */");
            }
        },
        Err(e) => {
            handle_download_failure(e, path, filename);
        }
    }
}

fn handle_download_failure(e: ureq::Error, path: &Path, filename: &str) {
    println!(
        "cargo:warning=Failed to download {}: {}. Using placeholder.",
        filename, e
    );
    // Create a placeholder file
    let _ = fs::write(path, b"/* Asset download failed during build */");
}

fn compress_asset(path: &Path, gz_path: &Path, filename: &str) {
    if !path.exists() {
        return;
    }

    let input = match fs::read(path) {
        Ok(data) => data,
        Err(_) => return,
    };

    let compressed = match create_compressed_data(&input) {
        Some(data) => data,
        None => return,
    };

    write_compressed_file(gz_path, &compressed, filename, input.len());
}

fn create_compressed_data(input: &[u8]) -> Option<Vec<u8>> {
    use flate2::write::GzEncoder;
    use flate2::Compression;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(input).ok()?;
    encoder.finish().ok()
}

fn write_compressed_file(gz_path: &Path, compressed: &[u8], filename: &str, original_size: usize) {
    if fs::write(gz_path, compressed).is_ok() {
        if let Ok(metadata) = fs::metadata(gz_path) {
            println!(
                "cargo:warning=Compressed {} ({} -> {} bytes)",
                filename,
                original_size,
                metadata.len()
            );
        }
    }
}

fn set_asset_hash_env() {
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
