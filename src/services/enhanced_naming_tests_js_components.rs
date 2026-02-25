
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
            assert!(
                function_names.contains(&"UserProfile".to_string()),
                "Should extract React function component name, got: {:?}",
                function_names
            );
            assert!(
                function_names.contains(&"ProductCard".to_string()),
                "Should extract arrow function component name, got: {:?}",
                function_names
            );
            assert!(
                function_names.contains(&"ShoppingCart".to_string()),
                "Should extract hook-using component name, got: {:?}",
                function_names
            );

            // Should also find the class and its methods
            let class_items: Vec<String> = items
                .iter()
                .filter_map(|item| match item {
                    AstItem::Struct { name, .. } => Some(name.clone()),
                    _ => None,
                })
                .collect();

            assert!(
                class_items.contains(&"Dashboard".to_string()),
                "Should extract React class component name"
            );

            // Constructor and render method should be found
            assert!(
                function_names.iter().any(|n| n.contains("constructor")),
                "Should find constructor method"
            );
            assert!(
                function_names.iter().any(|n| n.contains("render")),
                "Should find render method"
            );
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
            assert!(
                async_functions.contains(&"fetchUserData".to_string()),
                "Should extract async function declaration name"
            );
            assert!(
                async_functions.contains(&"processPayment".to_string()),
                "Should extract async arrow function name"
            );
            assert!(
                async_functions.iter().any(|n| n.contains("authenticate")),
                "Should find async method in class"
            );
            assert!(
                async_functions.iter().any(|n| n.contains("getData")),
                "Should find async method in class"
            );

            // Should also identify Promise-based function (not async but returns Promise)
            let promise_functions: Vec<String> = items
                .iter()
                .filter_map(|item| match item {
                    AstItem::Function { name, .. } if name.contains("uploadFile") => {
                        Some(name.clone())
                    }
                    _ => None,
                })
                .collect();

            assert!(
                !promise_functions.is_empty(),
                "Should find Promise-based function"
            );
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
            println!(
                "DEBUG: Higher-order functions extracted: {:?}",
                function_names
            );

            // EXPECTATION: Should extract meaningful names for higher-order functions
            assert!(
                function_names.contains(&"createValidator".to_string()),
                "Should extract higher-order function name"
            );
            assert!(
                function_names.contains(&"createApiClient".to_string()),
                "Should extract function factory name"
            );
            assert!(
                function_names.contains(&"createClickHandler".to_string()),
                "Should extract event handler creator name"
            );
            assert!(
                function_names.contains(&"processItems".to_string()),
                "Should extract callback processor name"
            );

            // Should also extract nested function names where possible
            assert!(
                function_names.iter().any(|n| n.contains("validateInput")),
                "Should extract nested function name from closure"
            );
            assert!(
                function_names.iter().any(|n| n.contains("handleClick")),
                "Should extract named event handler from closure"
            );

            // Should find object method definitions
            assert!(
                function_names
                    .iter()
                    .any(|n| n.contains("get") || n.contains("post")),
                "Should find object method definitions in factory"
            );
        }
