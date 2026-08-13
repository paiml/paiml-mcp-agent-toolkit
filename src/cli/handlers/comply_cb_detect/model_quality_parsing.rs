// model_quality_parsing.rs — File walking and model header parsing.
// Included by model_quality.rs; shares its module scope.

/// Walk directory for model files (*.gguf, *.apr, *.safetensors).
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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

/// Whether the file's magic bytes agree with the format its extension claims.
///
/// `analyze models` derived the reported format from the FILENAME EXTENSION
/// alone, so a 0-byte `.safetensors`, an 8-byte `NOTAGGUF` and a plain-text
/// `.apr` were all inventoried as valid models of their declared format —
/// while this validator, which reads the actual header, was never called from
/// the inventory path. A format nobody verified must not be reported as fact.
#[must_use]
pub fn model_header_matches_extension(path: &Path) -> bool {
    parse_model_header(path).is_some()
}

/// Largest declared metadata/header block we will read into memory. A file may
/// claim any length it likes; refusing an implausible one is what keeps a
/// corrupt header from becoming an allocation.
const MAX_MODEL_HEADER_BYTES: u64 = 100_000_000;

/// Fill `buf` from `file`, returning how many bytes were actually read.
///
/// `Read::read` is permitted to return a short count even when more bytes are
/// available, so the number it returns is not a fact about the file. Every
/// length check below is a claim about the FILE, which means the read has to be
/// driven to EOF or to a full buffer first.
fn read_header_bytes(file: &mut File, buf: &mut [u8]) -> std::io::Result<usize> {
    let mut filled = 0;
    while filled < buf.len() {
        match file.read(&mut buf[filled..]) {
            Ok(0) => break,
            Ok(n) => filled += n,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
    Ok(filled)
}

/// Parse minimal header from model file (never loads tensor data).
fn parse_model_header(path: &Path) -> Option<ModelMetadata> {
    let ext = path.extension()?.to_str()?;
    let format = ModelFormat::from_extension(ext)?;
    let file_size = fs::metadata(path).ok()?.len();

    let mut file = File::open(path).ok()?;
    let mut header_buf = [0u8; 64];
    let bytes_read = read_header_bytes(&mut file, &mut header_buf).ok()?;
    if bytes_read < 8 {
        return None;
    }
    // Hand on ONLY the bytes that exist. `header_buf` is a zeroed 64-byte stack
    // array, so passing it whole made every `buf.len() < N` guard below a check
    // on the buffer's size rather than the file's: a truncated model read its
    // fields out of the zero padding and validated.
    let header = &header_buf[..bytes_read];

    match format {
        ModelFormat::Gguf => parse_gguf_header(header, file_size),
        ModelFormat::Apr => parse_apr_header(header, &mut file, file_size),
        ModelFormat::SafeTensors => parse_safetensors_header(header, &mut file, file_size),
    }
}

/// GGUF's fixed header: magic (4) + version (4) + tensor_count (8) +
/// metadata_kv_count (8).
const GGUF_HEADER_BYTES: usize = 24;

fn parse_gguf_header(buf: &[u8], file_size: u64) -> Option<ModelMetadata> {
    // GGUF magic: "GGUF" (0x46554747 LE) at offset 0
    //
    // A file shorter than the fixed header has no tensor count to report; the
    // previous bound of 16 also stopped short of the metadata KV count that
    // sits at offset 16, so a 16-byte file was accepted as a whole header.
    if buf.len() < GGUF_HEADER_BYTES || file_size < GGUF_HEADER_BYTES as u64 {
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

    // The declared metadata block must actually be present. Previously the read
    // was attempted ONLY for `0 < metadata_len < 100 MB` and every other value
    // fell through to `Some(..)` with no tensor count — so the two most
    // implausible claims a file can make, a zero-length index and one four
    // gigabytes long, were the two that validated without reading a byte. That
    // is also how a text file starting "APRIL " passed: its next four bytes
    // decode to a length far above the limit.
    let metadata_end = metadata_len.checked_add(8)?;
    if metadata_len == 0 || metadata_len > MAX_MODEL_HEADER_BYTES || metadata_end > file_size {
        return None;
    }

    // Check for CRC footer (last 4 bytes of file). A failed seek left the
    // cursor wherever the header read had put it, so `read_exact` then reported
    // "has CRC" on four arbitrary bytes from the middle of the file.
    let has_crc = if file_size > 4 {
        let mut crc_buf = [0u8; 4];
        file.seek(SeekFrom::End(-4)).is_ok() && file.read_exact(&mut crc_buf).is_ok()
    } else {
        false
    };

    // Parse JSON metadata to count tensors. A metadata block that is not UTF-8
    // is not the JSON index this format defines, so it fails rather than
    // reporting an unknown tensor count.
    let mut json_buf = vec![0u8; metadata_len as usize];
    file.seek(SeekFrom::Start(8)).ok()?;
    file.read_exact(&mut json_buf).ok()?;
    let text = std::str::from_utf8(&json_buf).ok()?;
    // Count "name" fields in tensor index as a rough tensor count
    let tensor_count = text.matches("\"name\"").count() as u64;

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

    // Sanity check: header should be < 100MB, and the header it declares must
    // fit inside the file. The old bound was `header_len < file_size`, which
    // ignored the 8 length bytes and — worse — was used to CHOOSE whether to
    // read at all: a file whose declared header ran past EOF skipped the read
    // and still returned success, so a truncated model was indistinguishable
    // from a whole one.
    let header_end = header_len.checked_add(8)?;
    if header_len == 0 || header_len > MAX_MODEL_HEADER_BYTES || header_end > file_size {
        return None;
    }

    // Read JSON header. SafeTensors defines this block as a JSON object, so it
    // is parsed here rather than assumed — this is the eager validation that
    // makes a load failure surface at load time (Jidoka / fail fast) instead of
    // becoming an inventory entry that claims a format nobody verified.
    let mut json_buf = vec![0u8; header_len as usize];
    file.seek(SeekFrom::Start(8)).ok()?;
    file.read_exact(&mut json_buf).ok()?;
    let text = std::str::from_utf8(&json_buf).ok()?;
    if !serde_json::from_str::<serde_json::Value>(text).is_ok_and(|v| v.is_object()) {
        return None;
    }
    // Count tensor entries (each has "dtype" field)
    let count = text.matches("\"dtype\"").count();
    // Subtract 1 for the __metadata__ entry if present
    let tensor_count = if text.contains("__metadata__") && count > 0 {
        (count - 1) as u64
    } else {
        count as u64
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
