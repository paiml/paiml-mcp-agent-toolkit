/// Calculate Levenshtein distance between two strings
#[must_use]
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_len = a.len();
    let b_len = b.len();

    // Handle empty string cases
    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    // Initialize distance matrix
    let mut matrix = initialize_distance_matrix(a_len, b_len);

    // Calculate distances using dynamic programming
    calculate_edit_distances(&mut matrix, a, b);

    matrix[a_len][b_len]
}

/// Initialize the distance matrix with base values
fn initialize_distance_matrix(a_len: usize, b_len: usize) -> Vec<Vec<usize>> {
    let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];

    // Initialize first column (deletions from source)
    for i in 0..=a_len {
        matrix[i][0] = i;
    }

    // Initialize first row (insertions to match target)
    for j in 0..=b_len {
        matrix[0][j] = j;
    }

    matrix
}

/// Calculate edit distances for all positions in the matrix
fn calculate_edit_distances(matrix: &mut Vec<Vec<usize>>, a: &str, b: &str) {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    for i in 1..=a_chars.len() {
        for j in 1..=b_chars.len() {
            matrix[i][j] = calculate_cell_distance(matrix, i, j, a_chars[i - 1] == b_chars[j - 1]);
        }
    }
}

/// Calculate the minimum edit distance for a single cell
fn calculate_cell_distance(matrix: &[Vec<usize>], i: usize, j: usize, chars_match: bool) -> usize {
    let substitution_cost = usize::from(!chars_match);

    let deletion_cost = matrix[i - 1][j] + 1;
    let insertion_cost = matrix[i][j - 1] + 1;
    let substitution = matrix[i - 1][j - 1] + substitution_cost;

    deletion_cost.min(insertion_cost).min(substitution)
}
