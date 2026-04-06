#![cfg_attr(coverage_nightly, coverage(off))]
//! Utility functions shared across CB-500 series checks.

use std::path::Path;

// Use concat! to avoid self-detection by CB-501/CB-502 scanners
pub(super) const DOT_UNWRAP: &str = concat!(".unwr", "ap()");
pub(super) const DOT_EXPECT_QUOTE: &str = concat!(".expe", "ct(\"");

// Use concat! to avoid self-detection by CB-513 scanner
pub(super) const UNWRAP_OR_ELSE_DISCARD: &str = concat!(".unwrap_or_el", "se(|_|");
pub(super) const MAP_ERR_DISCARD: &str = concat!(".map_er", "r(|_|");

// Use concat! to avoid self-detection by CB-514 scanner
pub(super) const EPRINTLN_DEBUG: &str = concat!("eprintln!(\"[DEB", "UG");
pub(super) const EPRINTLN_DBG: &str = concat!("eprintln!(\"[DB", "G");
pub(super) const EPRINTLN_TRACE: &str = concat!("eprintln!(\"[TRA", "CE");

/// Returns true if the line starts a function definition.
pub(super) fn is_fn_start(trimmed: &str) -> bool {
    trimmed.starts_with("pub fn ")
        || trimmed.starts_with("fn ")
        || trimmed.starts_with("pub async fn ")
        || trimmed.starts_with("async fn ")
}

/// Returns true if the line starts a loop construct.
pub(super) fn is_loop_start(trimmed: &str) -> bool {
    trimmed.starts_with("for ")
        || trimmed.starts_with("while ")
        || trimmed == "loop {"
        || trimmed.starts_with("loop {")
}

/// Returns true if the line starts a spawn closure (thread::spawn, tokio::spawn, etc.)
pub(super) fn is_spawn_call(trimmed: &str) -> bool {
    trimmed.contains("thread::spawn")
        || trimmed.contains("tokio::spawn")
        || trimmed.contains("rayon::spawn")
        || trimmed.contains("spawn_blocking")
}

/// Helper: check if panic macro text appears only inside a string literal
pub(super) fn is_macro_in_string_literal(trimmed: &str, macros: &[&str]) -> bool {
    if !trimmed.contains('"') {
        return false;
    }
    let before_string = trimmed.split('"').next().unwrap_or("");
    !macros.iter().any(|m| before_string.contains(m))
}

/// Helper: walk directory for files with a specific extension
pub(super) fn walkdir_files_with_ext(
    dir: &Path,
    ext: &str,
) -> Result<Vec<std::path::PathBuf>, std::io::Error> {
    debug_assert!(dir.exists(), "dir must exist: {}", dir.display());
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            files.extend(walkdir_files_with_ext(&path, ext)?);
        } else if path.extension().map(|e| e == ext).unwrap_or(false) {
            files.push(path);
        }
    }
    Ok(files)
}

/// Check if a function definition line is a test function (by attribute or name).
pub(super) fn is_test_fn_definition(trimmed: &str, line_idx: usize, lines: &[&str]) -> bool {
    let has_test_attr = (1..=3).any(|back| {
        line_idx >= back && {
            let prev = lines[line_idx - back].trim();
            prev == "#[test]" || prev == "#[tokio::test]" || prev.starts_with("#[cfg(test")
        }
    });
    let fn_name_lower = trimmed.to_lowercase();
    has_test_attr
        || fn_name_lower.contains("test_")
        || fn_name_lower.contains("_test")
        || fn_name_lower.contains("roundtrip")
}

/// Lossy transform pairs: (forward, reverse, path_stems).
/// If both halves appear in the same function, it's suspicious -- unless the file
/// path contains one of the path_stems (indicating the module implements that transform).
pub(super) const LOSSY_TRANSFORM_PAIRS: &[(&str, &str, &[&str])] = &[
    ("quantize", "dequantize", &["quant", "qlora", "lora"]),
    (
        "encode",
        "decode",
        &["codec", "encoding", "decoder", "encoder"],
    ),
    (
        "compress",
        "decompress",
        &["compress", "zlib", "gzip", "lz4", "zstd"],
    ),
    ("serialize", "deserialize", &["serde", "serial", "marshal"]),
    ("pack", "unpack", &["pack", "msgpack"]),
    ("to_bytes", "from_bytes", &["bytes", "binary"]),
    ("to_f16", "to_f32", &[]),
    ("to_bf16", "to_f32", &[]),
];

/// Check function body for lossy transform pairs. Returns matched pair or None.
/// Skips pairs when the file path indicates the module implements that transform
/// (e.g. skip quantize/dequantize pair in files under a "quant" module).
pub(super) fn find_lossy_pair<'a>(fn_content: &str, file_path: &str) -> Option<(&'a str, &'a str)> {
    let filtered: String = fn_content
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.starts_with("#[derive(") && !t.starts_with("//")
        })
        .collect::<Vec<_>>()
        .join("\n")
        .to_lowercase();
    let path_lower = file_path.to_lowercase();
    LOSSY_TRANSFORM_PAIRS.iter().find_map(|(fwd, rev, stems)| {
        // Skip if the file path matches the transform domain
        if stems.iter().any(|s| path_lower.contains(s)) {
            return None;
        }
        if !filtered.contains(*rev) {
            return None;
        }
        // Ensure fwd appears independently, not only as substring of rev
        let without_rev = filtered.replace(rev, "");
        if without_rev.contains(*fwd) {
            Some((*fwd, *rev))
        } else {
            None
        }
    })
}

/// Binary read patterns that should be preceded by magic byte validation.
pub(super) const BINARY_READ_PATTERNS: &[&str] = &[
    "read_exact(",
    "from_le_bytes(",
    "from_be_bytes(",
    "read_u32::",
    "read_u64::",
    "read_i32::",
    "read_i64::",
];

pub(super) const MAGIC_VALIDATION_PATTERNS: &[&str] = &[
    "magic",
    "MAGIC",
    "signature",
    "SIGNATURE",
    "header_magic",
    "file_type",
    "format_version",
    "FILE_MAGIC",
];

/// I/O context markers: function must have actual file/stream I/O to be
/// considered "binary format parsing" (not just byte math in hash/quantize code).
pub(super) const IO_CONTEXT_PATTERNS: &[&str] = &[
    "File::",
    "BufReader",
    "BufRead",
    "Cursor::",
    "stdin",
    "from_reader(",
    ".read(",
    ".read_to_end(",
    "open(",
    "Read>",
    "impl Read",
];

/// Classify a non-comment code line for CB-521 binary parsing analysis.
/// Returns (has_binary_read, has_magic_check, has_io_context).
pub(super) fn classify_cb521_line(trimmed: &str) -> (bool, bool, bool) {
    let binary = BINARY_READ_PATTERNS.iter().any(|p| trimmed.contains(p));
    let magic = MAGIC_VALIDATION_PATTERNS
        .iter()
        .any(|p| trimmed.contains(p));
    let io = IO_CONTEXT_PATTERNS.iter().any(|p| trimmed.contains(p));
    (binary, magic, io)
}

/// Expensive initialization patterns for CB-520.
pub(super) const EXPENSIVE_INIT_PATTERNS: &[&str] = &[
    "::new(",
    "::open(",
    "::connect(",
    "::create(",
    "::load(",
    "::init(",
    "::build(",
    "::from_file(",
    "::from_path(",
    "::read_to_string(",
    "File::open(",
];

/// Check if a cast on line `i` is covered by a nearby `#[allow(clippy::cast_*)]`
/// annotation or `// SAFETY:` comment (within preceding 5 lines or function scope).
pub(super) fn is_cast_allowed(lines: &[&str], i: usize) -> bool {
    // Check same line
    let trimmed = lines[i].trim();
    if trimmed.contains("allow(clippy::cast") {
        return true;
    }
    // Check preceding 1-5 lines for annotation or SAFETY comment
    for back in 1..=5 {
        if i < back {
            break;
        }
        let prev = lines[i - back].trim();
        if prev.contains("allow(clippy::cast") || prev.starts_with("// SAFETY:") {
            return true;
        }
        // Stop looking if we hit a blank line or non-attribute/comment line
        if !prev.starts_with("#[") && !prev.starts_with("//") && !prev.is_empty() {
            break;
        }
    }
    false
}
