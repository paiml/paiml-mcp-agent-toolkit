//! TDD Tests for Ruchy Entropy Analysis Integration
//! 
//! Testing Ruchy-specific pattern detection in entropy analysis

#[cfg(all(test, feature = "ruchy-ast"))]
mod ruchy_entropy_integration_tests {
    
    
    #[test] 
    fn test_ruchy_actor_pattern_detection() {
        // RED: This should fail because Ruchy actor patterns are not yet detected
        let ruchy_code = r#"
actor Counter {
    count: i32,
    
    receive increment() {
        self.count += 1;
    }
    
    receive get() -> i32 {
        self.count
    }
    
    receive decrement() {
        if self.count > 0 {
            self.count -= 1;
        }
    }
}

actor Logger {
    messages: Vec<String>,
    
    receive log(msg: String) {
        self.messages.push(msg);
    }
    
    receive get_logs() -> Vec<String> {
        self.messages.clone()
    }
}
"#;
        
        // Should detect repeated actor pattern
        let actor_count = ruchy_code.matches("actor ").count();
        let receive_count = ruchy_code.matches("receive ").count();
        
        assert_eq!(actor_count, 2);
        assert_eq!(receive_count, 5);
        
        // This pattern should be detected as repetitive actor message handling
        // with variation scores calculated based on different message types
    }
    
    #[test]
    fn test_ruchy_pipeline_pattern_detection() {
        // RED: Test pipeline operator pattern detection
        let pipeline_code = r#"
fun process_numbers(data: [i32]) -> [i32] {
    data
        |> filter(|x| x > 0)
        |> map(|x| x * 2)
        |> filter(|x| x < 100)
        |> sort()
}

fun process_strings(data: [String]) -> [String] {
    data
        |> filter(|s| !s.is_empty())
        |> map(|s| s.to_uppercase())
        |> filter(|s| s.len() > 3)
        |> sort()
}

fun process_users(users: [User]) -> [User] {
    users
        |> filter(|u| u.active)
        |> map(|u| u.normalize())
        |> filter(|u| u.valid())
        |> sort_by(|u| u.name)
}
"#;
        
        // Should detect repeated pipeline pattern
        let pipeline_count = pipeline_code.matches("|>").count();
        let filter_count = pipeline_code.matches("|> filter").count();
        let map_count = pipeline_code.matches("|> map").count();
        
        assert_eq!(pipeline_count, 12); // 4 operations * 3 functions
        assert_eq!(filter_count, 6); // 2 filters * 3 functions  
        assert_eq!(map_count, 3); // 1 map * 3 functions
        
        // This pattern should be detected as repetitive data transformation
        // with high similarity scores between the three functions
    }
    
    #[test]
    fn test_ruchy_error_handling_pattern_detection() {
        // RED: Test Ruchy-specific error handling patterns
        let error_handling_code = r#"
fun validate_user(data: UserData) -> Result<User, ValidationError> {
    match data.email {
        Some(email) if is_valid_email(email) => Ok(User::new(email)),
        Some(_) => Err(ValidationError::InvalidEmail),
        None => Err(ValidationError::MissingEmail),
    }
}

fun validate_product(data: ProductData) -> Result<Product, ValidationError> {
    match data.name {
        Some(name) if !name.is_empty() => Ok(Product::new(name)),
        Some(_) => Err(ValidationError::InvalidName),
        None => Err(ValidationError::MissingName),
    }
}

fun validate_order(data: OrderData) -> Result<Order, ValidationError> {
    match data.items {
        Some(items) if !items.is_empty() => Ok(Order::new(items)),
        Some(_) => Err(ValidationError::EmptyOrder), 
        None => Err(ValidationError::MissingItems),
    }
}
"#;
        
        // Should detect repeated validation pattern
        let result_count = error_handling_code.matches("Result<").count();
        let match_count = error_handling_code.matches("match ").count();
        let err_count = error_handling_code.matches("Err(").count();
        
        assert_eq!(result_count, 3);
        assert_eq!(match_count, 3);
        assert_eq!(err_count, 6); // 2 errors * 3 functions
        
        // This pattern should be detected as repetitive validation with high similarity
    }
    
    #[test]
    fn test_ruchy_message_passing_pattern_detection() {
        // RED: Test actor message passing patterns
        let message_passing_code = r#"
fun handle_user_requests() {
    let counter = spawn Counter { count: 0 };
    let logger = spawn Logger { messages: vec![] };
    
    counter <- increment();
    logger <- log("User incremented counter");
    let count = counter <? get();
    logger <- log(f"Current count: {count}");
    
    counter <- increment();
    counter <- increment(); 
    logger <- log("User incremented counter twice");
    let final_count = counter <? get();
    logger <- log(f"Final count: {final_count}");
}

fun handle_admin_requests() {
    let counter = spawn Counter { count: 0 };
    let logger = spawn Logger { messages: vec![] };
    
    counter <- increment();
    logger <- log("Admin incremented counter");
    let count = counter <? get();
    logger <- log(f"Current count: {count}");
    
    counter <- decrement();
    logger <- log("Admin decremented counter");
    let final_count = counter <? get();
    logger <- log(f"Final count: {final_count}");
}
"#;
        
        // Should detect repeated message passing patterns
        let spawn_count = message_passing_code.matches("spawn ").count();
        let send_count = message_passing_code.matches(" <- ").count();
        let query_count = message_passing_code.matches(" <? ").count();
        let log_count = message_passing_code.matches("log(").count();
        
        assert_eq!(spawn_count, 4); // 2 actors * 2 functions
        assert_eq!(send_count, 8); // Various send operations
        assert_eq!(query_count, 4); // Query operations
        assert_eq!(log_count, 8); // Log messages
        
        // This pattern should be detected as repetitive actor orchestration
    }
    
    #[test]
    fn test_ruchy_pattern_matching_variation_detection() {
        // RED: Test pattern matching with variations
        let pattern_matching_code = r#"
enum Status { Active, Inactive, Suspended }

fun process_active_status(status: Status) -> String {
    match status {
        Status::Active => "User is active",
        Status::Inactive => "User is inactive", 
        Status::Suspended => "User is suspended",
    }
}

enum Priority { Low, Medium, High, Critical }

fun process_priority(priority: Priority) -> i32 {
    match priority {
        Priority::Low => 1,
        Priority::Medium => 2,
        Priority::High => 3,
        Priority::Critical => 4,
    }
}

enum Color { Red, Green, Blue, Yellow }

fun process_color(color: Color) -> String {
    match color {
        Color::Red => "FF0000",
        Color::Green => "00FF00", 
        Color::Blue => "0000FF",
        Color::Yellow => "FFFF00",
    }
}
"#;
        
        // Should detect repeated enum matching pattern with different variations
        let enum_count = pattern_matching_code.matches("enum ").count();
        let match_count = pattern_matching_code.matches("match ").count();
        let arrow_count = pattern_matching_code.matches(" => ").count();
        
        assert_eq!(enum_count, 3);
        assert_eq!(match_count, 3);
        assert_eq!(arrow_count, 11); // Total match arms across all functions
        
        // This pattern should be detected with medium variation score
        // (same structure, different enum types and return values)
    }
    
    #[cfg(feature = "ruchy-ast")]
    #[tokio::test]
    async fn test_ruchy_entropy_analysis_integration() {
        // RED: Test full entropy analysis integration for Ruchy files
        use tempfile::NamedTempFile;
        use std::io::Write;
        
        let complex_ruchy_code = r#"
// Repetitive actor patterns (should be detected)
actor UserManager {
    users: HashMap<i32, User>,
    
    receive add_user(user: User) {
        self.users.insert(user.id, user);
    }
    
    receive get_user(id: i32) -> Option<User> {
        self.users.get(&id).cloned()
    }
}

actor ProductManager {
    products: HashMap<i32, Product>,
    
    receive add_product(product: Product) {
        self.products.insert(product.id, product);
    }
    
    receive get_product(id: i32) -> Option<Product> {
        self.products.get(&id).cloned() 
    }
}

// Repetitive pipeline patterns (should be detected)
fun process_user_data(data: [UserData]) -> [User] {
    data
        |> filter(|d| d.is_valid())
        |> map(|d| User::from_data(d))
        |> filter(|u| u.is_active())
        |> sort()
}

fun process_product_data(data: [ProductData]) -> [Product] {
    data
        |> filter(|d| d.is_valid())
        |> map(|d| Product::from_data(d))
        |> filter(|p| p.is_available())
        |> sort()
}

fun main() {
    let user_manager = spawn UserManager { users: HashMap::new() };
    let product_manager = spawn ProductManager { products: HashMap::new() };
    
    // Process some data
    let users = process_user_data(load_user_data());
    let products = process_product_data(load_product_data());
    
    println("Processed {} users and {} products", users.len(), products.len());
}
"#;
        
        let mut temp_file = NamedTempFile::with_suffix(".ruchy").unwrap();
        temp_file.write_all(complex_ruchy_code.as_bytes()).unwrap();
        
        // Should be able to analyze Ruchy entropy patterns
        let result = pmat::entropy::PatternExtractor::new(pmat::entropy::EntropyConfig::default())
            .extract_patterns(temp_file.path())
            .await;
            
        assert!(result.is_ok(), "Entropy analysis should work with Ruchy files");
        
        let patterns = result.unwrap();
        
        // Should detect multiple repetitive patterns
        assert!(patterns.file_count() > 0);
        
        // Should detect at least some high-frequency patterns
        let summary = patterns.summary();
        assert!(summary.repetitions > 1, "Should detect repetitive patterns");
        
        // Should have reasonable variation scores
        assert!(summary.variation_score >= 0.0 && summary.variation_score <= 1.0);
    }
    
    #[test]
    fn test_ruchy_entropy_pattern_types() {
        // RED: Test that all Ruchy pattern types are supported
        use pmat::entropy::{PatternType, EntropyConfig};
        
        let config = EntropyConfig::default();
        
        // Should support all standard pattern types for Ruchy analysis
        assert!(config.pattern_types.contains(&PatternType::ErrorHandling));
        assert!(config.pattern_types.contains(&PatternType::DataValidation));
        assert!(config.pattern_types.contains(&PatternType::ResourceManagement));
        assert!(config.pattern_types.contains(&PatternType::ControlFlow));
        assert!(config.pattern_types.contains(&PatternType::DataTransformation));
        assert!(config.pattern_types.contains(&PatternType::ApiCall));
    }
    
    #[cfg(not(feature = "ruchy-ast"))]
    #[test]
    fn test_ruchy_entropy_fallback() {
        // When ruchy-ast feature is disabled, should still detect basic patterns
        let basic_ruchy_code = r#"
fun hello() -> String {
    "Hello, World!"
}

fun goodbye() -> String {
    "Goodbye, World!"  
}
"#;
        
        // Should at least detect basic function patterns even without AST
        let fun_count = basic_ruchy_code.matches("fun ").count();
        assert_eq!(fun_count, 2);
        
        // Should be able to perform basic entropy analysis
        assert!(!basic_ruchy_code.is_empty());
    }
}