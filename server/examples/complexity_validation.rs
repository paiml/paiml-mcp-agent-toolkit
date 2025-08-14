//! Complexity Validation Test
//!
//! This file contains functions with manually calculated complexity
//! to validate pmat's complexity analysis accuracy.

/// Base complexity function (should be 1)
/// Manual calculation: 1 (base complexity, no branches)
pub fn base_complexity() -> i32 {
    42
}

/// Single if statement (should be 2)
/// Manual calculation: 1 (base) + 1 (if branch) = 2
pub fn single_if(x: i32) -> i32 {
    if x > 0 {
        // +1
        x
    } else {
        0
    }
}

/// Two independent if statements (should be 3)  
/// Manual calculation: 1 (base) + 1 (first if) + 1 (second if) = 3
pub fn two_independent_ifs(x: i32, y: i32) -> i32 {
    let mut result = 0;

    if x > 0 {
        // +1
        result += x;
    }

    if y > 0 {
        // +1
        result += y;
    }

    result
}

/// Nested if statements (should be 4)
/// Manual calculation: 1 (base) + 1 (outer if) + 1 (inner if) + 1 (else) = 4
/// Cognitive complexity should be higher due to nesting
pub fn nested_ifs(x: i32) -> i32 {
    if x > 0 {
        // +1 cyclomatic, +1 cognitive
        if x > 10 {
            // +1 cyclomatic, +2 cognitive (nested)
            x * 2
        } else {
            // +1 cyclomatic, +1 cognitive
            x + 1
        }
    } else {
        0
    }
}

/// Match statement (should count each arm)
/// Manual calculation: 1 (base) + 3 (match arms) = 4
pub fn match_statement(x: i32) -> &'static str {
    match x {
        1 => "one",   // +1
        2 => "two",   // +1
        _ => "other", // +1
    }
}

/// Loop with condition (should count loop + condition)
/// Manual calculation: 1 (base) + 1 (for loop) + 1 (if condition) = 3
pub fn loop_with_condition(numbers: &[i32]) -> i32 {
    let mut sum = 0;

    for &num in numbers {
        // +1
        if num > 0 {
            // +1
            sum += num;
        }
    }

    sum
}

/// Complex function with multiple control structures
/// Manual calculation:
/// 1 (base) + 1 (if x > 0) + 1 (for loop) + 1 (if num % 2) + 1 (match) (3 arms) = 7
pub fn complex_function(x: i32, numbers: &[i32]) -> i32 {
    if x <= 0 {
        // +1
        return 0;
    }

    let mut result = x;

    for &num in numbers {
        // +1
        if num % 2 == 0 {
            // +1
            result += num;
        }

        let modifier = match num {
            // +1 for match, +1 for each arm
            0 => 0,      // +1
            1..=10 => 1, // +1
            _ => 2,      // +1
        };

        result += modifier;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_functions_work() {
        assert_eq!(base_complexity(), 42);
        assert_eq!(single_if(5), 5);
        assert_eq!(single_if(-1), 0);
        assert_eq!(two_independent_ifs(3, 4), 7);
        assert_eq!(nested_ifs(15), 30);
        assert_eq!(nested_ifs(5), 6);
        assert_eq!(match_statement(1), "one");
        assert_eq!(loop_with_condition(&[1, 2, -1, 3]), 6);
        assert_eq!(complex_function(0, &[]), 0);
        assert_eq!(complex_function(5, &[2, 3, 4]), 13);
    }
}

fn main() {
    println!("Complexity validation examples:");
    println!("base_complexity(): {}", base_complexity());
    println!("single_if(5): {}", single_if(5));
    println!("two_independent_ifs(3, 4): {}", two_independent_ifs(3, 4));
    println!("nested_ifs(15): {}", nested_ifs(15));
    println!("match_statement(2): {}", match_statement(2));
    println!(
        "loop_with_condition([1,2,3]): {}",
        loop_with_condition(&[1, 2, 3])
    );
    println!(
        "complex_function(5, [2,4,6]): {}",
        complex_function(5, &[2, 4, 6])
    );

    println!("\nRun complexity analysis with:");
    println!("pmat analyze complexity --include \"server/examples/complexity_validation.rs\"");

    println!("\nExpected complexity values:");
    println!("- base_complexity: Cyclomatic=1, Cognitive=0");
    println!("- single_if: Cyclomatic=2, Cognitive=1");
    println!("- two_independent_ifs: Cyclomatic=3, Cognitive=2");
    println!("- nested_ifs: Cyclomatic=4, Cognitive=4 (nesting penalty)");
    println!("- match_statement: Cyclomatic=4, Cognitive=1");
    println!("- loop_with_condition: Cyclomatic=3, Cognitive=2");
    println!("- complex_function: Cyclomatic=7, Cognitive=6");
}
