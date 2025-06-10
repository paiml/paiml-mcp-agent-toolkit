//! Performance regression detection tests
//! Target: Detect performance degradation in critical paths

use std::time::{Duration, Instant};

#[test]
fn test_string_performance() {
    let start = Instant::now();

    let mut s = String::new();
    for i in 0..1000 {
        s.push_str(&format!("Item {}", i));
    }

    let duration = start.elapsed();
    assert!(
        duration < Duration::from_millis(100),
        "String concatenation took {:?}, expected < 100ms",
        duration
    );
}

#[test]
fn test_vector_performance() {
    let start = Instant::now();

    let mut vec = Vec::new();
    for i in 0..10000 {
        vec.push(i);
    }

    let duration = start.elapsed();
    assert!(
        duration < Duration::from_millis(10),
        "Vector operations took {:?}, expected < 10ms",
        duration
    );
}

#[test]
fn test_hashmap_performance() {
    use std::collections::HashMap;

    let start = Instant::now();

    let mut map = HashMap::new();
    for i in 0..1000 {
        map.insert(i, i * 2);
    }

    let duration = start.elapsed();
    assert!(
        duration < Duration::from_millis(50),
        "HashMap operations took {:?}, expected < 50ms",
        duration
    );
}

#[test]
fn test_sorting_performance() {
    let mut data: Vec<i32> = (0..10000).rev().collect();

    let start = Instant::now();
    data.sort();
    let duration = start.elapsed();

    assert!(
        duration < Duration::from_millis(10),
        "Sorting took {:?}, expected < 10ms",
        duration
    );

    assert_eq!(data[0], 0);
    assert_eq!(data[9999], 9999);
}

#[test]
fn test_iteration_performance() {
    let data: Vec<i32> = (0..10000).collect();

    let start = Instant::now();
    let sum: i32 = data.iter().sum();
    let duration = start.elapsed();

    assert!(
        duration < Duration::from_millis(5),
        "Iteration took {:?}, expected < 5ms",
        duration
    );

    assert_eq!(sum, (0..10000).sum());
}
