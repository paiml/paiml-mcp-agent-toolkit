// Test Artifacts for Pathology Detection
// Following Toyota Way principles for quality assurance

use std::fs;
use std::path::Path;

/// Create test artifacts demonstrating various code pathologies
pub fn create_test_artifacts(base_path: &Path) -> std::io::Result<()> {
    // Create directory structure
    fs::create_dir_all(base_path.join("rust"))?;
    fs::create_dir_all(base_path.join("python"))?;
    fs::create_dir_all(base_path.join("typescript"))?;
    
    // Rust artifacts
    create_rust_artifacts(&base_path.join("rust"))?;
    
    // Python artifacts
    create_python_artifacts(&base_path.join("python"))?;
    
    // TypeScript artifacts
    create_typescript_artifacts(&base_path.join("typescript"))?;
    
    Ok(())
}

fn create_rust_artifacts(path: &Path) -> std::io::Result<()> {
    // 1. High Complexity Example
    fs::write(path.join("high_complexity.rs"), r#"
// High cyclomatic complexity example
fn process_data(input: &[i32], mode: u8, flags: u32) -> Result<Vec<i32>, &'static str> {
    let mut result = Vec::new();
    
    if input.is_empty() {
        return Err("Empty input");
    }
    
    for &value in input {
        if mode == 1 {
            if value > 0 {
                if flags & 0x01 != 0 {
                    result.push(value * 2);
                } else if flags & 0x02 != 0 {
                    result.push(value * 3);
                } else {
                    result.push(value);
                }
            } else if value < 0 {
                if flags & 0x04 != 0 {
                    result.push(-value);
                } else {
                    result.push(value / 2);
                }
            } else {
                result.push(0);
            }
        } else if mode == 2 {
            match value {
                v if v > 100 => {
                    if flags & 0x08 != 0 {
                        result.push(v - 100);
                    } else {
                        result.push(100);
                    }
                }
                v if v > 50 => result.push(v / 2),
                v if v > 0 => result.push(v * 2),
                _ => result.push(0),
            }
        } else if mode == 3 {
            // More nested conditions
            if value % 2 == 0 {
                if value % 3 == 0 {
                    if value % 5 == 0 {
                        result.push(value);
                    } else {
                        result.push(value + 5);
                    }
                } else {
                    result.push(value + 3);
                }
            } else {
                result.push(value + 1);
            }
        } else {
            return Err("Invalid mode");
        }
    }
    
    Ok(result)
}
"#)?;

    // 2. Dead Code Example
    fs::write(path.join("dead_code.rs"), r#"
// Dead code example with unreachable functions
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, i32>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }
    
    pub fn process(&mut self, key: &str, value: i32) {
        self.cache.insert(key.to_string(), value);
    }
    
    // Dead code - never called
    fn unused_helper(&self, key: &str) -> Option<i32> {
        self.cache.get(key).copied()
    }
    
    // Dead code - private and unused
    fn calculate_something(&self, a: i32, b: i32) -> i32 {
        a * b + self.cache.len() as i32
    }
    
    // Dead code - complex unused function
    fn unused_complex_logic(&mut self) {
        let keys: Vec<_> = self.cache.keys().cloned().collect();
        for key in keys {
            if let Some(value) = self.cache.get_mut(&key) {
                *value *= 2;
            }
        }
    }
}

// Dead code - entire module unused
mod unused_module {
    pub fn unused_function() -> String {
        "This is never used".to_string()
    }
    
    pub struct UnusedStruct {
        field: i32,
    }
    
    impl UnusedStruct {
        pub fn new() -> Self {
            Self { field: 0 }
        }
    }
}
"#)?;

    // 3. Duplicate Code Example
    fs::write(path.join("duplicate_code.rs"), r#"
// Duplicate code example
pub struct UserManager {
    users: Vec<User>,
}

pub struct User {
    id: u32,
    name: String,
    email: String,
}

impl UserManager {
    // Duplicate pattern 1
    pub fn find_user_by_id(&self, id: u32) -> Option<&User> {
        for user in &self.users {
            if user.id == id {
                return Some(user);
            }
        }
        None
    }
    
    // Duplicate pattern 2 (same logic, different field)
    pub fn find_user_by_name(&self, name: &str) -> Option<&User> {
        for user in &self.users {
            if user.name == name {
                return Some(user);
            }
        }
        None
    }
    
    // Duplicate pattern 3 (same logic, another field)
    pub fn find_user_by_email(&self, email: &str) -> Option<&User> {
        for user in &self.users {
            if user.email == email {
                return Some(user);
            }
        }
        None
    }
    
    // More duplication
    pub fn count_users_with_name(&self, name: &str) -> usize {
        let mut count = 0;
        for user in &self.users {
            if user.name == name {
                count += 1;
            }
        }
        count
    }
    
    pub fn count_users_with_email_domain(&self, domain: &str) -> usize {
        let mut count = 0;
        for user in &self.users {
            if user.email.ends_with(domain) {
                count += 1;
            }
        }
        count
    }
}
"#)?;

    // 4. Provability Example (with proof annotations)
    fs::write(path.join("provable_code.rs"), r#"
// Example with provable properties
#![allow(dead_code)]

/// A verified safe integer operation that cannot overflow
/// 
/// # Safety Invariants
/// - Input must be within i32::MIN/2 to i32::MAX/2
/// - Result is guaranteed to not overflow
#[inline]
pub fn safe_double(x: i32) -> Option<i32> {
    // Proof: if x is in range [-2^30, 2^30], then 2*x is in range [-2^31, 2^31]
    const SAFE_BOUND: i32 = i32::MAX / 2;
    
    if x >= -SAFE_BOUND && x <= SAFE_BOUND {
        Some(x * 2)
    } else {
        None
    }
}

/// Memory-safe bounded buffer
pub struct BoundedBuffer<const N: usize> {
    data: [u8; N],
    len: usize,
}

impl<const N: usize> BoundedBuffer<N> {
    /// Creates a new empty buffer
    /// 
    /// # Proof of Safety
    /// - Memory is stack-allocated with compile-time size
    /// - len is initialized to 0, maintaining invariant len <= N
    pub const fn new() -> Self {
        Self {
            data: [0; N],
            len: 0,
        }
    }
    
    /// Push a byte to the buffer
    /// 
    /// # Proof of Safety
    /// - Returns Err if buffer is full, preventing overflow
    /// - Maintains invariant: len <= N
    pub fn push(&mut self, byte: u8) -> Result<(), &'static str> {
        if self.len < N {
            self.data[self.len] = byte;
            self.len += 1;
            Ok(())
        } else {
            Err("Buffer full")
        }
    }
    
    /// Get a slice of the valid data
    /// 
    /// # Proof of Safety
    /// - Slice bounds are always valid: 0..self.len where self.len <= N
    pub fn as_slice(&self) -> &[u8] {
        &self.data[..self.len]
    }
}

/// Thread-safe counter with verified atomicity
pub struct VerifiedCounter {
    value: std::sync::atomic::AtomicU64,
}

impl VerifiedCounter {
    /// Increment counter atomically
    /// 
    /// # Proof of Thread Safety
    /// - Uses SeqCst ordering for total order guarantee
    /// - No data races possible due to atomic operations
    pub fn increment(&self) -> u64 {
        self.value.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }
}

// Proof: VerifiedCounter is Send + Sync
unsafe impl Send for VerifiedCounter {}
unsafe impl Sync for VerifiedCounter {}
"#)?;

    Ok(())
}

fn create_python_artifacts(path: &Path) -> std::io::Result<()> {
    // 1. High Complexity Python
    fs::write(path.join("high_complexity.py"), r#"
# High complexity Python example
def process_complex_data(data, config):
    """Process data with high cyclomatic complexity"""
    result = []
    
    if not data:
        return None
    
    for item in data:
        if config.get('mode') == 'standard':
            if item['type'] == 'A':
                if item['value'] > 100:
                    if config.get('transform'):
                        result.append(item['value'] * 2)
                    else:
                        result.append(item['value'])
                elif item['value'] > 50:
                    result.append(item['value'] / 2)
                else:
                    if item.get('special'):
                        result.append(item['value'] * 3)
                    else:
                        result.append(0)
            elif item['type'] == 'B':
                if item['status'] == 'active':
                    if item['priority'] > 5:
                        result.append(item['value'] + 100)
                    else:
                        result.append(item['value'] + 50)
                elif item['status'] == 'pending':
                    result.append(item['value'])
                else:
                    result.append(-1)
            else:
                # Type C handling
                if item.get('category') == 'premium':
                    if item['value'] > 200:
                        result.append(item['value'] * 1.5)
                    else:
                        result.append(item['value'] * 1.2)
                else:
                    result.append(item['value'])
        elif config.get('mode') == 'advanced':
            # More complex logic
            try:
                if item['value'] % 2 == 0:
                    if item['value'] % 3 == 0:
                        if item['value'] % 5 == 0:
                            result.append(item['value'])
                        else:
                            result.append(item['value'] + 5)
                    else:
                        result.append(item['value'] + 3)
                else:
                    result.append(item['value'] + 1)
            except KeyError:
                result.append(0)
        else:
            raise ValueError("Unknown mode")
    
    return result
"#)?;

    // 2. Dead Code Python
    fs::write(path.join("dead_code.py"), r#"
# Dead code example in Python
import math
from typing import List, Optional

class DataAnalyzer:
    def __init__(self):
        self.data = []
        self.cache = {}
    
    def add_data(self, value: float):
        self.data.append(value)
    
    def get_mean(self) -> float:
        if not self.data:
            return 0.0
        return sum(self.data) / len(self.data)
    
    # Dead code - never called
    def _calculate_median(self) -> float:
        if not self.data:
            return 0.0
        sorted_data = sorted(self.data)
        n = len(sorted_data)
        if n % 2 == 0:
            return (sorted_data[n//2-1] + sorted_data[n//2]) / 2
        return sorted_data[n//2]
    
    # Dead code - unused helper
    def _calculate_variance(self) -> float:
        if len(self.data) < 2:
            return 0.0
        mean = self.get_mean()
        return sum((x - mean) ** 2 for x in self.data) / (len(self.data) - 1)
    
    # Dead code - complex unused method
    def _advanced_analysis(self) -> dict:
        results = {
            'mean': self.get_mean(),
            'median': self._calculate_median(),
            'variance': self._calculate_variance(),
            'std_dev': math.sqrt(self._calculate_variance()),
            'range': max(self.data) - min(self.data) if self.data else 0
        }
        return results

# Dead code - unused function
def unused_utility_function(data: List[float]) -> Optional[float]:
    """This function is never called"""
    if not data:
        return None
    return sum(x ** 2 for x in data) / len(data)

# Dead code - entire class unused
class UnusedProcessor:
    def __init__(self, config: dict):
        self.config = config
        self.results = []
    
    def process(self, item):
        # Complex logic that's never executed
        if self.config.get('mode') == 'fast':
            return item * 2
        return item
    
    def get_results(self):
        return self.results
"#)?;

    // 3. Duplicate Code Python
    fs::write(path.join("duplicate_code.py"), r#"
# Duplicate code patterns in Python
class StudentManager:
    def __init__(self):
        self.students = []
    
    # Duplication pattern 1
    def find_student_by_id(self, student_id):
        for student in self.students:
            if student['id'] == student_id:
                return student
        return None
    
    # Duplication pattern 2
    def find_student_by_name(self, name):
        for student in self.students:
            if student['name'] == name:
                return student
        return None
    
    # Duplication pattern 3
    def find_student_by_email(self, email):
        for student in self.students:
            if student['email'] == email:
                return student
        return None
    
    # More duplication
    def get_students_by_grade(self, grade):
        result = []
        for student in self.students:
            if student['grade'] == grade:
                result.append(student)
        return result
    
    def get_students_by_major(self, major):
        result = []
        for student in self.students:
            if student['major'] == major:
                result.append(student)
        return result
    
    # Duplicate validation logic
    def validate_student_data(self, student):
        if not student.get('id'):
            return False, "ID is required"
        if not student.get('name'):
            return False, "Name is required"
        if not student.get('email'):
            return False, "Email is required"
        if '@' not in student.get('email', ''):
            return False, "Invalid email"
        return True, None
    
    def validate_teacher_data(self, teacher):
        if not teacher.get('id'):
            return False, "ID is required"
        if not teacher.get('name'):
            return False, "Name is required"
        if not teacher.get('email'):
            return False, "Email is required"
        if '@' not in teacher.get('email', ''):
            return False, "Invalid email"
        return True, None
"#)?;

    Ok(())
}

fn create_typescript_artifacts(path: &Path) -> std::io::Result<()> {
    // 1. High Complexity TypeScript
    fs::write(path.join("highComplexity.ts"), r#"
// High complexity TypeScript example
interface Config {
    mode: 'basic' | 'advanced' | 'expert';
    features: {
        validation: boolean;
        transformation: boolean;
        caching: boolean;
    };
}

interface DataItem {
    id: string;
    type: 'A' | 'B' | 'C';
    value: number;
    status?: 'active' | 'inactive' | 'pending';
    metadata?: Record<string, any>;
}

export function processComplexData(items: DataItem[], config: Config): number[] {
    const results: number[] = [];
    
    if (!items || items.length === 0) {
        return results;
    }
    
    for (const item of items) {
        if (config.mode === 'basic') {
            if (item.type === 'A') {
                if (item.value > 100) {
                    if (config.features.transformation) {
                        results.push(item.value * 2);
                    } else {
                        results.push(item.value);
                    }
                } else if (item.value > 50) {
                    if (item.status === 'active') {
                        results.push(item.value * 1.5);
                    } else {
                        results.push(item.value);
                    }
                } else {
                    results.push(0);
                }
            } else if (item.type === 'B') {
                switch (item.status) {
                    case 'active':
                        if (item.metadata?.priority > 5) {
                            results.push(item.value + 100);
                        } else {
                            results.push(item.value + 50);
                        }
                        break;
                    case 'pending':
                        if (config.features.validation) {
                            results.push(item.value * 0.8);
                        } else {
                            results.push(item.value);
                        }
                        break;
                    default:
                        results.push(-1);
                }
            } else {
                // Type C
                if (config.features.caching && item.metadata?.cached) {
                    results.push(item.metadata.cachedValue);
                } else {
                    const computed = item.value * Math.random();
                    results.push(Math.floor(computed));
                }
            }
        } else if (config.mode === 'advanced') {
            // More nested complexity
            try {
                if (item.value % 2 === 0) {
                    if (item.value % 3 === 0) {
                        if (item.value % 5 === 0) {
                            results.push(item.value);
                        } else {
                            results.push(item.value + 5);
                        }
                    } else {
                        results.push(item.value + 3);
                    }
                } else {
                    results.push(item.value + 1);
                }
            } catch (error) {
                results.push(0);
            }
        } else {
            // Expert mode with even more complexity
            const factor = item.metadata?.factor || 1;
            if (factor > 1) {
                if (factor > 2) {
                    if (factor > 3) {
                        results.push(item.value * factor);
                    } else {
                        results.push(item.value * (factor - 1));
                    }
                } else {
                    results.push(item.value + factor);
                }
            } else {
                results.push(item.value);
            }
        }
    }
    
    return results;
}
"#)?;

    // 2. Dead Code TypeScript
    fs::write(path.join("deadCode.ts"), r#"
// Dead code example in TypeScript
export class UserService {
    private users: Map<string, User> = new Map();
    
    constructor() {}
    
    addUser(user: User): void {
        this.users.set(user.id, user);
    }
    
    getUser(id: string): User | undefined {
        return this.users.get(id);
    }
    
    // Dead code - never called
    private validateUserData(user: User): boolean {
        return !!(user.id && user.name && user.email);
    }
    
    // Dead code - unused method
    private calculateUserScore(user: User): number {
        let score = 0;
        if (user.verified) score += 10;
        if (user.premium) score += 20;
        if (user.activityLevel > 50) score += 15;
        return score;
    }
    
    // Dead code - complex unused logic
    private optimizeUserData(): void {
        const sortedUsers = Array.from(this.users.values())
            .sort((a, b) => b.activityLevel - a.activityLevel);
        
        for (const user of sortedUsers) {
            if (user.activityLevel < 10) {
                user.status = 'inactive';
            } else if (user.activityLevel < 50) {
                user.status = 'moderate';
            } else {
                user.status = 'active';
            }
        }
    }
}

// Dead code - unused interface
interface UserMetrics {
    loginCount: number;
    lastLogin: Date;
    totalSpent: number;
    referrals: number;
}

// Dead code - unused function
function calculateAverageMetrics(metrics: UserMetrics[]): UserMetrics {
    if (metrics.length === 0) {
        return {
            loginCount: 0,
            lastLogin: new Date(),
            totalSpent: 0,
            referrals: 0
        };
    }
    
    const sum = metrics.reduce((acc, m) => ({
        loginCount: acc.loginCount + m.loginCount,
        lastLogin: acc.lastLogin,
        totalSpent: acc.totalSpent + m.totalSpent,
        referrals: acc.referrals + m.referrals
    }));
    
    return {
        loginCount: sum.loginCount / metrics.length,
        lastLogin: new Date(),
        totalSpent: sum.totalSpent / metrics.length,
        referrals: sum.referrals / metrics.length
    };
}

// Dead code - entire class never instantiated
class UnusedAnalytics {
    private data: any[] = [];
    
    collect(event: any): void {
        this.data.push({
            ...event,
            timestamp: new Date()
        });
    }
    
    analyze(): any {
        return {
            total: this.data.length,
            unique: new Set(this.data.map(d => d.userId)).size
        };
    }
}

interface User {
    id: string;
    name: string;
    email: string;
    verified?: boolean;
    premium?: boolean;
    activityLevel: number;
    status?: string;
}
"#)?;

    // 3. Duplicate Code TypeScript
    fs::write(path.join("duplicateCode.ts"), r#"
// Duplicate code patterns in TypeScript
export class ProductManager {
    private products: Product[] = [];
    
    // Duplication pattern 1
    findProductById(id: string): Product | undefined {
        for (const product of this.products) {
            if (product.id === id) {
                return product;
            }
        }
        return undefined;
    }
    
    // Duplication pattern 2
    findProductByName(name: string): Product | undefined {
        for (const product of this.products) {
            if (product.name === name) {
                return product;
            }
        }
        return undefined;
    }
    
    // Duplication pattern 3
    findProductBySku(sku: string): Product | undefined {
        for (const product of this.products) {
            if (product.sku === sku) {
                return product;
            }
        }
        return undefined;
    }
    
    // More duplication - filtering
    getProductsByCategory(category: string): Product[] {
        const result: Product[] = [];
        for (const product of this.products) {
            if (product.category === category) {
                result.push(product);
            }
        }
        return result;
    }
    
    getProductsByPriceRange(min: number, max: number): Product[] {
        const result: Product[] = [];
        for (const product of this.products) {
            if (product.price >= min && product.price <= max) {
                result.push(product);
            }
        }
        return result;
    }
    
    // Duplicate validation
    validateProductData(product: Product): ValidationResult {
        if (!product.id) {
            return { valid: false, error: 'ID is required' };
        }
        if (!product.name) {
            return { valid: false, error: 'Name is required' };
        }
        if (!product.sku) {
            return { valid: false, error: 'SKU is required' };
        }
        if (product.price < 0) {
            return { valid: false, error: 'Price must be positive' };
        }
        return { valid: true };
    }
    
    validateOrderData(order: Order): ValidationResult {
        if (!order.id) {
            return { valid: false, error: 'ID is required' };
        }
        if (!order.customerId) {
            return { valid: false, error: 'Customer ID is required' };
        }
        if (!order.items || order.items.length === 0) {
            return { valid: false, error: 'Order must have items' };
        }
        if (order.total < 0) {
            return { valid: false, error: 'Total must be positive' };
        }
        return { valid: true };
    }
}

interface Product {
    id: string;
    name: string;
    sku: string;
    category: string;
    price: number;
}

interface Order {
    id: string;
    customerId: string;
    items: string[];
    total: number;
}

interface ValidationResult {
    valid: boolean;
    error?: string;
}
"#)?;

    Ok(())
}