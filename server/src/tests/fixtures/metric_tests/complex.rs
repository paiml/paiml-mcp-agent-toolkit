// Complex functions with high cyclomatic complexity
fn complex_function(items: &[i32]) -> i32 {
    let mut sum = 0;
    for i in 0..items.len() {
        if items[i] > 0 {
            for j in 0..items[i] {
                if j % 2 == 0 {
                    sum += j;
                } else {
                    sum -= j;
                }
            }
        } else if items[i] < -10 {
            match items[i] {
                -20..=-15 => sum *= 2,
                -14..=-11 => sum /= 2,
                _ => sum = 0,
            }
        }
    }
    sum
}

fn recursive_fibonacci(n: i32) -> i32 {
    match n {
        0 => 0,
        1 => 1,
        n if n > 1 => recursive_fibonacci(n - 1) + recursive_fibonacci(n - 2),
        _ => panic!("Negative input"),
    }
}

fn deeply_nested_conditionals(a: i32, b: i32, c: i32, d: i32) -> i32 {
    if a > 0 {
        if b > 0 {
            if c > 0 {
                if d > 0 {
                    a + b + c + d
                } else if d < -10 {
                    a + b + c - d
                } else {
                    a + b + c
                }
            } else if c < -10 {
                if d > 0 {
                    a + b - c + d
                } else {
                    a + b - c - d
                }
            } else {
                a + b
            }
        } else if b < -10 {
            if c > 0 {
                a - b + c
            } else {
                a - b - c
            }
        } else {
            a
        }
    } else if a < -10 {
        match (b, c, d) {
            (b, c, d) if b > 0 && c > 0 && d > 0 => -a + b + c + d,
            (b, c, _) if b > 0 && c > 0 => -a + b + c,
            (b, _, _) if b > 0 => -a + b,
            _ => -a,
        }
    } else {
        0
    }
}
