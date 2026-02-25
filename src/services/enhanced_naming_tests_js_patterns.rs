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
            assert!(
                export_functions.contains(&"calculateTax".to_string()),
                "Should extract named export function"
            );
            assert!(
                export_functions.contains(&"formatCurrency".to_string()),
                "Should extract named export arrow function"
            );

            // Should find the class and its methods
            let class_items: Vec<String> = items
                .iter()
                .filter_map(|item| match item {
                    AstItem::Struct { name, .. } => Some(name.clone()),
                    _ => None,
                })
                .collect();

            assert!(
                class_items.contains(&"PaymentProcessor".to_string()),
                "Should extract default export class name"
            );

            // Should track imports
            assert!(
                import_items.iter().any(|path| path.contains("react")),
                "Should track React import"
            );
            assert!(
                import_items.iter().any(|path| path.contains("lodash")),
                "Should track lodash import"
            );
            assert!(
                import_items
                    .iter()
                    .any(|path| path.contains("./api/client")),
                "Should track local module import"
            );
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
            assert!(
                function_names.contains(&"processUser".to_string()),
                "Should extract function with destructured parameters"
            );
            assert!(
                function_names.contains(&"createQuery".to_string()),
                "Should extract function returning tagged template function"
            );
            assert!(
                function_names.contains(&"generateIds".to_string()),
                "Should extract generator function name"
            );
            assert!(
                function_names.contains(&"fetchPages".to_string()),
                "Should extract async generator function name"
            );

            // Should find class and its methods
            let class_items: Vec<String> = items
                .iter()
                .filter_map(|item| match item {
                    AstItem::Struct { name, .. } => Some(name.clone()),
                    _ => None,
                })
                .collect();

            assert!(
                class_items.contains(&"DataCache".to_string()),
                "Should extract class with private fields"
            );

            // Should find static and instance methods
            assert!(
                function_names.iter().any(|n| n.contains("getInstance")),
                "Should find static method"
            );
            assert!(
                function_names
                    .iter()
                    .any(|n| n.contains("get") && n.contains("DataCache")),
                "Should find instance method with class context"
            );
            assert!(
                function_names
                    .iter()
                    .any(|n| n.contains("set") && n.contains("DataCache")),
                "Should find instance method with class context"
            );
        }
