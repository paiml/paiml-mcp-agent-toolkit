#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=src/installer/mod.rs");
    println!("cargo:rerun-if-changed=../scripts/install.sh");
    println!("cargo:rerun-if-changed=assets/vendor/");
    println!("cargo:rerun-if-changed=assets/demo/");
    println!("cargo:rerun-if-changed=../assets/demo/");
    println!("cargo:rerun-if-changed=templates/");
    println!("cargo:rerun-if-changed=src/schema/refactor_state.capnp");

    // Declare custom cfg flag for cargo publish detection
    println!("cargo:rustc-check-cfg=cfg(cargo_publish)");

    // Verify critical dependencies at build time
    verify_dependency_versions();

    // Compress templates at build time
    compress_templates();

    // Download and compress assets for demo mode
    // Skip asset downloading during cargo publish to avoid modifying source directory
    if env::var("CARGO_FEATURE_DEMO").is_ok() && !is_publishing() {
        download_and_compress_assets();
        minify_demo_assets();
    }

    // Compile Cap'n Proto schema for MCP server
    compile_capnp_schema();
}

/// Check if we're in a cargo publish context
fn is_publishing() -> bool {
    // During cargo publish, the package is extracted to a temp directory
    let is_publish = env::var("CARGO_PKG_VERSION").is_ok()
        && env::current_dir()
            .map(|dir| dir.to_string_lossy().contains("/target/package/"))
            .unwrap_or(false);

    if is_publish {
        println!("cargo:rustc-cfg=cargo_publish");
    }

    is_publish
}

/// Verifies critical dependencies exist in Cargo.lock
///
/// # Panics
///
/// Panics if Cargo.lock doesn't exist or critical dependencies are missing
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
        assert!(
            lock_content.contains(&format!("name = \"{dep}\"")),
            "Critical dependency {dep} not found"
        );
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

const fn get_asset_definitions() -> [(&'static str, &'static str); 4] {
    [
        (
            "https://unpkg.com/gridjs@6.0.6/dist/gridjs.umd.js",
            "gridjs.min.js",
        ),
        (
            "https://unpkg.com/gridjs@6.0.6/dist/theme/mermaid.min.css",
            "gridjs-mermaid.min.css",
        ),
        (
            "https://unpkg.com/mermaid@latest/dist/mermaid.min.js",
            "mermaid.min.js",
        ),
        ("https://unpkg.com/d3@latest/dist/d3.min.js", "d3.min.js"),
    ]
}

fn process_assets(assets: &[(&str, &str)]) {
    let vendor_dir = Path::new("assets/vendor");

    for (url, filename) in assets {
        let path = vendor_dir.join(filename);
        let gz_path = vendor_dir.join(format!("{filename}.gz"));

        if should_skip_asset(&gz_path) {
            continue;
        }

        ensure_asset_downloaded(&path, &gz_path, url, filename);
        compress_asset(&path, &gz_path, filename);
    }
}

fn should_skip_asset(gz_path: &Path) -> bool {
    gz_path.exists()
}

fn ensure_asset_downloaded(path: &Path, gz_path: &Path, url: &str, filename: &str) {
    if !path.exists() {
        // Check if we're in a docs.rs build environment
        if env::var("DOCS_RS").is_ok() {
            println!("cargo:warning=Skipping asset download in docs.rs environment: {filename}");
            // Create a placeholder file for docs.rs builds
            let _ = fs::write(path, b"/* Asset skipped in docs.rs build */");
            // Also create an empty gzipped placeholder to satisfy include_bytes!
            let _ = fs::write(gz_path, b"");
        } else {
            download_asset(url, path, filename);
        }
    }
}

fn download_asset(url: &str, path: &Path, filename: &str) {
    println!("cargo:warning=Downloading {filename} from {url}");

    match ureq::get(url).call() {
        Ok(mut response) => match response.body_mut().read_to_vec() {
            Ok(content) => {
                if let Err(e) = fs::write(path, &content) {
                    println!("cargo:warning=Failed to write {filename}: {e}");
                }
            }
            Err(e) => {
                println!("cargo:warning=Failed to read {filename}: {e}");
                let _ = fs::write(path, b"/* Asset download failed during build */");
            }
        },
        Err(e) => {
            handle_download_failure(&e, path, filename);
        }
    }
}

fn handle_download_failure(e: &ureq::Error, path: &Path, filename: &str) {
    println!("cargo:warning=Failed to download {filename}: {e}. Using placeholder.");
    // Create a placeholder file
    let _ = fs::write(path, b"/* Asset download failed during build */");
}

fn compress_asset(path: &Path, gz_path: &Path, filename: &str) {
    if !path.exists() {
        return;
    }

    let Ok(input) = fs::read(path) else { return };

    let Some(compressed) = create_compressed_data(&input) else {
        return;
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
    println!("cargo:rustc-env=ASSET_HASH={hash}");
}

/// Compresses template files at build time
///
/// # Panics
///
/// Panics if `OUT_DIR` environment variable is not set
fn compress_templates() {
    use std::collections::HashMap;

    let templates_dir = Path::new("templates");
    if !templates_dir.exists() {
        println!("cargo:warning=Templates directory not found, skipping compression");
        return;
    }

    let mut templates = HashMap::new();
    let mut total_original = 0usize;

    // Recursively collect all template files
    if let Ok(entries) = collect_template_files(templates_dir) {
        for entry in entries {
            if let Some((name, content)) = read_template_file(&entry) {
                total_original += content.len();
                templates.insert(name, content);
            }
        }
    }

    if templates.is_empty() {
        println!("cargo:warning=No templates found for compression");
        return;
    }

    // Compress all templates together
    let serialized = serde_json_to_string(&templates);
    if let Some(compressed) = create_compressed_data(serialized.as_bytes()) {
        let total_compressed = compressed.len();

        // Generate compressed template constants
        let compressed_hex = generate_hex_string(&compressed);
        let template_code = generate_template_code(&compressed_hex, templates.len());

        // Write to output file
        let out_dir = env::var("OUT_DIR").unwrap();
        let dest_path = Path::new(&out_dir).join("compressed_templates.rs");
        if fs::write(dest_path, template_code).is_ok() {
            #[allow(clippy::cast_precision_loss)]
            let reduction_percent = (1.0 - total_compressed as f64 / total_original as f64) * 100.0;
            println!(
                "cargo:warning=Compressed {} templates ({} -> {} bytes, {:.1}% reduction)",
                templates.len(),
                total_original,
                total_compressed,
                reduction_percent
            );
        }
    }
}

/// Collects template files from directory
///
/// # Errors
///
/// Returns error if directory cannot be read
fn collect_template_files(dir: &Path) -> Result<Vec<std::path::PathBuf>, std::io::Error> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            files.extend(collect_template_files(&path)?);
        } else if path
            .extension()
            .is_some_and(|ext| ext == "hbs" || ext == "json")
        {
            files.push(path);
        }
    }
    Ok(files)
}

fn read_template_file(path: &Path) -> Option<(String, String)> {
    let name = path
        .strip_prefix("templates")
        .ok()?
        .to_string_lossy()
        .to_string();
    let content = fs::read_to_string(path).ok()?;
    Some((name, content))
}

fn serde_json_to_string<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string())
}

fn generate_hex_string(data: &[u8]) -> String {
    data.iter().fold(String::new(), |mut acc, b| {
        use std::fmt::Write;
        let _ = write!(acc, "{b:02x}");
        acc
    })
}

fn generate_template_code(hex: &str, count: usize) -> String {
    format!(
        r#"// Auto-generated compressed templates
use std::collections::HashMap;
use once_cell::sync::Lazy;

const COMPRESSED_TEMPLATES: &str = "{hex}";

pub static TEMPLATES: Lazy<HashMap<String, String>> = Lazy::new(|| {{
    use flate2::read::GzDecoder;
    use std::io::Read;
    
    let compressed = hex::decode(COMPRESSED_TEMPLATES).expect("Valid hex");
    let mut decoder = GzDecoder::new(&compressed[..]);
    let mut decompressed = String::new();
    decoder.read_to_string(&mut decompressed).expect("Decompression failed");
    
    serde_json::from_str(&decompressed).expect("Valid JSON")
}});

// Template count: {count}
"#
    )
}

fn minify_demo_assets() {
    println!("cargo:warning=Minifying demo assets...");

    let demo_dir = Path::new("../assets/demo");
    let output_dir = Path::new("assets/demo");
    let _ = fs::create_dir_all(output_dir);

    // Minify JavaScript
    minify_js_file(&demo_dir.join("app.js"), &output_dir.join("app.min.js"));

    // Minify CSS
    minify_css_file(
        &demo_dir.join("style.css"),
        &output_dir.join("style.min.css"),
    );

    // Copy other demo assets as-is
    copy_demo_asset(
        &demo_dir.join("favicon.ico"),
        &output_dir.join("favicon.ico"),
    );
}

fn minify_js_file(input_path: &Path, output_path: &Path) {
    if !input_path.exists() {
        println!(
            "cargo:warning=JavaScript file not found: {}",
            input_path.display()
        );
        return;
    }

    let content = match fs::read_to_string(input_path) {
        Ok(content) => content,
        Err(e) => {
            println!("cargo:warning=Failed to read JS file: {e}");
            return;
        }
    };

    let minified = simple_js_minify(&content);

    if let Err(e) = fs::write(output_path, &minified) {
        println!("cargo:warning=Failed to write minified JS: {e}");
        return;
    }

    #[allow(clippy::cast_precision_loss)]
    let reduction = (1.0 - minified.len() as f64 / content.len() as f64) * 100.0;
    println!(
        "cargo:warning=Minified JavaScript: {} -> {} bytes ({:.1}% reduction)",
        content.len(),
        minified.len(),
        reduction
    );
}

fn minify_css_file(input_path: &Path, output_path: &Path) {
    if !input_path.exists() {
        println!("cargo:warning=CSS file not found: {}", input_path.display());
        return;
    }

    let content = match fs::read_to_string(input_path) {
        Ok(content) => content,
        Err(e) => {
            println!("cargo:warning=Failed to read CSS file: {e}");
            return;
        }
    };

    let minified = simple_css_minify(&content);

    if let Err(e) = fs::write(output_path, &minified) {
        println!("cargo:warning=Failed to write minified CSS: {e}");
        return;
    }

    #[allow(clippy::cast_precision_loss)]
    let reduction = (1.0 - minified.len() as f64 / content.len() as f64) * 100.0;
    println!(
        "cargo:warning=Minified CSS: {} -> {} bytes ({:.1}% reduction)",
        content.len(),
        minified.len(),
        reduction
    );
}

fn copy_demo_asset(input_path: &Path, output_path: &Path) {
    if input_path.exists() {
        let _ = fs::copy(input_path, output_path);
    }
}

fn simple_js_minify(content: &str) -> String {
    content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("//"))
        .collect::<Vec<_>>()
        .join(" ")
        .replace("; ", ";")
        .replace(", ", ",")
        .replace(" = ", "=")
        .replace(" + ", "+")
        .replace(" { ", "{")
        .replace(" } ", "}")
        .replace("{ ", "{")
        .replace(" }", "}")
}

fn simple_css_minify(content: &str) -> String {
    content
        .lines()
        .map(|line| {
            let line = line.trim();
            // Remove CSS comments
            if line.starts_with("/*") && line.ends_with("*/") {
                ""
            } else {
                line
            }
        })
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("")
        .replace("; ", ";")
        .replace(": ", ":")
        .replace(", ", ",")
        .replace(" { ", "{")
        .replace(" } ", "}")
        .replace("{ ", "{")
        .replace(" }", "}")
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

    // Also hash demo assets
    if let Ok(entries) = fs::read_dir("assets/demo") {
        for entry in entries.filter_map(Result::ok) {
            if let Ok(content) = fs::read(entry.path()) {
                content.hash(&mut hasher);
            }
        }
    }

    format!("{:x}", hasher.finish())
}

/// Compiles Cap'n Proto schema for MCP server
fn compile_capnp_schema() {
    // Only compile schema if MCP server feature is enabled or explicitly requested
    if env::var("CARGO_FEATURE_MCP_SERVER").is_ok() || env::var("PMAT_BUILD_MCP").is_ok() {
        let schema_path = Path::new("src/schema/refactor_state.capnp");

        if schema_path.exists() {
            println!("cargo:warning=Compiling Cap'n Proto schema for MCP server");

            let out_dir = env::var("OUT_DIR").expect("OUT_DIR environment variable must be set");

            // Use capnpc to compile the schema
            match capnpc::CompilerCommand::new()
                .src_prefix("src/schema")
                .file("src/schema/refactor_state.capnp")
                .output_path(&out_dir)
                .run()
            {
                Ok(_) => {
                    println!("cargo:warning=Successfully compiled Cap'n Proto schema");
                }
                Err(e) => {
                    // Don't fail the build if Cap'n Proto compilation fails
                    // The code will fall back to JSON serialization
                    println!("cargo:warning=Failed to compile Cap'n Proto schema: {}. Using JSON fallback.", e);
                }
            }
        } else {
            println!("cargo:warning=Cap'n Proto schema file not found, skipping compilation");
        }
    }
}
