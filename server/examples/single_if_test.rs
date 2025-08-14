//! Single if test

/// Should have exactly cyclomatic=2, cognitive=1
pub fn single_if_only(x: i32) -> i32 {
    if x > 0 {
        // +1 cyclomatic, +1 cognitive
        x
    } else {
        0
    }
}

fn main() {
    println!("Single if test");
}
