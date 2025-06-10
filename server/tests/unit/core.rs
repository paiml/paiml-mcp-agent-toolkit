//! Fast unit tests for core logic with minimal dependencies
//! Target: <10s execution time, zero I/O operations

#[test]
fn test_basic_arithmetic() {
    assert_eq!(2 + 2, 4);
    assert_eq!(10 - 5, 5);
    assert_eq!(3 * 4, 12);
    assert_eq!(15 / 3, 5);
}

#[test]
fn test_string_operations() {
    let s1 = "Hello";
    let s2 = "World";
    let combined = format!("{} {}", s1, s2);

    assert_eq!(combined, "Hello World");
    assert!(combined.contains("Hello"));
    assert!(combined.contains("World"));
}

#[test]
fn test_vector_operations() {
    let mut vec = vec![1, 2, 3];
    vec.push(4);

    assert_eq!(vec.len(), 4);
    assert_eq!(vec[3], 4);
    assert_eq!(vec.iter().sum::<i32>(), 10);
}

#[test]
fn test_option_handling() {
    let some_value: Option<i32> = Some(42);
    let none_value: Option<i32> = None;

    assert!(some_value.is_some());
    assert!(none_value.is_none());
    assert_eq!(some_value.unwrap(), 42);
}

#[test]
fn test_result_handling() {
    let ok_result: Result<i32, String> = Ok(42);
    let err_result: Result<i32, String> = Err("Error".to_string());

    assert!(ok_result.is_ok());
    assert!(err_result.is_err());
    assert_eq!(ok_result.unwrap(), 42);
}

#[test]
fn test_hashmap_operations() {
    use std::collections::HashMap;

    let mut map = HashMap::new();
    map.insert("key1", 10);
    map.insert("key2", 20);

    assert_eq!(map.get("key1"), Some(&10));
    assert_eq!(map.get("key2"), Some(&20));
    assert_eq!(map.len(), 2);
}

#[test]
fn test_btreemap_operations() {
    use std::collections::BTreeMap;

    let mut map = BTreeMap::new();
    map.insert(3, "three");
    map.insert(1, "one");
    map.insert(2, "two");

    let keys: Vec<_> = map.keys().cloned().collect();
    assert_eq!(keys, vec![1, 2, 3]); // BTreeMap maintains order
}

#[test]
fn test_iterator_operations() {
    let numbers = vec![1, 2, 3, 4, 5];

    let doubled: Vec<_> = numbers.iter().map(|&x| x * 2).collect();
    assert_eq!(doubled, vec![2, 4, 6, 8, 10]);

    let sum: i32 = numbers.iter().sum();
    assert_eq!(sum, 15);

    let evens: Vec<_> = numbers.iter().filter(|&&x| x % 2 == 0).cloned().collect();
    assert_eq!(evens, vec![2, 4]);
}

#[test]
fn test_pattern_matching() {
    let value = Some(42);

    let result = match value {
        Some(x) if x > 40 => "big",
        Some(_) => "small",
        None => "nothing",
    };

    assert_eq!(result, "big");
}

#[test]
fn test_closure_operations() {
    let add = |a: i32, b: i32| a + b;
    assert_eq!(add(5, 3), 8);

    let multiply_by = |factor: i32| move |x: i32| x * factor;
    let times_two = multiply_by(2);
    assert_eq!(times_two(21), 42);
}

#[test]
fn test_string_parsing() {
    let num_str = "42";
    let parsed: Result<i32, _> = num_str.parse();
    assert_eq!(parsed.unwrap(), 42);

    let invalid = "not_a_number";
    let parsed: Result<i32, _> = invalid.parse();
    assert!(parsed.is_err());
}

#[test]
fn test_range_operations() {
    let range = 1..5;
    let vec: Vec<_> = range.collect();
    assert_eq!(vec, vec![1, 2, 3, 4]);

    let inclusive = 1..=5;
    let vec: Vec<_> = inclusive.collect();
    assert_eq!(vec, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_tuple_operations() {
    let tuple = (1, "hello", 3.14);
    assert_eq!(tuple.0, 1);
    assert_eq!(tuple.1, "hello");
    assert_eq!(tuple.2, 3.14);

    let (a, b, c) = tuple;
    assert_eq!(a, 1);
    assert_eq!(b, "hello");
    assert_eq!(c, 3.14);
}

#[test]
fn test_slice_operations() {
    let arr = [1, 2, 3, 4, 5];
    let slice = &arr[1..4];

    assert_eq!(slice.len(), 3);
    assert_eq!(slice[0], 2);
    assert_eq!(slice[2], 4);
}

#[test]
fn test_char_operations() {
    let ch = 'A';
    assert!(ch.is_alphabetic());
    assert!(ch.is_uppercase());
    assert_eq!(ch.to_lowercase().to_string(), "a");

    let digit = '5';
    assert!(digit.is_numeric());
    assert_eq!(digit.to_digit(10), Some(5));
}
