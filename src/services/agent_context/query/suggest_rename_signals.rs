// ── Signal analyzers ───────────────────────────────────────────────────────

/// Check for a single dominant struct/enum/trait definition.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn try_dominant_type(entries: &[&FunctionEntry]) -> Option<(String, f32, String)> {
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
        type_method_counts.sort_by_key(|b| std::cmp::Reverse(b.1));

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

/// Try to use the original filename base (before `_part_` splitting) as the rename.
/// Rejects generic bases like "mod", "tests", "lib", and anything in GENERIC_NAMES.
fn try_original_base(file_path: &str) -> Option<(String, f32, String)> {
    let stem = Path::new(file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    let base = strip_part_segments(stem);

    // Reject too-short, generic, or identical-to-stem bases
    if base.len() < 4
        || base.len() > 30
        || base == stem
        || base.starts_with("mod_")
        || base.contains("_from_")
        || matches!(base.as_str(), "mod" | "lib" | "main" | "tests" | "test")
        || GENERIC_NAMES.contains(&base.as_str())
        || !is_valid_module_name(&base)
    {
        return None;
    }

    Some((
        base.clone(),
        0.82,
        format!("Original filename base ({base})"),
    ))
}

/// Check if >70% of functions share a keyword theme.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn try_function_theme(entries: &[&FunctionEntry]) -> Option<(String, f32, String)> {
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn try_common_prefix(entries: &[&FunctionEntry]) -> Option<(String, f32, String)> {
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn try_doc_comment_consensus(
    entries: &[&FunctionEntry],
) -> Option<(String, f32, String)> {
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

