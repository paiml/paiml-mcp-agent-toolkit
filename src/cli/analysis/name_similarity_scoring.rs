/// The name patterns for every scope, compiled ONCE for the process.
///
/// They were built inside `extract_names`, which runs once per file, so every
/// file in the tree paid to recompile its scope's patterns. Measured before the
/// hoist, a controlled same-bytes experiment put 80% of `analyze
/// name-similarity`'s runtime in a constant charged per FILE rather than per
/// byte — 201 near-empty files cost 302ms against 376ms for the same bytes in
/// 202 real ones — while `analyze complexity` over the same pair showed no gap.
/// Regex compilation was the only difference.
///
/// `expect` rather than `?`: these are compile-time literals, so a failure here
/// is a bug in this file and not a condition a caller can handle.
static NAME_PATTERNS: std::sync::LazyLock<NamePatterns> = std::sync::LazyLock::new(|| {
    use regex::Regex;
    let re = |p: &str| Regex::new(p).expect("static name pattern must compile");
    NamePatterns {
        functions: vec![
            (re(r"(?m)^(?:\w+\s+)*fn\s+(\w+)"), "function"),
            (re(r"(?m)^(?:\w+\s+)*function\s+(\w+)"), "function"),
            (re(r"(?m)^def\s+(\w+)"), "function"),
        ],
        types: vec![
            (re(r"(?m)^(?:\w+\s+)*struct\s+(\w+)"), "struct"),
            (re(r"(?m)^(?:\w+\s+)*class\s+(\w+)"), "class"),
            (re(r"(?m)^(?:\w+\s+)*enum\s+(\w+)"), "enum"),
            (re(r"(?m)^(?:\w+\s+)*interface\s+(\w+)"), "interface"),
        ],
        variables: vec![
            (re(r"(?m)^(?:\w+\s+)*let\s+(?:mut\s+)?(\w+)"), "variable"),
            (re(r"(?m)^(?:\w+\s+)*const\s+(\w+)"), "constant"),
            (re(r"(?m)^(?:\w+\s+)*var\s+(\w+)"), "variable"),
        ],
        all: vec![
            (re(r"(?m)^(?:\w+\s+)*fn\s+(\w+)"), "function"),
            (re(r"(?m)^(?:\w+\s+)*struct\s+(\w+)"), "struct"),
            (re(r"(?m)^(?:\w+\s+)*let\s+(?:mut\s+)?(\w+)"), "variable"),
            (re(r"(?m)^(?:\w+\s+)*const\s+(\w+)"), "constant"),
        ],
    }
});

struct NamePatterns {
    functions: Vec<(regex::Regex, &'static str)>,
    types: Vec<(regex::Regex, &'static str)>,
    variables: Vec<(regex::Regex, &'static str)>,
    all: Vec<(regex::Regex, &'static str)>,
}

// Extract names from content based on scope
fn extract_names(
    content: &str,
    file: &str,
    scope: crate::cli::SearchScope,
) -> Result<Vec<(String, String, usize, String)>> {
    let mut names = Vec::new();

    let patterns: &[(regex::Regex, &str)] = match scope {
        crate::cli::SearchScope::Functions => &NAME_PATTERNS.functions,
        crate::cli::SearchScope::Types => &NAME_PATTERNS.types,
        crate::cli::SearchScope::Variables => &NAME_PATTERNS.variables,
        crate::cli::SearchScope::All => &NAME_PATTERNS.all,
    };

    for (line_no, line) in content.lines().enumerate() {
        for (pattern, kind) in patterns {
            if let Some(captures) = pattern.captures(line) {
                if let Some(name_match) = captures.get(1) {
                    names.push((
                        name_match.as_str().to_string(),
                        file.to_string(),
                        line_no + 1,
                        (*kind).to_string(),
                    ));
                }
            }
        }
    }

    Ok(names)
}

// Find similar names
fn find_similar_names(
    query: &str,
    candidates: Vec<(String, String, usize, String)>,
    threshold: f32,
    phonetic: bool,
    fuzzy: bool,
    case_sensitive: bool,
) -> Result<Vec<NameMatch>> {
    use crate::cli::analysis_utilities::{calculate_edit_distance, calculate_soundex};

    let mut matches = Vec::new();
    let query_lower = if case_sensitive {
        query.to_string()
    } else {
        query.to_lowercase()
    };
    let query_soundex = if phonetic {
        calculate_soundex(query)
    } else {
        String::new()
    };

    for (name, file, line, kind) in candidates {
        let name_compare = if case_sensitive {
            name.clone()
        } else {
            name.to_lowercase()
        };

        // Calculate similarity
        let edit_distance = calculate_edit_distance(&query_lower, &name_compare);
        let max_len = query.len().max(name.len());
        let similarity = if max_len > 0 {
            1.0 - (edit_distance as f32 / max_len as f32)
        } else {
            0.0
        };

        // Check phonetic match
        let phonetic_match = if phonetic {
            calculate_soundex(&name) == query_soundex
        } else {
            false
        };

        // Apply fuzzy matching boost
        let final_score = if fuzzy && name_compare.contains(&query_lower) {
            (similarity + 0.3).min(1.0)
        } else {
            similarity
        };

        // Check threshold
        if final_score >= threshold || phonetic_match {
            matches.push(NameMatch {
                name,
                file,
                line,
                kind,
                similarity_score: final_score,
                edit_distance,
                phonetic_match,
            });
        }
    }

    Ok(matches)
}
