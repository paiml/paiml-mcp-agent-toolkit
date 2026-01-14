// Simple Rust fixture for DAP integration testing
fn calculate_sum(x: i32, y: i32) -> i32 {
    let result = x + y;
    let doubled = result * 2;
    println!("Result: {}", doubled);
    doubled
}

fn main() {
    let a = 10;
    let b = 20;
    let sum = calculate_sum(a, b);
    println!("Sum: {}", sum);
}
