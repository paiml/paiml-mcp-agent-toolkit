// tests/calculator_test.rs
// Comprehensive test suite for Rust calculator
// Demonstrates mutation testing with various operators

use rust_fixtures::calculator::*;

// ============================================================
// ARITHMETIC OPERATOR TESTS (AOR)
// ============================================================

#[test]
fn test_add() {
    assert_eq!(add(2, 3), 5);
    assert_eq!(add(-1, -1), -2);
    assert_eq!(add(0, 5), 5);
    assert_eq!(add(-3, 5), 2);
    assert_eq!(add(100, 200), 300);
}

#[test]
fn test_subtract() {
    assert_eq!(subtract(5, 3), 2);
    assert_eq!(subtract(-1, -1), 0);
    assert_eq!(subtract(0, 5), -5);
    assert_eq!(subtract(-3, 5), -8);
    assert_eq!(subtract(10, 3), 7);
}

#[test]
fn test_multiply() {
    assert_eq!(multiply(2, 3), 6);
    assert_eq!(multiply(-2, 3), -6);
    assert_eq!(multiply(0, 5), 0);
    assert_eq!(multiply(-2, -3), 6);
    assert_eq!(multiply(7, 8), 56);
}

#[test]
fn test_divide() {
    assert_eq!(divide(6, 3), 2);
    assert_eq!(divide(-6, 3), -2);
    assert_eq!(divide(5, 2), 2); // Integer division
    assert_eq!(divide(5, 0), 0); // Division by zero
    assert_eq!(divide(20, 4), 5);
}

#[test]
fn test_modulo() {
    assert_eq!(modulo(7, 3), 1);
    assert_eq!(modulo(10, 5), 0);
    assert_eq!(modulo(5, 2), 1);
    assert_eq!(modulo(5, 0), 0); // Modulo by zero
    assert_eq!(modulo(13, 4), 1);
}

// ============================================================
// RELATIONAL OPERATOR TESTS (ROR)
// ============================================================

#[test]
fn test_greater_than() {
    assert!(greater_than(5, 3));
    assert!(!greater_than(3, 5));
    assert!(!greater_than(3, 3));
    assert!(greater_than(100, 50));
    assert!(!greater_than(-5, 0));
}

#[test]
fn test_less_than() {
    assert!(less_than(3, 5));
    assert!(!less_than(5, 3));
    assert!(!less_than(3, 3));
    assert!(less_than(-10, 0));
    assert!(!less_than(100, 50));
}

#[test]
fn test_greater_or_equal() {
    assert!(greater_or_equal(5, 3));
    assert!(greater_or_equal(3, 3));
    assert!(!greater_or_equal(3, 5));
    assert!(greater_or_equal(10, 10));
    assert!(!greater_or_equal(5, 10));
}

#[test]
fn test_less_or_equal() {
    assert!(less_or_equal(3, 5));
    assert!(less_or_equal(3, 3));
    assert!(!less_or_equal(5, 3));
    assert!(less_or_equal(10, 10));
    assert!(!less_or_equal(15, 10));
}

#[test]
fn test_equal() {
    assert!(equal(3, 3));
    assert!(!equal(3, 5));
    assert!(equal(0, 0));
    assert!(equal(-5, -5));
    assert!(!equal(10, 11));
}

#[test]
fn test_not_equal() {
    assert!(not_equal(3, 5));
    assert!(!not_equal(3, 3));
    assert!(not_equal(10, 20));
    assert!(!not_equal(-5, -5));
    assert!(not_equal(0, 1));
}

// ============================================================
// LOGICAL OPERATOR TESTS (LOR)
// ============================================================

#[test]
fn test_logical_and() {
    assert!(logical_and(true, true));
    assert!(!logical_and(true, false));
    assert!(!logical_and(false, true));
    assert!(!logical_and(false, false));
}

#[test]
fn test_logical_or() {
    assert!(logical_or(true, true));
    assert!(logical_or(true, false));
    assert!(logical_or(false, true));
    assert!(!logical_or(false, false));
}

// ============================================================
// BITWISE OPERATOR TESTS (BOR)
// ============================================================

#[test]
fn test_bitwise_and() {
    assert_eq!(bitwise_and(5, 3), 1);   // 101 & 011 = 001
    assert_eq!(bitwise_and(12, 10), 8); // 1100 & 1010 = 1000
    assert_eq!(bitwise_and(15, 7), 7);  // 1111 & 0111 = 0111
    assert_eq!(bitwise_and(0, 255), 0);
}

#[test]
fn test_bitwise_or() {
    assert_eq!(bitwise_or(5, 3), 7);    // 101 | 011 = 111
    assert_eq!(bitwise_or(12, 10), 14); // 1100 | 1010 = 1110
    assert_eq!(bitwise_or(8, 4), 12);   // 1000 | 0100 = 1100
    assert_eq!(bitwise_or(0, 255), 255);
}

#[test]
fn test_bitwise_xor() {
    assert_eq!(bitwise_xor(5, 3), 6);   // 101 ^ 011 = 110
    assert_eq!(bitwise_xor(12, 10), 6); // 1100 ^ 1010 = 0110
    assert_eq!(bitwise_xor(15, 15), 0); // Same values = 0
    assert_eq!(bitwise_xor(255, 0), 255);
}

#[test]
fn test_left_shift() {
    assert_eq!(left_shift(5, 1), 10);  // 101 << 1 = 1010
    assert_eq!(left_shift(3, 2), 12);  // 011 << 2 = 1100
    assert_eq!(left_shift(1, 4), 16);  // 1 << 4 = 10000
    assert_eq!(left_shift(7, 3), 56);
}

#[test]
fn test_right_shift() {
    assert_eq!(right_shift(10, 1), 5); // 1010 >> 1 = 101
    assert_eq!(right_shift(12, 2), 3); // 1100 >> 2 = 011
    assert_eq!(right_shift(16, 4), 1); // 10000 >> 4 = 1
    assert_eq!(right_shift(100, 2), 25);
}

#[test]
fn test_bitwise_not() {
    assert_eq!(bitwise_not(5), !5);
    assert_eq!(bitwise_not(0), -1);
    assert_eq!(bitwise_not(-1), 0);
    assert_eq!(bitwise_not(255), !255);
}

// ============================================================
// RANGE OPERATOR TESTS (RANGEOR) - RUST-SPECIFIC
// ============================================================

#[test]
fn test_exclusive_range_sum() {
    assert_eq!(exclusive_range_sum(0, 5), 10);  // 0+1+2+3+4
    assert_eq!(exclusive_range_sum(1, 4), 6);   // 1+2+3
    assert_eq!(exclusive_range_sum(5, 5), 0);   // Empty range
}

#[test]
fn test_inclusive_range_sum() {
    assert_eq!(inclusive_range_sum(0, 5), 15);  // 0+1+2+3+4+5
    assert_eq!(inclusive_range_sum(1, 4), 10);  // 1+2+3+4
    assert_eq!(inclusive_range_sum(5, 5), 5);   // Single element
}

#[test]
fn test_exclusive_range_collect() {
    assert_eq!(exclusive_range_collect(0, 5), vec![0, 1, 2, 3, 4]);
    assert_eq!(exclusive_range_collect(1, 4), vec![1, 2, 3]);
    assert_eq!(exclusive_range_collect(5, 5), vec![] as Vec<i32>);
}

#[test]
fn test_inclusive_range_collect() {
    assert_eq!(inclusive_range_collect(0, 5), vec![0, 1, 2, 3, 4, 5]);
    assert_eq!(inclusive_range_collect(1, 4), vec![1, 2, 3, 4]);
    assert_eq!(inclusive_range_collect(5, 5), vec![5]);
}

// ============================================================
// PATTERN MATCHING TESTS (PMR) - RUST-SPECIFIC
// ============================================================

#[test]
fn test_unwrap_option() {
    assert_eq!(unwrap_option(Some(42)), 42);
    assert_eq!(unwrap_option(None), 0);
    assert_eq!(unwrap_option(Some(-5)), -5);
    assert_eq!(unwrap_option(Some(0)), 0);
}

#[test]
fn test_unwrap_result() {
    assert_eq!(unwrap_result(Ok(42)), 42);
    assert_eq!(unwrap_result(Err("error".to_string())), 0);
    assert_eq!(unwrap_result(Ok(-5)), -5);
    assert_eq!(unwrap_result(Ok(0)), 0);
}

#[test]
fn test_is_some() {
    assert!(is_some(Some(42)));
    assert!(!is_some(None));
    assert!(is_some(Some(0)));
}

#[test]
fn test_is_ok() {
    assert!(is_ok(Ok(42)));
    assert!(!is_ok(Err("error".to_string())));
    assert!(is_ok(Ok(0)));
}

// ============================================================
// METHOD CHAINING TESTS (MCR) - RUST-SPECIFIC
// ============================================================

#[test]
fn test_map_filter_example() {
    let input = vec![1, 2, 3, 4, 5];
    let result = map_filter_example(input);
    assert_eq!(result, vec![6, 8, 10]);  // [2,4,6,8,10] filtered to >5
}

#[test]
fn test_filter_map_example() {
    let input = vec![-2, -1, 0, 1, 2, 3];
    let result = filter_map_example(input);
    assert_eq!(result, vec![2, 4, 6]);  // Positive values doubled
}

#[test]
fn test_unwrap_or_example() {
    assert_eq!(unwrap_or_example(Some(42)), 42);
    assert_eq!(unwrap_or_example(None), 0);
}

#[test]
fn test_unwrap_or_default_example() {
    assert_eq!(unwrap_or_default_example(Some(42)), 42);
    assert_eq!(unwrap_or_default_example(None), 0);
}

// ============================================================
// BORROW/REFERENCE TESTS (LBM) - RUST-SPECIFIC
// ============================================================

#[test]
fn test_borrow_immutable() {
    let value = 42;
    assert_eq!(borrow_immutable(&value), 42);

    let value2 = -5;
    assert_eq!(borrow_immutable(&value2), -5);
}

#[test]
fn test_borrow_mutable() {
    let mut value = 42;
    borrow_mutable(&mut value);
    assert_eq!(value, 43);

    borrow_mutable(&mut value);
    assert_eq!(value, 44);
}

#[test]
fn test_return_reference() {
    let value = 42;
    let borrowed = return_reference(&value);
    assert_eq!(*borrowed, 42);
}

#[test]
fn test_return_mutable_reference() {
    let mut value = 42;
    let borrowed = return_mutable_reference(&mut value);
    assert_eq!(*borrowed, 43);

    *borrowed += 1;
    assert_eq!(value, 44);
}
