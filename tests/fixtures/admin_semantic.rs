// Test fixture with semantically similar code (Type-4 clones)

// Imperative style
fn calculate_total_imperative(items: &[Item]) -> f64 {
    let mut total = 0.0;
    for item in items {
        total += item.price * item.quantity as f64;
    }
    total
}

// Functional style - semantically equivalent
fn calculate_total_functional(items: &[Item]) -> f64 {
    items.iter()
        .map(|item| item.price * item.quantity as f64)
        .sum()
}

// Different implementation, same semantics
fn find_max_imperative(numbers: &[i32]) -> Option<i32> {
    if numbers.is_empty() {
        return None;
    }
    let mut max = numbers[0];
    for &num in &numbers[1..] {
        if num > max {
            max = num;
        }
    }
    Some(max)
}

// Functional equivalent
fn find_max_functional(numbers: &[i32]) -> Option<i32> {
    numbers.iter().max().copied()
}

// Pattern matching vs if-else
fn status_to_string_if(status: Status) -> &'static str {
    if status == Status::Active {
        "active"
    } else if status == Status::Pending {
        "pending"
    } else if status == Status::Cancelled {
        "cancelled"
    } else {
        "unknown"
    }
}

fn status_to_string_match(status: Status) -> &'static str {
    match status {
        Status::Active => "active",
        Status::Pending => "pending",
        Status::Cancelled => "cancelled",
        _ => "unknown",
    }
}