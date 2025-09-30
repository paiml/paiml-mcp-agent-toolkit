//! Context Analysis Demo - Demonstrates unified pmat context functionality
//!
//! This example shows how to use the unified context command with different
//! languages and formats, demonstrating the enhanced naming and multi-language
//! support implemented in the unified context handler.

use std::process::Command;
use std::fs;
use tempfile::TempDir;
use std::path::Path;

fn main() {
    println!("🚀 PMAT Context Analysis Demo");
    println!("===============================");

    println!("\n1. Testing TypeScript Context Analysis");
    test_typescript_context();

    println!("\n2. Testing JavaScript Context Analysis");
    test_javascript_context();

    println!("\n3. Testing WASM Context Analysis");
    test_wasm_context();

    println!("\n4. Testing Multi-Language Project");
    test_multi_language_context();

    println!("\n5. Testing Comprehensive Language Test Directory");
    test_comprehensive_language_test();

    println!("\n✅ Context Analysis Demo Completed!");
    println!("All examples demonstrate the unified pmat context command");
    println!("with enhanced naming and multi-language support.");
}

fn test_typescript_context() {
    println!("Creating TypeScript test project...");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let ts_content = r#"
// Enhanced TypeScript example with various function types
interface UserProfile {
    id: number;
    name: string;
    getDisplayName(): string;
}

enum UserRole {
    Admin,
    User,
    Guest
}

function calculateUserScore(profile: UserProfile, role: UserRole): number {
    return profile.id * 100;
}

const processUserData = async (userData: any) => {
    return await fetchUserDetails(userData.id);
};

class UserService {
    private users: UserProfile[] = [];

    createUser(profile: UserProfile): UserProfile {
        this.users.push(profile);
        return profile;
    }

    static validateUserRole(role: UserRole): boolean {
        return Object.values(UserRole).includes(role);
    }

    async fetchUserById(id: number): Promise<UserProfile | null> {
        return this.users.find(u => u.id === id) || null;
    }
}

const userUtils = {
    formatUserName(user: UserProfile): string {
        return `${user.name} (${user.id})`;
    },

    isValidUser: (user: UserProfile) => user.id > 0
};

async function fetchUserDetails(id: number): Promise<any> {
    return { id, name: `User ${id}` };
}
"#;

    let ts_file = temp_dir.path().join("user_service.ts");
    fs::write(&ts_file, ts_content).expect("Failed to write TypeScript file");

    println!("Running: cargo run -- context --project-path {} --format llm-optimized", temp_dir.path().display());

    let output = Command::new("cargo")
        .args(["run", "--", "context", "--project-path", temp_dir.path().to_str().unwrap(), "--format", "llm-optimized"])
        .output()
        .expect("Failed to run cargo command");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("✅ TypeScript analysis successful!");

        // Extract and display key metrics
        if let Some(functions_line) = stdout.lines().find(|line| line.contains("Functions:")) {
            println!("   {}", functions_line);
        }
        if let Some(files_line) = stdout.lines().find(|line| line.contains("Files:")) {
            println!("   {}", files_line);
        }

        // Check if we detected the expected functions
        let function_count = extract_function_count(&stdout);
        if function_count >= 7 {
            println!("   🎯 Successfully detected {} TypeScript functions!", function_count);
            println!("   Expected: calculateUserScore, processUserData, createUser, validateUserRole, fetchUserById, formatUserName, fetchUserDetails");
        } else {
            println!("   ⚠️  Only detected {} functions, expected at least 7", function_count);
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("❌ TypeScript analysis failed: {}", stderr);
    }
}

fn test_javascript_context() {
    println!("Creating JavaScript test project...");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let js_content = r#"
// Enhanced JavaScript example with various function patterns
function calculateTotalPrice(items, taxRate) {
    return items.reduce((total, item) => total + item.price, 0) * (1 + taxRate);
}

const processOrderData = async (orderData) => {
    const processedData = await validateOrder(orderData);
    return formatOrderResponse(processedData);
};

class ShoppingCart {
    constructor() {
        this.items = [];
        this.totalValue = 0;
    }

    addItem(item) {
        this.items.push(item);
        this.updateTotal();
        return this.items.length;
    }

    removeItem(itemId) {
        const index = this.items.findIndex(item => item.id === itemId);
        if (index > -1) {
            this.items.splice(index, 1);
            this.updateTotal();
        }
    }

    updateTotal() {
        this.totalValue = this.items.reduce((sum, item) => sum + item.price, 0);
    }

    static createEmptyCart() {
        return new ShoppingCart();
    }
}

const orderUtils = {
    formatOrderId: (id) => `ORD-${id.toString().padStart(6, '0')}`,

    validateOrderItems(items) {
        return items.every(item => item.price > 0 && item.quantity > 0);
    },

    calculateShipping: function(weight, destination) {
        return weight * 2.5 + (destination === 'international' ? 15 : 5);
    }
};

async function validateOrder(orderData) {
    // Validation logic
    return orderData;
}

function formatOrderResponse(processedOrder) {
    return {
        id: orderUtils.formatOrderId(processedOrder.id),
        status: 'processed',
        items: processedOrder.items
    };
}
"#;

    let js_file = temp_dir.path().join("shopping_cart.js");
    fs::write(&js_file, js_content).expect("Failed to write JavaScript file");

    println!("Running: cargo run -- context --project-path {} --format llm-optimized", temp_dir.path().display());

    let output = Command::new("cargo")
        .args(["run", "--", "context", "--project-path", temp_dir.path().to_str().unwrap(), "--format", "llm-optimized"])
        .output()
        .expect("Failed to run cargo command");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("✅ JavaScript analysis successful!");

        if let Some(functions_line) = stdout.lines().find(|line| line.contains("Functions:")) {
            println!("   {}", functions_line);
        }

        let function_count = extract_function_count(&stdout);
        if function_count >= 10 {
            println!("   🎯 Successfully detected {} JavaScript functions!", function_count);
            println!("   Expected: calculateTotalPrice, processOrderData, addItem, removeItem, updateTotal, createEmptyCart, formatOrderId, validateOrderItems, calculateShipping, validateOrder, formatOrderResponse");
        } else {
            println!("   ⚠️  Only detected {} functions, expected at least 10", function_count);
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("❌ JavaScript analysis failed: {}", stderr);
    }
}

fn test_wasm_context() {
    println!("Creating WASM test project...");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let wasm_content = r#"
(module
  ;; Import memory from host environment
  (import "env" "memory" (memory 1))

  ;; Mathematical operations
  (func $add (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.add
  )

  (func $multiply (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.mul
  )

  ;; Recursive fibonacci implementation
  (func $fibonacci (param $n i32) (result i32)
    (local $temp i32)

    local.get $n
    i32.const 2
    i32.lt_s
    if (result i32)
      local.get $n
    else
      local.get $n
      i32.const 1
      i32.sub
      call $fibonacci

      local.get $n
      i32.const 2
      i32.sub
      call $fibonacci

      i32.add
    end
  )

  ;; Factorial calculation
  (func $factorial (param $n i32) (result i32)
    (local $result i32)
    (local $counter i32)

    i32.const 1
    local.set $result
    i32.const 1
    local.set $counter

    loop $factorial_loop
      local.get $counter
      local.get $n
      i32.gt_s
      if
        br $factorial_loop
      end

      local.get $result
      local.get $counter
      i32.mul
      local.set $result

      local.get $counter
      i32.const 1
      i32.add
      local.set $counter

      br $factorial_loop
    end

    local.get $result
  )

  ;; Power function
  (func $power (param $base i32) (param $exp i32) (result i32)
    (local $result i32)
    (local $counter i32)

    i32.const 1
    local.set $result
    i32.const 0
    local.set $counter

    loop $power_loop
      local.get $counter
      local.get $exp
      i32.ge_s
      if
        br $power_loop
      end

      local.get $result
      local.get $base
      i32.mul
      local.set $result

      local.get $counter
      i32.const 1
      i32.add
      local.set $counter

      br $power_loop
    end

    local.get $result
  )

  ;; Export all functions for use
  (export "add" (func $add))
  (export "multiply" (func $multiply))
  (export "fibonacci" (func $fibonacci))
  (export "factorial" (func $factorial))
  (export "power" (func $power))
)
"#;

    let wasm_file = temp_dir.path().join("math_operations.wat");
    fs::write(&wasm_file, wasm_content).expect("Failed to write WASM file");

    println!("Running: cargo run -- context --project-path {} --format llm-optimized", temp_dir.path().display());

    let output = Command::new("cargo")
        .args(["run", "--", "context", "--project-path", temp_dir.path().to_str().unwrap(), "--format", "llm-optimized"])
        .output()
        .expect("Failed to run cargo command");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("✅ WASM analysis successful!");

        if let Some(functions_line) = stdout.lines().find(|line| line.contains("Functions:")) {
            println!("   {}", functions_line);
        }

        let function_count = extract_function_count(&stdout);
        if function_count >= 5 {
            println!("   🎯 Successfully detected {} WASM functions!", function_count);
            println!("   Expected: add, multiply, fibonacci, factorial, power");
        } else {
            println!("   ⚠️  Only detected {} functions, expected at least 5", function_count);
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("❌ WASM analysis failed: {}", stderr);
    }
}

fn test_multi_language_context() {
    println!("Creating multi-language test project...");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create Rust file
    let rust_content = r#"
pub fn calculate_hash(input: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    hasher.finish()
}

pub async fn fetch_data_async(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Simulated async operation
    Ok(format!("Data from {}", url))
}

pub struct DataProcessor {
    cache: std::collections::HashMap<String, String>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            cache: std::collections::HashMap::new(),
        }
    }

    pub fn process_data(&mut self, key: &str, data: String) -> String {
        self.cache.insert(key.to_string(), data.clone());
        data.to_uppercase()
    }
}
"#;

    let rust_file = temp_dir.path().join("processor.rs");
    fs::write(&rust_file, rust_content).expect("Failed to write Rust file");

    // Create Python file
    let python_content = r#"
def calculate_fibonacci(n):
    """Calculate fibonacci number using dynamic programming."""
    if n <= 1:
        return n

    a, b = 0, 1
    for _ in range(2, n + 1):
        a, b = b, a + b

    return b

class DataAnalyzer:
    def __init__(self):
        self.data = []

    def add_data_point(self, value):
        """Add a data point to the analyzer."""
        self.data.append(value)

    def calculate_average(self):
        """Calculate the average of all data points."""
        if not self.data:
            return 0
        return sum(self.data) / len(self.data)

    def find_max_value(self):
        """Find the maximum value in the dataset."""
        return max(self.data) if self.data else None

async def process_async_data(data_list):
    """Process data asynchronously."""
    import asyncio
    await asyncio.sleep(0.1)  # Simulated async work
    return [item * 2 for item in data_list]

def validate_email(email):
    """Validate email format using regex."""
    import re
    pattern = r'^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$'
    return bool(re.match(pattern, email))
"#;

    let python_file = temp_dir.path().join("analyzer.py");
    fs::write(&python_file, python_content).expect("Failed to write Python file");

    // Create a simple TypeScript file
    let simple_ts_content = r#"
export function formatCurrency(amount: number, currency: string = 'USD'): string {
    return new Intl.NumberFormat('en-US', {
        style: 'currency',
        currency: currency
    }).format(amount);
}

export const debounce = <T extends (...args: any[]) => void>(
    func: T,
    delay: number
): ((...args: Parameters<T>) => void) => {
    let timeoutId: NodeJS.Timeout;
    return (...args: Parameters<T>) => {
        clearTimeout(timeoutId);
        timeoutId = setTimeout(() => func(...args), delay);
    };
};
"#;

    let simple_ts_file = temp_dir.path().join("utils.ts");
    fs::write(&simple_ts_file, simple_ts_content).expect("Failed to write simple TypeScript file");

    println!("Running: cargo run -- context --project-path {} --format llm-optimized", temp_dir.path().display());

    let output = Command::new("cargo")
        .args(["run", "--", "context", "--project-path", temp_dir.path().to_str().unwrap(), "--format", "llm-optimized"])
        .output()
        .expect("Failed to run cargo command");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("✅ Multi-language analysis successful!");

        if let Some(language_line) = stdout.lines().find(|line| line.starts_with("Project:") && line.contains("(")) {
            println!("   {}", language_line);
        }
        if let Some(files_line) = stdout.lines().find(|line| line.contains("Files:")) {
            println!("   {}", files_line);
        }
        if let Some(functions_line) = stdout.lines().find(|line| line.contains("Functions:")) {
            println!("   {}", functions_line);
        }

        let function_count = extract_function_count(&stdout);
        if function_count >= 10 {
            println!("   🎯 Successfully detected {} functions across multiple languages!", function_count);
            println!("   Expected functions from Rust, Python, and TypeScript files");
        } else {
            println!("   ⚠️  Only detected {} functions, expected at least 10", function_count);
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("❌ Multi-language analysis failed: {}", stderr);
    }
}

fn test_comprehensive_language_test() {
    println!("Testing comprehensive language test directory...");

    let test_dir = Path::new("comprehensive_language_test");
    if !test_dir.exists() {
        println!("⚠️  Comprehensive language test directory not found, skipping...");
        return;
    }

    println!("Running: cargo run -- context --project-path comprehensive_language_test --format llm-optimized");

    let output = Command::new("cargo")
        .args(["run", "--", "context", "--project-path", "comprehensive_language_test", "--format", "llm-optimized"])
        .output()
        .expect("Failed to run cargo command");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("✅ Comprehensive language test analysis successful!");

        if let Some(language_line) = stdout.lines().find(|line| line.starts_with("Project:") && line.contains("(")) {
            println!("   {}", language_line);
        }
        if let Some(files_line) = stdout.lines().find(|line| line.contains("Files:")) {
            println!("   {}", files_line);
        }
        if let Some(functions_line) = stdout.lines().find(|line| line.contains("Functions:")) {
            println!("   {}", functions_line);
        }

        let function_count = extract_function_count(&stdout);
        println!("   🎯 Detected {} functions in comprehensive test suite", function_count);

        // Check if we're detecting the expected high number of functions
        if function_count >= 50 {
            println!("   🚀 Excellent! High function count indicates comprehensive language detection");
        } else if function_count >= 20 {
            println!("   ✅ Good function detection across multiple languages");
        } else {
            println!("   ⚠️  Lower than expected function count - may indicate detection issues");
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("❌ Comprehensive language test analysis failed: {}", stderr);
    }
}

fn extract_function_count(output: &str) -> usize {
    output
        .lines()
        .find(|line| line.contains("Functions:"))
        .and_then(|line| line.split("Functions:").nth(1))
        .and_then(|part| part.trim().parse().ok())
        .unwrap_or(0)
}