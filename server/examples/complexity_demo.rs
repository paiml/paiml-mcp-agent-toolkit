//! Complexity Analysis Demo
//! 
//! This example demonstrates various complexity patterns that `pmat analyze complexity` 
//! can detect and analyze. Run with:
//! 
//! ```bash
//! cargo run --example complexity_demo
//! pmat analyze complexity --include "server/examples/complexity_demo.rs"
//! ```
//! 
//! Expected complexity analysis results:
//! - simple_function: Low complexity (1)
//! - http_request_handler: Moderate complexity (~8-10)
//! - order_processing_pipeline: High complexity (~15-20)
//! - user_authentication: Very high complexity (~25+)

use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct User {
    pub id: u32,
    pub email: String,
    pub role: UserRole,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub enum UserRole {
    Guest,
    Customer,
    Premium,
    Admin,
}

#[derive(Debug)]
pub struct Order {
    pub id: u32,
    pub user_id: u32,
    pub items: Vec<OrderItem>,
    pub status: OrderStatus,
    pub total_amount: f64,
}

#[derive(Debug)]
pub struct OrderItem {
    pub product_id: u32,
    pub quantity: u32,
    pub price: f64,
}

#[derive(Debug)]
pub enum OrderStatus {
    Pending,
    Processing,
    Shipped,
    Delivered,
    Cancelled,
    Refunded,
}

/// Simple function with minimal complexity (Cyclomatic: 1, Cognitive: 0)
/// This demonstrates the baseline for complexity analysis
pub fn simple_function(x: i32) -> i32 {
    x * 2 + 1
}

/// HTTP request handler with moderate complexity (Cyclomatic: ~8, Cognitive: ~10)
/// Demonstrates realistic branching patterns in web applications
pub async fn http_request_handler(
    method: &str,
    path: &str,
    headers: HashMap<String, String>,
    body: Option<String>,
) -> Result<String, Box<dyn std::error::Error>> {
    // Validate request method
    match method {
        "GET" => handle_get_request(path, &headers).await,
        "POST" => {
            if let Some(request_body) = body {
                handle_post_request(path, &headers, &request_body).await
            } else {
                Err("POST request requires body".into())
            }
        }
        "PUT" => {
            if let Some(request_body) = body {
                if headers.contains_key("Content-Type") {
                    handle_put_request(path, &headers, &request_body).await
                } else {
                    Err("PUT request requires Content-Type header".into())
                }
            } else {
                Err("PUT request requires body".into())
            }
        }
        "DELETE" => {
            if headers.contains_key("Authorization") {
                handle_delete_request(path, &headers).await
            } else {
                Err("DELETE request requires authorization".into())
            }
        }
        _ => Err(format!("Unsupported HTTP method: {}", method).into()),
    }
}

async fn handle_get_request(
    path: &str,
    _headers: &HashMap<String, String>,
) -> Result<String, Box<dyn std::error::Error>> {
    if path.starts_with("/api/") {
        if path.contains("/users/") {
            Ok("User data".to_string())
        } else if path.contains("/orders/") {
            Ok("Order data".to_string())
        } else {
            Ok("Generic API data".to_string())
        }
    } else if path == "/health" {
        Ok("OK".to_string())
    } else {
        Err("Not found".into())
    }
}

async fn handle_post_request(
    path: &str,
    _headers: &HashMap<String, String>,
    _body: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    if path == "/api/users" {
        Ok("User created".to_string())
    } else if path == "/api/orders" {
        Ok("Order created".to_string())
    } else {
        Err("Invalid POST endpoint".into())
    }
}

async fn handle_put_request(
    _path: &str,
    _headers: &HashMap<String, String>,
    _body: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    Ok("Resource updated".to_string())
}

async fn handle_delete_request(
    _path: &str,
    _headers: &HashMap<String, String>,
) -> Result<String, Box<dyn std::error::Error>> {
    Ok("Resource deleted".to_string())
}

/// Order processing pipeline with high complexity (Cyclomatic: ~15, Cognitive: ~20)
/// Demonstrates complex business logic with multiple validation steps
pub fn order_processing_pipeline(
    order: Order,
    user: User,
    inventory: &mut HashMap<u32, u32>,
    payment_info: Option<String>,
) -> Result<Order, String> {
    // Validate user eligibility
    if !user.is_active {
        return Err("User account is not active".to_string());
    }

    // Check user role permissions for order size
    let order_limit = match user.role {
        UserRole::Guest => 100.0,
        UserRole::Customer => 1000.0,
        UserRole::Premium => 5000.0,
        UserRole::Admin => f64::MAX,
    };

    if order.total_amount > order_limit {
        return Err(format!(
            "Order amount ${:.2} exceeds limit ${:.2} for user role {:?}",
            order.total_amount, order_limit, user.role
        ));
    }

    // Validate inventory availability
    for item in &order.items {
        if let Some(available_quantity) = inventory.get(&item.product_id) {
            if *available_quantity < item.quantity {
                return Err(format!(
                    "Insufficient inventory for product {}: requested {}, available {}",
                    item.product_id, item.quantity, available_quantity
                ));
            }
        } else {
            return Err(format!("Product {} not found in inventory", item.product_id));
        }
    }

    // Process payment based on order amount and user role
    let payment_required = match user.role {
        UserRole::Admin => false, // Admin orders don't require payment
        _ => order.total_amount > 0.0,
    };

    if payment_required {
        match payment_info {
            Some(ref payment) if !payment.is_empty() => {
                // Validate payment information
                if payment.len() < 10 {
                    return Err("Invalid payment information".to_string());
                }
                
                // Different validation for different payment methods
                if payment.starts_with("card_") {
                    if payment.len() != 20 {
                        return Err("Invalid card token format".to_string());
                    }
                } else if payment.starts_with("bank_") {
                    if payment.len() != 25 {
                        return Err("Invalid bank transfer format".to_string());
                    }
                } else if payment.starts_with("crypto_") {
                    if payment.len() < 30 {
                        return Err("Invalid crypto address format".to_string());
                    }
                } else {
                    return Err("Unsupported payment method".to_string());
                }
            }
            _ => return Err("Payment information required".to_string()),
        }
    }

    // Update inventory
    for item in &order.items {
        if let Some(available_quantity) = inventory.get_mut(&item.product_id) {
            *available_quantity -= item.quantity;
        }
    }

    // Determine shipping method based on order value and user role
    let shipping_time = match user.role {
        UserRole::Premium | UserRole::Admin => {
            if order.total_amount > 500.0 {
                Duration::from_secs(24 * 3600) // 1 day
            } else {
                Duration::from_secs(2 * 24 * 3600) // 2 days
            }
        }
        UserRole::Customer => {
            if order.total_amount > 100.0 {
                Duration::from_secs(3 * 24 * 3600) // 3 days
            } else {
                Duration::from_secs(5 * 24 * 3600) // 5 days
            }
        }
        UserRole::Guest => Duration::from_secs(7 * 24 * 3600), // 7 days
    };

    println!("Order processed successfully. Estimated shipping time: {:?}", shipping_time);

    // Return updated order with processing status
    let mut processed_order = order;
    processed_order.status = OrderStatus::Processing;
    Ok(processed_order)
}

/// User authentication with very high complexity (Cyclomatic: ~25+, Cognitive: ~30+)
/// Demonstrates complex authentication logic with multiple factors and edge cases
pub fn user_authentication(
    email: &str,
    password: &str,
    two_factor_code: Option<&str>,
    ip_address: &str,
    user_agent: &str,
    remember_me: bool,
) -> Result<(User, String), String> {
    // Input validation
    if email.is_empty() || password.is_empty() {
        return Err("Email and password are required".to_string());
    }

    // Email format validation
    if !email.contains('@') || !email.contains('.') {
        return Err("Invalid email format".to_string());
    }

    let email_parts: Vec<&str> = email.split('@').collect();
    if email_parts.len() != 2 || email_parts[0].is_empty() || email_parts[1].is_empty() {
        return Err("Invalid email format".to_string());
    }

    // Password strength validation
    if password.len() < 8 {
        return Err("Password must be at least 8 characters".to_string());
    }

    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    if !has_uppercase || !has_lowercase || !has_digit || !has_special {
        return Err("Password must contain uppercase, lowercase, digit, and special character".to_string());
    }

    // Simulate user lookup (in real app, this would be a database query)
    let user = match email {
        "admin@example.com" => User {
            id: 1,
            email: email.to_string(),
            role: UserRole::Admin,
            is_active: true,
        },
        "premium@example.com" => User {
            id: 2,
            email: email.to_string(),
            role: UserRole::Premium,
            is_active: true,
        },
        "customer@example.com" => User {
            id: 3,
            email: email.to_string(),
            role: UserRole::Customer,
            is_active: true,
        },
        "inactive@example.com" => User {
            id: 4,
            email: email.to_string(),
            role: UserRole::Customer,
            is_active: false,
        },
        _ => return Err("User not found".to_string()),
    };

    // Check if user account is active
    if !user.is_active {
        return Err("User account is deactivated".to_string());
    }

    // Simulate password verification (in real app, use proper hashing)
    let password_valid = match email {
        "admin@example.com" => password == "AdminPass123!",
        "premium@example.com" => password == "PremiumPass456@",
        "customer@example.com" => password == "CustomerPass789#",
        "inactive@example.com" => password == "InactivePass000$",
        _ => false,
    };

    if !password_valid {
        return Err("Invalid password".to_string());
    }

    // Check for suspicious activity based on IP and User-Agent
    let is_suspicious_ip = ip_address.starts_with("192.168.") 
        || ip_address.starts_with("10.")
        || ip_address == "127.0.0.1";

    let is_suspicious_agent = user_agent.is_empty() 
        || user_agent.len() < 10
        || user_agent.to_lowercase().contains("bot")
        || user_agent.to_lowercase().contains("crawler");

    // Two-factor authentication requirements
    let requires_2fa = match user.role {
        UserRole::Admin => true,
        UserRole::Premium => is_suspicious_ip || is_suspicious_agent,
        UserRole::Customer => is_suspicious_ip && is_suspicious_agent,
        UserRole::Guest => false,
    };

    if requires_2fa {
        match two_factor_code {
            Some(code) if !code.is_empty() => {
                // Validate 2FA code format
                if code.len() != 6 || !code.chars().all(|c| c.is_ascii_digit()) {
                    return Err("2FA code must be 6 digits".to_string());
                }

                // Simulate 2FA verification (in real app, verify against stored secret)
                let expected_code = match user.id {
                    1 => "123456", // admin
                    2 => "654321", // premium
                    3 => "111222", // customer
                    _ => "000000",
                };

                if code != expected_code {
                    return Err("Invalid 2FA code".to_string());
                }
            }
            _ => return Err("Two-factor authentication required".to_string()),
        }
    }

    // Generate session token based on user role and remember_me preference
    let token_duration = if remember_me {
        match user.role {
            UserRole::Admin => "7d",      // 7 days for admin
            UserRole::Premium => "30d",   // 30 days for premium
            UserRole::Customer => "14d",  // 14 days for customer
            UserRole::Guest => "1d",      // 1 day for guest
        }
    } else {
        match user.role {
            UserRole::Admin => "4h",      // 4 hours for admin
            UserRole::Premium => "8h",    // 8 hours for premium
            UserRole::Customer => "6h",   // 6 hours for customer
            UserRole::Guest => "2h",      // 2 hours for guest
        }
    };

    // Generate secure token (simplified for demo)
    let session_token = format!("session_{}_{}_{}", user.id, token_duration, 
        ip_address.replace('.', ""));

    // Additional logging for suspicious activity
    if is_suspicious_ip || is_suspicious_agent {
        println!("Warning: Suspicious login detected for user {} from IP {} with agent '{}'", 
            user.email, ip_address, user_agent);
    }

    // Final validation based on time-based restrictions
    let current_hour = 14; // Simulated current hour (2 PM)
    let login_allowed = match user.role {
        UserRole::Admin => true, // Admin can login anytime
        UserRole::Premium => (6..=23).contains(&current_hour), // 6 AM to 11 PM
        UserRole::Customer => (8..=22).contains(&current_hour), // 8 AM to 10 PM
        UserRole::Guest => (9..=21).contains(&current_hour), // 9 AM to 9 PM
    };

    if !login_allowed {
        return Err(format!("Login not allowed at this time for {:?} users", user.role));
    }

    Ok((user, session_token))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Complexity Analysis Demo");
    println!("=========================");
    println!();
    
    println!("Testing simple_function...");
    let result = simple_function(5);
    println!("simple_function(5) = {}", result);
    println!();

    println!("Testing http_request_handler...");
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    
    match http_request_handler("GET", "/api/users/123", headers.clone(), None).await {
        Ok(response) => println!("GET request successful: {}", response),
        Err(e) => println!("GET request failed: {}", e),
    }
    println!();

    println!("Testing order_processing_pipeline...");
    let user = User {
        id: 1,
        email: "customer@example.com".to_string(),
        role: UserRole::Customer,
        is_active: true,
    };

    let order = Order {
        id: 1001,
        user_id: user.id,
        items: vec![
            OrderItem {
                product_id: 1,
                quantity: 2,
                price: 25.99,
            },
            OrderItem {
                product_id: 2,
                quantity: 1,
                price: 99.99,
            },
        ],
        status: OrderStatus::Pending,
        total_amount: 151.97,
    };

    let mut inventory = HashMap::new();
    inventory.insert(1, 10);
    inventory.insert(2, 5);

    match order_processing_pipeline(order, user, &mut inventory, Some("card_1234567890123456".to_string())) {
        Ok(processed_order) => println!("Order processed successfully: {:?}", processed_order.status),
        Err(e) => println!("Order processing failed: {}", e),
    }
    println!();

    println!("Testing user_authentication...");
    match user_authentication(
        "customer@example.com",
        "CustomerPass789#",
        Some("111222"),
        "192.168.1.100",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        true,
    ) {
        Ok((user, token)) => println!("Authentication successful for user: {} (token: {})", user.email, token),
        Err(e) => println!("Authentication failed: {}", e),
    }

    println!();
    println!("🎯 Run complexity analysis with:");
    println!("   pmat analyze complexity --include \"server/examples/complexity_demo.rs\"");
    println!();
    println!("Expected complexity levels:");
    println!("- simple_function: Low (Cyclomatic: 1)");
    println!("- http_request_handler: Moderate (Cyclomatic: ~8)");
    println!("- order_processing_pipeline: High (Cyclomatic: ~15)");
    println!("- user_authentication: Very High (Cyclomatic: ~25+)");

    Ok(())
}