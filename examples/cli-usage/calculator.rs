/// Simple calculator for mutation testing demonstration
///
/// This example demonstrates how mutation testing can find gaps in test coverage.
/// Run: `pmat analyze mutate --path examples/cli-usage/calculator.rs`

/// Add two numbers
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Subtract two numbers
pub fn subtract(a: i32, b: i32) -> i32 {
    a - b
}

/// Multiply two numbers
pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

/// Divide two numbers
pub fn divide(a: i32, b: i32) -> Option<i32> {
    if b == 0 {
        None
    } else {
        Some(a / b)
    }
}

/// Check if number is positive
pub fn is_positive(n: i32) -> bool {
    n > 0
}

/// Check if number is even
pub fn is_even(n: i32) -> bool {
    n % 2 == 0
}

/// Find maximum of two numbers
pub fn max(a: i32, b: i32) -> i32 {
    if a > b {
        a
    } else {
        b
    }
}

/// Check if two numbers are equal
pub fn are_equal(a: i32, b: i32) -> bool {
    a == b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
        assert_eq!(add(0, 0), 0);
        assert_eq!(add(-1, 1), 0);
        assert_eq!(add(100, 200), 300);
    }

    #[test]
    fn test_subtract() {
        assert_eq!(subtract(5, 3), 2);
        assert_eq!(subtract(0, 0), 0);
        // Missing: negative results, edge cases
    }

    #[test]
    fn test_multiply() {
        assert_eq!(multiply(2, 3), 6);
        assert_eq!(multiply(0, 5), 0);
        // Missing: negative numbers, -1 edge case
    }

    #[test]
    fn test_divide() {
        assert_eq!(divide(6, 2), Some(3));
        assert_eq!(divide(5, 0), None);
        // Missing: negative division, division by 1
    }

    #[test]
    fn test_is_positive() {
        assert!(is_positive(1));
        assert!(!is_positive(0));
        // Missing: negative numbers
    }

    #[test]
    fn test_is_even() {
        assert!(is_even(2));
        assert!(is_even(0));
        assert!(!is_even(1));
        // Good coverage!
    }

    #[test]
    fn test_max() {
        assert_eq!(max(5, 3), 5);
        assert_eq!(max(3, 5), 5);
        // Missing: equal numbers edge case!
    }

    // Note: are_equal has NO tests!
    // Mutation testing will reveal this gap with 0% score
}
