// Example file for testing complexity analysis
// This demonstrates various complexity levels

/// Simple function with complexity 1
fn simple() -> i32 {
    42
}

/// Function with moderate complexity (nested if statements)
fn moderate_complexity(x: i32) -> i32 {
    if x > 10 {
        if x > 20 {
            x * 2
        } else {
            x + 5
        }
    } else {
        x - 3
    }
}

/// Function with higher complexity (multiple branches and loops)
fn high_complexity(numbers: &[i32]) -> i32 {
    let mut sum = 0;
    
    for num in numbers {
        if *num > 0 {
            if *num % 2 == 0 {
                sum += *num * 2;
            } else {
                if *num > 10 {
                    sum += *num + 1;
                } else {
                    sum += *num;
                }
            }
        } else if *num < 0 {
            match *num {
                -1 => sum -= 1,
                -2 => sum -= 2,
                _ => sum -= 10,
            }
        }
    }
    
    sum
}

/// Recursive function
fn factorial(n: u32) -> u32 {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}