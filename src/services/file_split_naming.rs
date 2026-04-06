// ── Cluster Naming ─────────────────────────────────────────────────────────

/// Generic prefixes that don't make good cluster names.
/// These are common verbs/prepositions that pass the 4-char min-length
/// check in try_common_prefix but don't convey semantic meaning.
const GENERIC_PREFIX_BLOCKLIST: &[&str] = &[
    "from",
    "into",
    "with",
    "make",
    "create",
    "build",
    "parse",
    "check",
    "test",
    "init",
    "load",
    "save",
    "read",
    "write",
    "send",
    "recv",
    "handle",
    "process",
    "convert",
    "transform",
    "validate",
    "exec",
    "run",
    "call",
    "apply",
    "update",
    "delete",
    "remove",
    "find",
    "get",
    "set",
    "new",
    "try",
    "is",
    "has",
    "can",
    "should",
];

/// Name a cluster using the suggest-rename signal cascade.
fn name_cluster(entries: &[&FunctionEntry], file_path: &str) -> (String, String, f32) {
    // Try each signal in priority order

    // 1. Dominant type (struct/enum/trait)
    if let Some((name, confidence, _reason)) = try_dominant_type(entries) {
        return (name, "DominantType".to_string(), confidence);
    }

    // 2. Function theme (>70% share a keyword)
    if let Some((name, confidence, _reason)) = try_function_theme(entries) {
        return (name, "FunctionTheme".to_string(), confidence);
    }

    // 3. Common prefix (skip generic verbs/prepositions)
    if let Some((name, confidence, _reason)) = try_common_prefix(entries) {
        if !GENERIC_PREFIX_BLOCKLIST.contains(&name.as_str()) {
            return (name, "CommonPrefix".to_string(), confidence);
        }
    }

    // 4. Doc comment consensus
    if let Some((name, confidence, _reason)) = try_doc_comment_consensus(entries) {
        return (name, "DocCommentConsensus".to_string(), confidence);
    }

    // 5. Context word from function names
    if let Some(word) = find_context_word(entries) {
        return (word, "ContextWord".to_string(), 0.5);
    }

    // 6. Fallback: use file stem + cluster index
    let stem = Path::new(file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("module");
    (format!("{}_cluster", stem), "Fallback".to_string(), 0.2)
}
