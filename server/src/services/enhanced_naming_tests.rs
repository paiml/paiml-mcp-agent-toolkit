//! Comprehensive TDD Tests for Enhanced Naming in JavaScript/TypeScript/WASM
//!
//! Following extreme TDD approach - these tests define the enhanced naming behaviors
//! we want to implement for better deep context generation.

#[cfg(test)]
mod enhanced_javascript_naming_tests {
    use crate::services::context::AstItem;
    use crate::services::enhanced_typescript_visitor::EnhancedTypeScriptVisitor;
    use std::path::Path;

    #[cfg(feature = "typescript-ast")]
    mod javascript_real_world_tests {
        use super::*;
        use std::sync::Arc;
        use swc_common::{FileName, SourceMap};
        use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, EsSyntax};
        use swc_ecma_ast::Module;

        fn parse_javascript(code: &str) -> Module {
            let source_map = Arc::new(SourceMap::default());
            let source_file = source_map
                .new_source_file(FileName::Custom("test.js".into()).into(), code.to_string());

            let lexer = Lexer::new(
                Syntax::Es(EsSyntax {
                    jsx: true,
                    fn_bind: false,
                    decorators: true,
                    decorators_before_export: true,
                    export_default_from: true,
                    import_attributes: true,
                    auto_accessors: false,
                    explicit_resource_management: false,
                    allow_super_outside_method: false,
                    allow_return_outside_function: false,
                }),
                Default::default(),
                StringInput::from(&*source_file),
                None,
            );

            let mut parser = Parser::new_from(lexer);
            parser.parse_module().expect("Failed to parse JavaScript")
        }

        /// Test: React component names should be extracted with descriptive names
        #[test]
        fn test_react_component_names_extraction() {
            let code = r#"
                // Named function component
                function UserProfile({ userId, onEdit }) {
                    return <div className="profile">User {userId}</div>;
                }

                // Arrow function component
                const ProductCard = ({ product, onAddToCart }) => {
                    return (
                        <div className="card">
                            <h3>{product.name}</h3>
                            <button onClick={onAddToCart}>Add to Cart</button>
                        </div>
                    );
                };

                // Component with hooks
                const ShoppingCart = () => {
                    const [items, setItems] = useState([]);
                    const [isLoading, setIsLoading] = useState(false);

                    return <div>{items.length} items</div>;
                };

                // Class component
                class Dashboard extends React.Component {
                    constructor(props) {
                        super(props);
                        this.state = { data: null };
                    }

                    render() {
                        return <div>Dashboard</div>;
                    }
                }
            "#;

            let module = parse_javascript(code);
            let visitor = EnhancedTypeScriptVisitor::new(Path::new("components.js"));
            let items = visitor.extract_items(&module);

            let function_names: Vec<String> = items
                .iter()
                .filter_map(|item| match item {
                    AstItem::Function { name, .. } => Some(name.clone()),
                    _ => None,
                })
                .collect();

            // DEBUG: Print what we actually extracted
            println!("DEBUG: Extracted function names: {:?}", function_names);

            // EXPECTATION: Should extract descriptive React component names
            assert!(function_names.contains(&"UserProfile".to_string()),
                "Should extract React function component name, got: {:?}", function_names);
            assert!(function_names.contains(&"ProductCard".to_string()),
                "Should extract arrow function component name, got: {:?}", function_names);
            assert!(function_names.contains(&"ShoppingCart".to_string()),
                "Should extract hook-using component name, got: {:?}", function_names);

            // Should also find the class and its methods
            let class_items: Vec<String> = items
                .iter()
                .filter_map(|item| match item {
                    AstItem::Struct { name, .. } => Some(name.clone()),
                    _ => None,
                })
                .collect();

            assert!(class_items.contains(&"Dashboard".to_string()),
                "Should extract React class component name");

            // Constructor and render method should be found
            assert!(function_names.iter().any(|n| n.contains("constructor")),
                "Should find constructor method");
            assert!(function_names.iter().any(|n| n.contains("render")),
                "Should find render method");
        }

        /// Test: Async functions and Promise patterns should be clearly identified
        #[test]
        fn test_async_function_patterns_extraction() {
            let code = r#"
                // Async function declaration
                async function fetchUserData(userId) {
                    const response = await fetch(`/api/users/${userId}`);
                    return response.json();
                }

                // Async arrow function
                const processPayment = async (paymentData) => {
                    try {
                        const result = await paymentService.process(paymentData);
                        return { success: true, transactionId: result.id };
                    } catch (error) {
                        return { success: false, error: error.message };
                    }
                };

                // Promise-based function
                function uploadFile(file) {
                    return new Promise((resolve, reject) => {
                        const formData = new FormData();
                        formData.append('file', file);

                        fetch('/upload', { method: 'POST', body: formData })
                            .then(response => response.json())
                            .then(resolve)
                            .catch(reject);
                    });
                }

                // Async method in class
                class ApiClient {
                    async authenticate(credentials) {
                        const response = await this.post('/auth', credentials);
                        this.token = response.token;
                        return response;
                    }

                    async getData(endpoint) {
                        return this.get(endpoint);
                    }
                }
            "#;

            let module = parse_javascript(code);
            let visitor = EnhancedTypeScriptVisitor::new(Path::new("api.js"));
            let items = visitor.extract_items(&module);

            let async_functions: Vec<String> = items
                .iter()
                .filter_map(|item| match item {
                    AstItem::Function { name, is_async, .. } if *is_async => Some(name.clone()),
                    _ => None,
                })
                .collect();

            // EXPECTATION: Should identify all async functions
            assert!(async_functions.contains(&"fetchUserData".to_string()),
                "Should extract async function declaration name");
            assert!(async_functions.contains(&"processPayment".to_string()),
                "Should extract async arrow function name");
            assert!(async_functions.iter().any(|n| n.contains("authenticate")),
                "Should find async method in class");
            assert!(async_functions.iter().any(|n| n.contains("getData")),
                "Should find async method in class");

            // Should also identify Promise-based function (not async but returns Promise)
            let promise_functions: Vec<String> = items
                .iter()
                .filter_map(|item| match item {
                    AstItem::Function { name, .. } if name.contains("uploadFile") => Some(name.clone()),
                    _ => None,
                })
                .collect();

            assert!(!promise_functions.is_empty(), "Should find Promise-based function");
        }

        /// Test: Higher-order functions and closures should preserve meaningful names
        #[test]
        fn test_higher_order_functions_and_closures() {
            let code = r#"
                // Higher-order function
                function createValidator(rules) {
                    return function validateInput(input) {
                        for (const rule of rules) {
                            if (!rule.test(input)) {
                                return { valid: false, error: rule.message };
                            }
                        }
                        return { valid: true };
                    };
                }

                // Function factory
                const createApiClient = (baseUrl, apiKey) => {
                    const client = {
                        get: async (endpoint) => {
                            const response = await fetch(`${baseUrl}${endpoint}`, {
                                headers: { 'Authorization': `Bearer ${apiKey}` }
                            });
                            return response.json();
                        },

                        post: async (endpoint, data) => {
                            const response = await fetch(`${baseUrl}${endpoint}`, {
                                method: 'POST',
                                headers: {
                                    'Authorization': `Bearer ${apiKey}`,
                                    'Content-Type': 'application/json'
                                },
                                body: JSON.stringify(data)
                            });
                            return response.json();
                        }
                    };

                    return client;
                };

                // Event handler creators
                function createClickHandler(action, data) {
                    return function handleClick(event) {
                        event.preventDefault();
                        action(data);
                    };
                }

                // Callback patterns
                const processItems = (items, processorFn) => {
                    return items.map((item, index) => {
                        const processedItem = processorFn(item, index);
                        return { ...item, ...processedItem };
                    });
                };
            "#;

            let module = parse_javascript(code);
            let visitor = EnhancedTypeScriptVisitor::new(Path::new("functional.js"));
            let items = visitor.extract_items(&module);

            let function_names: Vec<String> = items
                .iter()
                .filter_map(|item| match item {
                    AstItem::Function { name, .. } => Some(name.clone()),
                    _ => None,
                })
                .collect();

            // DEBUG: Print what we actually extracted
            println!("DEBUG: Higher-order functions extracted: {:?}", function_names);

            // EXPECTATION: Should extract meaningful names for higher-order functions
            assert!(function_names.contains(&"createValidator".to_string()),
                "Should extract higher-order function name");
            assert!(function_names.contains(&"createApiClient".to_string()),
                "Should extract function factory name");
            assert!(function_names.contains(&"createClickHandler".to_string()),
                "Should extract event handler creator name");
            assert!(function_names.contains(&"processItems".to_string()),
                "Should extract callback processor name");

            // Should also extract nested function names where possible
            assert!(function_names.iter().any(|n| n.contains("validateInput")),
                "Should extract nested function name from closure");
            assert!(function_names.iter().any(|n| n.contains("handleClick")),
                "Should extract named event handler from closure");

            // Should find object method definitions
            assert!(function_names.iter().any(|n| n.contains("get") || n.contains("post")),
                "Should find object method definitions in factory");
        }

        /// Test: Module exports and imports should be tracked with aliases
        #[test]
        fn test_module_exports_and_imports_tracking() {
            let code = r#"
                // Named exports
                export function calculateTax(amount, rate) {
                    return amount * rate;
                }

                export const formatCurrency = (amount, locale = 'en-US') => {
                    return new Intl.NumberFormat(locale, {
                        style: 'currency',
                        currency: 'USD',
                    }).format(amount);
                };

                // Default export
                export default class PaymentProcessor {
                    constructor(apiKey) {
                        this.apiKey = apiKey;
                    }

                    async processPayment(payment) {
                        // Implementation
                        return { success: true };
                    }
                }

                // Re-exports
                export { UserService } from './services/user';
                export { default as OrderService } from './services/order';
                export * from './utils/validation';

                // Named imports
                import React, { useState, useEffect } from 'react';
                import { debounce } from 'lodash';
                import * as api from './api/client';
                import { validateEmail as emailValidator } from './utils/validation';
            "#;

            let module = parse_javascript(code);
            let visitor = EnhancedTypeScriptVisitor::new(Path::new("payments.js"));
            let items = visitor.extract_items(&module);

            let export_functions: Vec<String> = items
                .iter()
                .filter_map(|item| match item {
                    AstItem::Function { name, .. } => Some(name.clone()),
                    _ => None,
                })
                .collect();

            let import_items: Vec<String> = items
                .iter()
                .filter_map(|item| match item {
                    AstItem::Use { path, .. } => Some(path.clone()),
                    _ => None,
                })
                .collect();

            // EXPECTATION: Should extract exported function names
            assert!(export_functions.contains(&"calculateTax".to_string()),
                "Should extract named export function");
            assert!(export_functions.contains(&"formatCurrency".to_string()),
                "Should extract named export arrow function");

            // Should find the class and its methods
            let class_items: Vec<String> = items
                .iter()
                .filter_map(|item| match item {
                    AstItem::Struct { name, .. } => Some(name.clone()),
                    _ => None,
                })
                .collect();

            assert!(class_items.contains(&"PaymentProcessor".to_string()),
                "Should extract default export class name");

            // Should track imports
            assert!(import_items.iter().any(|path| path.contains("react")),
                "Should track React import");
            assert!(import_items.iter().any(|path| path.contains("lodash")),
                "Should track lodash import");
            assert!(import_items.iter().any(|path| path.contains("./api/client")),
                "Should track local module import");
        }

        /// Test: ES6+ features should be handled properly
        #[test]
        fn test_es6_features_extraction() {
            let code = r#"
                // Destructuring in function parameters
                function processUser({ id, name, email, preferences = {} }) {
                    console.log(`Processing user ${name} (${id})`);
                    return { id, name, email, ...preferences };
                }

                // Template literals and tagged templates
                const createQuery = (table) => (strings, ...values) => {
                    return strings.reduce((query, string, i) => {
                        return query + string + (values[i] || '');
                    }, '');
                };

                // Generator functions
                function* generateIds(prefix = 'id') {
                    let counter = 0;
                    while (true) {
                        yield `${prefix}_${counter++}`;
                    }
                }

                // Async generators
                async function* fetchPages(url) {
                    let page = 1;
                    let hasMore = true;

                    while (hasMore) {
                        const response = await fetch(`${url}?page=${page}`);
                        const data = await response.json();

                        yield data.items;
                        hasMore = data.hasMore;
                        page++;
                    }
                }

                // Class with static methods and private fields
                class DataCache {
                    #cache = new Map();
                    #maxSize = 100;

                    static instance = null;

                    static getInstance() {
                        if (!DataCache.instance) {
                            DataCache.instance = new DataCache();
                        }
                        return DataCache.instance;
                    }

                    get(key) {
                        return this.#cache.get(key);
                    }

                    set(key, value) {
                        if (this.#cache.size >= this.#maxSize) {
                            const firstKey = this.#cache.keys().next().value;
                            this.#cache.delete(firstKey);
                        }
                        this.#cache.set(key, value);
                    }
                }
            "#;

            let module = parse_javascript(code);
            let visitor = EnhancedTypeScriptVisitor::new(Path::new("modern.js"));
            let items = visitor.extract_items(&module);

            let function_names: Vec<String> = items
                .iter()
                .filter_map(|item| match item {
                    AstItem::Function { name, .. } => Some(name.clone()),
                    _ => None,
                })
                .collect();

            // EXPECTATION: Should handle ES6+ features properly
            assert!(function_names.contains(&"processUser".to_string()),
                "Should extract function with destructured parameters");
            assert!(function_names.contains(&"createQuery".to_string()),
                "Should extract function returning tagged template function");
            assert!(function_names.contains(&"generateIds".to_string()),
                "Should extract generator function name");
            assert!(function_names.contains(&"fetchPages".to_string()),
                "Should extract async generator function name");

            // Should find class and its methods
            let class_items: Vec<String> = items
                .iter()
                .filter_map(|item| match item {
                    AstItem::Struct { name, .. } => Some(name.clone()),
                    _ => None,
                })
                .collect();

            assert!(class_items.contains(&"DataCache".to_string()),
                "Should extract class with private fields");

            // Should find static and instance methods
            assert!(function_names.iter().any(|n| n.contains("getInstance")),
                "Should find static method");
            assert!(function_names.iter().any(|n| n.contains("get") && n.contains("DataCache")),
                "Should find instance method with class context");
            assert!(function_names.iter().any(|n| n.contains("set") && n.contains("DataCache")),
                "Should find instance method with class context");
        }
    }

    /// Test: JSDoc comments should be extracted for enhanced context
    #[test]
    fn test_jsdoc_extraction_for_enhanced_context() {
        // NOTE: This test is designed to FAIL initially as JSDoc extraction is not yet implemented
        let _code = r#"
            /**
             * Calculates the total price including tax
             * @param {number} basePrice - The base price before tax
             * @param {number} taxRate - The tax rate as a decimal (e.g., 0.08 for 8%)
             * @param {boolean} includeShipping - Whether to include shipping costs
             * @returns {number} The total price including tax and optional shipping
             * @example
             * const total = calculateTotal(100, 0.08, true);
             * // Returns: 118 (100 + 8% tax + 10 shipping)
             */
            function calculateTotal(basePrice, taxRate, includeShipping = false) {
                let total = basePrice * (1 + taxRate);
                if (includeShipping) {
                    total += 10; // Standard shipping
                }
                return total;
            }

            /**
             * User account management service
             * @class UserService
             * @description Handles all user-related operations including authentication and profile management
             */
            class UserService {
                /**
                 * Authenticates a user with email and password
                 * @async
                 * @param {string} email - User's email address
                 * @param {string} password - User's password
                 * @returns {Promise<{token: string, user: User}>} Authentication result
                 * @throws {AuthenticationError} When credentials are invalid
                 */
                async authenticate(email, password) {
                    // Implementation
                    return { token: 'abc123', user: { email } };
                }
            }
        "#;

        // This test should initially FAIL - we expect JSDoc extraction to be implemented
        // When properly implemented, function items should include JSDoc information
        let _expected_jsdoc_info = vec![
            ("calculateTotal", "Calculates the total price including tax"),
            ("authenticate", "Authenticates a user with email and password"),
        ];

        // TODO: Implement JSDoc parsing in EnhancedTypeScriptVisitor
        // For now, this test serves as a specification for the feature
        assert!(false, "JSDoc extraction not yet implemented - this test defines the requirement");
    }
}

#[cfg(test)]
mod enhanced_typescript_naming_tests {
    use crate::services::context::AstItem;
    use crate::services::enhanced_typescript_visitor::EnhancedTypeScriptVisitor;
    use std::path::Path;

    #[cfg(feature = "typescript-ast")]
    mod typescript_real_world_tests {
        use super::*;
        use std::sync::Arc;
        use swc_common::{FileName, SourceMap};
        use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsSyntax};
        use swc_ecma_ast::Module;

        fn parse_typescript(code: &str) -> Module {
            let source_map = Arc::new(SourceMap::default());
            let source_file = source_map
                .new_source_file(FileName::Custom("test.ts".into()).into(), code.to_string());

            let lexer = Lexer::new(
                Syntax::Typescript(TsSyntax {
                    tsx: false,
                    decorators: true,
                    dts: false,
                    no_early_errors: true,
                    disallow_ambiguous_jsx_like: true,
                }),
                Default::default(),
                StringInput::from(&*source_file),
                None,
            );

            let mut parser = Parser::new_from(lexer);
            parser.parse_module().expect("Failed to parse TypeScript")
        }

        /// Test: TypeScript type information should be preserved in function names
        #[test]
        fn test_typescript_type_information_extraction() {
            let code = r#"
                interface User {
                    id: string;
                    name: string;
                    email: string;
                    roles: UserRole[];
                }

                type UserRole = 'admin' | 'user' | 'guest';

                interface ApiResponse<T> {
                    data: T;
                    status: number;
                    message?: string;
                }

                // Function with typed parameters and return type
                async function fetchUser(userId: string): Promise<ApiResponse<User>> {
                    const response = await fetch(`/api/users/${userId}`);
                    return response.json() as ApiResponse<User>;
                }

                // Generic function
                function createRepository<T extends { id: string }>(
                    entityName: string
                ): Repository<T> {
                    return new Repository<T>(entityName);
                }

                // Function with union types and optional parameters
                function processPayment(
                    amount: number,
                    method: 'credit' | 'debit' | 'paypal',
                    options?: {
                        currency?: 'USD' | 'EUR' | 'GBP';
                        description?: string;
                        metadata?: Record<string, any>;
                    }
                ): Promise<{ success: boolean; transactionId?: string }> {
                    // Implementation
                    return Promise.resolve({ success: true, transactionId: 'txn_123' });
                }

                // Class with typed methods and properties
                class DataService<T> {
                    private cache: Map<string, T> = new Map();

                    constructor(private apiUrl: string) {}

                    async get<K extends keyof T>(id: string, fields?: K[]): Promise<Pick<T, K> | null> {
                        // Implementation
                        return null;
                    }

                    async update(id: string, updates: Partial<T>): Promise<T> {
                        // Implementation
                        return {} as T;
                    }
                }
            "#;

            let module = parse_typescript(code);
            let visitor = EnhancedTypeScriptVisitor::new(Path::new("typed-service.ts"));
            let items = visitor.extract_items(&module);

            let interfaces: Vec<String> = items
                .iter()
                .filter_map(|item| match item {
                    AstItem::Trait { name, .. } => Some(name.clone()),
                    _ => None,
                })
                .collect();

            let functions: Vec<String> = items
                .iter()
                .filter_map(|item| match item {
                    AstItem::Function { name, .. } => Some(name.clone()),
                    _ => None,
                })
                .collect();

            // EXPECTATION: Should extract TypeScript interfaces and types
            assert!(interfaces.contains(&"User".to_string()),
                "Should extract User interface");
            assert!(interfaces.contains(&"ApiResponse".to_string()),
                "Should extract generic ApiResponse interface");

            // EXPECTATION: Should extract functions with type information preserved
            assert!(functions.contains(&"fetchUser".to_string()),
                "Should extract async typed function");
            assert!(functions.contains(&"createRepository".to_string()),
                "Should extract generic function");
            assert!(functions.contains(&"processPayment".to_string()),
                "Should extract function with union types");

            // Should find generic class and its methods
            let classes: Vec<String> = items
                .iter()
                .filter_map(|item| match item {
                    AstItem::Struct { name, .. } => Some(name.clone()),
                    _ => None,
                })
                .collect();

            assert!(classes.contains(&"DataService".to_string()),
                "Should extract generic class");

            // Should find typed methods
            assert!(functions.iter().any(|n| n.contains("get") && n.contains("DataService")),
                "Should find generic method with constraints");
            assert!(functions.iter().any(|n| n.contains("update") && n.contains("DataService")),
                "Should find method with Partial type");
        }

        /// Test: React TypeScript components with props interfaces
        #[test]
        fn test_react_typescript_components_with_props() {
            let code = r#"
                interface UserProfileProps {
                    user: {
                        id: string;
                        name: string;
                        email: string;
                        avatar?: string;
                    };
                    onEdit: (userId: string) => void;
                    onDelete: (userId: string) => Promise<void>;
                    showActions?: boolean;
                }

                interface ProductCardProps {
                    product: Product;
                    onAddToCart: (product: Product, quantity: number) => void;
                    className?: string;
                }

                // Functional component with typed props
                const UserProfile: React.FC<UserProfileProps> = ({
                    user,
                    onEdit,
                    onDelete,
                    showActions = true
                }) => {
                    const handleEdit = () => onEdit(user.id);
                    const handleDelete = async () => await onDelete(user.id);

                    return (
                        <div className="user-profile">
                            <img src={user.avatar || '/default-avatar.png'} alt={user.name} />
                            <h3>{user.name}</h3>
                            <p>{user.email}</p>
                            {showActions && (
                                <div>
                                    <button onClick={handleEdit}>Edit</button>
                                    <button onClick={handleDelete}>Delete</button>
                                </div>
                            )}
                        </div>
                    );
                };

                // Component with hooks and custom hook
                const ProductCard: React.FC<ProductCardProps> = ({ product, onAddToCart, className }) => {
                    const [quantity, setQuantity] = React.useState<number>(1);
                    const [isLoading, setIsLoading] = React.useState<boolean>(false);

                    const handleAddToCart = React.useCallback(async () => {
                        setIsLoading(true);
                        try {
                            await onAddToCart(product, quantity);
                        } finally {
                            setIsLoading(false);
                        }
                    }, [product, quantity, onAddToCart]);

                    return (
                        <div className={`product-card ${className || ''}`}>
                            <h4>{product.name}</h4>
                            <p>${product.price}</p>
                            <input
                                type="number"
                                value={quantity}
                                onChange={(e) => setQuantity(parseInt(e.target.value))}
                                min="1"
                            />
                            <button onClick={handleAddToCart} disabled={isLoading}>
                                {isLoading ? 'Adding...' : 'Add to Cart'}
                            </button>
                        </div>
                    );
                };

                // Higher-order component
                function withAuth<P extends {}>(Component: React.ComponentType<P>) {
                    return function AuthenticatedComponent(props: P) {
                        const user = useAuth();

                        if (!user) {
                            return <div>Please log in</div>;
                        }

                        return <Component {...props} />;
                    };
                }
            "#;

            let module = parse_typescript(code);
            let visitor = EnhancedTypeScriptVisitor::new(Path::new("components.tsx"));
            let items = visitor.extract_items(&module);

            let interfaces: Vec<String> = items
                .iter()
                .filter_map(|item| match item {
                    AstItem::Trait { name, .. } => Some(name.clone()),
                    _ => None,
                })
                .collect();

            let functions: Vec<String> = items
                .iter()
                .filter_map(|item| match item {
                    AstItem::Function { name, .. } => Some(name.clone()),
                    _ => None,
                })
                .collect();

            // EXPECTATION: Should extract React component prop interfaces
            assert!(interfaces.contains(&"UserProfileProps".to_string()),
                "Should extract props interface");
            assert!(interfaces.contains(&"ProductCardProps".to_string()),
                "Should extract props interface");

            // EXPECTATION: Should extract React components with meaningful names
            assert!(functions.contains(&"UserProfile".to_string()),
                "Should extract typed React functional component");
            assert!(functions.contains(&"ProductCard".to_string()),
                "Should extract typed React functional component with hooks");
            assert!(functions.contains(&"withAuth".to_string()),
                "Should extract higher-order component");

            // Should extract nested event handlers with context
            assert!(functions.iter().any(|n| n.contains("handleEdit") || n.contains("UserProfile")),
                "Should extract event handler with component context");
            assert!(functions.iter().any(|n| n.contains("handleDelete") || n.contains("UserProfile")),
                "Should extract async event handler");
            assert!(functions.iter().any(|n| n.contains("handleAddToCart") || n.contains("ProductCard")),
                "Should extract callback handler");
        }

        /// Test: TypeScript decorators and metadata
        #[test]
        fn test_typescript_decorators_and_metadata() {
            let code = r#"
                // Decorator functions
                function Controller(route: string) {
                    return function <T extends { new (...args: any[]): {} }>(constructor: T) {
                        return class extends constructor {
                            route = route;
                        };
                    };
                }

                function Get(path: string) {
                    return function (target: any, propertyKey: string, descriptor: PropertyDescriptor) {
                        descriptor.value.route = path;
                        descriptor.value.method = 'GET';
                    };
                }

                function Post(path: string) {
                    return function (target: any, propertyKey: string, descriptor: PropertyDescriptor) {
                        descriptor.value.route = path;
                        descriptor.value.method = 'POST';
                    };
                }

                // Class with decorators
                @Controller('/api/users')
                class UserController {
                    constructor(private userService: UserService) {}

                    @Get('/')
                    async getAllUsers(): Promise<User[]> {
                        return this.userService.findAll();
                    }

                    @Get('/:id')
                    async getUserById(id: string): Promise<User | null> {
                        return this.userService.findById(id);
                    }

                    @Post('/')
                    async createUser(userData: CreateUserDto): Promise<User> {
                        return this.userService.create(userData);
                    }
                }

                // Service class with dependency injection
                @Injectable()
                class UserService {
                    constructor(
                        @Inject('UserRepository') private userRepo: IUserRepository,
                        @Inject('Logger') private logger: ILogger
                    ) {}

                    async findAll(): Promise<User[]> {
                        this.logger.info('Fetching all users');
                        return this.userRepo.findMany({});
                    }

                    async findById(id: string): Promise<User | null> {
                        this.logger.info(`Fetching user with ID: ${id}`);
                        return this.userRepo.findOne({ id });
                    }

                    async create(userData: CreateUserDto): Promise<User> {
                        this.logger.info('Creating new user');
                        const user = await this.userRepo.create(userData);
                        return user;
                    }
                }
            "#;

            let module = parse_typescript(code);
            let visitor = EnhancedTypeScriptVisitor::new(Path::new("controller.ts"));
            let items = visitor.extract_items(&module);

            let functions: Vec<String> = items
                .iter()
                .filter_map(|item| match item {
                    AstItem::Function { name, .. } => Some(name.clone()),
                    _ => None,
                })
                .collect();

            let classes: Vec<String> = items
                .iter()
                .filter_map(|item| match item {
                    AstItem::Struct { name, .. } => Some(name.clone()),
                    _ => None,
                })
                .collect();

            // EXPECTATION: Should extract decorator functions
            assert!(functions.contains(&"Controller".to_string()),
                "Should extract class decorator function");
            assert!(functions.contains(&"Get".to_string()),
                "Should extract method decorator function");
            assert!(functions.contains(&"Post".to_string()),
                "Should extract method decorator function");

            // EXPECTATION: Should extract decorated classes
            assert!(classes.contains(&"UserController".to_string()),
                "Should extract decorated controller class");
            assert!(classes.contains(&"UserService".to_string()),
                "Should extract decorated service class");

            // Should extract decorated methods with controller context
            assert!(functions.iter().any(|n| n.contains("getAllUsers") && n.contains("UserController")),
                "Should extract decorated controller method");
            assert!(functions.iter().any(|n| n.contains("getUserById") && n.contains("UserController")),
                "Should extract decorated controller method");
            assert!(functions.iter().any(|n| n.contains("createUser") && n.contains("UserController")),
                "Should extract decorated controller method");

            // Should extract service methods
            assert!(functions.iter().any(|n| n.contains("findAll") && n.contains("UserService")),
                "Should extract service method");
            assert!(functions.iter().any(|n| n.contains("create") && n.contains("UserService")),
                "Should extract service method");
        }
    }
}

#[cfg(test)]
mod enhanced_wasm_naming_tests {
    use crate::services::context::AstItem;
    use crate::services::languages::wasm::WasmModuleAnalyzer;
    use std::path::Path;

    /// Test: WASM export names should be extracted instead of generic function_N
    #[test]
    fn test_wasm_export_names_extraction() {
        let wat_code = r#"
(module
  ;; Import external function
  (import "env" "log" (func $log (param i32)))

  ;; Exported functions with meaningful names
  (func $add (export "add") (param $x i32) (param $y i32) (result i32)
    local.get $x
    local.get $y
    i32.add
  )

  (func $multiply (export "multiply") (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.mul
  )

  (func $fibonacci (export "fibonacci") (param $n i32) (result i32)
    (if (i32.lt_s (local.get $n) (i32.const 2))
      (then (local.get $n))
      (else
        (i32.add
          (call $fibonacci (i32.sub (local.get $n) (i32.const 1)))
          (call $fibonacci (i32.sub (local.get $n) (i32.const 2)))
        )
      )
    )
  )

  ;; Memory and table exports
  (memory 1)
  (export "memory" (memory 0))

  (table 10 funcref)
  (export "function_table" (table 0))
)
        "#;

        let analyzer = WasmModuleAnalyzer::new(Path::new("math.wasm"));
        let items = analyzer
            .analyze_wat_text(wat_code)
            .expect("Should parse WAT text with exports");

        let function_names: Vec<String> = items
            .iter()
            .filter_map(|item| match item {
                AstItem::Function { name, .. } => Some(name.clone()),
                _ => None,
            })
            .collect();

        // EXPECTATION: Should extract meaningful export names instead of generic names
        assert!(function_names.iter().any(|name| name.contains("add")),
            "Should extract 'add' function name from export, got: {:?}", function_names);
        assert!(function_names.iter().any(|name| name.contains("multiply")),
            "Should extract 'multiply' function name from export, got: {:?}", function_names);
        assert!(function_names.iter().any(|name| name.contains("fibonacci")),
            "Should extract 'fibonacci' function name from export, got: {:?}", function_names);

        // Should NOT contain generic function_N names when export names are available
        assert!(!function_names.iter().any(|name| name.starts_with("function_") && name.chars().last().unwrap().is_ascii_digit()),
            "Should not use generic function_N names when export names available, got: {:?}", function_names);
    }

    /// Test: WASM import functions should be tracked with module context
    #[test]
    fn test_wasm_import_functions_tracking() {
        let wat_code = r#"
(module
  ;; Various imports from different modules
  (import "env" "console_log" (func $console_log (param i32)))
  (import "env" "memory" (memory 1))
  (import "js" "Math.random" (func $random (result f64)))
  (import "js" "Date.now" (func $now (result f64)))
  (import "wasi_snapshot_preview1" "fd_write" (func $fd_write (param i32 i32 i32 i32) (result i32)))

  ;; Functions that use imports
  (func $log_message (export "log_message") (param $msg_ptr i32)
    local.get $msg_ptr
    call $console_log
  )

  (func $get_random_number (export "get_random_number") (result f64)
    call $random
  )

  (func $write_to_stdout (export "write_to_stdout") (param $ptr i32) (param $len i32) (result i32)
    i32.const 1  ;; stdout fd
    local.get $ptr
    local.get $len
    i32.const 0  ;; nwritten_ptr
    call $fd_write
  )
)
        "#;

        let analyzer = WasmModuleAnalyzer::new(Path::new("imports.wasm"));
        let items = analyzer
            .analyze_wat_text(wat_code)
            .expect("Should parse WAT text with imports");

        let function_names: Vec<String> = items
            .iter()
            .filter_map(|item| match item {
                AstItem::Function { name, .. } => Some(name.clone()),
                _ => None,
            })
            .collect();

        // EXPECTATION: Should extract exported functions that use imports
        assert!(function_names.iter().any(|name| name.contains("log_message")),
            "Should extract function that uses import");
        assert!(function_names.iter().any(|name| name.contains("get_random_number")),
            "Should extract function that calls imported JS function");
        assert!(function_names.iter().any(|name| name.contains("write_to_stdout")),
            "Should extract function that uses WASI import");

        // TODO: Import tracking is not yet implemented - this test defines the requirement
        // When implemented, should also track import statements:
        // assert!(items.iter().any(|item| matches!(item, AstItem::Use { path, .. } if path.contains("env.console_log"))));
    }

    /// Test: Complex WASM modules with nested functions and locals
    #[test]
    fn test_complex_wasm_module_analysis() {
        let wat_code = r#"
(module
  ;; Type definitions
  (type $binary_op (func (param i32 i32) (result i32)))
  (type $unary_op (func (param i32) (result i32)))

  ;; Function table for indirect calls
  (table $functions 4 funcref)
  (elem (i32.const 0) $add $sub $mul $div)

  ;; Basic arithmetic operations
  (func $add (type $binary_op)
    local.get 0
    local.get 1
    i32.add
  )

  (func $sub (type $binary_op)
    local.get 0
    local.get 1
    i32.sub
  )

  (func $mul (type $binary_op)
    local.get 0
    local.get 1
    i32.mul
  )

  (func $div (type $binary_op)
    local.get 0
    local.get 1
    i32.div_s
  )

  ;; Higher-order function using function table
  (func $apply_operation (export "apply_operation") (param $a i32) (param $b i32) (param $op_index i32) (result i32)
    local.get $a
    local.get $b
    local.get $op_index
    call_indirect (type $binary_op)
  )

  ;; Recursive factorial function
  (func $factorial (export "factorial") (param $n i32) (result i32)
    (local $result i32)
    (local $i i32)

    ;; Initialize result to 1
    i32.const 1
    local.set $result

    ;; Initialize counter to 1
    i32.const 1
    local.set $i

    ;; Loop from 1 to n
    (loop $factorial_loop
      ;; Check if i <= n
      local.get $i
      local.get $n
      i32.le_s

      (if
        (then
          ;; result = result * i
          local.get $result
          local.get $i
          i32.mul
          local.set $result

          ;; i = i + 1
          local.get $i
          i32.const 1
          i32.add
          local.set $i

          ;; Continue loop
          br $factorial_loop
        )
      )
    )

    local.get $result
  )

  ;; Array processing function with memory operations
  (func $sum_array (export "sum_array") (param $ptr i32) (param $len i32) (result i32)
    (local $sum i32)
    (local $i i32)
    (local $current_ptr i32)

    ;; Initialize sum to 0
    i32.const 0
    local.set $sum

    ;; Initialize index to 0
    i32.const 0
    local.set $i

    ;; Initialize current pointer
    local.get $ptr
    local.set $current_ptr

    (loop $sum_loop
      ;; Check if i < len
      local.get $i
      local.get $len
      i32.lt_s

      (if
        (then
          ;; Load value from memory and add to sum
          local.get $sum
          local.get $current_ptr
          i32.load
          i32.add
          local.set $sum

          ;; Move to next array element (increment by 4 bytes for i32)
          local.get $current_ptr
          i32.const 4
          i32.add
          local.set $current_ptr

          ;; Increment index
          local.get $i
          i32.const 1
          i32.add
          local.set $i

          ;; Continue loop
          br $sum_loop
        )
      )
    )

    local.get $sum
  )

  ;; Memory allocation
  (memory $mem 1)
  (export "memory" (memory $mem))
)
        "#;

        let analyzer = WasmModuleAnalyzer::new(Path::new("complex.wasm"));
        let items = analyzer
            .analyze_wat_text(wat_code)
            .expect("Should parse complex WAT module");

        let function_names: Vec<String> = items
            .iter()
            .filter_map(|item| match item {
                AstItem::Function { name, .. } => Some(name.clone()),
                _ => None,
            })
            .collect();

        // EXPECTATION: Should extract all function names including internal and exported
        assert!(function_names.iter().any(|name| name.contains("add")),
            "Should extract arithmetic function name");
        assert!(function_names.iter().any(|name| name.contains("apply_operation")),
            "Should extract higher-order function name");
        assert!(function_names.iter().any(|name| name.contains("factorial")),
            "Should extract recursive function name");
        assert!(function_names.iter().any(|name| name.contains("sum_array")),
            "Should extract memory processing function name");

        // Should have reasonable number of functions (not just exports)
        assert!(function_names.len() >= 6,
            "Should extract both internal and exported functions, got {} functions: {:?}",
            function_names.len(), function_names);
    }

    /// Test: WASM binary analysis should extract function information
    #[test]
    fn test_wasm_binary_function_extraction() {
        // This is a minimal valid WASM binary that exports an "add" function
        let wasm_binary: &[u8] = &[
            0x00, 0x61, 0x73, 0x6d, // Magic number
            0x01, 0x00, 0x00, 0x00, // Version
            // Type section
            0x01, 0x07, 0x01, 0x60, 0x02, 0x7f, 0x7f, 0x01, 0x7f,
            // Function section
            0x03, 0x02, 0x01, 0x00,
            // Export section
            0x07, 0x07, 0x01, 0x03, 0x61, 0x64, 0x64, 0x00, 0x00,
            // Code section
            0x0a, 0x09, 0x01, 0x07, 0x00, 0x20, 0x00, 0x20, 0x01, 0x6a, 0x0b,
        ];

        let analyzer = WasmModuleAnalyzer::new(Path::new("add.wasm"));
        let items = analyzer
            .analyze_wasm_binary(wasm_binary)
            .expect("Should parse WASM binary");

        let function_names: Vec<String> = items
            .iter()
            .filter_map(|item| match item {
                AstItem::Function { name, .. } => Some(name.clone()),
                _ => None,
            })
            .collect();

        // EXPECTATION: Should extract function from binary (currently limited)
        assert!(!function_names.is_empty(),
            "Should extract at least one function from WASM binary");

        // Note: This test currently expects the basic implementation
        // Enhanced implementation should extract export names from binary sections
        println!("Extracted functions from WASM binary: {:?}", function_names);
    }

    /// Test: WASM module validation should catch errors but preserve names
    #[test]
    fn test_wasm_validation_with_name_preservation() {
        let invalid_wat_code = r#"
(module
  ;; This has some issues but should still extract function names where possible
  (func $valid_function (export "valid") (param i32) (result i32)
    local.get 0
    i32.const 1
    i32.add
  )

  ;; This function has a naming issue but name should still be extracted
  (func $another-function (export "another")
    ;; Missing return type but has a name
    nop
  )
)
        "#;

        let analyzer = WasmModuleAnalyzer::new(Path::new("mixed.wasm"));
        // Should not panic even with parsing issues
        let result = analyzer.analyze_wat_text(invalid_wat_code);

        match result {
            Ok(items) => {
                let function_names: Vec<String> = items
                    .iter()
                    .filter_map(|item| match item {
                        AstItem::Function { name, .. } => Some(name.clone()),
                        _ => None,
                    })
                    .collect();

                // Should extract names where possible despite validation issues
                assert!(function_names.iter().any(|name| name.contains("valid_function")),
                    "Should extract valid function names even with module issues");
            }
            Err(_) => {
                // Parsing might fail, which is acceptable for malformed WAT
                // The key point is that we don't panic and handle errors gracefully
                println!("WAT parsing failed gracefully as expected for invalid code");
            }
        }
    }
}

#[cfg(test)]
mod enhanced_naming_integration_tests {
    use crate::services::context::AstItem;

    /// Test: Integration test combining JavaScript, TypeScript, and WASM naming
    #[test]
    fn test_multi_language_enhanced_naming_integration() {
        // This test defines the expected behavior when analyzing projects with multiple languages

        let _expected_javascript_names = vec![
            "UserProfile",           // React component
            "ProductCard",          // Arrow function component
            "fetchUserData",        // Async function
            "processPayment",       // Async arrow function
            "createApiClient",      // Factory function
        ];

        let _expected_typescript_names = vec![
            "fetchUser",            // Typed async function
            "createRepository",     // Generic function
            "DataService",         // Generic class
            "UserController",      // Decorated class
            "getAllUsers",         // Decorated method
        ];

        let _expected_wasm_names = vec![
            "add",                 // Export name from WAT
            "multiply",            // Export name from WAT
            "fibonacci",           // Complex function name
            "apply_operation",     // Higher-order function
            "sum_array",          // Memory processing function
        ];

        // This test serves as a specification for the integration
        // When implemented, the enhanced naming system should:
        // 1. Extract meaningful names for all three languages
        // 2. Preserve type information for TypeScript
        // 3. Extract export names for WASM
        // 4. Handle React component patterns in JS/TS
        // 5. Maintain qualified names with module context

        // TODO: Implement the integration once individual language enhancements are complete
        assert!(false, "Integration test not yet implemented - defines requirements");
    }

    /// Test: Deep context markdown output should include enhanced names
    #[test]
    fn test_deep_context_markdown_enhanced_names() {
        // This test defines how enhanced names should appear in deep_context.md output

        let _expected_markdown_patterns = vec![
            "React Component: `UserProfile` (with props: UserProfileProps)",
            "Async Function: `fetchUserData(userId: string): Promise<ApiResponse<User>>`",
            "WASM Export: `fibonacci(n: i32): i32` (recursive function)",
            "Generic Method: `DataService<T>.get<K>(id: string, fields?: K[]): Promise<Pick<T, K>>`",
            "Event Handler: `ProductCard.handleAddToCart` (async callback)",
        ];

        // When enhanced naming is implemented, deep_context.md should show:
        // 1. Component names with their prop interfaces
        // 2. Function signatures with full type information
        // 3. WASM function signatures with parameter types
        // 4. Method names with class context
        // 5. Semantic descriptions (e.g., "recursive", "async callback")

        // TODO: Implement enhanced markdown formatting once naming extraction is complete
        assert!(false, "Deep context markdown enhancement not yet implemented");
    }
}

/// Test utility functions for enhanced naming tests
#[cfg(test)]
mod test_utilities {
    use crate::services::context::AstItem;

    /// Creates a mock AstItem for testing purposes
    pub fn create_mock_function(name: &str, is_async: bool, line: usize) -> AstItem {
        AstItem::Function {
            name: name.to_string(),
            visibility: "public".to_string(),
            is_async,
            line,
        }
    }

    /// Creates a mock class/struct for testing
    pub fn create_mock_class(name: &str, fields_count: usize, line: usize) -> AstItem {
        AstItem::Struct {
            name: name.to_string(),
            visibility: "public".to_string(),
            fields_count,
            derives: vec![],
            line,
        }
    }

    /// Validates that function names follow enhanced naming conventions
    pub fn validate_enhanced_function_names(names: &[String]) -> Vec<String> {
        let mut issues = Vec::new();

        for name in names {
            // Should not be generic placeholders
            if name.starts_with("function_") && name.chars().last().unwrap().is_ascii_digit() {
                issues.push(format!("Generic name found: {}", name));
            }

            // Should not be "anonymous" unless truly anonymous
            if name == "anonymous" {
                issues.push(format!("Anonymous function without context: {}", name));
            }

            // Should include context for methods (ClassName::method)
            if name.contains("::") {
                // Good - has context
            } else if name.len() < 3 {
                issues.push(format!("Name too short, might lack context: {}", name));
            }
        }

        issues
    }

    /// Validates WASM function names are descriptive
    pub fn validate_wasm_function_names(names: &[String], module_name: &str) -> Vec<String> {
        let mut issues = Vec::new();

        for name in names {
            // Should include module context
            if !name.contains(module_name) {
                issues.push(format!("Missing module context in WASM function: {}", name));
            }

            // Should not be just numeric suffixes
            if name.ends_with("_0") || name.ends_with("_1") || name.ends_with("_2") {
                issues.push(format!("Numeric suffix suggests generic naming: {}", name));
            }
        }

        issues
    }
}