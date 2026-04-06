/// Calculate similarity scores for all names
#[must_use]
pub fn calculate_similarities(
    all_names: &[NameInfo],
    query: &str,
    threshold: f32,
    case_sensitive: bool,
    fuzzy: bool,
    phonetic: bool,
) -> Vec<NameSimilarityResult> {
    debug_assert!(!query.is_empty(), "query must not be empty");
    let mut similarities = Vec::new();
    let query_lower = if case_sensitive {
        query.to_string()
    } else {
        query.to_lowercase()
    };

    for name_info in all_names {
        let name_to_compare = if case_sensitive {
            name_info.name.clone()
        } else {
            name_info.name.to_lowercase()
        };

        let similarity_score =
            calculate_combined_similarity(&query_lower, &name_to_compare, fuzzy, phonetic);

        if similarity_score >= threshold {
            similarities.push(NameSimilarityResult {
                name: name_info.name.clone(),
                kind: name_info.kind.clone(),
                file_path: name_info.file_path.clone(),
                line: name_info.line,
                similarity: similarity_score,
                phonetic_match: false,
                fuzzy_match: fuzzy,
            });
        }
    }

    similarities.sort_by(|a, b| {
        b.similarity
            .partial_cmp(&a.similarity)
            .expect("internal error")
    });
    similarities
}

/// Calculate combined similarity score
fn calculate_combined_similarity(query: &str, name: &str, fuzzy: bool, phonetic: bool) -> f32 {
    debug_assert!(!query.is_empty(), "query must not be empty");
    debug_assert!(!name.is_empty(), "name must not be empty");
    let mut score = super::analysis_utilities::calculate_string_similarity(query, name);

    if fuzzy {
        let edit_distance = super::analysis_utilities::calculate_edit_distance(query, name);
        let max_len = query.len().max(name.len()) as f32;
        let fuzzy_score = if max_len > 0.0 {
            1.0 - (edit_distance as f32 / max_len)
        } else {
            1.0
        };
        score = score.max(fuzzy_score);
    }

    if phonetic {
        let query_soundex = super::analysis_utilities::calculate_soundex(query);
        let name_soundex = super::analysis_utilities::calculate_soundex(name);
        if query_soundex == name_soundex {
            score = score.max(0.8);
        }
    }

    score
}
