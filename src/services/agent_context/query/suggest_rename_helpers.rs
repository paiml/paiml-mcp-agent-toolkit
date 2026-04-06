// ── Helpers ────────────────────────────────────────────────────────────────

/// Convert CamelCase to snake_case.
pub(crate) fn to_snake_case(name: &str) -> String {
    debug_assert!(!name.is_empty(), "name must not be empty");
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
    debug_assert!(!path.is_empty(), "path must not be empty");
    debug_assert!(!new_filename.is_empty(), "new_filename must not be empty");
    if let Some(dir_end) = path.rfind('/') {
        format!("{}/{new_filename}", &path[..dir_end])
    } else {
        new_filename.to_string()
    }
}

/// Strip `_part_XX` segments from a filename stem.
fn strip_part_segments(stem: &str) -> String {
    debug_assert!(!stem.is_empty(), "stem must not be empty");
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
    debug_assert!(!file_path.is_empty(), "file_path must not be empty");
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
    debug_assert!(!suggested_path.is_empty(), "suggested_path must not be empty");
    index.file_index.contains_key(suggested_path)
}

/// Find a context word from function names (most common non-trivial word).
pub(crate) fn find_context_word(entries: &[&FunctionEntry]) -> Option<String> {
    debug_assert!(!entries.is_empty(), "entries must not be empty");
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
    debug_assert!(!word.is_empty(), "word must not be empty");
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
    debug_assert!(!name.is_empty(), "name must not be empty");
    if name.is_empty() {
        return false;
    }
    let first = name.as_bytes()[0];
    let starts_valid = first.is_ascii_alphabetic() || first == b'_';
    starts_valid && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

