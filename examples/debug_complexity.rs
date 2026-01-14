//! Debug complexity - super simple cases

/// Base case - should be cyclomatic=1, cognitive=0
pub fn debug_base() -> i32 {
    42
}

/// Single if, no else - should be cyclomatic=2, cognitive=1
pub fn debug_if_no_else(x: i32) -> i32 {
    if x > 0 {
        x
    } else {
        0
    }
}

/// Just condition - should be cyclomatic=1, cognitive=0
pub fn debug_condition(x: i32) -> bool {
    x > 0
}

fn main() {
    println!("Debug complexity examples");
}
