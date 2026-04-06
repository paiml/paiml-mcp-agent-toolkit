// Helper functions

/// Detect identity operations like +0, *1, -0, /1
fn detect_identity_operations(original: &str, mutated: &str) -> bool {
    debug_assert!(!original.is_empty(), "original must not be empty");
    debug_assert!(!mutated.is_empty(), "mutated must not be empty");
    // Check if original has identity op and mutated removes it
    let has_add_zero = original.contains("+ 0") || original.contains("+0");
    let has_mul_one = original.contains("* 1") || original.contains("*1");
    let has_sub_zero = original.contains("- 0") || original.contains("-0");
    let has_div_one = original.contains("/ 1") || original.contains("/1");
    let has_mul_zero =
        original.contains("* 0") || original.contains("*0") || original.contains("0 *");

    let identity_removed = (has_add_zero || has_mul_one || has_sub_zero || has_div_one)
        && (mutated.len() < original.len());

    let mul_zero_simplified = has_mul_zero && mutated.trim() == "0";

    identity_removed || mul_zero_simplified
}

/// Detect boolean tautologies
fn detect_boolean_tautology(original: &str, mutated: &str) -> bool {
    debug_assert!(!original.is_empty(), "original must not be empty");
    debug_assert!(!mutated.is_empty(), "mutated must not be empty");
    detect_or_true_tautology(original, mutated)
        || detect_and_false_contradiction(original, mutated)
        || detect_or_false_identity(original, mutated)
        || detect_and_true_identity(original, mutated)
        || detect_double_negation(original, mutated)
}

/// Detects x || true → true (tautology simplification)
fn detect_or_true_tautology(original: &str, mutated: &str) -> bool {
    debug_assert!(!original.is_empty(), "original must not be empty");
    debug_assert!(!mutated.is_empty(), "mutated must not be empty");
    original.contains("|| true")
        && (mutated.contains("{ true }") || mutated.trim().ends_with("true"))
        && mutated.len() < original.len()
        && !mutated.contains("||")
}

/// Detects x && false → false (contradiction simplification)
fn detect_and_false_contradiction(original: &str, mutated: &str) -> bool {
    debug_assert!(!original.is_empty(), "original must not be empty");
    debug_assert!(!mutated.is_empty(), "mutated must not be empty");
    original.contains("&& false")
        && (mutated.contains("{ false }") || mutated.trim().ends_with("false"))
        && mutated.len() < original.len()
        && !mutated.contains("&&")
}

/// Detects x || false → x (identity for OR)
fn detect_or_false_identity(original: &str, mutated: &str) -> bool {
    debug_assert!(!original.is_empty(), "original must not be empty");
    debug_assert!(!mutated.is_empty(), "mutated must not be empty");
    original.contains("|| false")
        && !mutated.contains("||")
        && !mutated.contains("false")
        && mutated.len() < original.len()
}

/// Detects x && true → x (identity for AND)
fn detect_and_true_identity(original: &str, mutated: &str) -> bool {
    debug_assert!(!original.is_empty(), "original must not be empty");
    debug_assert!(!mutated.is_empty(), "mutated must not be empty");
    original.contains("&& true")
        && !mutated.contains("&&")
        && !mutated.contains("true")
        && mutated.len() < original.len()
}

/// Detects !!x → x (double negation elimination)
fn detect_double_negation(original: &str, mutated: &str) -> bool {
    debug_assert!(!original.is_empty(), "original must not be empty");
    debug_assert!(!mutated.is_empty(), "mutated must not be empty");
    original.contains("!!") && !mutated.contains("!!")
}

/// Detect commutative operation swap
fn detect_commutative_swap(original: &str, mutated: &str) -> bool {
    debug_assert!(!original.is_empty(), "original must not be empty");
    debug_assert!(!mutated.is_empty(), "mutated must not be empty");
    let orig_tokens: Vec<&str> = original.split_whitespace().collect();
    let mut_tokens: Vec<&str> = mutated.split_whitespace().collect();

    if orig_tokens.len() != mut_tokens.len() || orig_tokens.len() < 3 {
        return false;
    }

    // Check each potential commutative operation
    for i in 0..orig_tokens.len() - 2 {
        if is_commutative_op(orig_tokens[i + 1])
            && has_swapped_operands(&orig_tokens, &mut_tokens, i)
        {
            return true;
        }
    }

    false
}

/// Check if operands are swapped at given position
fn has_swapped_operands(orig: &[&str], mutated: &[&str], pos: usize) -> bool {
    let (op, a, b) = (orig[pos + 1], orig[pos], orig[pos + 2]);

    mutated
        .windows(3)
        .any(|window| window[1] == op && window[0] == b && window[2] == a)
}

/// Check if operator is commutative
fn is_commutative_op(op: &str) -> bool {
    debug_assert!(!op.is_empty(), "op must not be empty");
    matches!(op, "+" | "*" | "&&" | "||" | "==" | "!=")
}

/// Extract operator patterns from source pair
fn extract_operator_patterns(original: &str, mutated: &str) -> Vec<String> {
    debug_assert!(!original.is_empty(), "original must not be empty");
    debug_assert!(!mutated.is_empty(), "mutated must not be empty");
    let mut patterns = Vec::new();

    // Identity operations
    if original.contains("+ 0") || original.contains("+0") {
        patterns.push("add_zero_identity".to_string());
    }
    if original.contains("* 1") || original.contains("*1") {
        patterns.push("mul_one_identity".to_string());
    }
    if original.contains("- 0") || original.contains("-0") {
        patterns.push("sub_zero_identity".to_string());
    }
    if original.contains("/ 1") || original.contains("/1") {
        patterns.push("div_one_identity".to_string());
    }

    // Boolean patterns
    if original.contains("|| true") {
        patterns.push("or_true_tautology".to_string());
    }
    if original.contains("&& false") {
        patterns.push("and_false_tautology".to_string());
    }
    if original.contains("!!") {
        patterns.push("double_negation".to_string());
    }

    // Associative patterns
    if original.contains("(")
        && mutated.contains("(")
        && original.matches('(').count() == mutated.matches('(').count()
    {
        patterns.push("associative_grouping".to_string());
    }

    patterns
}

/// Calculate token-based similarity
fn calculate_token_similarity(s1: &str, s2: &str) -> f64 {
    debug_assert!(!s1.is_empty(), "s1 must not be empty");
    debug_assert!(!s2.is_empty(), "s2 must not be empty");
    let tokens1: Vec<&str> = s1.split_whitespace().collect();
    let tokens2: Vec<&str> = s2.split_whitespace().collect();

    let max_len = tokens1.len().max(tokens2.len());
    if max_len == 0 {
        return 1.0;
    }

    let common = tokens1.iter().filter(|t| tokens2.contains(t)).count();
    common as f64 / max_len as f64
}

/// Calculate Levenshtein distance
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    debug_assert!(!s1.is_empty(), "s1 must not be empty");
    debug_assert!(!s2.is_empty(), "s2 must not be empty");
    let len1 = s1.len();
    let len2 = s2.len();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut prev_row: Vec<usize> = (0..=len2).collect();
    let mut curr_row = vec![0; len2 + 1];

    for (i, c1) in s1.chars().enumerate() {
        curr_row[0] = i + 1;

        for (j, c2) in s2.chars().enumerate() {
            let cost = if c1 == c2 { 0 } else { 1 };
            curr_row[j + 1] = (curr_row[j] + 1)
                .min(prev_row[j + 1] + 1)
                .min(prev_row[j] + cost);
        }

        std::mem::swap(&mut prev_row, &mut curr_row);
    }

    prev_row[len2]
}
