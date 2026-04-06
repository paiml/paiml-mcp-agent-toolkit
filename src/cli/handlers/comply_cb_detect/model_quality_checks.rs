// model_quality_checks.rs — CB-1000 series detection functions.
// Included by model_quality.rs; shares its module scope.

// =============================================================================
// CB-1000: Missing Model Card
// =============================================================================

pub fn detect_cb1000_missing_model_card(project_path: &Path) -> Vec<CbPatternViolation> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
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
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
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
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
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
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
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
                let name = f.file_name().and_then(|n| n.to_str()).unwrap_or("");
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
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
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
// CB-1004: Missing Architecture (GGUF)
// =============================================================================

pub fn detect_cb1004_missing_architecture(project_path: &Path) -> Vec<CbPatternViolation> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let model_files = walkdir_model_files(project_path);
    let mut violations = Vec::new();

    for file_path in &model_files {
        if file_path.extension().and_then(|e| e.to_str()) != Some("gguf") {
            continue;
        }

        // Read enough header to check for architecture KV
        let content = match fs::read(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // GGUF files should contain "general.architecture" key in metadata
        // Simple byte scan — GGUF metadata keys are stored as strings
        let needle = b"general.architecture";
        let has_arch = content.windows(needle.len()).any(|w| w == needle);

        if !has_arch && content.len() > 100 {
            let rel = file_path
                .strip_prefix(project_path)
                .unwrap_or(file_path)
                .display()
                .to_string();

            violations.push(CbPatternViolation {
                pattern_id: "CB-1004".to_string(),
                file: rel,
                line: 0,
                description:
                    "GGUF file missing `general.architecture` metadata key (BUG-EXPORT-004)"
                        .to_string(),
                severity: Severity::Warning,
            });
        }
    }

    violations
}

// =============================================================================
// CB-1005: Quantization Mismatch
// =============================================================================

/// Common quantization names that appear in filenames.
const QUANT_NAMES: &[&str] = &[
    "q2_k", "q3_k", "q4_k", "q4_0", "q4_1", "q5_k", "q5_0", "q5_1", "q6_k", "q8_0", "q8_1", "f16",
    "f32", "bf16", "q4_k_m", "q4_k_s", "q5_k_m", "q5_k_s", "q3_k_m", "q3_k_s", "q3_k_l", "q6_k_l",
    "q2_k_s", "iq4_xs", "iq4_nl",
];

pub fn detect_cb1005_quantization_mismatch(project_path: &Path) -> Vec<CbPatternViolation> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let model_files = walkdir_model_files(project_path);
    let mut violations = Vec::new();

    for file_path in &model_files {
        if file_path.extension().and_then(|e| e.to_str()) != Some("gguf") {
            continue;
        }

        let filename = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Check if filename claims a quantization type
        let claimed_quant = QUANT_NAMES.iter().find(|q| filename.contains(*q));

        if let Some(quant) = claimed_quant {
            // For F32 claims, check file size ratio
            // F32 models are ~4x larger than Q4 models
            if *quant == "f32" {
                let file_size = fs::metadata(file_path).map(|m| m.len()).unwrap_or(0);
                // A small F32 GGUF (< 100KB) with "f32" in name is suspicious
                if file_size < 100_000 && file_size > 0 {
                    let rel = file_path
                        .strip_prefix(project_path)
                        .unwrap_or(file_path)
                        .display()
                        .to_string();

                    violations.push(CbPatternViolation {
                        pattern_id: "CB-1005".to_string(),
                        file: rel,
                        line: 0,
                        description: format!(
                            "Filename claims {} quantization but file is suspiciously small ({} bytes) (BUG-1)",
                            quant.to_uppercase(),
                            file_size
                        ),
                        severity: Severity::Warning,
                    });
                }
            }
        }
    }

    violations
}

// =============================================================================
// CB-1008: APR Missing CRC
// =============================================================================

pub fn detect_cb1008_apr_missing_crc(project_path: &Path) -> Vec<CbPatternViolation> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
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
