// Extract names from content based on scope
fn extract_names(
    content: &str,
    file: &str,
    scope: crate::cli::SearchScope,
) -> Result<Vec<(String, String, usize, String)>> {
    debug_assert!(!content.is_empty(), "content must not be empty");
    debug_assert!(!file.is_empty(), "file must not be empty");
    use regex::Regex;

    let mut names = Vec::new();

    let patterns = match scope {
        crate::cli::SearchScope::Functions => vec![
            (Regex::new(r"(?m)^(?:\w+\s+)*fn\s+(\w+)")?, "function"),
            (Regex::new(r"(?m)^(?:\w+\s+)*function\s+(\w+)")?, "function"),
            (Regex::new(r"(?m)^def\s+(\w+)")?, "function"),
        ],
        crate::cli::SearchScope::Types => vec![
            (Regex::new(r"(?m)^(?:\w+\s+)*struct\s+(\w+)")?, "struct"),
            (Regex::new(r"(?m)^(?:\w+\s+)*class\s+(\w+)")?, "class"),
            (Regex::new(r"(?m)^(?:\w+\s+)*enum\s+(\w+)")?, "enum"),
            (
                Regex::new(r"(?m)^(?:\w+\s+)*interface\s+(\w+)")?,
                "interface",
            ),
        ],
        crate::cli::SearchScope::Variables => vec![
            (
                Regex::new(r"(?m)^(?:\w+\s+)*let\s+(?:mut\s+)?(\w+)")?,
                "variable",
            ),
            (Regex::new(r"(?m)^(?:\w+\s+)*const\s+(\w+)")?, "constant"),
            (Regex::new(r"(?m)^(?:\w+\s+)*var\s+(\w+)")?, "variable"),
        ],
        crate::cli::SearchScope::All => vec![
            (Regex::new(r"(?m)^(?:\w+\s+)*fn\s+(\w+)")?, "function"),
            (Regex::new(r"(?m)^(?:\w+\s+)*struct\s+(\w+)")?, "struct"),
            (
                Regex::new(r"(?m)^(?:\w+\s+)*let\s+(?:mut\s+)?(\w+)")?,
                "variable",
            ),
            (Regex::new(r"(?m)^(?:\w+\s+)*const\s+(\w+)")?, "constant"),
        ],
    };

    for (line_no, line) in content.lines().enumerate() {
        for (pattern, kind) in &patterns {
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
    debug_assert!(!query.is_empty(), "query must not be empty");
    debug_assert!(threshold >= 0.0, "threshold must be non-negative");
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
