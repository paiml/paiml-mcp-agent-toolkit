//! File with exactly one function

/// Should be cyclomatic=2, cognitive=1
pub fn single_if(x: i32) -> i32 {
    if x > 0 {
        x
    } else {
        0
    }
}

fn main() {
    println!("One function test");
}
