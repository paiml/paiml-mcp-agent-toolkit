// calculator.rs
// Rust Calculator - Implementation for mutation testing
// All mutation operator types for comprehensive testing

// ============================================================
// ARITHMETIC OPERATORS (AOR)
// ============================================================

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn subtract(a: i32, b: i32) -> i32 {
    a - b
}

pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

pub fn divide(a: i32, b: i32) -> i32 {
    if b == 0 {
        0
    } else {
        a / b
    }
}

pub fn modulo(a: i32, b: i32) -> i32 {
    if b == 0 {
        0
    } else {
        a % b
    }
}

// ============================================================
// RELATIONAL OPERATORS (ROR)
// ============================================================

pub fn greater_than(a: i32, b: i32) -> bool {
    a > b
}

pub fn less_than(a: i32, b: i32) -> bool {
    a < b
}

pub fn greater_or_equal(a: i32, b: i32) -> bool {
    a >= b
}

pub fn less_or_equal(a: i32, b: i32) -> bool {
    a <= b
}

pub fn equal(a: i32, b: i32) -> bool {
    a == b
}

pub fn not_equal(a: i32, b: i32) -> bool {
    a != b
}

// ============================================================
// LOGICAL OPERATORS (LOR)
// ============================================================

pub fn logical_and(a: bool, b: bool) -> bool {
    a && b
}

pub fn logical_or(a: bool, b: bool) -> bool {
    a || b
}

// ============================================================
// BITWISE OPERATORS (BOR)
// ============================================================

pub fn bitwise_and(a: i32, b: i32) -> i32 {
    a & b
}

pub fn bitwise_or(a: i32, b: i32) -> i32 {
    a | b
}

pub fn bitwise_xor(a: i32, b: i32) -> i32 {
    a ^ b
}

pub fn left_shift(a: i32, shift: u32) -> i32 {
    a << shift
}

pub fn right_shift(a: i32, shift: u32) -> i32 {
    a >> shift
}

pub fn bitwise_not(a: i32) -> i32 {
    !a
}

// ============================================================
// RANGE OPERATORS (RANGEOR) - RUST-SPECIFIC
// ============================================================

pub fn exclusive_range_sum(start: i32, end: i32) -> i32 {
    (start..end).sum()
}

pub fn inclusive_range_sum(start: i32, end: i32) -> i32 {
    (start..=end).sum()
}

pub fn exclusive_range_collect(start: i32, end: i32) -> Vec<i32> {
    (start..end).collect()
}

pub fn inclusive_range_collect(start: i32, end: i32) -> Vec<i32> {
    (start..=end).collect()
}

// ============================================================
// PATTERN MATCHING (PMR) - RUST-SPECIFIC
// ============================================================

pub fn unwrap_option(value: Option<i32>) -> i32 {
    match value {
        Some(x) => x,
        None => 0,
    }
}

pub fn unwrap_result(value: Result<i32, String>) -> i32 {
    match value {
        Ok(x) => x,
        Err(_) => 0,
    }
}

pub fn is_some(value: Option<i32>) -> bool {
    matches!(value, Some(_))
}

pub fn is_ok(value: Result<i32, String>) -> bool {
    matches!(value, Ok(_))
}

// ============================================================
// METHOD CHAINING (MCR) - RUST-SPECIFIC
// ============================================================

pub fn map_filter_example(values: Vec<i32>) -> Vec<i32> {
    values
        .iter()
        .map(|x| x * 2)
        .filter(|x| *x > 5)
        .copied()
        .collect()
}

pub fn filter_map_example(values: Vec<i32>) -> Vec<i32> {
    values
        .iter()
        .filter(|x| **x > 0)
        .map(|x| x * 2)
        .copied()
        .collect()
}

pub fn unwrap_or_example(value: Option<i32>) -> i32 {
    value.unwrap_or(0)
}

pub fn unwrap_or_default_example(value: Option<i32>) -> i32 {
    value.unwrap_or_default()
}

// ============================================================
// BORROW/REFERENCE OPERATORS (LBM) - RUST-SPECIFIC
// ============================================================

pub fn borrow_immutable(value: &i32) -> i32 {
    *value
}

pub fn borrow_mutable(value: &mut i32) {
    *value += 1;
}

pub fn return_reference(value: &i32) -> &i32 {
    value
}

pub fn return_mutable_reference(value: &mut i32) -> &mut i32 {
    *value += 1;
    value
}
