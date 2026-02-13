#![cfg_attr(coverage_nightly, coverage(off))]
//! CB-1000 Series: MLOps Model Quality Detection
//!
//! Header-only analysis of ML model binary files (GGUF, APR, SafeTensors).
//! Never loads tensor data — parses only metadata for quality checks.
//!
//! Based on: BUG-GGUF-001/002 (aprender), BUG-212 (safetensors sharding),
//! LAYOUT-002 (APR row-major mandate), Sculley et al. (2015) ML tech debt.

use super::types::*;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

/// Directories to skip when walking for model files.
const SKIP_DIRS: &[&str] = &[
    ".git", "node_modules", "target", ".pmat", "vendor", "build", "dist",
    "__pycache__", ".venv",
];

/// Model file extensions we recognize.
const MODEL_EXTENSIONS: &[&str] = &["gguf", "apr", "safetensors"];

/// Maximum tensor count before flagging as likely corrupt (BUG-GGUF-001).
const MAX_TENSOR_COUNT: u64 = 100_000;

/// File size threshold for "consider quantization" advisory (10 GB).
const LARGE_MODEL_THRESHOLD: u64 = 10 * 1024 * 1024 * 1024;

// =============================================================================
// Model format detection
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelFormat {
    Gguf,
    Apr,
    SafeTensors,
}

impl ModelFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            "gguf" => Some(Self::Gguf),
            "apr" => Some(Self::Apr),
            "safetensors" => Some(Self::SafeTensors),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Gguf => "GGUF",
            Self::Apr => "APR",
            Self::SafeTensors => "SafeTensors",
        }
    }
}

/// Minimal model metadata extracted from header only.
#[derive(Debug)]
pub struct ModelMetadata {
    pub format: ModelFormat,
    pub file_size_bytes: u64,
    pub tensor_count: Option<u64>,
    pub architecture: Option<String>,
    pub has_crc: bool,
}

// =============================================================================
// File walking
// =============================================================================

/// Walk directory for model files (*.gguf, *.apr, *.safetensors).
pub fn walkdir_model_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    walk_model_recursive(dir, &mut files);
    files
}

fn walk_model_recursive(dir: &Path, files: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let dir_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            if !SKIP_DIRS.contains(&dir_name) {
                walk_model_recursive(&path, files);
            }
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if MODEL_EXTENSIONS.contains(&ext) {
                files.push(path);
            }
        }
    }
}

/// Parse minimal header from model file (never loads tensor data).
fn parse_model_header(path: &Path) -> Option<ModelMetadata> {
    let ext = path.extension()?.to_str()?;
    let format = ModelFormat::from_extension(ext)?;
    let file_size = fs::metadata(path).ok()?.len();

    let mut file = File::open(path).ok()?;
    let mut header_buf = [0u8; 64];
    let bytes_read = file.read(&mut header_buf).ok()?;
    if bytes_read < 8 {
        return None;
    }

    match format {
        ModelFormat::Gguf => parse_gguf_header(&header_buf, file_size),
        ModelFormat::Apr => parse_apr_header(&header_buf, &mut file, file_size),
        ModelFormat::SafeTensors => {
            parse_safetensors_header(&header_buf, &mut file, file_size)
        }
    }
}

fn parse_gguf_header(buf: &[u8], file_size: u64) -> Option<ModelMetadata> {
    // GGUF magic: "GGUF" (0x46554747 LE) at offset 0
    if buf.len() < 16 {
        return None;
    }
    let magic = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
    if magic != 0x4655_4747 {
        return None;
    }

    // Version at offset 4 (u32 LE)
    let _version = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);

    // Tensor count at offset 8 (u64 LE)
    let tensor_count = u64::from_le_bytes([
        buf[8], buf[9], buf[10], buf[11], buf[12], buf[13], buf[14], buf[15],
    ]);

    // Metadata count at offset 16 (u64 LE) — we extract architecture from this
    // but for now we just report tensor count
    Some(ModelMetadata {
        format: ModelFormat::Gguf,
        file_size_bytes: file_size,
        tensor_count: Some(tensor_count),
        architecture: None, // Would need full KV parse
        has_crc: false,     // GGUF has no CRC
    })
}

fn parse_apr_header(buf: &[u8], file: &mut File, file_size: u64) -> Option<ModelMetadata> {
    if buf.len() < 8 {
        return None;
    }
    // APR magic: "APR2" at offset 0
    if &buf[0..4] != b"APR2" && &buf[0..3] != b"APR" {
        return None;
    }

    // Metadata length at offset 4 (u32 LE)
    let metadata_len = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]) as u64;

    // Check for CRC footer (last 4 bytes of file)
    let has_crc = if file_size > 4 {
        file.seek(SeekFrom::End(-4)).ok();
        let mut crc_buf = [0u8; 4];
        file.read_exact(&mut crc_buf).is_ok()
    } else {
        false
    };

    // Parse JSON metadata to count tensors
    let tensor_count = if metadata_len > 0 && metadata_len < 100_000_000 {
        let mut json_buf = vec![0u8; metadata_len as usize];
        file.seek(SeekFrom::Start(8)).ok()?;
        file.read_exact(&mut json_buf).ok()?;
        if let Ok(text) = std::str::from_utf8(&json_buf) {
            // Count "name" fields in tensor index as a rough tensor count
            text.matches("\"name\"").count() as u64
        } else {
            0
        }
    } else {
        0
    };

    Some(ModelMetadata {
        format: ModelFormat::Apr,
        file_size_bytes: file_size,
        tensor_count: if tensor_count > 0 {
            Some(tensor_count)
        } else {
            None
        },
        architecture: None,
        has_crc,
    })
}

fn parse_safetensors_header(
    buf: &[u8],
    file: &mut File,
    file_size: u64,
) -> Option<ModelMetadata> {
    if buf.len() < 8 {
        return None;
    }
    // Header length (u64 LE) at offset 0
    let header_len = u64::from_le_bytes([
        buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
    ]);

    // Sanity check: header should be < 100MB
    if header_len == 0 || header_len > 100_000_000 {
        return None;
    }

    // Read JSON header
    let tensor_count = if header_len < file_size {
        let mut json_buf = vec![0u8; header_len as usize];
        file.seek(SeekFrom::Start(8)).ok()?;
        file.read_exact(&mut json_buf).ok()?;
        if let Ok(text) = std::str::from_utf8(&json_buf) {
            // Count tensor entries (each has "dtype" field)
            let count = text.matches("\"dtype\"").count();
            // Subtract 1 for the __metadata__ entry if present
            if text.contains("__metadata__") && count > 0 {
                (count - 1) as u64
            } else {
                count as u64
            }
        } else {
            0
        }
    } else {
        0
    };

    Some(ModelMetadata {
        format: ModelFormat::SafeTensors,
        file_size_bytes: file_size,
        tensor_count: if tensor_count > 0 {
            Some(tensor_count)
        } else {
            None
        },
        architecture: None,
        has_crc: false,
    })
}

// =============================================================================
// CB-1000: Missing Model Card
// =============================================================================

pub fn detect_cb1000_missing_model_card(project_path: &Path) -> Vec<CbPatternViolation> {
    let model_files = walkdir_model_files(project_path);
    let mut violations = Vec::new();

    // Group model files by directory
    let mut dirs_with_models: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
    for f in &model_files {
        if let Some(parent) = f.parent() {
            dirs_with_models
                .entry(parent.to_path_buf())
                .or_default()
                .push(f.clone());
        }
    }

    for (dir, files) in &dirs_with_models {
        let has_readme = dir.join("README.md").exists()
            || dir.join("readme.md").exists()
            || dir.join("model_card.md").exists()
            || dir.join("MODEL_CARD.md").exists();

        if !has_readme {
            let rel = dir
                .strip_prefix(project_path)
                .unwrap_or(dir)
                .display()
                .to_string();
            let model_names: Vec<String> = files
                .iter()
                .filter_map(|f| f.file_name().map(|n| n.to_string_lossy().to_string()))
                .collect();

            violations.push(CbPatternViolation {
                pattern_id: "CB-1000".to_string(),
                file: rel,
                line: 0,
                description: format!(
                    "Model directory has {} model file(s) but no model card (README.md): {}",
                    model_names.len(),
                    model_names.join(", ")
                ),
                severity: Severity::Warning,
            });
        }
    }

    violations
}

// =============================================================================
// CB-1001: Oversized Tensor Count
// =============================================================================

pub fn detect_cb1001_oversized_tensor_count(project_path: &Path) -> Vec<CbPatternViolation> {
    let model_files = walkdir_model_files(project_path);
    let mut violations = Vec::new();

    for file_path in &model_files {
        let metadata = match parse_model_header(file_path) {
            Some(m) => m,
            None => continue,
        };
        let rel = file_path
            .strip_prefix(project_path)
            .unwrap_or(file_path)
            .display()
            .to_string();

        if let Some(count) = metadata.tensor_count {
            if count > MAX_TENSOR_COUNT {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-1001".to_string(),
                    file: rel,
                    line: 0,
                    description: format!(
                        "{} file has {} tensors (limit: {}) — likely corrupt header (BUG-GGUF-001)",
                        metadata.format.name(),
                        count,
                        MAX_TENSOR_COUNT
                    ),
                    severity: Severity::Error,
                });
            }
        }
    }

    violations
}

// =============================================================================
// CB-1002: Missing Tokenizer
// =============================================================================

pub fn detect_cb1002_missing_tokenizer(project_path: &Path) -> Vec<CbPatternViolation> {
    let model_files = walkdir_model_files(project_path);
    let mut violations = Vec::new();

    // Group by directory
    let mut dirs_with_models: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
    for f in &model_files {
        if let Some(parent) = f.parent() {
            dirs_with_models
                .entry(parent.to_path_buf())
                .or_default()
                .push(f.clone());
        }
    }

    for (dir, files) in &dirs_with_models {
        // Check for any language model (heuristic: GGUF files are typically LLMs)
        let has_llm = files.iter().any(|f| {
            f.extension()
                .and_then(|e| e.to_str())
                .map(|e| e == "gguf")
                .unwrap_or(false)
        });

        if !has_llm {
            continue;
        }

        let has_tokenizer = dir.join("tokenizer.json").exists()
            || dir.join("tokenizer.model").exists()
            || dir.join("vocab.json").exists();

        if !has_tokenizer {
            let rel = dir
                .strip_prefix(project_path)
                .unwrap_or(dir)
                .display()
                .to_string();

            violations.push(CbPatternViolation {
                pattern_id: "CB-1002".to_string(),
                file: rel,
                line: 0,
                description:
                    "GGUF model directory missing tokenizer (tokenizer.json/tokenizer.model)"
                        .to_string(),
                severity: Severity::Warning,
            });
        }
    }

    violations
}

// =============================================================================
// CB-1006: Sharded SafeTensors Without Index
// =============================================================================

pub fn detect_cb1006_sharded_without_index(project_path: &Path) -> Vec<CbPatternViolation> {
    let model_files = walkdir_model_files(project_path);
    let mut violations = Vec::new();

    // Group by directory
    let mut dirs_with_models: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
    for f in &model_files {
        if let Some(parent) = f.parent() {
            dirs_with_models
                .entry(parent.to_path_buf())
                .or_default()
                .push(f.clone());
        }
    }

    for (dir, files) in &dirs_with_models {
        // Detect sharded pattern: model-00001-of-00003.safetensors
        let sharded_files: Vec<&PathBuf> = files
            .iter()
            .filter(|f| {
                let name = f
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                name.contains("-of-") && name.ends_with(".safetensors")
            })
            .collect();

        if sharded_files.len() > 1 {
            let has_index = dir.join("model.safetensors.index.json").exists();
            if !has_index {
                let rel = dir
                    .strip_prefix(project_path)
                    .unwrap_or(dir)
                    .display()
                    .to_string();

                violations.push(CbPatternViolation {
                    pattern_id: "CB-1006".to_string(),
                    file: rel,
                    line: 0,
                    description: format!(
                        "{} sharded SafeTensors files without model.safetensors.index.json (BUG-212)",
                        sharded_files.len()
                    ),
                    severity: Severity::Error,
                });
            }
        }
    }

    violations
}

// =============================================================================
// CB-1007: Excessive File Size
// =============================================================================

pub fn detect_cb1007_excessive_file_size(project_path: &Path) -> Vec<CbPatternViolation> {
    let model_files = walkdir_model_files(project_path);
    let mut violations = Vec::new();

    for file_path in &model_files {
        let file_size = match fs::metadata(file_path) {
            Ok(m) => m.len(),
            Err(_) => continue,
        };

        if file_size > LARGE_MODEL_THRESHOLD {
            let rel = file_path
                .strip_prefix(project_path)
                .unwrap_or(file_path)
                .display()
                .to_string();
            let size_gb = file_size as f64 / (1024.0 * 1024.0 * 1024.0);

            violations.push(CbPatternViolation {
                pattern_id: "CB-1007".to_string(),
                file: rel,
                line: 0,
                description: format!(
                    "Model file is {:.1} GB — consider quantization or sharding",
                    size_gb
                ),
                severity: Severity::Info,
            });
        }
    }

    violations
}

// =============================================================================
// CB-1008: APR Missing CRC
// =============================================================================

pub fn detect_cb1008_apr_missing_crc(project_path: &Path) -> Vec<CbPatternViolation> {
    let model_files = walkdir_model_files(project_path);
    let mut violations = Vec::new();

    for file_path in &model_files {
        if file_path.extension().and_then(|e| e.to_str()) != Some("apr") {
            continue;
        }

        let metadata = match parse_model_header(file_path) {
            Some(m) => m,
            None => continue,
        };

        if !metadata.has_crc {
            let rel = file_path
                .strip_prefix(project_path)
                .unwrap_or(file_path)
                .display()
                .to_string();

            violations.push(CbPatternViolation {
                pattern_id: "CB-1008".to_string(),
                file: rel,
                line: 0,
                description: "APR file missing CRC32 footer checksum".to_string(),
                severity: Severity::Warning,
            });
        }
    }

    violations
}
