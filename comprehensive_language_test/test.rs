// Rust test file
fn main() {
    println!("Hello Rust");
}

fn add_numbers(a: i32, b: i32) -> i32 {
    a + b
}

struct Calculator {
    value: f64,
}

impl Calculator {
    fn new() -> Self {
        Self { value: 0.0 }
    }

    fn calculate(&mut self, op: char, val: f64) {
        match op {
            '+' => self.value += val,
            '-' => self.value -= val,
            _ => {}
        }
    }
}