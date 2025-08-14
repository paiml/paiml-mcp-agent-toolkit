fn used_function() -> i32 {
    42
}

fn unused_function() -> i32 {
    100
}

fn main() {
    println!("{}", used_function());
}