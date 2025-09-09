// Test file to verify cognitive complexity calculation fix

// Test 1: Simple if-else chain (should be 4)
fn simple_if_else_chain(x: i32) -> &'static str {
    if x == 1 {
        "one"
    } else if x == 2 {
        "two"
    } else if x == 3 {
        "three"
    } else if x == 4 {
        "four"
    } else {
        "other"
    }
}

// Test 2: Nested if (should be 1 + (1+1) = 3)
fn nested_if(x: i32, y: i32) -> i32 {
    if x > 0 {                  // +1 (no nesting)
        if y > 0 {              // +1 + 1 (nesting level 1)
            x + y
        } else {
            x
        }
    } else {
        0
    }
}

// Test 3: Loop with nested if (should be 1 + (1+1) = 3)
fn loop_with_nested_if(items: &[i32]) -> i32 {
    let mut sum = 0;
    for item in items {         // +1 (no nesting)
        if *item > 0 {          // +1 + 1 (nesting level 1)
            sum += item;
        }
    }
    sum
}

// Test 4: Match with guards (should be 1 + 1 + 1 = 3)
fn match_with_guards(x: Option<i32>) -> i32 {
    match x {                    // +1
        Some(n) if n > 0 => n,  // +1 for guard
        Some(n) if n < 0 => -n, // +1 for guard
        _ => 0,
    }
}

// Test 5: Binary operators (should be 2)
fn binary_operators(x: bool, y: bool) -> bool {
    x && y || !x  // +1 for &&, +1 for ||
}

// Test 6: Complex nested structure
fn complex_nested(x: i32) -> i32 {
    let mut result = 0;
    
    if x > 0 {                          // +1 (no nesting)
        for i in 0..x {                 // +1 + 1 (nesting level 1)
            if i % 2 == 0 {             // +1 + 2 (nesting level 2)
                if i > 10 {             // +1 + 3 (nesting level 3)
                    result += i * 2;
                } else {
                    result += i;
                }
            }
        }
    }
    
    result
    // Expected: 1 + 2 + 3 + 4 = 10
}