#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

fn main() {
    // Embed the source revision so a binary can be checked against the tree it
    // claims to come from (see `emit_build_provenance`).
    emit_build_provenance();

    // Only watch files that actually exist - missing files cause constant rebuilds
    println!("cargo:rerun-if-changed=assets/vendor/");
    println!("cargo:rerun-if-changed=assets/demo/");
    println!("cargo:rerun-if-changed=../assets/demo/");
    println!("cargo:rerun-if-changed=templates/");
    println!("cargo:rerun-if-changed=src/schema/refactor_state.capnp");
    println!("cargo:rerun-if-env-changed=PMAT_FAST_BUILD");
    println!("cargo:rerun-if-env-changed=CARGO_LLVM_COV");
    println!("cargo:rerun-if-env-changed=SKIP_MCP_TABLES");

    // Declare custom cfg flags
    println!("cargo:rustc-check-cfg=cfg(cargo_publish)");
    println!("cargo:rustc-check-cfg=cfg(coverage)");
    println!("cargo:rustc-check-cfg=cfg(coverage_attr_stable)");
    println!("cargo:rustc-check-cfg=cfg(kani)");

    // GH-283: `coverage_attribute` was stabilized in Rust 1.94. Using
    // `#![feature(coverage_attribute)]` on 1.94+ stable triggers E0554.
    // Emit `coverage_attr_stable` so lib/bin headers can skip the feature
    // gate when the attribute is already stabilized.
    if rustc_is_at_least_1_94() {
        println!("cargo:rustc-cfg=coverage_attr_stable");
    }

    // Fast build mode for development - skip heavy operations but generate stubs
    if env::var("PMAT_FAST_BUILD").is_ok() {
        println!("cargo:warning=Fast build mode enabled - skipping heavy build operations");
        let out_dir = env::var("OUT_DIR").expect("OUT_DIR must be set");
        generate_stub_files(&out_dir);
        return;
    }

    // Emit CONTRACT_* env vars from binding.yaml for #[contract] proc macro
    emit_contract_env_vars();

    // KAIZEN-0178: Generate MCP tool schema metadata from JSON files.
    // Walks `mcp_tool_schemas/` and emits one authoritative Rust module so
    // handlers cannot silently advertise empty `inputSchema`. Missing JSON →
    // `include_str!` fails at compile time (the design constraint).
    generate_mcp_tool_schemas();

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
    // compile_capnp_schema(); // REMOVED: Cap'n Proto dependency eliminated (unused bloat)

    // Generate MCP discovery optimization tables
    // Skip during coverage builds to prevent hangs
    if env::var("CARGO_LLVM_COV").is_err() && env::var("SKIP_MCP_TABLES").is_err() {
        generate_mcp_discovery_tables();
    } else {
        println!("cargo:warning=Skipping MCP discovery table generation (coverage build or SKIP_MCP_TABLES set)");
        // Generate stub files to allow compilation
        let out_dir = env::var("OUT_DIR").expect("OUT_DIR must be set");
        generate_stub_files(&out_dir);
    }
}

/// GH-283: Return true when the compiler is Rust >= 1.94.
///
/// Parses `rustc --version` output (e.g. `rustc 1.94.1 (e408947bf 2026-03-25)`)
/// and compares the reported `minor` version against the 1.94 cutoff. Returns
/// false on any parse error so older compilers keep the feature gate.
fn rustc_is_at_least_1_94() -> bool {
    let rustc = env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let Ok(output) = std::process::Command::new(rustc).arg("--version").output() else {
        return false;
    };
    if !output.status.success() {
        return false;
    }
    let Ok(stdout) = std::str::from_utf8(&output.stdout) else {
        return false;
    };
    let Some(version) = stdout.split_whitespace().nth(1) else {
        return false;
    };
    let mut parts = version.split('.');
    let Some(major) = parts.next().and_then(|s| s.parse::<u32>().ok()) else {
        return false;
    };
    let Some(minor) = parts.next().and_then(|s| s.parse::<u32>().ok()) else {
        return false;
    };
    major > 1 || (major == 1 && minor >= 94)
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
        "tokio",     // Async runtime
        "serde",     // Serialization
        "minijinja", // Template engine
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
        let hash_path = vendor_dir.join(format!("{filename}.hash"));

        if should_skip_asset(&path, &gz_path, &hash_path) {
            println!("cargo:warning=Skipping unchanged asset: {filename} (O(1) hash check)");
            continue;
        }

        ensure_asset_downloaded(&path, &gz_path, url, filename);
        compress_asset(&path, &gz_path, &hash_path, filename);
    }
}

fn should_skip_asset(source_path: &Path, gz_path: &Path, hash_path: &Path) -> bool {
    // O(1) optimization: Skip if output exists AND source hasn't changed
    if !gz_path.exists() || !source_path.exists() {
        return false;
    }

    // Hash-based check: O(1) for unchanged files
    !has_file_changed(source_path, hash_path)
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

fn compress_asset(path: &Path, gz_path: &Path, hash_path: &Path, filename: &str) {
    if !path.exists() {
        return;
    }

    let Ok(input) = fs::read(path) else { return };

    let Some(compressed) = create_compressed_data(&input) else {
        return;
    };

    write_compressed_file(gz_path, &compressed, filename, input.len());

    // Save hash for O(1) skip detection on next build
    if let Some(hash) = calculate_file_hash(path) {
        let _ = write_hash_file(hash_path, &hash);
    }
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
    let templates_dir = Path::new("templates");

    if !validate_templates_directory(templates_dir) {
        return;
    }

    // O(1) optimization: Skip if templates haven't changed
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR must be set");
    let hash_path = Path::new(&out_dir).join("templates.hash");

    let current_hash = calculate_templates_hash(templates_dir);

    // O(1) skip when available (standard-deps enabled and hash matches)
    if let Some(h) = &current_hash {
        if let Some(stored_hash) = read_stored_hash(&hash_path) {
            if *h == stored_hash {
                println!("cargo:warning=Skipping unchanged templates (O(1) hash check)");
                return;
            }
        }
    }

    let (templates, total_original) = load_all_templates(templates_dir);

    if templates.is_empty() {
        println!("cargo:warning=No templates found for compression");
        return;
    }

    compress_and_save_templates(&templates, total_original);

    // Save hash for O(1) skip detection on next build (best-effort)
    if let Some(h) = current_hash {
        let _ = write_hash_file(&hash_path, &h);
    }
}

/// Calculate combined hash of all template files.
///
/// Returns `None` under `--no-default-features` (no `sha2` in build-deps);
/// callers fall back to unconditional recompression, losing only the O(1)
/// skip optimization.
#[cfg(feature = "standard-deps")]
fn calculate_templates_hash(templates_dir: &Path) -> Option<String> {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    let mut file_count = 0;

    if let Ok(files) = collect_template_files(templates_dir) {
        // Sort files for deterministic hashing
        let mut sorted_files = files;
        sorted_files.sort();

        for file in sorted_files {
            if let Ok(content) = fs::read(&file) {
                // Include filename in hash for renames
                hasher.update(file.to_string_lossy().as_bytes());
                hasher.update(&content);
                file_count += 1;
            }
        }
    }

    if file_count == 0 {
        return None;
    }

    // sha2 0.11's finalize() returns an Array<u8> that no longer impls LowerHex;
    // encode the digest bytes to lowercase hex explicitly.
    Some(
        hasher
            .finalize()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect(),
    )
}

#[cfg(not(feature = "standard-deps"))]
fn calculate_templates_hash(_templates_dir: &Path) -> Option<String> {
    None
}

/// Validate templates directory exists (cognitive complexity ≤2)
fn validate_templates_directory(templates_dir: &Path) -> bool {
    if templates_dir.exists() {
        true
    } else {
        println!("cargo:warning=Templates directory not found, skipping compression");
        false
    }
}

/// Load all template files and return templates map with total size (cognitive complexity ≤6)
fn load_all_templates(templates_dir: &Path) -> (std::collections::HashMap<String, String>, usize) {
    use std::collections::HashMap;
    let mut templates = HashMap::new();
    let mut total_original = 0usize;

    if let Ok(entries) = collect_template_files(templates_dir) {
        for entry in entries {
            if let Some((name, content)) = read_template_file(&entry) {
                total_original += content.len();
                templates.insert(name, content);
            }
        }
    }

    (templates, total_original)
}

/// Compress templates and save to output file (cognitive complexity ≤8)
fn compress_and_save_templates(
    templates: &std::collections::HashMap<String, String>,
    total_original: usize,
) {
    let serialized = serde_json_to_string(templates);

    if let Some(compressed) = create_compressed_data(serialized.as_bytes()) {
        let total_compressed = compressed.len();
        let template_code = generate_template_output(&compressed, templates.len());

        write_compressed_templates_file(&template_code);
        print_compression_stats(templates.len(), total_original, total_compressed);
    }
}

/// Generate template output code (cognitive complexity ≤2)
fn generate_template_output(compressed: &[u8], template_count: usize) -> String {
    let compressed_hex = generate_hex_string(compressed);
    generate_template_code(&compressed_hex, template_count)
}

/// Write compressed templates to output file (cognitive complexity ≤2)
fn write_compressed_templates_file(template_code: &str) {
    let out_dir = env::var("OUT_DIR")
        .expect("OUT_DIR environment variable must be set by Cargo during build");
    let dest_path = Path::new(&out_dir).join("compressed_templates.rs");
    let _ = write_if_changed(&dest_path, template_code);
}

/// Print compression statistics (cognitive complexity ≤3)
fn print_compression_stats(template_count: usize, total_original: usize, total_compressed: usize) {
    #[allow(clippy::cast_precision_loss)]
    let reduction_percent = (1.0 - total_compressed as f64 / total_original as f64) * 100.0;

    println!(
        "cargo:warning=Compressed {template_count} templates ({total_original} -> {total_compressed} bytes, {reduction_percent:.1}% reduction)"
    );
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
use std::sync::LazyLock;

const COMPRESSED_TEMPLATES: &str = "{hex}";

fn hex_decode_templates(s: &str) -> Vec<u8> {{
    (0..s.len())
        .step_by(2)
        .filter_map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
}}

pub static TEMPLATES: LazyLock<HashMap<String, String>> = LazyLock::new(|| {{
    use flate2::read::GzDecoder;
    use std::io::Read;

    let compressed = hex_decode_templates(COMPRESSED_TEMPLATES);
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

    // O(1) optimization: Skip if unchanged
    let Some(file_name) = output_path.file_name() else {
        println!("cargo:warning=Invalid output path: no file name");
        return;
    };
    let hash_path = output_path.with_file_name(format!("{}.hash", file_name.to_string_lossy()));
    if output_path.exists() && !has_file_changed(input_path, &hash_path) {
        println!(
            "cargo:warning=Skipping unchanged JavaScript: {} (O(1) hash check)",
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

    // Save hash for O(1) skip detection on next build
    if let Some(hash) = calculate_file_hash(input_path) {
        let _ = write_hash_file(&hash_path, &hash);
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

    // O(1) optimization: Skip if unchanged
    let Some(file_name) = output_path.file_name() else {
        println!("cargo:warning=Invalid output path: no file name");
        return;
    };
    let hash_path = output_path.with_file_name(format!("{}.hash", file_name.to_string_lossy()));
    if output_path.exists() && !has_file_changed(input_path, &hash_path) {
        println!(
            "cargo:warning=Skipping unchanged CSS: {} (O(1) hash check)",
            input_path.display()
        );
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

    // Save hash for O(1) skip detection on next build
    if let Some(hash) = calculate_file_hash(input_path) {
        let _ = write_hash_file(&hash_path, &hash);
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

/// Calculate SHA256 hash of a file for change detection.
///
/// Returns `None` under `--no-default-features` (no `sha2` in build-deps);
/// callers treat this as "changed" and reprocess unconditionally.
#[cfg(feature = "standard-deps")]
fn calculate_file_hash(path: &Path) -> Option<String> {
    use sha2::{Digest, Sha256};

    let content = fs::read(path).ok()?;
    let mut hasher = Sha256::new();
    hasher.update(&content);
    // sha2 0.11's finalize() returns an Array<u8> that no longer impls LowerHex;
    // encode the digest bytes to lowercase hex explicitly.
    Some(
        hasher
            .finalize()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect(),
    )
}

#[cfg(not(feature = "standard-deps"))]
fn calculate_file_hash(_path: &Path) -> Option<String> {
    None
}

/// Read stored hash from .hash file
fn read_stored_hash(hash_path: &Path) -> Option<String> {
    fs::read_to_string(hash_path)
        .ok()
        .map(|s| s.trim().to_string())
}

/// Write hash to .hash file
fn write_hash_file(hash_path: &Path, hash: &str) -> bool {
    fs::write(hash_path, hash).is_ok()
}

/// Check if source file has changed by comparing hashes
fn has_file_changed(source_path: &Path, hash_path: &Path) -> bool {
    // If hash file doesn't exist, file has "changed" (needs processing)
    if !hash_path.exists() {
        return true;
    }

    // Calculate current hash
    let Some(current_hash) = calculate_file_hash(source_path) else {
        return true; // Can't read source, assume changed
    };

    // Compare with stored hash
    let Some(stored_hash) = read_stored_hash(hash_path) else {
        return true; // Can't read stored hash, assume changed
    };

    current_hash != stored_hash
}

/// Compiles Cap'n Proto schema for MCP server
/// DISABLED: Cap'n Proto removed (unused bloat dependency)
#[allow(dead_code)]
fn compile_capnp_schema() {
    // REMOVED: capnpc dependency eliminated (unused)
    // Reason: 0 references found in codebase, removing bloat
    println!("cargo:warning=Cap'n Proto schema compilation skipped (dependency removed)");
}

/// Generate MCP discovery optimization tables for <10ms initialization
fn generate_mcp_discovery_tables() {
    println!("cargo:warning=Generating MCP discovery optimization tables");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR must be set");

    // Generate tool registry
    generate_tool_registry(&out_dir);

    // Generate alias table
    generate_alias_table(&out_dir);

    // Generate trigram index
    generate_trigram_index(&out_dir);
}

/// Generate static PHF map of all MCP tools for zero-copy initialization
fn generate_tool_registry(out_dir: &str) {
    let dest_path = Path::new(out_dir).join("tool_registry.rs");

    // Tool definitions from the current MCP server
    let tools = vec![
        (
            "analyze_complexity",
            "Analyze code complexity metrics (cyclomatic, cognitive)",
            vec!["complexity", "analyze", "metrics"],
        ),
        (
            "analyze_satd",
            "Find self-admitted technical debt in comments",
            vec!["satd", "debt", "todo", "fixme"],
        ),
        (
            "analyze_dead_code",
            "Detect unused functions and variables",
            vec!["dead", "unused", "code"],
        ),
        (
            "analyze_dag",
            "Generate dependency graphs and visualizations",
            vec!["dependency", "graph", "dag", "architecture"],
        ),
        (
            "analyze_deep_context",
            "Generate comprehensive codebase context",
            vec!["context", "summary", "analysis"],
        ),
        (
            "analyze_big_o",
            "Analyze algorithmic complexity",
            vec!["big-o", "algorithm", "performance"],
        ),
        (
            "refactor.start",
            "Begin refactoring workflow",
            vec!["refactor", "start", "begin"],
        ),
        (
            "refactor.nextIteration",
            "Continue refactoring process",
            vec!["refactor", "next", "continue"],
        ),
        (
            "refactor.getState",
            "Get current refactoring state",
            vec!["refactor", "state", "status"],
        ),
        (
            "refactor.stop",
            "End refactoring workflow",
            vec!["refactor", "stop", "end"],
        ),
        (
            "quality_gate",
            "Run comprehensive quality analysis",
            vec!["quality", "gate", "check"],
        ),
        (
            "quality_proxy",
            "Intercept and validate code changes",
            vec!["quality", "proxy", "validate"],
        ),
        (
            "git_operation",
            "Execute git operations",
            vec!["git", "version", "control"],
        ),
        (
            "generate_context",
            "Generate AI-optimized context",
            vec!["generate", "context", "ai"],
        ),
        (
            "scaffold_project",
            "Create project scaffolding",
            vec!["scaffold", "create", "generate", "project"],
        ),
        // Sprint 31: TDG System MCP Tools
        (
            "tdg_system_diagnostics",
            "Get comprehensive TDG system diagnostics and health monitoring",
            vec!["tdg", "diagnostics", "health", "monitoring", "system"],
        ),
        (
            "tdg_storage_management",
            "Manage TDG storage operations (stats, cleanup, flush, migrate)",
            vec!["tdg", "storage", "management", "cleanup", "migrate"],
        ),
        (
            "tdg_analyze_with_storage",
            "Analyze files using transactional TDG storage with caching",
            vec!["tdg", "analyze", "storage", "transactional", "cache"],
        ),
        (
            "tdg_performance_metrics",
            "Get real-time TDG performance metrics and adaptive thresholds",
            vec!["tdg", "performance", "metrics", "adaptive", "thresholds"],
        ),
        (
            "tdg_configure_storage",
            "Configure and validate TDG storage backends",
            vec!["tdg", "configure", "storage", "backend", "sled", "rocksdb"],
        ),
        (
            "tdg_health_check",
            "Comprehensive TDG system health check with recommendations",
            vec!["tdg", "health", "check", "recommendations", "status"],
        ),
        // Phase 4: Organizational Intelligence Integration
        (
            "generate_defect_aware_prompt",
            "Generate context-aware AI prompts from organizational intelligence and defect patterns",
            vec!["prompt", "defect", "organizational", "intelligence", "oip", "ai"],
        ),
    ];

    let mut registry_code = String::from(
        "// Auto-generated tool registry for zero-copy MCP initialization\n\n\
         #[derive(Debug, Clone)]\n\
         pub struct ToolMeta {\n\
             pub name: &'static str,\n\
             pub description: &'static str,\n\
             pub keywords: &'static [&'static str],\n\
         }\n\n\
         pub static TOOL_REGISTRY: std::sync::LazyLock<std::collections::HashMap<&'static str, ToolMeta>> = std::sync::LazyLock::new(|| {\n\
             let mut m = std::collections::HashMap::new();\n"
    );

    for (name, desc, keywords) in &tools {
        registry_code.push_str(&format!(
            "    m.insert(\"{name}\", ToolMeta {{\n\
                 name: \"{name}\",\n\
                 description: \"{desc}\",\n\
                 keywords: &{keywords:?},\n\
             }});\n"
        ));
    }

    registry_code.push_str("    m\n});\n");

    if let Err(e) = write_if_changed(&dest_path, &registry_code) {
        println!("cargo:warning=Failed to write tool registry: {e}");
    }
}

/// Generate alias dispatch table from empirical usage patterns
fn generate_alias_table(out_dir: &str) {
    let dest_path = Path::new(out_dir).join("alias_table.rs");

    let aliases = vec![
        (
            "analyze_complexity",
            vec![
                "complexity",
                "cyclomatic",
                "cognitive",
                "analyze code",
                "code complexity",
                "mccabe",
                "sonar",
                "analyze",
                "metrics",
            ],
        ),
        (
            "analyze_satd",
            vec![
                "debt",
                "technical debt",
                "todo",
                "fixme",
                "hack",
                "satd",
                "find debt",
                "find todo",
                "self admitted",
                "admitted debt",
            ],
        ),
        (
            "analyze_dag",
            vec![
                "dependency",
                "dependencies",
                "graph",
                "visualize",
                "diagram",
                "show dependencies",
                "dependency graph",
                "architecture",
                "dag",
            ],
        ),
        (
            "scaffold_project",
            vec![
                "scaffold",
                "create",
                "generate",
                "make",
                "new",
                "init",
                "create project",
                "generate project",
                "new project",
                "project template",
            ],
        ),
        (
            "generate_context",
            vec![
                "context",
                "summary",
                "generate context",
                "ai context",
                "codebase context",
                "analyze codebase",
                "understand code",
            ],
        ),
        (
            "quality_gate",
            vec![
                "quality",
                "check quality",
                "quality check",
                "gate",
                "validate",
                "quality analysis",
                "code quality",
                "standards",
            ],
        ),
        (
            "refactor.start",
            vec![
                "refactor",
                "refactoring",
                "start refactor",
                "begin refactor",
                "improve code",
                "clean code",
                "restructure",
            ],
        ),
        (
            "git_operation",
            vec![
                "git",
                "version control",
                "commit",
                "branch",
                "merge",
                "git command",
                "source control",
            ],
        ),
        // Sprint 31: TDG System MCP Tool Aliases
        (
            "tdg_system_diagnostics",
            vec![
                "tdg diagnostics",
                "system health",
                "tdg health",
                "monitoring",
                "system status",
                "tdg status",
                "diagnostics",
                "health check",
            ],
        ),
        (
            "tdg_storage_management",
            vec![
                "tdg storage",
                "storage stats",
                "cleanup storage",
                "storage cleanup",
                "flush storage",
                "migrate storage",
                "storage migrate",
                "tdg cache",
            ],
        ),
        (
            "tdg_analyze_with_storage",
            vec![
                "tdg analyze",
                "analyze tdg",
                "transactional analysis",
                "cached analysis",
                "storage analysis",
                "tdg file",
            ],
        ),
        (
            "tdg_performance_metrics",
            vec![
                "tdg performance",
                "performance metrics",
                "adaptive thresholds",
                "tdg metrics",
                "system performance",
                "threshold management",
            ],
        ),
        (
            "tdg_configure_storage",
            vec![
                "configure tdg",
                "storage config",
                "backend config",
                "tdg backend",
                "sled config",
                "rocksdb config",
                "storage backend",
            ],
        ),
        (
            "tdg_health_check",
            vec![
                "tdg health",
                "system health",
                "health status",
                "tdg check",
                "system check",
                "health recommendations",
            ],
        ),
        // Phase 4: Organizational Intelligence Integration
        (
            "generate_defect_aware_prompt",
            vec![
                "defect aware",
                "ai prompt",
                "organizational intelligence",
                "oip prompt",
                "context prompt",
                "defect patterns",
                "quality prompt",
                "intelligent prompt",
                "org intelligence",
                "prompt generation",
            ],
        ),
    ];

    let mut alias_code = String::from(
        "// Auto-generated alias table for MCP tool discovery\n\n\
         pub static ALIAS_TABLE: std::sync::LazyLock<std::collections::HashMap<&'static str, Vec<&'static str>>> = std::sync::LazyLock::new(|| {\n\
             let mut m = std::collections::HashMap::new();\n"
    );

    for (tool, tool_aliases) in &aliases {
        alias_code.push_str(&format!(
            "    m.insert(\"{tool}\", vec!{tool_aliases:?});\n"
        ));
    }

    alias_code.push_str("    m\n});\n");

    if let Err(e) = write_if_changed(&dest_path, &alias_code) {
        println!("cargo:warning=Failed to write alias table: {e}");
    }
}

/// Generate trigram index for fuzzy matching
fn generate_trigram_index(out_dir: &str) {
    let dest_path = Path::new(out_dir).join("trigram_index.rs");

    let trigram_code = r#"// Auto-generated trigram index for fuzzy matching
pub struct TrigramIndex;

impl TrigramIndex {
    #[inline(always)]
    pub fn pack_trigram(s: &[u8]) -> u32 {
        if s.len() < 3 { return 0; }
        (s[0] as u32) | ((s[1] as u32) << 8) | ((s[2] as u32) << 16)
    }
    
    pub fn similarity_score(&self, query: &str, candidate: &str) -> f32 {
        let q_bytes = query.to_lowercase().into_bytes();
        let c_bytes = candidate.to_lowercase().into_bytes();
        
        if q_bytes.len() < 3 || c_bytes.len() < 3 {
            return 0.0;
        }
        
        // Collect query trigrams
        let mut q_trigrams = Vec::with_capacity(q_bytes.len().saturating_sub(2));
        for i in 0..q_bytes.len().saturating_sub(2) {
            q_trigrams.push(Self::pack_trigram(&q_bytes[i..i+3]));
        }
        
        // Collect candidate trigrams
        let mut c_trigrams = Vec::with_capacity(c_bytes.len().saturating_sub(2));
        for i in 0..c_bytes.len().saturating_sub(2) {
            c_trigrams.push(Self::pack_trigram(&c_bytes[i..i+3]));
        }
        
        // Count matches
        let mut matches = 0;
        for q_tri in &q_trigrams {
            if c_trigrams.contains(q_tri) {
                matches += 1;
            }
        }
        
        // Jaccard similarity coefficient
        let union_size = q_trigrams.len() + c_trigrams.len() - matches;
        if union_size == 0 { return 0.0; }
        
        matches as f32 / union_size as f32
    }
    
    pub fn find_best_match<'a>(&self, query: &str, candidates: &[(&'a str, &str)]) -> Option<(&'a str, f32)> {
        let mut best_match = ("", 0.0f32);
        
        for (name, description) in candidates {
            // Check both name and description
            let name_score = self.similarity_score(query, name);
            let desc_score = self.similarity_score(query, description) * 0.7; // Weight description lower
            let combined = name_score.max(desc_score);
            
            if combined > best_match.1 {
                best_match = (name, combined);
            }
        }
        
        if best_match.1 > 0.4 {  // Empirically determined threshold
            Some(best_match)
        } else {
            None
        }
    }
}
"#;

    if let Err(e) = write_if_changed(&dest_path, trigram_code) {
        println!("cargo:warning=Failed to write trigram index: {e}");
    }
}

/// Generate stub files for coverage builds to avoid compilation errors
/// Write file only if content changed (avoids triggering recompilation)
fn write_if_changed(path: &Path, content: &str) -> Result<(), std::io::Error> {
    if path.exists() {
        if let Ok(existing) = fs::read_to_string(path) {
            if existing == content {
                // Content unchanged - skip write to preserve mtime
                return Ok(());
            }
        }
    }
    // Content changed or file doesn't exist - write it
    println!("cargo:warning=Writing changed file: {}", path.display());
    fs::write(path, content)
}

fn write_stub(out_dir: &str, filename: &str, content: &str) {
    let dest = Path::new(out_dir).join(filename);
    if let Err(e) = write_if_changed(&dest, content) {
        println!("cargo:warning=Failed to write {filename} stub: {e}");
    }
}

fn generate_stub_files(out_dir: &str) {
    generate_stub_tool_registry(out_dir);
    generate_stub_alias_table(out_dir);
    generate_stub_trigram_index(out_dir);
    generate_stub_compressed_templates(out_dir);
}

fn generate_stub_tool_registry(out_dir: &str) {
    let tool_registry = r#"
// Functional tool registry for coverage builds
use std::collections::HashMap;
use std::sync::LazyLock;

pub struct ToolMeta {
    pub name: &'static str,
    pub description: &'static str,
    pub keywords: &'static [&'static str],
}

pub static TOOL_REGISTRY: LazyLock<HashMap<&'static str, ToolMeta>> =
    LazyLock::new(|| {
        let mut m = HashMap::new();
        m.insert("analyze_complexity", ToolMeta {
            name: "analyze_complexity",
            description: "Analyze code complexity metrics (cyclomatic, cognitive)",
            keywords: &["complexity", "analyze", "metrics"],
        });
        m.insert("quality_gate", ToolMeta {
            name: "quality_gate",
            description: "Run comprehensive quality analysis",
            keywords: &["quality", "gate", "check"],
        });
        m.insert("scaffold_project", ToolMeta {
            name: "scaffold_project",
            description: "Create project scaffolding",
            keywords: &["scaffold", "create", "generate", "project"],
        });
        m.insert("analyze_satd", ToolMeta {
            name: "analyze_satd",
            description: "Find self-admitted technical debt in comments",
            keywords: &["satd", "debt", "todo", "fixme"],
        });
        m.insert("analyze_dag", ToolMeta {
            name: "analyze_dag",
            description: "Generate dependency graphs and visualizations",
            keywords: &["dependency", "graph", "dag", "architecture"],
        });
        m.insert("generate_context", ToolMeta {
            name: "generate_context",
            description: "Generate AI-optimized context",
            keywords: &["generate", "context", "ai"],
        });
        m.insert("refactor.start", ToolMeta {
            name: "refactor.start",
            description: "Begin refactoring workflow",
            keywords: &["refactor", "start", "begin"],
        });
        m.insert("git_operation", ToolMeta {
            name: "git_operation",
            description: "Execute git operations",
            keywords: &["git", "version", "control"],
        });
        m.insert("analyze_dead_code", ToolMeta {
            name: "analyze_dead_code",
            description: "Detect unused functions and variables",
            keywords: &["dead", "unused", "code"],
        });
        m.insert("analyze_big_o", ToolMeta {
            name: "analyze_big_o",
            description: "Analyze algorithmic complexity",
            keywords: &["big-o", "algorithm", "performance"],
        });
        m.insert("analyze_deep_context", ToolMeta {
            name: "analyze_deep_context",
            description: "Generate comprehensive codebase context",
            keywords: &["context", "summary", "analysis"],
        });
        m.insert("refactor.nextIteration", ToolMeta {
            name: "refactor.nextIteration",
            description: "Continue refactoring process",
            keywords: &["refactor", "next", "continue"],
        });
        m.insert("refactor.getState", ToolMeta {
            name: "refactor.getState",
            description: "Get current refactoring state",
            keywords: &["refactor", "state", "status"],
        });
        m.insert("refactor.stop", ToolMeta {
            name: "refactor.stop",
            description: "End refactoring workflow",
            keywords: &["refactor", "stop", "end"],
        });
        m.insert("quality_proxy", ToolMeta {
            name: "quality_proxy",
            description: "Intercept and validate code changes",
            keywords: &["quality", "proxy", "validate"],
        });
        m
    });
"#;

    write_stub(out_dir, "tool_registry.rs", tool_registry);
}

fn generate_stub_alias_table(out_dir: &str) {
    let alias_table = r#"
// Functional alias table for coverage builds
pub static ALIAS_TABLE: std::sync::LazyLock<std::collections::HashMap<&'static str, Vec<&'static str>>> =
    std::sync::LazyLock::new(|| {
        let mut m = std::collections::HashMap::new();
        m.insert("analyze_complexity", vec!["complexity", "analyze", "metrics", "complxity", "complx"]);
        m.insert("analyze_satd", vec!["debt", "technical debt", "todo", "fixme"]);
        m.insert("analyze_dag", vec!["dependency", "dependencies", "graph", "show dependencies", "dependency graph"]);
        m.insert("scaffold_project", vec!["scaffold", "create", "generate", "create project", "scafold"]);
        m.insert("generate_context", vec!["context", "generate context", "ai context"]);
        m.insert("quality_gate", vec!["quality", "check quality", "quality check", "qualit"]);
        m.insert("refactor.start", vec!["refactor", "start refactor", "refactr"]);
        m.insert("git_operation", vec!["git", "version control"]);
        m
    });
"#;

    write_stub(out_dir, "alias_table.rs", alias_table);
}

fn generate_stub_trigram_index(out_dir: &str) {
    write_stub(out_dir, "trigram_index.rs", TRIGRAM_INDEX_STUB);
}

const TRIGRAM_INDEX_STUB: &str = r#"
// Functional trigram index for coverage builds
pub struct TrigramIndex;

impl TrigramIndex {
    #[inline(always)]
    pub fn pack_trigram(s: &[u8]) -> u32 {
        if s.len() < 3 { return 0; }
        (s[0] as u32) | ((s[1] as u32) << 8) | ((s[2] as u32) << 16)
    }

    fn collect_trigrams(bytes: &[u8]) -> Vec<u32> {
        (0..bytes.len().saturating_sub(2))
            .map(|i| Self::pack_trigram(&bytes[i..i+3]))
            .collect()
    }

    fn jaccard_trigram_score(q: &[u32], c: &[u32]) -> f32 {
        let matches = q.iter().filter(|t| c.contains(t)).count();
        let union = q.len() + c.len() - matches;
        if union == 0 { 0.0 } else { matches as f32 / union as f32 }
    }

    pub fn similarity_score(&self, query: &str, candidate: &str) -> f32 {
        let q_lower = query.to_lowercase();
        let c_lower = candidate.to_lowercase();
        if q_lower == c_lower { return 1.0; }
        if c_lower.contains(&q_lower) { return 0.8; }
        let q_bytes = q_lower.into_bytes();
        let c_bytes = c_lower.into_bytes();
        if q_bytes.len() < 3 || c_bytes.len() < 3 { return 0.0; }
        Self::jaccard_trigram_score(&Self::collect_trigrams(&q_bytes), &Self::collect_trigrams(&c_bytes))
    }

    pub fn find_best_match<'a>(&self, query: &str, candidates: &[(&'a str, &str)]) -> Option<(&'a str, f32)> {
        candidates.iter()
            .map(|(name, desc)| {
                let score = self.similarity_score(query, name)
                    .max(self.similarity_score(query, desc) * 0.7);
                (*name, score)
            })
            .filter(|(_, s)| *s > 0.4)
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
    }
}
"#;

fn generate_stub_compressed_templates(out_dir: &str) {
    let compressed_templates = r#"
// Stub compressed templates for fast/coverage builds
// Real compression happens in full build mode via compress_templates()

use std::sync::LazyLock;
use std::collections::HashMap;

pub static COMPRESSED_TEMPLATES: LazyLock<HashMap<&'static str, Vec<u8>>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    // Stub template data - "stub test data" as raw bytes
    m.insert("context.md.tera", b"stub test data".to_vec());
    m.insert("satd.md.tera", b"stub test data".to_vec());
    m
});
"#;

    write_stub(out_dir, "compressed_templates.rs", compressed_templates);
    // NOTE: AllImplemented binding enforcement now lives in the live build
    // path (`emit_contract_env_vars`, build.rs:main) targeting the in-tree
    // `contracts/binding.yaml`. The former block here was dead code — gated
    // behind `standard-deps`, reachable only from the fast-build stub path,
    // and pointed at the deprecated `../../provable-contracts` tree (audit
    // ALADR-008 / L1-1).
}

/// Emit CONTRACT_* env vars from contracts/binding.yaml.
///
/// Each binding with status=implemented generates a
/// `CONTRACT_<CONTRACT>_<EQUATION>=bound` env var that the
/// `#[contract]` proc macro reads at compile time.
fn emit_contract_env_vars() {
    let binding_path = Path::new("contracts/binding.yaml");
    println!("cargo:rerun-if-changed=contracts/binding.yaml");

    if !binding_path.exists() {
        return;
    }

    let content = match fs::read_to_string(binding_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    // Parse YAML manually (no serde_yaml in build.rs)
    let mut current_contract = String::new();
    let mut current_equation = String::new();
    let mut current_status = String::new();
    // AllImplemented gaps: bindings whose status is neither `implemented` nor
    // an acknowledged WIP marker. Any such binding fails the build (L1 poka-yoke).
    let mut gaps: Vec<String> = Vec::new();

    for line in content.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("- contract:") {
            emit_or_record_binding(
                &current_contract,
                &current_equation,
                &current_status,
                &mut gaps,
            );
            current_contract = rest.trim().to_string();
            current_equation.clear();
            current_status.clear();
        } else if let Some(rest) = t.strip_prefix("equation:") {
            current_equation = rest.trim().to_string();
        } else if let Some(rest) = t.strip_prefix("status:") {
            current_status = rest.trim().to_string();
        }
    }
    // Emit last
    emit_or_record_binding(
        &current_contract,
        &current_equation,
        &current_status,
        &mut gaps,
    );

    // AllImplemented (L1 poka-yoke, audit ALADR-008): the live build path now
    // enforces the in-tree `contracts/binding.yaml` on every `cargo build`.
    // This replaces the dead panic that was buried in the fast-build stub
    // branch and targeted the deprecated `../../provable-contracts` path.
    if !gaps.is_empty() {
        for g in &gaps {
            println!("cargo:warning=[contract] AllImplemented gap: {g}");
        }
        panic!(
            "[contract] AllImplemented: {} binding(s) with a disallowed status \
             (expected implemented | pending | partial). Fix the binding, mark \
             it `pending`, or remove it from contracts/binding.yaml.",
            gaps.len()
        );
    }
}

/// Emit a `=bound` env var for an implemented binding, or record an
/// AllImplemented gap for a disallowed status. `pending`/`partial` are
/// acknowledged work-in-progress and neither bind nor fail; any other value
/// (notably `not_implemented` or a typo'd status) is a gap.
fn emit_or_record_binding(contract: &str, equation: &str, status: &str, gaps: &mut Vec<String>) {
    if contract.is_empty() || equation.is_empty() {
        return;
    }
    let key = make_contract_env_key(contract, equation);
    match status {
        "implemented" => println!("cargo:rustc-env={key}=bound"),
        "pending" | "partial" => {}
        other => gaps.push(format!("{key} (status: '{other}')")),
    }
}

/// KAIZEN-0178: Build-time MCP tool schema code generation.
///
/// Reads every `mcp_tool_schemas/*.json` file, validates that it parses as
/// JSON with `{name, description, inputSchema}`, and emits
/// `$OUT_DIR/mcp_tool_schemas_gen.rs`. The emitted code exposes:
///
/// * `pub static MCP_TOOL_SCHEMAS: &[(&str, &str)]` — slice of
///   `(tool_name, raw_json_string)` for every schema file found.
/// * Per-tool `include_str!` constants so the macro layer and any
///   registration audit can reference them by tool name.
///
/// If a schema file referenced by handler code is deleted, the `include_str!`
/// it expanded to will fail with `E0432`/`E0433` style errors at compile time
/// — satisfying the "missing schema = compile error" acceptance constraint.
fn generate_mcp_tool_schemas() {
    let schema_dir = Path::new("mcp_tool_schemas");
    println!("cargo:rerun-if-changed=mcp_tool_schemas");

    if !schema_dir.exists() {
        println!("cargo:warning=mcp_tool_schemas/ not found — skipping codegen");
        return;
    }

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR must be set");
    let dest = Path::new(&out_dir).join("mcp_tool_schemas_gen.rs");

    let mut entries: Vec<(String, std::path::PathBuf)> = Vec::new();

    let Ok(read_dir) = fs::read_dir(schema_dir) else {
        println!("cargo:warning=Failed to read mcp_tool_schemas/");
        return;
    };

    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "json") {
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            if stem.is_empty() {
                continue;
            }
            println!("cargo:rerun-if-changed={}", path.display());
            validate_tool_schema(&path, &stem);
            entries.push((stem, path));
        }
    }

    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let code = render_tool_schema_module(&entries);
    if let Err(e) = write_if_changed(&dest, &code) {
        panic!("KAIZEN-0178: failed to write generated tool schema module: {e}");
    }
    println!(
        "cargo:warning=KAIZEN-0178: generated {} MCP tool schema(s)",
        entries.len()
    );
}

/// Validate a single schema JSON file at build time.
///
/// Enforces the invariant that every schema declares `name`,
/// `description`, and `inputSchema.type == "object"`. Mismatches panic the
/// build — the whole point is to catch bad schemas before they reach
/// runtime `tools/list`.
fn validate_tool_schema(path: &Path, stem: &str) {
    let content =
        fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let parsed: serde_json::Value = serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("{} is not valid JSON: {e}", path.display()));

    let name = parsed
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("{}: missing `name`", path.display()));
    assert_eq!(
        name,
        stem,
        "{}: `name` field ({name}) must match file stem ({stem})",
        path.display()
    );
    assert!(
        parsed
            .get("description")
            .and_then(|v| v.as_str())
            .is_some_and(|s| !s.is_empty()),
        "{}: `description` must be a non-empty string",
        path.display()
    );
    let input_schema = parsed
        .get("inputSchema")
        .unwrap_or_else(|| panic!("{}: missing `inputSchema`", path.display()));
    let ty = input_schema.get("type").and_then(|v| v.as_str());
    assert_eq!(
        ty,
        Some("object"),
        "{}: `inputSchema.type` must be \"object\"",
        path.display()
    );
}

/// Render the generated module source for a sorted list of schemas.
///
/// Emits two things:
///
/// 1. `MCP_TOOL_SCHEMAS: &[(&str, &str)]` — iterable registry.
/// 2. A `schema_const!` `macro_rules!` that maps a literal tool name to
///    its per-tool `include_str!("…/<name>.json")` constant. Referencing an
///    unknown tool name from a handler via `schema_const!("missing")`
///    expands to a non-existent arm → **compile error**, which satisfies
///    the KAIZEN-0178 acceptance constraint (removing a JSON file must
///    break the build, not degrade silently).
fn render_tool_schema_module(entries: &[(String, std::path::PathBuf)]) -> String {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR set by cargo");
    let abs_path = |path: &std::path::PathBuf| {
        path.canonicalize()
            .unwrap_or_else(|_| Path::new(&manifest_dir).join(path))
    };
    let mut out = String::new();
    out.push_str(
        "// AUTO-GENERATED by build.rs (KAIZEN-0178). Do not edit.\n\
         //\n\
         // Source of truth: `mcp_tool_schemas/<tool_name>.json`.\n\
         // Deleting a referenced schema JSON triggers a compile error via `include_str!`.\n\n",
    );
    out.push_str("/// Raw JSON contents of every MCP tool schema, keyed by tool name.\n");
    out.push_str("///\n");
    out.push_str("/// Slice entries are sorted by tool name for deterministic iteration.\n");
    out.push_str("pub static MCP_TOOL_SCHEMAS: &[(&str, &str)] = &[\n");
    for (name, path) in entries {
        out.push_str(&format!(
            "    (\"{name}\", include_str!(r\"{}\")),\n",
            abs_path(path).display()
        ));
    }
    out.push_str("];\n\n");

    // Per-tool literal lookup macro. Referencing an unknown name fails to
    // match any arm → compile error ("no rules expected the token …").
    out.push_str("/// Compile-time tool schema lookup by literal tool name.\n");
    out.push_str("///\n");
    out.push_str("/// See [`tool_metadata!`] for the public macro that wraps this.\n");
    out.push_str("#[macro_export]\n");
    out.push_str("macro_rules! __kaizen0178_schema_const {\n");
    for (name, path) in entries {
        out.push_str(&format!(
            "    (\"{name}\") => {{ include_str!(r\"{}\") }};\n",
            abs_path(path).display()
        ));
    }
    out.push_str("}\n");
    out
}

fn make_contract_env_key(contract: &str, equation: &str) -> String {
    let c = contract.to_uppercase().replace(['-', '.', ' '], "_");
    let e = equation.to_uppercase().replace(['-', '.', ' '], "_");
    // Strip .yaml suffix
    let c = c.trim_end_matches("_YAML");
    format!("CONTRACT_{c}_{e}")
}

/// Embed the exact source revision the binary was built from.
///
/// Version numbers repeat across builds, so `pmat --version` alone cannot tell a
/// fresh binary from a stale one. During v3.28.2 three separate measurements were
/// reported against binaries that did not match the tree being tested — a stale
/// artifact, a workspace build, and a lockfile build — and one of those shipped a
/// headline fix that did not work. A commit SHA plus a dirty flag makes the
/// mismatch checkable instead of a matter of trust.
///
/// Emits `unknown` when there is no git metadata, which is the normal case for a
/// crates.io tarball build; verification tooling treats `unknown` as "cannot
/// confirm" rather than as a pass.
fn emit_build_provenance() {
    let sha = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    // `--porcelain` prints one line per modified path; empty means clean.
    let dirty = std::process::Command::new("git")
        .args(["status", "--porcelain", "--untracked-files=no"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            if String::from_utf8_lossy(&o.stdout).trim().is_empty() {
                "clean"
            } else {
                "dirty"
            }
        })
        .unwrap_or("unknown");

    println!("cargo:rustc-env=PMAT_GIT_SHA={sha}");
    println!("cargo:rustc-env=PMAT_GIT_DIRTY={dirty}");
    // Rebuild when the checked-out revision changes, so the embedded SHA cannot
    // go stale behind an otherwise-cached build.
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/index");
}
