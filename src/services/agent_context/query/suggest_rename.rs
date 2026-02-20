#![cfg_attr(coverage_nightly, coverage(off))]
//! Semantic file rename suggestions for `_part_` files (Issue #233).
//!
//! Analyzes FunctionEntry definitions within `_part_XX` files to suggest
//! meaningful names based on cascading signal priority.

use crate::services::agent_context::function_index::DefinitionType;
use crate::services::agent_context::{AgentContextIndex, FunctionEntry};
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;

// ── Types ──────────────────────────────────────────────────────────────────

/// Signal type used to determine the suggested name
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum RenameSignal {
    /// Single dominant struct/enum/trait → snake_case of that name
    DominantType,
    /// Existing suffix expansion (_attn → attention, _ops → operations)
    ExistingSuffix,
    /// >70% of functions share a keyword theme (forward, serialize, test, etc.)
    FunctionTheme,
    /// Longest common prefix across all function names (min 4 chars)
    CommonPrefix,
    /// Dominant keyword extracted from doc comments
    DocCommentConsensus,
    /// Multiple weak signals combined
    Mixed,
    /// No meaningful signal found
    NoSignal,
}

/// A rename suggestion for a single `_part_` file
#[derive(Debug, Clone, Serialize)]
pub struct RenameSuggestion {
    /// Current file path (relative to project root)
    pub current_path: String,
    /// Suggested new filename (just the stem, no directory)
    pub suggested_name: String,
    /// Full suggested path
    pub suggested_path: String,
    /// Confidence score 0.0-1.0
    pub confidence: f32,
    /// Human-readable reasoning
    pub reasoning: String,
    /// Signal type that produced this suggestion
    pub signal: RenameSignal,
    /// Parent file that include!()s or #[path=] this file (if detected)
    pub parent_file: Option<String>,
    /// Inclusion pattern (include! or #[path])
    pub inclusion_pattern: Option<String>,
    /// Number of definitions in the file
    pub definition_count: usize,
}

// ── Suffix expansion table ─────────────────────────────────────────────────

const SUFFIX_EXPANSIONS: &[(&str, &str)] = &[
    ("_attn", "attention"),
    ("_ops", "operations"),
    ("_impl", "implementation"),
    ("_util", "utilities"),
    ("_cfg", "config"),
    ("_fmt", "formatting"),
    ("_conv", "conversion"),
    ("_init", "initialization"),
    ("_exec", "execution"),
    ("_proc", "processing"),
    ("_gen", "generation"),
    ("_val", "validation"),
    ("_ser", "serialization"),
    ("_deser", "deserialization"),
    ("_alloc", "allocation"),
    ("_disp", "dispatch"),
    ("_fwd", "forward"),
    ("_bwd", "backward"),
    ("_norm", "normalization"),
    ("_trans", "transform"),
];

// ── Theme keywords ─────────────────────────────────────────────────────────

const THEME_KEYWORDS: &[(&str, &str)] = &[
    ("forward", "forward"),
    ("backward", "backward"),
    ("serialize", "serialization"),
    ("deserialize", "deserialization"),
    ("parse", "parsing"),
    ("format", "formatting"),
    ("render", "rendering"),
    ("validate", "validation"),
    ("encode", "encoding"),
    ("decode", "decoding"),
    ("build", "builder"),
    ("create", "construction"),
    ("init", "initialization"),
    ("load", "loading"),
    ("save", "persistence"),
    ("read", "reading"),
    ("write", "writing"),
    ("convert", "conversion"),
    ("transform", "transform"),
    ("display", "display"),
    ("test", "tests"),
    ("bench", "benchmarks"),
    ("config", "config"),
    ("error", "errors"),
    ("handle", "handler"),
    ("dispatch", "dispatch"),
    ("compute", "computation"),
    ("calculate", "calculation"),
    ("process", "processing"),
    ("analyze", "analysis"),
    ("cache", "cache"),
    ("index", "indexing"),
    ("search", "search"),
    ("query", "query"),
    ("emit", "emission"),
    ("collect", "collection"),
    ("merge", "merging"),
    ("sort", "sorting"),
    ("filter", "filtering"),
    ("map", "mapping"),
    ("reduce", "reduction"),
];

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

/// Check if a file path contains `_part_` in its filename stem.
pub(crate) fn is_part_file(path: &str) -> bool {
    let filename = Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    filename.contains("_part_")
}

// ── Core analysis ──────────────────────────────────────────────────────────

fn analyze_file_for_rename(
    file_path: &str,
    entries: &[&FunctionEntry],
    index: &AgentContextIndex,
) -> RenameSuggestion {
    let parent_file = detect_parent_file(file_path, index);

    // Cascading signal priority: try each analyzer in order
    let signals: [(RenameSignal, Option<(String, f32, String)>); 5] = [
        (RenameSignal::DominantType, try_dominant_type(entries)),
        (
            RenameSignal::ExistingSuffix,
            try_existing_suffix(file_path, entries),
        ),
        (RenameSignal::FunctionTheme, try_function_theme(entries)),
        (RenameSignal::CommonPrefix, try_common_prefix(entries)),
        (
            RenameSignal::DocCommentConsensus,
            try_doc_comment_consensus(entries),
        ),
    ];

    for (signal, result) in signals {
        if let Some((name, confidence, reasoning)) = result {
            return build_suggestion(
                file_path,
                &name,
                confidence,
                reasoning,
                signal,
                parent_file,
                entries.len(),
                index,
            );
        }
    }

    // Fallback: no strong signal
    RenameSuggestion {
        current_path: file_path.to_string(),
        suggested_name: String::new(),
        suggested_path: String::new(),
        confidence: 0.30,
        reasoning: "No dominant signal found".to_string(),
        signal: RenameSignal::NoSignal,
        parent_file,
        inclusion_pattern: Some("include!".to_string()),
        definition_count: entries.len(),
    }
}

/// Build a RenameSuggestion from a successful signal analysis.
fn build_suggestion(
    file_path: &str,
    name: &str,
    confidence: f32,
    reasoning: String,
    signal: RenameSignal,
    parent_file: Option<String>,
    definition_count: usize,
    index: &AgentContextIndex,
) -> RenameSuggestion {
    let suggested_name = format!("{name}.rs");
    let suggested_path = replace_filename(file_path, &suggested_name);

    // Reject if suggestion collides with parent file
    if collides_with_parent(&suggested_path, &parent_file) {
        return RenameSuggestion {
            current_path: file_path.to_string(),
            suggested_name: String::new(),
            suggested_path: String::new(),
            confidence: 0.10,
            reasoning: format!("{reasoning} [same as parent]"),
            signal: RenameSignal::NoSignal,
            parent_file,
            inclusion_pattern: Some("include!".to_string()),
            definition_count,
        };
    }

    // Penalize if name matches parent directory (redundant: graph/graph.rs)
    let dir_penalty = matches_parent_dir(file_path, name);

    let collision = check_collision(&suggested_path, index);
    let mut final_confidence = confidence;
    let mut final_reasoning = reasoning;
    if collision {
        final_confidence *= 0.5;
    }
    if dir_penalty {
        final_confidence *= 0.70;
        final_reasoning = format!("{final_reasoning} [redundant with parent dir]");
    }
    RenameSuggestion {
        current_path: file_path.to_string(),
        suggested_name,
        suggested_path,
        confidence: final_confidence,
        reasoning: final_reasoning,
        signal,
        parent_file,
        inclusion_pattern: Some("include!".to_string()),
        definition_count,
    }
}

/// Check if the suggested path matches the parent file path.
fn collides_with_parent(suggested_path: &str, parent_file: &Option<String>) -> bool {
    parent_file
        .as_ref()
        .is_some_and(|parent| suggested_path == parent)
}

/// Check if the suggested name matches the immediate parent directory.
/// E.g., `graph/mod_part_02.rs → graph.rs` is redundant inside a `graph/` dir.
fn matches_parent_dir(file_path: &str, suggested_name: &str) -> bool {
    let parts: Vec<&str> = file_path.rsplitn(2, '/').collect();
    if parts.len() < 2 {
        return false;
    }
    // parts[0] = filename, parts[1] = everything before last /
    let parent_dir = parts[1].rsplit('/').next().unwrap_or("");
    parent_dir == suggested_name
}

// ── Signal analyzers ───────────────────────────────────────────────────────

/// Check for a single dominant struct/enum/trait definition.
fn try_dominant_type(entries: &[&FunctionEntry]) -> Option<(String, f32, String)> {
    let type_defs: Vec<&FunctionEntry> = entries
        .iter()
        .filter(|e| {
            matches!(
                e.definition_type,
                DefinitionType::Struct | DefinitionType::Enum | DefinitionType::Trait
            )
        })
        .copied()
        .collect();

    if type_defs.len() == 1 {
        let name = &type_defs[0].function_name;
        let snake = to_snake_case(name);
        let kind = match type_defs[0].definition_type {
            DefinitionType::Struct => "struct",
            DefinitionType::Enum => "enum",
            DefinitionType::Trait => "trait",
            _ => "type",
        };
        return Some((
            snake,
            0.95,
            format!(
                "Dominant type ({kind} {name}) | {} definitions",
                entries.len()
            ),
        ));
    }

    // If there are exactly 2+ types, check if one has significantly more methods
    if type_defs.len() >= 2 {
        // Count methods associated with each type by prefix matching
        let mut type_method_counts: Vec<(&str, usize)> = type_defs
            .iter()
            .map(|td| {
                let snake = to_snake_case(&td.function_name);
                let count = entries
                    .iter()
                    .filter(|e| {
                        e.definition_type == DefinitionType::Function
                            && e.function_name.contains(&snake)
                    })
                    .count();
                (td.function_name.as_str(), count)
            })
            .collect();
        type_method_counts.sort_by(|a, b| b.1.cmp(&a.1));

        if let Some((dominant_name, dominant_count)) = type_method_counts.first() {
            let total_fns = entries
                .iter()
                .filter(|e| e.definition_type == DefinitionType::Function)
                .count();
            if total_fns > 0 && *dominant_count as f32 / total_fns as f32 > 0.5 {
                let snake = to_snake_case(dominant_name);
                return Some((
                    snake,
                    0.90,
                    format!(
                        "Dominant type ({dominant_name}, {dominant_count}/{total_fns} methods) | {} definitions",
                        entries.len()
                    ),
                ));
            }
        }
    }

    None
}

/// Check for known suffix expansions in the filename.
fn try_existing_suffix(
    file_path: &str,
    entries: &[&FunctionEntry],
) -> Option<(String, f32, String)> {
    let stem = Path::new(file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    // Strip _part_XX segments to get the base suffix
    let base = strip_part_segments(stem);

    for (suffix, expansion) in SUFFIX_EXPANSIONS {
        if base.ends_with(suffix) {
            // Try to get context from function names
            let context = find_context_word(entries);
            let name = if let Some(ctx) = context {
                format!("{ctx}_{expansion}")
            } else {
                expansion.to_string()
            };
            return Some((
                name.clone(),
                0.88,
                format!(
                    "Suffix expanded ({suffix} → {expansion}) | {} definitions",
                    entries.len()
                ),
            ));
        }
    }

    None
}

/// Check if >70% of functions share a keyword theme.
fn try_function_theme(entries: &[&FunctionEntry]) -> Option<(String, f32, String)> {
    let fn_entries: Vec<&&FunctionEntry> = entries
        .iter()
        .filter(|e| e.definition_type == DefinitionType::Function)
        .collect();

    if fn_entries.is_empty() {
        return None;
    }

    let mut keyword_counts: HashMap<&str, usize> = HashMap::new();
    for entry in &fn_entries {
        let name_lower = entry.function_name.to_lowercase();
        for (keyword, _) in THEME_KEYWORDS {
            if name_lower.contains(keyword) {
                *keyword_counts.entry(keyword).or_insert(0) += 1;
            }
        }
    }

    // Find the keyword with the highest share
    let total = fn_entries.len();
    let mut best: Option<(&str, usize)> = None;
    for (keyword, count) in &keyword_counts {
        if best.map_or(true, |(_, bc)| *count > bc) {
            best = Some((keyword, *count));
        }
    }

    if let Some((keyword, count)) = best {
        let share = count as f32 / total as f32;
        if share >= 0.70 {
            let theme_name = THEME_KEYWORDS
                .iter()
                .find(|(k, _)| *k == keyword)
                .map(|(_, v)| *v)
                .unwrap_or(keyword);
            return Some((
                theme_name.to_string(),
                0.85,
                format!(
                    "Function theme ({keyword}, {count}/{total} = {:.0}%) | {} definitions",
                    share * 100.0,
                    entries.len()
                ),
            ));
        }
    }

    None
}

/// Find the longest common prefix across function names (min 4 chars).
fn try_common_prefix(entries: &[&FunctionEntry]) -> Option<(String, f32, String)> {
    let fn_names: Vec<&str> = entries
        .iter()
        .filter(|e| e.definition_type == DefinitionType::Function)
        .map(|e| e.function_name.as_str())
        .collect();

    if fn_names.len() < 2 {
        return None;
    }

    let prefix = longest_common_prefix(&fn_names);
    // Trim trailing underscore
    let prefix = prefix.trim_end_matches('_');

    if prefix.len() >= 4 {
        return Some((
            prefix.to_string(),
            0.80,
            format!("Common prefix ({prefix}_*) | {} functions", fn_names.len()),
        ));
    }

    None
}

/// Extract dominant keyword from doc comments.
fn try_doc_comment_consensus(entries: &[&FunctionEntry]) -> Option<(String, f32, String)> {
    let docs: Vec<&str> = entries
        .iter()
        .filter_map(|e| e.doc_comment.as_deref())
        .collect();

    if docs.is_empty() {
        return None;
    }

    // Tokenize and count words (length >= 5, valid identifiers, not stopwords)
    let mut word_counts: HashMap<String, usize> = HashMap::new();
    for doc in &docs {
        let mut seen = std::collections::HashSet::new();
        for word in doc.split_whitespace() {
            let word = word
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase();
            if word.len() >= 5
                && is_valid_module_name(&word)
                && !is_stopword(&word)
                && seen.insert(word.clone())
            {
                *word_counts.entry(word).or_insert(0) += 1;
            }
        }
    }

    // Find the dominant word (appears in >50% of docs)
    let total = docs.len();
    let mut best: Option<(String, usize)> = None;
    for (word, count) in &word_counts {
        if best.as_ref().map_or(true, |(_, bc)| count > bc) {
            best = Some((word.clone(), *count));
        }
    }

    if let Some((word, count)) = best {
        let share = count as f32 / total as f32;
        if share >= 0.50 && count >= 2 {
            return Some((
                word.clone(),
                0.70,
                format!(
                    "Doc comment consensus ({word}, {count}/{total} docs) | {} definitions",
                    entries.len()
                ),
            ));
        }
    }

    None
}

// ── Helpers ────────────────────────────────────────────────────────────────

/// Convert CamelCase to snake_case.
pub(crate) fn to_snake_case(name: &str) -> String {
    let chars: Vec<char> = name.chars().collect();
    let mut result = String::with_capacity(name.len() + 4);
    let mut prev_lower = false;
    let mut prev_upper = false;

    for (i, &ch) in chars.iter().enumerate() {
        if ch.is_uppercase() {
            let needs_separator = prev_lower || needs_acronym_break(prev_upper, &chars, i);
            if needs_separator {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap_or(ch));
            prev_lower = false;
            prev_upper = true;
        } else {
            result.push(ch);
            prev_lower = ch.is_alphabetic();
            prev_upper = false;
        }
    }

    result
}

/// Check if we need an underscore before an uppercase char in an acronym sequence.
/// E.g., RMSNorm → rms_norm: the N needs a break because the next char is lowercase.
fn needs_acronym_break(prev_upper: bool, chars: &[char], i: usize) -> bool {
    prev_upper && chars.get(i + 1).is_some_and(|next| next.is_lowercase())
}

/// Replace the filename in a path, preserving the directory.
fn replace_filename(path: &str, new_filename: &str) -> String {
    if let Some(dir_end) = path.rfind('/') {
        format!("{}/{new_filename}", &path[..dir_end])
    } else {
        new_filename.to_string()
    }
}

/// Strip `_part_XX` segments from a filename stem.
fn strip_part_segments(stem: &str) -> String {
    let mut result = String::new();
    let mut rest = stem;

    while !rest.is_empty() {
        if let Some(idx) = rest.find("_part_") {
            result.push_str(&rest[..idx]);
            // Skip _part_XX (digits following)
            let after = &rest[idx + 6..];
            let digit_end = after
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(after.len());
            rest = &after[digit_end..];
        } else {
            result.push_str(rest);
            break;
        }
    }

    result
}

/// Detect parent file that includes this part file.
fn detect_parent_file(file_path: &str, index: &AgentContextIndex) -> Option<String> {
    // Look for parent mod.rs or the base file without _part_ suffix
    let dir = file_path.rfind('/').map(|i| &file_path[..i])?;
    let mod_rs = format!("{dir}/mod.rs");
    if index.file_index.contains_key(&mod_rs) {
        return Some(mod_rs);
    }

    // Try the base filename (strip _part_XX and .rs)
    let stem = Path::new(file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    let base = strip_part_segments(stem);
    if !base.is_empty() && base != stem {
        let parent = format!("{dir}/{base}.rs");
        if index.file_index.contains_key(&parent) {
            return Some(parent);
        }
    }

    None
}

/// Check if the suggested path already exists in the index.
pub(crate) fn check_collision(suggested_path: &str, index: &AgentContextIndex) -> bool {
    index.file_index.contains_key(suggested_path)
}

/// Find a context word from function names (most common non-trivial word).
fn find_context_word(entries: &[&FunctionEntry]) -> Option<String> {
    let mut word_counts: HashMap<String, usize> = HashMap::new();
    for entry in entries {
        if entry.definition_type != DefinitionType::Function {
            continue;
        }
        // Split by underscore, take words >= 4 chars
        for part in entry.function_name.split('_') {
            let part = part.to_lowercase();
            if part.len() >= 4 && !is_stopword(&part) {
                *word_counts.entry(part).or_insert(0) += 1;
            }
        }
    }

    word_counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(word, _)| word)
}

/// Compute the longest common prefix of a slice of strings.
fn longest_common_prefix(strings: &[&str]) -> String {
    if strings.is_empty() {
        return String::new();
    }
    let first: Vec<char> = strings[0].chars().collect();
    let mut prefix_len = first.len();

    for s in &strings[1..] {
        let chars: Vec<char> = s.chars().collect();
        prefix_len = prefix_len.min(chars.len());
        for i in 0..prefix_len {
            if first[i] != chars[i] {
                prefix_len = i;
                break;
            }
        }
    }

    first[..prefix_len].iter().collect()
}

/// Check if a word is a common English stopword.
fn is_stopword(word: &str) -> bool {
    matches!(
        word,
        "this"
            | "that"
            | "with"
            | "from"
            | "have"
            | "will"
            | "been"
            | "were"
            | "they"
            | "them"
            | "their"
            | "what"
            | "when"
            | "where"
            | "which"
            | "while"
            | "into"
            | "over"
            | "then"
            | "than"
            | "also"
            | "each"
            | "more"
            | "most"
            | "only"
            | "some"
            | "such"
            | "other"
            | "self"
            | "none"
            | "true"
            | "false"
            | "returns"
            | "return"
            | "function"
            | "should"
            | "given"
            | "expect"
            | "verify"
            | "check"
            | "assert"
            | "arguments"
            | "correctly"
            | "properly"
    )
}

/// Check if a name is valid as a Rust module name.
/// Must be ASCII, start with a letter or underscore, contain only alphanumerics/underscores.
fn is_valid_module_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    let first = name.as_bytes()[0];
    let starts_valid = first.is_ascii_alphabetic() || first == b'_';
    starts_valid && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::agent_context::function_index::DefinitionType;
    use crate::services::agent_context::{FunctionEntry, QualityMetrics};

    fn make_entry(name: &str, def_type: DefinitionType, doc: Option<&str>) -> FunctionEntry {
        FunctionEntry {
            file_path: "test/mod_part_01.rs".to_string(),
            function_name: name.to_string(),
            signature: format!("fn {name}()"),
            definition_type: def_type,
            doc_comment: doc.map(|d| d.to_string()),
            source: String::new(),
            start_line: 1,
            end_line: 10,
            language: "rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: String::new(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: vec![],
        }
    }

    #[test]
    fn test_is_part_file_positive() {
        assert!(is_part_file("src/llm/mod_part_02.rs"));
        assert!(is_part_file("src/llm/mod_part_02_part_04.rs"));
        assert!(is_part_file("foo_part_03_attn.rs"));
        assert!(is_part_file("deep/path/utils_part_01.rs"));
    }

    #[test]
    fn test_is_part_file_negative() {
        assert!(!is_part_file("mod.rs"));
        assert!(!is_part_file("attention.rs"));
        assert!(!is_part_file("src/lib.rs"));
        assert!(!is_part_file("partial.rs"));
        assert!(!is_part_file("src/my_partition.rs"));
    }

    #[test]
    fn test_dominant_type_struct() {
        let entries = vec![
            make_entry("AttentionCache", DefinitionType::Struct, None),
            make_entry("new", DefinitionType::Function, None),
            make_entry("get", DefinitionType::Function, None),
        ];
        let refs: Vec<&FunctionEntry> = entries.iter().collect();
        let result = try_dominant_type(&refs);
        assert!(result.is_some());
        let (name, confidence, _) = result.unwrap();
        assert_eq!(name, "attention_cache");
        assert!((confidence - 0.95).abs() < 0.01);
    }

    #[test]
    fn test_dominant_type_enum() {
        let entries = vec![
            make_entry("TokenKind", DefinitionType::Enum, None),
            make_entry("from_str", DefinitionType::Function, None),
        ];
        let refs: Vec<&FunctionEntry> = entries.iter().collect();
        let result = try_dominant_type(&refs);
        assert!(result.is_some());
        let (name, confidence, _) = result.unwrap();
        assert_eq!(name, "token_kind");
        assert!((confidence - 0.95).abs() < 0.01);
    }

    #[test]
    fn test_function_theme_forward() {
        let entries = vec![
            make_entry("forward_pass", DefinitionType::Function, None),
            make_entry("forward_batch", DefinitionType::Function, None),
            make_entry("forward_single", DefinitionType::Function, None),
        ];
        let refs: Vec<&FunctionEntry> = entries.iter().collect();
        let result = try_function_theme(&refs);
        assert!(result.is_some());
        let (name, confidence, _) = result.unwrap();
        assert_eq!(name, "forward");
        assert!((confidence - 0.85).abs() < 0.01);
    }

    #[test]
    fn test_common_prefix() {
        let entries = vec![
            make_entry("serialize_json", DefinitionType::Function, None),
            make_entry("serialize_yaml", DefinitionType::Function, None),
            make_entry("serialize_toml", DefinitionType::Function, None),
        ];
        let refs: Vec<&FunctionEntry> = entries.iter().collect();
        let result = try_common_prefix(&refs);
        assert!(result.is_some());
        let (name, confidence, _) = result.unwrap();
        assert_eq!(name, "serialize");
        assert!((confidence - 0.80).abs() < 0.01);
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("AttentionCache"), "attention_cache");
        assert_eq!(to_snake_case("RMSNorm"), "rms_norm");
        assert_eq!(to_snake_case("HTMLParser"), "html_parser");
        assert_eq!(to_snake_case("simple"), "simple");
        assert_eq!(to_snake_case("IOError"), "io_error");
    }

    #[test]
    fn test_collision_lowers_confidence() {
        // Build a minimal index with a file that would collide
        let index = AgentContextIndex {
            functions: vec![make_entry("attention", DefinitionType::Function, None)],
            name_index: HashMap::new(),
            file_index: {
                let mut m = HashMap::new();
                m.insert("src/attention.rs".to_string(), vec![0]);
                m.insert("src/mod_part_01.rs".to_string(), vec![]);
                m
            },
            corpus: vec![],
            corpus_lower: vec![],
            name_frequency: HashMap::new(),
            calls: HashMap::new(),
            called_by: HashMap::new(),
            graph_metrics: vec![],
            project_root: std::path::PathBuf::from("."),
            manifest: crate::services::agent_context::IndexManifest {
                version: "test".to_string(),
                built_at: String::new(),
                project_root: ".".to_string(),
                function_count: 1,
                file_count: 1,
                languages: vec![],
                avg_tdg_score: 0.0,
                file_checksums: HashMap::new(),
                last_incremental_changes: 0,
            },
            db_path: None,
            coverage_off_files: std::collections::HashSet::new(),
        };

        assert!(check_collision("src/attention.rs", &index));
        assert!(!check_collision("src/cache.rs", &index));
    }

    #[test]
    fn test_generic_name_penalty() {
        let mut suggestions = vec![RenameSuggestion {
            current_path: "src/mod_part_01.rs".to_string(),
            suggested_name: "construction.rs".to_string(),
            suggested_path: "src/construction.rs".to_string(),
            confidence: 0.85,
            reasoning: "Function theme".to_string(),
            signal: RenameSignal::FunctionTheme,
            parent_file: None,
            inclusion_pattern: None,
            definition_count: 5,
        }];
        penalize_generic_names(&mut suggestions);
        assert!(
            suggestions[0].confidence < 0.60,
            "Generic name should be penalized: {}",
            suggestions[0].confidence
        );
        assert!(suggestions[0].reasoning.contains("[generic name penalty]"));
    }

    #[test]
    fn test_disambiguate_collisions() {
        let mut suggestions = vec![
            RenameSuggestion {
                current_path: "src/commands/run_part_03.rs".to_string(),
                suggested_name: "dispatch.rs".to_string(),
                suggested_path: "src/commands/dispatch.rs".to_string(),
                confidence: 0.85,
                reasoning: "Theme".to_string(),
                signal: RenameSignal::FunctionTheme,
                parent_file: None,
                inclusion_pattern: None,
                definition_count: 3,
            },
            RenameSuggestion {
                current_path: "src/commands/serve_part_02.rs".to_string(),
                suggested_name: "dispatch.rs".to_string(),
                suggested_path: "src/commands/dispatch.rs".to_string(),
                confidence: 0.85,
                reasoning: "Theme".to_string(),
                signal: RenameSignal::FunctionTheme,
                parent_file: None,
                inclusion_pattern: None,
                definition_count: 4,
            },
        ];
        disambiguate_collisions(&mut suggestions);

        // Both should now have unique names with numeric suffixes
        assert_ne!(suggestions[0].suggested_name, suggestions[1].suggested_name);
        assert!(
            suggestions[0].suggested_name.contains("dispatch_"),
            "got: {}",
            suggestions[0].suggested_name
        );
        // Both should be marked as disambiguated
        assert!(suggestions[0].reasoning.contains("[disambiguated]"));
        assert!(suggestions[1].reasoning.contains("[disambiguated]"));
        // Confidence should be reduced
        assert!(suggestions[0].confidence < 0.85);
    }

    #[test]
    fn test_strip_part_segments() {
        assert_eq!(strip_part_segments("mod_part_02"), "mod");
        assert_eq!(strip_part_segments("mod_part_02_part_04"), "mod");
        assert_eq!(strip_part_segments("utils_part_01_attn"), "utils_attn");
        assert_eq!(strip_part_segments("simple"), "simple");
    }

    #[test]
    fn test_collides_with_parent() {
        assert!(collides_with_parent(
            "src/mod.rs",
            &Some("src/mod.rs".to_string())
        ));
        assert!(!collides_with_parent(
            "src/attention.rs",
            &Some("src/mod.rs".to_string())
        ));
        assert!(!collides_with_parent("src/mod.rs", &None));
    }

    #[test]
    fn test_is_valid_module_name() {
        assert!(is_valid_module_name("attention_cache"));
        assert!(is_valid_module_name("forward"));
        assert!(is_valid_module_name("_private"));
        assert!(!is_valid_module_name(""));
        assert!(!is_valid_module_name("has-hyphen"));
        assert!(!is_valid_module_name("123numeric"));
        assert!(!is_valid_module_name("has space"));
    }

    #[test]
    fn test_matches_parent_dir() {
        assert!(matches_parent_dir("src/graph/mod_part_02.rs", "graph"));
        assert!(matches_parent_dir(
            "deep/path/cache/mod_part_01.rs",
            "cache"
        ));
        assert!(!matches_parent_dir("src/graph/mod_part_02.rs", "attention"));
        assert!(!matches_parent_dir("mod_part_02.rs", "graph"));
    }

    #[test]
    fn test_longest_common_prefix_unicode() {
        // Verify no panic on non-ASCII (CB-506 fix)
        assert_eq!(longest_common_prefix(&["café_a", "café_b"]), "café_");
        assert_eq!(longest_common_prefix(&["abc", "abd"]), "ab");
        assert_eq!(longest_common_prefix(&["xyz"]), "xyz");
        assert_eq!(longest_common_prefix(&[]), "");
    }
}
