// ── Public API ─────────────────────────────────────────────────────────────

/// Names that are too generic to be useful as file names.
const GENERIC_NAMES: &[&str] = &[
    "test",
    "tests",
    "construction",
    "print",
    "output",
    "format",
    "class",
    "model",
    "builder",
    "rendering",
    "dispatch",
    "display",
    "handler",
    "processing",
    "collection",
    "computation",
    "calculation",
    "token",
    "create",
    "helper",
    "data",
    "error",
    "errors",
    "file",
    "loading",
    "config",
    "cache",
    "graph",
    "value",
    "input",
    "head",
    "apply",
    "length",
    "path",
    "trace",
    "tensor",
    "mock",
    "index",
    "state",
    "context",
    "result",
    "entry",
    "node",
    "layer",
    "extract",
    "scatter",
    "pattern",
    "dimension",
    "approximate",
    "forward",
    "backward",
    "transform",
    "convert",
    "query",
    "search",
    "update",
    "write",
    "parse",
    "merge",
    "using",
    "operations",
    "models",
    "compute",
    "valid",
    "point",
    "fields",
    "capture",
    "tokens",
    "after",
    "before",
    "examples",
    "stream",
    "batch",
    "hidden",
    "memory",
    "fails",
    "elements",
    "execution",
    "custom",
    "default",
    "output",
    "simple",
    "basic",
    "common",
    "general",
    "other",
    "status",
    "action",
    "event",
    "source",
    "target",
    "object",
    "module",
    "service",
    "component",
    "manager",
    "utils",
    "types",
    "traits",
    "impls",
    "current",
    "direct",
    "internal",
    "wrapped",
    "block",
    "stays",
    "works",
    "needs",
    "takes",
    "makes",
    "given",
    "allow",
    "their",
    "these",
    "about",
    "being",
    "first",
    "where",
    "which",
    "since",
];

/// Find all `_part_` files and suggest semantic renames.
///
/// Returns suggestions sorted by confidence (highest first).
/// Applies post-processing: generic name penalty and inter-suggestion collision detection.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn suggest_renames(
    index: &AgentContextIndex,
    path_filter: Option<&str>,
) -> Vec<RenameSuggestion> {
    let mut suggestions: Vec<RenameSuggestion> = index
        .file_index
        .keys()
        .filter(|path| is_part_file(path))
        .filter(|path| path_filter.map(|pf| path.contains(pf)).unwrap_or(true))
        .filter_map(|path| {
            let entries = index.get_by_file(path);
            if entries.is_empty() {
                return None;
            }
            Some(analyze_file_for_rename(path, &entries, index))
        })
        .collect();

    // Post-processing: penalize generic names
    penalize_generic_names(&mut suggestions);

    // Post-processing: detect inter-suggestion collisions (same dir + same name)
    disambiguate_collisions(&mut suggestions);

    suggestions.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    suggestions
}

/// Penalize suggestions with overly generic names.
fn penalize_generic_names(suggestions: &mut [RenameSuggestion]) {
    debug_assert!(true, "contract: penalize_generic_names");
    for s in suggestions.iter_mut() {
        let stem = s.suggested_name.trim_end_matches(".rs");
        if GENERIC_NAMES.contains(&stem) {
            s.confidence *= 0.60;
            s.reasoning = format!("{} [generic name penalty]", s.reasoning);
        }
    }
}

/// Detect and resolve collisions between suggestions in the same directory.
///
/// When multiple `_part_` files map to the same suggested name in the same dir,
/// disambiguate by appending a distinguishing suffix from the original filename.
fn disambiguate_collisions(suggestions: &mut Vec<RenameSuggestion>) {
    debug_assert!(true, "contract: disambiguate_collisions");
    // Group by (directory, suggested_name)
    let mut groups: HashMap<(String, String), Vec<usize>> = HashMap::new();
    for (i, s) in suggestions.iter().enumerate() {
        if s.suggested_name.is_empty() {
            continue;
        }
        let dir = s
            .current_path
            .rfind('/')
            .map(|idx| s.current_path[..idx].to_string())
            .unwrap_or_default();
        groups
            .entry((dir, s.suggested_name.clone()))
            .or_default()
            .push(i);
    }

    // For groups with >1 entry, disambiguate with numeric suffix
    for ((_, _), indices) in &groups {
        if indices.len() <= 1 {
            continue;
        }
        // Sort by current_path for deterministic numbering
        let mut sorted_indices = indices.clone();
        sorted_indices.sort_by(|&a, &b| {
            suggestions[a]
                .current_path
                .cmp(&suggestions[b].current_path)
        });

        for (seq, &idx) in sorted_indices.iter().enumerate() {
            let base = suggestions[idx]
                .suggested_name
                .trim_end_matches(".rs")
                .to_string();
            let new_name = format!("{base}_{}.rs", seq + 1);
            let new_path = replace_filename(&suggestions[idx].current_path, &new_name);
            suggestions[idx].suggested_name = new_name;
            suggestions[idx].suggested_path = new_path;
            suggestions[idx].confidence *= 0.80;
            suggestions[idx].reasoning = format!("{} [disambiguated]", suggestions[idx].reasoning);
        }
    }
}

/// Check if a file path contains a `_partN` or `_part_` split pattern in its filename stem.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn is_part_file(path: &str) -> bool {
    debug_assert!(!path.is_empty(), "path must not be empty");
    let filename = Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    if let Some(idx) = filename.find("_part") {
        let rest = &filename[idx + 5..];
        rest.starts_with('_') || rest.chars().next().is_some_and(|c| c.is_ascii_digit())
    } else {
        false
    }
}

