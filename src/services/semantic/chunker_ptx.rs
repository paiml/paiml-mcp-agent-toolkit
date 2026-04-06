// PTX standalone file chunker
// Extracts .entry and .func blocks from PTX assembly files

fn chunk_ptx_file(source: &str) -> Result<Vec<CodeChunk>, String> {
    debug_assert!(!source.is_empty(), "source must not be empty");
    let mut chunks = Vec::new();
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;

    // Skip PTX metadata lines (.version, .target) — not function chunks
    // These are available in the file-level content if needed

    while i < lines.len() {
        let trimmed = lines[i].trim();

        // Match .entry (kernel) or .func (device function) declarations
        let is_entry = trimmed.starts_with(".entry") || trimmed.starts_with(".visible .entry");
        let is_func = trimmed.starts_with(".func") || trimmed.starts_with(".visible .func");

        if is_entry || is_func {
            let start_line = i + 1; // 1-indexed
            let name = extract_ptx_kernel_name(trimmed);

            // Find the end of the function body (matching braces)
            let mut brace_depth = 0;
            let mut end_line = i;
            let mut found_body = false;

            for j in i..lines.len() {
                let line = lines[j];
                if line.contains('{') {
                    brace_depth += line.matches('{').count();
                    found_body = true;
                }
                if line.contains('}') {
                    brace_depth -= line.matches('}').count();
                }
                end_line = j;
                if found_body && brace_depth == 0 {
                    break;
                }
            }

            let content: String = lines[i..=end_line].join("\n");
            let checksum = format!("{:x}", md5_hash(content.as_bytes()));

            chunks.push(CodeChunk {
                file_path: String::new(),
                chunk_type: ChunkType::Function,
                chunk_name: name,
                language: "ptx".to_string(),
                start_line,
                end_line: end_line + 1,
                content,
                content_checksum: checksum,
            });

            i = end_line + 1;
        } else {
            i += 1;
        }
    }

    Ok(chunks)
}

/// Extract kernel/function name from PTX declaration line
fn extract_ptx_kernel_name(line: &str) -> String {
    debug_assert!(!line.is_empty(), "line must not be empty");
    // Patterns:
    //   .entry vector_add(...) {
    //   .visible .entry kernel_name(...)
    //   .func (.reg .f32 result) warp_reduce(.reg .f32 val)
    //   .visible .func func_name(...)
    let line = line.trim();

    // Remove visibility prefix
    let line = line.strip_prefix(".visible").map_or(line, str::trim);

    // Remove .entry/.func prefix
    let after_keyword = if let Some(rest) = line.strip_prefix(".entry") {
        rest.trim()
    } else if let Some(rest) = line.strip_prefix(".func") {
        let rest = rest.trim();
        // .func may have return type: .func (.reg .f32 result) name(...)
        if rest.starts_with('(') {
            // Skip return type in parens
            if let Some(close) = rest.find(')') {
                rest[close + 1..].trim()
            } else {
                rest
            }
        } else {
            rest
        }
    } else {
        line
    };

    // Extract name before '(' or end of line
    after_keyword
        .split(|c: char| c == '(' || c.is_whitespace())
        .next()
        .unwrap_or("unknown")
        .to_string()
}

/// Simple hash for PTX content checksums
fn md5_hash(data: &[u8]) -> u64 {
    debug_assert!(!data.is_empty(), "data must not be empty");
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for &byte in data {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}
