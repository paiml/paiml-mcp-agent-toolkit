/// Simple calculator library for demonstrating mutation testing
///
/// This example shows how mutation testing can detect gaps in test coverage.

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

/// Divide two numbers (returns None if division by zero)
pub fn divide(a: i32, b: i32) -> Option<i32> {
    if b == 0 {
        None
    } else {
        Some(a / b)
    }
}

/// Check if a number is even
pub fn is_even(n: i32) -> bool {
    n % 2 == 0
}

/// Calculate the maximum of two numbers
pub fn max(a: i32, b: i32) -> i32 {
    if a > b {
        a
    } else {
        b
    }
}

/// Calculate factorial (iterative)
pub fn factorial(n: u32) -> u64 {
    if n == 0 {
        return 1;
    }

    let mut result = 1u64;
    for i in 1..=n {
        result *= i as u64;
    }
    result
}

/// Check if a number is prime
pub fn is_prime(n: u32) -> bool {
    if n <= 1 {
        return false;
    }
    if n == 2 {
        return true;
    }
    if n % 2 == 0 {
        return false;
    }

    let limit = (n as f64).sqrt() as u32;
    for i in (3..=limit).step_by(2) {
        if n % i == 0 {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
        assert_eq!(add(-1, 1), 0);
        assert_eq!(add(0, 0), 0);
    }

    #[test]
    fn test_subtract() {
        assert_eq!(subtract(5, 3), 2);
        assert_eq!(subtract(1, 1), 0);
        assert_eq!(subtract(0, 5), -5);
    }

    #[test]
    fn test_multiply() {
        assert_eq!(multiply(3, 4), 12);
        assert_eq!(multiply(0, 5), 0);
        assert_eq!(multiply(-2, 3), -6);
    }

    #[test]
    fn test_divide() {
        assert_eq!(divide(10, 2), Some(5));
        assert_eq!(divide(7, 3), Some(2));
        assert_eq!(divide(5, 0), None);
    }

    #[test]
    fn test_is_even() {
        assert!(is_even(4));
        assert!(is_even(0));
        assert!(!is_even(3));
        assert!(!is_even(-1));
    }

    #[test]
    fn test_max() {
        assert_eq!(max(5, 3), 5);
        assert_eq!(max(2, 8), 8);
        assert_eq!(max(4, 4), 4);
    }

    #[test]
    fn test_factorial() {
        assert_eq!(factorial(0), 1);
        assert_eq!(factorial(1), 1);
        assert_eq!(factorial(5), 120);
        assert_eq!(factorial(10), 3628800);
    }

    #[test]
    fn test_is_prime() {
        assert!(!is_prime(0));
        assert!(!is_prime(1));
        assert!(is_prime(2));
        assert!(is_prime(3));
        assert!(!is_prime(4));
        assert!(is_prime(5));
        assert!(!is_prime(9));
        assert!(is_prime(11));
        assert!(!is_prime(15));
        assert!(is_prime(17));
    }
}
