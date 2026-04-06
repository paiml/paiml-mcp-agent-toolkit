// String utilities - extracted for file health (CB-040)
// Name similarity helpers
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn extract_identifiers(content: &str) -> Vec<super::NameInfo> {
    let mut identifiers = Vec::new();
    let mut seen = HashSet::new();

    let patterns = get_identifier_patterns();

    for (pattern_str, kind) in patterns {
        extract_identifiers_for_pattern(content, pattern_str, kind, &mut identifiers, &mut seen);
    }

    identifiers
}

/// Get identifier extraction patterns for different languages
fn get_identifier_patterns() -> Vec<(&'static str, &'static str)> {
    vec![
        // Function/method definitions
        (r"(?m)^\s*(?:pub\s+)?(?:async\s+)?fn\s+(\w+)", "function"),
        (r"(?m)^\s*def\s+(\w+)", "function"),
        (r"(?m)^\s*function\s+(\w+)", "function"),
        (
            r"(?m)^\s*(?:public|private|protected)?\s*(?:static)?\s*\w+\s+(\w+)\s*\(",
            "function",
        ),
        // Class/struct/interface definitions
        (r"(?m)^\s*(?:pub\s+)?struct\s+(\w+)", "struct"),
        (r"(?m)^\s*(?:pub\s+)?enum\s+(\w+)", "enum"),
        (r"(?m)^\s*(?:pub\s+)?trait\s+(\w+)", "trait"),
        (r"(?m)^\s*class\s+(\w+)", "class"),
        (r"(?m)^\s*interface\s+(\w+)", "interface"),
        (r"(?m)^\s*type\s+(\w+)", "type"),
        // Variable/constant definitions
        (r"(?m)^\s*(?:pub\s+)?(?:const|static)\s+(\w+)", "constant"),
        (r"(?m)^\s*(?:let|const|var)\s+(\w+)", "variable"),
        (r"(?m)^\s*(\w+)\s*=\s*", "variable"),
    ]
}

/// Extract identifiers for a specific pattern
fn extract_identifiers_for_pattern(
    content: &str,
    pattern_str: &str,
    kind: &str,
    identifiers: &mut Vec<super::NameInfo>,
    seen: &mut HashSet<String>,
) {
    use regex::Regex;

    if let Ok(re) = Regex::new(pattern_str) {
        for (line_num, line) in content.lines().enumerate() {
            for cap in re.captures_iter(line) {
                if let Some(name_match) = cap.get(1) {
                    let name = name_match.as_str().to_string();

                    // Skip if we've already seen this identifier
                    if seen.insert(name.clone()) {
                        identifiers.push(super::NameInfo {
                            name,
                            kind: kind.to_string(),
                            file_path: PathBuf::from(""), // Will be filled by caller
                            line: line_num + 1,
                        });
                    }
                }
            }
        }
    }
}

/// Calculates normalized string similarity using Levenshtein distance
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::cli::analysis_utilities::calculate_string_similarity;
///
/// assert_eq!(calculate_string_similarity("hello", "hello"), 1.0);
/// assert_eq!(calculate_string_similarity("", ""), 1.0);
/// assert!(calculate_string_similarity("hello", "xyz") < 0.5);
/// ```
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
pub fn calculate_string_similarity(s1: &str, s2: &str) -> f32 {
    // Normalized Levenshtein distance for basic string similarity
    if s1.is_empty() && s2.is_empty() {
        return 1.0;
    }

    if s1 == s2 {
        return 1.0;
    }

    // Calculate Jaccard similarity based on character n-grams
    let n = 2; // bigrams
    let ngrams1 = get_ngrams(s1, n);
    let ngrams2 = get_ngrams(s2, n);

    if ngrams1.is_empty() && ngrams2.is_empty() {
        // Fall back to exact character matching for very short strings
        let common_chars = s1.chars().filter(|c| s2.contains(*c)).count();
        let total_chars = s1.len().max(s2.len());
        return if total_chars > 0 {
            common_chars as f32 / total_chars as f32
        } else {
            0.0
        };
    }

    let intersection: HashSet<_> = ngrams1.intersection(&ngrams2).cloned().collect();
    let union: HashSet<_> = ngrams1.union(&ngrams2).cloned().collect();

    if union.is_empty() {
        0.0
    } else {
        intersection.len() as f32 / union.len() as f32
    }
}

/// Get character n-grams from a string
fn get_ngrams(s: &str, n: usize) -> HashSet<String> {
    let chars: Vec<char> = s.chars().collect();
    let mut ngrams = HashSet::new();

    if chars.len() >= n {
        for i in 0..=chars.len() - n {
            let ngram: String = chars[i..i + n].iter().collect();
            ngrams.insert(ngram);
        }
    } else {
        // For strings shorter than n, use the whole string as an n-gram
        ngrams.insert(s.to_string());
    }

    ngrams
}

/// Calculates the Levenshtein edit distance between two strings
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::cli::analysis_utilities::calculate_edit_distance;
///
/// assert_eq!(calculate_edit_distance("kitten", "sitting"), 3);
/// assert_eq!(calculate_edit_distance("hello", "hello"), 0);
/// assert_eq!(calculate_edit_distance("", "abc"), 3);
/// ```
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
pub fn calculate_edit_distance(s1: &str, s2: &str) -> usize {
    // Levenshtein distance implementation
    let len1 = s1.chars().count();
    let len2 = s2.chars().count();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    // Create a 2D matrix for dynamic programming
    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    // Initialize first row and column
    for (i, row) in matrix.iter_mut().enumerate().take(len1 + 1) {
        row[0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    // Fill the matrix
    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = usize::from(s1_chars[i - 1] != s2_chars[j - 1]);

            matrix[i][j] = std::cmp::min(
                std::cmp::min(
                    matrix[i - 1][j] + 1, // deletion
                    matrix[i][j - 1] + 1, // insertion
                ),
                matrix[i - 1][j - 1] + cost, // substitution
            );
        }
    }

    matrix[len1][len2]
}

#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
pub fn calculate_soundex(s: &str) -> String {
    // Soundex phonetic algorithm implementation
    if s.is_empty() {
        return String::new();
    }

    let s_upper = s.to_uppercase();
    let chars: Vec<char> = s_upper.chars().filter(|c| c.is_alphabetic()).collect();

    if chars.is_empty() {
        return String::new();
    }

    let mut soundex = String::new();
    soundex.push(chars[0]);

    let mut prev_code = soundex_code(chars[0]);

    for &ch in &chars[1..] {
        let code = soundex_code(ch);

        // Skip if same as previous code or if it's 0 (vowels and similar)
        if code != '0' && code != prev_code {
            soundex.push(code);
            prev_code = code;

            // Soundex codes are traditionally 4 characters
            if soundex.len() >= 4 {
                break;
            }
        } else if code == '0' {
            // Reset prev_code for vowels to allow consonants after vowels
            prev_code = '0';
        }
    }

    // Pad with zeros if necessary
    while soundex.len() < 4 {
        soundex.push('0');
    }

    // Ensure exactly 4 characters
    soundex.truncate(4);
    soundex
}

/// Get Soundex code for a character
fn soundex_code(ch: char) -> char {
    match ch {
        'B' | 'F' | 'P' | 'V' => '1',
        'C' | 'G' | 'J' | 'K' | 'Q' | 'S' | 'X' | 'Z' => '2',
        'D' | 'T' => '3',
        'L' => '4',
        'M' | 'N' => '5',
        'R' => '6',
        _ => '0', // A, E, I, O, U, H, W, Y and others
    }
}

// Helper function for params conversion
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn params_to_json(
    params: Vec<(String, serde_json::Value)>,
) -> serde_json::Map<String, serde_json::Value> {
    params.into_iter().collect()
}

// Table printing function
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn print_table(items: &[std::sync::Arc<crate::models::template::TemplateResource>]) {
    if items.is_empty() {
        println!("No templates found.");
        return;
    }

    // Calculate column widths
    let mut name_width = "Name".len();
    let mut toolchain_width = "Toolchain".len();
    let mut category_width = "Category".len();
    let mut desc_width = "Description".len();

    for item in items {
        name_width = name_width.max(item.name.len());
        toolchain_width = toolchain_width.max(item.toolchain.as_str().len());
        category_width = category_width.max(format!("{:?}", item.category).len());
        desc_width = desc_width.max(60.min(item.description.len()));
    }

    // Add padding
    name_width += 2;
    toolchain_width += 2;
    category_width += 2;
    desc_width += 2;

    // Print header
    println!(
        "┌{}┬{}┬{}┬{}┐",
        "─".repeat(name_width),
        "─".repeat(toolchain_width),
        "─".repeat(category_width),
        "─".repeat(desc_width)
    );

    println!(
        "│{:^name_width$}│{:^toolchain_width$}│{:^category_width$}│{:^desc_width$}│",
        "Name",
        "Toolchain",
        "Category",
        "Description",
        name_width = name_width,
        toolchain_width = toolchain_width,
        category_width = category_width,
        desc_width = desc_width
    );

    println!(
        "├{}┼{}┼{}┼{}┤",
        "─".repeat(name_width),
        "─".repeat(toolchain_width),
        "─".repeat(category_width),
        "─".repeat(desc_width)
    );

    // Print rows
    for item in items {
        let toolchain = item.toolchain.as_str();
        let category = format!("{:?}", item.category);
        let description = item.description.chars().take(60).collect::<String>();
        let description = if item.description.len() > 60 {
            format!("{description}...")
        } else {
            description
        };

        println!(
            "│{:<name_width$}│{:<toolchain_width$}│{:<category_width$}│{:<desc_width$}│",
            format!(" {} ", item.name),
            format!(" {} ", toolchain),
            format!(" {} ", category),
            format!(" {} ", description),
            name_width = name_width,
            toolchain_width = toolchain_width,
            category_width = category_width,
            desc_width = desc_width
        );
    }

    // Print footer
    println!(
        "└{}┴{}┴{}┴{}┘",
        "─".repeat(name_width),
        "─".repeat(toolchain_width),
        "─".repeat(category_width),
        "─".repeat(desc_width)
    );
}

// Deleted estimate_cyclomatic_complexity - using proper AST analysis instead

// Comprehensive analysis helper functions
