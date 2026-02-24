/// Parse `siblings` array from workspace.toml content.
///
/// Handles: `siblings = ["../aprender", "../trueno"]`
/// Minimal parser — no full TOML dependency needed for one key.
pub(crate) fn parse_workspace_siblings(content: &str) -> Vec<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("siblings") {
            let rest = rest.trim().strip_prefix('=').unwrap_or("").trim();
            if let Some(inner) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                return inner
                    .split(',')
                    .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
    }
    Vec::new()
}

/// Build a corpus document string for a single function entry.
///
/// Used by find_similar() when corpus was not pre-built (SQLite load path).
pub(crate) fn build_corpus_entry(func: &FunctionEntry) -> String {
    format!(
        "{name} {name} {sig} {sig} {doc} {doc} {path} {idents}",
        name = func.function_name,
        sig = func.signature,
        doc = func.doc_comment.as_deref().unwrap_or(""),
        path = func.file_path,
        idents = extract_identifiers(&func.source)
    )
}

/// Build name_index, file_index, and corpus from functions.
pub(crate) fn build_indices(functions: &[FunctionEntry]) -> BuildIndicesResult {
    build_indices_impl(functions, true)
}

/// Build name_index and file_index only (skip corpus construction).
///
/// Used by SQLite load path where FTS5 handles search, saving ~36MB
/// of corpus string allocation for 90K functions.
pub(crate) fn build_indices_without_corpus(functions: &[FunctionEntry]) -> BuildIndicesResult {
    build_indices_impl(functions, false)
}

fn build_indices_impl(functions: &[FunctionEntry], include_corpus: bool) -> BuildIndicesResult {
    let mut result = BuildIndicesResult {
        name_index: HashMap::new(),
        file_index: HashMap::new(),
        corpus: if include_corpus {
            Vec::with_capacity(functions.len())
        } else {
            Vec::new()
        },
    };

    for (idx, func) in functions.iter().enumerate() {
        // Cap name_index entries per name to prevent pathological sizes
        // for common names like "new" (can have 10,000+ entries)
        let name_entries = result
            .name_index
            .entry(func.function_name.clone())
            .or_default();
        if name_entries.len() < 100 {
            name_entries.push(idx);
        }
        result
            .file_index
            .entry(func.file_path.clone())
            .or_default()
            .push(idx);

        if include_corpus {
            result.corpus.push(build_corpus_entry(func));
        }
    }

    result
}

/// Compute SHA256 hash of file content
pub(super) fn compute_file_sha256(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}
