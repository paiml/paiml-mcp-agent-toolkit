//! Isolated Complexity Test Cases
//! 
//! Each function tests exactly one complexity pattern to isolate the root cause

/// Test 1: Base case (Expected: Cyclomatic=1, Cognitive=0)
pub fn test_base() -> i32 {
    42
}

/// Test 2: Single if (Expected: Cyclomatic=2, Cognitive=1)
pub fn test_single_if(x: i32) -> i32 {
    if x > 0 {  // +1 cyclomatic, +1 cognitive
        x
    } else {
        0
    }
}

/// Test 3: Single match with 3 arms (Expected: Cyclomatic=4, Cognitive=1)
pub fn test_match_3_arms(x: i32) -> &'static str {
    match x {
        1 => "one",      // +1 cyclomatic
        2 => "two",      // +1 cyclomatic
        _ => "other",    // +1 cyclomatic
    }
}

/// Test 4: Single loop (Expected: Cyclomatic=2, Cognitive=1)
pub fn test_single_loop(numbers: &[i32]) -> i32 {
    let mut sum = 0;
    for &num in numbers {  // +1 cyclomatic, +1 cognitive
        sum += num;
    }
    sum
}

/// Test 5: Nested if (Expected: Cyclomatic=4, Cognitive=4)
pub fn test_nested_if(x: i32) -> i32 {
    if x > 0 {      // +1 cyclomatic, +1 cognitive
        if x > 10 { // +1 cyclomatic, +2 cognitive (nesting penalty)
            x * 2
        } else {    // +1 cyclomatic, +1 cognitive
            x + 1
        }
    } else {
        0
    }
}

/// Test 6: Two independent ifs (Expected: Cyclomatic=3, Cognitive=2)
pub fn test_two_ifs(x: i32, y: i32) -> i32 {
    let mut result = 0;
    
    if x > 0 {  // +1 cyclomatic, +1 cognitive
        result += x;
    }
    
    if y > 0 {  // +1 cyclomatic, +1 cognitive
        result += y;
    }
    
    result
}

/// Test 7: Logical operators (Expected: Cyclomatic=3, Cognitive=2)
pub fn test_logical_ops(x: i32, y: i32) -> bool {
    x > 0 && y > 0  // +2 cyclomatic for && operator, +1 cognitive for each condition
}

fn main() {
    println!("Running complexity isolation tests...");
    
    println!("test_base(): {}", test_base());
    println!("test_single_if(5): {}", test_single_if(5));
    println!("test_match_3_arms(1): {}", test_match_3_arms(1));
    println!("test_single_loop([1,2,3]): {}", test_single_loop(&[1, 2, 3]));
    println!("test_nested_if(15): {}", test_nested_if(15));
    println!("test_two_ifs(3, 4): {}", test_two_ifs(3, 4));
    println!("test_logical_ops(5, 3): {}", test_logical_ops(5, 3));
    
    println!("\nExpected complexity values:");
    println!("- test_base: Cyclomatic=1, Cognitive=0");
    println!("- test_single_if: Cyclomatic=2, Cognitive=1");
    println!("- test_match_3_arms: Cyclomatic=4, Cognitive=1");
    println!("- test_single_loop: Cyclomatic=2, Cognitive=1");
    println!("- test_nested_if: Cyclomatic=4, Cognitive=4");
    println!("- test_two_ifs: Cyclomatic=3, Cognitive=2");
    println!("- test_logical_ops: Cyclomatic=3, Cognitive=2");
}