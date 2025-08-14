//! Single function test to isolate complexity calculation bug

/// Simple function that should have exactly cyclomatic=1, cognitive=0
pub fn only_function() -> i32 {
    42
}

fn main() {
    println!("Result: {}", only_function());
}
