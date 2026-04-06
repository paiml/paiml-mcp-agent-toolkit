// model_quality_parsing.rs — File walking and model header parsing.
// Included by model_quality.rs; shares its module scope.

/// Walk directory for model files (*.gguf, *.apr, *.safetensors).
pub fn walkdir_model_files(dir: &Path) -> Vec<PathBuf> {
    debug_assert!(dir.exists(), "dir must exist: {}", dir.display());
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
            let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
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
        ModelFormat::SafeTensors => parse_safetensors_header(&header_buf, &mut file, file_size),
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

fn parse_safetensors_header(buf: &[u8], file: &mut File, file_size: u64) -> Option<ModelMetadata> {
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
